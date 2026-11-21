use super::*;
use crate::helpers_shared_by_the_tests::parsed;

#[test]
fn capacity_cells_are_nine_distinct_cells_and_the_shards_are_forty_five_round_major() {
    let cells = capacity_cells();
    assert_eq!(cells.len(), 9);
    assert_eq!(cells.iter().collect::<BTreeSet<_>>().len(), 9);
    assert_eq!(cells[4].device_bytes + MEBIBYTE, 16 * TEBIBYTE);
    let shards = capacity_shards();
    assert_eq!(shards.len(), 45);
    assert_eq!(shards[0].0, "h2-device-4096-mebibytes-units-0-round-1");
    assert_eq!(shards[9].2, 2, "第 10 片已是第 2 轮");
    assert_eq!(
        capacity_shard_by_name("h2-device-65536-mebibytes-units-10240-round-5")
            .map(|(cell, round)| (cell.sequentially_written_units, round)),
        Some((10240, 5))
    );
    assert_eq!(
        capacity_shard_by_name("h2-device-65536-mebibytes-units-1024-round-1"),
        None,
        "64 GiB 上没有 n = 1024 的格"
    );
}

#[test]
fn written_contents_have_the_registered_lengths_and_differ_between_calls() {
    assert_eq!(sequential_write_content(3).len(), 3 * 32634);
    assert_eq!(measured_overwrite_content(1, 0).len(), 4096);
    assert_ne!(
        measured_overwrite_content(1, 0),
        measured_overwrite_content(1, 1)
    );
    assert_ne!(
        measured_overwrite_content(1, 4),
        measured_overwrite_content(2, 0)
    );
    assert_ne!(
        measured_overwrite_content(1, 0)[..3000],
        first_file_content()[..]
    );
}

#[test]
fn local_payload_constant_equals_the_implementation() {
    assert_eq!(
        u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("载荷容量"),
        DATA_UNIT_PAYLOAD_BYTES
    );
}

#[test]
fn measured_window_reports_the_buffer_allocated_inside_it() {
    let (length, window) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
        let buffer = vec![3u8; 8 << 20];
        std::hint::black_box(&buffer);
        buffer.len()
    });
    assert_eq!(length, 8 << 20);
    assert!(window.peak_over_baseline_bytes >= 8 << 20, "{window:?}");
}

/// 命令字：0 是没有命令（也是做完的回执），`FREE` 放掉手里握的，`STOP` 收工，别的数是「分这么多字节、写满、握着」。
const ANOTHER_THREAD_FREE_COMMAND: u64 = u64::MAX;
const ANOTHER_THREAD_STOP_COMMAND: u64 = u64::MAX - 1;

/// 一条在测试进程里和量窗并行的别的线程，按命令分或放一大份堆。命令与回执只用原子字，
/// 发命令的线程在窗里等回执时自己不分配（分配会进它自己的读数）。
struct AnotherThreadHoldingHeap {
    command: std::sync::Arc<AtomicU64>,
    worker: std::thread::JoinHandle<()>,
}

impl AnotherThreadHoldingHeap {
    fn start() -> Self {
        let command = std::sync::Arc::new(AtomicU64::new(0));
        let seen = std::sync::Arc::clone(&command);
        let worker = std::thread::spawn(move || {
            let mut held: Vec<u8> = Vec::new();
            loop {
                match seen.load(Ordering::Acquire) {
                    0 => std::thread::yield_now(),
                    ANOTHER_THREAD_STOP_COMMAND => break,
                    ANOTHER_THREAD_FREE_COMMAND => {
                        held = Vec::new();
                        seen.store(0, Ordering::Release);
                    }
                    bytes => {
                        held = vec![9u8; usize::try_from(bytes).expect("装得进 usize")];
                        std::hint::black_box(&held);
                        seen.store(0, Ordering::Release);
                    }
                }
            }
            drop(held);
        });
        Self { command, worker }
    }

