//! 设备上记的 Out of Memory 与设备丢失：按线程上那段活认是不是显存不够、编内核的那几段活，
//! 同一个设备上丢失那一行只打一次。

use crate::gpu_memory_ledger::{
    gpu_device_lost_line, CauseOfTheDeviceLoss, DeviceConditionOfACard, DeviceWhenTheWorkReturned,
    WorkOnTheCardOfThisThread, WorksEndingTogetherOnADevice,
};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 同一个设备上，别的线程上那段活记的 Out of Memory 说不成这段活的显存不够
/// （这段活真正的 panic 照样往外抛）；这段活自己这条线程上记的说得成。
#[test]
fn an_out_of_memory_in_the_work_of_another_thread_does_not_explain_a_failure_of_this_work() {
    let condition = DeviceConditionOfACard::default();
    let this_work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let _another_work = WorkOnTheCardOfThisThread::begin();
            condition.record_an_out_of_memory("别的用例在这张卡上撞满".to_string());
        });
    });
    assert_eq!(condition.out_of_memory_reports(), reports_before + 1);
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            this_work.identifier(),
            reports_before,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        None,
        "别的线程上那段活的 Out of Memory：说不成这段活的显存不够"
    );
    condition.record_an_out_of_memory("这段活自己判一块时".to_string());
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            this_work.identifier(),
            reports_before,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        Some("这段活自己判一块时".to_string()),
        "这段活自己这条线程上记的：说得成"
    );
}

/// 判器内核在活另起的线程上编，那几条线程上没有活；在这个设备上为这段活编内核的那一刻记的 Out of Memory 算这段活的，
/// 编完之后没有活的线程上记的哪段活都说不成。
#[test]
fn an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then() {
    let condition = DeviceConditionOfACard::default();
    let compiling_work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    condition.compile_for_the_work_of_this_thread(|| {
        std::thread::scope(|scope| {
            scope.spawn(|| {
                condition.record_an_out_of_memory("另起的线程上编恢复内核时没接住的".to_string());
            });
        });
    });
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            compiling_work.identifier(),
            reports_before,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        Some("另起的线程上编恢复内核时没接住的".to_string()),
        "编内核的那一刻另起的线程上记的：算这段活的"
    );
    let reports_after_compiling = condition.out_of_memory_reports();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            condition.record_an_out_of_memory("编完之后没有活的线程上记的".to_string());
        });
    });
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            compiling_work.identifier(),
            reports_after_compiling,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        None,
        "编完之后没有活的线程上记的：说不成"
    );
}

/// 设备为别的原因丢了，而这段活开始之后没记过 Out of Memory（早先一段活上处理过的一次不算），
/// 说不成显存不够；这段活开始之后记过（别的活上的也算：丢了设备谁都做不成），说成设备因它丢了。
#[test]
fn lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began() {
    let condition = DeviceConditionOfACard::default();
    condition.record_an_out_of_memory("早先处理过的一次".to_string());
    let work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    condition.record_the_device_lost("Unknown：驱动重置".to_string(), CauseOfTheDeviceLoss::Other);
    let device_when_the_work_returned = condition.device_when_the_work_returns();
    assert_eq!(
        device_when_the_work_returned,
        DeviceWhenTheWorkReturned::Lost
    );
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            work.identifier(),
            reports_before,
            device_when_the_work_returned
        ),
        None,
        "设备为别的原因丢了而这段活开始之后没报过：不是显存不够"
    );
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let _another_work = WorkOnTheCardOfThisThread::begin();
            condition.record_an_out_of_memory("别的活上撞满".to_string());
        });
    });
    assert!(
        condition
            .out_of_memory_explaining_a_failure_of_the_work(
                work.identifier(),
                reports_before,
                device_when_the_work_returned
            )
            .is_some_and(|reason| reason.starts_with("设备丢了")),
        "设备丢了、这段活开始之后报过：说成设备因它丢了"
    );
}

