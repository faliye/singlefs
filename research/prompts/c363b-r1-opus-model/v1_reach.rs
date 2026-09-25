//! c363b-r1 攻方腿（V1）：今天的代码上，固定点分配失败走不走得到。
//! 只在冻结副本的拷贝里跑（`/tmp/claude-1000/c363b-r1-opus/tree`），不改产品代码、不改 harness，只加这一个测试文件。
//! 自己的驱动直接调 core 的入口（mkfs 同一进程那条会话、覆盖写、顺序写、建 inode、抬 F、可写挂载、回退），
//! 每次发布之前拍一份分配器，发布被拒时按拒绝的单元分类，并在那份分配器上算：活数据、defer、隔离、扣住、各档还放得下几个落点、
//! D28 已定项 1 的式子（`admission.rs`，今天没有调用点）在「保留池 0、挂载期承诺量 0」时的余量。

use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::{
    instance_switch_reserve_on_one_device, AdmissionReading, BytesOnOneDevice,
    BytesSummedOverAllReplicas, MetadataBlocks, PoolWideCommitments,
};
use singlefs_core::allocator::{PlacementRefusal, PoolAllocator, UnitFootprint};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_sequential_write, warm_up, FirstFile, PoolVersion, PoolWriter, PublishError,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::scenario::{first_file_content, FIXED_WRITE_TIME_SECONDS};

type Dev = SparseBlockDevice;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    /// 覆盖写，一个数据单元（2000 + 种子 字节）。
    Ow,
    /// 顺序写 n 个满数据单元（文件变成 n 个单元：活数据 2n 槽）。
    Seq(usize),
    /// 建 n 个 inode（一次发布，不写用户数据：重写的全是固定点）。
    Inodes(u64),
    /// 抬 F：新 F = 现行 F + steps mod (txg − F + 3)（与 harness 的 `FloorTargetChoice` 同一个取法）。
    Raise(u64),
    /// 关掉会话、可写挂载。
    Mount,
    /// 关掉会话、回退到根环里 (txg, 实例) 从新到旧第 i 条（mod 条数）。
    Rollback(u64),
}

impl Op {
    fn name(self) -> String {
        match self {
            Op::Ow => "Ow".into(),
            Op::Seq(n) => format!("Seq{n}"),
            Op::Inodes(n) => format!("Ino{n}"),
            Op::Raise(s) => format!("Raise{s}"),
            Op::Mount => "Mount".into(),
            Op::Rollback(i) => format!("Rb{i}"),
        }
    }
    fn data_demand_slots(self) -> i128 {
        match self {
            Op::Ow => 2,
            Op::Seq(n) => 2 * i128::try_from(n).expect("小"),
            _ => 0,
        }
    }
    /// 这一步发布的固定点量取哪一类上一次做成的发布（`need` 表的键）。
    fn need_key(self) -> &'static str {
        match self {
            Op::Ow => "ow",
            Op::Seq(_) => "seq",
            Op::Inodes(_) => "ino",
            Op::Raise(_) => "empty",
            Op::Mount | Op::Rollback(_) => "mount",
        }
    }
}

struct Session {
    allocator: PoolAllocator,
    current: PoolVersion,
    instance: InstanceGeneration,
}

struct Pool {
    params: MakeFilesystemParameters,
    device_bytes: u64,
    devices: Vec<(DeviceIdentity, Dev)>,
    session: Option<Session>,
    /// 各类发布上一次做成时重写的固定点槽数（非数据单元的跨度之和）。
    need: BTreeMap<&'static str, u64>,
    mounts: u64,
}

fn fixed_point_slots(output: &TransactionOutput) -> u64 {
    output
        .rewritten
        .iter()
        .filter(|unit| !matches!(unit, TransactionUnit::Data(_)))
        .map(|unit| unit.span_slots())
        .sum()
}

fn fixed_point_slots_of_version(version: &PoolVersion) -> Option<u64> {
    match version {
        PoolVersion::WithFile(output) => Some(fixed_point_slots(output)),
        PoolVersion::WithoutFile(_) => None,
    }
}

