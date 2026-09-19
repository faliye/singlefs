//! 同 c381_probe.rs，去掉臂开关与 C381 错误成员，给 HEAD 版 core（未改）用：只跑甲。
//! c381-r1 云端攻方腿（Opus）探针。只在攻方仓副本上跑：core 打了 `c381-arms.diff`（四条臂按线程选），harness 取 HEAD 版。
//! 设备：稀疏内存盘外面一层全局写计数（两盘共用一个下标）：`faults` 里的下标那一次写报错且不落盘（瞬时错），
//! 下标 ≥ `crash_at` 的写一律报错且不落盘（断电）。断电之后清掉故障、可写挂载、跑 checker。
//! 前缀固定：mkfs → 取号 → 暖机 → 第一个事务（txg 3）→ 覆盖写 B（txg 4）；之后覆盖写 C（txg 5）在它的第 k 次写上报错，
//! 再由「用户」按动作序列接着做（动作全放开扫），每个断电点各跑一次。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::Mutex;

use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};

const IMAGE_BYTES: u64 = 4 << 30;

thread_local! {
    /// 开着时注入的瞬时错是「写已落盘、设备照样报错」（FUA 报错而内容已持久的那一种）。
    pub static LYING: Cell<bool> = const { Cell::new(false) };
}

#[derive(Default)]
struct Control {
    index: Cell<u64>,
    faults: RefCell<BTreeSet<u64>>,
    crash_at: Cell<Option<u64>>,
}

struct Device {
    inner: SparseBlockDevice,
    control: Rc<Control>,
}

impl BlockDevice for Device {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let index = self.control.index.get();
        self.control.index.set(index + 1);
        if self.control.crash_at.get().is_some_and(|crash| index >= crash) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("断电")));
        }
        if self.control.faults.borrow().contains(&index) {
            // 「报错但已落盘」（写落了、设备照样报错）只在 `LYING` 开着时。
            if LYING.with(Cell::get) {
                let _ = self.inner.write_at(offset, bytes, durability);
            }
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("注入的瞬时写错")));
        }
        self.inner.write_at(offset, bytes, durability)
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

#[derive(Clone)]
pub struct Snapshot {
    images: Vec<singlefs_harness::crash::SparseDevice>,
    allocator: PoolAllocator,
    b: TransactionOutput,
}

fn devices_from(images: &[singlefs_harness::crash::SparseDevice], control: &Rc<Control>) -> Vec<(DeviceIdentity, Device)> {
    images
        .iter()
        .enumerate()
        .map(|(number, image)| {
            let mut inner = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            inner.image = image.clone();
            (DeviceIdentity(u32::try_from(number).expect("盘号")), Device { inner, control: control.clone() })
        })
        .collect()
}

#[must_use]
pub fn prefix() -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let empty = vec![singlefs_harness::crash::SparseDevice::default(); 2];
    let mut devices = devices_from(&empty, &control);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES), DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES)]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    let b = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &first_file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务");
        publish_overwrite(&mut writer, &mut allocator, &first, FirstFile { content: &[7u8; 4100], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, instance).expect("B")
    };
    Snapshot { images: devices.iter().map(|(_, device)| device.inner.image.clone()).collect(), allocator, b }
}

/// 用户在 C 失败之后的动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// 重发：乙有「结局不确定」的那一版时以它为上一版、txg + 1、同内容；否则从调用方手里的现行版原样重发上一次失败的内容。
    Retry,
    /// 从现行版发一次空发布。
    Empty,
    /// 从现行版发一次内容不同的覆盖写。
    Other,
    /// 从 C 之前那一版（B）发一次空发布：Z2-c 原历史的第 2 步。
    EmptyFromB,
    /// 放下写入口、可写挂载（进程内重开）。
    Remount,
}
pub const ACTIONS: [Action; 5] = [Action::Retry, Action::Empty, Action::Other, Action::EmptyFromB, Action::Remount];

pub const C_CONTENT: [u8; 3700] = [9u8; 3700];
const OTHER_CONTENT: [u8; 2000] = [3u8; 2000];

