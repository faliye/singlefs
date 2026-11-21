use super::*;
use crate::gpu_memory_ledger::{BookingRefusal, NeedLargerThanEveryCard};

/// 假的 sysfs 里 PCI 总线那一截的样子。
enum FakePciBus<'devices> {
    /// `<根>/bus` 在，`<根>/bus/pci` 不在。
    NotThere,
    /// `<根>/bus/pci` 在，它下面没有设备子目录。
    ThereWithoutItsDevicesDirectory,
    /// `<根>/bus/pci/devices/<地址>` 每个设备一个子目录，`vendor` 与 `class` 照写；`None` 的那个文件不建。
    ThereWithDevices(&'devices [(&'devices str, Option<&'devices str>, Option<&'devices str>)]),
}

/// 照 `pci_bus` 建一棵假的 sysfs，交回 `(根, 总线目录 <根>/bus/pci)`；根丢掉时连同整棵删掉。
fn fake_sysfs(
    test_name: &str,
    pci_bus: &FakePciBus<'_>,
) -> (
    crate::crash_library::ExclusiveTemporaryDirectory,
    std::path::PathBuf,
) {
    let root = crate::crash_library::ExclusiveTemporaryDirectory::create(&format!(
        "gpu-test-start-sysfs-{test_name}"
    ))
    .expect("建得了一次性临时目录");
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

/// 取卡入口只在服务交的卡里挑，挑最空也限在这几张：序号 0、3 比交的两张都空，没点名就不挑；交的两张照空闲显存从大到小排，额度跟着各自那张。不要卡。
/// 判别力（`crates/mutations.tsv`「测试取卡入口：点名之外的卡也挑」「测试取卡入口：交的卡里挑最满的」）。
#[test]
fn only_the_cards_handed_over_are_picked_emptiest_first_even_when_another_card_is_emptier() {
    let handed_over = [(1, 8192), (2, 16384)];
    let reported = [(0, 30000), (1, 10000), (2, 20000), (3, 25000)];
    assert_eq!(
        handed_over_cards_emptiest_first(&handed_over, &reported),
        vec![(2, 16384), (1, 8192)]
    );
}

/// 服务交的卡一张都没起来要判红，不交回空卡表，判红的原因点名交的卡与 `nvidia-smi` 报的卡。不要卡。
/// 判别力（`crates/mutations.tsv`「测试取卡入口：交的卡一张都用不了时交回空卡表」）。
#[test]
fn handed_over_cards_of_which_none_started_give_the_reason_to_fail() {
    let started_cards: Vec<u32> = Vec::new();
    let reason = started_cards_or_the_reason_the_handed_over_cards_do_not_run(
        started_cards,
        &[(4, 8192)],
        &[(0, 30000)],
    )
    .expect_err("一张都没起来要交判红的原因");
    assert!(
        reason.contains("[(4, 8192)]") && reason.contains("[(0, 30000)]"),
        "原因要点名交的卡与 nvidia-smi 报的卡：{reason}"
    );
}

/// 真卡接线：服务交了卡时，取卡入口起出来的每张卡都在交给这一台的卡里；服务没交卡时一个设备都不起，打「本次未跑」那一行。
/// 本机没被交的卡有一张空闲显存够门槛，入口不看交的卡时这一条才红（只有交的这几张用得上的机器上入口看不看都一样）。
/// 判别力（`crates/mutations.tsv`「测试取卡入口：不看服务交的卡」）。
#[test]
fn the_entry_under_cards_handed_out_by_the_service_starts_only_those_cards() {
    let test_name = "the_entry_under_cards_handed_out_by_the_service_starts_only_those_cards";
    let cards_handed_out =
        crate::gpu_card_configuration::GpuConfigurationOfTheCaller::of_this_process()
            .cards_by_priority(None)
            .expect("服务交的卡读得成");
    let Some(handed_out) = cards_handed_out else {
        println!(
            "{}",
            not_run_line(test_name, "服务没经 GPU_CARDS_BY_PRIORITY 交卡")
        );
        return;
    };
    let handed_over_indices: Vec<u32> = handed_out
        .cards_of_this_host()
        .iter()
        .map(|(index, _)| *index)
        .collect();
    let Some(cards) = usable_cards_or_report_not_run(test_name) else {
        return;
    };
    for card in &cards {
        assert!(
            handed_over_indices.contains(&card.index),
            "起了服务没交的卡：序号 {}，交给这一台的是 {handed_over_indices:?}",
            card.index
        );
    }
}

/// 场景：调用方那一份的卡串没设、是空串、只有空白，GPU 用例照它取卡（[`super::gpu_test_start::usable_cards_or_report_not_run`] 的判法，
/// 取的是不打那一行的 [`super::gpu_test_start::usable_cards_of_the_configuration_or_the_not_run_line`]）。
/// 预期：都当这台机器没有卡，交回未跑与「本次未跑」那一行（点名这条用例、原因点名调用方没交卡串），不交卡表；
/// 不读多机配置、不找本机的卡、不起设备，那一行不打到标准输出（打了门禁就把这条单测当成本次未跑）。不要卡。
/// 判别力（`crates/mutations.tsv`「测试取卡入口 没交卡串也往下交卡表（不报本次未跑）」）：没交卡串那一支交回空卡表，这条红。
#[test]
fn the_entry_without_a_card_string_of_the_caller_reports_not_run_and_starts_no_device() {
    use super::{usable_cards_of_the_configuration_or_the_not_run_line, TakenOrNotRun};
    let test_name =
        "the_entry_without_a_card_string_of_the_caller_reports_not_run_and_starts_no_device";
    for card_string in [None, Some(""), Some("  ")] {
        match usable_cards_of_the_configuration_or_the_not_run_line(
            test_name,
            &crate::gpu_card_configuration::GpuConfigurationOfTheCaller::for_the_unit_tests(
                card_string,
            ),
        ) {
            TakenOrNotRun::NotRun { line } => assert!(
                line.starts_with(&format!(
                    "{} test={test_name} reason=",
                    crate::gpu_memory_ledger::GPU_NOT_RUN_MARKER
                )) && line.contains("调用方没交卡串"),
                "卡串 {card_string:?}：交回的「本次未跑」那一行点名这条用例、原因点名调用方没交卡串：{line}"
            ),
            TakenOrNotRun::Taken(cards) => panic!(
                "卡串 {card_string:?}：当没有卡、报本次未跑，不交卡表（交回了 {} 张）",
                cards.len()
            ),
        }
    }
}

/// 场景：调用方那一份的卡串是空串（服务交了一个空白卡串），卡池照它起池（[`super::gpu_test_start::card_pool_start_outcome_on`]）。
/// 预期：起池判成没卡、原因点名调用方没交卡串，不起池（不开账）、不找本机的卡，不当「本机能用的卡全用」。不要卡。
/// 判别力（`crates/mutations.tsv`「显卡卡串 空串也当交了卡」）：空串当交了，读卡项读不成、照本机有没有卡判，原因不点名没交卡串，这条红。
#[test]
fn a_card_pool_start_on_a_blank_card_string_starts_no_card_pool_and_is_not_run() {
    use super::{card_pool_start_outcome_on, GpuCardPoolStartOutcome};
    let configuration =
        crate::gpu_card_configuration::GpuConfigurationOfTheCaller::for_the_unit_tests(Some(""));
    let outcome = match card_pool_start_outcome_on(&configuration) {
        GpuCardPoolStartOutcome::NotRunNoGpuAdapter { reason } => Ok(reason),
        GpuCardPoolStartOutcome::Started(_) => Err("起了池".to_string()),
        GpuCardPoolStartOutcome::CardsPresentButNotUsable { reason } => {
            Err(format!("判成有卡而用不上：{reason}"))
        }
    };
    assert!(
        outcome
            .as_ref()
            .is_ok_and(|reason| reason.contains("调用方没交卡串")),
        "空卡串：起池判成没卡、原因点名调用方没交卡串，不起池：{outcome:?}"
    );
}

/// 场景：调用方那一份的卡串没设、是空串、只有空白，判硬件的那一步给「这台机器有 NVIDIA 的卡」，GPU 用例开头取卡池把手
/// （[`super::gpu_card_pool_or_report_not_run`] 的判法，取的是不打那一行的 [`super::gpu_card_pool_of_the_configuration_or_the_not_run_line`]）。
/// 预期：拿把手这一步就交回未跑与「本次未跑」那一行（点名这条用例、原因点名调用方没交卡串），不交把手、不 panic；
/// 判硬件的那一步一回都不调（不找本机的卡）；那一行不打到标准输出（打了门禁就把这条单测当成本次未跑）。
/// 交回把手的话，要池里的卡那一路（`GpuCardPoolStartedAtTheFirstLease::card_pool_started_now`）起池判成没卡就 panic。不要卡。
/// 判别力（`crates/mutations.tsv`「GPU 卡池把手 没交卡串也往下判硬件、交把手（要池里的卡时 panic）」）：没交卡串那一支往下判硬件、交回把手，这条红。
#[test]
fn the_card_pool_handle_without_a_card_string_of_the_caller_is_not_run_before_the_hardware_is_judged(
) {
    use super::{gpu_card_pool_of_the_configuration_or_the_not_run_line, TakenOrNotRun};
    let test_name =
        "the_card_pool_handle_without_a_card_string_of_the_caller_is_not_run_before_the_hardware_is_judged";
    for card_string in [None, Some(""), Some("  ")] {
        let is_the_hardware_judged = std::cell::Cell::new(false);
        let taken = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gpu_card_pool_of_the_configuration_or_the_not_run_line(
                test_name,
                &crate::gpu_card_configuration::GpuConfigurationOfTheCaller::for_the_unit_tests(
                    card_string,
                ),
                || {
                    is_the_hardware_judged.set(true);
                    NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool
                },
            )
        }));
        match taken {
            Ok(TakenOrNotRun::NotRun { line }) => assert!(
                line.starts_with(&format!(
                    "{} test={test_name} reason=",
                    crate::gpu_memory_ledger::GPU_NOT_RUN_MARKER
                )) && line.contains("调用方没交卡串"),
                "卡串 {card_string:?}：交回的「本次未跑」那一行点名这条用例、原因点名调用方没交卡串：{line}"
            ),
            Ok(TakenOrNotRun::Taken(handle)) => panic!(
                "卡串 {card_string:?}：没交卡串就当没有卡、拿把手这一步报本次未跑，不交把手（交回了 {handle:?}）"
            ),
            Err(_payload) => panic!("卡串 {card_string:?}：拿卡池把手时 panic 了"),
        }
        assert!(
            !is_the_hardware_judged.get(),
            "卡串 {card_string:?}：没交卡串不判这台机器有没有 NVIDIA 的卡"
        );
    }
}

