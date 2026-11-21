//! m4-gpu-wait-outside-the-lock 第三轮判决「三、改法」第 5、7 条的 A/B 探针。子命令：
//!   replan-after-the-window：见 `replan_after_the_window` 文件头（第 5 条，四臂）。
//!   rollback-gate-after-the-window：见 `rollback_gate_after_the_window` 文件头（第 7 条，两臂）。
//! 核心层里不公开的几步经仓副本里追加的 `singlefs_core::mounted_session::probe_replan_after_the_window`（模型目录 `core-patch/`）调。
mod judged;
mod pool;
mod replan_after_the_window;
mod rollback_gate_after_the_window;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("replan-after-the-window") => replan_after_the_window::run(),
        Some("rollback-gate-after-the-window") => rollback_gate_after_the_window::run(),
        _ => {
            eprintln!("用法：gpu-wait-outside-the-lock-r3-ab-probe replan-after-the-window | rollback-gate-after-the-window");
            std::process::exit(2);
        }
    }
}
