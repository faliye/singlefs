//! J2 的 G1：改后的取卡判定（`gpu_unit_checks::gpu_test_start`）在几种机器上的结局。PCI 设备表用假的 sysfs 目录喂
//! `read_pci_nvidia_display_controllers_under`（与用例走同一个读法），Vulkan 枚举与起好的卡数按场景给，交 `decide_gpu_test_start`；
//! 末尾三行是这台机器上真枚举、真读 /sys/bus/pci/devices、真起卡的结局（R2_LIVE=1 时才跑）。
use std::path::{Path, PathBuf};

use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    decide_gpu_test_start, enumerate_vulkan_adapters, read_pci_nvidia_display_controllers_under,
    AdapterDeviceKind, EnumeratedAdapter, PCI_DEVICES_DIRECTORY,
};

fn fake_sysfs(root: &Path, name: &str, devices: &[(&str, Option<&str>, Option<&str>)]) -> PathBuf {
    let directory = root.join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("建得了");
    for (slot, vendor, class) in devices {
        let device = directory.join(slot);
        std::fs::create_dir_all(&device).expect("建得了");
        if let Some(vendor) = vendor {
            std::fs::write(device.join("vendor"), format!("{vendor}\n")).expect("写得了");
        }
        if let Some(class) = class {
            std::fs::write(device.join("class"), format!("{class}\n")).expect("写得了");
        }
    }
    directory
}

fn adapter(vendor: u32, kind: AdapterDeviceKind) -> EnumeratedAdapter {
    EnumeratedAdapter { vendor_identifier: vendor, device_kind: kind }
}

fn main() {
    let root = PathBuf::from(std::env::var("R2_SCRATCH").expect("要给草稿目录 R2_SCRATCH")).join("fake-sysfs");
    let missing = root.join("no-such-directory");
    let empty = fake_sysfs(&root, "empty", &[]);
    let one_nvidia = fake_sysfs(&root, "one-nvidia", &[("0000:01:00.0", Some("0x10de"), Some("0x030000")), ("0000:01:00.1", Some("0x10de"), Some("0x040300"))]);
    let two_nvidia = fake_sysfs(&root, "two-nvidia", &[("0000:01:00.0", Some("0x10de"), Some("0x030000")), ("0000:02:00.0", Some("0x10de"), Some("0x030000"))]);
    let nvidia_3d = fake_sysfs(&root, "nvidia-3d-controller", &[("0000:01:00.0", Some("0x10de"), Some("0x030200"))]);
    let virtio_gpu = fake_sysfs(&root, "virtio-gpu-only", &[("0000:00:02.0", Some("0x1af4"), Some("0x030000"))]);
    let one_device_without_class = fake_sysfs(&root, "intel-host-bridge-without-class", &[("0000:00:00.0", Some("0x8086"), None)]);
    let llvmpipe = adapter(0x10005, AdapterDeviceKind::SoftwareRenderingOnTheCpu);
    let nvidia_discrete = adapter(0x10de, AdapterDeviceKind::DiscreteGpu);
    let nvidia_virtual = adapter(0x10de, AdapterDeviceKind::VirtualGpu);
    let virtio_virtual = adapter(0x1af4, AdapterDeviceKind::VirtualGpu);
    let scenarios: Vec<(&str, Vec<EnumeratedAdapter>, &Path, usize)> = vec![
        ("vm_or_container_without_sys_bus_pci_and_no_gpu", vec![llvmpipe], &missing, 0),
        ("container_whose_sysfs_lists_no_pci_device", vec![llvmpipe], &empty, 0),
        ("vm_with_virtio_gpu_only", vec![virtio_virtual, llvmpipe], &virtio_gpu, 0),
        ("no_gpu_host_with_one_pci_device_whose_class_is_unreadable", vec![llvmpipe], &one_device_without_class, 0),
        ("container_on_a_gpu_host_without_the_nvidia_icd", vec![llvmpipe], &two_nvidia, 0),
        ("vm_guest_whose_nvidia_vgpu_vulkan_reports_virtual", vec![nvidia_virtual], &one_nvidia, 0),
        ("datacenter_card_class_3d_controller_driver_missing", vec![], &nvidia_3d, 0),
        ("two_cards_on_pci_one_in_vulkan_one_started", vec![nvidia_discrete], &two_nvidia, 1),
        ("two_cards_on_pci_two_in_vulkan_one_started", vec![nvidia_discrete, nvidia_discrete], &two_nvidia, 1),
        ("one_card_healthy", vec![nvidia_discrete, llvmpipe], &one_nvidia, 1),
    ];
    for (name, adapters, directory, usable) in scenarios {
        let pci = read_pci_nvidia_display_controllers_under(directory);
        let decision = decide_gpu_test_start(&adapters, &pci, usable);
        println!("G1 scenario={name} pci={pci:?} usable_cards={usable} decision={decision:?}");
    }
    if std::env::var("R2_LIVE").is_ok_and(|value| value == "1") {
        let adapters = enumerate_vulkan_adapters();
        let pci = read_pci_nvidia_display_controllers_under(Path::new(PCI_DEVICES_DIRECTORY));
        let usable = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards().len();
        let decision = decide_gpu_test_start(&adapters, &pci, usable);
        println!(
            "G1LIVE vk_icd_filenames={:?} adapters={adapters:?} pci={pci:?} usable_cards={usable} decision={decision:?}",
            std::env::var("VK_ICD_FILENAMES").ok()
        );
    }
}
