// 正推腿：第 D 条「真 panic 被说成显存不够」的两形，只用公开函数。
use singlefs_harness::gpu_memory_ledger::{
    DeviceConditionOfACard, WorkOnTheCardOfThisThread, WorksEndingTogetherOnADevice,
};

fn two_pieces(device_lost: bool) -> (Option<String>, Option<String>) {
    let condition = &DeviceConditionOfACard::default();
    let together = &WorksEndingTogetherOnADevice::new();
    let a_ending = together.join();
    let b_ending = together.join();
    std::thread::scope(|scope| {
        // A：真 panic（没有任何 OOM 记录跟它有关）
        let a = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let before = condition.out_of_memory_reports();
            condition.end_the_work_and_explain_its_failure(&work, before, None, a_ending)
        });
        // B：范围里接到了 OOM；设备丢没丢按参数
        let b = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let before = condition.out_of_memory_reports();
            std::thread::sleep(std::time::Duration::from_millis(50));
            if device_lost {
                condition.record_the_device_lost("Unknown：提交时显存不够".to_string());
            }
            condition.end_the_work_and_explain_its_failure(
                &work,
                before,
                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                b_ending,
            )
        });
        (a.join().unwrap(), b.join().unwrap())
    })
}

#[test]
fn a_real_panic_next_to_a_piece_that_ran_out_of_memory() {
    let (a, b) = two_pieces(false);
    println!("EXPLAIN device_lost=false: piece A (real panic) explained as {a:?}; piece B (caught OOM) explained as {b:?}");
    let (a, b) = two_pieces(true);
    println!("EXPLAIN device_lost=true:  piece A (real panic) explained as {a:?}; piece B (caught OOM) explained as {b:?}");
}
