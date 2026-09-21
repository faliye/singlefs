//! c381-r2 云端攻方腿（Opus）探针。只在攻方仓副本上跑（core 打了 `c381-r2-arms-core.diff`）。
//! 与第一轮的探针的区别：故障模型多了撕裂写（半个落盘）、屏障报错、整块盘持续失败；前缀多了五种（根落别的区域、抬过 F、
//! 回退挂载之后、盘上已有被抛弃时间线）；臂只留甲（今天的代码，做对照）、丙、戊，戊的实例切换由驱动做（近似：进程内可写重开）。
#![allow(clippy::too_many_arguments, clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::Mutex;

use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{choose_root, choose_superblock, instance_table_of_root, walk_to_file};
use singlefs_core::transaction::{
    acquire_instance, c381_clear_stop, c381_get, c381_set_arm, c381_set_read_only, c381_with, publish_first_file,
    publish_overwrite, publish_version, warm_up, FirstFile, InstanceTablePlan, PoolWriter,
    PublishError, PublishPlan, TransactionOutput,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};

pub const IMAGE_BYTES: u64 = 4 << 30;
/// 探针写的固定落点：镜像末尾一个 512 字节的格子（不属于任何结构；「目标设备的固定落点」在条款里没有指定落在哪）。
pub const PROBE_OFFSET: u64 = IMAGE_BYTES - 4096;
/// D23（journal 的角色与格式） 已定项 14 的 N_switch。
pub const N_SWITCH: u32 = 3;
/// 撕裂的粒度：块设备只收整扇区的写（`SparseBlockDevice` 的断言），而根槽宽就等于判定宽度（D22（单元原子性怎么合成） 已定项 2），
/// 所以一次根槽写在这个装置上撕不开——半个 = 0 字节，退化成「报错、不落盘」。
pub const SECTOR_BYTES: usize = 512;
#[must_use]
pub fn torn_prefix_bytes(length: usize) -> usize {
    (length / 2) / SECTOR_BYTES * SECTOR_BYTES
}

/// 一次写上注入什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// 报错、不落盘（第一轮的瞬时错）。
    Transient,
    /// 写落了、设备照样报错（第一轮的 `LYING`）。
    Lying,
    /// 前半落盘、后半不落，然后报错（撕裂 + 报错）。
    TornError,
    /// 前半落盘、后半不落，返回成功（撕裂而设备不说：写不做原子性承诺的那一形）。
    TornSilent,
}

#[derive(Default)]
pub struct Control {
    pub index: Cell<u64>,
    pub barrier_index: Cell<u64>,
    pub faults: RefCell<BTreeMap<u64, Fault>>,
    pub barrier_faults: RefCell<BTreeSet<u64>>,
    pub crash_at: Cell<Option<u64>>,
    /// 断电那一次写把前半落进盘（断电撕裂）。
    pub crash_is_torn: Cell<bool>,
    /// 这块盘从这个写下标起一律失败（持续失败：探针写也写不进去）。
    pub dead_device_from: RefCell<BTreeMap<u32, u64>>,
    /// 最近一次报错的写落在哪块盘（戊的探针写要「目标设备」）。
    pub last_failed_device: Cell<Option<u32>>,
    pub probe_writes: Cell<u64>,
    /// 这块盘上这一段字节永远写不进去（坏扇区：设备整体还活着，探针写落在别处照样过）。
    pub bad_ranges: RefCell<Vec<(u32, u64, u64)>>,
    /// 每一次写的（盘, 偏移, 长度）：两遍法的第一遍用它取 C 写到哪几个落点。
    pub log: RefCell<Vec<(u32, u64, usize)>>,
}

impl Control {
    pub fn crashed(&self) -> bool {
        self.crash_at
            .get()
            .is_some_and(|crash| self.index.get() > crash)
    }
}

pub struct Device {
    pub inner: SparseBlockDevice,
    pub number: u32,
    pub control: Rc<Control>,
}

