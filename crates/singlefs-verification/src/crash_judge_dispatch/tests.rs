use super::*;
use crate::crash_judge_gpu::recovery_kernel_source;

/// 卡上一次派活要多久：整次的用时与最久那一次提交，只由这次派活带的序号段定。
struct SecondsOnAFakeCard {
    of_the_dispatch: f64,
    of_the_longest_submission: f64,
}

/// 假卡上判完一条流交回的两样：逐次派的序号段，与判器交回的结局（逐个状态的结论，或停在拆不开的下限上的错）。
type DispatchesAndOutcomeOnAFakeCard = (
    Vec<std::ops::Range<u64>>,
    Result<Vec<[u32; VERDICT_WORDS]>, super::StuckAtTheIndivisibleFloor>,
);

/// 经 `schedule` 判 `ordinals`（流一共 `ordinals.end` 个状态），派活交给假的卡（第一个结论字写序号），交回逐次派的序号段与判器交回的结局。
fn judge_on_a_fake_card_until_it_stops(
    schedule: &super::DispatchWidthSchedule,
    ordinals: std::ops::Range<u64>,
    seconds_on_the_card: &dyn Fn(&std::ops::Range<u64>) -> SecondsOnAFakeCard,
) -> DispatchesAndOutcomeOnAFakeCard {
    let mut dispatched = Vec::new();
    let state_count = ordinals.end;
    let outcome = schedule.judge_ordinals(ordinals, "fake card", state_count, |one_dispatch| {
        let seconds = seconds_on_the_card(&one_dispatch);
        dispatched.push(one_dispatch.clone());
        super::TimedDispatch {
            verdicts: one_dispatch
                .map(|ordinal| {
                    let mut words = [0u32; VERDICT_WORDS];
                    words[0] = u32::try_from(ordinal).expect("假的流只有几千个状态");
                    words
                })
                .collect(),
            seconds: seconds.of_the_dispatch,
            seconds_of_the_longest_submission: seconds.of_the_longest_submission,
        }
    });
    (dispatched, outcome)
}

/// 同 [`judge_on_a_fake_card_until_it_stops`]，判器不许停在下限上；只交回逐次派的序号段，核每个序号恰好判一次、结论按序号次序。
fn judge_on_a_fake_card(
    schedule: &super::DispatchWidthSchedule,
    ordinals: std::ops::Range<u64>,
    seconds_on_the_card: &dyn Fn(&std::ops::Range<u64>) -> SecondsOnAFakeCard,
) -> Vec<std::ops::Range<u64>> {
    let (dispatched, outcome) =
        judge_on_a_fake_card_until_it_stops(schedule, ordinals.clone(), seconds_on_the_card);
    let verdicts = outcome.expect("这条假流上判器不停在拆不开的下限上");
    let judged: Vec<u64> = verdicts.iter().map(|words| u64::from(words[0])).collect();
    let expected: Vec<u64> = ordinals.collect();
    assert!(judged == expected, "每个序号恰好判一次、按序号次序");
    dispatched
}

/// 不管带几个状态，每次派活都是 `seconds` 秒、只有一次提交的假卡：状态在卡上并行，加宽不加时。
fn the_same_seconds_whatever_the_width(
    seconds: f64,
) -> impl Fn(&std::ops::Range<u64>) -> SecondsOnAFakeCard {
    move |_one_dispatch| SecondsOnAFakeCard {
        of_the_dispatch: seconds,
        of_the_longest_submission: seconds,
    }
}

/// 每个状态 `seconds_per_state` 秒、一次派活只有一次提交的假卡。
fn seconds_in_proportion_to_the_states(
    seconds_per_state: f64,
) -> impl Fn(&std::ops::Range<u64>) -> SecondsOnAFakeCard {
    move |one_dispatch| {
        let seconds = (one_dispatch.end - one_dispatch.start) as f64 * seconds_per_state;
        SecondsOnAFakeCard {
            of_the_dispatch: seconds,
            of_the_longest_submission: seconds,
        }
    }
}

fn states_of_each_dispatch(dispatched: &[std::ops::Range<u64>]) -> Vec<u64> {
    dispatched
        .iter()
        .map(|one_dispatch| one_dispatch.end - one_dispatch.start)
        .collect()
}

