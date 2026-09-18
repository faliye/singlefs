//! m2-wave2-code-r1 云端攻方腿（Opus）Z3 探针：聚簇段登记。
//! z3a：重开之后登记表为空，用户数据落进上一次挂载开过、仍装着提交内生块的段（扫每次挂载之后的覆盖写次数）。
//! z3b：小盘上同一次挂载里开过的段全登记，用户数据分配不出来，而登记过的段里还有空的偶数槽对。

mod opus_probe_common;

use std::collections::BTreeMap;

use opus_probe_common::*;
use singlefs_core::address::SlotNumber;
use singlefs_core::transaction::{PublishError, TransactionOutput, TransactionUnit};

const UNIT_AREA_START: u64 = 50176;
const SEGMENT: u64 = 64;

fn segment_of(slot: u64) -> u64 {
    UNIT_AREA_START + (slot - UNIT_AREA_START) / SEGMENT * SEGMENT
}

/// 每个槽最后一次被哪个角色写（跨度两槽的两个槽都记）。
fn note_units(last_writer: &mut BTreeMap<u64, (TransactionUnit, u64)>, output: &TransactionOutput) {
    for unit in &output.units {
        if !output.rewritten.contains(&unit.identity) {
            continue;
        }
        let span = unit.identity.span_slots();
        for offset in 0..span {
            last_writer.insert(unit.slot.0 + offset, (unit.identity, output.root.checkpoint_txg.0));
        }
    }
}

/// 某段里此刻占着（位图里不空）且最后写它的是提交内生角色的槽：(槽, 角色, 写它那次发布的 txg, 记录是否已释放)。
fn commit_generated_in_segment(
    pool: &Pool,
    last_writer: &BTreeMap<u64, (TransactionUnit, u64)>,
    segment: u64,
) -> Vec<(u64, TransactionUnit, u64, &'static str)> {
    let map = &pool.allocator.devices[0];
    let mut found = Vec::new();
    for slot in segment..segment + SEGMENT {
        if map.is_free(SlotNumber(slot)) {
            continue;
        }
        if let Some((identity, txg)) = last_writer.get(&slot) {
            if *identity != TransactionUnit::Data {
                let record = pool
                    .allocator
                    .records()
                    .iter()
                    .find(|record| record.device.0 == 0 && record.slot.0 <= slot && slot < record.slot.0 + u64::from(record.span_slots));
                let state = match record {
                    Some(record) if record.is_released => "released-deferred",
                    Some(_) => "allocated",
                    None => "no-record",
                };
                found.push((slot, *identity, *txg, state));
            }
        }
    }
    found
}

