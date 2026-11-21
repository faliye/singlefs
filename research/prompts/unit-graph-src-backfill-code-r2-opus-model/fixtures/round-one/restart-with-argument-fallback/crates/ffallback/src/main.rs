fn main() {
    let this_program = std::env::current_exe().unwrap_or_else(|_| {
        if let Some(first_argument) = std::env::args_os().next() {
            std::path::PathBuf::from(first_argument)
        } else {
            panic!("拿不到程序路径")
        }
    });
    if std::env::args().count() > 1 {
        return;
    }
    std::process::Command::new(&this_program)
        .arg("role")
        .status()
        .unwrap();
}
