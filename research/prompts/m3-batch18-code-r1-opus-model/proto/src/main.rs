//! 第 18 批代码轮第一轮攻方腿（Opus）的原型：池级 checker 删除态判法在合法删除中间态上误不误红、该红的漏不漏、坏镜像是不是只红自己，
//! 与 GPU 判器那一侧要的 CPU 期望。只跑 harness 档：不起 checker 档的测试目标，只把 singlefs-checker-tier 当库调层 0 枚举。
#![allow(dead_code, clippy::all)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;

mod badimages;
mod crashpoints;
mod diskloss;
mod gpugap;
mod n9;
mod judge;
mod pool;
mod probe;
mod sweep;

fn threads() -> usize {
    std::env::var("M3B18_THREADS").ok().and_then(|n| n.parse().ok()).unwrap_or(1)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("probe") => probe::run(),
        Some("diskloss") => diskloss::run(threads()),
        Some("n9") => n9::run(),
        Some("gpugap") => gpugap::run(),
        Some("badimages") => badimages::run(),
        Some("crashpoints") => crashpoints::run(threads(), args.get(2).and_then(|n| n.parse().ok()).unwrap_or(0), args.get(3).and_then(|n| n.parse().ok()).unwrap_or(usize::MAX)),
        Some("sweep") => sweep::run(args.get(2).and_then(|n| n.parse().ok()), threads()),
        other => {
            eprintln!("用法：m3b18-opus-proto <probe|…>，收到 {other:?}");
            std::process::exit(2);
        }
    }
}
