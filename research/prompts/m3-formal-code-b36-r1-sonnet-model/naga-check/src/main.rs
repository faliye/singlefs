// 解析并校验 WGSL 源码（与 crates/singlefs-checker-tier/src/crash_judge_gpu.rs 单测 parse_and_validate 同一套 naga 调用）；用法 naga-check <文件>
fn main() {
    let path = std::env::args().nth(1).expect("文件");
    let source = std::fs::read_to_string(&path).expect("读源码");
    let module = match naga::front::wgsl::parse_str(&source) {
        Ok(m) => m,
        Err(e) => { println!("PARSE-FAIL {path}\n{}", e.emit_to_string(&source)); std::process::exit(2); }
    };
    let mut validator = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all());
    match validator.validate(&module) {
        Ok(_) => println!("OK {path} functions={} entry_points={}", module.functions.len(), module.entry_points.len()),
        Err(e) => { println!("VALIDATE-FAIL {path}\n{}", e.emit_to_string(&source)); std::process::exit(3); }
    }
}
