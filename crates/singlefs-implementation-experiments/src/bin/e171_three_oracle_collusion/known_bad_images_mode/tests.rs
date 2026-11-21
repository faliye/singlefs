//! 模式 known-bad-images：已知坏镜像与 E170（O3规约执行器第一版） 坏状态的录入与判的单测。

use super::*;

/// 两块盘上各改一个扇区的两份镜像：补丁要两块盘都带上，叠回干净镜像之后摘要与改坏的那份相同（X15：补丁提取只取盘 0 的差，这里红）。
#[test]
fn the_sector_difference_between_two_images_carries_the_changed_sectors_of_every_device() {
    let devices = [DeviceIdentity(0), DeviceIdentity(1)];
    let mut clean = MemoryPool::with_devices(&devices, 1 << 20);
    clean.apply_writes(&[bytes_write(0, 0, vec![0x11; 512])]);
    let mut bad = clean.clone();
    bad.apply_writes(&[
        bytes_write(0, 512, vec![0x44; 512]),
        bytes_write(1, 1024, vec![0x55; 1024]),
    ]);
    let patches = sector_difference_as_patches(&clean, &bad);
    assert_eq!(
        patches,
        vec![
            BytePatch {
                device: 0,
                offset: 512,
                bytes: vec![0x44; 512]
            },
            BytePatch {
                device: 1,
                offset: 1024,
                bytes: vec![0x55; 1024]
            },
        ],
        "盘 1 上相邻的两个扇区并成一段补丁"
    );
    let mut patched = clean.clone();
    apply_patches(&mut patched, &patches);
    assert_eq!(
        sha256_text(&memory_pool_digest(&patched)),
        sha256_text(&memory_pool_digest(&bad))
    );
    assert_ne!(
        sha256_text(&memory_pool_digest(&clean)),
        sha256_text(&memory_pool_digest(&bad)),
        "改坏之前与之后摘要不同"
    );
}

/// E170 的 P 行只翻一个字节：补丁不按扇区对齐，施加时读出罩住它的整扇区、只换那一个字节、再整扇区写回。
#[test]
fn one_byte_patch_inside_a_sector_changes_only_that_byte() {
    let devices = [DeviceIdentity(0), DeviceIdentity(1)];
    let mut pool = MemoryPool::with_devices(&devices, 1 << 20);
    pool.apply_writes(&[bytes_write(1, 512, vec![0x11; 1024])]);
    apply_patches(
        &mut pool,
        &[BytePatch {
            device: 1,
            offset: 700,
            bytes: vec![0xEE],
        }],
    );
    let read_back = pool.devices[&DeviceIdentity(1)].read(DeviceOffsetInBytes(512), 1024);
    assert_eq!(read_back[700 - 512], 0xEE);
    assert!(read_back
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 700 - 512 || *byte == 0x11));
}

#[test]
fn patch_list_text_round_trips_through_the_patches_file() {
    let patches = vec![
        BytePatch {
            device: 1,
            offset: 4096,
            bytes: vec![1, 2, 3],
        },
        BytePatch {
            device: 0,
            offset: 0,
            bytes: vec![9; 5],
        },
    ];
    let mut patches_file = Vec::new();
    let list = patch_list_text(&patches, &mut patches_file);
    assert_eq!(list, "1:4096:0:3,0:0:3:5");
    assert_eq!(patches_of_a_patch_list_text(&list, &patches_file), patches);
    assert_eq!(patch_list_text(&[], &mut Vec::new()), "-");
}

#[test]
fn the_sampling_threads_argument_reaches_the_known_bad_images_mode() {
    let arguments = [
        "known-bad-images",
        "--dump-directory",
        "导出目录",
        "--output-directory",
        "目录",
        "--sampling-threads",
        "3",
    ]
    .map(str::to_string);
    let Some(ParsedCommand::KnownBadImages(command_line)) = parse_command_line(&arguments) else {
        panic!("known-bad-images 带 --sampling-threads 3 要认");
    };
    assert_eq!(
        command_line.switches.sampling_threads,
        NonZeroUsize::new(3),
        "坏镜像模式的线程数取同一个参数"
    );
}

