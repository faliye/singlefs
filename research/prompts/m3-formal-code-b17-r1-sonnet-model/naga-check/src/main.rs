// 草稿：只把两个池级 checker 内核的源码交给 naga 解析与校验（与 crash_judge_gpu.rs 的 parse_and_validate 同一套调用），不跑任何 checker 档测试。
use singlefs_checker_tier::crash_judge_gpu::{checker_scan_kernel_source, checker_walk_kernel_source, recovery_kernel_source};

fn parse_and_validate(name: &str, source: &str) -> bool {
    let module = match wgpu::naga::front::wgsl::parse_str(source) {
        Ok(module) => module,
        Err(error) => {
            println!("{name} 解析失败：\n{}", error.emit_to_string(source));
            return false;
        }
    };
    let mut validator = wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::all(),
    );
    match validator.validate(&module) {
        Ok(_) => {
            println!("{name} 解析与校验通过（{} 字节源码）", source.len());
            true
        }
        Err(error) => {
            println!("{name} 校验失败：\n{}", error.emit_to_string(source));
            false
        }
    }
}

fn main() {
    let results = [
        parse_and_validate("恢复内核", &recovery_kernel_source()),
        parse_and_validate("池级 checker 走读内核", &checker_walk_kernel_source()),
        parse_and_validate("池级 checker 扫描内核", &checker_scan_kernel_source()),
    ];
    std::process::exit(if results.iter().all(|ok| *ok) { 0 } else { 1 });
}
