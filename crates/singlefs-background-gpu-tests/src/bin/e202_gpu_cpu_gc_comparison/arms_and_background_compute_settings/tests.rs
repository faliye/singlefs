use super::*;

fn two_discrete_gpus() -> Vec<DiscreteGpuSeenByThisProcess> {
    vec![
        DiscreteGpuSeenByThisProcess {
            pci_bus_address: "0000:01:00.0".to_string(),
            model: "lower".to_string(),
            driver: "driver".to_string(),
            driver_information: "driver information".to_string(),
        },
        DiscreteGpuSeenByThisProcess {
            pci_bus_address: "0000:02:00.0".to_string(),
            model: "higher".to_string(),
            driver: "driver".to_string(),
            driver_information: "driver information".to_string(),
        },
    ]
}

/// 第九节变异「全部卡臂也只点一张」在这里红。
#[test]
fn the_every_card_arm_names_every_handed_over_card_and_gives_one_limit_per_card() {
    let settings = background_compute_settings_of_the_arm(
        ComputeOfTheArm::GpuOnEveryCard,
        &two_discrete_gpus(),
    );
    assert_eq!(settings.background_compute, BackgroundComputeBackend::Gpu);
    assert_eq!(
        settings.background_gpu_devices,
        Some(vec![
            gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
            gpu_device_identifier_of_the_pci_bus_address("0000:02:00.0"),
        ])
    );
    assert_eq!(
        settings.background_gpu_memory_limit,
        Some(vec![GpuMemoryInBytes(1 << 30); 2])
    );
}

#[test]
fn the_one_card_arm_names_the_card_with_the_lowest_pci_address() {
    let settings = background_compute_settings_of_the_arm(
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        &two_discrete_gpus(),
    );
    assert_eq!(
        settings.background_gpu_devices,
        Some(vec![gpu_device_identifier_of_the_pci_bus_address(
            "0000:01:00.0"
        )])
    );
    assert_eq!(
        settings.background_gpu_memory_limit,
        Some(vec![GpuMemoryInBytes(1 << 30)])
    );
    assert_eq!(
        background_compute_settings_of_the_arm(ComputeOfTheArm::Cpu, &[]),
        BackgroundComputeSettings::PRODUCT
    );
}

#[test]
fn arms_needing_more_cards_than_seen_are_not_run() {
    assert_eq!(reason_the_arm_is_not_run(ComputeOfTheArm::Cpu, 0), None);
    assert!(
        reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress, 0)
            .is_some()
    );
    assert_eq!(
        reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress, 1),
        None
    );
    assert!(reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnEveryCard, 1).is_some());
    assert_eq!(
        reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnEveryCard, 2),
        None
    );
}
