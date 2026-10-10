//! 攻方原型（m3-batch10z-b-code-r1 攻方腿，草稿副本里的，不进仓）：被抛弃根 C 记在根环表里之后，挂上之后读故障落在 C 的根槽（X1）
//! 或 C 的账（对照，同入库用例），用户动作（故障前覆盖写几次、之后卸载 / 写到推出抬 F / 管理员回退再卸载）放开扫。
//! 造盘照入库用例 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 的 `pool_whose_newest_root_was_abandoned_by_a_crash_recovery`。
mod common_admission;

use std::cell::Cell;
use std::rc::Rc;

use common_admission::{
    checker_violations_on, start_plain, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_space_admission, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::NodePointer;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::collections::BTreeMap;

const CONTENT_BYTES_OF_THE_ABANDONED_VERSION: usize = 2500;

struct Built {
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    image: MemoryPool,
    write_time_seconds: u64,
    abandoned_root: RootRecord,
}

fn rd(pool: &PoolUnderTest<SparseBlockDevice>, d: DeviceIdentity, o: DeviceOffsetInBytes, n: usize) -> Vec<u8> {
    let (_, b) = pool.devices.iter().find(|(i, _)| *i == d).unwrap();
    let mut v = vec![0u8; n];
    b.read_at(o, &mut v).unwrap();
    v
}
fn wr(pool: &mut PoolUnderTest<SparseBlockDevice>, d: DeviceIdentity, o: DeviceOffsetInBytes, bytes: &[u8]) {
    let (_, b) = pool.devices.iter_mut().find(|(i, _)| *i == d).unwrap();
    b.write_at(o, bytes, WriteDurability::Plain).unwrap();
}

fn root_slot_place(parameters: &MakeFilesystemParameters, root: &RootRecord) -> (DeviceIdentity, DeviceOffsetInBytes, usize) {
    let rs = target_for_publish(root.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
    let d = parameters.region_devices[usize::try_from(rs.region).unwrap()];
    let o = slot_offset(rs, parameters.geometry.fixed_structure_slot_spacing);
    (d, o, usize::try_from(parameters.geometry.physical_block_size).unwrap())
}

fn build(overwrites_before: usize) -> Built {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    for _ in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写");
    }
    pool.crash_and_mount_writable().expect("取号 2");
    pool.overwrite(CONTENT_BYTES_OF_THE_ABANDONED_VERSION).expect("C");
    let abandoned_version = pool.current_file_version().clone();
    let parameters = pool.parameters.clone();
    let mut hidden_places = vec![root_slot_place(&parameters, &abandoned_version.root)];
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).unwrap();
    for pointer in &abandoned_version.data_pointers {
        for location in pointer.locations {
            hidden_places.push((location.device, location.slot.to_device_offset(), data_unit_bytes));
        }
    }
    let mut hidden_bytes = Vec::new();
    for (d, o, n) in &hidden_places {
        let b = rd(&pool, *d, *o, *n);
        wr(&mut pool, *d, *o, &vec![0u8; *n]);
        hidden_bytes.push((*d, *o, b));
    }
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let rolled: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(
                pool.devices.as_slice(), *identity, slot_spacing, &parameters.filesystem_identifier);
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] = <[_; 2]>::try_from(verified).expect("两槽");
            let g = newest.quantities.slot_generation;
            let mut not_rotated = older;
            not_rotated.quantities.slot_generation = g.checked_sub(2).unwrap();
            (*identity, DeviceOffsetInBytes((g % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing), not_rotated.to_slot())
        })
        .collect();
    for (d, o, b) in &rolled {
        wr(&mut pool, *d, *o, b);
    }
    let abandoning = pool.crash_and_mount_writable().expect("取号 3");
    assert!(abandoning.effective_root.checkpoint_txg < abandoned_version.root.checkpoint_txg);
    for (d, o, b) in &hidden_bytes {
        wr(&mut pool, *d, *o, b);
    }
    pool.session = None;
    Built {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        image: pool.image(),
        write_time_seconds: pool.write_time_seconds,
        abandoned_root: abandoned_version.root,
    }
}

#[derive(Clone)]
struct Fault {
    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    active: Rc<Cell<bool>>,
    reads_refused: Rc<Cell<u64>>,
}
impl Fault {
    fn refuses(&self, d: DeviceIdentity, o: u64, n: u64) -> bool {
        let hit = self.active.get() && self.ranges.iter().any(|(rd, s, e)| *rd == d && o < *e && o + n > *s);
        if hit {
            self.reads_refused.set(self.reads_refused.get() + 1);
        }
        hit
    }
}
fn fault_on_account(root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    [root.tree_table, root.mapping_root, root.allocation_record_tree_root]
        .into_iter()
        .filter(|p| *p != NodePointer::empty_root())
        .flat_map(|p| p.locations)
        .map(|l| (l.device, l.slot.to_device_offset().0, l.slot.to_device_offset().0 + singlefs_format::SLOT_BYTES))
        .collect()
}
fn fault_on_root_slot(parameters: &MakeFilesystemParameters, root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    let (d, o, n) = root_slot_place(parameters, root);
    vec![(d, o.0, o.0 + u64::try_from(n).unwrap())]
}

struct Dev {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
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
    let text = format!("{value:?}");
    let cut: String = text.chars().take(150).collect();
    cut.replace(' ', "")
}

fn reuse_violations(image: &MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image)
        .into_iter()
        .filter(|l| l.starts_with("I-7.4") || l.starts_with("I-4.8"))
        .map(|l| l.chars().take(60).collect())
        .collect();
    if v.is_empty() { "-".into() } else { v.join("|").replace(' ', "") }
}

