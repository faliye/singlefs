//! X2：只在另一个二进制里执行到 checksum 的强测试，加一条进程内的弱测试。
#[test]
fn checksum_stays_below_the_modulus() {
    assert!(mcore::tool::checksum(b"") < 65521);
}

fn tool_lines() -> Vec<String> {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_model-tool"))
        .arg("abc")
        .output()
        .expect("起 model-tool");
    String::from_utf8(out.stdout).expect("UTF-8").lines().map(str::to_string).collect()
}

#[test]
fn the_tool_binary_prints_the_checksum() {
    assert_eq!(tool_lines()[0], "30833");
}

/// X2b：digest_only_in_tool 只在 model-tool 里执行。
#[test]
fn the_tool_binary_prints_the_digest() {
    assert_eq!(tool_lines()[1], "882");
}
