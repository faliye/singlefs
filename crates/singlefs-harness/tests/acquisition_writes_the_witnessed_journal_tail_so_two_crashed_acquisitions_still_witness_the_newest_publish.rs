//! C554 乙-配置续（用户 2026-09-27 JST 12:08 定；岔路单 `research/prompts/c554-fix-forks.md` 第 2 行；定义取自 E158 第 3 次跑登记
//! `research/prompts/e158-r3-prereg.md` 第 348 行与第 4 次跑登记 `research/prompts/e158-r4-prereg.md` 第 413 行）：取号那一写的 tail
//! 不写 0，写取号那一刻按判据 N-配置 同一取法读到的见证值 c_见证（每块盘两槽里全部自证过的系统配置槽的 `journal_tail` 取最大，
//! 一份都没有时 0）；取号失败的回卷写带同一个值。
//!
//! 为什么要它：C554 乙（可写挂载读到的样子里有更新的东西读不出就重读一次，仍读不出就拒可写）判「更新」靠系统配置见证的 tail。
//! 连着两次取号之后崩溃、中间没有发布轮换（E158 第 3 段 H1g a = 2 那一形），每块盘两槽都换成了取号写；取号写 tail 0 时见证就丢了，
//! 最新那条根暂时读不出的时候可写挂载判不出、照常挂载并把它当成被抛弃（`research/prompts/e158-r4-seg3-runner-report.md` 第 9 行：
//! -配置 四臂各丢写 16/32、全在 a = 2，-配置续 0/32）。
//!
//! 每条用例从盘上的字节核（`recovery::verified_system_configuration_slots` 解录制流施加出来的镜像），不经写者算的量。
mod common;

use common::{
    build_pool, disk_snapshot, format_pool, geometry, parameters, publish_overwrite_in_process,
    unreadable_root_slot_of, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, Recorded, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_writable, MountError, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::recovery::{verified_system_configuration_slots, PoolReader};
use singlefs_core::transaction::{
    acquire_instance, AcquisitionRollback, InstanceAcquisitionFailed, PoolWriter, TransactionOutput,
};
use singlefs_format::{
    DATA_UNIT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// mkfs 写进每个系统配置槽的 tail（`.claude/kb/layout/01-first-txn.md` 一「journal tail」那一行：mkfs 0）。
const JOURNAL_TAIL_WRITTEN_BY_MAKE_FILESYSTEM: u64 = 0;

/// 一个自证过的系统配置槽里这几条用例要看的三样。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SlotReading {
    slot_generation: u64,
    journal_instance: InstanceGeneration,
    journal_tail: u64,
}

/// 一块盘两槽里自证过（整槽校验和过、fsid 与本池相同）的系统配置，按世代号排好。
fn slot_readings_of<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
) -> Vec<SlotReading> {
    let pool_parameters = parameters();
    let mut readings: Vec<SlotReading> = verified_system_configuration_slots(
        reader,
        device,
        u64::from(pool_parameters.geometry.fixed_structure_slot_spacing),
        &pool_parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|system_configuration| SlotReading {
        slot_generation: system_configuration.quantities.slot_generation,
        journal_instance: system_configuration.quantities.journal_instance,
        journal_tail: system_configuration.quantities.journal_tail,
    })
    .collect();
    readings.sort();
    readings
}