impl BlockDevice for Device {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let index = self.control.index.get();
        self.control.index.set(index + 1);
        self.control
            .log
            .borrow_mut()
            .push((self.number, offset.0, bytes.len()));
        let in_bad_range = self.control.bad_ranges.borrow().iter().any(|(device, start, end)| {
            *device == self.number && offset.0 < *end && offset.0 + bytes.len() as u64 > *start
        });
        if in_bad_range {
            self.control.last_failed_device.set(Some(self.number));
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "坏扇区：这一段永远写不进去",
            )));
        }
        let dead = self
            .control
            .dead_device_from
            .borrow()
            .get(&self.number)
            .is_some_and(|from| index >= *from);
        if let Some(crash) = self.control.crash_at.get() {
            if index >= crash {
                if index == crash && self.control.crash_is_torn.get() {
                    let half = torn_prefix_bytes(bytes.len());
                    if half > 0 {
                        let _ = self.inner.write_at(offset, &bytes[..half], durability);
                    }
                }
                self.control.last_failed_device.set(Some(self.number));
                return Err(BlockDeviceError::InputOutput(std::io::Error::other("断电")));
            }
        }
        if dead {
            self.control.last_failed_device.set(Some(self.number));
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "整块盘持续失败",
            )));
        }
        let fault = self.control.faults.borrow().get(&index).copied();
        match fault {
            None => self.inner.write_at(offset, bytes, durability),
            Some(Fault::Transient) => {
                self.control.last_failed_device.set(Some(self.number));
                Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "注入的瞬时写错",
                )))
            }
            Some(Fault::Lying) => {
                let _ = self.inner.write_at(offset, bytes, durability);
                self.control.last_failed_device.set(Some(self.number));
                Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "写已落盘、设备照样报错",
                )))
            }
            Some(Fault::TornError) => {
                let half = torn_prefix_bytes(bytes.len());
                if half > 0 {
                    let _ = self.inner.write_at(offset, &bytes[..half], durability);
                }
                self.control.last_failed_device.set(Some(self.number));
                Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "撕裂写 + 报错",
                )))
            }
            Some(Fault::TornSilent) => {
                let half = torn_prefix_bytes(bytes.len());
                if half == 0 {
                    return Ok(());
                }
                self.inner.write_at(offset, &bytes[..half], durability)
            }
        }
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let index = self.control.barrier_index.get();
        self.control.barrier_index.set(index + 1);
        if self.control.barrier_faults.borrow().contains(&index) {
            self.control.last_failed_device.set(Some(self.number));
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "注入的屏障报错",
            )));
        }
        if self.control.crashed() {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("断电")));
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

pub fn devices_from(
    images: &[singlefs_harness::crash::SparseDevice],
    control: &Rc<Control>,
) -> Vec<(DeviceIdentity, Device)> {
    images
        .iter()
        .enumerate()
        .map(|(number, image)| {
            let mut inner = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            inner.image = image.clone();
            let number = u32::try_from(number).expect("盘号");
            (
                DeviceIdentity(number),
                Device {
                    inner,
                    number,
                    control: control.clone(),
                },
            )
        })
        .collect()
}

pub fn images_of(devices: &[(DeviceIdentity, Device)]) -> Vec<singlefs_harness::crash::SparseDevice> {
    devices
        .iter()
        .map(|(_, device)| device.inner.image.clone())
        .collect()
}

