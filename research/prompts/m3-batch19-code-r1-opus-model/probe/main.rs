//! m3-batch19-code-r1 攻方原型（Opus 腿）。只在冻结副本的拷贝上编跑；core 打了臂开关与错实现补丁（patches/core-arms-and-mutants.diff：线程各自的臂 `b19_set_arm` / 环境变量 B19_ARM，错实现 B19_MUT）。
//! MODE=n34：窄盘起点（单元区 SLOTS 槽、第一个文件之后覆盖写 K 次、U>1 时顺序写长到 U 个单元再覆盖写缩成一个），录一次不注错的卸载，
//!   卸载写流按层 0 枚举域逐个崩溃状态（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开，每个状态跑 checker），
//!   每个状态上逐臂：N3 那一形再可写挂载（准入判着、影子账开、批大小 1），看写行与暖机写下的单元落不落在卸载之前现行那一版换下的槽 R 上、
//!   落不落在挂载进门时扣住的槽 H 上；N4 那一形再可写挂载（准入关、影子账开），第一次顺序写把文件写回 U 个单元，看结局。
//! MODE=n7：见文件后半。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use std::time::Instant;

use common_admission::*;
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches, mount_writable_with_test_only_switches_and_deletion_switches,
    unmount, MountError, ShadowLedger, Unmounted, b19_set_arm, B19_HELD_AT_ESTABLISH, B19_WARM_UPS,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::FirstFile;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes_and_entries, MemoryPool, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedPublishEntry, RecordingBlockDevice, SharedStream};

mod n7;
mod extra;
mod lone;

pub fn env_list(name: &str, default: &str) -> Vec<u64> {
    std::env::var(name).unwrap_or_else(|_| default.to_string()).split(',').filter(|s| !s.is_empty())
        .map(|s| s.trim().parse::<u64>().expect("十进制")).collect()
}
pub fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().map_or(default, |v| v.parse().expect("十进制"))
}

pub fn batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

pub fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is) in writes.iter().zip(persisted) {
        if *is {
            write.contents.apply_to(pool.devices.get_mut(&write.device).expect("盘"), write.offset);
        }
    }
    pool
}

pub fn short(text: String) -> String {
    text.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("").to_string()
}

/// N3 那一形的再挂：结局类（ok / err:成员 / panic）、写行与暖机写下的槽 ∩ R、∩ H、暖机次数、H 的大小。
fn n3_remount(image: &MemoryPool, pool: &PoolUnderTest<SparseBlockDevice>, released: &BTreeSet<u64>) -> (String, usize, usize, usize, usize) {
    B19_HELD_AT_ESTABLISH.with(|h| h.borrow_mut().clear());
    B19_WARM_UPS.with(|w| w.set(0));
    let mut devices = plain_devices_on(image);
    let result = catch_unwind(AssertUnwindSafe(|| {
        mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, batches_of_one())
    }));
    let held: BTreeSet<u64> = B19_HELD_AT_ESTABLISH.with(|h| h.borrow().iter().map(|(_, s)| *s).collect());
    let warm_ups = B19_WARM_UPS.with(|w| w.get());
    match result {
        Err(_) => ("panic".into(), 0, 0, warm_ups, held.len()),
        Ok(Err(error)) => (format!("err:{}", short(format!("{error:?}"))), 0, 0, warm_ups, held.len()),
        Ok(Ok(mounted)) => {
            let written: Vec<u64> = std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes)
                .filter_map(|v| v.version_of_trees())
                .flat_map(|v| v.units.iter().flat_map(|u| u.slot.0..u.slot.0 + u.identity.span_slots()))
                .collect();
            let in_r = written.iter().filter(|s| released.contains(s)).count();
            let in_h = written.iter().filter(|s| held.contains(s)).count();
            ("ok".into(), in_r, in_h, warm_ups, held.len())
        }
    }
}