fn empty_plan(previous: &TransactionOutput) -> PublishPlan<'static> {
    PublishPlan {
        txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
        counter: previous.record.counter + 1,
        transaction: 0,
        instance: previous.root.instance,
        back_chain: back_chain_of(&previous.record_bytes),
        file: None,
        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    }
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    /// 每一步的结果（C 在最前）。
    pub steps: Vec<String>,
    /// 断电那一刻（或序列走完、没断电）的镜像上 checker 红的条目。
    pub checker_at_crash: Vec<String>,
    /// 清故障之后可写挂载：Ok(所选根 txg) / Err / panic。
    pub remount: String,
    /// 重开之后的镜像上 checker 红的条目。
    pub checker_after_remount: Vec<String>,
    /// 这段历史总共发了几次写（含报错的）。
    pub writes: u64,
    /// C 结束时的写下标。
    pub writes_after_c: u64,
}

fn image(devices: &[(DeviceIdentity, Device)]) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.inner.image.clone())).collect::<BTreeMap<_, _>>(), device_size_in_bytes: IMAGE_BYTES }
}

fn violated(devices: &[(DeviceIdentity, Device)]) -> Vec<String> {
    check_pool_image(&image(devices))
        .iter()
        .filter(|(_, verdict)| format!("{verdict:?}").starts_with("Violated"))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect()
}

fn short(result: &Result<TransactionOutput, PublishError>) -> String {
    match result {
        Ok(output) => format!("ok(txg {})", output.root.checkpoint_txg.0),
        Err(PublishError::BlockDevice(_)) => "err(BlockDevice)".to_string(),
        Err(other) => format!("err({})", format!("{other:?}").split([' ', '{', '(']).next().unwrap_or("?")),
    }
}

/// 一段历史：`c_fault` = C 的第几次写报错（相对 C 开始的下标），`later_faults` = C 之后的写下标（相对 C 开始）上的瞬时错，
/// `crash_at` = 相对 C 开始的写下标，从它起断电。
#[must_use]
pub fn run(parameters: &MakeFilesystemParameters, snapshot: &Snapshot, arm: u8, c_fault: Option<u64>, later_faults: &[u64], actions: &[Action], crash_at: Option<u64>) -> Outcome {
    let _ = arm;
    let control = Rc::new(Control::default());
    control.faults.borrow_mut().extend(c_fault.iter().copied().chain(later_faults.iter().copied()));
    control.crash_at.set(crash_at);
    let mut devices = devices_from(&snapshot.images, &control);
    let mut allocator = snapshot.allocator.clone();
    let b = snapshot.b.clone();
    let mut current = b.clone();
    let mut uncertain: Option<TransactionOutput> = None;
    // 上一次失败的发布的内容（`None` = 空发布）与它的上一版：「原样重发」用。
    let mut last_failed: Option<(Option<Vec<u8>>, TransactionOutput)> = None;
    let mut outcome = Outcome::default();
    let crashed = |control: &Control| control.crash_at.get().is_some_and(|crash| control.index.get() > crash);
    let mut record = |result: Result<TransactionOutput, PublishError>, content: Option<Vec<u8>>, previous: &TransactionOutput, current: &mut TransactionOutput, uncertain: &mut Option<TransactionOutput>, last_failed: &mut Option<(Option<Vec<u8>>, TransactionOutput)>, steps: &mut Vec<String>| {
        steps.push(short(&result));
        match result {
            Ok(output) => {
                *current = output;
                *uncertain = None;
                *last_failed = None;
            }
            Err(_) => *last_failed = Some((content, previous.clone())),
        }
    };
    {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let result = publish_overwrite(&mut writer, &mut allocator, &b, FirstFile { content: &C_CONTENT, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 }, b.root.instance);
        let previous = b.clone();
        record(result, Some(C_CONTENT.to_vec()), &previous, &mut current, &mut uncertain, &mut last_failed, &mut outcome.steps);
    }
    outcome.writes_after_c = control.index.get();
    let mut session_alive = true;
    for action in actions {
        if crashed(&control) || !session_alive {
            break;
        }
        match action {
            Action::Remount => {
                let mounted = catch_unwind(AssertUnwindSafe(|| mount_writable(parameters, &mut devices)));
                match mounted {
                    Ok(Ok(mounted)) => {
                        outcome.steps.push(format!("remount ok(chosen {})", mounted.output.chosen_root.checkpoint_txg.0));
                        allocator = mounted.allocator;
                        match mounted.current.into_file_version() {
                            Some(version) => current = version,
                            None => session_alive = false,
                        }
                        uncertain = None;
                        last_failed = None;
                    }
                    Ok(Err(error)) => {
                        outcome.steps.push(format!("remount err({})", format!("{error:?}").chars().take(80).collect::<String>()));
                        session_alive = false;
                    }
                    Err(_) => {
                        outcome.steps.push("remount PANIC".to_string());
                        session_alive = false;
                    }
                }
            }
            Action::Retry | Action::Empty | Action::Other | Action::EmptyFromB => {
                let (previous, content): (TransactionOutput, Option<Vec<u8>>) = match action {
                    Action::Retry => match (&uncertain, &last_failed) {
                        (Some(version), Some((content, _))) => (version.clone(), content.clone()),
                        (None, Some((content, previous))) => (previous.clone(), content.clone()),
                        _ => (current.clone(), Some(C_CONTENT.to_vec())),
                    },
                    Action::Empty => (current.clone(), None),
                    Action::Other => (current.clone(), Some(OTHER_CONTENT.to_vec())),
                    Action::EmptyFromB => (b.clone(), None),
                    Action::Remount => unreachable!(),
                };
                let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                let result = catch_unwind(AssertUnwindSafe(|| match &content {
                    Some(bytes) => publish_overwrite(&mut writer, &mut allocator, &previous, FirstFile { content: bytes, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180 }, previous.root.instance),
                    None => publish_version(&mut writer, &mut allocator, empty_plan(&previous), Some(&previous)),
                }));
                match result {
                    Ok(result) => record(result, content, &previous, &mut current, &mut uncertain, &mut last_failed, &mut outcome.steps),
                    Err(_) => {
                        outcome.steps.push("publish PANIC".to_string());
                        session_alive = false;
                    }
                }
            }
        }
    }
    outcome.writes = control.index.get();
    outcome.checker_at_crash = violated(&devices);
    control.faults.borrow_mut().clear();
    control.crash_at.set(None);
    let mounted = catch_unwind(AssertUnwindSafe(|| mount_writable(parameters, &mut devices)));
    outcome.remount = match mounted {
        Ok(Ok(mounted)) => format!("ok(chosen {})", mounted.output.chosen_root.checkpoint_txg.0),
        Ok(Err(error)) => format!("REFUSED {}", format!("{error:?}").chars().take(120).collect::<String>()),
        Err(_) => "PANIC".to_string(),
    };
    outcome.checker_after_remount = violated(&devices);
    outcome
}