impl Pool {
    fn start(unit_area_slots: u64) -> Self {
        let params = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
        let device_bytes = (UNIT_AREA_START_SLOT + unit_area_slots) * SLOT_BYTES;
        let mut devices: Vec<(DeviceIdentity, Dev)> = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
        let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&params, &devices, &genesis);
        let content = first_file_content();
        let (instance, output) = {
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let output = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
            (instance, output)
        };
        // 各类发布的固定点量在第一次做成之前取第一个文件那次发布的（同一张角色表去掉数据单元）。
        let first_need = fixed_point_slots(&output);
        let need: BTreeMap<&'static str, u64> =
            [("ow", first_need), ("seq", first_need), ("ino", first_need)].into_iter().collect();
        Pool {
            params,
            device_bytes,
            devices,
            session: Some(Session {
                allocator,
                current: PoolVersion::WithFile(output),
                instance,
            }),
            need,
            mounts: 0,
        }
    }

    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: self.device_bytes,
        }
    }
}

/// 一步的结局。
#[derive(Clone, Debug)]
enum Outcome {
    Applied,
    NotApplicable(&'static str),
    Refused {
        /// 成员名（嵌套的连里层）。
        member: String,
        /// 落点被拒时是哪个单元。
        unit: Option<TransactionUnit>,
        /// 是不是 `NoFreeSlotOnAnyDevice`。
        no_free_slot: bool,
        /// 被拒在「取号之前在分配器拷贝上逐次取落点」那一道（挂载自己的落点干跑，任何写之前）。
        at_mount_dry_run: bool,
    },
}

fn refusal_name(refusal: &PlacementRefusal) -> &'static str {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => "NoFreeSlotOnAnyDevice",
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. } => "SomeDevicesFull",
        PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. } => {
            "UserDataSlotsDiffer"
        }
        PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => "CommitGeneratedDiffer",
    }
}

fn classify_publish(prefix: &str, error: &PublishError) -> Outcome {
    match error {
        PublishError::PlacementRefused { unit, refusal } => Outcome::Refused {
            member: format!("{prefix}PlacementRefused({})", refusal_name(refusal)),
            unit: Some(*unit),
            no_free_slot: *refusal == PlacementRefusal::NoFreeSlotOnAnyDevice,
            at_mount_dry_run: false,
        },
        other => {
            let debug = format!("{other:?}");
            let head = debug.split([' ', '{', '(']).next().unwrap_or("?").to_string();
            Outcome::Refused {
                member: format!("{prefix}{head}"),
                unit: None,
                no_free_slot: false,
                at_mount_dry_run: false,
            }
        }
    }
}

fn classify_mount(prefix: &str, error: &MountError) -> Outcome {
    match error {
        MountError::Publish(failed) => classify_publish(&format!("{prefix}Publish/"), &failed.cause),
        MountError::RaiseFloorSequencePublishFailed(failed) => {
            classify_publish(&format!("{prefix}RaiseSeq/"), &failed.cause)
        }
        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            unit, refusal, ..
        } => Outcome::Refused {
            member: format!("{prefix}DryRunBeforeAcquisition({})", refusal_name(refusal)),
            unit: Some(*unit),
            no_free_slot: *refusal == PlacementRefusal::NoFreeSlotOnAnyDevice,
            at_mount_dry_run: true,
        },
        other => {
            let debug = format!("{other:?}");
            let head = debug.split([' ', '{', '(']).next().unwrap_or("?").to_string();
            Outcome::Refused {
                member: format!("{prefix}{head}"),
                unit: None,
                no_free_slot: false,
                at_mount_dry_run: false,
            }
        }
    }
}

fn content(length: usize, seed: u64) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index as u64 * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

impl Pool {
    /// 做一步；返回结局与这一步之前、之后的现行 txg。
    fn apply(&mut self, op: Op) -> (Outcome, u64, u64) {
        let txg_before = self
            .session
            .as_ref()
            .map_or(0, |session| session.current.root().checkpoint_txg.0);
        let outcome = self.apply_inner(op);
        let txg_after = self
            .session
            .as_ref()
            .map_or(0, |session| session.current.root().checkpoint_txg.0);
        (outcome, txg_before, txg_after)
    }