/// N4 那一形：再挂（准入关、影子账开），第一次顺序写写回 `length` 字节。结局类。
fn n4_regrow(image: &MemoryPool, pool: &PoolUnderTest<SparseBlockDevice>, length: usize) -> String {
    let mut devices = plain_devices_on(image);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let mounted = match mount_writable_with_test_only_switches(&pool.parameters, &mut devices,
            SpaceAdmission::SkippedByTheTestOnlySwitch, ShadowLedger::On) {
            Ok(m) => m,
            Err(error) => return format!("mount-err:{}", short(format!("{error:?}"))),
        };
        let floor = mounted.current.root().rollback_floor;
        let (_, mut session) = MountedSession::of_the_mount(mounted);
        let time = pool.write_time_seconds + 1;
        let content = content_of(length, time);
        match session.publish_user_change(&mut devices, UserChange::SequentialWrite(FirstFile { content: &content, write_time_seconds: time })) {
            Ok(p) => format!("ok fr={} v={} floor_same={}", p.floor_raises.len(), p.versions_published.len(), session.current.root().rollback_floor == floor),
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(n)) => format!("nospace fr={} stop={}", n.floor_raises.len(), short(format!("{:?}", n.stop))),
            Err(other) => format!("refused:{}", short(format!("{other:?}"))),
        }
    }));
    outcome.unwrap_or_else(|_| "panic".into())
}

