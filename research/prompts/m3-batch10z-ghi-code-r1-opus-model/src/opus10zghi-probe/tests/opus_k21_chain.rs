//! m3-batch10z-ghi-code-r1 攻方腿原型 K2-1：丙罩不罩得住被抛弃根 R 引用、却不是 R 自己写的、也不在所选那一版的账里的单元（R 与所选那一版 P 之间还有一次发布 Q）。
//! 造盘：第一个文件之后覆盖写 k 次（最后一版是 P），再录 Q、R 两次发布，崩在 R 的根槽落盘之后、R 的系统配置轮换之前。
//! 可写挂载那一次读回全零：Q 那次轮换写的系统配置槽（每块盘）、Q 的根槽、Q 的记录（末条或全部）、R 自己写的账那几个单元；F2 退回 P。挂上之后撤故障。

#[path = "../../singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, device_bytes_of_the_384_slot_small_devices,
    DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor_to_the_admission_ceiling, unmount,
    RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, closed_form_state_count, MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 读回全零的几段（每块盘各自一份，写到哪一段那一段就不再藏）；`active` 撤了之后照读。
#[derive(Clone)]
struct Fault {
    ranges: Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>>,
    active: Rc<Cell<bool>>,
    hits: Rc<Cell<u64>>,
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
        if !self.fault.active.get() || singlefs_core::mount::OPUS_BYPASS_READ_FAULTS.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        let n = u64::try_from(buffer.len()).unwrap();
        for (d, s, l) in self.fault.ranges.borrow().iter() {
            if *d == self.identity && overlaps(*s, *l, offset.0, n) {
                let from = usize::try_from(s.saturating_sub(offset.0)).unwrap();
                let to = usize::try_from((s + l).min(offset.0 + n) - offset.0).unwrap();
                buffer[from..to].fill(0);
                self.fault.hits.set(self.fault.hits.get() + 1);
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let n = u64::try_from(bytes.len()).unwrap();
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


use singlefs_harness::memory_pool::RetainedWrite;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};

type Key = (InstanceGeneration, CheckpointTxg);

/// 录下来的 Q、R：Q 之前那一刻的盘面、Q 与 R 两次发布的全部写（按落盘次序）、两条根槽写的下标。
struct Chain {
    pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    q_root: usize,
    r_root: usize,
    p_key: Key,
    q_key: Key,
    r_key: Key,
}

fn one_publish(pool: &mut PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>, kind: char) {
    match kind {
        'o' => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
        'n' => pool.create_one_inode().map(|_| ()),
        's' => pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000).map(|_| ()),
        other => panic!("发布种类 {other}"),
    }
    .expect("录的发布落得下");
}

fn key_of(pool: &PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>) -> Key {
    let root = *pool.session().current.root();
    (root.instance, root.checkpoint_txg)
}

/// 起池、覆盖写 k 次（P），再录 Q（种类 `q`）与 R（种类 `r`）。录的两次之间推了抬 F（多出根槽写）的交 None。
fn record_chain(k: u64, q: char, r: char) -> Option<Chain> {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for _ in 0..k {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写落得下");
    }
    let p_key = key_of(&pool);
    let base = pool.image();
    let before = stream.operation_count();
    one_publish(&mut pool, q);
    let q_key = key_of(&pool);
    one_publish(&mut pool, r);
    let r_key = key_of(&pool);
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) =
        writes_and_segments_with_stream_indexes(&operations[before..], &fixed_geometry_of_the_384_slot_small_devices());
    let roots: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).collect();
    if roots.len() != 2 {
        return None;
    }
    Some(Chain { pool, base, writes, q_root: roots[0], r_root: roots[1], p_key, q_key, r_key })
}

fn root_of(chain: &Chain, index: usize) -> singlefs_core::root_record::RootRecord {
    singlefs_core::root_record::RootRecord::parse_slot(
        chain.writes[index].bytes().expect("根槽写带字节"),
        &chain.pool.parameters.filesystem_identifier,
    )
    .expect("根自证")
}

fn range_of(w: &RetainedWrite) -> (DeviceIdentity, u64, u64) {
    (w.device, w.offset.0, w.length_in_bytes())
}

