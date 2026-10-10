// 攻方腿 m3-batch10t-code-r1 K4-2：扫源码检查的词法剥离在哪种写法上看漏。上面接的是冻结副本
// crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs 第 21–30 行
// （起卡函数名）、第 62–68 行（BypassingLine）、第 89–424 行（剥离与找名字的几个函数）原样拷过来（build_scan_bypass_test.sh 拼）。
// 每个样本打一行 `MODEL scenario=k4-2 sample=… lines_found=…`：今天的扫法找到的行号（空就是看漏了）。

fn report(sample_name: &str, lines: &[(usize, String)]) {
    let found: Vec<usize> = lines_mentioning("tests/sample.rs", lines, &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY)
        .iter()
        .map(|bypass| bypass.line_number)
        .collect();
    println!("MODEL scenario=k4-2 sample={sample_name} lines_found={found:?}");
}

#[test]
fn k4_2_samples() {
    // 对照：普通调用，扫得出（第 2 行）。
    report("plain_call", &code_lines("fn takes_cards() {\n    let _cards = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards();\n}\n"));
    // 原始 C 字符串 `cr"\"`：`c` 之后的 `r` 跟在名字字符后面，剥离把它当普通字符串、把 `\` 当转义，字符串一直吞到下一个 `"`。
    report("raw_c_string_with_a_backslash", &code_lines("fn takes_cards() {\n    let _s = cr\"\\\"; let _cards = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards(); let _t = \"\";\n}\n"));
    // 原始 C 字符串带 `#`：内容里有 `"`，剥离在那个 `"` 收尾，后面的 `b"#…` 又开了一个字节串。
    report("raw_c_string_with_hashes", &code_lines("fn takes_cards() {\n    let _s = cr#\"x\"b\"#; let _cards = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards(); let _t = \"\";\n}\n"));
    // 库源码里 `#[cfg(test)] mod gpu_tests;` 把测试放进另一份文件：那份文件里没有 cfg(test)，按产品代码不扫。
    let (module_file_lines, items) = test_code_lines_of_a_library_source(&code_lines(
        "#[test]\nfn takes_cards() {\n    let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n}\n",
    ));
    println!("MODEL scenario=k4-2 sample=out_of_line_test_module_file test_items_collected={items}");
    report("out_of_line_test_module_file", &module_file_lines);
}