/// 场景：调用方交了卡串（卡项不读，写法照 `GPU_CARDS_BY_PRIORITY`），判硬件的那一步给「这台机器有 NVIDIA 的卡」，取卡池把手
/// （[`super::gpu_card_pool_of_the_configuration_or_the_not_run_line`]）。预期：交卡池把手，判硬件的那一步调过一回。不要卡、不起池。
/// 判别力（`crates/mutations.tsv`「GPU 卡池把手 交了卡串、有卡也报本次未跑」）：有卡那一支交回未跑，这条红。
#[test]
fn the_card_pool_handle_with_a_card_string_of_the_caller_on_a_machine_with_nvidia_cards_is_handed_out(
) {
    use super::{gpu_card_pool_of_the_configuration_or_the_not_run_line, TakenOrNotRun};
    let test_name =
        "the_card_pool_handle_with_a_card_string_of_the_caller_on_a_machine_with_nvidia_cards_is_handed_out";
    let is_the_hardware_judged = std::cell::Cell::new(false);
    let taken = gpu_card_pool_of_the_configuration_or_the_not_run_line(
        test_name,
        &crate::gpu_card_configuration::GpuConfigurationOfTheCaller::for_the_unit_tests(Some(
            "127.0.0.1:0:4G",
        )),
        || {
            is_the_hardware_judged.set(true);
            NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool
        },
    );
    assert!(
        matches!(taken, TakenOrNotRun::Taken(_)),
        "交了卡串、这台机器有 NVIDIA 的卡：交卡池把手（交回了 {taken:?}）"
    );
    assert!(
        is_the_hardware_judged.get(),
        "交了卡串：判过这台机器有没有 NVIDIA 的卡"
    );
}

