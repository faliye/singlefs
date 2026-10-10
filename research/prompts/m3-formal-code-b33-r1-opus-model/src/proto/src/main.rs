//! 第 33 批代码轮攻方原型（P2「F1、33-c 两支部分没判在合法历史上会不会多报」）：
//! mkfs 之后只走树表 0 条那一族的操作（可写挂载 M、零单元发布 E、冷启动恢复 C、每盘最新系统配置槽读回全零的可写挂载 Z），
//! 末尾可选一次第一个文件版本 F；整条录制流按层 0 的枚举域（`enumerate_layer0_selecting_versions_observing_each_state`，
//! 每一段都展开、原地覆写取三态）取崩溃状态，每个状态在观察者里再跑一遍池级 checker，按 I-5.4 的判定分类计数。
//! 每段历史开跑前用 `layer0_state_count_with_torn_in_place_overwrites` 算全量状态数：超过 1_000_000 的那段不跑，
//! 合计越过 10_000_000 就停（派发与定义的上限）。打印行以 `B33P` 开头。
use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, SeededRandomSource,
};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const STATES_PER_HISTORY_AT_MOST: u64 = 1_000_000;
const STATES_IN_TOTAL_AT_MOST: u64 = 10_000_000;

fn operation_of(letter: char, seed: u64) -> HistoryOperation {
    match letter {
        'M' => HistoryOperation::CloseAndMountWritable,
        'E' => HistoryOperation::PublishWithoutUnits,
        'C' => HistoryOperation::ColdStartRecover,
        'Z' => HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
        'F' => HistoryOperation::PublishFirstFile(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: seed },
            fill_seed: seed,
        }),
        other => panic!("不认得的操作字母 {other}"),
    }
}

/// 写死的几段（33-c 用例那一形与它的近邻），加按种子抽的树表 0 条一族（长度 3–8，不带 F：带 F 的一段全量约 1.68 × 10⁷ 个状态，超过一段 10⁶ 的上限）。
/// 环境变量：B33P_SCRIPTED=0 不跑写死的几段；B33P_RANDOM_FROM、B33P_RANDOM_HISTORIES 取种子区间 [from, from + count)（分批跑）。
fn histories(random_from: u64, random_count: u64, scripted_too: bool) -> Vec<String> {
    let mut scripted: Vec<String> = if scripted_too {
        ["MM", "MMF", "MME", "MMEF", "MMM", "MMMF", "MMEME", "MMEMEF", "MZM", "MZMF", "MCME", "MMZF"]
            .iter()
            .map(|text| (*text).to_owned())
            .collect()
    } else {
        Vec::new()
    };
    for seed in random_from..random_from + random_count {
        let mut source = SeededRandomSource::from_seed(0xB33_0000 + seed);
        let length = 3 + source.below(6);
        let mut text = String::from("M");
        for _ in 1..length {
            text.push(['M', 'E', 'E', 'C', 'Z'][usize::try_from(source.below(5)).expect("小")]);
        }
        scripted.push(text);
    }
    scripted
}

#[derive(Default)]
struct Counts {
    states: u64,
    holds: u64,
    not_applicable: u64,
    violated: u64,
    partially_f1: u64,
    partially_33c: u64,
    partially_other: u64,
    other_invariants_violated_states: u64,
    first_partial: Option<String>,
    first_violation_elsewhere: Option<String>,
}