pub fn memory_pool(devices: &[(DeviceIdentity, Device)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect::<BTreeMap<_, _>>(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

pub fn violated(devices: &[(DeviceIdentity, Device)]) -> Vec<String> {
    check_pool_image(&memory_pool(devices))
        .iter()
        .filter(|(_, verdict)| format!("{verdict:?}").starts_with("Violated"))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect()
}

/// 从盘上读现行那一版的文件内容（择根 → 走到文件）。
pub fn file_content(devices: &Vec<(DeviceIdentity, Device)>) -> Option<Vec<u8>> {
    let superblock = choose_superblock(devices).ok()?;
    let root = choose_root(devices, &superblock)?;
    let mut fallbacks = 0usize;
    walk_to_file(devices, &root, &mut fallbacks).ok().flatten()
}

/// 一段前缀：盘上的镜像、内存里的分配器、现行那一版与当前实例代号。
#[derive(Clone)]
pub struct Snapshot {
    pub label: &'static str,
    pub images: Vec<singlefs_harness::crash::SparseDevice>,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub instance: InstanceGeneration,
    pub content: Vec<u8>,
    pub note: String,
}

pub fn empty_plan(previous: &TransactionOutput, instance: InstanceGeneration) -> PublishPlan<'static> {
    PublishPlan {
        txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
        counter: previous.record.counter + 1,
        transaction: 0,
        instance,
        back_chain: back_chain_of(&previous.record_bytes),
        file: None,
        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    }
}

pub const B_CONTENT: [u8; 4100] = [7u8; 4100];
pub const C_CONTENT: [u8; 3700] = [9u8; 3700];
pub const OTHER_CONTENT: [u8; 2000] = [3u8; 2000];

/// 前缀一：mkfs → 取号 → 暖机 → 第一个事务（txg 3）→ 覆盖写 B（txg 4）。与第一轮同一个起点。
#[must_use]
pub fn base_snapshot() -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let empty = vec![singlefs_harness::crash::SparseDevice::default(); 2];
    let mut devices = devices_from(&empty, &control);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let b = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            FirstFile {
                content: &first_file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务");
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile {
                content: &B_CONTENT,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("B")
    };
    Snapshot {
        label: "base",
        images: images_of(&devices),
        allocator,
        current: b,
        instance: InstanceGeneration(1),
        content: B_CONTENT.to_vec(),
        note: "A(txg3) → B(txg4)，实例 1；C 是 txg 5（根落区域 2）".to_string(),
    }
}

/// 在一段前缀之后再发 `count` 次空发布：把 C 的 txg 推到别的区域。
#[must_use]
pub fn plus_empty_publishes(base: &Snapshot, count: u64, label: &'static str) -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let mut devices = devices_from(&base.images, &control);
    let mut allocator = base.allocator.clone();
    let mut current = base.current.clone();
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        for _ in 0..count {
            current = publish_version(
                &mut writer,
                &mut allocator,
                empty_plan(&current, base.instance),
                Some(&current),
            )
            .expect("空发布");
        }
    }
    let next = current.root.checkpoint_txg.0 + 1;
    Snapshot {
        label,
        images: images_of(&devices),
        allocator,
        current,
        instance: base.instance,
        content: base.content.clone(),
        note: format!("base + {count} 次空发布；C 是 txg {next}（根落区域 {}）", next % 3),
    }
}

/// 在一段前缀之后再覆盖写 `count` 次（抬 F 要至少 4 个非空有效根）。
#[must_use]
pub fn plus_overwrites(base: &Snapshot, count: u64, label: &'static str) -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let mut devices = devices_from(&base.images, &control);
    let mut allocator = base.allocator.clone();
    let mut current = base.current.clone();
    let mut content = base.content.clone();
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        for step in 0..count {
            content = vec![u8::try_from(200 + step).expect("小于 256"); 1000 + usize::try_from(step).expect("小")];
            current = publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 300 + step,
                },
                base.instance,
            )
            .expect("覆盖写");
        }
    }
    let next = current.root.checkpoint_txg.0 + 1;
    Snapshot {
        label,
        images: images_of(&devices),
        allocator,
        current,
        instance: base.instance,
        content,
        note: format!("{} + {count} 次覆盖写；C 是 txg {next}（根落区域 {}）", base.label, next % 3),
    }
}

/// 进程内可写重开：取号 2、写行、暖机两次。之后的 C 落在新实例里。
#[must_use]
pub fn remounted(base: &Snapshot, label: &'static str) -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let mut devices = devices_from(&base.images, &control);
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let instance = mounted.output.instance;
    let allocator = mounted.allocator;
    let current = mounted.current.into_file_version().expect("现行那一版带文件");
    let next = current.root.checkpoint_txg.0 + 1;
    Snapshot {
        label,
        images: images_of(&devices),
        allocator,
        current,
        instance,
        content: base.content.clone(),
        note: format!(
            "{} + 进程内可写重开（实例 {}，写行 + 暖机）；C 是 txg {next}（根落区域 {}）",
            base.label,
            instance.0,
            next % 3
        ),
    }
}

