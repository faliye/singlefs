
    // ── 攻方腿草稿（device-library-write-many-code-r3）：接在 mod tests 末尾，被攻过零轮。

    #[test]
    fn in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {
        if is_the_child_half() {
            DeviceLibrary::in_memory().write("items/bad key.out", "x");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "内存那一种的单条 write 与转交那一种核同一条相对键规则；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn read_answered_failed_exits_with_the_code_in_the_reply() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
                .read("items/a.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_failed_exits_with_the_code_in_the_reply",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(75),
            "回 failed 75 的读照 75 退，不当键不在；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes() {
        let counterpart = relay_with_prepared_replies(b"found 2\n\xff\xfeabsent\n");
        assert_eq!(
            counterpart.device_library.read("items/not-utf8.out"),
            None,
            "found 后面的内容不是 UTF-8 时交 None（文件头「found 后面的内容不是 UTF-8 时 read 交 None」）"
        );
        assert_eq!(
            counterpart.device_library.read("items/missing.out"),
            None,
            "下一条回应 absent 照常读成不在：非 UTF-8 的两个字节恰好读走"
        );
        assert_eq!(
            requests_after_the_device_is_dropped(counterpart),
            "read items/not-utf8.out\nread items/missing.out\n"
        );
    }

    /// 本进程此刻的常驻内存（/proc/self/status 的 VmRSS，字节）。
    fn resident_bytes_of_this_process() -> usize {
        let status = std::fs::read_to_string("/proc/self/status").expect("读得了 /proc/self/status");
        let line = status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .expect("/proc/self/status 里有 VmRSS 一行");
        let kibibytes: usize = line
            .trim_start_matches("VmRSS:")
            .trim()
            .trim_end_matches("kB")
            .trim()
            .parse()
            .expect("VmRSS 是 kB 数");
        kibibytes * 1024
    }

    #[test]
    fn read_answered_found_touches_no_more_memory_than_the_contents_that_arrived() {
        const ANNOUNCED_BYTES: usize = 32 * 1024 * 1024;
        const CHUNK_BYTES: usize = 64 * 1024;
        let (_requests_for_the_counterpart, request_end_of_the_device) =
            std::io::pipe().expect("建请求管道");
        let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
            std::io::pipe().expect("建回应管道");
        let device_library = DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
                requests: std::fs::File::from(std::os::fd::OwnedFd::from(request_end_of_the_device)),
                replies: std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
                    reply_end_of_the_device,
                ))),
            }))),
        };
        reply_end_of_the_counterpart
            .write_all(format!("found {ANNOUNCED_BYTES}\n").as_bytes())
            .expect("回应管道写得进");
        let resident_before = resident_bytes_of_this_process();
        let reader = {
            let device_library = device_library.clone();
            std::thread::spawn(move || device_library.read("items/a.out").map(|text| text.len()))
        };
        // 内容一个字节都还没写：装置线程这时卡在读内容上。等到常驻内存涨过报的一半，或等满三秒。
        let started = std::time::Instant::now();
        let mut growth_while_no_contents_arrived = 0_usize;
        while started.elapsed() < std::time::Duration::from_secs(3) {
            growth_while_no_contents_arrived =
                resident_bytes_of_this_process().saturating_sub(resident_before);
            if growth_while_no_contents_arrived >= ANNOUNCED_BYTES / 2 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let chunk = vec![b'v'; CHUNK_BYTES];
        for _ in 0..ANNOUNCED_BYTES / CHUNK_BYTES {
            reply_end_of_the_counterpart.write_all(&chunk).expect("回应管道写得进");
        }
        assert_eq!(reader.join().expect("装置线程照常收尾"), Some(ANNOUNCED_BYTES));
        println!(
            "name=resident_growth_before_contents announced={ANNOUNCED_BYTES} growth={growth_while_no_contents_arrived}"
        );
        assert!(
            growth_while_no_contents_arrived < ANNOUNCED_BYTES / 2,
            "found 报了 {ANNOUNCED_BYTES} 字节、内容一个字节都没来，常驻内存已经涨了 {growth_while_no_contents_arrived} 字节：内容来之前就把报的字节数整段写零"
        );
    }