/// 挂载那一刻读回全零的几段。`faults` 是逗号分开的几样：cfgq（Q 那次轮换的系统配置槽）、qroot、qlast（Q 末条记录）、qrec（Q 全部记录）、
/// racct（R 写的树表、中央映射树根、分配记录树根那几份）、qacct（Q 写的那几份）。
fn ranges_at_mount(chain: &Chain, faults: &str) -> Vec<(DeviceIdentity, u64, u64)> {
    let mut out = Vec::new();
    let q_records: Vec<&RetainedWrite> = chain.writes[..chain.q_root].iter().filter(|w| w.kind == StepKind::JournalRecord).collect();
    let last_offset = q_records.iter().map(|w| w.offset.0).max();
    let account_of = |root_index: usize, from: usize, to: usize| -> Vec<(DeviceIdentity, u64, u64)> {
        let root = root_of(chain, root_index);
        let pointers = [root.tree_table, root.mapping_root, root.allocation_record_tree_root];
        chain.writes[from..to]
            .iter()
            .filter(|w| w.kind == StepKind::UnitWrite)
            .filter(|w| pointers.iter().any(|p| p.locations.iter().any(|l| l.device == w.device && l.slot.to_device_offset() == w.offset)))
            .map(range_of)
            .collect()
    };
    for fault in faults.split(',') {
        match fault {
            "cfgq" => out.extend(chain.writes[chain.q_root..chain.r_root].iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).map(range_of)),
            "qroot" => out.push(range_of(&chain.writes[chain.q_root])),
            "qlast" => out.extend(q_records.iter().filter(|w| Some(w.offset.0) == last_offset).map(|w| range_of(w))),
            "qrec" => out.extend(q_records.iter().map(|w| range_of(w))),
            "racct" => out.extend(account_of(chain.r_root, chain.q_root, chain.r_root)),
            "qacct" => out.extend(account_of(chain.q_root, 0, chain.q_root)),
            "none" | "" => {}
            other => panic!("故障 {other}"),
        }
    }
    out
}

fn short<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}").chars().take(140).collect::<String>().replace(' ', "")
}

fn violations_text(image: &MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image).into_iter().map(|l| l.chars().take(110).collect::<String>().replace(' ', "")).collect();
    if v.is_empty() { "-".into() } else { v.join("|") }
}

fn covered(placements: &[singlefs_core::allocator::PlacementOnDevice], w: &RetainedWrite) -> bool {
    placements.iter().any(|p| {
        let start = p.slot.to_device_offset().0;
        p.device == w.device && start <= w.offset.0 && w.offset.0 < start + p.span * singlefs_format::SLOT_BYTES
    })
}

