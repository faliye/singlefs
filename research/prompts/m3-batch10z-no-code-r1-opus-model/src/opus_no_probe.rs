//! m3-batch10z-no-code-r1 攻方探针（只在草稿副本里）：起点取自 m3-batch10z-lm-code-r1 攻方探针 opus_lm_probe.rs（记一段带回退或建的历史、挂载那一刻读回全零的故障、挂上之后随机用户动作），加：每一步之后分配器里隔离还护着而空闲 / 只扣住的槽、分配位与隔离位同时置着的槽按归属拆、挂载与抬 F 时按起点槽豁免而跨度更宽的计数、卸载再挂与崩溃再挂两个动作。
#![allow(clippy::all, unused, reason = "草稿探针")]

mod common_unwitnessed_publish;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, root_is_in_the_ring, DeviceOverASparseImage,
    PoolUnderTest, RangesReadingBackAllZero, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor_to_the_admission_ceiling,
    RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice, SparseDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[derive(Clone)]
struct Fault {
    ranges: Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>>,
    active: Rc<Cell<bool>>,
}
pub struct Dev {
    identity: DeviceIdentity,
    inner: RecordingBlockDevice<SparseBlockDevice>,
    fault: Fault,
}
fn overlaps(a: u64, al: u64, b: u64, bl: u64) -> bool {
    a < b + bl && b < a + al
}
impl BlockDevice for Dev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        if !self.fault.active.get() {
            return Ok(());
        }
        let n = buffer.len() as u64;
        for (d, s, l) in self.fault.ranges.borrow().iter() {
            if *d == self.identity && overlaps(*s, *l, offset.0, n) {
                let from = s.saturating_sub(offset.0) as usize;
                let to = ((s + l).min(offset.0 + n) - offset.0) as usize;
                buffer[from..to].fill(0);
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let n = bytes.len() as u64;
        let me = self.identity;
        self.fault.ranges.borrow_mut().retain(|(d, s, l)| !(*d == me && overlaps(*s, *l, offset.0, n)));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        let me = self.identity;
        self.fault.ranges.borrow_mut().retain(|(d, s, l)| !(*d == me && overlaps(*s, *l, offset.0, length)));
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
        &self.inner.wrapped_device().image
    }
}

type Key = (InstanceGeneration, CheckpointTxg);

struct Chain {
    pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    root_indices: Vec<usize>,
    q_key: Option<Key>,
    r_key: Key,
    revived: Vec<(DeviceIdentity, u64)>,
    r_old: Option<singlefs_core::root_record::RootRecord>,
    full_writes: Vec<RetainedWrite>,
}

fn key_of<D: DeviceOverASparseImage>(pool: &PoolUnderTest<D>) -> Key {
    let root = *pool.session().current.root();
    (root.instance, root.checkpoint_txg)
}

fn device_bytes_with_unit_area_slots(unit_area_slots: u64) -> u64 {
    (1408 + 815 + unit_area_slots) * singlefs_format::SLOT_BYTES
}

/// 动作记号：o 覆盖写、n 建 inode、s 顺序写、b<数> 回退到 versions 里倒数第 1+数 版、B<数> 回退到 txg = 当前 txg + 1 − 数 的那一版。
fn tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in text.chars() {
        if c.is_ascii_digit() {
            out.last_mut().expect("数字跟在 b/B 后").push(c);
        } else {
            out.push(c.to_string());
        }
    }
    out
}

fn data_slots(pool: &PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>) -> Vec<(DeviceIdentity, u64)> {
    pool.current_file_version().data_pointers.iter().flat_map(|p| p.locations).map(|l| (l.device, l.slot.0)).collect()
}