fn classify(verdicts: &[(&'static str, InvariantVerdict)], counts: &mut Counts, label: &str) {
    counts.states += 1;
    let violated_elsewhere: Vec<&str> = verdicts
        .iter()
        .filter(|(name, verdict)| *name != "I-5.4" && matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    if !violated_elsewhere.is_empty() {
        counts.other_invariants_violated_states += 1;
        counts
            .first_violation_elsewhere
            .get_or_insert_with(|| format!("{label}: {violated_elsewhere:?}"));
    }
    let i54 = &verdicts.iter().find(|(name, _)| *name == "I-5.4").expect("I-5.4").1;
    match i54 {
        InvariantVerdict::Holds => counts.holds += 1,
        InvariantVerdict::NotApplicable(_) => counts.not_applicable += 1,
        InvariantVerdict::Violated(_) => counts.violated += 1,
        InvariantVerdict::PartiallyJudged(detail) => {
            if detail.contains("树表 0 条、分配记录树由根记录直接持有") {
                counts.partially_33c += 1;
            } else if detail.contains("树表指针非零而树表读不出") {
                counts.partially_f1 += 1;
            } else {
                counts.partially_other += 1;
            }
            counts
                .first_partial
                .get_or_insert_with(|| format!("{label}: {detail}"));
        }
    }
}

fn main() {
    let random_count: u64 = std::env::var("B33P_RANDOM_HISTORIES")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(24);
    let random_from: u64 = std::env::var("B33P_RANDOM_FROM")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(0);
    let scripted_too = std::env::var("B33P_SCRIPTED").map_or(true, |text| text != "0");
    let execution = HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let geometry = execution.device_width.fixed_geometry();
    let mut total_states = 0u64;
    let mut totals = Counts::default();
    let mut skipped: BTreeMap<String, u64> = BTreeMap::new();
    for (index, letters) in histories(random_from, random_count, scripted_too).iter().enumerate() {
        let seed = random_from * 1000 + u64::try_from(index).expect("小");
        let history = GeneratedHistory {
            seed: HistorySeed(seed),
            starting_point: HistoryStartingPoint::AfterMakeFilesystem,
            operations: letters.chars().map(|letter| operation_of(letter, seed)).collect(),
        };
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(&history, execution, &stream, &mut |_| {});
        let outcomes: Vec<String> = run
            .outcomes
            .iter()
            .map(|outcome| format!("{outcome:?}").chars().take(24).collect())
            .collect();
        let operations = stream.retained_operations();
        let mkfs = run.operations_written_by_make_filesystem;
        let mut base = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        base.apply(&operations[..mkfs]);
        let (writes, segments) = writes_and_segments(&operations[mkfs..], &geometry);
        let Some(judged_root_index) = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua) else {
            println!("B33P history={letters} ending={:?} writes=0 skipped=no_root_write", run.ending);
            continue;
        };
        let full_states =
            layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
        if full_states > STATES_PER_HISTORY_AT_MOST || total_states + full_states > STATES_IN_TOTAL_AT_MOST {
            println!("B33P history={letters} full_states={full_states} skipped=over_the_cap");
            skipped.insert(letters.clone(), full_states);
            continue;
        }
        total_states += full_states;
        let mut counts = Counts::default();
        let tally = enumerate_layer0_selecting_versions_observing_each_state(
            &base,
            &writes,
            &segments,
            judged_root_index,
            &[],
            &|_segment_index, _segment| true,
            &mut |image, _report| {
                let verdicts = check_pool_image(image);
                classify(&verdicts, &mut counts, letters);
            },
        );
        println!(
            "B33P history={letters} ending={:?} outcomes={outcomes:?} segments={} full_states={full_states} enumerated={} observed={} i54_holds={} i54_not_applicable={} i54_violated={} i54_partially_f1={} i54_partially_33c={} i54_partially_other={} states_with_other_invariants_violated={} first_partial={:?} first_violation_elsewhere={:?}",
            format!("{:?}", run.ending).chars().take(40).collect::<String>(),
            segments.len(),
            tally.states,
            counts.states,
            counts.holds,
            counts.not_applicable,
            counts.violated,
            counts.partially_f1,
            counts.partially_33c,
            counts.partially_other,
            counts.other_invariants_violated_states,
            counts.first_partial,
            counts.first_violation_elsewhere
        );
        totals.states += counts.states;
        totals.holds += counts.holds;
        totals.not_applicable += counts.not_applicable;
        totals.violated += counts.violated;
        totals.partially_f1 += counts.partially_f1;
        totals.partially_33c += counts.partially_33c;
        totals.partially_other += counts.partially_other;
        totals.other_invariants_violated_states += counts.other_invariants_violated_states;
    }
    println!(
        "B33P total states={} full_states_sum={total_states} i54_holds={} i54_not_applicable={} i54_violated={} i54_partially_f1={} i54_partially_33c={} i54_partially_other={} states_with_other_invariants_violated={} skipped={skipped:?}",
        totals.states, totals.holds, totals.not_applicable, totals.violated, totals.partially_f1,
        totals.partially_33c, totals.partially_other, totals.other_invariants_violated_states
    );
}
