//! m2-closeout-code-r3 云端攻方腿 X5 ①：第二轮 Y1 的单故障逐序号扫换一组历史再抽一次。
//! 起点：两块单元区 384 槽的小盘（journal 环 6 MiB）上 mkfs、取号、暖机、第一个文件，再按种子随机走几步
//! （覆盖写、建一个 inode、管理员回退、准入抬 F、崩了再可写挂载）。每个起点只建一次，之后每段从内存里那一份拷。
//! 入口：管理员回退（到一个候选）、准入抬 F、正常卸载、会话覆盖写、崩了再可写挂载。每段只注入一次（整池第 n 次读、写或屏障报块设备错）。
//! 每段之后两件：(a) 第二轮那一套——有冻结的就撤掉故障原样重发、盘面池级 checker、拷一份崩了再可写挂载、再 checker、读回内容对不对；
//! (b) 入口一返回立刻掉电：录下的写流（从起点镜像起、到入口返回为止）按层 0 的枚举域全量展开（`enumerate_layer0_versions`），
//! 每个崩溃状态恢复、oracle、池级 checker、记录核对器。状态数先按 `closed_form_state_count` 与带第三态的精确数算，一段超过
//! `OPUS_R3_PER_STREAM_LIMIT`（缺省 10⁶）的不跑、整条腿累计超过 `OPUS_R3_BUDGET` 的不跑，都打出来。
//! 环境变量：`OPUS_R3_SEEDS`（逗号分隔，缺省 1..=6）、`OPUS_R3_ENTRIES`、`OPUS_R3_KINDS`（none,read,write,barrier）、
//! `OPUS_R3_ENUMERATE`（0 关 (b)）、`OPUS_R3_READ_STRIDE`（读序号步长，缺省 1）、`OPUS_R3_BUDGET`、`OPUS_R3_PER_STREAM_LIMIT`。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use common_admission::{checker_violations_on, content_of, start_plain, OVERWRITE_BYTES};
use singlefs_checker_tier::crash::{
    enumerate_layer0_versions, layer0_state_count_with_torn_in_place_overwrites,
    Layer0SegmentExpansion,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    unmount, RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{resend_the_frozen_publish, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_of_write, writes_and_segments_with_stream_indexes_and_entries, MemoryPool,
    PublishedVersion, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedPublishEntry, RecordingBlockDevice, SharedStream};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Read,
    Write,
    Barrier,
}

#[derive(Default)]
struct State {
    target: Option<(Kind, u64)>,
    reads: u64,
    writes: u64,
    barriers: u64,
    fired: Option<(Kind, u64, u32, u64)>,
    writes_before_fire: u64,
    writes_after_fire: u64,
}

#[derive(Clone, Default)]
struct Shared(Rc<RefCell<State>>);

impl Shared {
    fn decide(&self, kind: Kind, device: u32, offset: u64) -> bool {
        let mut state = self.0.borrow_mut();
        let ordinal = match kind {
            Kind::Read => {
                state.reads += 1;
                state.reads
            }
            Kind::Write => {
                state.writes += 1;
                state.writes
            }
            Kind::Barrier => {
                state.barriers += 1;
                state.barriers
            }
        };
        if state.target == Some((kind, ordinal)) && state.fired.is_none() {
            state.fired = Some((kind, ordinal, device, offset));
            return true;
        }
        false
    }
    fn note_write(&self) {
        let mut state = self.0.borrow_mut();
        if state.fired.is_some() {
            state.writes_after_fire += 1;
        } else {
            state.writes_before_fire += 1;
        }
    }
    fn disarm(&self) {
        self.0.borrow_mut().target = None;
    }
}

struct SweepDevice {
    inner: SparseBlockDevice,
    device: u32,
    shared: Shared,
}

