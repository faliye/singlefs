//! m3-batch10z-d-code-r1 攻方腿原型（草稿副本里的，不进仓）：被抛弃根 C（崩溃恢复抛弃，造法照入库
//! `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 的 `pool_whose_newest_root_was_abandoned_by_a_crash_recovery`，
//! 之前覆盖写次数放开）之后再可写挂载一次（实例 4），故障三种：
//! - `slot-after`（K2-1，H1）：挂上之后 C 的根槽每次读都读回全零（`slot-after-err`：报读错）；
//! - `slot-zero-at-mount`（K2-3）：只在挂载那一刻 C 的根槽读回全零，挂上之后照读；
//! - `none`：对照。
//! 用户动作放开扫，终态与每一步之后都过池级 checker（只数 I-7.4 / I-4.8）。
#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;

use std::cell::Cell;
use std::rc::Rc;

use common_admission_pool::{
    checker_violations_on, device_bytes_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_device, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor_to_the_admission_ceiling, unmount,
    RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

struct Built {
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    image: MemoryPool,
    write_time_seconds: u64,
    abandoned_root: RootRecord,
    content_of_each_root: std::collections::BTreeMap<(singlefs_core::address::InstanceGeneration, singlefs_core::address::CheckpointTxg), Vec<u8>>,
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
fn root_slot_place(parameters: &MakeFilesystemParameters, root: &RootRecord) -> (DeviceIdentity, u64, u64) {
    let rs = target_for_publish(root.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
    let d = parameters.region_devices[usize::try_from(rs.region).unwrap()];
    let o = slot_offset(rs, parameters.geometry.fixed_structure_slot_spacing);
    (d, o.0, u64::from(parameters.geometry.physical_block_size))
}

/// 照入库造法：实例 1 覆盖写 `overwrites_before` 次，崩了再挂（取号 2），覆盖写一次得 C，藏 C 的根槽与数据单元、系统配置退回没见证 C，
/// 崩了再挂（取号 3，择根落到 C 之前那一条、写行抛弃 C），把藏的写回去，进程退出。
fn build(overwrites_before: usize, writes_in_instance_3: usize) -> Built {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    for _ in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写");
    }
    pool.crash_and_mount_writable().expect("取号 2");
    pool.overwrite(2500).expect("C");
    let abandoned = pool.current_file_version().clone();
    let parameters = pool.parameters.clone();
    let (d, o, n) = root_slot_place(&parameters, &abandoned.root);
    let mut hidden: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = Vec::new();
    let mut places = vec![(d, DeviceOffsetInBytes(o), usize::try_from(n).unwrap())];
    for pointer in &abandoned.data_pointers {
        for location in pointer.locations {
            places.push((location.device, location.slot.to_device_offset(), usize::try_from(singlefs_format::DATA_UNIT_BYTES).unwrap()));
        }
    }
    for (d, o, n) in places {
        let bytes = rd(&pool, d, o, n);
        wr(&mut pool, d, o, &vec![0u8; n]);
        hidden.push((d, o, bytes));
    }
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let rolled: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(
                pool.devices.as_slice(), *identity, spacing, &parameters.filesystem_identifier);
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] = <[_; 2]>::try_from(verified).expect("两槽");
            let g = newest.quantities.slot_generation;
            let mut not_rotated = older;
            not_rotated.quantities.slot_generation = g.checked_sub(2).unwrap();
            (*identity, DeviceOffsetInBytes((g % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing), not_rotated.to_slot())
        })
        .collect();
    for (d, o, b) in &rolled {
        wr(&mut pool, *d, *o, b);
    }
    let abandoning = pool.crash_and_mount_writable().expect("取号 3");
    assert!(abandoning.effective_root.checkpoint_txg < abandoned.root.checkpoint_txg);
    for (d, o, b) in &hidden {
        wr(&mut pool, *d, *o, b);
    }
    for _ in 0..writes_in_instance_3 {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 3 覆盖写");
    }
    pool.session = None;
    Built {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        image: pool.image(),
        write_time_seconds: pool.write_time_seconds,
        abandoned_root: abandoned.root,
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

/// C 的根槽那一段读故障：`zero` 读回全零，否则报读错；`active` 关着时照读。
#[derive(Clone)]
struct Fault {
    range: (DeviceIdentity, u64, u64),
    zero: bool,
    active: Rc<Cell<bool>>,
    hits: Rc<Cell<u64>>,
}
pub struct Dev {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    fault: Fault,
}
impl BlockDevice for Dev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let n = u64::try_from(buffer.len()).unwrap();
        let (d, s, l) = self.fault.range;
        let hit = self.fault.active.get() && d == self.identity && s < offset.0 + n && offset.0 < s + l;
        if hit {
            self.fault.hits.set(self.fault.hits.get() + 1);
            if !self.fault.zero {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other("读故障")));
            }
        }
        self.inner.read_at(offset, buffer)?;
        if hit {
            let from = usize::try_from(s.saturating_sub(offset.0)).unwrap();
            let to = usize::try_from((s + l).min(offset.0 + n) - offset.0).unwrap();
            buffer[from..to].fill(0);
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        // OPUS_FAULT_DROPS_ON_WRITE=1：这个进程写到那一槽（C 被盖掉、那一槽换成它自己的根），故障就撤（与入库读回全零那一层同一个模型）。
        let (d, s, l) = self.fault.range;
        let n = u64::try_from(bytes.len()).unwrap();
        if std::env::var("OPUS_FAULT_DROPS_ON_WRITE").is_ok() && d == self.identity && s < offset.0 + n && offset.0 < s + l {
            self.fault.active.set(false);
        }
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
    format!("{value:?}").chars().take(110).collect::<String>().replace(' ', "")
}
fn reuse_violations(image: &MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image)
        .into_iter()
        .filter(|l| l.starts_with("I-7.4") || l.starts_with("I-4.8"))
        .map(|l| l.chars().take(70).collect::<String>().replace(' ', ""))
        .collect();
    if v.is_empty() { "-".into() } else { v.join("|") }
}