/// 施加一个动作；回退时交回回退目标。
fn apply(pool: &mut PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>, token: &str, versions: &[singlefs_core::root_record::RootRecord]) -> Option<singlefs_core::root_record::RootRecord> {
    let kind = token.chars().next().unwrap();
    match kind {
        'o' => { pool.overwrite(OVERWRITE_BYTES).expect("o"); None }
        'n' => { pool.create_one_inode().expect("n"); None }
        'd' => {
            let inode = singlefs_core::address::InodeNumber(pool.current_file_version().inode_number_watermark() - 1);
            pool.write_time_seconds += 1;
            let write_time_seconds = pool.write_time_seconds;
            let published = pool.publish(singlefs_core::mounted_session::UserChange::Unlink { inode, write_time_seconds }, None).expect("d");
            for version in &published.versions_published {
                let root = version.root;
                pool.note_root_of_the_current_content(&root);
            }
            None
        }
        's' => { pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000).expect("s"); None }
        'S' => { pool.sequential_write(20 * singlefs_core::unit::data_unit_payload_capacity()).expect("S"); None }
        'b' | 'B' => {
            let n: u64 = token[1..].parse().unwrap();
            let target = if kind == 'b' {
                versions[versions.len() - 1 - n as usize]
            } else {
                let want = pool.session().current.root().checkpoint_txg.0 + 1 - n;
                *versions.iter().find(|v| v.checkpoint_txg.0 == want).expect("那个 txg 的版本在表里")
            };
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            let PoolVersion::WithFile(current) = &mut session.current else { panic!() };
            let outcome = singlefs_core::mount::roll_back_by_a_forward_publish(&parameters, &mut pool.devices, &mut session.allocator, current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg });
            pool.session = Some(session);
            outcome.unwrap_or_else(|e| panic!("回退做不成：{:?}", e));
            if let Some(c) = pool.content_of_each_root.get(&(target.instance, target.checkpoint_txg)).cloned() {
                pool.content_of_the_current_version = c;
            }
            let root = *pool.session().current.root();
            pool.note_root_of_the_current_content(&root);
            Some(target)
        }
        other => panic!("{other}"),
    }
}

fn record(unit_area_slots: u64, prefix: &str, mid: &str, r: &str) -> Result<Chain, String> {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_with_unit_area_slots(unit_area_slots),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    let mut versions = vec![*pool.session().current.root()];
    let mut data_of: std::collections::BTreeMap<Key, Vec<(DeviceIdentity, u64)>> = Default::default();
    data_of.insert(key_of(&pool), data_slots(&pool));
    let step = |pool: &mut PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>, t: &str, versions: &mut Vec<singlefs_core::root_record::RootRecord>, data_of: &mut std::collections::BTreeMap<Key, Vec<(DeviceIdentity, u64)>>| {
        let target = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| apply(pool, t, versions)));
        let target = target.map_err(|_| format!("动作 {t} 做不成"))?;
        versions.push(*pool.session().current.root());
        data_of.insert(key_of(pool), data_slots(pool));
        Ok::<_, String>(target)
    };
    for t in tokens(prefix) {
        step(&mut pool, &t, &mut versions, &mut data_of)?;
    }
    let base = pool.image();
    let p_data = data_slots(&pool);
    let before = stream.operation_count();
    let mut q_key = None;
    let mid_tokens = tokens(mid);
    for t in &mid_tokens {
        step(&mut pool, t, &mut versions, &mut data_of)?;
        q_key = Some(key_of(&pool));
    }
    let r_old = step(&mut pool, r, &mut versions, &mut data_of)?;
    let revived = r_old.map_or(Vec::new(), |t| data_of[&(t.instance, t.checkpoint_txg)].iter().filter(|x| !p_data.contains(x)).copied().collect());
    let mut r_key = key_of(&pool);
    let operations = stream.retained_operations();
    let (mut writes, _s, _i) = writes_and_segments_with_stream_indexes(&operations[before..], &fixed_geometry_of_the_384_slot_small_devices());
    let (full_writes, _s, _i) = writes_and_segments_with_stream_indexes(&operations[..], &fixed_geometry_of_the_384_slot_small_devices());
    let mut root_indices: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).collect();
    if r.starts_with('d') && std::env::var("S2_DELETE_CRASH_AT_INTENT").is_ok() && root_indices.len() > 1 {
        let first = root_indices[0];
        writes.truncate(first + 1);
        root_indices.truncate(1);
        let root = singlefs_core::root_record::RootRecord::parse_slot(writes[first].bytes().expect("根槽写带字节"), &pool.parameters.filesystem_identifier).expect("根自证");
        r_key = (root.instance, root.checkpoint_txg);
    }
    if root_indices.len() != 1 + mid_tokens.len() && !(r.starts_with('d') && root_indices.len() > mid_tokens.len()) {
        return Err(format!("根槽写 {} 次", root_indices.len()));
    }
    Ok(Chain { pool, base, writes, root_indices, q_key, r_key, revived, r_old, full_writes })
}