/// 一段历史：录好的 Q、R，挂载那一刻的故障，挂上之后的用户动作（字母串）。交回一行结局与这段历史枚举过的崩溃状态数。
fn run_one(chain: &Chain, faults: &str, actions: &str, crash_steps: &str) -> (String, bool, u64) {
    let mut crash_image = chain.base.clone();
    crash_image.apply_writes(&chain.writes[..=chain.r_root]);
    let parameters = chain.pool.parameters.clone();
    let stream = SharedStream::retaining_contents();
    let shared = Fault { ranges: Rc::new(RefCell::new(ranges_at_mount(chain, faults))), active: Rc::new(Cell::new(true)), hits: Rc::new(Cell::new(0)) };
    let mut devices: Vec<(DeviceIdentity, Dev)> = crash_image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(crash_image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, Dev { identity: *identity, inner: RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()), fault: shared.clone() })
        })
        .collect();
    let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return (format!("mount=Err:{}", short(&e)), false, 0),
    };
    shared.active.set(std::env::var("OPUS_PERSIST").is_ok());
    let selected = (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0);
    let occupancy = mounted.allocator.root_ring_occupancy().expect("根环表");
    let c_all = occupancy.placements_named_by_unapplied_journal_records();
    let c_r = occupancy
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(chain.r_key.0, chain.r_key.1)
        .unwrap_or_default();
    let q_units: Vec<&RetainedWrite> = chain.writes[..chain.q_root].iter().filter(|w| w.kind == StepKind::UnitWrite).collect();
    let q_in_c_r = q_units.iter().filter(|w| covered(&c_r, w)).count();
    let q_in_c_all = q_units.iter().filter(|w| covered(&c_all, w)).count();
    let diag = format!(
        "selected={selected:?} c_all={} c_r={} q_unit_writes={} q_in_c_r={q_in_c_r} q_in_c_all={q_in_c_all} faulted_reads={}",
        c_all.len(), c_r.len(), q_units.len(), shared.hits.get()
    );
    let (output, session) = MountedSession::of_the_mount(mounted);
    let Some(content) = chain.pool.content_of_each_root.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned() else {
        return (format!("{diag} mount=ok-unknown-content"), false, 0);
    };
    let mut content_of_each_root = chain.pool.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert((root.instance, root.checkpoint_txg), content.clone());
    }
    let mut pool = PoolUnderTest {
        parameters: parameters.clone(),
        device_bytes: chain.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: chain.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content,
        content_of_each_root,
    };
    let stepcheck = std::env::var("OPUS_STEPCHECK").is_ok();
    let mut steps = Vec::new();
    let mut first_red_step = String::from("-");
    let mut windows: Vec<(usize, MemoryPool, usize, usize)> = Vec::new();
    let mut versions_seen = vec![*pool.session().current.root()];
    let mut hit = false;
    let mut first_preroot_red = String::from("-");
    for (index, action) in actions.chars().enumerate() {
        let before_image = pool.image();
        let before_ops = stream.operation_count();
        let outcome = step(&mut pool, action, &mut versions_seen, &stream);
        windows.push((index, before_image, before_ops, stream.operation_count()));
        let stop = outcome.starts_with("Err") || pool.session.is_none();
        let mut extra = String::new();
        if std::env::var("OPUS_PREROOT").is_ok() {
            // 这一步最后一条根槽写之前的那个状态（层 0 的段边界前缀：单元、记录都落了，这一步最后那条根没落）过 checker。
            let operations = stream.retained_operations();
            let (writes, _segments, _ix) = writes_and_segments_with_stream_indexes(&operations[before_ops..], &fixed_geometry_of_the_384_slot_small_devices());
            if let Some(last_root) = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua) {
                let mut state = windows.last().unwrap().1.clone();
                state.apply_writes(&writes[..last_root]);
                let r_in = common_unwitnessed_publish::root_is_in_the_ring(&state, chain.r_key);
                let q_over = q_units.iter().filter(|w| {
                    let length = usize::try_from(w.length_in_bytes()).unwrap();
                    !w.contents.still_on_disk(&state.devices[&w.device].read(w.offset, length))
                }).count();
                let v = violations_text(&state);
                if v.contains("I-7.4") || v.contains("I-4.8") {
                    hit = true;
                    if first_preroot_red == "-" {
                        first_preroot_red = format!("{index}{action}:r_in_ring={r_in}:q_units_overwritten={q_over}:{v}");
                    }
                }
                extra = format!("[r_in={r_in},q_over={q_over}]");
            }
        }
        steps.push(format!("{action}={outcome}{extra}"));
        if stepcheck && first_red_step == "-" {
            let v = violations_text(&pool.image());
            if v.contains("I-7.4") || v.contains("I-4.8") {
                first_red_step = format!("{index}{action}:{v}");
            }
        }
        if stop {
            break;
        }
    }
    let final_violations = violations_text(&pool.image());
    hit = hit || final_violations.contains("I-7.4") || final_violations.contains("I-4.8") || first_red_step != "-";
    let mut crash_text = Vec::new();
    let mut states_run = 0u64;
    if crash_steps != "none" {
        let operations = stream.retained_operations();
        let versions = pool.published_versions();
        let last = windows.len().saturating_sub(1);
        for (index, base, from, to) in &windows {
            let chosen = match crash_steps {
                "all" => true,
                "last" => *index == last,
                list => list.split('+').any(|s| s.parse::<usize>().ok() == Some(*index)),
            };
            if !chosen || from == to {
                continue;
            }
            let (writes, segments, _ix) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(
                &operations[*from..*to],
                &spans_between(&stream, *from, *to),
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            let closed_form = closed_form_state_count(&segments);
            let Some(judged) = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua) else {
                crash_text.push(format!("c{index}=no-root"));
                continue;
            };
            if closed_form > 1_000_000 {
                crash_text.push(format!("c{index}=skipped({closed_form})"));
                continue;
            }
            let tally = singlefs_checker_tier::crash::enumerate_layer0_versions(base, &writes, &segments, judged, &versions);
            states_run += tally.states;
            let red = |name: &str| tally.checker_violated_states.get(name).copied().unwrap_or(0);
            if red("I-7.4") > 0 || red("I-4.8") > 0 {
                hit = true;
            }
            crash_text.push(format!(
                "c{index}=states:{} I-7.4:{} I-4.8:{} oracle:{} first74:{}",
                tally.states, red("I-7.4"), red("I-4.8"), tally.violations,
                tally.checker_first_violation.get("I-7.4").map_or("-".into(), |s| s.chars().take(100).collect::<String>().replace(' ', ""))
            ));
        }
    }
    let isolated: String = pool.session.as_ref().map_or("no-session".into(), |session| {
        session.allocator.devices.iter().map(|map| map.isolated_slots().to_string()).collect::<Vec<_>>().join("+")
    });
    (
        format!(
            "{diag} {} first_red_step={first_red_step} first_preroot_red={first_preroot_red} isolated_at_end={isolated} final={final_violations} crash=[{}]",
            steps.join(" "),
            crash_text.join(";")
        ),
        hit,
        states_run,
    )
}

