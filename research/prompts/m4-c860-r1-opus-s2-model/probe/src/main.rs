//! C860 攻方第二次抽样的探针（只在攻方副本里）：四种「删除中」认法在随机历史每一步、与删除步每个崩溃状态上各判 I-1.13 什么。
//! 用法：
//!   c860-probe histories <比重名> <首个种子> <种子数> <步数>
//!   c860-probe crash <比重名> <种子> <步数> <要枚举的操作种类> <最多几步>
#![allow(dead_code, unused_imports, reason = "攻方副本里的探针：tests/common 整份带进来")]
#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
use std::collections::BTreeMap;
use std::sync::Mutex;

use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::{c860_probe_begin, c860_probe_take, check_pool_image, C860_GLOBAL};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_library::StageOutcome;
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_versions, CrashPointReplayItem,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::fault_injection::SharedFaultPlan;
use singlefs_harness::history::{
    execute_history_handing_out_every_step_image, execute_history_with,
    generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
    HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    root_identity_of_write, writes_and_segments, MemoryPool, PublishedVersion,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn weights_named(name: &str) -> &'static GenerationWeights {
    match name {
        "broad" => &GenerationWeights::BROAD,
        "deleting" => &GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
        "reuse" => &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        "rollback" => &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        "orphans" => &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
        "snapshot" => &GenerationWeights::SNAPSHOT_OPERATIONS,
        "compaction" => &GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
        "broadcompaction" => &GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED,
        "byinode" => &GenerationWeights::WRITES_BY_INODE_NUMBER,
        "truncation" => &GenerationWeights::TRUNCATION,
        other => panic!("没有这个比重名：{other}"),
    }
}

fn execution() -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

/// 一张镜像上四种认法各判什么：探针没记（I-1.13 在比表之前就交了判定）时四种都取池级 checker 的那一个判定类。
fn arms_on(image: &dyn singlefs_checker::image::ImageReader) -> (Vec<(String, String)>, String) {
    c860_probe_begin();
    let verdicts = check_pool_image(image);
    let probe = c860_probe_take();
    let official = verdicts
        .iter()
        .find(|(invariant, _)| *invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX)
        .map(|(_, verdict)| verdict.clone())
        .expect("I-1.13 在清单里");
    if probe.is_empty() {
        let class = match &official {
            InvariantVerdict::NotApplicable(reason) => format!("na:{reason}"),
            other => format!("{other:?}"),
        };
        return (
            ["jia", "yi", "bing", "ding"].iter().map(|arm| (arm.to_string(), class.clone())).collect(),
            String::new(),
        );
    }
    let mut meta = String::new();
    let mut arms = Vec::new();
    for (arm, holds, detail) in probe {
        if arm == "meta" {
            meta = detail;
            continue;
        }
        arms.push((arm.to_string(), if holds { "holds".to_string() } else { format!("violated:{detail}") }));
    }
    (arms, meta)
}

fn threads() -> usize {
    std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|text| text.parse().ok()).unwrap_or(4)
}

