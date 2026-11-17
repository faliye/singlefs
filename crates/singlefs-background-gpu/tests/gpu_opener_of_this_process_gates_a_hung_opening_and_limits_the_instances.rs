//! 开卡器（`gpu_opener_of_this_process`）不要卡的那几格：实例与开卡都换上替身，不碰 Vulkan。工作线程挂住时挂载到点拒、不再多起一条枚举线程；回来之后闸放开；
//! 工作线程过了期限才 panic 时闸照样放开；建实例挂住时读计数、丢实例都不被挂住。实例进程一份、反复用，新建有上限。
//! 期限按卡数算：枚举挂住时到枚举期限拒；枚举报了几张卡之后期限延到枚举期限加卡数乘每张卡的期限（期限用用例自己定的短时长）。
//! 真开卡在测试包 `crates/singlefs-background-gpu-tests/tests/` 的 `mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs` 与 `mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs`。

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use singlefs_background_gpu::background_gpu::{
    OpeningsNotReturnedInThisProcess, VulkanInstanceOfThisProcess,
};
use singlefs_background_gpu::gpu_opener_of_this_process::{
    GpuOpenerOfThisProcess, OpeningDeadlinesOfTheOpener, VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST,
};
use singlefs_core::background_compute::BackgroundComputeSettings;
use singlefs_mount_runtime::{BackgroundGpuNotOpened, OpensTheBackgroundGpu};

/// 到点的期限：替身的工作线程挂着，这个长度只决定用例等多久。
const DEADLINE_OF_THE_HUNG_OPENING: Duration = Duration::from_millis(50);
/// 工作线程不挂的用例给的期限：健康的替身远用不了这么久。
const DEADLINE_NEVER_REACHED: Duration = Duration::from_secs(600);

/// 枚举期限与每张卡的期限都取 `deadline`：工作线程不报卡数的用例，到点拒在枚举期限上。
fn deadlines_both_at(deadline: Duration) -> OpeningDeadlinesOfTheOpener {
    OpeningDeadlinesOfTheOpener {
        enumeration: deadline,
        each_gpu: deadline,
    }
}
/// 工作线程结束给的时间：放开之后立刻结束，这个长度只决定闸没放开时用例红得多快。
const TIME_FOR_THE_WORKER_THREAD_TO_END: Duration = Duration::from_secs(60);
/// 建实例挂着时读开卡器给的时间：不挂住时立刻回来。
const TIME_ALLOWED_FOR_A_READ_OF_THE_OPENER: Duration = Duration::from_secs(30);
/// 替身开卡的结局：一律拒（这几格不要把手）。
const REFUSAL_OF_THE_DOUBLE: &str = "替身开卡一律拒";

fn opener_refusing_every_opening(
    instances_made: Arc<AtomicU32>,
    openings: Arc<AtomicU32>,
) -> GpuOpenerOfThisProcess<u32> {
    GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
        deadlines_both_at(DEADLINE_NEVER_REACHED),
        Arc::new(move || instances_made.fetch_add(1, Ordering::SeqCst)),
        Arc::new(
            move |_instance,
                  _selection,
                  _memory_leaked,
                  _openings_not_returned,
                  _report_the_gpus_to_open| {
                openings.fetch_add(1, Ordering::SeqCst);
                Err(REFUSAL_OF_THE_DOUBLE.to_string())
            },
        ),
    )
}

fn refusal_of_the_double() -> BackgroundGpuNotOpened {
    BackgroundGpuNotOpened::RefusedByTheOpening {
        description: REFUSAL_OF_THE_DOUBLE.to_string(),
    }
}

