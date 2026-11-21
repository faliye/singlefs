//! 本包真卡测试目标共用：经环境变量 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（次序照交来的次序），对上本机的 Vulkan 独显，
//! 与给每张参与的卡的显存上限。
//! 卡归 GPU 服务：统一任务服务只调配 CPU，任务不拿卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）。交来的卡在环境变量
//! `GPU_CARDS_BY_PRIORITY`（测试源头几行写的 `// task-server-gpu-cards: <张数>` 是要几张的声明，`task-server/README.md`「源文件头的声明」），
//! 每项 `<HOSTS 里的地址>:<nvidia-smi 序号>:<额度>`，空白隔开；没交而有领卡通道的到判段才按需领（[`cards_leased_on_demand_unless_handed_over`]）：真卡用例只点名交来的这几张（挂载参数 `background_gpu_devices`），不再点本机全部独显——
//! 同一台上别的卡可能分给了别的任务。这一台是 `HOSTS` 里哪一项只由多机配置判法认（`scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses`
//! 打的 `THIS_HOST_ADDRESS`，全仓只那一份判法）；`nvidia-smi` 序号对到 wgpu 的适配器按 PCI 总线地址对（去掉域、小写，
//! 同验证包 `gpu_unit_checks` 文件头「按 PCI 总线号对上 wgpu 的适配器」的对法；本包测试不依赖验证包，各写各的）。
//! 变量没设、或这一台一张都没交：打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」；交了卡却对不上本机任何适配器、或交来的比这一份要的少：判红。
//! 七·3 验收「真卡用例每张参与的卡显存上限给 1 GiB（测小上限切片的几条照用更小的）」：每张参与的卡各给
//! [`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`]；专门测小上限切片的那几条照用各自更小的值。
//! 解析与对卡是纯函数（单测在 `tests/real_gpu_tests_open_only_the_handed_over_gpus.rs`，不挂特性、不要卡），读环境、起判法、起 `nvidia-smi`、
//! 枚举适配器的那一层薄壳在 [`handed_over_gpus_or_report_not_run`]。
//! 挂载之后这个进程只许在交来的那几张卡上有设备（D26（后台整理与放置回收） 已定项 13 第 2 条「挂载时只开参与的卡」）：[`assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run`]
//! 照服务判跑错卡的同一个读法（`task-server/task_server.py` 的 `compute_processes_now`：`nvidia-smi --query-compute-apps` 列出的进程号与卡的 UUID，
//! UUID 经 `--query-gpu=index,uuid` 对到序号）认这个进程在哪几张卡上，上了别的卡判红；读不成打 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
//! 交回 [`GpusThisProcessIsOn::NotReadSoThisRunIsNotRun`] 让调用方看得见（对拍用例照它交回没跑成、不入库），不当绿。
//! 一个进程只建一个 Vulkan 实例（[`VULKAN_INSTANCE_OF_THIS_TEST_PROCESS`]）：服务交了这一台的卡时，进程里头一次取卡的那条用例建它，
//! 之后各条用例取卡（[`handed_over_gpus_or_report_not_run`]）都连同交来的卡交回它的克隆（指同一个实例），对卡、开卡、再枚举都用它；
//! 服务没交卡时不建（不碰 GPU）。建它的是哪一条用例只看哪一条先取卡，单跑哪一条都由那一条建。
//! 取卡打的那一行（「本次未跑」的记号行与交了卡的那一行）拼法在 [`line_reporting_the_gpus_taken`]；核这个进程在哪几张卡上时读不成、读到空打的
//! 「本次未跑」记号行同样经 [`not_run_line_of_the_test`] 拼，结局与那一行由不打行的 [`gpus_this_process_is_on_and_the_line_to_print`] 定。
//! 这几行的字、读不成与读到空时交回的结局、取卡时先读 `nvidia-smi` 后取实例的先后，
//! 不要卡的单测在 `tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs`。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use singlefs_background_gpu::adapter_inventory::{
    gpu_device_identifier_of_the_pci_bus_address,
    identifiers_of_the_discrete_gpus_in_the_enumeration, AdapterDeviceKind, BackgroundGpuSelection,
    EnumeratedAdapterDescription,
};
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, BackgroundGpu, VulkanInstanceOfThisProcess,
};
use singlefs_core::background_compute::{GpuDeviceIdentifier, GpuMemoryInBytes};

/// 真卡用例给每张参与的卡的显存上限：1 GiB。
pub const MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU: GpuMemoryInBytes = GpuMemoryInBytes(1 << 30);

/// 交给这个进程的卡所在的环境变量（卡归 GPU 服务，统一任务服务只调配 CPU：`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）。
pub const VARIABLE_OF_THE_HANDED_OVER_GPUS: &str = "GPU_CARDS_BY_PRIORITY";

/// 服务读张数声明只读源文件头这么多行（`task-server/task_server_lib.py` 的 `DECLARED_HEADER_LINES`）。
pub const LINES_OF_THE_SOURCE_HEADER_THE_TASK_SERVER_READS: usize = 60;

/// 测试源头声明要同一台几张卡的那一行的开头（`task-server/task_server_lib.py` 的 `DECLARED_SOURCE_GPU_CARDS_MARK`）。
pub const GPU_CARD_COUNT_DECLARATION_MARK: &str = "// task-server-gpu-cards:";

/// 一张卡在 `nvidia-smi` 里的序号（`GPU_CARDS_BY_PRIORITY` 每项中间那一段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NvidiaSmiIndex(pub u32);