impl BlockDevice for SweepDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.shared.decide(Kind::Read, self.device, offset.0) {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.shared.decide(Kind::Write, self.device, offset.0) {
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)?;
        self.shared.note_write();
        Ok(())
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        if self.shared.decide(Kind::Write, self.device, offset.0) {
            return Err(injected_block_device_error("清零"));
        }
        self.inner.write_zeroes_at(offset, length)?;
        self.shared.note_write();
        Ok(())
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self.shared.decide(Kind::Barrier, self.device, 0) {
            return Err(injected_block_device_error("屏障"));
        }
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

type Device = RecordingBlockDevice<SweepDevice>;

/// 一个起点：盘面、会话、每条根带的文件内容、这一刻现行那一版的内容，与走过的步骤。
struct Base {
    seed: u64,
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: MemoryPool,
    session: MountedSession,
    write_time_seconds: u64,
    content_of_each_root: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    current_content: Vec<u8>,
    steps: Vec<String>,
    rollback_target: Option<RollbackTarget>,
}

fn clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

/// 候选：记过内容的根里 txg ≥ 现行 F、比现行那一版旧的（带文件）；按 txg 排。
fn rollback_candidates(
    content_of_each_root: &BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    session: &MountedSession,
) -> Vec<RollbackTarget> {
    let current = *session.current.root();
    content_of_each_root
        .keys()
        .filter(|(_, txg)| *txg >= current.rollback_floor && *txg < current.checkpoint_txg)
        .map(|(instance, checkpoint_txg)| RollbackTarget { instance: *instance, checkpoint_txg: *checkpoint_txg })
        .collect()
}

fn short(text: String) -> String {
    let cut: String = text.chars().take(200).collect();
    cut.replace('\n', " ")
}

fn build_base(seed: u64) -> Base {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    let mut rng = Lcg(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x5eed);
    // `OPUS_R3_LONG=1`：长历史档，步数 6–9、覆盖写的权重 3/8（凑够 4 个非空版本，准入抬 F 才抬得动、F 才离开 0）。
    let long = env_u64("OPUS_R3_LONG", 0) == 1;
    let length = if long { 6 + rng.next() % 4 } else { 3 + rng.next() % 4 };
    let mut steps = Vec::new();
    for _ in 0..length {
        let choice = if long { [0, 0, 1, 2, 3, 4, 5, 4][usize::try_from(rng.next() % 8).expect("小")] } else { rng.next() % 6 };
        let step = match choice {
            0 | 1 => format!("overwrite:{:?}", pool.overwrite(OVERWRITE_BYTES).map(|_| ()).map_err(|error| short(format!("{error:?}")))),
            2 => format!("create_one_inode:{:?}", pool.create_one_inode().map(|_| ()).map_err(|error| short(format!("{error:?}")))),
            3 => {
                let mut session = pool.session.take().expect("会话");
                let candidates = rollback_candidates(&pool.content_of_each_root, &session);
                let text = if candidates.is_empty() {
                    "rollback:no_candidate".to_string()
                } else {
                    let target = candidates[usize::try_from(rng.next()).expect("小") % candidates.len()];
                    let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
                    let outcome = roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut session.allocator, current, target);
                    let ok = outcome.is_ok();
                    let text = format!("rollback_to_{}_{}:{}", target.instance.0, target.checkpoint_txg.0, match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("{error:?}")) });
                    if ok {
                        pool.content_of_the_current_version = pool.content_of_each_root[&(target.instance, target.checkpoint_txg)].clone();
                        let root = *session.current.root();
                        pool.session = Some(session);
                        pool.note_root_of_the_current_content(&root);
                        steps.push(text);
                        continue;
                    }
                    text
                };
                pool.session = Some(session);
                text
            }
            4 => {
                let mut session = pool.session.take().expect("会话");
                let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
                let outcome = raise_rollback_floor_to_the_admission_ceiling(&pool.parameters, &mut pool.devices, &mut session.allocator, current, ShadowLedger::On);
                pool.session = Some(session);
                match outcome {
                    Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
                        pool.note_floor_raises(std::slice::from_ref(&raised));
                        format!("raise_floor:Raised_to_{}", raised.ceiling.0)
                    }
                    Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => format!("raise_floor:AtCeiling_{}_{}", floor.0, ceiling.0),
                    Err(error) => format!("raise_floor:{}", short(format!("{error:?}"))),
                }
            }
            _ => format!("crash_remount:{:?}", pool.crash_and_mount_writable().map(|_| ()).map_err(|error| short(format!("{error:?}")))),
        };
        steps.push(step);
    }
    let session = pool.session.take().expect("会话");
    let candidates = rollback_candidates(&pool.content_of_each_root, &session);
    let rollback_target = if candidates.is_empty() { None } else { Some(candidates[usize::try_from(rng.next()).expect("小") % candidates.len()]) };
    Base {
        seed,
        parameters: pool.parameters.clone(),
        image: pool.image(),
        session,
        write_time_seconds: pool.write_time_seconds,
        content_of_each_root: pool.content_of_each_root.clone(),
        current_content: pool.content_of_the_current_version.clone(),
        steps,
        rollback_target,
    }
}

fn devices_on(image: &MemoryPool, shared: &Shared, stream: &SharedStream) -> Vec<(DeviceIdentity, Device)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            inner.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, SweepDevice { inner, device: identity.0, shared: shared.clone() }, stream.clone()),
            )
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, Device)], bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().inner.image.clone())).collect(),
        device_size_in_bytes: bytes,
    }
}

