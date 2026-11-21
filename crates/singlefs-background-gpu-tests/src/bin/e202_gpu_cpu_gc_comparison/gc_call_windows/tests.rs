use super::*;

fn reading(
    process_milliseconds: u64,
    gc_thread_milliseconds: u64,
    foreground_milliseconds: u64,
    main_milliseconds: u64,
) -> CallWindowReading {
    CallWindowReading {
        wall: Duration::from_millis(1000 + process_milliseconds),
        process_cpu: Duration::from_millis(process_milliseconds),
        gc_thread_cpu: Duration::from_millis(gc_thread_milliseconds),
        foreground_thread_cpu: Duration::from_millis(foreground_milliseconds),
        main_thread_cpu: Duration::from_millis(main_milliseconds),
        cpu_by_task: BTreeMap::new(),
    }
}

/// 同 [`reading`]，另带这一端读到的每条线程的 schedstat（线程号，毫秒）。
fn reading_with_the_tasks(
    process_milliseconds: u64,
    gc_thread_milliseconds: u64,
    tasks: &[(u64, u64)],
) -> CallWindowReading {
    CallWindowReading {
        cpu_by_task: tasks
            .iter()
            .map(|(task, milliseconds)| (*task, Duration::from_millis(*milliseconds)))
            .collect(),
        ..reading(process_milliseconds, gc_thread_milliseconds, 0, 0)
    }
}

/// 登记第十二节修订第 1 条：窗口里有线程退出、或线程号让新线程重用了（关窗时那一号的 CPU 比开窗时少）的那一窗作废（第十一节作废第 1 条），
/// 两端之和相减仍不是负的也作废。小池单测（任务 6790）读到过这样一窗：进程口径 170.84191ms、GC 线程口径 214.0609ms。
/// 这里线程 7 开窗时用掉 60 毫秒、窗口里退出：GC 线程（线程 3）走了 40、线程 8 走了 20，两端之和只多 0，进程口径 0 小于 GC 线程口径 40。
#[test]
fn the_window_over_which_any_task_exited_or_its_number_was_reused_is_void() {
    let opening = reading_with_the_tasks(150, 50, &[(3, 50), (7, 60), (8, 40)]);
    let every_task_stays = reading_with_the_tasks(210, 90, &[(3, 90), (7, 60), (8, 60)]);
    let window = call_window_between(opening.clone(), every_task_stays).expect("没有线程退出");
    assert_eq!(window.process_cpu, Duration::from_millis(40 + 20));
    assert_eq!(window.gc_thread_cpu, Duration::from_millis(40));
    let task_seven_exited = reading_with_the_tasks(150, 90, &[(3, 90), (8, 60)]);
    assert_eq!(
        call_window_between(opening.clone(), task_seven_exited),
        None
    );
    let task_seven_reused = reading_with_the_tasks(155, 90, &[(3, 90), (7, 5), (8, 60)]);
    assert_eq!(call_window_between(opening, task_seven_reused), None);
}

#[test]
fn process_cpu_of_the_window_subtracts_the_foreground_and_main_threads_but_keeps_the_driver_threads(
) {
    let window = call_window_between(reading(100, 50, 10, 5), reading(200, 90, 11, 7))
        .expect("增量都不是负的");
    assert_eq!(window.process_cpu, Duration::from_millis(100 - 1 - 2));
    assert_eq!(window.gc_thread_cpu, Duration::from_millis(40));
    assert_eq!(window.wall, Duration::from_millis(100));
}

#[test]
fn the_window_whose_clock_goes_backwards_is_not_counted() {
    assert_eq!(
        call_window_between(reading(200, 50, 10, 5), reading(100, 90, 11, 7)),
        None
    );
}

/// 假的窗口钟：进程 CPU 由调用与多判两段各自往前拨。
struct ClocksAdvancedByTheWork {
    process_cpu: Cell<Duration>,
}

impl CallWindowClocks for ClocksAdvancedByTheWork {
    fn read(&self, _end: EndOfTheWindow) -> Result<CallWindowReading, ClockReadFailed> {
        Ok(CallWindowReading {
            wall: self.process_cpu.get(),
            process_cpu: self.process_cpu.get(),
            gc_thread_cpu: self.process_cpu.get(),
            foreground_thread_cpu: Duration::ZERO,
            main_thread_cpu: Duration::ZERO,
            cpu_by_task: BTreeMap::new(),
        })
    }
}

