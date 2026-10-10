//! 正推腿对 K1/K2 的探针（只用 `singlefs_harness::gpu_memory_ledger` 的公开接口，假的空闲读数、假进程，不要显卡）。
//! 放进 `crates/singlefs-harness/tests/` 跑：`cargo test -p singlefs-harness --test k_forward_ledger_probe -- --nocapture`。
//! 容量闭包照 `gpu_unit_checks.rs` 的 `open_the_card_pool` 里那一段抄（配置额度给得很大，等于不限额，容量 = 估计 − 驱动）。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, BookingRefusal, GpuMemoryLedger, LedgerSettings,
    RecordedProcess,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const DRIVER: u64 = 2560;
const HUGE_CONFIGURED_QUOTA: u64 = 1_000_000;

fn scratch_directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("k-forward-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn settings(directory: &std::path::Path, pid: u32, longest_wait: Duration) -> LedgerSettings {
    LedgerSettings {
        directory: directory.to_path_buf(),
        this_process: RecordedProcess {
            process_identifier: pid,
            start_time_in_clock_ticks: 1,
        },
        is_alive: Box::new(|_| true),
        mebibytes_each_process_keeps_on_a_card: DRIVER,
        poll_interval_for_memory_returned_by_other_processes: Duration::from_millis(10),
        longest_wait,
    }
}

/// 同卡池的容量闭包：配置额度 Some(很大)、没设额度变量：budget = min(配置额度, 估计)，容量 = budget − 驱动。
fn open(
    directory: &std::path::Path,
    pid: u32,
    stems: &[&str],
    read_free: &dyn Fn(usize) -> Result<u64, String>,
    longest_wait: Duration,
) -> GpuMemoryLedger {
    GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
        settings(directory, pid, longest_wait),
        stems.iter().map(|stem| (*stem).to_string()).collect(),
        read_free,
        Box::new(|_position, estimate| {
            card_ledger_capacity_mebibytes(
                None,
                Some(HUGE_CONFIGURED_QUOTA),
                HUGE_CONFIGURED_QUOTA.min(estimate),
                estimate,
                DRIVER,
            )
        }),
    )
    .expect("临时目录下的账本读写得了")
}

/// P1：开账之后外人才占卡，没有别的进程再开账：账的容量还是开账那一刻的。
#[test]
fn probe_a_the_free_memory_is_read_only_when_the_ledger_opens() {
    let directory = scratch_directory("a");
    let reads = AtomicU64::new(0);
    let free = AtomicU64::new(30_000);
    let ledger = open(
        &directory,
        100,
        &["card-a"],
        &|_| {
            reads.fetch_add(1, Ordering::SeqCst);
            Ok(free.load(Ordering::SeqCst))
        },
        Duration::from_millis(100),
    );
    free.store(5_000, Ordering::SeqCst); // 外人占走 25 000
    let capacity = ledger.card_capacities_mebibytes().unwrap();
    let booking = ledger.book_waiting_for_room(20_000);
    println!(
        "PROBE_A reads_of_the_free_memory={} fake_free_now={} ledger_capacity={capacity:?} booking_of_20000={:?}",
        reads.load(Ordering::SeqCst),
        free.load(Ordering::SeqCst),
        booking.as_ref().map(|booking| booking.mebibytes()).map_err(|_| "refused")
    );
    assert_eq!(reads.load(Ordering::SeqCst), 1, "开账之后不再读");
    assert!(booking.is_ok(), "按开账时的 30 000 借得到 20 000");
    let _ = std::fs::remove_dir_all(&directory);
}

/// P2：开账时外人占着（读到 5 000），之后外人走了；同一个进程要借 10 000：红。别的进程开账读到 30 000 之后才借得到。
#[test]
fn probe_b_an_outsider_that_left_after_the_ledger_opened_still_caps_the_process_that_opened_then() {
    let directory = scratch_directory("b");
    let free = AtomicU64::new(5_000);
    let first = open(
        &directory,
        100,
        &["card-a"],
        &|_| Ok(free.load(Ordering::SeqCst)),
        Duration::from_millis(100),
    );
    free.store(30_000, Ordering::SeqCst); // 外人走了
    let before = first.book_waiting_for_room(10_000).map(|b| b.mebibytes());
    println!("PROBE_B same_process_after_the_outsider_left={before:?}");
    assert!(matches!(
        before,
        Err(BookingRefusal::NeedLargerThanEveryCard(_))
    ));
    let _second = open(
        &directory,
        200,
        &["card-a"],
        &|_| Ok(free.load(Ordering::SeqCst)),
        Duration::from_millis(100),
    );
    let after = first.book_waiting_for_room(10_000).map(|b| b.mebibytes());
    println!("PROBE_B after_another_process_opened_and_read_30000={after:?}");
    assert!(after.is_ok(), "别的进程开账抬高了估计，早开账的这一笔借得到");
    let _ = std::fs::remove_dir_all(&directory);
}

