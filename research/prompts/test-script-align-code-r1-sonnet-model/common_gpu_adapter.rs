//! GPU 对拍用例开跑前取卡：本机一个能用的 GPU 适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`、
//! 用例当场返回成功（门禁按这一行逐条报「本次未跑」，上游 `test-script-discipline.md`「分快慢两档」：要特定硬件的那几条在没有那种硬件的机器上报「本次未跑」）；
//! 有适配器而一张卡都没起来（驱动不对、显存不够、上下文起不来）照旧判红。
//! 判定是纯函数 [`decide_gpu_test_start`]，输入是适配器枚举的结果；不挂特性的单测在
//! `gpu_adapter_probe_reports_not_run_without_an_adapter_and_red_when_no_card_starts.rs`。
#![allow(
    dead_code,
    reason = "取卡那一截挂 gpu 特性：不挂特性的单测文件只用到判定那一半"
)]

/// 崩溃放量只用 NVIDIA 独显：厂商号与 `gpu_unit_checks` 里 `usable_gpu_cards` 筛卡的那一个相同（那一份是私有常量，这里取不到）。
pub const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

/// 适配器的种类（与 Vulkan 报的物理设备类型一一对应）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterDeviceKind {
    DiscreteGpu,
    IntegratedGpu,
    VirtualGpu,
    SoftwareRenderingOnTheCpu,
    OtherOrUnknown,
}

/// Vulkan 枚举出来的一个适配器里，判定要看的两样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnumeratedAdapter {
    pub vendor_identifier: u32,
    pub device_kind: AdapterDeviceKind,
}

impl EnumeratedAdapter {
    /// 崩溃放量认的那一种卡：NVIDIA 独显（`usable_gpu_cards` 只从这一种里起上下文）。
    #[must_use]
    pub fn is_an_nvidia_discrete_gpu(&self) -> bool {
        self.vendor_identifier == NVIDIA_VENDOR_IDENTIFIER
            && self.device_kind == AdapterDeviceKind::DiscreteGpu
    }
}

/// 一条 GPU 对拍用例开跑前的三种结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuTestStart {
    /// 至少一张卡起好了上下文：照常跑。
    RunOnTheFirstUsableCard,
    /// 枚举出的适配器里没有一张 NVIDIA 独显：这台机器没有这种硬件，报「本次未跑」。
    NotRunNoGpuAdapter { reason: String },
    /// 有 NVIDIA 独显适配器，却一张卡都没起来：环境坏了，判红。
    AdapterPresentButNoCardStarted { reason: String },
}

/// `adapters` 是 Vulkan 枚举的结果，`usable_card_count` 是其中起好上下文的卡数（`usable_gpu_cards` 交回几张）。
#[must_use]
pub fn decide_gpu_test_start(
    adapters: &[EnumeratedAdapter],
    usable_card_count: usize,
) -> GpuTestStart {
    if usable_card_count > 0 {
        return GpuTestStart::RunOnTheFirstUsableCard;
    }
    let nvidia_discrete_adapter_count = adapters
        .iter()
        .filter(|adapter| adapter.is_an_nvidia_discrete_gpu())
        .count();
    if nvidia_discrete_adapter_count == 0 {
        GpuTestStart::NotRunNoGpuAdapter {
            reason: format!(
                "Vulkan 枚举出 {} 个适配器，没有一张 NVIDIA 独显：{adapters:?}",
                adapters.len()
            ),
        }
    } else {
        GpuTestStart::AdapterPresentButNoCardStarted {
            reason: format!(
                "Vulkan 枚举出 {nvidia_discrete_adapter_count} 张 NVIDIA 独显，一张都没起来（nvidia-smi 读不到这张卡、空闲显存低于门槛、或上下文起不来）：修好驱动或腾出显存再跑"
            ),
        }
    }
}

/// 这条用例要不要报「本次未跑」时打的那一行。
#[must_use]
pub fn not_run_line(test_name: &str, reason: &str) -> String {
    format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")
}

#[cfg(feature = "gpu")]
fn adapter_device_kind(device_type: wgpu::DeviceType) -> AdapterDeviceKind {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => AdapterDeviceKind::DiscreteGpu,
        wgpu::DeviceType::IntegratedGpu => AdapterDeviceKind::IntegratedGpu,
        wgpu::DeviceType::VirtualGpu => AdapterDeviceKind::VirtualGpu,
        wgpu::DeviceType::Cpu => AdapterDeviceKind::SoftwareRenderingOnTheCpu,
        wgpu::DeviceType::Other => AdapterDeviceKind::OtherOrUnknown,
    }
}

/// 本机 Vulkan 枚举出的每个适配器（与 `usable_gpu_cards` 同一个后端）。
#[cfg(feature = "gpu")]
#[must_use]
pub fn enumerate_vulkan_adapters() -> Vec<EnumeratedAdapter> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
        .into_iter()
        .map(|adapter| {
            let information = adapter.get_info();
            EnumeratedAdapter {
                vendor_identifier: information.vendor,
                device_kind: adapter_device_kind(information.device_type),
            }
        })
        .collect()
}

/// 取空闲显存最大的那张卡；本机没有 NVIDIA 独显适配器时打 [`not_run_line`] 那一行、交回 `None`（调用的用例当场返回）。
///
/// # Panics
/// 有 NVIDIA 独显适配器而一张卡都没起来。
#[cfg(feature = "gpu")]
#[must_use]
pub fn first_usable_card_or_report_not_run(
    test_name: &str,
) -> Option<singlefs_checker_tier::gpu_unit_checks::GpuCard> {
    let mut cards = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards();
    match decide_gpu_test_start(&enumerate_vulkan_adapters(), cards.len()) {
        GpuTestStart::RunOnTheFirstUsableCard => Some(cards.remove(0)),
        GpuTestStart::NotRunNoGpuAdapter { reason } => {
            println!("{}", not_run_line(test_name, &reason));
            None
        }
        GpuTestStart::AdapterPresentButNoCardStarted { reason } => {
            panic!("{test_name}：{reason}")
        }
    }
}