    fn apply_inner(&mut self, op: Op) -> Outcome {
        match op {
            Op::Mount | Op::Rollback(_) => {
                self.session = None;
                let mounted = match op {
                    Op::Mount => mount_writable(&self.params, &mut self.devices),
                    Op::Rollback(index) => {
                        let configuration =
                            choose_system_configuration(&self.devices).expect("系统配置");
                        let mut roots: Vec<(u64, u32)> = readable_roots(
                            &self.devices,
                            &configuration.immutable.region_devices,
                            &configuration.immutable.sizes,
                            &configuration.immutable.filesystem_identifier,
                        )
                        .into_iter()
                        .map(|root| (root.checkpoint_txg.0, root.instance.0))
                        .collect();
                        roots.sort_unstable_by(|left, right| right.cmp(left));
                        roots.dedup();
                        if roots.is_empty() {
                            return Outcome::NotApplicable("没有可读根");
                        }
                        let count = u64::try_from(roots.len()).expect("条数");
                        let (txg, instance) =
                            roots[usize::try_from(index % count).expect("小于条数")];
                        mount_rollback(
                            &self.params,
                            &mut self.devices,
                            RollbackTarget {
                                instance: InstanceGeneration(instance),
                                checkpoint_txg: CheckpointTxg(txg),
                            },
                            ShadowLedger::On,
                        )
                    }
                    _ => unreachable!(),
                };
                match mounted {
                    Ok(mounted) => {
                        self.mounts += 1;
                        if let Some(need) = mounted
                            .output
                            .warm_up_publishes
                            .last()
                            .and_then(fixed_point_slots_of_version)
                        {
                            self.need.insert("empty", need);
                        }
                        if let Some(need) = fixed_point_slots_of_version(&mounted.output.row_publish)
                        {
                            self.need.insert("mount", need);
                        }
                        self.session = Some(Session {
                            allocator: mounted.allocator,
                            current: mounted.current,
                            instance: mounted.output.instance,
                        });
                        Outcome::Applied
                    }
                    Err(error) => classify_mount("Mount/", &error),
                }
            }
            Op::Raise(steps) => {
                let Some(session) = self.session.as_mut() else {
                    return Outcome::NotApplicable("没有会话");
                };
                let PoolVersion::WithFile(current) = &mut session.current else {
                    return Outcome::NotApplicable("现行版本树表 0 条");
                };
                let floor = current.root.rollback_floor.0;
                let choices = current.root.checkpoint_txg.0.saturating_sub(floor) + 3;
                let new_floor = CheckpointTxg(floor + steps % choices);
                match raise_rollback_floor(
                    &self.params,
                    &mut self.devices,
                    &mut session.allocator,
                    current,
                    new_floor,
                    ShadowLedger::On,
                ) {
                    Ok(raised) => {
                        if let Some(last) = raised.publishes.last() {
                            self.need.insert("empty", fixed_point_slots(last));
                        }
                        Outcome::Applied
                    }
                    Err(error) => classify_mount("Raise/", &error),
                }
            }
            Op::Ow | Op::Seq(_) | Op::Inodes(_) => {
                let Some(session) = self.session.as_mut() else {
                    return Outcome::NotApplicable("没有会话");
                };
                let PoolVersion::WithFile(previous) = &session.current else {
                    return Outcome::NotApplicable("现行版本树表 0 条");
                };
                let previous = previous.clone();
                let seed = previous.root.checkpoint_txg.0;
                let mut writer = PoolWriter::new(&self.params, self.devices.as_mut_slice());
                let result = match op {
                    Op::Ow => {
                        let bytes = content(2000 + usize::try_from(seed % 997).expect("小"), seed);
                        publish_overwrite(
                            &mut writer,
                            &mut session.allocator,
                            &previous,
                            FirstFile {
                                content: &bytes,
                                write_time_seconds: FIXED_WRITE_TIME_SECONDS + seed,
                            },
                            session.instance,
                        )
                    }
                    Op::Seq(units) => {
                        let bytes = content(units * data_unit_payload_capacity(), seed);
                        publish_sequential_write(
                            &mut writer,
                            &mut session.allocator,
                            &previous,
                            FirstFile {
                                content: &bytes,
                                write_time_seconds: FIXED_WRITE_TIME_SECONDS + seed,
                            },
                            session.instance,
                        )
                    }
                    Op::Inodes(count) => publish_new_inodes(
                        &mut writer,
                        &mut session.allocator,
                        &previous,
                        count,
                        FIXED_WRITE_TIME_SECONDS + seed,
                        session.instance,
                    ),
                    _ => unreachable!(),
                };
                match result {
                    Ok(output) => {
                        self.need.insert(op.need_key(), fixed_point_slots(&output));
                        session.current = PoolVersion::WithFile(output);
                        Outcome::Applied
                    }
                    Err(error) => classify_publish("", &error),
                }
            }
        }
    }
}