/// P3：一张卡容量 20 000、别的进程已记 12 000 + 驱动 2 560，空 5 440。按状态数借（每个状态 1 000，要 30 个）：
/// 锁里按「整张卡的容量」压成 20 个（20 000），放不下，整笔等；按「剩下的空」压是 5 个，放得下。
#[test]
fn probe_c_states_are_compressed_to_the_card_capacity_not_to_the_room_left() {
    let directory = scratch_directory("c");
    let first = open(&directory, 100, &["card-a"], &|_| Ok(22_560), Duration::from_millis(50));
    let first_lease = first.book_waiting_for_room(12_000).expect("空卡放得下 12 000");
    let second = open(&directory, 200, &["card-a"], &|_| Ok(8_000), Duration::from_millis(300));
    let capacity = second.card_capacities_mebibytes().unwrap();
    // 对照：同一个进程直接借 5 000 的一笔，当场借到，说明卡上放得下
    let control = second.book_waiting_for_room(5_000).map(|b| b.mebibytes());
    drop(control.as_ref().ok());
    let started = std::time::Instant::now();
    let by_states = second.book_states_on_the_cards_waiting_only_when_none_has_room(
        &[0],
        30,
        &[30],
        &|states| states * 1_000,
    );
    println!(
        "PROBE_C capacity={capacity:?} room_left={} control_lease_of_5000={control:?} states_booking={} waited_ms={}",
        capacity[0] + DRIVER - 12_000 - DRIVER - DRIVER,
        match &by_states {
            Ok(booked) => format!("booked {:?}", booked.iter().map(|b| b.as_ref().map(|s| s.states)).collect::<Vec<_>>()),
            Err(BookingRefusal::WaitedPastTheLongestWait { need_mebibytes, .. }) => format!("WaitedPastTheLongestWait need={need_mebibytes}"),
            Err(other) => format!("{other:?}"),
        },
        started.elapsed().as_millis()
    );
    drop(first_lease);
    let _ = std::fs::remove_dir_all(&directory);
}

/// P4：两张卡，第 0 张被别的进程记了 12 000（空 5 440），第 1 张空着（容量 20 000）。要装 30 个状态、每张最多 30 个。
#[test]
fn probe_d_a_card_with_some_room_gets_no_states_while_the_next_card_takes_a_full_load() {
    let directory = scratch_directory("d");
    let first = open(&directory, 100, &["card-a", "card-b"], &|_| Ok(22_560), Duration::from_millis(50));
    let first_lease = first.book_on_the_card_waiting_for_room(0, 12_000).expect("放得下");
    let second = open(&directory, 200, &["card-a", "card-b"], &|position| Ok(if position == 0 { 8_000 } else { 22_560 }), Duration::from_millis(300));
    let booked = second
        .book_states_on_the_cards_waiting_only_when_none_has_room(&[0, 1], 30, &[30, 30], &|states| states * 1_000)
        .expect("第 1 张空着，借得到");
    let per_card: Vec<Option<u64>> = booked.iter().map(|b| b.as_ref().map(|s| s.states)).collect();
    println!("PROBE_D states_per_card={per_card:?} (card 0 had room for 5 states, 30 states wanted)");
    drop(booked);
    drop(first_lease);
    let _ = std::fs::remove_dir_all(&directory);
}

