//! 草稿探针（不进仓）：故障注入快档 24 段 × 4 个注入点，在 CPU 池级 checker 上就地判；归成新发现的那几条逐条打出被吞的写、
//! 被吞之后同一槽上的写、插回去那张盘与设备不说谎那一趟的盘差在哪几次写，以及三张盘（注入之后、插回去、不说谎）各自的违例。
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault,
    inject_faults_into_history_handing_out_the_images, FailureAfterAFault, ImagesAfterAFault,
    InjectedFault, RecordedStreamOfAnImage, SharedFaultPlan,
};
use singlefs_harness::history::{
    execute_history_handing_out_every_step_image, generate_history_with_weights,
    GenerationWeights, HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RetainedOperation, SharedStream};

const SLOT: u64 = 16384;

const fn unchecked() -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn image_of(operations: &[RetainedOperation], prefix: usize) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        HistoryDeviceWidth::FourGibibytes.device_bytes(),
    );
    pool.apply(&operations[..prefix]);
    pool
}

fn worker_threads() -> usize {
    std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1)
}

fn judge(images: &ImagesAfterAFault) -> Vec<FailureAfterAFault> {
    let ops = &images.image_after_the_fault.operations;
    let step_violations = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step_image| violations_of(&image_of(ops, step_image.operations_recorded)))
        .collect();
    let after = violations_of(&image_of(ops, ops.len()));
    let still = images.image_with_the_swallowed_writes_landed.as_ref().map_or_else(
        Default::default,
        |landed| {
            let swallowed_slot = landed
                .operations
                .iter()
                .zip(ops.iter())
                .position(|(l, r)| !same(l, r))
                .map(|index| landed.operations[index].operation.offset.0 / SLOT);
            violations_of(&image_of(&landed.operations, landed.operations.len()))
                .into_iter()
                .filter(|(invariant, detail)| {
                    !(std::env::var("PROBE_CLASSIFIER_ACCEPTS_I315_ON_THE_SWALLOWED_SLOT").is_ok()
                        && *invariant == "I-3.15"
                        && swallowed_slot.is_some_and(|slot| detail.contains(&format!("槽 {slot} 的分配记录"))))
                })
                .map(|(invariant, _)| invariant)
                .collect()
        },
    );
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after, still)
}

fn describe(geometry: &FixedGeometry, index: usize, op: &RetainedOperation) -> String {
    let o = &op.operation;
    let class = op.contents.as_ref().and_then(|c| c.get(6).copied());
    let tree = op
        .contents
        .as_ref()
        .filter(|c| c.len() >= 51 && c[6] == 2)
        .map(|c| (u64::from_le_bytes(c[42..50].try_into().unwrap()), c[50]));
    format!(
        "#{index} 盘{} {:?} {} 槽{} 偏移{} 长{} 码={class:?} 码2的(树,层)={tree:?} 哈希={:016x}",
        o.device.0,
        o.kind,
        geometry.classify(o).name(),
        o.offset.0 / SLOT,
        o.offset.0,
        o.length,
        o.content_hash
    )
}

fn same(left: &RetainedOperation, right: &RetainedOperation) -> bool {
    left.operation == right.operation && left.contents == right.contents
}

fn compare(geometry: &FixedGeometry, name: &str, left: &[RetainedOperation], right: &[RetainedOperation]) {
    let first = left.iter().zip(right).position(|(l, r)| !same(l, r));
    let differing = left.iter().zip(right).filter(|(l, r)| !same(l, r)).count();
    println!(
        "  COMPARE {name}: 左 {} 步、右 {} 步；第一处不同 {first:?}；同下标不同的步数 {differing}",
        left.len(),
        right.len()
    );
    if let Some(start) = first {
        let end = (start + 12).min(left.len().max(right.len()));
        for index in start..end {
            let l = left.get(index).map_or("-".to_string(), |op| describe(geometry, index, op));
            let r = right.get(index).map_or("-".to_string(), |op| describe(geometry, index, op));
            println!("    左 {l}\n    右 {r}");
        }
    }
}

