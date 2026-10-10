//! 复算第 33 批 S7 的三个钉值，并用 naga 的校验器再验三份内核源码（draft，不入库）
use singlefs_checker_tier::crash_judge_dispatch::{kernel_inlined_size, LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL};
use singlefs_checker_tier::crash_judge_gpu::{checker_scan_kernel_source, checker_walk_kernel_source, recovery_kernel_source};
use wgpu::naga::{front::wgsl, valid::{Capabilities, ValidationFlags, Validator}};

fn main() {
    for (name, source) in [
        ("recovery", recovery_kernel_source()),
        ("checker-scan", checker_scan_kernel_source()),
        ("checker-walk", checker_walk_kernel_source()),
    ] {
        let module = wgsl::parse_str(&source).expect("parse");
        let validated = Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module);
        let size = kernel_inlined_size(&source);
        println!(
            "KERNEL {name} expressions={} margin={} validate={}",
            size.expressions,
            LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL - size.expressions,
            if validated.is_ok() { "ok".to_string() } else { format!("{:?}", validated.err()) }
        );
    }
}
