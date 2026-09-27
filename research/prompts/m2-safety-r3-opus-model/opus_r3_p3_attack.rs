//! m2-safety-r3 云端攻方腿（Opus）：在冻结副本的拷贝上实现 P1–P3（式子两处改动在 `singlefs_core::admission::OpusR3Arms`，
//! 只在草稿副本里；C283 在这个驱动里：准入报 `SpaceAdmissionRefused` 就按 D16（发布语义） 已定项 1 推空发布抬 F 到准入上限，
//! 推完重判，一次准入至多 8 次发布），攻 P3。回退走挂着时的 `roll_back_by_a_forward_publish`，卸载走 `unmount`。
//! 崩溃状态照层 0 的枚举域取（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开），每个状态恢复之后跑 checker，
//! 再按同一臂可写挂载、挂载之后的镜像再跑一次 checker。流全在本文件里自己造。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, unused_variables, unused_mut)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::{opus_r3_take_traces, set_opus_r3_arms, OpusR3Arms, OpusR3Trace, SpaceAdmission};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, MountError,
    RollbackError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite, publish_sequential_write, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions_observing_each_state, writes_and_segments, CrashImage,
    Layer0Tally, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice, SparseDevice,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::scenario::{first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation};

// ---------------------------------------------------------------- 臂

/// T、P1、P2、P3（正文第一节的式子表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Arm {
    name: &'static str,
    /// 可用(d) 去掉「− defer 待释放(d)」。
    nd: bool,
    /// 需求(d) = 这次发布重写的全部角色的 span_slots。
    a1n: bool,
    /// 判不过时先推空发布抬 F（C283）。
    c283: bool,
    /// 攻方腿自己的零轮变体：准入放行而落点取不到（`PlacementRefused`）时同样先推空发布抬 F 再重试（只在这条腿的模型上量过）。
    c283_on_placement: bool,
    /// 攻方腿自己的零轮变体：发布准入按下一次挂载的 rows0（+1）留链的份额（只在这条腿的模型上量过）。
    next_row: bool,
}

fn arm_named(name: &str) -> Arm {
    match name {
        "T" => Arm { name: "T", nd: false, a1n: false, c283: false, c283_on_placement: false, next_row: false },
        "P1" => Arm { name: "P1", nd: false, a1n: false, c283: true, c283_on_placement: false, next_row: false },
        "P2" => Arm { name: "P2", nd: true, a1n: true, c283: false, c283_on_placement: false, next_row: false },
        "P3" => Arm { name: "P3", nd: true, a1n: true, c283: true, c283_on_placement: false, next_row: false },
        "P1pl" => Arm { name: "P1pl", nd: false, a1n: false, c283: true, c283_on_placement: true, next_row: false },
        "P3pl" => Arm { name: "P3pl", nd: true, a1n: true, c283: true, c283_on_placement: true, next_row: false },
        "P3r" => Arm { name: "P3r", nd: true, a1n: true, c283: true, c283_on_placement: false, next_row: true },
        other => panic!("unknown arm {other}"),
    }
}

thread_local! {
    static ARM: Cell<Arm> = Cell::new(Arm { name: "T", nd: false, a1n: false, c283: false, c283_on_placement: false, next_row: false });
}

fn set_arm(arm: Arm) {
    ARM.with(|cell| cell.set(arm));
    set_opus_r3_arms(OpusR3Arms { drop_defer_term: arm.nd, charge_all_new_writes: arm.a1n, publish_reserves_next_mount_row: arm.next_row });
}
fn arm() -> Arm {
    ARM.with(Cell::get)
}

/// 一次准入至多几次发布（D16（发布语义） 已定项 1「准入」那一行：B = 4 + 2 k_tol，k_tol = 2）：算上被准入的那一次用户发布。
const PUBLISHES_PER_ADMISSION: usize = 8;

// ---------------------------------------------------------------- 设备：稀疏盘 + 可开关的录制（带内容，屏障与 FUA 照录）

type Log = Rc<RefCell<Option<Vec<RetainedOperation>>>>;

struct CapDev {
    inner: SparseBlockDevice,
    id: u32,
    log: Log,
}

fn push_op(log: &Log, device: u32, kind: RecordedOperationKind, offset: u64, length: u64, contents: Option<&[u8]>) {
    if let Some(ops) = log.borrow_mut().as_mut() {
        // 连续几道屏障之间没有写 ⇒ 记成一道（与 `SharedStream` 同一条规则）。
        if kind == RecordedOperationKind::Barrier
            && ops.last().is_some_and(|previous| previous.operation.kind == RecordedOperationKind::Barrier)
        {
            return;
        }
        ops.push(RetainedOperation {
            operation: RecordedOperation {
                device: DeviceIdentity(device),
                kind,
                offset: DeviceOffsetInBytes(offset),
                length,
                content_hash: contents.map_or(0, singlefs_harness::fnv1a_64),
            },
            contents: contents.map(<[u8]>::to_vec),
        });
    }
}

impl BlockDevice for CapDev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let kind = match durability {
            WriteDurability::Plain => RecordedOperationKind::Write,
            _ => RecordedOperationKind::WriteForceUnitAccess,
        };
        push_op(&self.log, self.id, kind, offset.0, bytes.len() as u64, Some(bytes));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        push_op(&self.log, self.id, RecordedOperationKind::WriteZeroes, offset.0, length, None);
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        push_op(&self.log, self.id, RecordedOperationKind::Barrier, 0, 0, None);
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

/// 驱动自己记的账（不读 core 的判法）：每条根是哪一个用户可见状态（状态号只在用户发布改了内容时加一；空发布、写行、暖机、
/// 抬 F、卸载的根照抄上一版的号；回退的根取目标的号）。
#[derive(Clone, Default)]
struct Book {
    state_of_root: BTreeMap<(u32, u64), u64>,
    content_of_state: BTreeMap<u64, Rc<Vec<u8>>>,
    next_state: u64,
    abandoned: BTreeSet<(u32, u64)>,
}

/// 一段历史里准入的累计（按用户发布数与准入内推的空发布数分开）。
#[derive(Clone, Copy, Debug, Default)]
struct Stats {
    user_published: usize,
    user_refused: usize,
    empties_in_admission: usize,
    raises: usize,
}

struct Pool {
    slots: u64,
    device_bytes: u64,
    params: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, CapDev)>,
    log: Log,
    session: Option<Sess>,
    time: u64,
    /// 现行那一版的状态号（会话里的）。
    state: u64,
    book: Book,
    stats: Stats,
}

fn params() -> MakeFilesystemParameters {
    HistoryDeviceWidth::UnitAreaOf384Slots.parameters()
}

fn geometry() -> FixedGeometry {
    let p = params();
    FixedGeometry {
        fixed_structure_slot_spacing: p.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: p.geometry.journal_ring_bytes,
        root_ring_slots_per_region: p.geometry.root_ring_slots_per_region,
    }
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

fn content(len: usize, seed: u64) -> Vec<u8> {
    (0..len).map(|i| ((i as u64 * 131 + seed * 7 + 1) % 251) as u8).collect()
}

fn cap() -> usize {
    data_unit_payload_capacity()
}

fn member_of_debug(text: &str) -> String {
    let cut = text.find(|c| c == '(' || c == '{' || c == ' ').unwrap_or(text.len());
    text[..cut].to_string()
}

fn publish_member(error: &PublishError) -> String {
    let text = format!("{error:?}");
    let head = member_of_debug(&text);
    if head == "PlacementRefused" {
        let unit = text.split("unit: ").nth(1).map(member_of_debug).unwrap_or_default();
        format!("PlacementRefused[{unit}]")
    } else {
        head
    }
}

fn mount_member(error: &MountError) -> String {
    let text = format!("{error:?}");
    let head = member_of_debug(&text);
    if head == "RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite" {
        let cause = text.split("cause: ").nth(1).map(member_of_debug).unwrap_or_default();
        let unit = text.split("unit: ").nth(1).map(member_of_debug).unwrap_or_default();
        format!("{head}[{cause}:{unit}]")
    } else if head == "Publish" || head == "RaiseFloorSequencePublishFailed" {
        format!("{head}:{}", text.chars().take(120).collect::<String>())
    } else {
        head
    }
}

fn rollback_member(error: &RollbackError) -> String {
    let text = format!("{error:?}");
    let head = member_of_debug(&text);
    if head == "TargetNotACandidate" {
        let why = text.split("exclusion: ").nth(1).map(member_of_debug).unwrap_or_default();
        format!("{head}[{why}]")
    } else if head == "Publish" {
        let cause = text.split("cause: ").nth(1).map(member_of_debug).unwrap_or_default();
        format!("{head}[{cause}]")
    } else {
        head
    }
}

/// 用户手里的动作。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Op {
    /// 覆盖写一个数据单元以内的内容（字节数）。
    Ow(usize),
    /// 顺序写：整份文件换成 units 个满载数据单元（+ 余量字节）。
    Seq { units: usize, extra: usize },
    /// 截断成 0 字节（删）。
    Empty,
    /// 建 n 个空 inode（改用户可见状态的小发布）。
    Inodes(u64),
    /// 正常卸载（`unmount`）再可写挂载。
    UMount,
    /// 崩了（会话丢掉、盘上停在最后一次发布之后）再可写挂载。
    CMount,
    /// 只正常卸载（之后没有会话）。
    UnmountOnly,
    /// 只可写挂载（没有会话时）。
    MountOnly,
    /// 挂着时回退到第 k 新的、与现行不同的状态（k 从 1 数；驱动按自己记的账挑，core 判它在不在候选集里）。
    Rollback(usize),
    /// 抬 F 到准入上限（C283 那一步单拎出来，测试入口）。
    RaiseF,
    /// 崩溃恢复抛弃最新那条根：最新根槽与它的数据单元暂时读不出，崩了再挂，写回。
    Abandon,
}

impl Op {
    fn is_user_publish(&self) -> bool {
        matches!(self, Op::Ow(_) | Op::Seq { .. } | Op::Empty | Op::Inodes(_))
    }
}

