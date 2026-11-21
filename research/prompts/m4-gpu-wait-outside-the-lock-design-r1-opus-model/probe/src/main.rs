//! 设计三方 m4-gpu-wait-outside-the-lock-design-r1 攻方腿的探针。在今天的核心层与运行库上量拆法要靠的两样，不拆代码：
//!   stale-base：回锁复核只比活意图这类不含「现行那一版没换」的判据时，锁外那一批建在取输入那一刻的那一版上发出去，盘上留下什么；
//!   lock-order：锁外等 GPU 时持着 GPU 实现那把锁，前台切换到 gpu 先取会话锁再取它，前台读被挡多久、持着它回会话锁死不死锁。
//! 用法：gpu-wait-outside-the-lock-r1-probe stale-base | lock-order
//! 退出码：0 跑完；2 用法错；3 先跑一段估出的全量挂钟超过 40 分钟，没跑全量。
mod lock_order;
mod pool;
mod stale_base;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["stale-base"] => stale_base::run(),
        ["lock-order"] => lock_order::run(),
        _ => {
            eprintln!("用法：gpu-wait-outside-the-lock-r1-probe stale-base | lock-order");
            std::process::exit(2);
        }
    }
}
