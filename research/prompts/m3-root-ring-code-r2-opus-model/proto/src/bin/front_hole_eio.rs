//! 改法 A 的代价那一格：前洞（txg h 的覆盖写记录已持久、根槽没写，环还没转过来，slot(h) 从没写过、读出来自证不过），
//! 实例 2 的可写挂载那一刻读 slot(h) 报读错（`mode`=mount；none 是对照），之后照读。覆盖写推到 slot(h) 成了窗口里最小的那一刻
//! （E2 重读它：读出来自证不过，盘上就是没写过），每一版之后在盘面上跑池级 checker。
//! 用法：front_hole_eio <S> <h> <mount|none>
use std::cell::Cell;
use std::rc::Rc;

use e2_attack_proto::{build_plain, content, geometry, parameters, recording_of, red, FaultyDevice, WRITE_TIME};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let h: u64 = a[2].parse().expect("h");
    let mode = a[3].clone();
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let ring = 3 * s;
    assert!(h < ring, "前洞：环还没转过来");
    // 实例 1 写到 h − 1，h 那次覆盖写录下来、切在根槽写之前。
    let image = {
        let base = build_plain(s, h.saturating_sub(6).max(1));
        let mut devices = e2_attack_proto::devices_of(&base);
        let mounted = mount_writable(&parameters, &mut devices).expect("重开");
        let mut allocator = mounted.allocator.clone();
        let mut current = mounted.current.clone().into_file_version().expect("有文件");
        let instance = mounted.output.instance;
        assert!(current.root.checkpoint_txg.0 < h, "重开写行与暖机之后还没到 h：换小一点的 h");
        while current.root.checkpoint_txg.0 + 1 < h {
            let salt = current.root.checkpoint_txg.0 + 1;
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
        }
        let before = e2_attack_proto::pool_of(&devices);
        let stream = SharedStream::retaining_contents();
        let mut recording = recording_of(&before, &stream);
        {
            let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
            let out = publish_overwrite(&mut pool, &mut allocator.clone(), &current, FirstFile { content: &content(h), write_time_seconds: WRITE_TIME + h }, instance).expect("洞那一次");
            assert_eq!(out.root.checkpoint_txg.0, h);
        }
        let operations = stream.retained_operations();
        let root_write = operations.iter().position(|r| geo.classify(&r.operation) == StepKind::RootRecordFua).expect("根槽写");
        let mut image = before;
        image.apply(&operations[..root_write]);
        image
    };
    let x = target_for_publish(CheckpointTxg(h), parameters.geometry.root_ring_slots_per_region);
    let x_device = parameters.region_devices[usize::try_from(x.region).expect("区域")];
    let x_offset = slot_offset(x, parameters.geometry.fixed_structure_slot_spacing).0;
    println!("SETUP S={s} h={h} slot=({},{}) red_before_mount={:?}", x.region, x.slot, red(&image));
    let fault_mode = Rc::new(Cell::new(if mode == "mount" { 1 } else { 0 }));
    let hits = Rc::new(Cell::new(0u64));
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = recording_of(&image, &stream)
        .into_iter()
        .map(|(id, inner)| (id, FaultyDevice { device: id, inner, target: (x_device, x_offset, x_offset + 512), mode: fault_mode.clone(), flip_at: 40, hits: hits.clone() }))
        .collect();
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
    fault_mode.set(0);
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    let snap = || { let mut i = image.clone(); i.apply(&stream.retained_operations()); i };
    let mark = format!("RootRingSlot {{ region: {}, slot: {} }}", x.region, x.slot);
    let occupant = |allocator: &singlefs_core::allocator::PoolAllocator| {
        let text = format!("{:?}", allocator.root_ring_occupancy());
        text.find(&mark).map_or_else(|| "（表里没有这一槽）".to_string(), |i| text[i..].chars().take(110).collect())
    };
    println!("MOUNTED current={} x_in_table={} red={:?}", current.root.checkpoint_txg.0, occupant(&allocator), red(&snap()));
    let stop = std::env::var("UNTIL").ok().and_then(|v| v.parse().ok()).unwrap_or(h + 2 * ring);
    while current.root.checkpoint_txg.0 < stop {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
        let r = red(&snap());
        println!("PUBLISHED txg={} x_in_table={} red={r:?}", current.root.checkpoint_txg.0, occupant(&allocator));
    }
}
