//! 正推腿：第 10z 批第五块 K1-2。上一轮攻方 attack_10zc_endings.rs 的甲形（跨组）与乙形（同组放宽）、正推 explain_demo.rs，改到新接口：
//! `record_the_device_lost(why, cause)`、`device_when_the_work_returns()`、`end_the_work_and_explain_its_failure(work, before, device_when_returned, caught, ending)`。
//! 只用 harness 档 `DeviceConditionOfACard` 等的公开函数（一行不改）。wgpu 那一半（哪一刻丢设备、哪一刻读「交回那一刻设备丢没丢」）是推的，次序由各格给定；
//! checker 档 `on_the_card_catching_out_of_memory_ending_together`（冻结副本第 2187–2225 行）的调用次序：begin → reports_before → 活 → device_when_the_work_returns → 范围弹出 → end_the_work_and_explain_its_failure。
use singlefs_harness::gpu_memory_ledger::{
    CauseOfTheDeviceLoss, DeviceConditionOfACard, DeviceWhenTheWorkReturned, WorkOnTheCardOfThisThread,
    WorksEndingTogetherOnADevice,
};
use std::sync::mpsc::channel;
use std::time::Duration;

fn show(x: &Option<String>) -> String {
    match x {
        Some(s) => format!("Some({})", s.chars().take(40).collect::<String>()),
        None => "None".to_string(),
    }
}

/// 甲（跨组）：X 范围里接到 OOM、wgpu 因它丢了设备（回调记丢失，缘由 cause）；Y 因设备丢了先失败、先收完，不在同一组。
fn cross_group(same_group: bool, cause: CauseOfTheDeviceLoss) -> (Option<String>, Option<String>) {
    let condition = &DeviceConditionOfACard::default();
    let shared = &WorksEndingTogetherOnADevice::new();
    let alone_x = &WorksEndingTogetherOnADevice::new();
    let alone_y = &WorksEndingTogetherOnADevice::new();
    let (ending_x, ending_y) = if same_group { (shared.join(), shared.join()) } else { (alone_x.join(), alone_y.join()) };
    let (y_began_sender, y_began) = channel::<()>();
    let (y_ended_sender, y_ended) = channel::<()>();
    std::thread::scope(|scope| {
        let x = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            y_began.recv().unwrap();
            condition.record_the_device_lost("Unknown：Out of memory".to_string(), cause);
            let returned = condition.device_when_the_work_returns();
            if !same_group {
                let _ = y_ended.recv_timeout(Duration::from_secs(60));
            }
            condition.end_the_work_and_explain_its_failure(&work, reports_before, returned, Some("序号 0 的卡判一块时报了 Out of Memory".to_string()), ending_x)
        });
        let y = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            y_began_sender.send(()).unwrap();
            while !condition.is_lost() {
                std::thread::yield_now();
            }
            let returned = condition.device_when_the_work_returns();
            let explained = condition.end_the_work_and_explain_its_failure(&work, reports_before, returned, None, ending_y);
            let _ = y_ended_sender.send(());
            explained
        });
        (x.join().unwrap(), y.join().unwrap())
    })
}

#[test]
fn k1_4_cross_group() {
    for cause in [CauseOfTheDeviceLoss::GpuMemoryShort, CauseOfTheDeviceLoss::Other] {
        for same_group in [false, true] {
            let (x, y) = cross_group(same_group, cause);
            println!("ENDINGS_NEW k1_4=cross-group cause={cause:?} same_group={same_group} x_told_out_of_memory={} y_told_out_of_memory={} y={}", x.is_some(), y.is_some(), show(&y));
        }
    }
}

