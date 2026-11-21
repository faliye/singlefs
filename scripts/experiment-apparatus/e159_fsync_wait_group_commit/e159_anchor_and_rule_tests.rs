use e7_index_bench::{u64_from_nanoseconds, u64_from_u128, usize_from_u64};
use super::*;

/// B4：甲、F1、P=1、k=1（锚点格），带根槽那块盘。与第三节 `crates/` 测试表逐项比
/// （跑前登记停机条款 S1 的一部分）：写 172544 字节、写调用 11 次。
/// （`jia_outcome_single_device_bytes` 现在是模块顶层的共用函数，`print_anchors` 也用它。）
#[test]
fn jia_single_fsync_batch_matches_crates_test_table() {
    let shape = main_shape(1, Family::OneDataUnitPerFile);
    let outcome = solve_jia_batch_publish(shape, 1, true);
    assert_eq!(jia_outcome_single_device_bytes(&outcome), 172_544, "B4：单盘总字节要等于 crates 测试表逐项加总");
    // `outcome.write_calls` 里根槽只算了 1 次（`+ 1`，不是 `+ DEVICE_COUNT`），两盘合计是奇数，
    // 直接除以 2 会截断丢 1；单盘写调用数照 `BatchPlan::total_write_calls` 同样的口径重算。
    let mut random_state = 99u64;
    let plan = plan_jia_batch(shape, 1, true, &mut random_state);
    assert_eq!(plan.total_write_calls(), 11, "单盘写调用应是 8 单元 + 1 记录 + 根槽 + 系统配置槽 = 11 次");
}

/// B1：wal_full-K10、F1、P=10⁴、k=1：单盘 118784 字节、6 次写调用。
#[test]
fn write_ahead_log_full_single_fsync_batch_anchor() {
    let shape = main_shape(10_000, Family::OneDataUnitPerFile);
    let outcome = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 1, true);
    assert_eq!(outcome.total_bytes() / DEVICE_COUNT, 118_784);
    assert_eq!(outcome.write_calls / DEVICE_COUNT, 6);
}

/// B2：yi-M-K10、F1、P=10⁴、k=1：单盘 86016 字节、4 次写调用。
#[test]
fn write_ahead_log_leaf_single_fsync_batch_anchor() {
    let shape = main_shape(10_000, Family::OneDataUnitPerFile);
    let outcome = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 1, true);
    assert_eq!(outcome.total_bytes() / DEVICE_COUNT, 86_016);
    assert_eq!(outcome.write_calls / DEVICE_COUNT, 4);
}

/// B3：F8A、P=10⁴（主几何），WAL 两臂；b=1 与 b=16 的批，单盘写字节期望值
/// （跑前登记第十三节 13.5 python 脚本原样输出：365345.862 / 4313361.603 / 316188.444 / 4264111.306）。
#[test]
fn main_geometry_write_ahead_log_batch_expected_bytes() {
    let shape = main_shape(10_000, Family::EightAdjacentDataUnitsPerFsync);
    let write_ahead_log_full_outcome_batch_size_one = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 1, true);
    let write_ahead_log_full_outcome_batch_size_sixteen = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 16, true);
    let write_ahead_log_leaf_outcome_batch_size_one = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 1, true);
    let write_ahead_log_leaf_outcome_batch_size_sixteen = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 16, true);
    // 单盘容差放宽到 2.0：模型内部按 (两盘合计的整数字节).round() 逐字段取整，
    // 8 个字段各自的取整误差累加、再除以 2，理论上界正是 8 × 0.5 / 2 = 2.0。
    let close = |actual: f64, expected: f64| (actual - expected).abs() < 2.0;
    assert!(close(write_ahead_log_full_outcome_batch_size_one.total_bytes() as f64 / DEVICE_COUNT as f64, 365_345.862));
    assert!(close(write_ahead_log_full_outcome_batch_size_sixteen.total_bytes() as f64 / DEVICE_COUNT as f64, 4_313_361.603));
    assert!(close(write_ahead_log_leaf_outcome_batch_size_one.total_bytes() as f64 / DEVICE_COUNT as f64, 316_188.444));
    assert!(close(write_ahead_log_leaf_outcome_batch_size_sixteen.total_bytes() as f64 / DEVICE_COUNT as f64, 4_264_111.306));
}