/// 一次用户发布的结局：结局字串、准入里推的空发布数、抬 F 次数。
#[derive(Clone, Debug)]
struct PubOutcome {
    result: String,
    empties: usize,
    raises: usize,
}

fn root_key(version: &PoolVersion) -> (u32, u64) {
    (version.root().instance.0, version.root().checkpoint_txg.0)
}
fn out_key(version: &TransactionOutput) -> (u32, u64) {
    (version.root.instance.0, version.root.checkpoint_txg.0)
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
        let first = first_file_content();
        let (instance, output) = {
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let output = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile { content: &first, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
            (instance, output)
        };
        let mut book = Book::default();
        book.next_state = 2;
        book.content_of_state.insert(1, Rc::new(first));
        book.state_of_root.insert(out_key(&output), 1);
        let mut pool = Pool {
            slots,
            device_bytes,
            params,
            devices,
            log,
            session: Some(Sess { allocator, current: PoolVersion::WithFile(output), instance }),
            time: FIXED_WRITE_TIME_SECONDS,
            state: 1,
            book,
            stats: Stats::default(),
        };
        let m = pool.apply(&Op::CMount);
        assert_eq!(m, "ok", "起点：mkfs、第一个文件之后崩了再挂");
        pool
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
            state: self.state,
            book: self.book.clone(),
            stats: self.stats,
        }
    }

    /// 崩溃状态的镜像 + 这一段历史的账（会话没有）。
    fn from_images(&self, images: &[(DeviceIdentity, SparseDevice)]) -> Pool {
        let log: Log = Rc::new(RefCell::new(None));
        Pool {
            slots: self.slots,
            device_bytes: self.device_bytes,
            params: self.params.clone(),
            devices: devices_of(images, self.device_bytes, &log),
            log,
            session: None,
            time: self.time + 10_000,
            state: self.state,
            book: self.book.clone(),
            stats: Stats::default(),
        }
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
                InvariantVerdict::Violated(detail) => Some(format!("{inv}: {}", detail.chars().take(160).collect::<String>())),
                _ => None,
            })
            .collect()
    }

    /// 会话里这一刻按这一臂的式子读的逐盘可用的最小值（槽），需求 0。
    fn available_now(&self) -> Option<i128> {
        let s = self.session.as_ref()?;
        let reading = singlefs_core::admission::admission_reading_before_a_publish(&s.allocator, s.current.file_version());
        reading.available_on_each_device().iter().map(|(_, a)| a.0.div_euclid(i128::from(SLOT_BYTES))).min()
    }

    /// 设备 0 上 (已分配, defer, 空闲, 隔离)。
    fn counts(&self) -> Option<(u64, u64, u64, u64)> {
        self.session.as_ref().map(|s| {
            let d = &s.allocator.devices[0];
            (d.allocated_slots(), d.deferred_slots(), d.free_slots(), d.isolated_slots())
        })
    }

    fn floor(&self) -> Option<u64> {
        self.session.as_ref().map(|s| s.current.root().rollback_floor.0)
    }

    fn current_txg(&self) -> Option<u64> {
        self.session.as_ref().map(|s| s.current.root().checkpoint_txg.0)
    }

    fn content_of_current(&self) -> Option<Rc<Vec<u8>>> {
        self.book.content_of_state.get(&self.state).cloned()
    }

    /// 一次发布出来的根记成哪一个状态。
    fn note_root(&mut self, key: (u32, u64), state: u64) {
        self.book.state_of_root.insert(key, state);
    }

    fn new_state(&mut self, content: Option<Vec<u8>>) -> u64 {
        let id = self.book.next_state;
        self.book.next_state += 1;
        if let Some(c) = content {
            self.book.content_of_state.insert(id, Rc::new(c));
        }
        id
    }

    fn raw_publish(&mut self, op: &Op) -> Result<(), PublishError> {
        self.time += 1;
        let time = self.time;
        let s = self.session.as_mut().expect("用户发布要有会话");
        let PoolVersion::WithFile(previous) = &s.current else { panic!("现行那一版带文件") };
        let previous = previous.clone();
        let mut writer = PoolWriter::new(&self.params, self.devices.as_mut_slice());
        let (result, new_content) = match op {
            Op::Ow(len) => {
                let bytes = content(*len, time);
                (publish_overwrite(&mut writer, &mut s.allocator, &previous, FirstFile { content: &bytes, write_time_seconds: time }, s.instance), Some(bytes))
            }
            Op::Empty => (publish_overwrite(&mut writer, &mut s.allocator, &previous, FirstFile { content: &[], write_time_seconds: time }, s.instance), Some(Vec::new())),
            Op::Seq { units, extra } => {
                let bytes = content(units * cap() + extra, time);
                (publish_sequential_write(&mut writer, &mut s.allocator, &previous, FirstFile { content: &bytes, write_time_seconds: time }, s.instance), Some(bytes))
            }
            Op::Inodes(n) => (publish_new_inodes(&mut writer, &mut s.allocator, &previous, *n, time, s.instance), None),
            _ => unreachable!(),
        };
        let output = result?;
        let key = out_key(&output);
        s.current = PoolVersion::WithFile(output);
        let content_after = match new_content {
            Some(c) => Some(c),
            None => self.content_of_current().map(|c| (*c).clone()),
        };
        let id = self.new_state(content_after);
        self.note_root(key, id);
        self.state = id;
        Ok(())
    }

    /// 抬 F 那一串要推几次（照 `mount::txgs_of_the_publishes_carrying_the_floor_to_every_device` 的规则：从现行 txg + 1 起逐次加一，
    /// 直到每块盘上都落过一条）。驱动在推之前按它判预算。
    fn planned_empties(&self) -> usize {
        let txg = self.current_txg().expect("会话");
        let s = self.params.geometry.root_ring_slots_per_region;
        let mut seen = BTreeSet::new();
        let mut n = 0;
        let mut t = txg;
        while seen.len() < 2 {
            t += 1;
            n += 1;
            let region = singlefs_core::root_ring::target_for_publish(CheckpointTxg(t), s).region;
            seen.insert(self.params.region_devices[region as usize]);
        }
        n
    }

    /// 准入上限（`raise_rollback_floor` 在任何写与分配器改动之前按上限拒，拒里带上限）。
    fn ceiling(&mut self) -> Result<u64, String> {
        let s = self.session.as_mut().ok_or("no-session")?;
        let PoolVersion::WithFile(current) = &mut s.current else { return Err("no-file".into()) };
        match raise_rollback_floor(&self.params, &mut self.devices, &mut s.allocator, current, CheckpointTxg(u64::MAX / 2), ShadowLedger::On) {
            Err(MountError::RollbackFloorAboveCeiling { ceiling, .. }) => Ok(ceiling.0),
            Err(error) => Err(format!("probe:{}", mount_member(&error))),
            Ok(_) => Err("probe-raised?!".into()),
        }
    }

    /// 抬 F 到 `to`：做成交回推了几次空发布。
    fn raise_to(&mut self, to: u64) -> Result<usize, String> {
        let state = self.state;
        let s = self.session.as_mut().ok_or("no-session")?;
        let PoolVersion::WithFile(current) = &mut s.current else { return Err("no-file".into()) };
        match raise_rollback_floor(&self.params, &mut self.devices, &mut s.allocator, current, CheckpointTxg(to), ShadowLedger::On) {
            Ok(raised) => {
                let keys: Vec<(u32, u64)> = raised.publishes.iter().map(out_key).collect();
                for k in &keys {
                    self.note_root(*k, state);
                }
                Ok(keys.len())
            }
            Err(error) => Err(mount_member(&error)),
        }
    }

    /// 用户发布，按臂带不带 C283：准入报拒就推空发布把 F 抬到准入上限、推完重判；一次准入至多 8 次发布（含这次用户发布）；
    /// 上限不高于现行 F（抬不动）、预算不够再推一串、推的那一串自己报错，就报拒。
    fn user_publish(&mut self, op: &Op) -> PubOutcome {
        let mut empties = 0;
        let mut raises = 0;
        loop {
            match self.raw_publish(op) {
                Ok(()) => {
                    self.stats.user_published += 1;
                    self.stats.empties_in_admission += empties;
                    self.stats.raises += raises;
                    return PubOutcome { result: "ok".into(), empties, raises };
                }
                Err(PublishError::SpaceAdmissionRefused(_)) if arm().c283 => {
                    if let Some(o) = self.c283_step(&mut empties, &mut raises, "SpaceAdmissionRefused") {
                        return o;
                    }
                }
                Err(PublishError::PlacementRefused { .. }) if arm().c283_on_placement => {
                    if let Some(o) = self.c283_step(&mut empties, &mut raises, "PlacementRefused") {
                        return o;
                    }
                }
                Err(error) => {
                    self.stats.user_refused += 1;
                    self.stats.empties_in_admission += empties;
                    self.stats.raises += raises;
                    return PubOutcome { result: publish_member(&error), empties, raises };
                }
            }
        }
    }

    /// C283 的一步：预算够、上限高于现行 F 就推一串抬 F（交回 None 让调用方重判），否则交回报拒的结局。
    fn c283_step(&mut self, empties: &mut usize, raises: &mut usize, refused_by: &str) -> Option<PubOutcome> {
        {
            {
                {
                    let planned = self.planned_empties();
                    let e = *empties;
                    let refuse = |why: String| PubOutcome { result: format!("{refused_by};{why}"), empties: e, raises: *raises };
                    let outcome = if e + planned + 1 > PUBLISHES_PER_ADMISSION {
                        Some(refuse(format!("budget(e{e}+p{planned})")))
                    } else {
                        match self.ceiling() {
                            Err(err) => Some(refuse(err)),
                            Ok(ceiling) if ceiling <= self.floor().unwrap_or(0) => Some(refuse(format!("F-at-ceiling(e{e})"))),
                            Ok(ceiling) => match self.raise_to(ceiling) {
                                Ok(n) => {
                                    *empties += n;
                                    *raises += 1;
                                    None
                                }
                                Err(err) => Some(refuse(format!("raise:{err}(e{e})"))),
                            },
                        }
                    };
                    if let Some(o) = &outcome {
                        self.stats.user_refused += 1;
                        self.stats.empties_in_admission += o.empties;
                        self.stats.raises += o.raises;
                    }
                    outcome
                }
            }
        }
    }
}