/// 场景：调用方交了卡串，判硬件的那一步给「这台机器没有 NVIDIA 的卡」（带一句原因），取卡池把手
/// （[`super::gpu_card_pool_of_the_configuration_or_the_not_run_line`]）。预期：交回未跑与「本次未跑」那一行，原因是判硬件给的那一句，不交把手。不要卡。
/// 判别力（`crates/mutations.tsv`「GPU 卡池把手 交了卡串、没卡也交把手」）：没卡那一支交回把手，这条红。
#[test]
fn the_card_pool_handle_with_a_card_string_of_the_caller_on_a_machine_without_nvidia_cards_is_not_run(
) {
    use super::{gpu_card_pool_of_the_configuration_or_the_not_run_line, TakenOrNotRun};
    let test_name =
        "the_card_pool_handle_with_a_card_string_of_the_caller_on_a_machine_without_nvidia_cards_is_not_run";
    let reason_of_the_hardware = "假的：Vulkan 与 PCI 上都没有 NVIDIA 的卡";
    let taken = gpu_card_pool_of_the_configuration_or_the_not_run_line(
        test_name,
        &crate::gpu_card_configuration::GpuConfigurationOfTheCaller::for_the_unit_tests(Some(
            "127.0.0.1:0:4G",
        )),
        || NvidiaHardwareOfThisMachine::Absent {
            reason: reason_of_the_hardware.to_string(),
        },
    );
    match taken {
        TakenOrNotRun::NotRun { line } => assert_eq!(
            line,
            not_run_line(test_name, reason_of_the_hardware),
            "交了卡串、这台机器没有 NVIDIA 的卡：「本次未跑」那一行的原因是判硬件给的那一句"
        ),
        TakenOrNotRun::Taken(handle) => {
            panic!("交了卡串、这台机器没有 NVIDIA 的卡：报本次未跑，不交把手（交回了 {handle:?}）")
        }
    }
}