/// B9：来宾 `logical_block_size` 由跑真设备探针核（跑前登记第十一节 S3），这里只钉主几何的
/// `physical_block_size_bytes` 常量本身取的是 512（根槽宽度依赖这个值）。
#[test]
fn main_geometry_primary_physical_block_size_is_512() {
    assert_eq!(Geometry::main().physical_block_size_bytes, 512);
}

/// M7 的正确性锚点（第九节）：随机取整长期不偏，期望值的小数部分就是「多写一个」的频率。
#[test]
fn round_stochastic_is_unbiased_over_many_draws() {
    let expected = 3.0489; // 与跑前登记第十三节 13.5 的 extent 期望个数同量级
    let mut state = 42u64;
    let draws = 20_000u64;
    let total: u64 = (0..draws).map(|_| round_stochastic(expected, &mut state)).sum();
    let observed_mean = total as f64 / draws as f64;
    assert!((observed_mean - expected).abs() < 0.02, "20000 次取整的均值 {observed_mean} 应逼近期望 {expected}");
}

#[test]
fn round_stochastic_never_returns_negative_or_absurd_values_for_zero() {
    let mut state = 1u64;
    for _ in 0..1000 {
        assert_eq!(round_stochastic(0.0, &mut state), 0);
    }
}

/// M11 的变异目标：边界 `ratio == 2.0` 必须判「越线」（`>=`，不是 `>`）。
#[test]
fn classify_ratio_boundary_at_threshold_counts_as_crossed() {
    assert_eq!(classify_ratio(2.0, 2.0), RatioVerdict::CrossedUp);
    assert_eq!(classify_ratio(1.999, 2.0), RatioVerdict::NotCrossed);
    assert_eq!(classify_ratio(0.5, 2.0), RatioVerdict::CrossedDown);
}

/// M12 的变异目标：四胜一不该算「一致」，五轮不同档只能记「不稳定」。
#[test]
fn merge_five_rounds_four_against_one_is_unstable_not_majority() {
    let verdicts = [RatioVerdict::CrossedUp, RatioVerdict::CrossedUp, RatioVerdict::CrossedUp, RatioVerdict::CrossedUp, RatioVerdict::NotCrossed];
    assert_eq!(merge_five_rounds(&verdicts), FiveRoundVerdict::Unstable);
}

#[test]
fn merge_five_rounds_unanimous_reports_that_verdict() {
    let verdicts = [RatioVerdict::NotCrossed; 5];
    assert_eq!(merge_five_rounds(&verdicts), FiveRoundVerdict::Consistent(RatioVerdict::NotCrossed));
}

#[test]
fn trend_rule_expanding_requires_three_consecutive_five_percent_steps() {
    assert_eq!(trend_rule_label(&[1.0, 1.06, 1.13, 1.21]), TrendLabel::ExpandingAlongAxis);
    assert_eq!(trend_rule_label(&[1.0, 1.06, 1.061, 1.062]), TrendLabel::Converging);
}

/// M8 的变异目标：`T_time` 若读成 50 s，10 s 窗口里应有 0 次触发；正确的 5 s 应至少触发 1 次。
#[test]
fn checkpoint_trigger_fires_on_time_threshold_with_correct_constant_but_not_with_mutated_one() {
    let last_checkpoint = Instant::now() - Duration::from_secs(6);
    assert!(should_trigger_checkpoint(Instant::now(), last_checkpoint, 0, Duration::from_millis(5000), 256 * 1024 * 1024), "5 s 阈值下，距上次 6 s 应当触发");
    assert!(!should_trigger_checkpoint(Instant::now(), last_checkpoint, 0, Duration::from_millis(50_000), 256 * 1024 * 1024), "M8：读成 50 s 时同样的间隔不该触发");
}

#[test]
fn checkpoint_trigger_fires_on_dirty_bytes_threshold_even_before_time_threshold() {
    let last_checkpoint = Instant::now();
    assert!(should_trigger_checkpoint(Instant::now(), last_checkpoint, 256 * 1024 * 1024, Duration::from_millis(5000), 256 * 1024 * 1024));
    assert!(!should_trigger_checkpoint(Instant::now(), last_checkpoint, 1, Duration::from_millis(5000), 256 * 1024 * 1024));
}