/// 重开、再覆盖写一次，然后回退挂载到 (1, 3)：盘上留下被抛弃的时间线（实例 1 的 txg 4 起、实例 2 的全部根）。
#[must_use]
pub fn rolled_back(base: &Snapshot, label: &'static str) -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let mut devices = devices_from(&base.images, &control);
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let current = mounted.current.into_file_version().expect("带文件");
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &OTHER_CONTENT,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 90,
            },
            instance,
        )
        .expect("回退之前再写一版");
    }
    let rolled = mount_rollback(
        &parameters,
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退到 (1, 3)");
    let instance = rolled.output.instance;
    let allocator = rolled.allocator;
    let current = rolled.current.into_file_version().expect("带文件");
    let next = current.root.checkpoint_txg.0 + 1;
    Snapshot {
        label,
        images: images_of(&devices),
        allocator,
        current,
        instance,
        // 回退到 A：盘上的文件内容回到第一个事务那一版。
        content: first_file_content(),
        note: format!(
            "重开 + 再写一版 + 回退挂载到 (1, 3)（实例 {}，盘上已有被抛弃的时间线）；C 是 txg {next}（根落区域 {}）",
            instance.0,
            next % 3
        ),
    }
}

/// 抬 F：从一段前缀出发，尽量抬到最高的可抬下界。抬不动时 `note` 里写明为什么。
#[must_use]
pub fn floor_raised(base: &Snapshot, label: &'static str) -> Snapshot {
    let parameters = e142_parameters(512, 512);
    let control = Rc::new(Control::default());
    let mut devices = devices_from(&base.images, &control);
    let mut allocator = base.allocator.clone();
    let mut current = base.current.clone();
    let mut note = String::new();
    let mut raised_to = None;
    for candidate in (1..=current.root.checkpoint_txg.0).rev() {
        let mut attempt = current.clone();
        let mut attempt_allocator = allocator.clone();
        let mut attempt_devices = devices_from(&images_of(&devices), &control);
        match raise_rollback_floor(
            &parameters,
            &mut attempt_devices,
            &mut attempt_allocator,
            &mut attempt,
            CheckpointTxg(candidate),
            ShadowLedger::On,
        ) {
            Ok(raised) => {
                note = format!(
                    "抬 F 到 {candidate}：{} 次空发布、回收 {} 个落点",
                    raised.publishes.len(),
                    raised.reclaimed.len()
                );
                devices = attempt_devices;
                allocator = attempt_allocator;
                current = attempt;
                raised_to = Some(candidate);
                break;
            }
            Err(error) => {
                note = format!("抬 F 到 {candidate} 被拒：{error:?}");
            }
        }
    }
    let next = current.root.checkpoint_txg.0 + 1;
    Snapshot {
        label,
        images: images_of(&devices),
        allocator,
        current,
        instance: base.instance,
        content: base.content.clone(),
        note: format!(
            "{}（F = {:?}）；C 是 txg {next}（根落区域 {}）",
            note,
            raised_to,
            next % 3
        ),
    }
}

/// C 失败之后「用户」的动作（戊的切换不由用户发起，由驱动按失败表走；这里的动作在切换之后照跑）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// 原样重发上一次失败的内容（从调用方手里的现行版）。
    Retry,
    /// 从现行版发一次空发布。
    Empty,
    /// 从现行版发一次内容不同的覆盖写。
    Other,
    /// 放下写入口、进程内可写重开。
    Remount,
}
pub const ACTIONS: [Action; 4] = [Action::Retry, Action::Empty, Action::Other, Action::Remount];