/// 场景：GPU 用例开头取卡池把手的接线（[`super::gpu_card_pool_or_report_not_run`] 照它取，取的是不打那一行的
/// [`super::gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line`]），读调用方那一份的那一步由用例交进来、卡串没设，
/// 判硬件的那一步给「这台机器有 NVIDIA 的卡」。卡串设没设由用例自己定，不看这一趟的环境。
/// 预期：读了一回调用方那一份、交给判法：交回未跑与「本次未跑」那一行（点名这条用例、原因点名调用方没交卡串），不交把手；
/// 判硬件的那一步一回都不调；那一行不打到标准输出（打了门禁就把这条单测当成本次未跑）。不要卡。
/// 判别力（`crates/mutations.tsv`「GPU 卡池把手 取卡池的接线不把调用方那一份交给判法、只看硬件（没交卡串也交把手）」）：
/// 接线改回只看硬件、交回把手，这条红。
#[test]
fn the_card_pool_entry_hands_the_configuration_of_the_caller_to_the_judgement_and_without_a_card_string_hands_out_no_handle(
) {
    use super::{
        gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line, TakenOrNotRun,
    };
    let test_name = "the_card_pool_entry_hands_the_configuration_of_the_caller_to_the_judgement_and_without_a_card_string_hands_out_no_handle";
    let configuration_without_a_card_string =
        crate::gpu_card_configuration::GpuConfigurationOfTheCaller::for_the_unit_tests(None);
    let times_the_configuration_of_the_caller_is_read = std::cell::Cell::new(0_u32);
    let is_the_hardware_judged = std::cell::Cell::new(false);
    let taken = gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line(
        test_name,
        || {
            times_the_configuration_of_the_caller_is_read
                .set(times_the_configuration_of_the_caller_is_read.get() + 1);
            &configuration_without_a_card_string
        },
        || {
            is_the_hardware_judged.set(true);
            NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool
        },
    );
    match taken {
        TakenOrNotRun::NotRun { line } => assert!(
            line.starts_with(&format!(
                "{} test={test_name} reason=",
                crate::gpu_memory_ledger::GPU_NOT_RUN_MARKER
            )) && line.contains("调用方没交卡串"),
            "调用方没交卡串：交回的「本次未跑」那一行点名这条用例、原因点名调用方没交卡串：{line}"
        ),
        TakenOrNotRun::Taken(handle) => panic!(
            "调用方没交卡串：取卡池的接线把调用方那一份交给判法、报本次未跑，不交把手（交回了 {handle:?}）"
        ),
    }
    assert_eq!(
        times_the_configuration_of_the_caller_is_read.get(),
        1,
        "取卡池的接线读一回调用方那一份"
    );
    assert!(
        !is_the_hardware_judged.get(),
        "调用方没交卡串：不判这台机器有没有 NVIDIA 的卡"
    );
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
fn pci_bus_directory_that_is_not_there_is_read_as_the_bus_not_being_visible() {
    let (root, bus_directory) = fake_sysfs("bus-not-there", &FakePciBus::NotThere);
    let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
    drop(root);
    assert!(
        matches!(read, PciNvidiaDisplayControllers::PciBusNotVisible { .. }),
        "总线目录整个不在是看不见 PCI 总线，不是设备表读不出：{read:?}"
    );
}

#[test]
fn visible_pci_bus_without_its_devices_directory_is_an_unreadable_listing_not_an_invisible_bus() {
    let (root, bus_directory) = fake_sysfs(
        "bus-without-devices",
        &FakePciBus::ThereWithoutItsDevicesDirectory,
    );
    let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
    drop(root);
    assert!(
        matches!(read, PciNvidiaDisplayControllers::ListingUnreadable { .. }),
        "总线目录在、设备子目录列不出是读不出，不许当成看不见总线：{read:?}"
    );
}

#[test]
fn visible_pci_bus_with_a_device_whose_vendor_cannot_be_read_and_no_nvidia_adapter_is_red() {
    let (root, bus_directory) = fake_sysfs(
        "device-vendor-unreadable",
        &FakePciBus::ThereWithDevices(&[("0000:00:1f.3", None, Some("0x040300\n"))]),
    );
    let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
    drop(root);
    let start = decide_gpu_test_start(&[], &read, 0);
    assert!(
        matches!(
            start,
            GpuTestStart::PciBusVisibleButListingUnreadable { .. }
        ),
        "总线看得见而某个设备的 vendor 读不出，分不清没卡与驱动坏，照旧判红、不报「本次未跑」；读到 {read:?}，判成了 {start:?}"
    );
}

/// 的条款：卡池只认进程起来时（第一次换环境之前）调用方的那份配置，不跟用例临时改的环境走。确定地造出
/// 「别的用例拿着改环境的锁换着临时配置时第一次起池」那种交错（`crate::gpu_card_configuration::while_another_thread_holds_a_temporary_gpu_configuration`：
/// 那条线程把多机配置、显卡先后换成它的临时那一份，这边起完池它才放）：起池照的是调用方给这个进程的那一份
/// （`GpuConfigurationOfTheCaller::of_this_process`），不是那条线程的。起池那一步只找卡、开账，不起设备、不借显存，没有卡的机器上照样判
/// （判的是起池照哪一份配置挑卡，不看找没找到卡）。
#[test]
fn the_card_pool_started_while_another_thread_holds_a_temporary_configuration_picks_from_the_configuration_of_the_caller(
) {
    use super::start_the_gpu_card_pool;
    use crate::gpu_card_configuration::{
        while_another_thread_holds_a_temporary_gpu_configuration, GpuConfigurationOfTheCaller,
        CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION,
    };
    let (start, temporary_configuration) = while_another_thread_holds_a_temporary_gpu_configuration(
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

/// 排队那几条用例最多等多久（秒）：到点还没交回就判红（排队等不到头就是这样红，不挂住整个测试二进制）。
const LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS: u64 = 120;

/// 排队那几条用例的假卡：一张卡、容量是整数份，借在假卡的锁里判放不放得下（同卡池那一份账的形状）：放不下交「此刻借不到」，
/// 比容量还大交「比每张卡都大」。`held_outside_this_process_shares` 是别的进程占着的（不经排队、不会还）。
/// 记着此刻借出去几份、同一时刻最多借出去几份、借不到过几回。
struct FakeCardOfShares {
    capacity_shares: u64,
    held: std::sync::Mutex<SharesOnTheFakeCard>,
}

#[derive(Clone, Copy)]
struct SharesOnTheFakeCard {
    held_shares: u64,
    most_held_at_once_shares: u64,
    attempts_without_room: u64,
}

/// 假卡借出去的一笔：丢掉时先还份数，再丢数进排队的那一笔（同 `GpuCardLease` 的字段次序：账上那一笔先还，再叫醒排队的）。
struct FakeLeaseOfShares<'test> {
    card: &'test FakeCardOfShares,
    shares: u64,
    _counted_out_of_the_waiting_room: LeaseCountedOutOfTheWaitingRoom<'test>,
}

impl Drop for FakeLeaseOfShares<'_> {
    fn drop(&mut self) {
        let mut held = self.card.held_locked();
        held.held_shares = held
            .held_shares
            .checked_sub(self.shares)
            .expect("每笔只还一次：借的时候假卡上加过这么多");
    }
}

impl FakeCardOfShares {
    fn with_capacity_and_held_outside_this_process(
        capacity_shares: u64,
        held_outside_this_process_shares: u64,
    ) -> Self {
        Self {
            capacity_shares,
            held: std::sync::Mutex::new(SharesOnTheFakeCard {
                held_shares: held_outside_this_process_shares,
                most_held_at_once_shares: held_outside_this_process_shares,
                attempts_without_room: 0,
            }),
        }
    }

    fn held_locked(&self) -> std::sync::MutexGuard<'_, SharesOnTheFakeCard> {
        self.held
            .lock()
            .expect("假卡的锁里只加减几个数，不 panic：锁不会有毒")
    }

    /// 照假卡借 `shares` 份一次；借到的那一笔数进 `waiting_room`。
    fn attempt_to_lease<'test>(
        &'test self,
        waiting_room: &'test CardLeaseWaitingRoom,
        shares: u64,
    ) -> LeaseAttempt<FakeLeaseOfShares<'test>> {
        let mut held = self.held_locked();
        if shares > self.capacity_shares {
            return LeaseAttempt::LargerThanEveryCard {
                reason: format!(
                    "要借 {shares} 份，假卡空着时也只借得出 {} 份",
                    self.capacity_shares
                ),
            };
        }
        if held.held_shares + shares > self.capacity_shares {
            held.attempts_without_room += 1;
            return LeaseAttempt::NoRoomNow {
                reason: format!(
                    "假卡上此刻借出去 {} 份，放不下 {shares} 份",
                    held.held_shares
                ),
            };
        }
        held.held_shares += shares;
        held.most_held_at_once_shares = held.most_held_at_once_shares.max(held.held_shares);
        LeaseAttempt::Leased(FakeLeaseOfShares {
            card: self,
            shares,
            _counted_out_of_the_waiting_room: waiting_room.count_a_lease_out(),
        })
    }

    fn shares_now(&self) -> SharesOnTheFakeCard {
        *self.held_locked()
    }
}

