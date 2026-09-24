//! m2-newq-r1 云端攻方腿的驱动第四版（接手腿，只在副本 repo4 里跑；第三版之上加 F11 与 N1 候选丁）：在第一版之上加 F8–F10、N2 拒绝臂、复用检测列。
//! 前缀与故障写死，用户动作（覆盖写 O、关闭重挂 M、抬 F 到上限 F）按长度 1–4 全枚举。
//! 结果一行一次跑，写进环境变量 `NEWQ_OUT` 指的文件。
#![allow(clippy::too_many_lines, dead_code)]

mod common;

use std::cell::RefCell;
use std::fmt::Write as _;
use std::io::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use common::{build_pool, file_content, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::allocator::{Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::bytes::ByteWriter;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::unit::seal_header_checksum;
use singlefs_core::mount::{mount_writable, raise_rollback_floor, MountError, ShadowLedger};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::release_check_model as rc;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};

const SLOT: u64 = 16384;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Persistence {
    Once,
    UntilWritten,
    /// 读写都失败、永远（坏介质：写不进去，写也报错）。
    Forever,
}

#[derive(Clone, Debug)]
struct ReadFault {
    device: DeviceIdentity,
    start: u64,
    end: u64,
    persistence: Persistence,
    active: bool,
}

#[derive(Default)]
struct FaultPlan {
    faults: Vec<ReadFault>,
    reads_refused: u64,
    writes_refused: u64,
}

struct Flaky {
    identity: DeviceIdentity,
    inner: Recorded,
    plan: Rc<RefCell<FaultPlan>>,
}

impl BlockDevice for Flaky {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let start = offset.0;
        let end = start + buffer.len() as u64;
        let mut plan = self.plan.borrow_mut();
        let mut refuse = false;
        for fault in &mut plan.faults {
            if fault.active && fault.device == self.identity && fault.start < end && start < fault.end {
                refuse = true;
                if fault.persistence == Persistence::Once {
                    fault.active = false;
                }
            }
        }
        if refuse {
            plan.reads_refused += 1;
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("injected read error")));
        }
        drop(plan);
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let start = offset.0;
        let end = start + bytes.len() as u64;
        if self.plan.borrow().faults.iter().any(|fault| fault.active && fault.persistence == Persistence::Forever
            && fault.device == self.identity && fault.start < end && start < fault.end) {
            self.plan.borrow_mut().writes_refused += 1;
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("injected write error")));
        }
        for fault in &mut self.plan.borrow_mut().faults {
            if fault.persistence == Persistence::UntilWritten && fault.device == self.identity && fault.start < end && start < fault.end {
                fault.active = false;
            }
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    /// F1 读错一次（两份镜像各一次），之后读得出。
    TransientReadErrorBothMirrors,
    /// F2 读错直到那一段被重写（潜在扇区错），两份镜像。
    PersistentReadErrorBothMirrors,
    /// F3 同上，只有盘 0 那份。
    PersistentReadErrorDeviceZero,
    /// F4 两份镜像的字节都坏了（读得出、校验和对不上）。
    RotBothMirrors,
    /// F5 只有盘 0 那份坏了。
    RotDeviceZero,
    /// F6 写者错（注入）：上一版映射里数据单元那条的槽号指到现行实例表的槽（活单元、跨度同为 2）。
    MappingPointsAtLiveInstanceTable,
    /// F7 写者错（注入）：指到上一版 inode 叶容器的槽（同一次发布也要换下它）。
    MappingPointsAtLeafContainerReleasedInTheSamePublish,
    /// F8 写者错（落盘）：注入那次发布把映射里数据单元那条的槽号写成现行实例表的槽，节点照常封、照常落盘。
    WriterBugOnDiskToInstanceTable,
    /// F9 坏介质：上一版数据单元那个槽两块盘都读写皆失败、永远。
    BadMediaReadAndWriteForever,
    /// F10 写者错（落盘）：池里建 240 个 inode 分成两片叶，注入那次发布把数据单元那条的槽号写成右边那片叶
    /// （覆盖写只重写最左那片，右边那片一直照抄、活着，跨度同为 2）。
    WriterBugOnDiskToCarriedLeaf,
    /// F11 写者错（落盘）：注入那次发布把数据单元那条的槽号写成一个没分配的槽（数据槽往后 1000 槽）。
    WriterBugOnDiskToUnallocatedSlot,
}

