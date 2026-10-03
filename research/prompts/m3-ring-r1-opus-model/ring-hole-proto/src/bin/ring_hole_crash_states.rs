//! m3-ring-r1 攻方原型 H2：带洞的池上可写挂载（补洞挂点 P1 / P2 根前 / P2 根后 / P3 / 不补）录一条流，
//! 按层 0 的枚举域（每段都展开、原地覆写三态）逐个崩溃状态跑恢复 + oracle + 池级 checker（枚举器自己判），
//! 每个状态再接着做「下一次可写挂载 + 用户动作后缀」，每一步之后跑池级 checker、量盘上门槛与多扣。
//!
//! 用法：ring_hole_crash_states <S> <front|back> <挂点 0..4> <后缀覆盖写次数 K> <回退扫描 0|1> [count-only]
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;

use ring_hole_proto::{
    build_prefix, devices_of, disk_metrics, geometry, hole_txg, occupant_of_the_hole_slot,
    parameters, recording_of, violated, content, HolePosition, WRITE_TIME, FILESYSTEM_IDENTIFIER,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, set_hole_fill_point, HoleFillPoint,
    RollbackTarget,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, PublishedVersion,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn point_of(code: &str) -> HoleFillPoint {
    match code {
        "0" => HoleFillPoint::NotFilled,
        "1" => HoleFillPoint::P1AfterAcquisition,
        "2" => HoleFillPoint::P2BeforeRowRoot,
        "3" => HoleFillPoint::P2AfterRowRoot,
        "4" => HoleFillPoint::P3AfterWarmUp,
        "5" => HoleFillPoint::P2RecordedBeforeRowPublish,
        other => panic!("挂点 {other}"),
    }
}

fn materialize(image: &singlefs_harness::memory_pool::CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, persisted) in image.writes.iter().zip(&image.persisted) {
        if *persisted {
            write.contents.apply_to(
                pool.devices.get_mut(&write.device).expect("盘"),
                write.offset,
            );
        }
    }
    pool
}

