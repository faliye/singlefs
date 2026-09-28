//! C335 探查（调查员草稿，只在仓副本里）：一个根槽持续读不出（可选：写也失败）时，连着做 N 轮「可写挂载 → 覆盖写一次 → 正常卸载」，
//! 逐轮打印挂载成没成、实例表行数、切换预留、准入判定、这一轮写的根落在哪些槽。不断言，只打印。
mod common_admission;

use common_admission::{DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::admission_reading_of_a_writable_mount;
use singlefs_core::mount::{unmount, MountSpaceAdmission, Unmounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, NamedRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

type Inner = FaultInjectingBlockDevice<SparseBlockDevice>;
type Outer = FaultInjectingBlockDevice<Inner>;

/// 外面再包一层：`extra` 里点名的几段（盘、起点、长度）按重叠判，读报块设备错（边界臂用来坏掉末条记录）。
pub struct Wrapped(
    pub Outer,
    pub DeviceIdentity,
    pub std::rc::Rc<std::cell::RefCell<Vec<(DeviceIdentity, u64, u64)>>>,
);

impl singlefs_core::block_device::BlockDevice for Wrapped {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let length = buffer.len() as u64;
        let hit = self.2.borrow().iter().any(|(device, start, span)| {
            *device == self.1 && offset.0 < start + span && *start < offset.0 + length
        });
        if hit {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("探查点名的这一段读坏"),
            ));
        }
        self.0.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.0.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.0.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.0.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.0.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.0.size_in_bytes()
    }
}

