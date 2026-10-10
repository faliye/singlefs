//! 攻方小程序（m3-batch10z-lm-code-r1）：把第十三块两条扫源码用例的判法逐字搬过来，读的源码换成命令行给的文件。
//! 用法：scan_probe gpu <gpu_unit_checks.rs>｜scan_probe crash <crash_amplification.rs>；绿退 0，红退 1。
const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";

fn code_lines_outside_the_tests(source: &str) -> Vec<String> {
    let tests_start = source.find(TESTS_MODULE_OPENING).expect("这份文件有测试模块");
    let tests_end = tests_start + source[tests_start..].find(TOP_LEVEL_CLOSING_BRACE_LINE).expect("测试模块在顶格的右括号处收尾");
    format!("{}{}", &source[..tests_start], &source[tests_end..])
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_string)
        .collect()
}

fn whole_name_uses_of_the_stop_request(line: &str) -> usize {
    let name = concat!("stop_is_", "requested");
    let is_part_of_a_name = |neighbour: Option<char>| {
        neighbour.is_some_and(|neighbour_character| neighbour_character.is_alphanumeric() || neighbour_character == '_')
    };
    line.match_indices(name)
        .filter(|(position, _name)| {
            !is_part_of_a_name(line[..*position].chars().next_back())
                && !is_part_of_a_name(line[position + name.len()..].chars().next())
        })
        .count()
}

fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it(source: &str) {
    let stop_parameter = concat!("stop_is_", "requested: &(dyn Fn() -> bool + Sync),");
    let lines = code_lines_outside_the_tests(source);
    let mut layers_handed_the_stop_request = Vec::new();
    for (parameter_line_index, _parameter_line) in lines.iter().enumerate().filter(|(_line_index, line)| line.trim() == stop_parameter) {
        let signature = lines[..parameter_line_index].iter().rev().find(|line| line.contains("fn ")).expect("参数行之前有函数签名");
        let layer = signature.split("fn ").nth(1).and_then(|after_fn| after_fn.split(['(', '<']).next()).expect("签名里 fn 之后是函数名").to_string();
        let rest_of_the_layer: Vec<&String> = lines[parameter_line_index + 1..].iter().take_while(|line| line.as_str() != "\u{7d}").collect();
        let uses: usize = rest_of_the_layer.iter().map(|line| whole_name_uses_of_the_stop_request(line)).sum();
        assert!(uses >= 1, "{layer} 收了叫停却没用上");
        layers_handed_the_stop_request.push(layer);
    }
    assert_eq!(
        layers_handed_the_stop_request,
        vec!["check_blocks_where_until", "check_pending_blocks_in_pieces", "judge_my_blocks_on_gpu_cards",
             "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from", "check_selected_blocks", "judge_my_blocks_on_gpu"],
        "收叫停的是这六层"
    );
}

fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking(source: &str) {
    let code = source.split(TESTS_MODULE_OPENING).next().expect("split 至少交一段");
    let signature = "fn take_in_the_devices_found_idle(&self) \u{7b}";
    let start = code.find(signature).unwrap_or_else(|| panic!("源码里有 {signature}"));
    let after = &code[start + signature.len()..];
    let body = &after[..after.find("\n            \u{7d}\n").expect("借之前那一步以缩进十二格的右括号收尾")];
    let body_without_comments_and_spaces: String = body.lines().filter(|line| !line.trim_start().starts_with("//")).flat_map(str::chars).filter(|character| !character.is_whitespace()).collect();
    for (expected, what) in [
        ("self.ledger.take_in_the_devices_kept_before_booking(", "整个交给账的那一步"),
        ("|card_position|self.cards[card_position].started.started_now(),", "列出来那一格留着的设备，不等正在起的"),
        ("|started:&GpuCard|started.device_condition().is_lost(),", "丢没丢照设备的情况"),
        ("|started:&GpuCard|{device_found_idle_by_a_poll(&std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{started.device().poll(wgpu::PollType::Poll)}),))},", "poll 一次不等、接住 panic、照队列空了判"),
        ("|started:&GpuCard|{started.device_on_the_books.as_deref().expect(", "账上的那一台是这台设备自己的"),
    ] {
        assert_eq!(body_without_comments_and_spaces.matches(expected).count(), 1, "借之前那一步要有一处 {expected}（{what}）");
    }
    assert_eq!(body_without_comments_and_spaces.matches("|started:&GpuCard|").count(), 3, "取到设备之后的三样各一个闭包、不多不少");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = std::fs::read_to_string(&args[2]).expect("读源码");
    let mode = args[1].clone();
    let outcome = std::panic::catch_unwind(move || match mode.as_str() {
        "gpu" => the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking(&source),
        "crash" => every_layer_handed_the_stop_request_hands_it_on_or_asks_it(&source),
        other => panic!("模式 {other}"),
    });
    match outcome {
        Ok(()) => { println!("SCAN_GREEN {} {}", args[1], args[2]); }
        Err(_) => { println!("SCAN_RED {} {}", args[1], args[2]); std::process::exit(1); }
    }
}
