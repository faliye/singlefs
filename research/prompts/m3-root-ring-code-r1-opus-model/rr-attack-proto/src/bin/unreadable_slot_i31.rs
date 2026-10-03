//! m3-root-ring-code-r1 攻方原型 K2-e（挂载时读不出的槽）：确定性复现。
//! mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`（每次都落盘）→ txg `last`+1 的覆盖写在根槽写之前崩（记录已持久、根槽没持久）。
//! 下一次可写挂载（实例 2）读根环时，txg `last`+1 落到的那个根环槽读不出（`mode`=mount：只在挂载那一次；persist：整条命都读不出）。
//! 之后逐次覆盖写，每次发布之后在盘面（不带读故障）上跑池级 checker，打印违例的不变量与 I-3.1 原文，以及分配器那张根环表里那个槽的住户。
//! 用法：unreadable_slot_i31 <S> <last> <mount|persist|none> <挂载之后覆盖写次数>
use std::cell::RefCell;
use std::rc::Rc;

use rr_attack_proto::{build_plain, content, geometry, parameters, pool_of, recording_of, HidingDevice, WRITE_TIME};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn red(image: &MemoryPool) -> (Vec<&'static str>, String) {
    let mut detail = String::new();
    let names = singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(text) => {
                if name == "I-3.1" { detail = text; }
                Some(name)
            }
            _ => None,
        })
        .collect();
    (names, detail)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let last: u64 = a[2].parse().expect("last");
    let mode = a[3].clone();
    let after: u64 = a[4].parse().expect("次数");
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let (image, _) = build_plain(s, last);
    // txg last+1：录下来，截在根槽写之前。
    let crashed = {
        let devices = rr_attack_proto::devices_of(&image);
        let mut devices = devices;
        let singlefs_core::mount::Mounted { output, mut allocator, current } =
            mount_writable(&parameters, &mut devices).expect("重开");
        let _ = output;
        let mut current = current.into_file_version().expect("有文件");
        // 重开写了行与暖机，盘上 newest 往前挪了；接着覆盖写到 last（txg 以盘上为准）。
        while current.root.checkpoint_txg.0 < last {
            let salt = current.root.checkpoint_txg.0 + 1;
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, output_instance(&current)).expect("覆盖写");
        }
        let before = pool_of(&devices);
        let stream = SharedStream::retaining_contents();
        let mut recording = recording_of(&before, &stream);
        {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            let salt = current.root.checkpoint_txg.0 + 1;
            let out = publish_overwrite(&mut pool, &mut allocator.clone(), &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, output_instance(&current)).expect("被截的那次");
            println!("CRASHED_PUBLISH txg={}", out.root.checkpoint_txg.0);
        }
        let operations = stream.retained_operations();
        let root_write = operations.iter().position(|r| geo.classify(&r.operation) == StepKind::RootRecordFua).expect("根槽写");
        let mut image = before;
        image.apply(&operations[..root_write]);
        image
    };
    let (names, _) = red(&crashed);
    println!("CRASH_STATE checker_red={names:?}");
    let newest_before = rr_attack_proto::disk_metrics(&crashed).map(|m| m.newest_txg).expect("有根");
    let hole_txg = newest_before + 1;
    let slot = target_for_publish(CheckpointTxg(hole_txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(slot.region).expect("区域")];
    let off = slot_offset(slot, parameters.geometry.fixed_structure_slot_spacing).0;
    let occupant = rr_attack_proto::readable_root_txg_at_slot(&crashed, slot);
    println!("HOLE txg={hole_txg} slot=({},{}) device={} occupant_txg={occupant:?} newest_readable={newest_before}", slot.region, slot.slot, device.0);
    let hidden: Rc<RefCell<Vec<_>>> = Rc::new(RefCell::new(Vec::new()));
    if mode != "none" {
        hidden.borrow_mut().push((device, off, off + 512));
    }
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = recording_of(&crashed, &stream)
        .into_iter()
        .map(|(id, rec)| (id, HidingDevice { device: id, inner: rec, hidden: hidden.clone() }))
        .collect();
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
    if mode == "mount" {
        hidden.borrow_mut().clear();
    }
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let occupancy = format!("{:?}", allocator.root_ring_occupancy());
    let mark = format!("RootRingSlot {{ region: {}, slot: {} }}", slot.region, slot.slot);
    let at = occupancy.find(&mark).map(|i| occupancy[i..].chars().take(140).collect::<String>());
    println!("MOUNTED instance={} row_txg={} occupant_in_table={at:?}", instance.0, mounted.output.row_publish.root().checkpoint_txg.0);
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    let snapshot = |devices: &Vec<(singlefs_core::address::DeviceIdentity, HidingDevice<_>)>| -> MemoryPool {
        let _ = devices;
        let mut image = crashed.clone();
        image.apply(&stream.retained_operations());
        image
    };
    let (names, detail) = red(&snapshot(&devices));
    println!("AFTER txg={} red={names:?} {}", current.root.checkpoint_txg.0, detail.chars().take(160).collect::<String>());
    let enum_txgs: Vec<u64> = std::env::var("ENUM_TXGS").ok().map_or(Vec::new(), |v| v.split(':').map(|x| x.parse().expect("txg")).collect());
    let mut versions: Vec<singlefs_harness::memory_pool::PublishedVersion> = Vec::new();
    let file_content = match singlefs_core::recovery::recover(&crashed, singlefs_core::recovery::JournalPolicy::Consult).outcome {
        singlefs_core::recovery::RecoveryOutcome::FileRead { content, .. } => content,
        other => panic!("挂载之前那一版该有文件：{other:?}"),
    };
    // 挂载写的行与暖机那几版内容与恢复出来的那一版相同。
    for txg in mounted.output.row_publish.root().checkpoint_txg.0..=current.root.checkpoint_txg.0 {
        versions.push(singlefs_harness::memory_pool::PublishedVersion { instance, checkpoint_txg: CheckpointTxg(txg), content: file_content.clone() });
    }
    for step in 0..after {
        let salt = 1000 + step;
        let before_publish = snapshot(&devices);
        let operations_before = stream.retained_operations().len();
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        let empty = std::env::var("STEP_EMPTY").is_ok_and(|v| v == "1");
        let kept = versions.last().map_or_else(|| file_content.clone(), |v| v.content.clone());
        current = if empty {
            rr_attack_proto::publish_empty(&mut pool, &mut allocator, &current, instance).expect("空发布")
        } else {
            publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写")
        };
        versions.push(singlefs_harness::memory_pool::PublishedVersion { instance, checkpoint_txg: current.root.checkpoint_txg, content: if empty { kept } else { content(salt) } });
        if enum_txgs.contains(&current.root.checkpoint_txg.0) {
            let operations = stream.retained_operations()[operations_before..].to_vec();
            let (writes, segments) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
            let count = singlefs_harness::memory_pool::closed_form_state_count(&segments);
            let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
            println!("ENUM txg={} segments={} closed_form_state_count={count}", current.root.checkpoint_txg.0, segments.len());
            assert!(count <= 1_000_000, "一次全量不超过约 10^6 个状态");
            let tally = singlefs_checker_tier::crash::enumerate_layer0_versions(&before_publish, &writes, &segments, root_index, &versions);
            println!("ENUM_TALLY txg={} states={} oracle_violations={} checker={}", current.root.checkpoint_txg.0, tally.states, tally.violations, tally.checker_counts_by_invariant());
            for (name, first) in &tally.checker_first_violation {
                println!("ENUM_FIRST txg={} {name} {}", current.root.checkpoint_txg.0, first.chars().take(200).collect::<String>());
            }
        }
        let (names, detail) = red(&snapshot(&devices));
        println!("AFTER txg={} red={names:?} {}", current.root.checkpoint_txg.0, detail.chars().take(160).collect::<String>());
    }
}

fn output_instance(current: &singlefs_core::transaction::TransactionOutput) -> singlefs_core::address::InstanceGeneration {
    current.root.instance
}
