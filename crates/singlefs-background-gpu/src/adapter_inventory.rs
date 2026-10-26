//! 卡清单与挂载那一刻的校验（D26（后台整理与放置回收） 已定项 13 第 2–4 条），全是纯函数：输入是枚举到的适配器的描述，不碰 GPU。
//!
//! 哪些算显卡：只有独立显卡；集成显卡、虚拟显卡、软件 Vulkan（CPU 类型的适配器，例如 Mesa 的 lavapipe）与认不出类型的都不算、不进「全部」（第 2 条）。
//! 一张卡的稳定标识是 Vulkan 报的 PCI 总线地址（wgpu 适配器信息的 `device_pci_bus_id`，形如 `0000:01:00.0`）：同型号的两张卡名字相同、
//! 总线地址不同，重启与驱动升级都不改它。`background_gpu_devices` 点的就是这个串（[`gpu_device_identifier_of_the_pci_bus_address`]）。
//! 全部显卡按总线地址的字典序排：没给 `background_gpu_devices` 时 `background_gpu_memory_limit` 按这个次序一张卡一个（第 3 条），
//! Vulkan 的枚举次序不保证每次相同，不拿它当次序。
//!
//! 第 4 条的校验只有一份，在核心层（`singlefs_core::background_compute::admit_background_compute_settings`，挂载与挂着切换共用）：
//! 这里把同一次枚举交成核心层的卡清单（[`DiscreteGpuInventorySnapshot`]）交给它判，判过之后按同一份清单取参与的卡，判与用读的是同一份输入。

use singlefs_core::background_compute::{
    admit_background_compute_settings, BackgroundComputeBackend, BackgroundComputeSettings,
    BackgroundComputeSettingsRefused, GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes,
};

/// 适配器是哪一类设备：wgpu 的 `DeviceType` 一一对过来（[`Self::of_the_wgpu_device_type`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterDeviceKind {
    DiscreteGpu,
    IntegratedGpu,
    VirtualGpu,
    /// 在 CPU 上跑的软件实现（Vulkan 的 `VK_PHYSICAL_DEVICE_TYPE_CPU`，lavapipe 就是这一类）。
    SoftwareRenderingOnTheCpu,
    OtherOrUnknown,
}

impl AdapterDeviceKind {
    #[must_use]
    pub fn of_the_wgpu_device_type(device_type: wgpu::DeviceType) -> Self {
        match device_type {
            wgpu::DeviceType::DiscreteGpu => AdapterDeviceKind::DiscreteGpu,
            wgpu::DeviceType::IntegratedGpu => AdapterDeviceKind::IntegratedGpu,
            wgpu::DeviceType::VirtualGpu => AdapterDeviceKind::VirtualGpu,
            wgpu::DeviceType::Cpu => AdapterDeviceKind::SoftwareRenderingOnTheCpu,
            wgpu::DeviceType::Other => AdapterDeviceKind::OtherOrUnknown,
        }
    }

    /// 算不算第 2 条的「显卡」：只有独立显卡算。
    #[must_use]
    pub fn counts_as_a_background_gpu(self) -> bool {
        match self {
            AdapterDeviceKind::DiscreteGpu => true,
            AdapterDeviceKind::IntegratedGpu
            | AdapterDeviceKind::VirtualGpu
            | AdapterDeviceKind::SoftwareRenderingOnTheCpu
            | AdapterDeviceKind::OtherOrUnknown => false,
        }
    }
}

/// 枚举到的一个适配器，判「哪些算显卡」只读这几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumeratedAdapterDescription {
    pub name: String,
    pub device_kind: AdapterDeviceKind,
    /// wgpu 适配器信息的 `device_pci_bus_id`；驱动不报时是空串。
    pub pci_bus_address: String,
    /// 最大的一块设备本地内存堆（显存总量）；没读到是 `None`（不算显卡的适配器不去读）。
    pub device_local_memory: Option<GpuMemoryInBytes>,
}

