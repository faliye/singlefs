//! 第 10z 批第三块代码轮第一轮攻方模型（m3-batch10z-c-code-r1，攻方腿）：K1-4「同块几段都报到再认」。
//! 只用冻结副本 harness 档的 `DeviceConditionOfACard`、`WorksEndingTogetherOnADevice`、`WorkOnTheCardOfThisThread` 原样（一行不改），
//! 把 checker 档 `on_the_card_catching_out_of_memory_ending_together`（crash_amplification.rs 第 2176 行起）那几步按同样的次序调：
//! 先记下 `reports_before`，活做完（这里不碰卡），把范围里接到的交给 `end_the_work_and_explain_its_failure`。
//! wgpu 那一半（哪一刻丢设备、哪一刻范围交回）是推的，由各格的次序给定。
use singlefs_harness::gpu_memory_ledger::{
    DeviceConditionOfACard, WorkOnTheCardOfThisThread, WorksEndingTogetherOnADevice,
};
use std::sync::mpsc::channel;
use std::time::Duration;

/// 甲（跨组）：同一个设备上两段活不在同一组报到里（两条用例共用卡池的同一个设备，或卡线程的单段活与别的活）。
/// X 的范围里接到 Out of Memory、wgpu 因它丢了设备；Y 因设备丢了先失败、先收完。`same_group` 为真是对照：两段在同一组。
fn cross_group(same_group: bool) -> (Option<String>, Option<String>) {
    let condition = &DeviceConditionOfACard::default();
    let shared = &WorksEndingTogetherOnADevice::new();
    let alone_x = &WorksEndingTogetherOnADevice::new();
    let alone_y = &WorksEndingTogetherOnADevice::new();
    let (ending_x, ending_y) = if same_group {
        (shared.join(), shared.join())
    } else {
        (alone_x.join(), alone_y.join())
    };
    let (y_began_sender, y_began) = channel::<()>();
    let (y_ended_sender, y_ended) = channel::<()>();
    std::thread::scope(|scope| {
        let x = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            // Y 的活已经开始（它的 reports_before 在这之前记下）
            y_began.recv().unwrap();
            // X 的活里那一次提交撞上 Out of Memory：wgpu 丢了设备（回调记丢失），那一句留在 X 的范围里，X 的活还没收完
            condition.record_the_device_lost("Unknown：提交时显存不够".to_string());
            // 跨组时 X 收得比 Y 晚（Y 先失败先收完）；同组时 Y 在等 X，X 不等 Y
            if !same_group {
                let _ = y_ended.recv_timeout(Duration::from_secs(60));
            }
            condition.end_the_work_and_explain_its_failure(
                &work,
                reports_before,
                Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
                ending_x,
            )
        });
        let y = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            y_began_sender.send(()).unwrap();
            // Y 的活因设备丢了失败（等到丢失记下再收）
            while !condition.is_lost() {
                std::thread::yield_now();
            }
            let explained =
                condition.end_the_work_and_explain_its_failure(&work, reports_before, None, ending_y);
            let _ = y_ended_sender.send(());
            explained
        });
        (x.join().unwrap(), y.join().unwrap())
    })
}

#[test]
fn k1_4_a_work_failing_on_a_device_lost_by_another_group_is_not_told_out_of_memory() {
    for same_group in [false, true] {
        let (x, y) = cross_group(same_group);
        println!(
            "ATTACK k1_4=cross-group same_group={same_group} x_told_out_of_memory={} y_told_out_of_memory={} y_reason={y:?}",
            x.is_some(),
            y.is_some()
        );
    }
}

/// 乙（同组里放宽）：一块两段 A、B。A 的活自己 panic 了（真毛病，与显存无关），A 收完时设备没丢、也没有记过 Out of Memory；
/// 之后 B 的范围里接到一次致命的 Out of Memory、设备丢了。A 照「等齐再认」认：从它开始起设备丢了、记过 Out of Memory，说成显存不够。
/// 对照一：A 在收完那一刻就认（改前的次序）。对照二：A 收完那一刻设备丢没丢（被判的系统那一刻看得到的判别量）。
fn genuine_panic_then_fatal_out_of_memory_in_the_same_block() -> (Option<String>, Option<String>, bool) {
    let condition = &DeviceConditionOfACard::default();
    let together = &WorksEndingTogetherOnADevice::new();
    let ending_a = together.join();
    let ending_b = together.join();
    let (a_failed_sender, a_failed) = channel::<()>();
    std::thread::scope(|scope| {
        let a = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            // A 的活 panic 了（真毛病）：这一刻设备没丢、没有 Out of Memory
            let lost_when_a_failed = condition.is_lost();
            let told_at_the_failure =
                condition.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before);
            a_failed_sender.send(()).unwrap();
            let told_after_waiting =
                condition.end_the_work_and_explain_its_failure(&work, reports_before, None, ending_a);
            (told_after_waiting, told_at_the_failure, lost_when_a_failed)
        });
        scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            a_failed.recv().unwrap();
            condition.record_the_device_lost("Unknown：提交时显存不够".to_string());
            condition.end_the_work_and_explain_its_failure(
                &work,
                reports_before,
                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                ending_b,
            )
        });
        a.join().unwrap()
    })
}