impl Pool {
    fn mount(&mut self) -> String {
        assert!(self.session.is_none(), "挂载之前没有会话");
        match mount_writable_with_space_admission(&self.params, &mut self.devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => {
                let effective = (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0);
                let state = match self.book.state_of_root.get(&effective) {
                    Some(s) => *s,
                    None => self.new_state(None),
                };
                self.state = state;
                let row = root_key(&mounted.output.row_publish);
                self.note_root(row, state);
                for w in &mounted.output.warm_up_publishes {
                    let k = root_key(w);
                    self.note_root(k, state);
                }
                self.session = Some(Sess { allocator: mounted.allocator, current: mounted.current, instance: mounted.output.instance });
                "ok".into()
            }
            Err(error) => mount_member(&error),
        }
    }

    fn unmount_only(&mut self) -> String {
        let Some(mut s) = self.session.take() else { return "no-session".into() };
        let state = self.state;
        match unmount(&self.params, &mut self.devices, &mut s.allocator, &mut s.current, ShadowLedger::On) {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(u)) => {
                for p in &u.publishes {
                    self.note_root(out_key(p), state);
                }
                "ok".into()
            }
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "ok-nothing".into(),
            Err(error) => format!("unmount:{}", mount_member(&error)),
        }
    }

    /// 根环里读得出的根（txg, 实例, 根上的 F）。
    fn ring_roots(&self) -> Vec<(u64, u32, u64)> {
        let image = self.memory_pool();
        let Some(geometry) = singlefs_checker::image::chosen_system_configurations(&image)
            .into_iter()
            .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        else {
            return Vec::new();
        };
        let mut roots: Vec<(u64, u32, u64)> = singlefs_checker::image::valid_roots(&image, &geometry)
            .into_iter()
            .map(|(_, _, view)| (view.checkpoint_txg, view.instance, view.rollback_floor))
            .collect();
        roots.sort_unstable_by(|l, r| r.cmp(l));
        roots.dedup();
        roots
    }

    /// 按驱动自己的账：根环里（没被抛弃、状态已知、带文件）按状态去重，从新到旧，每个状态取它最新的那条根。
    /// 交回 (状态号, 那条根的 txg, 实例)。第一个是现行状态。
    fn distinct_states_in_ring(&self) -> Vec<(u64, u64, u32)> {
        let mut out: Vec<(u64, u64, u32)> = Vec::new();
        for (txg, instance, _) in self.ring_roots() {
            if self.book.abandoned.contains(&(instance, txg)) {
                continue;
            }
            let Some(state) = self.book.state_of_root.get(&(instance, txg)) else { continue };
            if !self.book.content_of_state.contains_key(state) && *state != self.state {
                // 状态号认得、内容不知道的（挂载落到账外的根）照样算一个状态
            }
            if out.iter().any(|(s, _, _)| s == state) {
                continue;
            }
            out.push((*state, txg, instance));
        }
        out
    }

    fn rollback(&mut self, k: usize) -> String {
        if self.session.is_none() {
            return "no-session".into();
        }
        let others: Vec<(u64, u64, u32)> = self.distinct_states_in_ring().into_iter().filter(|(s, _, _)| *s != self.state).collect();
        let Some((state, txg, instance)) = others.get(k - 1).copied() else { return format!("rb-no-{k}th-state/{}", others.len()) };
        self.rollback_to(state, txg, instance)
    }

    fn rollback_to(&mut self, state: u64, txg: u64, instance: u32) -> String {
        let s = self.session.as_mut().expect("会话");
        let PoolVersion::WithFile(current) = &mut s.current else { return "no-file".into() };
        let target = RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) };
        match roll_back_by_a_forward_publish(&self.params, &mut self.devices, &mut s.allocator, current, target) {
            Ok(_) => {
                let key = out_key(current);
                self.note_root(key, state);
                self.state = state;
                "ok".into()
            }
            Err(error) => rollback_member(&error),
        }
    }

    /// 崩溃恢复抛弃最新那条根（`tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 同一形：
    /// 最新根槽与它引用的数据单元暂时读不出，崩了再挂，择根落到前一条，再原样写回）。要在一次改了数据单元的用户发布之后调。
    fn abandon(&mut self) -> String {
        let Some(s) = self.session.as_ref() else { return "no-session".into() };
        let PoolVersion::WithFile(newest) = &s.current else { return "no-file".into() };
        let newest = newest.clone();
        let p = &self.params;
        let target = singlefs_core::root_ring::target_for_publish(newest.root.checkpoint_txg, p.geometry.root_ring_slots_per_region);
        let root_device = p.region_devices[target.region as usize];
        let root_offset = singlefs_core::root_ring::slot_offset(target, p.geometry.fixed_structure_slot_spacing);
        let root_bytes = p.geometry.physical_block_size as usize;
        let data_bytes = singlefs_format::DATA_UNIT_BYTES as usize;
        let places: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = std::iter::once((root_device, root_offset, root_bytes))
            .chain(newest.data_pointers.iter().flat_map(|ptr| ptr.locations).map(|l| (l.device, l.slot.to_device_offset(), data_bytes)))
            .collect();
        let mut saved = Vec::new();
        for (d, off, len) in &places {
            let dev = &mut self.devices.iter_mut().find(|(i, _)| i == d).unwrap().1;
            let bytes = dev.inner.image.read(*off, *len);
            dev.inner.image.write(*off, &vec![0u8; *len]);
            saved.push((*d, *off, bytes));
        }
        self.session = None;
        let old_instance = newest.root.instance.0;
        let m = self.mount();
        // 原样写回，只写回挂载期间没被写过的那几处（还是全 0）：挂载把那一槽合法地复用了的，写回旧字节等于设备吞掉一次写，不是合法历史。
        let mut restored = 0usize;
        let mut reused = 0usize;
        for (d, off, bytes) in &saved {
            let dev = &mut self.devices.iter_mut().find(|(i, _)| i == d).unwrap().1;
            if dev.inner.image.read(*off, bytes.len()).iter().all(|b| *b == 0) {
                dev.inner.image.write(*off, bytes);
                restored += 1;
            } else {
                reused += 1;
            }
        }
        if m != "ok" {
            return format!("abandon-mount:{m}");
        }
        // 被抛弃的：旧实例里 txg 高于择到的根的那几条（按驱动的账记）。
        let chosen = self.session.as_ref().map(|s| s.current.root().checkpoint_txg.0).unwrap_or(0);
        let keys: Vec<(u32, u64)> = self.book.state_of_root.keys().copied().filter(|(i, t)| *i == old_instance && *t == newest.root.checkpoint_txg.0).collect();
        for k in keys {
            self.book.abandoned.insert(k);
        }
        if reused > 0 { format!("ok(reused {reused}/{})", restored + reused) } else { "ok".into() }
    }

    fn apply(&mut self, op: &Op) -> String {
        match op {
            op if op.is_user_publish() => {
                if self.session.is_none() {
                    return "no-session".into();
                }
                self.user_publish(op).result
            }
            Op::UMount => {
                let u = self.unmount_only();
                if !u.starts_with("ok") {
                    // 卸载那一串没做成：照样丢掉会话再挂（进程退出）。
                    let m = self.mount();
                    return format!("{u};mount={m}");
                }
                self.mount()
            }
            Op::CMount => {
                self.session = None;
                self.mount()
            }
            Op::UnmountOnly => self.unmount_only(),
            Op::MountOnly => {
                if self.session.is_some() {
                    return "already".into();
                }
                self.mount()
            }
            Op::Rollback(k) => self.rollback(*k),
            Op::RaiseF => {
                if self.session.is_none() {
                    return "no-session".into();
                }
                match self.ceiling() {
                    Err(e) => e,
                    Ok(c) if c <= self.floor().unwrap_or(0) => format!("F-at-ceiling({c})"),
                    Ok(c) => match self.raise_to(c) {
                        Ok(n) => format!("ok(F={c},n={n})"),
                        Err(e) => e,
                    },
                }
            }
            Op::Abandon => self.abandon(),
            _ => unreachable!(),
        }
    }

    fn capture_start(&self) {
        *self.log.borrow_mut() = Some(Vec::new());
    }
    fn capture_take(&self) -> Vec<RetainedOperation> {
        self.log.borrow_mut().take().unwrap_or_default()
    }
}

fn short(s: &str) -> String {
    s.replace("SpaceAdmissionRefusedBeforeAcquisition", "SARBA")
        .replace("SpaceAdmissionRefused", "SAR")
        .replace("PlacementRefusedBeforeAcquisitionMountAdmissionUndecided", "PRBA")
        .replace("RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite", "RFSR")
}

