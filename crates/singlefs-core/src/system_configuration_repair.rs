//! 系统配置见证判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过）之后的离线出路（α'，C596（见证判不出的系统配置盘面只剩只读挂载）；
//! 三方判决 `research/prompts/m3-close-sysconf-r1-main-verification.md` 第三节形态甲、`research/prompts/m3-close-sysconf-r2-main-verification.md` 第四节 α'、
//! `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U2 与第三节；岔路单 `research/prompts/m3-close-sysconf-r2-forks.md` S1 一行，
//! 用户 2026-10-19 定「许，另加扫单元写序」，2026-10-20 再采「离线出路加 birth_txg 闸」）。
//!
//! 调用方（管理员工具）断言一个版本；这里查过才写，只重写不自证的系统配置槽，不发布、不取号。只在取号写两轮（甲'）之上用：
//! 取号写两轮之下每块盘两槽都带着取号时的新号，这里写的实例代号取盘上读得到的最大值、不加一，下一次可写挂载取号照常加一。
//! 两样救不了的照实写进契约：持续读错的槽这里写了之后照旧读错，可写挂载照旧拒（结局与只读相同）；本实例写下的单元一个都解不开时，
//! 单元区扫描漏掉它们（`research/prompts/m3-close-sysconf-r3-opus-output.md`「扫描漏掉读不出写序的单元」）。

use singlefs_format::{
    DATA_UNIT_BYTES, NODE_BYTES, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use crate::block_device::{BlockDevice, BlockDeviceError};
use crate::journal::JournalRecordPlaceInPublish;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::recovery::{
    choose_system_configuration_of_the_pool, every_root_ring_slot, highest_root_instance,
    read_root_ring_slot, scan_journal, unit_area_start_of_the_chosen_system_configuration,
    verified_system_configuration_slots, PoolReader, RecoveryFailure, RootRingSlotReading,
};
use crate::system_configuration::SystemConfiguration;
use crate::transaction::{CommitStep, PoolWriter};
use crate::unit::{
    parse_data_unit, parse_index_node, parse_packed_unit, unit_filesystem_identifier,
};

/// 一块盘单元区里解得开头、fsid 是本池的单元，写序实例代号与 `birth_txg` 各自的最大值（[`write_order_and_birth_in_the_unit_area_of_the_device`]）。
/// 一个都没有时两项都是 `None`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteOrderAndBirthInTheUnitArea {
    pub highest_write_order_instance: Option<InstanceGeneration>,
    pub highest_birth_txg: Option<CheckpointTxg>,
}

/// 扫一块盘整个单元区（从 `unit_area_start` 起到盘尾，按 16 KiB 槽逐个读）：每个槽先读 16 KiB，全是 0 的跳过（没有单元从这里起）；
/// 解得开码 2 节点头就取它的实例代号与 `birth_txg`，解不开再读 32 KiB 试数据单元与打包记录单元，取写序实例代号与 `birth_txg`；
/// 单元头记的 fsid 不是本池的（`unit::unit_filesystem_identifier`）不算。读不出、解不开的槽跳过（扫描漏掉它们，模块文档那一条）。
/// 离线出路的 birth_txg 闸与写的实例代号用它（[`rewrite_system_configuration_slots_for_the_asserted_version`]），
/// 可写挂载取号补写两槽全废的盘之前的闸四也用它（`mount.rs` 的补写闸）。只读盘，内存只留两个最大值。
///
/// 循环：按槽（轮数 = 盘的槽数减单元区起点），跨轮只更新两个最大值，提前出口只有「这一槽跳过」。
#[must_use]
pub fn write_order_and_birth_in_the_unit_area_of_the_device<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    unit_area_start: SlotNumber,
    filesystem_identifier: &[u8; 16],
) -> WriteOrderAndBirthInTheUnitArea {
    let filesystem_identifier_in_unit_headers = unit_filesystem_identifier(filesystem_identifier);
    let slots_on_the_device = reader
        .device_size_in_bytes(device)
        .map_or(0, |device_bytes| device_bytes / SLOT_BYTES);
    let node_bytes = usize::try_from(NODE_BYTES).expect("码 2 节点 16384 字节");
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("数据单元 32768 字节");
    let mut scanned = WriteOrderAndBirthInTheUnitArea {
        highest_write_order_instance: None,
        highest_birth_txg: None,
    };
    for slot in unit_area_start.0..slots_on_the_device {
        let offset = SlotNumber(slot).to_device_offset();
        let Some(bytes_of_the_slot) = reader.read(device, offset, node_bytes) else {
            continue;
        };
        if bytes_of_the_slot.iter().all(|byte| *byte == 0) {
            continue;
        }
        let header = match parse_index_node(&bytes_of_the_slot) {
            Ok(index_node) => Some((
                index_node.filesystem_identifier,
                index_node.instance,
                index_node.birth_txg,
            )),
            Err(_not_an_index_node) => {
                reader
                    .read(device, offset, data_unit_bytes)
                    .and_then(|unit_bytes| {
                        parse_data_unit(&unit_bytes)
                            .map(|data_unit| {
                                (
                                    data_unit.filesystem_identifier,
                                    data_unit.write_order.instance,
                                    data_unit.birth_txg,
                                )
                            })
                            .or_else(|_not_a_data_unit| {
                                parse_packed_unit(&unit_bytes).map(|packed_unit| {
                                    (
                                        packed_unit.filesystem_identifier,
                                        packed_unit.write_order.instance,
                                        packed_unit.birth_txg,
                                    )
                                })
                            })
                            .ok()
                    })
            }
        };
        let Some((filesystem_identifier_in_the_header, instance, birth_txg)) = header else {
            continue;
        };
        if filesystem_identifier_in_the_header != filesystem_identifier_in_unit_headers {
            continue;
        }
        scanned.highest_write_order_instance =
            scanned.highest_write_order_instance.max(Some(instance));
        scanned.highest_birth_txg = scanned.highest_birth_txg.max(Some(birth_txg));
    }
    scanned
}

