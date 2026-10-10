//! 攻方原型（m3-batch10-code-r2 云端攻方，只在冻结副本的拷贝上跑；不是入库用例）。
//! 放进拷贝的 `crates/singlefs-harness/tests/` 跑：`cargo test -p singlefs-harness --test opus_r2_rr2_attack -- --nocapture --test-threads=1`。
//! 开关（拷贝的 mount.rs 上，`mount-switches.patch`）：SINGLEFS_OPUS_RR2_OFF 退回 rr2 之前的触发；SINGLEFS_OPUS_REREAD_REFUSES 造一个
//! 重读那一遍恒报 Q3 的错实现。输出每行以 `OPUS-` 开头。
//!
//! A：年轻的环上一次瞬时读错（整池第 n 次读报错或翻一位，n 扫遍一次不注错挂载的全部读）。
//! B：成熟的窄盘上第一遍一块盘全坏、重读那一遍按各种形状坏（重读比第一遍坏），再挂之后用户发 k 次改动（k 扫 0..=K），
//!    每一步核两块盘的池级 checker 与「环里每条自证有效根在丢掉一块盘之后还读得回挂载之前读得回的全部文件」。
//! C：Q3 判据复算为真的盘面在真挂载上交不交得出 Q3 那一拒；错拒 Q3 的实现在第二截、第三截各判成什么。

mod common_admission;

use common_admission::{
    checker_violations_on, plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    mount_writable_with_test_only_switches_and_deletion_switches, unmount, BeforeTheOneReread,
    MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover,
    JournalPolicy, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount,
    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, WritableMountRefusalVerdict,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;

fn rr2_label() -> &'static str {
    if std::env::var_os("SINGLEFS_OPUS_RR2_OFF").is_some() {
        "rr2_off"
    } else {
        "rr2_on"
    }
}

fn member(error: &MountError) -> String {
    let text = format!("{error:?}");
    let end = text
        .find(|c: char| c == '(' || c == '{' || c == ' ')
        .unwrap_or(text.len());
    text[..end].to_string()
}

fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 点名的槽在它自己的第几次读上读成什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotFault {
    Fails,
    Zeros,
    Garbage,
}

/// 整池第 n 次读上注入什么（A 用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GlobalFault {
    Fails,
    FlipsBitZeroOfByte(u64),
}

struct ReadsState {
    spacing: u64,
    slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
    reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
    slot_plan: BTreeMap<(RootRingSlot, u64), SlotFault>,
    slot_faults_fired: u64,
    global_reads: u64,
    global_plan: Option<(u64, GlobalFault)>,
    /// 整池那一次注入落在哪个根环槽、是那个槽第几次读（不落在根环里是 None）。
    global_fired: Option<Option<(RootRingSlot, u64)>>,
}

#[derive(Clone)]
struct SharedReads(Rc<RefCell<ReadsState>>);

impl SharedReads {
    fn on(image: &MemoryPool) -> Self {
        let system_configuration = choose_system_configuration(image).expect("系统配置");
        let immutable = system_configuration.immutable;
        let slots = every_root_ring_slot(&immutable.sizes)
            .into_iter()
            .map(|ring_slot| {
                (
                    ring_slot,
                    immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                    slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing).0,
                )
            })
            .collect();
        SharedReads(Rc::new(RefCell::new(ReadsState {
            spacing: u64::from(immutable.sizes.fixed_structure_slot_spacing),
            slots,
            reads_of_each_slot: BTreeMap::new(),
            slot_plan: BTreeMap::new(),
            slot_faults_fired: 0,
            global_reads: 0,
            global_plan: None,
            global_fired: None,
        })))
    }

    fn reads_of_each_slot(&self) -> BTreeMap<RootRingSlot, u64> {
        self.0.borrow().reads_of_each_slot.clone()
    }
}

enum ReadAction {
    Pass,
    Fail,
    Fill(u8, u64),
    Flip(u64),
}

struct FaultDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    reads: SharedReads,
}