pub fn arm_name(arm: u8) -> &'static str {
    ["甲", "乙", "丙", "丁"][usize::from(arm)]
}

/// C 的第几次写落在哪一段：按 C 自己的写序（两盘各一次的单元写、记录、根槽、超级块槽）。
#[must_use]
pub fn stage_of_c_write(parameters: &MakeFilesystemParameters, snapshot: &Snapshot) -> Vec<&'static str> {
    // 在无故障的 C 上数：单元写 2×8、记录 2、根 1、超级块 2。
    let base = run(parameters, snapshot, 0, None, &[], &[], None);
    let total = base.writes_after_c;
    (0..total)
        .map(|index| if index < 16 { "S1" } else if index < 19 { "S2" } else { "S3" })
        .collect()
}

pub fn sweep<F: Fn(&MakeFilesystemParameters, &Snapshot, u8, u64) -> Vec<(String, Outcome)> + Sync>(arms: &[u8], c_writes: u64, body: F) -> Vec<(u8, u64, String, Outcome)> {
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let work: Vec<(u8, u64)> = arms.iter().flat_map(|arm| (0..c_writes).map(move |k| (*arm, k))).collect();
    let results = Mutex::new(Vec::new());
    let next = Mutex::new(0usize);
    std::thread::scope(|scope| {
        for _ in 0..24 {
            scope.spawn(|| loop {
                let item = {
                    let mut guard = next.lock().expect("锁");
                    let item = work.get(*guard).copied();
                    *guard += 1;
                    item
                };
                let Some((arm, k)) = item else { break };
                for (label, outcome) in body(&parameters, &snapshot, arm, k) {
                    results.lock().expect("锁").push((arm, k, label, outcome));
                }
            });
        }
    });
    let mut results = results.into_inner().expect("锁");
    results.sort_by(|left, right| (left.0, left.1, &left.2).cmp(&(right.0, right.1, &right.2)));
    results
}
