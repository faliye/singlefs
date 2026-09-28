//! 代码轮第二轮云端攻方腿（m2-closeout-code-r2）Y1、Y7：挂着之后的五个入口（会话覆盖写、正常卸载、抬 F、管理员回退、崩了再可写挂载）
//! 各跑一遍，逐个序号在「整池第 n 次读 / 写 / 屏障」上注入一次块设备错（单故障，只那一次），看：
//! ① 入口报了错时，报错之前发没发过写（「写了再拒」）；② 注入的那一次之后还发了几次写与屏障；
//! ③ 入口之后的盘面池级 checker 红不红；④ 撤掉故障之后原样重发冻结的那次（有的话）、再崩了再可写挂载，做不做得成、checker 红不红。
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

fn image_of(devices: &[(DeviceIdentity, SweepDevice)], bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.inner.image.clone())).collect(),
        device_size_in_bytes: bytes,
    }
}

fn short(text: String) -> String {
    let cut: String = text.chars().take(160).collect();
    cut.replace('\n', " ")
}

const ENTRIES: [&str; 5] = ["session_overwrite", "unmount", "raise_floor_to_the_admission_ceiling", "rollback_to_1_4", "crash_remount"];

/// 跑一个入口：交回结局的短描述与这一次之后会话（挂载那一格是新会话）上有没有冻结着的发布。
fn run_entry(entry: &str, base: &Base, devices: &mut Vec<(DeviceIdentity, SweepDevice)>) -> (String, Option<MountedSession>) {
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

#[test]
fn single_fault_ordinal_sweep_over_the_entries_after_a_writable_mount() {
    let started = std::time::Instant::now();
    let base = base();
    println!("SWEEP base_built seconds={:.2}", started.elapsed().as_secs_f64());
    let entries = env_list("OPUS_Y1_ENTRIES", &ENTRIES);
    let kinds = env_list("OPUS_Y1_KINDS", &["read", "write", "barrier"]);
    let stride: u64 = std::env::var("OPUS_Y1_STRIDE").ok().and_then(|value| value.parse().ok()).unwrap_or(1);
    let count_only = std::env::var("OPUS_Y1_COUNT_ONLY").is_ok();
    let read_head: u64 = std::env::var("OPUS_Y1_READ_HEAD").ok().and_then(|value| value.parse().ok()).unwrap_or(1000);
    let read_tail: u64 = std::env::var("OPUS_Y1_READ_TAIL").ok().and_then(|value| value.parse().ok()).unwrap_or(3000);
    let bytes = base.image.device_size_in_bytes;
    let mut flagged: Vec<String> = Vec::new();
    let mut runs = 0u64;
    for entry in &entries {
        let shared = Shared::default();
        let mut devices = devices_on(&base.image, &shared);
        let clean_started = std::time::Instant::now();
        let (outcome, _) = run_entry(entry, &base, &mut devices);
        let (reads, writes, barriers) = {
            let state = shared.0.borrow();
            (state.reads, state.writes, state.barriers)
        };
        println!(
            "SWEEP entry={entry} clean outcome={outcome} reads={reads} writes={writes} barriers={barriers} seconds={:.2}",
            clean_started.elapsed().as_secs_f64()
        );
        if count_only {
            continue;
        }
        for kind_name in &kinds {
            let (kind, total, step) = match kind_name.as_str() {
                "read" => (Kind::Read, reads, stride),
                "write" => (Kind::Write, writes, 1),
                "barrier" => (Kind::Barrier, barriers, 1),
                other => panic!("没有这种 {other}"),
            };
            // 读：前 OPUS_Y1_READ_HEAD 次与末 OPUS_Y1_READ_TAIL 次逐个，中间按步长（挂载扫 journal 环那几十万次读取样）；写与屏障逐个。
            let ordinals: Vec<u64> = (1..=total)
                .filter(|ordinal| kind != Kind::Read || step == 1 || *ordinal <= read_head || *ordinal + read_tail > total || ordinal % step == 0)
                .collect();
            for ordinal in ordinals {
                runs += 1;
                let shared = Shared::default();
                shared.0.borrow_mut().target = Some((kind, ordinal));
                let mut devices = devices_on(&base.image, &shared);
                let (outcome, session) = run_entry(entry, &base, &mut devices);
                let (fired, before, after_writes, after_barriers) = {
                    let state = shared.0.borrow();
                    (state.fired, state.successful_writes_before_fire, state.successful_writes_after_fire, state.successful_barriers_after_fire)
                };
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
                let mut remount_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
                    .devices
                    .iter()
                    .map(|(identity, sparse)| {
                        let mut device = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
                        device.image = sparse.clone();
                        (*identity, device)
                    })
                    .collect();
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
                    "SWEEP entry={entry} kind={kind_name} n={ordinal} fired={fired:?} outcome={outcome} writes_before={before} writes_after={after_writes} barriers_after={after_barriers} frozen={frozen} resend={resend} checker={} first={:?} remount={remount} checker_after_remount={} first_after={:?}",
                    violations.len(),
                    violations.first(),
                    violations_after_remount.len(),
                    violations_after_remount.first()
                );
                println!("{line}");
                let refused_after_writes = kind == Kind::Read && outcome.starts_with("Err") && before > 0;
                if !violations.is_empty() || remount != "Ok" || !violations_after_remount.is_empty() || refused_after_writes || after_writes > 0 {
                    flagged.push(line);
                }
            }
        }
    }
    println!("SWEEP runs={runs} flagged={} seconds={:.1}", flagged.len(), started.elapsed().as_secs_f64());
    for line in &flagged {
        println!("FLAG {line}");
    }
}
