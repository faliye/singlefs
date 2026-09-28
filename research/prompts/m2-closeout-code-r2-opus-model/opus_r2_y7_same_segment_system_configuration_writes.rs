//! 代码轮第二轮云端攻方腿（m2-closeout-code-r2）Y7 的 A2a 与 Y6 的关段规则：同一段里同一块盘两次系统配置写（第一轮没量）。
//! 与 `opus_r2_y1_y7_single_fault_ordinal_sweep.rs` 同一个起点、同一组入口、同一种单故障（整池第 n 次写 / 屏障报一次块设备错，另加不注入那一趟），
//! 这里在注入层外面再包一层录制：入口之后冻结的那次原样重发、再在同一组盘上崩了再可写挂载，三步录在同一条流里；
//! 按 `segments::SegmentClosingRule` 切段，数每一段里每块盘的系统配置槽写，另数「FUA 写之前同一块盘上同一段里还有没被屏障放行的普通写」。
//! 环境变量：`OPUS_Y1_ENTRIES`（逗号分隔的入口名，缺省全部）、`OPUS_Y1_KINDS`（read,write,barrier，缺省全部）、
//! `OPUS_Y1_STRIDE`（读序号的步长，缺省 1；大于 1 时前 `OPUS_Y1_READ_HEAD`（缺省 1000）次与末 `OPUS_Y1_READ_TAIL`（缺省 3000）次仍逐个）、`OPUS_Y1_COUNT_ONLY=1`（只数每个入口不注入时的调用数）。
mod common_admission;

use std::cell::RefCell;
use std::rc::Rc;

use common_admission::{checker_violations_on, content_of, start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount, RollbackTarget, ShadowLedger};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{resend_the_frozen_publish, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::segments::{FixedGeometry, SegmentAfterOperation, SegmentClosingRule, StepKind};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;

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
    successful_writes_before_fire: u64,
    successful_writes_after_fire: u64,
    successful_barriers_after_fire: u64,
    successful_writes_total: u64,
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
        state.successful_writes_total += 1;
        if state.fired.is_some() {
            state.successful_writes_after_fire += 1;
        } else {
            state.successful_writes_before_fire += 1;
        }
    }
    fn note_barrier(&self) {
        let mut state = self.0.borrow_mut();
        if state.fired.is_some() {
            state.successful_barriers_after_fire += 1;
        }
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
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
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
        self.inner.barrier()?;
        self.shared.note_barrier();
        Ok(())
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

struct Base {
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: MemoryPool,
    session: MountedSession,
    write_time_seconds: u64,
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

/// 起点：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）。只建一次，之后每段从它拷。
fn base() -> Base {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let image = pool.image();
    Base {
        parameters: pool.parameters.clone(),
        image,
        session: pool.session.take().expect("会话"),
        write_time_seconds: pool.write_time_seconds,
    }
}

fn devices_on(image: &MemoryPool, shared: &Shared) -> Vec<(DeviceIdentity, SweepDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            inner.image = sparse.clone();
            (*identity, SweepDevice { inner, device: identity.0, shared: shared.clone() })
        })
        .collect()
}


fn short(text: String) -> String {
    let cut: String = text.chars().take(160).collect();
    cut.replace('\n', " ")
}

const ENTRIES: [&str; 5] = ["session_overwrite", "unmount", "raise_floor_to_the_admission_ceiling", "rollback_to_1_4", "crash_remount"];

/// 跑一个入口：交回结局的短描述与这一次之后会话（挂载那一格是新会话）上有没有冻结着的发布。
fn run_entry<Device: BlockDevice>(entry: &str, base: &Base, devices: &mut Vec<(DeviceIdentity, Device)>) -> (String, Option<MountedSession>) {
    let mut session = clone_session(&base.session);
    match entry {
        "session_overwrite" => {
            let write_time_seconds = base.write_time_seconds + 1;
            let content = content_of(OVERWRITE_BYTES, write_time_seconds);
            let outcome = session.publish_user_change(devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds }));
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session))
        }
        "unmount" => {
            let outcome = unmount(&base.parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session))
        }
        "raise_floor_to_the_admission_ceiling" => {
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
            let outcome = raise_rollback_floor_to_the_admission_ceiling(&base.parameters, devices, &mut session.allocator, current, ShadowLedger::On);
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session))
        }
        "rollback_to_1_4" => {
            let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
            let outcome = roll_back_by_a_forward_publish(
                &base.parameters,
                devices,
                &mut session.allocator,
                current,
                RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) },
            );
            (match outcome { Ok(_) => "Ok".to_string(), Err(error) => short(format!("Err {error:?}")) }, Some(session))
        }
        "crash_remount" => match mount_writable(&base.parameters, devices) {
            Ok(mounted) => {
                let (_, session) = MountedSession::of_the_mount(mounted);
                ("Ok".to_string(), Some(session))
            }
            Err(error) => (short(format!("Err {error:?}")), None),
        },
        other => panic!("没有这个入口 {other}"),
    }
}

fn env_list(name: &str, default: &[&str]) -> Vec<String> {
    std::env::var(name)
        .map(|value| value.split(',').map(str::to_string).collect())
        .unwrap_or_else(|_| default.iter().map(|entry| entry.to_string()).collect())
}


