
// 攻方模型：rerun.sh 把这一段接到副本的 background_gpu.rs 末尾（只在副本里），读得到私有的 `instance` 字段与 `without_any_backend`。
#[cfg(test)]
mod attack_tests {
    use super::*;
    use crate::adapter_inventory::gpu_device_identifier_of_the_pci_bus_address;

    const WAIT_LIMIT_OF_THE_TABLE: Duration = Duration::from_secs(5);
    const ADDRESS_OF_THE_CARD: &str = "0000:01:00.0";
    const ADDRESS_OF_THE_OTHER_CARD: &str = "0000:41:00.0";
    const EIGHT_GIBIBYTES_IN_BYTES: u64 = 8 << 30;

    fn adapter(
        device_kind: AdapterDeviceKind,
        pci_bus_address: &str,
        memory_bytes: Option<u64>,
    ) -> EnumeratedAdapterDescription {
        EnumeratedAdapterDescription {
            name: format!("{device_kind:?} at [{pci_bus_address}]"),
            device_kind,
            pci_bus_address: pci_bus_address.to_string(),
            device_local_memory: memory_bytes.map(GpuMemoryInBytes),
        }
    }

    fn software_vulkan() -> EnumeratedAdapterDescription {
        adapter(AdapterDeviceKind::SoftwareRenderingOnTheCpu, "", None)
    }

    fn discrete(pci_bus_address: &str) -> EnumeratedAdapterDescription {
        adapter(
            AdapterDeviceKind::DiscreteGpu,
            pci_bus_address,
            Some(EIGHT_GIBIBYTES_IN_BYTES),
        )
    }

    fn selection(
        named: Option<&[&str]>,
        memory_limits: Option<&[u64]>,
        wait_limit: Duration,
    ) -> BackgroundGpuSelection {
        BackgroundGpuSelection {
            background_gpu_devices: named.map(|addresses| {
                addresses
                    .iter()
                    .map(|address| gpu_device_identifier_of_the_pci_bus_address(address))
                    .collect()
            }),
            background_gpu_memory_limit: memory_limits
                .map(|limits| limits.iter().copied().map(GpuMemoryInBytes).collect()),
            background_gpu_wait_limit: wait_limit,
        }
    }

    fn refusal_of_the_arm(
        arm: &str,
        selection: &BackgroundGpuSelection,
        descriptions: &[EnumeratedAdapterDescription],
    ) -> Option<BackgroundGpuOpenRefused> {
        let refused =
            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(selection, descriptions)
                .err();
        println!("PURE_ARM name={arm} refused={refused:?}");
        refused
    }