impl DeviceOverASparseImage for Wrapped {
    fn sparse_image(&self) -> &SparseDevice {
        &self.0.wrapped_device().wrapped_device().image
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WritesToTheBadSlot {
    Land,
    Fail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhichSlot {
    /// 固定点名一个槽（开跑时它住着较旧的根）。
    Fixed(RootRingSlot),
    /// 第 `arm_after_cycle` 轮卸载之后，点名最后一条根落的那个槽（它是比下一次所选根新的那一条）。
    NewestRootAfterCycle(usize),
    /// 同上，另把那条根那次发布的末条 journal 记录（两份）也坏掉（边界臂：超出「一个根槽」的前提）。
    NewestRootAndItsLastRecordAfterCycle(usize),
}

pub fn variant_name(debug: &str) -> String {
    debug
        .split(|c| c == '(' || c == '{' || c == ' ')
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn slot_text(slot: RootRingSlot) -> String {
    format!("({},{})", slot.region, slot.slot)
}

pub struct Probe {
    pub pool: PoolUnderTest<Wrapped>,
    pub read_plan: SharedFaultPlan,
    pub write_plan: SharedFaultPlan,
    pub width: HistoryDeviceWidth,
    pub extra: std::rc::Rc<std::cell::RefCell<Vec<(DeviceIdentity, u64, u64)>>>,
}

pub fn start(width: HistoryDeviceWidth) -> Probe {
    let read_plan = SharedFaultPlan::unarmed(width.fixed_geometry());
    let write_plan = SharedFaultPlan::unarmed(width.fixed_geometry());
    let (r, w) = (read_plan.clone(), write_plan.clone());
    let extra = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let extra_for_devices = extra.clone();
    let pool = PoolUnderTest::start_after_the_first_file(width, move |identity, device| {
        Wrapped(
            FaultInjectingBlockDevice::new(
                identity,
                FaultInjectingBlockDevice::new(identity, device, w.clone()),
                r.clone(),
            ),
            identity,
            extra_for_devices.clone(),
        )
    });
    Probe {
        pool,
        read_plan,
        write_plan,
        width,
        extra,
    }
}

pub fn arm(probe: &Probe, slot: RootRingSlot, writes: WritesToTheBadSlot) {
    let parameters = probe.width.parameters();
    let target = RootRingSlotTarget {
        named_slots: NamedRootRingSlots::NONE.with(slot),
        region_devices: parameters.region_devices,
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
    };
    probe
        .read_plan
        .arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(target));
    if writes == WritesToTheBadSlot::Fail {
        probe.write_plan.arm(FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::WithinNamedRootRingSlots(target),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        });
    }
}

/// 一轮的读数（一行）。
#[derive(Debug, Clone)]
pub struct CycleLine {
    pub cycle: usize,
    pub mount: String,
    pub instance: u32,
    pub rows_written: usize,
    pub rows0: u64,
    pub table_rows_on_disk: Option<usize>,
    pub reserve_slots_per_device: u64,
    pub chain_pages: u64,
    pub admission: String,
    pub overwrite: String,
    pub unmount: String,
    pub roots: Vec<(u64, RootRingSlot)>,
    pub bad_slot_written: bool,
    pub read_only: String,
    pub chosen: (u32, u64),
    pub effective: (u32, u64),
    pub read_stage: String,
    pub last_record_counter: Option<u64>,
}

impl CycleLine {
    pub fn render(&self, label: &str) -> String {
        let roots: Vec<String> = self
            .roots
            .iter()
            .map(|(txg, slot)| format!("{txg}@{}", slot_text(*slot)))
            .collect();
        format!(
            "name=c335-probe arm={label} cycle={} chosen={:?} effective={:?} read_stage={} last_record_counter={:?} mount={} instance={} rows_written={} rows0={} table_rows_on_disk={:?} reserve_slots_per_device={} chain_pages={} admission={} overwrite={} unmount={} bad_slot_written={} read_only={} roots=[{}]",
            self.cycle,
            self.chosen,
            self.effective,
            self.read_stage,
            self.last_record_counter,
            self.mount,
            self.instance,
            self.rows_written,
            self.rows0,
            self.table_rows_on_disk,
            self.reserve_slots_per_device,
            self.chain_pages,
            self.admission,
            self.overwrite,
            self.unmount,
            self.bad_slot_written,
            self.read_only,
            roots.join(",")
        )
    }
}

pub fn admission_name(admission: &MountSpaceAdmission) -> &'static str {
    match admission {
        MountSpaceAdmission::AdmittedBeforeAcquisition => "取号之前就够",
        MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => "推了再判够了",
        MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => "推满仍不够",
        MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "没判",
    }
}

/// 一轮：可写挂载 → 覆盖写一次 → 正常卸载；挂载被拒就试只读挂载。
pub fn one_cycle(probe: &mut Probe, cycle: usize, bad: Option<RootRingSlot>) -> CycleLine {
    let slots_per_region = probe.width.parameters().geometry.root_ring_slots_per_region;
    let at = |txg: u64| {
        (
            txg,
            target_for_publish(singlefs_core::address::CheckpointTxg(txg), slots_per_region),
        )
    };
    let mut line = CycleLine {
        cycle,
        mount: String::new(),
        instance: 0,
        rows_written: 0,
        rows0: 0,
        table_rows_on_disk: None,
        reserve_slots_per_device: 0,
        chain_pages: 0,
        admission: "-".into(),
        overwrite: "-".into(),
        unmount: "-".into(),
        roots: Vec::new(),
        bad_slot_written: false,
        read_only: "-".into(),
        chosen: (0, 0),
        effective: (0, 0),
        read_stage: "-".into(),
        last_record_counter: None,
    };
    let pool = &mut probe.pool;
    match pool.crash_and_mount_writable() {
        Err(error) => {
            line.mount = format!("Err:{}", variant_name(&format!("{error:?}")));
            line.read_only = match mount_read_only(&pool.devices) {
                Ok(read_only) => format!(
                    "Ok:({},{})",
                    read_only.effective_root.instance.0, read_only.effective_root.checkpoint_txg.0
                ),
                Err(failure) => format!("Err:{}", variant_name(&format!("{failure:?}"))),
            };
            return line;
        }
        Ok(output) => {
            line.mount = "Ok".into();
            line.chosen = (output.chosen_root.instance.0, output.chosen_root.checkpoint_txg.0);
            line.effective = (
                output.effective_root.instance.0,
                output.effective_root.checkpoint_txg.0,
            );
            line.read_stage = variant_name(&format!("{:?}", output.rereads.read_stage));
            line.instance = output.instance.0;
            line.rows_written = output.rows_written.len();
            line.admission = admission_name(&output.space_admission).into();
            line.roots.push(at(output.row_publish.root().checkpoint_txg.0));
            for version in &output.warm_up_publishes {
                line.roots.push(at(version.root().checkpoint_txg.0));
            }
            for raised in output.space_admission.floor_raises() {
                for publish in &raised.publishes {
                    line.roots.push(at(publish.root.checkpoint_txg.0));
                }
            }
        }
    }
    let session = pool.session.as_ref().expect("挂上了");
    line.rows0 = session.allocator.instance_rows_after_this_mounts_row_publish();
    line.chain_pages = (line.rows0 + 3).div_ceil(369).max(1);
    line.table_rows_on_disk = instance_table_chain_of_root(&pool.devices, session.current.root())
        .ok()
        .map(|chain| chain.records.rows.len());
    let reading =
        admission_reading_of_a_writable_mount(&session.allocator, session.current.file_version());
    line.reserve_slots_per_device = reading
        .per_device()
        .iter()
        .find(|terms| terms.device == DeviceIdentity(0))
        .map(|terms| terms.mount_time_commitment.0 / singlefs_format::SLOT_BYTES)
        .expect("盘 0");
    match pool.overwrite(OVERWRITE_BYTES) {
        Ok(published) => {
            line.overwrite = "Ok".into();
            for raised in &published.floor_raises {
                for publish in &raised.publishes {
                    line.roots.push(at(publish.root.checkpoint_txg.0));
                }
            }
            line.roots
                .push(at(pool.session().current.root().checkpoint_txg.0));
        }
        Err(refusal) => line.overwrite = format!("Err:{}", variant_name(&format!("{refusal:?}"))),
    }
    let mut session = pool.session.take().expect("会话");
    match unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    ) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            line.unmount = format!("Ok:F={}", raised.rollback_floor.0);
            line.last_record_counter = raised.publishes.last().map(|publish| publish.record.counter);
            for publish in &raised.publishes {
                line.roots.push(at(publish.root.checkpoint_txg.0));
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {
            line.unmount = "Ok:nothing".into()
        }
        Err(error) => line.unmount = format!("Err:{}", variant_name(&format!("{error:?}"))),
    }
    if let Some(bad) = bad {
        line.bad_slot_written = line.roots.iter().any(|(_, slot)| *slot == bad);
    }
    line
}

