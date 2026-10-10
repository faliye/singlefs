//! m3-batch10z-d-code-r1 攻方腿原型（草稿副本里的，不进仓）：K2-2 H2 借来的 C 那一份罩不罩得住 R 引用的全部单元。
//! 造盘照入库 `tests/common_unwitnessed_publish/mod.rs`：覆盖写 k 次后录 R（R 的种类放开），崩在 R 的根槽落盘之后、轮换之前（系统配置没见证 R）；
//! 可写挂载那一次 R 自己写出的某几个单元读回全零（R 走读不成、F2 退回 P），R 的根槽读得出、按 R 的表有效、根环表记成有效根，写行之后 R 被抛弃。
//! 挂上之后撤掉故障，用户动作放开扫；终态与（OPUS_CRASH=1 时）挂载起录的整条流的层 0 崩溃状态全量都过池级 checker。
//! 臂：`OPUS10ZD_ARM=today` 在抬 F 那一串不借 C（草稿副本 mount.rs 里一行开关），缺省是冻结副本原样（H2）。
#[path = "../../singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, device_bytes_of_the_384_slot_small_devices,
    units_of_r_overwritten_on, DeviceOverASparseImage, PoolUnderTest, RecordedPublish, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor_to_the_admission_ceiling, unmount,
    RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::root_ring::target_for_publish;
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
        if !self.fault.active.get() {
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

/// R 的种类：o 覆盖写一次，n 建一个 inode（数据单元不动，R 与 P 共用它们），s 顺序写三个数据单元。
#[derive(Clone, Copy, Debug)]
enum RKind {
    Overwrite,
    NewInode,
    Sequential3,
}
impl RKind {
    fn parse(s: &str) -> Self {
        match s {
            "o" => Self::Overwrite,
            "n" => Self::NewInode,
            "s" => Self::Sequential3,
            _ => panic!("R 的种类 {s}"),
        }
    }
}

/// 照入库 `record_one_publish_after_overwrites` 的录法，R 的种类放开（入库那一份只有覆盖写与顺序写三个数据单元）。
fn record_r(overwrites_before: u64, kind: RKind) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for _ in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let before = stream.operation_count();
    match kind {
        RKind::Overwrite => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
        RKind::NewInode => pool.create_one_inode().map(|_| ()),
        RKind::Sequential3 => pool
            .sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000)
            .map(|_| ()),
    }
    .expect("录的那次发布落得下");
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) =
        writes_and_segments_with_stream_indexes(&operations[before..], &fixed_geometry_of_the_384_slot_small_devices());
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, w)| w.kind == StepKind::RootRecordFua)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(root_indices.len(), 1, "录的那次发布只写一条根");
    RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    }
}

/// 挂载那一刻读回全零的几段：`acct` = R 自己写出的树表、中央映射树根、分配记录树根那几份；`tt` / `map` / `art` 只其中一样；
/// `data` = R 写出的单元里落在这三样之外的（入库 H2 用例那一形：R 的账读得出）；`none` 不藏。
fn ranges_at_mount(recorded: &RecordedPublish, fault: &str) -> Vec<(DeviceIdentity, u64, u64)> {
    let root = recorded.root();
    let pick = |pointers: Vec<singlefs_core::pointer::NodePointer>| -> Vec<(DeviceIdentity, u64, u64)> {
        pointers
            .iter()
            .flat_map(|p| recorded.unit_writes_at(p))
            .map(|w| (w.device, w.offset.0, w.length_in_bytes()))
            .collect()
    };
    match fault {
        "none" => Vec::new(),
        "acct" => pick(vec![root.tree_table, root.mapping_root, root.allocation_record_tree_root]),
        "tt" => pick(vec![root.tree_table]),
        "map" => pick(vec![root.mapping_root]),
        "art" => pick(vec![root.allocation_record_tree_root]),
        "data" => {
            let account: Vec<(DeviceIdentity, u64, u64)> =
                pick(vec![root.tree_table, root.mapping_root, root.allocation_record_tree_root]);
            recorded
                .unit_writes()
                .into_iter()
                .map(|w| (w.device, w.offset.0, w.length_in_bytes()))
                .filter(|r| !account.contains(r))
                .collect()
        }
        _ => panic!("故障 {fault}"),
    }
}

fn short<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}").chars().take(120).collect::<String>().replace(' ', "")
}