fn step(pool: &mut PoolUnderTest<Dev>, action: char) -> String {
    let user = |r: Result<singlefs_core::mounted_session::UserChangePublished, singlefs_core::mounted_session::UserChangeRefused>| match r {
        Ok(p) => format!("ok(raises={})", p.floor_raises.len()),
        Err(e) => format!("Err:{}", short(&e)),
    };
    match action {
        'o' => user(pool.overwrite(OVERWRITE_BYTES)),
        'n' => user(pool.create_one_inode()),
        's' => user(pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())),
        'W' => {
            for count in 1..=40 {
                match pool.overwrite(OVERWRITE_BYTES) {
                    Ok(p) if !p.floor_raises.is_empty() => return format!("ok(after={count})"),
                    Ok(_) => {}
                    Err(e) => return format!("Err@{count}:{}", short(&e)),
                }
            }
            "no-raise(40)".into()
        }
        'F' => {
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
            let outcome = raise_rollback_floor_to_the_admission_ceiling(&parameters, &mut pool.devices, &mut session.allocator, current, ShadowLedger::On);
            pool.session = Some(session);
            match outcome {
                Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
                    pool.note_floor_raises(std::slice::from_ref(&raised));
                    "ok(raised)".into()
                }
                Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }) => "ok(at-ceiling)".into(),
                Err(e) => format!("Err:{}", short(&e)),
            }
        }
        'u' => {
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            match unmount(&parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On) {
                Ok(Unmounted::FloorRaisedToTheCurrentVersion(_)) => "ok".into(),
                Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "ok-nothing".into(),
                Err(e) => format!("Err:{}", short(&e)),
            }
        }
        other => panic!("动作 {other}"),
    }
}

