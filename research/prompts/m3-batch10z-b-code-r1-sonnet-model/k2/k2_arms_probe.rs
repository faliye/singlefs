//! 草稿：K2-2 两臂量法（第 1 形 R：可写挂载那一刻按最新那条根的表判有效、写行之后才被抛弃）。
mod common_unwitnessed_publish;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, record_one_publish_after_overwrites, units_of_r_overwritten_on,
    DeviceOverASparseImage, PoolUnderTest, PublishRecordedAsR, RecordedPublish, OVERWRITE_BYTES,
    plain_devices_on,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{mount_writable_with_space_admission, unmount, ShadowLedger};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::NodePointer;
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Fault {
    Off,
    Error,
}

#[derive(Clone)]
struct Shared {
    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    mode: Rc<Cell<Fault>>,
    zero_ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    zero_active: Rc<Cell<bool>>,
}

struct Dev {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    shared: Shared,
}

impl BlockDevice for Dev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let len = u64::try_from(buffer.len()).unwrap();
        if self.shared.mode.get() == Fault::Error
            && self.shared.ranges.iter().any(|(d, s, e)| *d == self.identity && offset.0 < *e && offset.0 + len > *s)
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("probe fault")));
        }
        self.inner.read_at(offset, buffer)?;
        if self.shared.zero_active.get() {
            for (d, s, l) in self.shared.zero_ranges.iter() {
                if *d == self.identity && *s < offset.0 + len && offset.0 < *s + *l {
                    let from = usize::try_from(s.saturating_sub(offset.0)).unwrap();
                    let to = usize::try_from((s + l).min(offset.0 + len) - offset.0).unwrap();
                    buffer[from..to].fill(0);
                }
            }
        }
        Ok(())
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

fn ranges_of_the_account_of(root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    [root.tree_table, root.mapping_root, root.allocation_record_tree_root]
        .into_iter()
        .filter(|p| *p != NodePointer::empty_root())
        .flat_map(|p| p.locations)
        .map(|l| {
            let s = l.slot.to_device_offset().0;
            (l.device, s, s + singlefs_format::SLOT_BYTES)
        })
        .collect()
}

/// fault_at_mount / fault_after_mount：R 的账（树表、映射根、分配记录树根）在挂载那一刻、挂载之后是否读报错。
fn run(label: &str, kind: PublishRecordedAsR, overwrites_before: u64, fault_at_mount: bool, fault_after_mount: bool, max_overwrites: usize, zero_unit: Option<usize>) {
    let recorded: RecordedPublish = record_one_publish_after_overwrites(overwrites_before, kind);
    let r_root = recorded.pool.current_file_version().root;
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let shared = Shared {
        ranges: Rc::new(ranges_of_the_account_of(&r_root)),
        mode: Rc::new(Cell::new(if fault_at_mount { Fault::Error } else { Fault::Off })),
        zero_ranges: Rc::new(match zero_unit {
            None => Vec::new(),
            Some(i) => {
                let writes = recorded.unit_writes();
                let offset = writes[i].offset;
                writes.iter().filter(|w| w.offset == offset).map(|w| (w.device, w.offset.0, w.length_in_bytes())).collect()
            }
        }),
        zero_active: Rc::new(Cell::new(true)),
    };
    let mut devices: Vec<(DeviceIdentity, Dev)> = plain_devices_on(&crash_image)
        .into_iter()
        .map(|(identity, inner)| (identity, Dev { identity, inner, shared: shared.clone() }))
        .collect();
    let mounted = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => {
            println!("RESULT {label}: 可写挂载 M1 被拒: {e:?}");
            return;
        }
    };
    let selected = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
    let c_len_for_r = mounted.allocator.root_ring_occupancy().and_then(|o| o.c_arm_placements_for_the_valid_root(r_root.checkpoint_txg)).map(|v| v.len());
    let recorded_for_r = mounted.allocator.root_ring_occupancy().map(|o| o.placements_recorded_for_the_abandoned_root(r_root.instance, r_root.checkpoint_txg));
    shared.mode.set(if fault_after_mount { Fault::Error } else { Fault::Off });
    shared.zero_active.set(false);
    let (_out, session) = MountedSession::of_the_mount(mounted);
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let isolated_of = |pool: &PoolUnderTest<Dev>| -> Vec<u64> { pool.session().allocator.devices.iter().map(|d| d.isolated_slots()).collect() };
    let set_of = |pool: &PoolUnderTest<Dev>| -> Vec<Vec<usize>> { pool.session().allocator.devices.iter().map(|d| d.isolated_set_for_probe()).collect() };
    let mut set_after_first_raise: Option<Vec<Vec<usize>>> = None;
    let isolated_after_mount = isolated_of(&pool);
    let mut isolated_after_first_raise: Option<(usize, Vec<u64>)> = None;
    let mut refused: Option<(usize, String)> = None;
    let mut raises = 0usize;
    let mut first_violation: Option<(usize, Vec<String>)> = None;
    for index in 0..max_overwrites {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(p) => {
                raises += p.floor_raises.len();
                if isolated_after_first_raise.is_none() && !p.floor_raises.is_empty() {
                    isolated_after_first_raise = Some((index + 1, isolated_of(&pool)));
                    set_after_first_raise = Some(set_of(&pool));
                }
                if first_violation.is_none() && index % 3 == 2 {
                    let v: Vec<String> = checker_violations_on(&pool.image()).into_iter().filter(|v| v.starts_with("I-7.4") || v.starts_with("I-4.8")).collect();
                    if !v.is_empty() {
                        first_violation = Some((index + 1, v));
                    }
                }
            }
            Err(e) => {
                let s = format!("{e:?}");
                refused = Some((index + 1, s.chars().take(1200).collect()));
                break;
            }
        }
    }
    println!("ISOSET {label} {:?}", set_after_first_raise.as_ref().map(|v| v.iter().map(|d| { let mut h = 0u64; for x in d { h = h.wrapping_mul(1099511628211).wrapping_add(*x as u64 + 1); } (d.len(), h) }).collect::<Vec<_>>()));
    let image_after_writes = pool.image();
    let v_writes: Vec<String> = checker_violations_on(&image_after_writes).into_iter().filter(|v| v.starts_with("I-7.4") || v.starts_with("I-4.8")).collect();
    let overwritten = units_of_r_overwritten_on(&recorded, &image_after_writes);
    let mut session = pool.session.take().unwrap();
    let un = unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
    let un_s = match &un {
        Ok(_) => "做成".to_string(),
        Err(e) => format!("拒 {:?}", e).chars().take(1200).collect::<String>(),
    };
    let image_after_unmount = pool.image();
    let v_unmount: Vec<String> = checker_violations_on(&image_after_unmount).into_iter().filter(|v| v.starts_with("I-7.4") || v.starts_with("I-4.8")).collect();
    let overwritten_after_unmount = units_of_r_overwritten_on(&recorded, &image_after_unmount);
    println!(
        "RESULT {label}: arm_c={} selected_is_P={} recorded_for_R={:?} C_for_valid_R={:?} | 会话写 {max_overwrites} 次上限：被拒={:?} 推抬F串数={raises} 隔离槽数(挂后/首次推抬F后)={isolated_after_mount:?}/{isolated_after_first_raise:?} 逐步checker首个违例={first_violation:?} R单元被覆盖(写后)={} 违例(写后)={:?} | 卸载={un_s} R单元被覆盖(卸载后)={} 违例(卸载后)={:?}",
        std::env::var("SINGLEFS_ARM_C").is_ok(),
        selected == recorded.previous_key,
        recorded_for_r.map(|o| o.map(|v| v.len())),
        c_len_for_r,
        refused,
        overwritten.len(), v_writes,
        overwritten_after_unmount.len(), v_unmount,
    );
}