/// M8 的变异目标（直接测常量，不是 `should_trigger_checkpoint` 本身——那个函数的阈值是参数，
/// 真正会被读错的是 `run_cell_subcommand` 用的这个常量）：`T_time` 必须是 5000 ms，不是 50000 ms。
#[test]
fn checkpoint_time_threshold_constant_is_five_seconds() {
    assert_eq!(CHECKPOINT_TIME_THRESHOLD, Duration::from_millis(5000));
}

/// M13 的变异目标：开环 `issued_at` 必须取排定时刻，不取实际发出时刻——发晚的那一段也算等待。
#[test]
// wallclock-assert-load-safe: 睡 5 ms 之后只断言至少过了 4 ms，是下界，睡过头只会让差更大，负载高也过
fn open_loop_issued_at_uses_scheduled_instant_not_actual() {
    let scheduled = Instant::now();
    std::thread::sleep(Duration::from_millis(5));
    let actual = Instant::now();
    let issued_at = open_loop_issued_at(scheduled, actual);
    assert_eq!(issued_at, scheduled, "M13：issued_at 必须等于排定时刻，不能悄悄换成实际发出时刻");
    assert!(actual.duration_since(issued_at) >= Duration::from_millis(4), "晚发的那几毫秒要能被 W 吃进去");
}

/// M14 的变异目标：「放行到醒」这一段必须能为正——真线程的唤醒不可能恰好为 0。
#[test]
fn release_to_wake_is_positive_when_release_precedes_return() {
    let issued_at = Instant::now();
    let batch_started_at = issued_at;
    let made_durable_at = batch_started_at + Duration::from_micros(10);
    let released_at = made_durable_at;
    let returned_at = released_at + Duration::from_micros(3);
    let sample = SampleRecord { issued_at, batch_started_at, made_durable_at, released_at, returned_at, batch_size: 1 };
    assert!(sample.release_to_wake_nanos() > 0);
    assert!(!sample.violates_containment());
}

#[test]
fn containment_violation_detected_when_order_broken() {
    let now = Instant::now();
    let sample = SampleRecord { issued_at: now + Duration::from_micros(5), batch_started_at: now, made_durable_at: now, released_at: now, returned_at: now, batch_size: 1 };
    assert!(sample.violates_containment(), "issued_at 晚于 batch_started_at 时必须判违反");
}

#[test]
fn mann_whitney_a_is_one_half_when_samples_are_identical_distributions() {
    let first_sample = vec![10u64, 20, 30, 40];
    let second_sample = vec![10u64, 20, 30, 40];
    let discrimination = mann_whitney_discrimination_statistic(&first_sample, &second_sample);
    assert!((discrimination - 0.5).abs() < 1e-9);
}

#[test]
fn mann_whitney_a_is_near_one_when_a_always_greater() {
    let first_sample = vec![100u64, 200, 300];
    let second_sample = vec![1u64, 2, 3];
    assert!(mann_whitney_discrimination_statistic(&first_sample, &second_sample) > 0.99);
}

fn run_recorded_jia_batch(seed: u64) -> (BatchPlan, Instant, Vec<(Instant, StepMark)>) {
    let shape = main_shape(1, Family::OneDataUnitPerFile);
    let mut state = seed;
    let plan = plan_jia_batch(shape, 1, true, &mut state);
    let buffers = UnitBuffers::new(seed);
    let cursors = RegionCursors::new();
    let device = RecordingBlockDevice::new();
    let made_durable_at = execute_batch_persist(&device, &buffers, &cursors, &plan, 4);
    execute_batch_finalize_system_configuration(&device, &buffers, &cursors, &plan);
    let log = device.drain();
    (plan, made_durable_at, log)
}

fn split_by_barrier(marks: &[StepMark]) -> Vec<Vec<StepMark>> {
    let mut phases = vec![Vec::new()];
    for &mark in marks {
        if mark == StepMark::Barrier {
            phases.push(Vec::new());
        } else {
            phases.last_mut().unwrap().push(mark);
        }
    }
    phases
}

