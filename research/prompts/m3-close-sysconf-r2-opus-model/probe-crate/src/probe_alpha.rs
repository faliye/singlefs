//! 离线出路 α（攻方照形态甲的文字实现的，只在这份原型里）：调用方断言版本（checkpoint_txg），只重写不自证的槽，
//! 写的实例代号取各盘自证槽与根环的最大值（journal 环在这两个盘面上没有更高的实例，没读），世代号取同盘另一槽 + 1，
//! tail 取断言那一版的位置。`checked` 时先在根环里读断言那一版的根，读不出或不是它就一个字节都不写。
use crate::common_admission_pool::{fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices};
use crate::probe_body::{arm_name, reds, record_mount, spacing, sysconf_signature};
use crate::{image_of_probe, probe_devices_on, starting_point, ProbeDevice};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::highest_root_instance;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedOperationKind;

fn slots(device: &ProbeDevice, spacing: u64) -> Vec<(u64, Option<SystemConfiguration>)> {
    [0, spacing]
        .into_iter()
        .map(|offset| {
            let mut bytes = vec![0u8; 4096];
            let parsed = device.read_at(DeviceOffsetInBytes(offset), &mut bytes).ok().and_then(|()| SystemConfiguration::parse_slot(&bytes).ok());
            (offset, parsed)
        })
        .collect()
}

fn newest_slot_offset(device: &ProbeDevice, spacing: u64) -> u64 {
    slots(device, spacing)
        .into_iter()
        .filter_map(|(offset, slot)| slot.map(|slot| (slot.quantities.slot_generation, offset)))
        .max()
        .expect("这块盘有自证过的槽")
        .1
}

fn mount_described(devices: &mut Vec<(DeviceIdentity, ProbeDevice)>) -> (String, Option<u32>) {
    let parameters = parameters_of_the_384_slot_small_devices();
    match mount_writable(&parameters, devices) {
        Ok(mounted) => (
            format!("mounted instance={} effective_txg={}", mounted.output.instance.0, mounted.output.effective_root.checkpoint_txg.0),
            Some(mounted.output.instance.0),
        ),
        Err(error) => (format!("refused {}", format!("{error:?}").chars().take(110).collect::<String>()), None),
    }
}

/// α 本体：交回写了几槽或拒的原因。
fn alpha(devices: &mut [(DeviceIdentity, ProbeDevice)], asserted: CheckpointTxg, asserted_tail: u64, checked: bool, remap: bool) -> String {
    let parameters = parameters_of_the_384_slot_small_devices();
    let spacing = spacing();
    let fsid = parameters.filesystem_identifier;
    if checked {
        let target = target_for_publish(asserted, parameters.geometry.root_ring_slots_per_region);
        let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
        let offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
        let mut bytes = vec![0u8; 512];
        let holder = &devices.iter().find(|(identity, _)| *identity == device).expect("有这块盘").1;
        if holder.read_at(offset, &mut bytes).is_err() {
            return "refused_root_of_the_asserted_version_unreadable".into();
        }
        match RootRecord::parse_slot(&bytes, &fsid) {
            Some(root) if root.checkpoint_txg == asserted => {}
            _ => return "refused_root_of_the_asserted_version_absent".into(),
        }
    }
    let readable: Vec<SystemConfiguration> = devices.iter().flat_map(|(_, device)| slots(device, spacing)).filter_map(|(_, slot)| slot).collect();
    let pool = MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(), device_size_in_bytes: devices[0].1.inner.size_in_bytes() };
    let highest_root = highest_root_instance(&pool, &parameters.region_devices, &parameters.geometry, &fsid).unwrap_or(InstanceGeneration(0));
    let instance = readable.iter().map(|slot| slot.quantities.journal_instance).max().unwrap_or(InstanceGeneration(0)).max(highest_root);
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
            rewritten.quantities.journal_instance = instance;
            rewritten.quantities.journal_tail = asserted_tail;
            if device.write_at(DeviceOffsetInBytes(*offset), &rewritten.to_slot(), WriteDurability::Plain).is_err() {
                return format!("write_failed_after_{written}_slots");
            }
            if remap {
                device.unreadable.retain(|start| start != offset);
            }
            written += 1;
        }
    }
    format!("wrote_{written}_slots instance={}", instance.0)
}

fn post_image_reds(devices: &[(DeviceIdentity, ProbeDevice)]) -> Vec<String> {
    reds(&image_of_probe(devices, devices[0].1.inner.size_in_bytes()))
}

