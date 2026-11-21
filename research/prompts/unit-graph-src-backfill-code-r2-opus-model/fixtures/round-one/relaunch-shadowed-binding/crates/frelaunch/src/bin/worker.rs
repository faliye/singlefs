fn worker_role() -> u32 {
    3
}

fn main() {
    std::process::exit(worker_role() as i32);
}
