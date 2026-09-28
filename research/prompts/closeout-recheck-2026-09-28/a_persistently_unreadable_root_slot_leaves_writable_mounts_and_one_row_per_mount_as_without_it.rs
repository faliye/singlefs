//! C335（根槽持续读不出时实例表只增不减）的最小复现（调查员 2026-09-28，只在仓副本里）：两块单元区 384 槽的内存稀疏盘，
//! mkfs 同一个进程里取号、暖机、第一个文件之后，连着做 N 轮「可写挂载 → 覆盖写一次 → 正常卸载」，逐轮记挂载成没成、实例表行数、
//! 切换预留、准入判定、这一轮写的根落在哪些槽。根槽读失败由只供测试的开关 `FaultSchedule::every_read_of_named_root_ring_slots_fails`
//! 点名 (区域, 槽) 造，每一次读都失败；写失败那一臂另挂一层 `InjectedFault::WriteFails`，落点同一个槽。
//! 只钉今天的行为，不判它该是什么样。
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
) -> (Vec<CycleLine>, Probe) {
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
    (lines, probe)
}


/// 一轮里与坏槽无关的那几样：挂载、实例、写的行数、rows0、盘上行数、预留、片数、准入、覆盖写、卸载、写的根。
fn without_the_bad_slot_flag(line: &CycleLine) -> String {
    let mut copy = line.clone();
    copy.bad_slot_written = false;
    copy.render("")
}

fn control(cycles: usize) -> Vec<CycleLine> {
    let mut probe = start(HistoryDeviceWidth::UnitAreaOf384Slots);
    (1..=cycles)
        .map(|cycle| one_cycle(&mut probe, cycle, None))
        .collect()
}

/// 坏槽上住的是较旧的根（(区域 0, 槽 0) 开跑时住着 mkfs 的第 0 代根），读每一次都失败、写照样落盘：20 轮可写挂载全做成，
/// 每轮写一行（rows0 = 轮次），切换预留每块盘 104 槽、链 1 片，准入「取号之前就够」，覆盖写与卸载都做成；与不点名任何槽的对照臂逐轮相同。
/// 坏槽在第 4、8、12、16、20 轮被这一轮的根盖写过（txg 24、48、72、96、120 落在 (0,0)），之后那条根同样读不出，下一轮照常。
/// 判别力（钉的是路径跑到了）：读失败注入打中过；坏槽被轮到过。
#[test]
fn an_older_root_slot_unreadable_on_every_read_leaves_twenty_writable_mounts_as_in_the_control_arm_one_row_each() {
    let (lines, probe) = run(
        "older-read-fail-write-land",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::Fixed(RootRingSlot { region: 0, slot: 0 }),
        WritesToTheBadSlot::Land,
        20,
    );
    assert!(probe.read_plan.fired_count() > 0, "读失败注入打中过");
    for line in &lines {
        assert_eq!(
            (
                line.mount.as_str(),
                line.rows_written,
                line.rows0,
                line.table_rows_on_disk,
                line.reserve_slots_per_device,
                line.chain_pages,
                line.admission.as_str(),
                line.overwrite.as_str(),
                line.unmount.starts_with("Ok:F="),
            ),
            (
                "Ok",
                1,
                line.cycle as u64,
                Some(line.cycle),
                104,
                1,
                "取号之前就够",
                "Ok",
                true
            ),
            "第 {} 轮",
            line.cycle
        );
    }
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.bad_slot_written)
            .map(|line| line.cycle)
            .collect::<Vec<_>>(),
        vec![4, 8, 12, 16, 20],
        "坏槽被这几轮的根盖写过"
    );
    let control = control(20);
    assert_eq!(
        lines.iter().map(without_the_bad_slot_flag).collect::<Vec<_>>(),
        control.iter().map(without_the_bad_slot_flag).collect::<Vec<_>>(),
        "与对照臂逐轮相同"
    );
}