pub fn run(
    label: &str,
    width: HistoryDeviceWidth,
    which: WhichSlot,
    writes: WritesToTheBadSlot,
    cycles: usize,
) -> Vec<CycleLine> {
    let mut probe = start(width);
    let mut bad = None;
    if let WhichSlot::Fixed(slot) = which {
        arm(&probe, slot, writes);
        bad = Some(slot);
    }
    let mut lines = Vec::new();
    for cycle in 1..=cycles {
        let line = one_cycle(&mut probe, cycle, bad);
        eprintln!("{}", line.render(label));
        let (arm_after, with_record) = match which {
            WhichSlot::Fixed(_) => (0, false),
            WhichSlot::NewestRootAfterCycle(n) => (n, false),
            WhichSlot::NewestRootAndItsLastRecordAfterCycle(n) => (n, true),
        };
        if cycle == arm_after {
            let (_, slot) = *line.roots.last().expect("这一轮写过根");
            arm(&probe, slot, writes);
            bad = Some(slot);
            if with_record {
                let counter = line.last_record_counter.expect("卸载那一串有末条记录");
                let offset = singlefs_core::journal::record_offset(
                    counter,
                    probe.width.parameters().geometry.journal_ring_bytes,
                )
                .0;
                for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    probe.extra.borrow_mut().push((
                        device,
                        offset,
                        singlefs_format::JOURNAL_RECORD_BYTES,
                    ));
                }
            }
            eprintln!(
                "name=c335-probe arm={label} armed_after_cycle={cycle} bad_slot={} with_last_record={with_record} last_record_counter={:?}",
                slot_text(slot),
                line.last_record_counter
            );
        }
        lines.push(line);
    }
    lines
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_older_root_slot_reads_fail_writes_land() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    run(
        "older-read-fail-write-land",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::Fixed(RootRingSlot { region: 0, slot: 0 }),
        WritesToTheBadSlot::Land,
        cycles,
    );
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_older_root_slot_reads_and_writes_fail() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    run(
        "older-read-write-fail",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::Fixed(RootRingSlot { region: 0, slot: 0 }),
        WritesToTheBadSlot::Fail,
        cycles,
    );
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_newest_root_slot_goes_bad_after_cycle_two() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    run(
        "newest-read-fail-write-land",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::NewestRootAfterCycle(2),
        WritesToTheBadSlot::Land,
        cycles,
    );
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_no_bad_slot_control() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    run(
        "control-none",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::Fixed(RootRingSlot { region: 2, slot: 7 }),
        WritesToTheBadSlot::Land,
        0,
    );
    let mut probe = start(HistoryDeviceWidth::UnitAreaOf384Slots);
    for cycle in 1..=cycles {
        eprintln!("{}", one_cycle(&mut probe, cycle, None).render("control-none"));
    }
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_newest_root_slot_and_its_last_record_go_bad_after_cycle_two() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    run(
        "newest-and-record-read-fail",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::NewestRootAndItsLastRecordAfterCycle(2),
        WritesToTheBadSlot::Land,
        cycles,
    );
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_narrow_older_root_slot_reads_fail_writes_land() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    run(
        "narrow240-older-read-fail-write-land",
        HistoryDeviceWidth::UnitAreaOf240Slots,
        WhichSlot::Fixed(RootRingSlot { region: 0, slot: 0 }),
        WritesToTheBadSlot::Land,
        cycles,
    );
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_narrow_control() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let mut probe = start(HistoryDeviceWidth::UnitAreaOf240Slots);
    for cycle in 1..=cycles {
        eprintln!("{}", one_cycle(&mut probe, cycle, None).render("narrow240-control"));
    }
}

#[test]
#[ignore = "探查：手动跑"]
fn probe_newest_root_slot_reads_and_writes_fail_after_cycle_two() {
    let cycles = std::env::var("C335_CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    run(
        "newest-read-write-fail",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::NewestRootAfterCycle(2),
        WritesToTheBadSlot::Fail,
        cycles,
    );
}