impl Fault {
    const ALL: [Fault; 11] = [
        Fault::TransientReadErrorBothMirrors,
        Fault::PersistentReadErrorBothMirrors,
        Fault::PersistentReadErrorDeviceZero,
        Fault::RotBothMirrors,
        Fault::RotDeviceZero,
        Fault::MappingPointsAtLiveInstanceTable,
        Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish,
        Fault::WriterBugOnDiskToInstanceTable,
        Fault::BadMediaReadAndWriteForever,
        Fault::WriterBugOnDiskToCarriedLeaf,
        Fault::WriterBugOnDiskToUnallocatedSlot,
    ];
    fn name(self) -> &'static str {
        match self {
            Fault::TransientReadErrorBothMirrors => "F1-transient-eio-both",
            Fault::PersistentReadErrorBothMirrors => "F2-latent-eio-both",
            Fault::PersistentReadErrorDeviceZero => "F3-latent-eio-dev0",
            Fault::RotBothMirrors => "F4-rot-both",
            Fault::RotDeviceZero => "F5-rot-dev0",
            Fault::MappingPointsAtLiveInstanceTable => "F6-map-bug-live-instance-table",
            Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => "F7-map-bug-leaf-same-publish",
            Fault::WriterBugOnDiskToInstanceTable => "F8-writer-bug-disk-instance-table",
            Fault::BadMediaReadAndWriteForever => "F9-bad-media-rw-forever",
            Fault::WriterBugOnDiskToCarriedLeaf => "F10-writer-bug-disk-carried-leaf",
            Fault::WriterBugOnDiskToUnallocatedSlot => "F11-writer-bug-disk-unallocated-slot",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Arm {
    name: &'static str,
    arms: rc::Arms,
    /// N3「落盘」一臂的模拟：盘上多一个隔离集合，重挂之后原样施回（改盘上格式，副本里只在驱动里记）。
    persisted: bool,
}

fn arm(name: &'static str, enabled: bool, read: rc::OnReadFailure, mirror: rc::MirrorRule, scope: rc::IsolationScope,
       mismatch: rc::OnMismatch, recompute: bool, persisted: bool) -> Arm {
    Arm { name, arms: rc::Arms { enabled, on_read_failure: read, mirror_rule: mirror, scope, on_mismatch: mismatch, recompute_at_mount: recompute, rebuild_tolerates_unreadable_data: false, rebuild_refuses_on_mapping_mismatch: false, reread_once_on_read_failure: false }, persisted }
}

fn all_arms() -> Vec<Arm> {
    use rc::IsolationScope::{EveryDevice as All, MismatchingDeviceOnly as Bad};
    use rc::MirrorRule::{AnyMirror as Any, EveryMirror as Every};
    use rc::OnMismatch::{KeepRecordAllocated as Keep, ReleaseRecordAndIsolate as Iso};
    use rc::OnReadFailure::{AsMismatch as A, FailPublish as B, FailureTable as C};
    vec![
        arm("off(today)", false, A, Any, All, Iso, false, false),
        arm("N1A-any-isoAll-mem", true, A, Any, All, Iso, false, false),
        arm("N1A-every-isoAll-mem", true, A, Every, All, Iso, false, false),
        arm("N1A-every-isoBad-mem", true, A, Every, Bad, Iso, false, false),
        arm("N1B-any", true, B, Any, All, Iso, false, false),
        arm("N1B-every", true, B, Every, All, Iso, false, false),
        arm("N1C-any", true, C, Any, All, Iso, false, false),
        arm("N1C-every", true, C, Every, All, Iso, false, false),
        arm("N3-iso-recompute", true, A, Any, All, Iso, true, false),
        arm("N3-iso-persisted", true, A, Any, All, Iso, false, true),
        arm("N3-keep-allocated", true, A, Any, All, Keep, false, false),
        {
            let mut same = arm("N2-same-verdict-rebuild-tolerates-data", true, A, Any, All, Iso, false, false);
            same.arms.rebuild_tolerates_unreadable_data = true;
            same
        },
        {
            let mut refuse = arm("N2-rebuild-checks-mapping-refuses", true, A, Any, All, Iso, false, false);
            refuse.arms.rebuild_refuses_on_mapping_mismatch = true;
            refuse
        },
        {
            let mut reread = arm("N1D-reread-then-mismatch-iso-mem", true, A, Any, All, Iso, false, false);
            reread.arms.reread_once_on_read_failure = true;
            reread
        },
        {
            let mut reread = arm("N1D-reread-then-mismatch-keep-allocated", true, A, Any, All, Keep, false, false);
            reread.arms.reread_once_on_read_failure = true;
            reread
        },
    ]
}

fn content_of(seed: usize) -> Vec<u8> {
    let length = 3000 + seed * 13 % 700;
    (0..length).map(|index| u8::try_from((index * 7 + seed) % 253).expect("<256")).collect()
}

struct Run {
    pool: BuiltPool,
    devices: Option<Vec<(DeviceIdentity, Flaky)>>,
    plan: Rc<RefCell<FaultPlan>>,
    allocator: PoolAllocator,
    current: TransactionOutput,
    last_content: Vec<u8>,
    writable: bool,
    panicked: bool,
    seed: usize,
    persisted_isolation: Vec<(Placement, Vec<DeviceIdentity>)>,
    arm: Arm,
    switches: u64,
    /// 故障目标：槽号区间与原主人（身份、槽、字节 CRC）；之后哪一步有别的单元落进这个区间就记一笔。
    target: Option<((u64, u64), (TransactionUnit, u64, u32))>,
}

fn wrap(devices: Vec<(DeviceIdentity, Recorded)>, plan: &Rc<RefCell<FaultPlan>>) -> Vec<(DeviceIdentity, Flaky)> {
    devices.into_iter().map(|(identity, inner)| (identity, Flaky { identity, inner, plan: plan.clone() })).collect()
}

fn short(debug: String) -> String {
    let head: String = debug.chars().take(70).collect();
    head.replace(['\t', '\n'], " ")
}

impl Run {
    fn start(tag: &str, arm: Arm, fault: Fault) -> Run {
        rc::set_arms(rc::Arms::OFF);
        rc::reset_counters();
        rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| *bug.borrow_mut() = None);
        rc::REBUILD_MAPPING_MISMATCH.with(|count| *count.borrow_mut() = 0);
        let mut pool = build_pool(tag);
        let plan = Rc::new(RefCell::new(FaultPlan::default()));
        let devices = wrap(pool.devices.take().expect("开着"), &plan);
        let allocator = pool.allocator.clone();
        let current = pool.output.clone();
        let mut run = Run { pool, devices: Some(devices), plan, allocator, current, last_content: file_content(),
            writable: true, panicked: false, seed: 1, persisted_isolation: Vec::new(), arm, switches: 0, target: None };
        // 前缀：M（取号 2、写行，现行一版的实例表被重写）、O（上一版 = 这一版带文件、经映射可释放）。
        let m = run.remount();
        assert_eq!(m, "ok", "前缀 M");
        rc::set_arms(arm.arms);
        let o = run.overwrite();
        assert_eq!(o, "ok", "前缀 O");
        if fault == Fault::WriterBugOnDiskToCarriedLeaf {
            // 前缀多两步：建 240 个 inode（第一片叶满 233 条在末尾分裂成两片），再覆盖写一次（只重写最左那片）。
            let previous = run.current.clone();
            let publish_parameters = parameters();
            let devices = run.devices.as_mut().expect("开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            run.current = publish_new_inodes(&mut writer, &mut run.allocator, &previous, 240, FIXED_WRITE_TIME_SECONDS + 30, previous.root.instance)
                .expect("建 240 个 inode");
            assert!(run.current.inode_leaf_containers.len() >= 2, "分裂成两片叶");
            let o = run.overwrite();
            assert_eq!(o, "ok", "前缀第二个 O");
        }
        rc::reset_counters();
        rc::REBUILD_MAPPING_MISMATCH.with(|count| *count.borrow_mut() = 0);
        run
    }

    fn note_target(&mut self, identity: TransactionUnit, slot: SlotNumber) {
        let owner = self.current.units.iter().find(|unit| unit.identity == identity && unit.slot == slot).expect("原主人在这一版里");
        self.target = Some(((slot.0, slot.0 + identity.span_slots()), (identity, slot.0, crc32_castagnoli(&owner.bytes))));
    }

    /// 这一版里有没有别的单元落进故障目标那个区间（原主人照抄不算）。
    fn foreign_unit_on_target(&self) -> bool {
        let Some(((start, end), owner)) = self.target else { return false };
        self.current.units.iter().any(|unit| {
            let unit_end = unit.slot.0 + unit.identity.span_slots();
            unit.slot.0 < end && start < unit_end && (unit.identity, unit.slot.0, crc32_castagnoli(&unit.bytes)) != owner
        })
    }

    fn overwrite(&mut self) -> String {
        if self.panicked { return "skip(panicked)".into(); }
        if !self.writable { return "refused(not-writable)".into(); }
        let mut attempts = 0;
        loop {
            self.seed += 1;
            let content = content_of(self.seed);
            let publish_parameters = parameters();
            let previous = self.current.clone();
            let devices = self.devices.as_mut().expect("开着");
            let allocator = &mut self.allocator;
            let result = catch_unwind(AssertUnwindSafe(|| {
                let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
                publish_overwrite(&mut writer, allocator, &previous,
                    FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, previous.root.instance)
            }));
            match result {
                Err(panic) => {
                    self.panicked = true;
                    let message = panic.downcast_ref::<String>().cloned()
                        .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).to_string())).unwrap_or_default();
                    return format!("PANIC({})", short(message));
                }
                Ok(Ok(output)) => {
                    self.current = output;
                    self.last_content = content;
                    return if attempts == 0 { "ok".into() } else { format!("ok(after-{attempts}-switch)") };
                }
                Ok(Err(PublishError::ReleaseCheckReadFailedGoesToFailureTable { device, slot, .. })) => {
                    // D23 已定项 14 失败表：探针写（本装置写不失败 ⇒ 写得进去），再对失败的落点只读复核。
                    let readable = {
                        let devices = self.devices.as_ref().expect("开着");
                        let (_, flaky) = devices.iter().find(|(id, _)| *id == device).expect("盘");
                        let mut buffer = vec![0u8; usize::try_from(SLOT).expect("x")];
                        flaky.read_at(slot.to_device_offset(), &mut buffer).is_ok()
                    };
                    if !readable {
                        self.writable = false;
                        return "failure-table:read-only-until-next-mount".into();
                    }
                    attempts += 1;
                    self.switches += 1;
                    if attempts > 3 {
                        self.writable = false;
                        return "failure-table:N_switch-exceeded-read-only".into();
                    }
                    // 切换（第一版没实现）按「同一挂载里重做这次发布」模拟。
                }
                Ok(Err(error)) => return format!("err:{}", short(format!("{error:?}"))),
            }
        }
    }

    fn remount(&mut self) -> String {
        if self.panicked { return "skip(panicked)".into(); }
        drop(self.devices.take());
        self.pool.devices = None;
        let recorded = self.pool.reopen_recorded();
        let mut devices = wrap(recorded, &self.plan);
        let result = mount_writable(&parameters(), &mut devices);
        self.devices = Some(devices);
        match result {
            Ok(mounted) => {
                self.allocator = mounted.allocator;
                self.current = mounted.current.into_file_version().expect("带文件");
                self.writable = true;
                if self.arm.persisted {
                    for (placement, devices) in &self.persisted_isolation {
                        for device in devices {
                            self.allocator.isolate_abandoned(*device, placement.slot, placement.span);
                        }
                    }
                }
                "ok".into()
            }
            Err(error) => {
                self.writable = false;
                format!("mount-err:{}", short(format!("{error:?}")))
            }
        }
    }

    fn raise_floor(&mut self) -> String {
        if self.panicked { return "skip(panicked)".into(); }
        if !self.writable { return "refused(not-writable)".into(); }
        let devices = self.devices.as_mut().expect("开着");
        let mut probe = self.current.clone();
        let ceiling = match raise_rollback_floor(&parameters(), devices, &mut self.allocator, &mut probe, CheckpointTxg(u64::MAX), ShadowLedger::On) {
            Err(MountError::RollbackFloorAboveCeiling { ceiling, .. }) => ceiling,
            Err(error) => return format!("err:{}", short(format!("{error:?}"))),
            Ok(_) => unreachable!("u64::MAX 不会过上限"),
        };
        if ceiling <= self.current.root.rollback_floor {
            return "noop".into();
        }
        let mut current = self.current.clone();
        let allocator = &mut self.allocator;
        let result = catch_unwind(AssertUnwindSafe(|| raise_rollback_floor(&parameters(), devices, allocator, &mut current, ceiling, ShadowLedger::On)));
        match result {
            Err(_) => { self.panicked = true; "PANIC(raise)".into() }
            Ok(Ok(raised)) => { self.current = current; format!("ok(F={},reclaimed={})", ceiling.0, raised.reclaimed.len()) }
            Ok(Err(error)) => { self.current = current; format!("err:{}", short(format!("{error:?}"))) }
        }
    }

    fn inject(&mut self, fault: Fault) {
        let data = self.current.data_pointer.locations;
        match fault {
            Fault::MappingPointsAtLiveInstanceTable | Fault::WriterBugOnDiskToInstanceTable => {
                let slot = self.current.root.instance_table.locations[0].slot;
                self.note_target(TransactionUnit::InstanceTable, slot);
            }
            Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => {
                let slot = self.current.inode_leaf_containers[0].pointer.locations[0].slot;
                self.note_target(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(0)), slot);
            }
            Fault::WriterBugOnDiskToCarriedLeaf => {
                let slot = self.current.inode_leaf_containers[1].pointer.locations[0].slot;
                self.note_target(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1)), slot);
            }
            _ => self.note_target(TransactionUnit::Data, data[0].slot),
        }
        if fault == Fault::WriterBugOnDiskToUnallocatedSlot {
            let target = SlotNumber(data[0].slot.0 + 1000);
            assert!(self.allocator.devices.iter().all(|device| device.is_free(target)), "目标槽没分配");
            rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| *bug.borrow_mut() = Some(target));
            let o = self.overwrite();
            assert_eq!(o, "ok", "注入那次发布");
            rc::reset_counters();
            return;
        }
        if matches!(fault, Fault::WriterBugOnDiskToInstanceTable | Fault::WriterBugOnDiskToCarriedLeaf) {
            let target = self.target.expect("刚登记").0 .0;
            rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| *bug.borrow_mut() = Some(SlotNumber(target)));
            let o = self.overwrite();
            assert_eq!(o, "ok", "注入那次发布");
            assert!(rc::WRITER_BUG_DATA_ENTRY_TO.with(|bug| bug.borrow().is_none()), "注入用掉了");
            rc::reset_counters();
            return;
        }
        if fault == Fault::BadMediaReadAndWriteForever {
            let (start, end) = (data[0].slot.0 * SLOT, (data[0].slot.0 + 2) * SLOT);
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                self.plan.borrow_mut().faults.push(ReadFault { device, start, end, persistence: Persistence::Forever, active: true });
            }
            return;
        }
        let slot_range = |slot: SlotNumber, span: u64| (slot.0 * SLOT, (slot.0 + span) * SLOT);
        let (start, end) = slot_range(data[0].slot, 2);
        let rot = |devices: &[DeviceIdentity], run: &mut Run| {
            for device in devices {
                let flaky = run.devices.as_mut().expect("开着").iter_mut().find(|(id, _)| id == device).expect("盘");
                let garbage = vec![0xA5u8; 4096];
                flaky.1.inner.write_at(DeviceOffsetInBytes(start + 8192), &garbage, WriteDurability::Plain).expect("写坏");
                flaky.1.inner.barrier().expect("屏障");
            }
        };
        match fault {
            Fault::TransientReadErrorBothMirrors | Fault::PersistentReadErrorBothMirrors | Fault::PersistentReadErrorDeviceZero => {
                let persistence = if fault == Fault::TransientReadErrorBothMirrors { Persistence::Once } else { Persistence::UntilWritten };
                let devices: Vec<DeviceIdentity> = if fault == Fault::PersistentReadErrorDeviceZero { vec![DeviceIdentity(0)] } else { vec![DeviceIdentity(0), DeviceIdentity(1)] };
                for device in devices {
                    self.plan.borrow_mut().faults.push(ReadFault { device, start, end, persistence, active: true });
                }
            }
            Fault::RotBothMirrors => rot(&[DeviceIdentity(0), DeviceIdentity(1)], self),
            Fault::RotDeviceZero => rot(&[DeviceIdentity(0)], self),
            Fault::WriterBugOnDiskToInstanceTable | Fault::BadMediaReadAndWriteForever | Fault::WriterBugOnDiskToCarriedLeaf
            | Fault::WriterBugOnDiskToUnallocatedSlot => unreachable!("上面处置过"),
            Fault::MappingPointsAtLiveInstanceTable | Fault::MappingPointsAtLeafContainerReleasedInTheSamePublish => {
                let target = if fault == Fault::MappingPointsAtLiveInstanceTable {
                    self.current.root.instance_table.locations[0].slot
                } else {
                    self.current.inode_leaf_containers[0].pointer.locations[0].slot
                };
                let mapping = self.current.units.iter_mut().find(|unit| unit.identity == TransactionUnit::MappingTree).expect("映射");
                // 找出载荷 CRC 的位置（[header_end - 10, +4) 小端，罩 [header_end, 末尾)），改完重封两道校验和：模拟写者自己算错了条目、却把节点封好了。
                let header_end = (60..600).find(|h| {
                    mapping.bytes[h - 10..h - 6] == crc32_castagnoli(&mapping.bytes[*h..]).to_le_bytes()
                }).expect("映射节点的载荷 CRC 位置");
                for location in data {
                    let encode = |entry: LocationEntry| { let mut writer = ByteWriter::new(14); entry.write_to(&mut writer); writer.into_bytes() };
                    let old = encode(location);
                    let new = encode(LocationEntry { slot: target, ..location });
                    let at = mapping.bytes.windows(14).position(|window| window == old.as_slice()).expect("映射节点里有数据单元那条位置项");
                    mapping.bytes[at..at + 14].copy_from_slice(&new);
                }
                let crc = crc32_castagnoli(&mapping.bytes[header_end..]).to_le_bytes();
                mapping.bytes[header_end - 10..header_end - 6].copy_from_slice(&crc);
                seal_header_checksum(&mut mapping.bytes, header_end);
                assert!(singlefs_core::unit::parse_index_node(&mapping.bytes).is_ok(), "重封之后节点解得开");
            }
        }
    }

    fn step(&mut self, op: char) -> String {
        match op {
            'O' => self.overwrite(),
            'M' => self.remount(),
            'F' => self.raise_floor(),
            _ => unreachable!(),
        }
    }
}