/// 在这条线程名的线程上记两次丢失、再报一次没等成，交回打出的每一行（告诉一个收集的闭包，不真打：
/// 这些用例因断言红时输出里没有 `GPU_DEVICE_LOST`，变异判「抓到」照常算）。
fn marker_lines_told_on_a_thread(
    thread_builder: std::thread::Builder,
    condition: DeviceConditionOfACard,
    first_why: &'static str,
) -> (Vec<String>, DeviceConditionOfACard) {
    thread_builder
        .spawn(move || {
            let mut marker_lines = Vec::new();
            condition.record_the_device_lost_telling_the_marker_line(
                first_why.to_string(),
                CauseOfTheDeviceLoss::Other,
                |marker_line| marker_lines.push(marker_line.to_string()),
            );
            condition.record_the_device_lost_telling_the_marker_line(
                "Unknown：回调记的缘由更准".to_string(),
                CauseOfTheDeviceLoss::GpuMemoryShort,
                |marker_line| marker_lines.push(marker_line.to_string()),
            );
            condition.tell_the_device_lost_marker_line_the_first_time(
                "等卡又没等成",
                |marker_line| marker_lines.push(marker_line.to_string()),
            );
            (marker_lines, condition)
        })
        .expect("起得了线程")
        .join()
        .expect("这条线程上只记丢失、不 panic")
}

#[test]
fn the_device_lost_marker_line_is_told_for_the_first_loss_and_never_again_on_that_device() {
    let (marker_lines, condition) = marker_lines_told_on_a_thread(
        std::thread::Builder::new().name("some_test_name".to_string()),
        DeviceConditionOfACard::on_the_card("假的 GeForce 卡".to_string()),
        "等卡没等成：超时",
    );
    assert_eq!(
        marker_lines,
        vec![
            "GPU_DEVICE_LOST test=some_test_name card=假的 GeForce 卡 reason=等卡没等成：超时"
                .to_string()
        ],
        "同一个设备记两次丢失、再报一次没等成，只打头一次那一行"
    );
    assert_eq!(
        condition.lost_reason(),
        Some("Unknown：回调记的缘由更准".to_string()),
        "打不打那一行不改记丢失的行为：后记的照旧盖掉先记的"
    );
}

/// 头一次是只打行、不记丢失的那一处（等卡没等成而不记丢失）：打这一次；之后真记丢失不再打，设备照样记成丢了。
#[test]
fn failed_wait_that_told_the_marker_line_keeps_a_later_loss_from_telling_it_again() {
    let condition = DeviceConditionOfACard::on_the_card("假的卡".to_string());
    let mut marker_lines = Vec::new();
    condition.tell_the_device_lost_marker_line_the_first_time("等卡没等成", |marker_line| {
        marker_lines.push(marker_line.to_string());
    });
    assert!(!condition.is_lost(), "只打行不把设备记成丢了");
    condition.record_the_device_lost_telling_the_marker_line(
        "Unknown：驱动重置".to_string(),
        CauseOfTheDeviceLoss::Other,
        |marker_line| marker_lines.push(marker_line.to_string()),
    );
    assert!(condition.is_lost(), "之后记丢失照样记成丢了");
    assert_eq!(marker_lines.len(), 1, "只打一行：{marker_lines:?}");
    assert!(
        marker_lines[0].ends_with(" reason=等卡没等成"),
        "打的是头一次那一句：{marker_lines:?}"
    );
}

#[test]
fn the_device_lost_marker_line_takes_the_line_breaks_out_of_its_fields() {
    assert_eq!(
            gpu_device_lost_line(
                "some_test",
                "假的\n卡",
                "Validation Error\n\nCaused by:\r\n  第三行\u{b}第四行\u{2028}第五行\n"
            ),
            "GPU_DEVICE_LOST test=some_test card=假的 卡 reason=Validation Error Caused by:   第三行 第四行 第五行",
            "几行的错误文字接成一行，连着的几个换行算一处"
        );
}

#[test]
fn condition_that_is_not_on_a_card_tells_no_marker_line_and_still_records_the_loss() {
    let (marker_lines, condition) = marker_lines_told_on_a_thread(
        std::thread::Builder::new().name("some_test_name".to_string()),
        DeviceConditionOfACard::default(),
        "等卡没等成：超时",
    );
    assert_eq!(
        marker_lines,
        Vec::<String>::new(),
        "假的设备情况上记丢失不打那一行：用例因断言红时不许被认成卡出了事"
    );
    assert!(condition.is_lost(), "丢失照样记");
}