/// 本机的一张独立显卡。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteGpuOnThisHost {
    /// 它在这一次枚举的适配器列表里排第几（从 0 起），外壳拿它找回那个适配器。
    pub position_in_the_enumeration: usize,
    pub identifier: GpuDeviceIdentifier,
    /// 造 [`Self::identifier`] 的那个总线地址；全部显卡按它的字典序排。
    pub pci_bus_address: String,
    pub name: String,
    pub total_memory: GpuMemoryInBytes,
}

/// 枚举结果交不成一份卡清单：都是条款没写的形态，第一版不支持，挂载报错（不降级、不跳过那张卡）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscreteGpuListRefused {
    /// 一张独立显卡驱动没报 PCI 总线地址：卡的标识取总线地址（本模块文件头），没有地址的卡怎么点名条款没写，第一版不支持。
    DiscreteGpuWithoutAPciBusAddressIsNotSupported { name: String },
    /// 一张独立显卡读不到设备本地内存堆：第 3 条要拿上限比显存总量，读不到总量怎么判条款没写，第一版不支持。
    DiscreteGpuWithoutADeviceLocalMemoryHeapIsNotSupported { name: String },
    /// 两个独立显卡适配器报了同一个总线地址（同一张卡经两份驱动各枚举一次）：一张卡只该出现一次，用哪一份条款没写，第一版不支持。
    PciBusAddressReportedByTwoDiscreteGpusIsNotSupported { pci_bus_address: String },
}

/// `background_gpu_devices` 里点一张卡要写的标识：那张卡的 PCI 总线地址，按 wgpu 报的原样（本模块文件头）。
#[must_use]
pub fn gpu_device_identifier_of_the_pci_bus_address(pci_bus_address: &str) -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
}

/// 第 2 条「本机全部独立显卡」：只留独立显卡，按 PCI 总线地址的字典序排。
///
/// # Errors
/// [`DiscreteGpuListRefused`] 的三种，都只看独立显卡；不算显卡的适配器缺什么都不管。
pub fn discrete_gpus_of_the_enumeration(
    adapters: &[EnumeratedAdapterDescription],
) -> Result<Vec<DiscreteGpuOnThisHost>, DiscreteGpuListRefused> {
    let mut discrete_gpus: Vec<DiscreteGpuOnThisHost> = Vec::new();
    for (position_in_the_enumeration, adapter) in adapters.iter().enumerate() {
        if !adapter.device_kind.counts_as_a_background_gpu() {
            continue;
        }
        if adapter.pci_bus_address.is_empty() {
            return Err(
                DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
                    name: adapter.name.clone(),
                },
            );
        }
        let Some(total_memory) = adapter.device_local_memory else {
            return Err(
                DiscreteGpuListRefused::DiscreteGpuWithoutADeviceLocalMemoryHeapIsNotSupported {
                    name: adapter.name.clone(),
                },
            );
        };
        let identifier = gpu_device_identifier_of_the_pci_bus_address(&adapter.pci_bus_address);
        if discrete_gpus
            .iter()
            .any(|already_listed| already_listed.identifier == identifier)
        {
            return Err(
                DiscreteGpuListRefused::PciBusAddressReportedByTwoDiscreteGpusIsNotSupported {
                    pci_bus_address: adapter.pci_bus_address.clone(),
                },
            );
        }
        discrete_gpus.push(DiscreteGpuOnThisHost {
            position_in_the_enumeration,
            identifier,
            pci_bus_address: adapter.pci_bus_address.clone(),
            name: adapter.name.clone(),
            total_memory,
        });
    }
    discrete_gpus.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));
    Ok(discrete_gpus)
}

/// 同一次枚举交成核心层的卡清单（`GpuInventory`）：挂载那一刻的校验读它，之后取参与的卡读的也是它。
pub struct DiscreteGpuInventorySnapshot<'enumeration> {
    pub discrete_gpus: &'enumeration [DiscreteGpuOnThisHost],
}

