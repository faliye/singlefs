//! X2：测试经 CARGO_BIN_EXE_model-tool 起的另一个二进制。
fn main() {
    let argument = std::env::args().nth(1).expect("一个参数");
    println!("{}", mcore::tool::checksum(argument.as_bytes()));
    println!("{}", mcore::tool::digest_only_in_tool(argument.as_bytes()));
}