/// 轻的流：判器第一次派活带 1 个状态，用时短就一路加倍，涨满一条草稿缓冲才量宽度，量宽度从一条缓冲一档一档加，取量出来的那一档。
/// 每个状态的用时是 2 的负整数次幂，几档每个状态的用时逐字相同，取最窄的一档。
#[test]
fn light_flow_starts_at_one_state_doubles_up_to_one_lane_and_then_calibrates() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(100, 400);
    assert_eq!(schedule.width(), 1, "第一次派活之前就是 1 个状态");
    let dispatched = judge_on_a_fake_card(
        &schedule,
        0..2000,
        &seconds_in_proportion_to_the_states(1.0 / 65_536.0),
    );
    let states = states_of_each_dispatch(&dispatched);
    assert_eq!(
        states[..11],
        [1, 2, 4, 8, 16, 32, 64, 100, 200, 300, 400],
        "起步加倍，64 个之后宽度到了一条缓冲（100 个）才量，量的时候逐档 100、200、300、400"
    );
    assert_eq!(schedule.width(), 100, "量出来的那一档");
    assert!(
        states[11..]
            .iter()
            .all(|states_carried| *states_carried <= 100),
        "量完之后照量出来的宽度派（末尾那一次剩多少带多少）"
    );
}

/// 重的流（每个状态 0.6 秒，过了 0.5 秒，用时随状态数成正比：状态互相拖慢）：宽度 1 那一次 0.6 秒是拆不开的下限，没过驱动那条线的一半，加宽到 2；
/// 2 个状态 1.2 秒，过了那条线的一半、没过那条线，停在 2，不去量宽度（量宽度头一档就是一条缓冲）。末尾只剩 1 个状态，照带 1 个、宽度不动。
#[test]
fn heavy_flow_whose_time_grows_with_the_states_settles_at_two_states_without_calibrating() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(4, 16);
    let dispatched =
        judge_on_a_fake_card(&schedule, 0..40, &seconds_in_proportion_to_the_states(0.6));
    let mut expected = vec![1u64];
    expected.extend([2u64; 19]);
    expected.push(1);
    assert_eq!(states_of_each_dispatch(&dispatched), expected);
    assert_eq!(schedule.width(), 2);
}

/// 崩溃放量那条重的流在 5090 上的样子：一个状态单次提交 0.565 秒，状态在卡上并行、加宽不加时。宽度 1 那一次是拆不开的下限，
/// 不拿它跟 0.5 秒比：一路加倍，到一条草稿缓冲（64 个）就量宽度（64、128 两档），取每个状态最快的那一档。不停在 1 个状态上，也不报错。
#[test]
fn heavy_flow_whose_states_run_side_by_side_on_the_card_widens_from_the_indivisible_floor_up_to_the_capacity(
) {
    let schedule = super::DispatchWidthSchedule::starting_narrow(64, 128);
    let dispatched = judge_on_a_fake_card(
        &schedule,
        0..2000,
        &the_same_seconds_whatever_the_width(0.565),
    );
    let states = states_of_each_dispatch(&dispatched);
    assert_eq!(
        states[..8],
        [1, 2, 4, 8, 16, 32, 64, 128],
        "加倍到一条缓冲（64 个）就量宽度，逐档 64、128"
    );
    assert_eq!(schedule.width(), 128, "量出来的那一档：容量");
}

/// 单个状态的提交就过了驱动会掐的那条线（2.5 秒）：宽度停在 1 个状态，连着 [`super::DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING`] 次
/// 就报错、不再派；错里带卡、下限用时、判到哪、判了几个、一共几个，打成一行可 grep 的行。
#[test]
fn a_judge_stuck_at_the_indivisible_floor_over_the_driver_line_reports_it_after_eight_dispatches_in_a_row(
) {
    let schedule = super::DispatchWidthSchedule::starting_narrow(4, 16);
    let (dispatched, outcome) = judge_on_a_fake_card_until_it_stops(
        &schedule,
        0..40,
        &the_same_seconds_whatever_the_width(2.5),
    );
    assert_eq!(
        states_of_each_dispatch(&dispatched),
        vec![1; 8],
        "一个一个派，派到第 8 个就停"
    );
    let stuck = outcome.expect_err("停在下限上连着 8 次：报错");
    assert_eq!(
        stuck,
        super::StuckAtTheIndivisibleFloor {
            card_name: "fake card".to_string(),
            seconds_of_the_floor: 2.5,
            dispatches_in_a_row: 8,
            next_ordinal: 8,
            states_judged_by_the_judge: 8,
            state_count: 40,
        }
    );
    let line = stuck.line(Some("一条流"));
    assert!(
        line.starts_with(
            "CRASH_JUDGE_STUCK_AT_THE_INDIVISIBLE_FLOOR flow=一条流 card=\"fake card\" seconds_of_the_floor=2.500 dispatches_in_a_row=8 "
        ) && line.contains(" next_ordinal=8 states_judged_by_the_judge=8 state_count=40 "),
        "{line}"
    );
}

