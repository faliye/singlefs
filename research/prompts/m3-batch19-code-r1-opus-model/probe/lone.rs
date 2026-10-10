//! MODE=lone：N3 那一形的起点（SHAPES 里的 S:K:U），卸载写流取前缀 n（LONE_N 里每一个），在那份崩溃镜像上按臂（ARMS）再可写挂载一次、录写流，
//! 再挂的写流按层 0 枚举域逐个崩溃状态（每段都展开、每个状态跑 checker），每个状态丢盘 0 / 丢盘 1，剩下那块盘只读挂载、整读每个活文件，
//! 与卸载之前同一块盘上读出的比（只比活文件的字节，不比走到的根）。读法照 `the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs`
//! 的 `whole_read_of_the_live_files`（只抄读法）。
use std::collections::BTreeMap;

use crate::common_admission::*;
use crate::{batches_of_one, env_list, materialize, short};
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{b19_set_arm, mount_writable_with_test_only_switches_and_deletion_switches, unmount, ShadowLedger, Unmounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes_and_entries, MemoryPool, PublishedVersion, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedPublishEntry, RecordingBlockDevice, SharedStream};

fn without(image: &MemoryPool, lost: u32) -> MemoryPool {
    MemoryPool { devices: image.devices.iter().filter(|(id, _)| id.0 != lost).map(|(id, d)| (*id, d.clone())).collect(), device_size_in_bytes: image.device_size_in_bytes }
}

fn live_files(image: &MemoryPool) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let ro = mount_read_only(image).map_err(|f| format!("只读挂载:{}", short(format!("{f:?}"))))?;
    let mut out = Vec::new();
    for record in ro.mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED { continue; }
        let file = ro.mounted.open_file(image, InodeNumber(record.inode)).map_err(|f| format!("打开:{}", short(format!("{f:?}"))))?;
        let read = file.read_at(image, FileOffsetInBytes(0), record.size).map_err(|f| format!("读:{}", short(format!("{f:?}"))))?;
        out.push((record.inode, read.bytes));
    }
    Ok(out)
}

fn recorded<F: FnOnce(&mut Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>) -> R, R>(image: &MemoryPool, entry: Option<RecordedPublishEntry>, f: F)
    -> (R, Vec<singlefs_harness::memory_pool::RetainedWrite>, Vec<Vec<usize>>) {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(image).into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone()))).collect();
    let result = match entry { Some(e) => stream.record_entry(e, || f(&mut devices)), None => f(&mut devices) };
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&stream.retained_operations(), &stream.entry_spans(), &HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry());
    (result, writes, segments)
}

pub fn run() {
    let arms: Vec<u8> = env_list("ARMS", "0,6").into_iter().map(|a| a as u8).collect();
    for shape in std::env::var("SHAPES").expect("SHAPES").split(';').filter(|s| !s.is_empty()) {
        let v: Vec<u64> = shape.split(':').map(|x| x.parse().unwrap()).collect();
        let (slots, overwrites, units) = (v[0], v[1], v[2]);
        let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(HistoryDeviceWidth::UnitAreaOf384Slots, slots);
        for _ in 0..overwrites { pool.overwrite(OVERWRITE_BYTES).expect("覆盖写"); }
        if units > 1 {
            pool.sequential_write((units as usize - 1) * data_unit_payload_capacity() + 100).expect("长");
            pool.overwrite(OVERWRITE_BYTES).expect("缩");
        }
        let image = pool.image();
        let before: BTreeMap<u32, Result<Vec<(u64, Vec<u8>)>, String>> = [0u32, 1].into_iter().map(|l| (l, live_files(&without(&image, l)))).collect();
        let readable: Vec<String> = before.iter().map(|(l, r)| format!("lost{l}={}", if r.is_ok() { "ok".to_string() } else { r.clone().err().unwrap() })).collect();
        println!("LONEBEFORE S={slots} K={overwrites} U={units} txg={} {readable:?}", pool.session().current.root().checkpoint_txg.0);
        if crate::env_u64("LONE_ONLY_IF_READABLE", 0) == 1 && before.values().any(|r| r.is_err()) { continue; }
        let session = pool.session.as_ref().unwrap();
        let (mut allocator, mut current, shadow, parameters) = (session.allocator.clone(), session.current.clone(), session.shadow_ledger, pool.parameters.clone());
        let (unmounted, writes_u, _) = recorded(&image, Some(RecordedPublishEntry::Unmount), |d| unmount(&parameters, d, &mut allocator, &mut current, shadow));
        let mut versions = pool.published_versions();
        if let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted {
            for p in &raised.publishes { versions.push(PublishedVersion { instance: p.root.instance, checkpoint_txg: p.root.checkpoint_txg, content: pool.content_of_the_current_version.clone() }); }
        }
        for n in env_list("LONE_N", "1,15,16") {
            let persisted: Vec<bool> = (0..writes_u.len()).map(|i| (i as u64) < n).collect();
            let crashed = materialize(&image, &writes_u, &persisted);
            for arm in &arms {
                b19_set_arm(*arm);
                let (mounted, writes_r, segments_r) = recorded(&crashed, None, |d| {
                    mount_writable_with_test_only_switches_and_deletion_switches(&pool.parameters, d, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, batches_of_one())
                });
                let mut versions2 = versions.clone();
                match &mounted {
                    Ok(m) => for version in std::iter::once(&m.output.row_publish).chain(&m.output.warm_up_publishes) {
                        let root = version.root();
                        versions2.push(PublishedVersion { instance: root.instance, checkpoint_txg: root.checkpoint_txg, content: pool.content_of_the_current_version.clone() });
                    },
                    Err(e) => { println!("LONE S={slots} K={overwrites} U={units} n={n} arm={arm} remount-err {}", short(format!("{e:?}"))); continue; }
                }
                let states = closed_form_state_count(&segments_r);
                let judged = writes_r.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("再挂写了根");
                let mut histogram: BTreeMap<String, u64> = BTreeMap::new();
                let tally = enumerate_layer0_selecting_versions_observing_each_state(&crashed, &writes_r, &segments_r, judged, &versions2, &|_, _| true,
                    &mut |crash, _| {
                        let image2 = materialize(crash.base, crash.writes, &crash.persisted);
                        for lost in [0u32, 1] {
                            let after = live_files(&without(&image2, lost));
                            let current_files = vec![(1u64, pool.content_of_the_current_version.clone())];
                            let class = match (&after, &before[&lost]) {
                                (Ok(a), Ok(b)) if a == b => "same".to_string(),
                                (Ok(a), Ok(_)) if *a == current_files => "current-version (newer than what this device read before)".to_string(),
                                (Ok(_), Ok(_)) => "DIFFERENT_BYTES".to_string(),
                                (Err(e), _) => format!("UNREADABLE {e}"),
                                (Ok(_), Err(e)) => format!("before-unreadable {e}"),
                            };
                            *histogram.entry(format!("lost={lost} {class}")).or_default() += 1;
                        }
                    });
                b19_set_arm(0);
                println!("LONE S={slots} K={overwrites} U={units} n={n} arm={arm} remount_writes={} segments={:?} closed_form_states={states} states={} checker_violated={:?} oracle_violations={}",
                    writes_r.len(), segments_r.iter().map(Vec::len).collect::<Vec<_>>(), tally.states, tally.checker_violated_states, tally.violations);
                for (k, c) in &histogram { println!("LONEH S={slots} K={overwrites} U={units} n={n} arm={arm} count={c} {k}"); }
            }
        }
    }
}
