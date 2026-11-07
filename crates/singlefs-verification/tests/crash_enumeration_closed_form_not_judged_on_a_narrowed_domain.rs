//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 缩过域的那一趟崩溃枚举（`SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 设了）：各用例「枚举到的态数等于闭式」那一条断言经共用助手
//! （`crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed`）判，闭式按全量域算、这一趟不判，
//! 结果行写明缩过、展开上限与实际展开的态数；每个崩溃块照判（池级 checker、多版本 oracle 一条不红，枚举到的恰好是缩过之后那个域的态数）。
//! 不缩的那一趟照判闭式，由 `crash_state_enumeration` 的库单测钉。
//!
//! 流是新池新建文件流（mkfs → 取号两轮 → 暖机两次 → 第一个文件），段长 2、2、2、1、2、2、1、2、24、2、1、2：上限取 4，
//! 写数 24 的单元写那一段不展开，别的段照全量展开，几十个状态、单跑几秒。库是这一条自己的一次性库（临时 prune-server 里的一段名字空间），
//! 判器只开 CPU（多机配置这一条自己写），不写真服务。项名取新池新建文件流的名字起头：来历表按 `::` 之前的流名认这条流（同一条流、同一行来历）。

mod common;

use common::{build_pool, file_content, geometry};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
};
use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, ExpandUpToWritesPerSegment, EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::crash_library::DisposableLibrary;
use singlefs_verification::crash_state_enumeration::{
    assert_the_counted_states_equal_the_closed_form_unless_narrowed, enumerate_crash_states,
    ClosedFormStateCountJudgement, CrashPointReplayItem, CLOSED_FORM_NOT_JUDGED_LINE_PREFIX,
};
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::memory_pool::writes_and_segments;
use singlefs_verification::segments::StepKind;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_closed_form_not_judged_on_a_narrowed_domain.rs";

/// 这一趟缩到的展开上限：盖得住这条流里写数 2 与 1 的段，盖不住写数 24 的那一段。
const NARROWED_EXPAND_UP_TO: usize = 4;

/// 缩过域的那一趟：闭式不判（助手交回不判、打的结果行带缩过的上限与实际展开的态数），每个崩溃块照判（一条不红，
/// 枚举到的态数等于缩过之后那个域的闭式，且比全量域的闭式少）。
/// 判别力（`crates/mutations.tsv`）：助手缩过域也照判闭式，这条红在助手那一条闭式断言上。
// crash-case-check:not-a-crash-case 只测缩域那一趟闭式不判的机制：上限 4 缩掉 24 写那一段，几十个状态、单跑几秒，不罩新的崩溃状态
#[test]
fn a_narrowed_run_leaves_the_closed_form_unjudged_judges_every_block_and_names_the_expansion_in_its_line(
) {
    // 一次性库留着：只测缩过域那一趟闭式不判、每块照判（几十个状态），要从空库起跑
    let library = DisposableLibrary::new("closed-form-not-judged-on-a-narrowed-domain")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = library.join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了只开 CPU 的配置");
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let narrowed_expand_up_to_text = NARROWED_EXPAND_UP_TO.to_string();
    let _environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new(&narrowed_expand_up_to_text),
        ),
    ]);
    let pool = build_pool("closed-form-not-judged-on-a-narrowed-domain");
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2],
        "前提：新池新建文件流的段序列（只有写数 24 的那一段越过上限 4）"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽那一条");
    let closed_form_of_the_whole_domain =
        crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    let narrowed_limit = ExpandUpToWritesPerSegment(NARROWED_EXPAND_UP_TO);
    let closed_form_of_the_narrowed_domain = crash_state_count_with_torn_in_place_overwrites(
        &base,
        &writes,
        &segments,
        &|_segment_index, segment| narrowed_limit.expansion_of(segment),
    );
    assert!(
        closed_form_of_the_narrowed_domain < closed_form_of_the_whole_domain,
        "前提：缩过之后的域（{closed_form_of_the_narrowed_domain}）比全量域（{closed_form_of_the_whole_domain}）小"
    );
    let tally = enumerate_crash_states(
        CrashPointReplayItem {
            name: "new_pool_file_creation::closed_form_not_judged_when_narrowed_to_four_writes",
            setting_file: THIS_FILE,
        },
        &base,
        &writes,
        &segments,
        root_index,
        &file_content(),
    )
    .judged()
    .expect("守卫清掉了只录、只判与分片的变量：两段都在这一趟跑、交判的结果");
    assert_eq!(
        tally.states, closed_form_of_the_narrowed_domain,
        "每个崩溃块照判：枚举到的恰好是缩过之后那个域的每个状态"
    );
    assert_eq!(
        (tally.violations, tally.ignored_violations),
        (0, 0),
        "多版本 oracle 一条不红：{:?}",
        tally.first_violation
    );
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    assert_eq!(
        checker_violations, 0,
        "池级 checker 一条不红：{:?}",
        tally.checker_first_violation
    );
    let what = "closed_form_not_judged_on_a_narrowed_domain：用例自己的展开下的闭式";
    let judgement = assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form_of_the_whole_domain,
        what,
    );
    assert_eq!(
        judgement,
        ClosedFormStateCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: NARROWED_EXPAND_UP_TO,
            counted: tally.states.to_string(),
            closed_form: closed_form_of_the_whole_domain.to_string(),
        },
        "缩过域：闭式不判，交回这一趟的上限、实际展开的态数与全量域的闭式"
    );
    assert_eq!(
        judgement.line(what),
        Some(format!(
            "{CLOSED_FORM_NOT_JUDGED_LINE_PREFIX} expand_up_to={NARROWED_EXPAND_UP_TO} counted={} closed_form={closed_form_of_the_whole_domain} what={what}",
            tally.states
        )),
        "结果行写明缩过、展开上限与实际展开的态数"
    );
}
