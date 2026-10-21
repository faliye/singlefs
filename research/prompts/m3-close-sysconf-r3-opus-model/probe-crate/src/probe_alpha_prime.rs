//! α'（攻方照第二轮判决第四节的文字实现，只在这份原型里）：调用方断言版本（checkpoint_txg）；断言那一版的根（根环里按 txg 算的那一槽）
//! 或它的末条 journal 记录读得出、且 txg 是它，才写，否则一个字节都不写；只重写本盘不自证的槽（世代号取同盘自证槽 + 1）；写的实例代号取
//! 各盘自证槽、根环、journal 环（两块盘整环逐条解）的最大值，`scan` 时再并进单元区扫出的写序实例代号（每个槽按 32 KiB 解码 1 / 码 3、
//! 按 16 KiB 解码 2，解得开的才算）；tail 取断言那一版末条记录的计数器（找不到记录时取各盘自证槽 tail 的最大值）。
use crate::common_admission_pool::fixed_geometry_of_the_384_slot_small_devices;
use crate::probe_body::{arm_name, for_each_state, record_mount, reds, spacing, sysconf_signature, variant_name};
use crate::{image_of_probe, pool_parameters, probe_devices_on, starting_point_any, ProbeDevice};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::highest_root_instance;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::unit::{parse_data_unit, parse_index_node, parse_packed_unit, unit_filesystem_identifier};
use singlefs_harness::memory_pool::MemoryPool;
use std::collections::BTreeMap;

const SLOT: u64 = 16384;

fn slots(device: &ProbeDevice, spacing: u64) -> Vec<(u64, Option<SystemConfiguration>)> {
    [0, spacing]
        .into_iter()
        .map(|offset| {
            let mut bytes = vec![0u8; 4096];
            let parsed = device.read_at(DeviceOffsetInBytes(offset), &mut bytes).ok().and_then(|()| SystemConfiguration::parse_slot(&bytes).ok());
            let parsed = parsed.filter(|slot| slot.immutable.filesystem_identifier == pool_parameters().filesystem_identifier);
            (offset, parsed)
        })
        .collect()
}

/// 单元区里每个槽解一次，交回 (最大的写序实例代号, 解开的单元数, 读的槽数)。只读盘。
pub fn scan_unit_area(devices: &[(DeviceIdentity, ProbeDevice)]) -> (u32, u64, u64) {
    let parameters = pool_parameters();
    let start = singlefs_core::allocator::UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes).expect("单元区起点").slot().0;
    let mut highest = 0u32;
    let mut parsed = 0u64;
    let mut read = 0u64;
    for (_, device) in devices {
        let end = device.size_in_bytes() / SLOT;
        for slot in start..end {
            read += 1;
            let mut wide = vec![0u8; 2 * SLOT as usize];
            let offset = DeviceOffsetInBytes(slot * SLOT);
            if slot + 2 <= end && device.read_at(offset, &mut wide).is_ok() {
                if let Ok(header) = parse_data_unit(&wide) {
                    highest = highest.max(header.write_order.instance.0);
                    parsed += 1;
                    continue;
                }
                if let Ok(header) = parse_packed_unit(&wide) {
                    highest = highest.max(header.write_order.instance.0);
                    parsed += 1;
                    continue;
                }
            }
            let mut narrow = vec![0u8; SLOT as usize];
            if device.read_at(offset, &mut narrow).is_ok() {
                if let Ok(header) = parse_index_node(&narrow) {
                    highest = highest.max(header.instance.0);
                    parsed += 1;
                }
            }
        }
    }
    (highest, parsed, read)
}

/// 两块盘的 journal 环逐条解：交回 (最大的记录实例代号, 断言那一版末条记录的计数器)。
fn journal_ring(devices: &[(DeviceIdentity, ProbeDevice)], asserted: CheckpointTxg) -> (u32, Option<u64>) {
    let parameters = pool_parameters();
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    let expected = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let mut highest = 0u32;
    let mut last_of_asserted = None;
    for (_, device) in devices {
        for counter in 1..=(ring_bytes / 4096) {
            let mut bytes = vec![0u8; 4096];
            if device.read_at(record_offset(counter, ring_bytes), &mut bytes).is_err() {
                continue;
            }
            if let Some(record) = JournalRecord::parse(&bytes, expected) {
                highest = highest.max(record.instance.0);
                if record.checkpoint_txg == asserted && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish {
                    last_of_asserted = Some(last_of_asserted.map_or(record.counter, |kept: u64| kept.max(record.counter)));
                }
            }
        }
    }
    (highest, last_of_asserted)
}

