// 探针：把冻结副本里 common_gpu_adapter/mod.rs（原样拷来，不挂 gpu 特性的那一半）当模块编进来，
// 拿 decide_gpu_test_start 喂几种「Vulkan 枚举结果 + 起好上下文的卡数」，打它的结局。
// 这个纯函数只有这两样输入：没有 PCI 一类的第三个输入，所以「PCI 上有 NVIDIA 卡而 Vulkan 零个适配器」与「机器上根本没卡」喂给它的是同一对参数。
#[path = "common_gpu_adapter.rs"]
mod common_gpu_adapter;

use common_gpu_adapter::{
    decide_gpu_test_start, AdapterDeviceKind, EnumeratedAdapter, GpuTestStart,
    NVIDIA_VENDOR_IDENTIFIER,
};

fn outcome_name(start: &GpuTestStart) -> &'static str {
    match start {
        GpuTestStart::RunOnTheFirstUsableCard => "RunOnTheFirstUsableCard",
        GpuTestStart::NotRunNoGpuAdapter { .. } => "NotRunNoGpuAdapter",
        GpuTestStart::AdapterPresentButNoCardStarted { .. } => "AdapterPresentButNoCardStarted",
    }
}

fn main() {
    let nvidia_discrete = EnumeratedAdapter {
        vendor_identifier: NVIDIA_VENDOR_IDENTIFIER,
        device_kind: AdapterDeviceKind::DiscreteGpu,
    };
    let nvidia_virtual = EnumeratedAdapter {
        vendor_identifier: NVIDIA_VENDOR_IDENTIFIER,
        device_kind: AdapterDeviceKind::VirtualGpu,
    };
    let amd_discrete = EnumeratedAdapter {
        vendor_identifier: 0x1002,
        device_kind: AdapterDeviceKind::DiscreteGpu,
    };
    let intel_integrated = EnumeratedAdapter {
        vendor_identifier: 0x8086,
        device_kind: AdapterDeviceKind::IntegratedGpu,
    };
    let software = EnumeratedAdapter {
        vendor_identifier: 0x10005,
        device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
    };
    let cases: Vec<(&str, Vec<EnumeratedAdapter>, usize)> = vec![
        ("A 没有任何适配器（机器上没卡；也是 PCI 上插着 NVIDIA 卡而 Vulkan 驱动坏了枚举不出来的那一种）", vec![], 0),
        ("B 只有集显", vec![intel_integrated], 0),
        ("C 只有 AMD 独显", vec![amd_discrete], 0),
        ("D 只有 NVIDIA 虚拟 GPU", vec![nvidia_virtual], 0),
        ("E 只有软件渲染", vec![software], 0),
        ("F NVIDIA 独显枚举出来而一张卡都没起来", vec![nvidia_discrete], 0),
        ("G NVIDIA 独显且一张卡起好了", vec![nvidia_discrete], 1),
        ("H 集显加 NVIDIA 独显（独显没起来）", vec![intel_integrated, nvidia_discrete], 0),
    ];
    for (name, adapters, usable_card_count) in &cases {
        let start = decide_gpu_test_start(adapters, *usable_card_count);
        println!("case={name} outcome={}", outcome_name(&start));
    }
}
