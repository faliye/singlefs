// 第 10z 批第十一块 checker 档扫源码两条用例的逐字搬运（只把 include_str! 换成读文件、断言换成交 Err）。
const TESTS_MODULE_OPENING_GPU: &str = "#[cfg(test)]\nmod tests \u{7b}";
const TESTS_MODULE_OPENING_CRASH: &str = "#[cfg(test)]\nmod tests \u{7b}";
const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";

fn gpu(source: &str) -> Result<(), String> {
    let code = source.split(TESTS_MODULE_OPENING_GPU).next().expect("split 至少交一段");
    let body_until = |signature: &str, closing_line: &str| -> Result<String, String> {
        let start = code.find(signature).ok_or_else(|| format!("源码里有 {signature}"))?;
        let after = &code[start + signature.len()..];
        Ok(after[..after.find(closing_line).ok_or_else(|| format!("{signature} 收得了尾"))?].to_string())
    };
    let drop_body = body_until("impl Drop for GpuCardLease \u{7b}", "\n        \u{7d}\n")?;
    for expected in [
        "wait_for_the_device_and_take_in_the_outcome(",
        "&mut self.booking,",
        "&self.device_on_the_books,",
        "|| device.poll(poll_type_before_returning_a_lease()),",
        "device_wait_before_returning_a_lease,",
    ] {
        if drop_body.matches(expected).count() != 1 {
            return Err(format!("借据丢掉时那一步要有一处 {expected}"));
        }
    }
    for entry in [
        "fn lease_card(",
        "pub fn lease_the_first_card_with_room_now(",
        "pub fn book_the_scratch_of_the_cards_with_room_now(",
        "pub fn card_rooms_now_and_when_empty_of(",
    ] {
        let body = body_until(entry, "\n            \u{7d}\n")?;
        let code_lines: Vec<&str> = body.lines().filter(|line| !line.trim_start().starts_with("//")).collect();
        let polled = code_lines
            .iter()
            .position(|line| *line == "                self.take_in_the_devices_found_idle();")
            .ok_or_else(|| format!("{entry} 借之前先 poll 没等成的那几张卡（方法体顶层整行一句）"))?;
        let booked = code_lines
            .iter()
            .position(|line| line.contains("ledger"))
            .ok_or_else(|| format!("{entry} 照账借"))?;
        if !(polled < booked) {
            return Err(format!("{entry} 先 poll、再照账借"));
        }
    }
    Ok(())
}

fn code_lines_outside_the_tests(source: &str) -> Vec<String> {
    let tests_start = source.find(TESTS_MODULE_OPENING_CRASH).expect("这份文件有测试模块");
    let tests_end = tests_start + source[tests_start..].find(TOP_LEVEL_CLOSING_BRACE_LINE).expect("测试模块在顶格的右括号处收尾");
    format!("{}{}", &source[..tests_start], &source[tests_end..])
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_string)
        .collect()
}

fn crash(source: &str) -> Result<(), String> {
    let lines: Vec<String> = code_lines_outside_the_tests(source).iter().map(|line| line.trim().to_string()).collect();
    for (call_opening, what) in [
        (concat!("(ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu", "("), "第 ① 段在卡上判"),
        (concat!("let checking = check_selected_blocks", "("), "第 ① 段 CPU 判器"),
    ] {
        let openings: Vec<usize> = lines.iter().enumerate().filter(|(_i, line)| line.as_str() == call_opening).map(|(i, _)| i).collect();
        if openings.len() != 1 {
            return Err(format!("{what}：这份源码里有一处 {call_opening}（实际 {}）", openings.len()));
        }
        let arguments: Vec<&String> = lines[openings[0] + 1..].iter().take_while(|line| !line.starts_with(')')).collect();
        if arguments.last().map(|argument| argument.as_str()) != Some(concat!("&stop_is_", "requested,")) {
            return Err(format!("{what}收到的叫停是这一趟记住的那一个：最后一个参数是 {:?}", arguments.last()));
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = std::fs::read_to_string(&args[2]).expect("读得了源文件");
    let outcome = match args[1].as_str() {
        "gpu" => gpu(&source),
        "crash" => crash(&source),
        other => panic!("{other}"),
    };
    match outcome {
        Ok(()) => println!("GREEN"),
        Err(why) => println!("RED {why}"),
    }
}
