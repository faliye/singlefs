//! 判别力样本（红）：照 E152 旧写法，读子进程输出的循环里一边给行打时间戳一边转打。
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

fn run_child_and_relay_lines() {
    let started = Instant::now();
    let mut child = Command::new("child-binary").stdout(Stdio::piped()).spawn().expect("子进程起得来");
    let child_output = child.stdout.take().expect("刚用 Stdio::piped() 起的子进程一定有 stdout");
    for line in BufReader::new(child_output).lines() {
        let line = line.expect("读得出一行");
        let at_nanoseconds = started.elapsed().as_nanos();
        println!("at_nanoseconds={at_nanoseconds} {line}");
    }
    child.wait().expect("子进程收得了尾");
}

fn main() {
    run_child_and_relay_lines();
}