#[test]
fn the_cpu_rejudging_inside_the_window_counts_toward_the_process_cpu_of_the_call() {
    let clocks = ClocksAdvancedByTheWork {
        process_cpu: Cell::new(Duration::ZERO),
    };
    let mut calls = 0_u32;
    let measured = measure_one_gc_call(
        &clocks,
        &mut calls,
        |calls| {
            *calls += 1;
            clocks
                .process_cpu
                .set(clocks.process_cpu.get() + Duration::from_millis(3));
        },
        |_calls, ()| {
            clocks
                .process_cpu
                .set(clocks.process_cpu.get() + Duration::from_millis(7));
            (Some(Duration::from_millis(7)), RejudgeTally::default())
        },
    );
    assert_eq!(calls, 1);
    let window = measured.window.expect("钟都读得到");
    assert_eq!(window.process_cpu, Duration::from_millis(3 + 7));
    assert_eq!(measured.inner_rejudge_cpu, Duration::from_millis(7));
}

#[test]
fn an_unreadable_inner_timing_voids_the_window() {
    let clocks = ClocksAdvancedByTheWork {
        process_cpu: Cell::new(Duration::ZERO),
    };
    let measured = measure_one_gc_call(
        &clocks,
        &mut (),
        |_state| (),
        |_state, ()| (None, RejudgeTally::default()),
    );
    assert!(measured.window.is_none());
}

/// 第九节变异「交替跑的假钟不前进」在这里红。
#[test]
fn the_fake_clock_of_the_lockstep_run_advances_one_millisecond_per_call() {
    let caller = CallerOfTheLockstepRun::starting_at_zero();
    caller.advance_before_the_next_call();
    caller.advance_before_the_next_call();
    assert_eq!(
        caller.clock_reading(),
        CallerMonotonicClockReading(Duration::from_millis(2))
    );
    assert_eq!(
        caller.answer_before_the_next_batch(),
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    );
}

/// 包含自检的窗口两端（登记第五节「计时与计数怎么做」）：挂钟的窗口罩住 GC 线程 CPU 钟的窗口，一次什么都不做的调用挂钟也不短于 CPU 钟。
/// 读序反过来（CPU 钟在外、挂钟在里）时，读 CPU 钟那两次系统调用的开销只落在 CPU 钟里，这样的调用几乎每一次都判不过。
#[test]
fn the_wall_of_a_call_window_encloses_the_gc_thread_cpu_of_an_empty_call() {
    const EMPTY_CALLS: usize = 200;
    let both_stand_ins_wait = std::sync::Barrier::new(3);
    let (to_the_test, task_identifiers) = mpsc::channel::<u64>();
    std::thread::scope(|scope| {
        for _stand_in in ["前台线程", "主线程"] {
            let to_the_test = to_the_test.clone();
            let both_stand_ins_wait = &both_stand_ins_wait;
            scope.spawn(move || {
                to_the_test
                    .send(task_identifier_of_this_thread().expect("读得到线程号"))
                    .expect("测试线程在等");
                both_stand_ins_wait.wait();
            });
        }
        drop(to_the_test);
        let clocks = ClocksOfTheGcThread {
            started: StartOnTheMonotonicRawClock::now(),
            gc_thread_task: task_identifier_of_this_thread().ok(),
            foreground_thread_task: task_identifiers.recv().expect("前台线程替身的线程号"),
            main_thread_task: task_identifiers.recv().expect("主线程替身的线程号"),
        };
        let windows: Vec<CallWindow> = (0..EMPTY_CALLS)
            .filter_map(|_call| {
                measure_one_gc_call(
                    &clocks,
                    &mut (),
                    |_state: &mut ()| {},
                    |_state: &mut (), _outcome: &()| {
                        (Some(Duration::ZERO), RejudgeTally::default())
                    },
                )
                .window
            })
            .collect();
        both_stand_ins_wait.wait();
        assert!(windows.len() * 2 >= EMPTY_CALLS, "{}", windows.len());
        let wall_shorter_than_the_cpu: Vec<&CallWindow> = windows
            .iter()
            .filter(|window| window.wall < window.gc_thread_cpu)
            .collect();
        assert!(
            wall_shorter_than_the_cpu.is_empty(),
            "{wall_shorter_than_the_cpu:?}"
        );
    });
}

