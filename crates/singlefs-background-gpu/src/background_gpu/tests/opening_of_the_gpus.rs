//! 开卡用进程里交来的那一个 Vulkan 实例，枚举不到点名的卡、没有独显、等的上限为 0 时怎么拒。

use super::*;

/// 开卡取实例的替身：交回一个不开任何后端的实例（枚举交回空列表，不碰 GPU），记下开卡取了它几次。
struct VulkanInstanceCountingTheOpenings {
    instance_without_any_backend: VulkanInstanceOfThisProcess,
    openings_that_took_it: Cell<usize>,
}

impl VulkanInstanceHandedToTheOpening for VulkanInstanceCountingTheOpenings {
    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {
        self.openings_that_took_it
            .set(self.openings_that_took_it.get() + 1);
        self.instance_without_any_backend.clone()
    }
}

/// 同一个进程里开卡几次的轮数：开卡每次另建实例时取交来的那一份的次数是 0，与它不等。
const OPENINGS_ON_THE_HANDED_OVER_INSTANCE: usize = 3;

fn selection_naming_the_card_at(pci_bus_address: &str) -> BackgroundGpuSelection {
    BackgroundGpuSelection {
        background_gpu_devices: Some(vec![card_identifier(pci_bus_address)]),
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

/// 同一个进程里反复开卡，每一次都从调用方交来的那一份取实例、不另建：替身记下的次数等于开卡的次数。替身的实例不开后端、枚举是空的，
/// 点名的卡每一次都拒成「不在这一次枚举的清单里」、枚举到的适配器一个都没有（不要卡，也不建 Vulkan 实例）。
#[test]
fn every_opening_in_one_process_takes_the_vulkan_instance_handed_over_instead_of_creating_one() {
    let vulkan_instance = VulkanInstanceCountingTheOpenings {
        instance_without_any_backend: VulkanInstanceOfThisProcess::without_any_backend(),
        openings_that_took_it: Cell::new(0),
    };
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let selection = selection_naming_the_card_at("0000:ff:1f.7");
    for opening_round in 0..OPENINGS_ON_THE_HANDED_OVER_INSTANCE {
        let refused = open_background_gpu_with_the_memory_leaked_in_this_process(
            &selection,
            &memory_leaked,
            &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
            &vulkan_instance,
            |_gpus_to_open| {},
        )
        .err();
        assert_eq!(
                refused,
                Some(
                    BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                        NamedGpuDeviceNotOnTheListOfThisEnumeration {
                            device: card_identifier("0000:ff:1f.7"),
                            adapters_in_the_enumeration: Vec::new(),
                            unnamed_left_off_the_list: Vec::new(),
                        }
                    )
                ),
                "第 {opening_round} 轮（从 0 数）开卡在交来的不开后端的实例上枚举，一个适配器都没有，点名的卡不在清单里"
            );
    }
    assert_eq!(
        vulkan_instance.openings_that_took_it.get(),
        OPENINGS_ON_THE_HANDED_OVER_INSTANCE,
        "开了 {OPENINGS_ON_THE_HANDED_OVER_INSTANCE} 次卡，每一次都取交来的那一份实例（不另建）"
    );
}

/// 点名一张卡开包时的那一次枚举里一张独显都没有（只有软件 Vulkan）。
fn enumeration_with_only_the_software_vulkan() -> Vec<EnumeratedAdapterDescription> {
    vec![EnumeratedAdapterDescription {
        name: "llvmpipe software rasterizer".to_string(),
        device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
        pci_bus_address: String::new(),
        device_local_memory: None,
    }]
}

/// 点名的卡这一次枚举里根本没有：拒绝里列出枚举到的那一个软件 Vulkan，没有一张独显被留在清单外。
#[test]
fn naming_a_card_absent_from_the_enumeration_is_refused_listing_the_adapters_seen_and_none_left_off(
) {
    assert_eq!(
        discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
            &selection_naming_the_card_at("0000:01:00.0"),
            &enumeration_with_only_the_software_vulkan(),
        ),
        Err(
            BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                NamedGpuDeviceNotOnTheListOfThisEnumeration {
                    device: card_identifier("0000:01:00.0"),
                    adapters_in_the_enumeration: vec![AdapterSeenInTheEnumeration {
                        device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
                        pci_bus_address: String::new(),
                    }],
                    unnamed_left_off_the_list: Vec::new(),
                }
            )
        ),
        "枚举里没有这张卡：只列出枚举到的软件 Vulkan，清单外一张都没有"
    );
}

