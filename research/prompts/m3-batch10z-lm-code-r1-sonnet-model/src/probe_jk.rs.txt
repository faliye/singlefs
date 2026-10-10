//! 第 10z 批第十块实现员的量法探针（只在草稿副本里，不进仓）：攻方 `opus_k21_chain.rs` 的扫法搬到入库代码上，加回退那一形的代价量法。
#![allow(clippy::all, unused, reason = "草稿探针")]

mod common_unwitnessed_publish;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, root_is_in_the_ring, DeviceOverASparseImage,
    PoolUnderTest, OVERWRITE_BYTES,
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
    r_index: usize,
    r_old_key: Option<Key>,
    r_old_root: Option<singlefs_core::root_record::RootRecord>,
}

fn key_of<D: DeviceOverASparseImage>(pool: &PoolUnderTest<D>) -> Key {
    let root = *pool.session().current.root();
    (root.instance, root.checkpoint_txg)
}

fn device_bytes_with_unit_area_slots(unit_area_slots: u64) -> u64 {
    // 384 槽那一档：起点 1408 槽 + 815 + 384。
    (1408 + 815 + unit_area_slots) * singlefs_format::SLOT_BYTES
}

fn one_publish(pool: &mut PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>, kind: char, versions: &[singlefs_core::root_record::RootRecord], target_back: usize) {
    match kind {
        'o' => pool.overwrite(OVERWRITE_BYTES).map(|_| ()).expect("o"),
        'n' => pool.create_one_inode().map(|_| ()).expect("n"),
        's' => pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000).map(|_| ()).expect("s"),
        'b' => {
            let target = versions[versions.len() - 1 - target_back];
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            let PoolVersion::WithFile(current) = &mut session.current else { panic!() };
            singlefs_core::mount::roll_back_by_a_forward_publish(&parameters, &mut pool.devices, &mut session.allocator, current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg })
                .expect("回退做得成");
            pool.session = Some(session);
            if let Some(c) = pool.content_of_each_root.get(&(target.instance, target.checkpoint_txg)).cloned() {
                pool.content_of_the_current_version = c;
            }
            let root = *pool.session().current.root();
            pool.note_root_of_the_current_content(&root);
        }
        other => panic!("{other}"),
    }
}

fn data_of(pool: &PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>) -> Vec<(DeviceIdentity, u64)> {
    pool.current_file_version().data_pointers.iter().flat_map(|p| p.locations).map(|l| (l.device, l.slot.0)).collect()
}

fn record(unit_area_slots: u64, k: u64, pre: &str, q: Option<char>, q_back: usize, r: char, target_back: usize, post: &str) -> Option<Chain> {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_with_unit_area_slots(unit_area_slots),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    let mut versions = vec![*pool.session().current.root()];
    let mut data_slots_of_versions = vec![pool.current_file_version().data_pointers.iter().flat_map(|p| p.locations).map(|l| (l.device, l.slot.0)).collect::<Vec<_>>()];
    for _ in 0..k {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写落得下");
        versions.push(*pool.session().current.root());
        data_slots_of_versions.push(pool.current_file_version().data_pointers.iter().flat_map(|p| p.locations).map(|l| (l.device, l.slot.0)).collect());
    }
    for a in pre.chars() {
        let back = if a == 'b' { 1 } else { 0 };
        one_publish(&mut pool, a, &versions, back);
        versions.push(*pool.session().current.root());
        data_slots_of_versions.push(data_of(&pool));
    }
    let base = pool.image();
    let before = stream.operation_count();
    let mut q_key = None;
    let mut revived: Vec<(DeviceIdentity, u64)> = Vec::new();
    if let Some(q) = q {
        let p_data = data_of(&pool);
        if q == 'b' {
            revived = data_slots_of_versions[data_slots_of_versions.len() - 1 - q_back].iter().filter(|x| !p_data.contains(x)).copied().collect();
        }
        one_publish(&mut pool, q, &versions, q_back);
        q_key = Some(key_of(&pool));
        versions.push(*pool.session().current.root());
        data_slots_of_versions.push(data_of(&pool));
    }
    let p_data = data_of(&pool);
    let mut r_old_root = None;
    let mut r_old_key = None;
    if r == 'b' {
        revived = data_slots_of_versions[data_slots_of_versions.len() - 1 - target_back].iter().filter(|x| !p_data.contains(x)).copied().collect();
        let t = versions[versions.len() - 1 - target_back];
        r_old_root = Some(t);
        r_old_key = Some((t.instance, t.checkpoint_txg));
    }
    one_publish(&mut pool, r, &versions, target_back);
    let r_key = key_of(&pool);
    versions.push(*pool.session().current.root());
    data_slots_of_versions.push(data_of(&pool));
    for a in post.chars() {
        one_publish(&mut pool, a, &versions, 1);
        versions.push(*pool.session().current.root());
        data_slots_of_versions.push(data_of(&pool));
    }
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(&operations[before..], &fixed_geometry_of_the_384_slot_small_devices());
    let root_indices: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).collect();
    if root_indices.len() != 1 + usize::from(q.is_some()) + post.chars().count() {
        return None;
    }
    let r_index = usize::from(q.is_some());
    Some(Chain { pool, base, writes, root_indices, q_key, r_key, revived, r_index, r_old_key, r_old_root })
}

