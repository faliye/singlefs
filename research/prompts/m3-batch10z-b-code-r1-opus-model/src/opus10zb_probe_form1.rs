//! 攻方原型（m3-batch10z-b-code-r1 攻方腿，草稿副本里的，不进仓）：K2-2 第 1 形（K4-1 的 R）。
//! 造法照入库 `tests/common_unwitnessed_publish/mod.rs`：覆盖写 k 次后录 R（覆盖写一次），崩在 R 的根槽落盘之后、轮换之前；
//! 可写挂载那一次 R 写出的某一个单元读回全零（R 走读不成、退回 P），R 的根槽与账读得出，挂载那一刻按最新那条根（R）的表判有效、
//! 根环表记成有效根；写行之后 R 被抛弃。之后 R 的账（树表、映射根、分配记录树根）读报错：挂上之后（A）或只在挂载那一刻（B）。
mod common_unwitnessed_publish;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, devices_reading_back_all_zero_on, record_one_publish_after_overwrites,
    units_of_r_overwritten_on, DeviceOverASparseImage, PoolUnderTest, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable_with_space_admission, roll_back_by_a_forward_publish, unmount, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::NodePointer;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

#[derive(Clone)]
struct Fault {
    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    active: Rc<Cell<bool>>,
    hits: Rc<Cell<u64>>,
}
impl Fault {
    fn refuses(&self, d: DeviceIdentity, o: u64, n: u64) -> bool {
        let hit = self.active.get() && self.ranges.iter().any(|(rd, s, e)| *rd == d && o < *e && o + n > *s);
        if hit {
            self.hits.set(self.hits.get() + 1);
        }
        hit
    }
}
pub struct Dev {
    identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    fault: Fault,
}
impl BlockDevice for Dev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.fault.refuses(self.identity, offset.0, u64::try_from(buffer.len()).unwrap()) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("读故障")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}
impl DeviceOverASparseImage for Dev {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

fn short<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}").chars().take(150).collect::<String>().replace(' ', "")
}
fn reuse_violations(image: &MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image)
        .into_iter()
        .filter(|l| l.starts_with("I-7.4") || l.starts_with("I-4.8") || l.starts_with("I-7.2") || l.starts_with("I-2.1"))
        .map(|l| l.chars().take(60).collect())
        .collect();
    if v.is_empty() { "-".into() } else { v.join("|").replace(' ', "") }
}
fn account_ranges(root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    [root.tree_table, root.mapping_root, root.allocation_record_tree_root]
        .into_iter()
        .filter(|p| *p != NodePointer::empty_root())
        .flat_map(|p| p.locations)
        .map(|l| (l.device, l.slot.to_device_offset().0, l.slot.to_device_offset().0 + singlefs_format::SLOT_BYTES))
        .collect()
}

fn do_unmount(pool: &mut PoolUnderTest<Dev>) -> String {
    let Some(mut session) = pool.session.take() else { return "no-session".into() };
    match unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(r)) => format!("ok(unreadable={})", r.abandoned_roots_unreadable),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "ok-nothing".into(),
        Err(e) => format!("Err:{}", short(&e)),
    }
}
fn rollback_to(pool: &mut PoolUnderTest<Dev>, target: &RootRecord) -> String {
    let mut session = pool.session.take().unwrap();
    let singlefs_core::transaction::PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
    let outcome = roll_back_by_a_forward_publish(
        &pool.parameters, &mut pool.devices, &mut session.allocator, current,
        RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg },
    );
    pool.session = Some(session);
    match outcome { Ok(_) => "ok".into(), Err(e) => format!("Err:{}", short(&e)) }
}
fn write_n(pool: &mut PoolUnderTest<Dev>, n: usize, versions: &mut Vec<RootRecord>) -> String {
    let mut raises = 0;
    for index in 0..n {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(p) => {
                raises += p.floor_raises.len();
                versions.push(*pool.session().current.root());
            }
            Err(e) => return format!("Err@{}(raises={raises}):{}", index + 1, short(&e)),
        }
    }
    format!("ok(n={n},raises={raises})")
}

