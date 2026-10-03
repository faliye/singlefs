//! m3-formal-code-r3 攻方原型（只在冻结副本的拷贝上编、跑）。
//! k1：合法历史（随机生成、执行完成的）上，层 0 枚举域（每段都展开、原地覆写取三态）逐态判 RefFS 与 O1，
//! 按崩溃注入调用点同一种接法（`TranslatedHistory::judge_a_crash(crash_moment_in_segment(段), 读回)`，
//! O1 = `crash_recovery_disagreement(模型提交过的每一版, 持久的最新根, 读回)`）。
//! spans：只算段跨步的形状（不枚举）。
mod k4t1;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistorySeed,
    PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, root_identity_of_write, writes_and_segments_with_stream_indexes,
    CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::model::{crash_recovery_disagreement, ModelRootKey, ObservedReadBack};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::reference_filesystem_translation::{
    JointVerdictOfTheReferenceFileSystemAndTheModel, ReferenceFileSystemCrashMoment,
    ReferenceFileSystemTranslation, ReferenceFileSystemVote, TranslatedHistory,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::SharedStream;

fn weights_of(name: &str) -> GenerationWeights {
    match name {
        "broad" => GenerationWeights::BROAD,
        "rbf" => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        "reuse" => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        other => panic!("不认得的比重 {other}"),
    }
}

fn execution_of(width: &str) -> HistoryExecution {
    let device_width = match width {
        "4g" => HistoryDeviceWidth::FourGibibytes,
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        other => panic!("不认得的盘宽 {other}"),
    };
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

struct Prepared {
    history: GeneratedHistory,
    completed: bool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    stream_indexes: Vec<usize>,
    translated: TranslatedHistory,
    committed: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    after_mkfs: usize,
    after_last: usize,
    after_each: Vec<(StepPosition, usize)>,
    execution: HistoryExecution,
}

fn prepare(history: GeneratedHistory, execution: HistoryExecution) -> Prepared {
    let stream = SharedStream::retaining_contents();
    let mut committed: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let mut after_each = Vec::new();
    let mut after_last = 0usize;
    let mut translation = ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed.insert(key, file);
        }
        let n = stream.operation_count();
        after_last = n;
        after_each.push((observation.position, n));
        translation.note_an_observed_step(observation, n);
    });
    let completed = matches!(run.ending, HistoryEnding::Completed);
    let operations = stream.retained_operations();
    let geometry: FixedGeometry = execution.device_width.fixed_geometry();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&operations, &geometry);
    let translated = translation.translate(&operations, &geometry);
    assert_eq!(translated.segment_count(), segments.len());
    Prepared {
        history,
        completed,
        writes,
        segments,
        stream_indexes,
        translated,
        committed,
        after_mkfs: run.operations_written_by_make_filesystem,
        after_last,
        after_each,
        execution,
    }
}

fn step_of_stream_index(after_each: &[(StepPosition, usize)], stream_index: usize) -> Option<StepPosition> {
    after_each.iter().find(|(_, n)| *n > stream_index).map(|(p, _)| *p)
}

fn op_name(history: &GeneratedHistory, position: StepPosition) -> String {
    match position {
        StepPosition::StartingPoint => format!("start({:?})", history.starting_point),
        StepPosition::Operation(i) => format!("op{}:{:?}", i, history.operations[i].kind()),
    }
}

fn moment_name(history: &GeneratedHistory, moment: ReferenceFileSystemCrashMoment) -> String {
    match moment {
        ReferenceFileSystemCrashMoment::InsideStep(p) => op_name(history, p),
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep => "after-all".to_string(),
    }
}

fn candidate(p: &Prepared, segment: &[usize]) -> bool {
    segment
        .iter()
        .all(|w| (p.after_mkfs..p.after_last).contains(&p.stream_indexes[*w]))
}

fn read_back_name(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => format!("nofile@txg{}i{}", root.checkpoint_txg.0, root.instance.0),
        ObservedReadBack::FileRead { root, content } => format!(
            "file@txg{}i{}:len{}:b{:?}",
            root.checkpoint_txg.0,
            root.instance.0,
            content.len(),
            content.first()
        ),
        ObservedReadBack::Failed { what } => format!("failed:{}", what.chars().take(60).collect::<String>()),
    }
}