#[derive(Clone, Debug, Default)]
pub struct Spec {
    /// 第几次写（相对这一段历史开始，也就是 C 的第一次写）上注入什么；可以给多个下标（戊自己那几步上再失败用它）。
    pub faults: Vec<(u64, Fault)>,
    /// C 的第几道屏障（相对 C 开始的下标）报错。
    pub barrier_fault: Option<u64>,
    /// 哪块盘从相对下标几起一律失败。
    pub dead_device: Option<(u32, u64)>,
    /// 从相对下标几起断电。
    pub crash_at: Option<u64>,
    /// 断电那一次写把前半落进盘（断电撕裂）。
    pub crash_torn: bool,
    pub actions: Vec<Action>,
    /// 探针写打哪块盘：`false` = 报错的那一块（默认读法），`true` = 一律打盘 0。
    pub probe_to_device_zero: bool,
    /// 戊-B：切换的所选根取「被重发的那个在飞 checkpoint 所基于的根」（近似用回退挂载），而不是恢复择到的最新根。
    pub switch_selects_base_root: bool,
    /// 坏扇区：这几段字节在这块盘上永远写不进去。
    pub bad_ranges: Vec<(u32, u64, u64)>,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub steps: Vec<String>,
    pub checker_at_crash: Vec<String>,
    pub remount: String,
    pub checker_after_remount: Vec<String>,
    pub writes: u64,
    pub writes_after_c: u64,
    /// 戊做了几次切换、有没有转只读。
    pub switches: u32,
    pub read_only: bool,
    /// 调用方最后手里那一版的 txg（`None` = 没有成功的发布）。
    pub current_txg: Option<u64>,
    /// 调用方最后被确认（发布返回 Ok）的那份内容，与重开之后盘上读出来的内容是不是同一份。
    pub acknowledged_content_after_remount: &'static str,
    /// 这段历史结束时的实例代号（戊每切换一次烧一个）。
    pub final_instance: u32,
    /// 探针写发了几次。
    pub probe_writes: u64,
    /// 最后一次重开之后，最新根指着的那一版实例表里有几行（一片装 369 行，D18（块里携带什么信息） 已定项 11）。
    pub instance_rows: Option<usize>,
}

fn short(result: &Result<TransactionOutput, PublishError>) -> String {
    match result {
        Ok(output) => format!("ok(txg {})", output.root.checkpoint_txg.0),
        Err(PublishError::BlockDevice(_)) => "err(BlockDevice)".to_string(),
        Err(other) => format!(
            "err({})",
            format!("{other:?}")
                .split([' ', '{', '('])
                .next()
                .unwrap_or("?")
        ),
    }
}

/// 探针写：往目标盘的固定落点写 512 字节。写得进去 ⇒ 瞬时失败，写不进去 ⇒ 持续失败（D23（journal 的角色与格式） 已定项 14）。
fn probe_write(devices: &mut [(DeviceIdentity, Device)], target: u32, control: &Control) -> bool {
    control.probe_writes.set(control.probe_writes.get() + 1);
    let Some((_, device)) = devices.iter_mut().find(|(identity, _)| identity.0 == target) else {
        return false;
    };
    device
        .write_at(
            DeviceOffsetInBytes(PROBE_OFFSET),
            &[0xC3u8; 512],
            WriteDurability::ForceUnitAccess,
        )
        .is_ok()
}