impl FaultDevice {
    fn decide(&self, offset: DeviceOffsetInBytes) -> ReadAction {
        let mut state = self.reads.0.borrow_mut();
        state.global_reads += 1;
        let global_ordinal = state.global_reads;
        let spacing = state.spacing;
        let slot_hit = state
            .slots
            .iter()
            .find(|(_, device, start)| {
                *device == self.identity && offset.0 >= *start && offset.0 < *start + spacing
            })
            .map(|(ring_slot, _, start)| (*ring_slot, *start));
        let mut slot_ordinal = None;
        if let Some((ring_slot, _)) = slot_hit {
            let reads = state.reads_of_each_slot.entry(ring_slot).or_insert(0);
            *reads += 1;
            slot_ordinal = Some((ring_slot, *reads));
        }
        if let Some((n, fault)) = state.global_plan {
            if n == global_ordinal {
                state.global_fired = Some(slot_ordinal);
                return match fault {
                    GlobalFault::Fails => ReadAction::Fail,
                    GlobalFault::FlipsBitZeroOfByte(byte) => ReadAction::Flip(byte),
                };
            }
        }
        if let (Some((ring_slot, ordinal)), Some((_, start))) = (slot_ordinal, slot_hit) {
            if let Some(fault) = state.slot_plan.get(&(ring_slot, ordinal)).copied() {
                state.slot_faults_fired += 1;
                return match fault {
                    SlotFault::Fails => ReadAction::Fail,
                    SlotFault::Zeros => ReadAction::Fill(0, start),
                    SlotFault::Garbage => ReadAction::Fill(0xA5, start),
                };
            }
        }
        ReadAction::Pass
    }
}

impl BlockDevice for FaultDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        match self.decide(offset) {
            ReadAction::Pass => self.inner.read_at(offset, buffer),
            ReadAction::Fail => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "攻方原型注入的读错",
            ))),
            ReadAction::Fill(byte, slot_start) => {
                self.inner.read_at(offset, buffer)?;
                let spacing = self.reads.0.borrow().spacing;
                let first = usize::try_from(slot_start.saturating_sub(offset.0)).expect("偏移");
                let end = usize::try_from(
                    (slot_start + spacing - offset.0).min(u64::try_from(buffer.len()).expect("长")),
                )
                .expect("偏移");
                buffer[first..end].fill(byte);
                Ok(())
            }
            ReadAction::Flip(byte) => {
                self.inner.read_at(offset, buffer)?;
                if !buffer.is_empty() {
                    let index =
                        usize::try_from(byte % u64::try_from(buffer.len()).expect("长")).expect("下标");
                    buffer[index] ^= 1;
                }
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
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

/// 一次可写挂载（准入判着、影子账开），读经 `reads`；交回结局、挂载之后的盘面、每次调钩子那一刻每槽读过几次。
fn mount_through(
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    reads: &SharedReads,
) -> (Result<Mounted, MountError>, MemoryPool, Vec<BTreeMap<RootRingSlot, u64>>) {
    let mut devices: Vec<(DeviceIdentity, FaultDevice)> = plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultDevice {
                    identity,
                    inner: device,
                    reads: reads.clone(),
                },
            )
        })
        .collect();
    let seen = reads.clone();
    let mut at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>> = Vec::new();
    let mut note = || at_each_hook_call.push(seen.reads_of_each_slot());
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut note),
    );
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    (mounted, after, at_each_hook_call)
}

fn outcome_of(mounted: &Result<Mounted, MountError>) -> String {
    match mounted {
        Ok(mounted) => format!(
            "Ok({})",
            match mounted.output.rereads.root_ring_of_the_rebuilt_allocator {
                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead => "first",
                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread => "reread",
            }
        ),
        Err(error) => format!("Err({})", member(error)),
    }
}

// ---------------------------------------------------------------- A

fn fresh_pool() -> (MakeFilesystemParameters, MemoryPool) {
    let parameters = WIDTH.parameters();
    let device_bytes = WIDTH.device_bytes();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let image = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    (parameters, image)
}