/// 单个状态 1.5 秒：没过驱动那条线（2 秒），可过了它的一半，加一倍宽、状态互相拖慢时就过线，宽度不敢加，停在 1 个状态；
/// 照样连着 8 次就报错，不默默一个一个派下去。
#[test]
fn a_judge_stuck_at_the_indivisible_floor_over_half_the_driver_line_reports_it_too() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(4, 16);
    let (dispatched, outcome) = judge_on_a_fake_card_until_it_stops(
        &schedule,
        0..40,
        &the_same_seconds_whatever_the_width(1.5),
    );
    assert_eq!(states_of_each_dispatch(&dispatched), vec![1; 8]);
    assert_eq!(
        outcome
            .expect_err("停在下限上连着 8 次：报错")
            .dispatches_in_a_row,
        8
    );
}

/// 零星几个重状态（连着 7 个单个就过驱动那条线）之后状态变轻：连着的次数没到 8，不报错，变轻之后宽度照样涨回去。
#[test]
fn seven_heavy_states_in_a_row_at_the_indivisible_floor_are_not_reported() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(4, 16);
    let dispatched = judge_on_a_fake_card(&schedule, 0..40, &|one_dispatch| {
        let seconds = if one_dispatch.start < 7 { 2.5 } else { 0.001 };
        SecondsOnAFakeCard {
            of_the_dispatch: seconds,
            of_the_longest_submission: seconds,
        }
    });
    assert_eq!(
        states_of_each_dispatch(&dispatched)[..9],
        [1, 1, 1, 1, 1, 1, 1, 1, 2],
        "重的那 7 个一个一个派，变轻之后加倍"
    );
}

/// 每个状态 0.1 秒、用时随状态数成正比：宽度 1 那一次量了下限，之后最久一次提交不过驱动那条线的一半（1 秒）就加倍，1、2、4、8 个；
/// 16 个 1.6 秒，过了那条线的一半、没过那条线，停在 16，一直不过那条线。
#[test]
fn the_width_of_a_flow_slowing_in_proportion_settles_past_half_the_driver_line() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(2170, 31_291);
    let dispatched =
        judge_on_a_fake_card(&schedule, 0..79, &seconds_in_proportion_to_the_states(0.1));
    assert_eq!(
        states_of_each_dispatch(&dispatched),
        [1, 2, 4, 8, 16, 16, 16, 16]
    );
    assert_eq!(schedule.width(), 16);
}

/// 流半路变重再变轻：变重那一段（单个状态就过驱动那条线，2.5 秒）每次减半到 1 个状态，变回轻的又从 1 个涨回去。
/// 变重那一段到序号 385 为止，宽度到 1 之后只派 4 次，连着停在下限上的次数到不了报错那一档。
#[test]
fn the_width_falls_to_one_state_when_the_flow_turns_heavy_and_grows_back_when_it_turns_light() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(2170, 31_291);
    let heavy_span = 127u64..385u64;
    let dispatched = judge_on_a_fake_card(&schedule, 0..600, &|one_dispatch| {
        let touches_the_heavy_span =
            one_dispatch.start < heavy_span.end && one_dispatch.end > heavy_span.start;
        let seconds = if touches_the_heavy_span { 2.5 } else { 0.001 };
        SecondsOnAFakeCard {
            of_the_dispatch: seconds,
            of_the_longest_submission: seconds,
        }
    });
    assert_eq!(
        states_of_each_dispatch(&dispatched)[..15],
        [1, 2, 4, 8, 16, 32, 64, 128, 64, 32, 16, 8, 4, 2, 1],
        "加倍到 128 个撞上重的那一段（序号 127 起），之后减半到 1"
    );
    assert!(schedule.width() > 1, "变回轻的之后又涨回去");
}

/// 比的是最久那一次提交，不是整次：一次派活拆成很多次提交，整次 5 秒、最久一次 0.01 秒，照样算快。
#[test]
fn dispatch_of_many_quick_submissions_counts_as_quick() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(2170, 31_291);
    let dispatched = judge_on_a_fake_card(&schedule, 0..7, &|_one_dispatch| SecondsOnAFakeCard {
        of_the_dispatch: 5.0,
        of_the_longest_submission: 0.01,
    });
    assert_eq!(states_of_each_dispatch(&dispatched), [1, 2, 4]);
}