/// 落在 [from, to) 里的入口段，下标挪成从 from 起数。
fn spans_between(stream: &SharedStream, from: usize, to: usize) -> Vec<singlefs_harness::RecordedEntrySpan> {
    stream
        .entry_spans()
        .into_iter()
        .filter(|span| span.operations.start >= from && span.operations.end <= to)
        .map(|span| singlefs_harness::RecordedEntrySpan { entry: span.entry, operations: (span.operations.start - from)..(span.operations.end - from) })
        .collect()
}
/// 一个用户动作：o 覆盖写、n 建一个 inode、s 顺序写两个数据单元、W 覆盖写到准入推出一串抬 F（至多 40 次）、
/// F 抬 F 到准入上限、b 管理员回退到上一版、u 正常卸载。
fn step(pool: &mut PoolUnderTest<Dev>, action: char, versions_seen: &mut Vec<singlefs_core::root_record::RootRecord>, stream: &SharedStream) -> String {
    let user = |r: Result<singlefs_core::mounted_session::UserChangePublished, singlefs_core::mounted_session::UserChangeRefused>| match r {
        Ok(p) => format!("ok(raises={})", p.floor_raises.len()),
        Err(e) => format!("Err:{}", short(&e)),
    };
    let text = match action {
        'o' => user(pool.overwrite(OVERWRITE_BYTES)),
        'n' => user(pool.create_one_inode()),
        's' => user(pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())),
        'W' => {
            let mut text = String::from("no-raise(40)");
            for count in 1..=40 {
                match pool.overwrite(OVERWRITE_BYTES) {
                    Ok(p) if !p.floor_raises.is_empty() => {
                        text = format!("ok(after={count})");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        text = format!("Err@{count}:{}", short(&e));
                        break;
                    }
                }
            }
            text
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
                    format!("ok(raised,unreadable={})", raised.abandoned_roots_unreadable)
                }
                Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }) => "ok(at-ceiling)".into(),
                Err(e) => format!("Err:{}", short(&e)),
            }
        }
        'b' => {
            if versions_seen.len() < 2 {
                "skip".into()
            } else {
                let target = versions_seen[versions_seen.len() - 2];
                let parameters = pool.parameters.clone();
                let mut session = pool.session.take().expect("会话");
                let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
                let outcome = singlefs_core::mount::roll_back_by_a_forward_publish(
                    &parameters,
                    &mut pool.devices,
                    &mut session.allocator,
                    current,
                    singlefs_core::mount::RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg },
                );
                pool.session = Some(session);
                match outcome {
                    Ok(_) => {
                        if let Some(c) = pool.content_of_each_root.get(&(target.instance, target.checkpoint_txg)).cloned() {
                            pool.content_of_the_current_version = c;
                        }
                        let root = *pool.session().current.root();
                        pool.note_root_of_the_current_content(&root);
                        "ok".into()
                    }
                    Err(e) => format!("Err:{}", short(&e)),
                }
            }
        }
        'u' => {
            let parameters = pool.parameters.clone();
            let mut session = pool.session.take().expect("会话");
            let devices = &mut pool.devices;
            let outcome = stream.record_entry(singlefs_harness::RecordedPublishEntry::Unmount, || unmount(&parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On));
            match outcome {
                Ok(Unmounted::FloorRaisedToTheCurrentVersion(r)) => {
                    let roots: Vec<_> = r.publishes.iter().map(|p| p.root).collect();
                    for root in &roots {
                        pool.note_root_of_the_current_content(root);
                    }
                    format!("ok(unreadable={})", r.abandoned_roots_unreadable)
                }
                Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "ok-nothing".into(),
                Err(e) => format!("Err:{}", short(&e)),
            }
        }
        other => panic!("动作 {other}"),
    };
    if let Some(session) = pool.session.as_ref() {
        versions_seen.push(*session.current.root());
    }
    text
}