/// 一条假用例：排队借一份，借到之后拿着它等到 `every_fake_test_attempted` 成立（同一时刻有借着的、也有排队的，才判得出排队），
/// 交回它判出的结果（序号的平方加一）、还卡；排队交回没借到交 `None`。
///
/// # Panics
/// 排队交回「比每张卡都大」（一份不会比假卡大）。
fn fake_test_holding_one_share(
    card: &FakeCardOfShares,
    waiting_room: &CardLeaseWaitingRoom,
    ordinal: u64,
    attempted_once: &dyn Fn(),
    every_fake_test_attempted: &dyn Fn() -> bool,
) -> Option<u64> {
    let mut is_the_first_attempt = true;
    let lease = match waiting_room.lease_waiting_for_the_other_threads(|| {
        if is_the_first_attempt {
            is_the_first_attempt = false;
            attempted_once();
        }
        card.attempt_to_lease(waiting_room, 1)
    }) {
        LeaseAfterWaiting::Leased(lease) => lease,
        LeaseAfterWaiting::NoRoomAndNothingToWaitFor { reason: _ } => return None,
        LeaseAfterWaiting::LargerThanEveryCard { reason } => {
            panic!("假用例 {ordinal} 只借一份：{reason}")
        }
    };
    let holding_started = std::time::Instant::now();
    while !every_fake_test_attempted()
        && holding_started.elapsed()
            < std::time::Duration::from_secs(LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS)
    {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let judged = ordinal * ordinal + 1;
    drop(lease);
    Some(judged)
}

/// 「放开并按显存排队」：六条假用例各要一份、假卡只放得下两份，六条同时起（各一条线程）：同一时刻借着的不超过两份
/// （借到的两条拿着卡等每条都借过一次，正好两份同时借着），借不到的那四条排队、等到别的线程还卡再借，六条全部借到、都交回
/// （不死锁，[`LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS`] 之内），结果与同一组假用例一条一条串行跑逐项相同。不要卡。
#[test]
fn fake_tests_sharing_a_card_room_for_two_wait_their_turn_all_finish_and_agree_with_a_serial_run() {
    const FAKE_TESTS: u64 = 6;
    const CAPACITY_SHARES: u64 = 2;
    let serial_card =
        FakeCardOfShares::with_capacity_and_held_outside_this_process(CAPACITY_SHARES, 0);
    let serial_waiting_room = CardLeaseWaitingRoom::with_no_lease_out();
    let serial: Vec<Option<u64>> = (0..FAKE_TESTS)
        .map(|ordinal| {
            fake_test_holding_one_share(
                &serial_card,
                &serial_waiting_room,
                ordinal,
                &|| {},
                &|| true,
            )
        })
        .collect();
    assert!(
        serial.iter().all(Option::is_some),
        "串行跑一条一条借，每条都借得到：{serial:?}"
    );

    let card = std::sync::Arc::new(
        FakeCardOfShares::with_capacity_and_held_outside_this_process(CAPACITY_SHARES, 0),
    );
    let waiting_room = std::sync::Arc::new(CardLeaseWaitingRoom::with_no_lease_out());
    let fake_tests_that_attempted = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let (finished_sender, finished_receiver) = std::sync::mpsc::channel();
    let fake_test_threads: Vec<std::thread::JoinHandle<()>> = (0..FAKE_TESTS)
        .map(|ordinal| {
            let card = std::sync::Arc::clone(&card);
            let waiting_room = std::sync::Arc::clone(&waiting_room);
            let fake_tests_that_attempted = std::sync::Arc::clone(&fake_tests_that_attempted);
            let finished_sender = finished_sender.clone();
            std::thread::spawn(move || {
                let judged = fake_test_holding_one_share(
                    &card,
                    &waiting_room,
                    ordinal,
                    &|| {
                        fake_tests_that_attempted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    },
                    &|| {
                        fake_tests_that_attempted.load(std::sync::atomic::Ordering::SeqCst)
                            == FAKE_TESTS
                    },
                );
                finished_sender
                    .send((ordinal, judged))
                    .expect("收的一头等到六条都交回或到点才丢");
            })
        })
        .collect();
    drop(finished_sender);
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS);
    let mut parallel: Vec<Option<u64>> = vec![None; serial.len()];
    for finished_count in 0..FAKE_TESTS {
        let (ordinal, judged) = finished_receiver
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .unwrap_or_else(|error| {
                panic!(
                    "{LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS} 秒里只交回 {finished_count} 条假用例（{error}）：排队等不到头"
                )
            });
        parallel[usize::try_from(ordinal).expect("序号装得进 usize")] = judged;
    }
    for fake_test_thread in fake_test_threads {
        fake_test_thread.join().expect("假用例的线程不 panic");
    }
    let shares = card.shares_now();
    assert_eq!(
        parallel, serial,
        "并行排队跑的六条全部借到，结果与串行跑逐项相同"
    );
    assert_eq!(
        shares.most_held_at_once_shares, CAPACITY_SHARES,
        "同一时刻借着的不超过假卡放得下的两份（借到的两条拿着卡等每条都借过，正好两份同时借着）"
    );
    assert!(
        shares.attempts_without_room >= FAKE_TESTS - CAPACITY_SHARES,
        "借不到的那四条至少各借不到一回、排了队：借不到 {} 回",
        shares.attempts_without_room
    );
    assert_eq!(shares.held_shares, 0, "六条都还了卡");
}