impl GpuInventory for DiscreteGpuInventorySnapshot<'_> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.discrete_gpus
            .iter()
            .map(|discrete_gpu| (discrete_gpu.identifier.clone(), discrete_gpu.total_memory))
            .collect()
    }
}

/// 挂载给的卡与上限（挂载参数 `background_gpu_devices`、`background_gpu_memory_limit`，第 2、3 条；`None` 是不给）与等 GPU 的上限。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundGpuSelection {
    pub background_gpu_devices: Option<Vec<GpuDeviceIdentifier>>,
    pub background_gpu_memory_limit: Option<Vec<GpuMemoryInBytes>>,
    /// 每一处阻塞等 GPU 的上限（挂载参数 `background_gpu_wait_limit`；用户 2026-10-26 定等 GPU 结果「设超时，超时算出错」）：
    /// 挂载那一刻等 wgpu 的 future、挂着的时候等一次提交做完与回读映射好，各次都在它之内，超过就按 GPU 出错办（`crate::waiting_for_the_gpu`）。
    pub background_gpu_wait_limit: std::time::Duration,
}

impl BackgroundGpuSelection {
    /// 挂载递给核心层的那一组设置里的卡、显存上限与等 GPU 的上限（`background_compute` 那一项不看：开卡只在选 `gpu` 时开）：
    /// 开卡与核心层挂载照同一组校验，开出来的卡就是挂载认的那几张。
    #[must_use]
    pub fn of_the_settings(settings: &BackgroundComputeSettings) -> Self {
        Self {
            background_gpu_devices: settings.background_gpu_devices.clone(),
            background_gpu_memory_limit: settings.background_gpu_memory_limit.clone(),
            background_gpu_wait_limit: settings.background_gpu_wait_limit,
        }
    }
}

/// 一张参与的卡与给它的显存上限（`None` 是不设上限）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipatingGpu {
    pub discrete_gpu: DiscreteGpuOnThisHost,
    pub memory_limit: Option<GpuMemoryInBytes>,
}