/// S1（跑前登记停机条款 S1 的次序部分）与 A4：甲一次发布的次序 = 单元 → 屏障 → 记录 → 屏障 →
/// 根槽 FUA → 系统配置槽（先读两个、再写一个）；A5：两道屏障 + 1 次 FUA = 3 次 FLUSH。
#[test]
fn jia_single_fsync_batch_step_sequence_matches_crates_order() {
    let (plan, _made_durable_at, log) = run_recorded_jia_batch(7);
    let marks: Vec<StepMark> = log.iter().map(|&(_, mark)| mark).collect();
    let barrier_count = marks.iter().filter(|&&mark| mark == StepMark::Barrier).count();
    assert_eq!(barrier_count, 3, "A5：两道屏障 + 1 次 FUA 应等于 3 次 FLUSH（M1 去掉第二道屏障会让这里变 2）");
    let phases = split_by_barrier(&marks);
    assert_eq!(phases.len(), 4, "3 次屏障应把日志切成 4 段：单元段、记录段、根槽段、系统配置段");
    assert_eq!(phases[0].len(), usize_from_u64(plan.named_unit_total()), "第 1 段只应是单元写，条数等于 W");
    assert!(phases[0].iter().all(|mark| matches!(mark, StepMark::Write { kind } if *kind != UnitKind::Record && *kind != UnitKind::RootSlot && *kind != UnitKind::SystemConfigurationSlot)));
    assert_eq!(phases[1].len(), usize_from_u64(plan.record_count), "第 2 段只应是记录写");
    assert!(phases[1].iter().all(|mark| matches!(mark, StepMark::Write { kind: UnitKind::Record })));
    assert_eq!(phases[2], vec![StepMark::Write { kind: UnitKind::RootSlot }], "第 3 段（FUA 之前）只应是根槽写");
    assert_eq!(phases[3], vec![StepMark::Read { kind: UnitKind::SystemConfigurationSlot }, StepMark::Read { kind: UnitKind::SystemConfigurationSlot }, StepMark::Write { kind: UnitKind::SystemConfigurationSlot }], "M9 的反例：系统配置槽写之前必须先读两次");
}

/// A3：`made_durable_at` 必须是根槽那次 FUA（写 + 第 3 道屏障）返回之后的时刻，不能早于它
/// （M2 的变异目标：在第一道屏障之后就把这个时刻报出去）。
#[test]
// wallclock-assert-load-safe: 比的是同一线程里先后录下的两个时刻（根槽写与屏障先发生，made_durable_at 后取），单调时钟上次序由调用先后定，与耗时无关
fn jia_durable_instant_is_not_earlier_than_the_root_slot_fua() {
    let (_plan, made_durable_at, log) = run_recorded_jia_batch(11);
    let last_root_related_timestamp = log
        .iter()
        .filter(|&&(_, mark)| mark == StepMark::Barrier || mark == StepMark::Write { kind: UnitKind::RootSlot })
        .map(|&(instant, _)| instant)
        .take(3) // 前 3 次屏障/根槽写：第 3 次就是 FUA 的 barrier
        .last()
        .expect("至少要有 3 次屏障/根槽写事件");
    assert!(made_durable_at >= last_root_related_timestamp, "M2：made_durable_at 不能早于根槽 FUA 完成的时刻");
}

/// M10 的变异目标：根槽写宽度必须是主几何的 `physical_block_size_bytes`（512），不是 4096。
#[test]
fn root_slot_width_is_primary_physical_block_size() {
    assert_eq!(UnitKind::RootSlot.width_bytes(), PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES);
    assert_ne!(UnitKind::RootSlot.width_bytes(), SYSTEM_CONFIGURATION_SLOT_BYTES);
}

/// M9 的变异目标（直接测系统配置槽读次数）：一次甲批必须读两次系统配置槽。
#[test]
fn system_configuration_slot_is_read_twice_before_being_written() {
    let (_plan, _made_durable_at, log) = run_recorded_jia_batch(23);
    let marks = log.into_iter().map(|(_, mark)| mark).collect::<Vec<_>>();
    let read_count = marks.iter().filter(|&&mark| mark == StepMark::Read { kind: UnitKind::SystemConfigurationSlot }).count();
    assert_eq!(read_count, 2);
}