/// 戊：失败之后照 D23（journal 的角色与格式） 已定项 14 的失败表走——先探针写，写得进去走实例切换（近似：进程内可写重开 +
/// 重发在飞 checkpoint），写不进去转只读到下次挂载；连续切换越过 N_switch = 3 也转只读。
fn failure_table_driver(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    control: &Rc<Control>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    instance: &mut InstanceGeneration,
    pending_content: &[u8],
    // 在飞 checkpoint 所基于的那一版（戊-B 的切换按条款取它的根当所选根）。
    based_on: &TransactionOutput,
    spec: &Spec,
    outcome: &mut Outcome,
) -> (bool, bool) {
    let mut alive = true;
    loop {
        let target = if spec.probe_to_device_zero {
            0
        } else {
            control.last_failed_device.get().unwrap_or(0)
        };
        let probe_ok = probe_write(devices, target, control);
        outcome.steps.push(format!(
            "探针写(盘 {target}) {}",
            if probe_ok { "过" } else { "不过" }
        ));
        if !probe_ok {
            c381_set_read_only();
            outcome.read_only = true;
            outcome.steps.push("持续失败 → 转只读到下次挂载".to_string());
            break;
        }
        if outcome.switches >= N_SWITCH {
            c381_set_read_only();
            outcome.read_only = true;
            outcome
                .steps
                .push("连续切换越过 N_switch = 3 → 转只读".to_string());
            break;
        }
        outcome.switches += 1;
        c381_clear_stop();
        let mounted = if spec.switch_selects_base_root {
            let target = RollbackTarget {
                instance: based_on.root.instance,
                checkpoint_txg: based_on.root.checkpoint_txg,
            };
            catch_unwind(AssertUnwindSafe(|| {
                mount_rollback(parameters, devices, target, ShadowLedger::On)
            }))
        } else {
            catch_unwind(AssertUnwindSafe(|| mount_writable(parameters, devices)))
        };
        match mounted {
            Ok(Ok(mounted)) => {
                *allocator = mounted.allocator;
                *instance = mounted.output.instance;
                let chosen = mounted.output.chosen_root.checkpoint_txg.0;
                match mounted.current.into_file_version() {
                    Some(version) => {
                        outcome.steps.push(format!(
                            "切换#{}（近似：进程内可写重开）ok(所选根 txg {chosen}，新实例 {}，现行 txg {})",
                            outcome.switches,
                            instance.0,
                            version.root.checkpoint_txg.0
                        ));
                        *current = version;
                    }
                    None => {
                        outcome
                            .steps
                            .push(format!("切换#{} 之后现行那一版没有文件", outcome.switches));
                        alive = false;
                        break;
                    }
                }
                // 重发在飞 checkpoint：今天一次 checkpoint 只有一个调用者，它的号 > W ⇒ 按新写序重做。
                let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                let redo = catch_unwind(AssertUnwindSafe(|| {
                    publish_overwrite(
                        &mut writer,
                        allocator,
                        current,
                        FirstFile {
                            content: pending_content,
                            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
                        },
                        *instance,
                    )
                }));
                match redo {
                    Ok(Ok(version)) => {
                        outcome
                            .steps
                            .push(format!("重发在飞 checkpoint ok(txg {})", version.root.checkpoint_txg.0));
                        *current = version;
                        return (true, true);
                    }
                    Ok(Err(error)) => {
                        outcome
                            .steps
                            .push(format!("重发在飞 checkpoint {}", short(&Err(error))));
                        if c381_get().switch_pending {
                            continue;
                        }
                        alive = false;
                        break;
                    }
                    Err(_) => {
                        outcome.steps.push("重发 PANIC".to_string());
                        alive = false;
                        break;
                    }
                }
            }
            Ok(Err(error)) => {
                outcome.steps.push(format!(
                    "切换#{} 挂载失败({})",
                    outcome.switches,
                    format!("{error:?}").chars().take(60).collect::<String>()
                ));
                // 切换自己的写失败：再判一次（探针写先拦）。
                continue;
            }
            Err(_) => {
                outcome
                    .steps
                    .push(format!("切换#{} PANIC", outcome.switches));
                alive = false;
                break;
            }
        }
    }
    (false, alive)
}

