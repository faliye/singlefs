//! 子命令用的卡：服务交的卡、按需领卡、本进程看到的卡与全部卡臂开哪几张。

use super::*;

// ---- 子命令共用的开头：产物、开跑检查、crates 版本（停机条款第 2 条：与产物里已有的比）、卡 ----

/// 子命令开头读到的：产物里已有的结果行、这是第几次起这条命令、这一台的主机硬件、登记的重活线程数。卡不在开头取（[`cards_seen_by_the_command`]）。
pub(super) struct OpeningOfTheCommand {
    pub(super) lines_already_in_the_product: Vec<KindAndFieldsOfAResultLine>,
    pub(super) session_number: u64,
    pub(super) host_hardware: HostHardware,
    pub(super) registered_heavy_threads: usize,
}

/// 这条命令要不要卡（几何核与造池计时不要）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CardsOfTheCommand {
    /// 要卡：开头只核服务给了卡的来路（[`the_task_server_hands_over_cards`]），只用 CPU 的活做完、开设备之前才取（[`cards_seen_by_the_command`]）。
    TakenBeforeOpeningTheDevices,
    /// 不要卡：开头打一行 `name=gpus enumerated=false`，不碰 GPU。
    NotNeeded,
}

/// 服务给这个进程卡的来路有没有：环境变量 `GPU_CARDS_BY_PRIORITY` 设了（`environment_value`），或有领卡通道（`channel`）。只看，不领。
fn the_task_server_hands_over_cards(
    environment_value: Option<&str>,
    channel: Option<&CardLeaseChannelOfTheCaller>,
) -> bool {
    environment_value.is_some() || channel.is_some()
}

/// 此刻这个进程的环境里服务给没给卡的来路（[`the_task_server_hands_over_cards`] 照此刻的环境变量与领卡通道判）。
pub(super) fn the_task_server_hands_over_cards_to_this_process() -> bool {
    the_task_server_hands_over_cards(
        std::env::var(VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER)
            .ok()
            .as_deref(),
        CardLeaseChannelOfTheCaller::read_from_the_environment_now().as_ref(),
    )
}

/// 服务既没交卡也没给领卡通道时的那一句（产物 `name=stopped` 的理由与取卡的错）。
pub(super) const REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER: &str =
    "服务既没经 GPU_CARDS_BY_PRIORITY 交卡、也没给领卡通道：这条命令要经 task-server/task submit 交，装置不自己枚举本机的卡";

/// 装置用的卡串（与 `GPU_CARDS_BY_PRIORITY` 同格式）：环境变量给了照它、不领；没给而有领卡通道，经 `lease` 领（这一刻才领）；
/// 两样都没有报错（验证包 `cards_handed_over_or_leased_on_demand` 交回 `None` 的那一支），不退回自己枚举。
///
/// # Errors
/// 两样都没有；领卡没成。
fn cards_handed_to_the_apparatus(
    environment_value: Option<String>,
    channel: Option<&CardLeaseChannelOfTheCaller>,
    lease: &dyn Fn(&CardLeaseChannelOfTheCaller) -> Result<String, String>,
) -> Result<String, String> {
    cards_handed_over_or_leased_on_demand(environment_value, channel, lease)
        .map_err(|refusal| format!("按需领卡没成：{refusal}"))?
        .ok_or_else(|| REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER.to_string())
}

/// 服务交给这个进程、属于这一台的独显，连同枚举与开卡用的那一份进程级 Vulkan 实例（交的卡里没有这一台的：一张都没有、不建实例）。
#[derive(Clone)]
pub(super) struct CardsSeenByThisProcess {
    /// 交来的卡，按 PCI 地址次序（点名一张卡臂取头一张，全部卡臂照 `--cards` 从中挑，[`discrete_gpus_named_for_the_every_card_arm`]）。
    pub(super) discrete_gpus: Vec<DiscreteGpuSeenByThisProcess>,
    /// 同一批卡的 PCI 地址，照服务交来的卡串的先后（`--cards <张数>` 取排在前头的那几张）。
    pub(super) handed_over_pci_bus_addresses_in_the_handed_over_order: Vec<String>,
    pub(super) vulkan_instance: Option<VulkanInstanceOfThisProcess>,
}

/// 这个进程取到的卡：进程里只取一回（敏感度那一条的交替跑与并发跑共用，一个进程一份 Vulkan 实例），取不成的那一句之后每次交同一句。
static CARDS_SEEN_BY_THIS_PROCESS: std::sync::OnceLock<Result<CardsSeenByThisProcess, String>> =
    std::sync::OnceLock::new();

/// 取卡模块打 `GPUS_HANDED_OVER_BY_THE_TASK_SERVER test=<名>`、`NOT_RUN_NO_GPU_ADAPTER test=<名>` 那几行时用的名字。
const NAME_OF_THE_APPARATUS_IN_THE_CARD_LINES: &str = "e202_gpu_cpu_gc_comparison";

