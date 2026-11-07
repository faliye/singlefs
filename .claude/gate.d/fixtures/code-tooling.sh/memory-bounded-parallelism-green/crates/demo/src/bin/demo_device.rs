fn main() {
    let free_threads = 4;
    let free_bytes: u64 = 8 << 30;
    let started = start_group(free_threads, free_bytes);
    println!("{started}");
}

fn start_group(threads: usize, budget: u64) -> usize {
    threads.min((budget >> 30) as usize)
}