/// 按内容比两串写（不看下标）：只在左边的、只在右边的（多重集差）。
fn multiset_difference(geometry: &FixedGeometry, name: &str, left: &[RetainedOperation], right: &[RetainedOperation]) {
    let key = |op: &RetainedOperation| (op.operation.device.0, op.operation.offset.0, op.operation.length, op.operation.content_hash, format!("{:?}", op.operation.kind));
    let mut counts: BTreeMap<_, (i64, usize, usize)> = BTreeMap::new();
    for (index, op) in left.iter().enumerate() {
        let entry = counts.entry(key(op)).or_insert((0, index, usize::MAX));
        entry.0 += 1;
    }
    for (index, op) in right.iter().enumerate() {
        let entry = counts.entry(key(op)).or_insert((0, usize::MAX, index));
        entry.0 -= 1;
        if entry.2 == usize::MAX { entry.2 = index; }
    }
    let only_left: Vec<_> = counts.iter().filter(|(_, v)| v.0 > 0).collect();
    let only_right: Vec<_> = counts.iter().filter(|(_, v)| v.0 < 0).collect();
    println!("  MULTISET {name}: 只在左 {} 种、只在右 {} 种", only_left.len(), only_right.len());
    for (k, v) in only_left.iter().take(40) {
        println!("    只在左 ×{} 首见#{}: 盘{} 槽{} 长{} {} {}", v.0, v.1, k.0, k.1 / SLOT, k.2, k.4, geometry.classify(&left[v.1].operation).name());
    }
    for (k, v) in only_right.iter().take(40) {
        println!("    只在右 ×{} 首见#{}: 盘{} 槽{} 长{} {} {}", -v.0, v.2, k.0, k.1 / SLOT, k.2, k.4, geometry.classify(&right[v.2].operation).name());
    }
}

fn unlied_stream(seed: u64) -> Vec<RetainedOperation> {
    let execution = unchecked();
    let history = generate_history_with_weights(HistorySeed(seed), 20, &GenerationWeights::BROAD);
    let stream = SharedStream::retaining_contents();
    let _run = execute_history_handing_out_every_step_image(
        &history,
        execution,
        &stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |_| {},
    );
    stream.retained_operations()
}

fn print_verdicts(name: &str, image: &MemoryPool) {
    let picked: Vec<String> = check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| ["I-3.1", "I-3.11", "I-3.15"].contains(invariant))
        .map(|(invariant, verdict)| {
            let shape = match verdict {
                InvariantVerdict::Holds => "成立".to_string(),
                InvariantVerdict::Violated(_) => "违例".to_string(),
                InvariantVerdict::NotApplicable(reason) => format!("不适用（{reason}）"),
                InvariantVerdict::PartiallyJudged(reason) => format!("部分判（{reason}）"),
            };
            format!("{invariant}={shape}")
        })
        .collect();
    println!("  VERDICTS {name}: {}", picked.join("；"));
}

