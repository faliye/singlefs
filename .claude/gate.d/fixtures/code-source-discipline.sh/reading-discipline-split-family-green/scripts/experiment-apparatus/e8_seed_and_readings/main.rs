//! 判别力样本：e8 拆成入口 main.rs 加子模块 readings.rs 之后的样子（基准提交里它还是 scripts/experiment-apparatus/e8_seed_and_readings.rs 一个文件）。
//! 种子折叠（C59）在这份文件里；恒为字面量 0 的读数（C60）的字段声明在 readings.rs 里，初始化在这份文件里。两张豁免表的登记行跟着挪到这两份文件名下。

mod readings;
use readings::Reading;

fn next_random_word(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn run(seed: u64, round_count: u64) -> Reading {
    let mut random_state = seed | 1;
    let mut visited_nodes = 0u64;
    for _ in 0..round_count {
        visited_nodes += next_random_word(&mut random_state) % 8;
    }
    Reading { visited_nodes, units_read_on_commit: 0 }
}

fn main() {
    let reading = run(2, 16);
    println!("visited={} units_read={}", reading.visited_nodes, reading.units_read_on_commit);
}
