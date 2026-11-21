//! 对拍的设点文件、展开上限、显存额度与 CPU 判器线程数三个环境变量的读法，卡上显存不够报「本次未跑」不判红。

use super::*;

/// 崩溃点重放取样与崩溃点的设点文件：这份用例文件。
pub(super) const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs";

/// 判表先入库（对拍那一类的库，项名是用例名）、再读库，GPU 判器只拿读回的那一份算；只录那一趟交 `None`（用例当场返回）。
pub(super) fn packed_tables_through_the_library(
    item_name: &str,
    tables: &JudgeTables,
) -> Option<PackedJudgeTablesReadBack> {
    match judge_tables_through_the_pairing_library(item_name, &PackedJudgeTables::of(tables)) {
        StageOutcome::Judged(packed) => Some(packed),
        StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => None,
    }
}

const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";

pub(super) const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";

pub(super) const SECOND_FILE_BYTES: usize = 4100;

pub(super) const THIRD_FILE_BYTES: usize = 2500;

/// 展开上限变量的值（`None` 是没设）：没设取放量引擎的全量域上限 [`EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION`]；
/// 设了只往小缩（取变量与全量域上限里小的那个）；设了而不是非负整数就 panic、消息带变量名与原值，不退回全量域
/// （与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法：缩没缩规模要看得出来，写错的值不许悄悄变成全量域）。
/// 这一族三个变量（展开上限、显存额度、CPU 线程数）都经 [`number_from_the_variable_value_panicking_when_unreadable`] 判。
///
/// # Panics
/// 设了而不是非负整数。
fn expand_up_to_from_the_value_of_the_variable(value: Option<&str>) -> usize {
    number_from_the_variable_value_panicking_when_unreadable(
        EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
        value,
    )
    .map_or(
        EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
        |narrowed: usize| narrowed.min(EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION),
    )
}

/// # Panics
/// 展开上限变量设了而认不出（[`expand_up_to_from_the_value_of_the_variable`]），或它的值不是 UTF-8。
pub(super) fn expand_up_to() -> usize {
    expand_up_to_from_the_value_of_the_variable(
        process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 没设取全量域上限（与放量同一个常量）；设了只往小缩：比全量域上限小的照变量，比它大的仍取全量域上限。
#[test]
fn the_crash_judge_expand_up_to_takes_the_full_domain_of_the_amplification_when_unset_and_only_narrows_when_set(
) {
    assert_eq!(
        expand_up_to_from_the_value_of_the_variable(None),
        EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
        "没设：全量域上限"
    );
    assert_eq!(
        expand_up_to_from_the_value_of_the_variable(Some("6")),
        6,
        "设得比全量域上限小：照变量缩"
    );
    assert_eq!(
        expand_up_to_from_the_value_of_the_variable(Some("40")),
        EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
        "设得比全量域上限大：只往小缩，不放宽"
    );
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=\"eight\" 认不出")]
fn an_unreadable_crash_judge_expand_up_to_value_panics_naming_the_variable_and_the_value() {
    let _limit = expand_up_to_from_the_value_of_the_variable(Some("eight"));
}

/// 显存额度变量的值（`None` 是没设）：没设是 `None`（不给这张卡定额度）；设了而不是非负整数就 panic、消息带变量名与原值，
/// 不悄悄当成没设（量速度时照的就不是配置的额度）。
///
/// # Panics
/// 设了而不是非负整数。
fn card_quota_mebibytes_from_the_value_of_the_variable(value: Option<&str>) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable(CARD_QUOTA_ENVIRONMENT_VARIABLE, value)
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES=\"4GiB\" 认不出")]
fn an_unreadable_crash_judge_card_quota_value_panics_naming_the_variable_and_the_value() {
    let _quota = card_quota_mebibytes_from_the_value_of_the_variable(Some("4GiB"));
}

/// CPU 判器线程数变量的值（`None` 是没设）：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，
/// 设了照它，再没设取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_verification::heavy_threads`）；设了而不是正整数（0 也算）
/// 就 panic、消息带变量名与原值，不悄悄换成至少 1 或默认。
///
/// # Panics
/// 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
pub(super) fn cpu_threads_from_the_value_of_the_variable(
    value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> std::num::NonZeroUsize {
    own_variable_or_default_heavy_threads(CPU_THREADS_ENVIRONMENT_VARIABLE, value, thread_cap_value)
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CPU_THREADS=\"0\" 认不出")]
fn a_zero_crash_judge_cpu_threads_value_panics_naming_the_variable_and_the_value() {
    let _threads = cpu_threads_from_the_value_of_the_variable(Some("0"), None);
}

/// CPU 判器线程数变量没设：取登记的每台默认重活线程数，不取本机核数；设了照它。
#[test]
fn crash_judge_cpu_threads_take_the_registered_heavy_thread_default_when_unset() {
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, None),
        registered_heavy_threads(),
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS 与 SINGLEFS_THREAD_CAP 都没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, Some("5")).get(),
        5
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(Some("3"), Some("5")).get(),
        3
    );
}

