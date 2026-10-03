//! m3-ring-r2 攻方原型 G2：K1 那一类洞（新实例写行那次发布记录已持久、根槽没持久）之后的下一次可写挂载 k，
//! 录 k 的流，按层 0 的枚举域（每段都展开、原地覆写三态）逐个崩溃状态跑恢复 + oracle + 池级 checker（枚举器自己判），
//! 另用一条不经 checker 的直接判法：崩溃镜像上「环里那条比一圈还旧的根 R（txg h − 3S，住 slot(h)）」在不在候选集
//! （有效 ∧ txg ≥ F_生效 ∧ 一圈窗口按镜像上读得出的最新根算），在的话它引用的槽有没有一个字节与挂载 k 之前不同。
//! 一圈窗口的锚由环境变量 SINGLEFS_WINDOW_ANCHOR（off / ring / first_txg）选，checker 的窗口由 SINGLEFS_CHECKER_WINDOW=1 开。
//!
//! 用法：window_k1_states <S> <cut: row|all> <后缀覆盖写次数 K> <回退扫描 0|1> [count-only]
#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};

use ring_window_proto::{
    build_prefix, content, devices_of, disk_metrics, geometry, parameters, pool_of, recording_of,
    violated, FILESYSTEM_IDENTIFIER, WRITE_TIME,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, effective_rollback_floor,
    instance_table_of_root, readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table,
    PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const SLOT: u64 = 16384;

fn materialize(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, persisted) in image.writes.iter().zip(&image.persisted) {
        if *persisted {
            write.contents.apply_to(pool.devices.get_mut(&write.device).expect("盘"), write.offset);
        }
    }
    pool
}

/// 一条根引用的全部 (盘, 槽)：它那一版账里仍分配的记录；账为空（第 0 代 / 树表 0 条）时取实例表与树表两条指针。
fn placements_of(reader: &dyn PoolReader, root: &RootRecord) -> BTreeSet<(u32, u64)> {
    let mut out = BTreeSet::new();
    if let Ok(records) = allocation_records_under_root(reader, root) {
        for r in records.iter().filter(|r| !r.is_released) {
            for s in r.slot.0..r.slot.0 + u64::from(r.span_slots) {
                out.insert((r.device.0, s));
            }
        }
    }
    out
}

/// 镜像上按「一圈窗口、锚在读得出的最新根」判的候选集（有效 ∧ txg ≥ F_生效 ∧ txg > 最新 − 3S），交回 txg 表。
fn windowed_candidates(image: &MemoryPool) -> Option<(u64, Vec<RootRecord>)> {
    let configuration = choose_system_configuration(image).ok()?;
    let immutable = &configuration.immutable;
    let roots: Vec<RootRecord> = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|(_, r)| r)
    .collect();
    let newest = *roots.iter().max_by_key(|r| (r.checkpoint_txg, r.instance))?;
    let table = instance_table_of_root(image, &newest)?;
    let floor = effective_rollback_floor(image, &immutable.region_devices, &immutable.sizes, &immutable.filesystem_identifier);
    let ring_len = 3 * immutable.sizes.root_ring_slots_per_region.count();
    let candidates = roots
        .into_iter()
        .filter(|r| {
            !root_is_abandoned_by_the_instance_table(r, &table)
                && r.checkpoint_txg >= floor
                && r.checkpoint_txg.0 + ring_len > newest.checkpoint_txg.0
        })
        .collect();
    Some((newest.checkpoint_txg.0, candidates))
}

fn slot_bytes(image: &MemoryPool, device: u32, slot: u64) -> Option<Vec<u8>> {
    image.read(DeviceIdentity(device), DeviceOffsetInBytes(slot * SLOT), usize::try_from(SLOT).ok()?)
}