#[test]
fn hung_opening_times_out_and_gates_new_openings_until_it_returns() {
    let (release_the_opening, opening_is_released) = mpsc::channel::<()>();
    let opening_is_released = Arc::new(Mutex::new(opening_is_released));
    let openings = Arc::new(AtomicU32::new(0));
    let openings_of_the_double = Arc::clone(&openings);
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            deadlines_both_at(DEADLINE_OF_THE_HUNG_OPENING),
            Arc::new(|| 0),
            Arc::new(
                move |_instance,
                      _selection,
                      _memory_leaked,
                      _openings_not_returned,
                      _report_the_gpus_to_open| {
                    openings_of_the_double.fetch_add(1, Ordering::SeqCst);
                    let released = opening_is_released.lock().expect("只有替身拿这把锁").recv();
                    assert!(released.is_ok(), "用例放开之前不丢那一头");
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    let settings = BackgroundComputeSettings::PRODUCT;
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(BackgroundGpuNotOpened::TimedOut {
            deadline: DEADLINE_OF_THE_HUNG_OPENING
        }),
        "工作线程挂着：到点拒，调用线程不再等"
    );
    assert!(opener.is_an_opening_hung(), "工作线程还没回来");
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung),
        "上一次没回来：闸住新开卡"
    );
    assert_eq!(
        openings.load(Ordering::SeqCst),
        1,
        "闸住时不多起一条枚举线程"
    );
    release_the_opening.send(()).expect("替身还在等");
    let given_up_at = Instant::now() + DEADLINE_NEVER_REACHED;
    while opener.is_an_opening_hung() {
        assert!(Instant::now() < given_up_at, "放开之后工作线程回得来");
        std::thread::yield_now();
    }
    release_the_opening.send(()).expect("替身的接收端还在");
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(refusal_of_the_double()),
        "回来之后闸放开：下一次照常开卡"
    );
}

/// 工作线程过了期限之后才 panic（驱动回来时 panic）：走不到函数末尾，闸照样放开，下一次照常开卡。
#[test]
fn worker_thread_panicking_after_the_deadline_opens_the_gate() {
    let (release_the_opening, opening_is_released) = mpsc::channel::<()>();
    let opening_is_released = Arc::new(Mutex::new(opening_is_released));
    let openings = Arc::new(AtomicU32::new(0));
    let openings_of_the_double = Arc::clone(&openings);
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            deadlines_both_at(DEADLINE_OF_THE_HUNG_OPENING),
            Arc::new(|| 0),
            Arc::new(
                move |_instance,
                      _selection,
                      _memory_leaked,
                      _openings_not_returned,
                      _report_the_gpus_to_open| {
                    let opening = openings_of_the_double.fetch_add(1, Ordering::SeqCst);
                    if opening == 0 {
                        let released = opening_is_released
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .recv();
                        assert!(released.is_err(), "用例摆的：驱动回来时 panic");
                    }
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    let settings = BackgroundComputeSettings::PRODUCT;
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(BackgroundGpuNotOpened::TimedOut {
            deadline: DEADLINE_OF_THE_HUNG_OPENING
        }),
        "工作线程挂着：到点拒"
    );
    assert!(opener.is_an_opening_hung(), "工作线程还没回来");
    release_the_opening.send(()).expect("替身还在等");
    let given_up_at = Instant::now() + TIME_FOR_THE_WORKER_THREAD_TO_END;
    while opener.is_an_opening_hung() {
        assert!(
            Instant::now() < given_up_at,
            "工作线程 panic 了：闸要放开，不然这个进程再也开不了卡"
        );
        std::thread::yield_now();
    }
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(refusal_of_the_double()),
        "下一次照常开卡"
    );
}