/// `image` 上一次可写挂载在整池第 `first_failing_write` 次写起掉电（None 是做完）；交回之后的盘面与这次挂载发了几次写。
fn mount_losing_power_at(
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    first_failing_write: Option<u64>,
) -> (MemoryPool, u64, bool) {
    let plan = match first_failing_write {
        Some(write) => SharedFaultPlan::armed(
            WIDTH.fixed_geometry(),
            FaultSchedule {
                fault: InjectedFault::WriteFails,
                device: FaultDeviceSelector::EveryDevice,
                placement: FaultPlacement::AnyOffset,
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(write),
            },
        ),
        None => SharedFaultPlan::unarmed(WIDTH.fixed_geometry()),
    };
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(image)
            .into_iter()
            .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
            .collect();
    let mounted = singlefs_core::mount::mount_writable(parameters, &mut devices);
    let ok = mounted.is_ok();
    drop(mounted);
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    (after, plan.calls().writes, ok)
}

/// 年轻的环的几个起点：mkfs 之后；第一次可写挂载在第 w 次写起掉电（w 扫遍那次挂载的每一次写）；第一次可写挂载做完。
fn young_ring_starting_points() -> Vec<(String, MakeFilesystemParameters, MemoryPool)> {
    let (parameters, fresh) = fresh_pool();
    let (_, writes_of_the_first_mount, ok) = mount_losing_power_at(&parameters, &fresh, None);
    assert!(ok, "第一次可写挂载做得成");
    let mut points = vec![("mkfs".to_string(), parameters.clone(), fresh.clone())];
    let mut seen: Vec<MemoryPool> = vec![fresh.clone()];
    for write in 1..=writes_of_the_first_mount {
        let (after, _, _) = mount_losing_power_at(&parameters, &fresh, Some(write));
        if seen.iter().any(|image| *image == after) {
            continue;
        }
        seen.push(after.clone());
        points.push((format!("first-mount-loses-power-at-write-{write}"), parameters.clone(), after));
    }
    let (done, _, _) = mount_losing_power_at(&parameters, &fresh, None);
    points.push(("first-mount-done".to_string(), parameters.clone(), done));
    println!(
        "OPUS-A-POINTS writes_of_the_first_mount={writes_of_the_first_mount} distinct_points={}",
        points.len()
    );
    points
}

fn self_verified_roots_on_each_device(image: &MemoryPool) -> BTreeMap<u32, Vec<u64>> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = system_configuration.immutable;
    let mut out: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(
            image,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            out.entry(immutable.region_devices[usize::try_from(ring_slot.region).expect("区")].0)
                .or_default()
                .push(root.checkpoint_txg.0);
        }
    }
    out
}

#[test]
fn a_one_transient_read_fault_on_young_rings() {
    let label = rr2_label();
    let mut total_mounts = 0u64;
    let mut q3_total = 0u64;
    let filter = std::env::var("SINGLEFS_OPUS_POINT").unwrap_or_default();
    for (name, parameters, image) in young_ring_starting_points() {
        if !filter.is_empty() && name != filter {
            continue;
        }
        let baseline_reads = SharedReads::on(&image);
        let (baseline, baseline_after, hook_calls) = mount_through(&parameters, &image, &baseline_reads);
        let image_hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for (identity, device) in &baseline_after.devices {
                identity.0.hash(&mut hasher);
                device.written_sectors().hash(&mut hasher);
            }
            hasher.finish()
        };
        println!("OPUS-A-IMAGE {label} attribute_by_slot={} point={name} image_after_a_mount_without_faults={image_hash:016x}", std::env::var_os("SINGLEFS_OPUS_ATTRIBUTE_BY_SLOT").is_some());
        let read_count = baseline_reads.0.borrow().global_reads;
        let reads_per_slot: Vec<u64> = baseline_reads.reads_of_each_slot().values().copied().collect();
        println!(
            "OPUS-A-BASE {label} point={name} roots={:?} outcome={} hook_calls={} reads={read_count} reads_per_slot_min={:?} max={:?}",
            self_verified_roots_on_each_device(&image),
            outcome_of(&baseline),
            hook_calls.len(),
            reads_per_slot.iter().min(),
            reads_per_slot.iter().max()
        );
        for (fault_name, fault) in [("fails", GlobalFault::Fails), ("flip", GlobalFault::FlipsBitZeroOfByte(0))] {
            let mut tally: BTreeMap<String, u64> = BTreeMap::new();
            let mut q3_at: Vec<String> = Vec::new();
            let stride: u64 = std::env::var("SINGLEFS_OPUS_A_STRIDE").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
            for n in (1..=read_count).step_by(usize::try_from(stride).expect("步长")) {
                let reads = SharedReads::on(&image);
                reads.0.borrow_mut().global_plan = Some((n, fault));
                let (mounted, _, _) = mount_through(&parameters, &image, &reads);
                total_mounts += 1;
                let outcome = outcome_of(&mounted);
                if outcome.contains("NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread") {
                    q3_total += 1;
                    q3_at.push(format!("n={n}:{:?}", reads.0.borrow().global_fired));
                }
                *tally.entry(outcome).or_insert(0) += 1;
            }
            println!("OPUS-A {label} point={name} fault={fault_name} reads={read_count} tally={tally:?}");
            if !q3_at.is_empty() {
                println!("OPUS-A-Q3 {label} point={name} fault={fault_name} at={q3_at:?}");
            }
        }
    }
    println!("OPUS-A-SUM {label} mounts={total_mounts} q3_refusals={q3_total}");
}