fn histories(arguments: &[String]) {
    let weights_name = arguments[0].clone();
    let first_seed: u64 = arguments[1].parse().expect("首个种子");
    let seed_count: u64 = arguments[2].parse().expect("种子数");
    let operation_count: usize = arguments[3].parse().expect("步数");
    let next = Mutex::new(first_seed);
    let tally: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
    let findings: Mutex<Vec<String>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads() {
            scope.spawn(|| loop {
                let seed = {
                    let mut next = next.lock().expect("锁");
                    if *next >= first_seed + seed_count {
                        break;
                    }
                    *next += 1;
                    *next - 1
                };
                let history = generate_history_with_weights(
                    HistorySeed(seed),
                    operation_count,
                    weights_named(&weights_name),
                );
                let _run = execute_history_with(&history, execution(), &SharedStream::new(), &mut |observation| {
                    let (arms, meta) = arms_on(observation.image);
                    let deleting = if meta.is_empty() || meta.starts_with("being_deleted=0 ") { "none_deleting" } else { "some_deleting" };
                    let kind = observation.operation.map(|operation| format!("{:?}", operation.kind())).unwrap_or_else(|| "start".to_string());
                    let mut tally = tally.lock().expect("锁");
                    *tally.entry(format!("states {deleting}")).or_insert(0) += 1;
                    if meta.contains("scan_only=") && !meta.contains("scan_only=0") {
                        *tally.entry("states_with_a_scan_only_object_being_deleted".to_string()).or_insert(0) += 1;
                    }
                    if meta.contains("walk_tombstoned_nonzero_nlink=") && !meta.contains("walk_tombstoned_nonzero_nlink=0") {
                        *tally.entry("states_with_a_tombstoned_walked_record_of_nonzero_nlink".to_string()).or_insert(0) += 1;
                    }
                    let classes: Vec<String> = arms.iter().map(|(_, class)| if class.starts_with("violated") { "violated".to_string() } else if class.starts_with("na:") { "na".to_string() } else { class.clone() }).collect();
                    for ((arm, _), class) in arms.iter().zip(&classes) {
                        *tally.entry(format!("arm {arm} {deleting} {class}")).or_insert(0) += 1;
                    }
                    let differ = classes.iter().any(|class| *class != classes[0]);
                    let any_violated = classes.iter().any(|class| class == "violated");
                    if differ || any_violated {
                        let mut findings = findings.lock().expect("锁");
                        if findings.len() < 200 {
                            findings.push(format!(
                                "FINDING weights={weights_name} seed={seed} position={:?} kind={kind} meta=[{meta}] arms={arms:?}",
                                observation.position
                            ));
                        }
                    }
                });
            });
        }
    });
    for (key, count) in tally.into_inner().expect("锁") {
        println!("TALLY weights={weights_name} first_seed={first_seed} seeds={seed_count} ops={operation_count} {key} = {count}");
    }
    for finding in findings.into_inner().expect("锁") {
        println!("{finding}");
    }
}

fn crash(arguments: &[String]) {
    let weights_name = arguments[0].clone();
    let seed: u64 = arguments[1].parse().expect("种子");
    let operation_count: usize = arguments[2].parse().expect("步数");
    let wanted_kind = arguments[3].clone();
    let at_most: usize = arguments[4].parse().expect("最多几步");
    let state_cap: u64 = arguments.get(5).map(|text| text.parse().expect("状态上限")).unwrap_or(1_000_000);
    let history = generate_history_with_weights(HistorySeed(seed), operation_count, weights_named(&weights_name));
    let stream = SharedStream::retaining_contents();
    let geometry = execution().device_width.fixed_geometry();
    let mut step_images: Vec<(String, usize, MemoryPool)> = Vec::new();
    let _run = execute_history_handing_out_every_step_image(
        &history,
        execution(),
        &stream,
        &SharedFaultPlan::unarmed(geometry),
        &mut |_observation| {},
        &mut |step_image| {
            step_images.push((
                step_image.operation_kind.map(|kind| format!("{kind:?}")).unwrap_or_else(|| "start".to_string()),
                step_image.operations_recorded,
                step_image.image.clone(),
            ));
        },
    );
    let operations = stream.retained_operations();
    let kinds: Vec<&str> = step_images.iter().map(|(kind, _, _)| kind.as_str()).collect();
    println!("CRASH_HISTORY weights={weights_name} seed={seed} steps={} kinds={kinds:?}", step_images.len());
    let mut done = 0usize;
    let mut total_states = 0u64;
    for index in 1..step_images.len() {
        let (kind, recorded_after, _) = &step_images[index];
        if !(wanted_kind == "any" || kind == &wanted_kind) || done >= at_most {
            continue;
        }
        let (_, recorded_before, base) = &step_images[index - 1];
        let (writes, segments) = writes_and_segments(&operations[*recorded_before..*recorded_after], &geometry);
        let Some(root_index) = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua) else {
            println!("CRASH_STEP seed={seed} step={index} kind={kind} no_root_write");
            continue;
        };
        let states = crash_state_count_with_torn_in_place_overwrites(base, &writes, &segments, &|_, _| {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        });
        if states > state_cap || total_states + states > 4_000_000 {
            println!("CRASH_STEP seed={seed} step={index} kind={kind} states={states} skipped_over_the_cap");
            continue;
        }
        total_states += states;
        done += 1;
        let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
        let versions = [PublishedVersion { instance, checkpoint_txg, content: Vec::new() }];
        *C860_GLOBAL.lock().expect("锁") = Some((BTreeMap::new(), BTreeMap::new()));
        let name = format!("c860_probe_{weights_name}_{seed}_{index}");
        let started = std::time::Instant::now();
        let outcome = enumerate_crash_states_versions(
            CrashPointReplayItem { name: &name, setting_file: "crates/c860-probe/src/main.rs" },
            base,
            &writes,
            &segments,
            root_index,
            &versions,
        );
        let (counts, first) = C860_GLOBAL.lock().expect("锁").take().expect("开着");
        match outcome {
            StageOutcome::Judged(tally) => {
                let key = SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX;
                println!(
                    "CRASH_STEP seed={seed} step={index} kind={kind} writes={} segments={} states={states} tally_states={} official_jia_evaluated={:?} official_jia_violated={:?} official_jia_not_applicable={:?} seconds={:.1}",
                    writes.len(), segments.len(), tally.states,
                    tally.checker_evaluated_states.get(key), tally.checker_violated_states.get(key),
                    tally.checker_not_applicable_states.get(key), started.elapsed().as_secs_f64()
                );
                if let Some(first) = tally.checker_first_violation.get(key) {
                    println!("CRASH_STEP_OFFICIAL_FIRST_VIOLATION {first}");
                }
            }
            StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
                println!("CRASH_STEP seed={seed} step={index} kind={kind} states={states} not_judged_here");
            }
        }
        for (key, count) in &counts {
            println!("CRASH_ARM_CALLS seed={seed} step={index} {key} = {count}");
        }
        for (key, detail) in &first {
            println!("CRASH_ARM_FIRST_VIOLATION seed={seed} step={index} {key} :: {detail}");
        }
    }
    println!("CRASH_TOTAL seed={seed} steps_enumerated={done} states={total_states}");
}

