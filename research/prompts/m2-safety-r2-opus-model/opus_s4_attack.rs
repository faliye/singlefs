//! m2-safety-r2 云端攻方腿（Opus）：在冻结副本的拷贝上实现自己的 A1–A4（`singlefs_core::admission::OpusArms`，只在草稿副本里），
//! 用自己的驱动（不走 history 执行器：它没有顺序写、截断、建 inode）跑这条腿的历史。只调 `crates/` 今天的公开入口。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, unused_variables)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::{
    opus_take_traces, set_opus_arms, A3Rule, OpusArms, OpusTrace, SpaceAdmission,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor, MountError, ShadowLedger,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_sequential_write, warm_up, FirstFile, PoolVersion, PoolWriter, PublishError,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::scenario::{first_file_content, FIXED_WRITE_TIME_SECONDS};

// ---------------------------------------------------------------- 设备：稀疏盘 + 可开关的写捕获

#[derive(Clone, Debug)]
enum Wr {
    Bytes { dev: u32, off: u64, bytes: Vec<u8> },
    Zeros { dev: u32, off: u64, len: u64 },
}

type Log = Rc<RefCell<Option<Vec<Wr>>>>;

struct CapDev {
    inner: SparseBlockDevice,
    id: u32,
    log: Log,
}

impl BlockDevice for CapDev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if let Some(log) = self.log.borrow_mut().as_mut() {
            log.push(Wr::Bytes { dev: self.id, off: offset.0, bytes: bytes.to_vec() });
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        if let Some(log) = self.log.borrow_mut().as_mut() {
            log.push(Wr::Zeros { dev: self.id, off: offset.0, len: length });
        }
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

// ---------------------------------------------------------------- 池与会话

#[derive(Clone)]
struct Sess {
    allocator: PoolAllocator,
    current: PoolVersion,
    instance: InstanceGeneration,
}

struct Pool {
    slots: u64,
    device_bytes: u64,
    params: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, CapDev)>,
    log: Log,
    session: Option<Sess>,
    time: u64,
}

fn params() -> MakeFilesystemParameters {
    HistoryDeviceWidth::UnitAreaOf384Slots.parameters()
}

fn devices_of(images: &[(DeviceIdentity, SparseDevice)], device_bytes: u64, log: &Log) -> Vec<(DeviceIdentity, CapDev)> {
    images
        .iter()
        .map(|(identity, image)| {
            let mut inner = SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512));
            inner.image = image.clone();
            (*identity, CapDev { inner, id: identity.0, log: log.clone() })
        })
        .collect()
}

/// 内容：长度、种子。
fn content(len: usize, seed: u64) -> Vec<u8> {
    (0..len).map(|i| ((i as u64 * 131 + seed * 7 + 1) % 251) as u8).collect()
}

fn cap() -> usize {
    data_unit_payload_capacity()
}

fn member_of_debug(text: String) -> String {
    let cut = text.find(|c| c == '(' || c == '{' || c == ' ').unwrap_or(text.len());
    text[..cut].to_string()
}

fn publish_member(error: &PublishError) -> String {
    let text = format!("{error:?}");
    let head = member_of_debug(text.clone());
    if head == "PlacementRefused" {
        // 带上是哪个角色被拒
        let unit = text.split("unit: ").nth(1).map(|t| member_of_debug(t.to_string())).unwrap_or_default();
        format!("PlacementRefused[{unit}]")
    } else {
        head
    }
}