fn root_of(chain: &Chain, index: usize) -> singlefs_core::root_record::RootRecord {
    singlefs_core::root_record::RootRecord::parse_slot(chain.writes[index].bytes().expect("根槽写带字节"), &chain.pool.parameters.filesystem_identifier).expect("根自证")
}

fn range_of(w: &RetainedWrite) -> (DeviceIdentity, u64, u64) {
    (w.device, w.offset.0, w.length_in_bytes())
}

fn is_lineage_unit(w: &RetainedWrite) -> bool {
    w.bytes().and_then(|b| singlefs_core::unit::parse_packed_unit(b).ok()).map_or(false, |p| p.identity.record_type == singlefs_core::unit::PACKED_TYPE_LINEAGE)
}

/// 指针点到的单元在整条流里最后一次写（盘、偏移相同）的范围。
fn ranges_of_pointer_in(full: &[RetainedWrite], pointer: &singlefs_core::pointer::NodePointer) -> Vec<(DeviceIdentity, u64, u64)> {
    pointer.locations.iter().filter_map(|l| full.iter().rev().find(|w| w.kind == StepKind::UnitWrite && w.device == l.device && w.offset == l.slot.to_device_offset()).map(range_of)).collect()
}

/// 故障（挂载那一刻读回全零）：racct R 的账三个指针点到的单元；rall R 这次发布写的全部单元；rlin R 写的谱系片；
/// oldroot R_old 的根槽；oldacct R_old 的分配记录树根与中央映射根；cfgq、qroot、qlast 同第十块实现员探针。
fn ranges_at_mount(chain: &Chain, faults: &str, crash_image: &MemoryPool) -> Vec<(DeviceIdentity, u64, u64)> {
    let r_root = *chain.root_indices.last().unwrap();
    let q_root = if chain.q_key.is_some() { Some(chain.root_indices[chain.root_indices.len() - 2]) } else { None };
    let mut out = Vec::new();
    let r_from = q_root.map_or(0, |q| q + 1);
    for fault in faults.split(',') {
        match fault {
            "cfgq" => out.extend(chain.writes[q_root.unwrap()..r_root].iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).map(range_of)),
            "qroot" => out.push(range_of(&chain.writes[q_root.unwrap()])),
            "qlast" => {
                let q_records: Vec<&RetainedWrite> = chain.writes[..q_root.unwrap()].iter().filter(|w| w.kind == StepKind::JournalRecord).collect();
                let last = q_records.iter().map(|w| w.offset.0).max();
                out.extend(q_records.iter().filter(|w| Some(w.offset.0) == last).map(|w| range_of(w)));
            }
            "racct" => {
                let root = root_of(chain, r_root);
                let pointers = [root.tree_table, root.mapping_root, root.allocation_record_tree_root];
                out.extend(
                    chain.writes[r_from..r_root]
                        .iter()
                        .filter(|w| w.kind == StepKind::UnitWrite)
                        .filter(|w| pointers.iter().any(|p| p.locations.iter().any(|l| l.device == w.device && l.slot.to_device_offset() == w.offset)))
                        .map(range_of),
                );
            }
            "qacct" | "qlin" | "qall" => {
                let q = q_root.expect("有 Q");
                let q_from = if chain.root_indices.len() >= 3 { chain.root_indices[chain.root_indices.len() - 3] + 1 } else { 0 };
                let root = root_of(chain, q);
                let pointers = [root.tree_table, root.mapping_root, root.allocation_record_tree_root];
                out.extend(chain.writes[q_from..q].iter().filter(|w| w.kind == StepKind::UnitWrite).filter(|w| match fault {
                    "qacct" => pointers.iter().any(|p| p.locations.iter().any(|l| l.device == w.device && l.slot.to_device_offset() == w.offset)),
                    "qlin" => is_lineage_unit(w),
                    _ => true,
                }).map(range_of));
            }
            "rroot" => out.push(range_of(&chain.writes[r_root])),
            "rall" => out.extend(chain.writes[r_from..r_root].iter().filter(|w| w.kind == StepKind::UnitWrite).map(range_of)),
            "rtomb" => out.extend(chain.writes[r_from..r_root].iter().filter(|w| w.kind == StepKind::UnitWrite && w.bytes().and_then(|b| singlefs_core::unit::parse_packed_unit(b).ok()).map_or(false, |p| p.identity.record_type == singlefs_core::unit::PACKED_TYPE_TOMBSTONE)).map(range_of)),
            // R 写的打包单元（类标签 3 的码 3 单元）里不是谱系片的全部；rpk<k>：按写的次序第 k 片打包单元。
            "rnonlin" => out.extend(chain.writes[r_from..r_root].iter().filter(|w| w.kind == StepKind::UnitWrite && w.bytes().and_then(|b| singlefs_core::unit::parse_packed_unit(b).ok()).is_some() && !is_lineage_unit(w)).map(range_of)),
            f if f.starts_with("rpk") => {
                let k: usize = f[3..].parse().unwrap();
                let packed: Vec<&RetainedWrite> = chain.writes[r_from..r_root].iter().filter(|w| w.kind == StepKind::UnitWrite && w.bytes().and_then(|b| singlefs_core::unit::parse_packed_unit(b).ok()).is_some()).collect();
                if let Some(w) = packed.get(k) { out.push(range_of(w)); }
            }
            "rlin" => out.extend(chain.writes[r_from..r_root].iter().filter(|w| w.kind == StepKind::UnitWrite && is_lineage_unit(w)).map(range_of)),
            "oldroot" => {
                if let Some(old) = chain.r_old {
                    if root_is_in_the_ring(crash_image, (old.instance, old.checkpoint_txg)) {
                        out.extend(RangesReadingBackAllZero(Vec::new()).with_the_root_slot_of(crash_image, (old.instance, old.checkpoint_txg)).0);
                    }
                }
            }
            "oldacct" => {
                if let Some(old_in_memory) = chain.r_old {
                    let old = chain.full_writes.iter().filter(|w| w.kind == StepKind::RootRecordFua).filter_map(|w| singlefs_core::root_record::RootRecord::parse_slot(w.bytes()?, &chain.pool.parameters.filesystem_identifier)).filter(|root| (root.instance, root.checkpoint_txg) == (old_in_memory.instance, old_in_memory.checkpoint_txg)).last().expect("R_old 的根槽写录到了");
                    out.extend(ranges_of_pointer_in(&chain.full_writes, &old.allocation_record_tree_root));
                    out.extend(ranges_of_pointer_in(&chain.full_writes, &old.mapping_root));
                    out.extend(ranges_of_pointer_in(&chain.full_writes, &old.tree_table));
                    if std::env::var("S2_DEBUG").is_ok() { eprintln!("S2DEBUG oldacct locations={:?} ranges={:?}", old.allocation_record_tree_root.locations, out); }
                }
            }
            "none" | "" => {}
            other => panic!("故障 {other}"),
        }
    }
    out
}