/// 只数：每段历史里指定种类的每一步、它那段写流的崩溃状态数（不枚举）。
fn scan(arguments: &[String]) {
    let weights_name = arguments[0].clone();
    let first_seed: u64 = arguments[1].parse().expect("首个种子");
    let seed_count: u64 = arguments[2].parse().expect("种子数");
    let operation_count: usize = arguments[3].parse().expect("步数");
    let wanted_kind = arguments[4].clone();
    let geometry = execution().device_width.fixed_geometry();
    for seed in first_seed..first_seed + seed_count {
        let history = generate_history_with_weights(HistorySeed(seed), operation_count, weights_named(&weights_name));
        let stream = SharedStream::retaining_contents();
        let mut step_images: Vec<(String, usize, MemoryPool)> = Vec::new();
        let _run = execute_history_handing_out_every_step_image(
            &history, execution(), &stream, &SharedFaultPlan::unarmed(geometry), &mut |_observation| {},
            &mut |step_image| {
                step_images.push((
                    step_image.operation_kind.map(|kind| format!("{kind:?}")).unwrap_or_else(|| "start".to_string()),
                    step_image.operations_recorded,
                    step_image.image.clone(),
                ));
            },
        );
        let operations = stream.retained_operations();
        for index in 1..step_images.len() {
            let (kind, recorded_after, _) = &step_images[index];
            if !(wanted_kind == "any" || kind == &wanted_kind) {
                continue;
            }
            let (_, recorded_before, base) = &step_images[index - 1];
            let (writes, segments) = writes_and_segments(&operations[*recorded_before..*recorded_after], &geometry);
            let states = crash_state_count_with_torn_in_place_overwrites(base, &writes, &segments, &|_, _| {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            });
            let previous: Vec<&str> = step_images[..index].iter().map(|(kind, _, _)| kind.as_str()).collect();
            println!("SCAN weights={weights_name} seed={seed} step={index} kind={kind} writes={} states={states} before={previous:?}", writes.len());
        }
    }
}

