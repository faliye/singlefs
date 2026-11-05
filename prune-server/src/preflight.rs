//! 准入与运行条件的现判（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：
//! 直接起 `python3 <仓根>/.claude/singlefs-ai-sop/scripts/preflight.py check <源文件> [--force] -- <参数…>`，
//! 退出码 78 就退 78，别的非 0 退 1。声明写在两个二进制的源文件头。

use std::path::Path;
use std::process::Command;

/// 退出码：准入或运行条件不满足。
pub const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `source_relative_to_manifest` 是二进制源文件相对这个工程目录的路径。回 `Err(退出码)` 时调用方照它退出。
pub fn preflight(source_relative_to_manifest: &str, arguments: &[String]) -> Result<(), i32> {
    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = manifest_directory.join(source_relative_to_manifest);
    let script = manifest_directory
        .join("..")
        .join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("  ✗ 起不了 python3 判准入：{error}");
            eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
            return Err(PREFLIGHT_REFUSED_EXIT_CODE);
        }
    };
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        return Err(
            if output.status.code() == Some(PREFLIGHT_REFUSED_EXIT_CODE) {
                PREFLIGHT_REFUSED_EXIT_CODE
            } else {
                1
            },
        );
    }
    let verdict = String::from_utf8_lossy(&output.stdout);
    if verdict.starts_with("forced") {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        eprintln!("  · 这一次是 --force 强制跑的：{}", verdict.trim());
    }
    Ok(())
}
