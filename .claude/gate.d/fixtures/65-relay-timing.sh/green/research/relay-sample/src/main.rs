//! 判别力样本（绿）：辅助函数先把子进程输出读到 EOF、只记每行到达时刻，调用方读完再转打；另有一个读 /proc/self/io 的 lines() 循环。
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

struct ArrivedLine {
    at_nanoseconds: u128,
    text: String,
}

fn read_lines_with_arrival_times<Reader: BufRead>(reader: Reader, started: Instant) -> std::io::Result<Vec<ArrivedLine>> {
    let mut arrived_lines = Vec::new();
    for line in reader.lines() {
        let text = line?;
        arrived_lines.push(ArrivedLine { at_nanoseconds: started.elapsed().as_nanos(), text });
    }
    Ok(arrived_lines)
}

fn run_child_then_relay_lines() {
    let started = Instant::now();
    let mut child = Command::new("child-binary").stdout(Stdio::piped()).spawn().expect("子进程起得来");
    let child_output = child.stdout.take().expect("刚用 Stdio::piped() 起的子进程一定有 stdout");
    let arrived_lines = read_lines_with_arrival_times(BufReader::new(child_output), started).expect("读得到 EOF");
    child.wait().expect("子进程收得了尾");
    for arrived_line in arrived_lines {
        println!("at_nanoseconds={} {}", arrived_line.at_nanoseconds, arrived_line.text);
    }
}

fn print_process_io_counters_with_timestamps() {
    let started = Instant::now();
    let counter_text = std::fs::read_to_string("/proc/self/io").expect("读得到 /proc/self/io");
    for line in counter_text.lines() {
        println!("at_nanoseconds={} {line}", started.elapsed().as_nanos());
    }
}

fn main() {
    run_child_then_relay_lines();
    print_process_io_counters_with_timestamps();
}
