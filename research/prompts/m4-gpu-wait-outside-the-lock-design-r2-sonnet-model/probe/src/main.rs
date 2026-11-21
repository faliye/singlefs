//! 设计三方 m4-gpu-wait-outside-the-lock-design-r2 辩方腿的探针。第一轮攻方探针的 `stale-base` 只摆了一种前台改动（覆盖写第一个文件里的一个单元），
//! 45 段旧底历史里整理的第一次发布都被 `ReleaseTargetAlreadyReleased` 拒掉，没有一段走到「旧底上的整理发布做成了」那一格。这里把前台改动的种类放开：
//!   stale-base-kinds：前台改动种类 × 次数 k × 整理之后再发的次数 m，弱判据放行（把现行那一版换回取输入那一刻克隆的）与对照各臂，
//!                     逐段打出整理那一趟的结局原文（不截成 100 个字）与读回的内容，判丢不丢。
//! 用法：gpu-wait-outside-the-lock-r2-probe stale-base-kinds
//! 退出码：0 跑完；2 用法错。
mod pool;
mod stale_base_kinds;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["stale-base-kinds"] => stale_base_kinds::run(),
        _ => {
            eprintln!("用法：gpu-wait-outside-the-lock-r2-probe stale-base-kinds");
            std::process::exit(2);
        }
    }
}