/// 包含自检「每次调用挂钟 ≥ GC 线程 CPU」在长调用上（登记第十二节修订第 10 条；任务 6790 挂钟读 `CLOCK_MONOTONIC` 时 22 窗里 21 窗
/// GC 线程 CPU 比挂钟多 0.6–22 ppm）：一次调用自旋 100 ms 的线程 CPU，两端照 GC 调用的窗口读（挂钟在外、CPU 钟在里），每次挂钟都不短于 GC 线程 CPU。
/// 两端的读数直接相减，不经 `call_window_between`：同一进程里别的用例的线程在窗口里退出时那一窗作废，这里比的只是两样钟。
/// 挂钟读 `CLOCK_MONOTONIC` 时，本机的 NTP 正把钟调慢十几 ppm 的那一阵，一次调用少走一两微秒，比读钟开销留下的余量大，这一条判不过；
/// NTP 没在调频时两种钟都过：单测改不了本机的调频，所以这一条分得出两种钟只在调频的那一阵，不是每次都分得出。
#[test]
fn the_wall_of_a_call_window_encloses_the_gc_thread_cpu_of_a_long_spinning_call() {
    const SPINNING_CALLS: usize = 20;
    const GC_THREAD_CPU_OF_ONE_CALL: Duration = Duration::from_millis(100);
    let both_stand_ins_wait = std::sync::Barrier::new(3);
    let (to_the_test, task_identifiers) = mpsc::channel::<u64>();
    std::thread::scope(|scope| {
        for _stand_in in ["前台线程", "主线程"] {
            let to_the_test = to_the_test.clone();
            let both_stand_ins_wait = &both_stand_ins_wait;
            scope.spawn(move || {
                to_the_test
                    .send(task_identifier_of_this_thread().expect("读得到线程号"))
                    .expect("测试线程在等");
                both_stand_ins_wait.wait();
            });
        }
        drop(to_the_test);
        let clocks = ClocksOfTheGcThread {
            started: StartOnTheMonotonicRawClock::now(),
            gc_thread_task: task_identifier_of_this_thread().ok(),
            foreground_thread_task: task_identifiers.recv().expect("前台线程替身的线程号"),
            main_thread_task: task_identifiers.recv().expect("主线程替身的线程号"),
        };
        let mut calls_with_the_wall_behind_the_gc_thread_cpu: Vec<(Duration, Duration)> =
            Vec::new();
        for _spinning_call in 0..SPINNING_CALLS {
            let opening = clocks
                .read(EndOfTheWindow::Opening)
                .expect("两个替身都活着，读得到窗口开端的钟");
            let gc_thread_cpu_at_the_start_of_the_spin = host_cpu_time_of_this_thread();
            while host_cpu_time_of_this_thread()
                .saturating_sub(gc_thread_cpu_at_the_start_of_the_spin)
                < GC_THREAD_CPU_OF_ONE_CALL
            {
                std::hint::spin_loop();
            }
            let closing = clocks
                .read(EndOfTheWindow::Closing)
                .expect("两个替身都活着，读得到窗口末端的钟");
            let wall = closing
                .wall
                .checked_sub(opening.wall)
                .expect("CLOCK_MONOTONIC_RAW 单调不减");
            let gc_thread_cpu = closing
                .gc_thread_cpu
                .checked_sub(opening.gc_thread_cpu)
                .expect("线程 CPU 钟单调不减");
            if wall < gc_thread_cpu {
                calls_with_the_wall_behind_the_gc_thread_cpu.push((wall, gc_thread_cpu));
            }
        }
        both_stand_ins_wait.wait();
        assert!(
            calls_with_the_wall_behind_the_gc_thread_cpu.is_empty(),
            "这几次调用（挂钟，GC 线程 CPU）里挂钟比 GC 线程 CPU 短：{calls_with_the_wall_behind_the_gc_thread_cpu:?}"
        );
    });
}
