//! 臂与卡：交替跑的六条臂、本进程看到的独显、每条臂的后台计算设定。

use super::*;

// ---- 臂与卡（登记第五节「臂」「阳性对照」，第一节「卡」）----

/// 一条臂在哪算。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum ComputeOfTheArm {
    /// `background_compute=cpu`，卡与上限不给（产品默认）。
    Cpu,
    /// `gpu`，只点服务交来的独显里 PCI 地址最小的那一张，上限 1 GiB。
    GpuOnTheCardWithTheLowestPciAddress,
    /// `gpu`，`background_gpu_devices` 点名服务交来的全部独显（交几张开几张，本机没交的卡不开），每张卡上限 1 GiB。
    GpuOnEveryCard,
}

/// 多判不多判（阳性对照）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum RejudgingOfTheArm {
    NotRejudged,
    /// 每个确认的条目在 GC 线程上照核心层的 CPU 写法再判一遍（GPU 臂经包装、CPU 臂在每次 GC 调用返回之后、窗口之内）。
    EveryConfirmedEntryRejudgedOnTheCpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct LockstepArm {
    pub(super) compute: ComputeOfTheArm,
    pub(super) rejudging: RejudgingOfTheArm,
}

/// 交替跑的六条臂，按轮换的基准次序（第 r 次重复从第 r 条起轮换，登记第十四节）。
pub(super) const LOCKSTEP_ARMS: [LockstepArm; 6] = [
    LockstepArm {
        compute: ComputeOfTheArm::Cpu,
        rejudging: RejudgingOfTheArm::NotRejudged,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        rejudging: RejudgingOfTheArm::NotRejudged,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnEveryCard,
        rejudging: RejudgingOfTheArm::NotRejudged,
    },
    LockstepArm {
        compute: ComputeOfTheArm::Cpu,
        rejudging: RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        rejudging: RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnEveryCard,
        rejudging: RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu,
    },
];

impl ComputeOfTheArm {
    pub(super) const fn name(self) -> &'static str {
        match self {
            ComputeOfTheArm::Cpu => "cpu",
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress => "gpu_one_card",
            ComputeOfTheArm::GpuOnEveryCard => "gpu_every_card",
        }
    }

    pub(super) const fn uses_the_gpu(self) -> bool {
        match self {
            ComputeOfTheArm::Cpu => false,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress
            | ComputeOfTheArm::GpuOnEveryCard => true,
        }
    }
}

impl LockstepArm {
    pub(super) fn name(self) -> String {
        match self.rejudging {
            RejudgingOfTheArm::NotRejudged => self.compute.name().to_string(),
            RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu => {
                format!("{}_rejudged", self.compute.name())
            }
        }
    }

    pub(super) fn of_the_name(name: &str) -> Option<LockstepArm> {
        LOCKSTEP_ARMS.into_iter().find(|arm| arm.name() == name)
    }
}

/// 本进程看得到的一张独立显卡：PCI 总线地址（卡的标识，按 wgpu 报的原样）、型号与驱动的两样（wgpu 适配器信息里的 `driver`、`driver_info`，
/// 与地址、型号同一次 `get_info()` 读出；空串原样留着，写进产物时写 `none`，[`card_segment_value`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscreteGpuSeenByThisProcess {
    pub(super) pci_bus_address: String,
    pub(super) model: String,
    pub(super) driver: String,
    pub(super) driver_information: String,
}

/// 本进程看得到的独立显卡，按 PCI 地址次序（产品没点名时填满的次序）：在这个进程的那一份 Vulkan 实例（开卡也用它）上枚举一次，
/// 判法同产品（集显、虚拟显卡、软件 Vulkan、认不出类型的与没有总线地址的不算）。
pub(super) fn discrete_gpus_seen_by_this_process(
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> Result<Vec<DiscreteGpuSeenByThisProcess>, String> {
    let adapters = vulkan_instance
        .vulkan_adapters_enumerated_within(LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT)
        .map_err(|cause| format!("枚举适配器没在等的上限之内交回：{cause:?}"))?;
    let mut discrete_gpus: Vec<DiscreteGpuSeenByThisProcess> = adapters
        .iter()
        .map(wgpu::Adapter::get_info)
        .filter(|information| {
            AdapterDeviceKind::of_the_wgpu_device_type(information.device_type)
                .counts_as_a_background_gpu()
                && !information.device_pci_bus_id.is_empty()
        })
        .map(|information| DiscreteGpuSeenByThisProcess {
            pci_bus_address: information.device_pci_bus_id,
            model: information.name,
            driver: information.driver,
            driver_information: information.driver_info,
        })
        .collect();
    discrete_gpus.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));
    Ok(discrete_gpus)
}

/// 这条臂在服务交来 `gpu_count` 张独显（本进程看得到、只用的就是这几张）的任务上跑不跑得了；跑不了的交回 `name=not_run` 那一行的理由（登记第十一节作废第 8 条）。
pub(super) fn reason_the_arm_is_not_run(
    compute: ComputeOfTheArm,
    gpu_count: usize,
) -> Option<&'static str> {
    match compute {
        ComputeOfTheArm::Cpu => None,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress if gpu_count == 0 => {
            Some("本进程看得到的独立显卡一张都没有")
        }
        ComputeOfTheArm::GpuOnEveryCard if gpu_count == 0 => {
            Some("本进程看得到的独立显卡一张都没有")
        }
        ComputeOfTheArm::GpuOnEveryCard if gpu_count == 1 => {
            Some("本进程只看得到一张独立显卡，全部卡臂与点名一张卡臂相同")
        }
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress | ComputeOfTheArm::GpuOnEveryCard => {
            None
        }
    }
}

/// 这条臂的挂载参数（登记第五节「臂」）：只差 `background_compute` 与 GPU 那几项；每张参与的卡上限 1 GiB。
pub(super) fn background_compute_settings_of_the_arm(
    compute: ComputeOfTheArm,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
) -> BackgroundComputeSettings {
    let memory_limit = GpuMemoryInBytes(LOCAL_MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU);
    match compute {
        ComputeOfTheArm::Cpu => BackgroundComputeSettings::PRODUCT,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress => BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(
                discrete_gpus
                    .iter()
                    .take(1)
                    .map(|gpu| gpu_device_identifier_of_the_pci_bus_address(&gpu.pci_bus_address))
                    .collect(),
            ),
            background_gpu_memory_limit: Some(vec![memory_limit]),
            background_gpu_wait_limit: LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        ComputeOfTheArm::GpuOnEveryCard => BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(
                discrete_gpus
                    .iter()
                    .map(|gpu| gpu_device_identifier_of_the_pci_bus_address(&gpu.pci_bus_address))
                    .collect(),
            ),
            background_gpu_memory_limit: Some(vec![memory_limit; discrete_gpus.len()]),
            background_gpu_wait_limit: LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
    }
}

#[cfg(test)]
mod tests;