/// 发布之前那一刻的分配器读数（盘 0；两盘同槽，另报两盘是否对称）。
struct Diagnosis {
    capacity: u64,
    allocated: u64,
    deferred: u64,
    isolated: u64,
    free_slots: u64,
    /// 逐槽 `is_free`（不被分配、隔离、扣住挡）。
    allocatable: u64,
    live_data: u64,
    floor: u64,
    /// 按盘上的根环现算环里最旧可读根（F 那一半不取），在分配器拷贝上按这道门槛回收，回收得到的落点数：不是 0 就是谓词放行了而实现没收回来（C518 那一类）。
    /// 门槛按可读根取、不按实例表判有效：带被抛弃根的历史里门槛只会偏低、这一列只会少报。
    missed_reclaim: usize,
    /// 在分配器拷贝上连着取，各档还取得到几个落点。
    user_data_placeable: u64,
    one_slot_placeable: u64,
    two_slot_placeable: u64,
    /// D28 已定项 1 的式子，保留池 0、挂载期承诺量 0、待删与已承诺预留取第一版的 0：两块盘里较小的可用（槽）减这一步的用户数据需求。
    slack_without_reserves: i128,
    devices_symmetric: bool,
}

fn placeable(allocator: &PoolAllocator, generation: CheckpointTxg, mut take: impl FnMut(&mut PoolAllocator, CheckpointTxg) -> bool) -> u64 {
    let mut copy = allocator.clone();
    let mut count = 0;
    while count < 100_000 && take(&mut copy, generation) {
        count += 1;
    }
    count
}

fn diagnose(allocator: &PoolAllocator, current: &PoolVersion, op: Op, predicate_floor: CheckpointTxg) -> Diagnosis {
    let device = &allocator.devices[0];
    let allocatable = (UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device.unit_area_slots())
        .filter(|slot| device.is_free(singlefs_core::address::SlotNumber(*slot)))
        .count();
    let live_data = match current {
        PoolVersion::WithFile(output) => output
            .units
            .iter()
            .filter(|unit| matches!(unit.identity, TransactionUnit::Data(_)))
            .map(|unit| unit.identity.span_slots())
            .sum(),
        PoolVersion::WithoutFile(_) => 0,
    };
    let missed_reclaim = allocator
        .clone()
        .reclaim_released_up_to(predicate_floor, singlefs_core::allocator::ReclaimedReuse::Immediately)
        .len();
    let generation = CheckpointTxg(current.root().checkpoint_txg.0 + 1);
    let reading = AdmissionReading::of_allocator(
        allocator,
        BytesOnOneDevice::ZERO,
        PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas::ZERO),
    );
    let smallest_available = reading
        .available_on_each_device()
        .iter()
        .map(|(_, available)| available.0)
        .min()
        .expect("两块盘");
    let slot_bytes = i128::from(SLOT_BYTES);
    let first = &allocator.devices[0];
    let devices_symmetric = allocator.devices.iter().all(|other| {
        other.allocated_slots() == first.allocated_slots()
            && other.deferred_slots() == first.deferred_slots()
            && other.isolated_slots() == first.isolated_slots()
            && other.free_slots() == first.free_slots()
    });
    Diagnosis {
        capacity: device.unit_area_slots(),
        allocated: device.allocated_slots(),
        deferred: device.deferred_slots(),
        isolated: device.isolated_slots(),
        free_slots: device.free_slots(),
        allocatable: u64::try_from(allocatable).expect("槽数"),
        live_data,
        floor: current.root().rollback_floor.0,
        missed_reclaim,
        user_data_placeable: placeable(allocator, generation, |copy, generation| {
            copy.try_allocate_user_data(generation).is_ok()
        }),
        one_slot_placeable: placeable(allocator, generation, |copy, generation| {
            copy.try_allocate_commit_generated(UnitFootprint::OneSlot, generation)
                .is_ok()
        }),
        two_slot_placeable: placeable(allocator, generation, |copy, generation| {
            copy.try_allocate_commit_generated(UnitFootprint::TwoSlotsAligned, generation)
                .is_ok()
        }),
        slack_without_reserves: smallest_available.div_euclid(slot_bytes) - op.data_demand_slots(),
        devices_symmetric,
    }
}

