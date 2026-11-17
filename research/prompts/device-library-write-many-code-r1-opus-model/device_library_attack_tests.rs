
    // ── 攻方腿加的测试（device-library-write-many-code-r1）：插在 mod tests 末尾的右花括号之前 ──

    /// 子进程那一半的标记：设了它，点名的测试只做「接上预先写好的回应、调一次」，调完还没退就退 0。
    const ATTACK_CHILD_VARIABLE: &str = "DEVICE_LIBRARY_ATTACK_CHILD";

    /// 照测试名再起一次本测试二进制、只跑那一条的子进程那一半，交它的退出码（give_up 退进程，同一进程里测不了）。
    fn exit_code_of_the_child_half(test_name: &str) -> Option<i32> {
        std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .args([test_name, "--exact", "--nocapture", "--test-threads", "1"])
            .env(ATTACK_CHILD_VARIABLE, "1")
            .status()
            .expect("起得了子进程")
            .code()
    }

    #[test]
    fn write_many_answered_failed_exits_with_the_code_in_the_reply() {
        if std::env::var_os(ATTACK_CHILD_VARIABLE).is_some() {
            let counterpart = relay_with_prepared_replies(b"failed 75 prune-server stopped\n");
            counterpart
                .device_library
                .write_many(&[("items/a.out".to_string(), "x".to_string())]);
            std::process::exit(0);
        }
        assert_eq!(
            exit_code_of_the_child_half(
                "tests::write_many_answered_failed_exits_with_the_code_in_the_reply"
            ),
            Some(75),
            "回 failed 75 的一批照 75 退，不当写成了"
        );
    }

    #[test]
    fn write_answered_failed_exits_with_the_code_in_the_reply() {
        if std::env::var_os(ATTACK_CHILD_VARIABLE).is_some() {
            let counterpart = relay_with_prepared_replies(b"failed 75 prune-server stopped\n");
            counterpart.device_library.write("items/a.out", "x");
            std::process::exit(0);
        }
        assert_eq!(
            exit_code_of_the_child_half("tests::write_answered_failed_exits_with_the_code_in_the_reply"),
            Some(75),
            "回 failed 75 的单条写照 75 退，不当写成了"
        );
    }

    /// 观测（不是要求）：内存那一种收下转交那一侧会整批拒的相对键（带空格），单测里绿的装置真跑会退 3。
    #[test]
    fn in_memory_write_many_takes_a_key_the_relay_refuses() {
        let device_library = DeviceLibrary::in_memory();
        device_library.write_many(&[("items/bad key.out".to_string(), "x".to_string())]);
        assert_eq!(
            device_library.read("items/bad key.out").as_deref(),
            Some("x"),
            "内存那一种不核相对键"
        );
    }

    /// 登记的每台默认重活线程数：SINGLEFS_THREAD_CAP 设了照它，没设读仓根 .claude/heavy-thread-default 第一条非注释行。
    fn registered_heavy_threads() -> usize {
        if let Some(text) = std::env::var_os("SINGLEFS_THREAD_CAP") {
            return text.to_string_lossy().trim().parse().expect("SINGLEFS_THREAD_CAP 是正整数");
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.claude/heavy-thread-default");
        let text = std::fs::read_to_string(path).expect("读得到登记的默认重活线程数");
        text.lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .and_then(|line| line.split_whitespace().next())
            .and_then(|number| number.parse().ok())
            .expect("登记行以正整数起头")
    }

    /// 转交那一侧的替身（自己一个线程）：照协议逐个读请求、逐个回；read 回 found 加「键本身」，write 与 write-many 核每条内容等于键、回 written。
    /// 请求管道读到尽头交回写过的键（按到达次序）。
    fn serve_like_the_relay(
        requests: std::io::PipeReader,
        mut replies: std::io::PipeWriter,
    ) -> Vec<String> {
        use std::io::BufRead;
        let mut requests = std::io::BufReader::new(requests);
        let mut written = Vec::new();
        let mut read_entry = |requests: &mut std::io::BufReader<std::io::PipeReader>,
                              key: &str,
                              length: &str| {
            let mut data = vec![0_u8; length.parse().expect("字节数")];
            requests.read_exact(&mut data).expect("内容读得全");
            assert_eq!(data, key.as_bytes(), "{key} 的内容整条到达、没被别的线程的请求插进来");
            written.push(key.to_string());
        };
        loop {
            let mut line = String::new();
            if requests.read_line(&mut line).expect("读请求") == 0 {
                drop(read_entry);
                return written;
            }
            let words: Vec<String> = line.trim_end_matches('\n').split(' ').map(String::from).collect();
            match words.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
                ["read", key] => {
                    replies
                        .write_all(format!("found {}\n{key}", key.len()).as_bytes())
                        .expect("回得进");
                }
                ["write", key, length] => {
                    read_entry(&mut requests, key, length);
                    replies.write_all(b"written\n").expect("回得进");
                }
                ["write-many", count] => {
                    for _ in 0..count.parse::<usize>().expect("条数") {
                        let mut header = String::new();
                        requests.read_line(&mut header).expect("读条头");
                        let (key, length) = header
                            .trim_end_matches('\n')
                            .split_once(' ')
                            .expect("条头是 <相对键> <字节数>");
                        read_entry(&mut requests, key, length);
                    }
                    replies.write_all(b"written\n").expect("回得进");
                }
                other => panic!("认不出的请求 {other:?}"),
            }
        }
    }

    /// 几个线程共用一条管道：各自成批写（一批比管道缓冲大，write_all 要分几次写进去）、单条写、读回自己的键；
    /// 替身照到达次序回，每条请求整条到达、每个线程读回的是自己那一问的回应。
    #[test]
    fn threads_sharing_one_relay_never_interleave_requests_or_read_each_others_replies() {
        const ROUNDS: usize = 4;
        const ENTRIES_PER_BATCH: usize = 3000;
        let (requests_for_the_counterpart, request_end_of_the_device) =
            std::io::pipe().expect("建请求管道");
        let (reply_end_of_the_device, reply_end_of_the_counterpart) =
            std::io::pipe().expect("建回应管道");
        let device_library = DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
                requests: std::fs::File::from(std::os::fd::OwnedFd::from(request_end_of_the_device)),
                replies: std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
                    reply_end_of_the_device,
                ))),
            }))),
        };
        let counterpart = std::thread::spawn(move || {
            serve_like_the_relay(requests_for_the_counterpart, reply_end_of_the_counterpart)
        });
        let threads = registered_heavy_threads();
        std::thread::scope(|scope| {
            for thread in 0..threads {
                let device_library = device_library.clone();
                scope.spawn(move || {
                    for round in 0..ROUNDS {
                        let batch: Vec<(String, String)> = (0..ENTRIES_PER_BATCH)
                            .map(|entry| {
                                let key = format!("items/t{thread}-r{round}-e{entry}.out");
                                (key.clone(), key)
                            })
                            .collect();
                        device_library.write_many(&batch);
                        let single = format!("codes/t{thread}-r{round}.txt");
                        device_library.write(&single, &single);
                        let asked = &batch[ENTRIES_PER_BATCH - 1].0;
                        assert_eq!(
                            device_library.read(asked).as_deref(),
                            Some(asked.as_str()),
                            "线程 {thread} 第 {round} 轮读回的是自己那一问的回应"
                        );
                    }
                });
            }
        });
        drop(device_library);
        let written = counterpart.join().expect("替身线程核过每条请求整条到达");
        assert_eq!(written.len(), threads * ROUNDS * (ENTRIES_PER_BATCH + 1));
    }
