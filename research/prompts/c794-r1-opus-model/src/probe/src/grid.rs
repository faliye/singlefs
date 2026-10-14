//! MODE=grid：镜像形 × checker 臂。每一形从同一个起点池（384 槽小盘、第一个文件）起，先覆盖写 PRE 次，
//! 第 PRE+1 次覆盖写是事件那一次（丢写那一形另算，见 `find_reused_slot`），之后接着覆盖写，在事件之后第 0、1、… 次（CHECKPOINTS）各取一份镜像，
//! 每份镜像在四条 checker 臂下各判一次，打出违例的不变量。
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

use crate::flaky::{start_pool, Faults};
use crate::pool::OVERWRITE_BYTES;

pub fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
pub fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name).unwrap_or_else(|_| default.to_string()).split(',').filter(|s| !s.is_empty()).map(str::to_string).collect()
}

pub const ARMS: [&str; 4] = ["today", "relax_all", "relax_ring", "marker"];

pub fn violated(image: &MemoryPool, arm: &str, marked: &str) -> Vec<String> {
    std::env::set_var("OPUS_C794_CHECKER", if arm == "today" { "" } else { arm });
    std::env::set_var("OPUS_C794_MARKED", marked);
    let out = check_pool_image(image)
        .into_iter()
        .filter_map(|(n, v)| matches!(v, InvariantVerdict::Violated(_)).then(|| n.to_string()))
        .collect();
    std::env::remove_var("OPUS_C794_CHECKER");
    std::env::remove_var("OPUS_C794_MARKED");
    out
}

fn slot_on(version: &singlefs_core::transaction::TransactionOutput, device: u32) -> u64 {
    version.data_pointers[0].locations.iter().find(|l| l.device == DeviceIdentity(device)).expect("这块盘有一份").slot.0
}

/// 丢写那一形的测量跑：不注入，覆盖写到第一次「新数据单元在盘 1 上的起点槽是更早某一版数据单元用过的起点槽」为止，交回那是第几次覆盖写与那个槽。
pub fn find_reused_slot(limit: u64) -> Option<(u64, u64)> {
    let stream = SharedStream::new();
    let faults = Rc::new(RefCell::new(Faults::default()));
    let mut pool = start_pool(&stream, &faults);
    let mut used = vec![slot_on(pool.current_file_version(), 1)];
    for k in 1..=limit {
        pool.overwrite(OVERWRITE_BYTES).ok()?;
        let slot = slot_on(pool.current_file_version(), 1);
        if used.contains(&slot) {
            return Some((k, slot));
        }
        used.push(slot);
    }
    None
}

pub struct ShapeRun {
    pub marked: String,
    pub quarantined: usize,
    pub snapshots: Vec<(u64, MemoryPool)>,
    pub info: String,
}