/// 进程退出、重开，取一次号就崩：取号写完两槽、过了取号之后那道屏障，之后一个写都没发（写行与暖机都没开始）。
/// `acquire_instance` 与可写挂载取号走同一个写那一半（`transaction::write_acquired_instance`）。
fn acquire_once_then_crash(pool: &mut BuiltPool) -> InstanceGeneration {
    let pool_parameters = parameters();
    let mut devices = pool.reopen_recorded();
    let instance = {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        acquire_instance(&mut writer).expect("取号")
    };
    pool.devices = Some(devices);
    instance
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

const SECOND_VERSION_BYTES: usize = 4100;
const NEWEST_VERSION_BYTES: usize = 2500;

/// 新池新建文件（A，实例 1，txg 3，jsn 3）之后在同一个进程里覆盖写 B（txg 4，jsn 4）、C（txg 5，jsn 5）：C 的根 FUA 之后系统配置轮换，
/// 每块盘世代 7 那一槽写的是 C 那条记录的计数器 5——系统配置见证了 C。
struct PoolWithAWitnessedNewestPublish {
    pool: BuiltPool,
    second: TransactionOutput,
    newest: TransactionOutput,
}

fn pool_with_a_witnessed_newest_publish(tag: &str) -> PoolWithAWitnessedNewestPublish {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(SECOND_VERSION_BYTES, 3),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let newest = publish_overwrite_in_process(
        &mut pool,
        &second,
        &content_of(NEWEST_VERSION_BYTES, 11),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("C");
    pool.output = newest.clone();
    assert_eq!(
        (
            second.root.checkpoint_txg,
            second.record.counter,
            newest.root.checkpoint_txg,
            newest.record.counter
        ),
        (CheckpointTxg(4), 4, CheckpointTxg(5), 5),
        "暖机 txg 1、2（jsn 1、2），A txg 3（jsn 3），B、C 各一条记录：txg 4、5，jsn 4、5"
    );
    let image = pool.memory_pool();
    for disk in DISKS {
        assert_eq!(
            slot_readings_of(&image, disk),
            vec![
                SlotReading {
                    slot_generation: 6,
                    journal_instance: InstanceGeneration(1),
                    journal_tail: 4,
                },
                SlotReading {
                    slot_generation: 7,
                    journal_instance: InstanceGeneration(1),
                    journal_tail: 5,
                },
            ],
            "{disk:?}：B、C 各轮换一次系统配置，世代 7 见证 C"
        );
    }
    PoolWithAWitnessedNewestPublish {
        pool,
        second,
        newest,
    }
}

/// C 的根槽与它两份数据单元，每次读都坏：择根看不见 C、C 那条记录施加前验点名单元验不过——读阶段落到 B。
fn root_slot_and_data_units_of(publish: &TransactionOutput) -> Vec<UnreadableRange> {
    std::iter::once(unreadable_root_slot_of(
        publish.root.checkpoint_txg,
        FailingReadsOfARange::Every,
    ))
    .chain(
        publish
            .data_pointers
            .iter()
            .flat_map(|pointer| pointer.locations)
            .map(|location| UnreadableRange {
                device: location.device,
                offset_in_bytes: location.slot.to_device_offset().0,
                length_in_bytes: DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            }),
    )
    .collect()
}

/// 连着 `crashed_acquisitions` 次取号之后崩溃（每次取号写完两槽、过了屏障，写行与暖机都没开始），中间没有任何发布轮换：
/// 第二次起每块盘两槽都是取号写，世代 7 那一槽（C 的轮换）已被覆写。取号写带着见证值 5，两槽 tail 都是 5。
/// 之后 C 的根槽与数据单元这一次挂载里每次读都坏：读阶段落到 B（末条 jsn 4），系统配置见证到 5，判据为真；重读一遍仍落到 B
/// ⇒ 拒可写，取号之前拒，`DiskSnapshot` 不变。取号写 tail 写 0 时两槽都是 0、判据为假（`NothingWitnessed`），挂载做成、C 被抛弃。
fn crashed_acquisitions_in_a_row_still_witness_the_newest_publish(
    tag: &str,
    crashed_acquisitions: u32,
) {
    let PoolWithAWitnessedNewestPublish {
        mut pool,
        second,
        newest,
    } = pool_with_a_witnessed_newest_publish(tag);
    for acquisition_index in 0..crashed_acquisitions {
        assert_eq!(
            acquire_once_then_crash(&mut pool),
            InstanceGeneration(2 + acquisition_index),
            "第 {} 次崩掉的取号取的号",
            acquisition_index + 1
        );
    }
    let image = pool.memory_pool();
    let slot_readings_after_the_crashes: Vec<(DeviceIdentity, Vec<SlotReading>)> = DISKS
        .into_iter()
        .map(|disk| (disk, slot_readings_of(&image, disk)))
        .collect();

    let before = disk_snapshot(&image, &pool.stream);
    let unreadable = SharedUnreadableRanges::new(
        root_slot_and_data_units_of(&newest),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let refused = mount_writable(&parameters(), &mut devices);
    let expected_reading = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: second.root.instance,
            checkpoint_txg: second.root.checkpoint_txg,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: newest.record.counter,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter: second.record.counter,
            },
        },
    };
    assert!(
        matches!(
            &refused,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                if **still_unreadable
                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                        first_read: expected_reading,
                        reread: expected_reading,
                    }
        ),
        "{crashed_acquisitions} 次崩掉的取号之后系统配置仍见证 C（jsn 5 > B 的末条 4）：两遍都落到 B，拒可写；得到 {}",
        match &refused {
            Ok(mounted) => format!(
                "挂载做成：所选那一版 ({:?}, {:?})，读阶段 {:?}",
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg,
                mounted.output.rereads.read_stage
            ),
            Err(error) => format!("{error:?}"),
        }
    );
    pool.devices = Some(without_unreadable_ranges(devices));
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "取号之前拒：一个写、一道屏障都没发"
    );
    assert_eq!(
        slot_readings_after_the_crashes,
        DISKS
            .into_iter()
            .map(|disk| {
                (
                    disk,
                    [crashed_acquisitions - 1, crashed_acquisitions]
                        .into_iter()
                        .map(|acquisition_number| SlotReading {
                            slot_generation: 7 + u64::from(acquisition_number),
                            journal_instance: InstanceGeneration(1 + acquisition_number),
                            journal_tail: newest.record.counter,
                        })
                        .collect(),
                )
            })
            .collect::<Vec<_>>(),
        "崩掉的取号之后每块盘两槽都是取号写（世代 7 之后逐次 +1），C 那次轮换写的世代 7 已被覆写；tail 都带着它见证的 jsn {}",
        newest.record.counter
    );
}