/// P5：三个假进程各两条线程，同一张卡，反复按状态数借再还；借着的合计加驱动不许超过卡的估计（锁里记账的原子性）。
#[test]
fn probe_e_concurrent_state_bookings_never_exceed_the_estimate() {
    let directory = scratch_directory("e");
    static HELD: AtomicU64 = AtomicU64::new(0);
    static HIGHEST: AtomicU64 = AtomicU64::new(0);
    let ledgers: Vec<&'static GpuMemoryLedger> = [1u32, 2, 3]
        .iter()
        .map(|pid| {
            let ledger: &'static GpuMemoryLedger = Box::leak(Box::new(open(
                &directory, *pid, &["card-a"], &|_| Ok(40_000), Duration::from_secs(60),
            )));
            ledger
        })
        .collect();
    std::thread::scope(|scope| {
        for ledger in &ledgers {
            for _ in 0..2 {
                scope.spawn(move || {
                    for _ in 0..150 {
                        let booked = ledger
                            .book_states_on_the_cards_waiting_only_when_none_has_room(&[0], 8, &[8], &|states| states * 1_000)
                            .expect("等得到");
                        let mebibytes: u64 = booked.iter().flatten().map(|b| b.booking.mebibytes()).sum();
                        let now = HELD.fetch_add(mebibytes, Ordering::SeqCst) + mebibytes;
                        HIGHEST.fetch_max(now, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_micros(200));
                        HELD.fetch_sub(mebibytes, Ordering::SeqCst);
                        drop(booked);
                    }
                });
            }
        }
    });
    let highest = HIGHEST.load(Ordering::SeqCst);
    println!("PROBE_E highest_leases_held_at_once={highest} limit={}", 40_000 - 3 * DRIVER);
    assert!(highest <= 40_000 - 3 * DRIVER, "借着的合计加三份驱动不超过估计 40 000");
    let _ = std::fs::remove_dir_all(&directory);
}

/// P6：容量三者取最小（K4-1）：在一张格子上穷举，容量不越过任何一个上界。
#[test]
fn probe_f_the_capacity_never_exceeds_any_of_its_bounds() {
    let values = [0u64, 1, 2_559, 2_560, 4_000, 8_192, 20_000, 22_500, 30_000, 40_000];
    let mut cases = 0u64;
    for environment in std::iter::once(None).chain(values.iter().map(|v| Some(*v))) {
        for configured in std::iter::once(None).chain(values.iter().map(|v| Some(*v))) {
            for estimate in values {
                let budget = match configured {
                    Some(quota) => quota.min(estimate),
                    None => estimate / 4 * 3,
                };
                let capacity = card_ledger_capacity_mebibytes(environment, configured, budget, estimate, DRIVER);
                cases += 1;
                if let Some(env) = environment { assert!(capacity <= env, "{environment:?} {configured:?} {estimate}"); }
                if let Some(c) = configured { assert!(capacity <= c.saturating_sub(DRIVER), "{environment:?} {configured:?} {estimate}: {capacity}"); }
                assert!(capacity <= estimate.saturating_sub(DRIVER), "{environment:?} {configured:?} {estimate}: {capacity}");
                if configured.is_none() { assert!(capacity <= (estimate / 4 * 3).saturating_sub(DRIVER), "{environment:?} {configured:?} {estimate}: {capacity}"); }
            }
        }
    }
    println!("PROBE_F cases={cases}");
}

/// P7：卡上只剩 1 000 / 0 MiB（外人占满），不到一份驱动开销（2 560）：登记驱动那一份（调用方随后才在这张卡上起设备）照样登记得上，
/// 因为容量 = 估计 − 驱动 在 `saturating_sub` 里夹成 0，「已记 0 + 要的 0 ≤ 容量 0」成立；借 1 MiB 才被拒。
#[test]
fn probe_g_registering_the_driver_share_passes_on_a_card_that_has_less_free_memory_than_the_driver_share() {
    for free in [1_000u64, 0] {
        let directory = scratch_directory(&format!("g{free}"));
        let ledger = open(&directory, 100, &["card-a"], &|_| Ok(free), Duration::from_millis(100));
        let capacity = ledger.card_capacities_mebibytes().unwrap();
        let registered = ledger.register_this_process_on_the_card_waiting_for_room(0);
        let one_mebibyte = ledger.book_waiting_for_room(1).map(|b| b.mebibytes());
        println!(
            "PROBE_G free={free} capacity={capacity:?} registered={registered:?} registered_flag={:?} booking_of_1={one_mebibyte:?}",
            ledger.is_this_process_registered_on_the_card(0)
        );
        assert_eq!(registered, Ok(()), "登记得上");
        let _ = std::fs::remove_dir_all(&directory);
    }
}