/// 跑一段历史：前缀 → C（按 `spec` 注入故障）→ 臂自己的处置（戊由驱动走失败表）→ 用户动作 → 清故障、可写重开 → checker。
#[must_use]
pub fn run(parameters: &MakeFilesystemParameters, snapshot: &Snapshot, arm: u8, spec: &Spec) -> Outcome {
    c381_set_arm(arm);
    let control = Rc::new(Control::default());
    for (index, fault) in &spec.faults {
        control.faults.borrow_mut().insert(*index, *fault);
    }
    if let Some(index) = spec.barrier_fault {
        control.barrier_faults.borrow_mut().insert(index);
    }
    control.bad_ranges.borrow_mut().extend(spec.bad_ranges.iter().copied());
    if let Some((device, from)) = spec.dead_device {
        control.dead_device_from.borrow_mut().insert(device, from);
    }
    control.crash_at.set(spec.crash_at);
    control.crash_is_torn.set(spec.crash_torn);
    let mut devices = devices_from(&snapshot.images, &control);
    let mut allocator = snapshot.allocator.clone();
    let mut current = snapshot.current.clone();
    let mut instance = snapshot.instance;
    let mut acknowledged = snapshot.content.clone();
    let mut outcome = Outcome::default();
    let mut alive = true;
    {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let result = catch_unwind(AssertUnwindSafe(|| {
            publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &C_CONTENT,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
                },
                instance,
            )
        }));
        match result {
            Ok(result) => {
                outcome.steps.push(format!("C {}", short(&result)));
                if let Ok(version) = result {
                    current = version;
                    acknowledged = C_CONTENT.to_vec();
                }
            }
            Err(_) => {
                outcome.steps.push("C PANIC".to_string());
                alive = false;
            }
        }
    }
    outcome.writes_after_c = control.index.get();
    if arm >= 5 && alive && c381_get().switch_pending {
        let (redone, still_alive) = failure_table_driver(
            parameters,
            &mut devices,
            &control,
            &mut allocator,
            &mut current,
            &mut instance,
            &C_CONTENT,
            &snapshot.current,
            spec,
            &mut outcome,
        );
        if redone {
            acknowledged = C_CONTENT.to_vec();
        }
        alive = still_alive && !outcome.read_only;
    }
    for action in &spec.actions {
        if control.crashed() || !alive {
            break;
        }
        match action {
            Action::Remount => {
                c381_clear_stop();
                c381_set_arm(arm);
                let mounted = catch_unwind(AssertUnwindSafe(|| mount_writable(parameters, &mut devices)));
                match mounted {
                    Ok(Ok(mounted)) => {
                        outcome.steps.push(format!(
                            "重开 ok(所选根 {})",
                            mounted.output.chosen_root.checkpoint_txg.0
                        ));
                        allocator = mounted.allocator;
                        instance = mounted.output.instance;
                        match mounted.current.into_file_version() {
                            Some(version) => current = version,
                            None => alive = false,
                        }
                    }
                    Ok(Err(error)) => {
                        outcome.steps.push(format!(
                            "重开 err({})",
                            format!("{error:?}").chars().take(60).collect::<String>()
                        ));
                        alive = false;
                    }
                    Err(_) => {
                        outcome.steps.push("重开 PANIC".to_string());
                        alive = false;
                    }
                }
            }
            Action::Retry | Action::Empty | Action::Other => {
                let content: Option<&[u8]> = match action {
                    Action::Retry => Some(&C_CONTENT),
                    Action::Other => Some(&OTHER_CONTENT),
                    _ => None,
                };
                let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                let result = catch_unwind(AssertUnwindSafe(|| match content {
                    Some(bytes) => publish_overwrite(
                        &mut writer,
                        &mut allocator,
                        &current,
                        FirstFile {
                            content: bytes,
                            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
                        },
                        instance,
                    ),
                    None => publish_version(
                        &mut writer,
                        &mut allocator,
                        empty_plan(&current, instance),
                        Some(&current),
                    ),
                }));
                match result {
                    Ok(result) => {
                        outcome.steps.push(format!("{action:?} {}", short(&result)));
                        if let Ok(version) = result {
                            if let Some(bytes) = content {
                                acknowledged = bytes.to_vec();
                            }
                            current = version;
                        }
                    }
                    Err(_) => {
                        outcome.steps.push(format!("{action:?} PANIC"));
                        alive = false;
                    }
                }
            }
        }
    }
    outcome.writes = control.index.get();
    outcome.final_instance = instance.0;
    outcome.probe_writes = control.probe_writes.get();
    outcome.current_txg = Some(current.root.checkpoint_txg.0);
    outcome.checker_at_crash = violated(&devices);
    control.faults.borrow_mut().clear();
    control.barrier_faults.borrow_mut().clear();
    control.dead_device_from.borrow_mut().clear();
    control.crash_at.set(None);
    // 最后这一次是「下次挂载」：丙的「重开」与戊的「转只读到下次挂载」都在这里解除。
    c381_clear_stop();
    c381_with(|state| state.read_only = false);
    let mounted = catch_unwind(AssertUnwindSafe(|| mount_writable(parameters, &mut devices)));
    outcome.remount = match mounted {
        Ok(Ok(mounted)) => format!("ok(chosen {})", mounted.output.chosen_root.checkpoint_txg.0),
        Ok(Err(error)) => format!(
            "REFUSED {}",
            format!("{error:?}").chars().take(120).collect::<String>()
        ),
        Err(_) => "PANIC".to_string(),
    };
    outcome.checker_after_remount = violated(&devices);
    if std::env::var("C381_DEBUG_CONTENT").is_ok() {
        let read = file_content(&devices);
        eprintln!(
            "debug: 已确认内容 len={} 首字节={:?}；盘上读出 len={:?} 首字节={:?}",
            acknowledged.len(),
            acknowledged.first(),
            read.as_ref().map(std::vec::Vec::len),
            read.as_ref().and_then(|bytes| bytes.first())
        );
    }
    outcome.instance_rows = (|| {
        let superblock = choose_superblock(&devices).ok()?;
        let root = choose_root(&devices, &superblock)?;
        Some(instance_table_of_root(&devices, &root)?.rows.len())
    })();
    outcome.acknowledged_content_after_remount = match file_content(&devices) {
        Some(bytes) if bytes == acknowledged => "同一份",
        // fsync 没返回的那一版出现在盘上是允许的（D16（发布语义） 已定项 7 只管 fsync 返回之后）。
        Some(bytes) if bytes == C_CONTENT.to_vec() => "没确认的 C 那一份",
        Some(bytes) if bytes == OTHER_CONTENT.to_vec() => "没确认的 Other 那一份",
        Some(_) => "另一份（已确认的写不在盘上）",
        None => "读不出",
    };
    c381_set_arm(0);
    outcome
}

