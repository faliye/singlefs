//! 层 0 的 oracle（`classified_oracle_violation_for_versions`）与记录核对器（`check_records`）对 F2 退回那几形怎么判。不跑任何 checker 档的测试，
//! 只调它的两个库函数；崩溃状态取录的那次发布 R 的「根槽之前全落、根槽落、之后没落」那一个（层 0 的枚举域里的一个状态）。
#![allow(clippy::all, dead_code, unused_imports)]
#[path = "../../repo/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use common_unwitnessed_publish::*;
use singlefs_checker_tier::crash::{check_records, classified_oracle_violation_for_versions};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::recovery::{choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy, RootRingSlotReading};
use singlefs_harness::memory_pool::{newest_persisted_root, CrashImage, MemoryPool, RetainedWrite};

fn zero_writes(base: &MemoryPool, ranges: &[(DeviceIdentity, u64, u64)]) -> MemoryPool {
    let mut copy = base.clone();
    for (device, offset, length) in ranges {
        copy.devices.get_mut(device).unwrap().write(DeviceOffsetInBytes(*offset), &vec![0u8; *length as usize]);
    }
    copy
}

fn root_slot_range(image: &MemoryPool, key: (InstanceGeneration, CheckpointTxg)) -> (DeviceIdentity, u64, u64) {
    let configuration = choose_system_configuration(image).unwrap();
    let sizes = &configuration.immutable.sizes;
    every_root_ring_slot(sizes).into_iter().find_map(|ring_slot| match read_root_ring_slot(image, &configuration.immutable.region_devices, sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
        RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key => Some((
            configuration.immutable.region_devices[ring_slot.region as usize],
            singlefs_core::root_ring::slot_offset(ring_slot, sizes.fixed_structure_slot_spacing).0,
            u64::from(sizes.physical_block_size),
        )),
        _ => None,
    }).unwrap()
}

fn main() {
    for code in ["ow", "sw3"] {
        for overwrites_before in [1u64, 4] {
            let publish = if code == "ow" { PublishRecordedAsR::Overwrite } else { PublishRecordedAsR::SequentialWriteOfThreeDataUnits };
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish);
            let persisted: Vec<bool> = (0..recorded.writes.len()).map(|i| i <= recorded.root_index).collect();
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let r_units: Vec<(DeviceIdentity, u64, u64)> = recorded.unit_writes().iter().map(|w| (w.device, w.offset.0, w.length_in_bytes())).collect();
            let v_root = vec![root_slot_range(&crash, recorded.previous_key)];
            let versions = recorded.pool.published_versions();
            for (shape, hidden) in [("none", vec![]), ("Ru", r_units.clone()), ("Ru+Vr", [r_units.clone(), v_root.clone()].concat())] {
                // 读回全 0 的那几段落在基线（R 之前）与 R 的写上：基线上写零、R 的单元写从写表里拿掉换成写零之后的镜像——
                // 记录核对器按持久集合判，单元读回全 0 时就是那几次写不在盘上。
                let base = zero_writes(&recorded.base, &v_root.iter().copied().filter(|_| hidden.len() > r_units.len()).collect::<Vec<_>>());
                let mut writes: Vec<RetainedWrite> = recorded.writes.clone();
                let mut persisted_here = persisted.clone();
                if !hidden.is_empty() {
                    for (index, write) in writes.iter().enumerate() {
                        if r_units.iter().any(|(d, o, _)| *d == write.device && *o == write.offset.0) && write.kind == singlefs_harness::segments::StepKind::UnitWrite {
                            persisted_here[index] = false;
                        }
                    }
                }
                let image = CrashImage { base: &base, writes: &writes, persisted: persisted_here.clone() };
                let report = recover(&image, JournalPolicy::Consult);
                let newest = newest_persisted_root(&writes, &persisted_here);
                let verdict = classified_oracle_violation_for_versions(&report.outcome, report.effective_root, newest, &versions)
                    .map(|v| format!("{:?}", v.kind))
                    .unwrap_or_else(|| "none".to_string());
                let records = check_records(&image, report.effective_root);
                println!(
                    "OPUS ORACLE {code} k={overwrites_before} shape={shape} R=({},{}) V=({},{}) effective={:?} newest_persisted={:?} oracle={verdict} root_without_record={} claimed_state_missing_unit={}",
                    recorded.key.0 .0, recorded.key.1 .0, recorded.previous_key.0 .0, recorded.previous_key.1 .0,
                    report.effective_root.map(|(i, t)| (i.0, t.0)), newest.map(|(t, i)| (i.0, t.0)), records.root_without_record, records.claimed_state_missing_unit
                );
                let _ = &mut writes;
            }
        }
    }
}