fn unit_tag(unit: Option<TransactionUnit>) -> String {
    unit.map_or("-".to_string(), TransactionUnit::tag)
}

/// 一段历史：逐步做，每个被拒的步一行（`R`），末尾一行汇总（`H`）。
fn run_history(class: &str, unit_area_slots: u64, parameters: &str, ops: &[Op], checker_budget: usize) -> Vec<String> {
    let mut pool = Pool::start(unit_area_slots);
    let mut lines = Vec::new();
    let trace = std::env::var("V1_TRACE").is_ok_and(|wanted| {
        wanted.split(';').any(|one| one == format!("{class},{unit_area_slots},{parameters}"))
    });
    let mut applied = 0usize;
    let mut refusals = 0usize;
    let mut no_free_after_admission = 0usize;
    let mut fixed_point_hits = 0usize;
    let mut dry_run_refusals = 0usize;
    let mut checker_runs = 0usize;
    let mut first_fixed_point_hit = "-".to_string();
    let mut last_applied = "-".to_string();
    let mut applied_steps: Vec<usize> = Vec::new();
    for (index, op) in ops.iter().enumerate() {
        let predicate_floor = {
            let configuration = choose_system_configuration(&pool.devices).expect("系统配置");
            let roots = readable_roots(
                &pool.devices,
                &configuration.immutable.region_devices,
                &configuration.immutable.sizes,
                &configuration.immutable.filesystem_identifier,
            );
            // F 那一半不取：带新 F 的根没落满每块盘时 F 还没生效，按最新根带的 F 取会把不该收的算成漏收；只取环里最旧可读根，这一列只会少报。
            CheckpointTxg(roots.iter().map(|root| root.checkpoint_txg.0).min().unwrap_or(0))
        };
        let before = pool.session.as_ref().map(|session| {
            (
                diagnose(&session.allocator, &session.current, *op, predicate_floor),
                session.current.root().checkpoint_txg.0,
            )
        });
        let need = pool.need.get(op.need_key()).copied();
        let empty_need = pool.need.get("empty").copied().unwrap_or(4);
        let mount_commitment = instance_switch_reserve_on_one_device(
            pool.mounts + 2,
            MetadataBlocks(empty_need),
        )
        .0 / SLOT_BYTES;
        let (outcome, txg_before, txg_after) = pool.apply(*op);
        if trace {
            let after = pool.session.as_ref().map_or("no-session".to_string(), |session| {
                let d = &session.allocator.devices[0];
                let allocatable = (UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + d.unit_area_slots())
                    .filter(|slot| d.is_free(singlefs_core::address::SlotNumber(*slot)))
                    .count();
                format!(
                    "allocated={} deferred={} isolated={} free_slots={} allocatable={} F={}",
                    d.allocated_slots(),
                    d.deferred_slots(),
                    d.isolated_slots(),
                    d.free_slots(),
                    allocatable,
                    session.current.root().rollback_floor.0
                )
            });
            let shown = match &outcome {
                Outcome::Applied => "Applied".to_string(),
                Outcome::NotApplicable(why) => format!("NotApplicable({why})"),
                Outcome::Refused { member, unit, .. } => format!("Refused({member}@{})", unit_tag(*unit)),
            };
            lines.push(format!(
                "S\t{class}\t{unit_area_slots}\t{parameters}\t{index}\t{}\t{shown}\t{txg_before}\t{txg_after}\t{after}",
                op.name()
            ));
        }
        match outcome {
            Outcome::Applied => {
                applied += 1;
                last_applied = format!("{index}");
                applied_steps.push(index);
            }
            Outcome::NotApplicable(_) => {}
            Outcome::Refused {
                member,
                unit,
                no_free_slot,
                at_mount_dry_run,
            } => {
                refusals += 1;
                if at_mount_dry_run {
                    dry_run_refusals += 1;
                }
                let is_data = matches!(unit, Some(TransactionUnit::Data(_)));
                let kind = if !no_free_slot {
                    "other"
                } else if at_mount_dry_run {
                    "dry-run"
                } else if is_data {
                    "data"
                } else {
                    "fixed-point"
                };
                if no_free_slot && !at_mount_dry_run {
                    no_free_after_admission += 1;
                }
                let mut tier = String::from("-");
                let mut violations = String::from("-");
                if kind == "fixed-point" {
                    fixed_point_hits += 1;
                    if let (Some((diagnosis, _)), Some(need)) = (&before, need) {
                        let need = i128::from(need);
                        let commitment = i128::from(mount_commitment);
                        tier = if diagnosis.slack_without_reserves >= need + commitment {
                            "T3".into()
                        } else if diagnosis.slack_without_reserves >= need {
                            "T2".into()
                        } else {
                            "T1".into()
                        };
                    }
                    if first_fixed_point_hit == "-" {
                        first_fixed_point_hit = format!("{index}");
                    }
                    if checker_runs < checker_budget {
                        checker_runs += 1;
                        let red: Vec<&str> = check_pool_image(&pool.image())
                            .into_iter()
                            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
                            .map(|(invariant, _)| invariant)
                            .collect();
                        violations = if red.is_empty() { "green".into() } else { red.join(",") };
                    }
                }
                let diagnosis = before.as_ref().map_or("-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-".to_string(), |(d, _)| {
                    format!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        d.capacity,
                        d.allocated,
                        d.deferred,
                        d.isolated,
                        d.free_slots,
                        d.allocatable,
                        d.live_data,
                        d.floor,
                        d.missed_reclaim,
                        d.user_data_placeable,
                        d.one_slot_placeable,
                        d.two_slot_placeable,
                        d.slack_without_reserves,
                        d.devices_symmetric
                    )
                });
                lines.push(format!(
                    "R\t{class}\t{unit_area_slots}\t{parameters}\t{index}\t{}\t{member}\t{}\t{kind}\t{txg_before}\t{txg_after}\t{diagnosis}\t{}\t{mount_commitment}\t{tier}\t{violations}",
                    op.name(),
                    unit_tag(unit),
                    need.map_or("-".to_string(), |need| need.to_string()),
                ));
            }
        }
    }
    lines.push(format!(
        "H\t{class}\t{unit_area_slots}\t{parameters}\t{}\t{applied}\t{refusals}\t{no_free_after_admission}\t{fixed_point_hits}\t{dry_run_refusals}\t{first_fixed_point_hit}\t{last_applied}\t{}",
        ops.len(),
        applied_steps.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
    ));
    lines
}

