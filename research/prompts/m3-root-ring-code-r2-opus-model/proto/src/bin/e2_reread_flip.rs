//! K1 攻 E2 重读的结局「读得出而自证不过 ⇒ 当没有根」：根环槽 X 住着一条有效根 h（盘上字节完好），
//! 实例 2 的可写挂载那一刻读 X 报读错（按 E2 记成能住的最大 txg h）；之后读 X 按 `mode` 处置：
//!   flip  —— 每次读 X 读出来改一个字节（读故障，盘上不变），所以 X 成了窗口里最小的那个时那一次重读自证不过；
//!   clean —— 读得出（对照）；none —— 挂载那一刻也读得出（对照）。
//! 覆盖写推到 txg h + 3S − 1（X 是窗口里最小的那一刻，记这条根时重读 X），然后读故障撤掉，再发 txg h + 3S 那一次覆盖写：
//! 录它的写，崩在它的根槽写之前（X 还住着 h），看单元写有没有盖到 h 引用的槽、盘面上 checker 红不红；再把那一次发布的
//! 层 0 枚举域跑满（`enumerate_layer0_versions`，每段都展开）。
//! 用法：e2_reread_flip <S> <P：实例 1 覆盖写到的 txg> <back：h = P − back> <flip|clean|none> [enum]
use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