/// panic 带出来的文字（`panic!` 与断言带的是 `String` 或 `&str`）。
fn text_carried_by_the_panic(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&'static str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "panic 带的不是文字".to_string())
}

/// 本机看得到的独显（`seen_on_this_host`，PCI 地址次序）里服务交来的那几张（`handed_over_pci_bus_addresses`，wgpu 报的原样）：
/// 不在交来的里的不留。交回留下的那几张。
///
/// # Errors
/// 交来的有一张不在本机看得到的独显里。
pub(super) fn discrete_gpus_among_the_handed_over(
    seen_on_this_host: Vec<DiscreteGpuSeenByThisProcess>,
    handed_over_pci_bus_addresses: &[String],
) -> Result<Vec<DiscreteGpuSeenByThisProcess>, String> {
    let handed_over: Vec<DiscreteGpuSeenByThisProcess> = seen_on_this_host
        .into_iter()
        .filter(|gpu| handed_over_pci_bus_addresses.contains(&gpu.pci_bus_address))
        .collect();
    if handed_over.len() != handed_over_pci_bus_addresses.len() {
        return Err(format!(
            "服务交来的卡（{handed_over_pci_bus_addresses:?}）不全在本机看得到的独显里（对上的 {:?}）",
            handed_over
                .iter()
                .map(|gpu| gpu.pci_bus_address.as_str())
                .collect::<Vec<&str>>()
        ));
    }
    Ok(handed_over)
}

/// 取卡：卡串照 [`cards_handed_to_the_apparatus`]（领到的记给取卡模块），再经取卡模块（`handed_over_gpus_or_report_not_run`）对上本机的 Vulkan 独显、
/// 建这个进程的那一份实例，只留交来的那几张（[`discrete_gpus_among_the_handed_over`]）。
///
/// # Errors
/// 没有卡的来路、领卡没成；交来的卡对不上本机的独显（取卡模块判红的那几种，panic 的文字原样带出）；枚举等超过上限。
fn cards_taken_for_this_process() -> Result<CardsSeenByThisProcess, String> {
    let cards = cards_handed_to_the_apparatus(
        std::env::var(VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER).ok(),
        CardLeaseChannelOfTheCaller::read_from_the_environment_now().as_ref(),
        &|channel| channel.cards_leased_by_this_process().map(str::to_string),
    )?;
    note_the_cards_leased_on_demand_for_this_process(&cards);
    let handed_over = std::panic::catch_unwind(|| {
        handed_over_gpus_or_report_not_run(
            NAME_OF_THE_APPARATUS_IN_THE_CARD_LINES,
            LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT,
        )
    })
    .map_err(|payload| {
        format!(
            "服务交来的卡对不上本机的独显：{}",
            text_carried_by_the_panic(&*payload)
        )
    })?;
    let Some(handed_over) = handed_over else {
        return Ok(CardsSeenByThisProcess {
            discrete_gpus: Vec::new(),
            handed_over_pci_bus_addresses_in_the_handed_over_order: Vec::new(),
            vulkan_instance: None,
        });
    };
    let handed_over_pci_bus_addresses_in_the_handed_over_order: Vec<String> = handed_over
        .gpus
        .iter()
        .map(|gpu| gpu.pci_bus_address.clone())
        .collect();
    let discrete_gpus = discrete_gpus_among_the_handed_over(
        discrete_gpus_seen_by_this_process(&handed_over.vulkan_instance)?,
        &handed_over_pci_bus_addresses_in_the_handed_over_order,
    )?;
    Ok(CardsSeenByThisProcess {
        discrete_gpus,
        handed_over_pci_bus_addresses_in_the_handed_over_order,
        vulkan_instance: Some(handed_over.vulkan_instance),
    })
}

/// 这个进程取到的卡：头一次问的时候取（[`cards_taken_for_this_process`]），之后每次交同一份；取不成的那一句之后每次交同一句。
pub(super) fn cards_of_this_process() -> Result<&'static CardsSeenByThisProcess, &'static String> {
    CARDS_SEEN_BY_THIS_PROCESS
        .get_or_init(cards_taken_for_this_process)
        .as_ref()
}