/// 段跨步：段里有写落在关段那一步之前的某一步。打印 (关段那一步, 段里写所在的步与种类)。
fn spans(p: &Prepared, verbose: bool) -> (usize, usize, usize) {
    let mut spanning = 0;
    let mut spanning_with_earlier_root = 0;
    let mut cand = 0;
    for (si, segment) in p.segments.iter().enumerate() {
        if !candidate(p, segment) {
            continue;
        }
        cand += 1;
        let moment = p.translated.crash_moment_in_segment(si);
        let closing_step = match moment {
            ReferenceFileSystemCrashMoment::InsideStep(s) => Some(s),
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep => None,
        };
        let mut steps: BTreeSet<StepPosition> = BTreeSet::new();
        let mut earlier_root = false;
        for w in segment {
            let s = step_of_stream_index(&p.after_each, p.stream_indexes[*w]);
            if let Some(s) = s {
                steps.insert(s);
                if Some(s) != closing_step && p.writes[*w].kind == StepKind::RootRecordFua {
                    earlier_root = true;
                }
            }
        }
        if steps.len() > 1 || (steps.len() == 1 && steps.iter().next().copied() != closing_step) {
            spanning += 1;
            if earlier_root {
                spanning_with_earlier_root += 1;
            }
            if verbose {
                let kinds: Vec<String> = segment
                    .iter()
                    .map(|w| {
                        let s = step_of_stream_index(&p.after_each, p.stream_indexes[*w]);
                        format!(
                            "{}:{:?}:d{}{}",
                            s.map_or("?".to_string(), |s| op_name(&p.history, s)),
                            p.writes[*w].kind,
                            p.writes[*w].device.0,
                            if p.writes[*w].is_force_unit_access { "F" } else { "" }
                        )
                    })
                    .collect();
                println!(
                    "SPAN seed={} seg={} closing={} writes=[{}]",
                    p.history.seed.0,
                    si,
                    moment_name(&p.history, moment),
                    kinds.join(" ")
                );
            }
        }
    }
    (cand, spanning, spanning_with_earlier_root)
}

#[derive(Default)]
struct Tally {
    histories: u64,
    completed: u64,
    skipped_over_cap: u64,
    states: u64,
    joint: BTreeMap<String, u64>,
    by_moment_kind: BTreeMap<String, u64>,
    examples: Vec<String>,
}

fn joint_key(vote: &ReferenceFileSystemVote, o1: bool) -> String {
    let dummy = DUMMY.with(|d| d.clone());
    let j = JointVerdictOfTheReferenceFileSystemAndTheModel::of(vote, if o1 { Some(&dummy) } else { None });
    match vote {
        ReferenceFileSystemVote::Abstained(a) => format!("{} [{}]", j.name(), a.name()),
        _ => j.name().to_string(),
    }
}

thread_local! {
    static DUMMY: singlefs_harness::model::ModelDisagreement = singlefs_harness::model::ModelDisagreement::new(
        singlefs_harness::model::ModelDisagreementAspect::ColdStartReadBack, String::new(), String::new());
}