/// 建实例挂住的时候读 `instances_created`、调 `forget_the_instance_so_the_next_opening_creates_one`（修好驱动的路径第一步）都回得来；
/// 挂着的那一个回来之后不留给下一次开卡：它是丢之前的驱动清单建的。
#[test]
fn reads_of_the_opener_return_while_the_instance_maker_is_hung_and_a_forgotten_instance_is_not_kept(
) {
    let (release_the_maker, maker_is_released) = mpsc::channel::<()>();
    let maker_is_released = Mutex::new(maker_is_released);
    let instances_made = Arc::new(AtomicU32::new(0));
    let instances_made_of_the_double = Arc::clone(&instances_made);
    let opener: Arc<GpuOpenerOfThisProcess<u32>> = Arc::new(
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            deadlines_both_at(DEADLINE_OF_THE_HUNG_OPENING),
            Arc::new(move || {
                let instance = instances_made_of_the_double.fetch_add(1, Ordering::SeqCst);
                if instance == 0 {
                    let released = maker_is_released.lock().expect("只有替身拿这把锁").recv();
                    assert!(released.is_ok(), "用例放开之前不丢那一头");
                }
                instance
            }),
            Arc::new(
                |_instance,
                 _selection,
                 _memory_leaked,
                 _openings_not_returned,
                 _report_the_gpus_to_open| {
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        ),
    );
    let settings = BackgroundComputeSettings::PRODUCT;
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(BackgroundGpuNotOpened::TimedOut {
            deadline: DEADLINE_OF_THE_HUNG_OPENING
        }),
        "建实例挂着：到点拒"
    );
    // 工作线程真进了建实例才读：额度在建之前记上。
    let given_up_at = Instant::now() + TIME_FOR_THE_WORKER_THREAD_TO_END;
    while instances_made.load(Ordering::SeqCst) == 0 {
        assert!(Instant::now() < given_up_at, "工作线程走到建实例");
        std::thread::yield_now();
    }
    // 读放到另一条线程上：挂住时用例照样走到放开替身那一句，红在下面的断言上，不是挂死。
    let (answered, answer) = mpsc::channel::<u32>();
    let opener_of_the_reader = Arc::clone(&opener);
    let reader = std::thread::spawn(move || {
        let instances_created = opener_of_the_reader.instances_created();
        opener_of_the_reader.forget_the_instance_so_the_next_opening_creates_one();
        answered.send(instances_created).expect("用例还在等");
    });
    let answer_while_the_maker_is_hung = answer.recv_timeout(TIME_ALLOWED_FOR_A_READ_OF_THE_OPENER);
    release_the_maker.send(()).expect("替身还在等");
    reader.join().expect("读的线程不 panic");
    assert_eq!(
        answer_while_the_maker_is_hung,
        Ok(1),
        "建实例挂着：读计数、丢实例都回得来，额度已经记上"
    );
    let the_release_given_up_at = Instant::now() + DEADLINE_NEVER_REACHED;
    while opener.is_an_opening_hung() {
        assert!(
            Instant::now() < the_release_given_up_at,
            "放开之后工作线程回得来"
        );
        std::thread::yield_now();
    }
    assert_eq!(
        opener.open_the_background_gpu(&settings).err(),
        Some(refusal_of_the_double())
    );
    assert_eq!(
        instances_made.load(Ordering::SeqCst),
        2,
        "挂着的时候丢过实例：下一次开卡新建一个，不用丢之前建的那一个"
    );
    assert_eq!(opener.instances_created(), 2);
}

/// 产品的构造：实例第一次要开卡才建，建好开卡器不碰 Vulkan（不要卡）；真开卡走这一条构造的在
/// `crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs`。
#[test]
fn the_product_opener_creates_no_instance_until_the_first_opening() {
    let opener: GpuOpenerOfThisProcess<VulkanInstanceOfThisProcess> =
        GpuOpenerOfThisProcess::of_this_process();
    assert_eq!(opener.instances_created(), 0, "还没开过卡：一个实例都没建");
    assert!(!opener.is_an_opening_hung(), "还没开过卡：闸开着");
    opener.forget_the_instance_so_the_next_opening_creates_one();
    assert_eq!(opener.instances_created(), 0, "丢实例不算建");
}

/// 产品开卡器的期限：枚举期限 5 分钟；一张卡开不成要隔 1 分钟重开、至多重开 3 次，每次至多 5 分钟，所以一张卡的期限是 4 × 5 + 3 = 23 分钟
/// （期限小于它时，挂载那一刻重开到第几次就被开卡器的期限截住、报成超时，不是「重开 3 次都不成才拒绝挂载」）。
#[test]
fn the_product_opener_waits_for_each_gpu_through_four_openings_and_three_intervals() {
    assert_eq!(
        OpeningDeadlinesOfTheOpener::OF_THE_PRODUCT,
        OpeningDeadlinesOfTheOpener {
            enumeration: Duration::from_secs(5 * 60),
            each_gpu: Duration::from_secs(23 * 60),
        }
    );
}

#[test]
fn the_instance_is_created_once_for_the_process_and_reused_by_every_opening() {
    let instances_made = Arc::new(AtomicU32::new(0));
    let openings = Arc::new(AtomicU32::new(0));
    let opener = opener_refusing_every_opening(Arc::clone(&instances_made), Arc::clone(&openings));
    for _opening in 0..3 {
        assert_eq!(
            opener
                .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
                .err(),
            Some(refusal_of_the_double())
        );
    }
    assert_eq!(openings.load(Ordering::SeqCst), 3);
    assert_eq!(instances_made.load(Ordering::SeqCst), 1, "只建一个实例");
    assert_eq!(opener.instances_created(), 1);
}

