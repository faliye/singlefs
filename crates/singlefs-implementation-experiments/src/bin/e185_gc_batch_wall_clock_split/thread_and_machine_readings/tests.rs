use super::*;

/// N3：`schedstat` 的排队等 CPU 是第二个字段。
#[test]
fn scheduler_counters_give_the_run_queue_wait_after_the_time_on_the_cpu() {
    assert_eq!(
        run_queue_wait_nanoseconds_of_the_scheduler_counters("100 200 3\n"),
        Some(200)
    );
}

/// N5、N6、N10：`stat` 从最后一个 `)` 之后数栏（进程名 `(a b) c)` 里有空格与括号）。
#[test]
fn counter_record_fields_count_from_the_last_closing_parenthesis() {
    let counter_record_text = "4242 (a b) c) S 1 2 3 4 5 6 70 0 12 0 1400 150 0 0 20 0 1 0 100\n";
    assert_eq!(
        thread_counters_of_the_counter_record(counter_record_text),
        Some(ThreadCounters {
            minor_faults: 70,
            major_faults: 12,
            user_ticks: 1400,
            system_ticks: 150,
        })
    );
    assert_eq!(
        process_ticks_of_the_counter_record(counter_record_text),
        Some(1550)
    );
}

/// 核号是 `stat` 第 39 栏，从最后一个 `)` 之后数（进程名 `(e185 (worker) x)` 里有空格与括号）；第 38、40 栏是别的数，取错一栏就对不上。
#[test]
fn cpu_number_is_field_39_of_the_counter_record_counted_after_the_command_name() {
    let fields_from_the_state_on: Vec<String> = (3_usize..=52)
        .map(|field_number| match field_number {
            3 => "R".to_string(),
            39 => "7".to_string(),
            _ => (field_number * 1000).to_string(),
        })
        .collect();
    let counter_record_text = format!(
        "4242 (e185 (worker) x) {}\n",
        fields_from_the_state_on.join(" ")
    );
    assert_eq!(
        cpu_number_of_the_counter_record(&counter_record_text),
        Some(7),
        "第 39 栏是 7：{counter_record_text}"
    );
    assert_eq!(
        cpu_number_of_the_counter_record("4242 (a b) c) S 1 2 3\n"),
        None,
        "不到 39 栏的交回 None"
    );
}

/// 辅助向量按本机字节序的（类型、值）两个字一项拆；取类型 17（`AT_CLKTCK`）那一项的值，别的项（页大小 4096）不取。
#[test]
fn clock_ticks_are_the_value_of_the_clock_tick_entry_of_the_auxiliary_vector() {
    let auxiliary_vector: Vec<u8> = [
        (libc::AT_PAGESZ, 4096_u64),
        (libc::AT_CLKTCK, 100),
        (libc::AT_NULL, 0),
    ]
    .iter()
    .flat_map(|(entry_type, value)| {
        entry_type
            .to_ne_bytes()
            .into_iter()
            .chain(value.to_ne_bytes())
    })
    .collect();
    assert_eq!(
        clock_ticks_of_the_auxiliary_vector(&auxiliary_vector),
        Some(100)
    );
}

/// 没有 `AT_CLKTCK`、它排在 `AT_NULL` 之后、值是 0，都交回 `None`（读不到不当 100）。
#[test]
fn auxiliary_vector_without_a_usable_clock_tick_entry_reads_as_none() {
    fn auxiliary_vector_of(entries: &[(u64, u64)]) -> Vec<u8> {
        entries
            .iter()
            .flat_map(|(entry_type, value)| {
                entry_type
                    .to_ne_bytes()
                    .into_iter()
                    .chain(value.to_ne_bytes())
            })
            .collect()
    }
    assert_eq!(
        clock_ticks_of_the_auxiliary_vector(&auxiliary_vector_of(&[
            (libc::AT_PAGESZ, 4096),
            (libc::AT_NULL, 0),
        ])),
        None,
        "没有类型 17 那一项"
    );
    assert_eq!(
        clock_ticks_of_the_auxiliary_vector(&auxiliary_vector_of(&[
            (libc::AT_NULL, 0),
            (libc::AT_CLKTCK, 100),
        ])),
        None,
        "AT_NULL 之后的不算"
    );
    assert_eq!(
        clock_ticks_of_the_auxiliary_vector(&auxiliary_vector_of(&[
            (libc::AT_CLKTCK, 0),
            (libc::AT_NULL, 0),
        ])),
        None,
        "节拍数 0 不当读到"
    );
    assert_eq!(clock_ticks_of_the_auxiliary_vector(&[]), None, "空的");
}