const ENTRIES: [&str; 5] = ["rollback", "raise_floor_to_the_admission_ceiling", "unmount", "session_overwrite", "crash_remount"];

/// 跑一个入口：交回结局、这一次之后的会话（崩了再挂那一格是新会话）、入口写的根该带的内容（`None`：按根环里最新那条的内容）。
fn run_entry(entry: &str, base: &Base, devices: &mut Vec<(DeviceIdentity, Device)>, stream: &SharedStream) -> (String, Option<MountedSession>, Vec<u8>) {
    let mut session = clone_session(&base.session);
    match entry {
        "session_overwrite" => {
            let write_time_seconds = base.write_time_seconds + 1;
            let content = content_of(OVERWRITE_BYTES, write_time_seconds);
            let outcome = session.publish_user_change(devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds }));
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session), content)
        }
        "unmount" => {
            let outcome = stream.record_entry(RecordedPublishEntry::Unmount, || unmount(&base.parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On));
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session), base.current_content.clone())
        }
        "raise_floor_to_the_admission_ceiling" => {
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
            let outcome = raise_rollback_floor_to_the_admission_ceiling(&base.parameters, devices, &mut session.allocator, current, ShadowLedger::On);
            (match outcome {
                Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => format!("Ok Raised_to_{}", raised.ceiling.0),
                Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }) => "Ok AtCeiling".to_string(),
                Err(error) => short(format!("Err {error:?}")),
            }, Some(session), base.current_content.clone())
        }
        "rollback" => {
            let target = base.rollback_target.expect("有回退目标才跑这一格");
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
            let outcome = roll_back_by_a_forward_publish(&base.parameters, devices, &mut session.allocator, current, target);
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session), base.content_of_each_root[&(target.instance, target.checkpoint_txg)].clone())
        }
        "crash_remount" => match mount_writable(&base.parameters, devices) {
            Ok(mounted) => {
                let (_, session) = MountedSession::of_the_mount(mounted);
                ("Ok".to_string(), Some(session), base.current_content.clone())
            }
            Err(error) => (short(format!("Err {error:?}")), None, base.current_content.clone()),
        },
        other => panic!("没有这个入口 {other}"),
    }
}

fn env_list(name: &str, default: &[&str]) -> Vec<String> {
    std::env::var(name)
        .map(|value| value.split(',').map(str::to_string).collect())
        .unwrap_or_else(|_| default.iter().map(|entry| entry.to_string()).collect())
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

/// 这份镜像上读回的内容：看 journal 那一遍恢复择到哪条根、读出什么。
fn recovered(image: &MemoryPool) -> (Option<(InstanceGeneration, CheckpointTxg)>, Option<Vec<u8>>, String) {
    let report = recover(image, JournalPolicy::Consult);
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => (Some(root), Some(content), "FileRead".to_string()),
        RecoveryOutcome::NoFile { root } => (Some(root), None, "NoFile".to_string()),
        RecoveryOutcome::Failed { root, failure } => (root, None, short(format!("Failed {failure:?}"))),
    }
}