/// 要的比假卡空着时能借的都大（要三份、只放得下两份）：别的线程借着一份（照「此刻借不到」会等它），排队照样不等，
/// 当场交回「比每张卡都大」，原因原样带着要多少、最大的一张借得出多少（卡池照它判红）。不要卡。
#[test]
fn lease_larger_than_every_card_is_refused_at_once_with_the_sizes_instead_of_waiting() {
    let card =
        std::sync::Arc::new(FakeCardOfShares::with_capacity_and_held_outside_this_process(2, 0));
    let waiting_room = std::sync::Arc::new(CardLeaseWaitingRoom::with_no_lease_out());
    let held_by_this_thread = match card.attempt_to_lease(&waiting_room, 1) {
        LeaseAttempt::Leased(lease) => lease,
        LeaseAttempt::NoRoomNow { reason } | LeaseAttempt::LargerThanEveryCard { reason } => {
            panic!("空着的假卡借得到一份：{reason}")
        }
    };
    let (outcome_sender, outcome_receiver) = std::sync::mpsc::channel();
    let asking_thread = {
        let card = std::sync::Arc::clone(&card);
        let waiting_room = std::sync::Arc::clone(&waiting_room);
        std::thread::spawn(move || {
            let outcome = match waiting_room
                .lease_waiting_for_the_other_threads(|| card.attempt_to_lease(&waiting_room, 3))
            {
                LeaseAfterWaiting::Leased(_lease) => "借到了".to_string(),
                LeaseAfterWaiting::NoRoomAndNothingToWaitFor { reason } => {
                    format!("此刻借不到：{reason}")
                }
                LeaseAfterWaiting::LargerThanEveryCard { reason } => reason,
            };
            outcome_sender
                .send(outcome)
                .expect("收的一头等到交回或到点才丢");
        })
    };
    let outcome = outcome_receiver
        .recv_timeout(std::time::Duration::from_secs(
            LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS,
        ))
        .unwrap_or_else(|error| {
            panic!("要三份的那一条 {LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS} 秒没交回（{error}）：在等一张永远放不下它的卡")
        });
    drop(held_by_this_thread);
    asking_thread.join().expect("要三份的那条线程不 panic");
    assert_eq!(
        outcome, "要借 3 份，假卡空着时也只借得出 2 份",
        "当场交回「比每张卡都大」，原因原样带着要多少、借得出多少"
    );
}

