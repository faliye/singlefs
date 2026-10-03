//! m3-root-ring-code-r1 攻方原型 K2-b（抬 F 那一遍）：抬 F 读根环时，挂载那一刻读不出、之后仍读不出的槽当没有根（不在
//! `ring_slots_known_to_hold_a_root` 里，不重读、不拒），于是回收门槛 max(新 F, 窗口里读得出的最旧有效根) 越过那一槽里的根。
//! 造法：窗口最旧端先留两个洞（两次挂载都崩在写行那次根槽之前：洞那一槽留着上一圈的根，出了窗口），洞后面紧跟一条根 X，
//! 下一次挂载起 X 那一槽一直读不出；空发布推到 X 前面那两个洞成了窗口最旧的两格，准入抬 F（推两次就落满两块盘，盖掉的是两个洞），
//! 再发一次空发布，看它写的单元有没有盖到 X 那条根引用的槽、盘面上 checker 红不红。
//! 用法：raise_floor_hidden_slot <S> <P：第一个实例覆盖写到的 txg> <mode：persist|none>
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use rr_attack_proto::{build_plain, crash_a_mount_before_its_row_root, geometry, parameters, publish_empty, recording_of, HidingDevice};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::mount::{mount_writable, raise_rollback_floor_to_the_admission_ceiling, ShadowLedger};
use singlefs_core::recovery::{allocation_records_under_root, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::PoolWriter;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn red(image: &MemoryPool) -> Vec<String> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(text) => Some(format!("{name}：{}", text.chars().take(150).collect::<String>())),
            _ => None,
        })
        .collect()
}

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

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let p: u64 = a[2].parse().expect("P");
    let mode = a[3].clone();
    let extra_holes: u64 = std::env::var("HOLES").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let ring = 3 * s;
    let (mut image, _) = build_plain(s, p);
    let mut holes = Vec::new();
    for _ in 0..extra_holes {
        let (crashed, row_txg) = crash_a_mount_before_its_row_root(s, &image);
        holes.push(row_txg);
        image = crashed;
    }
    println!("HOLES {holes:?}");
    // 第三次挂载：写行那一版的根 X 落盘，接着发 m 次空发布，正常退出（不卸载）。
    let x_txg;
    {
        let mut devices = rr_attack_proto::devices_of(&image);
        let mounted = mount_writable(&parameters, &mut devices).expect("第三次挂载");
        x_txg = mounted.output.row_publish.root().checkpoint_txg.0;
        let mut allocator = mounted.allocator.clone();
        let mut current = mounted.current.clone().into_file_version().expect("有文件");
        let instance = mounted.output.instance;
        let m: u64 = std::env::var("AFTER_X").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
        for _ in 0..m {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_empty(&mut pool, &mut allocator, &current, instance).expect("空发布");
        }
        image = rr_attack_proto::pool_of(&devices);
        println!("LIFE3 instance={} x_txg={x_txg} newest={}", instance.0, current.root.checkpoint_txg.0);
    }
    let x_slot = target_for_publish(CheckpointTxg(x_txg), parameters.geometry.root_ring_slots_per_region);
    let x_device = parameters.region_devices[usize::try_from(x_slot.region).expect("区域")];
    let x_offset = slot_offset(x_slot, parameters.geometry.fixed_structure_slot_spacing).0;
    let x_root = rr_attack_proto::readable_root_at_txg(&image, x_txg).expect("X 落盘了");
    let x_refs = placements_of(&image, &x_root);
    println!("X txg={x_txg} slot=({},{}) device={} referenced_slots={}", x_slot.region, x_slot.slot, x_device.0, x_refs.len());
    // 第四次挂载：X 那一槽读不出（persist：整条命）。
    let hidden: Rc<RefCell<Vec<_>>> = Rc::new(RefCell::new(Vec::new()));
    if mode != "none" {
        hidden.borrow_mut().push((x_device, x_offset, x_offset + 512));
    }
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = recording_of(&image, &stream)
        .into_iter()
        .map(|(id, rec)| (id, HidingDevice { device: id, inner: rec, hidden: hidden.clone() }))
        .collect();
    let mounted = mount_writable(&parameters, &mut devices).expect("第四次挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    let snap = || { let mut i = image.clone(); i.apply(&stream.retained_operations()); i };
    println!("LIFE4 instance={} row={} red={:?}", instance.0, current.root.checkpoint_txg.0, red(&snap()));
    // 空发布推到锚 A = 第一个洞 + 3S − 1：X 前面的两个洞成了窗口最旧的两格。
    let anchor = holes[0] + ring - 1;
    while current.root.checkpoint_txg.0 < anchor {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_empty(&mut pool, &mut allocator, &current, instance).expect("空发布");
    }
    println!("ANCHOR {} red={:?}", current.root.checkpoint_txg.0, red(&snap()));
    let before_raise = current.root.checkpoint_txg.0;
    match raise_rollback_floor_to_the_admission_ceiling(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On) {
        Ok(raised) => println!("RAISED {}", format!("{raised:?}").chars().take(300).collect::<String>()),
        Err(e) => println!("RAISE_ERR {}", format!("{e:?}").chars().take(300).collect::<String>()),
    }
    let image_after_raise = snap();
    println!("AFTER_RAISE newest={} (was {before_raise}) x_still_on_disk={:?} red={:?}", current.root.checkpoint_txg.0, rr_attack_proto::readable_root_at_txg(&image_after_raise, x_txg).map(|r| r.checkpoint_txg.0), red(&image_after_raise));
    for step in 0..std::env::var("STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(3u64) {
        let ops_before = stream.retained_operations().len();
        let base = snap();
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = if std::env::var("STEP_OVERWRITE").is_ok_and(|v| v == "1") {
            let salt = 5000 + step;
            singlefs_core::transaction::publish_overwrite(&mut pool, &mut allocator, &current, singlefs_core::transaction::FirstFile { content: &rr_attack_proto::content(salt), write_time_seconds: rr_attack_proto::WRITE_TIME + salt }, instance).expect("覆盖写")
        } else {
            publish_empty(&mut pool, &mut allocator, &current, instance).expect("空发布")
        };
        let operations = stream.retained_operations()[ops_before..].to_vec();
        let (writes, _) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
        let mut overwrote = Vec::new();
        for w in writes.iter().filter(|w| w.kind == StepKind::UnitWrite) {
            let first = w.offset.0 / 16384;
            let last = (w.offset.0 + w.length_in_bytes().max(1) - 1) / 16384;
            for slot in first..=last {
                if x_refs.contains(&(w.device.0, slot)) { overwrote.push((w.device.0, slot)); }
            }
        }
        let enum_steps: Vec<u64> = std::env::var("ENUM_STEPS").ok().map_or(Vec::new(), |v| v.split(':').map(|x| x.parse().expect("步")).collect());
        if enum_steps.contains(&step) {
            let (writes, segments) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geo);
            let count = singlefs_harness::memory_pool::closed_form_state_count(&segments);
            println!("ENUM step={step} txg={} closed_form_state_count={count}", current.root.checkpoint_txg.0);
            assert!(count <= 1_000_000, "一次全量不超过约 10^6 个状态");
            let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
            let tally = singlefs_checker_tier::crash::enumerate_layer0_versions(&base, &writes, &segments, root_index, &[]);
            let wanted = ["I-3.1", "I-7.4", "I-2.1", "I-4.8", "I-7.9", "I-3.11"];
            let counts: Vec<String> = wanted.iter().map(|n| format!("{n}={}/{}", tally.checker_evaluated_states.get(n).copied().unwrap_or(0), tally.checker_violated_states.get(n).copied().unwrap_or(0))).collect();
            let all_red: Vec<String> = tally.checker_violated_states.iter().map(|(n, c)| format!("{n}:{c}")).collect();
            println!("ENUM_TALLY step={step} states={} checker_evaluated/violated {} all_violated={all_red:?}", tally.states, counts.join(" "));
        }
        // 崩在这次发布的根槽写之前：单元写全落、根没落。
        let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
        let mut crash = base.clone();
        crash.apply_writes(&writes[..root_index]);
        println!("STEP {step} txg={} unit_writes_over_x_refs={overwrote:?} x_on_disk={:?} crash_before_root_red={:?} after_red={:?}", current.root.checkpoint_txg.0, rr_attack_proto::readable_root_at_txg(&base, x_txg).map(|r| r.checkpoint_txg.0), red(&crash), red(&snap()));
    }
    let _ = DeviceIdentity(0);
}