/// 一段序号判完时剩的状态比宽度少，下一段接着用原来的宽度，不算带满。
#[test]
fn range_shorter_than_the_width_leaves_the_width_alone() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(2170, 31_291);
    let quick = seconds_in_proportion_to_the_states(0.001);
    assert_eq!(judge_on_a_fake_card(&schedule, 0..1, &quick).len(), 1);
    assert_eq!(schedule.width(), 2, "带满了 1 个状态：加倍");
    assert_eq!(judge_on_a_fake_card(&schedule, 1..2, &quick).len(), 1);
    assert_eq!(schedule.width(), 2, "宽度 2 只带了 1 个状态：不动");
}

/// 量过一次之后，派满 [`super::DISPATCHES_BETWEEN_CALIBRATIONS`] 次活再量一次：又出现比量出来的宽度宽的一次派活。
#[test]
fn the_width_is_calibrated_again_after_the_dispatches_between_calibrations() {
    let schedule = super::DispatchWidthSchedule::starting_narrow(4, 16);
    let dispatched = judge_on_a_fake_card(
        &schedule,
        0..4000,
        &seconds_in_proportion_to_the_states(1.0 / 65_536.0),
    );
    let states = states_of_each_dispatch(&dispatched);
    assert_eq!(
        states[..6],
        [1, 2, 4, 8, 12, 16],
        "起步涨到一条缓冲（4 个），接着头一次量宽度，逐档 4、8、12、16"
    );
    let dispatches_between =
        usize::try_from(super::DISPATCHES_BETWEEN_CALIBRATIONS).expect("装得进 usize");
    assert!(
        states[6..6 + dispatches_between]
            .iter()
            .all(|states_carried| *states_carried == 4),
        "两次量宽度之间照量出来的 4 个派"
    );
    assert_eq!(
        states[6 + dispatches_between..6 + dispatches_between + 4],
        [4, 8, 12, 15],
        "派满了再量一次；头一次最宽那一档 16 不是选中的，这一次只试到它之前的 15（knee_after_calibration）"
    );
}

/// 派活这一侧一轮的位置数与绑定号是扫描内核读的那两样：内核里 `C_ACCOUNTING_SLICE_POSITIONS` 是 1 左移 `C_ACCOUNTING_SLICE_POSITION_BITS` 位，
/// 分片那一趟的派活参数绑在第 [`super::ACCOUNTING_SLICE_BINDING`] 号。两边一漂，末片的上端不是内核认的「罩满」，每个状态记判不了（或只判了一截）。不用卡。
#[test]
fn the_accounting_slice_positions_and_binding_of_the_dispatch_are_the_ones_the_scan_kernel_reads() {
    let source = crate::crash_judge_gpu::checker_scan_kernel_source();
    let declaration_prefix = "const C_ACCOUNTING_SLICE_POSITION_BITS: u32 = ";
    let bits: u32 = source
        .lines()
        .find_map(|line| line.strip_prefix(declaration_prefix))
        .expect("扫描内核里有 C_ACCOUNTING_SLICE_POSITION_BITS 那一行")
        .strip_suffix("u;")
        .expect("那一行以 `u;` 收尾")
        .parse()
        .expect("位数是一个十进制数");
    assert_eq!(
        1u32 << bits,
        super::ACCOUNTING_SLICE_POSITIONS,
        "内核一轮 1 << {bits} 个位置，派活这一侧 {} 个",
        super::ACCOUNTING_SLICE_POSITIONS
    );
    let binding_declaration = format!(
        "@group(0) @binding({}) var<uniform> accounting_slice_parameters:",
        super::ACCOUNTING_SLICE_BINDING
    );
    assert_eq!(
        source.matches(binding_declaration.as_str()).count(),
        1,
        "扫描内核里分片那一趟的派活参数绑在第 {} 号",
        super::ACCOUNTING_SLICE_BINDING
    );
}

