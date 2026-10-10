//! MODE=n7：C479 开关（`InodeNumberWatermarkSource::PressedBackForTests`）在号、水位、墓碑、回退的组合里。
//! 起点：4 GiB 两块内存稀疏盘（`start_plain`，mkfs、取号、暖机、第一个文件，会话接着），按 COMBOS 里的一串动作走（经会话的产品路径）：
//!   cN 建 N 个空 inode；uX unlink inode X（产品批大小）；w 覆盖写第一个文件；rI 管理员回退到第 I 步之后那一版（0 = 第一个文件那一版）；
//!   x 崩溃恢复抛弃最新那条根（照 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 的造法，改到稀疏盘、
//!   藏的是最新那次发布重写的单元而不是数据单元），再可写挂载落到它前一条根。
//! 走完之后在拷贝上：关开关（`FromThePreviousVersionsAccounting`）经 `publish_new_inodes` 建 1 个，报全部违例；
//! 开关压到每个 p ∈ [1, 水位 − 1]、建 n ∈ NS 个，报 I-9.6 的判定与全部违例（或被拒 / panic）。
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::common_admission::*;
use crate::{env_list, short};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{
    publish_new_inodes, InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Pool = PoolUnderTest<SparseBlockDevice>;

fn fork(p: &Pool) -> Pool {
    let s = p.session();
    PoolUnderTest {
        parameters: p.parameters.clone(),
        device_bytes: p.device_bytes,
        devices: plain_devices_on(&p.image()),
        session: Some(MountedSession {
            allocator: s.allocator.clone(),
            current: s.current.clone(),
            instance: s.instance,
            shadow_ledger: s.shadow_ledger,
            parameters_and_device_table: s.parameters_and_device_table.clone(),
        }),
        write_time_seconds: p.write_time_seconds,
        content_of_the_current_version: p.content_of_the_current_version.clone(),
        content_of_each_root: p.content_of_each_root.clone(),
    }
}

fn current(p: &Pool) -> TransactionOutput {
    p.session().current.version_of_trees().expect("有树").clone()
}

fn violations(image: &MemoryPool) -> Vec<String> {
    checker_violations_on(image).into_iter().map(|v| v.chars().take(160).collect()).collect()
}

fn i96(image: &MemoryPool) -> String {
    match check_pool_image(image).into_iter().find(|(n, _)| *n == "I-9.6").map(|(_, v)| v) {
        Some(InvariantVerdict::Holds) => "holds".into(),
        Some(InvariantVerdict::Violated(d)) => format!("VIOLATED({})", d.chars().take(80).collect::<String>()),
        Some(other) => format!("{other:?}").chars().take(60).collect(),
        None => "absent".into(),
    }
}

/// 崩溃恢复抛弃最新那条根：藏它的根槽与它重写的每个单元（两块盘），见证它的系统配置槽写回成轮换之前那一代，可写挂载，再把藏的字节写回。
fn abandon_newest(p: &mut Pool) -> Result<(), String> {
    let newest = current(p);
    let params = p.parameters.clone();
    let target = singlefs_core::root_ring::target_for_publish(newest.root.checkpoint_txg, params.geometry.root_ring_slots_per_region);
    let root_dev = params.region_devices[target.region as usize];
    let root_off = singlefs_core::root_ring::slot_offset(target, params.geometry.fixed_structure_slot_spacing);
    let root_len = params.geometry.physical_block_size as usize;
    let mut places = vec![(root_dev, root_off, root_len)];
    for role in &newest.rewritten {
        let unit = newest.unit(*role);
        for (id, _) in &p.devices {
            places.push((*id, unit.slot.to_device_offset(), (unit.identity.span_slots() * singlefs_format::SLOT_BYTES) as usize));
        }
    }
    p.session = None;
    let idx = |devs: &Vec<(DeviceIdentity, SparseBlockDevice)>, id: DeviceIdentity| devs.iter().position(|(c, _)| *c == id).unwrap();
    let mut saved = Vec::new();
    for (id, off, len) in places {
        let i = idx(&p.devices, id);
        let mut bytes = vec![0u8; len];
        p.devices[i].1.read_at(off, &mut bytes).map_err(|e| format!("暂存:{e:?}"))?;
        p.devices[i].1.write_at(off, &vec![0u8; len], WriteDurability::Plain).map_err(|e| format!("清零:{e:?}"))?;
        saved.push((id, off, bytes));
    }
    let spacing = u64::from(params.geometry.fixed_structure_slot_spacing);
    let mut rewrites = Vec::new();
    for (id, _) in &p.devices {
        let mut slots = singlefs_core::recovery::verified_system_configuration_slots(p.devices.as_slice(), *id, spacing, &params.filesystem_identifier);
        slots.sort_by_key(|s| s.quantities.slot_generation);
        let [older, newest_slot] = <[_; 2]>::try_from(slots).map_err(|_| "两槽不都自证".to_string())?;
        let g = newest_slot.quantities.slot_generation;
        let mut back = older;
        back.quantities.slot_generation = g.checked_sub(2).ok_or("轮换不够两次")?;
        rewrites.push((*id, DeviceOffsetInBytes((g % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing), back.to_slot()));
    }
    for (id, off, bytes) in rewrites {
        let i = idx(&p.devices, id);
        p.devices[i].1.write_at(off, &bytes, WriteDurability::Plain).map_err(|e| format!("写回系统配置:{e:?}"))?;
    }
    let mounted = mount_writable(&params, &mut p.devices).map_err(|e| format!("挂载:{}", short(format!("{e:?}"))))?;
    for (id, off, bytes) in &saved {
        let i = idx(&p.devices, *id);
        p.devices[i].1.write_at(*off, bytes, WriteDurability::Plain).map_err(|e| format!("写回:{e:?}"))?;
    }
    let landed = mounted.output.effective_root.checkpoint_txg;
    let (_, session) = MountedSession::of_the_mount(mounted);
    p.session = Some(session);
    if landed.0 >= newest.root.checkpoint_txg.0 {
        return Err(format!("没落到更旧的根：{} ≥ {}", landed.0, newest.root.checkpoint_txg.0));
    }
    Ok(())
}

fn step(p: &mut Pool, op: &str, marks: &[(singlefs_core::address::InstanceGeneration, CheckpointTxg)]) -> Result<(), String> {
    let (kind, arg) = op.split_at(1);
    let refused = |e: singlefs_core::mounted_session::UserChangeRefused| short(format!("{e:?}"));
    match kind {
        "c" => p.publish(UserChange::NewInodes { count: arg.parse().unwrap(), write_time_seconds: p.write_time_seconds + 1 }, None).map(|_| ()).map_err(refused),
        "u" => p.publish(UserChange::Unlink { inode: InodeNumber(arg.parse().unwrap()), write_time_seconds: p.write_time_seconds + 1 }, None).map(|_| ()).map_err(refused),
        "w" => p.overwrite(OVERWRITE_BYTES).map(|_| ()).map_err(refused),
        "r" => {
            let (instance, checkpoint_txg) = marks[arg.parse::<usize>().unwrap()];
            let params = p.parameters.clone();
            let PoolUnderTest { session, devices, .. } = p;
            let s = session.as_mut().unwrap();
            let mut cur = s.current.version_of_trees().unwrap().clone();
            roll_back_by_a_forward_publish(&params, devices, &mut s.allocator, &mut cur, RollbackTarget { instance, checkpoint_txg })
                .map_err(|e| format!("rollback:{}", short(format!("{e:?}"))))?;
            s.current = PoolVersion::of_a_version_of_trees(cur);
            Ok(())
        }
        "x" => abandon_newest(p),
        other => Err(format!("动作 {other}")),
    }
}

fn create_with(p: &mut Pool, source: InodeNumberWatermarkSource, n: u64) -> Result<TransactionOutput, String> {
    let PoolUnderTest { session, devices, parameters, write_time_seconds, .. } = p;
    let s = session.as_mut().unwrap();
    let prev = s.current.version_of_trees().unwrap().clone();
    let mut w = PoolWriter::new(parameters, devices.as_mut_slice());
    w.set_inode_number_watermark_source(source);
    let out = publish_new_inodes(&mut w, &mut s.allocator, &prev, n, *write_time_seconds + 7, s.instance).map_err(|e| short(format!("{e:?}")))?;
    s.current = PoolVersion::of_a_version_of_trees(out.clone());
    Ok(out)
}

fn keys(out: &TransactionOutput) -> Vec<(u64, u64, u32)> {
    out.inode_leaf_container_contents().iter().flat_map(|c| c.records.iter().map(|r| (r.inode, r.object_birth.0, r.nlink))).collect()
}

pub fn run() {
    let ns = env_list("NS", "1,2,3");
    for combo in std::env::var("COMBOS").expect("COMBOS").split(';').filter(|s| !s.is_empty()) {
        let built = catch_unwind(AssertUnwindSafe(|| -> Result<Pool, String> {
            let mut p = start_plain(HistoryDeviceWidth::FourGibibytes);
            let first = current(&p);
            let mut marks = vec![(first.root.instance, first.root.checkpoint_txg)];
            for op in combo.split(',').filter(|s| !s.is_empty()) {
                step(&mut p, op, &marks).map_err(|e| format!("{op}:{e}"))?;
                let c = current(&p);
                marks.push((c.root.instance, c.root.checkpoint_txg));
            }
            Ok(p)
        }));
        let p = match built {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => { println!("COMBO {combo} SKIP {e}"); continue; }
            Err(_) => { println!("COMBO {combo} SKIP panic"); continue; }
        };
        let cur = current(&p);
        let wm = cur.inode_number_watermark();
        let tomb: Vec<u64> = cur.tombstone_containers.iter().flat_map(|c| c.contents.records.iter().map(|r| r.object.0)).collect();
        println!("COMBO {combo} txg={} wm={wm} keys={:?} tombstones={tomb:?} deleted_queue={} before={:?}", cur.root.checkpoint_txg.0, keys(&cur), cur.deleted_inodes.len(), violations(&p.image()));
        if crate::env_u64("N7_ENUM", 1) == 1 {
            enumerate_the_switch_off_create(&p, combo);
        }
        let mut off = fork(&p);
        match catch_unwind(AssertUnwindSafe(|| create_with(&mut off, InodeNumberWatermarkSource::FromThePreviousVersionsAccounting, 1))) {
            Ok(Ok(out)) => println!("OFF {combo} wm_after={} i96={} violations={:?}", out.inode_number_watermark(), i96(&off.image()), violations(&off.image())),
            Ok(Err(e)) => println!("OFF {combo} refused {e}"),
            Err(_) => println!("OFF {combo} panic"),
        }
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        for pressed in 1..wm {
            for n in &ns {
                let mut on = fork(&p);
                let src = InodeNumberWatermarkSource::PressedBackForTests(InodeNumberWatermarkPressedBackForTests::new(pressed).unwrap());
                let line = match catch_unwind(AssertUnwindSafe(|| create_with(&mut on, src, *n))) {
                    Ok(Ok(out)) => {
                        let img = on.image();
                        let v = violations(&img);
                        let red = i96(&img).starts_with("VIOLATED");
                        *counts.entry(format!("i96_red={red} others={}", v.iter().filter(|x| !x.starts_with("I-9.6")).count() > 0)).or_default() += 1;
                        format!("p={pressed} n={n} wm_after={} i96={} violations={:?}", out.inode_number_watermark(), i96(&img), v.iter().map(|x| x.split(':').next().unwrap_or("").to_string()).collect::<Vec<_>>())
                    }
                    Ok(Err(e)) => { *counts.entry(format!("refused {e}")).or_default() += 1; format!("p={pressed} n={n} refused {e}") }
                    Err(_) => { *counts.entry("panic".into()).or_default() += 1; format!("p={pressed} n={n} panic") }
                };
                println!("ON {combo} {line}");
            }
        }
        for (k, c) in &counts { println!("ONSUM {combo} count={c} {k}"); }
    }
}

/// 关开关那一次建 inode 录写流，按层 0 枚举域逐个崩溃状态跑 checker（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开）。
fn enumerate_the_switch_off_create(p: &Pool, combo: &str) {
    use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes_and_entries, PublishedVersion};
    use singlefs_harness::segments::StepKind;
    use singlefs_harness::{RecordingBlockDevice, SharedStream};
    let image = p.image();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image).into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone()))).collect();
    let s = p.session();
    let mut allocator = s.allocator.clone();
    let prev = s.current.version_of_trees().unwrap().clone();
    let out = {
        let mut w = PoolWriter::new(&p.parameters, devices.as_mut_slice());
        publish_new_inodes(&mut w, &mut allocator, &prev, 1, p.write_time_seconds + 7, s.instance)
    };
    let Ok(out) = out else { println!("OFFENUM {combo} refused"); return; };
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&stream.retained_operations(), &stream.entry_spans(), &HistoryDeviceWidth::FourGibibytes.fixed_geometry());
    let states = closed_form_state_count(&segments);
    if states > crate::env_u64("MAX_STATES", 1_000_000) { println!("OFFENUM {combo} skipped closed_form_states={states} (一段历史超过约 10^6，不跑) writes={} segments={:?}", writes.len(), segments.iter().map(Vec::len).collect::<Vec<_>>()); return; }
    let mut versions = p.published_versions();
    versions.push(PublishedVersion { instance: out.root.instance, checkpoint_txg: out.root.checkpoint_txg, content: p.content_of_the_current_version.clone() });
    let judged = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("建 inode 写了根");
    let tally = singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state(&image, &writes, &segments, judged, &versions, &|_, _| true, &mut |_, _| {});
    println!("OFFENUM {combo} writes={} segments={:?} closed_form_states={states} states={} checker_violated={:?} first={:?}",
        writes.len(), segments.iter().map(Vec::len).collect::<Vec<_>>(), tally.states, tally.checker_violated_states,
        tally.checker_first_violation.values().next().map(|v| v.chars().take(160).collect::<String>()));
}