// ---------------------------------------------------------------- B

fn plan_failing_every_write_from(first_failing_write: u64) -> SharedFaultPlan {
    SharedFaultPlan::armed(
        WIDTH.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
        },
    )
}

/// 造法照 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 的同名函数（抄造法）。
fn narrow_pool_whose_deletion_was_continued(
    unit_area_slots: u64,
    overwrites: u64,
    data_units_of_the_file: usize,
    batches_after_the_intent: u64,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            WIDTH,
            unit_area_slots,
        );
    for _ in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
        .expect("顺序写");
    {
        let PoolUnderTest {
            session,
            devices,
            parameters,
            write_time_seconds,
            ..
        } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut current = session.current.version_of_trees().expect("树").clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图");
        for _ in 0..batches_after_the_intent {
            current = publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                deletion_switches_with_batches_of_one().batch_size,
                session.instance,
            )
            .expect("一批");
        }
    }
    pool.session = None;
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("续做");
    let (_, session) = MountedSession::of_the_mount(mounted);
    pool.session = Some(session);
    pool
}

fn crashed_unmount_image(pool: &PoolUnderTest<SparseBlockDevice>, writes_landed: u64) -> MemoryPool {
    let image = pool.image();
    let plan = plan_failing_every_write_from(writes_landed + 1);
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&image)
            .into_iter()
            .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
            .collect();
    let session = pool.session();
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let unmounted = unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger);
    assert!(unmounted.is_err(), "卸载掉电");
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// B 的起点：(名字, 参数, 盘宽, 再挂的盘面)。
fn mature_starting_points() -> Vec<(String, MakeFilesystemParameters, u64, MemoryPool)> {
    let mut points = Vec::new();
    {
        let pool = narrow_pool_whose_deletion_was_continued(192, 9, 3, 1);
        points.push(("192:9:3:1-n8".to_string(), pool.parameters.clone(), pool.device_bytes, crashed_unmount_image(&pool, 8)));
    }
    {
        let pool = narrow_pool_whose_deletion_was_continued(216, 7, 5, 4);
        points.push(("216:7:5:4-now".to_string(), pool.parameters.clone(), pool.device_bytes, pool.image()));
    }
    for (slots, overwrites, units) in [(264u64, 6u64, 4usize), (192, 9, 3), (384, 20, 2)] {
        let mut pool =
            start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(WIDTH, slots);
        for _ in 0..overwrites {
            pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
        }
        pool.sequential_write((units - 1) * data_unit_payload_capacity() + 100).expect("顺序写");
        pool.session = None;
        points.push((format!("{slots}:{overwrites}:{units}-live-crash"), pool.parameters.clone(), pool.device_bytes, pool.image()));
    }
    points
}

