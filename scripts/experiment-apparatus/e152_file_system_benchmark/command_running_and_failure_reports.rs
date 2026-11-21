//! 子进程命令的描述与同步运行、来宾失败的报告与逐行输出的发射。

use super::*;

pub(super) struct CommandLine {
    pub(super) program: &'static str,
    pub(super) arguments: Vec<String>,
}

impl CommandLine {
    pub(super) fn new(program: &'static str, arguments: &[&str]) -> Self {
        Self {
            program,
            arguments: arguments.iter().map(|argument| (*argument).to_string()).collect(),
        }
    }
}

#[derive(Debug)]
pub(super) struct GuestFailure {
    pub(super) stage: &'static str,
    pub(super) detail: String,
}

impl GuestFailure {
    pub(super) fn new(stage: &'static str, detail: impl Into<String>) -> Self {
        Self { stage, detail: detail.into() }
    }
}

pub(super) struct Emitter {
    pub(super) emitted: u64,
    pub(super) context: String,
}

impl Emitter {
    pub(super) fn emit(&mut self, name: &str, body: &str) {
        self.emitted += 1;
        println!("E7RESULT name={name} {} {body}", self.context);
    }

    /// vm-bench.sh 的完整性闸认这一行：程序声称发了几条，宿主就得抓到几条。
    pub(super) fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 结果行按空格分字段：把自由文本（报错、dmesg）压成一个不含空白的字段。
pub(super) fn single_token(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join("_");
    if joined.is_empty() {
        "none".to_string()
    } else {
        joined.chars().take(400).collect()
    }
}

pub(super) fn run_checked(command: &CommandLine, stage: &'static str) -> Result<std::process::Output, GuestFailure> {
    let output = Command::new(command.program)
        .args(&command.arguments)
        .output()
        .map_err(|error| GuestFailure::new(stage, format!("{} 起不来：{error}", command.program)))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(GuestFailure::new(
            stage,
            format!(
                "{} {} 退出 {:?}：{}",
                command.program,
                command.arguments.join(" "),
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}
