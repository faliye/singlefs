//! checker 档模块：gpu_unit_checks
//! GPU 对拍用例开跑前的取卡判定（`gpu_unit_checks::gpu_test_start::decide_gpu_test_start`）每一种结局各一条：起好了卡照跑；
//! Vulkan 里没有 NVIDIA 独显、PCI 上也没有 NVIDIA 显示控制器，报「本次未跑」；Vulkan 枚举出 NVIDIA 独显而一张卡都没起来，判红；
//! PCI 上有 NVIDIA 显示控制器而 Vulkan 枚举不出（驱动坏了，C620），判红；Vulkan 里没有、PCI 总线看得见而设备表读不出（分不清没卡与驱动坏），判红。
//! 再加读 PCI 那一层：只数 NVIDIA 的显示控制器（不数同一块卡的音频功能、不数别家的显卡），设备表里一项读不出就算读不出。
//! 判定是纯函数、读 PCI 用假的设备目录，不要显卡；模块挂 `gpu` 特性，跟着快档带特性那一趟跑。
#![cfg(feature = "gpu")]

use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    count_nvidia_display_controllers, decide_gpu_test_start, not_run_line,
    read_pci_nvidia_display_controllers_under, AdapterDeviceKind, EnumeratedAdapter, GpuTestStart,
    PciDeviceIdentity, PciNvidiaDisplayControllers,
};
use singlefs_checker_tier::gpu_unit_checks::NVIDIA_VENDOR_IDENTIFIER;

const NO_NVIDIA_DISPLAY_CONTROLLER_ON_PCI: PciNvidiaDisplayControllers =
    PciNvidiaDisplayControllers::Counted {
        nvidia_display_controller_count: 0,
    };

fn nvidia_discrete_gpu() -> EnumeratedAdapter {
    EnumeratedAdapter {
        vendor_identifier: NVIDIA_VENDOR_IDENTIFIER,
        device_kind: AdapterDeviceKind::DiscreteGpu,
    }
}

/// 一棵假的 sysfs 设备目录：每个设备一个子目录，`vendor` 与 `class` 照写；`None` 的那个文件不建。用完由调用方删。
fn fake_pci_devices_directory(
    test_name: &str,
    devices: &[(&str, Option<&str>, Option<&str>)],
) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "singlefs-gpu-test-start-pci-{}-{test_name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (address, vendor_text, class_text) in devices {
        let device_directory = directory.join(address);
        std::fs::create_dir_all(&device_directory).expect("临时目录建得出");
        if let Some(vendor_text) = vendor_text {
            std::fs::write(device_directory.join("vendor"), vendor_text).expect("写得进临时目录");
        }
        if let Some(class_text) = class_text {
            std::fs::write(device_directory.join("class"), class_text).expect("写得进临时目录");
        }
    }
    directory
}

#[test]
fn zero_adapters_and_no_nvidia_display_controller_on_pci_is_reported_as_not_run() {
    match decide_gpu_test_start(&[], &NO_NVIDIA_DISPLAY_CONTROLLER_ON_PCI, 0) {
        GpuTestStart::NotRunNoGpuAdapter { reason } => {
            let line = not_run_line("some_gpu_parity_test", &reason);
            assert!(
                line.starts_with("NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason="),
                "门禁按这一行的开头认「本次未跑」：{line}"
            );
        }
        start @ (GpuTestStart::RunOnTheUsableCards
        | GpuTestStart::AdapterPresentButNoCardStarted { .. }
        | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { .. }
        | GpuTestStart::PciBusVisibleButListingUnreadable { .. }) => {
            panic!("Vulkan 零个适配器、PCI 上也没有 NVIDIA 显示控制器要报「本次未跑」，判成了 {start:?}")
        }
    }
}

#[test]
fn an_nvidia_discrete_adapter_whose_card_did_not_start_is_red() {
    match decide_gpu_test_start(
        &[nvidia_discrete_gpu()],
        &PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: 1,
        },
        0,
    ) {
        GpuTestStart::AdapterPresentButNoCardStarted { .. } => {}
        start @ (GpuTestStart::RunOnTheUsableCards
        | GpuTestStart::NotRunNoGpuAdapter { .. }
        | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { .. }
        | GpuTestStart::PciBusVisibleButListingUnreadable { .. }) => {
            panic!("有 NVIDIA 独显适配器而一张卡都没起来要判红，不算「没卡」；判成了 {start:?}")
        }
    }
}

#[test]
fn a_card_that_started_runs_the_test() {
    let start = decide_gpu_test_start(
        &[nvidia_discrete_gpu()],
        &PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: 1,
        },
        1,
    );
    assert_eq!(
        start,
        GpuTestStart::RunOnTheUsableCards,
        "起好了一张卡就照跑"
    );
}