/// 坏槽上住的是较旧的根，读与写都失败（真坏扇区）：根槽写失败这一版是「冻结、原样重发」（同一个 txg、同一个槽），不推进 txg。
/// 20 轮里第 4 轮卸载那一串失败（txg 24 落 (0,0)），第 9、14、19 轮可写挂载自己那一串失败（txg 48、72、96），交回 `Publish`；
/// 那几轮只读挂载照常、择到的版本带着失败那次的 journal 记录重放到的 txg；下一轮可写挂载照常做成、第一条根的 txg 是失败那个加一
/// （失败那次的记录已落盘，`first_txg_of_new_instance` 取环里记录的最大 txg 加一）。每轮写一行：做成的轮 rows0 = 轮次。
#[test]
fn an_older_root_slot_whose_reads_and_writes_fail_fails_one_mount_in_five_and_the_next_mount_skips_the_failed_txg() {
    let (lines, probe) = run(
        "older-read-write-fail",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::Fixed(RootRingSlot { region: 0, slot: 0 }),
        WritesToTheBadSlot::Fail,
        20,
    );
    assert!(probe.write_plan.fired_count() > 0, "写失败注入打中过");
    let failed: Vec<(usize, String, String)> = lines
        .iter()
        .filter(|line| line.mount != "Ok" || !line.unmount.starts_with("Ok:F="))
        .map(|line| (line.cycle, line.mount.clone(), line.unmount.clone()))
        .collect();
    assert_eq!(
        failed,
        vec![
            (4, "Ok".into(), "Err:RaiseFloorSequencePublishFailed".into()),
            (9, "Err:Publish".into(), "-".into()),
            (14, "Err:Publish".into(), "-".into()),
            (19, "Err:Publish".into(), "-".into()),
        ]
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.mount.starts_with("Err"))
            .map(|line| line.read_only.clone())
            .collect::<Vec<_>>(),
        vec!["Ok:(10,48)", "Ok:(15,72)", "Ok:(20,96)"],
        "可写挂载失败那几轮只读挂载照常"
    );
    for line in lines.iter().filter(|line| line.mount == "Ok") {
        assert_eq!(
            (line.rows0, line.admission.as_str(), line.reserve_slots_per_device),
            (line.cycle as u64, "取号之前就够", 104),
            "第 {} 轮",
            line.cycle
        );
    }
    let first_txg_of = |cycle: usize| lines[cycle - 1].roots.first().map(|(txg, _)| *txg);
    assert_eq!(
        (first_txg_of(5), first_txg_of(10), first_txg_of(15), first_txg_of(20)),
        (Some(25), Some(49), Some(73), Some(97)),
        "失败之后那一轮跳过失败的 txg"
    );
}

/// 坏槽上住的是比下一次所选根新的根：第 2 轮卸载那一串的最后一条根（txg 13，(1,4)）从此每一次读都失败。它那次发布的末条记录读得出：
/// 第 3 轮择根落到 (3,12)，重放那条记录施加到 (3,13)，所选那一版就是 13，系统配置见证的计数器不比它新，读阶段第一遍判完、不重读，
/// 可写挂载照常。之后 12 轮全做成；第 6、10 轮这一轮的最后一条根又落在坏槽（txg 37、61），第 7、11 轮同样靠重放补回。
#[test]
fn a_newest_root_slot_unreadable_on_every_read_is_replayed_from_its_last_record_and_every_writable_mount_is_made() {
    let (lines, probe) = run(
        "newest-read-fail-write-land",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::NewestRootAfterCycle(2),
        WritesToTheBadSlot::Land,
        12,
    );
    assert!(probe.read_plan.fired_count() > 0, "读失败注入打中过");
    assert!(lines.iter().all(|line| line.mount == "Ok"), "12 轮全做成");
    let replayed: Vec<(usize, (u32, u64), (u32, u64), String)> = lines
        .iter()
        .filter(|line| line.chosen != line.effective)
        .map(|line| (line.cycle, line.chosen, line.effective, line.read_stage.clone()))
        .collect();
    assert_eq!(
        replayed,
        vec![
            (3, (3, 12), (3, 13), "OnTheFirstRead".into()),
            (7, (7, 36), (7, 37), "OnTheFirstRead".into()),
            (11, (11, 60), (11, 61), "OnTheFirstRead".into()),
        ],
        "最新根读不出的那几轮靠重放补回"
    );
    assert!(lines.iter().all(|line| line.rows0 == line.cycle as u64));
}