    /// 发一条命令并等那条线程做完（回执是命令字回到 0）；30 秒还没做完就 panic，不挂住测试。
    fn tell(&self, command: u64) {
        let started = Instant::now();
        self.command.store(command, Ordering::Release);
        while self.command.load(Ordering::Acquire) != 0 {
            assert!(
                started.elapsed().as_secs() < 30,
                "别的线程 30 秒没做完命令 {command}"
            );
            std::thread::yield_now();
        }
    }

    fn finish(self) {
        self.command
            .store(ANOTHER_THREAD_STOP_COMMAND, Ordering::Release);
        self.worker.join().expect("别的线程正常收工");
    }
}

/// 别的线程在窗里放掉一大份、或分一大份并握着，窗报的仍恰好是发起窗的这条线程自己分的那一份。
/// 放掉的一例是原来的缺陷：全进程的「之后减之前」掉到 0（饱和），峰值也报不出这条线程分的 3 MiB。
#[test]
fn measured_window_counts_only_the_thread_that_measures_it() {
    const KEPT_BYTES: usize = 3 << 20;
    const OTHER_THREAD_BYTES: u64 = 40 << 20;
    let kept_bytes = u64::try_from(KEPT_BYTES).expect("装得进 u64");
    let other = AnotherThreadHoldingHeap::start();
    other.tell(OTHER_THREAD_BYTES);
    let (kept_while_another_thread_frees, while_another_thread_frees) =
        measure_in_a_window(&COUNTING_ALLOCATOR, || {
            let buffer = vec![7u8; KEPT_BYTES];
            std::hint::black_box(&buffer);
            other.tell(ANOTHER_THREAD_FREE_COMMAND);
            buffer
        });
    drop(kept_while_another_thread_frees);
    let (kept_while_another_thread_allocates, while_another_thread_allocates) =
        measure_in_a_window(&COUNTING_ALLOCATOR, || {
            let buffer = vec![7u8; KEPT_BYTES];
            std::hint::black_box(&buffer);
            other.tell(OTHER_THREAD_BYTES);
            buffer
        });
    drop(kept_while_another_thread_allocates);
    // 发起窗的线程自己在窗里放掉一份：基线与窗尾差的恰是放掉的字节，峰值多占为 0。
    other.tell(ANOTHER_THREAD_FREE_COMMAND);
    let released = vec![5u8; KEPT_BYTES];
    std::hint::black_box(&released);
    let ((), while_this_thread_frees) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
        other.tell(OTHER_THREAD_BYTES);
        drop(released);
    });
    other.finish();
    for window in [while_another_thread_frees, while_another_thread_allocates] {
        assert_eq!(window.peak_over_baseline_bytes, kept_bytes, "{window:?}");
        assert_eq!(
            window.in_use_after_bytes - window.baseline_bytes,
            kept_bytes,
            "{window:?}"
        );
    }
    assert_eq!(
        while_this_thread_frees.peak_over_baseline_bytes, 0,
        "{while_this_thread_frees:?}"
    );
    assert_eq!(
        while_this_thread_frees.baseline_bytes - while_this_thread_frees.in_use_after_bytes,
        kept_bytes,
        "{while_this_thread_frees:?}"
    );
}

#[test]
fn the_running_order_starts_every_round_with_the_largest_cell_and_keeps_all_forty_five() {
    let order = capacity_shards_in_running_order();
    assert_eq!(order.len(), 45);
    assert_eq!(order[0].0, "h2-device-16777215-mebibytes-units-0-round-1");
    assert_eq!(order[1].0, "h2-device-4194304-mebibytes-units-0-round-1");
    assert_eq!(
        order[2].0,
        "h2-device-1048576-mebibytes-units-10240-round-1"
    );
    assert_eq!(order[8].0, "h2-device-4096-mebibytes-units-0-round-1");
    assert_eq!(order[9].0, "h2-device-16777215-mebibytes-units-0-round-2");
    assert_eq!(order[9].2, 2);
    let running: BTreeSet<String> = order.into_iter().map(|(shard, _, _)| shard).collect();
    let merged: BTreeSet<String> = capacity_shards()
        .into_iter()
        .map(|(shard, _, _)| shard)
        .collect();
    assert_eq!(running, merged, "跑的次序与合并读的次序是同一批片");
}