/// `GPU_CARDS_BY_PRIORITY` 的一项：在 `HOSTS` 里哪个地址那一台、`nvidia-smi` 序号。额度这里不用（显存上限照七·3 验收另给）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuHandedOverInTheVariable {
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
pub fn gpus_handed_over_in_the_variable(
    handover_text: &str,
) -> Result<Vec<GpuHandedOverInTheVariable>, GpuHandoverUnreadable> {
    let mut handed_over: Vec<GpuHandedOverInTheVariable> = Vec::new();
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
        let gpu = GpuHandedOverInTheVariable {
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
/// 值读不成（[`gpus_handed_over_in_the_variable`] 的三种）；取不到这一台的地址（`this_host_address` 交回的原因）。
pub fn gpus_handed_to_this_host(
    handover_variable_value: Option<&str>,
    this_host_address: impl FnOnce() -> Result<String, String>,
) -> Result<GpusHandedToThisHost, String> {
    let handover_text = match handover_variable_value {
        Some(handover_text) if !handover_text.trim().is_empty() => handover_text,
        Some(_) | None => {
            return Ok(GpusHandedToThisHost::NotRun {
                reason: format!("没经 {VARIABLE_OF_THE_HANDED_OVER_GPUS} 交卡（没设或是空的）"),
            })
        }
    };
    let handed_over = gpus_handed_over_in_the_variable(handover_text).map_err(|unreadable| {
        format!("{VARIABLE_OF_THE_HANDED_OVER_GPUS}=「{handover_text}」读不成：{unreadable:?}")
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
                "交来的卡（{VARIABLE_OF_THE_HANDED_OVER_GPUS}=「{handover_text}」）里没有这一台（HOSTS 里的 {this_host}）的"
            ),
        });
    }
    Ok(GpusHandedToThisHost::Handed(indices_of_this_host))
}

/// bash `printf %q` 打出来的一个地址还原成原样：判法只打地址（十六进制数字、点、冒号）与隔开它们的空格，`%q` 只给空格前面加反斜杠，空串打成 `''`；
/// 别的引号写法判法不会打给这几行，认不出就报错，不猜（同 验证包 `gpu_card_configuration` 的同名读法）。
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
/// （验证包 `gpu_memory_ledger` 里有同名的一份；产品 GPU 包的测试不依赖验证包，各写各的）。
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

/// 读这个进程在哪几张卡上要的 `nvidia-smi` 参数：每个在卡上有计算上下文的进程一行「卡的 UUID, 进程号」（服务判跑错卡读的同一份）。
pub const NVIDIA_SMI_COMPUTE_PROCESS_ARGUMENTS: [&str; 2] = [
    "--query-compute-apps=gpu_uuid,pid",
    "--format=csv,noheader,nounits",
];

/// 卡的 UUID 对到 `nvidia-smi` 序号要的参数：每张卡一行「序号, UUID」。
pub const NVIDIA_SMI_INDEX_AND_UUID_ARGUMENTS: [&str; 2] =
    ["--query-gpu=index,uuid", "--format=csv,noheader"];

/// 进程号 `process_identifier` 在哪几张卡上有计算上下文（`nvidia-smi` 序号）：`compute_process_report` 是 [`NVIDIA_SMI_COMPUTE_PROCESS_ARGUMENTS`] 的输出，
/// `index_and_uuid_report` 是 [`NVIDIA_SMI_INDEX_AND_UUID_ARGUMENTS`] 的输出。别的进程的行不看。
///
/// # Errors
/// 一行切不出两段、序号或进程号不是数、同一个 UUID 报了两个序号、这个进程所在的卡的 UUID 对不上序号。
pub fn nvidia_smi_indices_the_process_is_on(
    compute_process_report: &str,
    index_and_uuid_report: &str,
    process_identifier: u32,
) -> Result<BTreeSet<NvidiaSmiIndex>, String> {
    let mut indices_by_uuid: BTreeMap<String, NvidiaSmiIndex> = BTreeMap::new();
    for line in index_and_uuid_report
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let (index_text, uuid) = line
            .split_once(',')
            .ok_or_else(|| format!("nvidia-smi 的一行「{line}」切不出序号与 UUID"))?;
        let nvidia_smi_index = NvidiaSmiIndex(
            index_text
                .trim()
                .parse()
                .map_err(|_| format!("nvidia-smi 的一行「{line}」序号不是数"))?,
        );
        if indices_by_uuid
            .insert(uuid.trim().to_string(), nvidia_smi_index)
            .is_some()
        {
            return Err(format!("nvidia-smi 把 UUID {} 报了两次", uuid.trim()));
        }
    }
    let mut indices_the_process_is_on = BTreeSet::new();
    for line in compute_process_report
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let (uuid, process_text) = line
            .split_once(',')
            .ok_or_else(|| format!("nvidia-smi 的一行「{line}」切不出 UUID 与进程号"))?;
        let process_on_the_gpu: u32 = process_text
            .trim()
            .parse()
            .map_err(|_| format!("nvidia-smi 的一行「{line}」进程号不是数"))?;
        if process_on_the_gpu != process_identifier {
            continue;
        }
        let nvidia_smi_index = indices_by_uuid.get(uuid.trim()).ok_or_else(|| {
            format!(
                "这个进程所在的卡 UUID {} 对不上 nvidia-smi 报的序号",
                uuid.trim()
            )
        })?;
        indices_the_process_is_on.insert(*nvidia_smi_index);
    }
    Ok(indices_the_process_is_on)
}