#[test]
fn an_nvidia_display_controller_on_pci_that_vulkan_does_not_enumerate_is_red() {
    match decide_gpu_test_start(
        &[],
        &PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: 2,
        },
        0,
    ) {
        GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { reason } => {
            assert!(
                reason.ends_with("驱动坏了：修驱动，或改配置关 GPU"),
                "判红的出路要写修驱动或改配置：{reason}"
            );
        }
        start @ (GpuTestStart::RunOnTheUsableCards
        | GpuTestStart::NotRunNoGpuAdapter { .. }
        | GpuTestStart::AdapterPresentButNoCardStarted { .. }
        | GpuTestStart::PciBusVisibleButListingUnreadable { .. }) => {
            panic!("PCI 上有 NVIDIA 显示控制器、Vulkan 零个适配器是驱动坏了，要判红、不算「没卡」；判成了 {start:?}")
        }
    }
}

#[test]
fn an_nvidia_display_controller_on_pci_with_only_a_software_renderer_in_vulkan_is_red() {
    /// Mesa 的软件渲染（llvmpipe）在 Vulkan 里报的厂商号：NVIDIA 的 Vulkan 驱动不在时枚举里只剩它。
    const MESA_SOFTWARE_RENDERER_VENDOR_IDENTIFIER: u32 = 0x1_0005;
    let software_renderer = EnumeratedAdapter {
        vendor_identifier: MESA_SOFTWARE_RENDERER_VENDOR_IDENTIFIER,
        device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
    };
    match decide_gpu_test_start(
        &[software_renderer],
        &PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: 2,
        },
        0,
    ) {
        GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { .. } => {}
        start @ (GpuTestStart::RunOnTheUsableCards
        | GpuTestStart::NotRunNoGpuAdapter { .. }
        | GpuTestStart::AdapterPresentButNoCardStarted { .. }
        | GpuTestStart::PciBusVisibleButListingUnreadable { .. }) => {
            panic!("Vulkan 只剩软件渲染（NVIDIA 的 Vulkan 驱动不在）而 PCI 上有卡，要判红；判成了 {start:?}")
        }
    }
}

#[test]
fn no_nvidia_adapter_and_an_unreadable_pci_listing_on_a_visible_bus_is_red() {
    let start = decide_gpu_test_start(
        &[],
        &PciNvidiaDisplayControllers::ListingUnreadable {
            reason: "假的：设备目录不在".to_string(),
        },
        0,
    );
    assert!(
        matches!(
            start,
            GpuTestStart::PciBusVisibleButListingUnreadable { .. }
        ),
        "Vulkan 里没有 NVIDIA 独显、PCI 又读不出时分不清没卡与驱动坏，不许报「本次未跑」；判成了 {start:?}"
    );
}

#[test]
fn the_audio_function_of_an_nvidia_card_and_another_vendors_display_controller_are_not_counted() {
    let devices = [
        ("0x10de\n", "0x030000\n"),
        ("0x10de\n", "0x040300\n"),
        ("0x8086\n", "0x030000\n"),
    ]
    .map(|(vendor_text, class_text)| {
        PciDeviceIdentity::from_sysfs_text(vendor_text, class_text).expect("sysfs 原文认得出")
    });
    assert_eq!(
        count_nvidia_display_controllers(&devices),
        1,
        "只数 NVIDIA 的显示控制器（class 基类 0x03），不数同一块卡的 HDA 音频功能（0x04）、不数别家的显卡：{devices:?}"
    );
}

#[test]
fn a_fake_pci_devices_directory_with_two_nvidia_cards_counts_their_two_display_controllers() {
    let directory = fake_pci_devices_directory(
        "two-cards",
        &[
            ("0000:01:00.0", Some("0x10de\n"), Some("0x030000\n")),
            ("0000:01:00.1", Some("0x10de\n"), Some("0x040300\n")),
            ("0000:10:00.0", Some("0x10de\n"), Some("0x030000\n")),
            ("0000:10:00.1", Some("0x10de\n"), Some("0x040300\n")),
            ("0000:00:02.0", Some("0x8086\n"), Some("0x030000\n")),
        ],
    );
    let counted = read_pci_nvidia_display_controllers_under(&directory);
    let _ = std::fs::remove_dir_all(&directory);
    assert_eq!(
        counted,
        PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: 2
        },
        "两块 NVIDIA 卡各一个显示控制器、一个音频功能，再加一张别家的显卡"
    );
}

#[test]
fn a_pci_device_without_a_class_file_makes_the_listing_unreadable() {
    let directory = fake_pci_devices_directory(
        "missing-class",
        &[
            ("0000:01:00.0", Some("0x10de\n"), Some("0x030000\n")),
            ("0000:00:1f.3", Some("0x8086\n"), None),
        ],
    );
    let counted = read_pci_nvidia_display_controllers_under(&directory);
    let _ = std::fs::remove_dir_all(&directory);
    assert!(
        matches!(
            counted,
            PciNvidiaDisplayControllers::ListingUnreadable { .. }
        ),
        "设备表里一项读不出就不算数过了：{counted:?}"
    );
}