fn k1_one(p: &Prepared, cap: u64, tally: &mut Tally, max_examples: usize) {
    let base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], p.execution.device_width.device_bytes());
    let cand: Vec<bool> = p.segments.iter().map(|s| candidate(p, s)).collect();
    let expansion = |si: usize, _s: &[usize]| {
        if cand[si] {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let count = layer0_state_count_with_torn_in_place_overwrites(&base, &p.writes, &p.segments, &expansion);
    if count > cap {
        tally.skipped_over_cap += 1;
        println!("SKIP seed={} states={}", p.history.seed.0, count);
        return;
    }
    let mut write_to_segment = vec![usize::MAX; p.writes.len()];
    for (si, s) in p.segments.iter().enumerate() {
        for w in s {
            write_to_segment[*w] = si;
        }
    }
    let roots: Vec<usize> = (0..p.writes.len()).filter(|w| p.writes[*w].kind == StepKind::RootRecordFua).collect();
    let judged_root = *roots.last().expect("有根写");
    let versions: Vec<PublishedVersion> = roots
        .iter()
        .filter_map(|w| {
            let (instance, checkpoint_txg) = root_identity_of_write(&p.writes[*w]);
            let key = model_root_key(instance, checkpoint_txg);
            p.committed.get(&key).and_then(|c| c.as_ref()).map(|c| PublishedVersion { instance, checkpoint_txg, content: c.to_vec() })
        })
        .collect();
    let n = p.writes.len();
    let mut observe = |image: &CrashImage<'_>, report: &RecoveryReport| {
        let first_withheld = (0..n).find(|w| !image.persisted[*w]);
        let moment = match first_withheld {
            Some(w) => p.translated.crash_moment_in_segment(write_to_segment[w]),
            None => ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        };
        let read_back = observed_read_back_after_a_crash(report);
        let newest = newest_persisted_root(&p.writes, &image.persisted[..n]).map(|(t, i)| model_root_key(i, t));
        let o1 = crash_recovery_disagreement(&p.committed, newest, &read_back);
        let judgement = p.translated.judge_a_crash(moment, &read_back);
        tally.states += 1;
        let key = joint_key(&judgement.vote, o1.is_some());
        *tally.joint.entry(key.clone()).or_insert(0) += 1;
        *tally.by_moment_kind.entry(format!("{} | {}", moment_name_kind(&p.history, moment), key)).or_insert(0) += 1;
        let interesting = matches!(judgement.vote, ReferenceFileSystemVote::Red { .. }) != o1.is_some()
            || matches!(judgement.vote, ReferenceFileSystemVote::Abstained(_)) && o1.is_some();
        if interesting && tally.examples.len() < max_examples {
            let withheld_step = first_withheld.and_then(|w| step_of_stream_index(&p.after_each, p.stream_indexes[w]));
            let withheld: Vec<String> = (0..n)
                .filter(|w| !image.persisted[*w] && write_to_segment[*w] == first_withheld.map_or(usize::MAX, |f| write_to_segment[f]))
                .map(|w| format!("w{}:{:?}:d{}", w, p.writes[w].kind, p.writes[w].device.0))
                .collect();
            tally.examples.push(format!(
                "EX seed={} key={} seg={} moment={} first_withheld_step={} withheld=[{}] read_back={} newest={:?} vote={:?} o1={:?}",
                p.history.seed.0,
                key,
                first_withheld.map_or(usize::MAX, |w| write_to_segment[w]),
                moment_name(&p.history, moment),
                withheld_step.map_or("none".to_string(), |s| op_name(&p.history, s)),
                withheld.join(" "),
                read_back_name(&read_back),
                newest,
                judgement.vote,
                o1.map(|d| format!("{:?}: {} / {}", d.aspect, d.model_answer, d.implementation_answer)),
            ));
        }
    };
    let _ = enumerate_layer0_selecting_versions_observing_each_state(
        &base,
        &p.writes,
        &p.segments,
        judged_root,
        &versions,
        &|si, _| cand[si],
        &mut observe,
    );
}

fn moment_name_kind(history: &GeneratedHistory, moment: ReferenceFileSystemCrashMoment) -> String {
    match moment {
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::StartingPoint) => "start".to_string(),
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(i)) => format!("{:?}", history.operations[i].kind()),
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep => "after-all".to_string(),
    }
}

