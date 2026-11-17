//! 攻方腿探针（m4-gpugc-mount-runtime-code-r1，模型文件，不入产品）：拷进冻结副本 `crates/singlefs-background-gpu/tests/` 跑，不要卡、不碰 Vulkan。
//! 只打 `name=` 开头的读数，不断言结论。

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use singlefs_background_gpu::gpu_opener_of_this_process::GpuOpenerOfThisProcess;
use singlefs_core::background_compute::BackgroundComputeSettings;
use singlefs_mount_runtime::OpensTheBackgroundGpu;

const DEADLINE_OF_THE_OPENING: Duration = Duration::from_millis(50);
const TIME_FOR_THE_WORKER_THREAD_TO_END: Duration = Duration::from_millis(300);
const TIME_ALLOWED_FOR_A_READ_OF_THE_OPENER: Duration = Duration::from_secs(1);
const REFUSAL_OF_THE_DOUBLE: &str = "替身开卡一律拒";

/// T5：工作线程过了期限之后才 panic（驱动回来时 panic）：闸由工作线程走到末尾才放开，panic 走不到那一句。
/// 对照臂：同样过了期限之后正常返回。判别量：工作线程都结束之后闸开没开、下一次开卡报什么。
#[test]
fn a_worker_thread_panicking_after_the_deadline_leaves_the_gate_closed() {
    for panics_after_the_deadline in [false, true] {
        let (release_the_opening, opening_is_released) = mpsc::channel::<()>();
        let opening_is_released = Arc::new(Mutex::new(opening_is_released));
        let opener: GpuOpenerOfThisProcess<u32> =
            GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
                DEADLINE_OF_THE_OPENING,
                Arc::new(|| 0),
                Arc::new(move |_instance, _selection, _memory_leaked| {
                    let released = opening_is_released
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .recv()
                        .is_ok();
                    if released && panics_after_the_deadline {
                        panic!("探针：驱动回来时 panic");
                    }
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                }),
            );
        let settings = BackgroundComputeSettings::PRODUCT;
        let first = opener.open_the_background_gpu(&settings).err();
        let sent = release_the_opening.send(()).is_ok();
        std::thread::sleep(TIME_FOR_THE_WORKER_THREAD_TO_END);
        let hung_after_the_worker_ended = opener.is_an_opening_hung();
        let second = opener.open_the_background_gpu(&settings).err();
        println!(
            "name=opener_panic_after_the_deadline panics={panics_after_the_deadline} first={first:?} released={sent} hung_after_the_worker_ended={hung_after_the_worker_ended} second={second:?}"
        );
    }
}

/// T5（判决打中 6 的路径）：建实例那一步挂住（模块文档写「驱动也可能挂在这里」），工作线程拿着实例那把锁。调用线程到点拒之后，修好驱动的路径第一步
/// `forget_the_instance_so_the_next_opening_creates_one` 与只读的 `instances_created` 都要这把锁。判别量：一秒之内回不回得来。
#[test]
fn reading_the_opener_while_the_instance_maker_is_hung() {
    for read_of_the_opener in ["instances_created", "forget_the_instance"] {
        let (release_the_maker, maker_is_released) = mpsc::channel::<()>();
        let maker_is_released = Mutex::new(maker_is_released);
        let opener: Arc<GpuOpenerOfThisProcess<u32>> = Arc::new(
            GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
                DEADLINE_OF_THE_OPENING,
                Arc::new(move || {
                    let _released = maker_is_released
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .recv()
                        .is_ok();
                    0
                }),
                Arc::new(|_instance, _selection, _memory_leaked| {
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                }),
            ),
        );
        let settings = BackgroundComputeSettings::PRODUCT;
        let first = opener.open_the_background_gpu(&settings).err();
        let (answered, answer) = mpsc::channel::<()>();
        let opener_of_the_reader = Arc::clone(&opener);
        let reader = std::thread::spawn(move || {
            if read_of_the_opener == "instances_created" {
                let _count = opener_of_the_reader.instances_created();
            } else {
                opener_of_the_reader.forget_the_instance_so_the_next_opening_creates_one();
            }
            let _ignored = answered.send(());
        });
        let answer_within_a_second = answer.recv_timeout(TIME_ALLOWED_FOR_A_READ_OF_THE_OPENER);
        let returned_within_a_second = !matches!(answer_within_a_second, Err(RecvTimeoutError::Timeout));
        println!(
            "name=opener_read_while_the_maker_is_hung read={read_of_the_opener} first={first:?} returned_within_a_second={returned_within_a_second}"
        );
        let _released = release_the_maker.send(()).is_ok();
        reader.join().expect("探针的线程不 panic");
    }
}
