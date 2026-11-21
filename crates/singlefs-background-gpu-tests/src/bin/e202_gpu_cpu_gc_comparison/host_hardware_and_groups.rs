//! 读数依赖的硬件：CPU 型号、逻辑 CPU 数、臂开的卡与硬件组。

use super::*;

/// 这一台的主机硬件读数（登记第十二节修订第 16 条「主机硬件」一行）：CPU 型号（`/proc/cpuinfo` 头一个 `model name` 行冒号后的值，
/// 去掉空白）与逻辑 CPU 数（`processor` 行的行数：这一台的，不是任务被准用的那几个）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HostHardware {
    pub(super) cpu_model: String,
    pub(super) logical_cpus: u64,
}

/// `/proc/cpuinfo` 一行的键与值（冒号前后各去掉空白）；没有冒号的行交回 `None`。
fn key_and_value_of_a_cpu_information_line(line: &str) -> Option<(&str, &str)> {
    line.split_once(':')
        .map(|(key, value)| (key.trim(), value.trim()))
}

/// 从 `/proc/cpuinfo` 的文字读出主机硬件。读不到 ≠ 读到 0：没有 `model name` 行、头一个 `model name` 冒号后是空的、
/// `processor` 行一行都没有，都交回读不成的那一句，不当成一组「不知道」接着跑。
///
/// # Errors
/// 上面那三种读不成。
fn host_hardware_of_the_cpu_information(cpu_information: &str) -> Result<HostHardware, String> {
    let cpu_model = cpu_information
        .lines()
        .filter_map(key_and_value_of_a_cpu_information_line)
        .find(|(key, _value)| *key == "model name")
        .map(|(_key, value)| value)
        .ok_or_else(|| "/proc/cpuinfo 里没有 model name 行".to_string())?;
    if cpu_model.is_empty() {
        return Err("/proc/cpuinfo 头一个 model name 行冒号后是空的".to_string());
    }
    let logical_cpus = u64::try_from(
        cpu_information
            .lines()
            .filter_map(key_and_value_of_a_cpu_information_line)
            .filter(|(key, _value)| *key == "processor")
            .count(),
    )
    .expect("行数装得进 u64");
    if logical_cpus == 0 {
        return Err("/proc/cpuinfo 里没有 processor 行".to_string());
    }
    Ok(HostHardware {
        cpu_model: field_value(cpu_model),
        logical_cpus,
    })
}

/// 子命令开头主机硬件那一段的结局。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HostHardwareOfTheSession {
    /// 读成了：读数与 `name=host_hardware` 那一行 `name=` 之后的字段。
    Read {
        host_hardware: HostHardware,
        fields: String,
    },
    /// 读不成：要打的两行（`readable=false` 的那一行与 `name=stopped clause=host_hardware_not_readable`），这条命令退 78。
    NotReadable { lines: Vec<(&'static str, String)> },
}

/// 主机硬件那一段（登记第十二节修订第 16 条「主机硬件」一行，每个子命令开头、`name=session_start` 之后、取卡之前，每趟一次）：
/// `cpu_information` 是读 `/proc/cpuinfo` 的结局（读文件本身失败的那一句原样带进来）。
pub(super) fn host_hardware_of_the_session(
    session_number: u64,
    cpu_information: Result<String, String>,
) -> HostHardwareOfTheSession {
    match cpu_information.and_then(|text| host_hardware_of_the_cpu_information(&text)) {
        Ok(host_hardware) => HostHardwareOfTheSession::Read {
            fields: format!(
                "session={session_number} cpu_model={} logical_cpus={}",
                host_hardware.cpu_model, host_hardware.logical_cpus
            ),
            host_hardware,
        },
        Err(reason) => HostHardwareOfTheSession::NotReadable {
            lines: vec![
                (
                    "host_hardware",
                    format!(
                        "session={session_number} readable=false reason={}",
                        field_value(&reason)
                    ),
                ),
                (
                    "stopped",
                    format!(
                        "clause=host_hardware_not_readable reason={} way_out=再交同一条命令，两台都读不到交主agent",
                        field_value(&reason)
                    ),
                ),
            ],
        },
    }
}

/// 组标识取组键 SHA-256 十六进制的前几个字（登记第十二节修订第 16 条「组标识怎么算」）。
const HARDWARE_GROUP_IDENTIFIER_HEXADECIMAL_DIGITS: usize = 16;

/// 一趟的硬件组（登记第十二节修订第 16 条「组标识怎么算」）：组键是四段按次序、每段一行拼成的文字——CPU 型号、逻辑 CPU 数、
/// 这一趟点名一张卡臂那一张（[`card_in_the_hardware_group_key`]）、全部卡臂那几张（张数，再按 PCI 地址次序以 `;` 连），
/// 这一趟没跑的那条臂写 `none`；组标识是组键 SHA-256 的十六进制前 [`HARDWARE_GROUP_IDENTIFIER_HEXADECIMAL_DIGITS`] 个字。
/// 档、趟号、照什么挑的卡（`named_by`）不进组键。两趟组键逐字相同就是同一组。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HardwareGroup {
    pub(super) identifier: String,
    cpu_model: String,
    logical_cpus: u64,
    /// 组键里点名一张卡臂那一段 `one_card=` 之后的值。
    pub(super) one_card: String,
    /// 组键里全部卡臂那一段 `every_card=` 之后的值。
    pub(super) every_card: String,
}

