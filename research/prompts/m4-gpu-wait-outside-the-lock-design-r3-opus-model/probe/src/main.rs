//! 设计三方 m4-gpu-wait-outside-the-lock-design-r3 攻方腿的探针。子命令：
//!   issue-after-the-window：见 `issue_after_the_window` 文件头。
mod issue_after_the_window;
mod pool;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("issue-after-the-window") => issue_after_the_window::run(),
        _ => {
            eprintln!("用法：gpu-wait-outside-the-lock-r3-probe issue-after-the-window");
            std::process::exit(2);
        }
    }
}