pub fn alpha_prime(devices: &mut [(DeviceIdentity, ProbeDevice)], asserted: CheckpointTxg, scan: bool, remap: bool) -> String {
    let parameters = pool_parameters();
    let spacing = spacing();
    let fsid = parameters.filesystem_identifier;
    let target = target_for_publish(asserted, parameters.geometry.root_ring_slots_per_region);
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    let mut bytes = vec![0u8; 512];
    let holder = &devices.iter().find(|(identity, _)| *identity == root_device).expect("有这块盘").1;
    let root_is_it = holder.read_at(offset, &mut bytes).is_ok() && RootRecord::parse_slot(&bytes, &fsid).is_some_and(|root| root.checkpoint_txg == asserted);
    let (highest_record, last_record) = journal_ring(devices, asserted);
    if !root_is_it && last_record.is_none() {
        return "refused_neither_root_nor_record_of_the_asserted_version".into();
    }
    let readable: Vec<SystemConfiguration> = devices.iter().flat_map(|(_, device)| slots(device, spacing)).filter_map(|(_, slot)| slot).collect();
    let tail = last_record.unwrap_or_else(|| readable.iter().map(|slot| slot.quantities.journal_tail).max().unwrap_or(0));
    let pool = MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(), device_size_in_bytes: devices[0].1.inner.size_in_bytes() };
    let highest_root = highest_root_instance(&pool, &parameters.region_devices, &parameters.geometry, &fsid).unwrap_or(InstanceGeneration(0));
    let highest_unit = if scan { scan_unit_area(devices).0 } else { 0 };
    let instance = readable.iter().map(|slot| slot.quantities.journal_instance.0).max().unwrap_or(0).max(highest_root.0).max(highest_record).max(highest_unit);
    let mut written = 0;
    for (_, device) in devices.iter_mut() {
        let on_this_device = slots(device, spacing);
        let Some(template) = on_this_device.iter().find_map(|(_, slot)| slot.clone()) else { continue };
        for (offset, slot) in &on_this_device {
            if slot.is_some() {
                continue;
            }
            let mut rewritten = template.clone();
            rewritten.quantities.slot_generation = template.quantities.slot_generation + 1;
            rewritten.quantities.journal_instance = InstanceGeneration(instance);
            rewritten.quantities.journal_tail = tail;
            if device.write_at(DeviceOffsetInBytes(*offset), &rewritten.to_slot(), WriteDurability::Plain).is_err() {
                return format!("write_failed_after_{written}_slots");
            }
            if remap {
                device.unreadable.retain(|start| start != offset);
            }
            written += 1;
        }
    }
    format!("wrote_{written}_slots instance={instance} tail={tail} by={}", if root_is_it { "root" } else { "record" })
}

fn mount_described(devices: &mut Vec<(DeviceIdentity, ProbeDevice)>) -> (String, Option<(u32, u64)>) {
    match mount_writable(&pool_parameters(), devices) {
        Ok(mounted) => (
            format!("mounted instance={} effective_txg={}", mounted.output.instance.0, mounted.output.effective_root.checkpoint_txg.0),
            Some((mounted.output.instance.0, mounted.output.effective_root.checkpoint_txg.0)),
        ),
        Err(error) => (format!("refused {}", variant_name(&format!("{error:?}"))), None),
    }
}

fn newest_slot_offset(device: &ProbeDevice, spacing: u64) -> u64 {
    slots(device, spacing).into_iter().filter_map(|(offset, slot)| slot.map(|slot| (slot.quantities.slot_generation, offset))).max().expect("有自证槽").1
}