/// 要卡的子命令在只用 CPU 的活做完、开设备之前调：取这个进程的卡（[`cards_of_this_process`]），打 `name=gpus` 一行。
/// 取不成打 `name=gpus enumerated=false reason=…` 与 `name=stopped clause=cards_not_taken`，交回退出码 78。
pub(super) fn cards_seen_by_the_command(
    product: &mut ProductWriter,
) -> Result<CardsSeenByThisProcess, i32> {
    match cards_of_this_process() {
        Ok(cards) => {
            emit_the_gpus(
                product,
                cards.vulkan_instance.is_some(),
                &cards.discrete_gpus,
            );
            Ok(cards.clone())
        }
        Err(reason) => {
            product.emit(
                "gpus",
                &format!("enumerated=false reason={}", field_value(reason)),
            );
            product.emit(
                "stopped",
                &format!(
                    "clause=cards_not_taken reason={} way_out=照reason修好再交同一条命令",
                    field_value(reason)
                ),
            );
            Err(EXIT_RUN_CONDITION_NOT_MET)
        }
    }
}

/// 这条臂在交来的卡上跑不跑得了：照交来的张数判（[`reason_the_arm_is_not_run`]），本机另有没交的卡不影响。
pub(super) fn reason_the_arm_is_not_run_on_the_cards(
    compute: ComputeOfTheArm,
    cards: &CardsSeenByThisProcess,
) -> Option<&'static str> {
    reason_the_arm_is_not_run(compute, cards.discrete_gpus.len())
}

/// `--cards` 至少给几张：只开一张时全部卡臂与点名一张卡臂是同一条臂。
pub(super) const CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST: usize = 2;

/// 全部卡臂开几张（`lockstep --cards <张数>`）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CardsOfTheEveryCardArm {
    /// 没给 `--cards`：服务交来几张开几张。
    EveryHandedOverCard,
    /// `--cards <张数>`：只开服务交来的卡串里排在前头的这么多张（命令行只认不小于 [`CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST`] 的）。
    FirstHandedOverCards(NonZeroUsize),
}

impl CardsOfTheEveryCardArm {
    /// `name=every_card_arm_cards` 一行里 `named_by=` 的值。
    pub(super) const fn name(self) -> &'static str {
        match self {
            CardsOfTheEveryCardArm::EveryHandedOverCard => "every_handed_over_card",
            CardsOfTheEveryCardArm::FirstHandedOverCards(_requested_card_count) => "cards_option",
        }
    }
}

/// 全部卡臂开的卡，按 PCI 地址次序（同 [`CardsSeenByThisProcess::discrete_gpus`]）：没给 `--cards` 是交来的全部；
/// 给了 N 是服务交来的卡串里排在前头的 N 张。这一台一张都没交的交回空（照「服务交的卡里没有这一台的」办，不当命令行不对）。
///
/// # Errors
/// N 多于这一台交来的张数：不截成交来的张数。
pub(super) fn discrete_gpus_named_for_the_every_card_arm(
    cards: &CardsSeenByThisProcess,
    cards_of_the_every_card_arm: CardsOfTheEveryCardArm,
) -> Result<Vec<DiscreteGpuSeenByThisProcess>, String> {
    match cards_of_the_every_card_arm {
        CardsOfTheEveryCardArm::EveryHandedOverCard => Ok(cards.discrete_gpus.clone()),
        CardsOfTheEveryCardArm::FirstHandedOverCards(_requested_card_count)
            if cards.discrete_gpus.is_empty() =>
        {
            Ok(Vec::new())
        }
        CardsOfTheEveryCardArm::FirstHandedOverCards(requested_card_count) => {
            let handed_over_order = &cards.handed_over_pci_bus_addresses_in_the_handed_over_order;
            if requested_card_count.get() > handed_over_order.len() {
                return Err(format!(
                    "--cards {requested_card_count} 多于服务交给这一台的 {} 张卡（{handed_over_order:?}）",
                    handed_over_order.len()
                ));
            }
            let first_handed_over: &[String] = &handed_over_order[..requested_card_count.get()];
            Ok(cards
                .discrete_gpus
                .iter()
                .filter(|gpu| first_handed_over.contains(&gpu.pci_bus_address))
                .cloned()
                .collect())
        }
    }
}

/// 记卡行与组键里卡的一样值：去掉空白（[`field_value`]），读出空串写 `none`（驱动的两样两台的卡填不填、填什么没量过，
/// 登记第十二节修订第 16 条「每一趟记哪几行」末段：空串写 `none`、不停）。
pub(super) fn card_segment_value(value: &str) -> String {
    if value.is_empty() {
        "none".to_string()
    } else {
        field_value(value)
    }
}

/// 一张卡进组键的那一段：地址、型号、驱动、驱动信息，以 `|` 连（登记第十二节修订第 16 条「组标识怎么算」）。
pub(super) fn card_in_the_hardware_group_key(gpu: &DiscreteGpuSeenByThisProcess) -> String {
    [
        card_segment_value(&gpu.pci_bus_address),
        card_segment_value(&gpu.model),
        card_segment_value(&gpu.driver),
        card_segment_value(&gpu.driver_information),
    ]
    .join("|")
}

#[cfg(test)]
mod tests;
