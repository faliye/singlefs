//! 第 21 批代码轮第一轮攻方腿（Opus）的原型（m3-batch21-code-r1）：合法历史上池级 checker 删除态判法误不误报（N1 第二次抽样）与 GPU 恢复内核
//! 会拒的形状（N3，CPU 算期望）；小写流上逐个崩溃状态跑池级 checker（`crash`）。只把 singlefs-checker-tier 当库调，不起 checker 档的测试目标与 bin。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;

mod badimages;
mod crash;
mod judge;
mod pool;
mod shapes;
mod sweep;

fn threads() -> usize {
    std::env::var("M3B21_THREADS").ok().and_then(|n| n.parse().ok()).unwrap_or(1)
}

fn arg(args: &[String], index: usize) -> Option<usize> {
    args.get(index).and_then(|n| n.parse().ok())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("sweep") => sweep::run(arg(&args, 2).unwrap_or(0), arg(&args, 3).unwrap_or(usize::MAX), threads()),
        Some("histories") => println!("HISTORIES {}", sweep::histories().len()),
        Some("count") => crash::count(),
        Some("shapes") => shapes::run(),
        Some("badimages") => badimages::run(),
        Some("crash") => crash::run(args.get(2).map(String::as_str).unwrap_or("")),
        other => {
            eprintln!("用法：m3b21-opus-proto <sweep <from> <to>|histories|count|crash <名>>，收到 {other:?}");
            std::process::exit(2);
        }
    }
}
