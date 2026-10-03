//! K2-G1：GPU 对拍取卡判定（合入的 tests/common_gpu_adapter/mod.rs，原样经 #[path] 引进来）在几种枚举结果上的结局，
//! 外加本机 PCI 上厂商号 0x10de、class 0x03xxxx 的显示控制器个数（C620 要的判别量），与这一趟进程里 Vulkan 真枚举到的。
#[path = "../../../tree/crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs"]
mod common_gpu_adapter;

use common_gpu_adapter::{decide_gpu_test_start, AdapterDeviceKind, EnumeratedAdapter, NVIDIA_VENDOR_IDENTIFIER};

fn nvidia_display_controllers_on_pci() -> usize {
    let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") else { return 0 };
    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            let read = |name: &str| std::fs::read_to_string(entry.path().join(name)).unwrap_or_default();
            read("vendor").trim() == "0x10de" && read("class").trim().starts_with("0x03")
        })
        .count()
}

fn main() {
    let adapter = |vendor_identifier, device_kind| EnumeratedAdapter { vendor_identifier, device_kind };
    let llvmpipe = adapter(0x10005, AdapterDeviceKind::SoftwareRenderingOnTheCpu);
    let cases: Vec<(&str, Vec<EnumeratedAdapter>)> = vec![
        ("vulkan_zero_adapters", vec![]),
        ("vulkan_only_llvmpipe", vec![llvmpipe]),
        ("nvidia_reported_as_virtual_gpu", vec![adapter(NVIDIA_VENDOR_IDENTIFIER, AdapterDeviceKind::VirtualGpu)]),
        ("nvidia_reported_as_other", vec![adapter(NVIDIA_VENDOR_IDENTIFIER, AdapterDeviceKind::OtherOrUnknown)]),
        ("nvidia_integrated", vec![adapter(NVIDIA_VENDOR_IDENTIFIER, AdapterDeviceKind::IntegratedGpu)]),
        ("amd_discrete_only", vec![adapter(0x1002, AdapterDeviceKind::DiscreteGpu)]),
        ("intel_integrated_only", vec![adapter(0x8086, AdapterDeviceKind::IntegratedGpu)]),
        ("nvidia_discrete_no_card_started", vec![adapter(NVIDIA_VENDOR_IDENTIFIER, AdapterDeviceKind::DiscreteGpu)]),
    ];
    for (name, adapters) in &cases {
        let decision = decide_gpu_test_start(adapters, 0);
        let kind = format!("{decision:?}");
        println!("K2G1 name=synthetic case={name} usable=0 decision={}", kind.split([' ', '{']).next().unwrap_or(""));
    }
    let pci = nvidia_display_controllers_on_pci();
    let adapters = common_gpu_adapter::enumerate_vulkan_adapters();
    let usable = if std::env::var_os("K2_SKIP_CONTEXTS").is_some() {
        0
    } else {
        singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards().len()
    };
    let decision = decide_gpu_test_start(&adapters, usable);
    println!(
        "K2G1 name=live vk_icd_filenames={:?} vk_driver_files={:?} pci_nvidia_display_controllers={pci} vulkan_adapters={adapters:?} usable_cards={usable} decision={decision:?}",
        std::env::var("VK_ICD_FILENAMES").ok(), std::env::var("VK_DRIVER_FILES").ok()
    );
}
