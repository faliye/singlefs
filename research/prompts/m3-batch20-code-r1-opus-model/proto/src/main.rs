//! 第 20 批代码轮第一轮攻方腿（Opus）的原型：配对改了之后池级 checker 删除态判法在合法删除中间态上误不误报判不了 / 误红（N1）、
//! 树表形态两个方向都配上而违例漏判（N1）、坏镜像换造法是不是仍只红目标（N3）、GPU 恢复那一侧要的 CPU 期望（N5）、H3 形状（N9）。
//! 只跑 harness 档：不起 checker 档的测试目标与 bin，只把 singlefs-checker-tier 当库调。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;

mod judge;
mod pool;
mod shapes;
mod badimages;
mod sweep;
mod n9;
mod layer0;

fn threads() -> usize {
    std::env::var("M3B20_THREADS").ok().and_then(|n| n.parse().ok()).unwrap_or(1)
}

fn arg(args: &[String], index: usize) -> Option<usize> {
    args.get(index).and_then(|n| n.parse().ok())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("shapes") => shapes::run(),
        Some("badimages") => badimages::run(),
        Some("sweep") => sweep::run(arg(&args, 2).unwrap_or(0), arg(&args, 3).unwrap_or(usize::MAX), threads()),
        Some("estimate") => sweep::estimate(arg(&args, 2).unwrap_or(50), threads()),
        Some("n9") => n9::run(),
        Some("layer0") => layer0::run(arg(&args, 2).unwrap_or(0), arg(&args, 3).unwrap_or(usize::MAX), threads()),
        other => {
            eprintln!("用法：m3b20-opus-proto <shapes|badimages|sweep <from> <to>|estimate <stride>|n9|layer0>，收到 {other:?}");
            std::process::exit(2);
        }
    }
}