fn root_of(chain: &Chain, index: usize) -> singlefs_core::root_record::RootRecord {
    singlefs_core::root_record::RootRecord::parse_slot(chain.writes[index].bytes().expect("根槽写带字节"), &chain.pool.parameters.filesystem_identifier).expect("根自证")
}

fn range_of(w: &RetainedWrite) -> (DeviceIdentity, u64, u64) {
    (w.device, w.offset.0, w.length_in_bytes())
}

fn ranges_at_mount(chain: &Chain, faults: &str, crash_image: &MemoryPool) -> Vec<(DeviceIdentity, u64, u64)> {
    let r_root = chain.root_indices[chain.r_index];
    let last_root = *chain.root_indices.last().unwrap();
    let q_root = if chain.q_key.is_some() { Some(chain.root_indices[0]) } else { None };
    let mut out = Vec::new();
    let r_from = q_root.map_or(0, |q| q);
    for fault in faults.split(',') {
        match fault {
            "cfgq" => out.extend(chain.writes[q_root.unwrap()..last_root].iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).map(range_of)),
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
            "sacct" => {
                for i in (chain.r_index + 1)..chain.root_indices.len() {
                    let root = root_of(chain, chain.root_indices[i]);
                    let pointers = [root.tree_table, root.mapping_root, root.allocation_record_tree_root];
                    out.extend(
                        chain.writes[chain.root_indices[i - 1]..chain.root_indices[i]]
                            .iter()
                            .filter(|w| w.kind == StepKind::UnitWrite)
                            .filter(|w| pointers.iter().any(|p| p.locations.iter().any(|l| l.device == w.device && l.slot.to_device_offset() == w.offset)))
                            .map(range_of),
                    );
                }
            }
            "qacct" => {
                let root = root_of(chain, q_root.unwrap());
                let pointers = [root.tree_table, root.mapping_root, root.allocation_record_tree_root];
                out.extend(
                    chain.writes[..q_root.unwrap()]
                        .iter()
                        .filter(|w| w.kind == StepKind::UnitWrite)
                        .filter(|w| pointers.iter().any(|p| p.locations.iter().any(|l| l.device == w.device && l.slot.to_device_offset() == w.offset)))
                        .map(range_of),
                );
            }
            "rlin" => {
                out.extend(
                    chain.writes[r_from..r_root]
                        .iter()
                        .filter(|w| w.kind == StepKind::UnitWrite)
                        .filter(|w| {
                            w.bytes().map_or(false, |bytes| {
                                singlefs_core::unit::parse_packed_unit(bytes).map_or(false, |packed| packed.identity.record_type == singlefs_core::unit::PACKED_TYPE_LINEAGE)
                            })
                        })
                        .map(range_of),
                );
            }
            "rold" => {
                let key = chain.r_old_key.expect("R_old");
                out.extend(common_unwitnessed_publish::RangesReadingBackAllZero::default().with_the_root_slot_of(crash_image, key).0);
            }
            "roldacct" => {
                let root = chain.r_old_root.expect("R_old");
                for (pointer, unit) in [
                    (root.tree_table, singlefs_core::transaction::TransactionUnit::TreeTable),
                    (root.allocation_record_tree_root, singlefs_core::transaction::TransactionUnit::AllocationTree),
                ] {
                    for l in pointer.locations.iter().filter(|l| l.slot.0 >= 1408) {
                        out.push((l.device, l.slot.to_device_offset().0, unit.span_slots() * singlefs_format::SLOT_BYTES));
                    }
                }
            }
            "none" | "" => {}
            other => panic!("故障 {other}"),
        }
    }
    out
}

