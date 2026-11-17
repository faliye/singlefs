// task-server-gpu-cards: 1
//! 攻方模型（只在副本里）：同一个 Vulkan 实例枚举时跟不跟得上驱动清单的变化。只建实例、只枚举，不开设备。
//! 藏驱动的写法同 compaction 那个真卡目标的 `with_every_vulkan_driver_hidden_a_mount_choosing_gpu_is_refused_before_any_disk_byte_changes`。
//! 跑前写下的翻面条件：同一个实例在驱动清单变了之后枚举结果跟着变，就撤回「进程级实例看不到修好之后的卡」这一条。

use std::ffi::OsString;
use std::time::Duration;

use singlefs_background_gpu::background_gpu::VulkanInstanceOfThisProcess;

const WAIT_LIMIT: Duration = Duration::from_secs(60);
const DRIVER_LIST_VARIABLES: [&str; 2] = ["VK_DRIVER_FILES", "VK_ICD_FILENAMES"];

fn discrete_gpus_seen(instance: &VulkanInstanceOfThisProcess) -> usize {
    instance
        .vulkan_adapters_enumerated_within(WAIT_LIMIT)
        .expect("枚举在上限之内交回")
        .iter()
        .filter(|adapter| adapter.get_info().device_type == wgpu::DeviceType::DiscreteGpu)
        .count()
}

fn hide_every_driver() {
    let driver_list_that_does_not_exist = std::env::temp_dir().join(format!(
        "singlefs-attack-model-without-a-vulkan-driver-{}.json",
        std::process::id()
    ));
    assert!(!driver_list_that_does_not_exist.exists());
    for variable in DRIVER_LIST_VARIABLES {
        std::env::set_var(variable, driver_list_that_does_not_exist.as_os_str());
    }
    std::env::remove_var("VK_ADD_DRIVER_FILES");
}

fn put_back(saved: &[(&str, Option<OsString>)]) {
    for (variable, value) in saved {
        match value {
            Some(value) => std::env::set_var(variable, value),
            None => std::env::remove_var(variable),
        }
    }
}

#[test]
fn the_driver_list_is_read_when_the_instance_is_created_and_not_when_it_enumerates() {
    let saved: Vec<(&str, Option<OsString>)> = ["VK_DRIVER_FILES", "VK_ICD_FILENAMES", "VK_ADD_DRIVER_FILES"]
        .into_iter()
        .map(|variable| (variable, std::env::var_os(variable)))
        .collect();

    hide_every_driver();
    let created_while_hidden = VulkanInstanceOfThisProcess::created_for_this_process();
    let hidden_at_creation = discrete_gpus_seen(&created_while_hidden);
    put_back(&saved);
    let same_instance_after_the_drivers_came_back = discrete_gpus_seen(&created_while_hidden);
    let created_after_they_came_back = VulkanInstanceOfThisProcess::created_for_this_process();
    let new_instance_after_they_came_back = discrete_gpus_seen(&created_after_they_came_back);

    hide_every_driver();
    let same_visible_instance_after_hiding = discrete_gpus_seen(&created_after_they_came_back);
    let created_after_hiding = VulkanInstanceOfThisProcess::created_for_this_process();
    let new_instance_after_hiding = discrete_gpus_seen(&created_after_hiding);
    put_back(&saved);

    let mapped = std::fs::read_to_string("/proc/self/maps").expect("Linux 上读得到");
    let lines_naming_the_loader = mapped.lines().filter(|line| line.contains("libvulkan")).count();
    println!(
        "VIEW hidden_at_creation={hidden_at_creation} same_instance_after_the_drivers_came_back={same_instance_after_the_drivers_came_back} new_instance_after_they_came_back={new_instance_after_they_came_back} same_visible_instance_after_hiding={same_visible_instance_after_hiding} new_instance_after_hiding={new_instance_after_hiding} lines_naming_libvulkan={lines_naming_the_loader}"
    );
    assert_eq!(hidden_at_creation, 0, "藏起来时建的实例一张独显都看不到");
    assert!(new_instance_after_they_came_back >= 1, "驱动回来之后新建的实例看得到独显（要卡）");
    assert_eq!(new_instance_after_hiding, 0, "藏起来之后新建的实例看不到");
    assert!(lines_naming_the_loader >= 1, "阳性对照：开了 Vulkan 后端的实例映射了加载器");
    assert_eq!(same_instance_after_the_drivers_came_back, 0, "打中：驱动回来之后同一个实例还是一张都看不到");
    assert_eq!(same_visible_instance_after_hiding, new_instance_after_they_came_back, "打中：藏起来之后同一个实例还报原来那几张");
}