/// 调用方断言的那一版：(实例代号, checkpoint_txg)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssertedVersion {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// 重写了的一个系统配置槽：哪块盘的槽几（槽 i 在设备内偏移 i × 固定结构槽距）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemConfigurationSlotRewritten {
    pub device: DeviceIdentity,
    pub slot_index: u64,
}

/// 离线出路写成了：重写了哪几槽（按写的次序；每个槽都自证时为空，一个字节没写），写进去的实例代号与 journal tail。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationSlotsRewritten {
    pub slots_rewritten: Vec<SystemConfigurationSlotRewritten>,
    pub journal_instance_written: InstanceGeneration,
    pub journal_tail_written: u64,
}

/// 离线出路没写（除 [`Self::WriteFailed`] 之外都在任何写之前返回，盘上逐字节不变）。成员按管理员要做的决定分：
/// 换一个断言的版本、先修盘、或接受只读。
#[derive(Debug)]
pub enum SystemConfigurationRepairRefused {
    /// 按调用方给的本池 fsid 择不到系统配置（[`choose_system_configuration_of_the_pool`] 的各个成员）：盘上没有一槽可择的本池系统配置，
    /// 这里拿不到几何，不写。
    SystemConfigurationNotChosen(RecoveryFailure),
    /// 断言那一版的根在根环里读不出、它那次发布带末条标志的记录在 journal 环里也读不出（或读出的不是它）：查不到就一个字节不写
    /// （第二轮判决 α' 的「查断言那一版的根或记录读得出、且是它」）。
    AssertedVersionReadableNeitherInTheRootRingNorInTheJournalRing { asserted: AssertedVersion },
    /// 断言那一版的根读得出、带末条标志的记录读不出：写进系统配置的 tail 该取什么，条款没写（第三轮攻方的 α' 取那一版末条记录的计数器，
    /// 末条读不出时没有这个数），第一版不支持，一个字节不写。
    TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion {
        asserted: AssertedVersion,
    },
    /// 单元区里有单元的 `birth_txg` 比断言那一版大（birth_txg 闸，用户 2026-10-20 采「离线出路加 birth_txg 闸」）：比它新的版本写过单元，
    /// 照写会把可恢复的盘面写成丢了已确认的那一版（第三轮攻方 U2「断言 T−1」那一格），一个字节不写。代价：发布途中崩、只落了单元的孤单元
    /// 也会让真实的断言被拒，池留在只读。`device` 是按盘表次序第一块扫到它的盘。
    UnitBornAfterTheAssertedVersion {
        asserted: AssertedVersion,
        device: DeviceIdentity,
        highest_birth_txg: CheckpointTxg,
    },
    /// 一块盘只有一槽自证，按逐盘计的世代号（它的世代号 + 1）算出来的下一写落在它自己那一槽上，不落在不自证的那一槽
    /// （槽世代号与槽号的关系 I-7.16 不成立）：这里只重写不自证的槽，算不出落点就不写。
    SelfVerifiedSlotWouldBeOverwritten { device: DeviceIdentity },
    /// 写或写之后那道屏障报错：报错之前已写的槽留着（`slots_rewritten_before_the_failure`），之后一个字节不再写。
    WriteFailed {
        cause: BlockDeviceError,
        slots_rewritten_before_the_failure: Vec<SystemConfigurationSlotRewritten>,
    },
}

