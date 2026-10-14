//! B2（C554 ②）：R（水位之上没施加的那次发布）根槽落了、系统配置没轮换就崩；下一次可写挂载时 R 的根槽与 R 的账（树表那一片两份，`acct`；
//! 再加映射根两份，`acct+map`）读回全零，R 的记录齐、读得出。R 点名的单元里有树表，施加前验不过、R 不施加；F1 乙不拒（记录齐）；
//! C 只拿到 R 的记录点名的落点，账那一份走 `None => continue`。之后这次挂载里覆盖写 j 次、正常卸载（抬 F、回收）。
//! 看 R 从 S 继承来的单元（R 的账里仍分配、不是 R 新分配的）这次挂载里有没有被写掉，R 的根槽还在不在；
//! 再把比 R 新的根全改坏，只读挂载能不能读回 R。臂由核心副本里的 OPUS_B2_ARM 定（空 / released / allocated / allocated_released / mapping）。
//! `MOUNT3=1` 时卸载之后再可写挂载一次（R 的根槽仍读回全零），再覆盖写 j 次，看继承单元。
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::mount::{mount_writable, unmount};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::readable_roots;
use singlefs_core::transaction::FirstFile;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::SharedStream;

use crate::b1::{counter_name, hide_ranges, last_root_write, start};
use crate::pool::{content_of, fixed_geometry_of_the_384_slot_small_devices, OVERWRITE_BYTES};
use crate::util::*;

type Placement = (DeviceIdentity, u64, u64);

fn bytes_at(image: &MemoryPool, (device, slot, span): Placement) -> Vec<u8> {
    image.devices[&device].read(DeviceOffsetInBytes(slot * SLOT_BYTES), (span * SLOT_BYTES) as usize)
}

/// 一次挂载（带读回全零的故障）、会话里覆盖写 j 次、正常卸载；交回卸载之后的镜像与一行说明。
fn one_process(image: &MemoryPool, hide: &[Range], j: u64, seed: u64) -> Result<(MemoryPool, String), String> {
    let stream = SharedStream::retaining_contents();
    let zeros = Rc::new(RefCell::new((hide.to_vec(), false)));
    let mut devices = faulty_recorded_devices(image, &stream, &zeros);
    let mounted = mount_writable(&params(), &mut devices).map_err(|e| format!("mount:{}", counter_name(&format!("{e:?}"))))?;
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    let mut note = format!(
        "eff_txg={} instance={} isolated={:?}",
        output.effective_root.checkpoint_txg.0, output.instance.0, output.isolated_slots_per_device
    );
    for i in 0..j {
        let c = content_of(OVERWRITE_BYTES, seed + i);
        session
            .publish_user_change(&mut devices, UserChange::Overwrite(FirstFile { content: &c, write_time_seconds: 1_790_000_000 + seed + i }))
            .map_err(|e| format!("overwrite_{i}:{}", counter_name(&format!("{e:?}"))))?;
    }
    let shadow = session.shadow_ledger;
    let unmounted = unmount(&params(), &mut devices, &mut session.allocator, &mut session.current, shadow)
        .map_err(|e| format!("unmount:{}", counter_name(&format!("{e:?}"))))?;
    let unmounted_kind = match &unmounted {
        singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) => format!("floor_raised_to_{}_publishes_{}", raised.rollback_floor.0, raised.publishes.len()),
        _ => "nothing_written".to_string(),
    };
    note.push_str(&format!(" last_txg={} unmounted={unmounted_kind}", session.current.root().checkpoint_txg.0));
    let after = image_of(&devices, image.device_size_in_bytes);
    Ok((after, note))
}

