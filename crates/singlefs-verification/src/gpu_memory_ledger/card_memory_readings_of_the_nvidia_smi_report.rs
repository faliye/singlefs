//! 从一份 `nvidia-smi -q -d MEMORY,PIDS` 报告读出每张卡此刻的显存与这个进程占的那一份，与每读一次跑一份报告的读法。

use super::*;

/// 一次读出每张卡的显存与每个进程占的那一份报告：`nvidia-smi` 带这几个参数（文本版，每张卡一段：`FB Memory Usage` 下的
/// `Total`、`Reserved`、`Free`，`Processes` 下每个进程的 `Process ID` 与 `Used GPU Memory`）。一次调用里两样都有，空闲与这个进程占着的
/// 是同一次读的（`--query-gpu` 与 `--query-compute-apps` 不能在一次调用里一起用，`nvidia-smi` 报「Only one --query-* switch」）。
/// 格式照本机驱动 595.84 的 `nvidia-smi` 文本模板（`strings` 读出的 `Process ID : %u`、`Used GPU Memory : %llu MiB`）。
pub const NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS: [&str; 3] = ["-q", "-d", "MEMORY,PIDS"];

/// PCI 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡（卡池按它对上 wgpu 的适配器与
/// `nvidia-smi` 报的卡）。
#[must_use]
pub fn pci_bus_identifier_without_the_domain(text: &str) -> String {
    let lowered = text.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

/// 一份报告里一张卡那一段读到的。
#[derive(Default)]
struct OneCardOfTheReport {
    total_mebibytes: Option<u64>,
    reserved_mebibytes: Option<u64>,
    free_mebibytes: Option<u64>,
    /// 这个进程那几行占的合计；列了这个进程而有一行读不出是 `Err`。
    used_by_this_process_mebibytes: Option<Result<u64, ()>>,
}

/// 报告里一张卡那一段眼下读到哪一小节。
#[derive(Clone, Copy, PartialEq, Eq)]
enum SectionOfTheReport {
    FrameBufferMemory,
    Processes,
    Other,
}

/// `32607 MiB` 那样的量（MiB）；读不出交 `None`。
fn mebibytes_of_a_report_value(value: &str) -> Option<u64> {
    value.strip_suffix("MiB")?.trim().parse().ok()
}

/// 一份 `nvidia-smi -q -d MEMORY,PIDS` 报告（[`NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`]）里 `buses` 这几张卡此刻的显存
/// （与 `buses` 逐张对上，`buses` 照 [`pci_bus_identifier_without_the_domain`] 写）：空闲、空卡上能用的（总量减保留），
/// 与进程号是 `this_process_identifier` 的那几行占的合计。这个进程在一张卡上另有不走这份账的设备
/// （`has_devices_outside_the_books_on_the_bus(总线号)` 为真）时，占的那一份记成 [`OccupiedByThisProcess::AlsoOutsideTheBooks`]。
///
/// # Errors
/// 报告里没有某张卡、那张卡的总量、保留、空闲读不出，或一行进程号认不出。
pub fn card_memory_readings_of_one_nvidia_smi_report(
    report: &str,
    this_process_identifier: u32,
    buses: &[String],
    has_devices_outside_the_books_on_the_bus: &dyn Fn(&str) -> bool,
) -> Result<Vec<CardMemoryReading>, String> {
    let mut cards: BTreeMap<String, OneCardOfTheReport> = BTreeMap::new();
    let mut current_bus: Option<String> = None;
    let mut section = SectionOfTheReport::Other;
    let mut the_listed_process_is_this_one = false;
    for line in report.lines() {
        if let Some(bus) = line.strip_prefix("GPU ") {
            let bus = pci_bus_identifier_without_the_domain(bus);
            cards.insert(bus.clone(), OneCardOfTheReport::default());
            current_bus = Some(bus);
            section = SectionOfTheReport::Other;
            the_listed_process_is_this_one = false;
            continue;
        }
        let Some(card) = current_bus.as_ref().and_then(|bus| cards.get_mut(bus)) else {
            continue;
        };
        let indentation = line.len() - line.trim_start().len();
        let (key, value) = match line.split_once(':') {
            Some((key, value)) => (key.trim(), value.trim()),
            None => (line.trim(), ""),
        };
        if indentation <= 4 {
            section = match key {
                "FB Memory Usage" => SectionOfTheReport::FrameBufferMemory,
                "Processes" => SectionOfTheReport::Processes,
                _ => SectionOfTheReport::Other,
            };
            continue;
        }
        match (section, key) {
            (SectionOfTheReport::FrameBufferMemory, "Total") => {
                card.total_mebibytes = mebibytes_of_a_report_value(value);
            }
            (SectionOfTheReport::FrameBufferMemory, "Reserved") => {
                card.reserved_mebibytes = mebibytes_of_a_report_value(value);
            }
            (SectionOfTheReport::FrameBufferMemory, "Free") => {
                card.free_mebibytes = mebibytes_of_a_report_value(value);
            }
            (SectionOfTheReport::Processes, "Process ID") => {
                let process_identifier: u32 = value
                    .parse()
                    .map_err(|_| format!("nvidia-smi 的报告里一行进程号认不出：{line:?}"))?;
                the_listed_process_is_this_one = process_identifier == this_process_identifier;
            }
            (SectionOfTheReport::Processes, "Used GPU Memory")
                if the_listed_process_is_this_one =>
            {
                card.used_by_this_process_mebibytes = Some(
                    match (
                        card.used_by_this_process_mebibytes,
                        mebibytes_of_a_report_value(value),
                    ) {
                        (None, Some(used)) => Ok(used),
                        (Some(Ok(earlier)), Some(used)) => Ok(earlier.saturating_add(used)),
                        (Some(Err(())), Some(_)) | (_, None) => Err(()),
                    },
                );
            }
            (
                SectionOfTheReport::FrameBufferMemory
                | SectionOfTheReport::Processes
                | SectionOfTheReport::Other,
                _,
            ) => {}
        }
    }
    buses
        .iter()
        .map(|bus| {
            let card = cards
                .get(bus)
                .ok_or_else(|| format!("nvidia-smi 的报告里没有总线 {bus} 那张卡"))?;
            let unreadable =
                |what: &str| format!("nvidia-smi 的报告里总线 {bus} 那张卡的 {what} 读不出");
            let total_mebibytes = card.total_mebibytes.ok_or_else(|| unreadable("Total"))?;
            let reserved_mebibytes = card
                .reserved_mebibytes
                .ok_or_else(|| unreadable("Reserved"))?;
            let free_mebibytes = card.free_mebibytes.ok_or_else(|| unreadable("Free"))?;
            let occupied_by_this_process = match card.used_by_this_process_mebibytes {
                None | Some(Err(())) => OccupiedByThisProcess::NotListed,
                Some(Ok(mebibytes)) => {
                    if has_devices_outside_the_books_on_the_bus(bus) {
                        OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes }
                    } else {
                        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }
                    }
                }
            };
            Ok(CardMemoryReading {
                free_mebibytes,
                total_less_reserved_mebibytes: total_mebibytes.saturating_sub(reserved_mebibytes),
                occupied_by_this_process,
            })
        })
        .collect()
}