use e2_attack_proto::{build_plain, content, geometry, parameters, recording_of, red, FaultyDevice, WRITE_TIME};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{allocation_records_under_root, choose_system_configuration, readable_roots_with_ring_slots};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn root_at(image: &MemoryPool, txg: u64) -> Option<singlefs_core::root_record::RootRecord> {
    let configuration = choose_system_configuration(image).ok()?;
    let i = &configuration.immutable;
    readable_roots_with_ring_slots(image, &i.region_devices, &i.sizes, &i.filesystem_identifier)
        .into_iter()
        .map(|(_, root)| root)
        .find(|root| root.checkpoint_txg.0 == txg)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let p: u64 = a[2].parse().expect("P");
    let back: u64 = a[3].parse().expect("back");
    let mode = a[4].clone();
    let enumerate = a.get(5).is_some_and(|x| x == "enum");
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let ring = 3 * s;
    let image = build_plain(s, p);
    let h = p - back;
    let x = target_for_publish(CheckpointTxg(h), parameters.geometry.root_ring_slots_per_region);
    let x_device = parameters.region_devices[usize::try_from(x.region).expect("区域")];
    let x_offset = slot_offset(x, parameters.geometry.fixed_structure_slot_spacing).0;
    let h_root = root_at(&image, h).expect("h 落盘了");
    let h_refs: BTreeSet<(u32, u64)> = allocation_records_under_root(&image, &h_root)
        .expect("h 的账")
        .iter()
        .filter(|r| !r.is_released)
        .flat_map(|r| (r.slot.0..r.slot.0 + u64::from(r.span_slots)).map(move |s| (r.device.0, s)))
        .collect();
    println!("SETUP S={s} P={p} h={h} slot=({},{}) device={} h_referenced_slots={}", x.region, x.slot, x_device.0, h_refs.len());
    let fault_mode = Rc::new(Cell::new(if mode == "none" { 0 } else { 1 }));
    let hits = Rc::new(Cell::new(0u64));
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = recording_of(&image, &stream)
        .into_iter()
        .map(|(id, inner)| (id, FaultyDevice { device: id, inner, target: (x_device, x_offset, x_offset + 512), mode: fault_mode.clone(), flip_at: 40, hits: hits.clone() }))
        .collect();
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
    fault_mode.set(if mode == "flip" { 2 } else { 0 });
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    let snap = || { let mut i = image.clone(); i.apply(&stream.retained_operations()); i };
    let mark = format!("RootRingSlot {{ region: {}, slot: {} }}", x.region, x.slot);
    let occupant = |allocator: &singlefs_core::allocator::PoolAllocator| {
        let text = format!("{:?}", allocator.root_ring_occupancy());
        text.find(&mark).map_or_else(|| "（表里没有这一槽）".to_string(), |i| text[i..].chars().take(120).collect())
    };
    println!("MOUNTED instance={} current={} hits={} x_in_table={} red={:?}", instance.0, current.root.checkpoint_txg.0, hits.get(), occupant(&allocator), red(&snap()));
    while current.root.checkpoint_txg.0 < h + ring - 1 {
        let salt = current.root.checkpoint_txg.0 + 1;
        // FLIP_ONLY_AT_ANCHOR=1：读故障只在发 txg h + 3S − 1 那一次里有（瞬时），之前之后照读。
        let only_at_anchor = std::env::var("FLIP_ONLY_AT_ANCHOR").is_ok_and(|v| v == "1");
        if mode == "flip" && only_at_anchor {
            fault_mode.set(if salt == h + ring - 1 { 2 } else { 0 });
        }
        let hits_before = hits.get();
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
        println!("PUBLISHED txg={} x_reads={} x_in_table={}", current.root.checkpoint_txg.0, hits.get() - hits_before, occupant(&allocator));
    }
    println!("ANCHOR txg={} hits={} x_in_table={} red={:?}", current.root.checkpoint_txg.0, hits.get(), occupant(&allocator), red(&snap()));
    fault_mode.set(0);
    if std::env::var("ROLLBACK_AT_ANCHOR").is_ok_and(|v| v == "1") {
        // 读故障撤掉之后管理员回退到 (实例 1, txg h)：h 在窗口里、按实例表有效、带文件。
        let target = singlefs_core::mount::RollbackTarget { instance: singlefs_core::address::InstanceGeneration(1), checkpoint_txg: CheckpointTxg(h) };
        let result = singlefs_core::mount::roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, target);
        println!("ROLLBACK to=(1,{h}) result={}", format!("{:?}", result.as_ref().map(|_| current.root.checkpoint_txg.0)).chars().take(400).collect::<String>());
        let after = snap();
        let read = singlefs_core::recovery::recover(&after, singlefs_core::recovery::JournalPolicy::Consult).outcome;
        let ok = matches!(&read, singlefs_core::recovery::RecoveryOutcome::FileRead { content: c, .. } if *c == content(h));
        println!("AFTER_ROLLBACK newest={} reads_version_h_content={ok} red={:?}", current.root.checkpoint_txg.0, red(&after));
        for step in 0..3u64 {
            let salt = 7000 + step;
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            match publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance) {
                Ok(next) => current = next,
                Err(e) => { println!("STEP_ERR {}", format!("{e:?}").chars().take(300).collect::<String>()); break; }
            }
            println!("AFTER_STEP txg={} red={:?}", current.root.checkpoint_txg.0, red(&snap()));
        }
        return;
    }
    let base = snap();
    let ops_before = stream.retained_operations().len();
    {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
    }
    let operations = stream.retained_operations()[ops_before..].to_vec();
    let (writes, segments) = writes_and_segments(&operations, &geo);
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let mut over = Vec::new();
    for w in writes.iter().filter(|w| w.kind == StepKind::UnitWrite) {
        let first = w.offset.0 / 16384;
        let last = (w.offset.0 + w.length_in_bytes().max(1) - 1) / 16384;
        for slot in first..=last {
            if h_refs.contains(&(w.device.0, slot)) { over.push((w.device.0, slot)); }
        }
    }
    let mut crash = base.clone();
    crash.apply_writes(&writes[..root_index]);
    println!("NEXT txg={} unit_writes_over_h_refs={over:?} h_on_disk_at_crash={:?} crash_before_root_red={:?}", current.root.checkpoint_txg.0, root_at(&crash, h).map(|r| r.checkpoint_txg.0), red(&crash));
    if enumerate {
        let count = closed_form_state_count(&segments);
        println!("ENUM txg={} segments={} closed_form_state_count={count}", current.root.checkpoint_txg.0, segments.len());
        assert!(count <= 1_100_000, "一次全量不超过约 10^6 个状态");
        let tally = singlefs_checker_tier::crash::enumerate_layer0_versions(&base, &writes, &segments, root_index, &[]);
        let all: Vec<String> = tally.checker_violated_states.iter().map(|(n, c)| format!("{n}:{c}")).collect();
        println!("ENUM_TALLY txg={} states={} checker_violated={all:?}", current.root.checkpoint_txg.0, tally.states);
        for (name, first) in &tally.checker_first_violation {
            println!("ENUM_FIRST {name} {}", first.chars().take(200).collect::<String>());
        }
    }
}