/// 环里每条自证过的根：(槽, 根)。
fn ring_roots(image: &MemoryPool) -> Vec<(RootRingSlot, RootRecord)> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = system_configuration.immutable;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| match read_root_ring_slot(
            image,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
            RootRingSlotReading::Bad(_) => None,
        })
        .collect()
}

fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
    MemoryPool {
        devices: image
            .devices
            .iter()
            .filter(|(identity, _)| **identity != lost)
            .map(|(identity, device)| (*identity, device.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 在 `root` 上打开、整读每个活文件。
fn files_at(image: &MemoryPool, root: &RootRecord) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let mounted = open_pool_for_read(image, root).map_err(|failure| format!("{failure:?}"))?;
    let mut files = Vec::new();
    for record in mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted
            .open_file(image, InodeNumber(record.inode))
            .map_err(|failure| format!("open {}: {failure:?}", record.inode))?;
        let read = file
            .read_at(image, FileOffsetInBytes(0), record.size)
            .map_err(|failure| format!("read {}: {failure:?}", record.inode))?;
        files.push((record.inode, read.bytes));
    }
    Ok(files)
}

type Baseline = BTreeMap<(RootRingSlot, u32, u64, u32), Result<Vec<(u64, Vec<u8>)>, String>>;

/// 挂载之前：环里每条根、丢掉每一块盘之后读回什么。
fn baseline_of(image: &MemoryPool) -> Baseline {
    let mut baseline = BTreeMap::new();
    for (ring_slot, root) in ring_roots(image) {
        for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
            baseline.insert(
                (ring_slot, root.instance.0, root.checkpoint_txg.0, lost.0),
                files_at(&image_without(image, lost), &root),
            );
        }
    }
    baseline
}

/// 这一刻还在环里（同一槽、同一条根）、挂载之前丢盘读得回的根，现在丢同一块盘读回的不一样的：(txg, 丢哪块, F)。
fn broken_roots(image: &MemoryPool, baseline: &Baseline, floor: u64) -> Vec<(u64, u32, bool)> {
    let mut broken = Vec::new();
    for (ring_slot, root) in ring_roots(image) {
        for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
            let Some(before) = baseline.get(&(ring_slot, root.instance.0, root.checkpoint_txg.0, lost.0)) else {
                continue;
            };
            if before.is_err() {
                continue;
            }
            if files_at(&image_without(image, lost), &root) != *before {
                broken.push((root.checkpoint_txg.0, lost.0, root.checkpoint_txg.0 >= floor));
            }
        }
    }
    broken
}

fn device_of_slot(image: &MemoryPool, ring_slot: RootRingSlot) -> DeviceIdentity {
    let immutable = choose_system_configuration(image).expect("系统配置").immutable;
    immutable.region_devices[usize::try_from(ring_slot.region).expect("区")]
}

/// 重建那一遍是每槽第几次读：第 k 次读每个槽都报错时钩子恰调一次、调的那一刻每槽恰读 k 次、挂载报重读过。
fn rebuild_pass_ordinal(parameters: &MakeFilesystemParameters, image: &MemoryPool) -> Option<u64> {
    let slots: Vec<RootRingSlot> = every_root_ring_slot(&choose_system_configuration(image).expect("配置").immutable.sizes);
    for k in 1..=20u64 {
        let reads = SharedReads::on(image);
        // 整环都坏：rr2 前后都重读，两条臂认得出同一个 k。
        for ring_slot in &slots {
            reads.0.borrow_mut().slot_plan.insert((*ring_slot, k), SlotFault::Fails);
        }
        let (mounted, _, hooks) = mount_through(parameters, image, &reads);
        if hooks.len() == 1
            && hooks[0].len() == slots.len()
            && hooks[0].values().all(|reads| *reads == k)
            && matches!(&mounted, Ok(m) if m.output.rereads.root_ring_of_the_rebuilt_allocator == RootRingOfTheRebuiltAllocatorRead::OnTheOneReread)
        {
            return Some(k);
        }
    }
    None
}