#[test]
fn thread_without_a_name_is_written_as_unnamed_in_the_marker_line() {
    let (marker_lines, _condition) = marker_lines_told_on_a_thread(
        std::thread::Builder::new(),
        DeviceConditionOfACard::on_the_card("假的卡".to_string()),
        "等卡没等成",
    );
    assert_eq!(
        marker_lines,
        vec!["GPU_DEVICE_LOST test=unnamed-thread card=假的卡 reason=等卡没等成".to_string()]
    );
}

/// （正推 N1）：同一条线程上活里又起一段活，里面那段丢掉之后，这条线程上记的 Out of Memory 算回外面那段。
#[test]
fn work_begun_inside_another_hands_the_thread_back_to_the_outer_work_when_it_ends() {
    let condition = DeviceConditionOfACard::default();
    let outer_work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    drop(WorkOnTheCardOfThisThread::begin());
    condition.record_an_out_of_memory("里面那段收完之后外面那段撞满".to_string());
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            outer_work.identifier(),
            reports_before,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        Some("里面那段收完之后外面那段撞满".to_string())
    );
}

/// （正推 N2、N3）：同一段活在这个设备上同时两处编内核，里面那一处编完、外面那一处还在编时，
/// 没有活的线程上记的 Out of Memory 照样算这段活的。
#[test]
fn work_compiling_in_two_places_at_once_stays_compiling_until_both_end() {
    let condition = DeviceConditionOfACard::default();
    let compiling_work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    condition.compile_for_the_work_of_this_thread(|| {
        condition.compile_for_the_work_of_this_thread(|| {});
        std::thread::scope(|scope| {
            scope.spawn(|| {
                condition.record_an_out_of_memory("外面那一处还在编时没接住的".to_string());
            });
        });
    });
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            compiling_work.identifier(),
            reports_before,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        Some("外面那一处还在编时没接住的".to_string())
    );
}

/// 编内核的闭包造出来交出去的那一份：堆上的一块。交回的是不是闭包造的那一块按地址认（复制、重造的地址不同），值另比。
struct OutputOfTheCompileClosure {
    heap_block: Box<[u64; 2]>,
}

/// 编内核的闭包造的那一块里的值。
const VALUE_MADE_IN_THE_COMPILE_CLOSURE: [u64; 2] = [0x5EED_C0DE, 7];

/// 经 `condition` 的 [`DeviceConditionOfACard::compile_for_the_work_of_this_thread`] 编一次：闭包里先做 `while_compiling`，
/// 再造一份 [`OutputOfTheCompileClosure`] 交出去。断言闭包恰好调了一次、交回的是闭包造的那一块（同一个地址、同一个值）。
/// 编内核那一步 panic 了也先数闭包调了几次再报。
/// 它靠的前提（登记了不沿它收的这个单元原样交回调用方闭包的结果、不改编出的内核，C1033（两个显存记账单元没有会红的检查））分三条路各一条用例钉。
fn assert_the_compile_closure_is_called_once_and_its_output_handed_back(
    condition: &DeviceConditionOfACard,
    while_compiling: impl Fn(),
) {
    let calls = AtomicUsize::new(0);
    let block_made_in_the_closure = std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
    let handed_back = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        condition.compile_for_the_work_of_this_thread(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            while_compiling();
            let output = OutputOfTheCompileClosure {
                heap_block: Box::new(VALUE_MADE_IN_THE_COMPILE_CLOSURE),
            };
            block_made_in_the_closure.store(
                std::ptr::from_ref::<[u64; 2]>(&*output.heap_block).cast_mut(),
                Ordering::SeqCst,
            );
            output
        })
    }));
    assert_eq!(calls.load(Ordering::SeqCst), 1, "编内核的闭包恰好调一次");
    let handed_back = handed_back.expect("闭包调了一次，编内核那一步却 panic 了");
    assert_eq!(
        *handed_back.heap_block, VALUE_MADE_IN_THE_COMPILE_CLOSURE,
        "交回的值是闭包造的那一份"
    );
    assert!(
        std::ptr::eq(
            &*handed_back.heap_block,
            block_made_in_the_closure.load(Ordering::SeqCst)
        ),
        "交回的是闭包造的那一块，不是复制或重造的"
    );
}