#[test]
fn two_crashed_acquisitions_in_a_row_still_witness_the_newest_publish_so_the_writable_mount_refuses_instead_of_abandoning_its_unreadable_root(
) {
    crashed_acquisitions_in_a_row_still_witness_the_newest_publish(
        "c554-yi-carry-two-acquisitions",
        2,
    );
}

/// 第三次取号读到的两槽都是取号写：见证值只从前两次取号写一路带过来。
#[test]
fn three_crashed_acquisitions_in_a_row_carry_the_witness_from_acquisition_write_to_acquisition_write(
) {
    crashed_acquisitions_in_a_row_still_witness_the_newest_publish(
        "c554-yi-carry-three-acquisitions",
        3,
    );
}

/// mkfs 之后的第一次取号：mkfs 写的两槽 tail 都是 0，见证值就是 0，取号写照旧写 0——新池新建文件的字节不变。
#[test]
fn the_first_acquisition_after_mkfs_still_writes_tail_zero_so_the_new_pool_file_creation_bytes_do_not_change(
) {
    let mut formatted = format_pool("c554-yi-carry-first-acquisition");
    let pool_parameters = parameters();
    let mut devices = formatted.devices.take().expect("mkfs 之后盘还开着");
    for disk in DISKS {
        assert_eq!(
            slot_readings_of(devices.as_slice(), disk)
                .into_iter()
                .map(|reading| reading.journal_tail)
                .collect::<Vec<_>>(),
            vec![
                JOURNAL_TAIL_WRITTEN_BY_MAKE_FILESYSTEM,
                JOURNAL_TAIL_WRITTEN_BY_MAKE_FILESYSTEM
            ],
            "{disk:?}：mkfs 两槽都种 tail 0"
        );
    }
    {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        assert_eq!(
            acquire_instance(&mut writer).expect("第一次取号"),
            InstanceGeneration(1)
        );
    }
    formatted.devices = Some(devices);
    let image = formatted.memory_pool();
    for disk in DISKS {
        let newest = *slot_readings_of(&image, disk)
            .last()
            .expect("每块盘都有自证过的系统配置");
        assert_eq!(
            newest,
            SlotReading {
                slot_generation: 2,
                journal_instance: InstanceGeneration(1),
                journal_tail: JOURNAL_TAIL_WRITTEN_BY_MAKE_FILESYSTEM,
            },
            "{disk:?}：取号写世代 2、实例代号 1、tail 0（`.claude/kb/layout/01-first-txn.md` a1 那一行）"
        );
    }
}