/// 假卡被别的进程占满（不经排队、不会还），这个进程里一笔都没借出去、也没有别的线程在借：排队不等，当场交回「没有可等的」
/// （卡池照它报本次未跑，同没排队时一样）。不要卡。
#[test]
fn lease_without_room_while_no_other_thread_holds_or_asks_is_answered_at_once_with_nothing_to_wait_for(
) {
    let card =
        std::sync::Arc::new(FakeCardOfShares::with_capacity_and_held_outside_this_process(2, 2));
    let waiting_room = std::sync::Arc::new(CardLeaseWaitingRoom::with_no_lease_out());
    let (outcome_sender, outcome_receiver) = std::sync::mpsc::channel();
    let asking_thread = {
        let card = std::sync::Arc::clone(&card);
        let waiting_room = std::sync::Arc::clone(&waiting_room);
        std::thread::spawn(move || {
            let outcome = match waiting_room
                .lease_waiting_for_the_other_threads(|| card.attempt_to_lease(&waiting_room, 1))
            {
                LeaseAfterWaiting::Leased(_lease) => "借到了".to_string(),
                LeaseAfterWaiting::NoRoomAndNothingToWaitFor { reason } => reason,
                LeaseAfterWaiting::LargerThanEveryCard { reason } => {
                    format!("比每张卡都大：{reason}")
                }
            };
            outcome_sender
                .send(outcome)
                .expect("收的一头等到交回或到点才丢");
        })
    };
    let outcome = outcome_receiver
        .recv_timeout(std::time::Duration::from_secs(
            LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS,
        ))
        .unwrap_or_else(|error| {
            panic!("别的进程占满的卡上借一份 {LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS} 秒没交回（{error}）：在等一笔这个进程里没有的借据")
        });
    asking_thread.join().expect("借一份的那条线程不 panic");
    assert_eq!(
        outcome, "假卡上此刻借出去 2 份，放不下 1 份",
        "当场交回「没有可等的」，原因原样是那一次借不到的原因"
    );
    assert_eq!(
        card.shares_now().attempts_without_room,
        1,
        "只借了一次，没有转圈再借"
    );
}

/// 一条线程借着一份、又要一份，此刻借不到（假卡只放得下两份，另一份在别的线程手里、到这条线程交回之后才还）：
/// 这条线程自己借着卡就不等（等它自己还的那一笔等不到头），当场交回「没有可等的」。不要卡。
#[test]
fn thread_holding_a_lease_that_asks_for_another_without_room_is_not_made_to_wait() {
    let card =
        std::sync::Arc::new(FakeCardOfShares::with_capacity_and_held_outside_this_process(2, 0));
    let waiting_room = std::sync::Arc::new(CardLeaseWaitingRoom::with_no_lease_out());
    let held_by_this_thread = match card.attempt_to_lease(&waiting_room, 1) {
        LeaseAttempt::Leased(lease) => lease,
        LeaseAttempt::NoRoomNow { reason } | LeaseAttempt::LargerThanEveryCard { reason } => {
            panic!("空着的假卡借得到一份：{reason}")
        }
    };
    let (outcome_sender, outcome_receiver) = std::sync::mpsc::channel();
    let asking_thread = {
        let card = std::sync::Arc::clone(&card);
        let waiting_room = std::sync::Arc::clone(&waiting_room);
        std::thread::spawn(move || {
            let held_by_the_asking_thread = match card.attempt_to_lease(&waiting_room, 1) {
                LeaseAttempt::Leased(lease) => lease,
                LeaseAttempt::NoRoomNow { reason }
                | LeaseAttempt::LargerThanEveryCard { reason } => {
                    panic!("借出去一份的假卡上还借得到一份：{reason}")
                }
            };
            let outcome = match waiting_room
                .lease_waiting_for_the_other_threads(|| card.attempt_to_lease(&waiting_room, 1))
            {
                LeaseAfterWaiting::Leased(_lease) => "借到了".to_string(),
                LeaseAfterWaiting::NoRoomAndNothingToWaitFor { reason } => reason,
                LeaseAfterWaiting::LargerThanEveryCard { reason } => {
                    format!("比每张卡都大：{reason}")
                }
            };
            drop(held_by_the_asking_thread);
            outcome_sender
                .send(outcome)
                .expect("收的一头等到交回或到点才丢");
        })
    };
    let outcome = outcome_receiver
        .recv_timeout(std::time::Duration::from_secs(
            LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS,
        ))
        .unwrap_or_else(|error| {
            panic!("借着一份再要一份的那一条 {LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS} 秒没交回（{error}）：借着卡还在排队")
        });
    drop(held_by_this_thread);
    asking_thread
        .join()
        .expect("借着一份再要一份的那条线程不 panic");
    assert_eq!(
        outcome, "假卡上此刻借出去 2 份，放不下 1 份",
        "借着卡的线程再借不等，当场交回「没有可等的」"
    );
}

/// 假卡借一次、结局照卡池的拒绝类型交（[`attempt_keeping_the_refusal_of`] 收的那一种）：此刻借不到交 `refusal_without_room_now`。
fn fake_lease_or_the_refusal_of_the_card_pool<'test>(
    card: &'test FakeCardOfShares,
    waiting_room: &'test CardLeaseWaitingRoom,
    refusal_without_room_now: &CardPoolLeaseRefusal,
) -> Result<FakeLeaseOfShares<'test>, CardPoolLeaseRefusal> {
    match card.attempt_to_lease(waiting_room, 1) {
        LeaseAttempt::Leased(lease) => Ok(lease),
        LeaseAttempt::NoRoomNow { reason: _ } => Err(refusal_without_room_now.clone()),
        LeaseAttempt::LargerThanEveryCard { reason } => {
            panic!("只借一份，假卡至少放得下一份：{reason}")
        }
    }
}