fn suffixes(max_len: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut frontier = vec![String::new()];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for prefix in &frontier {
            for op in ['O', 'M', 'F'] {
                let mut s = prefix.clone();
                s.push(op);
                next.push(s.clone());
                out.push(s);
            }
        }
        frontier = next;
    }
    out
}

fn one_run(fault: Fault, arm: Arm, suffix: &str, index: usize) -> String {
    let mut run = Run::start(&format!("newq{index}"), arm, fault);
    run.inject(fault);
    let mut outcomes = Vec::new();
    let mut reuse_steps = Vec::new();
    let mut isolated_by_check_total = 0u64;
    for (step_index, op) in suffix.chars().enumerate() {
        let mut outcome = run.step(op);
        if !run.panicked && run.foreign_unit_on_target() {
            reuse_steps.push(step_index);
            outcome.push_str("[reuse]");
        }
        // 「落盘」一臂：把这一步核出来隔离的记进模拟的盘上集合。
        if run.arm.persisted {
            rc::LOG.with(|log| {
                for event in log.borrow().iter().filter(|event| event.outcome == "released-and-isolated") {
                    let placement = Placement { slot: event.slot, span: 2 };
                    if !run.persisted_isolation.iter().any(|(p, _)| *p == placement) {
                        run.persisted_isolation.push((placement, vec![DeviceIdentity(0), DeviceIdentity(1)]));
                    }
                }
            });
        }
        outcomes.push(format!("{op}:{outcome}"));
    }
    let isolated_now: Vec<u64> = run.allocator.devices.iter().map(|d| d.isolated_slots()).collect();
    let free_minus_allocatable: Vec<i64> = run.allocator.devices.iter().map(|d| {
        let allocatable = (0..d.unit_area_slots()).filter(|i| d.is_free(SlotNumber(50176 + i))).count() as i64;
        d.free_slots() as i64 - allocatable
    }).collect();
    rc::ISOLATED_BY_CHECK.with(|isolated| isolated_by_check_total = isolated.borrow().iter().map(|(_, n)| n).sum());
    let events: Vec<String> = rc::LOG.with(|log| log.borrow().iter().filter(|e| e.outcome != "pass").map(|e| format!("{}@{}:{}", e.unit, e.slot.0, e.outcome)).collect());
    let (reads, read_bytes) = rc::READS.with(|r| *r.borrow());
    let at_mount = rc::ISOLATED_AT_MOUNT.with(|n| *n.borrow());
    let final_mount = if run.panicked { "skip".to_string() } else { run.remount() };
    let image = run.pool.memory_pool();
    let readback = match recover(&image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => if content == run.last_content { "same".to_string() } else { "DIFFERENT".to_string() },
        other => format!("fail:{}", short(format!("{other:?}"))),
    };
    let violated: Vec<&str> = check_pool_image(&image).into_iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| n).collect();
    let rebuild_mapping_mismatch = rc::REBUILD_MAPPING_MISMATCH.with(|count| *count.borrow());
    let writes_refused = run.plan.borrow().writes_refused;
    let mut line = String::new();
    write!(line, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}",
        fault.name(), arm.name, suffix, outcomes.join(" "), final_mount, readback, if violated.is_empty() { "-".to_string() } else { violated.join(",") },
        isolated_now, isolated_by_check_total, at_mount, free_minus_allocatable, reads, read_bytes, run.switches, run.plan.borrow().reads_refused,
        if events.is_empty() { "-".to_string() } else { events.join(";") }, reuse_steps, rebuild_mapping_mismatch, writes_refused).expect("写");
    line
}