/// 断言 T、T−1、T+1 × 三种盘面（T 的根读得出 / 根读不出而记录读得出 / 两样都读不出）× 扫与不扫；每盘最新一槽读错（写之后读得出）。
fn cells(overwrites: usize) {
    let (base, root) = starting_point_any(overwrites, false);
    let parameters = pool_parameters();
    let spacing = spacing();
    let latest = root.checkpoint_txg;
    let target = target_for_publish(latest, parameters.geometry.root_ring_slots_per_region);
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0;
    let clean = probe_devices_on(&base, spacing);
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    let expected = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let mut records_of_latest: Vec<u64> = Vec::new();
    for counter in 1..=(ring_bytes / 4096) {
        let mut bytes = vec![0u8; 4096];
        if clean[0].1.read_at(record_offset(counter, ring_bytes), &mut bytes).is_ok() {
            if let Some(record) = JournalRecord::parse(&bytes, expected) {
                if record.checkpoint_txg == latest {
                    records_of_latest.push(record_offset(counter, ring_bytes).0);
                }
            }
        }
    }
    println!("probe=alpha_prime_cells {} overwrites={overwrites} latest_txg={} root_device={} record_offsets_of_latest={records_of_latest:?} sysconf={}", arm_name(), latest.0, root_device.0, sysconf_signature(&base));
    for face in ["root_readable", "root_unreadable_record_readable", "neither_readable"] {
        for (assertion, asserted) in [("T", latest.0), ("T_minus_1", latest.0 - 1), ("T_plus_1", latest.0 + 1)] {
            for scan in [false, true] {
                let mut devices = probe_devices_on(&base, spacing);
                for (identity, device) in &mut devices {
                    let newest = newest_slot_offset(device, spacing);
                    device.unreadable.push(newest);
                    if face != "root_readable" && *identity == root_device {
                        device.unreadable.push(root_offset);
                    }
                    if face == "neither_readable" {
                        device.unreadable.extend(records_of_latest.iter().map(|offset| offset - offset % 4096));
                    }
                }
                let (before, _) = mount_described(&mut crate::probe_alpha_clone(&devices));
                let outcome = alpha_prime(&mut devices, CheckpointTxg(asserted), scan, true);
                let (with_faults, taken) = mount_described(&mut devices);
                let lost = taken.is_some_and(|(_, txg)| txg < latest.0);
                for (_, device) in &mut devices {
                    device.unreadable.clear();
                }
                let (faults_cleared, _) = mount_described(&mut devices);
                let reds_end = reds(&image_of_probe(&devices, devices[0].1.inner.size_in_bytes()));
                println!(
                    "probe=alpha_prime_cells {} face={face} assertion={assertion} scan={scan} before={before} alpha_prime={outcome} mount_with_faults={with_faults} lost_confirmed_txg_{}={lost} mount_after_faults_cleared={faults_cleared} checker_red_end={reds_end:?}",
                    arm_name(), latest.0
                );
            }
        }
    }
}

/// 扫单元区的挂钟与读的槽数：384 槽小盘的起点 k0（PROBE_DEVICES 定盘数）。
fn scan_cost() {
    let (base, _root) = starting_point_any(0, false);
    let devices = probe_devices_on(&base, spacing());
    let started = std::time::Instant::now();
    let (highest, parsed, read) = scan_unit_area(&devices);
    println!("probe=scan_cost {} geometry=384_slot_small device_bytes={} highest_instance={highest} parsed_units={parsed} slots_read={read} seconds={:.3}", arm_name(), devices[0].1.inner.size_in_bytes(), started.elapsed().as_secs_f64());
}