fn sweep(tag: &str, kind: PublishRecordedAsR, fault_at_mount: bool, fault_after_mount: bool) {
    let n = record_one_publish_after_overwrites(1, kind).unit_writes().len();
    println!("unit_writes of R ({tag}) = {n}");
    let mut seen_offsets = Vec::new();
    let modulus: usize = std::env::var("SWEEP_MOD").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let remainder: usize = std::env::var("SWEEP_REM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut unique_index = 0usize;
    for i in 0..n {
        let rec = record_one_publish_after_overwrites(1, kind);
        let off = rec.unit_writes()[i].offset;
        if seen_offsets.contains(&off) { continue; }
        seen_offsets.push(off);
        unique_index += 1;
        if unique_index % modulus != remainder { continue; }
        run(&format!("{tag} 挂载时读零R的第{i}个单元(偏移{})", off.0), kind, 1, fault_at_mount, fault_after_mount, 60, Some(i));
    }
}
#[test]
fn s_ow_none() { sweep("OW 账无故障", PublishRecordedAsR::Overwrite, false, false); }
#[test]
fn s_ow_after() { sweep("OW 挂后账读错", PublishRecordedAsR::Overwrite, false, true); }
#[test]
fn s_sq_none() { sweep("SQ 账无故障", PublishRecordedAsR::SequentialWriteOfThreeDataUnits, false, false); }
#[test]
fn s_sq_after() { sweep("SQ 挂后账读错", PublishRecordedAsR::SequentialWriteOfThreeDataUnits, false, true); }
#[test]
fn m_ow_atmount() { run("M-OW 仅挂载时R账读错(无读零)", PublishRecordedAsR::Overwrite, 1, true, false, 60, None); }
#[test]
fn m_sq_atmount() { run("M-SQ 仅挂载时R账读错(无读零)", PublishRecordedAsR::SequentialWriteOfThreeDataUnits, 1, true, false, 60, None); }
#[test]
fn m_ow_aftermount() { run("M-OW 仅挂后R账读错(无读零)", PublishRecordedAsR::Overwrite, 1, false, true, 60, None); }
#[test]
fn m_sq_aftermount() { run("M-SQ 仅挂后R账读错(无读零)", PublishRecordedAsR::SequentialWriteOfThreeDataUnits, 1, false, true, 60, None); }
#[test]
fn print_r_account_slots() {
    for kind in [PublishRecordedAsR::Overwrite, PublishRecordedAsR::SequentialWriteOfThreeDataUnits] {
        let rec = record_one_publish_after_overwrites(1, kind);
        let r = rec.pool.current_file_version().root;
        let f = |p: NodePointer| -> Vec<(u32, u64)> { p.locations.iter().map(|l| (l.device.0, l.slot.0)).collect() };
        println!("ACCOUNTSLOTS {kind:?} R=({:?},{:?}) tree_table={:?} mapping_root={:?} alloc_tree={:?}", r.instance, r.checkpoint_txg, f(r.tree_table), f(r.mapping_root), f(r.allocation_record_tree_root));
    }
}