#[test]
fn single_fault_ordinal_sweep_then_power_loss_over_seeded_histories() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = env_list("OPUS_R3_SEEDS", &["1", "2", "3", "4", "5", "6"]).iter().map(|seed| seed.parse().expect("种子")).collect();
    let entries = env_list("OPUS_R3_ENTRIES", &ENTRIES);
    let kinds = env_list("OPUS_R3_KINDS", &["none", "read", "write", "barrier"]);
    let enumerate = env_u64("OPUS_R3_ENUMERATE", 1) == 1;
    let enumerate_kinds = env_list("OPUS_R3_ENUMERATE_KINDS", &["none", "read", "write", "barrier"]);
    let read_stride = env_u64("OPUS_R3_READ_STRIDE", 1);
    let budget = env_u64("OPUS_R3_BUDGET", 3_000_000);
    let per_stream_limit = env_u64("OPUS_R3_PER_STREAM_LIMIT", 1_000_000);
    let mut states_spent: u64 = 0;
    let mut runs = 0u64;
    let mut flagged: Vec<String> = Vec::new();
    let mut skipped_enumerations: Vec<String> = Vec::new();
    for seed in seeds {
        let base_started = std::time::Instant::now();
        let base = build_base(seed);
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: base.parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: base.parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: base.parameters.geometry.root_ring_slots_per_region,
        };
        let current_root = *base.session.current.root();
        println!(
            "BASE seed={seed} steps={:?} current=({},{}) floor={} rollback_target={:?} seconds={:.2}",
            base.steps, current_root.instance.0, current_root.checkpoint_txg.0, current_root.rollback_floor.0,
            base.rollback_target.map(|target| (target.instance.0, target.checkpoint_txg.0)), base_started.elapsed().as_secs_f64()
        );
        let base_violations = checker_violations_on(&base.image);
        if !base_violations.is_empty() {
            flagged.push(format!("BASE seed={seed} checker={base_violations:?}"));
        }
        let bytes = base.image.device_size_in_bytes;
        for entry in &entries {
            if entry == "rollback" && base.rollback_target.is_none() {
                println!("SWEEP seed={seed} entry=rollback skipped=no_candidate");
                continue;
            }
            let shared = Shared::default();
            let stream = SharedStream::retaining_contents();
            let mut devices = devices_on(&base.image, &shared, &stream);
            let (outcome, _, _) = run_entry(entry, &base, &mut devices, &stream);
            let (reads, writes, barriers) = {
                let state = shared.0.borrow();
                (state.reads, state.writes, state.barriers)
            };
            println!("SWEEP seed={seed} entry={entry} clean outcome={outcome} reads={reads} writes={writes} barriers={barriers}");
            let mut faults: Vec<Option<(Kind, u64)>> = Vec::new();
            for kind_name in &kinds {
                match kind_name.as_str() {
                    "none" => faults.push(None),
                    "read" => faults.extend((1..=reads).filter(|ordinal| read_stride == 1 || ordinal % read_stride == 0 || *ordinal > reads.saturating_sub(300)).map(|ordinal| Some((Kind::Read, ordinal)))),
                    "write" => faults.extend((1..=writes).map(|ordinal| Some((Kind::Write, ordinal)))),
                    "barrier" => faults.extend((1..=barriers).map(|ordinal| Some((Kind::Barrier, ordinal)))),
                    other => panic!("没有这种 {other}"),
                }
            }
            for fault in faults {
                runs += 1;
                let shared = Shared::default();
                shared.0.borrow_mut().target = fault;
                let stream = SharedStream::retaining_contents();
                let mut devices = devices_on(&base.image, &shared, &stream);
                let (outcome, session, content_of_the_entry) = run_entry(entry, &base, &mut devices, &stream);
                let (fired, before, after_writes) = {
                    let state = shared.0.borrow();
                    (state.fired, state.writes_before_fire, state.writes_after_fire)
                };
                // (b) 入口一返回立刻掉电：录下的写流全量展开。
                let operations = stream.retained_operations();
                let mut enumeration = "-".to_string();
                let kind_name_of_the_fault = match fault { None => "none", Some((Kind::Read, _)) => "read", Some((Kind::Write, _)) => "write", Some((Kind::Barrier, _)) => "barrier" };
                if enumerate && !enumerate_kinds.iter().any(|kind| kind == kind_name_of_the_fault) {
                    enumeration = "not_selected".to_string();
                } else if enumerate {
                    let (stream_writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&operations, &stream.entry_spans(), &geometry);
                    if stream_writes.is_empty() {
                        enumeration = "no_writes".to_string();
                    } else {
                        let closed = closed_form_state_count(&segments);
                        let exact = layer0_state_count_with_torn_in_place_overwrites(&base.image, &stream_writes, &segments, &|_, _| Layer0SegmentExpansion::EveryProperSubset);
                        if closed > per_stream_limit || exact > per_stream_limit {
                            enumeration = format!("skipped_over_per_stream_limit closed={closed} exact={exact}");
                            skipped_enumerations.push(format!("seed={seed} entry={entry} fault={fault:?} closed={closed} exact={exact} reason=per_stream"));
                        } else if states_spent + exact > budget {
                            enumeration = format!("skipped_over_budget closed={closed} exact={exact}");
                            skipped_enumerations.push(format!("seed={seed} entry={entry} fault={fault:?} closed={closed} exact={exact} reason=budget"));
                        } else {
                            let mut versions: Vec<PublishedVersion> = base
                                .content_of_each_root
                                .iter()
                                .map(|((instance, checkpoint_txg), content)| PublishedVersion { instance: *instance, checkpoint_txg: *checkpoint_txg, content: content.clone() })
                                .collect();
                            let root_writes: Vec<usize> = stream_writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::RootRecordFua).map(|(index, _)| index).collect();
                            for index in &root_writes {
                                let (instance, checkpoint_txg) = root_identity_of_write(&stream_writes[*index]);
                                if !versions.iter().any(|version| version.instance == instance && version.checkpoint_txg == checkpoint_txg) {
                                    versions.push(PublishedVersion { instance, checkpoint_txg, content: content_of_the_entry.clone() });
                                }
                            }
                            // 入口的发布在根槽写之前失败时，恢复可以靠 journal 记录把那一版施加出来（「由记录施加出来的版本」）：
                            // 它的 (实例, txg) 是现行那一版之后的那几个 txg，内容与入口写的根相同。没有根槽写可认，按 txg 补上。
                            for step in 1..=4u64 {
                                let (instance, checkpoint_txg) = (current_root.instance, CheckpointTxg(current_root.checkpoint_txg.0 + step));
                                if !versions.iter().any(|version| version.instance == instance && version.checkpoint_txg == checkpoint_txg) {
                                    versions.push(PublishedVersion { instance, checkpoint_txg, content: content_of_the_entry.clone() });
                                }
                            }
                            let judged = root_writes.last().copied().unwrap_or(0);
                            let enumeration_started = std::time::Instant::now();
                            let tally = enumerate_layer0_versions(&base.image, &stream_writes, &segments, judged, &versions);
                            let enumeration_seconds = enumeration_started.elapsed().as_secs_f64();
                            states_spent += tally.states;
                            let checker_red: u64 = tally.checker_violated_states.values().sum();
                            enumeration = format!(
                                "states={} seconds={enumeration_seconds:.1} closed={closed} exact={exact} oracle={} ignored={} root_without_record={} missing_unit={} checker_red_states={checker_red} first={:?} first_checker={:?}",
                                tally.states, tally.violations, tally.ignored_violations, tally.record_root_without_record, tally.record_claimed_state_missing_unit,
                                tally.first_violation, tally.checker_first_violation.iter().next()
                            );
                            if tally.states != exact || tally.violations > 0 || tally.record_root_without_record > 0 || tally.record_claimed_state_missing_unit > 0 || checker_red > 0 {
                                flagged.push(format!("ENUM seed={seed} entry={entry} fault={fault:?} {enumeration}"));
                            }
                        }
                    }
                }
                // (a) 第二轮那一套：撤掉故障，冻结的原样重发；盘面 checker；拷一份崩了再可写挂载，再 checker，读回内容。
                shared.disarm();
                let mut frozen = false;
                let mut resend = "-".to_string();
                if let Some(mut session) = session {
                    if session.allocator.frozen_publish().is_some() {
                        frozen = true;
                        let parameters = session.parameters_and_device_table.parameters_on_disk().clone();
                        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                        resend = match resend_the_frozen_publish(&mut writer, &mut session.allocator) {
                            Ok(Some(_)) => "Ok".to_string(),
                            Ok(None) => "None".to_string(),
                            Err(error) => short(format!("Err {error:?}")),
                        };
                    }
                }
                let image = image_of(&devices, bytes);
                let violations = checker_violations_on(&image);
                let (read_root, read_content, read_outcome) = recovered(&image);
                let content_ok = match (&read_root, &read_content) {
                    (Some(root), Some(content)) => base.content_of_each_root.get(root).is_some_and(|known| known == content) || *content == content_of_the_entry,
                    _ => false,
                };
                let mut remount_devices = common_admission::plain_devices_on(&image);
                let (remount, violations_after_remount) = match mount_writable(&base.parameters, &mut remount_devices) {
                    Ok(_) => {
                        let after = MemoryPool {
                            devices: remount_devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
                            device_size_in_bytes: bytes,
                        };
                        ("Ok".to_string(), checker_violations_on(&after))
                    }
                    Err(error) => (short(format!("Err {error:?}")), Vec::new()),
                };
                let line = format!(
                    "SWEEP seed={seed} entry={entry} fault={fault:?} fired={fired:?} outcome={outcome} writes_before={before} writes_after={after_writes} frozen={frozen} resend={resend} checker={} first={:?} read={read_outcome} root={read_root:?} content_ok={content_ok} remount={remount} checker_after_remount={} first_after={:?} enum={enumeration}",
                    violations.len(), violations.first(), violations_after_remount.len(), violations_after_remount.first()
                );
                println!("{line}");
                let refused_after_writes = matches!(fault, Some((Kind::Read, _))) && outcome.starts_with("Err") && before > 0;
                if !violations.is_empty() || remount != "Ok" || !violations_after_remount.is_empty() || refused_after_writes || after_writes > 0 || !content_ok {
                    flagged.push(line);
                }
            }
        }
    }
    println!("SWEEP runs={runs} flagged={} states_spent={states_spent} skipped_enumerations={} seconds={:.1}", flagged.len(), skipped_enumerations.len(), started.elapsed().as_secs_f64());
    for line in &skipped_enumerations {
        println!("SKIP {line}");
    }
    for line in &flagged {
        println!("FLAG {line}");
    }
}