/// 这条线程上没有活（新起的一条线程，上面从没起过活）：编内核只是调闭包，原样交回它的结果。
#[test]
fn the_compile_closure_is_called_once_and_its_output_handed_back_on_a_thread_without_work() {
    let condition = DeviceConditionOfACard::default();
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                assert_the_compile_closure_is_called_once_and_its_output_handed_back(
                    &condition,
                    || {},
                );
            })
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    });
}

/// 这条线程上有活、编内核正常编完：设备上记着「在给这段活编」，闭包照样只调一次、原样交回它的结果。
#[test]
fn the_compile_closure_is_called_once_and_its_output_handed_back_for_the_work_of_this_thread() {
    let condition = DeviceConditionOfACard::default();
    let _work = WorkOnTheCardOfThisThread::begin();
    assert_the_compile_closure_is_called_once_and_its_output_handed_back(&condition, || {});
    assert!(!condition.is_lost(), "正常编完的那一条路上设备没丢");
}

/// 这条线程上有活，编内核的时候另起的线程上记了 Out of Memory、又因显存不够把设备记成丢了（记账那一侧走到设备丢失与显存不够那一支）：
/// 闭包照样只调一次、原样交回它的结果。之后核这一支真走到了：设备记成丢了，那次 Out of Memory 说得成这段活的。
#[test]
fn the_compile_closure_is_called_once_and_its_output_handed_back_while_the_device_is_lost_short_of_gpu_memory(
) {
    let condition = DeviceConditionOfACard::default();
    let work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    assert_the_compile_closure_is_called_once_and_its_output_handed_back(&condition, || {
        std::thread::scope(|scope| {
            scope.spawn(|| {
                condition.record_an_out_of_memory("另起的线程上编内核时显存不够".to_string());
                condition.record_the_device_lost(
                    "编内核时显存不够，设备丢了".to_string(),
                    CauseOfTheDeviceLoss::GpuMemoryShort,
                );
            });
        });
    });
    assert!(condition.is_lost(), "编内核的时候设备记成丢了");
    assert_eq!(
        condition.out_of_memory_explaining_a_failure_of_the_work(
            work.identifier(),
            reports_before,
            DeviceWhenTheWorkReturned::NotLost,
        ),
        Some("另起的线程上编内核时显存不够".to_string()),
        "编内核的时候另起的线程上记的 Out of Memory 算在这段活上（不靠设备丢了那一支也说得成）"
    );
}

/// 记了丢失的设备是丢了的，没记的不是（卡池照它判留着的设备要不要重起）；
/// 一段活交回那一刻照它记设备丢没丢。
#[test]
fn device_recorded_lost_is_lost_and_another_is_not() {
    let condition = DeviceConditionOfACard::default();
    assert!(!condition.is_lost(), "没记丢失");
    assert_eq!(
        condition.device_when_the_work_returns(),
        DeviceWhenTheWorkReturned::NotLost
    );
    condition.record_the_device_lost("Unknown：驱动重置".to_string(), CauseOfTheDeviceLoss::Other);
    assert!(condition.is_lost(), "记了丢失");
    assert_eq!(
        condition.device_when_the_work_returns(),
        DeviceWhenTheWorkReturned::Lost
    );
}

/// 一块的两段在同一个设备上并发，一段因设备丢了（wgpu 说设备丢了，不是它的显存不够）
/// 先失败、先来认；另一段范围里接到的 Out of Memory 要等那一段先收完才记进设备。先失败的那一段等两段都报到再认：认得出设备丢之前报过
/// Out of Memory，不判红。
#[test]
fn piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory() {
    let condition = &DeviceConditionOfACard::default();
    let together = &WorksEndingTogetherOnADevice::new();
    let failed_piece_ending = together.join();
    let caught_piece_ending = together.join();
    let explained = std::thread::scope(|scope| {
        let failed_piece = scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            condition.record_the_device_lost(
                "Unknown：Device is lost".to_string(),
                CauseOfTheDeviceLoss::Other,
            );
            condition.end_the_work_and_explain_its_failure(
                &work,
                reports_before,
                condition.device_when_the_work_returns(),
                None,
                failed_piece_ending,
            )
        });
        scope.spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            let waiting_since = std::time::Instant::now();
            while together.works_not_yet_ended() > 1 {
                assert!(
                    waiting_since.elapsed() < std::time::Duration::from_secs(60),
                    "先失败的那一段一分钟都没报到"
                );
                std::thread::yield_now();
            }
            condition.end_the_work_and_explain_its_failure(
                &work,
                reports_before,
                condition.device_when_the_work_returns(),
                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                caught_piece_ending,
            )
        });
        failed_piece.join().expect("先失败的那一段不 panic")
    });
    assert!(
        explained
            .as_deref()
            .is_some_and(|reason| reason.starts_with("设备丢了")),
        "等另一段记进设备之后再认：说成设备丢之前报过 Out of Memory，{explained:?}"
    );
}