impl HardwareGroup {
    /// `name=hardware_group` 那一行 `name=` 之后的字段。
    pub(super) fn fields_of_the_line(&self, session_number: u64, tier: UnitAreaTier) -> String {
        format!(
            "session={session_number} tier={} group={} cpu_model={} logical_cpus={} one_card={} every_card={}",
            tier.name(),
            self.identifier,
            self.cpu_model,
            self.logical_cpus,
            self.one_card,
            self.every_card
        )
    }
}

/// 这一趟的硬件组：`one_card` 是这一趟真跑的点名一张卡臂开的那一张，`every_card` 是这一趟真跑的全部卡臂开的那几张（按 PCI 地址次序）；
/// 没跑的那条臂交 `None`、组键里写 `none`。
pub(super) fn hardware_group_of(
    host_hardware: &HostHardware,
    one_card: Option<&DiscreteGpuSeenByThisProcess>,
    every_card: Option<&[DiscreteGpuSeenByThisProcess]>,
) -> HardwareGroup {
    let one_card_segment =
        one_card.map_or_else(|| "none".to_string(), card_in_the_hardware_group_key);
    let every_card_segment = every_card.map_or_else(
        || "none".to_string(),
        |gpus| {
            format!(
                "{}|{}",
                gpus.len(),
                gpus.iter()
                    .map(card_in_the_hardware_group_key)
                    .collect::<Vec<String>>()
                    .join(";")
            )
        },
    );
    let key = format!(
        "cpu_model={}\nlogical_cpus={}\none_card={one_card_segment}\nevery_card={every_card_segment}\n",
        host_hardware.cpu_model, host_hardware.logical_cpus
    );
    let mut digest = Sha256Digest::new();
    digest.feed(key.as_bytes());
    let identifier =
        digest.finish_as_hex()[..HARDWARE_GROUP_IDENTIFIER_HEXADECIMAL_DIGITS].to_string();
    HardwareGroup {
        identifier,
        cpu_model: host_hardware.cpu_model.clone(),
        logical_cpus: host_hardware.logical_cpus,
        one_card: one_card_segment,
        every_card: every_card_segment,
    }
}

/// `name=one_card_arm_card` 那一行 `name=` 之后的字段：点名一张卡臂这一趟开的那一张。
pub(super) fn one_card_arm_card_fields(
    session_number: u64,
    tier: UnitAreaTier,
    gpu: &DiscreteGpuSeenByThisProcess,
) -> String {
    format!(
        "session={session_number} tier={} pci_bus_address={} model={} driver={} driver_information={}",
        tier.name(),
        card_segment_value(&gpu.pci_bus_address),
        card_segment_value(&gpu.model),
        card_segment_value(&gpu.driver),
        card_segment_value(&gpu.driver_information)
    )
}

/// `name=every_card_arm_cards` 那一行 `name=` 之后的字段：全部卡臂这一趟开了几张、照什么开的、每张的地址、型号、驱动、驱动信息
/// （四个列表按 PCI 地址次序、逐张对齐，以 `,` 连）。
pub(super) fn every_card_arm_cards_fields(
    session_number: u64,
    tier: UnitAreaTier,
    cards_of_the_every_card_arm: CardsOfTheEveryCardArm,
    gpus: &[DiscreteGpuSeenByThisProcess],
) -> String {
    let joined = |value_of: fn(&DiscreteGpuSeenByThisProcess) -> &str| {
        gpus.iter()
            .map(|gpu| card_segment_value(value_of(gpu)))
            .collect::<Vec<String>>()
            .join(",")
    };
    format!(
        "session={session_number} tier={} cards_named={} named_by={} pci_bus_addresses={} models={} drivers={} driver_information_of_each_card={}",
        tier.name(),
        gpus.len(),
        cards_of_the_every_card_arm.name(),
        joined(|gpu| gpu.pci_bus_address.as_str()),
        joined(|gpu| gpu.model.as_str()),
        joined(|gpu| gpu.driver.as_str()),
        joined(|gpu| gpu.driver_information.as_str())
    )
}

#[cfg(test)]
mod tests;