fn reuse_violations(image: &MemoryPool) -> String {
    let full: Vec<String> = checker_violations_on(image).into_iter().filter(|l| l.starts_with("I-7.4") || l.starts_with("I-4.8")).collect();
    if std::env::var("S3_FULLRED").is_ok() && !full.is_empty() { println!("FULLRED {}", full.join(" || ")); }
    let v: Vec<String> = full.into_iter().map(|l| l.chars().take(80).collect::<String>().replace(' ', "")).collect();
    if v.is_empty() { "-".into() } else { v.join("|") }
}

fn isolated_of(pool: &PoolUnderTest<Dev>) -> u64 {
    pool.session.as_ref().map_or(0, |s| s.allocator.devices.iter().map(|m| m.isolated_slots()).sum())
}

fn probe_of(pool: &PoolUnderTest<Dev>) -> [u64; 6] {
    pool.session.as_ref().map_or([0; 6], |s| s.allocator.opus_no_probe())
}

fn counters() -> [u64; 4] {
    use std::sync::atomic::Ordering::Relaxed;
    use singlefs_core::mount::{OPUS_NO_AB_EXEMPT_PARTIAL, OPUS_NO_C_EXEMPT_PARTIAL, OPUS_NO_C_EXEMPT_PARTIAL_UNPROTECTED_SLOTS, OPUS_NO_DUP_KEY_WIDER};
    [OPUS_NO_C_EXEMPT_PARTIAL.load(Relaxed), OPUS_NO_C_EXEMPT_PARTIAL_UNPROTECTED_SLOTS.load(Relaxed), OPUS_NO_DUP_KEY_WIDER.load(Relaxed), OPUS_NO_AB_EXEMPT_PARTIAL.load(Relaxed)]
}

