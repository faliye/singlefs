// 正推腿自己的模型：把 checker 档几条扫源码用例的逻辑原样搬出来，读运行时给的源文件路径（CA_SRC、GU_SRC），不依赖 checker 档包。
#![allow(clippy::all, dead_code, unused)]
fn ca_source() -> String { std::fs::read_to_string(std::env::var("CA_SRC").expect("CA_SRC")).expect("read") }
fn gu_source() -> String { std::fs::read_to_string(std::env::var("GU_SRC").expect("GU_SRC")).expect("read") }
mod ca {
    use super::ca_source;
    const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
    /// 顶格的右括号自成一行（测试模块在这里收尾）。
    const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";
    fn code_lines_outside_the_tests() -> Vec<String> {
        let source_owned = ca_source(); let source = source_owned.as_str();
        let tests_start = source
            .find(TESTS_MODULE_OPENING)
            .expect("这份文件有测试模块");
        let tests_end = tests_start
            + source[tests_start..]
                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("测试模块在顶格的右括号处收尾");
        format!("{}{}", &source[..tests_start], &source[tests_end..])
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .map(str::to_string)
            .collect()
    }
    #[test]
    fn the_product_path_prints_the_not_run_line_of_a_lease_it_could_not_get() {
        let lines = code_lines_outside_the_tests();
        let start = lines
            .iter()
            .position(|line| line.trim() == "fn print_the_not_run_line_or_refuse(")
            .expect("产品路径有打印借不到那一行的那一半");
        let body = lines[start..]
            .iter()
            .take_while(|line| line.as_str() != "\u{7d}")
            .map(|line| line.trim().to_string())
            .collect::<Vec<String>>();
        let line_made = body
            .iter()
            .position(|line| {
                line == "let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;"
            })
            .unwrap_or_else(|| panic!("打印那一半先照不打印的那一半造那一行：{body:?}"));
        let line_printed = body
            .iter()
            .position(|line| line == "println!(\"{not_run_line}\");")
            .unwrap_or_else(|| panic!("打印那一半要把造出来的那一行打进标准输出：{body:?}"));
        let line_handed_back = body
            .iter()
            .position(|line| line == "Ok(not_run_line)")
            .unwrap_or_else(|| panic!("打印那一半交回造出来的那一行：{body:?}"));
        assert!(
            line_made < line_printed && line_printed < line_handed_back,
            "先造、再打、再交回：{body:?}"
        );
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.contains("print_the_not_run_line_or_refuse("))
                .count(),
            4,
            "产品路径的三处借不到都经打印的那一半（加上它自己的定义一处）"
        );
    }
    #[test]
    fn the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line() {
        let lines = code_lines_outside_the_tests();
        let arm = lines
            .iter()
            .position(|line| line.trim() == "AfterARoundOfTheCards::StoppedOnRequest => \u{7b}")
            .expect("第 ① 段几轮卡的收尾里有照叫停收尾那一支");
        let arm_body = lines[arm..]
            .iter()
            .take_while(|line| line.trim() != "break;")
            .cloned()
            .collect::<Vec<String>>()
            .join("\n");
        assert!(
            arm_body.contains(".not_run_line_of_a_stop_after_an_out_of_memory(")
                && arm_body.contains("PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE")
                && arm_body
                    .lines()
                    .any(|line| line.trim() == "println!(\"{not_run_line}\");"),
            "照叫停收尾那一支要把头一句 Out of Memory 那一行打进标准输出（整行比，`eprintln!` 不算）：{arm_body}"
        );
    }
    #[test]
    fn the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run(
    ) {
        let lines: Vec<String> = code_lines_outside_the_tests()
            .iter()
            .map(|line| line.trim().to_string())
            .collect();
        let count = |expected: &str| {
            lines
                .iter()
                .filter(|line| line.as_str() == expected)
                .count()
        };
        for (expected, what) in [
            (
                concat!("let stop = stop_request.", "stop_of_the_run();"),
                "一轮收尾问这一趟记住的叫停（A）",
            ),
            (
                concat!("&|| work.", "stop_of_the_cards(),"),
                "卡线程判块那一圈问卡上的叫停（C）",
            ),
            (
                concat!(
                    "let stop_is_requested = || stop_",
                    "request.is_requested();"
                ),
                "整趟收尾问的是记住的那一个（D）",
            ),
            (
                concat!(
                    "let stop_was_requested = gpu_memory_was_short || stop_",
                    "is_requested();"
                ),
                "整趟收尾照它判提早收尾（D）",
            ),
        ] {
            assert_eq!(count(expected), 1, "{what}：要有一行 {expected}");
        }
        let start = lines
            .iter()
            .position(|line| line.starts_with("fn start_the_next_card_with_states<"))
            .expect("有起下一张卡的那一个函数");
        let next_card_body: Vec<&String> = lines[start..]
            .iter()
            .take_while(|line| line.as_str() != "\u{7d}")
            .collect();
        assert!(
            next_card_body
                .iter()
                .any(|line| line.as_str() == concat!("work.", "stop_of_the_cards()")),
            "起下一张卡照卡上的叫停问（B）：{next_card_body:?}"
        );
    }
}
mod gu {
    use super::gu_source;
    const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
    /// 顶格的右括号自成一行（函数体在这里收尾）。
    const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";
    #[test]
    fn the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first() {
        let source_owned = gu_source(); let source = source_owned.as_str();
        let code = source
            .split(TESTS_MODULE_OPENING)
            .next()
            .expect("split 至少交一段");
        let body_until = |signature: &str, closing_line: &str| -> String {
            let start = code
                .find(signature)
                .unwrap_or_else(|| panic!("源码里有 {signature}"));
            let after = &code[start + signature.len()..];
            after[..after
                .find(closing_line)
                .unwrap_or_else(|| panic!("{signature} 收得了尾"))]
                .to_string()
        };
        let drop_body = body_until("impl Drop for GpuCardLease \u{7b}", "\n        \u{7d}\n");
        for expected in [
            "wait_for_the_device_and_take_in_the_outcome(",
            "&mut self.booking,",
            "&self.device_on_the_books,",
            "|| device.poll(poll_type_before_returning_a_lease()),",
            "device_wait_before_returning_a_lease,",
        ] {
            assert_eq!(
                drop_body.matches(expected).count(),
                1,
                "借据丢掉时那一步要有一处 {expected}：{drop_body}"
            );
        }
        for entry in [
            "fn lease_card(",
            "pub fn lease_the_first_card_with_room_now(",
            "pub fn book_the_scratch_of_the_cards_with_room_now(",
            "pub fn card_rooms_now_and_when_empty_of(",
        ] {
            let body = body_until(entry, "\n            \u{7d}\n");
            let polled = body
                .find("self.take_in_the_devices_found_idle();")
                .unwrap_or_else(|| panic!("{entry} 借之前先 poll 没等成的那几张卡：{body}"));
            let booked = body
                .find("ledger")
                .unwrap_or_else(|| panic!("{entry} 照账借：{body}"));
            assert!(polled < booked, "{entry} 先 poll、再照账借：{body}");
        }
    }
}