fn reuse_violations(image: &MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image).into_iter().filter(|l| l.starts_with("I-7.4") || l.starts_with("I-4.8")).map(|l| l.chars().take(80).collect::<String>().replace(' ', "")).collect();
    if v.is_empty() { "-".into() } else { v.join("|") }
}

fn isolated_of(pool: &PoolUnderTest<Dev>) -> u64 {
    pool.session.as_ref().map_or(0, |s| s.allocator.devices.iter().map(|m| m.isolated_slots()).sum())
}

fn step(pool: &mut PoolUnderTest<Dev>, action: char, versions_seen: &mut Vec<singlefs_core::root_record::RootRecord>) -> Result<String, String> {
    let user = |r: Result<singlefs_core::mounted_session::UserChangePublished, singlefs_core::mounted_session::UserChangeRefused>| match r {
        Ok(p) => Ok(format!("ok{}", p.floor_raises.len())),
        Err(e) => Err(format!("{e:?}").chars().take(60).collect::<String>().replace(' ', "")),
    };
    let out = match action {
        'o' => user(pool.overwrite(OVERWRITE_BYTES)),
        'n' => user(pool.create_one_inode()),
        's' => user(pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())),
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
                Err(e) => Err(format!("{e:?}").chars().take(60).collect::<String>().replace(' ', "")),
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
                    // 回退目标不在候选集这一类不算写入被拒（攻方同样不算准入）。
                    Err(e) => Ok(format!("bErr:{}", format!("{e:?}").chars().take(30).collect::<String>().replace(' ', ""))),
                }
            }
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
    let devices: Vec<(DeviceIdentity, Dev)> = crash_image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(crash_image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, Dev { identity: *identity, inner: RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()), fault: shared.clone() })
        })
        .collect();
    let mut devices = devices;
    let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount=Err:{}", format!("{e:?}").chars().take(80).collect::<String>().replace(' ', "")),
    };
    shared.active.set(persist);
    let selected = mounted.output.effective_root.checkpoint_txg.0;
    let occupancy = mounted.allocator.root_ring_occupancy().expect("根环表");
    let c_r = occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(chain.r_key.0, chain.r_key.1).unwrap_or_default();
    let revived_in_c_r = chain.revived.iter().filter(|(d, s)| c_r.iter().any(|p| p.device == *d && p.slot.0 == *s)).count();
    let iso_mount: u64 = mounted.allocator.devices.iter().map(|m| m.isolated_slots()).sum();
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
    let mut iso_after_first_raise: Option<u64> = None;
    let mut refused_steps = 0;
    let mut first_refused = String::from("-");
    let mut first_red = String::from("-");
    let mut r_left_at = String::from("-");
    for (index, action) in actions.chars().enumerate() {
        let before_image = pool.image();
        let before_ops = stream.operation_count();
        let outcome = step(&mut pool, action, &mut versions_seen);
        match &outcome {
            Ok(text) => {
                if iso_after_first_raise.is_none() && (text.starts_with("ok") && text != "ok0" || text == "F") {
                    iso_after_first_raise = Some(isolated_of(&pool));
                }
            }
            Err(text) => {
                refused_steps += 1;
                if first_refused == "-" {
                    first_refused = format!("{index}{action}:{text}");
                }
            }
        }
        iso_max = iso_max.max(isolated_of(&pool));
        if preroot && first_red == "-" {
            let operations = stream.retained_operations();
            let (writes, _segments, _ix) = writes_and_segments_with_stream_indexes(&operations[before_ops..], &fixed_geometry_of_the_384_slot_small_devices());
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
        if pool.session.is_none() {
            break;
        }
    }
    format!(
        "selected={selected} c_r={} revived={} revived_in_c_r={revived_in_c_r} iso_mount={iso_mount} iso_after_first_raise={} iso_max={iso_max} iso_end={} c_left={} abn_left={} refused_steps={refused_steps} first_refused={first_refused} r_left_at={r_left_at} hit={} first_red={first_red}",
        c_r.len(),
        chain.revived.len(),
        iso_after_first_raise.map_or("-".into(), |v| v.to_string()),
        isolated_of(&pool),
        pool.session.as_ref().map_or(0, |se| se.allocator.root_ring_occupancy().map_or(0, |o| o.placements_named_by_unapplied_journal_records().len())),
        pool.session.as_ref().map_or(0, |se| se.allocator.root_ring_occupancy().map_or(0, |o| o.ring_slots_recorded_as_holding_an_abandoned_root().values().map(|v| v.len()).sum::<usize>())),
        first_red != "-",
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

/// 环境变量：PROBE_UNIT_AREA_SLOTS、PROBE_KS（逗号）、PROBE_Q（空 = 没有 Q）、PROBE_R、PROBE_TARGET_BACKS（R = b 时回退到倒数第几版，逗号）、
/// PROBE_FAULTS、PROBE_ALPHABET、PROBE_COUNT、PROBE_SEED、PROBE_SHORTEST、PROBE_LONGEST、PROBE_PREROOT、PROBE_PERSIST、PROBE_LABEL。
#[test]
fn probe_sweep() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let label = env("PROBE_LABEL", "-");
    let arm = env("SINGLEFS_PROBE_ARM", "implemented");
    let unit_area_slots: u64 = env("PROBE_UNIT_AREA_SLOTS", "384").parse().unwrap();
    let ks: Vec<u64> = env("PROBE_KS", "3").split(',').map(|s| s.parse().unwrap()).collect();
    let q: Option<char> = env("PROBE_Q", "").chars().next();
    let pre = env("PROBE_PRE", "");
    let post = env("PROBE_POST", "");
    let q_back: usize = env("PROBE_Q_BACK", "1").parse().unwrap();
    let r: char = env("PROBE_R", "n").chars().next().unwrap();
    let target_backs: Vec<usize> = env("PROBE_TARGET_BACKS", "0").split(',').map(|s| s.parse().unwrap()).collect();
    let faults = env("PROBE_FAULTS", "racct");
    let letters: Vec<char> = env("PROBE_ALPHABET", "ons").chars().collect();
    let count: usize = env("PROBE_COUNT", "10").parse().unwrap();
    let seed: u64 = env("PROBE_SEED", "1").parse().unwrap();
    let shortest: usize = env("PROBE_SHORTEST", "10").parse().unwrap();
    let longest: usize = env("PROBE_LONGEST", "20").parse().unwrap();
    let preroot = std::env::var("PROBE_PREROOT").is_ok();
    let persist = std::env::var("PROBE_PERSIST").is_ok();
    let actions = random_actions(count, seed, &letters, shortest, longest);
    let mut histories = 0;
    for k in &ks {
        for target_back in &target_backs {
            let back = if r == 'b' { *target_back } else { 0 };
            let recorded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| record(unit_area_slots, *k, &pre, q, q_back, r, back, &post))).unwrap_or(None);
            let Some(chain) = recorded else {
                println!("PROBEJK label={label} arm={arm} k={k} back={back} record=skipped");
                continue;
            };
            for a in &actions {
                let started = std::time::Instant::now();
                let line = run_one(&chain, &faults, a, preroot, persist);
                histories += 1;
                println!(
                    "PROBEJK label={label} arm={arm} slots={unit_area_slots} k={k} q={q:?} r={r} back={back} faults={faults} persist={persist} actions={a} secs={:.2} {line}",
                    started.elapsed().as_secs_f64()
                );
            }
            if r != 'b' {
                break;
            }
        }
    }
    println!("PROBEJK_DONE label={label} arm={arm} histories={histories}");
}