fn rle(outcomes: &[String]) -> String {
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

fn threads() -> usize {
    std::env::var("OPUS_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(6).min(6)
}

/// 在至多 6 个线程上跑一批任务；输出按任务次序交回。
fn par_map<T: Send, R: Send>(items: Vec<T>, f: impl Fn(T) -> R + Sync) -> Vec<R> {
    let n = threads().max(1);
    let mut items: Vec<(usize, T)> = items.into_iter().enumerate().collect();
    items.reverse();
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

fn env_list(name: &str, default: &[&str]) -> Vec<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect())
        .unwrap_or_else(|| default.iter().map(|s| s.to_string()).collect())
}

fn slot_sizes() -> Vec<u64> {
    env_list("OPUS_SLOTS", &["240", "256", "320", "384", "448", "512", "640", "768", "1024"]).iter().map(|x| x.parse().unwrap()).collect()
}

const OW_BYTES: usize = 2999;

/// 一个 smoke 用例：起点、每条臂一次覆盖写、两支重挂。
#[test]
fn e0_smoke() {
    for name in ["T", "P1", "P2", "P3"] {
        set_arm(arm_named(name));
        let mut pool = Pool::start(384);
        let r = pool.apply(&Op::Ow(OW_BYTES));
        let u = pool.clone_pool().apply(&Op::UMount);
        let c = pool.clone_pool().apply(&Op::CMount);
        let rb = pool.apply(&Op::Rollback(1));
        let _ = opus_r3_take_traces();
        println!("E0 arm={name} ow={r} umount={u} cmount={c} rollback={rb} avail={:?} F={:?} states={:?} viol={}", pool.available_now(), pool.floor(), pool.distinct_states_in_ring(), pool.violations().len());
    }
}

// ---------------------------------------------------------------- 历史脚本（先可写挂载一次：起点已经挂着）

fn scripts() -> Vec<(&'static str, Vec<Op>)> {
    let ow = || Op::Ow(OW_BYTES);
    let mut v: Vec<(&'static str, Vec<Op>)> = Vec::new();
    v.push(("a-ow150", std::iter::repeat_n(ow(), 150).collect()));
    let mut b = vec![Op::Seq { units: 2, extra: 0 }, Op::Seq { units: 3, extra: 0 }, Op::Seq { units: 4, extra: 0 }];
    for _ in 0..40 {
        b.push(Op::Seq { units: 3, extra: 0 });
        b.push(Op::Seq { units: 4, extra: 0 });
    }
    v.push(("b-shrink-rewrite-4-3", b));
    let mut c = Vec::new();
    for _ in 0..10 {
        for u in 1..=6 {
            c.push(Op::Seq { units: u, extra: 0 });
        }
        c.push(Op::Empty);
    }
    v.push(("c-grow6-truncate-x10", c));
    let mut d = vec![Op::Empty];
    d.extend(std::iter::repeat_n(Op::Empty, 60));
    v.push(("d-empty-on-empty-x60", d));
    let mut e = vec![Op::Inodes(100), Op::Inodes(100), Op::Inodes(100), Op::Inodes(100)];
    e.extend(std::iter::repeat_n(ow(), 60));
    v.push(("e-inodes400-then-ow60", e));
    let mut f = Vec::new();
    for round in 0..8 {
        for u in [5usize, 1, 4, 2, 3, 1] {
            f.push(Op::Seq { units: u, extra: round * 37 });
        }
        f.push(Op::Ow(1 + round * 1000));
    }
    v.push(("f-fragment-5-1-4-2-3-1", f));
    // g：长到 8 个单元之后，每 5 次覆盖写正常卸载一次（B1 那一支混进会话里）。
    let mut g = vec![Op::Seq { units: 8, extra: 0 }];
    for i in 0..60 {
        g.push(ow());
        if i % 5 == 4 {
            g.push(Op::UMount);
        }
    }
    v.push(("g-seq8-ow60-umount-every5", g));
    // h：每 3 次覆盖写回退一次（挂着时回退混进会话里）。
    let mut h = vec![Op::Seq { units: 6, extra: 0 }];
    for i in 0..60 {
        h.push(ow());
        if i % 3 == 2 {
            h.push(Op::Rollback(1));
        }
    }
    v.push(("h-seq6-ow60-rollback-every3", h));
    v
}

/// E1：S4 / S4b，两支。沿每条脚本走，每一步用户发布做成之后，在拷贝上 (u) 正常卸载再挂、(c) 崩了再挂，各再写一次同样大小的覆盖写（按臂带不带 C283）：
/// 「发布放行 ∧ 下一次可写挂载被拒」记 S4，「挂载放行 ∧ 紧接着的覆盖写被拒」记 S4b。另报这条脚本上准入推的空发布累计、抬 F 次数。
#[test]
fn e1_s4_two_branches() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P2", "P3"]);
    let names: Vec<String> = env_list("OPUS_SCRIPTS", &[]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for (index, (name, _)) in scripts().iter().enumerate() {
                if names.is_empty() || names.iter().any(|n| n == name) {
                    tasks.push((slots, a.clone(), index));
                }
            }
        }
    }
    let lines = par_map(tasks, |(slots, a, index)| {
        let (name, ops) = &scripts()[index];
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        let mut s4 = [0usize; 2];
        let mut s4b = [0usize; 2];
        let mut first = [String::new(), String::new(), String::new(), String::new()];
        let mut outcomes = Vec::new();
        let mut probe_ow_empties = 0usize;
        for (step, op) in ops.iter().enumerate() {
            let r = pool.apply(op);
            outcomes.push(r.clone());
            if !(op.is_user_publish() && r == "ok") {
                continue;
            }
            for (b, branch) in [Op::UMount, Op::CMount].iter().enumerate() {
                let mut probe = pool.clone_pool();
                let m = probe.apply(branch);
                if m != "ok" {
                    s4[b] += 1;
                    if first[b].is_empty() {
                        first[b] = format!("step{step}:{op:?}->{}", short(&m));
                    }
                } else {
                    let w = probe.user_publish(&Op::Ow(OW_BYTES));
                    probe_ow_empties += w.empties;
                    if w.result != "ok" {
                        s4b[b] += 1;
                        if first[2 + b].is_empty() {
                            first[2 + b] = format!("step{step}:{op:?}->mount ok, ow {}", short(&w.result));
                        }
                    }
                }
            }
        }
        let _ = opus_r3_take_traces();
        let st = pool.stats;
        format!(
            "E1 slots={slots} arm={a} script={name} user_ok={} user_refused={} empties_in_admission={} raises={} S4u={} S4c={} S4bu={} S4bc={} probe_ow_empties={probe_ow_empties} first_S4u=[{}] first_S4c=[{}] first_S4bu=[{}] first_S4bc=[{}] outcomes=[{}]",
            st.user_published, st.user_refused, st.empties_in_admission, st.raises, s4[0], s4[1], s4b[0], s4b[1], first[0], first[1], first[2], first[3], rle(&outcomes)
        )
    });
    for l in lines {
        println!("{l}");
    }
}

fn trace_brief(traces: &[OpusR3Trace]) -> String {
    traces
        .iter()
        .map(|t| format!("{}{}:av{:?}/d{}/n{}/r{}/def{:?}/iso{:?}/mc{}/ck{}/rows{}", t.site, if t.admitted { "+" } else { "-" }, t.available_slots, t.demand_today_slots, t.all_new_slots, t.released_slots, t.deferred_slots, t.isolated_slots, t.mount_commitment_slots, t.checkpoint_share_slots, t.rows0))
        .collect::<Vec<_>>()
        .join(" ")
}

/// E9：逐步痕迹。OPUS_E9 = "盘宽:臂:脚本名:从第几步:到第几步"，分号隔开几条。
#[test]
fn e9_step_by_step_trace() {
    let spec = std::env::var("OPUS_E9").unwrap_or_else(|_| "384:P3:a-ow150:0:55".to_string());
    for item in spec.split(';') {
        let parts: Vec<&str> = item.split(':').collect();
        let slots: u64 = parts[0].parse().unwrap();
        let a = parts[1];
        let name = parts[2];
        let from: usize = parts[3].parse().unwrap();
        let upto: usize = parts[4].parse().unwrap();
        set_arm(arm_named(a));
        let (_, ops) = scripts().into_iter().find(|(n, _)| *n == name).expect("脚本名");
        let mut pool = Pool::start(slots);
        let _ = opus_r3_take_traces();
        for (step, op) in ops.iter().enumerate().take(upto + 1) {
            let before_empties = pool.stats.empties_in_admission;
            let r = pool.apply(op);
            let t = trace_brief(&opus_r3_take_traces());
            if step >= from {
                let s = pool.session.as_ref();
                let free_pairs = s.map(|s| {
                    let d = &s.allocator.devices[0];
                    format!("free={} alloc={} def={} iso={}", d.free_slots(), d.allocated_slots(), d.deferred_slots(), d.isolated_slots())
                });
                println!(
                    "E9 {slots} {a} {name} step{step} {op:?} -> {} empties+{} F={:?} txg={:?} avail_now={:?} {free_pairs:?} [{t}]",
                    short(&r),
                    pool.stats.empties_in_admission - before_empties,
                    pool.floor(),
                    pool.current_txg(),
                    pool.available_now()
                );
            }
        }
    }
}

// ---------------------------------------------------------------- E2：吸收态与放开扫的逃生序列

/// 用户手里的几步（放开扫：长度 ≤ 3 的全部序列，之后再写一次同样大小的覆盖写）。
fn escape_alphabet() -> Vec<Op> {
    vec![Op::UMount, Op::CMount, Op::Empty, Op::Inodes(1), Op::Rollback(1), Op::Rollback(2)]
}

/// 一个格的判定：重挂（两支）、写、删、回退（第 1–4 新的别的状态）各做不做得成；再放开扫长度 ≤ 3 的用户序列，数有几条之后写得进去。
fn absorbing_cell(pool: &Pool) -> String {
    let mut p = pool.clone_pool();
    let mu = p.apply(&Op::UMount);
    let mut p = pool.clone_pool();
    let mc = p.apply(&Op::CMount);
    let mut p = pool.clone_pool();
    let w = if p.session.is_some() { p.user_publish(&Op::Ow(OW_BYTES)).result } else { "no-session".into() };
    let mut p = pool.clone_pool();
    let d = if p.session.is_some() { p.user_publish(&Op::Empty).result } else { "no-session".into() };
    let mut rb = Vec::new();
    for k in 1..=4 {
        let mut p = pool.clone_pool();
        rb.push(p.apply(&Op::Rollback(k)));
    }
    let any_rb = rb.iter().any(|r| r == "ok");
    let absorbing = mu != "ok" && mc != "ok" && !w.starts_with("ok") && !d.starts_with("ok") && !any_rb;
    let alphabet = escape_alphabet();
    let mut seqs: Vec<Vec<Op>> = vec![Vec::new()];
    let mut all: Vec<Vec<Op>> = vec![Vec::new()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for s in &seqs {
            for a in &alphabet {
                let mut t = s.clone();
                t.push(a.clone());
                next.push(t);
            }
        }
        all.extend(next.iter().cloned());
        seqs = next;
    }
    let mut escapes = 0usize;
    let mut shortest: Option<String> = None;
    for s in &all {
        let mut p = pool.clone_pool();
        for op in s {
            let _ = p.apply(op);
        }
        let ok = p.session.is_some() && p.user_publish(&Op::Ow(OW_BYTES)).result == "ok";
        if ok {
            escapes += 1;
            if shortest.is_none() {
                shortest = Some(format!("{s:?}"));
            }
        }
    }
    let _ = opus_r3_take_traces();
    format!(
        "absorbing={absorbing} umount={} cmount={} ow={} delete={} rollback={:?} escapes={escapes}/{} shortest_escape={}",
        short(&mu), short(&mc), short(&w), short(&d), rb.iter().map(|r| short(r)).collect::<Vec<_>>(), all.len(), shortest.unwrap_or_else(|| "none".into())
    )
}