/// 再挂之后用户发 `steps` 次改动（有活文件就覆盖写、没有就建一个 inode），每一步之后核：两块盘的池级 checker 违例、受保护根被破坏。
fn after_the_remount(
    parameters: &MakeFilesystemParameters,
    device_bytes: u64,
    mounted: Mounted,
    image_after: &MemoryPool,
    baseline: &Baseline,
    steps: u64,
) -> Vec<String> {
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    // 窄盘扣不起环窗口（起点就是关着准入造的）：再挂之后的会话同样关准入，落点是唯一的墙。
    session.allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
        parameters: parameters.clone(),
        device_bytes,
        devices: plain_devices_on(image_after),
        session: Some(session),
        write_time_seconds: 1_788_100_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut findings = Vec::new();
    let mut check = |pool: &PoolUnderTest<SparseBlockDevice>, step: u64| {
        let image = pool.image();
        let floor = pool.session().current.root().rollback_floor.0;
        let violations = checker_violations_on(&image);
        let broken = broken_roots(&image, baseline, floor);
        let protected: Vec<_> = broken.iter().filter(|(_, _, at_or_above_f)| *at_or_above_f).collect();
        if !violations.is_empty() || !protected.is_empty() {
            findings.push(format!("step={step} F={floor} violations={violations:?} broken_at_or_above_F={protected:?}"));
        }
        broken.len()
    };
    let mut unprotected_broken_last = check(&pool, 0);
    let mut use_inodes = false;
    for step in 1..=steps {
        let outcome = if use_inodes {
            pool.create_one_inode().map(|_| ())
        } else {
            match pool.overwrite(OVERWRITE_BYTES) {
                Err(singlefs_core::mounted_session::UserChangeRefused::NoFileVersionToChange { .. }) => {
                    use_inodes = true;
                    pool.create_one_inode().map(|_| ())
                }
                other => other.map(|_| ()),
            }
        };
        if let Err(refusal) = outcome {
            findings.push(format!("step={step} refused={}", format!("{refusal:?}").chars().take(80).collect::<String>()));
            break;
        }
        unprotected_broken_last = check(&pool, step);
    }
    findings.push(format!("broken_roots_any_F_at_last_step={unprotected_broken_last}"));
    findings
}