/// 第一遍：跑一次无故障的 C，把它每一次写的（盘, 偏移, 长度）取出来。
#[must_use]
pub fn writes_of_c(parameters: &MakeFilesystemParameters, snapshot: &Snapshot) -> Vec<(u32, u64, usize)> {
    c381_set_arm(0);
    let control = Rc::new(Control::default());
    let mut devices = devices_from(&snapshot.images, &control);
    let mut allocator = snapshot.allocator.clone();
    {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &snapshot.current,
            FirstFile {
                content: &C_CONTENT,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            snapshot.instance,
        )
        .expect("无故障的 C");
    }
    let log = control.log.borrow().clone();
    log
}

pub fn arm_name(arm: u8) -> &'static str {
    match arm {
        0 => "甲",
        2 => "丙",
        5 => "戊-A",
        6 => "戊-B",
        other => Box::leak(format!("臂{other}").into_boxed_str()),
    }
}

/// 分类（跑前写死）：挂不上 / 盖写（I-2.1、I-4.8、I-7.2、I-7.4 任一红，或重开被拒）/ 丢写（已确认的那份内容重开之后不在盘上）/
/// 没确认的那一版（fsync 没返回的那一版出现在盘上，允许）/ 红（别的不变量红，实测只有 I-3.1）/ 转只读。都不沾的是干净。
#[must_use]
pub fn classify(outcome: &Outcome) -> String {
    let overwrite = |entries: &[String]| {
        entries.iter().any(|entry| {
            entry.starts_with("I-2.1")
                || entry.starts_with("I-4.8")
                || entry.starts_with("I-7.2")
                || entry.starts_with("I-7.4")
        })
    };
    let mut tags: Vec<&str> = Vec::new();
    if !outcome.remount.starts_with("ok") {
        tags.push("挂不上");
    }
    if overwrite(&outcome.checker_at_crash) || overwrite(&outcome.checker_after_remount) {
        tags.push("盖写");
    }
    match outcome.acknowledged_content_after_remount {
        "同一份" => {}
        "没确认的 C 那一份" | "没确认的 Other 那一份" => tags.push("没确认的那一版"),
        _ => tags.push("丢写"),
    }
    if !outcome.checker_at_crash.is_empty() || !outcome.checker_after_remount.is_empty() {
        tags.push("红");
    }
    if outcome.read_only {
        tags.push("转只读");
    }
    if tags.is_empty() {
        return "干净".to_string();
    }
    tags.join("+")
}

/// 并行跑一批：每个工件一行进度。
pub fn sweep<T: Send, F: Fn(&MakeFilesystemParameters, usize) -> T + Sync>(
    count: usize,
    threads: usize,
    body: F,
) -> Vec<(usize, T)> {
    let parameters = e142_parameters(512, 512);
    let results = Mutex::new(Vec::new());
    let next = Mutex::new(0usize);
    let done = std::sync::atomic::AtomicUsize::new(0);
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let item = {
                    let mut guard = next.lock().expect("锁");
                    let item = if *guard < count { Some(*guard) } else { None };
                    *guard += 1;
                    item
                };
                let Some(index) = item else { break };
                let value = body(&parameters, index);
                results.lock().expect("锁").push((index, value));
                let finished = done.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if finished % 25 == 0 || finished == count {
                    eprintln!("progress {finished}/{count} at {:?}", started.elapsed());
                }
            });
        }
    });
    let mut results = results.into_inner().expect("锁");
    results.sort_by_key(|(index, _)| *index);
    results
}
