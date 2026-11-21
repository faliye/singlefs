fn worker_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap().with_file_name("worker")
}

fn main() {
    let exe = worker_path();
    {
        let exe = std::env::current_exe().unwrap();
        eprintln!("launcher at {}", exe.display());
    }
    std::process::Command::new(&exe).status().unwrap();
}