/// 量速度的那条用例取一整张卡：空闲显存最大的那张（服务经 `GPU_CARDS_BY_PRIORITY` 交了卡时限在交的卡里，额度照交的），不经卡池记账（它量的就是一张卡独占时多快）。
/// `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 给了就当这张卡的显存额度（量速度时照配置的额度量；
/// 写错就 panic，[`card_quota_mebibytes_from_the_value_of_the_variable`]）。
/// 本机没有 NVIDIA 独显适配器、或服务交的卡一张都用不了时打「本次未跑」那一行、交回 `None`，用例当场返回；服务没交卡、有适配器而一张卡都没起来判红。
pub(super) fn the_whole_emptiest_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
    let quota = card_quota_mebibytes_from_the_value_of_the_variable(
        process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
    );
    let mut card = first_usable_card_or_report_not_run(test_name)?;
    if quota.is_some() {
        card.memory_quota_mebibytes = quota;
    }
    Some(card)
}

/// 卡上一段活（建判器、判）的结局。
#[derive(Debug)]
enum GpuWorkOnTheCard<Outcome> {
    Finished(Outcome),
    /// 卡上 wgpu 报了 Out of Memory：这条用例报「本次未跑」，打的就是这一行。
    NotRunTheGpuMemoryWasShort {
        not_run_line: String,
    },
}

/// 卡上一段活的结局怎么判（不用卡）：没报 Out of Memory 照交 `outcome`（活里 panic 了照样往外抛）；报了（`out_of_memory_error` 是 wgpu
/// 那一句），活做没做完都是显存不够，报「本次未跑」、不判红。显存账只记这个进程借的，两个进程同一刻读到同一份空闲、一起借超了，
/// 由这里兜底（G1 二「去掉跨进程账」，`singlefs_verification::gpu_memory_ledger` 文件头）；
/// 没卡、点名的卡找不到时怎么判不在这里，照旧经卡池（`gpu_test_start::gpu_card_pool_or_report_not_run`）。
fn gpu_work_on_the_card_with_out_of_memory_reported_as_not_run<Outcome>(
    test_name: &str,
    what_ran: &str,
    outcome: std::thread::Result<Outcome>,
    out_of_memory_error: Option<String>,
) -> GpuWorkOnTheCard<Outcome> {
    match (outcome, out_of_memory_error) {
        (Ok(outcome), None) => GpuWorkOnTheCard::Finished(outcome),
        (Ok(_) | Err(_), Some(error)) => GpuWorkOnTheCard::NotRunTheGpuMemoryWasShort {
            not_run_line: not_run_line(
                test_name,
                &format!(
                    "显存不够：{what_ran}时卡上 wgpu 报了 Out of Memory（{error}）。显存账只记这个进程借的，卡上别的进程同一刻借走了"
                ),
            ),
        },
        (Err(panic), None) => std::panic::resume_unwind(panic),
    }
}

/// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（[`gpu_work_on_the_card_with_out_of_memory_reported_as_not_run`]）：
/// 报了打「本次未跑」那一行、交 `None`，调用的用例当场返回；没报交 `work` 的结果。`work` 里因为缓冲没建成而 panic 的一并接住。
/// wgpu 的错误范围按线程分：`work` 要在调它的这条线程上建判器、派活。
pub(super) fn on_the_card_or_report_not_run_when_out_of_memory<Outcome>(
    test_name: &str,
    card: &GpuCard,
    what_ran: &str,
    work: impl FnOnce() -> Outcome,
) -> Option<Outcome> {
    let out_of_memory_scope = card
        .device()
        .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    let out_of_memory_error =
        pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
    match gpu_work_on_the_card_with_out_of_memory_reported_as_not_run(
        test_name,
        what_ran,
        outcome,
        out_of_memory_error,
    ) {
        GpuWorkOnTheCard::Finished(outcome) => Some(outcome),
        GpuWorkOnTheCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
            println!("{not_run_line}");
            None
        }
    }
}

/// 放量路径交回的错是不是卡上 wgpu 报了 Out of Memory 那一句（`singlefs_verification::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`，
/// 中间那一段是哪张卡、哪一段）：按那一句自己的头尾认，不另抄一份句子。
fn is_the_reason_that_the_card_ran_out_of_gpu_memory(reason: &str) -> bool {
    const PLACE_OF_WHAT_RAN_OUT: char = '\u{0}';
    let template = reason_for_running_out_of_gpu_memory(&PLACE_OF_WHAT_RAN_OUT.to_string());
    let (head, tail) = template
        .split_once(PLACE_OF_WHAT_RAN_OUT)
        .expect("那一句里有放「哪一段、哪张卡」的地方");
    reason.len() >= head.len() + tail.len() && reason.starts_with(head) && reason.ends_with(tail)
}