#[derive(Default)]
struct Aggregate {
    by_label: BTreeMap<String, u64>,
    violations: BTreeMap<String, u64>,
    first_violation_detail: BTreeMap<String, String>,
    max_over_withheld_after_next_mount: u64,
    max_over_withheld_in_suffix: u64,
    max_missing_after_next_mount: usize,
    steps: u64,
    rollbacks: u64,
    rollback_refusals: BTreeMap<String, u64>,
    all_states_by_roots: BTreeMap<String, u64>,
    followed_states: u64,
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let slots_per_region: u64 = arguments[1].parse().expect("S");
    let position = match arguments[2].as_str() {
        "front" => HolePosition::Front,
        "back" => HolePosition::Back,
        other => panic!("{other}"),
    };
    let point = point_of(&arguments[3]);
    let suffix_overwrites: u64 = arguments[4].parse().expect("K");
    let sweep_rollbacks = arguments[5] == "1";
    let count_only = arguments.get(6).is_some_and(|a| a == "count-only");
    let parameters = parameters(slots_per_region);
    let without_file = std::env::var("PREFIX_WITHOUT_FILE").is_ok_and(|v| v == "1");
    let started = std::time::Instant::now();
    let prefix = if without_file {
        ring_hole_proto::build_prefix_without_file(slots_per_region, 3 * slots_per_region + 3)
    } else {
        build_prefix(slots_per_region, hole_txg(slots_per_region, position))
    };
    let hole = prefix.hole;
    set_hole_fill_point(point);
    let base = prefix.image.clone();
    let before = disk_metrics(&base).expect("挂载之前的盘面");
    println!(
        "PROTO name=prefix s={slots_per_region} position={position:?} point={point:?} hole={hole} hole_slot_occupant={:?} newest={} oldest_valid={} floor={} missing={:?} over_withheld={} withheld={}",
        occupant_of_the_hole_slot(&base, hole), before.newest_txg, before.oldest_valid, before.floor, before.missing_txgs, before.over_withheld, before.withheld_by_floor
    );
    // 被测的那一次可写挂载，录流。
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&base, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("被测的可写挂载");
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry(&parameters));
    let mut versions = prefix.versions.clone();
    // 跟进只在单元段（整段都是单元写、至少 3 次写）落了 0、1 或全部时做；其余状态只进枚举器自己的恢复 + oracle + checker。
    let unit_segments: Vec<Vec<usize>> = segments
        .iter()
        .filter(|segment| segment.len() >= 3 && segment.iter().all(|i| writes[*i].kind == StepKind::UnitWrite))
        .cloned()
        .collect();
    for version in std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes) {
        if version.file_version().is_some() {
            versions.push(PublishedVersion {
                instance: version.root().instance,
                checkpoint_txg: version.root().checkpoint_txg,
                content: prefix.hole_content.clone(),
            });
        }
    }
    let root_txg_of = |index: usize| -> Option<u64> {
        let write = &writes[index];
        (write.kind == StepKind::RootRecordFua)
            .then(|| write.bytes().and_then(|b| RootRecord::parse_slot(b, &FILESYSTEM_IDENTIFIER)))
            .flatten()
            .map(|root| root.checkpoint_txg.0)
    };
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let judged_root_index = (0..writes.len())
        .find(|index| root_txg_of(*index) == Some(row_txg))
        .expect("写行那次根槽写");
    let fill_index = (0..writes.len()).find(|index| root_txg_of(*index) == Some(hole));
    let root_indexes: Vec<(usize, u64)> = (0..writes.len())
        .filter_map(|index| root_txg_of(index).map(|txg| (index, txg)))
        .collect();
    let segment_text: Vec<String> = segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .map(|index| match root_txg_of(*index) {
                    Some(txg) => format!("root{txg}"),
                    None => writes[*index].kind.name().to_string(),
                })
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect();
    let closed_form = closed_form_state_count(&segments);
    let with_torn = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &|_, _| {
        Layer0SegmentExpansion::EveryProperSubset
    });
    println!(
        "PROTO name=stream writes={} segments={} closed_form_states={closed_form} layer0_states_with_torn={with_torn} fill_write_index={fill_index:?} judged_root_index={judged_root_index} roots={root_indexes:?} segment_sizes={:?}",
        writes.len(),
        segments.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    println!("PROTO name=segments list={}", segment_text.join(" | "));
    let after = disk_metrics(&{
        let mut pool = base.clone();
        pool.apply_writes(&writes);
        pool
    })
    .expect("挂载做完的盘面");
    println!(
        "PROTO name=mounted instance={} row_txg={row_txg} warm_ups={} newest={} oldest_valid={} floor={} missing={:?} over_withheld={} withheld={} candidates={:?}",
        mounted.output.instance.0,
        mounted.output.warm_up_publishes.len(),
        after.newest_txg, after.oldest_valid, after.floor, after.missing_txgs, after.over_withheld, after.withheld_by_floor, after.candidate_txgs
    );
    if count_only || with_torn > 1_000_000 {
        println!("PROTO name=count_only_or_over_budget states={with_torn}");
        return;
    }
    let mut aggregate = Aggregate::default();
    let persisted_label = |persisted: &[bool]| -> String {
        let mut parts = Vec::new();
        for (index, txg) in &root_indexes {
            parts.push(format!("r{txg}={}", u8::from(persisted[*index])));
        }
        parts.join(",")
    };
    let mut observe = |image: &singlefs_harness::memory_pool::CrashImage<'_>,
                       report: &singlefs_core::recovery::RecoveryReport| {
        let original = &image.persisted[..writes.len()];
        let label_roots = persisted_label(original);
        *aggregate.all_states_by_roots.entry(label_roots.clone()).or_default() += 1;
        let followed = unit_segments.iter().all(|segment| {
            let landed = segment.iter().filter(|i| original[**i]).count();
            landed <= 1 || landed == segment.len()
        });
        if !followed {
            return;
        }
        aggregate.followed_states += 1;
        let state = materialize(image);
        let crash_metrics = disk_metrics(&state);
        let mut devices = devices_of(&state);
        let next = mount_writable(&parameters, &mut devices);
        let mut record = |aggregate: &mut Aggregate, key: String, detail: String| {
            *aggregate.violations.entry(key.clone()).or_default() += 1;
            aggregate.first_violation_detail.entry(key).or_insert(detail);
        };
        let Ok(next) = next else {
            let label = format!("{label_roots}|next_mount=err:{:?}", next.err().map(|e| format!("{e:?}").chars().take(60).collect::<String>()));
            *aggregate.by_label.entry(label).or_default() += 1;
            return;
        };
        let image_after_mount = ring_hole_proto::pool_of(&devices);
        let metrics = disk_metrics(&image_after_mount).expect("下一次挂载之后的盘面");
        let label = format!(
            "{label_roots}|chosen={} effective={} crash_floor={} next_fill={} after_next_mount: newest={} floor={} missing={:?} hole_slot={:?}",
            next.output.chosen_root.checkpoint_txg.0,
            next.output.effective_root.checkpoint_txg.0,
            crash_metrics.as_ref().map_or(0, |m| m.floor),
            next.output.effective_root.checkpoint_txg > next.output.chosen_root.checkpoint_txg,
            metrics.newest_txg,
            metrics.floor,
            metrics.missing_txgs,
            occupant_of_the_hole_slot(&image_after_mount, hole),
        );
        let _ = report;
        *aggregate.by_label.entry(label.clone()).or_default() += 1;
        aggregate.max_over_withheld_after_next_mount = aggregate.max_over_withheld_after_next_mount.max(metrics.over_withheld);
        aggregate.max_missing_after_next_mount = aggregate.max_missing_after_next_mount.max(metrics.missing_txgs.len());
        for name in violated(&image_after_mount) {
            record(&mut aggregate, format!("after_next_mount:{name}"), label.clone());
        }
        let instance = next.output.instance;
        let mut allocator = next.allocator.clone();
        let mut current = match next.current.clone().into_file_version() {
            Some(current) => current,
            None => {
                let bytes = content(999);
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                match singlefs_core::transaction::publish_first_file(&mut pool, &mut allocator, next.current.root(), FirstFile { content: &bytes, write_time_seconds: WRITE_TIME }, instance, next.current.record_bytes()) {
                    Ok(first) => first,
                    Err(error) => {
                        record(&mut aggregate, format!("first_file_err:{:?}", format!("{error:?}").chars().take(50).collect::<String>()), label.clone());
                        return;
                    }
                }
            }
        };
        let base_image = ring_hole_proto::pool_of(&devices);
        let base_current = current.clone();
        let base_allocator = allocator.clone();
        for step in 0..suffix_overwrites {
            let salt = 1000 + step;
            let bytes = content(salt);
            let published = {
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + salt }, instance)
            };
            aggregate.steps += 1;
            match published {
                Ok(next_version) => current = next_version,
                Err(error) => {
                    record(&mut aggregate, format!("suffix_publish_err:{:?}", format!("{error:?}").chars().take(50).collect::<String>()), label.clone());
                    break;
                }
            }
            let image_now = ring_hole_proto::pool_of(&devices);
            for name in violated(&image_now) {
                record(&mut aggregate, format!("suffix_step{step}:{name}"), label.clone());
            }
            if let Some(m) = disk_metrics(&image_now) {
                aggregate.max_over_withheld_in_suffix = aggregate.max_over_withheld_in_suffix.max(m.over_withheld);
            }
        }
        if sweep_rollbacks {
            for target_txg in metrics.candidate_txgs.clone() {
                let Some(target) = ring_hole_proto::readable_root_at_txg(&image_after_mount, target_txg) else { continue };
                if target.checkpoint_txg.0 == metrics.newest_txg { continue; }
                let mut devices = devices_of(&base_image);
                let mut allocator = base_allocator.clone();
                let mut current = base_current.clone();
                aggregate.rollbacks += 1;
                let rolled = roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg });
                if let Err(error) = rolled {
                    *aggregate.rollback_refusals.entry(format!("{error:?}").chars().take(60).collect()).or_default() += 1;
                    continue;
                }
                let image_now = ring_hole_proto::pool_of(&devices);
                for name in violated(&image_now) {
                    record(&mut aggregate, format!("rollback_to_{}:{name}", if target_txg == hole { "hole".to_string() } else { "other".to_string() }), label.clone());
                }
                for step in 0..2u64 {
                    let salt = 2000 + step;
                    let bytes = content(salt);
                    let published = {
                        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                        publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + salt }, instance)
                    };
                    match published {
                        Ok(v) => current = v,
                        Err(error) => { record(&mut aggregate, format!("after_rollback_publish_err:{:?}", format!("{error:?}").chars().take(50).collect::<String>()), label.clone()); break; }
                    }
                    let image_now = ring_hole_proto::pool_of(&devices);
                    for name in violated(&image_now) {
                        record(&mut aggregate, format!("after_rollback_step{step}:{name}"), label.clone());
                    }
                }
            }
        }
        let _ = CheckpointTxg(0);
    };
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &base,
        &writes,
        &segments,
        judged_root_index,
        &versions,
        &|_, _| true,
        &mut observe,
    );
    let checker_violations: Vec<String> = tally
        .checker_violated_states
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(name, count)| format!("{name}:{count}"))
        .collect();
    println!(
        "PROTO name=tally states={} oracle_violations={} failed_states={} ignored_violations={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violated={:?} first_violation={:?} first_checker={:?}",
        tally.states, tally.violations, tally.failed_states, tally.ignored_violations, tally.record_root_without_record, tally.record_claimed_state_missing_unit, checker_violations, tally.first_violation,
        tally.checker_first_violation.iter().next()
    );
    for (label, count) in &aggregate.all_states_by_roots {
        println!("PROTO name=all_states_by_roots count={count} roots_landed={label}");
    }
    println!("PROTO name=followed states={}", aggregate.followed_states);
    for (label, count) in &aggregate.by_label {
        println!("PROTO name=state_class count={count} {label}");
    }
    for (key, count) in &aggregate.violations {
        println!("PROTO name=follow_up_violation key={key} count={count} first_state={}", aggregate.first_violation_detail[key]);
    }
    for (key, count) in &aggregate.rollback_refusals {
        println!("PROTO name=rollback_refused count={count} cause={key}");
    }
    println!(
        "PROTO name=summary s={slots_per_region} position={position:?} point={point:?} states={} follow_up_violation_kinds={} max_over_withheld_after_next_mount={} max_missing_after_next_mount={} max_over_withheld_in_suffix={} suffix_steps={} rollbacks={} seconds={:.1}",
        tally.states,
        aggregate.violations.len(),
        aggregate.max_over_withheld_after_next_mount,
        aggregate.max_missing_after_next_mount,
        aggregate.max_over_withheld_in_suffix,
        aggregate.steps,
        aggregate.rollbacks,
        started.elapsed().as_secs_f64()
    );
}