fn repeat(op: Op, times: usize) -> Vec<Op> {
    vec![op; times]
}

fn widths(low: u64, high: u64, step: u64) -> Vec<u64> {
    (low..=high).step_by(usize::try_from(step).expect("步长")).collect()
}

/// 各类历史：(类名, 宽度, 参数, 操作序列)。用户决定的步（覆盖写几次、抬到哪、退到哪、挂几次）放开扫，只固定前缀的形状。
fn histories() -> Vec<(String, u64, String, Vec<Op>)> {
    let step: u64 = std::env::var("V1_WIDTH_STEP").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let coarse = step.max(8);
    let classes: String = std::env::var("V1_CLASSES").unwrap_or_else(|_| "ABCDEFGHI".into());
    let mut all = Vec::new();
    // A：mkfs 同一个进程那条会话，一次挂载都没做，连着覆盖写。
    if classes.contains('A') {
        for width in widths(64, 480, step) {
            all.push(("A-mkfs-process-overwrite".into(), width, "K=72".into(), repeat(Op::Ow, 72)));
        }
    }
    // B：可写挂载一次之后连着覆盖写。
    if classes.contains('B') {
        for width in widths(64, 480, step) {
            let mut ops = vec![Op::Mount];
            ops.extend(repeat(Op::Ow, 72));
            all.push(("B-mounted-overwrite".into(), width, "K=72".into(), ops));
        }
    }
    // C：活数据真占满：顺序写 n 个满单元（活 2n 槽），之后只建 inode（不释放数据、每次只重写固定点）；有挂载与没挂载两种前缀。
    if classes.contains('C') {
        for width in widths(64, 480, coarse) {
            for units in [1usize, 2, 4, 8, 16, 24, 32, 48, 64, 80, 96, 112, 128, 140] {
                for mounted in [false, true] {
                    let mut ops = Vec::new();
                    if mounted {
                        ops.push(Op::Mount);
                    }
                    ops.push(Op::Seq(units));
                    ops.extend(repeat(Op::Inodes(1), 48));
                    all.push((
                        format!("C-live-data-then-inodes{}", if mounted { "-mounted" } else { "" }),
                        width,
                        format!("n={units}"),
                        ops,
                    ));
                }
            }
        }
    }
    // D：覆盖写 k 次之后抬 F（目标放开扫），再覆盖写 8 次。
    if classes.contains('D') {
        for width in widths(64, 480, coarse) {
            for mounted in [false, true] {
                for prefix in [4usize, 10, 20, 24, 26, 30, 40, 60] {
                    for steps in [0u64, 1, 3, 8, 20, 36, 1000] {
                        let mut ops = Vec::new();
                        if mounted {
                            ops.push(Op::Mount);
                        }
                        ops.extend(repeat(Op::Ow, prefix));
                        ops.push(Op::Raise(steps));
                        ops.extend(repeat(Op::Ow, 8));
                        all.push((
                            format!("D-raise{}", if mounted { "-mounted" } else { "" }),
                            width,
                            format!("k={prefix},s={steps}"),
                            ops,
                        ));
                    }
                }
            }
        }
    }
    // E：覆盖写 k 次之后回退到根环里第 i 新的根，再覆盖写 30 次。
    if classes.contains('E') {
        for width in widths(64, 480, coarse) {
            for prefix in [4usize, 10, 26, 40] {
                for index in [0u64, 1, 2, 3, 5, 8, 13, 23] {
                    let mut ops = repeat(Op::Ow, prefix);
                    ops.push(Op::Rollback(index));
                    ops.extend(repeat(Op::Ow, 30));
                    all.push(("E-rollback".into(), width, format!("k={prefix},i={index}"), ops));
                }
            }
        }
    }
    // F：反复顺序写 n 个单元（活 2n 槽，每次换下上一版的 n 个进 defer）。
    if classes.contains('F') {
        for width in widths(64, 480, coarse) {
            for units in [1usize, 2, 3, 4, 6, 8, 12, 16, 24, 32] {
                all.push(("F-repeated-sequential".into(), width, format!("n={units}"), repeat(Op::Seq(units), 40)));
            }
        }
    }
    // G：覆盖写 k 次、挂载，重复 4 轮，再覆盖写 30 次。
    if classes.contains('G') {
        for width in widths(64, 480, coarse) {
            for prefix in [1usize, 3, 10, 25] {
                let mut ops = Vec::new();
                for _ in 0..4 {
                    ops.extend(repeat(Op::Ow, prefix));
                    ops.push(Op::Mount);
                }
                ops.extend(repeat(Op::Ow, 30));
                all.push(("G-remount-cycles".into(), width, format!("k={prefix}"), ops));
            }
        }
    }
    // H：活数据占满之后抬 F（目标放开扫），再建 inode。
    if classes.contains('H') {
        for width in widths(64, 480, coarse) {
            for units in [8usize, 32, 64, 96, 128] {
                for prefix in [0usize, 10, 26] {
                    for steps in [0u64, 3, 20, 1000] {
                        let mut ops = vec![Op::Seq(units)];
                        ops.extend(repeat(Op::Inodes(1), prefix));
                        ops.push(Op::Raise(steps));
                        ops.extend(repeat(Op::Inodes(1), 20));
                        all.push((
                            "H-live-data-raise".into(),
                            width,
                            format!("n={units},k={prefix},s={steps}"),
                            ops,
                        ));
                    }
                }
            }
        }
    }
    // I：卡住之后由用户决定的一到两步放开扫（前缀固定：覆盖写 30 次或顺序写 32 个单元再建 26 次 inode，然后抬 F），末尾一次覆盖写。
    if classes.contains('I') {
        let mut candidates: Vec<Op> = vec![Op::Ow, Op::Inodes(1), Op::Seq(1), Op::Mount];
        candidates.extend([0u64, 1, 3, 8, 20, 1000].map(Op::Raise));
        candidates.extend([0u64, 1, 2, 3].map(Op::Rollback));
        let mut suffixes: Vec<Vec<Op>> = Vec::new();
        for first in &candidates {
            suffixes.push(vec![*first]);
            for second in &candidates {
                suffixes.push(vec![*first, *second]);
            }
        }
        for width in widths(64, 480, 16) {
            for prefix_kind in ["ow30", "seq32-ino26"] {
                for steps in [0u64, 1000] {
                    for suffix in &suffixes {
                        let mut ops = match prefix_kind {
                            "ow30" => repeat(Op::Ow, 30),
                            _ => {
                                let mut ops = vec![Op::Seq(32)];
                                ops.extend(repeat(Op::Inodes(1), 26));
                                ops
                            }
                        };
                        let raise_index = ops.len();
                        ops.push(Op::Raise(steps));
                        ops.extend(suffix.iter().copied());
                        ops.push(Op::Ow);
                        let names: Vec<String> = suffix.iter().map(|op| op.name()).collect();
                        all.push((
                            "I-after-refused-raise".into(),
                            width,
                            format!("p={prefix_kind},s={steps},raise_at={raise_index},suffix={}", names.join("+")),
                            ops,
                        ));
                    }
                }
            }
        }
    }
    all
}