/// `indices_the_process_is_on` 里不是交来的卡的那几张，按序号排；都是交来的是空。
#[must_use]
pub fn nvidia_smi_indices_not_handed_over(
    indices_the_process_is_on: &BTreeSet<NvidiaSmiIndex>,
    handed_over_gpus: &[HandedOverGpu],
) -> Vec<NvidiaSmiIndex> {
    indices_the_process_is_on
        .iter()
        .filter(|nvidia_smi_index| {
            !handed_over_gpus
                .iter()
                .any(|handed_over| handed_over.nvidia_smi_index == **nvidia_smi_index)
        })
        .copied()
        .collect()
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

/// 服务交来、对上了本机 Vulkan 独显的卡（照交来的次序），连同对卡时枚举用的那一份 Vulkan 实例（这个进程的那一份的克隆，指同一个实例）：
/// 用例里开卡、再枚举都用它，不另建。`Instance` 只在不要卡的单测里换成数建了几次的替身。
pub struct HandedOverGpusSeenThroughTheInstance<Instance = VulkanInstanceOfThisProcess> {
    pub vulkan_instance: Instance,
    pub gpus: Vec<HandedOverGpu>,
}

/// 照 `selection` 在 `vulkan_instance` 上开卡。卡是服务交来、对上了本机适配器的，开不了判红。
///
/// # Panics
/// 开卡拒了。
#[must_use]
pub fn opened_background_gpu(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    selection: &BackgroundGpuSelection,
) -> BackgroundGpu {
    open_background_gpu(selection, vulkan_instance).unwrap_or_else(|refused| {
        panic!("服务交来、对上了本机适配器的卡开不了后台 GPU：{refused:?}（点名的 {selection:?}）")
    })
}

/// 本机这一次在 `vulkan_instance` 上枚举到的适配器（与产品开卡用同一个实例），在 `wait_limit` 之内等。
///
/// # Panics
/// 等枚举超过 `wait_limit`。
fn adapter_descriptions_on_this_host(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    wait_limit: Duration,
) -> Vec<EnumeratedAdapterDescription> {
    vulkan_instance
        .vulkan_adapters_enumerated_within(wait_limit)
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

/// 此刻 `vulkan_instance` 上看得到的本机独立显卡，按 wgpu 枚举的次序（照产品那一份判法数：集显、虚拟显卡、软件 Vulkan、认不出类型的与没有总线地址的不算，
/// `adapter_inventory::identifiers_of_the_discrete_gpus_in_the_enumeration`）。只给「Vulkan 驱动全藏起来时一张都看不到」那一条数卡用，开卡点名交来的卡。
///
/// # Panics
/// 等枚举超过 `wait_limit`。
#[must_use]
pub fn discrete_gpus_on_this_host(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    wait_limit: Duration,
) -> Vec<GpuDeviceIdentifier> {
    identifiers_of_the_discrete_gpus_in_the_enumeration(&adapter_descriptions_on_this_host(
        vulkan_instance,
        wait_limit,
    ))
}

/// 此刻本机 Vulkan 独显各自的总线地址（wgpu 报的原样），哪些算独显照产品那一份判法（[`discrete_gpus_on_this_host`]）。
fn pci_bus_addresses_of_the_discrete_gpus_on_this_host(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    wait_limit: Duration,
) -> Vec<String> {
    let descriptions = adapter_descriptions_on_this_host(vulkan_instance, wait_limit);
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
/// `SINGLEFS_MULTI_HOST_CONFIG`、没设照主工作树找多机配置），读它打的这一台的地址（[`this_host_address_emitted_by_the_configuration_check`]）。
/// 取卡认这一台与对拍模块临时 prune-server 绑这一台的地址，都从这里取。
///
/// # Errors
/// 判法起不了、退非 0（认不出这一台）、没打或打空了 `THIS_HOST_ADDRESS`。
pub fn this_host_address_by_the_configuration_check() -> Result<String, String> {
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

/// 跑一次 `nvidia-smi`，交回标准输出。
fn output_of_nvidia_smi(arguments: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("nvidia-smi 起不来：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nvidia-smi {arguments:?} 退了 {}：{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 核这个进程此刻在哪几张卡上的结局（[`assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run`]）。上了交来的卡之外的卡不交回，判红。
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpusThisProcessIsOn {
    /// 读成了，这个进程只在交来的卡上（打过 `GPUS_THIS_PROCESS_IS_ON` 那一行）。
    OnlyTheHandedOver,
    /// 读不成、或读到这个进程一张卡都不在（打过 `NOT_RUN_NO_GPU_ADAPTER` 那一行）：这一条整条记本次未跑，调用方不当绿、不把这一趟的结论入库。
    NotReadSoThisRunIsNotRun,
}

/// 挂载之后、卸载之前调：这个进程此刻在哪几张卡上有设备（[`nvidia_smi_indices_the_process_is_on`]，读法同服务判跑错卡），判法在
/// [`gpus_this_process_is_on_and_the_line_to_print`]，打行在 [`gpus_this_process_is_on_judged_against_the_handed_over`]。
///
/// # Panics
/// 这个进程上了交来的卡之外的卡。
pub fn assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run(
    test_name: &str,
    handed_over_gpus: &[HandedOverGpu],
) -> GpusThisProcessIsOn {
    let indices_the_process_is_on = output_of_nvidia_smi(&NVIDIA_SMI_COMPUTE_PROCESS_ARGUMENTS)
        .and_then(|compute_processes| {
            let index_and_uuid = output_of_nvidia_smi(&NVIDIA_SMI_INDEX_AND_UUID_ARGUMENTS)?;
            nvidia_smi_indices_the_process_is_on(
                &compute_processes,
                &index_and_uuid,
                std::process::id(),
            )
        });
    gpus_this_process_is_on_judged_against_the_handed_over(
        test_name,
        indices_the_process_is_on,
        handed_over_gpus,
    )
}

/// 核这个进程在哪几张卡上的结局与要打的那一行（[`gpus_this_process_is_on_and_the_line_to_print`] 交回，还没打）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpusThisProcessIsOnAndTheLineToPrint {
    pub outcome: GpusThisProcessIsOn,
    /// 结局是 [`GpusThisProcessIsOn::NotReadSoThisRunIsNotRun`] 时是 `NOT_RUN_NO_GPU_ADAPTER` 那一行，
    /// 是 [`GpusThisProcessIsOn::OnlyTheHandedOver`] 时是 `GPUS_THIS_PROCESS_IS_ON` 那一行。
    pub line_to_print: String,
}

/// 读到的这个进程所在的卡（`indices_the_process_is_on`，读不成是 `Err`）与交来的卡比，定结局与要打的那一行，自己不打（打在
/// [`gpus_this_process_is_on_judged_against_the_handed_over`]）：读不成、或读到一张卡都不在（此刻交来的卡上开着设备，读到空就是这个读法看不见它的设备）
/// 是 [`GpusThisProcessIsOn::NotReadSoThisRunIsNotRun`] 与 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`（拼法在
/// [`not_run_line_when_the_gpus_of_the_process_are_not_read`] 与 [`not_run_line_when_the_process_is_seen_on_no_gpu`]，进程号是 `process_identifier`）；
/// 读到了、都是交来的卡，是 [`GpusThisProcessIsOn::OnlyTheHandedOver`] 与 `GPUS_THIS_PROCESS_IS_ON test=<用例名> nvidia_smi_indices=<序号，逗号隔开>`。
/// 不打行是为了让不要卡的单测直接核交回的结局与那一行：测试自己打出 `NOT_RUN_NO_GPU_ADAPTER` 行，变异判法就把那条测试记成本次未跑。
///
/// # Panics
/// 这个进程上了交来的卡之外的卡（`GPUS_THIS_PROCESS_IS_ON` 那一行写在 panic 的话里）。
pub fn gpus_this_process_is_on_and_the_line_to_print(
    test_name: &str,
    indices_the_process_is_on: Result<BTreeSet<NvidiaSmiIndex>, String>,
    handed_over_gpus: &[HandedOverGpu],
    process_identifier: u32,
) -> GpusThisProcessIsOnAndTheLineToPrint {
    let indices_the_process_is_on = match indices_the_process_is_on {
        Ok(indices_the_process_is_on) if indices_the_process_is_on.is_empty() => {
            return GpusThisProcessIsOnAndTheLineToPrint {
                outcome: GpusThisProcessIsOn::NotReadSoThisRunIsNotRun,
                line_to_print: not_run_line_when_the_process_is_seen_on_no_gpu(
                    test_name,
                    process_identifier,
                ),
            };
        }
        Ok(indices_the_process_is_on) => indices_the_process_is_on,
        Err(reason) => {
            return GpusThisProcessIsOnAndTheLineToPrint {
                outcome: GpusThisProcessIsOn::NotReadSoThisRunIsNotRun,
                line_to_print: not_run_line_when_the_gpus_of_the_process_are_not_read(
                    test_name, &reason,
                ),
            };
        }
    };
    let gpus_line = format!(
        "GPUS_THIS_PROCESS_IS_ON test={test_name} nvidia_smi_indices={}",
        indices_the_process_is_on
            .iter()
            .map(|nvidia_smi_index| nvidia_smi_index.0.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let not_handed_over =
        nvidia_smi_indices_not_handed_over(&indices_the_process_is_on, handed_over_gpus);
    assert!(
        not_handed_over.is_empty(),
        "这个进程上了服务没交给它的卡（nvidia-smi 序号 {not_handed_over:?}；交来的是 {handed_over_gpus:?}；读到的：{gpus_line}）"
    );
    GpusThisProcessIsOnAndTheLineToPrint {
        outcome: GpusThisProcessIsOn::OnlyTheHandedOver,
        line_to_print: gpus_line,
    }
}

/// [`gpus_this_process_is_on_and_the_line_to_print`]（进程号取这个进程的）定的那一行打到标准输出，交回它定的结局。
///
/// # Panics
/// 这个进程上了交来的卡之外的卡。
pub fn gpus_this_process_is_on_judged_against_the_handed_over(
    test_name: &str,
    indices_the_process_is_on: Result<BTreeSet<NvidiaSmiIndex>, String>,
    handed_over_gpus: &[HandedOverGpu],
) -> GpusThisProcessIsOn {
    let judged = gpus_this_process_is_on_and_the_line_to_print(
        test_name,
        indices_the_process_is_on,
        handed_over_gpus,
        std::process::id(),
    );
    println!("{}", judged.line_to_print);
    judged.outcome
}

/// 这个进程按需领到的卡（与 `GPU_CARDS_BY_PRIORITY` 同格式）：环境变量没给卡、给了领卡通道时，到判段要开卡的那一刻才领
/// （[`cards_leased_on_demand_unless_handed_over`]，领到记在这里）；环境变量给了卡的照环境变量，不看这里。
static CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS: std::sync::OnceLock<String> =
    std::sync::OnceLock::new();

/// 记下这个进程按需领到的卡：[`handed_over_gpus_or_report_not_run`] 在环境变量没给卡时照它。进程里只记一回，之后再记的不改。
pub fn note_the_cards_leased_on_demand_for_this_process(cards: &str) {
    CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS.get_or_init(|| cards.to_string());
}

// ---- 按需领卡 ----
// 卡归 GPU 批处理验证服务（`gpu-verification-server/`），它照多机配置开卡、一直持着；统一任务服务只调配 CPU，任务不拿卡
// （`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）。崩溃判段之外用卡的测试交 GPU 服务代跑的接口待定，在那之前这里照
// 领卡客户端（GPU 服务的控制命令 `gpu-verification-server/control lease take`）向这一台的 GPU 服务领、进程退出时还，与验证包领卡同一个协议、各写各的。

/// 领卡通道的三个环境变量：统一任务服务交给每次尝试的尝试标签、任务号、第几次尝试。三个都在、都不空才算有通道，原样交给领卡客户端。
pub const CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES: [&str; 3] = [
    "TASK_SERVER_ATTEMPT_LABEL",
    "TASK_SERVER_TASK",
    "TASK_SERVER_ATTEMPT",
];
/// 提交方那一份仓的根（统一任务服务交给每次尝试的）：领卡客户端取它下面的 [`CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY`]。
pub const SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE: &str = "TASK_SERVER_SOURCE_REPOSITORY";
/// 领卡客户端在仓里的路径（从仓根起）：GPU 服务的控制命令，领是 `lease take`，还是 `lease return <卡串>`。
pub const CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY: &str = "gpu-verification-server/control";
/// 起领卡客户端的解释器：客户端是 python 脚本，照 `python3 <客户端> lease …` 起，不靠它的执行位。
const CARD_LEASE_CLIENT_INTERPRETER: &str = "python3";
/// 领卡客户端认领的一方的那个环境变量：`<领的进程号>:<它的启动刻>`。还卡那一条在领的进程退出之后才起，客户端只认这个变量。
const CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE: &str = "GPU_CARD_LEASE_BORROWER";
/// 多机配置在哪的那个环境变量：设了照原样交给领卡客户端，没设从客户端的环境里拿掉。
const MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_MULTI_HOST_CONFIG";
/// `/proc/<进程号>/stat` 里启动刻是第几项（从 1 数，`man 5 proc`）。
const START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE: usize = 22;
/// `/proc/<进程号>/stat` 里命令名（带括号、可以有空格与右括号）之后的头一项是第几项。
const FIRST_FIELD_NUMBER_AFTER_THE_COMMAND_NAME_IN_THE_PROCESS_STATUS_LINE: usize = 3;
/// 还卡子进程跑的 shell：标准输入读到收尾（领的进程退出），再起 `<解释器> <客户端> lease return <卡串>`（`$0` 解释器、`$1` 客户端、`$2` 卡串）。
const RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS: &str =
    r#"while read -r line_never_written; do :; done; exec "$0" "$1" lease return "$2""#;

/// `/proc/<进程号>/stat` 那一行里的启动刻：命令名带括号，里面可以有空格与右括号，从最后一个右括号之后起数。
///
/// # Errors
/// 没有右括号；右括号之后不够项数；那一项不是数。
pub fn start_time_since_boot_in_clock_ticks_from_the_process_status_line(
    process_status_line: &str,
) -> Result<u64, String> {
    let (_, fields_after_the_command_name) = process_status_line
        .rsplit_once(')')
        .ok_or_else(|| format!("进程状态文件里没有命令名的右括号：「{process_status_line}」"))?;
    let start_time = fields_after_the_command_name
        .split_whitespace()
        .nth(
            START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE
                - FIRST_FIELD_NUMBER_AFTER_THE_COMMAND_NAME_IN_THE_PROCESS_STATUS_LINE,
        )
        .ok_or_else(|| {
            format!(
                "进程状态文件里不够 {START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE} 项：「{process_status_line}」"
            )
        })?;
    start_time.parse().map_err(|_| {
        format!(
            "进程状态文件第 {START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE} 项（启动刻）不是数：「{start_time}」"
        )
    })
}

/// 这个进程按需领卡那一回的结局（领到的卡串，连同等这个进程退出就还卡的子进程）：进程里只领一回，被拒的那一句记下来、之后每次交同一句。
static CARD_LEASE_OF_THIS_PROCESS: std::sync::OnceLock<
    Result<(String, std::process::Child), String>,
> = std::sync::OnceLock::new();

/// 给起领卡客户端的命令设环境：通道的三个变量、领的一方，多机配置路径设了的照设、没设的拿掉。
fn card_lease_client_command(
    client: &Path,
    channel: &[(&'static str, std::ffi::OsString)],
    borrower: &str,
) -> std::process::Command {
    let mut command = std::process::Command::new(CARD_LEASE_CLIENT_INTERPRETER);
    command.arg(client);
    give_the_environment_of_the_lease(&mut command, channel, borrower);
    command
}

fn give_the_environment_of_the_lease(
    command: &mut std::process::Command,
    channel: &[(&'static str, std::ffi::OsString)],
    borrower: &str,
) {
    command.envs(
        channel
            .iter()
            .map(|(name, value)| (*name, value.as_os_str())),
    );
    command.env(CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE, borrower);
    match std::env::var_os(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE) {
        Some(configuration) => {
            command.env(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE, configuration)
        }
        None => command.env_remove(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE),
    };
}

/// 领卡客户端在哪：提交方那一份仓的根给了就是它下面那一份；没给从当前目录（cargo 把测试进程的当前目录设成包目录）往上逐层找。
fn card_lease_client() -> Result<std::path::PathBuf, String> {
    if let Some(repository) = std::env::var_os(SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE) {
        let client = Path::new(&repository).join(CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY);
        return if client.is_file() {
            Ok(client)
        } else {
            Err(format!(
                "{SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE} 点的仓根下没有领卡客户端：{} 不是文件",
                client.display()
            ))
        };
    }
    let current_directory = std::env::current_dir()
        .map_err(|error| format!("取不到这个进程的当前目录（领卡客户端从这里往上找）：{error}"))?;
    current_directory
        .ancestors()
        .map(|directory| directory.join(CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY))
        .find(|client| client.is_file())
        .ok_or_else(|| {
            format!(
                "从 {} 往上找不到领卡客户端 {CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY}",
                current_directory.display()
            )
        })
}

/// 向这一台的 GPU 服务领一回卡：`python3 <客户端> lease take` 的标准输出去空白是卡串；领到之后起还卡的子进程，这个进程退出（正常结束、panic、
/// 被杀都算）它的标准输入就关、它起 `lease return <卡串>`。
///
/// # Errors
/// 找不到客户端、读不出启动刻、起不了客户端、客户端退非 0、退 0 而没交卡串、起不了还卡的子进程（领到的当场还掉）。
fn cards_leased_through_the_client(
    channel: &[(&'static str, std::ffi::OsString)],
) -> Result<(String, std::process::Child), String> {
    let client = card_lease_client()?;
    let process_status_line = std::fs::read_to_string("/proc/self/stat").map_err(|error| {
        format!("读不了 /proc/self/stat（领卡客户端要这个进程的启动刻）：{error}")
    })?;
    let borrower = format!(
        "{}:{}",
        std::process::id(),
        start_time_since_boot_in_clock_ticks_from_the_process_status_line(&process_status_line)?
    );
    let taken = card_lease_client_command(&client, channel, &borrower)
        .args(["lease", "take"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .output()
        .map_err(|error| format!("起不了领卡客户端 {}：{error}", client.display()))?;
    if !taken.status.success() {
        return Err(format!(
            "{CARD_LEASE_CLIENT_INTERPRETER} {} lease take 退 {:?}：不重试、不退回 CPU；原因在客户端往标准错误打的那几行",
            client.display(),
            taken.status.code()
        ));
    }
    let cards = String::from_utf8_lossy(&taken.stdout).trim().to_string();
    let return_the_cards_at_once = |cards_to_return: &str| {
        // 领到了而往下走不通：交回的卡串当场还，结局不改这一句错
        let _returned = card_lease_client_command(&client, channel, &borrower)
            .args(["lease", "return"])
            .arg(cards_to_return)
            .stdin(std::process::Stdio::null())
            .status();
    };
    if cards.is_empty() {
        return_the_cards_at_once(&cards);
        return Err(format!(
            "{CARD_LEASE_CLIENT_INTERPRETER} {} lease take 退 0 而没交卡串，已当场 lease return",
            client.display()
        ));
    }
    let mut command_returning_the_cards_at_the_exit = std::process::Command::new("sh");
    command_returning_the_cards_at_the_exit
        .arg("-c")
        .arg(RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS)
        .arg(CARD_LEASE_CLIENT_INTERPRETER)
        .arg(&client)
        .arg(&cards);
    give_the_environment_of_the_lease(
        &mut command_returning_the_cards_at_the_exit,
        channel,
        &borrower,
    );
    match command_returning_the_cards_at_the_exit
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(return_waiting_for_the_exit) => Ok((cards, return_waiting_for_the_exit)),
        Err(error) => {
            return_the_cards_at_once(&cards);
            Err(format!(
                "领到了 {cards}，起不了等这个进程退出就还卡的子进程（sh）：{error}；领到的已当场还掉"
            ))
        }
    }
}

/// 判段要开卡的那一刻调：环境变量 [`VARIABLE_OF_THE_HANDED_OVER_GPUS`] 给了卡的照它，不领；没给而有领卡通道（三个变量都在、都不空）的，
/// 这一刻向这一台的 GPU 服务领（进程里只领一回），领到记给 [`note_the_cards_leased_on_demand_for_this_process`]；两样都没有的不领
/// （取卡那一步照没交卡报「本次未跑」）。
///
/// # Errors
/// 领卡没成（那一句；进程里之后每次交同一句，不重试）。
pub fn cards_leased_on_demand_unless_handed_over() -> Result<(), String> {
    if std::env::var_os(VARIABLE_OF_THE_HANDED_OVER_GPUS).is_some() {
        return Ok(());
    }
    let mut channel = Vec::with_capacity(CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES.len());
    for name in CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES {
        match std::env::var_os(name).filter(|value| !value.is_empty()) {
            Some(value) => channel.push((name, value)),
            None => return Ok(()),
        }
    }
    let leased = CARD_LEASE_OF_THIS_PROCESS
        .get_or_init(|| {
            let outcome_of_the_lease = cards_leased_through_the_client(&channel);
            if let Ok((cards, _return_waiting_for_the_exit)) = &outcome_of_the_lease {
                eprintln!("GPU_CARDS_LEASED_ON_DEMAND cards={cards}");
            }
            outcome_of_the_lease
        })
        .as_ref()
        .map_err(Clone::clone)?;
    note_the_cards_leased_on_demand_for_this_process(&leased.0);
    Ok(())
}

/// 这个测试进程的那一份 Vulkan 实例：服务交了这一台的卡时，进程里头一次取卡（[`handed_over_gpus_or_report_not_run`]）建它，之后各次取卡都交它的克隆、
/// 不再建；服务没交卡的进程里一直不建。为什么一个进程只建一个：同一个进程里新建的实例多了，Vulkan 驱动不一定还认得出独显
/// （[`VulkanInstanceOfThisProcess`] 文档「为什么一个进程只建一个」那一段）。实例看到的驱动清单定在建它那一刻，要另看一份驱动清单的用例
/// （Vulkan 驱动全藏起来的那一次）在它自己起的子进程里另建，不取这一份。
/// 建它的那一次 panic 了，这一格照旧是空的（`OnceLock::get_or_init` 的初始化 panic 不留值），下一条取卡的用例再建一次：这一条不守。
/// 建实例按 [`VulkanInstanceOfThisProcess::created_for_this_process`] 的约定不报错（驱动不在也交回实例），走到 panic 只有 wgpu 自己倒下这一种；
/// 倒下的那条用例已经红了；下一条再建也只建一份（再倒下就照样红），一个进程里建的个数不超过取卡的用例数，建成之后不再建。
static VULKAN_INSTANCE_OF_THIS_TEST_PROCESS: std::sync::OnceLock<VulkanInstanceOfThisProcess> =
    std::sync::OnceLock::new();

/// 跑一次 `nvidia-smi`（[`NVIDIA_SMI_INDEX_AND_PCI_BUS_ARGUMENTS`]）读每张卡的序号与总线地址。
fn pci_buses_reported_by_nvidia_smi() -> Result<BTreeMap<NvidiaSmiIndex, String>, String> {
    pci_buses_by_nvidia_smi_index(&output_of_nvidia_smi(
        &NVIDIA_SMI_INDEX_AND_PCI_BUS_ARGUMENTS,
    )?)
}

/// 照交来的卡取卡的结局（[`handed_over_gpus_through_the_instance_kept_for_this_process`]）。
pub enum GpusTakenThroughTheInstanceKeptForThisProcess<Instance> {
    /// 服务没交卡、或交的卡里没有这一台的（带一句原因）：没取实例，也没读 `nvidia-smi`。
    NotRun { reason: String },
    /// 这一台交来的卡逐张对上了这个进程那一份实例看得到的独显，连同那一份实例的克隆。
    Handed(HandedOverGpusSeenThroughTheInstance<Instance>),
}

/// 照 `GPU_CARDS_BY_PRIORITY` 的值（`handover_variable_value`，没设是 `None`）挑这一台的卡（[`gpus_handed_to_this_host`]），读本机每张卡的序号与总线地址
/// （`pci_buses_reported`），取这个进程的那一份实例，逐张对上它看得到的独显（`pci_bus_addresses_seen_through` 在这一份实例上枚举）。
/// 实例放在 `instance_kept`：还没有时这一次由 `instance_created` 建、存进去，已有就交它的克隆，一个进程里取几次卡都只建一次。
/// 服务没交卡、或交的卡里没有这一台的：在读 `nvidia-smi` 与取实例之前交回 [`GpusTakenThroughTheInstanceKeptForThisProcess::NotRun`]，不建实例、不碰 GPU。
/// 交了这一台的卡：先读本机的卡、再取实例，读不成本机的卡就在建实例之前判红。
/// 读本机的几样与建实例都从参数来：真卡那一份是 [`handed_over_gpus_or_report_not_run`]，不要卡的单测换上现造的输入与数建了几次的替身。
///
/// # Panics
/// 变量读不成、认不出这一台、读不成本机的卡、交来的卡有一张对不上实例看得到的独显：判红。
pub fn handed_over_gpus_through_the_instance_kept_for_this_process<Instance: Clone>(
    handover_variable_value: Option<&str>,
    this_host_address: impl FnOnce() -> Result<String, String>,
    pci_buses_reported: impl FnOnce() -> Result<BTreeMap<NvidiaSmiIndex, String>, String>,
    instance_kept: &std::sync::OnceLock<Instance>,
    instance_created: impl FnOnce() -> Instance,
    pci_bus_addresses_seen_through: impl FnOnce(&Instance) -> Vec<String>,
) -> GpusTakenThroughTheInstanceKeptForThisProcess<Instance> {
    let handed_to_this_host = gpus_handed_to_this_host(handover_variable_value, this_host_address)
        .unwrap_or_else(|reason| panic!("服务交的卡认不成：{reason}"));
    let handed_over_indices = match handed_to_this_host {
        GpusHandedToThisHost::NotRun { reason } => {
            return GpusTakenThroughTheInstanceKeptForThisProcess::NotRun { reason };
        }
        GpusHandedToThisHost::Handed(handed_over_indices) => handed_over_indices,
    };
    let pci_buses_by_index = pci_buses_reported().unwrap_or_else(|reason| {
        panic!("服务交了卡（序号 {handed_over_indices:?}），读不成本机的卡：{reason}")
    });
    let vulkan_instance = instance_kept.get_or_init(instance_created).clone();
    let pci_bus_addresses_of_the_discrete_adapters =
        pci_bus_addresses_seen_through(&vulkan_instance);
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
    GpusTakenThroughTheInstanceKeptForThisProcess::Handed(HandedOverGpusSeenThroughTheInstance {
        vulkan_instance,
        gpus: handed_over_gpus,
    })
}

/// 这一条记「本次未跑」时打的那一行：`NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`。门禁、变异判法与任务服务按行首这个记号认「本次未跑」
/// （`scripts/mutation/crates-mutation-rows.py` 的 `NOT_RUN_LINE`：记号、空格、`test=` 接不带空白的用例名，再接 ` reason=` 与原因；
/// `task-server/task_server.py` 的 `gpu_not_run_test`）。
#[must_use]
pub fn not_run_line_of_the_test(test_name: &str, reason: &str) -> String {
    format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")
}

/// 读不成这个进程在哪几张卡上时（[`gpus_this_process_is_on_and_the_line_to_print`]）要打的那一行：
/// [`not_run_line_of_the_test`]，原因带读不成的那一句 `reason_not_read`。
#[must_use]
pub fn not_run_line_when_the_gpus_of_the_process_are_not_read(
    test_name: &str,
    reason_not_read: &str,
) -> String {
    not_run_line_of_the_test(
        test_name,
        &format!("读不成这个进程在哪几张卡上：{reason_not_read}"),
    )
}

/// 读到这个进程一张卡都不在时（[`gpus_this_process_is_on_and_the_line_to_print`]；此刻交来的卡上开着设备，读到空就是这个读法看不见它的设备）
/// 要打的那一行：[`not_run_line_of_the_test`]，原因带这个进程的进程号 `process_identifier`。
#[must_use]
pub fn not_run_line_when_the_process_is_seen_on_no_gpu(
    test_name: &str,
    process_identifier: u32,
) -> String {
    not_run_line_of_the_test(
        test_name,
        &format!(
            "nvidia-smi 列的计算进程里没有这个进程（进程号 {process_identifier}），而交来的卡上开着它的设备：读法看不见它"
        ),
    )
}

/// 取到这一台的卡时打的那一行：`GPUS_HANDED_OVER_TO_THE_TEST test=<用例名> gpus=<nvidia-smi 序号>@<wgpu 报的总线地址>,…`，照交来的次序、逗号隔开。
#[must_use]
pub fn handed_over_gpus_line_of_the_test(test_name: &str, gpus: &[HandedOverGpu]) -> String {
    format!(
        "GPUS_HANDED_OVER_TO_THE_TEST test={test_name} gpus={}",
        gpus.iter()
            .map(|gpu| format!("{}@{}", gpu.nvidia_smi_index.0, gpu.pci_bus_address))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// 取卡的结局（[`handed_over_gpus_through_the_instance_kept_for_this_process`] 交回的）对应打的那一行：服务没交卡、或交的卡里没有这一台的
/// 打 [`not_run_line_of_the_test`]（带结局里的原因），交了这一台的卡打 [`handed_over_gpus_line_of_the_test`]。
#[must_use]
pub fn line_reporting_the_gpus_taken<Instance>(
    test_name: &str,
    taken: &GpusTakenThroughTheInstanceKeptForThisProcess<Instance>,
) -> String {
    match taken {
        GpusTakenThroughTheInstanceKeptForThisProcess::NotRun { reason } => {
            not_run_line_of_the_test(test_name, reason)
        }
        GpusTakenThroughTheInstanceKeptForThisProcess::Handed(handed_over) => {
            handed_over_gpus_line_of_the_test(test_name, &handed_over.gpus)
        }
    }
}

/// 服务交给这个进程、属于这一台的卡，逐张对上本机的 Vulkan 独显，照交来的次序；连同这个进程的那一份 Vulkan 实例（[`VULKAN_INSTANCE_OF_THIS_TEST_PROCESS`]，
/// 进程里头一次取卡时建）的克隆交回（用例里开卡、再枚举都用它）。判法在 [`handed_over_gpus_through_the_instance_kept_for_this_process`]，
/// 打哪一行在 [`line_reporting_the_gpus_taken`]：服务没交卡、或交的卡里没有这一台的，不建实例，打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`
/// 报「本次未跑」、交回 `None`。
///
/// # Panics
/// 变量读不成、多机配置判法认不出这一台、`nvidia-smi` 起不来或读不成、交来的卡有一张对不上本机的独显：判红。
#[must_use]
pub fn handed_over_gpus_or_report_not_run(
    test_name: &str,
    wait_limit: Duration,
) -> Option<HandedOverGpusSeenThroughTheInstance> {
    let handover_variable_value = std::env::var(VARIABLE_OF_THE_HANDED_OVER_GPUS)
        .ok()
        .or_else(|| CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS.get().cloned());
    let taken = handed_over_gpus_through_the_instance_kept_for_this_process(
        handover_variable_value.as_deref(),
        this_host_address_by_the_configuration_check,
        pci_buses_reported_by_nvidia_smi,
        &VULKAN_INSTANCE_OF_THIS_TEST_PROCESS,
        VulkanInstanceOfThisProcess::created_for_this_process,
        |vulkan_instance| {
            pci_bus_addresses_of_the_discrete_gpus_on_this_host(vulkan_instance, wait_limit)
        },
    );
    println!("{}", line_reporting_the_gpus_taken(test_name, &taken));
    match taken {
        GpusTakenThroughTheInstanceKeptForThisProcess::NotRun { .. } => None,
        GpusTakenThroughTheInstanceKeptForThisProcess::Handed(handed_over) => Some(handed_over),
    }
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
) -> Option<HandedOverGpusSeenThroughTheInstance> {
    let handed_over = handed_over_gpus_or_report_not_run(test_name, wait_limit)?;
    Some(HandedOverGpusSeenThroughTheInstance {
        vulkan_instance: handed_over.vulkan_instance,
        gpus: handed_over_gpus_of_the_declared_count(handed_over.gpus, declared_gpu_count)
            .unwrap_or_else(|reason| panic!("{reason}")),
    })
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
    let handed_over = handed_over_gpus_or_report_not_run(test_name, wait_limit)?;
    Some(opened_background_gpu(
        &handed_over.vulkan_instance,
        &selection_over_the_handed_over_gpus(
            &handed_over.gpus,
            MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
            wait_limit,
        ),
    ))
}

/// `card_count` 张参与的卡各一个 [`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`]（挂载参数 `background_gpu_memory_limit` 按次序一张卡一个、不补齐）。
#[must_use]
pub fn memory_limits_of_one_gibibyte_each(card_count: usize) -> Vec<GpuMemoryInBytes> {
    vec![MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU; card_count]
}