fn mount_member(error: &MountError) -> String {
    let text = format!("{error:?}");
    let head = member_of_debug(text.clone());
    if head == "Publish" || head == "RaiseFloorSequencePublishFailed" {
        format!("{head}:{}", text.chars().take(160).collect::<String>())
    } else {
        head
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Op {
    /// 覆盖写一个数据单元以内的内容（字节数）。
    Ow(usize),
    /// 顺序写：整份文件换成 units 个满载数据单元（+ 余量字节）。
    Seq { units: usize, extra: usize },
    /// 截断成 0 字节（覆盖写空内容）。
    Empty,
    /// 建 n 个空 inode。
    Inodes(u64),
    /// 关掉会话、可写挂载。
    Remount,
    /// 抬 F 到上限。
    RaiseF,
    /// 关掉会话，回退：根环里从新到旧逐条试，第一条做成的就是它（用户选哪条放开扫）。
    RollbackAny,
}

impl Pool {
    fn start(slots: u64) -> Pool {
        let params = params();
        let device_bytes = (UNIT_AREA_START_SLOT + slots) * SLOT_BYTES;
        let log: Log = Rc::new(RefCell::new(None));
        let empty: Vec<(DeviceIdentity, SparseDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)].into_iter().map(|d| (d, SparseDevice::default())).collect();
        let mut devices = devices_of(&empty, device_bytes, &log);
        let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&params, &devices, &genesis);
        allocator.set_space_admission(SpaceAdmission::JudgedByTheFormula);
        let content = first_file_content();
        let (instance, output) = {
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let output = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
            (instance, output)
        };
        Pool {
            slots,
            device_bytes,
            params,
            devices,
            log,
            session: Some(Sess { allocator, current: PoolVersion::WithFile(output), instance }),
            time: FIXED_WRITE_TIME_SECONDS,
        }
    }

    fn images(&self) -> Vec<(DeviceIdentity, SparseDevice)> {
        self.devices.iter().map(|(d, dev)| (*d, dev.inner.image.clone())).collect()
    }

    fn clone_pool(&self) -> Pool {
        let log: Log = Rc::new(RefCell::new(None));
        Pool {
            slots: self.slots,
            device_bytes: self.device_bytes,
            params: self.params.clone(),
            devices: devices_of(&self.images(), self.device_bytes, &log),
            log,
            session: self.session.clone(),
            time: self.time,
        }
    }

    /// 崩溃后的池：镜像 + 没有会话。
    fn from_images(slots: u64, images: &[(DeviceIdentity, SparseDevice)]) -> Pool {
        let params = params();
        let device_bytes = (UNIT_AREA_START_SLOT + slots) * SLOT_BYTES;
        let log: Log = Rc::new(RefCell::new(None));
        Pool { slots, device_bytes, params, devices: devices_of(images, device_bytes, &log), log, session: None, time: FIXED_WRITE_TIME_SECONDS + 10_000 }
    }

    fn memory_pool(&self) -> MemoryPool {
        MemoryPool {
            devices: self.devices.iter().map(|(d, dev)| (*d, dev.inner.image.clone())).collect::<BTreeMap<_, _>>(),
            device_size_in_bytes: self.device_bytes,
        }
    }

    fn violations(&self) -> Vec<String> {
        check_pool_image(&self.memory_pool())
            .into_iter()
            .filter_map(|(inv, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some(format!("{inv}: {}", detail.chars().take(200).collect::<String>())),
                _ => None,
            })
            .collect()
    }

    /// 设备 0 上 (已分配, defer, 空闲, 隔离)。
    fn counts(&self) -> Option<(u64, u64, u64, u64)> {
        self.session.as_ref().map(|s| {
            let d = &s.allocator.devices[0];
            (d.allocated_slots(), d.deferred_slots(), d.free_slots(), d.isolated_slots())
        })
    }

    fn file_units(&self) -> Option<usize> {
        self.session.as_ref().and_then(|s| s.current.file_version()).map(|v| v.data_pointers.len())
    }

    fn apply(&mut self, op: &Op) -> String {
        self.time += 1;
        let time = self.time;
        match op {
            Op::Remount => {
                self.session = None;
                match mount_writable_with_space_admission(&self.params, &mut self.devices, SpaceAdmission::JudgedByTheFormula) {
                    Ok(mounted) => {
                        self.session = Some(Sess {
                            allocator: mounted.allocator,
                            current: mounted.current,
                            instance: mounted.output.instance,
                        });
                        "ok".into()
                    }
                    Err(error) => mount_member(&error),
                }
            }
            Op::RollbackAny => {
                self.session = None;
                let image = self.memory_pool();
                let Some(geometry) = singlefs_checker::image::chosen_system_configurations(&image)
                    .into_iter()
                    .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
                else {
                    return "rb-no-sc".into();
                };
                let mut roots: Vec<(u64, u32)> = singlefs_checker::image::valid_roots(&image, &geometry)
                    .into_iter()
                    .map(|(_, _, view)| (view.checkpoint_txg, view.instance))
                    .collect();
                roots.sort_unstable_by(|l, r| r.cmp(l));
                roots.dedup();
                let mut members: Vec<String> = Vec::new();
                for (index, (txg, instance)) in roots.iter().enumerate() {
                    let mut trial = self.clone_pool();
                    let result = singlefs_core::mount::mount_rollback_with_space_admission(
                        &trial.params,
                        &mut trial.devices,
                        singlefs_core::mount::RollbackTarget {
                            instance: InstanceGeneration(*instance),
                            checkpoint_txg: CheckpointTxg(*txg),
                        },
                        ShadowLedger::On,
                        SpaceAdmission::JudgedByTheFormula,
                    );
                    match result {
                        Ok(mounted) => {
                            trial.session = Some(Sess { allocator: mounted.allocator, current: mounted.current, instance: mounted.output.instance });
                            *self = trial;
                            return format!("rb-ok@{index}/{}", roots.len());
                        }
                        Err(error) => members.push(mount_member(&error)),
                    }
                }
                members.sort();
                members.dedup();
                format!("rb-none/{}{:?}", roots.len(), members)
            }
            Op::RaiseF => {
                let Some(session) = self.session.as_mut() else { return "no-session".into() };
                let PoolVersion::WithFile(current) = &mut session.current else { return "no-file".into() };
                let probe = raise_rollback_floor(&self.params, &mut self.devices, &mut session.allocator, current, CheckpointTxg(u64::MAX / 2), ShadowLedger::On);
                let ceiling = match probe {
                    Err(MountError::RollbackFloorAboveCeiling { ceiling, .. }) => ceiling,
                    Err(error) => return format!("probe:{}", mount_member(&error)),
                    Ok(_) => return "probe-raised?!".into(),
                };
                if ceiling <= current.root.rollback_floor {
                    return format!("F-at-ceiling({})", ceiling.0);
                }
                match raise_rollback_floor(&self.params, &mut self.devices, &mut session.allocator, current, ceiling, ShadowLedger::On) {
                    Ok(raised) => format!("ok(F={},reclaimed={})", ceiling.0, raised.reclaimed.len()),
                    Err(error) => mount_member(&error),
                }
            }
            _ => {
                let Some(session) = self.session.as_mut() else { return "no-session".into() };
                let PoolVersion::WithFile(previous) = &session.current else { return "no-file".into() };
                let previous = previous.clone();
                let mut writer = PoolWriter::new(&self.params, self.devices.as_mut_slice());
                let result = match op {
                    Op::Ow(len) => {
                        let bytes = content(*len, time);
                        publish_overwrite(&mut writer, &mut session.allocator, &previous, FirstFile { content: &bytes, write_time_seconds: time }, session.instance)
                    }
                    Op::Empty => publish_overwrite(&mut writer, &mut session.allocator, &previous, FirstFile { content: &[], write_time_seconds: time }, session.instance),
                    Op::Seq { units, extra } => {
                        let bytes = content(units * cap() + extra, time);
                        publish_sequential_write(&mut writer, &mut session.allocator, &previous, FirstFile { content: &bytes, write_time_seconds: time }, session.instance)
                    }
                    Op::Inodes(n) => publish_new_inodes(&mut writer, &mut session.allocator, &previous, *n, time, session.instance),
                    _ => unreachable!(),
                };
                match result {
                    Ok(output) => {
                        session.current = PoolVersion::WithFile(output);
                        "ok".into()
                    }
                    Err(error) => publish_member(&error),
                }
            }
        }
    }

    fn capture_start(&self) {
        *self.log.borrow_mut() = Some(Vec::new());
    }
    fn capture_take(&self) -> Vec<Wr> {
        self.log.borrow_mut().take().unwrap_or_default()
    }
}