fn rollback_to(pool: &mut PoolUnderTest<Dev>, target: &RootRecord) -> String {
    let mut session = pool.session.take().unwrap();
    let singlefs_core::transaction::PoolVersion::WithFile(current) = &mut session.current else {
        panic!("带文件")
    };
    let outcome = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg },
    );
    pool.session = Some(session);
    match outcome {
        Ok(_) => "ok".into(),
        Err(e) => format!("Err:{}", short(&e)),
    }
}

fn do_unmount(pool: &mut PoolUnderTest<Dev>) -> String {
    let Some(mut session) = pool.session.take() else { return "no-session".into() };
    match unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(r)) => format!("ok(unreadable={})", r.abandoned_roots_unreadable),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "ok-nothing".into(),
        Err(e) => format!("Err:{}", short(&e)),
    }
}

/// 写到推出第一串抬 F 再写 3 次（至多 `at_most` 次）；交回 (结局, 写了几次, 推过几串抬 F)。
fn write_until_raise(pool: &mut PoolUnderTest<Dev>, at_most: usize) -> (String, usize, usize) {
    let mut raises = 0;
    let mut after = 0;
    for index in 0..at_most {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(published) => {
                raises += published.floor_raises.len();
                if raises > 0 {
                    after += 1;
                    if after > 3 {
                        return ("ok".into(), index + 1, raises);
                    }
                }
            }
            Err(e) => return (format!("Err@{}:{}", index + 1, short(&e)), index + 1, raises),
        }
    }
    ("ok-noraise".into(), at_most, raises)
}

fn run(built: &Built, fault_kind: &str, k_before: usize, mode: &str) -> String {
    let ranges = match fault_kind {
        "slot" => fault_on_root_slot(&built.parameters, &built.abandoned_root),
        "account" => fault_on_account(&built.abandoned_root),
        "none" => Vec::new(),
        _ => unreachable!(),
    };
    let fault = Fault { ranges: Rc::new(ranges), active: Rc::new(Cell::new(false)), reads_refused: Rc::new(Cell::new(0)) };
    let mut devices: Vec<(DeviceIdentity, Dev)> = common_admission::plain_devices_on(&built.image)
        .into_iter()
        .map(|(identity, inner)| (identity, Dev { identity, inner, fault: fault.clone() }))
        .collect();
    let mounted = match mount_writable_with_space_admission(&built.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount=Err:{}", short(&e)),
    };
    let occupancy = mounted.allocator.root_ring_occupancy().unwrap();
    let c_slot = target_for_publish(built.abandoned_root.checkpoint_txg, built.parameters.geometry.root_ring_slots_per_region);
    let recorded = occupancy
        .placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg)
        .map_or(0, |p| p.len());
    let kind = occupancy.opus_occupant_kind(c_slot);
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool = PoolUnderTest {
        parameters: built.parameters.clone(),
        device_bytes: built.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: built.write_time_seconds + 1000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut versions: Vec<RootRecord> = vec![*pool.session().current.root()];
    let mut pre = "ok".to_string();
    for index in 0..k_before {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(_) => versions.push(*pool.session().current.root()),
            Err(e) => {
                pre = format!("Err@{}:{}", index + 1, short(&e));
                break;
            }
        }
    }
    fault.active.set(true);
    let mut steps = Vec::new();
    for step in mode.split('+') {
        let kind_before = pool.session.as_ref().map_or("gone".to_string(), |s| {
            s.allocator.root_ring_occupancy().map_or("no-table".into(), |o| o.opus_occupant_kind(c_slot))
        });
        let outcome = match step {
            "unmount" => do_unmount(&mut pool),
            "write" => {
                let (o, n, r) = write_until_raise(&mut pool, 60);
                format!("{o}(n={n},raises={r})")
            }
            "rollback" => {
                if versions.len() >= 2 {
                    let target = versions[versions.len() - 2];
                    rollback_to(&mut pool, &target)
                } else {
                    "skip".into()
                }
            }
            _ => unreachable!(),
        };
        steps.push(format!("{step}[{kind_before}]={outcome}"));
        if pool.session.is_none() && step != "unmount" {
            break;
        }
    }
    let still = pool.session.as_ref().map_or("gone".to_string(), |s| {
        s.allocator.root_ring_occupancy().map_or("no-table".into(), |o| o.opus_occupant_kind(c_slot))
    });
    format!(
        "c={:?} kind_at_mount={kind} recorded={recorded} pre={pre} {} slot_now={still} faulted_reads={} checker={}",
        (built.abandoned_root.instance.0, built.abandoned_root.checkpoint_txg.0),
        steps.join(" "),
        fault.reads_refused.get(),
        reuse_violations(&pool.image())
    )
}

#[test]
fn sweep() {
    let arm = singlefs_core::mount::opus_arm();
    let starts: Vec<usize> = std::env::var("OPUS_STARTS")
        .unwrap_or_else(|_| "1".into())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let faults: Vec<String> = std::env::var("OPUS_FAULTS").unwrap_or_else(|_| "slot,account".into()).split(',').map(String::from).collect();
    let modes: Vec<String> = std::env::var("OPUS_MODES")
        .unwrap_or_else(|_| "unmount,write+unmount,rollback+unmount,rollback+write+unmount,write+rollback+unmount".into())
        .split(',')
        .map(String::from)
        .collect();
    let ks: Vec<usize> = std::env::var("OPUS_KS").unwrap_or_else(|_| "0,1,2,3,5,8".into()).split(',').map(|s| s.parse().unwrap()).collect();
    for start in &starts {
        let built = build(*start);
        for fault_kind in &faults {
            for k in &ks {
                for mode in &modes {
                    let line = run(&built, fault_kind, *k, mode);
                    println!("OPUSX1 arm={arm} start={start} fault={fault_kind} k={k} mode={mode} {line}");
                }
            }
        }
    }
}