#[test]
fn b_reread_worse_than_the_first_pass_on_mature_rings() {
    let label = rr2_label();
    let steps: u64 = std::env::var("SINGLEFS_OPUS_B_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(16);
    let filter = std::env::var("SINGLEFS_OPUS_POINT").unwrap_or_default();
    for (name, parameters, device_bytes, image) in mature_starting_points() {
        if !filter.is_empty() && name != filter {
            continue;
        }
        let Some(k) = rebuild_pass_ordinal(&parameters, &image) else {
            println!("OPUS-B-SKIP {label} point={name} 认不出重建那一遍");
            continue;
        };
        let roots = ring_roots(&image);
        let slots: Vec<RootRingSlot> = every_root_ring_slot(&choose_system_configuration(&image).expect("配置").immutable.sizes);
        println!(
            "OPUS-B-POINT {label} point={name} rebuild_ordinal={k} slots={} roots={:?}",
            slots.len(),
            roots.iter().map(|(s, r)| (s.region, s.slot, r.checkpoint_txg.0)).collect::<Vec<_>>()
        );
        let baseline = baseline_of(&image);
        let mut arms: Vec<(String, BTreeMap<(RootRingSlot, u64), SlotFault>)> = vec![("nofault".to_string(), BTreeMap::new())];
        let only_d: Option<u32> = std::env::var("SINGLEFS_OPUS_B_D").ok().and_then(|v| v.parse().ok());
        if only_d.is_some_and(|d| d != 0) {
            arms.clear();
        }
        for d in [DeviceIdentity(0), DeviceIdentity(1)] {
            if only_d.is_some_and(|only| only != d.0) {
                continue;
            }
            let other = DeviceIdentity(1 - d.0);
            for first in [SlotFault::Fails, SlotFault::Zeros] {
                let first_pass: BTreeMap<(RootRingSlot, u64), SlotFault> = slots
                    .iter()
                    .filter(|s| device_of_slot(&image, **s) == d)
                    .map(|s| ((*s, k), first))
                    .collect();
                let with = |extra: Vec<(RootRingSlot, SlotFault)>| {
                    let mut plan = first_pass.clone();
                    for (s, f) in extra {
                        plan.insert((s, k + 1), f);
                    }
                    plan
                };
                let tag = format!("first=d{}:{first:?}", d.0);
                arms.push((format!("{tag} reread=none"), with(vec![])));
                arms.push((format!("{tag} reread=same"), with(slots.iter().filter(|s| device_of_slot(&image, **s) == d).map(|s| (*s, first)).collect())));
                arms.push((format!("{tag} reread=other-all"), with(slots.iter().filter(|s| device_of_slot(&image, **s) == other).map(|s| (*s, SlotFault::Fails)).collect())));
                arms.push((format!("{tag} reread=whole"), with(slots.iter().map(|s| (*s, SlotFault::Fails)).collect())));
                let newest_other = roots
                    .iter()
                    .filter(|(s, _)| device_of_slot(&image, *s) == other)
                    .max_by_key(|(_, r)| r.checkpoint_txg.0)
                    .map(|(s, _)| *s);
                arms.push((
                    format!("{tag} reread=other-but-newest"),
                    with(slots.iter().filter(|s| device_of_slot(&image, **s) == other && Some(**s) != newest_other).map(|s| (*s, SlotFault::Zeros)).collect()),
                ));
                for (s, _) in &roots {
                    for f in [SlotFault::Fails, SlotFault::Zeros] {
                        arms.push((format!("{tag} reread=one:{}/{}:{f:?}", s.region, s.slot), with(vec![(*s, f)])));
                    }
                }
            }
        }
        let mut tally: BTreeMap<String, u64> = BTreeMap::new();
        let shard: Option<(usize, usize)> = std::env::var("SINGLEFS_OPUS_B_SHARD").ok().and_then(|v| {
            let (i, n) = v.split_once('/')?;
            Some((i.parse().ok()?, n.parse().ok()?))
        });
        for (index, (arm, plan)) in arms.into_iter().enumerate() {
            if shard.is_some_and(|(i, n)| index % n != i) {
                continue;
            }
            let reads = SharedReads::on(&image);
            reads.0.borrow_mut().slot_plan = plan;
            let (mounted, after, hooks) = mount_through(&parameters, &image, &reads);
            let outcome = outcome_of(&mounted);
            *tally.entry(outcome.clone()).or_insert(0) += 1;
            let findings = match mounted {
                Ok(mounted) => after_the_remount(&parameters, device_bytes, mounted, &after, &baseline, steps),
                Err(_) => vec![format!("image_unchanged={}", after == image)],
            };
            println!("OPUS-B {label} point={name} arm=[{arm}] outcome={outcome} hooks={} findings={findings:?}", hooks.len());
        }
        println!("OPUS-B-TALLY {label} point={name} {tally:?}");
    }
}

// ---------------------------------------------------------------- C

fn zero_the_ring(image: &MemoryPool) -> MemoryPool {
    let immutable = choose_system_configuration(image).expect("配置").immutable;
    let mut zeroed = image.clone();
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区")];
        let offset = slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing);
        let sparse = zeroed.devices.get_mut(&device).expect("盘");
        sparse.zero_fill(offset, u64::from(immutable.sizes.fixed_structure_slot_spacing));
    }
    zeroed
}

/// 拿一个假的 Q3 拒去问判官：Q3 判据在这份盘面上复算为真吗（没注读故障、被拒那次不带故障层）。
fn q3_verdict_on(image: &MemoryPool) -> WritableMountRefusalVerdict {
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
        image,
        recover(image, JournalPolicy::Consult).effective_root,
        &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ReadFaultsInTheCrashState::NoneInjected,
        ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
    )
}

fn q3_criterion_true(verdict: &WritableMountRefusalVerdict) -> bool {
    matches!(verdict, WritableMountRefusalVerdict::Illegal { conditions_not_met, .. }
        if conditions_not_met.iter().any(|c| matches!(c, RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults)))
}