fn short<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}").chars().take(60).collect::<String>().replace(' ', "")
}

/// 动作：o 覆盖写、n 建 inode、s 顺序写两单元多、S 顺序写 20 单元、F 抬 F 到准入上限、b 回退到上一版、u 正常卸载（下一个动作之前再挂）、
/// c 进程退出不卸载（下一个动作之前再挂）。
fn step(pool: &mut PoolUnderTest<Dev>, action: char, versions_seen: &mut Vec<singlefs_core::root_record::RootRecord>) -> Result<String, String> {
    if pool.session.is_none() {
        pool.crash_and_mount_writable().map_err(|e| format!("remount:{}", short(e)))?;
        versions_seen.push(*pool.session().current.root());
    }
    let user = |r: Result<singlefs_core::mounted_session::UserChangePublished, singlefs_core::mounted_session::UserChangeRefused>| match r {
        Ok(p) => Ok(format!("ok{}", p.floor_raises.len())),
        Err(e) => Err(short(e)),
    };
    let out = match action {
        'o' => user(pool.overwrite(OVERWRITE_BYTES)),
        'n' => user(pool.create_one_inode()),
        's' => user(pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())),
        'S' => user(pool.sequential_write(20 * singlefs_core::unit::data_unit_payload_capacity())),
        'F' => {
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            let PoolVersion::WithFile(current) = &mut session.current else { panic!() };
            let outcome = raise_rollback_floor_to_the_admission_ceiling(&parameters, &mut pool.devices, &mut session.allocator, current, ShadowLedger::On);
            pool.session = Some(session);
            match outcome {
                Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
                    pool.note_floor_raises(std::slice::from_ref(&raised));
                    Ok("F".into())
                }
                Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }) => Ok("F=".into()),
                Err(e) => Err(short(e)),
            }
        }
        'b' => {
            if versions_seen.len() < 2 {
                Ok("skip".into())
            } else {
                let target = versions_seen[versions_seen.len() - 2];
                let parameters = pool.parameters.clone();
                let mut session = pool.session.take().expect("会话");
                let PoolVersion::WithFile(current) = &mut session.current else { panic!() };
                let outcome = singlefs_core::mount::roll_back_by_a_forward_publish(&parameters, &mut pool.devices, &mut session.allocator, current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg });
                pool.session = Some(session);
                match outcome {
                    Ok(_) => {
                        if let Some(c) = pool.content_of_each_root.get(&(target.instance, target.checkpoint_txg)).cloned() {
                            pool.content_of_the_current_version = c;
                        }
                        let root = *pool.session().current.root();
                        pool.note_root_of_the_current_content(&root);
                        Ok("b".into())
                    }
                    Err(e) => Ok(format!("bErr:{}", short(e))),
                }
            }
        }
        'u' => {
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            let outcome = singlefs_core::mount::unmount(&parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
            match outcome {
                Ok(_) => {
                    let probe = session.allocator.opus_no_probe();
                    Ok(if probe[0] + probe[1] > 0 { format!("u:exp={}/{}", probe[0], probe[1]) } else { "u".into() })
                }
                Err(e) => {
                    pool.session = Some(session);
                    Err(format!("u:{}", short(e)))
                }
            }
        }
        'c' => {
            pool.session = None;
            Ok("c".into())
        }
        other => panic!("动作 {other}"),
    };
    if let Some(session) = pool.session.as_ref() {
        versions_seen.push(*session.current.root());
    }
    out
}