fn print_violations(name: &str, violations: &[(&'static str, String)]) {
    println!("  VIOLATIONS {name}: {:?}", violations.iter().map(|(i, _)| *i).collect::<Vec<_>>());
    for (invariant, detail) in violations {
        if *invariant == "I-3.15" || *invariant == "I-2.1" {
            println!("    {invariant}: {detail}");
        }
    }
}

fn detail(images: &ImagesAfterAFault, label: &str) {
    let geometry = images.device_width.fixed_geometry();
    let after = &images.image_after_the_fault;
    let landed: &RecordedStreamOfAnImage = images.image_with_the_swallowed_writes_landed.as_ref().expect("吞了写");
    println!(
        "DETAIL {label} seed={} ({}) reopen={} 注入之后 {} 步、mkfs {} 步、掉电接缝 {:?}",
        images.seed.0,
        images.drawn.render(),
        images.reopen,
        after.operations.len(),
        after.operations_written_by_make_filesystem,
        after.earliest_stream_index_after_each_power_loss
    );
    let inserted = landed
        .operations
        .iter()
        .zip(after.operations.iter())
        .position(|(l, r)| !same(l, r))
        .unwrap_or(after.operations.len());
    let swallowed = &landed.operations[inserted];
    println!("  SWALLOWED {}", describe(&geometry, inserted, swallowed));
    let step = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .find(|s| s.operations_recorded > inserted)
        .map(|s| (s.position, s.operation_kind, s.operations_recorded));
    println!("  落在哪一步（第一张录到它之后的每步镜像）: {step:?}");
    let slot = swallowed.operation.offset.0 / SLOT;
    for (index, op) in after.operations.iter().enumerate().skip(inserted) {
        let s = op.operation.offset.0 / SLOT;
        if op.operation.length > 0 && s <= slot && slot < s + op.operation.length.div_ceil(SLOT) {
            println!("  之后写到同一槽 {}", describe(&geometry, index, op));
        }
    }
    let unlied = unlied_stream(images.seed.0);
    let placement = |op: &RetainedOperation| (op.operation.device.0, format!("{:?}", op.operation.kind), op.operation.offset.0, op.operation.length);
    let same_placements = landed.operations.len() == unlied.len()
        && landed.operations.iter().zip(&unlied).all(|(l, r)| placement(l) == placement(r));
    let mut content_differs_by_kind: BTreeMap<String, u64> = BTreeMap::new();
    for (l, r) in landed.operations.iter().zip(&unlied) {
        if !same(l, r) {
            let tree = l.contents.as_ref().filter(|c| c.len() >= 51 && c[6] == 2).map(|c| u64::from_le_bytes(c[42..50].try_into().unwrap()));
            *content_differs_by_kind.entry(format!("{} 码2树={tree:?}", geometry.classify(&l.operation).name())).or_insert(0) += 1;
        }
    }
    let first_differing = landed.operations.iter().zip(&unlied).position(|(l, r)| !same(l, r));
    let step_of_first = first_differing.and_then(|index| images.step_images_of_the_run_with_the_fault.iter().find(|s| s.operations_recorded > index).map(|s| (s.position, s.operation_kind)));
    println!("  PLACEMENTS 插回去 vs 不说谎：每一步的(盘, 种类, 偏移, 长度)逐个相同={same_placements}；内容不同的步按种类 {content_differs_by_kind:?}；第一处内容不同落在的步 {step_of_first:?}");
    println!("  插回去那张盘前 {} 步与不说谎那一趟逐字节相同={}", inserted + 1, landed.operations[..=inserted].iter().zip(&unlied).all(|(l, r)| same(l, r)));
    compare(&geometry, "插回去那张盘（左） vs 设备不说谎（右）", &landed.operations, &unlied);
    multiset_difference(&geometry, "插回去那张盘（左） vs 设备不说谎（右）", &landed.operations, &unlied);
    compare(&geometry, "注入之后（左） vs 设备不说谎（右）", &after.operations, &unlied);
    print_verdicts("注入之后", &image_of(&after.operations, after.operations.len()));
    print_verdicts("插回去", &image_of(&landed.operations, landed.operations.len()));
    print_verdicts("不说谎", &image_of(&unlied, unlied.len()));
    eprintln!("PHASE 注入之后");
    print_violations("注入之后", &violations_of(&image_of(&after.operations, after.operations.len())));
    eprintln!("PHASE 插回去");
    print_violations("插回去", &violations_of(&image_of(&landed.operations, landed.operations.len())));
    eprintln!("PHASE 不说谎");
    print_violations("不说谎", &violations_of(&image_of(&unlied, unlied.len())));
}

#[test]
fn probe_swallowed_writes() {
    let execution = unchecked();
    let only_seed: Option<u64> = std::env::var("PROBE_SEED").ok().and_then(|v| v.parse().ok());
    let seeds: Vec<u64> = (0..24)
        .map(|offset| SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + offset)
        .filter(|seed| only_seed.is_none_or(|only| only == *seed))
        .collect();
    let next = AtomicUsize::new(0);
    let collected: Mutex<Vec<(u64, usize, ImagesAfterAFault, String)>> = Mutex::new(Vec::new());
    let tallies = Mutex::new(BTreeMap::<String, u64>::new());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads() {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(seed) = seeds.get(index) else { break };
                let history = generate_history_with_weights(HistorySeed(*seed), 20, &GenerationWeights::BROAD);
                let mut handed_out = Vec::new();
                let _ = inject_faults_into_history_handing_out_the_images(&history, execution, 4, &mut |images| {
                    handed_out.push(images);
                });
                for (point, images) in handed_out.into_iter().enumerate() {
                    for failure in judge(&images) {
                        let form = match &failure {
                            FailureAfterAFault::KnownRed(form) => format!("known-red {}", form.render()),
                            FailureAfterAFault::LeftByTheLyingDevice { signature } => format!("lying-device {signature:?}"),
                            FailureAfterAFault::NewFinding { signature, .. } => format!("NEW {signature:?}"),
                        };
                        *tallies.lock().unwrap().entry(form.clone()).or_insert(0) += 1;
                        if images.drawn.fault == InjectedFault::WriteIsSwallowed {
                            collected.lock().unwrap().push((*seed, point, images.clone(), form));
                        }
                    }
                }
            });
        }
    });
    for (form, count) in tallies.into_inner().unwrap() {
        println!("TALLY {count} {form}");
    }
    let mut collected = collected.into_inner().unwrap();
    collected.sort_by_key(|(seed, point, _, _)| (*seed, *point));
    for (seed, point, images, form) in &collected {
        println!("SWALLOWED_POINT seed={seed} point={point} {} → {form}", images.drawn.render());
    }
    let detail_all = std::env::var("PROBE_DETAIL_ALL").is_ok();
    for (seed, point, images, form) in &collected {
        if form.starts_with("NEW") || detail_all {
            if let Ok(slot) = std::env::var("PROBE_SLOT_OF_POINT") { if !slot.starts_with(&format!("{point}:")) { continue; } }
            detail(images, &format!("point={point} {form} (seed {seed})"));
        }
    }
}