/// 挂载那一刻的校验（第 4 条）与参与的卡：先交核心层那一套校验判，过了再按同一份清单取参与的卡，
/// 没点名就是全部独立显卡（按总线地址排），点了名就是点名的那几张、按点名的次序，上限按次序一张一个。
///
/// # Errors
/// 核心层的 `BackgroundComputeSettingsRefused`：GPU 不在（一张独立显卡都没有）、点名的卡不存在、上限个数与参与的卡数不等、
/// 上限超过那张卡的显存总量，以及核心层另拒的两种条款没写的点法（空列表、同一张卡点两次）。
pub fn participating_gpus_admitted_by_the_mount_checks(
    selection: &BackgroundGpuSelection,
    discrete_gpus: &[DiscreteGpuOnThisHost],
) -> Result<Vec<ParticipatingGpu>, BackgroundComputeSettingsRefused> {
    let inventory = DiscreteGpuInventorySnapshot { discrete_gpus };
    admit_background_compute_settings(
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: selection.background_gpu_devices.clone(),
            background_gpu_memory_limit: selection.background_gpu_memory_limit.clone(),
            background_gpu_wait_limit: selection.background_gpu_wait_limit,
        },
        Some(&inventory),
    )?;
    let participating_without_limits: Vec<DiscreteGpuOnThisHost> =
        match &selection.background_gpu_devices {
            None => discrete_gpus.to_vec(),
            Some(named_devices) => named_devices
                .iter()
                .map(|named_device| {
                    discrete_gpus
                        .iter()
                        .find(|discrete_gpu| &discrete_gpu.identifier == named_device)
                        .cloned()
                        .expect("核心层的校验判过点名的卡都在这一份清单里；这里找不到说明判与取读的不是同一份清单")
                })
                .collect(),
        };
    let participating = match &selection.background_gpu_memory_limit {
        None => participating_without_limits
            .into_iter()
            .map(|discrete_gpu| ParticipatingGpu {
                discrete_gpu,
                memory_limit: None,
            })
            .collect(),
        Some(memory_limits) => {
            assert_eq!(
                memory_limits.len(),
                participating_without_limits.len(),
                "核心层的校验判过上限个数与参与的卡数相等"
            );
            participating_without_limits
                .into_iter()
                .zip(memory_limits.iter().copied())
                .map(|(discrete_gpu, memory_limit)| ParticipatingGpu {
                    discrete_gpu,
                    memory_limit: Some(memory_limit),
                })
                .collect()
        }
    };
    Ok(participating)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIBIBYTE: u64 = 1 << 30;
    const WAIT_LIMIT_UNDER_TEST: std::time::Duration = std::time::Duration::from_secs(5);

    fn adapter(
        name: &str,
        device_kind: AdapterDeviceKind,
        pci_bus_address: &str,
        device_local_memory_in_gibibytes: u64,
    ) -> EnumeratedAdapterDescription {
        EnumeratedAdapterDescription {
            name: name.to_string(),
            device_kind,
            pci_bus_address: pci_bus_address.to_string(),
            device_local_memory: Some(GpuMemoryInBytes(
                device_local_memory_in_gibibytes * GIBIBYTE,
            )),
        }
    }

    /// 一张独显、一张集显、一个软件 Vulkan（CPU 类型）：本机常见的那一种摆法。
    fn discrete_integrated_and_software_adapters() -> Vec<EnumeratedAdapterDescription> {
        vec![
            adapter(
                "integrated graphics",
                AdapterDeviceKind::IntegratedGpu,
                "0000:00:02.0",
                2,
            ),
            adapter(
                "llvmpipe software rasterizer",
                AdapterDeviceKind::SoftwareRenderingOnTheCpu,
                "",
                0,
            ),
            adapter(
                "discrete graphics",
                AdapterDeviceKind::DiscreteGpu,
                "0000:01:00.0",
                32,
            ),
        ]
    }

    #[test]
    fn only_the_discrete_gpu_counts_among_discrete_integrated_and_software_adapters() {
        let discrete_gpus =
            discrete_gpus_of_the_enumeration(&discrete_integrated_and_software_adapters())
                .expect("独显有总线地址与显存");
        assert_eq!(
            discrete_gpus,
            vec![DiscreteGpuOnThisHost {
                position_in_the_enumeration: 2,
                identifier: gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
                pci_bus_address: "0000:01:00.0".to_string(),
                name: "discrete graphics".to_string(),
                total_memory: GpuMemoryInBytes(32 * GIBIBYTE),
            }],
            "集显与软件 Vulkan 不进「全部」，只留独显，位置指回它在枚举里的那一个"
        );
    }

    #[test]
    fn virtual_and_unknown_adapters_do_not_count_as_background_gpus() {
        let adapters = vec![
            adapter("virtual", AdapterDeviceKind::VirtualGpu, "0000:03:00.0", 8),
            adapter(
                "unknown",
                AdapterDeviceKind::OtherOrUnknown,
                "0000:04:00.0",
                8,
            ),
        ];
        assert_eq!(
            discrete_gpus_of_the_enumeration(&adapters),
            Ok(Vec::new()),
            "只有独立显卡算显卡"
        );
    }

    #[test]
    fn discrete_gpus_are_listed_by_pci_bus_address_not_by_enumeration_order() {
        let adapters = vec![
            adapter(
                "card at the high bus address",
                AdapterDeviceKind::DiscreteGpu,
                "0000:41:00.0",
                16,
            ),
            adapter(
                "card at the low bus address",
                AdapterDeviceKind::DiscreteGpu,
                "0000:01:00.0",
                32,
            ),
        ];
        let discrete_gpus = discrete_gpus_of_the_enumeration(&adapters).expect("两张独显都齐");
        let names: Vec<&str> = discrete_gpus
            .iter()
            .map(|discrete_gpu| discrete_gpu.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "card at the low bus address",
                "card at the high bus address"
            ],
            "按总线地址排"
        );
        assert_eq!(
            discrete_gpus
                .iter()
                .map(|discrete_gpu| discrete_gpu.position_in_the_enumeration)
                .collect::<Vec<_>>(),
            vec![1, 0],
            "位置照旧指回枚举里的那一个"
        );
    }

    #[test]
    fn a_discrete_gpu_without_a_pci_bus_address_is_refused() {
        let adapters = vec![adapter("no address", AdapterDeviceKind::DiscreteGpu, "", 8)];
        assert_eq!(
            discrete_gpus_of_the_enumeration(&adapters),
            Err(
                DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
                    name: "no address".to_string()
                }
            )
        );
    }

    #[test]
    fn a_discrete_gpu_without_a_device_local_memory_heap_is_refused() {
        let mut no_heap = adapter("no heap", AdapterDeviceKind::DiscreteGpu, "0000:01:00.0", 8);
        no_heap.device_local_memory = None;
        assert_eq!(
            discrete_gpus_of_the_enumeration(&[no_heap]),
            Err(
                DiscreteGpuListRefused::DiscreteGpuWithoutADeviceLocalMemoryHeapIsNotSupported {
                    name: "no heap".to_string()
                }
            )
        );
    }

    #[test]
    fn two_discrete_gpus_reporting_the_same_pci_bus_address_are_refused() {
        let adapters = vec![
            adapter(
                "one driver",
                AdapterDeviceKind::DiscreteGpu,
                "0000:01:00.0",
                8,
            ),
            adapter(
                "other driver",
                AdapterDeviceKind::DiscreteGpu,
                "0000:01:00.0",
                8,
            ),
        ];
        assert_eq!(
            discrete_gpus_of_the_enumeration(&adapters),
            Err(
                DiscreteGpuListRefused::PciBusAddressReportedByTwoDiscreteGpusIsNotSupported {
                    pci_bus_address: "0000:01:00.0".to_string()
                }
            )
        );
    }

    fn two_discrete_gpus() -> Vec<DiscreteGpuOnThisHost> {
        discrete_gpus_of_the_enumeration(&[
            adapter(
                "big card",
                AdapterDeviceKind::DiscreteGpu,
                "0000:01:00.0",
                32,
            ),
            adapter(
                "small card",
                AdapterDeviceKind::DiscreteGpu,
                "0000:41:00.0",
                16,
            ),
        ])
        .expect("两张独显都齐")
    }

    #[test]
    fn no_discrete_gpu_refuses_the_mount_as_gpu_not_present() {
        let discrete_gpus = discrete_gpus_of_the_enumeration(&[
            adapter(
                "integrated graphics",
                AdapterDeviceKind::IntegratedGpu,
                "0000:00:02.0",
                2,
            ),
            adapter(
                "llvmpipe",
                AdapterDeviceKind::SoftwareRenderingOnTheCpu,
                "",
                0,
            ),
        ])
        .expect("没有独显也交得出空清单");
        assert_eq!(
            participating_gpus_admitted_by_the_mount_checks(
                &BackgroundGpuSelection {
                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
                    background_gpu_devices: None,
                    background_gpu_memory_limit: None,
                },
                &discrete_gpus,
            ),
            Err(BackgroundComputeSettingsRefused::GpuNotPresent),
            "只有集显与软件 Vulkan 时 GPU 不在，拒绝挂载、不降级"
        );
    }

    #[test]
    fn naming_a_gpu_that_is_not_a_discrete_gpu_of_this_host_refuses_the_mount() {
        let integrated_address = gpu_device_identifier_of_the_pci_bus_address("0000:00:02.0");
        assert_eq!(
            participating_gpus_admitted_by_the_mount_checks(
                &BackgroundGpuSelection {
                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
                    background_gpu_devices: Some(vec![integrated_address.clone()]),
                    background_gpu_memory_limit: None,
                },
                &two_discrete_gpus(),
            ),
            Err(
                BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                    device: integrated_address
                }
            ),
            "点名的是集显（不在显卡清单里）：点名的卡不存在"
        );
    }

    #[test]
    fn a_memory_limit_count_different_from_the_participating_gpus_refuses_the_mount() {
        assert_eq!(
            participating_gpus_admitted_by_the_mount_checks(
                &BackgroundGpuSelection {
                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
                    background_gpu_devices: None,
                    background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(GIBIBYTE)]),
                },
                &two_discrete_gpus(),
            ),
            Err(
                BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                    memory_limits_given: 1,
                    participating_devices: 2,
                }
            ),
            "两张卡只给一个上限：不补齐，报错"
        );
    }

    #[test]
    fn a_memory_limit_above_the_total_memory_of_its_gpu_refuses_the_mount() {
        assert_eq!(
            participating_gpus_admitted_by_the_mount_checks(
                &BackgroundGpuSelection {
                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
                    background_gpu_devices: None,
                    background_gpu_memory_limit: Some(vec![
                        GpuMemoryInBytes(GIBIBYTE),
                        GpuMemoryInBytes(17 * GIBIBYTE),
                    ]),
                },
                &two_discrete_gpus(),
            ),
            Err(
                BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice {
                    device: gpu_device_identifier_of_the_pci_bus_address("0000:41:00.0"),
                    memory_limit: GpuMemoryInBytes(17 * GIBIBYTE),
                    total_memory: GpuMemoryInBytes(16 * GIBIBYTE),
                }
            ),
            "16 GiB 的卡给了 17 GiB 的上限"
        );
    }

    #[test]
    fn named_gpus_participate_in_the_named_order_with_their_memory_limits() {
        let participating = participating_gpus_admitted_by_the_mount_checks(
            &BackgroundGpuSelection {
                background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
                background_gpu_devices: Some(vec![
                    gpu_device_identifier_of_the_pci_bus_address("0000:41:00.0"),
                    gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
                ]),
                background_gpu_memory_limit: Some(vec![
                    GpuMemoryInBytes(2 * GIBIBYTE),
                    GpuMemoryInBytes(3 * GIBIBYTE),
                ]),
            },
            &two_discrete_gpus(),
        )
        .expect("两张卡都在、上限都不超");
        let named_order: Vec<(&str, Option<GpuMemoryInBytes>)> = participating
            .iter()
            .map(|gpu| (gpu.discrete_gpu.name.as_str(), gpu.memory_limit))
            .collect();
        assert_eq!(
            named_order,
            vec![
                ("small card", Some(GpuMemoryInBytes(2 * GIBIBYTE))),
                ("big card", Some(GpuMemoryInBytes(3 * GIBIBYTE))),
            ],
            "点名的次序就是上限对应的次序"
        );
    }

    #[test]
    fn the_selection_of_the_mount_settings_carries_their_gpus_memory_limits_and_wait_limit() {
        let settings = BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![gpu_device_identifier_of_the_pci_bus_address(
                "0000:41:00.0",
            )]),
            background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(2 * GIBIBYTE)]),
            background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
        };
        assert_eq!(
            BackgroundGpuSelection::of_the_settings(&settings),
            BackgroundGpuSelection {
                background_gpu_devices: Some(vec![gpu_device_identifier_of_the_pci_bus_address(
                    "0000:41:00.0"
                )]),
                background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(2 * GIBIBYTE)]),
                background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
            },
            "开卡照挂载那一组的卡、上限与等 GPU 的上限"
        );
    }

    #[test]
    fn without_named_gpus_every_discrete_gpu_participates_without_a_memory_limit() {
        let participating = participating_gpus_admitted_by_the_mount_checks(
            &BackgroundGpuSelection {
                background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
                background_gpu_devices: None,
                background_gpu_memory_limit: None,
            },
            &two_discrete_gpus(),
        )
        .expect("不给卡也不给上限：全部独显、不设限");
        assert_eq!(participating.len(), 2, "全部独显都用");
        assert!(
            participating.iter().all(|gpu| gpu.memory_limit.is_none()),
            "不给上限就不设限"
        );
    }
}