/// M3 的变异目标：把某个臂的分支换成别的臂的规划函数（「臂写混」）。三条臂在同一个几何、
/// 同一个批大小上必须给出三个不同的字节数，且各自与直接调用对应 `solve_*` 函数（B1/B2 已锚定
/// 的值）一致——这样任何一支被换成别的分支都会被下面两个 `assert_eq!` 抓到。
#[test]
fn plan_batch_for_arm_routes_each_arm_to_its_own_solver_not_a_different_one() {
    let shape = main_shape(10_000, Family::OneDataUnitPerFile);
    let mut state_full = 5u64;
    let write_ahead_log_full_plan = plan_batch_for_arm(Arm::WriteAheadLogFull, shape, 1, true, &mut state_full);
    assert_eq!(write_ahead_log_full_plan.total_write_calls(), 6, "M3：wal_full-K10 那一支必须调用 WAL 的规划函数（B1：6 次写调用）");
    let mut state_leaf = 5u64;
    let write_ahead_log_leaf_plan = plan_batch_for_arm(Arm::WriteAheadLogLeaf, shape, 1, true, &mut state_leaf);
    assert_eq!(write_ahead_log_leaf_plan.total_write_calls(), 4, "M3：乙-M-K10 那一支必须调用乙的规划函数（B2：4 次写调用），不是 wal_full 的");
    let mut state_jia = 5u64;
    let jia_plan = plan_batch_for_arm(Arm::Jia, shape, 1, true, &mut state_jia);
    assert_ne!(jia_plan.total_write_calls(), write_ahead_log_full_plan.total_write_calls(), "M3：甲那一支不许换成 WAL 的规划函数");
}

/// M4 的变异目标：「组提交开」也把批封顶为 1（组提交名存实亡）。组提交开时必须把队列里
/// 现有的全部请求收成一批，不是只取 1 个。
#[test]
fn drain_batch_from_queue_collects_everything_when_group_commit_is_on() {
    let (sender, _receiver) = mpsc::channel();
    let mut queue: VecDeque<QueueEntry> = VecDeque::new();
    queue.push_back(QueueEntry { reply: sender.clone() });
    queue.push_back(QueueEntry { reply: sender.clone() });
    queue.push_back(QueueEntry { reply: sender });
    let batch = drain_batch_from_queue(&mut queue, true);
    assert_eq!(batch.len(), 3, "M4：组提交开时应当把队列里全部 3 个请求收成一批");
    assert!(queue.is_empty(), "收完之后队列应当空");
}

#[test]
fn drain_batch_from_queue_collects_only_one_when_group_commit_is_off() {
    let (sender, _receiver) = mpsc::channel();
    let mut queue: VecDeque<QueueEntry> = VecDeque::new();
    queue.push_back(QueueEntry { reply: sender.clone() });
    queue.push_back(QueueEntry { reply: sender });
    let batch = drain_batch_from_queue(&mut queue, false);
    assert_eq!(batch.len(), 1, "组提交关时一批只能有 1 个（第一节「组提交关」）");
    assert_eq!(queue.len(), 1, "剩下那个必须留在队列里，等下一批");
}

/// M5 的变异目标：`t_issue` 改成在收批时刻取，漏掉排队那一段。用 `SleepingBlockDevice`
/// （固定屏障延迟，不碰真磁盘）逼出排队：4 个闭环调用方、组提交关（一批只服务 1 个），
/// 发布者一批的服务时间约 2×`barrier_delay`，跑的时间窗口够让后到的调用方排上队。
/// 正确实现下，至少要有一个样本的 `batch_started_at − issued_at`（排队段）明显为正；
/// 若 `issued_at` 被换成收批时刻，这一段恒为 0，断言必红。
#[test]
// wallclock-assert: 真线程跑 400 ms 固定窗口，要求窗口内样本数不少于 4 且最大排队段至少 10 ms，窗口内能跑几批随负载变
fn closed_loop_queueing_delay_is_counted_when_group_commit_is_off() {
    let device: Arc<dyn BlockDevice> = Arc::new(SleepingBlockDevice { barrier_delay: Duration::from_millis(15) });
    let configuration = CellConfiguration {
        arm: Arm::WriteAheadLogFull,
        concurrent_fsync_count: 4,
        shared: true,
        group_commit_enabled: false,
        warmup: Duration::from_millis(0),
        duration: Duration::from_millis(400),
        worker_count: 1,
        pc1_inject_duration: None,
        checkpoint_time_threshold: Duration::from_secs(5),
        effective_dirty_bytes_threshold: 256 * 1024 * 1024,
        seed: 17,
        shape: main_shape(10_000, Family::OneDataUnitPerFile),
    };
    let report = run_closed_loop_cell(device, configuration);
    assert!(report.total_samples >= 4, "M5：窗口太短，样本不够（读不到 ≠ 读到 0）");
    let maximum_queueing_nanos = report
        .samples_in_window
        .iter()
        .map(|sample| sample.batch_started_at.saturating_duration_since(sample.issued_at).as_nanos())
        .max()
        .unwrap();
    assert!(u64_from_u128(maximum_queueing_nanos) >= u64_from_nanoseconds(Duration::from_millis(10).as_nanos()), "M5：4 个调用方抢一个串行发布者，至少要有一个样本排过队（观测到 {maximum_queueing_nanos} ns）");
}

