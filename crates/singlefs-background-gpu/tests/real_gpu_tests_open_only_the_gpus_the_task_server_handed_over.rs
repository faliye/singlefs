//! 真卡测试只开服务交来的卡（`common_discrete_gpus_of_this_host`）那几个纯函数的单测：读 `GPU_CARDS_BY_PRIORITY`、挑这一台的、
//! `nvidia-smi` 序号按 PCI 总线地址对上 wgpu 的独显、交来的比源头声明的少判红、开卡只点名交来的卡；另核两张、三张卡那两个测试目标源头声明的张数
//! 与它们的用例要的张数相同（服务照源头的声明分卡）。不要卡、不挂特性，`cargo test -p singlefs-background-gpu --test <本目标>` 就能跑。
mod common_discrete_gpus_of_this_host;

use std::collections::BTreeMap;
use std::time::Duration;

use common_discrete_gpus_of_this_host::{
    gpu_card_count_declared_in_the_source_header, gpus_handed_over_by_the_task_server,
    gpus_handed_to_this_host, handed_over_gpus_matched_to_the_adapters,
    handed_over_gpus_of_the_declared_count, identifiers_of_the_handed_over_gpus,
    pci_bus_identifier_without_the_domain, pci_buses_by_nvidia_smi_index,
    selection_over_the_handed_over_gpus, this_host_address_emitted_by_the_configuration_check,
    GpuHandedOverByTheTaskServer, GpuHandoverUnreadable, GpusHandedToThisHost, HandedOverGpu,
    HandedOverGpuNotMatched, NvidiaSmiIndex, MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
};
use singlefs_background_gpu::adapter_inventory::gpu_device_identifier_of_the_pci_bus_address;

/// 这一台在用例里的地址（文档用的地址段）。
const THIS_HOST: &str = "192.0.2.1";
/// 第二台在用例里的地址（文档用的地址段）。
const OTHER_HOST: &str = "192.0.2.2";

/// 用例里不该被问到这一台是谁：问到就判红。
fn this_host_must_not_be_asked() -> Result<String, String> {
    panic!("变量没设或是空白时不该起多机配置判法问这一台是谁")
}

/// 用例里这一台的地址是 [`THIS_HOST`]。
fn this_host_is_the_documentation_address() -> Result<String, String> {
    Ok(THIS_HOST.to_string())
}

fn handed_over_gpu(nvidia_smi_index: u32, pci_bus_address: &str) -> HandedOverGpu {
    HandedOverGpu {
        nvidia_smi_index: NvidiaSmiIndex(nvidia_smi_index),
        pci_bus_address: pci_bus_address.to_string(),
    }
}

#[test]
fn each_entry_is_cut_from_the_right_so_an_ipv6_host_address_keeps_its_colons() {
    assert_eq!(
        gpus_handed_over_by_the_task_server("fd00::1:2:16G 192.0.2.1:0:32G"),
        Ok(vec![
            GpuHandedOverByTheTaskServer {
                host_address: "fd00::1".to_string(),
                nvidia_smi_index: NvidiaSmiIndex(2),
            },
            GpuHandedOverByTheTaskServer {
                host_address: "192.0.2.1".to_string(),
                nvidia_smi_index: NvidiaSmiIndex(0),
            },
        ]),
        "从右边切两刀：IPv6 地址自带的冒号留在地址里，次序照写的次序"
    );
}

#[test]
fn an_entry_with_an_empty_quota_is_unreadable() {
    assert_eq!(
        gpus_handed_over_by_the_task_server("192.0.2.1:0:"),
        Err(GpuHandoverUnreadable::EntryWithoutAddressIndexAndQuota {
            entry: "192.0.2.1:0:".to_string()
        }),
        "额度那一段是空的：读不成，不猜"
    );
}

#[test]
fn the_same_gpu_handed_over_twice_is_unreadable() {
    assert_eq!(
        gpus_handed_over_by_the_task_server("192.0.2.1:0:16G 192.0.2.1:0:16G"),
        Err(GpuHandoverUnreadable::SameGpuHandedOverTwice {
            entry: "192.0.2.1:0:16G".to_string()
        }),
        "同一台的同一张卡交了两次：读不成"
    );
}

#[test]
fn an_unset_or_blank_variable_does_not_run_and_does_not_ask_which_host_this_is() {
    for handover_variable_value in [None, Some(""), Some("  \t ")] {
        assert!(
            matches!(
                gpus_handed_to_this_host(handover_variable_value, this_host_must_not_be_asked),
                Ok(GpusHandedToThisHost::NotRun { .. })
            ),
            "变量 {handover_variable_value:?}：本次未跑，不问这一台是谁"
        );
    }
}