/// C555 形（今天的一轮取号流的每个崩溃状态、落了本实例非系统配置写的那些）：每盘最新一槽读错（写之后读得出）；α' 断言盘上最新的根，
/// 扫与不扫两臂，外加「本实例写下的、解得开的单元全部翻坏一个字节」再扫一臂；之后可写挂载，取到的号不大于本实例就记 COLLIDE。
fn scan_on_the_c555_states() {
    let (base, _root) = starting_point_any(0, false);
    let (operations, outcome) = record_mount(&base, false, None);
    let acquired = outcome.expect("录的那一次可写挂载做成");
    let (writes, segments) = singlefs_harness::memory_pool::writes_and_segments(&operations, &fixed_geometry_of_the_384_slot_small_devices());
    let spacing = spacing();
    let fsid = pool_parameters().filesystem_identifier;
    let (judged, tallies) = for_each_state(&base, &writes, &segments, &|ordinal, pool, all_writes, persisted, tally: &mut BTreeMap<String, u64>| {
        let footprint = all_writes.iter().zip(persisted).any(|(write, p)| *p && write.kind != singlefs_harness::segments::StepKind::SystemConfigurationSlot);
        if !footprint {
            return;
        }
        *tally.entry("footprint_states".into()).or_default() += 1;
        let newest_root_txg = {
            let parameters = pool_parameters();
            singlefs_core::recovery::highest_root_instance(pool, &parameters.region_devices, &parameters.geometry, &fsid);
            let devices = probe_devices_on(pool, spacing);
            let mut best = 0u64;
            for txg in 0..64u64 {
                let target = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
                let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
                let mut bytes = vec![0u8; 512];
                let holder = &devices.iter().find(|(identity, _)| *identity == device).expect("盘").1;
                if holder.read_at(slot_offset(target, parameters.geometry.fixed_structure_slot_spacing), &mut bytes).is_ok() {
                    if let Some(root) = RootRecord::parse_slot(&bytes, &fsid) {
                        if root.checkpoint_txg.0 == txg {
                            best = best.max(txg);
                        }
                    }
                }
            }
            best
        };
        for arm in ["no_scan", "scan", "scan_after_flipping_every_unit_of_the_instance"] {
            let mut devices = probe_devices_on(pool, spacing);
            if arm == "scan_after_flipping_every_unit_of_the_instance" {
                flip_units_of(&mut devices, acquired);
            }
            for (_, device) in &mut devices {
                let newest = newest_slot_offset(device, spacing);
                device.unreadable.push(newest);
            }
            let outcome = alpha_prime(&mut devices, CheckpointTxg(newest_root_txg), arm != "no_scan", true);
            let wrote = outcome.starts_with("wrote");
            let key = match mount_described(&mut devices).1 {
                Some((instance, _)) if instance <= acquired => {
                    let red = reds(&image_of_probe(&devices, devices[0].1.inner.size_in_bytes()));
                    *tally.entry(format!("{arm}/COLLIDE/checker_red={red:?}")).or_default() += 1;
                    "COLLIDE".to_string()
                }
                Some(_) => "mounted_above".to_string(),
                None => format!("refused_after_alpha_prime_wrote={wrote}"),
            };
            *tally.entry(format!("{arm}/{key}")).or_default() += 1;
            let _ = ordinal;
        }
    });
    let mut all: BTreeMap<String, u64> = BTreeMap::new();
    for tally in tallies {
        for (key, count) in tally {
            *all.entry(key).or_default() += count;
        }
    }
    println!("probe=alpha_prime_c555_scan {} acquired={acquired} states_judged={judged} tally={all:?}", arm_name());
}

fn flip_units_of(devices: &mut [(DeviceIdentity, ProbeDevice)], instance: u32) {
    let parameters = pool_parameters();
    let start = singlefs_core::allocator::UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes).expect("单元区起点").slot().0;
    for (_, device) in devices.iter_mut() {
        let end = device.size_in_bytes() / SLOT;
        for slot in start..end {
            let offset = DeviceOffsetInBytes(slot * SLOT);
            let mut wide = vec![0u8; 2 * SLOT as usize];
            let mut hit = false;
            if slot + 2 <= end && device.inner.read_at(offset, &mut wide).is_ok() {
                hit = parse_data_unit(&wide).is_ok_and(|h| h.write_order.instance.0 == instance) || parse_packed_unit(&wide).is_ok_and(|h| h.write_order.instance.0 == instance);
            }
            let mut narrow = vec![0u8; SLOT as usize];
            if !hit && device.inner.read_at(offset, &mut narrow).is_ok() {
                hit = parse_index_node(&narrow).is_ok_and(|h| h.instance.0 == instance);
            }
            if hit {
                let mut sector = vec![0u8; 512];
                device.inner.read_at(offset, &mut sector).expect("读");
                sector[60] ^= 0xFF;
                device.inner.write_at(offset, &sector, WriteDurability::Plain).expect("写回翻坏的");
            }
        }
    }
}

pub fn run(args: &[String]) {
    for name in args {
        match name.as_str() {
            "cells_k2" => cells(2),
            "scan_cost" => scan_cost(),
            "c555_scan" => scan_on_the_c555_states(),
            other => panic!("认不出 {other}"),
        }
    }
}