/// 乙：一块两段 A、B，A 真 panic（与显存无关）。`b_loses_the_device_before_a_reads` 为真是窗口形：B 的致命 OOM 在 A 读「交回那一刻设备丢没丢」之前已经让设备丢了；
/// 为假是 A 先读（设备没丢），B 之后才丢。两形都问 A 被说成什么。
fn genuine_panic(b_loses_the_device_before_a_reads: bool, cause: CauseOfTheDeviceLoss) -> (Option<String>, DeviceWhenTheWorkReturned) {
    let condition = &DeviceConditionOfACard::default();
    let together = &WorksEndingTogetherOnADevice::new();
    let (ending_a, ending_b) = (together.join(), together.join());
    let (a_read_sender, a_read) = channel::<()>();
    let (b_lost_sender, b_lost) = channel::<()>();
    std::thread::scope(|scope| {
        let a = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            if b_loses_the_device_before_a_reads {
                b_lost.recv().unwrap();
            }
            let returned = condition.device_when_the_work_returns();
            let _ = a_read_sender.send(());
            let told = condition.end_the_work_and_explain_its_failure(&work, reports_before, returned, None, ending_a);
            (told, returned)
        });
        scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            if !b_loses_the_device_before_a_reads {
                a_read.recv().unwrap();
            }
            condition.record_the_device_lost("Unknown：Out of memory".to_string(), cause);
            let _ = b_lost_sender.send(());
            let returned = condition.device_when_the_work_returns();
            condition.end_the_work_and_explain_its_failure(&work, reports_before, returned, Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()), ending_b)
        });
        a.join().unwrap()
    })
}

#[test]
fn k1_4_genuine_panic_next_to_a_fatal_out_of_memory() {
    for cause in [CauseOfTheDeviceLoss::GpuMemoryShort, CauseOfTheDeviceLoss::Other] {
        for before in [false, true] {
            let (told, returned) = genuine_panic(before, cause);
            println!("ENDINGS_NEW k1_4=genuine-panic cause={cause:?} b_loses_before_a_reads={before} a_returned={returned:?} a_told_out_of_memory={} a={}", told.is_some(), show(&told));
        }
    }
}

/// 丙：真的显存不够。Y 的活因设备无效失败（它自己的线程上没有 OOM 报到）。
/// 正常：回调先记丢失，Y 再读 → Lost → 认得出。窗口 1：wgpu 先置设备无效、后调回调（wgpu-core 30.0.1 resource.rs 第 5420 行与第 5424 行），Y 在两者之间读 → NotLost → 回调才记 → Y 认不出。
fn real_out_of_memory(window_one: bool, cause: CauseOfTheDeviceLoss) -> Option<String> {
    let condition = DeviceConditionOfACard::default();
    let alone = WorksEndingTogetherOnADevice::new();
    let work = WorkOnTheCardOfThisThread::begin();
    let before = condition.out_of_memory_reports();
    let ending = alone.join();
    let returned;
    if window_one {
        returned = condition.device_when_the_work_returns();
        condition.record_the_device_lost("Unknown：Out of memory".to_string(), cause);
    } else {
        condition.record_the_device_lost("Unknown：Out of memory".to_string(), cause);
        returned = condition.device_when_the_work_returns();
    }
    condition.end_the_work_and_explain_its_failure(&work, before, returned, None, ending)
}

#[test]
fn k1_4_real_out_of_memory_windows() {
    for cause in [CauseOfTheDeviceLoss::GpuMemoryShort, CauseOfTheDeviceLoss::Other] {
        for window_one in [false, true] {
            let told = real_out_of_memory(window_one, cause);
            println!("ENDINGS_NEW k1_4=real-out-of-memory cause={cause:?} window_one={window_one} y_told_out_of_memory={} y={}", told.is_some(), show(&told));
        }
    }
}

/// 函数语义表：一段活自己没报过 OOM；设备（丢了 Lost / 没丢 NotLost）× 缘由 × 这段活开始之后别的活有没有记过 OOM。
#[test]
fn k1_4_truth_table_of_the_explanation() {
    for returned in [DeviceWhenTheWorkReturned::Lost, DeviceWhenTheWorkReturned::NotLost] {
        for cause in [CauseOfTheDeviceLoss::GpuMemoryShort, CauseOfTheDeviceLoss::Other] {
            for lost_recorded in [true, false] {
                for other_work_report in [false, true] {
                    let condition = DeviceConditionOfACard::default();
                    let work = WorkOnTheCardOfThisThread::begin();
                    let before = condition.out_of_memory_reports();
                    if other_work_report {
                        // 别的线程上另一段活的 OOM（在这条线程上记就算这段活的，所以放到别的线程）
                        std::thread::scope(|s| {
                            s.spawn(|| {
                                let _other = WorkOnTheCardOfThisThread::begin();
                                condition.record_an_out_of_memory("别的段的 OOM".to_string());
                            });
                        });
                    }
                    if lost_recorded {
                        condition.record_the_device_lost("Unknown：x".to_string(), cause);
                    }
                    let told = condition.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), before, returned);
                    println!("ENDINGS_NEW table returned={returned:?} cause={cause:?} loss_recorded={lost_recorded} other_work_reported_oom={other_work_report} => {}", show(&told));
                }
            }
        }
    }
}