fn run_n34(slots: u64, overwrites: u64, units: u64, arms: &[u8]) {
    let t0 = Instant::now();
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let built = catch_unwind(AssertUnwindSafe(|| -> Result<PoolUnderTest<SparseBlockDevice>, String> {
        let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(width, slots);
        for i in 0..overwrites {
            pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{}", short(format!("{e:?}"))))?;
        }
        if units > 1 {
            pool.sequential_write((units as usize - 1) * data_unit_payload_capacity() + 100).map_err(|e| format!("grow:{}", short(format!("{e:?}"))))?;
            pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("shrink:{}", short(format!("{e:?}"))))?;
        }
        Ok(pool)
    }));
    let label = format!("S={slots} K={overwrites} U={units}");
    let pool = match built {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => { println!("START {label} SKIP {e}"); return; }
        Err(_) => { println!("START {label} SKIP panic"); return; }
    };
    let released: BTreeSet<u64> = pool.current_file_version().released.iter()
        .flat_map(|p| p.slot.0..p.slot.0 + p.span).collect();
    let image = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image).into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone()))).collect();
    let session = pool.session.as_ref().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let shadow = session.shadow_ledger;
    let parameters = pool.parameters.clone();
    let unmounted = stream.record_entry(RecordedPublishEntry::Unmount, || unmount(&parameters, &mut devices, &mut allocator, &mut current, shadow));
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&stream.retained_operations(), &stream.entry_spans(), &width.fixed_geometry());
    let states = closed_form_state_count(&segments);
    let kinds: Vec<String> = writes.iter().map(|w| format!("{}@{}", w.kind.name(), w.device.0)).collect();
    println!("START {label} txg={} released={:?} unmount={} writes={} segments={:?} closed_form_states={states} kinds={kinds:?}",
        session.current.root().checkpoint_txg.0, released, match &unmounted { Ok(_) => "Ok".to_string(), Err(e) => format!("Err:{}", short(format!("{e:?}"))) }, writes.len(), segments.iter().map(Vec::len).collect::<Vec<_>>());
    if states > env_u64("MAX_STATES", 1_000_000) { println!("ENUM_SKIPPED {label} states={states}"); return; }
    let Some(judged) = writes.iter().position(|w| w.kind == StepKind::RootRecordFua) else { println!("NO_ROOT_FUA {label}"); return; };
    let mut versions = pool.published_versions();
    // 卸载那一串抬 F 的空发布带的根：内容与现行那一版相同（层 0 的 oracle 按走到的根判读回）。
    if let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted {
        for publish in &raised.publishes {
            let root = publish.root;
            versions.push(singlefs_harness::memory_pool::PublishedVersion {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
                content: pool.content_of_the_current_version.clone(),
            });
        }
    }
    let regrow_length = (units as usize).saturating_sub(1) * data_unit_payload_capacity() + 100;
    // 第一段：层 0 枚举（每段都展开，每个状态恢复、跑 checker；线程数照 SINGLEFS_LAYER0_THREADS），观察者只记下每个状态的持久向量。
    let mut table: Option<Vec<RetainedWrite>> = None;
    let mut states_seen: Vec<Vec<bool>> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(&image, &writes, &segments, judged, &versions, &|_, _| true,
        &mut |crash, _report| {
            match &table {
                None => table = Some(crash.writes.to_vec()),
                Some(existing) => assert_eq!(existing.len(), crash.writes.len(), "每个状态的写表同一张"),
            }
            states_seen.push(crash.persisted.clone());
        });
    println!("ENUM {label} states={} observed={} violations={} failed_states={} checker_violated={:?} first_violation={:?}", tally.states, states_seen.len(), tally.violations, tally.failed_states, tally.checker_violated_states, tally.first_violation.as_ref().map(|v| v.chars().take(200).collect::<String>()));
    let table = table.expect("至少一个状态");
    // 第二段：每个状态上逐臂再挂，按 SINGLEFS_THREAD_CAP 个线程分（状态序号 mod 线程数），臂是线程各自的。
    let threads = env_u64("SINGLEFS_THREAD_CAP", 1).max(1) as usize;
    let base_len = writes.len();
    let results: Vec<(BTreeMap<String, u64>, Vec<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads).map(|worker| {
            let (table, states_seen, image, pool, released, arms) = (&table, &states_seen, &image, &pool, &released, arms);
            let label = label.clone();
            scope.spawn(move || {
                let mut histogram: BTreeMap<String, u64> = BTreeMap::new();
                let mut lines = Vec::new();
                for (index, persisted) in states_seen.iter().enumerate().filter(|(i, _)| i % threads == worker) {
                    let crashed = materialize(image, table, persisted);
                    let n = persisted.iter().take_while(|p| **p).count();
                    let is_prefix = persisted.iter().skip(n).all(|p| !*p) && n <= base_len;
                    for arm in arms {
                        b19_set_arm(*arm);
                        let (n3, in_r, in_h, warm_ups, held) = n3_remount(&crashed, pool, released);
                        let n4 = if units > 1 { n4_regrow(&crashed, pool, regrow_length) } else { "-".into() };
                        let key = format!("arm={arm} held>0={} n3={n3} inR>0={} inH>0={} wu={warm_ups} n4={n4}", held > 0, in_r > 0, in_h > 0);
                        *histogram.entry(key).or_default() += 1;
                        if env_u64("DUMP_LEGAL_NOSPACE", 0) == 1 && *arm == 0 && (n4.starts_with("nospace") || n4.starts_with("mount-err") || n4.starts_with("refused")) {
                            let bits: String = persisted.iter().map(|p| if *p { '1' } else { '0' }).collect();
                            lines.push(format!("LEGALFAIL {label} n=0 state={index} persisted={bits} held={held} wu={warm_ups} n4={n4}"));
                        }
                        if is_prefix && env_u64("PRINT_PREFIX", 1) == 1 {
                            lines.push(format!("PREFIX {label} n={n} state={index} arm={arm} held={held} n3={n3} inR={in_r} inH={in_h} wu={warm_ups} n4={n4}"));
                        }
                    }
                    b19_set_arm(0);
                }
                (histogram, lines)
            })
        }).collect();
        handles.into_iter().map(|h| h.join().expect("工作线程")).collect()
    });
    let mut histogram: BTreeMap<String, u64> = BTreeMap::new();
    let mut lines: Vec<String> = Vec::new();
    for (h, l) in results {
        for (k, c) in h { *histogram.entry(k).or_default() += c; }
        lines.extend(l);
    }
    lines.sort_by_key(|l| (l.split(" n=").nth(1).and_then(|r| r.split(' ').next()).and_then(|x| x.parse::<u64>().ok()).unwrap_or(0), l.clone()));
    for l in &lines { println!("{l}"); }
    for (key, count) in &histogram { println!("HIST {label} count={count} {key}"); }
    println!("DONE {label} elapsed={:.1}s", t0.elapsed().as_secs_f64());
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mode = std::env::var("MODE").unwrap_or_else(|_| "n34".into());
    match mode.as_str() {
        "n34" => {
            let arms: Vec<u8> = env_list("ARMS", "0,1,2,3,4,5,6").into_iter().map(|a| a as u8).collect();
            for shape in std::env::var("SHAPES").expect("SHAPES=S:K:U;…").split(';').filter(|s| !s.is_empty()) {
                let v: Vec<u64> = shape.split(':').map(|x| x.parse().expect("数")).collect();
                run_n34(v[0], v[1], v[2], &arms);
            }
        }
        "n7" => n7::run(),
        "qf" => extra::qf(),
        "c2" => extra::c2(),
        "lone" => lone::run(),
        other => panic!("MODE {other}"),
    }
}
