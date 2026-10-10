use singlefs_checker::walk::{
    ContainerCapacityKnobs, LineageContainerCapacityKnob, RetirementContainerCapacityKnob,
    TombstoneContainerCapacityKnob,
};
use singlefs_checker_tier::crash_judge_dispatch::kernel_inlined_size;
use singlefs_checker_tier::crash_judge_gpu::{
    checker_scan_kernel_source, checker_scan_kernel_source_given_the_container_capacity_knobs,
    checker_walk_kernel_source, recovery_kernel_source,
};

fn validated(label: &str, source: &str) {
    let module = wgpu::naga::front::wgsl::parse_str(source).expect("parse");
    let info = wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::all(),
    )
    .validate(&module);
    println!("VALIDATE {label} {}", if info.is_ok() { "ok" } else { "ERR" });
}

fn main() {
    for (name, src) in [
        ("recovery", recovery_kernel_source()),
        ("checker-scan", checker_scan_kernel_source()),
        ("checker-walk", checker_walk_kernel_source()),
    ] {
        let s = kernel_inlined_size(&src);
        println!("SIZE {name} {}", s.expressions);
        validated(name, &src);
    }
    let knobs = [
        ("tomb1", ContainerCapacityKnobs { tombstone: Some(TombstoneContainerCapacityKnob::new(1).unwrap()), lineage: None, retirement: None }),
        ("lin1", ContainerCapacityKnobs { tombstone: None, lineage: Some(LineageContainerCapacityKnob::new(1).unwrap()), retirement: None }),
        ("ret959", ContainerCapacityKnobs { tombstone: None, lineage: None, retirement: Some(RetirementContainerCapacityKnob::new(959).unwrap()) }),
        ("all", ContainerCapacityKnobs {
            tombstone: Some(TombstoneContainerCapacityKnob::new(815).unwrap()),
            lineage: Some(LineageContainerCapacityKnob::new(1359).unwrap()),
            retirement: Some(RetirementContainerCapacityKnob::new(2).unwrap()) }),
    ];
    for (name, k) in knobs {
        let src = checker_scan_kernel_source_given_the_container_capacity_knobs(k);
        let s = kernel_inlined_size(&src);
        println!("SIZE scan-{name} {}", s.expressions);
        validated(&format!("scan-{name}"), &src);
        let head: String = src.lines().filter(|l| l.contains("CONTAINER_CAPACITY_KNOB: u32")).collect::<Vec<_>>().join(" | ");
        println!("CONSTS scan-{name} {head}");
    }
}
