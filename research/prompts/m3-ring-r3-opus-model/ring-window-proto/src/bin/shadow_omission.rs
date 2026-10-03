//! m3-ring-r3 攻方原型 H1 四处同改：影子账的候选过滤漏改一处，在哪个盘面上红（第二轮攻方推的那一格）。
//! 构造：K1 前缀（洞 h = 3S + 4，slot(h) 留着比一圈还旧的根 R = h − 3S）→ 挂载 k、覆盖写 n 次（窗口分配器在 R 出窗口之后回收 R 独占的单元，
//! 之后的发布 A 把单元落进其中某个槽）→ 崩在 A 的根槽已落、系统配置没轮换 → 挂载 k+1 时 A 的根槽与 A 的单元暂时读不出
//! （恢复落到 A − 1，写行之后 A 被抛弃；C554 那一形的做法），k+1 只挂载不写 → 挂载 k+2（A 读得出、被抛弃；R 仍在环里、在窗口外）→ 覆盖写 m 次。
//! 每一步量：A 的单元还在不在（字节与当初写的相同）、池级 checker。
//!
//! 用法：shadow_omission <S> <n> <m>（臂由环境变量：SINGLEFS_WINDOW_ANCHOR、SINGLEFS_SHADOW_NO_WINDOW、SINGLEFS_CHECKER_WINDOW…）
#![allow(clippy::too_many_lines)]

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use ring_window_proto::{build_prefix, content, devices_of, geometry, parameters, pool_of, recording_of, violated, HidingDevice, WRITE_TIME};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{allocation_records_under_root, choose_system_configuration, readable_roots_with_ring_slots, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const SLOT: u64 = 16384;

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

fn root_at(image: &MemoryPool, txg: u64) -> Option<RootRecord> {
    let c = choose_system_configuration(image).ok()?;
    readable_roots_with_ring_slots(image, &c.immutable.region_devices, &c.immutable.sizes, &c.immutable.filesystem_identifier)
        .into_iter().map(|(_, r)| r).find(|r| r.checkpoint_txg.0 == txg)
}

fn a_units_intact(image: &MemoryPool, a_units: &[RetainedWrite]) -> usize {
    a_units.iter().filter(|w| {
        let device = image.devices.get(&w.device).expect("盘");
        let len = usize::try_from(w.length_in_bytes()).expect("长");
        let now = device.read(w.offset, len);
        w.contents.still_on_disk(&now)
    }).count()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let n: u64 = a[2].parse().expect("n");
    let m: u64 = a[3].parse().expect("m");
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let h = 3 * s + 4;
    std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
    let prefix = build_prefix(s, h);
    std::env::remove_var("CRASH_THE_ROW_PUBLISH");
    let r_txg = h - 3 * s;
    let r = root_at(&prefix.image, r_txg).expect("R 在 slot(h)");
    let r_slots = placements_of(&prefix.image, &r);
    // 挂载 k，覆盖写 n 次，录下来。
    let stream = SharedStream::retaining_contents();
    let mut rec = recording_of(&prefix.image, &stream);
    let mk = mount_writable(&parameters, &mut rec).expect("挂载 k");
    let inst_k = mk.output.instance;
    let mut alloc = mk.allocator.clone();
    let mut cur = mk.current.clone().into_file_version().expect("带文件");
    for step in 0..n {
        let mut pool = PoolWriter::new(&parameters, rec.as_mut_slice());
        cur = publish_overwrite(&mut pool, &mut alloc, &cur, FirstFile { content: &content(800 + step), write_time_seconds: WRITE_TIME + 800 + step }, inst_k).expect("覆盖写");
    }
    let a_txg = cur.root.checkpoint_txg.0;
    drop(rec);
    let ops = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&ops, &geo);
    let root_segs: Vec<usize> = segments.iter().enumerate().filter(|(_, seg)| seg.iter().any(|i| writes[*i].kind == StepKind::RootRecordFua)).map(|(k, _)| k).collect();
    let last = *root_segs.last().expect("根");
    let prev = root_segs[root_segs.len() - 2];
    let mut crashed = prefix.image.clone();
    for seg in &segments[..=last] {
        for i in seg { crashed.apply_writes(std::slice::from_ref(&writes[*i])); }
    }
    let a_root_write = segments[last].iter().copied().find(|i| writes[*i].kind == StepKind::RootRecordFua).expect("A 根");
    let a_units: Vec<RetainedWrite> = segments[prev + 1..last].iter().flatten().filter(|i| writes[**i].kind == StepKind::UnitWrite).map(|i| writes[*i].clone()).collect();
    let a_root = root_at(&crashed, a_txg).expect("A 的根在镜像上");
    let a_slots = placements_of(&crashed, &a_root);
    let a_written_slots: BTreeSet<(u32, u64)> = a_units.iter().flat_map(|w| (w.offset.0 / SLOT..=(w.offset.0 + w.length_in_bytes() - 1) / SLOT).map(move |sl| (w.device.0, sl))).collect();
    let shared: Vec<(u32, u64)> = a_written_slots.intersection(&r_slots).copied().collect();
    println!("SETUP s={s} n={n} R={r_txg} A=({},{a_txg}) a_unit_writes={} a_slots={} shared_slots_written_by_A_and_referenced_by_R={shared:?} anchor={:?} shadow_no_window={}",
        inst_k.0, a_units.len(), a_slots.len(), singlefs_core::mount::window_anchor(), std::env::var("SINGLEFS_SHADOW_NO_WINDOW").unwrap_or_default());
    // 挂载 k+1：A 的根槽与 A 的单元在这一次挂载里读不出。
    let hidden: Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let w = &writes[a_root_write];
        hidden.borrow_mut().push((w.device, w.offset.0, w.offset.0 + w.length_in_bytes()));
        for u in &a_units { hidden.borrow_mut().push((u.device, u.offset.0, u.offset.0 + u.length_in_bytes())); }
    }
    let mut d1: Vec<(DeviceIdentity, HidingDevice<_>)> = devices_of(&crashed).into_iter().map(|(id, dev)| (id, HidingDevice { device: id, inner: dev, hidden: hidden.clone() })).collect();
    let m1 = match mount_writable(&parameters, &mut d1) { Ok(x) => x, Err(e) => { println!("MOUNT_K1 err={}", format!("{e:?}").chars().take(160).collect::<String>()); return; } };
    hidden.borrow_mut().clear();
    let img1 = MemoryPool { devices: d1.iter().map(|(id, hd)| (*id, hd.inner.image.clone())).collect(), device_size_in_bytes: crashed.device_size_in_bytes };
    println!("MOUNT_K1 chosen=({},{}) effective=({},{}) row_txg={} a_units_intact={}/{} red={:?}", m1.output.chosen_root.instance.0, m1.output.chosen_root.checkpoint_txg.0,
        m1.output.effective_root.instance.0, m1.output.effective_root.checkpoint_txg.0, m1.output.row_publish.root().checkpoint_txg.0, a_units_intact(&img1, &a_units), a_units.len(), violated(&img1));
    // 挂载 k+2：A 读得出（被抛弃），R 仍在环里。
    let mut d2 = devices_of(&img1);
    let m2 = match mount_writable(&parameters, &mut d2) { Ok(x) => x, Err(e) => { println!("MOUNT_K2 err={}", format!("{e:?}").chars().take(160).collect::<String>()); return; } };
    let inst2 = m2.output.instance;
    let img2 = pool_of(&d2);
    println!("MOUNT_K2 row_txg={} abandoned_roots_unreadable={} R_still_in_ring={} A_in_ring={} a_units_intact={}/{} red={:?}", m2.output.row_publish.root().checkpoint_txg.0,
        m2.output.abandoned_roots_unreadable, root_at(&img2, r_txg).is_some(), root_at(&img2, a_txg).is_some(), a_units_intact(&img2, &a_units), a_units.len(), violated(&img2));
    let maps = m2.allocator.proto_device_maps();
    let shared_state: Vec<String> = shared.iter().filter(|(d, _)| *d == 0).map(|(_, sl)| format!("{sl}:free={}", maps[0].is_free(singlefs_core::address::SlotNumber(*sl)))).collect();
    let c = choose_system_configuration(&img2).expect("配置");
    let f_eff = singlefs_core::recovery::effective_rollback_floor(&img2, &c.immutable.region_devices, &c.immutable.sizes, &c.immutable.filesystem_identifier);
    let r_now = root_at(&img2, r_txg).map(|r| placements_of(&img2, &r).iter().filter(|(d, _)| *d == 0).copied().collect::<Vec<_>>());
    println!("MOUNT_K2_SHARED_SLOTS dev0 {shared_state:?} F_eff={} R_dev0_placements={r_now:?}", f_eff.0);
    let mut al2 = m2.allocator.clone();
    let mut c2 = m2.current.clone().into_file_version().expect("带文件");
    for step in 0..m {
        let mut pool = PoolWriter::new(&parameters, d2.as_mut_slice());
        c2 = publish_overwrite(&mut pool, &mut al2, &c2, FirstFile { content: &content(900 + step), write_time_seconds: WRITE_TIME + 900 + step }, inst2).expect("覆盖写");
        let img = pool_of(&d2);
        println!("STEP {step} txg={} A_in_ring={} a_units_intact={}/{} red={:?}", c2.root.checkpoint_txg.0, root_at(&img, a_txg).is_some(), a_units_intact(&img, &a_units), a_units.len(), violated(&img));
    }
    let _ = DeviceOffsetInBytes(0);
    let _ = CheckpointTxg(0);
}