fn run_one(chain: &Chain, faults: &str, actions: &str, preroot: bool, persist: bool) -> String {
    let r_root = *chain.root_indices.last().unwrap();
    let mut crash_image = chain.base.clone();
    crash_image.apply_writes(&chain.writes[..=r_root]);
    let parameters = chain.pool.parameters.clone();
    let stream = SharedStream::retaining_contents();
    let shared = Fault { ranges: Rc::new(RefCell::new(ranges_at_mount(chain, faults, &crash_image))), active: Rc::new(Cell::new(true)) };
    let mut devices: Vec<(DeviceIdentity, Dev)> = crash_image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(crash_image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, Dev { identity: *identity, inner: RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()), fault: shared.clone() })
        })
        .collect();
    let counters_before = counters();
    let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount=Err:{}", format!("{e:?}").chars().take(80).collect::<String>().replace(' ', "")),
    };
    let counters_mount = counters();
    shared.active.set(persist);
    let selected = mounted.output.effective_root.checkpoint_txg.0;
    let iso_mount: u64 = mounted.allocator.devices.iter().map(|m| m.isolated_slots()).sum();
    let probe_mount = mounted.allocator.opus_no_probe();
    if let Ok(w) = std::env::var("S3_WATCH_SLOT") {
        println!("WATCH mount {}", mounted.allocator.opus_no_watch(w.parse().unwrap()));
    }
    if std::env::var("S3_LIST_DBL").is_ok() {
        println!("DBL mount {}", mounted.allocator.opus_no_dbl_slots());
    }
    let (output, session) = MountedSession::of_the_mount(mounted);
    let Some(content) = chain.pool.content_of_each_root.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned() else {
        return format!("selected={selected} mount=ok-unknown-content");
    };
    let mut content_of_each_root = chain.pool.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert((root.instance, root.checkpoint_txg), content.clone());
    }
    let mut pool = PoolUnderTest { parameters: parameters.clone(), device_bytes: chain.pool.device_bytes, devices, session: Some(session), write_time_seconds: chain.pool.write_time_seconds + 10_000, content_of_the_current_version: content, content_of_each_root };
    let mut versions_seen = vec![*pool.session().current.root()];
    let mut iso_max = isolated_of(&pool);
    let mut probe_max = probe_mount;
    let mut first_exposed = if probe_mount[0] + probe_mount[1] > 0 { "mount".to_string() } else { "-".to_string() };
    let mut first_dbl_c = if probe_mount[4] + probe_mount[5] > 0 { "mount".to_string() } else { "-".to_string() };
    let mut refused_steps = 0;
    let mut first_refused = String::from("-");
    let mut first_red = String::from("-");
    let mut r_left_at = String::from("-");
    let mut done = String::new();
    for (index, action) in actions.chars().enumerate() {
        let before_image = pool.image();
        let before_ops = stream.operation_count();
        let outcome = step(&mut pool, action, &mut versions_seen);
        match &outcome {
            Ok(text) => done.push_str(&format!("{action}{} ", if text.starts_with("ok") { "" } else { text.as_str() })),
            Err(text) => {
                refused_steps += 1;
                done.push_str(&format!("{action}!{} ", text.chars().take(12).collect::<String>()));
                if first_refused == "-" {
                    first_refused = format!("{index}{action}:{text}");
                }
                if text.starts_with("remount:") {
                    break;
                }
            }
        }
        iso_max = iso_max.max(isolated_of(&pool));
        let probe = probe_of(&pool);
        for k in 0..6 {
            probe_max[k] = probe_max[k].max(probe[k]);
        }
        if first_exposed == "-" && probe[0] + probe[1] > 0 {
            first_exposed = format!("{index}{action}:free={}:held={}", probe[0], probe[1]);
        }
        if first_dbl_c == "-" && probe[4] + probe[5] > 0 {
            first_dbl_c = format!("{index}{action}:c={}:other={}", probe[4], probe[5]);
        }
        if let (Ok(w), Some(session)) = (std::env::var("S3_WATCH_SLOT"), pool.session.as_ref()) {
            println!("WATCH step={index} {}", session.allocator.opus_no_watch(w.parse().unwrap()));
        }
        if std::env::var("S3_TRACE").is_ok() {
            let txg = pool.session.as_ref().map_or(0, |s| s.current.root().checkpoint_txg.0);
            println!("TRACE step={index} action={action} outcome={} iso={} probe={probe:?} txg={txg} r_in={}", match &outcome { Ok(t) => t.clone(), Err(t) => format!("REFUSED:{t}") }, isolated_of(&pool), root_is_in_the_ring(&pool.image(), chain.r_key));
        }
        if preroot && first_red == "-" {
            let operations = stream.retained_operations();
            let tail = &operations[before_ops..];
            let spans = if action == 'u' { vec![singlefs_harness::RecordedEntrySpan { entry: singlefs_harness::RecordedPublishEntry::Unmount, operations: 0..tail.len() }] } else { Vec::new() };
            let (writes, _segments, _ix) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(tail, &spans, &fixed_geometry_of_the_384_slot_small_devices());
            if let Some(last_root) = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua) {
                let mut state = before_image.clone();
                state.apply_writes(&writes[..last_root]);
                let v = reuse_violations(&state);
                if v != "-" {
                    first_red = format!("{index}{action}preroot:r_in={}:{v}", root_is_in_the_ring(&state, chain.r_key));
                }
            }
        }
        if first_red == "-" {
            let image = pool.image();
            let v = reuse_violations(&image);
            if v != "-" {
                first_red = format!("{index}{action}after:r_in={}:{v}", root_is_in_the_ring(&image, chain.r_key));
            }
        }
        if r_left_at == "-" && !root_is_in_the_ring(&pool.image(), chain.r_key) {
            r_left_at = format!("{index}");
        }
    }
    let counters_end = counters();
    let delta = |a: [u64; 4], b: [u64; 4]| format!("{}/{}/{}/{}", b[0] - a[0], b[1] - a[1], b[2] - a[2], b[3] - a[3]);
    format!(
        "selected={selected} iso_mount={iso_mount} probe_mount={} probe_max={} iso_max={iso_max} iso_end={} cnt_mount={} cnt_all={} refused_steps={refused_steps} first_refused={first_refused} r_left_at={r_left_at} first_exposed={first_exposed} first_dbl_c={first_dbl_c} hit={} first_red={first_red} done={}",
        probe_mount.iter().map(u64::to_string).collect::<Vec<_>>().join("/"),
        probe_max.iter().map(u64::to_string).collect::<Vec<_>>().join("/"),
        isolated_of(&pool),
        delta(counters_before, counters_mount),
        delta(counters_before, counters_end),
        first_red != "-",
        done.trim_end().replace(' ', ","),
    )
}