fn violations_text(image: &MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image)
        .into_iter()
        .map(|l| l.chars().take(90).collect::<String>().replace(' ', ""))
        .collect();
    if v.is_empty() { "-".into() } else { v.join("|") }
}

struct Outcome {
    line: String,
    hit: bool,
}

/// 一段历史：录好的 R、挂载那一刻的故障、挂上之后的用户动作（字母串）。
fn run_one(recorded: &RecordedPublish, fault: &str, actions: &str, crash: bool) -> Outcome {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let parameters = recorded.pool.parameters.clone();
    let stream = SharedStream::retaining_contents();
    let shared = Fault {
        ranges: Rc::new(RefCell::new(ranges_at_mount(recorded, fault))),
        active: Rc::new(Cell::new(true)),
        hits: Rc::new(Cell::new(0)),
    };
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
        Err(e) => return Outcome { line: format!("mount=Err:{}", short(&e)), hit: false },
    };
    // OPUS_PERSIST=1：挂上之后故障照旧（R 的账在抬 F 那一遍也读回全零），不撤。
    shared.active.set(std::env::var("OPUS_PERSIST").is_ok());
    let selected = (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0);
    let r_root = recorded.root();
    let occupancy = mounted.allocator.root_ring_occupancy().expect("根环表");
    let borrowed = occupancy
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(r_root.instance, r_root.checkpoint_txg)
        .map_or(-1, |p| i64::try_from(p.len()).unwrap());
    let recorded_abandoned = occupancy
        .placements_recorded_for_the_abandoned_root(r_root.instance, r_root.checkpoint_txg)
        .map_or(-1, |p| i64::try_from(p.len()).unwrap());
    let _r_slot = target_for_publish(r_root.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content = recorded
        .pool
        .content_of_each_root
        .get(&(output.effective_root.instance, output.effective_root.checkpoint_txg))
        .cloned()
        .expect("落到录过的一版");
    let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert((root.instance, root.checkpoint_txg), content.clone());
    }
    let mut pool = PoolUnderTest {
        parameters: parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content,
        content_of_each_root,
    };
    let mut steps = Vec::new();
    let mut first_red_step = String::from("-");
    let mut versions_seen = vec![*pool.session().current.root()];
    for (index, action) in actions.chars().enumerate() {
        let outcome = step(&mut pool, action, &mut versions_seen, &stream);
        let stop = outcome.starts_with("Err") || pool.session.is_none();
        steps.push(format!("{action}={outcome}"));
        if first_red_step == "-" && std::env::var("OPUS_STEPCHECK").is_ok() {
            let v = violations_text(&pool.image());
            if v.contains("I-7.4") || v.contains("I-4.8") {
                first_red_step = format!("{index}{action}");
            }
        }
        if stop {
            break;
        }
    }
    let image = pool.image();
    let isolated_at_end: String = pool.session.as_ref().map_or("no-session".into(), |session| {
        session.allocator.devices.iter().map(|map| map.isolated_slots().to_string()).collect::<Vec<_>>().join("+")
    });
    let final_violations = violations_text(&image);
    let mut crash_text = String::from("crash=not-run");
    let operations = stream.retained_operations();
    let (writes, segments, _indexes) =
        singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(&operations, &stream.entry_spans(), &fixed_geometry_of_the_384_slot_small_devices());
    let closed_form = closed_form_state_count(&segments);
    if std::env::var("OPUS_SEGDUMP").is_ok() {
        let mut big: Vec<(usize, usize)> = segments.iter().enumerate().map(|(i, s)| (s.len(), i)).collect();
        big.sort_unstable_by(|a, b| b.cmp(a));
        let mount_end = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
        println!("SEGDUMP writes={} segments={} first_root_write={mount_end} biggest={:?}", writes.len(), segments.len(),
            big.iter().take(6).map(|(n, i)| (*n, *i, segments[*i].first().copied(), writes[segments[*i][0]].kind)).collect::<Vec<_>>());
    }
    let mut crash_hit = false;
    if crash {
        if closed_form > 1_000_000 {
            crash_text = format!("crash=skipped(closed_form={closed_form})");
        } else {
            let judged = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
            let versions = pool.published_versions();
            let tally = singlefs_checker_tier::crash::enumerate_layer0_versions(&crash_image, &writes, &segments, judged, &versions);
            let red = |name: &str| tally.checker_violated_states.get(name).copied().unwrap_or(0);
            crash_hit = red("I-7.4") > 0 || red("I-4.8") > 0;
            crash_text = format!(
                "crash_states={} closed_form={closed_form} oracle_violations={} I-7.4={} I-4.8={} I-2.1={} I-7.2={} first74={}",
                tally.states, tally.violations, red("I-7.4"), red("I-4.8"), red("I-2.1"), red("I-7.2"),
                tally.checker_first_violation.get("I-7.4").map_or("-".into(), |s| s.chars().take(80).collect::<String>().replace(' ', ""))
            );
        }
    }
    let hit = final_violations.contains("I-7.4") || final_violations.contains("I-4.8") || crash_hit;
    Outcome {
        line: format!(
            "selected={selected:?} borrowed_at_mount={borrowed} recorded_abandoned={recorded_abandoned} faulted_reads={} {} r_own_units_overwritten={} first_red_step={first_red_step} isolated_at_end={isolated_at_end} final={final_violations} {crash_text}",
            shared.hits.get(),
            steps.join(" "),
            units_of_r_overwritten_on(recorded, &image).len(),
        ),
        hit,
    }
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
    let arm = std::env::var("OPUS10ZD_ARM").unwrap_or_else(|_| "h2".into());
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let crash = std::env::var("OPUS_CRASH").is_ok();
    let ks: Vec<u64> = env("OPUS_KS", "3").split(',').map(|s| s.parse().unwrap()).collect();
    let kinds: Vec<String> = env("OPUS_RKINDS", "n").split(',').map(String::from).collect();
    let faults: Vec<String> = env("OPUS_FAULTS", "acct").split(',').map(String::from).collect();
    let actions_spec = env("OPUS_ACTIONS", "Wo");
    let actions: Vec<String> = if let Some(spec) = actions_spec.strip_prefix("RAND") {
        // RAND<条数>:<种子>:<前缀>:<最短>:<最长>：前缀之后接随机长度、随机字母（OPUS_ALPHABET）的串。
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
    } else if let Some(n) = actions_spec.strip_prefix("ALL") {
        all_action_sequences(n.parse().unwrap(), &env("OPUS_ALPHABET", "onsFWb"))
    } else {
        actions_spec.split(',').map(String::from).collect()
    };
    let (shard, shards): (usize, usize) = {
        let text = env("OPUS_SHARD", "0/1");
        let (a, b) = text.split_once('/').unwrap();
        (a.parse().unwrap(), b.parse().unwrap())
    };
    let actions: Vec<String> = actions.into_iter().enumerate().filter(|(i, _)| i % shards == shard).map(|(_, a)| a).collect();
    for k in &ks {
        for kind in &kinds {
            let recorded = record_r(*k, RKind::parse(kind));
            for fault in &faults {
                for a in &actions {
                    let started = std::time::Instant::now();
                    let outcome = run_one(&recorded, fault, a, crash);
                    println!(
                        "OPUS10ZD arm={arm} k={k} rkind={kind} r={:?} p={:?} fault={fault} actions={a} hit={} secs={:.2} {}",
                        (recorded.key.0 .0, recorded.key.1 .0),
                        (recorded.previous_key.0 .0, recorded.previous_key.1 .0),
                        outcome.hit,
                        started.elapsed().as_secs_f64(),
                        outcome.line
                    );
                }
            }
        }
    }
}

/// 不带 u 的字母串长 0..=n−1 各接 u 或不接，再加长 n 不接 u 的（u 只在末尾）；去掉空串。
fn all_action_sequences(n: usize, alphabet: &str) -> Vec<String> {
    let letters: Vec<char> = alphabet.chars().collect();
    let mut by_length: Vec<Vec<String>> = vec![vec![String::new()]];
    for _ in 0..n {
        let next: Vec<String> = by_length.last().unwrap().iter().flat_map(|p| letters.iter().map(move |c| format!("{p}{c}"))).collect();
        by_length.push(next);
    }
    let mut out = Vec::new();
    for (length, seqs) in by_length.iter().enumerate() {
        for seq in seqs {
            if !seq.is_empty() {
                out.push(seq.clone());
            }
            if length < n {
                out.push(format!("{seq}u"));
            }
        }
    }
    out
}