/// 第三截的取舍（照 `crash_injection.rs` 的 `reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults`）：
/// 只有「没注读故障时判据复算为真」那三条之一在列，才报红。
fn third_cut_reports_red(verdict: &WritableMountRefusalVerdict) -> bool {
    matches!(verdict, WritableMountRefusalVerdict::Illegal { conditions_not_met, .. }
        if conditions_not_met.iter().any(|c| matches!(c,
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
            | RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults
            | RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults)))
}

#[test]
fn c_q3_criterion_reachability() {
    let label = rr2_label();
    let mut images: Vec<(String, MakeFilesystemParameters, MemoryPool)> =
        if std::env::var_os("SINGLEFS_OPUS_REREAD_REFUSES").is_some() {
            // 错实现下第一次可写挂载就拒（新池都重读），年轻环那几个起点造不出来：只取 mkfs 那一个。
            let (parameters, image) = fresh_pool();
            vec![("mkfs".to_string(), parameters, image)]
        } else {
            young_ring_starting_points()
        };
    for (name, parameters, _, image) in mature_starting_points() {
        images.push((name, parameters, image));
    }
    let mut true_count = 0;
    for (name, parameters, image) in &images {
        let verdict = q3_verdict_on(image);
        let zeroed = zero_the_ring(image);
        let zeroed_verdict = q3_verdict_on(&zeroed);
        let mut devices = plain_devices_on(&zeroed);
        let zeroed_mount = singlefs_core::mount::mount_writable(parameters, &mut devices);
        if q3_criterion_true(&verdict) {
            true_count += 1;
        }
        println!(
            "OPUS-C1 {label} point={name} criterion_true_on_the_image={} | ring_zeroed: criterion_true={} real_writable_mount={}",
            q3_criterion_true(&verdict),
            q3_criterion_true(&zeroed_verdict),
            outcome_of(&zeroed_mount)
        );
    }
    println!("OPUS-C1-SUM {label} images={} criterion_true={true_count}", images.len());
    if std::env::var_os("SINGLEFS_OPUS_REREAD_REFUSES").is_some() {
        for (name, parameters, image) in &images {
            let reads = SharedReads::on(image);
            let (mounted, after, _) = mount_through(parameters, image, &reads);
            let Err(refusal) = &mounted else {
                println!("OPUS-C2 point={name} outcome={}", outcome_of(&mounted));
                continue;
            };
            let verdict = judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
                image,
                recover(image, JournalPolicy::Consult).effective_root,
                refusal,
                if after == *image { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed },
                ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
                ReadFaultsInTheCrashState::NoneInjected,
                ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
            );
            println!(
                "OPUS-C2 point={name} wrong_implementation_refusal={} second_cut_verdict={verdict:?} third_cut_reports_red={}",
                member(refusal),
                third_cut_reports_red(&verdict)
            );
        }
    }
}

// ---------------------------------------------------------------- D

/// rr2 那个用例文件的两个起点上，丢掉一块盘之后只读挂载走到的那一版有几个活文件（那条「整读每个活文件」断言罩着几个字节）。
#[test]
fn d_live_files_on_the_starting_points_of_the_rr2_cases() {
    for (name, _, _, image) in mature_starting_points() {
        for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
            let lone = image_without(&image, lost);
            let summary = match singlefs_core::mounted_read::mount_read_only(&lone) {
                Ok(read_only) => {
                    let root = read_only.effective_root;
                    let files = files_at(&lone, &root).map(|files| {
                        files.iter().map(|(inode, bytes)| (*inode, bytes.len())).collect::<Vec<_>>()
                    });
                    let all_inodes: Vec<(u64, u64, u64)> = read_only
                        .mounted
                        .inode_records()
                        .iter()
                        .map(|record| (record.inode, u64::from(record.nlink), record.size))
                        .collect();
                    format!("root_txg={} live_files={files:?} inode_records(inode,nlink,size)={all_inodes:?}", root.checkpoint_txg.0)
                }
                Err(failure) => format!("read_only_mount_failed={failure:?}"),
            };
            println!("OPUS-D point={name} lost={} {summary}", lost.0);
        }
    }
}