pub fn main_b2() {
    let ks: Vec<u64> = env_list("K_LIST", "2,3,4,5").iter().map(|s| s.parse().expect("K")).collect();
    let hides = env_list("HIDE_LIST", "acct,acct+map");
    let arms = env_list("ARMS", ",released,allocated,allocated_released,mapping");
    let arms: Vec<String> = if arms.is_empty() { vec![String::new()] } else { arms };
    let j = env_u64("J", 3);
    let mount3 = env_u64("MOUNT3", 0) == 1;
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    for k in &ks {
        for hide in &hides {
            for arm in std::iter::once(String::new()).chain(arms.iter().filter(|a| !a.is_empty()).cloned()) {
                std::env::set_var("OPUS_B2_ARM", &arm);
                let st = start(*k, OVERWRITE_BYTES);
                let r = &st.p;
                let hide_r: Vec<Range> = hide_ranges(&hide.replace("+map", ""), r)
                    .into_iter()
                    .chain(if hide.contains("map") {
                        r.root.mapping_root.locations.iter().map(|l| (l.device, l.slot.0 * SLOT_BYTES, SLOT_BYTES)).collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    })
                    .collect();
                let (w, _) = writes_and_segments(&st.p_ops, &geometry);
                let root_w = last_root_write(&w);
                let persisted: Vec<bool> = (0..w.len()).map(|i| i <= root_w).collect();
                let crashed = materialize(&st.base, &w, &persisted);
                let r_ref: BTreeSet<Placement> = r.allocation_records.iter().filter(|x| !x.is_released).map(|x| (x.device, x.slot.0, u64::from(x.span_slots))).collect();
                let r_new: BTreeSet<Placement> = r.allocation_records.iter().filter(|x| !x.is_released && x.generation == r.root.checkpoint_txg).map(|x| (x.device, x.slot.0, u64::from(x.span_slots))).collect();
                let inherited: Vec<Placement> = r_ref.difference(&r_new).copied().collect();
                let r_root = root_slot_range(r.root.checkpoint_txg);
                let r_root_bytes = crashed.devices[&r_root.0].read(DeviceOffsetInBytes(r_root.1), r_root.2 as usize);
                let report = |label: &str, image: &MemoryPool, note: &str| {
                    let changed_inherited = inherited.iter().filter(|p| bytes_at(image, **p) != bytes_at(&crashed, **p)).count();
                    let changed_new = r_new.iter().filter(|p| bytes_at(image, **p) != bytes_at(&crashed, **p)).count();
                    let r_in_ring = image.devices[&r_root.0].read(DeviceOffsetInBytes(r_root.1), r_root.2 as usize) == r_root_bytes;
                    // 比 R 新的根全改坏（根槽清零），只读挂载落到哪、读回什么。
                    let mut broken = image.clone();
                    let sc = singlefs_core::recovery::choose_system_configuration(image).expect("系统配置");
                    let newer: Vec<CheckpointTxg> = readable_roots(image, &sc.immutable.region_devices, &sc.immutable.sizes, &sc.immutable.filesystem_identifier)
                        .iter()
                        .filter(|x| x.checkpoint_txg > r.root.checkpoint_txg)
                        .map(|x| x.checkpoint_txg)
                        .collect();
                    for t in &newer {
                        let (d, off, len) = root_slot_range(*t);
                        broken.devices.get_mut(&d).unwrap().write(DeviceOffsetInBytes(off), &vec![0u8; len as usize]);
                    }
                    let (root, result) = read_only(&broken);
                    let verdict = match &result {
                        Ok(c) if Some(c) == st.contents.get(&crate::b1::key(&r.root)) => "reads_r".to_string(),
                        Ok(_) => "reads_other_version".to_string(),
                        Err(e) => format!("failed_{}", counter_name(e).chars().take(60).collect::<String>()),
                    };
                    println!(
                        "OPUS-B2 k={k} hide={hide} arm={arm:?} j={j} stage={label} r_txg={} inherited={} r_new={} changed_inherited={changed_inherited} changed_new={changed_new} r_root_slot_still_r={r_in_ring} newer_roots_zeroed={} ro_after_zeroing={:?}:{verdict} {note}",
                        r.root.checkpoint_txg.0,
                        inherited.len(),
                        r_new.len(),
                        newer.len(),
                        root.map(|x| x.1 .0),
                    );
                };
                match one_process(&crashed, &hide_r, j, 100 * k) {
                    Ok((after2, note)) => {
                        report("after_mount2_unmount", &after2, &note);
                        if mount3 {
                            match one_process(&after2, &[r_root], j, 200 * k) {
                                Ok((after3, note3)) => report("after_mount3_unmount", &after3, &note3),
                                Err(e) => println!("OPUS-B2 k={k} hide={hide} arm={arm:?} stage=mount3 refused={e}"),
                            }
                        }
                    }
                    Err(e) => println!("OPUS-B2 k={k} hide={hide} arm={arm:?} j={j} stage=mount2 refused={e}"),
                }
            }
        }
    }
}