/// 固定场景（tests/common 的搭法，镜像放 TMPDIR）：一段删除写流的崩溃状态逐个枚举、四种认法各判。
fn fixed(arguments: &[String]) {
    use common_deletion_in_progress::{
        pool_with_a_three_unit_file, pool_with_new_inodes, publish_the_intent, publish_the_intent_of,
        publish_the_next_step_in_batches_of_one, roll_back_to, unlink_to_the_end,
    };
    let scenario = arguments[0].clone();
    let state_cap: u64 = arguments.get(1).map(|text| text.parse().expect("状态上限")).unwrap_or(200_000);
    let share = arguments.get(2).cloned();
    let (pool, operations_before, base) = match scenario.as_str() {
        "right_leaf_retired" | "left_absorbs_right" | "right_leaf_intent_only" => {
            let mut pool = pool_with_new_inodes(&format!("c860-crash-{scenario}"), 233);
            let before = pool.retained_operations().len();
            let base = pool.memory_pool();
            match scenario.as_str() {
                "right_leaf_retired" => { unlink_to_the_end(&mut pool, 234); }
                "left_absorbs_right" => { unlink_to_the_end(&mut pool, 2); }
                _ => { publish_the_intent_of(&mut pool, 234); }
            }
            (pool, before, base)
        }
        "three_unit_batches_of_one" => {
            let mut pool = pool_with_a_three_unit_file("c860-crash-three");
            let before = pool.retained_operations().len();
            let base = pool.memory_pool();
            publish_the_intent(&mut pool);
            while !pool.output.deleted_inodes.is_empty() {
                publish_the_next_step_in_batches_of_one(&mut pool);
            }
            (pool, before, base)
        }
        "rolled_back_to_the_intent_then_continued" => {
            let mut pool = pool_with_a_three_unit_file("c860-crash-rollback");
            let intent = publish_the_intent(&mut pool);
            while !pool.output.deleted_inodes.is_empty() {
                publish_the_next_step_in_batches_of_one(&mut pool);
            }
            let before = pool.retained_operations().len();
            let base = pool.memory_pool();
            roll_back_to(&mut pool, &intent);
            while !pool.output.deleted_inodes.is_empty() {
                publish_the_next_step_in_batches_of_one(&mut pool);
            }
            (pool, before, base)
        }
        other => panic!("没有这个场景：{other}"),
    };
    let operations = pool.retained_operations();
    let geometry = common::geometry();
    let (writes, segments) = writes_and_segments(&operations[operations_before..], &geometry);
    let states = crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &|_, _| {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    });
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("FIXED scenario={scenario} writes={} segments={} segment_sizes={sizes:?} states={states}", writes.len(), segments.len());
    if states > state_cap {
        println!("FIXED scenario={scenario} skipped_over_the_cap cap={state_cap}");
        return;
    }
    if let Some(share) = &share {
        // 分片：照崩溃放量的读法，这一趟只判这一份块。
        unsafe { std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_SHARE", share) };
    }
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("有根槽写");
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion { instance, checkpoint_txg, content: Vec::new() }];
    *C860_GLOBAL.lock().expect("锁") = Some((BTreeMap::new(), BTreeMap::new()));
    let name = format!("c860_probe_fixed_{scenario}");
    let started = std::time::Instant::now();
    let outcome = enumerate_crash_states_versions(
        CrashPointReplayItem { name: &name, setting_file: "crates/c860-probe/src/main.rs" },
        &base, &writes, &segments, root_index, &versions,
    );
    let (counts, first) = C860_GLOBAL.lock().expect("锁").take().expect("开着");
    let key = SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX;
    match outcome {
        StageOutcome::Judged(tally) => println!(
            "FIXED scenario={scenario} judged tally_states={} official_jia_evaluated={:?} official_jia_violated={:?} official_jia_not_applicable={:?} seconds={:.1}",
            tally.states, tally.checker_evaluated_states.get(key), tally.checker_violated_states.get(key),
            tally.checker_not_applicable_states.get(key), started.elapsed().as_secs_f64()
        ),
        StageOutcome::JudgedMyShareOnly { blocks_judged_here, blocks_left_to_other_hosts } => println!(
            "FIXED scenario={scenario} share={share:?} blocks_judged_here={blocks_judged_here} blocks_left={blocks_left_to_other_hosts} seconds={:.1}",
            started.elapsed().as_secs_f64()
        ),
        StageOutcome::RecordedOnly => println!("FIXED scenario={scenario} recorded_only"),
    }
    for (key, count) in &counts {
        println!("FIXED_ARM_CALLS scenario={scenario} {key} = {count}");
    }
    for (key, detail) in &first {
        println!("FIXED_ARM_FIRST_VIOLATION scenario={scenario} {key} :: {detail}");
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("histories") => histories(&arguments[2..]),
        Some("crash") => crash(&arguments[2..]),
        Some("scan") => scan(&arguments[2..]),
        Some("fixed") => fixed(&arguments[2..]),
        _ => panic!("用法见文件头"),
    }
}