/// wgpu 因显存不够丢设备时给的那一句：`wgpu_hal::DeviceError::OutOfMemory` 说的（wgpu-hal 30.0.1 `src/lib.rs` 里 `DeviceError::OutOfMemory` 的 `#[error("Out of memory")]`），
/// 丢失回调照 `{reason:?}：{message}` 记（`singlefs_verification::gpu_unit_checks`）。
const LOST_FOR_LACK_OF_MEMORY: &str = "Unknown：Out of memory";

/// （攻方甲形，`attack_10zc_endings.rs` 的 `cross-group`）：同一个设备上不在同一组的两段活
/// （同一测试进程里两条用例共用卡池的设备）：X 的范围里接到 Out of Memory、wgpu 因它丢了设备，X 还没收完；Y 因设备丢了先失败、先收完，
/// 等的只是它自己。设备是因显存不够丢的，Y 认成显存不够、不判红；X 收完认它自己那一句。同组对照照旧认得出。
#[test]
fn work_failing_on_a_device_another_group_lost_for_lack_of_memory_is_told_out_of_memory() {
    for same_group in [false, true] {
        let condition = &DeviceConditionOfACard::default();
        let shared = &WorksEndingTogetherOnADevice::new();
        let alone_out_of_memory_work = &WorksEndingTogetherOnADevice::new();
        let alone_bystander_work = &WorksEndingTogetherOnADevice::new();
        let (ending_of_the_out_of_memory_work, ending_of_the_bystander_work) = if same_group {
            (shared.join(), shared.join())
        } else {
            (alone_out_of_memory_work.join(), alone_bystander_work.join())
        };
        let (bystander_began_sender, bystander_began) = std::sync::mpsc::channel::<()>();
        let (bystander_ended_sender, bystander_ended) = std::sync::mpsc::channel::<()>();
        let (out_of_memory_work_told, bystander_told) = std::thread::scope(|scope| {
            let out_of_memory_work_thread = scope.spawn(move || {
                let work = WorkOnTheCardOfThisThread::begin();
                let reports_before = condition.out_of_memory_reports();
                bystander_began
                    .recv_timeout(std::time::Duration::from_secs(60))
                    .expect("Y 一分钟之内开了活");
                condition.record_the_device_lost(
                    LOST_FOR_LACK_OF_MEMORY.to_string(),
                    CauseOfTheDeviceLoss::GpuMemoryShort,
                );
                let device_when_the_work_returned = condition.device_when_the_work_returns();
                if !same_group {
                    // 不在同一组：Y 先收完，X 收得比 Y 晚
                    bystander_ended
                        .recv_timeout(std::time::Duration::from_secs(60))
                        .expect("Y 一分钟之内收完");
                }
                condition.end_the_work_and_explain_its_failure(
                    &work,
                    reports_before,
                    device_when_the_work_returned,
                    Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
                    ending_of_the_out_of_memory_work,
                )
            });
            let bystander_work_thread = scope.spawn(move || {
                let work = WorkOnTheCardOfThisThread::begin();
                let reports_before = condition.out_of_memory_reports();
                bystander_began_sender.send(()).expect("X 在等 Y 开活");
                let waiting_since = std::time::Instant::now();
                while !condition.is_lost() {
                    assert!(
                        waiting_since.elapsed() < std::time::Duration::from_secs(60),
                        "X 一分钟都没丢设备"
                    );
                    std::thread::yield_now();
                }
                let explained = condition.end_the_work_and_explain_its_failure(
                    &work,
                    reports_before,
                    condition.device_when_the_work_returns(),
                    None,
                    ending_of_the_bystander_work,
                );
                // 同组时 X 不等这一句，发不出去也无妨
                let _ = bystander_ended_sender.send(());
                explained
            });
            (
                out_of_memory_work_thread.join().expect("X 不 panic"),
                bystander_work_thread.join().expect("Y 不 panic"),
            )
        });
        assert_eq!(
            out_of_memory_work_told,
            Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
            "同组 {same_group}：X 认它自己那一句"
        );
        assert!(
            bystander_told.is_some(),
            "同组 {same_group}：设备因显存不够丢了，Y 认成显存不够、不判红"
        );
    }
}