#[test]
fn sweep() {
    let out = std::env::var("NEWQ_OUT").expect("NEWQ_OUT");
    let only_fault = std::env::var("NEWQ_FAULT").ok();
    let only_arm = std::env::var("NEWQ_ARM").ok();
    let max_len: usize = std::env::var("NEWQ_MAXLEN").ok().map_or(4, |s| s.parse().expect("数"));
    let workers: usize = std::env::var("NEWQ_WORKERS").ok().map_or(4, |s| s.parse().expect("数"));
    let mut jobs = Vec::new();
    for fault in Fault::ALL {
        if only_fault.as_deref().is_some_and(|f| !fault.name().starts_with(f)) { continue; }
        for arm in all_arms() {
            if only_arm.as_deref().is_some_and(|a| a != arm.name) { continue; }
            let chosen: Vec<String> = match std::env::var("NEWQ_SUFFIX") {
                Ok(list) => list.split(',').map(str::to_string).collect(),
                Err(_) => suffixes(max_len),
            };
            for suffix in chosen {
                jobs.push((fault, arm.name, suffix));
            }
        }
    }
    let total = jobs.len();
    let jobs = std::sync::Mutex::new(jobs.into_iter().enumerate().collect::<Vec<_>>());
    let file = std::sync::Mutex::new(std::fs::File::create(&out).expect("建输出"));
    writeln!(file.lock().unwrap(), "fault\tarm\tsuffix\tsteps\tfinal_mount\treadback\tviolated\tisolated_per_device\tisolated_by_check\tisolated_at_mount\tfree_minus_allocatable\tcheck_reads\tcheck_read_bytes\tswitches\treads_refused\tcheck_events\treuse_steps\trebuild_mapping_mismatch\twrites_refused").unwrap();
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let Some((index, (fault, arm_name, suffix))) = jobs.lock().unwrap().pop() else { break };
                let arm = all_arms().into_iter().find(|a| a.name == arm_name).unwrap();
                let line = one_run(fault, arm, &suffix, index);
                writeln!(file.lock().unwrap(), "{line}").unwrap();
            });
        }
    });
    eprintln!("runs={total}");
}