/// 一条线程与几条线程各走一趟模式：停机条款都过，结果行的字节与产物文件的字节逐个相同。
fn assert_the_known_bad_images_mode_leaves_the_same_on_several_threads_as_on_one_thread(
    scenario: &str,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) {
    let dump = DumpDirectoryForKnownBadImages::new(scenario);
    let on_one_thread = dump.run_on_with_the_tables(NonZeroUsize::MIN, tables);
    let on_several_threads = dump.run_on_with_the_tables(SEVERAL_THREADS, tables);
    assert_eq!(
        on_one_thread.ending,
        Ok(0),
        "停机条款都过：{}",
        on_one_thread.result_lines
    );
    assert_eq!(
        on_one_thread
            .result_lines
            .lines()
            .filter(|line| line.contains("name=known_bad_entry "))
            .count(),
        dump.known_bad_entry_count,
        "每条已知坏镜像一行结果行：{}",
        on_one_thread.result_lines
    );
    assert_eq!(
        on_one_thread
            .result_lines
            .lines()
            .filter(|line| line.contains("name=e170_bad_row "))
            .count(),
        (PATCHED_E170_BAD_ROWS_OF_EACH_STREAM + UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
            * STREAM_NAMES.len(),
        "每行 E170 坏状态一行结果行：{}",
        on_one_thread.result_lines
    );
    assert_eq!(
        on_one_thread.files.keys().collect::<Vec<&String>>(),
        [
            "e170-bad-rows-o2.tsv",
            "known-bad-o2.tsv",
            "known-bad-patches.bin",
            "known-bad-states.tsv"
        ]
        .map(str::to_string)
        .iter()
        .collect::<Vec<&String>>(),
        "这个模式写的四个产物文件"
    );
    assert!(
        on_one_thread.result_lines.ends_with(&format!(
            "E7RESULT name=done emitted={}\n",
            on_one_thread.result_lines.lines().count()
        )),
        "收尾行写进同一份输出、数得对：{}",
        on_one_thread.result_lines
    );
    assert_eq!(
        on_several_threads, on_one_thread,
        "几条线程与一条线程的退出码、结果行、产物文件逐字相同"
    );
}

#[test]
fn known_bad_images_mode_on_several_threads_prints_and_writes_the_same_as_on_one_thread() {
    assert_the_known_bad_images_mode_leaves_the_same_on_several_threads_as_on_one_thread(
        "cpu-pairing",
        &PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        ),
    );
}

/// 同一条对拍走产品路径：每张盘一张给定状态表录进判定库、判段交引擎，表名与表的内容与产品路径相同。
#[cfg(feature = "gpu")]
#[test]
fn known_bad_images_mode_through_the_given_states_engine_on_several_threads_prints_and_writes_the_same_as_on_one_thread(
) {
    assert_the_known_bad_images_mode_leaves_the_same_on_several_threads_as_on_one_thread(
        "engine-pairing",
        &GivenStatesTablesInTheLibrary,
    );
}

#[test]
fn judged_tables_keep_their_names_on_several_threads() {
    let dump = DumpDirectoryForKnownBadImages::new("table-names");
    let judged_table_names = Mutex::new(Vec::new());
    let outcome = dump.run_on(SEVERAL_THREADS, &|table_name, pool| {
        judged_table_names
            .lock()
            .expect("这把锁里不 panic")
            .push(table_name.to_string());
        pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
    });
    assert_eq!(outcome.ending, Ok(0));
    let mut expected_table_names = vec!["known-bad/clean".to_string()];
    for entry_index in 0..dump.known_bad_entry_count {
        expected_table_names.push(format!("known-bad/entry-{entry_index}/direct"));
        expected_table_names.push(format!("known-bad/entry-{entry_index}/patched"));
    }
    for name in STREAM_NAMES {
        for row_position in
            0..PATCHED_E170_BAD_ROWS_OF_EACH_STREAM + UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM
        {
            let row_identifier = e170_bad_row_identifier(row_position);
            let label = name.directory_name();
            expected_table_names.push(format!("e170-bad-row/{label}/{row_identifier}/base"));
            expected_table_names.push(format!("e170-bad-row/{label}/{row_identifier}/patched"));
        }
    }
    expected_table_names.sort_unstable();
    let mut judged_table_names = judged_table_names.into_inner().expect("这把锁里不 panic");
    judged_table_names.sort_unstable();
    assert_eq!(
        judged_table_names, expected_table_names,
        "每张表判一次、表名不随线程变（表名进录入摘要，判定库的键不变）"
    );
}

/// 同时在判的调用要等到几次才算「同时」。
const SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR: usize = 2;

/// 表名以这个前缀起头的判定，等多久等不到另一次同时的调用就不再等（一条线程走完那一趟只等这一回）。
const SECONDS_A_JUDGE_CALL_WAITS_FOR_ANOTHER: u64 = 30;