fn random_actions(count: usize, seed: u64, letters: &[char], shortest: usize, longest: usize) -> Vec<String> {
    let mut state = seed;
    let mut next = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 33) as usize
    };
    (0..count)
        .map(|_| {
            let length = shortest + next() % (longest - shortest + 1);
            (0..length).map(|_| letters[next() % letters.len()]).collect()
        })
        .collect()
}

/// 环境变量：S3_SLOTS、S3_PREFIXES（分号隔开）、S3_MID（分号隔开，可空）、S3_RS（分号隔开的 R 动作）、S3_FAULTS（分号隔开的故障组）、
/// S3_ALPHABET、S3_COUNT、S3_SEED、S3_SHORTEST、S3_LONGEST、S3_PREROOT、S3_PERSIST、S3_LABEL；S3_ACTIONS（分号隔开，给了就不随机）。
/// 前缀 × 中间 × R × 故障组 × 用户动作串全扫；每段历史起点状态只建一次（record），每个故障组 × 动作串从内存里那一份拷。
#[test]
fn s3_sweep() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let label = env("S3_LABEL", "-");
    let arm = env("OPUS_NO_ARM", "today");
    let slots: u64 = env("S3_SLOTS", "384").parse().unwrap();
    let list = |k: &str, d: &str| env(k, d).split(';').map(String::from).collect::<Vec<_>>();
    let prefixes = list("S3_PREFIXES", "oooo");
    let mids = list("S3_MID", "");
    let rs = list("S3_RS", "b2");
    let fault_groups = list("S3_FAULTS", "racct");
    let letters: Vec<char> = env("S3_ALPHABET", "ons").chars().collect();
    let count: usize = env("S3_COUNT", "10").parse().unwrap();
    let seed: u64 = env("S3_SEED", "1").parse().unwrap();
    let shortest: usize = env("S3_SHORTEST", "10").parse().unwrap();
    let longest: usize = env("S3_LONGEST", "20").parse().unwrap();
    let preroot = std::env::var("S3_PREROOT").is_ok();
    let persist = std::env::var("S3_PERSIST").is_ok();
    let actions = match std::env::var("S3_ACTIONS") {
        Ok(given) => given.split(';').map(String::from).collect(),
        Err(_) => random_actions(count, seed, &letters, shortest, longest),
    };
    let mut histories = 0;
    let mut hits = 0;
    for prefix in &prefixes {
        for mid in &mids {
            for r in &rs {
                let chain = match record(slots, prefix, mid, r) {
                    Ok(chain) => chain,
                    Err(why) => {
                        println!("S3 label={label} arm={arm} slots={slots} prefix={prefix} mid={mid} r={r} record=skipped:{}", why.replace(' ', ""));
                        continue;
                    }
                };
                let r_root = *chain.root_indices.last().unwrap();
                let mut crash_image = chain.base.clone();
                crash_image.apply_writes(&chain.writes[..=r_root]);
                let r_old_txg = chain.r_old.map_or("-".to_string(), |o| o.checkpoint_txg.0.to_string());
                let r_old_in_ring = chain.r_old.map_or(false, |o| root_is_in_the_ring(&crash_image, (o.instance, o.checkpoint_txg)));
                for faults in &fault_groups {
                    for a in &actions {
                        let started = std::time::Instant::now();
                        let line = run_one(&chain, faults, a, preroot, persist);
                        histories += 1;
                        if line.contains("hit=true") {
                            hits += 1;
                        }
                        println!(
                            "S3 label={label} arm={arm} slots={slots} prefix={prefix} mid={mid} r={r} r_txg={} r_old_txg={r_old_txg} r_old_in_ring={r_old_in_ring} faults={faults} persist={persist} actions={a} secs={:.2} {line}",
                            chain.r_key.1 .0,
                            started.elapsed().as_secs_f64()
                        );
                    }
                }
            }
        }
    }
    println!("S3_DONE label={label} arm={arm} histories={histories} hits={hits}");
}