/// 双机崩溃放量入口给多机配置 `GPU_KERNEL_SERIAL_COMPILE_HOSTS` 点名的那几台设的变量（`serial_compile_environment=(` 那一行里的 `<变量名>=1`）
/// 就是判器读的那一个：两边的名字一漂，那几台照几条一起编内核、内存紧时撞上限，判出来的结论不变，别的用例都看不见。不用卡。
#[test]
fn the_two_host_crash_amplification_entry_sets_the_variable_the_judge_reads_to_compile_its_kernels_one_at_a_time(
) {
    let entry_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/crash/crash-amplification-two-hosts.sh");
    let entry = std::fs::read_to_string(&entry_path).unwrap_or_else(|error| {
        panic!("双机崩溃放量入口 {} 读不出：{error}", entry_path.display())
    });
    let serial_compile_environment_line = entry
        .lines()
        .find(|line| line.starts_with("serial_compile_environment=("))
        .expect("双机崩溃放量入口里有给内核一个一个编的那几台设环境的 serial_compile_environment=( 那一行");
    let assignment = format!(
        " {}=1 ",
        super::GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE
    );
    assert!(
        serial_compile_environment_line.contains(&assignment),
        "双机崩溃放量入口给内核一个一个编的那几台设的环境里要有「{}」（判器读的变量名），实际那一行：{serial_compile_environment_line}",
        assignment.trim()
    );
}

/// 这个进程到此刻为止常驻内存的峰值（MiB，`/proc/self/status` 的 VmHWM）；读不到报 0。
fn peak_resident_mebibytes() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .map_or(0, |kibibytes| kibibytes / 1024)
}

/// 三个内核在本机第一张能用的卡上一个一个编（从小到大），每编完一个打一行：编了多久、到那一刻进程常驻内存的峰值、
/// 内核原文多大。查「这台机器编判器内核要多少内存、卡在哪一个上」用；中途被内存上限杀掉，已经打出来的行还在。
/// 要量冷编，把驱动的盘上缓存指到一个空目录（环境变量 `__GL_SHADER_DISK_CACHE_PATH`）。
#[test]
#[ignore = "要本机一张能用的 NVIDIA 卡（没有 NVIDIA 卡报「本次未跑」，有卡而用不上判红）；冷编每个内核几分钟"]
fn each_kernel_compile_time_and_peak_memory_one_at_a_time() {
    use crate::crash_judge_gpu::{checker_scan_kernel_source, checker_walk_kernel_source};
    use std::io::Write;
    let Some(card) = crate::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run(
        "each_kernel_compile_time_and_peak_memory_one_at_a_time",
    ) else {
        return;
    };
    println!(
        "GPU_KERNEL_COMPILE_CARD card={:?} index={} free_mebibytes={} peak_resident_mebibytes_before={}",
        card.name,
        card.index,
        card.free_memory_mebibytes,
        peak_resident_mebibytes()
    );
    for (kernel, source, entry_point) in [
        ("recovery", recovery_kernel_source(), KERNEL_ENTRY_POINT),
        (
            "checker-scan",
            checker_scan_kernel_source(),
            KERNEL_ENTRY_POINT,
        ),
        (
            "checker-scan-allocation-records-disjoint",
            checker_scan_kernel_source(),
            CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
        ),
        (
            "checker-scan-allocation-generations-through-the-rollback-floor-raises",
            checker_scan_kernel_source(),
            CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
        ),
        (
            "checker-accounting-head",
            checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
        ),
        (
            "checker-accounting-slice",
            checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
        ),
        (
            "checker-accounting-decide",
            checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
        ),
        (
            "checker-accounting-final",
            checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
        ),
        (
            "checker-references-and-snapshots",
            checker_scan_kernel_source(),
            CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
        ),
        (
            "checker-scan-rebuild",
            checker_scan_kernel_source(),
            CHECKER_SCAN_REBUILD_ENTRY_POINT,
        ),
        (
            "checker-walk",
            checker_walk_kernel_source(),
            KERNEL_ENTRY_POINT,
        ),
    ] {
        println!(
            "GPU_KERNEL_COMPILE_STARTED kernel={kernel} source_bytes={}",
            source.len()
        );
        std::io::stdout().flush().expect("刷得出去");
        let started = std::time::Instant::now();
        let _pipeline = super::compute_pipeline(&card, kernel, &source, entry_point);
        println!(
            "GPU_KERNEL_COMPILE kernel={kernel} seconds={:.1} peak_resident_mebibytes={}",
            started.elapsed().as_secs_f64(),
            peak_resident_mebibytes()
        );
        std::io::stdout().flush().expect("刷得出去");
    }
}