/// 同时在判的调用数：`in_flight` 是此刻几次，`most_in_flight` 是到此刻最多几次，`gave_up` 是等过一回没等到。
#[derive(Default)]
struct JudgeCallsInFlight {
    in_flight: usize,
    most_in_flight: usize,
    gave_up: bool,
}

/// 让表名带某个前缀的判定互相等到有 [`SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR`] 次同时在判：几条线程时必等得到，一条线程时等够时限一回就不再等。
struct RendezvousOfJudgeCalls {
    table_name_prefix: &'static str,
    calls: Mutex<JudgeCallsInFlight>,
    one_arrived: std::sync::Condvar,
}

impl RendezvousOfJudgeCalls {
    fn new(table_name_prefix: &'static str) -> Self {
        Self {
            table_name_prefix,
            calls: Mutex::new(JudgeCallsInFlight::default()),
            one_arrived: std::sync::Condvar::new(),
        }
    }

    fn judge(&self, table_name: &str, pool: &MemoryPool) -> Vec<String> {
        if table_name.starts_with(self.table_name_prefix) {
            let deadline = Instant::now()
                + std::time::Duration::from_secs(SECONDS_A_JUDGE_CALL_WAITS_FOR_ANOTHER);
            let mut calls = self.calls.lock().expect("这把锁里不 panic");
            calls.in_flight += 1;
            calls.most_in_flight = calls.most_in_flight.max(calls.in_flight);
            self.one_arrived.notify_all();
            while calls.most_in_flight < SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR && !calls.gave_up {
                let Some(time_left) = deadline.checked_duration_since(Instant::now()) else {
                    calls.gave_up = true;
                    break;
                };
                calls = self
                    .one_arrived
                    .wait_timeout(calls, time_left)
                    .expect("这把锁里不 panic")
                    .0;
            }
            calls.in_flight -= 1;
        }
        pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
    }

    fn most_in_flight(&self) -> usize {
        self.calls.lock().expect("这把锁里不 panic").most_in_flight
    }
}

/// 带 `--sampling-threads` 走一趟模式（线程数照产品路径 `sampling_threads_of_this_run` 取），表名带这个前缀的判定要有几次同时在判。
fn assert_the_tables_with_this_prefix_are_judged_at_once(
    scenario: &str,
    table_name_prefix: &'static str,
) {
    let dump = DumpDirectoryForKnownBadImages::new(scenario);
    let rendezvous = RendezvousOfJudgeCalls::new(table_name_prefix);
    let outcome = dump.run_on(SEVERAL_THREADS, &|table_name, pool| {
        rendezvous.judge(table_name, pool)
    });
    assert_eq!(outcome.ending, Ok(0));
    let threads_of_this_run = sampling_threads_of_this_run(RunSwitches {
        sampling_threads: Some(SEVERAL_THREADS),
        ..RunSwitches::default()
    });
    let expected_at_once = SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR.min(threads_of_this_run.get());
    assert!(
        rendezvous.most_in_flight() >= expected_at_once,
        "{table_name_prefix} 的表最多 {} 次同时在判，这一趟开 {threads_of_this_run} 条线程，至少要 {expected_at_once} 次",
        rendezvous.most_in_flight()
    );
}

#[test]
fn known_bad_entries_are_judged_on_several_threads_at_once() {
    assert_the_tables_with_this_prefix_are_judged_at_once("entries-at-once", "known-bad/entry-");
}

#[test]
fn e170_bad_rows_are_judged_on_several_threads_at_once() {
    assert_the_tables_with_this_prefix_are_judged_at_once("rows-at-once", "e170-bad-row/");
}

const PANIC_MESSAGE_OF_THE_JUDGE: &str = "坏镜像线程对拍里造的判器崩";

/// 走到这张表就 panic 的判器，别的表照判。
fn judge_panicking_at<'table>(
    panicking_table_name: &'table str,
) -> impl Fn(&str, &MemoryPool) -> Vec<String> + Sync + 'table {
    move |table_name, pool| {
        assert_ne!(
            table_name, panicking_table_name,
            "{PANIC_MESSAGE_OF_THE_JUDGE}"
        );
        pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
    }
}