/// （攻方乙形 `genuine-panic-widened`、正推 `explain_demo.rs`）：一块两段 A、B 同组。
/// A 的活自己真 panic 了，交回那一刻设备没丢、设备上一条 Out of Memory 都没有；A 报到之后 B 的范围里接到 Out of Memory（设备丢没丢两样都试），
/// B 收完把它记进设备。A 等齐再认：只认算在它自己头上的，说不成显存不够（照样往外抛、判红）；B 认得出它自己那一句。
#[test]
fn genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out()
{
    for later_piece_loses_the_device in [false, true] {
        let condition = &DeviceConditionOfACard::default();
        let together = &WorksEndingTogetherOnADevice::new();
        let ending_of_the_panicking_piece = together.join();
        let ending_of_the_later_piece = together.join();
        let (panicking_piece_told, later_piece_told) = std::thread::scope(|scope| {
            let panicking_piece_thread = scope.spawn(move || {
                let work = WorkOnTheCardOfThisThread::begin();
                let reports_before = condition.out_of_memory_reports();
                // A 的活 panic 了（真毛病）：交回这一刻设备没丢
                let device_when_the_work_returned = condition.device_when_the_work_returns();
                condition.end_the_work_and_explain_its_failure(
                    &work,
                    reports_before,
                    device_when_the_work_returned,
                    None,
                    ending_of_the_panicking_piece,
                )
            });
            let later_piece_thread = scope.spawn(move || {
                let work = WorkOnTheCardOfThisThread::begin();
                let reports_before = condition.out_of_memory_reports();
                let waiting_since = std::time::Instant::now();
                while together.works_not_yet_ended() > 1 {
                    assert!(
                        waiting_since.elapsed() < std::time::Duration::from_secs(60),
                        "A 一分钟都没报到"
                    );
                    std::thread::yield_now();
                }
                if later_piece_loses_the_device {
                    condition.record_the_device_lost(
                        LOST_FOR_LACK_OF_MEMORY.to_string(),
                        CauseOfTheDeviceLoss::GpuMemoryShort,
                    );
                }
                condition.end_the_work_and_explain_its_failure(
                    &work,
                    reports_before,
                    condition.device_when_the_work_returns(),
                    Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                    ending_of_the_later_piece,
                )
            });
            (
                panicking_piece_thread.join().expect("A 不 panic"),
                later_piece_thread.join().expect("B 不 panic"),
            )
        });
        assert_eq!(
                panicking_piece_told, None,
                "B 丢设备 {later_piece_loses_the_device}：A 交回那一刻设备没丢，它的 panic 说不成显存不够"
            );
        assert_eq!(
            later_piece_told,
            Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
            "B 丢设备 {later_piece_loses_the_device}：B 认它自己那一句"
        );
    }
}

/// （正推 `probes/killers_new.rs` M23）：同一段活上记了两次 Out of Memory，认出来的是最后那一次。
#[test]
fn the_last_out_of_memory_recorded_on_a_work_is_the_one_told() {
    let condition = DeviceConditionOfACard::default();
    let work = WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    condition.record_an_out_of_memory("先报的".to_string());
    condition.record_an_out_of_memory("后报的".to_string());
    assert_eq!(
        condition
            .out_of_memory_explaining_a_failure_of_the_work(
                work.identifier(),
                reports_before,
                DeviceWhenTheWorkReturned::NotLost
            )
            .as_deref(),
        Some("后报的")
    );
}