fn recorded_devices_on(image: &MemoryPool, shared: &Shared, stream: &SharedStream) -> Vec<(DeviceIdentity, RecordingBlockDevice<SweepDevice>)> {
    devices_on(image, shared)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect()
}

/// 一条流按关段规则切开：每段里每块盘的系统配置槽写数的最大值、出现「一块盘一段里两次系统配置写」的段数，
/// 以及 FUA 写发出时同一块盘上同一段里还有没被屏障放行的普通写的次数。
fn same_segment_counts(operations: &[singlefs_harness::RecordedOperation], geometry: &FixedGeometry) -> (usize, usize, usize, Vec<String>) {
    let mut rule = SegmentClosingRule::default();
    let mut per_device: BTreeMap<u32, usize> = BTreeMap::new();
    let mut unreleased_plain: BTreeMap<u32, usize> = BTreeMap::new();
    let (mut max_per_device, mut segments_with_two, mut fua_after_unreleased_plain) = (0usize, 0usize, 0usize);
    let mut examples = Vec::new();
    for (index, operation) in operations.iter().enumerate() {
        match operation.kind {
            RecordedOperationKind::Write | RecordedOperationKind::WriteZeroes => {
                *unreleased_plain.entry(operation.device.0).or_insert(0) += 1;
                if geometry.classify(operation) == StepKind::SystemConfigurationSlot {
                    let count = per_device.entry(operation.device.0).or_insert(0);
                    *count += 1;
                    if *count == 2 {
                        segments_with_two += 1;
                        if examples.len() < 3 {
                            examples.push(format!("stream_index={index} device={} offset={}", operation.device.0, operation.offset.0));
                        }
                    }
                    max_per_device = max_per_device.max(*count);
                }
            }
            RecordedOperationKind::WriteForceUnitAccess => {
                if unreleased_plain.get(&operation.device.0).copied().unwrap_or(0) > 0 {
                    fua_after_unreleased_plain += 1;
                }
                unreleased_plain.insert(operation.device.0, 0);
            }
            RecordedOperationKind::Barrier => {
                unreleased_plain.insert(operation.device.0, 0);
            }
        }
        if rule.after(operation) == SegmentAfterOperation::Closes {
            per_device.clear();
        }
    }
    (max_per_device, segments_with_two, fua_after_unreleased_plain, examples)
}

#[test]
fn system_configuration_writes_in_one_segment_on_one_device_across_entries_faults_resends_and_remounts() {
    let started = std::time::Instant::now();
    let base = base();
    let geometry = HistoryDeviceWidth::FourGibibytes.fixed_geometry();
    let entries = env_list("OPUS_A2A_ENTRIES", &ENTRIES);
    let bytes = base.image.device_size_in_bytes;
    let (mut runs, mut runs_with_two, mut runs_with_fua) = (0u64, 0u64, 0u64);
    let mut worst = 0usize;
    for entry in &entries {
        let clean = Shared::default();
        let mut devices = devices_on(&base.image, &clean);
        let _ = run_entry(entry, &base, &mut devices);
        let (writes, barriers) = {
            let state = clean.0.borrow();
            (state.writes, state.barriers)
        };
        let mut plans: Vec<Option<(Kind, u64)>> = vec![None];
        plans.extend((1..=writes).map(|n| Some((Kind::Write, n))));
        plans.extend((1..=barriers).map(|n| Some((Kind::Barrier, n))));
        for plan in plans {
            runs += 1;
            let shared = Shared::default();
            shared.0.borrow_mut().target = plan;
            let stream = SharedStream::new();
            let mut devices = recorded_devices_on(&base.image, &shared, &stream);
            let (outcome, session) = run_entry(entry, &base, &mut devices);
            let mut resend = "-".to_string();
            if let Some(mut session) = session {
                if session.allocator.frozen_publish().is_some() {
                    let parameters = session.parameters_and_device_table.parameters_on_disk().clone();
                    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                    resend = match resend_the_frozen_publish(&mut writer, &mut session.allocator) {
                        Ok(_) => "Ok".to_string(),
                        Err(error) => short(format!("Err {error:?}")),
                    };
                }
            }
            let remount = match mount_writable(&base.parameters, &mut devices) {
                Ok(_) => "Ok".to_string(),
                Err(error) => short(format!("Err {error:?}")),
            };
            let (max_per_device, segments_with_two, fua_after_plain, examples) = same_segment_counts(&stream.operations(), &geometry);
            worst = worst.max(max_per_device);
            if segments_with_two > 0 {
                runs_with_two += 1;
            }
            if fua_after_plain > 0 {
                runs_with_fua += 1;
            }
            println!(
                "A2A entry={entry} plan={plan:?} outcome={outcome} resend={resend} remount={remount} operations={} max_sc_writes_per_device_in_a_segment={max_per_device} segments_with_two_sc_writes_on_one_device={segments_with_two} fua_after_unreleased_plain_write={fua_after_plain} examples={examples:?}",
                stream.operation_count()
            );
            let _ = bytes;
        }
    }
    println!(
        "A2A runs={runs} runs_with_two_sc_writes_on_one_device_in_a_segment={runs_with_two} runs_with_fua_after_unreleased_plain={runs_with_fua} worst_per_device={worst} seconds={:.1}",
        started.elapsed().as_secs_f64()
    );
}