#[test]
fn panic_on_a_known_bad_entry_prints_the_same_lines_on_several_threads_as_on_one_thread() {
    const ENTRY_OF_THE_PANIC: usize = 3;
    let dump = DumpDirectoryForKnownBadImages::new("known-bad-entry-panic");
    let panicking_table_name = format!("known-bad/entry-{ENTRY_OF_THE_PANIC}/patched");
    let judge = judge_panicking_at(&panicking_table_name);
    let on_one_thread = dump.run_on(NonZeroUsize::MIN, &judge);
    let on_several_threads = dump.run_on(SEVERAL_THREADS, &judge);
    assert!(
        on_one_thread
            .ending
            .as_ref()
            .is_err_and(|text| text.contains(PANIC_MESSAGE_OF_THE_JUDGE)),
        "判器的 panic 原样抛出来：{:?}",
        on_one_thread.ending
    );
    assert!(
        on_one_thread.result_lines.contains(&format!(
            "name=known_bad_entry index={} ",
            ENTRY_OF_THE_PANIC - 1
        )) && !on_one_thread
            .result_lines
            .contains(&format!("name=known_bad_entry index={ENTRY_OF_THE_PANIC} ")),
        "结果行打到 panic 的前一条为止：{}",
        on_one_thread.result_lines
    );
    assert!(on_one_thread.files.is_empty(), "产物在判完之后才写");
    assert_eq!(
        on_several_threads, on_one_thread,
        "几条线程与一条线程的 panic 消息、结果行、产物文件逐字相同"
    );
}

#[test]
fn panic_on_an_e170_bad_row_prints_and_writes_the_same_on_several_threads_as_on_one_thread() {
    const ROW_OF_THE_PANIC: usize = 2;
    let dump = DumpDirectoryForKnownBadImages::new("e170-bad-row-panic");
    let label = StreamName::NewPoolFileCreation.directory_name();
    let row_identifier = e170_bad_row_identifier(ROW_OF_THE_PANIC);
    let panicking_table_name = format!("e170-bad-row/{label}/{row_identifier}/base");
    let judge = judge_panicking_at(&panicking_table_name);
    let on_one_thread = dump.run_on(NonZeroUsize::MIN, &judge);
    let on_several_threads = dump.run_on(SEVERAL_THREADS, &judge);
    assert!(
        on_one_thread
            .ending
            .as_ref()
            .is_err_and(|text| text.contains(PANIC_MESSAGE_OF_THE_JUDGE)),
        "判器的 panic 原样抛出来：{:?}",
        on_one_thread.ending
    );
    let row_line_start = |row_position: usize| {
        format!(
            "name=e170_bad_row stream={label} row={} ",
            e170_bad_row_identifier(row_position)
        )
    };
    assert!(
        on_one_thread
            .result_lines
            .contains(&row_line_start(ROW_OF_THE_PANIC - 1))
            && !on_one_thread
                .result_lines
                .contains(&row_line_start(ROW_OF_THE_PANIC)),
        "结果行打到 panic 的前一行为止：{}",
        on_one_thread.result_lines
    );
    assert_eq!(
        on_one_thread.files.keys().collect::<Vec<&String>>(),
        [
            "known-bad-o2.tsv",
            "known-bad-patches.bin",
            "known-bad-states.tsv"
        ]
        .map(str::to_string)
        .iter()
        .collect::<Vec<&String>>(),
        "已知坏镜像的产物已经写了，E170 坏状态那一份在判完之后才写"
    );
    assert_eq!(
        on_several_threads, on_one_thread,
        "几条线程与一条线程的 panic 消息、结果行、产物文件逐字相同"
    );
}

#[test]
fn no_known_bad_entry_after_a_panicking_one_is_judged_on_one_thread() {
    const ENTRY_OF_THE_PANIC: usize = 3;
    let dump = DumpDirectoryForKnownBadImages::new("no-entry-after-the-panic");
    let panicking_table_name = format!("known-bad/entry-{ENTRY_OF_THE_PANIC}/direct");
    let judge = judge_panicking_at(&panicking_table_name);
    let judged_table_names = Mutex::new(Vec::new());
    let outcome = dump.run_on(NonZeroUsize::MIN, &|table_name, pool| {
        judged_table_names
            .lock()
            .expect("这把锁里不 panic")
            .push(table_name.to_string());
        judge(table_name, pool)
    });
    assert!(outcome.ending.is_err(), "判器 panic 了这一趟要 panic");
    assert_eq!(
        judged_table_names
            .into_inner()
            .expect("这把锁里不 panic")
            .last(),
        Some(&panicking_table_name),
        "panic 的那张表之后一张都不再判"
    );
}