/// 假卡只放得下一份、在这条线程手里；另一条线程照卡池「排队、拒绝留原样」那一路借一份，头一次的拒绝是 `refusal_of_the_first_attempt`。
/// 等它借不到过一回（或交回）之后这条线程还卡；交回它的结局：借到了是 `Ok`，没借到是那一次的拒绝。
fn outcome_of_a_thread_asking_through_the_pool_while_this_thread_holds_the_only_share(
    refusal_of_the_first_attempt: CardPoolLeaseRefusal,
) -> Result<(), CardPoolLeaseRefusal> {
    let card =
        std::sync::Arc::new(FakeCardOfShares::with_capacity_and_held_outside_this_process(1, 0));
    let waiting_room = std::sync::Arc::new(CardLeaseWaitingRoom::with_no_lease_out());
    let held_by_this_thread = match card.attempt_to_lease(&waiting_room, 1) {
        LeaseAttempt::Leased(lease) => lease,
        LeaseAttempt::NoRoomNow { reason } | LeaseAttempt::LargerThanEveryCard { reason } => {
            panic!("空着的假卡借得到一份：{reason}")
        }
    };
    let (outcome_sender, outcome_receiver) = std::sync::mpsc::channel();
    let asking_thread = {
        let card = std::sync::Arc::clone(&card);
        let waiting_room = std::sync::Arc::clone(&waiting_room);
        std::thread::spawn(move || {
            let outcome = waiting_room
                .lease_waiting_for_the_other_threads_keeping_the_refusal(|| {
                    attempt_keeping_the_refusal_of(fake_lease_or_the_refusal_of_the_card_pool(
                        &card,
                        &waiting_room,
                        &refusal_of_the_first_attempt,
                    ))
                })
                .and_then(|settled| settled)
                .map(drop);
            outcome_sender
                .send(outcome)
                .expect("收的一头等到交回或到点才丢");
        })
    };
    let asking_started = std::time::Instant::now();
    let mut outcome_before_the_share_was_returned = None;
    while card.shares_now().attempts_without_room == 0
        && outcome_before_the_share_was_returned.is_none()
        && asking_started.elapsed()
            < std::time::Duration::from_secs(LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS)
    {
        outcome_before_the_share_was_returned = outcome_receiver
            .recv_timeout(std::time::Duration::from_millis(1))
            .ok();
    }
    drop(held_by_this_thread);
    let outcome = outcome_before_the_share_was_returned.unwrap_or_else(|| {
        outcome_receiver
            .recv_timeout(std::time::Duration::from_secs(
                LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS,
            ))
            .unwrap_or_else(|error| {
                panic!("还了卡之后 {LONGEST_WAIT_FOR_THE_FAKE_TESTS_SECONDS} 秒没交回（{error}）：排队的没被叫醒")
            })
    });
    asking_thread.join().expect("借一份的那条线程不 panic");
    outcome
}

/// 给定状态表的判器那一路（[`GpuCardPool::lease_the_first_card_with_room_waiting_for_the_other_threads`]）：账上此刻每张卡都放不下，
/// 而占着的是这个进程别的线程，等它还了卡再借、借得到，不把那一次的「此刻借不到」交回去（交回去调用方报显存不够）。不要卡。
#[test]
fn lease_keeping_the_refusal_waits_for_another_thread_of_this_process_when_no_card_has_room_now() {
    assert_eq!(
        outcome_of_a_thread_asking_through_the_pool_while_this_thread_holds_the_only_share(
            CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 1,
                card_rooms_mebibytes: vec![0],
            })
        ),
        Ok(()),
        "这个进程别的线程借着的那一份还了就借得到：等，不交「此刻借不到」"
    );
}

/// 同上，头一次是借到的卡上起设备时显存不够（那一笔已经还了）：同样等这个进程别的线程还卡再借。不要卡。
#[test]
fn lease_keeping_the_refusal_waits_for_another_thread_of_this_process_when_the_device_start_ran_out_of_memory(
) {
    assert_eq!(
        outcome_of_a_thread_asking_through_the_pool_while_this_thread_holds_the_only_share(
            CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
                reason: "显存不够：造的".to_string(),
            }
        ),
        Ok(()),
        "起设备时显存不够同此刻借不到：等别的线程还卡"
    );
}

/// 卡上的显存读不出：等多久都一样，别的线程借着卡也不等，当场把那一次的拒绝原样交回（调用方判红）。不要卡。
#[test]
fn lease_keeping_the_refusal_hands_back_unreadable_card_memory_at_once_without_waiting() {
    let unreadable = CardPoolLeaseRefusal::Booking(BookingRefusal::CardMemoryUnreadable {
        reason: "造的读不出".to_string(),
    });
    assert_eq!(
        outcome_of_a_thread_asking_through_the_pool_while_this_thread_holds_the_only_share(
            unreadable.clone()
        ),
        Err(unreadable),
        "读不出不是此刻借不到：不等，拒绝原样交回"
    );
}

/// 要的比每张卡空着时能借的都大：等多久都放不下，别的线程借着卡也不等，当场把那一次的拒绝原样交回。不要卡。
#[test]
fn lease_keeping_the_refusal_hands_back_a_need_larger_than_every_card_at_once_without_waiting() {
    let larger = CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
        NeedLargerThanEveryCard {
            need_mebibytes: 3,
            largest_ceiling_mebibytes: 2,
        },
    ));
    assert_eq!(
        outcome_of_a_thread_asking_through_the_pool_while_this_thread_holds_the_only_share(
            larger.clone()
        ),
        Err(larger),
        "比每张卡都大不是此刻借不到：不等，拒绝原样交回"
    );
}