/// E2：每条脚本走完的池、第一次 S4（两支）落下的池、第一次在会话里写不进去的池，各判一格。
#[test]
fn e2_absorbing_and_escape_sweep() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P2", "P3"]);
    let names: Vec<String> = env_list("OPUS_SCRIPTS", &[]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for (index, (name, _)) in scripts().iter().enumerate() {
                if names.is_empty() || names.iter().any(|n| n == name) {
                    tasks.push((slots, a.clone(), index));
                }
            }
        }
    }
    let lines = par_map(tasks, |(slots, a, index)| {
        let (name, ops) = &scripts()[index];
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        let mut out = Vec::new();
        let mut first_u: Option<Pool> = None;
        let mut first_c: Option<Pool> = None;
        let mut first_stuck: Option<(usize, Pool)> = None;
        for (step, op) in ops.iter().enumerate() {
            let before = pool.clone_pool();
            let r = pool.apply(op);
            if op.is_user_publish() && r != "ok" && first_stuck.is_none() {
                first_stuck = Some((step, before));
            }
            if op.is_user_publish() && r == "ok" {
                if first_u.is_none() {
                    let mut p = pool.clone_pool();
                    if p.apply(&Op::UMount) != "ok" {
                        first_u = Some(p);
                    }
                }
                if first_c.is_none() {
                    let mut p = pool.clone_pool();
                    if p.apply(&Op::CMount) != "ok" {
                        first_c = Some(p);
                    }
                }
            }
        }
        out.push(format!("E2 slots={slots} arm={a} script={name} cell=end {}", absorbing_cell(&pool)));
        if let Some(p) = first_u {
            out.push(format!("E2 slots={slots} arm={a} script={name} cell=first-S4u {}", absorbing_cell(&p)));
        }
        if let Some(p) = first_c {
            out.push(format!("E2 slots={slots} arm={a} script={name} cell=first-S4c {}", absorbing_cell(&p)));
        }
        if let Some((step, p)) = first_stuck {
            out.push(format!("E2 slots={slots} arm={a} script={name} cell=first-refused@{step} {}", absorbing_cell(&p)));
        }
        out.join("\n")
    });
    for l in lines {
        println!("{l}");
    }
}

// ---------------------------------------------------------------- E3：删了再写要几步

/// 长到放行的最大单元数（逐个长），交回 N 与最后一次的结局。
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

/// E3：D3（空间分配） 已定项 9 第 2 条。近满（逐个长到被拒，或先覆盖写 k 次再长）→ 截断成 0（删）→ 按分支 (s) 不动 / (u) 正常卸载再挂 /
/// (c) 崩了再挂 → 写回同样大小；不成就放开扫用户的填充步（建 1 个 inode / 覆盖写 1 字节 / 崩了再挂 / 正常卸载再挂），每一步之后重试，至多 8 步。
/// 报：删那一次的结局与它里面推的空发布；写回做成之前用户发布几次（删那一次不算、填充里的改用户可见状态的发布算）、准入里推的空发布累计几次。
#[test]
fn e3_delete_then_write_same_size() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P2", "P3"]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for pre in [0usize, 20, 60] {
                for branch in ["s", "u", "c"] {
                    for filler in ["inode", "ow1", "cmount", "umount"] {
                        tasks.push((slots, a.clone(), pre, branch, filler));
                    }
                }
            }
        }
    }
    let lines = par_map(tasks, |(slots, a, pre, branch, filler)| {
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        for _ in 0..pre {
            let _ = pool.apply(&Op::Ow(OW_BYTES));
        }
        let (n, why) = grow_until_refused(&mut pool, 40);
        if n == 0 {
            return format!("E3 slots={slots} arm={a} pre={pre} branch={branch} filler={filler} N=0 grow_stop={} (长不出一个单元)", short(&why));
        }
        let stats0 = pool.stats;
        let delete = pool.user_publish(&Op::Empty);
        let b = match branch {
            "u" => pool.apply(&Op::UMount),
            "c" => pool.apply(&Op::CMount),
            _ => "-".into(),
        };
        let stats1 = pool.stats;
        let retry = Op::Seq { units: n, extra: 0 };
        let mut steps = Vec::new();
        let mut success_after: Option<usize> = None;
        let mut user_publishes = 0usize;
        let mut dead = false;
        if pool.session.is_some() {
            let r = pool.user_publish(&retry);
            steps.push(format!("{}(e{})", short(&r.result), r.empties));
            if r.result == "ok" {
                success_after = Some(0);
            } else {
                for j in 1..=8 {
                    let f = match filler {
                        "inode" => {
                            let o = pool.user_publish(&Op::Inodes(1));
                            if o.result == "ok" {
                                user_publishes += 1;
                            }
                            format!("{}(e{})", short(&o.result), o.empties)
                        }
                        "ow1" => {
                            let o = pool.user_publish(&Op::Ow(1));
                            if o.result == "ok" {
                                user_publishes += 1;
                            }
                            format!("{}(e{})", short(&o.result), o.empties)
                        }
                        "cmount" => short(&pool.apply(&Op::CMount)),
                        _ => short(&pool.apply(&Op::UMount)),
                    };
                    if pool.session.is_none() {
                        steps.push(format!("{f}/dead"));
                        dead = true;
                        break;
                    }
                    let r = pool.user_publish(&retry);
                    steps.push(format!("{f}/{}(e{})", short(&r.result), r.empties));
                    if r.result == "ok" {
                        success_after = Some(j);
                        break;
                    }
                }
            }
        } else {
            dead = true;
        }
        let _ = opus_r3_take_traces();
        let v = pool.violations();
        let empties_after_delete = pool.stats.empties_in_admission - stats1.empties_in_admission;
        format!(
            "E3 slots={slots} arm={a} pre={pre} branch={branch} filler={filler} N={n} grow_stop={} delete={}(e{}) branch_op={} success_after={success_after:?} user_publishes_before_success={user_publishes} empties_in_admission_after_delete={empties_after_delete} dead={dead} steps=[{}] viol={}",
            short(&why),
            short(&delete.result),
            delete.empties,
            short(&b),
            steps.join(" "),
            v.len()
        )
    });
    for l in lines {
        println!("{l}");
    }
}

// ---------------------------------------------------------------- E4：P3 的多扣

/// E4：沿主线（按 OPUS_MAIN 臂走，默认 P3）每一次用户发布之前，在两份拷贝上同一步分别按 P1、P3 做一次（带各自的 C283），
/// 数「P3 拒而 P1 放行」「P1 拒而 P3 放行」各几格；主线上每次按 P3 式子放行的发布，量它的多扣：
/// 预言 = 这次之前的可用 − 这次判的需求（全部新写），实际 = 这次之后同一式子（需求 0）读出的可用，多扣 = 实际 − 预言
/// （正 = 需求那一边算多了：这次根写下时按谓词回收的那一笔、换下的固定点落进可用那一边之类）。根环转过之前与之后分开报。
#[test]
fn e4_overcharge_and_p1_p3_verdicts() {
    let main_arms = env_list("OPUS_MAIN", &["P3", "P1"]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for m in &main_arms {
            for (index, _) in scripts().iter().enumerate() {
                tasks.push((slots, m.clone(), index));
            }
        }
    }
    let lines = par_map(tasks, |(slots, m, index)| {
        let (name, ops) = &scripts()[index];
        let main = arm_named(&m);
        set_arm(main);
        let mut pool = Pool::start(slots);
        let ring = 3 * pool.params.geometry.root_ring_slots_per_region.count();
        let mut p3_only_refused = 0usize;
        let mut p1_only_refused = 0usize;
        let mut both_refused = 0usize;
        let mut both_ok = 0usize;
        let mut first_p3_only = String::new();
        let mut first_p1_only = String::new();
        let mut hist_before: BTreeMap<i128, usize> = BTreeMap::new();
        let mut hist_after: BTreeMap<i128, usize> = BTreeMap::new();
        for (step, op) in ops.iter().enumerate() {
            if op.is_user_publish() && pool.session.is_some() {
                let mut a = pool.clone_pool();
                set_arm(arm_named("P1"));
                let ra = a.user_publish(op);
                let mut b = pool.clone_pool();
                set_arm(arm_named("P3"));
                let rb = b.user_publish(op);
                set_arm(main);
                let _ = opus_r3_take_traces();
                match (ra.result == "ok", rb.result == "ok") {
                    (true, false) => {
                        p3_only_refused += 1;
                        if first_p3_only.is_empty() {
                            first_p3_only = format!("step{step}:{op:?} P1=ok(e{}) P3={}(e{})", ra.empties, short(&rb.result), rb.empties);
                        }
                    }
                    (false, true) => {
                        p1_only_refused += 1;
                        if first_p1_only.is_empty() {
                            first_p1_only = format!("step{step}:{op:?} P1={}(e{}) P3=ok(e{})", short(&ra.result), ra.empties, rb.empties);
                        }
                    }
                    (false, false) => both_refused += 1,
                    (true, true) => both_ok += 1,
                }
            }
            let txg_before = pool.current_txg().unwrap_or(0);
            let _ = opus_r3_take_traces();
            let r = pool.apply(op);
            let traces = opus_r3_take_traces();
            if main.name == "P3" && op.is_user_publish() && r == "ok" {
                // 最后一条发布痕迹就是做成的那一次（C283 推的空发布不判准入、不留痕迹）。
                if let Some(t) = traces.iter().rev().find(|t| t.site == "publish" && t.admitted) {
                    let predicted = t.available_slots.iter().min().copied().unwrap_or(0) - i128::from(t.judged_demand_slots);
                    let actual = pool.available_now().unwrap_or(0);
                    let over = actual - predicted;
                    if txg_before >= ring {
                        *hist_after.entry(over).or_default() += 1;
                    } else {
                        *hist_before.entry(over).or_default() += 1;
                    }
                }
            }
        }
        format!(
            "E4 slots={slots} main={m} script={name} both_ok={both_ok} both_refused={both_refused} P3_only_refused={p3_only_refused} P1_only_refused={p1_only_refused} first_P3_only=[{first_p3_only}] first_P1_only=[{first_p1_only}] overcharge_before_ring_wraps={hist_before:?} overcharge_after_ring_wraps={hist_after:?}"
        )
    });
    for l in lines {
        println!("{l}");
    }
}