fn apply_writes(images: &mut [(DeviceIdentity, SparseDevice)], writes: &[Wr]) {
    for w in writes {
        match w {
            Wr::Bytes { dev, off, bytes } => {
                let image = &mut images.iter_mut().find(|(d, _)| d.0 == *dev).unwrap().1;
                image.write(DeviceOffsetInBytes(*off), bytes);
            }
            Wr::Zeros { dev, off, len } => {
                let image = &mut images.iter_mut().find(|(d, _)| d.0 == *dev).unwrap().1;
                image.zero_fill(DeviceOffsetInBytes(*off), *len);
            }
        }
    }
}

// ---------------------------------------------------------------- 臂

fn arms_named(name: &str) -> OpusArms {
    let mut arms = OpusArms::default();
    for part in name.split('+') {
        match part {
            "T" => {}
            "A1" => arms.a1_count_own_release = true,
            "ND" => arms.drop_defer_term = true,
            "A2" => {
                arms.a1_count_own_release = true;
                arms.drop_defer_term = true;
            }
            "A3ge" => arms.a3 = Some(A3Rule::ReleasedAtLeastAllNew),
            "A3gt" => arms.a3 = Some(A3Rule::ReleasedMoreThanAllNew),
            "A3d" => arms.a3 = Some(A3Rule::ReleasedAtLeastDemand),
            "A4" => arms.a4_mount_own_writes_only = true,
            // 零轮：需求 = 这次新写的全部槽；A1x = 再加换下的槽（配今天的式子），A1n = 不加（配删掉 defer 那一项的式子）。
            "A1x" => {
                arms.charge_all_new_writes = true;
                arms.a1_count_own_release = true;
            }
            "A1n" => arms.charge_all_new_writes = true,
            other => panic!("unknown arm part {other}"),
        }
    }
    arms
}