/// 卡池读显存的那一份：每读一次照 `run_the_report` 取一份报告（卡池是跑一次 `nvidia-smi`），整张表照这一份读
/// （[`card_memory_readings_of_one_nvidia_smi_report`]）：空闲与这个进程占着的不分两次读（文件头）。
#[must_use]
pub fn card_memory_reader_of_one_report_per_reading(
    run_the_report: Box<dyn Fn() -> Result<String, String> + Send + Sync>,
    this_process_identifier: u32,
    buses: Vec<String>,
    has_devices_outside_the_books_on_the_bus: Box<dyn Fn(&str) -> bool + Send + Sync>,
) -> CardMemoryReader {
    Box::new(move || {
        let report = run_the_report()?;
        card_memory_readings_of_one_nvidia_smi_report(
            &report,
            this_process_identifier,
            &buses,
            &*has_devices_outside_the_books_on_the_bus,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        card_memory_readings_of_one_nvidia_smi_report, pci_bus_identifier_without_the_domain,
    };
    use crate::gpu_memory_ledger::tests::{
        an_rtx_5090_in_the_report, no_device_outside_the_books, nvidia_smi_report, CardInTheReport,
        ANOTHER_PROCESS, AN_EMPTY_RTX_5090_HAS, BUS_OF_THE_RTX_5060_TI, BUS_OF_THE_RTX_5090,
        THIS_PROCESS,
    };
    use crate::gpu_memory_ledger::{CardMemoryReading, OccupiedByThisProcess};

    /// 一份报告里读出每张卡 FB 那一小节的空闲与空卡上能用的（BAR1 的不算），与这个进程那几行占的合计
    /// （别的进程的不算）；这个进程在一张卡上另有不走账的设备时那一份记成分不出；没列这个进程是没列；报告里没有点名的卡是读不出。
    #[test]
    fn one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card()
    {
        let report = nvidia_smi_report(&[
            an_rtx_5090_in_the_report(
                20_000,
                &[
                    (ANOTHER_PROCESS, 9_000),
                    (THIS_PROCESS, 3_000),
                    (THIS_PROCESS, 211),
                ],
            ),
            CardInTheReport {
                bus: BUS_OF_THE_RTX_5060_TI,
                total_mebibytes: 16_311,
                reserved_mebibytes: 462,
                free_mebibytes: 15_849,
                processes: &[(ANOTHER_PROCESS, 500)],
            },
        ]);
        let buses = vec![
            pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090),
            pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5060_TI),
        ];
        assert_eq!(buses, vec!["01:00.0".to_string(), "10:00.0".to_string()]);
        assert_eq!(
            card_memory_readings_of_one_nvidia_smi_report(
                &report,
                THIS_PROCESS,
                &buses,
                &no_device_outside_the_books
            ),
            Ok(vec![
                CardMemoryReading {
                    free_mebibytes: 20_000,
                    total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                    occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                        mebibytes: 3_211
                    },
                },
                CardMemoryReading {
                    free_mebibytes: 15_849,
                    total_less_reserved_mebibytes: 16_311 - 462,
                    occupied_by_this_process: OccupiedByThisProcess::NotListed,
                },
            ]),
            "FB 那一小节的空闲与总量减保留；这个进程两行相加，别的进程不算；第二张卡没列这个进程"
        );
        assert_eq!(
            card_memory_readings_of_one_nvidia_smi_report(&report, THIS_PROCESS, &buses, &|bus| {
                bus == "01:00.0"
            })
            .map(|readings| readings[0].occupied_by_this_process),
            Ok(OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 3_211 }),
            "这个进程在这张卡上另有不走账的设备：占的那一份分不出"
        );
        assert!(
            card_memory_readings_of_one_nvidia_smi_report(
                &report,
                THIS_PROCESS,
                &["02:00.0".to_string()],
                &no_device_outside_the_books
            )
            .is_err(),
            "报告里没有点名的卡：读不出"
        );
    }

    /// （正推 X16）：报告里这个进程那一行占的量读不出（`N/A`），记成没列（少借），不当成占了极大。
    #[test]
    fn an_unreadable_memory_of_this_process_in_the_report_is_not_listed_not_a_huge_occupancy() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
            20_000,
            &[(ANOTHER_PROCESS, 9_000), (THIS_PROCESS, 3_141)],
        )])
        .replace(": 3141 MiB", ": N/A");
        assert_eq!(
            card_memory_readings_of_one_nvidia_smi_report(
                &report,
                THIS_PROCESS,
                &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
                &no_device_outside_the_books
            )
            .map(|readings| readings[0].occupied_by_this_process),
            Ok(OccupiedByThisProcess::NotListed)
        );
    }

    /// （正推 X17）：报告里一行进程号认不出，报错（判红），不悄悄当成 0 号进程。
    #[test]
    fn an_unreadable_process_identifier_in_the_report_is_an_error_not_process_zero() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
            20_000,
            &[(ANOTHER_PROCESS, 9_000), (THIS_PROCESS, 3_141)],
        )])
        .replace(": 7777\n", ": abc\n");
        let readings = card_memory_readings_of_one_nvidia_smi_report(
            &report,
            THIS_PROCESS,
            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            &no_device_outside_the_books,
        );
        assert!(
            readings
                .as_ref()
                .is_err_and(|reason| reason.contains("进程号认不出")),
            "{readings:?}"
        );
    }

    /// （正推 X18）：报告里点名的卡的 Free 读不出，报错（判红），不悄悄当成空 0。
    #[test]
    fn an_unreadable_free_memory_in_the_report_is_an_error_not_zero() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(20_000, &[])])
            .replace(": 20000 MiB", ": N/A");
        let readings = card_memory_readings_of_one_nvidia_smi_report(
            &report,
            THIS_PROCESS,
            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            &no_device_outside_the_books,
        );
        assert!(
            readings
                .as_ref()
                .is_err_and(|reason| reason.contains("Free 读不出")),
            "{readings:?}"
        );
    }

    /// （正推 X19）：报告里点名的卡没有 Reserved 那一行（老驱动的样子），报错（判红），不悄悄当成保留 0
    /// （那样空卡上能用的就成了总量）。
    #[test]
    fn missing_reserved_memory_in_the_report_is_an_error_not_zero() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(20_000, &[])])
            .lines()
            .filter(|line| !line.trim_start().starts_with("Reserved"))
            .collect::<Vec<&str>>()
            .join("\n");
        let readings = card_memory_readings_of_one_nvidia_smi_report(
            &report,
            THIS_PROCESS,
            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            &no_device_outside_the_books,
        );
        assert!(
            readings
                .as_ref()
                .is_err_and(|reason| reason.contains("Reserved 读不出")),
            "{readings:?}"
        );
    }
}