// ---------------------------------------------------------------- E5：崩在准入推空发布的那一串中间、崩在删除类发布中间（层 0 枚举域）

/// 这条腿全部 E5 跑的全量状态数（`closed_form_state_count` 按流算），跨任务累加，超过约 10⁷ 剩下的不跑。
static E5_STATES_BUDGET_USED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
const E5_STATES_PER_STREAM_LIMIT: u64 = 1_000_000;
const E5_STATES_TOTAL_LIMIT: u64 = 10_000_000;

fn versions_known(book: &Book) -> Vec<PublishedVersion> {
    book.state_of_root
        .iter()
        .filter_map(|((instance, txg), state)| {
            book.content_of_state.get(state).map(|c| PublishedVersion {
                instance: InstanceGeneration(*instance),
                checkpoint_txg: CheckpointTxg(*txg),
                content: (**c).clone(),
            })
        })
        .collect()
}

/// 在 `pool` 这一刻录下 `op`（按当前臂，带 C283），按层 0 的枚举域（每段都展开）枚举这条流的崩溃状态：
/// 每个状态恢复之后跑 checker（枚举器自己跑，崩溃镜像上），再按同一臂可写挂载、挂载之后的镜像再跑一次 checker，
/// 再写一次同样的 `op`（写不进就崩了再挂再写一次）。只数出状态数、不跑时 `count_only`。
fn enumerate_op(tag: &str, pool: &Pool, op: &Op, count_only: bool) -> String {
    enumerate_stream(tag, pool, op, count_only, false)
}

/// `raise_only`：录的流只到准入推的那一串抬 F 为止（先写系统配置那一步与那几次空发布），不含之后的用户发布；
/// 崩溃状态落在「准入推空发布的那一串」里，挂载之后再写一次 `op`。
fn enumerate_stream(tag: &str, pool: &Pool, op: &Op, count_only: bool, raise_only: bool) -> String {
    let base = pool.memory_pool();
    let mut run = pool.clone_pool();
    run.capture_start();
    let o = if raise_only {
        let c = run.ceiling();
        match c {
            Ok(c) if c > run.floor().unwrap_or(0) => match run.raise_to(c) {
                Ok(n) => PubOutcome { result: format!("raised(F={c})"), empties: n, raises: 1 },
                Err(e) => PubOutcome { result: e, empties: 0, raises: 0 },
            },
            Ok(c) => PubOutcome { result: format!("F-at-ceiling({c})"), empties: 0, raises: 0 },
            Err(e) => PubOutcome { result: e, empties: 0, raises: 0 },
        }
    } else {
        run.user_publish(op)
    };
    let ops = run.capture_take();
    let _ = opus_r3_take_traces();
    let (writes, segments) = writes_and_segments(&ops, &geometry());
    let count = closed_form_state_count(&segments);
    let seg_sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    let head = format!("{tag} op={op:?} outcome={}(e{},raises{}) writes={} segments={seg_sizes:?} states={count}", short(&o.result), o.empties, o.raises, writes.len());
    if count_only {
        return format!("{head} (只数)");
    }
    if count > E5_STATES_PER_STREAM_LIMIT {
        return format!("{head} SKIPPED(一条流超过 {E5_STATES_PER_STREAM_LIMIT})");
    }
    let used = E5_STATES_BUDGET_USED.fetch_add(count, std::sync::atomic::Ordering::SeqCst);
    if used + count > E5_STATES_TOTAL_LIMIT {
        return format!("{head} SKIPPED(全腿累计将超过 {E5_STATES_TOTAL_LIMIT}，已用 {used})");
    }
    let Some(judged) = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua) else {
        return format!("{head} SKIPPED(流里没有根槽写)");
    };
    let versions = versions_known(&run.book);
    let mut mount_tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut writable_tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut post_violation_states = 0u64;
    let mut first_post = String::new();
    let mut first_stuck = String::new();
    let mut index = 0u64;
    let mut observe = |image: &CrashImage<'_>, _report: &singlefs_core::recovery::RecoveryReport| {
        let mut mp = MemoryPool { devices: image.base.devices.clone(), device_size_in_bytes: image.base.device_size_in_bytes };
        for (w, p) in image.writes.iter().zip(&image.persisted) {
            if *p {
                w.contents.apply_to(mp.devices.get_mut(&w.device).expect("盘"), w.offset);
            }
        }
        let images: Vec<(DeviceIdentity, SparseDevice)> = mp.devices.into_iter().collect();
        let mut q = pool.from_images(&images);
        let m = q.mount();
        *mount_tally.entry(short(&m)).or_default() += 1;
        if m == "ok" {
            let v = q.violations();
            if !v.is_empty() {
                post_violation_states += 1;
                if first_post.is_empty() {
                    first_post = format!("state{index}: {}", v[0]);
                }
            }
            let w = q.user_publish(op);
            let key = if w.result == "ok" {
                format!("now(e{})", w.empties.min(3))
            } else {
                let mut r = q.clone_pool();
                let m2 = r.apply(&Op::CMount);
                let w2 = if m2 == "ok" { r.user_publish(op).result } else { format!("remount:{}", short(&m2)) };
                if w2 == "ok" { "after-cmount".to_string() } else { format!("never({}|{})", short(&w.result), short(&w2)) }
            };
            if key.starts_with("never") && first_stuck.is_empty() {
                first_stuck = format!("state{index}: {key}");
            }
            *writable_tally.entry(key).or_default() += 1;
        } else if first_stuck.is_empty() {
            first_stuck = format!("state{index}: mount {}", short(&m));
        }
        index += 1;
    };
    let parallelism = singlefs_harness::crash::Layer0Parallelism {
        worker_threads: std::num::NonZeroUsize::MIN,
        worker_threads_source: singlefs_harness::crash::Layer0WorkerThreadsSource::GivenByCaller,
        slice_length: singlefs_harness::crash::Layer0SliceLength::ScaledToWorkerThreads,
    };
    let tally = singlefs_harness::crash::enumerate_layer0_in_state_slices(
        &base,
        &writes,
        &segments,
        judged,
        &versions,
        &|_segment_index, _segment| true,
        parallelism,
        Some(&mut observe),
    );
    let _ = opus_r3_take_traces();
    let pre_checker: u64 = tally.checker_violated_states.values().sum();
    format!(
        "{head} enumerated={} oracle_violations={} pre_checker_violations(按条累加)={pre_checker} pre_first={:?} post_mount_checker_violation_states={post_violation_states} post_first=[{first_post}] mount={mount_tally:?} writable={writable_tally:?} first_stuck=[{first_stuck}] oracle_first={:?}",
        tally.states,
        tally.violations,
        tally.checker_first_violation.iter().next(),
        tally.first_violation
    )
}

/// E5a：准入推空发布的那一串。沿脚本走（按臂），每一次用户发布之前在拷贝上试一次：它在准入里推了空发布，就把这一刻取下来；
/// 每条 (盘宽, 臂, 脚本) 取前 OPUS_E5_POINTS 个这样的点（默认 1），录那一次（抬 F 的系统配置写、那一串空发布、用户发布）的流枚举。
/// E5b：删除类发布。近满（逐个长到被拒）之后截断成 0 那一次，录流枚举。
#[test]
fn e5_crash_inside_the_admission_chain_and_the_delete() {
    let count_only = std::env::var("OPUS_E5_COUNT_ONLY").is_ok();
    let points: usize = std::env::var("OPUS_E5_POINTS").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let arms = env_list("OPUS_ARMS", &["P1", "P3"]);
    let names = env_list("OPUS_SCRIPTS", &["a-ow150", "c-grow6-truncate-x10"]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for (index, (name, _)) in scripts().iter().enumerate() {
                if names.iter().any(|n| n == name) {
                    tasks.push((slots, a.clone(), Some(index)));
                }
            }
            tasks.push((slots, a.clone(), None));
        }
    }
    // 取样（估时之后缩的）：OPUS_E5_PICK = "盘宽:臂:脚本名;…"，只跑点到的那几条历史；崩溃状态一个不缩。
    if let Ok(pick) = std::env::var("OPUS_E5_PICK") {
        let picks: Vec<(u64, String, String)> = pick
            .split(';')
            .map(|item| {
                let parts: Vec<&str> = item.split(':').collect();
                (parts[0].parse().unwrap(), parts[1].to_string(), parts[2].to_string())
            })
            .collect();
        tasks.retain(|(slots, a, index)| match index {
            Some(i) => picks.iter().any(|(s, pa, n)| s == slots && pa == a && *n == scripts()[*i].0),
            None => false,
        });
    }
    let lines = par_map(tasks, |(slots, a, index)| {
        set_arm(arm_named(&a));
        let mut out = Vec::new();
        match index {
            Some(index) => {
                let (name, ops) = &scripts()[index];
                let mut pool = Pool::start(slots);
                let mut taken = 0;
                for (step, op) in ops.iter().enumerate() {
                    if op.is_user_publish() && pool.session.is_some() && taken < points {
                        let mut probe = pool.clone_pool();
                        let o = probe.user_publish(op);
                        let _ = opus_r3_take_traces();
                        if o.empties > 0 {
                            taken += 1;
                            out.push(enumerate_stream(&format!("E5a slots={slots} arm={a} script={name} step={step} (只录抬 F 那一串)"), &pool, op, count_only, true));
                        }
                    }
                    let _ = pool.apply(op);
                }
                if taken == 0 {
                    out.push(format!("E5a slots={slots} arm={a} script={name} 没有一步在准入里推空发布"));
                }
            }
            None if std::env::var("OPUS_E5_SKIP_DELETE").is_ok() => {}
            None => {
                let mut pool = Pool::start(slots);
                let (n, why) = grow_until_refused(&mut pool, 40);
                out.push(enumerate_op(&format!("E5b slots={slots} arm={a} N={n} grow_stop={}", short(&why)), &pool, &Op::Empty, count_only));
            }
        }
        out.join("\n")
    });
    for l in lines {
        println!("{l}");
    }
}

