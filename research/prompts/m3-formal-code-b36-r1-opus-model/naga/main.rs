//! 第 36 批代码轮第一轮攻方原型（GPU 那一半，不上卡）：36-2 前后的走读内核与扫描内核、与 `crates/mutations.tsv` 里点名「下一次挂载先施加的那一版」
//! 的四行 GPU 变异改过的内核，各经 naga 解析与校验（与 `crash_judge_gpu.rs` 里那道「先经 naga 解析与校验」同一个 Validator 设定），
//! 再数展开后的表达式（`crash_judge_dispatch::kernel_inlined_size`，预算 `LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`）。
//! 用法：b36naga <36-2 之前的三份 wgsl 所在目录> <mutations.tsv>
use singlefs_checker_tier::crash_judge_dispatch::{kernel_inlined_size, LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL};
use singlefs_checker_tier::crash_judge_gpu::{
    checker_scan_kernel_source, checker_walk_kernel_source, GPU_JUDGE_CHECKER_COMMON_SOURCE, GPU_JUDGE_CHECKER_SCAN_SOURCE,
    GPU_JUDGE_CHECKER_SOURCE,
};

fn validate(label: &str, source: &str) {
    let verdict = match wgpu::naga::front::wgsl::parse_str(source) {
        Err(error) => format!("parse-error {}", error.emit_to_string(source).lines().take(6).collect::<Vec<_>>().join(" | ")),
        Ok(module) => {
            let mut validator = wgpu::naga::valid::Validator::new(wgpu::naga::valid::ValidationFlags::all(), wgpu::naga::valid::Capabilities::all());
            match validator.validate(&module) {
                Ok(_) => "valid".to_string(),
                Err(error) => format!("invalid {}", error.emit_to_string(source).lines().take(6).collect::<Vec<_>>().join(" | ")),
            }
        }
    };
    let expressions = if verdict == "valid" { kernel_inlined_size(source).expressions.to_string() } else { "-".to_string() };
    println!("B36G kernel={label} naga={verdict} inlined_expressions={expressions} budget={LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL}");
}

fn main() {
    let pre_dir = std::env::args().nth(1).expect("36-2 之前的 wgsl 目录");
    let table = std::env::args().nth(2).expect("mutations.tsv");
    let read = |name: &str| std::fs::read_to_string(format!("{pre_dir}/{name}")).expect("读得出");
    let (pre_common, pre_walk, pre_scan) = (read("crash_judge_checker_common.wgsl"), read("crash_judge_checker.wgsl"), read("crash_judge_checker_scan.wgsl"));
    let walk_after = checker_walk_kernel_source();
    let scan_after = checker_scan_kernel_source();
    assert_eq!(walk_after.matches(GPU_JUDGE_CHECKER_COMMON_SOURCE).count(), 1);
    assert_eq!(walk_after.matches(GPU_JUDGE_CHECKER_SOURCE).count(), 1);
    assert_eq!(scan_after.matches(GPU_JUDGE_CHECKER_SCAN_SOURCE).count(), 1);
    let walk_before = walk_after.replace(GPU_JUDGE_CHECKER_COMMON_SOURCE, &pre_common).replace(GPU_JUDGE_CHECKER_SOURCE, &pre_walk);
    let scan_before = scan_after.replace(GPU_JUDGE_CHECKER_COMMON_SOURCE, &pre_common).replace(GPU_JUDGE_CHECKER_SCAN_SOURCE, &pre_scan);
    validate("walk-before-36-2", &walk_before);
    validate("walk-after-36-2", &walk_after);
    validate("scan-before-36-2", &scan_before);
    validate("scan-after-36-2", &scan_after);
    let wanted = ["第36批 36-2", "第34批 34-1 GPU 判器 RV：I-5.4 不收下一次挂载", "第35批 35-1 GPU 判器 FL：下一次挂载先施加"];
    for (number, line) in std::fs::read_to_string(&table).expect("变异表").lines().enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 6 || !wanted.iter().any(|w| fields[0].starts_with(w)) {
            continue;
        }
        let (file, anchor, replacement) = (fields[1], fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n"));
        let (component, kernels): (&str, Vec<(&str, &String)>) = match file {
            "crates/singlefs-checker-tier/src/crash_judge_checker.wgsl" => (GPU_JUDGE_CHECKER_SOURCE, vec![("walk", &walk_after)]),
            "crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl" => (GPU_JUDGE_CHECKER_SCAN_SOURCE, vec![("scan", &scan_after)]),
            "crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl" => (GPU_JUDGE_CHECKER_COMMON_SOURCE, vec![("walk", &walk_after), ("scan", &scan_after)]),
            _ => continue,
        };
        let hits = component.matches(anchor.as_str()).count();
        println!("B36G row={} anchor_hits_in_the_file={hits} name={}", number + 1, fields[0].chars().take(60).collect::<String>());
        if hits != 1 {
            continue;
        }
        let mutated_component = component.replacen(anchor.as_str(), &replacement, 1);
        for (kernel, source) in kernels {
            validate(&format!("{kernel}-mutated-by-row-{}", number + 1), &source.replace(component, &mutated_component));
        }
    }
}