/// C555 形：取号之后只写了单元（切在第一条 journal 记录之前），每盘最新那一槽持续读错；三种盘的行为（写之后能读 / 写之后照样读错 / 写不进去）。
fn c555_surface() {
    let (base, _root) = starting_point(0, false);
    let (operations, outcome) = record_mount(&base, false, None);
    let acquired = outcome.expect("录的那一次可写挂载做成");
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let first_record = operations
        .iter()
        .position(|retained| {
            matches!(retained.operation.kind, RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess)
                && geometry.classify(&retained.operation) == StepKind::JournalRecord
        })
        .expect("挂载里写过 journal 记录");
    let mut cut = base.clone();
    cut.apply(&operations[..first_record]);
    let spacing = spacing();
    println!("probe=alpha_c555 {} acquired={acquired} cut_at={first_record} of={} sysconf={}", arm_name(), operations.len(), sysconf_signature(&cut));
    for behavior in ["remap", "persistent", "unwritable"] {
        let mut devices = probe_devices_on(&cut, spacing);
        for (_, device) in &mut devices {
            let newest = newest_slot_offset(device, spacing);
            device.unreadable.push(newest);
            device.writes_fail_on_unreadable = behavior == "unwritable";
        }
        let (first_mount, _) = mount_described(&mut probe_clone(&devices));
        let tail = devices.iter().flat_map(|(_, d)| slots(d, spacing)).filter_map(|(_, s)| s).map(|s| s.quantities.journal_tail).max().unwrap_or(0);
        let alpha_outcome = alpha(&mut devices, CheckpointTxg(0), tail, false, behavior == "remap");
        let (second_mount, instance) = mount_described(&mut devices);
        let collided = instance.is_some_and(|instance| instance <= acquired);
        let reds_after = if instance.is_some() { post_image_reds(&devices) } else { Vec::new() };
        println!(
            "probe=alpha_c555 {} behavior={behavior} before_alpha={first_mount} alpha={alpha_outcome} after_alpha={second_mount} collides_with_units_of_instance_{acquired}={collided} checker_red={reds_after:?}",
            arm_name()
        );
    }
}

fn probe_clone(devices: &[(DeviceIdentity, ProbeDevice)]) -> Vec<(DeviceIdentity, ProbeDevice)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut inner = singlefs_harness::memory_pool::SparseBlockDevice::new(device.inner.size_in_bytes(), singlefs_core::block_device::PhysicalBlockSizeInBytes(512));
            inner.image = device.inner.image.clone();
            (*identity, ProbeDevice { identity: *identity, inner, unreadable: device.unreadable.clone(), writes_fail_on_unreadable: device.writes_fail_on_unreadable, sysconf_spacing: device.sysconf_spacing, sysconf_writes: device.sysconf_writes.clone(), fail_sysconf_write_from: None, only_once: true })
        })
        .collect()
}

/// R2 收掉的那一形：每盘最新一槽读错，同时最新根（txg T）那一槽读不出；α 断言 T、T−1、T+1，查与不查根环。
fn r2_closed_surface(overwrites: usize) {
    let (base, root) = starting_point(overwrites, false);
    let parameters = parameters_of_the_384_slot_small_devices();
    let spacing = spacing();
    let latest = root.checkpoint_txg;
    let target = target_for_publish(latest, parameters.geometry.root_ring_slots_per_region);
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0;
    println!("probe=alpha_r2_closed {} overwrites={overwrites} latest_txg={} root_device={} root_offset={root_offset} sysconf={}", arm_name(), latest.0, root_device.0, sysconf_signature(&base));
    let clean = probe_devices_on(&base, spacing);
    let tails: Vec<(u64, u64)> = clean.iter().map(|(_, d)| {
        let newest = newest_slot_offset(d, spacing);
        let all = slots(d, spacing);
        let newest_tail = all.iter().find(|(o, _)| *o == newest).and_then(|(_, s)| s.as_ref()).expect("最新槽").quantities.journal_tail;
        let other_tail = all.iter().find(|(o, _)| *o != newest).and_then(|(_, s)| s.as_ref()).map_or(0, |s| s.quantities.journal_tail);
        (newest_tail, other_tail)
    }).collect();
    let (newest_tail, other_tail) = tails[0];
    for (assertion, asserted, tail) in [("T", latest.0, newest_tail), ("T_minus_1", latest.0 - 1, other_tail), ("T_plus_1", latest.0 + 1, newest_tail + 1)] {
        for checked in [true, false] {
            let mut devices = probe_devices_on(&base, spacing);
            for (identity, device) in &mut devices {
                let newest = newest_slot_offset(device, spacing);
                device.unreadable.push(newest);
                if *identity == root_device {
                    device.unreadable.push(root_offset);
                }
            }
            let (before, _) = mount_described(&mut probe_clone(&devices));
            let alpha_outcome = alpha(&mut devices, CheckpointTxg(asserted), tail, checked, true);
            let (with_root_unreadable, instance) = mount_described(&mut devices);
            for (_, device) in &mut devices {
                device.unreadable.clear();
            }
            let (root_readable_again, _) = mount_described(&mut devices);
            let reds_end = post_image_reds(&devices);
            println!(
                "probe=alpha_r2_closed {} assertion={assertion} checked={checked} before_alpha={before} alpha={alpha_outcome} mount_root_unreadable={with_root_unreadable} then_root_readable_mount={root_readable_again} checker_red_end={reds_end:?} took_instance={instance:?}",
                arm_name()
            );
        }
    }
}

pub fn run(args: &[String]) {
    for name in args {
        match name.as_str() {
            "c555" => c555_surface(),
            "r2_closed_k2" => r2_closed_surface(2),
            "r2_closed_k1" => r2_closed_surface(1),
            other => panic!("认不出 {other}"),
        }
    }
}