// ---------------------------------------------------------------- E6：准入推空发布抬 F 会不会抬掉用户还要的回退候选

/// 这条时间线上（没被抛弃的根，按 txg 从新到旧）最近 4 个不同状态：按驱动自己的账数，不读 core 的「非空」判法。
/// 交回状态号，第一个是现行状态。
fn e6_scripts() -> Vec<(&'static str, Vec<Op>)> {
    let mut all_scripts = scripts();
    // i：每次覆盖写之后崩了再挂一次（暖机与写行的根挤根环）。
    let mut i_ops = vec![Op::Seq { units: 6, extra: 0 }];
    for _ in 0..40 {
        i_ops.push(Op::Ow(OW_BYTES));
        i_ops.push(Op::CMount);
    }
    all_scripts.push(("i-seq6-ow-cmount-x40", i_ops));
    // j：两个状态来回回退，再夹覆盖写。
    let mut j_ops = vec![Op::Seq { units: 6, extra: 0 }, Op::Ow(OW_BYTES), Op::Ow(OW_BYTES), Op::Ow(OW_BYTES)];
    for _ in 0..20 {
        j_ops.push(Op::Rollback(1));
        j_ops.push(Op::Rollback(1));
        j_ops.push(Op::Ow(OW_BYTES));
    }
    all_scripts.push(("j-rollback-pingpong", j_ops));
    // k：每次覆盖写之后崩了再挂两次（只靠用户的挂载挤根环，对照）。
    let mut k_ops = vec![Op::Seq { units: 6, extra: 0 }];
    for _ in 0..30 {
        k_ops.push(Op::Ow(OW_BYTES));
        k_ops.push(Op::CMount);
        k_ops.push(Op::CMount);
    }
    all_scripts.push(("k-seq6-ow-cmount2-x30", k_ops));
    // l：近满（长到 10 个单元）之后每次覆盖写夹一次崩了再挂（准入推的空发布与挂载的根一起挤根环）。
    let mut l_ops = vec![Op::Seq { units: 10, extra: 0 }];
    for _ in 0..40 {
        l_ops.push(Op::Ow(OW_BYTES));
        l_ops.push(Op::CMount);
    }
    all_scripts.push(("l-seq10-ow-cmount-x40", l_ops));
    all_scripts
}

fn newest_four_states_on_the_timeline(pool: &Pool) -> Vec<u64> {
    let mut roots: Vec<(&(u32, u64), &u64)> = pool.book.state_of_root.iter().filter(|(k, _)| !pool.book.abandoned.contains(k)).collect();
    roots.sort_by(|l, r| (r.0 .1, r.0 .0).cmp(&(l.0 .1, l.0 .0)));
    let mut out: Vec<u64> = Vec::new();
    for (_, s) in roots {
        if !out.contains(s) {
            out.push(*s);
        }
        if out.len() == 4 {
            break;
        }
    }
    out
}

/// 一个状态还是不是回退候选：根环里有它的根、没被抛弃、txg ≥ 现行根上的 F，且在拷贝上真回退得过去（现行状态本身算在）。
/// 交回 "ok" / "evicted"（环里没有它的根）/ "below-F"（环里有、都低于 F）/ 回退的错。
fn candidate_status(pool: &Pool, state: u64) -> String {
    if state == pool.state {
        return "current".into();
    }
    let f = pool.floor().unwrap_or(0);
    let ring = pool.ring_roots();
    let mine: Vec<(u64, u32)> = ring
        .iter()
        .filter(|(t, i, _)| pool.book.state_of_root.get(&(*i, *t)) == Some(&state) && !pool.book.abandoned.contains(&(*i, *t)))
        .map(|(t, i, _)| (*t, *i))
        .collect();
    if mine.is_empty() {
        return "evicted".into();
    }
    let Some((t, i)) = mine.iter().copied().filter(|(t, _)| *t >= f).max() else { return "below-F".into() };
    let mut p = pool.clone_pool();
    let r = p.rollback_to(state, t, i);
    let _ = opus_r3_take_traces();
    r
}

/// E6：沿脚本走（按臂），每一步之后核「这条时间线上最近 4 个不同状态」是不是都还是回退候选；一格不是，按这一步是什么分类记：
/// 这一步的准入推了空发布（C283 那一串）/ 用户卸载、挂载、回退 / 别的。另加两条脚本：挂载与回退夹在覆盖写之间（i、j）。
#[test]
fn e6_newest_four_states_stay_candidates() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P3"]);
    let n_scripts = e6_scripts().len();
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for index in 0..n_scripts {
                tasks.push((slots, a.clone(), index));
            }
        }
    }
    let lines = par_map(tasks, |(slots, a, index)| {
        let all_scripts = e6_scripts();
        let (name, ops) = &all_scripts[index];
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        let mut lost: BTreeMap<String, usize> = BTreeMap::new();
        let mut first: BTreeMap<String, String> = BTreeMap::new();
        let mut checked = 0usize;
        // 最近一次正常卸载抬到的 F（B1：卸载之后候选只剩现行那一版，D16（发布语义） 已定项 1「正常卸载的代价」）。
        let mut last_unmount_floor = 0u64;
        for (step, op) in ops.iter().enumerate() {
            let empties_before = pool.stats.empties_in_admission;
            let r = pool.apply(op);
            if pool.session.is_none() {
                break;
            }
            if matches!(op, Op::UMount) && r == "ok" {
                last_unmount_floor = pool.floor().unwrap_or(0);
            }
            let pushed = pool.stats.empties_in_admission - empties_before;
            let cause = if pushed > 0 {
                "admission-pushed"
            } else if matches!(op, Op::UMount | Op::CMount | Op::Rollback(_) | Op::Abandon) {
                "user-mount-or-rollback"
            } else {
                "other"
            };
            for s in newest_four_states_on_the_timeline(&pool) {
                let st = candidate_status(&pool, s);
                checked += 1;
                if st == "ok" || st == "current" {
                    continue;
                }
                // 正常卸载之后 F = 那一刻现行那一版的 txg（B1 定的代价）：这个状态最新的根不高于那个 F，单列，不算打中。
                let newest_root_of_s = pool.book.state_of_root.iter().filter(|(k, v)| **v == s && !pool.book.abandoned.contains(k)).map(|(k, _)| k.1).max().unwrap_or(0);
                let b1 = newest_root_of_s < last_unmount_floor;
                // 根环里还剩几个不同状态：≤ 3 时是 D16（发布语义） 已定项 1 首段写明的边界（环里只有这几个状态，任何机制给不出环里没有的）。
                let ring_states = pool.distinct_states_in_ring().len();
                let st = if st == "evicted" { format!("evicted(ring-has-{ring_states}-states)") } else { st };
                let key = format!("{}:{}", if b1 { "before-last-umount(B1)" } else { cause }, st);
                *lost.entry(key.clone()).or_default() += 1;
                first.entry(key).or_insert_with(|| format!("step{step}:{op:?}->{} F={:?} txg={:?} state{s}", short(&r), pool.floor(), pool.current_txg()));
            }
        }
        format!("E6 slots={slots} arm={a} script={name} checked={checked} lost={lost:?} first={first:?}")
    });
    for l in lines {
        println!("{l}");
    }
}

// ---------------------------------------------------------------- E7：行数逼近 369 的整数倍（挂载路径 rows0 的分叉）

/// E7：先连着挂载把实例表的行数推到 369 附近（每次可写挂载写一行，行不回收）：交替崩了再挂与一次覆盖写，至 rows0 = R0；
/// 之后在同一个会话里覆盖写到被拒（按臂，带 C283），每一次放行之后在拷贝上两支（u / c）重挂，数 S4、S4b；
/// 并报挂载路径那一次读数里的 rows0、挂载期承诺量，与会话里（发布路径）同一刻读数的差。
#[test]
fn e7_instance_rows_near_a_multiple_of_369() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P2", "P3"]);
    let targets: Vec<u64> = env_list("OPUS_E7_ROWS", &["360", "363", "364", "365", "366", "367", "368", "369", "370"]).iter().map(|x| x.parse().unwrap()).collect();
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            tasks.push((slots, a.clone()));
        }
    }
    let lines = par_map(tasks, |(slots, a)| {
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        let mut out = Vec::new();
        let rows0 = |p: &Pool| p.session.as_ref().map(|s| s.allocator.instance_rows_after_this_mounts_row_publish()).unwrap_or(0);
        let mut guard = 0;
        for target in &targets {
            while rows0(&pool) < *target && guard < 2000 {
                guard += 1;
                let m = pool.apply(&Op::CMount);
                if m != "ok" {
                    out.push(format!("E7 slots={slots} arm={a} 推行数时挂载被拒 rows0={} {}", rows0(&pool), short(&m)));
                    break;
                }
            }
            if pool.session.is_none() || rows0(&pool) != *target {
                out.push(format!("E7 slots={slots} arm={a} target={target} 没推到 rows0={}", rows0(&pool)));
                break;
            }
            let mut p = pool.clone_pool();
            let mut admitted = 0;
            let mut s4 = [0usize; 2];
            let mut s4b = [0usize; 2];
            let mut first = [String::new(), String::new()];
            let mut stop = String::new();
            for step in 0..200 {
                let r = p.user_publish(&Op::Ow(OW_BYTES));
                if r.result != "ok" {
                    stop = format!("step{step}:{}", short(&r.result));
                    break;
                }
                admitted += 1;
                for (b, branch) in [Op::UMount, Op::CMount].iter().enumerate() {
                    let mut q = p.clone_pool();
                    let _ = opus_r3_take_traces();
                    let m = q.apply(branch);
                    let traces = opus_r3_take_traces();
                    if m != "ok" {
                        s4[b] += 1;
                        if first[b].is_empty() {
                            let mt = traces.iter().rev().find(|t| t.site == "mount").map(|t| format!("mount av{:?} mc{} rows{}", t.available_slots, t.mount_commitment_slots, t.rows0)).unwrap_or_default();
                            first[b] = format!("step{step}:{} [{mt}] session_av={:?} session_rows0={} then {}", short(&m), p.available_now(), rows0(&p), absorbing_cell(&q));
                        }
                    } else if q.user_publish(&Op::Ow(OW_BYTES)).result != "ok" {
                        s4b[b] += 1;
                    }
                }
            }
            let _ = opus_r3_take_traces();
            out.push(format!(
                "E7 slots={slots} arm={a} rows0={target} admitted={admitted} stop=[{stop}] S4u={} S4c={} S4bu={} S4bc={} first_S4u=[{}] first_S4c=[{}]",
                s4[0], s4[1], s4b[0], s4b[1], first[0], first[1]
            ));
        }
        out.join("\n")
    });
    for l in lines {
        println!("{l}");
    }
}