/// 边界臂（超出「一个根槽」的前提）：第 2 轮最后一条根（txg 13）的根槽与它那次发布的末条 journal 记录（两份）一起从此读不出。
/// 第 3 轮起每一次可写挂载都在取号之前被拒（`NewerStateStillUnreadableAfterOneReread`，C554 乙-配置续），只读挂载照常读回 (3,12)；
/// 拒的几轮一个字节都不写，行数停在 2，盘面第 3 轮与第 12 轮逐字节相同——没有可写挂载，这两处也不会被盖写。
#[test]
fn a_newest_root_slot_and_its_last_record_unreadable_refuse_every_writable_mount_from_the_first() {
    let mut probe = start(HistoryDeviceWidth::UnitAreaOf384Slots);
    one_cycle(&mut probe, 1, None);
    let second = one_cycle(&mut probe, 2, None);
    let (_, slot) = *second.roots.last().expect("第 2 轮写过根");
    assert_eq!((second.roots.last().unwrap().0, slot), (13, RootRingSlot { region: 1, slot: 4 }));
    arm(&probe, slot, WritesToTheBadSlot::Land);
    let counter = second.last_record_counter.expect("卸载那一串有末条记录");
    let offset = singlefs_core::journal::record_offset(
        counter,
        probe.width.parameters().geometry.journal_ring_bytes,
    )
    .0;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        probe
            .extra
            .borrow_mut()
            .push((device, offset, singlefs_format::JOURNAL_RECORD_BYTES));
    }
    let image_before = probe.pool.image();
    for cycle in 3..=12 {
        let line = one_cycle(&mut probe, cycle, Some(slot));
        assert_eq!(
            (line.mount.as_str(), line.read_only.as_str()),
            ("Err:NewerStateStillUnreadableAfterOneReread", "Ok:(3,12)"),
            "第 {cycle} 轮"
        );
    }
    assert!(probe.pool.image() == image_before, "拒的几轮一个字节都不写");
    let root = mount_read_only(&probe.pool.devices)
        .expect("只读挂载")
        .effective_root;
    assert_eq!(
        instance_table_chain_of_root(&probe.pool.devices, &root)
            .expect("实例表读得出")
            .records
            .rows
            .len(),
        2,
        "行数停在 2"
    );
}

/// 第 2 问：行数跨 369 一片（链从 1 片变 2 片）在第 367 轮（rows0 = 367，⌈(367 + 3) ÷ 369⌉ = 2），切换预留每块盘从 104 槽涨到 112 槽
/// （(N_switch + 1) × 2 = 8），准入仍「取号之前就够」，372 轮全做成。坏槽上住较旧的根、读失败写照落的那一臂。
#[test]
#[ignore = "harness 耗时用例：debug 下 372 轮约 80 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rows_cross_one_instance_table_page_at_the_367th_mount_and_the_reserve_grows_by_eight_slots_while_every_mount_is_admitted_before_acquisition() {
    let (lines, _) = run(
        "older-read-fail-write-land-372",
        HistoryDeviceWidth::UnitAreaOf384Slots,
        WhichSlot::Fixed(RootRingSlot { region: 0, slot: 0 }),
        WritesToTheBadSlot::Land,
        372,
    );
    for line in &lines {
        let two_pages = line.cycle >= 367;
        assert_eq!(
            (
                line.mount.as_str(),
                line.rows0,
                line.chain_pages,
                line.reserve_slots_per_device,
                line.admission.as_str()
            ),
            (
                "Ok",
                line.cycle as u64,
                if two_pages { 2 } else { 1 },
                if two_pages { 112 } else { 104 },
                "取号之前就够"
            ),
            "第 {} 轮",
            line.cycle
        );
    }
}