/// 新池新建文件之后每块盘两槽：世代 4（暖机第二次，tail 2）、世代 5（新池新建文件，tail 3）。盘 `damaged_device` 世代 5 那一槽坏掉（清零），
/// 池里最大的 tail 3 只在另一块盘上读得出：第二次取号写进每块盘的 tail 都是 3——取整池的最大值，不是各盘自己的，也不是最小的。
fn a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lost_its_newest_slot(
    tag: &str,
    damaged_device: DeviceIdentity,
) {
    let mut pool = build_pool(tag);
    let new_pool_file_creation_counter = pool.output.record.counter;
    assert_eq!(
        new_pool_file_creation_counter, 3,
        "暖机 jsn 1、2，新池新建文件 jsn 3"
    );
    let slot_spacing_in_bytes = u64::from(geometry().fixed_structure_slot_spacing);
    {
        let devices: &mut Vec<(DeviceIdentity, Recorded)> =
            pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == damaged_device)
            .expect("池里有这块盘");
        device
            .write_at(
                DeviceOffsetInBytes(
                    (5 % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing_in_bytes,
                ),
                &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
                WriteDurability::Plain,
            )
            .expect("世代 5 那一槽清零");
    }
    let before_the_acquisition = pool.memory_pool();
    let tails_on_disk: Vec<(DeviceIdentity, Vec<u64>)> = DISKS
        .into_iter()
        .map(|disk| {
            (
                disk,
                slot_readings_of(&before_the_acquisition, disk)
                    .into_iter()
                    .map(|reading| reading.journal_tail)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        tails_on_disk,
        DISKS
            .into_iter()
            .map(|disk| {
                if disk == damaged_device {
                    (disk, vec![2])
                } else {
                    (disk, vec![2, new_pool_file_creation_counter])
                }
            })
            .collect::<Vec<_>>(),
        "坏掉的那块盘只剩世代 4 那一槽（tail 2），另一块两槽 tail 2、3"
    );
    let largest_tail_of_the_pool = tails_on_disk
        .iter()
        .flat_map(|(_, tails)| tails.iter().copied())
        .max()
        .expect("池里有自证过的系统配置");
    assert_eq!(largest_tail_of_the_pool, new_pool_file_creation_counter);

    assert_eq!(acquire_once_then_crash(&mut pool), InstanceGeneration(2));
    let after_the_acquisition = pool.memory_pool();
    for disk in DISKS {
        let acquisition_writes: Vec<SlotReading> = slot_readings_of(&after_the_acquisition, disk)
            .into_iter()
            .filter(|reading| reading.journal_instance == InstanceGeneration(2))
            .collect();
        assert_eq!(
            acquisition_writes
                .iter()
                .map(|reading| reading.journal_tail)
                .collect::<Vec<_>>(),
            vec![largest_tail_of_the_pool],
            "{disk:?}：一份取号写，tail 是取号前池里两槽最大的 tail {largest_tail_of_the_pool}：{acquisition_writes:?}"
        );
    }
}

#[test]
fn a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_first_device_lost_its_newest_slot(
) {
    a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lost_its_newest_slot(
        "c554-yi-carry-device-zero-damaged",
        DeviceIdentity(0),
    );
}

#[test]
fn a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_second_device_lost_its_newest_slot(
) {
    a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lost_its_newest_slot(
        "c554-yi-carry-device-one-damaged",
        DeviceIdentity(1),
    );
}

/// 取号写在盘 1 上报错：盘 0 那份已写出、回卷成旧代号 1。回卷是「像没取过号」，回卷写的 tail 与取号写同是取号那一刻读到的见证值 3，
/// 不退回 0；盘 1 一个字节都没写成。
#[test]
fn a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero() {
    /// 系统配置两槽住在偏移 0 与 4096，都在这个界之下。
    const SYSTEM_CONFIGURATION_SLOTS_END_OFFSET: u64 = 8192;
    let mut built = build_pool("c554-yi-carry-rolled-back");
    let new_pool_file_creation_counter = built.output.record.counter;
    let plan = SharedFaultPlan::unarmed(geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = built
        .devices
        .take()
        .expect("新池新建文件写完，盘还开着")
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, inner, plan.clone()),
            )
        })
        .collect();
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_END_OFFSET),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let failure = {
        let pool_parameters = parameters();
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        match acquire_instance(&mut writer).expect_err("盘 1 的取号写报错，取号必须失败") {
            InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
            InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness {
                device,
            } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
        }
    };
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "盘 0 那份已写出，要回卷：{failure:?}"
    );
    assert_eq!(
        slot_readings_of(devices.as_slice(), DeviceIdentity(0)),
        vec![
            SlotReading {
                slot_generation: 6,
                journal_instance: InstanceGeneration(2),
                journal_tail: new_pool_file_creation_counter,
            },
            SlotReading {
                slot_generation: 7,
                journal_instance: InstanceGeneration(1),
                journal_tail: new_pool_file_creation_counter,
            },
        ],
        "盘 0：取号写世代 6 带新号 2，回卷写世代 7 带旧号 1；两写的 tail 都是取号前的见证值 {new_pool_file_creation_counter}"
    );
    assert_eq!(
        slot_readings_of(devices.as_slice(), DeviceIdentity(1)),
        vec![
            SlotReading {
                slot_generation: 4,
                journal_instance: InstanceGeneration(1),
                journal_tail: 2,
            },
            SlotReading {
                slot_generation: 5,
                journal_instance: InstanceGeneration(1),
                journal_tail: new_pool_file_creation_counter,
            },
        ],
        "盘 1 一个字节都没写成"
    );
}