fn run_one(recorded: &RecordedPublish, r_root: &RootRecord, zero_offset: u64, variation: &str, mode: &str) -> String {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let group: Vec<&_> = recorded.unit_writes().into_iter().filter(|w| w.offset.0 == zero_offset).collect();
    let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&group);
    if variation == "B" {
        ranges.0.extend(account_ranges(r_root).into_iter().map(|(d, s, e)| (d, s, e - s)));
    }
    let mut mount_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let parameters = recorded.pool.parameters.clone();
    let mounted = match mount_writable_with_space_admission(&parameters, &mut mount_devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount=Err:{}", short(&e)),
    };
    let selected = (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0);
    let occupancy = mounted.allocator.root_ring_occupancy().unwrap();
    let r_slot = target_for_publish(r_root.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
    let kind = occupancy.opus_occupant_kind(r_slot);
    let cring = occupancy.opus_placements_named_on_ring_slot(r_slot).map_or(0, |p| p.len());
    let image_after: MemoryPool = MemoryPool {
        devices: mount_devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(),
        device_size_in_bytes: recorded.pool.device_bytes,
    };
    let fault = Fault { ranges: Rc::new(account_ranges(r_root)), active: Rc::new(Cell::new(variation == "A")), hits: Rc::new(Cell::new(0)) };
    let devices: Vec<(DeviceIdentity, Dev)> = common_unwitnessed_publish::plain_devices_on(&image_after)
        .into_iter()
        .map(|(identity, inner)| (identity, Dev { identity, inner, fault: fault.clone() }))
        .collect();
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool = PoolUnderTest {
        parameters: parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut versions = vec![*pool.session().current.root()];
    let mut steps = Vec::new();
    for step in mode.split('+') {
        let outcome = if let Some(n) = step.strip_prefix('w') {
            write_n(&mut pool, n.parse().unwrap(), &mut versions)
        } else if step == "unmount" {
            do_unmount(&mut pool)
        } else if step == "rollback" {
            if versions.len() >= 2 { let t = versions[versions.len() - 2]; rollback_to(&mut pool, &t) } else { "skip".into() }
        } else {
            unreachable!()
        };
        let stop = outcome.starts_with("Err");
        steps.push(format!("{step}={outcome}"));
        if stop || pool.session.is_none() {
            break;
        }
    }
    let image = pool.image();
    format!(
        "selected={selected:?} r_kind_at_mount={kind} cring_r_slot={cring} {} r_units_overwritten={} faulted_reads={} checker={}",
        steps.join(" "),
        units_of_r_overwritten_on(recorded, &image).len(),
        fault.hits.get(),
        reuse_violations(&image)
    )
}

#[test]
fn sweep() {
    let arm = singlefs_core::mount::opus_arm();
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let ks: Vec<u64> = env("OPUS_KS", "5").split(',').map(|s| s.parse().unwrap()).collect();
    let kinds: Vec<String> = env("OPUS_KINDS", "ow").split(',').map(String::from).collect();
    let variations: Vec<String> = env("OPUS_VARS", "none,A,B").split(',').map(String::from).collect();
    let modes: Vec<String> = env("OPUS_MODES", "unmount,w3+unmount,w12+unmount,w3+rollback+unmount,w12+rollback+w6+unmount").split(',').map(String::from).collect();
    let only_offsets: Option<Vec<u64>> = std::env::var("OPUS_OFFSETS").ok().map(|s| s.split(',').map(|x| x.parse().unwrap()).collect());
    for k in &ks {
        for kind in &kinds {
            let publish = if kind == "ow" { PublishRecordedAsR::Overwrite } else { PublishRecordedAsR::SequentialWriteOfThreeDataUnits };
            let recorded = record_one_publish_after_overwrites(*k, publish);
            let r_root = RootRecord::parse_slot(
                recorded.root_slot_write().bytes().expect("根槽写带字节"),
                &recorded.pool.parameters.filesystem_identifier,
            )
            .expect("R 自证");
            let mut offsets: Vec<u64> = recorded.unit_writes().iter().map(|w| w.offset.0).collect();
            offsets.sort_unstable();
            offsets.dedup();
            if let Some(only) = &only_offsets {
                offsets.retain(|o| only.contains(o));
            }
            for zero in &offsets {
                for variation in &variations {
                    for mode in &modes {
                        let line = run_one(&recorded, &r_root, *zero, variation, mode);
                        println!("OPUSF1 arm={arm} k={k} kind={kind} r={:?} p={:?} zero={zero} var={variation} mode={mode} {line}",
                            (r_root.instance.0, r_root.checkpoint_txg.0), (recorded.previous_key.0 .0, recorded.previous_key.1 .0));
                    }
                }
            }
        }
    }
}