/// M6 的变异目标：PC1 注入的忙等被挪到放行之后（`released_at` 算出来之后才等）。
/// 正确实现下，`released_at − made_durable_at` 在被注入的那些批上应当明显偏大
/// （约等于注入时长），在没被注入的批上应当接近 0；两者都要出现，否则说明注入没起作用
/// 或者起作用的时机不对。
#[test]
// wallclock-assert: 真线程跑 300 ms 固定窗口，要求样本数不少于 4，且放行间隔既有不小于 15 ms 的注入批又有小于 15 ms 的未注入批，窗口内批数与间隔随负载变
fn pc1_injected_wait_lands_before_release_not_after() {
    let inject = Duration::from_millis(30);
    let device: Arc<dyn BlockDevice> = Arc::new(SleepingBlockDevice { barrier_delay: Duration::from_millis(1) });
    let configuration = CellConfiguration {
        arm: Arm::Jia,
        concurrent_fsync_count: 1,
        shared: true,
        group_commit_enabled: true,
        warmup: Duration::from_millis(0),
        duration: Duration::from_millis(300),
        worker_count: 1,
        pc1_inject_duration: Some(inject),
        checkpoint_time_threshold: Duration::from_secs(5),
        effective_dirty_bytes_threshold: 256 * 1024 * 1024,
        seed: 29,
        shape: main_shape(10_000, Family::OneDataUnitPerFile),
    };
    let report = run_closed_loop_cell(device, configuration);
    assert!(report.total_samples >= 4, "M6：窗口太短，样本不够（读不到 ≠ 读到 0）");
    let gaps_nanos: Vec<u128> = report.samples_in_window.iter().map(|sample| sample.released_at.saturating_duration_since(sample.made_durable_at).as_nanos()).collect();
    let half_inject_nanos = inject.as_nanos() / 2;
    let has_injected_gap = gaps_nanos.iter().any(|&gap| gap >= half_inject_nanos);
    let has_uninjected_gap = gaps_nanos.iter().any(|&gap| gap < half_inject_nanos);
    assert!(has_injected_gap, "M6：偶数号批次应当在 made_durable_at 之后、released_at 之前吃到那 30 ms（观测 {gaps_nanos:?}）");
    assert!(has_uninjected_gap, "M6：奇数号批次不该被注入，released_at 应当紧跟 made_durable_at（观测 {gaps_nanos:?}）");
}

/// 给了缩规模的暖机、时长或文件数，`cell` 的结果行要写缩过、不当正式产物；三项都是登记值不加这一段。
#[test]
fn reduced_cell_scale_is_marked_and_registered_scale_is_not() {
    let registered_arguments = parse_arguments(&["cell".to_string(), "--device".to_string(), "device".to_string()]);
    assert_eq!(reduced_scale_note(&registered_arguments), "");
    let reduced_arguments = parse_arguments(&["cell".to_string(), "--device".to_string(), "device".to_string(), "--duration-seconds".to_string(), "0.5".to_string(), "--file-count".to_string(), "100".to_string()]);
    assert_eq!(
        reduced_scale_note(&reduced_arguments),
        " reduced=yes registered_duration_seconds=2 registered_file_count=10000 not_official_product=yes"
    );
}
