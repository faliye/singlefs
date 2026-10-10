//! 攻方原型（m3-batch10o-code-r1 云端攻方，只在冻结副本的拷贝上；不是入库用例）：跨进程显存账的几形（K4）。
//! 只用 `singlefs_harness::gpu_memory_ledger` 的公开接口与假进程号，不碰显卡。每条打一行 `OPUSO1-K4 …`。
//! 容量照 checker 档 `gpu_test_start::open_the_ledger_of_the_started_cards` 的算法在这里抄一份（`memory_budget_mebibytes_of` 在 checker 档）：
//! 没给额度取空闲的 3/4，给了额度取 min(额度, 空闲)；再照 `card_ledger_capacity_mebibytes` 减驱动那一份 2560。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, BookingRefusal, CardToRegister, GpuMemoryLedger, LedgerSettings,
    RecordedProcess,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DRIVER: u64 = 2560;

fn budget(config_quota: Option<u64>, free: u64) -> u64 {
    match config_quota {
        Some(q) => q.min(free),
        None => free / 4 * 3,
    }
}

fn dir(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("opus-o1-k4-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

/// 一个假进程起池：`free_read` 是它自己起池时读到的空闲显存；交回账与它看到的「清场后空闲」（账本里记的那一份）。
fn register(
    directory: &PathBuf,
    pid: u32,
    alive: Arc<Mutex<Vec<u32>>>,
    free_read: u64,
    env_quota: Option<u64>,
    config_quota: Option<u64>,
    longest_wait: Duration,
) -> (GpuMemoryLedger, u64) {
    let seen = Arc::new(Mutex::new(0u64));
    let seen2 = seen.clone();
    let ledger = GpuMemoryLedger::register_this_process_on_the_cards(
        LedgerSettings {
            directory: directory.clone(),
            this_process: RecordedProcess { process_identifier: pid, start_time_in_clock_ticks: 1 },
            is_alive: Box::new(move |p: RecordedProcess| alive.lock().unwrap().contains(&p.process_identifier)),
            mebibytes_each_other_process_keeps_on_a_card: DRIVER,
            poll_interval_for_memory_returned_by_other_processes: Duration::from_millis(20),
            longest_wait,
        },
        &[CardToRegister { file_stem: "card-01-00.0".into(), free_memory_read_by_this_process_mebibytes: free_read }],
        move |_pos, f| {
            *seen2.lock().unwrap() = f;
            card_ledger_capacity_mebibytes(env_quota, budget(config_quota, f), f, DRIVER)
        },
    )
    .expect("账本");
    let f = *seen.lock().unwrap();
    (ledger, f)
}

/// K4-3 甲：账上记的「清场后空闲」在一串重叠的进程之间一直传下去，不随卡上真实的空闲变。
/// P1 起池时卡空 30000；之后一个不记账的 GPU 进程（崩溃放量、对拍里取整张卡那条、别的服务）占了 20000；P2 起池时自己读到 10000，
/// 账却按 30000 给它容量，借满容量 → 卡上要的比真空闲多。
#[test]
fn k4_stale_free_memory_is_inherited_by_overlapping_processes() {
    let d = dir("stale-high");
    let alive = Arc::new(Mutex::new(vec![1u32, 2]));
    let (p1, f1) = register(&d, 1, alive.clone(), 30_000, None, None, Duration::from_secs(5));
    let (p2, f2) = register(&d, 2, alive.clone(), 10_000, None, None, Duration::from_secs(5));
    let cap2 = p2.card_capacities_mebibytes()[0];
    // P2 借到它看到的容量减去 P1 那一份驱动开销为止
    let need = cap2 - DRIVER;
    let b = p2.book_waiting_for_room(need).expect("账上放得下");
    let real_free_at_p2_start = 10_000u64;
    println!(
        "OPUSO1-K4 stale-high p1_free_recorded={f1} p2_free_read=10000 p2_free_used={f2} p2_capacity={cap2} p2_booked={} p2_driver={DRIVER} p2_needs_on_card={} real_free_when_p2_started={real_free_at_p2_start} over_by={}",
        b.mebibytes(),
        b.mebibytes() + DRIVER,
        (b.mebibytes() + DRIVER).saturating_sub(real_free_at_p2_start)
    );
    assert_eq!(f2, 30_000, "P2 照账本里 P1 记的那一份");
    // P1 退出，P2 还活着；P3 起池时读到 31000（外人走了），照样拿 30000：只要链不断，记的那一份不换
    alive.lock().unwrap().retain(|p| *p != 1);
    drop(p1);
    alive.lock().unwrap().push(3);
    let (_p3, f3) = register(&d, 3, alive.clone(), 31_000, None, None, Duration::from_secs(5));
    println!("OPUSO1-K4 chain p3_free_read=31000 p3_free_used={f3}");
    drop(b);
    let _ = std::fs::remove_dir_all(&d);
}

/// K4-3 乙：反过来，P1 起池那一刻卡被外人占着（读到 10000），外人走了以后链上的进程照 10000 算：
/// 要 6000 的用例「永远放不下」判红，要 3000 的在 P1 借着 2000 时等满最长等待报本次未跑——卡上其实空着 27000 以上。
#[test]
fn k4_stale_low_free_memory_makes_tests_red_or_not_run_while_the_card_is_empty() {
    let d = dir("stale-low");
    let alive = Arc::new(Mutex::new(vec![1u32, 2]));
    let (p1, f1) = register(&d, 1, alive.clone(), 10_000, None, None, Duration::from_millis(300));
    let (p2, f2) = register(&d, 2, alive.clone(), 30_000, None, None, Duration::from_millis(300));
    let cap2 = p2.card_capacities_mebibytes()[0];
    let red = p2.book_waiting_for_room(6000).err();
    let held = p1.book_waiting_for_room(2000).expect("P1 借 2000");
    let not_run = p2.book_waiting_for_room(3000).err();
    println!(
        "OPUSO1-K4 stale-low p1_free_recorded={f1} p2_free_read=30000 p2_free_used={f2} p2_capacity={cap2} need6000={red:?} need3000_while_p1_holds2000={not_run:?}"
    );
    assert!(matches!(red, Some(BookingRefusal::NeedLargerThanEveryCard(_))));
    assert!(matches!(not_run, Some(BookingRefusal::WaitedPastTheLongestWait { .. })));
    drop(held);
    let _ = std::fs::remove_dir_all(&d);
}

/// K4-1：设了 SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES，账的容量不再经配置给这张卡的额度（GPU_CARDS_BY_PRIORITY 的 8G）与 3/4 那一份。
#[test]
fn k4_environment_quota_replaces_the_configured_card_quota() {
    let f = 30_000;
    let with_config_only = card_ledger_capacity_mebibytes(None, budget(Some(8192), f), f, DRIVER);
    let with_env = card_ledger_capacity_mebibytes(Some(20_000), budget(Some(8192), f), f, DRIVER);
    let with_env_no_config = card_ledger_capacity_mebibytes(Some(40_000), budget(None, f), f, DRIVER);
    let no_env_no_config = card_ledger_capacity_mebibytes(None, budget(None, f), f, DRIVER);
    println!(
        "OPUSO1-K4 env-quota free={f} config8192_no_env={with_config_only} config8192_env20000={with_env} no_config_no_env={no_env_no_config} no_config_env40000={with_env_no_config}"
    );
    assert!(with_env > with_config_only);
}

/// K4-2：两个进程几乎同时起池，P2 的设备先起（驱动那一份已在卡上）、P1 的 nvidia-smi 后读：P1 先拿到锁、记下少了 P2 那一份的空闲；
/// 两边又各给对方记一份驱动开销，P2 的那一份算了两次。
#[test]
fn k4_simultaneous_first_registration_counts_one_driver_share_twice() {
    let d = dir("simultaneous");
    let alive = Arc::new(Mutex::new(vec![1u32, 2]));
    let (p1, f1) = register(&d, 1, alive.clone(), 30_000 - DRIVER, None, None, Duration::from_millis(200));
    let (p2, f2) = register(&d, 2, alive.clone(), 30_000, None, None, Duration::from_millis(200));
    let cap = p1.card_capacities_mebibytes()[0];
    // 照 3/4 那一份能借多少：全卡按真实 30000 算时两个进程合计能借 3/4*30000 - 2*2560
    let mut total = 0u64;
    let mut held = Vec::new();
    for _ in 0..1000 {
        match p1.book_waiting_for_room(100) {
            Ok(b) => {
                total += 100;
                held.push(b);
            }
            Err(_) => break,
        }
    }
    let ideal = budget(None, 30_000) - 2 * DRIVER;
    println!("OPUSO1-K4 simultaneous f1={f1} f2={f2} capacity={cap} booked_in_100s={total} with_true_free={ideal} short_by={}", ideal - total);
    drop(held);
    drop(p2);
    let _ = std::fs::remove_dir_all(&d);
}

/// K4-2：被 SIGKILL、父进程还没收尸的进程（僵尸）照「还活着」算：它借的那一份要等收尸才回收。用真的子进程判。
#[test]
fn k4_a_killed_but_unreaped_process_is_judged_alive() {
    use singlefs_harness::gpu_memory_ledger::{process_is_alive_by_its_entry_in_the_process_filesystem, start_time_in_clock_ticks_from_the_process_status_line};
    let mut child = std::process::Command::new("sleep").arg("1000").spawn().expect("sleep");
    let pid = child.id();
    let line = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    let rec = RecordedProcess { process_identifier: pid, start_time_in_clock_ticks: start_time_in_clock_ticks_from_the_process_status_line(&line).unwrap() };
    child.kill().expect("SIGKILL");
    std::thread::sleep(Duration::from_millis(200));
    let zombie = process_is_alive_by_its_entry_in_the_process_filesystem(rec);
    let state = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok().and_then(|l| l.rsplit_once(')').map(|(_, r)| r.split_whitespace().next().unwrap_or("").to_string()));
    child.wait().unwrap();
    let reaped = process_is_alive_by_its_entry_in_the_process_filesystem(rec);
    println!("OPUSO1-K4 zombie state_after_kill={state:?} alive_while_zombie={zombie} alive_after_reap={reaped}");
}