/// 根环里最旧的可读根的 txg（被抛弃的不另排除：这些历史里没有回退）。
fn oldest_ring_txg(pool: &Pool) -> u64 {
    let parameters = parameters();
    singlefs_core::recovery::readable_roots(
        &pool.devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .iter()
    .map(|root| root.checkpoint_txg.0)
    .min()
    .unwrap_or(0)
}

/// 段里的提交内生块分三档：live（现行版本的账里仍分配）、ring（已释放，释放代 − 1 ≥ 环里最旧根的 txg：环里还有根引用它）、garbage（环里已没有根引用）。
fn strong(found: &[(u64, TransactionUnit, u64, &'static str)], pool: &Pool, oldest: u64) -> Vec<(u64, TransactionUnit, u64, String)> {
    found
        .iter()
        .filter_map(|(slot, identity, txg, state)| {
            let record = pool.allocator.records().iter().find(|record| record.device.0 == 0 && record.slot.0 <= *slot && *slot < record.slot.0 + u64::from(record.span_slots))?;
            if !record.is_released {
                return Some((*slot, *identity, *txg, format!("live({state})")));
            }
            if record.generation.0 >= oldest + 1 {
                return Some((*slot, *identity, *txg, format!("ring-referenced(release={} oldest_ring={oldest})", record.generation.0)));
            }
            None
        })
        .collect()
}

fn run_pattern_history(pattern: &[usize], max_mounts: usize) -> Option<String> {
    let mut pool = fresh_pool();
    let mut last_writer: BTreeMap<u64, (TransactionUnit, u64)> = BTreeMap::new();
    note_units(&mut last_writer, &pool.current.clone());
    let mut earlier_segments: Vec<u64> = pool.allocator.cluster_segments().iter().map(|slot| slot.0).collect();
    let mut seed = 0usize;
    for mount_number in 1..=max_mounts {
        let per_session = pattern[(mount_number - 1) % pattern.len()];
        match remount(&mut pool) {
            MountOutcome::Mounted(mounted) => {
                let _ = take_probe_log();
                note_units(&mut last_writer, mounted.output.row_publish.file_version().expect("带文件"));
                for warm_up in &mounted.output.warm_up_publishes {
                    note_units(&mut last_writer, warm_up.file_version().expect("带文件"));
                }
            }
            MountOutcome::Refused(error) => return Some(format!("pattern={pattern:?} mount={mount_number} refused {error:?} (no strong hit)")),
            MountOutcome::Panicked(message) => return Some(format!("pattern={pattern:?} mount={mount_number} PANICKED {message} (no strong hit before)")),
        }
        for write in 0..per_session {
            seed += 1;
            match overwrite(&mut pool, seed) {
                Ok(output) => {
                    note_units(&mut last_writer, &output.clone());
                    let data_slot = output.unit(TransactionUnit::Data).slot.0;
                    let segment = segment_of(data_slot);
                    if !earlier_segments.contains(&segment) {
                        continue;
                    }
                    let registered_now = pool.allocator.cluster_segments().iter().any(|slot| slot.0 == segment);
                    let found = commit_generated_in_segment(&pool, &last_writer, segment);
                    let oldest = oldest_ring_txg(&pool);
                    let strong_found = strong(&found, &pool, oldest);
                    if !strong_found.is_empty() {
                        return Some(format!(
                            "STRONG_HIT pattern={pattern:?} mount={mount_number} write_in_session={} txg={} data_slot={data_slot} segment={segment} registered_in_this_mount={registered_now} this_mount_cluster_segments={:?} oldest_ring_txg={oldest} co_located={strong_found:?}",
                            write + 1,
                            output.root.checkpoint_txg.0,
                            pool.allocator.cluster_segments(),
                        ));
                    }
                }
                Err(error) => return Some(format!("pattern={pattern:?} mount={mount_number} overwrite refused {error:?} (no strong hit)")),
            }
        }
        earlier_segments.extend(pool.allocator.cluster_segments().iter().map(|slot| slot.0));
        earlier_segments.sort_unstable();
        earlier_segments.dedup();
    }
    None
}

#[test]
fn z3a_strong_user_data_next_to_referenced_commit_generated_blocks() {
    let max_mounts = std::env::var("OPUS_Z3A_MOUNTS").ok().and_then(|value| value.parse().ok()).unwrap_or(40);
    let mut patterns: Vec<Vec<usize>> = (1..=40).map(|k| vec![k]).collect();
    for a in [0usize, 1, 4, 8] {
        for b in [12usize, 16, 20, 24, 30, 36] {
            patterns.push(vec![a, b]);
            patterns.push(vec![b, a]);
        }
    }
    let mut strong_hits = 0;
    for pattern in &patterns {
        match run_pattern_history(pattern, max_mounts) {
            Some(line) => {
                if line.starts_with("STRONG_HIT") {
                    strong_hits += 1;
                }
                println!("{line}");
            }
            None => println!("pattern={pattern:?}: no strong hit in {max_mounts} mounts"),
        }
    }
    println!("patterns={} strong_hits={strong_hits}", patterns.len());
}

fn run_reopen_history(per_session: usize, max_mounts: usize) -> Option<String> {
    let mut pool = fresh_pool();
    let mut last_writer: BTreeMap<u64, (TransactionUnit, u64)> = BTreeMap::new();
    note_units(&mut last_writer, &pool.current.clone());
    // 上一次挂载（或实例 1）开过的聚簇段：记下来，重开之后拿来比。
    let mut previous_session_segments: Vec<u64> = pool.allocator.cluster_segments().iter().map(|slot| slot.0).collect();
    let mut seed = 0usize;
    for mount_number in 1..=max_mounts {
        match remount(&mut pool) {
            MountOutcome::Mounted(mounted) => {
                let _ = take_probe_log();
                note_units(&mut last_writer, mounted.output.row_publish.file_version().expect("带文件"));
                for warm_up in &mounted.output.warm_up_publishes {
                    note_units(&mut last_writer, warm_up.file_version().expect("带文件"));
                }
            }
            MountOutcome::Refused(error) => return Some(format!("per_session={per_session} mount={mount_number} refused {error:?} (no hit)")),
            MountOutcome::Panicked(message) => return Some(format!("per_session={per_session} mount={mount_number} PANICKED {message}")),
        }
        for write in 0..per_session {
            seed += 1;
            match overwrite(&mut pool, seed) {
                Ok(output) => {
                    note_units(&mut last_writer, &output.clone());
                    let data_slot = output.unit(TransactionUnit::Data).slot.0;
                    let segment = segment_of(data_slot);
                    let registered_now = pool.allocator.cluster_segments().iter().any(|slot| slot.0 == segment);
                    let commit_generated = commit_generated_in_segment(&pool, &last_writer, segment);
                    if previous_session_segments.contains(&segment) && !commit_generated.is_empty() {
                        return Some(format!(
                            "HIT per_session={per_session} mount={mount_number} write_in_session={} txg={} data_slot={data_slot} segment={segment} registered_in_this_mount={registered_now} previous_session_segments={previous_session_segments:?} this_mount_cluster_segments={:?} commit_generated_in_segment={commit_generated:?} current_version_units={:?}",
                            write + 1,
                            output.root.checkpoint_txg.0,
                            pool.allocator.cluster_segments(),
                            output.units.iter().map(|unit| (unit.identity, unit.slot.0)).collect::<Vec<_>>()
                        ));
                    }
                }
                Err(error) => return Some(format!("per_session={per_session} mount={mount_number} overwrite refused {error:?} (no hit)")),
            }
        }
        // 这一次挂载开过的段，就是下一次重开之后「上次留下的段」。
        previous_session_segments.extend(pool.allocator.cluster_segments().iter().map(|slot| slot.0));
        previous_session_segments.sort_unstable();
        previous_session_segments.dedup();
    }
    None
}

#[test]
fn z3a_user_data_lands_in_a_segment_opened_by_an_earlier_mount() {
    let max_mounts = std::env::var("OPUS_Z3A_MOUNTS").ok().and_then(|value| value.parse().ok()).unwrap_or(60);
    for per_session in [0usize, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24, 30] {
        match run_reopen_history(per_session, max_mounts) {
            Some(line) => println!("{line}"),
            None => println!("per_session={per_session}: no hit in {max_mounts} mounts"),
        }
    }
}


/// 一次挂载里登记表最多长到几段：挂载之后一路覆盖写到被准入拒（或 400 次），每次挂载各记一次；再挂载、再写，挂载 20 次。
#[test]
fn z3c_how_many_segments_one_mount_registers() {
    for initial in [0usize, 30, 45] {
        let mut pool = fresh_pool();
        let mut seed = 0usize;
        for _ in 0..initial {
            seed += 1;
            overwrite(&mut pool, seed).expect("初始覆盖写");
        }
        println!("initial={initial}: instance 1 registered {} segments; unit area segments {}", pool.allocator.cluster_segments().len(), pool.allocator.devices[0].unit_area_slots() / 64);
        let mut maximum = 0usize;
        for mount_number in 1..=20 {
            match remount(&mut pool) {
                MountOutcome::Mounted(_) => {}
                MountOutcome::Refused(error) => {
                    println!("  mount={mount_number} refused {error:?}");
                    break;
                }
                MountOutcome::Panicked(message) => {
                    println!("  mount={mount_number} PANICKED {message}");
                    break;
                }
            }
            let _ = take_probe_log();
            let mut writes = 0;
            let stop = loop {
                if writes >= 400 {
                    break "400 writes".to_string();
                }
                seed += 1;
                match overwrite(&mut pool, seed) {
                    Ok(_) => writes += 1,
                    Err(error) => break format!("{error:?}").chars().take(70).collect::<String>(),
                }
            };
            let registered = pool.allocator.cluster_segments().len();
            maximum = maximum.max(registered);
            println!("  mount={mount_number} writes_in_session={writes} registered={registered} open_segment={:?} stop={stop}", pool.allocator.open_segment());
        }
        println!("initial={initial}: max registered in one mount = {maximum}");
    }
}