/// 放量路径在调用方起的卡上判一趟（`crash_amplification::judge_my_blocks_on_gpu_cards`）的结局：卡上 wgpu 报了 Out of Memory
/// （放量路径已打它自己的「本次未跑」那一行、交回的错是那一句）时这条用例也报「本次未跑」、交 `None`；别的错判红。
///
/// # Panics
/// 放量路径交回 Out of Memory 之外的错。
pub(super) fn judged_on_the_cards_or_report_not_run_when_out_of_memory<Judged>(
    test_name: &str,
    judged: Result<Judged, String>,
) -> Option<Judged> {
    match judged {
        Ok(judged) => Some(judged),
        Err(reason) if is_the_reason_that_the_card_ran_out_of_gpu_memory(&reason) => {
            println!("{}", not_run_line(test_name, &reason));
            None
        }
        Err(reason) => panic!("{test_name}：放量路径没判完：{reason}"),
    }
}

/// 卡上报了 Out of Memory（判器建成、判的时候报的，与建判器时缓冲没建成、连带 panic 的两形）判成「本次未跑」：那一行带
/// `NOT_RUN_NO_GPU_ADAPTER` 记号、用例名与「显存不够」，不判红；没报照交结果。假的 wgpu 错误句子交给判那一支的函数，不用卡。
#[test]
fn an_out_of_memory_reported_on_the_card_is_reported_as_not_run_and_not_as_red() {
    const TEST_NAME: &str =
        "an_out_of_memory_reported_on_the_card_is_reported_as_not_run_and_not_as_red";
    const FAKE_OUT_OF_MEMORY_ERROR: &str = "Out of Memory";
    let finished = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run(
        TEST_NAME,
        "判",
        Ok(7_u32),
        None,
    );
    assert!(
        matches!(finished, GpuWorkOnTheCard::Finished(7)),
        "没报 Out of Memory 照交结果：{finished:?}"
    );
    let reported_after_the_work = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run(
        TEST_NAME,
        "判",
        Ok(7_u32),
        Some(FAKE_OUT_OF_MEMORY_ERROR.to_string()),
    );
    let reported_with_a_panic = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run::<u32>(
        TEST_NAME,
        "建判器",
        Err(Box::new("缓冲没建成")),
        Some(FAKE_OUT_OF_MEMORY_ERROR.to_string()),
    );
    for reported in [reported_after_the_work, reported_with_a_panic] {
        let GpuWorkOnTheCard::NotRunTheGpuMemoryWasShort { not_run_line } = &reported else {
            panic!("卡上报了 Out of Memory 要报本次未跑、不判红：{reported:?}");
        };
        assert!(
            not_run_line.starts_with(&format!(
                "{GPU_NOT_RUN_MARKER} test={TEST_NAME} reason=显存不够："
            )),
            "那一行带记号、用例名，原因写明显存不够：{not_run_line}"
        );
        assert!(
            not_run_line.contains(FAKE_OUT_OF_MEMORY_ERROR),
            "那一行带 wgpu 报的那一句：{not_run_line}"
        );
    }
}

/// 没报 Out of Memory 而活里 panic 了（判错、内核没判完）：照样往外抛、判红，不当成显存不够。
#[test]
#[should_panic(expected = "内核没把状态判完")]
fn a_panic_on_the_card_without_an_out_of_memory_is_still_red() {
    let _not_reached = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run::<u32>(
        "a_panic_on_the_card_without_an_out_of_memory_is_still_red",
        "判",
        Err(Box::new("内核没把状态判完")),
        None,
    );
}

/// 放量路径交回的错：卡上报了 Out of Memory 那一句认成显存不够（中间那一段是哪张卡、哪一段，照放量路径的写法造），
/// 别的错（建不出表、拒判）不认，照旧判红。
#[test]
fn only_the_out_of_memory_reason_of_the_product_path_is_taken_as_the_card_running_out_of_gpu_memory(
) {
    assert!(is_the_reason_that_the_card_ran_out_of_gpu_memory(
        &reason_for_running_out_of_gpu_memory("序号 0 的卡判一块时（Out of Memory）")
    ));
    for other_reason in [
        "建不出这条流的表：不支持清零写",
        "显存不够：此刻池里没有一张卡借得出 12 MiB",
        "",
    ] {
        assert!(
            !is_the_reason_that_the_card_ran_out_of_gpu_memory(other_reason),
            "「{other_reason}」不是卡上报了 Out of Memory"
        );
    }
}

/// 一条草稿缓冲装几个状态，按卡池里绑定上限最小的那张卡估（借卡之前还不知道借到哪张）。
pub(super) fn states_of_one_scratch_lane_on_every_card_of(
    cards: GpuCardPoolStartedAtTheFirstLease,
) -> usize {
    states_per_scratch_lane(cards.smallest_maximum_storage_buffer_binding_size())
}