    /// 纯函数的每一臂各交回哪个成员（读码读出来的，这条用例把读出来的逐臂钉成断言）。
    #[test]
    fn every_arm_of_the_pure_function_gives_the_member_read_from_the_code() {
        let named_card: &[&str] = &[ADDRESS_OF_THE_CARD];
        let wait = WAIT_LIMIT_OF_THE_TABLE;
        assert!(matches!(
            refusal_of_the_arm("named_absent_only_software", &selection(Some(named_card), None, wait), &[software_vulkan()]),
            Some(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_discrete_without_address", &selection(Some(named_card), None, wait), &[discrete(""), software_vulkan()]),
            Some(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("not_named_only_software", &selection(None, None, wait), &[software_vulkan()]),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::GpuNotPresent))
        ));
        assert!(matches!(
            refusal_of_the_arm("not_named_discrete_without_address", &selection(None, None, wait), &[discrete("")]),
            Some(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_card_without_a_heap", &selection(Some(named_card), None, wait), &[adapter(AdapterDeviceKind::DiscreteGpu, ADDRESS_OF_THE_CARD, None)]),
            Some(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_card_reported_twice", &selection(Some(named_card), None, wait), &[discrete(ADDRESS_OF_THE_CARD), discrete(ADDRESS_OF_THE_CARD)]),
            Some(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("empty_named_list", &selection(Some(&[]), None, wait), &[software_vulkan()]),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_twice", &selection(Some(&[ADDRESS_OF_THE_CARD, ADDRESS_OF_THE_CARD]), None, wait), &[discrete(ADDRESS_OF_THE_CARD)]),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided { .. }))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_integrated", &selection(Some(named_card), None, wait), &[adapter(AdapterDeviceKind::IntegratedGpu, ADDRESS_OF_THE_CARD, None)]),
            Some(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("limit_count_different", &selection(Some(named_card), Some(&[1 << 30, 1 << 30]), wait), &[discrete(ADDRESS_OF_THE_CARD)]),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices { .. }))
        ));
        assert!(matches!(
            refusal_of_the_arm("limit_above_the_total", &selection(Some(named_card), Some(&[EIGHT_GIBIBYTES_IN_BYTES + 1]), wait), &[discrete(ADDRESS_OF_THE_CARD)]),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice { .. }))
        ));
        assert!(matches!(
            refusal_of_the_arm("wait_limit_zero_through_the_pure_function_only", &selection(Some(named_card), None, Duration::ZERO), &[discrete(ADDRESS_OF_THE_CARD)]),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::GpuWaitLimitOfZero))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_the_empty_address", &selection(Some(&[""]), None, wait), &[discrete("")]),
            Some(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("not_named_other_card_without_a_heap", &selection(None, None, wait), &[discrete(ADDRESS_OF_THE_CARD), adapter(AdapterDeviceKind::DiscreteGpu, ADDRESS_OF_THE_OTHER_CARD, None)]),
            Some(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported(_))
        ));
        assert!(matches!(
            refusal_of_the_arm("named_absent_before_the_limit_count", &selection(Some(&[ADDRESS_OF_THE_OTHER_CARD, ADDRESS_OF_THE_CARD]), Some(&[1 << 30]), wait), &[discrete(ADDRESS_OF_THE_CARD)]),
            Some(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(_))
        ));
        assert_eq!(
            refusal_of_the_arm("named_present_other_discrete_without_address", &selection(Some(named_card), None, wait), &[discrete(ADDRESS_OF_THE_CARD), discrete("")]),
            None,
            "点名的那张交得成：没点名的交不成也不拒"
        );
    }

    fn refusal_of_opening_on_the_empty_instance(
        arm: &str,
        selection: &BackgroundGpuSelection,
    ) -> Option<BackgroundGpuOpenRefused> {
        let instance = VulkanInstanceOfThisProcess::without_any_backend();
        let refused = open_background_gpu(selection, &instance).err();
        println!("OPEN_ARM name={arm} refused={refused:?}");
        refused
    }

    /// 枚举为空时走开卡入口拒成哪个成员：不挂住。
    #[test]
    fn an_empty_enumeration_is_refused_as_the_member_of_the_selection() {
        let wait = WAIT_LIMIT_OF_THE_TABLE;
        assert!(matches!(
            refusal_of_opening_on_the_empty_instance("empty_enumeration_not_named", &selection(None, None, wait)),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::GpuNotPresent))
        ));
        let Some(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(named_refusal)) =
            refusal_of_opening_on_the_empty_instance("empty_enumeration_named", &selection(Some(&[ADDRESS_OF_THE_CARD]), None, wait))
        else {
            panic!("点名时枚举为空应当拒成新成员");
        };
        assert!(named_refusal.adapters_in_the_enumeration.is_empty());
        assert!(matches!(
            refusal_of_opening_on_the_empty_instance("empty_enumeration_empty_named_list", &selection(Some(&[]), None, wait)),
            Some(BackgroundGpuOpenRefused::SettingsRefused(BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided))
        ));
    }

    /// 等的上限给 0（配错，核心层的成员是 `GpuWaitLimitOfZero`，出路是改参数）走开卡入口：枚举那一步先判期限，拒成「枚举适配器没成」。
    /// 点名、没点名、空列表三种点法都一样。这条断言打中才绿；核心层的成员真出来，这条就红，打中撤回。
    #[test]
    fn a_wait_limit_of_zero_through_the_opening_is_refused_as_the_enumeration_failing() {
        let refused_as_the_enumeration_failing = |refused: Option<BackgroundGpuOpenRefused>| {
            matches!(
                refused,
                Some(BackgroundGpuOpenRefused::AdapterEnumerationFailed {
                    cause: GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit }
                }) if wait_limit.is_zero()
            )
        };
        assert!(refused_as_the_enumeration_failing(refusal_of_opening_on_the_empty_instance(
            "wait_limit_zero_not_named",
            &selection(None, None, Duration::ZERO)
        )));
        assert!(refused_as_the_enumeration_failing(refusal_of_opening_on_the_empty_instance(
            "wait_limit_zero_named",
            &selection(Some(&[ADDRESS_OF_THE_CARD]), None, Duration::ZERO)
        )));
        assert!(refused_as_the_enumeration_failing(refusal_of_opening_on_the_empty_instance(
            "wait_limit_zero_empty_named_list",
            &selection(Some(&[]), None, Duration::ZERO)
        )));
    }

    /// wgpu 的 `enumerate_adapters` 在造 future 那一步就把枚举做完（wgpu 的 `Instance::enumerate_adapters` 与核心后端交回 `ready`）：
    /// 这一步在 `wait_for_the_wgpu_future` 的参数里求值，期限从它做完才起算。这里用一段睡 300 毫秒的参数代表它：上限 1 毫秒照样交回成功，不报超时。
    #[test]
    fn work_done_while_the_future_is_built_is_not_under_the_wait_limit() {
        let started = std::time::Instant::now();
        let outcome = wait_for_the_wgpu_future(
            {
                std::thread::sleep(Duration::from_millis(300));
                std::future::ready(7_u32)
            },
            Duration::from_millis(1),
            &SteadyClockOfThisProcess::starting_now(),
        );
        let elapsed = started.elapsed();
        println!("FUTURE_BUILT_BEFORE_THE_DEADLINE outcome={outcome:?} elapsed_millis={}", elapsed.as_millis());
        assert_eq!(outcome, Ok(7));
        assert!(elapsed >= Duration::from_millis(300));
    }

    /// 任何机器都能跑的钉法：产品实现交回的就是交进来的那一个实例（`wgpu::Instance` 按 `Arc` 地址比相等）；另建的不等是阳性对照。
    /// 全程不开后端、不装 Vulkan 加载器。
    #[test]
    fn the_product_opening_takes_the_very_instance_handed_over() {
        let handed_over = VulkanInstanceOfThisProcess::without_any_backend();
        let taken = handed_over.vulkan_instance_for_this_opening();
        assert!(
            taken.instance == handed_over.instance,
            "产品交给开卡的实例与交进来的不是同一个"
        );
        let created_again = VulkanInstanceOfThisProcess::without_any_backend();
        assert!(
            created_again.instance != handed_over.instance,
            "阳性对照：另建的实例按 Arc 地址比不相等"
        );
    }

    /// 不开后端的实例走完建、枚举、开卡拒绝，进程里没有映射 Vulkan 加载器。这条在 `attack_tests` 这一批里跑，这一批别的用例也都不开后端。
    #[test]
    fn an_instance_without_any_backend_maps_no_vulkan_loader() {
        let refused = refusal_of_opening_on_the_empty_instance(
            "no_loader_mapped",
            &selection(Some(&[ADDRESS_OF_THE_CARD]), None, WAIT_LIMIT_OF_THE_TABLE),
        );
        assert!(refused.is_some());
        let mapped = std::fs::read_to_string("/proc/self/maps").expect("Linux 上读得到");
        let lines_naming_the_loader = mapped.lines().filter(|line| line.contains("libvulkan")).count();
        println!("LOADER_MAPPED lines_naming_libvulkan={lines_naming_the_loader}");
        assert_eq!(lines_naming_the_loader, 0);
    }
}