#[test]
fn only_the_gpus_handed_to_this_host_are_kept_in_the_handed_order() {
    let handover_text =
        format!("{OTHER_HOST}:0:16G {THIS_HOST}:1:16G {OTHER_HOST}:2:10G {THIS_HOST}:0:32G");
    assert_eq!(
        gpus_handed_to_this_host(
            Some(handover_text.as_str()),
            this_host_is_the_documentation_address,
        ),
        Ok(GpusHandedToThisHost::Handed(vec![
            NvidiaSmiIndex(1),
            NvidiaSmiIndex(0)
        ])),
        "只留这一台的两张，次序照交来的次序（序号 1 在 0 前面）"
    );
}

#[test]
fn gpus_handed_only_to_the_other_host_do_not_run_here() {
    let handover_text = format!("{OTHER_HOST}:0:16G {OTHER_HOST}:1:16G");
    assert!(
        matches!(
            gpus_handed_to_this_host(
                Some(handover_text.as_str()),
                this_host_is_the_documentation_address,
            ),
            Ok(GpusHandedToThisHost::NotRun { .. })
        ),
        "交的卡全是第二台的：这一台本次未跑"
    );
}

#[test]
fn this_host_address_is_read_from_the_this_host_line_and_not_from_the_peer_line() {
    let printed = format!(
        "HOSTS={OTHER_HOST}\\ {THIS_HOST}\nTHIS_HOST_ADDRESS={THIS_HOST}\nPEER_HOST_ADDRESS={OTHER_HOST}\nGPU_KERNEL_SERIAL_COMPILE_HOSTS=''\n"
    );
    assert_eq!(
        this_host_address_emitted_by_the_configuration_check(&printed),
        Ok(THIS_HOST.to_string()),
        "取 THIS_HOST_ADDRESS 那一行"
    );
}

#[test]
fn the_nvidia_smi_bus_with_an_eight_digit_domain_matches_the_wgpu_address_with_four() {
    assert_eq!(
        (
            pci_bus_identifier_without_the_domain("00000000:0A:00.0"),
            pci_bus_identifier_without_the_domain("0000:0a:00.0")
        ),
        ("0a:00.0".to_string(), "0a:00.0".to_string()),
        "nvidia-smi 报八位的域、大写，wgpu 报四位的域、小写：去掉域、小写之后相同"
    );
}

#[test]
fn nvidia_smi_rows_are_read_into_buses_by_index() {
    assert_eq!(
        pci_buses_by_nvidia_smi_index("0, 00000000:01:00.0\n1, 00000000:10:00.0\n"),
        Ok(BTreeMap::from([
            (NvidiaSmiIndex(0), "01:00.0".to_string()),
            (NvidiaSmiIndex(1), "10:00.0".to_string()),
        ])),
        "每行「序号, 总线地址」，总线地址去掉域"
    );
}

#[test]
fn an_index_nvidia_smi_reports_twice_is_unreadable() {
    assert!(
        pci_buses_by_nvidia_smi_index("0, 00000000:01:00.0\n0, 00000000:10:00.0\n").is_err(),
        "同一个序号报了两次：读不成"
    );
}

/// 本机两张卡：`nvidia-smi` 序号 0 在总线 01、序号 1 在总线 10。
fn buses_of_two_gpus() -> BTreeMap<NvidiaSmiIndex, String> {
    BTreeMap::from([
        (NvidiaSmiIndex(0), "01:00.0".to_string()),
        (NvidiaSmiIndex(1), "10:00.0".to_string()),
    ])
}

#[test]
fn handed_over_gpus_are_matched_to_the_adapters_on_the_same_bus_in_the_handed_order() {
    assert_eq!(
        handed_over_gpus_matched_to_the_adapters(
            &[NvidiaSmiIndex(1), NvidiaSmiIndex(0)],
            &buses_of_two_gpus(),
            &["0000:01:00.0".to_string(), "0000:10:00.0".to_string()],
        ),
        Ok(vec![
            handed_over_gpu(1, "0000:10:00.0"),
            handed_over_gpu(0, "0000:01:00.0")
        ]),
        "照交来的次序（序号 1 在前），各自对上同一条总线上的适配器、带 wgpu 报的原样"
    );
}