// ---------------------------------------------------------------- E8：同一段历史里挂着时回退过、又有根被抛弃（被抛弃根独占量的分叉）

/// E8：长到 6 个单元，覆盖写 k 次，挂着时回退到上一个状态，再覆盖写一次，然后崩溃恢复抛弃最新那条根（它引用的数据单元只被它引用）；
/// 之后覆盖写到被拒（按臂，带 C283），每一次放行之后在拷贝上两支重挂，数 S4、S4b；每一步另报会话里（发布路径）读数的隔离槽与
/// 同一刻崩了再挂那一次挂载路径读数的隔离槽、可用之差（结构性分叉那一格）。
#[test]
fn e8_rollback_then_abandoned_root() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P2", "P3"]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for k in [2usize, 6, 12] {
                for order in ["rb-ow-abandon", "ow-rb-abandon-after-ow", "no-rollback-ow-abandon"] {
                    tasks.push((slots, a.clone(), k, order));
                }
            }
        }
    }
    let lines = par_map(tasks, |(slots, a, k, order)| {
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        let mut pre = Vec::new();
        pre.push(pool.apply(&Op::Seq { units: 6, extra: 0 }));
        for _ in 0..k {
            pre.push(pool.apply(&Op::Ow(OW_BYTES)));
        }
        match order {
            "rb-ow-abandon" => {
                pre.push(pool.apply(&Op::Rollback(1)));
                pre.push(pool.apply(&Op::Ow(OW_BYTES)));
            }
            "ow-rb-abandon-after-ow" => {
                pre.push(pool.apply(&Op::Ow(OW_BYTES)));
                pre.push(pool.apply(&Op::Rollback(2)));
                pre.push(pool.apply(&Op::Ow(OW_BYTES)));
            }
            _ => {
                // 对照：同样多的覆盖写，不回退。
                pre.push(pool.apply(&Op::Ow(OW_BYTES)));
                pre.push(pool.apply(&Op::Ow(OW_BYTES)));
            }
        }
        let ab = pool.apply(&Op::Abandon);
        pre.push(format!("abandon={ab}"));
        let viol_after_abandon = pool.violations().len();
        pre.push(format!("viol_after_abandon={viol_after_abandon}"));
        if pool.session.is_none() {
            return format!("E8 slots={slots} arm={a} k={k} order={order} pre=[{}] 抛弃之后没有会话", rle(&pre));
        }
        let iso0 = pool.counts().map(|c| c.3).unwrap_or(0);
        let mut admitted = 0;
        let mut s4 = [0usize; 2];
        let mut s4b = [0usize; 2];
        let mut first = [String::new(), String::new(), String::new(), String::new()];
        let mut divergences: BTreeMap<String, usize> = BTreeMap::new();
        let mut stop = String::new();
        for step in 0..200 {
            let r = pool.user_publish(&Op::Ow(OW_BYTES));
            if r.result != "ok" {
                stop = format!("step{step}:{}", short(&r.result));
                break;
            }
            admitted += 1;
            let session_iso = pool.counts().map(|c| c.3).unwrap_or(0);
            let session_av = pool.available_now().unwrap_or(0);
            for (b, branch) in [Op::UMount, Op::CMount].iter().enumerate() {
                let mut q = pool.clone_pool();
                let _ = opus_r3_take_traces();
                let m = q.apply(branch);
                let traces = opus_r3_take_traces();
                if let Some(t) = traces.iter().rev().find(|t| t.site == "mount") {
                    if b == 1 {
                        let key = format!("iso(session {session_iso} -> mount {:?})", t.isolated_slots);
                        *divergences.entry(key).or_default() += 1;
                    }
                }
                if m != "ok" {
                    s4[b] += 1;
                    if first[b].is_empty() {
                        let mt = traces.iter().rev().find(|t| t.site == "mount").map(|t| format!("mount av{:?} iso{:?} def{:?} mc{}", t.available_slots, t.isolated_slots, t.deferred_slots, t.mount_commitment_slots)).unwrap_or_default();
                        first[b] = format!("step{step}:{} [{mt}] session_av={session_av} session_iso={session_iso} then {}", short(&m), absorbing_cell(&q));
                    }
                } else {
                    let w = q.user_publish(&Op::Ow(OW_BYTES));
                    if w.result != "ok" {
                        s4b[b] += 1;
                        if first[2 + b].is_empty() {
                            first[2 + b] = format!("step{step}:{}", short(&w.result));
                        }
                    }
                }
            }
        }
        let _ = opus_r3_take_traces();
        let v = pool.violations();
        format!(
            "E8 slots={slots} arm={a} k={k} order={order} pre=[{}] iso_after_abandon={iso0} admitted={admitted} stop=[{stop}] S4u={} S4c={} S4bu={} S4bc={} first=[{}] divergences={divergences:?} viol={}",
            rle(&pre), s4[0], s4[1], s4b[0], s4b[1], first.join(" | "), v.len()
        )
    });
    for l in lines {
        println!("{l}");
    }
}

/// E6d：E6 一格的逐步明细（OPUS_E6D = "盘宽:臂:脚本名:从:到"）：每步之后打根环里的根（txg、实例、F、驱动记的状态号）与最近 4 个状态各自的候选判定。
#[test]
fn e6d_detail() {
    let spec = std::env::var("OPUS_E6D").unwrap_or_else(|_| "240:T:j-rollback-pingpong:30:36".to_string());
    let parts: Vec<&str> = spec.split(':').collect();
    let slots: u64 = parts[0].parse().unwrap();
    set_arm(arm_named(parts[1]));
    let (_, ops) = e6_scripts().into_iter().find(|(n, _)| *n == parts[2]).unwrap();
    let from: usize = parts[3].parse().unwrap();
    let upto: usize = parts[4].parse().unwrap();
    let mut pool = Pool::start(slots);
    for (step, op) in ops.iter().enumerate().take(upto + 1) {
        let r = pool.apply(op);
        if step < from {
            continue;
        }
        let ring: Vec<String> = pool.ring_roots().iter().map(|(t, i, f)| format!("{t}/{i}/F{f}/s{:?}", pool.book.state_of_root.get(&(*i, *t)))).collect();
        let four: Vec<String> = newest_four_states_on_the_timeline(&pool).iter().map(|s| format!("s{s}:{}", candidate_status(&pool, *s))).collect();
        let _ = opus_r3_take_traces();
        println!("E6d step{step} {op:?} -> {} state={} ring({})=[{}] four=[{}]", short(&r), pool.state, ring.len(), ring.join(" "), four.join(" "));
    }
}

// ---------------------------------------------------------------- E10：只重挂（不写）能不能把池挂死

/// E10：起点之后先长到 u 个单元（0 = 不长），然后只做 (c) 崩了再挂 / (u) 正常卸载再挂，至多 400 次；报第一次挂载被拒是第几次、
/// 被拒那一刻的挂载读数（可用、defer、隔离、挂载期承诺量、rows0），与被拒之后的吸收态格。
#[test]
fn e10_remount_only() {
    let arms = env_list("OPUS_ARMS", &["T", "P1", "P2", "P3"]);
    let mut tasks = Vec::new();
    for slots in slot_sizes() {
        for a in &arms {
            for u in [0usize, 4, 10] {
                for branch in ["c", "u"] {
                    tasks.push((slots, a.clone(), u, branch));
                }
            }
        }
    }
    let lines = par_map(tasks, |(slots, a, u, branch)| {
        set_arm(arm_named(&a));
        let mut pool = Pool::start(slots);
        let grow = if u > 0 { pool.apply(&Op::Seq { units: u, extra: 0 }) } else { "-".into() };
        let op = if branch == "c" { Op::CMount } else { Op::UMount };
        let mut refused_at = None;
        let mut detail = String::new();
        for i in 1..=400 {
            let before = pool.clone_pool();
            let _ = opus_r3_take_traces();
            let r = pool.apply(&op);
            let traces = opus_r3_take_traces();
            if r != "ok" {
                refused_at = Some(i);
                let mt = traces.iter().rev().find(|t| t.site == "mount").map(|t| format!("av{:?} alloc{:?} def{:?} iso{:?} mc{} ck{} rows{}", t.available_slots, t.allocated_slots, t.deferred_slots, t.isolated_slots, t.mount_commitment_slots, t.checkpoint_share_slots, t.rows0)).unwrap_or_default();
                let prev = before.counts().map(|c| format!("before: alloc={} def={} free={} iso={} avail_now={:?}", c.0, c.1, c.2, c.3, before.available_now())).unwrap_or_default();
                detail = format!("{} [{mt}] {prev} then {}", short(&r), absorbing_cell(&pool));
                break;
            }
        }
        format!("E10 slots={slots} arm={a} grow={u}:{} branch={branch} first_refused_remount={refused_at:?} detail=[{detail}]", short(&grow))
    });
    for l in lines {
        println!("{l}");
    }
}