#[test]
fn new_instances_stop_at_the_limit_of_the_process_even_when_the_earlier_ones_are_forgotten() {
    let instances_made = Arc::new(AtomicU32::new(0));
    let openings = Arc::new(AtomicU32::new(0));
    let opener = opener_refusing_every_opening(Arc::clone(&instances_made), Arc::clone(&openings));
    for _rebuild in 0..VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST {
        assert_eq!(
            opener
                .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
                .err(),
            Some(refusal_of_the_double())
        );
        opener.forget_the_instance_so_the_next_opening_creates_one();
    }
    assert_eq!(
        opener
            .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
            .err(),
        Some(BackgroundGpuNotOpened::InstanceLimitOfThisProcessReached {
            instances_created: VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST
        }),
        "丢掉旧实例不还额度：到上限就拒"
    );
    assert_eq!(
        instances_made.load(Ordering::SeqCst),
        VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST
    );
    assert_eq!(
        openings.load(Ordering::SeqCst),
        VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST,
        "到上限那一次连开卡都没走"
    );
}

/// 建实例 panic 了没占住实例：额度退回，开到上限之外那一次仍不报上限（挂住占额度的那一格在 `reads_of_the_opener_return_while_the_instance_maker_is_hung_and_a_forgotten_instance_is_not_kept`）。
#[test]
fn an_instance_maker_panicking_gives_the_limit_back() {
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            deadlines_both_at(DEADLINE_NEVER_REACHED),
            Arc::new(|| -> u32 { panic!("用例摆的：建实例 panic") }),
            Arc::new(
                |_instance,
                 _selection,
                 _memory_leaked,
                 _openings_not_returned,
                 _report_the_gpus_to_open| {
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    for opening in 1..=VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST + 1 {
        let outcome = opener
            .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
            .err();
        assert!(
            matches!(
                outcome,
                Some(BackgroundGpuNotOpened::RefusedByTheOpening { .. })
            ),
            "第 {opening} 次开卡：报工作线程 panic，不报额度：{outcome:?}"
        );
        assert_eq!(
            opener.instances_created(),
            0,
            "第 {opening} 次开卡：panic 的那一个额度退回了"
        );
    }
}

/// 按卡数算期限的那几格用的期限：枚举 1 秒、每张卡 2 秒（用例自己定的短时长，产品的都是单卡上限五分钟）。
const DEADLINES_COUNTED_BY_THE_GPUS: OpeningDeadlinesOfTheOpener = OpeningDeadlinesOfTheOpener {
    enumeration: Duration::from_secs(1),
    each_gpu: Duration::from_secs(2),
};
/// 替身枚举完报回要开的卡数：比按三张卡算的多一张。
const GPUS_REPORTED_BY_THE_ENUMERATION: usize = 4;

/// 等挂着的替身工作线程回来、闸放开（用例收尾，不把挂着的线程留给别的用例）。
fn wait_until_the_gate_opens(opener: &GpuOpenerOfThisProcess<u32>) {
    let given_up_at = Instant::now() + TIME_FOR_THE_WORKER_THREAD_TO_END;
    while opener.is_an_opening_hung() {
        assert!(Instant::now() < given_up_at, "放开之后工作线程回得来");
        std::thread::yield_now();
    }
}

/// 枚举报了 4 张卡之后挂住：到枚举期限加 4 张卡的期限（1 + 4 × 2 = 9 秒）才拒，拒绝带的是这个期限，从开卡起等的不比它短。
#[test]
fn an_opening_hung_after_the_enumeration_reported_four_gpus_times_out_at_the_enumeration_plus_four_gpu_deadlines(
) {
    let (release_the_opening, opening_is_released) = mpsc::channel::<()>();
    let opening_is_released = Arc::new(Mutex::new(opening_is_released));
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            DEADLINES_COUNTED_BY_THE_GPUS,
            Arc::new(|| 0),
            Arc::new(
                move |_instance,
                      _selection,
                      _memory_leaked,
                      _openings_not_returned,
                      report_the_gpus_to_open| {
                    report_the_gpus_to_open(GPUS_REPORTED_BY_THE_ENUMERATION);
                    let released = opening_is_released.lock().expect("只有替身拿这把锁").recv();
                    assert!(released.is_ok(), "用例放开之前不丢那一头");
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    let opening_started_at = Instant::now();
    let outcome = opener
        .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
        .err();
    let waited = opening_started_at.elapsed();
    release_the_opening.send(()).expect("替身还在等");
    wait_until_the_gate_opens(&opener);
    let deadline_after_four_gpus = Duration::from_secs(1 + 4 * 2);
    assert_eq!(
        outcome,
        Some(BackgroundGpuNotOpened::TimedOut {
            deadline: deadline_after_four_gpus
        }),
        "枚举报了 4 张卡：期限延到枚举期限加 4 张卡的期限"
    );
    assert!(
        waited >= deadline_after_four_gpus,
        "到点之前不拒：等了 {waited:?}"
    );
}

/// 枚举报了 4 张卡、在枚举期限加 3 张卡的期限（7 秒）之后、加 4 张卡的期限（9 秒）之前回来：照常交回替身的结局，不在按三张卡算的期限上拒。
#[test]
fn an_opening_that_reported_four_gpus_and_returns_past_three_gpu_deadlines_is_not_refused() {
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            DEADLINES_COUNTED_BY_THE_GPUS,
            Arc::new(|| 0),
            Arc::new(
                |_instance,
                 _selection,
                 _memory_leaked,
                 _openings_not_returned,
                 report_the_gpus_to_open| {
                    let reported_at = Instant::now();
                    report_the_gpus_to_open(GPUS_REPORTED_BY_THE_ENUMERATION);
                    std::thread::sleep(
                        Duration::from_secs(8).saturating_sub(reported_at.elapsed()),
                    );
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    assert_eq!(
        opener
            .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
            .err(),
        Some(refusal_of_the_double()),
        "8 秒回来：按 4 张卡算的期限 9 秒之内，交回替身的结局"
    );
}

/// 枚举本身挂住（一张卡都没报）：到枚举期限（1 秒）就拒，不等任何一张卡的期限。
#[test]
fn an_opening_hung_in_the_enumeration_times_out_at_the_enumeration_deadline() {
    let (release_the_opening, opening_is_released) = mpsc::channel::<()>();
    let opening_is_released = Arc::new(Mutex::new(opening_is_released));
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            DEADLINES_COUNTED_BY_THE_GPUS,
            Arc::new(|| 0),
            Arc::new(
                move |_instance,
                      _selection,
                      _memory_leaked,
                      _openings_not_returned,
                      _report_the_gpus_to_open| {
                    let released = opening_is_released.lock().expect("只有替身拿这把锁").recv();
                    assert!(released.is_ok(), "用例放开之前不丢那一头");
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    let opening_started_at = Instant::now();
    let outcome = opener
        .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
        .err();
    let waited = opening_started_at.elapsed();
    release_the_opening.send(()).expect("替身还在等");
    wait_until_the_gate_opens(&opener);
    assert_eq!(
        outcome,
        Some(BackgroundGpuNotOpened::TimedOut {
            deadline: Duration::from_secs(1)
        }),
        "枚举挂住：拒在枚举期限上"
    );
    assert!(
        waited >= Duration::from_secs(1),
        "到枚举期限之前不拒：等了 {waited:?}"
    );
}

/// 一张卡上前一次开卡没回来就不再起下一次，那一张表管到整个进程：开卡器交给每一次开卡的是它持有的同一张（前一次挂载里挂住的那张卡，
/// 下一次挂载开它同样不再起）。
#[test]
fn every_opening_is_handed_the_table_of_the_openings_not_returned_held_by_the_opener() {
    let tables_handed_over: Arc<Mutex<Vec<OpeningsNotReturnedInThisProcess>>> =
        Arc::new(Mutex::new(Vec::new()));
    let tables_noted_by_the_double = Arc::clone(&tables_handed_over);
    let opener: GpuOpenerOfThisProcess<u32> =
        GpuOpenerOfThisProcess::with_the_instance_maker_and_the_opening(
            deadlines_both_at(DEADLINE_NEVER_REACHED),
            Arc::new(|| 0),
            Arc::new(
                move |_instance,
                      _selection,
                      _memory_leaked,
                      openings_not_returned,
                      _report_the_gpus_to_open| {
                    tables_noted_by_the_double
                        .lock()
                        .expect("只有替身拿这把锁")
                        .push(openings_not_returned.clone());
                    Err(REFUSAL_OF_THE_DOUBLE.to_string())
                },
            ),
        );
    for _mount in 0..2 {
        assert_eq!(
            opener
                .open_the_background_gpu(&BackgroundComputeSettings::PRODUCT)
                .err(),
            Some(refusal_of_the_double())
        );
    }
    let held_by_the_opener = opener.openings_not_returned_in_this_process();
    let tables = tables_handed_over.lock().expect("替身已结束");
    assert_eq!(tables.len(), 2, "两次挂载各开一次卡");
    assert!(
        tables
            .iter()
            .all(|table| table.is_the_same_table_as(&held_by_the_opener)),
        "两次开卡拿到的都是开卡器持有的那一张"
    );
}