fn threads() -> usize {
    std::env::var("OPUS_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8).min(8)
}

/// 在至多 8 个线程上跑一批任务，每个任务自己设臂；输出按任务次序交回。
fn par_map<T: Send, R: Send>(items: Vec<T>, f: impl Fn(T) -> R + Sync) -> Vec<R> {
    let n = threads().max(1);
    let items: Vec<(usize, T)> = items.into_iter().enumerate().collect();
    let queue = std::sync::Mutex::new(items);
    let results = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..n {
            scope.spawn(|| loop {
                let next = queue.lock().unwrap().pop();
                let Some((index, item)) = next else { break };
                let r = f(item);
                results.lock().unwrap().push((index, r));
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, r)| r).collect()
}

fn trace_brief(traces: &[OpusTrace]) -> String {
    traces
        .iter()
        .map(|t| format!("{}{}:av{:?}/d{}/r{}/n{}{}", t.path, if t.admitted { "+" } else { "-" }, t.available_slots, t.demand_slots, t.released_slots, t.all_new_slots, if t.exempted { "/EX" } else { "" }))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------- 公用：一段脚本跑完，交回每步的结局

fn run_script(pool: &mut Pool, ops: &[Op]) -> Vec<String> {
    ops.iter().map(|op| pool.apply(op)).collect()
}

const OW_BYTES: usize = 2999;

fn slot_sizes() -> Vec<u64> {
    std::env::var("OPUS_SLOTS")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or_else(|| vec![240, 256, 320, 384, 448, 512, 640, 768, 1024])
}

/// 放开扫的用户后缀：卡住之后用户手里的几步。每一种从同一个卡住的池拷一份起跑，交回 (后缀名, 各步结局)。
fn escape_suffixes() -> Vec<(&'static str, Vec<Op>)> {
    let ow = || Op::Ow(OW_BYTES);
    let mut v: Vec<(&'static str, Vec<Op>)> = Vec::new();
    v.push(("ow", vec![ow()]));
    v.push(("remount,ow", vec![Op::Remount, ow()]));
    v.push(("remount,ow,ow", vec![Op::Remount, ow(), ow()]));
    v.push(("remount x2,ow", vec![Op::Remount, Op::Remount, ow()]));
    v.push(("remount x4,ow", vec![Op::Remount, Op::Remount, Op::Remount, Op::Remount, ow()]));
    v.push(("remount x6,ow", std::iter::repeat_n(Op::Remount, 6).chain([ow()]).collect()));
    v.push(("remount x8,ow", std::iter::repeat_n(Op::Remount, 8).chain([ow()]).collect()));
    v.push(("raiseF,ow", vec![Op::RaiseF, ow()]));
    v.push(("empty,ow", vec![Op::Empty, ow()]));
    v.push(("empty,remount,ow", vec![Op::Empty, Op::Remount, ow()]));
    v.push(("empty,raiseF,ow", vec![Op::Empty, Op::RaiseF, ow()]));
    v.push(("remount,raiseF,ow", vec![Op::Remount, Op::RaiseF, ow()]));
    v
}

/// E1：先可写挂载一次，再连着覆盖写直到第一次被拒（至多 120 次）；卡住之后放开扫用户后缀；A1 一族另量「放过那一次之后池还好不好」。
#[test]
fn e1_overwrite_until_refused_then_sweep_user_suffixes() {
    let arm_names: Vec<String> = std::env::var("OPUS_ARMS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect())
        .unwrap_or_else(|| ["T", "A1", "A2", "ND", "A4", "A1+A4", "A2+A4"].iter().map(|s| s.to_string()).collect());
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for arm in &arm_names {
            tasks.push((slots, arm.clone()));
        }
    }
    let lines = par_map(tasks, |(slots, arm)| {
        let arms = arms_named(&arm);
        set_opus_arms(arms);
        let mut out = Vec::new();
        let mut pool = Pool::start(slots);
        let first_mount = pool.apply(&Op::Remount);
        let _ = opus_take_traces();
        let mut admitted = 0usize;
        let mut refusal = String::from("-");
        let mut refused_trace = String::new();
        let mut last_ok_trace = String::new();
        for _ in 0..120 {
            let before = pool.clone_pool();
            let r = pool.apply(&Op::Ow(OW_BYTES));
            let traces = opus_take_traces();
            if r == "ok" {
                admitted += 1;
                last_ok_trace = trace_brief(&traces);
            } else {
                refusal = r;
                refused_trace = trace_brief(&traces);
                // A1 一族：把这一次放过去（只关 A1 那一半，别的照旧），之后照原臂重挂、再写。
                if arms.a1_count_own_release {
                    let mut probe = before;
                    set_opus_arms(OpusArms { a1_count_own_release: false, ..arms });
                    let let_through = probe.apply(&Op::Ow(OW_BYTES));
                    set_opus_arms(arms);
                    let remount = probe.apply(&Op::Remount);
                    let next = probe.apply(&Op::Ow(OW_BYTES));
                    let _ = opus_take_traces();
                    out.push(format!(
                        "E1 slots={slots} arm={arm} A1-probe let_through={let_through} then remount={remount} then ow={next} violations={}",
                        probe.violations().len()
                    ));
                }
                break;
            }
        }
        let counts = pool.counts();
        out.insert(0, format!(
            "E1 slots={slots} arm={arm} first_mount={first_mount} admitted={admitted} refusal={refusal} counts(alloc,def,free,iso)={counts:?} last_ok=[{last_ok_trace}] refused=[{refused_trace}]"
        ));
        for (name, suffix) in escape_suffixes() {
            let mut p = pool.clone_pool();
            let rs = run_script(&mut p, &suffix);
            let traces = opus_take_traces();
            let v = p.violations();
            out.push(format!("E1 slots={slots} arm={arm} suffix=[{name}] -> {rs:?} violations={}{}", v.len(), if v.is_empty() { String::new() } else { format!(" first={}", v[0]) }));
        }
        out.join("\n")
    });
    for l in lines {
        println!("{l}");
    }
}

/// 结局序列按游程压缩：ok×5,SAR×3……
fn rle(outcomes: &[String]) -> String {
    let short = |s: &str| {
        s.replace("SpaceAdmissionRefusedBeforeAcquisition", "SARBA")
            .replace("SpaceAdmissionRefused", "SAR")
            .replace("PlacementRefusedBeforeAcquisitionMountAdmissionUndecided", "PRBA")
    };
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < outcomes.len() {
        let mut j = i;
        while j < outcomes.len() && outcomes[j] == outcomes[i] {
            j += 1;
        }
        out.push(if j - i > 1 { format!("{}x{}", short(&outcomes[i]), j - i) } else { short(&outcomes[i]) });
        i = j;
    }
    out.join(",")
}

fn exempt_count(traces: &[OpusTrace]) -> usize {
    traces.iter().filter(|t| t.exempted).count()
}

/// 这条腿的历史脚本（先可写挂载一次）。名字 → 操作序列。
fn e2_scripts() -> Vec<(&'static str, Vec<Op>)> {
    let ow = || Op::Ow(OW_BYTES);
    let mut v: Vec<(&'static str, Vec<Op>)> = Vec::new();
    // (a) 同样大小的覆盖写 150 次，被拒也接着试。
    v.push(("a-ow150", std::iter::repeat_n(ow(), 150).collect()));
    // (b) 伪装：先长到 4 个单元，之后 [缩成 3 个单元并全部重写, 长回 4 个] 交替 40 轮。
    let mut b = vec![Op::Seq { units: 2, extra: 0 }, Op::Seq { units: 3, extra: 0 }, Op::Seq { units: 4, extra: 0 }];
    for _ in 0..40 {
        b.push(Op::Seq { units: 3, extra: 0 });
        b.push(Op::Seq { units: 4, extra: 0 });
    }
    v.push(("b-shrink-rewrite-4-3", b));
    // (b2) 伪装的极端：一直「缩一字节不换单元数」——同样单元数的重写。
    let mut b2 = vec![Op::Seq { units: 3, extra: 100 }];
    for i in 0..80 {
        b2.push(Op::Seq { units: 3, extra: 99 - (i % 50) });
    }
    v.push(("b2-same-units-shrinking-bytes", b2));
    // (c) 近满连续删再写：长到 k 个单元（逐个长，被拒就停在那里），截断成 0，再写回同样大小；循环 10 轮。
    let mut c = Vec::new();
    for _ in 0..10 {
        for u in 1..=6 {
            c.push(Op::Seq { units: u, extra: 0 });
        }
        c.push(Op::Empty);
    }
    v.push(("c-grow6-truncate-x10", c));
    // (d) 空文件上反复「截断成 0」（换下的 = 新写的）。
    let mut d = vec![Op::Empty];
    d.extend(std::iter::repeat_n(Op::Empty, 60));
    v.push(("d-empty-on-empty-x60", d));
    // (e) 多文件：先建 400 个 inode（分几次），再覆盖写 60 次。
    let mut e = vec![Op::Inodes(100), Op::Inodes(100), Op::Inodes(100), Op::Inodes(100)];
    e.extend(std::iter::repeat_n(ow(), 60));
    v.push(("e-inodes400-then-ow60", e));
    // (f) 碎片化：单元数 5,1,4,2,3,1,5 反复，中间夹覆盖写。
    let mut f = Vec::new();
    for round in 0..8 {
        for u in [5usize, 1, 4, 2, 3, 1] {
            f.push(Op::Seq { units: u, extra: round * 37 });
        }
        f.push(Op::Ow(1 + round * 1000));
    }
    v.push(("f-fragment-5-1-4-2-3-1", f));
    v
}

#[test]
fn e2_histories_per_arm() {
    let arm_names: Vec<String> = std::env::var("OPUS_ARMS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect())
        .unwrap_or_else(|| {
            ["T", "A1", "A2", "A4", "A3ge", "A3gt", "A3d", "A2+A3ge", "A2+A3gt", "A2+A3d", "A2+A3gt+A4", "A2+A3ge+A4"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        });
    let scripts = e2_scripts();
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for arm in &arm_names {
            for (index, _) in scripts.iter().enumerate() {
                tasks.push((slots, arm.clone(), index));
            }
        }
    }
    let lines = par_map(tasks, |(slots, arm, index)| {
        let (name, ops) = &e2_scripts()[index];
        set_opus_arms(arms_named(&arm));
        let mut pool = Pool::start(slots);
        let first_mount = pool.apply(&Op::Remount);
        let _ = opus_take_traces();
        let outcomes = run_script(&mut pool, ops);
        let traces = opus_take_traces();
        let exempt = exempt_count(&traces);
        let counts = pool.counts();
        let units = pool.file_units();
        let violations_after_script = pool.violations();
        let mut tails = Vec::new();
        for (tail_name, tail_ops) in [
            ("remount,ow,remount,ow", vec![Op::Remount, Op::Ow(OW_BYTES), Op::Remount, Op::Ow(OW_BYTES)]),
            ("raiseF,ow", vec![Op::RaiseF, Op::Ow(OW_BYTES)]),
            ("rollbackAny,ow,remount,ow", vec![Op::RollbackAny, Op::Ow(OW_BYTES), Op::Remount, Op::Ow(OW_BYTES)]),
        ] {
            let mut p = pool.clone_pool();
            let tail: Vec<String> = tail_ops.iter().map(|op| p.apply(op)).collect();
            let _ = opus_take_traces();
            let v = p.violations();
            tails.push(format!("[{tail_name}]={} viol={}{}", rle(&tail), v.len(), if v.is_empty() { String::new() } else { format!(" first={}", v[0]) }));
        }
        format!(
            "E2 slots={slots} arm={arm} script={name} first_mount={first_mount} outcomes=[{}] exempted={exempt} counts={counts:?} file_units={units:?} viol_after_script={} then {}",
            rle(&outcomes),
            violations_after_script.len(),
            tails.join(" ")
        )
    });
    for l in lines {
        println!("{l}");
    }
}

fn default_arm_list() -> Vec<String> {
    std::env::var("OPUS_ARMS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect())
        .unwrap_or_else(|| {
            ["T", "A1", "A2", "ND", "A4", "A3ge", "A3gt", "A2+A3ge", "A2+A3gt", "A2+A4", "A2+A3gt+A4", "A2+A3ge+A4"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        })
}

/// E3：S4 形状的探测。沿每条脚本走，每一步用户发布做成之后，在拷贝上关掉重挂一次、再写一次同样大小的覆盖写：
/// 「发布放行 ∧ 下一次可写挂载被拒」记 S4；「挂载放行 ∧ 紧接着的覆盖写被拒」记 S4b（A4 那一问）。
#[test]
fn e3_s4_shape_after_every_admitted_publish() {
    let scripts = e2_scripts();
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for arm in default_arm_list() {
            for (index, _) in scripts.iter().enumerate() {
                tasks.push((slots, arm.clone(), index));
            }
        }
    }
    let lines = par_map(tasks, |(slots, arm, index)| {
        let (name, ops) = &e2_scripts()[index];
        set_opus_arms(arms_named(&arm));
        let mut pool = Pool::start(slots);
        let _ = pool.apply(&Op::Remount);
        let mut admitted = 0;
        let mut s4 = 0;
        let mut s4b = 0;
        let mut first_s4 = String::new();
        let mut first_s4b = String::new();
        for (step, op) in ops.iter().enumerate() {
            let r = pool.apply(op);
            if r != "ok" {
                continue;
            }
            admitted += 1;
            let mut probe = pool.clone_pool();
            let m = probe.apply(&Op::Remount);
            if m != "ok" {
                s4 += 1;
                if first_s4.is_empty() {
                    first_s4 = format!("step{step}:{op:?}->mount {m}");
                }
            } else {
                let w = probe.apply(&Op::Ow(OW_BYTES));
                if w != "ok" {
                    s4b += 1;
                    if first_s4b.is_empty() {
                        first_s4b = format!("step{step}:{op:?}->mount ok, ow {w}");
                    }
                }
            }
        }
        let _ = opus_take_traces();
        format!("E3 slots={slots} arm={arm} script={name} admitted={admitted} S4={s4} S4b={s4b} first_S4=[{first_s4}] first_S4b=[{first_s4b}]")
    });
    for l in lines {
        println!("{l}");
    }
}

/// 长到放行的最大单元数（逐个长），交回 N 与最后一次被拒的成员。
fn grow_until_refused(pool: &mut Pool, max_units: usize) -> (usize, String) {
    let mut last = 0;
    for u in 1..=max_units {
        let r = pool.apply(&Op::Seq { units: u, extra: 0 });
        if r != "ok" {
            return (last, r);
        }
        last = u;
    }
    (last, "-".into())
}

/// E4：D3（空间分配） 已定项 9 第 2 条「删掉 s 字节之后同样大小的写在有界步数内成功」。近满（逐个长到被拒）→ 截断成 0（删）→
/// 写回同样大小；不成就放开扫用户手里的几步：j 次改变用户可见状态的小发布（建 1 个 inode）、m 次重挂、抬 F，每一步之后都重试。
#[test]
fn e4_delete_then_write_same_size_within_bounded_steps() {
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for arm in default_arm_list() {
            for filler in ["inode", "remount", "raiseF", "inode+raiseF", "ow1+raiseF"] {
                tasks.push((slots, arm.clone(), filler));
            }
        }
    }
    let lines = par_map(tasks, |(slots, arm, filler)| {
        set_opus_arms(arms_named(&arm));
        let mut pool = Pool::start(slots);
        let _ = pool.apply(&Op::Remount);
        let (n, why) = grow_until_refused(&mut pool, 40);
        let delete = pool.apply(&Op::Empty);
        let mut steps: Vec<String> = Vec::new();
        let mut success_after: Option<usize> = None;
        let retry = Op::Seq { units: n, extra: 0 };
        let r0 = pool.apply(&retry);
        steps.push(r0.clone());
        if r0 == "ok" {
            success_after = Some(0);
        } else {
            for j in 1..=8 {
                let f = match filler {
                    "inode" => pool.apply(&Op::Inodes(1)),
                    "remount" => pool.apply(&Op::Remount),
                    "inode+raiseF" => {
                        let a = pool.apply(&Op::Inodes(1));
                        let b = pool.apply(&Op::RaiseF);
                        format!("{a}+{b}")
                    }
                    "ow1+raiseF" => {
                        let a = pool.apply(&Op::Ow(1));
                        let b = pool.apply(&Op::RaiseF);
                        format!("{a}+{b}")
                    }
                    _ => pool.apply(&Op::RaiseF),
                };
                let r = pool.apply(&retry);
                steps.push(format!("{f}/{r}"));
                if r == "ok" {
                    success_after = Some(j);
                    break;
                }
            }
        }
        let _ = opus_take_traces();
        let v = pool.violations();
        format!(
            "E4 slots={slots} arm={arm} filler={filler} N={n} grow_stop={why} delete={delete} success_after={success_after:?} steps=[{}] viol={}",
            steps.join(" "),
            v.len()
        )
    });
    for l in lines {
        println!("{l}");
    }
}

/// E5：崩在删除类发布中间。近满（逐个长到被拒），把截断成 0 那一次发布的写逐条录下，每个前缀（0..=全部）当一个崩溃点：
/// 拷发布之前的镜像、施加前缀、按同一臂可写挂载（含恢复），checker 判挂载之后的镜像（结束状态），再看写不写得回去
/// （重写 N 个单元；不成就 [抬 F, 重写]、[建 inode, 抬 F, 重写] 各至多 4 轮）。
#[test]
fn e5_crash_inside_a_delete_class_publish() {
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for arm in default_arm_list() {
            tasks.push((slots, arm.clone()));
        }
    }
    let lines = par_map(tasks, |(slots, arm)| {
        set_opus_arms(arms_named(&arm));
        let mut pool = Pool::start(slots);
        let _ = pool.apply(&Op::Remount);
        let (n, _) = grow_until_refused(&mut pool, 40);
        let before = pool.images();
        pool.capture_start();
        let delete = pool.apply(&Op::Empty);
        let writes = pool.capture_take();
        let _ = opus_take_traces();
        if delete != "ok" {
            return format!("E5 slots={slots} arm={arm} N={n} delete={delete} (删除被拒，没有崩溃点可枚举)");
        }
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut viol_points = 0usize;
        let mut first_viol = String::new();
        let mut first_stuck = String::new();
        for cut in 0..=writes.len() {
            let mut images = before.clone();
            apply_writes(&mut images, &writes[..cut]);
            let mut p = Pool::from_images(slots, &images);
            let pre_violations = p.violations();
            let m = p.apply(&Op::Remount);
            let post_violations = if m == "ok" { p.violations() } else { Vec::new() };
            let mut writable = String::from("-");
            if m == "ok" {
                let r = p.apply(&Op::Seq { units: n, extra: 0 });
                if r == "ok" {
                    writable = "now".into();
                } else {
                    let mut q = p.clone_pool();
                    for round in 1..=4 {
                        let _ = q.apply(&Op::RaiseF);
                        if q.apply(&Op::Seq { units: n, extra: 0 }) == "ok" {
                            writable = format!("raiseF@{round}");
                            break;
                        }
                    }
                    if writable == "-" {
                        let mut q = p.clone_pool();
                        for round in 1..=4 {
                            let _ = q.apply(&Op::Inodes(1));
                            let _ = q.apply(&Op::RaiseF);
                            if q.apply(&Op::Seq { units: n, extra: 0 }) == "ok" {
                                writable = format!("inode+raiseF@{round}");
                                break;
                            }
                        }
                    }
                    if writable == "-" {
                        writable = format!("never({r})");
                    }
                }
            } else {
                let mut q = p.clone_pool();
                let rb = q.apply(&Op::RollbackAny);
                writable = format!("mount-refused;rollback={rb}");
            }
            let _ = opus_take_traces();
            if !pre_violations.is_empty() || !post_violations.is_empty() {
                viol_points += 1;
                if first_viol.is_empty() {
                    first_viol = format!("cut={cut}: pre={:?} post={:?}", pre_violations.first(), post_violations.first());
                }
            }
            let key = format!("mount={m} writable={}", writable.split('(').next().unwrap_or(""));
            if (m != "ok" || writable.starts_with("never")) && first_stuck.is_empty() {
                first_stuck = format!("cut={cut}/{}: {m} {writable}", writes.len());
            }
            *tally.entry(key).or_default() += 1;
        }
        format!(
            "E5 slots={slots} arm={arm} N={n} delete=ok crash_points={} tally={tally:?} viol_points={viol_points} first_viol=[{first_viol}] first_stuck=[{first_stuck}]",
            writes.len() + 1
        )
    });
    for l in lines {
        println!("{l}");
    }
}

/// E6：A1 的账对不对。每次放行的覆盖写：A1 预言的「这次之后的可用」= 这次之前的可用 − 需求 − 这次换下的槽；
/// 实际 = 这次之后、同一个式子（同一臂）在需求 0 时读出的可用。差 = 预言 − 实际（正 = A1 少扣了，负 = A1 多扣了）。
/// 同一段历史按两种式子读：带 defer 那一项（今天）与删掉它（A2 的另一半）。
#[test]
fn e6_a1_prediction_against_the_next_reading() {
    let mut tasks = Vec::new();
    for slots in [384u64, 512, 768, 1024, 2048] {
        for arm in ["T", "ND"] {
            tasks.push((slots, arm));
        }
    }
    let lines = par_map(tasks, |(slots, arm)| {
        set_opus_arms(arms_named(arm));
        let mut pool = Pool::start(slots);
        let _ = pool.apply(&Op::Remount);
        let _ = opus_take_traces();
        let mut hist: BTreeMap<i128, usize> = BTreeMap::new();
        let mut detail = Vec::new();
        for step in 0..60 {
            let r = pool.apply(&Op::Ow(OW_BYTES));
            let traces = opus_take_traces();
            if r != "ok" {
                detail.push(format!("step{step}:{r}"));
                break;
            }
            let t = traces.last().expect("判过准入");
            let s = pool.session.as_ref().unwrap();
            let post = singlefs_core::admission::opus_admission_reading_before_a_publish(&s.allocator, s.current.file_version())
                .available_on_each_device()[0]
                .1
                 .0
                / i128::from(SLOT_BYTES);
            let predicted = t.available_slots[0] - i128::from(t.demand_slots) - i128::from(t.released_slots);
            let error = predicted - post;
            *hist.entry(error).or_default() += 1;
            if step < 3 || (step > 20 && step < 27) {
                detail.push(format!("step{step}:pre{} d{} r{} n{} post{} err{}", t.available_slots[0], t.demand_slots, t.released_slots, t.all_new_slots, post, error));
            }
        }
        format!("E6 slots={slots} formula={arm} error_histogram(slots->count)={hist:?} detail=[{}]", detail.join(" "))
    });
    for l in lines {
        println!("{l}");
    }
}

/// E7：逐步痕迹。OPUS_E7 = "盘宽:臂:脚本名:到第几步"，分号隔开几条。每步打结局、准入痕迹、设备 0 的计数，放行的那几步另在拷贝上重挂一次。
#[test]
fn e7_step_by_step_trace() {
    let spec = std::env::var("OPUS_E7").unwrap_or_else(|_| {
        "448:ND+A1n+A3gt:c-grow6-truncate-x10:21;448:ND+A1n+A3gt:b-shrink-rewrite-4-3:18;240:A3ge:a-ow150:18;240:A1:a-ow150:6;256:A2:a-ow150:12".to_string()
    });
    for item in spec.split(';') {
        let parts: Vec<&str> = item.split(':').collect();
        let slots: u64 = parts[0].parse().unwrap();
        let arm = parts[1];
        let name = parts[2];
        let upto: usize = parts[3].parse().unwrap();
        set_opus_arms(arms_named(arm));
        let (_, ops) = e2_scripts().into_iter().find(|(n, _)| *n == name).expect("脚本名");
        let mut pool = Pool::start(slots);
        let m0 = pool.apply(&Op::Remount);
        let t0 = trace_brief(&opus_take_traces());
        println!("E7 {slots} {arm} {name} mount0={m0} [{t0}] counts={:?}", pool.counts());
        for (step, op) in ops.iter().enumerate().take(upto + 1) {
            let r = pool.apply(op);
            let t = trace_brief(&opus_take_traces());
            let probe = if r == "ok" {
                let mut p = pool.clone_pool();
                let m = p.apply(&Op::Remount);
                let mt = trace_brief(&opus_take_traces());
                format!(" probe_mount={m} [{mt}]")
            } else {
                String::new()
            };
            println!("E7 {slots} {arm} {name} step{step} {op:?} -> {r} [{t}] counts(alloc,def,free,iso)={:?}{probe}", pool.counts());
        }
    }
}