#[test]
fn v1_reach() {
    let threads: usize = std::env::var("SINGLEFS_THREAD_CAP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        .clamp(1, 5);
    let checker_budget: usize = std::env::var("V1_CHECKER_BUDGET").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let jobs = histories();
    let mut buckets: Vec<Vec<(usize, (String, u64, String, Vec<Op>))>> = (0..threads).map(|_| Vec::new()).collect();
    for (index, job) in jobs.into_iter().enumerate() {
        buckets[index % threads].push((index, job));
    }
    let mut results: Vec<(usize, Vec<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = buckets
            .into_iter()
            .map(|bucket| {
                scope.spawn(move || {
                    bucket
                        .into_iter()
                        .map(|(index, (class, width, parameters, ops))| {
                            (index, run_history(&class, width, &parameters, &ops, checker_budget))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
    });
    results.sort_by_key(|(index, _)| *index);
    println!("# R\tclass\tU\tparams\tstep\top\tmember\tunit\tkind\ttxg_before\ttxg_after\tcap\tallocated\tdeferred\tisolated\tfree_slots\tallocatable\tlive_data\tF\tmissed_reclaim\tdata_placeable\tone_slot_placeable\ttwo_slot_placeable\tslack0\tsymmetric\tneed\tmount_commitment\ttier\tchecker");
    println!("# H\tclass\tU\tparams\tops\tapplied\trefusals\tnofree_after_admission\tfixed_point_hits\tdry_run_refusals\tfirst_fixed_point_hit_step");
    for (_, lines) in results {
        for line in lines {
            println!("{line}");
        }
    }
}