#[test]
fn k1_4_a_genuine_panic_before_the_device_is_lost_is_told_out_of_memory_after_waiting() {
    let (after_waiting, at_the_failure, lost_when_a_failed) =
        genuine_panic_then_fatal_out_of_memory_in_the_same_block();
    println!(
        "ATTACK k1_4=genuine-panic-widened told_after_waiting={} told_at_the_failure={} lost_when_a_failed={lost_when_a_failed} reason={after_waiting:?}",
        after_waiting.is_some(),
        at_the_failure.is_some()
    );
}

/// 丙（挂不挂住）：一组 `pieces` 段，每段一条线程（同 checker 档 check_pending_blocks_in_pieces，第 1498–1510 行：报到在起线程之前都领出来）。
/// 每段随机一种收法：照常经 `end_the_work_and_explain_its_failure` 收（范围里有没有 Out of Memory 随机）、活 panic 了（报到随 unwind 丢掉）、
/// 活里套一段单独一组的活（同 on_the_card_catching_out_of_memory 的 `alone`）再照常收；收之前随机让出几次。
/// 另试两种起线程的毛病：起到第 k 段时起线程那一步 panic（剩下的报到随迭代器丢掉）。每组 60 秒内收齐算没挂住。
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
    let (done_sender, done) = channel::<()>();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::thread::scope(|scope| {
            for (index, (ending, (how, yields))) in endings.into_iter().zip(plans).enumerate() {
                if spawn_fails_at == Some(index) {
                    panic!("起线程失败（模型）");
                }
                let done_sender = done_sender.clone();
                scope.spawn(move || {
                    let work = WorkOnTheCardOfThisThread::begin();
                    let reports_before = condition.out_of_memory_reports();
                    for _ in 0..yields {
                        std::thread::yield_now();
                    }
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || match how {
                        0 => {
                            let _ = condition.end_the_work_and_explain_its_failure(
                                &work,
                                reports_before,
                                (yields % 2 == 0).then(|| "Out of Memory".to_string()),
                                ending,
                            );
                        }
                        1 => {
                            let _moved = ending;
                            panic!("活 panic 了（模型）");
                        }
                        _ => {
                            let inner_alone = WorksEndingTogetherOnADevice::new();
                            let inner = WorkOnTheCardOfThisThread::begin();
                            let _ = condition.end_the_work_and_explain_its_failure(
                                &inner,
                                reports_before,
                                None,
                                inner_alone.join(),
                            );
                            drop(inner);
                            let _ = condition.end_the_work_and_explain_its_failure(
                                &work,
                                reports_before,
                                None,
                                ending,
                            );
                        }
                    }));
                    let _ = result;
                    let _ = done_sender.send(());
                });
            }
        });
    }));
    drop(done_sender);
    let _ = outcome;
    // scope 返回就是每条线程都收完了；没挂住
    let _ = done.try_iter().count();
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
                    if spawn_fails_at.is_some() {
                        with_spawn_failure += 1;
                    } else {
                        clean += 1;
                    }
                }
            }
        }
        let _ = finished_sender.send((groups, clean, with_spawn_failure));
    });
    let outcome = finished.recv_timeout(Duration::from_secs(60));
    println!("ATTACK k1_4=hang-probe outcome={outcome:?} (Err 就是 60 秒内没收齐：挂住)");
    assert!(outcome.is_ok(), "挂住了");
}

/// 甲的对照格（同组）重跑 2000 次：Y 认成显存不够的次数。给 K3 量「改回去」的变异在同一形上漏几次。
#[test]
fn k1_4_same_group_repeated() {
    let runs = 2_000;
    let told = (0..runs).filter(|_| cross_group(true).1.is_some()).count();
    println!("ATTACK k1_4=same-group-repeated runs={runs} y_told_out_of_memory={told}");
}
