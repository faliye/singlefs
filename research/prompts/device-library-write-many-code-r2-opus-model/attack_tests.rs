
    // ── 攻方腿（device-library-write-many-code-r2）加的几条，只进攻方腿拷出来的树。
    //    observed_ 起头的断言现状（现状下绿就是观测成立）；别的按文件头协议断言该有的样子（现状下红就是打中）；
    //    catch_rate_ 与 listing_ 起头的只在设了各自的环境变量时干活，没设时立刻绿。

    #[test]
    fn observed_read_with_a_newline_in_the_key_sends_a_second_request_after_the_read() {
        let counterpart = relay_with_prepared_replies(b"absent\n");
        assert_eq!(
            counterpart
                .device_library
                .read("items/a.out\nwrite items/injected.out 0"),
            None,
            "转交回的头一行是 absent，读成键不在"
        );
        assert_eq!(
            requests_after_the_device_is_dropped(counterpart),
            "read items/a.out\nwrite items/injected.out 0\n",
            "read 不核相对键：键里的换行原样上管道，转交那一侧读出两条各自合规的请求"
        );
    }

    #[test]
    fn observed_in_memory_read_with_a_key_the_relay_refuses_reads_as_missing() {
        let device_library = DeviceLibrary::in_memory();
        assert_eq!(device_library.read("items/bad key.out"), None, "带空格的键：内存那一种读成不在、不退");
        assert_eq!(device_library.read(""), None, "空键：内存那一种读成不在、不退");
    }

    #[test]
    fn observed_read_answered_found_with_bytes_that_are_not_utf8_reads_as_missing() {
        let counterpart = relay_with_prepared_replies(b"found 2\n\xff\xfe");
        assert_eq!(
            counterpart.device_library.read("items/a.out"),
            None,
            "found 之后的内容不是 UTF-8：交 None（读成不在、现判），不照退出码退"
        );
    }

    #[test]
    fn read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(format!("found {}\n", 1_usize << 62).as_bytes())
                .read("items/a.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "found 报 2^62 字节（不过 isize::MAX、分配不出）：文件头「回应认不出退 3」；标准错误：{}",
            outcome.standard_error
        );
    }

    /// ATTACK_PROBE_SAMPLES=<同时几份>x<几批> 设了时：照 a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply
    /// 原样调「几份同时 × 几批」次，数它红了几次，写进 ATTACK_PROBE_OUT（一行 probe_samples=<共几次> probe_red=<红几次> concurrent=<份> batches=<批>）。
    #[test]
    fn catch_rate_of_the_read_lock_probe() {
        let (Some(samples), Some(out)) = (
            std::env::var_os("ATTACK_PROBE_SAMPLES"),
            std::env::var_os("ATTACK_PROBE_OUT"),
        ) else {
            return;
        };
        let samples = samples.into_string().expect("ATTACK_PROBE_SAMPLES 是 <份>x<批>");
        let (concurrent, batches) = samples.split_once('x').expect("ATTACK_PROBE_SAMPLES 是 <份>x<批>");
        let concurrent: usize = concurrent.parse().expect("份数是正整数");
        let batches: usize = batches.parse().expect("批数是正整数");
        let mut red = 0_usize;
        for _batch in 0..batches {
            let probes: Vec<_> = (0..concurrent)
                .map(|_| std::thread::spawn(a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply))
                .collect();
            for probe in probes {
                if probe.join().is_err() {
                    red += 1;
                }
            }
        }
        std::fs::write(
            out,
            format!(
                "probe_samples={} probe_red={red} concurrent={concurrent} batches={batches}\n",
                concurrent * batches
            ),
        )
        .expect("ATTACK_PROBE_OUT 写得进");
    }

    /// ATTACK_KEY_RULE_LISTING 设了时：把 is_relative_key_the_relay_accepts 收的单个字符（码位十六进制，一行一个）与几条边界键
    /// （「sample <UTF-8 字节十六进制> <收不收>」）写进那个文件，交 key_rules_cross_check.py 与 device-code-key.py 的 RELATIVE_KEY_PATTERN 比。
    #[test]
    fn listing_of_what_the_key_rule_accepts() {
        let Some(out) = std::env::var_os("ATTACK_KEY_RULE_LISTING") else {
            return;
        };
        let mut text = String::new();
        for code in 0_u32..=0x10FFFF {
            if let Some(character) = char::from_u32(code) {
                if is_relative_key_the_relay_accepts(&character.to_string()) {
                    text.push_str(&format!("{code:x}\n"));
                }
            }
        }
        let long_key = "a".repeat(4096);
        let samples: [&str; 16] = [
            "", ".", "..", "/", "/abs", "a//b", "../x", "items/..", "-", "a b", "a\u{a0}b", "\u{e9}", "\u{663}", "a\r", "a\n",
            long_key.as_str(),
        ];
        for sample in samples {
            let hexadecimal: String = sample.bytes().map(|byte| format!("{byte:02x}")).collect();
            text.push_str(&format!(
                "sample {} {}\n",
                if hexadecimal.is_empty() { "-".to_string() } else { hexadecimal },
                is_relative_key_the_relay_accepts(sample)
            ));
        }
        std::fs::write(out, text).expect("ATTACK_KEY_RULE_LISTING 写得进");
    }