/// 一块盘两个系统配置槽各读一次：自证过（整槽校验和过、fsid 是本池的）的那几槽的 (槽几, 世代号)，与不自证的那几槽（读不出、自证不过、别的池的）。
struct SlotsOfOneDeviceForTheRewrite {
    self_verified: Vec<(u64, u64)>,
    not_self_verifying: Vec<u64>,
}

fn slots_of_one_device_for_the_rewrite<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    system_configuration: &SystemConfiguration,
) -> SlotsOfOneDeviceForTheRewrite {
    let spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    let mut slots = SlotsOfOneDeviceForTheRewrite {
        self_verified: Vec::new(),
        not_self_verifying: Vec::new(),
    };
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let parsed = reader
            .read(
                device,
                crate::address::DeviceOffsetInBytes(slot_index * spacing),
                slot_bytes,
            )
            .and_then(|bytes| SystemConfiguration::parse_slot(&bytes).ok())
            .filter(|parsed| {
                parsed.immutable.filesystem_identifier
                    == system_configuration.immutable.filesystem_identifier
            });
        match parsed {
            Some(self_verified) => slots
                .self_verified
                .push((slot_index, self_verified.quantities.slot_generation)),
            None => slots.not_self_verifying.push(slot_index),
        }
    }
    slots
}