#[test]
fn shard_context_needs_a_known_host_a_numbered_service_attempt_and_one_filesystem_word() {
    assert_eq!(
        CapacityShardContext::parse("abc", "第二台", "4/5:2", "ext2/ext3"),
        Some(CapacityShardContext {
            binary_sha256: "abc".to_string(),
            host: ShardHost::SecondMachine,
            service_attempt: "4/5:2".to_string(),
            image_filesystem: "ext2/ext3".to_string(),
        })
    );
    for (host, attempt, filesystem) in [
        ("local", "0/5:1", "ext2/ext3"),
        ("本机", "0/5", "ext2/ext3"),
        ("本机", "a/5:1", "ext2/ext3"),
        ("本机", "0/5:", "ext2/ext3"),
        ("本机", "0/5:1", ""),
        ("本机", "0/5:1", "ext 4"),
    ] {
        assert_eq!(
            CapacityShardContext::parse("abc", host, attempt, filesystem),
            None,
            "{host} {attempt} {filesystem:?}"
        );
    }
}

/// 小盘（256 MiB、环 1 MiB）文件镜像上的 H2 一格走产物同一条路：n = 0 时 X4 五次都分到、X5 五次都发成；n = 16 时 X4 五次都分到、
/// 记录数比 n = 0 多出至少 16 条（记录格 X5 的结局照报、不在这里判：多单元文件上覆盖写的行为由产物给）；镜像文件跑完就删。
#[test]
fn small_capacity_cell_on_file_images_reaches_every_measurement() {
    let mut records_by_units = Vec::new();
    for units in [0u64, 16] {
        // 守卫认领临时目录：用例做完、断言失败、panic 都删掉它
        let directory =
            singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
                std::env::temp_dir().join(format!("e183-h2-cell-{}-{units}", std::process::id())),
            );
        std::fs::create_dir_all(&*directory).expect("临时目录");
        let cell = CapacityCell {
            device_bytes: 256 * MEBIBYTE,
            sequentially_written_units: units,
        };
        let mut emitter = Emitter::collecting_lines();
        let context = CapacityShardContext::parse("same", "本机", "0/5:1", "ext2/ext3")
            .expect("合法的片参数");
        let end = run_capacity_cell_with_a_journal_ring(
            "small",
            cell,
            1,
            MEBIBYTE,
            &directory,
            &context,
            &mut emitter,
        );
        let left_behind = std::fs::read_dir(&directory).expect("列目录").count();
        let lines = emitter.into_collected_lines();
        assert_eq!(end, ShardEnd::Finished);
        assert_eq!(left_behind, 0, "镜像文件跑完就删");
        let status = status_of_a_capacity_shard(&parsed(&lines), "same");
        let facts = status.facts().expect("片做完就有读数");
        assert_eq!(facts.host.as_deref(), Some("本机"), "{lines:?}");
        assert!(
            facts.high_water_mark_kibibytes.is_some(),
            "片做完都以进程高水位那一行收尾：{lines:?}"
        );
        assert!(
            facts.high_water_mark_before_kibibytes.is_some()
                && facts.high_water_mark_after_kibibytes.is_some(),
            "{lines:?}"
        );
        // 测量那一条路本身：不看 X22（同一个测试进程里别的用例抬过高水位，PC-高水位在这里不一定过）。
        let reading = status
            .measured_reading()
            .unwrap_or_else(|| panic!("n = {units}：{status:?}\n{}", lines.join("\n")));
        assert!(
            reading
                .allocate
                .iter()
                .all(|call| call.slots_from_unit_area_start.is_some()),
            "{lines:?}"
        );
        if units == 0 {
            assert!(
                reading.publish.iter().all(|call| call.published),
                "{lines:?}"
            );
        }
        assert!(reading.copy_bytes > 0);
        records_by_units.push(reading.records);
    }
    assert!(
        records_by_units[1] >= records_by_units[0] + 16,
        "{records_by_units:?}"
    );
}