pub fn run_shape(shape: &str, checkpoints: &[u64]) -> ShapeRun {
    let pre = env_u64("PRE", 2);
    let (event, lost) = if shape == "lost_write" {
        let (k, slot) = find_reused_slot(env_u64("LOST_SEARCH", 200)).expect("200 次覆盖写之内有复用的槽");
        (k + 1, Some((k, slot)))
    } else {
        (pre + 1, None)
    };
    let stream = SharedStream::new();
    let faults = Rc::new(RefCell::new(Faults::default()));
    let mut pool = start_pool(&stream, &faults);
    let last = event + checkpoints.iter().copied().max().unwrap_or(0);
    let mut run = ShapeRun { marked: String::new(), quarantined: 0, snapshots: Vec::new(), info: format!("event_overwrite={event}") };
    for k in 1..=last {
        if let Some((swallow_at, slot)) = lost {
            if k == swallow_at {
                faults.borrow_mut().swallow.push((DeviceIdentity(1), slot * SLOT_BYTES, 2 * SLOT_BYTES));
                run.info += &format!(" swallowed_slot={slot}");
            }
        }
        if k == event {
            let slot1 = slot_on(pool.current_file_version(), 1);
            let slot0 = slot_on(pool.current_file_version(), 0);
            run.info += &format!(" replaced_unit_slots=d0:{slot0},d1:{slot1}");
            match shape {
                "transient" => faults.borrow_mut().read_fail.push((DeviceIdentity(1), slot1 * SLOT_BYTES, 2)),
                "transient_both" => {
                    faults.borrow_mut().read_fail.push((DeviceIdentity(0), slot0 * SLOT_BYTES, 2));
                    faults.borrow_mut().read_fail.push((DeviceIdentity(1), slot1 * SLOT_BYTES, 2));
                }
                "persist_flip" => {
                    let image = pool.devices[1].1.image_mut();
                    let at = slot1 * SLOT_BYTES + 4096;
                    let mut sector = image.read(DeviceOffsetInBytes(at), 512);
                    sector[0] ^= 0xff;
                    image.write(DeviceOffsetInBytes(at), &sector);
                }
                "leak" => std::env::set_var("OPUS_C794_LEAK_SLOT", slot1.to_string()),
                "bogus" => std::env::set_var("OPUS_C794_BOGUS", "1"),
                "z10" => std::env::set_var("OPUS_C794_Z10", "1"),
                "control" | "lost_write" => {}
                other => panic!("形 {other} 认不得"),
            }
        }
        let outcome = pool.overwrite(OVERWRITE_BYTES);
        for v in ["OPUS_C794_LEAK_SLOT", "OPUS_C794_BOGUS", "OPUS_C794_Z10"] {
            std::env::remove_var(v);
        }
        faults.borrow_mut().swallow.clear();
        if let Err(e) = outcome {
            run.info += &format!(" overwrite_{k}_refused={}", format!("{e:?}").chars().take(80).collect::<String>());
            break;
        }
        if k == event {
            let q = &pool.current_file_version().quarantined_after_release_checksum_mismatch;
            run.quarantined = q.len();
            run.marked = q.iter().map(|c| format!("{}:{}", c.device.0, c.placement.slot.0)).collect::<Vec<_>>().join(",");
        }
        if k >= event && checkpoints.contains(&(k - event)) {
            run.snapshots.push((k - event, pool.image()));
        }
    }
    let f = faults.borrow();
    run.info += &format!(" failed_reads={} swallowed_writes={} newest_txg={}", f.failed_reads, f.swallowed_writes, pool.session().current.root().checkpoint_txg.0);
    run
}

fn differing_sectors(a: &MemoryPool, b: &MemoryPool) -> usize {
    let mut n = 0;
    for (id, da) in &a.devices {
        let db = &b.devices[id];
        let sa = da.written_sectors();
        let sb = db.written_sectors();
        let keys: std::collections::BTreeSet<&u64> = sa.keys().chain(sb.keys()).collect();
        for k in keys {
            let zero = vec![0u8; 512];
            if sa.get(k).unwrap_or(&zero) != sb.get(k).unwrap_or(&zero) {
                n += 1;
            }
        }
    }
    n
}

pub fn main_grid() {
    let checkpoints: Vec<u64> = env_list("CHECKPOINTS", "0,1,30,60").iter().map(|s| s.parse().expect("数")).collect();
    let shapes = env_list("SHAPES", "control,transient,transient_both,persist_flip,lost_write,leak,bogus,z10");
    let mut images: BTreeMap<String, Vec<(u64, MemoryPool)>> = BTreeMap::new();
    for shape in &shapes {
        let run = run_shape(shape, &checkpoints);
        println!("OPUS-SHAPE shape={shape} quarantined_copies={} marked=[{}] {}", run.quarantined, run.marked, run.info);
        for (step, image) in &run.snapshots {
            for arm in ARMS {
                let v = violated(image, arm, &run.marked);
                let i311 = v.iter().any(|n| n == "I-3.11");
                let i31 = v.iter().any(|n| n == "I-3.1");
                println!("OPUS-GRID shape={shape} step=+{step} arm={arm} i3_11_red={i311} i3_1_red={i31} violated={v:?}");
            }
        }
        images.insert(shape.clone(), run.snapshots);
    }
    for (a, b) in [("transient", "leak"), ("transient_both", "leak"), ("transient", "control")] {
        let (Some(ia), Some(ib)) = (images.get(a), images.get(b)) else { continue };
        for ((sa, xa), (sb, xb)) in ia.iter().zip(ib) {
            assert_eq!(sa, sb);
            println!("OPUS-IDENTICAL left={a} right={b} step=+{sa} images_equal={} differing_sectors={}", xa == xb, differing_sectors(xa, xb));
        }
    }
}