/// 枚举里有一张独显而它没报总线地址：拒绝里列出这张独显（地址是空串）与它被留在清单外的原因，与「枚举里没有」分得开。
#[test]
fn naming_a_card_while_the_discrete_gpu_reports_no_bus_address_is_refused_listing_that_gpu_left_off(
) {
    let mut adapters = vec![EnumeratedAdapterDescription {
        name: "discrete graphics without an address".to_string(),
        device_kind: AdapterDeviceKind::DiscreteGpu,
        pci_bus_address: String::new(),
        device_local_memory: Some(GpuMemoryInBytes(EIGHT_GIBIBYTES)),
    }];
    adapters.extend(enumeration_with_only_the_software_vulkan());
    assert_eq!(
        discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
            &selection_naming_the_card_at("0000:01:00.0"),
            &adapters,
        ),
        Err(
            BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                NamedGpuDeviceNotOnTheListOfThisEnumeration {
                    device: card_identifier("0000:01:00.0"),
                    adapters_in_the_enumeration: vec![
                        AdapterSeenInTheEnumeration {
                            device_kind: AdapterDeviceKind::DiscreteGpu,
                            pci_bus_address: String::new(),
                        },
                        AdapterSeenInTheEnumeration {
                            device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
                            pci_bus_address: String::new(),
                        },
                    ],
                    unnamed_left_off_the_list: vec![
                        DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
                            name: "discrete graphics without an address".to_string(),
                        }
                    ],
                }
            )
        ),
        "有独显而它没报总线地址：列出这张独显（地址空串）与它没收进清单的原因"
    );
}

/// 核心层别的拒照原样交回：一张独显都没有、也没点名时是「GPU 不在」。
#[test]
fn no_discrete_gpu_without_named_cards_is_refused_as_gpu_not_present_through_the_settings() {
    assert_eq!(
        discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
            &BackgroundGpuSelection {
                background_gpu_devices: None,
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            &enumeration_with_only_the_software_vulkan(),
        ),
        Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuNotPresent
        )),
        "没点名、一张独显都没有：照核心层的「GPU 不在」交回"
    );
}

/// 产品对「开卡取实例」的实现交回的是调用方手里的那一个实例，不是另建的一个：两边的 wgpu 实例相等
/// （wgpu 30.0.1 比较两个实例比的是内部那一份 `Arc` 的地址）。用不开后端的实例，不要卡、也不碰 Vulkan。
/// 对照：各自另建的两个实例互不相等，比较分得出是不是同一个。
#[test]
fn the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one() {
    let held = VulkanInstanceOfThisProcess::without_any_backend();
    let handed = held.vulkan_instance_for_this_opening();
    assert!(
        handed.instance == held.instance,
        "交给开卡的是调用方手里的那一个实例"
    );
    let created_separately = VulkanInstanceOfThisProcess::without_any_backend();
    assert!(
        created_separately.instance != held.instance,
        "另建的实例与手里的不是同一个：比较分得出来"
    );
}

/// 等的上限给 0 是配错，出路是改参数：开卡在枚举之前就拒成核心层的 `GpuWaitLimitOfZero`，没点名、点名一张、空列表三种选卡形态都一样。
/// 实例不开后端（不要卡、也不碰 Vulkan）：先枚举的话，上限 0 在枚举那一步就拒成 `AdapterEnumerationFailed`，这条断言红。
#[test]
fn wait_limit_of_zero_is_refused_as_the_settings_member_before_the_enumeration_for_every_form_of_the_selection(
) {
    let vulkan_instance = VulkanInstanceOfThisProcess::without_any_backend();
    for (form_of_the_selection, background_gpu_devices) in [
        ("没点名", None),
        ("点名一张", Some(vec![card_identifier("0000:ff:1f.7")])),
        ("空列表", Some(Vec::new())),
    ] {
        let refused = open_background_gpu(
            &BackgroundGpuSelection {
                background_gpu_devices,
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: Duration::ZERO,
            },
            &vulkan_instance,
        )
        .err();
        assert_eq!(
            refused,
            Some(BackgroundGpuOpenRefused::SettingsRefused(
                BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
            )),
            "{form_of_the_selection}：上限 0 拒成设置配错，不是枚举没成"
        );
    }
}