/// 读不到不当 0：文件不在交回 `None`；在的照读。
#[test]
fn missing_kernel_file_reads_as_none_not_zero() {
    assert_eq!(
        parsed_value_of_the_file(
            Path::new("/proc/e185-no-such-kernel-file"),
            run_queue_wait_nanoseconds_of_the_scheduler_counters
        ),
        None
    );
    let path =
        std::env::temp_dir().join(format!("e185-kernel-file-reading-{}", std::process::id()));
    std::fs::write(&path, "5 7 9\n").expect("写得进临时文件");
    let value =
        parsed_value_of_the_file(&path, run_queue_wait_nanoseconds_of_the_scheduler_counters);
    std::fs::remove_file(&path).expect("删掉这条单测建的文件");
    assert_eq!(value, Some(7));
    assert_eq!(increase_of(Some(3), Some(5)), Some(2));
    assert_eq!(increase_of(Some(5), Some(3)), None, "回绕不当负数");
    assert_eq!(increase_of(None, Some(5)), None);
}

/// N10：`/proc/stat` 首行前八栏是全部，idle 与 iowait 之外是忙；隔一秒的增量折成整机忙、本进程与其余三个比例。
#[test]
fn machine_shares_split_the_busy_jiffies_into_this_process_and_the_others() {
    let before = cpu_jiffies_of_the_system_counters(
        "cpu  100 0 50 800 50 0 0 0 7 0\ncpu0 1 2 3 4 5 6 7 8\n",
    );
    assert_eq!(
        before,
        Some(CpuJiffies {
            busy: 150,
            total: 1000,
        })
    );
    let after = Some(CpuJiffies {
        busy: 450,
        total: 2000,
    });
    let shares = machine_shares(before, after, Some(10), Some(110)).expect("四样都读得到");
    assert!((shares.machine_busy_share - 0.3).abs() < 1e-12);
    assert!((shares.this_process_share - 0.1).abs() < 1e-12);
    assert!((shares.others_share - 0.2).abs() < 1e-12);
    assert_eq!(machine_shares(before, None, Some(10), Some(110)), None);
}

/// N4、N7、N10：`status`、`pressure`、`loadavg`、`meminfo` 各取对那一个数。
#[test]
fn status_pressure_load_and_memory_texts_give_their_numbers() {
    assert_eq!(
        involuntary_switches_of_the_status(
            "voluntary_ctxt_switches:\t3\nnonvoluntary_ctxt_switches:\t17\n"
        ),
        Some(17)
    );
    assert_eq!(
        pressure_some_ten_seconds_of(
            "some avg10=2.86 avg60=3.76 avg300=9.41 total=1\nfull avg10=1.00 avg60=0.00 avg300=0.00 total=0\n"
        ),
        Some(2.86)
    );
    assert_eq!(
        load_average_one_minute_of("18.47 29.47 33.50 2/1000 123\n"),
        Some(18.47)
    );
    assert_eq!(
        memory_available_bytes_of("MemTotal:  100 kB\nMemAvailable:   2048 kB\n"),
        Some(2_097_152)
    );
}

/// N3–N9：在本线程上读得到排队时间、非自愿切换、`stat` 计数、负载与 CPU 号（压力文件要内核开 PSI，不判）。
#[test]
fn thread_and_machine_snapshot_reads_this_thread() {
    let started = Instant::now();
    let snapshot = thread_and_machine_snapshot(started);
    assert!(
        snapshot.run_queue_wait_nanoseconds.is_some(),
        "{snapshot:?}"
    );
    assert!(snapshot.involuntary_switches.is_some(), "{snapshot:?}");
    assert!(snapshot.thread_counters.is_some(), "{snapshot:?}");
    assert!(snapshot.load_average_one_minute.is_some(), "{snapshot:?}");
    assert!(snapshot.cpu_number.is_some(), "{snapshot:?}");
    assert!(clock_ticks_per_second().is_some_and(|ticks| ticks > 0));
    assert!(snapshot.seconds_since_the_segment_start >= 0.0);
}