#[test]
fn opus_sweep() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let arm = env("OPUS10ZGHI_ARM", "bing");
    let ks: Vec<u64> = env("OPUS_KS", "3").split(',').map(|s| s.parse().unwrap()).collect();
    let qs: Vec<char> = env("OPUS_Q", "s").chars().filter(|c| *c != ',').collect();
    let rs: Vec<char> = env("OPUS_R", "n").chars().filter(|c| *c != ',').collect();
    let fault_sets: Vec<String> = env("OPUS_FAULTS", "cfgq,qroot,qlast,racct").split(';').map(String::from).collect();
    let crash_steps = env("OPUS_CRASH_STEPS", "none");
    let actions_spec = env("OPUS_ACTIONS", "o");
    let actions: Vec<String> = if let Some(spec) = actions_spec.strip_prefix("RAND") {
        let parts: Vec<&str> = spec.split(':').collect();
        let (count, mut state, prefix, shortest, longest): (usize, u64, &str, usize, usize) =
            (parts[0].parse().unwrap(), parts[1].parse().unwrap(), parts[2], parts[3].parse().unwrap(), parts[4].parse().unwrap());
        let letters: Vec<char> = env("OPUS_ALPHABET", "ons").chars().collect();
        let mut next = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            usize::try_from(state >> 33).unwrap()
        };
        (0..count)
            .map(|_| {
                let length = shortest + next() % (longest - shortest + 1);
                let tail: String = (0..length).map(|_| letters[next() % letters.len()]).collect();
                format!("{prefix}{tail}")
            })
            .collect()
    } else {
        actions_spec.split(',').map(String::from).collect()
    };
    let mut states_total = 0u64;
    let mut histories = 0u64;
    for k in &ks {
        for q in &qs {
            for r in &rs {
                let Some(chain) = record_chain(*k, *q, *r) else {
                    println!("OPUS10ZGHI arm={arm} k={k} q={q} r={r} record=skipped(roots!=2)");
                    continue;
                };
                for faults in &fault_sets {
                    for a in &actions {
                        let started = std::time::Instant::now();
                        let (line, hit, states) = run_one(&chain, faults, a, &crash_steps);
                        states_total += states;
                        histories += 1;
                        println!(
                            "OPUS10ZGHI arm={arm} k={k} q={q} r={r} p={:?} qk={:?} rk={:?} faults={faults} actions={a} hit={hit} secs={:.2} states={states} {line}",
                            (chain.p_key.0 .0, chain.p_key.1 .0),
                            (chain.q_key.0 .0, chain.q_key.1 .0),
                            (chain.r_key.0 .0, chain.r_key.1 .0),
                            started.elapsed().as_secs_f64(),
                        );
                    }
                }
            }
        }
    }
    println!("OPUS10ZGHI_DONE arm={arm} histories={histories} crash_states_total={states_total}");
}