#[derive(Default)]
struct Aggregate {
    direct_hits: BTreeMap<String, u64>,
    direct_first: BTreeMap<String, String>,
    follow: BTreeMap<String, u64>,
    follow_first: BTreeMap<String, String>,
    followed_states: u64,
    states_by_roots: BTreeMap<String, u64>,
    rollbacks: u64,
    rollback_refusals: BTreeMap<String, u64>,
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let slots_per_region: u64 = arguments[1].parse().expect("S");
    let cut_after_row = match arguments[2].as_str() {
        "row" => true,
        "all" => false,
        other => panic!("{other}"),
    };
    let suffix_overwrites: u64 = arguments[3].parse().expect("K");
    let sweep_rollbacks = arguments[4] == "1";
    let count_only = arguments.get(5).is_some_and(|a| a == "count-only");
    let parameters = parameters(slots_per_region);
    let started = std::time::Instant::now();
    let hole = 3 * slots_per_region + 4;
    std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
    std::env::set_var("HOLE_TXG", hole.to_string());
    let prefix = build_prefix(slots_per_region, hole);
    std::env::remove_var("CRASH_THE_ROW_PUBLISH");
    let base = prefix.image.clone();
    let configuration = choose_system_configuration(&base).expect("系统配置");
    let target = target_for_publish(CheckpointTxg(hole), configuration.immutable.sizes.root_ring_slots_per_region);
    let stale = readable_roots_with_ring_slots(
        &base,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|(slot, _)| *slot == target)
    .map(|(_, root)| root)
    .expect("slot(h) 上住着上一圈的根");
    let stale_placements = placements_of(&base, &stale);
    let stale_bytes: BTreeMap<(u32, u64), Vec<u8>> = stale_placements
        .iter()
        .map(|(d, s)| ((*d, *s), slot_bytes(&base, *d, *s).expect("读得出")))
        .collect();
    let before = disk_metrics(&base).expect("挂载 k 之前的盘面");
    println!(
        "PROTO name=prefix s={slots_per_region} anchor={:?} hole={hole} stale_root_txg={} stale_root_instance={} stale_placements={} newest={} oldest_valid={} floor={} window_floor={} valid_outside_window={:?}",
        singlefs_core::mount::window_anchor(),
        stale.checkpoint_txg.0, stale.instance.0, stale_placements.len(), before.newest_txg, before.oldest_valid, before.floor, before.window_floor, before.valid_roots_outside_the_window
    );
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&base, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("挂载 k");
    let operations = stream.retained_operations();
    let (all_writes, all_segments) = writes_and_segments(&operations, &geometry(&parameters));
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let root_txg_of = |write: &singlefs_harness::memory_pool::RetainedWrite| -> Option<u64> {
        (write.kind == StepKind::RootRecordFua)
            .then(|| write.bytes().and_then(|b| RootRecord::parse_slot(b, &FILESYSTEM_IDENTIFIER)))
            .flatten()
            .map(|root| root.checkpoint_txg.0)
    };
    let row_root_index = (0..all_writes.len()).find(|i| root_txg_of(&all_writes[*i]) == Some(row_txg)).expect("写行那次根槽写");
    // cut=row：这段历史是「挂载 k 做完写行那次发布（含系统配置轮换）就断电」；它的崩溃状态 = 前面各段全落 × 当前段任意组合，一个不落。
    let keep_segments = if cut_after_row {
        let row_segment = all_segments.iter().position(|s| s.contains(&row_root_index)).expect("写行根所在段");
        let mut keep = row_segment + 1;
        while keep < all_segments.len()
            && all_segments[keep].iter().all(|i| matches!(all_writes[*i].kind, StepKind::SystemConfigurationSlot | StepKind::Barrier))
        {
            keep += 1;
        }
        keep
    } else {
        all_segments.len()
    };
    let segments: Vec<Vec<usize>> = all_segments[..keep_segments].to_vec();
    let last_write = segments.iter().flatten().copied().max().expect("有写");
    let writes = all_writes[..=last_write].to_vec();
    let mut versions = prefix.versions.clone();
    for version in std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes) {
        if version.file_version().is_some() && (!cut_after_row || version.root().checkpoint_txg.0 == row_txg) {
            versions.push(PublishedVersion {
                instance: version.root().instance,
                checkpoint_txg: version.root().checkpoint_txg,
                content: prefix.hole_content.clone(),
            });
        }
    }
    let root_indexes: Vec<(usize, u64)> = (0..writes.len()).filter_map(|i| root_txg_of(&writes[i]).map(|t| (i, t))).collect();
    let segment_text: Vec<String> = segments
        .iter()
        .map(|segment| segment.iter().map(|i| root_txg_of(&writes[*i]).map_or_else(|| writes[*i].kind.name().to_string(), |t| format!("root{t}"))).collect::<Vec<_>>().join("+"))
        .collect();
    let closed_form = closed_form_state_count(&segments);
    let with_torn = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &|_, _| Layer0SegmentExpansion::EveryProperSubset);
    // 挂载 k 的写里落在 R 引用的槽上的有几次（按写的 (盘, 偏移) 算覆盖的槽）。
    let writes_into_stale: Vec<(usize, u64)> = writes
        .iter()
        .enumerate()
        .filter(|(_, w)| w.kind == StepKind::UnitWrite)
        .filter_map(|(i, w)| {
            let first = w.offset.0 / SLOT;
            let len = w.length_in_bytes();
            let last = (w.offset.0 + len.max(1) - 1) / SLOT;
            (first..=last).find(|s| stale_placements.contains(&(w.device.0, *s))).map(|s| (i, s))
        })
        .collect();
    println!(
        "PROTO name=stream instance={} row_txg={row_txg} warm_ups={} cut={} writes={} segments={} closed_form_states={closed_form} layer0_states_with_torn={with_torn} roots={root_indexes:?} unit_writes_into_stale_root_slots={writes_into_stale:?} segment_sizes={:?}",
        mounted.output.instance.0,
        mounted.output.warm_up_publishes.len(),
        if cut_after_row { "row" } else { "all" },
        writes.len(),
        segments.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    println!("PROTO name=segments list={}", segment_text.join(" | "));
    if count_only || with_torn > 1_000_000 {
        println!("PROTO name=count_only_or_over_budget states={with_torn}");
        return;
    }
    let unit_segments: Vec<Vec<usize>> = segments
        .iter()
        .filter(|s| s.len() >= 3 && s.iter().all(|i| writes[*i].kind == StepKind::UnitWrite))
        .cloned()
        .collect();
    let mut aggregate = Aggregate::default();
    let mut observe = |image: &CrashImage<'_>, _report: &singlefs_core::recovery::RecoveryReport| {
        let original = &image.persisted[..writes.len()];
        let label_roots = root_indexes.iter().map(|(i, t)| format!("r{t}={}", u8::from(original[*i]))).collect::<Vec<_>>().join(",");
        *aggregate.states_by_roots.entry(label_roots.clone()).or_default() += 1;
        let state = materialize(image);
        // 直接判法（不经 checker）。
        if let Some((newest, candidates)) = windowed_candidates(&state) {
            let stale_is_candidate = candidates.iter().any(|r| r.checkpoint_txg == stale.checkpoint_txg && r.instance == stale.instance);
            let changed: Vec<(u32, u64)> = stale_bytes
                .iter()
                .filter(|((d, s), bytes)| slot_bytes(&state, *d, *s).as_ref() != Some(*bytes))
                .map(|(k, _)| *k)
                .collect();
            let key = format!("newest={newest} stale_candidate={stale_is_candidate} stale_slots_changed={}", !changed.is_empty());
            *aggregate.direct_hits.entry(key.clone()).or_default() += 1;
            aggregate.direct_first.entry(key).or_insert_with(|| format!("{label_roots} changed={:?}", changed.iter().take(4).collect::<Vec<_>>()));
        }
        let followed = unit_segments.iter().all(|segment| {
            let landed = segment.iter().filter(|i| original[**i]).count();
            landed <= 1 || landed == segment.len()
        });
        if !followed || suffix_overwrites == 0 {
            return;
        }
        aggregate.followed_states += 1;
        let mut record = |aggregate: &mut Aggregate, key: String, detail: String| {
            *aggregate.follow.entry(key.clone()).or_default() += 1;
            aggregate.follow_first.entry(key).or_insert(detail);
        };
        let mut devices = devices_of(&state);
        let next = match mount_writable(&parameters, &mut devices) {
            Ok(next) => next,
            Err(error) => {
                record(&mut aggregate, format!("next_mount_err:{}", format!("{error:?}").chars().take(60).collect::<String>()), label_roots.clone());
                return;
            }
        };
        let after_mount = pool_of(&devices);
        for name in violated(&after_mount) {
            record(&mut aggregate, format!("after_next_mount:{name}"), label_roots.clone());
        }
        let instance = next.output.instance;
        let mut allocator = next.allocator.clone();
        let Some(mut current) = next.current.clone().into_file_version() else {
            record(&mut aggregate, "next_mount_without_file".to_string(), label_roots.clone());
            return;
        };
        let base_image = pool_of(&devices);
        let base_current = current.clone();
        let base_allocator = allocator.clone();
        for step in 0..suffix_overwrites {
            let salt = 1000 + step;
            let bytes = content(salt);
            let published = {
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + salt }, instance)
            };
            match published {
                Ok(v) => current = v,
                Err(error) => {
                    record(&mut aggregate, format!("suffix_err:{}", format!("{error:?}").chars().take(50).collect::<String>()), label_roots.clone());
                    break;
                }
            }
            let now = pool_of(&devices);
            for name in violated(&now) {
                record(&mut aggregate, format!("suffix_step{step}:{name}"), label_roots.clone());
            }
        }
        if sweep_rollbacks {
            let Some((_, candidates)) = windowed_candidates(&base_image) else { return };
            let newest_txg = base_current.root.checkpoint_txg;
            for target in candidates.iter().filter(|r| r.checkpoint_txg != newest_txg) {
                let mut devices = devices_of(&base_image);
                let mut allocator = base_allocator.clone();
                let mut current = base_current.clone();
                aggregate.rollbacks += 1;
                if let Err(error) = roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg }) {
                    *aggregate.rollback_refusals.entry(format!("{error:?}").chars().take(70).collect()).or_default() += 1;
                    continue;
                }
                for name in violated(&pool_of(&devices)) {
                    record(&mut aggregate, format!("rollback:{name}"), format!("{label_roots} target={}", target.checkpoint_txg.0));
                }
                for step in 0..2u64 {
                    let bytes = content(2000 + step);
                    let published = {
                        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                        publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + 2000 + step }, instance)
                    };
                    match published {
                        Ok(v) => current = v,
                        Err(_) => break,
                    }
                    for name in violated(&pool_of(&devices)) {
                        record(&mut aggregate, format!("after_rollback_step{step}:{name}"), format!("{label_roots} target={}", target.checkpoint_txg.0));
                    }
                }
            }
        }
    };
    let judged_root_index = row_root_index;
    let tally = enumerate_layer0_selecting_versions_observing_each_state(&base, &writes, &segments, judged_root_index, &versions, &|_, _| true, &mut observe);
    let checker_violations: Vec<String> = tally.checker_violated_states.iter().filter(|(_, c)| **c > 0).map(|(n, c)| format!("{n}:{c}")).collect();
    println!(
        "PROTO name=tally states={} oracle_violations={} failed_states={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violated={:?} first_violation={:?} first_checker={:?}",
        tally.states, tally.violations, tally.failed_states, tally.record_root_without_record, tally.record_claimed_state_missing_unit, checker_violations, tally.first_violation, tally.checker_first_violation.iter().next()
    );
    for (label, count) in &aggregate.states_by_roots {
        println!("PROTO name=states_by_roots count={count} roots_landed={label}");
    }
    for (key, count) in &aggregate.direct_hits {
        println!("PROTO name=direct count={count} {key} first={}", aggregate.direct_first[key]);
    }
    for (key, count) in &aggregate.follow {
        println!("PROTO name=follow_up count={count} key={key} first={}", aggregate.follow_first[key]);
    }
    for (key, count) in &aggregate.rollback_refusals {
        println!("PROTO name=rollback_refused count={count} cause={key}");
    }
    println!(
        "PROTO name=summary s={slots_per_region} anchor={:?} checker_window={} states={} followed={} rollbacks={} seconds={:.1}",
        singlefs_core::mount::window_anchor(),
        std::env::var("SINGLEFS_CHECKER_WINDOW").unwrap_or_default(),
        tally.states, aggregate.followed_states, aggregate.rollbacks, started.elapsed().as_secs_f64()
    );
}