fn run_one(built: &Built, fault: &str, actions: &str) -> String {
    let shared = Fault {
        range: root_slot_place(&built.parameters, &built.abandoned_root),
        zero: !fault.ends_with("-err"),
        active: Rc::new(Cell::new(fault.starts_with("slot-zero-at-mount") || fault.starts_with("slot-err-at-mount"))),
        hits: Rc::new(Cell::new(0)),
    };
    let mut devices: Vec<(DeviceIdentity, Dev)> = built
        .image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(built.image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, Dev { identity: *identity, inner: device, fault: shared.clone() })
        })
        .collect();
    let mounted = match mount_writable_with_space_admission(&built.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount=Err:{}", short(&e)),
    };
    let occupancy = mounted.allocator.root_ring_occupancy().expect("根环表");
    let recorded = occupancy
        .placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg)
        .map_or(-1, |p| i64::try_from(p.len()).unwrap());
    let abandoned_slots = occupancy.ring_slots_recorded_as_holding_an_abandoned_root().len();
    shared.active.set(fault.starts_with("slot-after"));
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content = built
        .content_of_each_root
        .get(&(output.effective_root.instance, output.effective_root.checkpoint_txg))
        .cloned()
        .unwrap_or_default();
    let mut content_of_each_root = built.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert((root.instance, root.checkpoint_txg), content.clone());
    }
    let mut pool = PoolUnderTest {
        parameters: built.parameters.clone(),
        device_bytes: built.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: built.write_time_seconds + 10_000,
        content_of_the_current_version: content,
        content_of_each_root,
    };
    let mut steps = Vec::new();
    let mut first_red = String::from("-");
    for (index, action) in actions.chars().enumerate() {
        let outcome = step(&mut pool, action);
        let stop = outcome.starts_with("Err") || pool.session.is_none();
        steps.push(format!("{action}={outcome}"));
        if first_red == "-" {
            let v = reuse_violations(&pool.image());
            if v != "-" {
                first_red = format!("{index}{action}:{v}");
            }
        }
        if stop {
            break;
        }
    }
    let image = pool.image();
    format!(
        "recorded_at_mount={recorded} abandoned_slots_in_table={abandoned_slots} faulted_reads={} {} first_red={first_red} final={}",
        shared.hits.get(),
        steps.join(" "),
        reuse_violations(&image)
    )
}

#[test]
fn opus_sweep() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let ks: Vec<usize> = env("OPUS_KS", "1").split(',').map(|s| s.parse().unwrap()).collect();
    let i3: Vec<usize> = env("OPUS_I3", "0").split(',').map(|s| s.parse().unwrap()).collect();
    let faults: Vec<String> = env("OPUS_FAULTS", "slot-after").split(',').map(String::from).collect();
    let alphabet: Vec<char> = env("OPUS_ALPHABET", "onsFW").chars().collect();
    let max_len: usize = env("OPUS_MAXLEN", "3").parse().unwrap();
    let mut actions: Vec<String> = vec![String::new()];
    let mut layer = vec![String::new()];
    for _ in 0..max_len {
        layer = layer.iter().flat_map(|p| alphabet.iter().map(move |c| format!("{p}{c}"))).collect();
        actions.extend(layer.iter().cloned());
    }
    let actions: Vec<String> = actions.into_iter().flat_map(|a| [a.clone(), format!("{a}u")]).filter(|a| !a.is_empty()).collect();
    for k in &ks {
        for w3 in &i3 {
            let built = build(*k, *w3);
            for fault in &faults {
                for a in &actions {
                    let line = run_one(&built, fault, a);
                    println!(
                        "OPUS10ZDX k={k} i3={w3} c={:?} fault={fault} actions={a} {line}",
                        (built.abandoned_root.instance.0, built.abandoned_root.checkpoint_txg.0)
                    );
                }
            }
        }
    }
}