#[test]
fn handed_over_gpu_on_a_bus_without_a_discrete_adapter_is_red_and_not_skipped() {
    assert_eq!(
        handed_over_gpus_matched_to_the_adapters(
            &[NvidiaSmiIndex(0), NvidiaSmiIndex(1)],
            &buses_of_two_gpus(),
            &["0000:01:00.0".to_string()],
        ),
        Err(HandedOverGpuNotMatched::NoDiscreteAdapterOnTheBus {
            nvidia_smi_index: NvidiaSmiIndex(1),
            pci_bus: "10:00.0".to_string()
        }),
        "序号 1 那条总线上没有 Vulkan 独显：判红，不跳过它、不换别的卡"
    );
}

#[test]
fn handed_over_index_nvidia_smi_does_not_report_is_red() {
    assert_eq!(
        handed_over_gpus_matched_to_the_adapters(
            &[NvidiaSmiIndex(5)],
            &buses_of_two_gpus(),
            &["0000:01:00.0".to_string(), "0000:10:00.0".to_string()],
        ),
        Err(HandedOverGpuNotMatched::NotReportedByNvidiaSmi {
            nvidia_smi_index: NvidiaSmiIndex(5)
        }),
        "nvidia-smi 没报序号 5：判红"
    );
}

#[test]
fn fewer_gpus_handed_over_than_the_header_declares_is_red() {
    assert!(
        handed_over_gpus_of_the_declared_count(vec![handed_over_gpu(0, "0000:01:00.0")], 2)
            .is_err(),
        "源头声明两张、交来一张：判红，不降级成一张跑"
    );
}

#[test]
fn more_gpus_handed_over_than_the_header_declares_are_cut_to_the_declared_count_in_the_handed_order(
) {
    assert_eq!(
        handed_over_gpus_of_the_declared_count(
            vec![
                handed_over_gpu(2, "0000:41:00.0"),
                handed_over_gpu(0, "0000:01:00.0"),
                handed_over_gpu(1, "0000:10:00.0"),
            ],
            2
        ),
        Ok(vec![
            handed_over_gpu(2, "0000:41:00.0"),
            handed_over_gpu(0, "0000:01:00.0")
        ]),
        "整包一起跑时服务照最大的声明多交：照交来的次序取头两张"
    );
}

#[test]
fn the_selection_names_the_handed_over_gpus_in_the_handed_order_with_one_limit_each() {
    let handed_over = vec![
        handed_over_gpu(1, "0000:10:00.0"),
        handed_over_gpu(0, "0000:01:00.0"),
    ];
    let wait_limit = Duration::from_secs(60);
    let selection = selection_over_the_handed_over_gpus(
        &handed_over,
        MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
        wait_limit,
    );
    assert_eq!(
        (
            selection.background_gpu_devices,
            selection.background_gpu_memory_limit,
            selection.background_gpu_wait_limit
        ),
        (
            Some(vec![
                gpu_device_identifier_of_the_pci_bus_address("0000:10:00.0"),
                gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
            ]),
            Some(vec![MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU; 2]),
            wait_limit
        ),
        "只点名交来的两张（照交来的次序，不是本机全部独显），每张一个上限"
    );
    assert_eq!(
        identifiers_of_the_handed_over_gpus(&handed_over),
        vec![
            gpu_device_identifier_of_the_pci_bus_address("0000:10:00.0"),
            gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
        ]
    );
}

/// 一份测试源里用例照着取卡的那个常量（`const GPU_CARDS_DECLARED_IN_THE_HEADER: usize = <张数>;`）的值；没有那一行是 `None`。
fn gpu_count_the_tests_ask_for(source: &str) -> Option<usize> {
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix("const GPU_CARDS_DECLARED_IN_THE_HEADER: usize = ")?
            .strip_suffix(';')?
            .parse()
            .ok()
    })
}

#[test]
fn the_two_gpu_target_declares_in_its_header_the_gpu_count_its_tests_ask_for() {
    let source = include_str!(
        "checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs"
    );
    assert_eq!(
        (
            gpu_card_count_declared_in_the_source_header(source),
            gpu_count_the_tests_ask_for(source)
        ),
        (Some(2), Some(2)),
        "两张卡那个目标：源头声明两张（服务照它分），用例也要两张"
    );
}

#[test]
fn the_three_gpu_target_declares_in_its_header_the_gpu_count_its_tests_ask_for() {
    let source = include_str!(
        "checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs"
    );
    assert_eq!(
        (
            gpu_card_count_declared_in_the_source_header(source),
            gpu_count_the_tests_ask_for(source)
        ),
        (Some(3), Some(3)),
        "三张卡那个目标：源头声明三张（服务照它分），用例也要三张"
    );
}