/// 量一次恢复内核在本机第一张能用的卡上的编译时间（要显卡；打一行 `GPU_KERNEL_COMPILE kernel=recovery seconds=…`）：
/// 连跑两次、比第二次快不快，就知道驱动的盘上着色器缓存管不管 Vulkan 管线。
#[test]
#[ignore = "要本机一张能用的 NVIDIA 卡（没有 NVIDIA 卡报「本次未跑」，有卡而用不上判红），编一次几十秒"]
fn recovery_kernel_compile_time_on_the_first_usable_card() {
    let Some(card) = crate::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run(
        "recovery_kernel_compile_time_on_the_first_usable_card",
    ) else {
        return;
    };
    let started = std::time::Instant::now();
    let _pipeline = super::compute_pipeline(
        &card,
        "crash-judge-recovery",
        &recovery_kernel_source(),
        KERNEL_ENTRY_POINT,
    );
    println!(
        "GPU_KERNEL_COMPILE kernel=recovery seconds={:.1}",
        started.elapsed().as_secs_f64()
    );
}

/// 场景：在卡池借的一张卡上，照调用方给的宽度（一条草稿缓冲装 1 个状态、一次派活 2 个状态）建判器（`on_card_with_the_given_dispatch_width`）；
/// 表是现造的 256 个零字、3 个状态，经入库那一种字节编码与摘要读回（`PackedJudgeTablesReadBack::from_the_library_bytes`），不判状态、只看判器开了什么。
/// 预期：判器一条缓冲装 1 个、开 2 条缓冲、一次派活最多 2 个，照给的开，不照这张卡的绑定上限与显存额度另算；
/// 对照：照这张卡的显存额度算的那一份（`DispatchWidthGivenToTheJudge::within_the_memory_budget_of_the_card`）一条缓冲装的不是 1 个
/// （不是的话下面那几句分不出判器用的是哪一份）。不读仓里的文件。
/// 判别力（`crates/mutations.tsv`）：上卡的入口不收给的宽度、照这张卡的显存额度重算，一条缓冲装的个数那一句红。
#[test]
fn the_judge_opens_its_scratch_lanes_at_the_width_the_caller_gives_not_at_the_memory_budget_of_the_card(
) {
    use super::{
        mebibytes_a_judge_needs, DispatchWidthGivenToTheJudge, GpuCrashJudge, PackedJudgeTables,
        PackedJudgeTablesReadBack,
    };
    use crate::crash_judge_gpu::JudgeKernelSources;
    use crate::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str = "the_judge_opens_its_scratch_lanes_at_the_width_the_caller_gives_not_at_the_memory_budget_of_the_card";
    const STATES_PER_LANE_GIVEN: usize = 1;
    const STATES_PER_DISPATCH_GIVEN: usize = 2;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let bytes = PackedJudgeTables {
        words: vec![0; 256],
        state_count: 3,
    }
    .to_bytes();
    let packed = PackedJudgeTablesReadBack::from_the_library_bytes(
        &bytes,
        &crate::sha256::sha256_digest(&bytes),
    )
    .expect("摘要是照这份字节现算的、字节是 to_bytes 编的：读得回");
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        mebibytes_a_judge_needs(&packed, STATES_PER_DISPATCH_GIVEN),
    ) else {
        return;
    };
    let width_of_the_memory_budget =
        DispatchWidthGivenToTheJudge::within_the_memory_budget_of_the_card(
            lease.card(),
            &packed,
            STATES_PER_DISPATCH_GIVEN,
        );
    assert_ne!(
        width_of_the_memory_budget.states_per_lane, STATES_PER_LANE_GIVEN,
        "对照：照这张卡的绑定上限一条缓冲装的个数与给的不同"
    );
    let judge = GpuCrashJudge::on_card_with_the_given_dispatch_width(
        lease.card(),
        &packed,
        DispatchWidthGivenToTheJudge::of_lanes_holding(
            STATES_PER_LANE_GIVEN,
            STATES_PER_DISPATCH_GIVEN,
        ),
        &JudgeKernelSources::of_the_judge(),
    );
    let opened = (
        judge.states_per_lane,
        judge.scratch_lanes.len(),
        judge.states_per_dispatch(),
    );
    drop(judge);
    drop(lease);
    assert_eq!(
        opened.0, STATES_PER_LANE_GIVEN,
        "一条草稿缓冲装的个数照调用方给的，不照这张卡的绑定上限另算"
    );
    assert_eq!(
        (opened.1, opened.2),
        (
            STATES_PER_DISPATCH_GIVEN.div_ceil(STATES_PER_LANE_GIVEN),
            STATES_PER_DISPATCH_GIVEN
        ),
        "草稿缓冲的条数与一次派活最多几个照调用方给的开"
    );
}
