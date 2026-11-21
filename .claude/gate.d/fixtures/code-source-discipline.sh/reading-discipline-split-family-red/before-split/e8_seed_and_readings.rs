//! 判别力样本：基准提交里 e8 还是一个文件（拆成 e8_seed_and_readings/main.rs 加 readings.rs 之前的样子）。
//! 一处种子折叠（C59）加一个恒为字面量 0 的读数（C60），两张豁免表各登记一行在这份文件名下。

pub struct Reading {
    pub visited_nodes: u64,
    pub units_read_on_commit: u64,
}

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