/// 丁（挂不挂住，上一轮攻方丙形）：每组 60 秒内收齐；新接口下重跑。
fn one_group(seed: &mut u64, pieces: usize, spawn_fails_at: Option<usize>) -> bool {
    let mut next = |bound: u64| {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        *seed % bound
    };
    let plans: Vec<(u64, u64)> = (0..pieces).map(|_| (next(3), next(50))).collect();
    let condition = &DeviceConditionOfACard::default();
    let together = &WorksEndingTogetherOnADevice::new();
    let endings: Vec<_> = (0..pieces).map(|_| together.join()).collect();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::thread::scope(|scope| {
            for (index, (ending, (how, yields))) in endings.into_iter().zip(plans).enumerate() {
                if spawn_fails_at == Some(index) {
                    panic!("起线程失败（模型）");
                }
                scope.spawn(move || {
                    let work = WorkOnTheCardOfThisThread::begin();
                    let reports_before = condition.out_of_memory_reports();
                    for _ in 0..yields {
                        std::thread::yield_now();
                    }
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || match how {
                        0 => {
                            let returned = condition.device_when_the_work_returns();
                            let _ = condition.end_the_work_and_explain_its_failure(&work, reports_before, returned, (yields % 2 == 0).then(|| "Out of Memory".to_string()), ending);
                        }
                        1 => {
                            let _moved = ending;
                            panic!("活 panic 了（模型）");
                        }
                        _ => {
                            let inner_alone = WorksEndingTogetherOnADevice::new();
                            let inner = WorkOnTheCardOfThisThread::begin();
                            let returned = condition.device_when_the_work_returns();
                            let _ = condition.end_the_work_and_explain_its_failure(&inner, reports_before, returned, None, inner_alone.join());
                            drop(inner);
                            let returned = condition.device_when_the_work_returns();
                            let _ = condition.end_the_work_and_explain_its_failure(&work, reports_before, returned, None, ending);
                        }
                    }));
                });
            }
        });
    }));
    let _ = outcome;
    together.works_not_yet_ended() == 0
}

#[test]
fn k1_4_groups_of_pieces_never_wait_forever() {
    let (finished_sender, finished) = channel::<(u64, u64, u64)>();
    std::thread::spawn(move || {
        let mut seed = 0x2545_F491_4F6C_DD1D_u64;
        let (mut groups, mut clean, mut with_spawn_failure) = (0u64, 0u64, 0u64);
        for pieces in [1usize, 2, 3, 4, 8, 16, 32] {
            for round in 0..40 {
                let spawn_fails_at = (round % 4 == 3).then(|| round % pieces);
                let ok = one_group(&mut seed, pieces, spawn_fails_at);
                groups += 1;
                if ok {
                    if spawn_fails_at.is_some() { with_spawn_failure += 1 } else { clean += 1 }
                }
            }
        }
        let _ = finished_sender.send((groups, clean, with_spawn_failure));
    });
    let outcome = finished.recv_timeout(Duration::from_secs(60));
    println!("ENDINGS_NEW k1_4=hang-probe outcome={outcome:?} (Err 就是 60 秒内没收齐：挂住)");
    assert!(outcome.is_ok(), "挂住了");
}

/// 同组对照 2000 次（甲形同组）：Y 认成显存不够的次数
#[test]
fn k1_4_same_group_and_cross_group_repeated() {
    let runs = 2_000;
    let same = (0..runs).filter(|_| cross_group(true, CauseOfTheDeviceLoss::GpuMemoryShort).1.is_some()).count();
    let cross = (0..runs).filter(|_| cross_group(false, CauseOfTheDeviceLoss::GpuMemoryShort).1.is_some()).count();
    println!("ENDINGS_NEW k1_4=repeated runs={runs} same_group_y_told={same} cross_group_y_told={cross}");
}