fn describe_history(p: &Prepared) -> String {
    let ops: Vec<String> = p.history.operations.iter().map(|o| format!("{o:?}")).collect();
    format!("start={:?} ops=[{}] completed={}", p.history.starting_point, ops.join(", "), p.completed)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    match mode {
        // spans <weights> <width> <ops> <seed_from> <seed_to> [verbose]
        "spans" => {
            let weights = weights_of(&args[2]);
            let execution = execution_of(&args[3]);
            let ops: usize = args[4].parse().unwrap();
            let from: u64 = args[5].parse().unwrap();
            let to: u64 = args[6].parse().unwrap();
            let verbose = args.get(7).is_some();
            let (mut c, mut s, mut r, mut h, mut done) = (0, 0, 0, 0, 0);
            for seed in from..to {
                let p = prepare(generate_history_with_weights(HistorySeed(seed), ops, &weights), execution);
                h += 1;
                if !p.completed {
                    continue;
                }
                done += 1;
                let (a, b, d) = spans(&p, verbose);
                if verbose && b > 0 {
                    println!("HIST seed={} {}", seed, describe_history(&p));
                }
                c += a;
                s += b;
                r += d;
            }
            println!("name=spans histories={h} completed={done} candidate_segments={c} spanning={s} spanning_with_an_earlier_root_write={r}");
        }
        // k1 <weights> <width> <ops> <seed_from> <seed_to> <cap_per_history> <max_examples>
        "k1" => {
            let weights = weights_of(&args[2]);
            let execution = execution_of(&args[3]);
            let ops: usize = args[4].parse().unwrap();
            let from: u64 = args[5].parse().unwrap();
            let to: u64 = args[6].parse().unwrap();
            let cap: u64 = args[7].parse().unwrap();
            let max_examples: usize = args[8].parse().unwrap();
            let mut tally = Tally::default();
            for seed in from..to {
                let p = prepare(generate_history_with_weights(HistorySeed(seed), ops, &weights), execution);
                tally.histories += 1;
                if !p.completed {
                    continue;
                }
                tally.completed += 1;
                let before = tally.examples.len();
                k1_one(&p, cap, &mut tally, max_examples);
                if tally.examples.len() > before {
                    println!("HIST seed={} {}", seed, describe_history(&p));
                }
            }
            for e in &tally.examples {
                println!("{e}");
            }
            for (k, v) in &tally.by_moment_kind {
                println!("BYMOMENT {k} count={v}");
            }
            for (k, v) in &tally.joint {
                println!("JOINT {k} count={v}");
            }
            println!(
                "name=k1 histories={} completed={} skipped_over_cap={} states={}",
                tally.histories, tally.completed, tally.skipped_over_cap, tally.states
            );
        }
        // segs <weights> <width> <ops> <seed>：每个候选段的关段步、写数与种类
        "segs" => {
            let weights = weights_of(&args[2]);
            let execution = execution_of(&args[3]);
            let ops: usize = args[4].parse().unwrap();
            let seed: u64 = args[5].parse().unwrap();
            let p = prepare(generate_history_with_weights(HistorySeed(seed), ops, &weights), execution);
            println!("HIST seed={} {}", seed, describe_history(&p));
            for (si, segment) in p.segments.iter().enumerate() {
                if !candidate(&p, segment) {
                    continue;
                }
                let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
                for w in segment {
                    *kinds.entry(format!("{:?}:d{}", p.writes[*w].kind, p.writes[*w].device.0)).or_insert(0) += 1;
                }
                println!("SEG {} closing={} writes={} kinds={:?}", si, moment_name(&p.history, p.translated.crash_moment_in_segment(si)), segment.len(), kinds);
            }
        }
        "k4t1" => k4t1::run(),
        // inject <weights> <width> <ops> <seed_from> <seed_to> <crash_points_per_history>：崩溃注入调用点原样（它自己的抽样），只汇总 RefFS ∧ O1 两截的票
        "inject" => {
            use singlefs_checker_tier::crash_injection::{inject_crashes_into_history, CrashInjectionTally, CrashPointDraw};
            let weights = weights_of(&args[2]);
            let execution = execution_of(&args[3]);
            let ops: usize = args[4].parse().unwrap();
            let from: u64 = args[5].parse().unwrap();
            let to: u64 = args[6].parse().unwrap();
            let per: usize = args[7].parse().unwrap();
            let mut tally = CrashInjectionTally::default();
            let mut findings = 0usize;
            for seed in from..to {
                let injection = inject_crashes_into_history(
                    &generate_history_with_weights(HistorySeed(seed), ops, &weights),
                    execution,
                    CrashPointDraw::Sampled { crash_points_per_history: per },
                    None,
                );
                tally.absorb(&injection.tally);
                for f in &injection.new_findings {
                    findings += 1;
                    if findings <= 10 {
                        let text = format!("{:?}", f.observation);
                        println!("FINDING seed={} stage={:?} read_back={} obs={}", seed, f.stage, f.read_back.chars().take(200).collect::<String>(), text.chars().take(600).collect::<String>());
                    }
                }
            }
            for (k, v) in &tally.reference_filesystem_and_model_verdicts_on_crash_images {
                println!("CRASH_IMAGE {} count={}", k.name(), v);
            }
            for (k, v) in &tally.reference_filesystem_and_model_verdicts_on_second_crashes {
                println!("SECOND_CRASH {} count={}", k.name(), v);
            }
            for (k, v) in &tally.reference_filesystem_abstentions {
                println!("ABSTAIN {} count={}", k, v);
            }
            println!("name=inject histories={} crash_points={} second_crash_points={} new_findings={}", to - from, tally.crash_points, tally.second_crash_points, findings);
        }
        _ => panic!("用法：r3probe spans|k1|segs|k4t1 …"),
    }
}