/// 离线出路（α'）：调用方断言 `asserted` 那一版，查过才写——
///
/// 1. 按调用方的本池 fsid 择系统配置（拿几何）。
/// 2. 断言那一版的根在根环里读得出且是它，或它那次发布带末条标志的记录在 journal 环里读得出且是它；两样都没有一个字节不写。
///    tail 取那条末条记录的计数器：根读得出而末条读不出时条款没写取什么，一个字节不写。
/// 3. 扫每块盘整个单元区（[`write_order_and_birth_in_the_unit_area_of_the_device`]）：`birth_txg` 最大值大于断言那一版就一个字节不写。
/// 4. 写的实例代号取各盘自证槽、根环、journal 环与单元写序里的最大值（不加一）。
/// 5. 只重写不自证的槽（读不出、自证不过、别的池的）：逐盘按世代号写（`transaction::PoolWriter` 的系统配置槽写，世代号逐盘 +1、F 带整池生效值），
///    写完一道池屏障。每个槽都自证时一个字节不写，交回空表。
///
/// 只在取号写两轮（甲'）之上用（模块文档）。
///
/// # Errors
/// 见 [`SystemConfigurationRepairRefused`] 各成员；除写报错之外都在任何写之前返回，盘上逐字节不变。
#[allow(
    clippy::ptr_arg,
    reason = "choose_system_configuration_of_the_pool 与 scan_journal 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rewrite_system_configuration_slots_for_the_asserted_version<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    asserted: AssertedVersion,
) -> Result<SystemConfigurationSlotsRewritten, SystemConfigurationRepairRefused> {
    let system_configuration =
        choose_system_configuration_of_the_pool(&*devices, &parameters.filesystem_identifier)
            .map_err(SystemConfigurationRepairRefused::SystemConfigurationNotChosen)?;
    let immutable = &system_configuration.immutable;
    let root_of_the_asserted_version_is_readable = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .any(|ring_slot| {
            matches!(
                read_root_ring_slot(
                    &*devices,
                    &immutable.region_devices,
                    &immutable.sizes,
                    &immutable.filesystem_identifier,
                    ring_slot,
                ),
                RootRingSlotReading::SelfVerified(root)
                    if root.instance == asserted.instance
                        && root.checkpoint_txg == asserted.checkpoint_txg
            )
        });
    let records = scan_journal(&*devices, &system_configuration);
    let last_record_of_the_asserted_version = records
        .values()
        .filter(|record| {
            record.instance == asserted.instance
                && record.checkpoint_txg == asserted.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter);
    let journal_tail_written = match (
        root_of_the_asserted_version_is_readable,
        last_record_of_the_asserted_version,
    ) {
        (_, Some(last_record)) => last_record.counter,
        (true, None) => {
            return Err(
                SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion {
                    asserted,
                },
            )
        }
        (false, None) => {
            return Err(
                SystemConfigurationRepairRefused::AssertedVersionReadableNeitherInTheRootRingNorInTheJournalRing {
                    asserted,
                },
            )
        }
    };
    let unit_area_start =
        unit_area_start_of_the_chosen_system_configuration(&system_configuration).slot();
    let mut highest_write_order_instance_in_the_unit_areas: Option<InstanceGeneration> = None;
    for device in devices.device_identities() {
        let unit_area = write_order_and_birth_in_the_unit_area_of_the_device(
            &*devices,
            device,
            unit_area_start,
            &immutable.filesystem_identifier,
        );
        if let Some(highest_birth_txg) = unit_area
            .highest_birth_txg
            .filter(|highest_birth_txg| *highest_birth_txg > asserted.checkpoint_txg)
        {
            return Err(
                SystemConfigurationRepairRefused::UnitBornAfterTheAssertedVersion {
                    asserted,
                    device,
                    highest_birth_txg,
                },
            );
        }
        highest_write_order_instance_in_the_unit_areas =
            highest_write_order_instance_in_the_unit_areas
                .max(unit_area.highest_write_order_instance);
    }
    let spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
    let highest_instance_in_the_self_verified_slots = devices
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                &*devices,
                device,
                spacing,
                &immutable.filesystem_identifier,
            )
        })
        .map(|self_verified| self_verified.quantities.journal_instance)
        .max();
    let highest_instance_in_the_root_ring = highest_root_instance(
        &*devices,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let highest_instance_in_the_journal_ring = records.values().map(|record| record.instance).max();
    let journal_instance_written = [
        highest_instance_in_the_self_verified_slots,
        highest_instance_in_the_root_ring,
        highest_instance_in_the_journal_ring,
        highest_write_order_instance_in_the_unit_areas,
    ]
    .into_iter()
    .flatten()
    .max()
    .unwrap_or(asserted.instance)
    .max(asserted.instance);
    // 先把每块盘要重写的槽判完，再写：判与写读同一份输入，有一块盘算不出落点就一个字节不写。
    let mut slots_to_rewrite: Vec<(usize, Vec<u64>)> = Vec::new();
    for (index, (device, _)) in devices.iter().enumerate() {
        let slots = slots_of_one_device_for_the_rewrite(&*devices, *device, &system_configuration);
        if slots.not_self_verifying.is_empty() {
            continue;
        }
        if let Some(&(_, highest_self_verified_generation)) = slots
            .self_verified
            .iter()
            .max_by_key(|(_, generation)| *generation)
        {
            let slot_of_the_next_write =
                (highest_self_verified_generation + 1) % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
            if !slots.not_self_verifying.contains(&slot_of_the_next_write) {
                return Err(
                    SystemConfigurationRepairRefused::SelfVerifiedSlotWouldBeOverwritten {
                        device: *device,
                    },
                );
            }
        }
        slots_to_rewrite.push((index, slots.not_self_verifying));
    }
    let mut slots_rewritten: Vec<SystemConfigurationSlotRewritten> = Vec::new();
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    for (index, not_self_verifying) in slots_to_rewrite {
        let device = pool.devices[index].0;
        // 两槽都不自证时写两次：头一次世代号 1 落槽 1，第二次世代号 2 落槽 0（逐盘计）；一槽不自证时写一次，落在它上面（上面判过）。
        for _slot_not_self_verifying in &not_self_verifying {
            let generation_before = verified_system_configuration_slots(
                &*pool.devices,
                device,
                spacing,
                &immutable.filesystem_identifier,
            )
            .iter()
            .map(|self_verified| self_verified.quantities.slot_generation)
            .max()
            .unwrap_or(0);
            if let Err(cause) = pool
                .write_system_configuration_slot_carrying_the_pool_effective_floor(
                    index,
                    journal_tail_written,
                    journal_instance_written,
                )
            {
                return Err(SystemConfigurationRepairRefused::WriteFailed {
                    cause,
                    slots_rewritten_before_the_failure: slots_rewritten,
                });
            }
            slots_rewritten.push(SystemConfigurationSlotRewritten {
                device,
                slot_index: (generation_before + 1) % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            });
        }
    }
    if !slots_rewritten.is_empty() {
        if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {
            return Err(SystemConfigurationRepairRefused::WriteFailed {
                cause,
                slots_rewritten_before_the_failure: slots_rewritten,
            });
        }
    }
    Ok(SystemConfigurationSlotsRewritten {
        slots_rewritten,
        journal_instance_written,
        journal_tail_written,
    })
}
