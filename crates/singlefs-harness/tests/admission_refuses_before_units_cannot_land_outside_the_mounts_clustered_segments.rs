//! C545（空间准入罩不住分裂与聚簇段层） 取「准入先拒」：发布路径在走分配记录树的固定点、在分配器的拷贝上
//! 取落点之前，先判这次的单元落不落得下（`singlefs_core::admission::admit_the_units_landing_on_every_device`）——数据单元要这次挂载开过的
//! 聚簇段之外成对的空槽，提交内生块要没挡的槽——不够交回 `PublishError::SpaceAdmissionRefused`，在任何写之前，两块盘逐字节不变、分配器不动。
//! 实审 A4c 造出来的两格（报告 `research/prompts/m2-rev-a4c-implementer-report.md` 第三节）此前交回 `PlacementRefused`，钉成准入先拒；
//! 小盘为环窗口补足之后（用户 2026-10-02 定 Q-N「甲」）普通写被式子先拒，那两格改用删除类发布（不判式子、落点那一判照判）去撞（Q-T「甲」），
//! 数据单元那一半由一格造盘面的覆盖写罩着；另有一格数据单元落得下、提交内生块落不下的；再加一格对照：准入用只供测试的开关关掉时
//! 落点那一道照样兜着，会话推过抬 F 再发成。盘都是内存里的稀疏盘，两块、单元区 256 / 384 槽（`HistoryDeviceWidth` 那两档）。

mod common_admission;

use common_admission::{
    checker_violations_on, content_of, plain_devices_on, start_plain, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
    space_budget_of_role, AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceShortOfDemand,
    SpaceAdmission, SpaceBudgetOfARole,
};
use singlefs_core::allocator::{PlacementRefusal, PoolAllocator};
use singlefs_core::journal::back_chain_of;
use singlefs_core::transaction::{
    publish_overwrite, publish_unlink_intent, publish_unlink_step, publish_version, FirstFile,
    InstanceTablePlan, PoolVersion, PoolWriter, PublishError, PublishPlan, TransactionOutput,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_format::CLUSTER_SEGMENT_SLOTS;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

/// 删除的就是第一个文件。
const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 这次挂载的会话上直接发一次空发布（暖机、抬 F 与卸载推的那一种：不写文件内容、实例表照抄），不经会话的准入与推抬 F。
fn publish_an_empty_version_directly_on_the_session(pool: &mut PoolUnderTest<SparseBlockDevice>) {
    let session = pool.session.as_mut().expect("这次挂载的会话");
    let PoolVersion::WithFile(current) = &session.current else {
        panic!("第一个文件之后现行那一版带文件");
    };
    let current = current.clone();
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let published = publish_version(
        &mut writer,
        &mut session.allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: current
                .highest_transaction_number_in_this_instance,
            instance: current.root.instance,
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(current.root.instance_table),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(&current),
    )
    .expect("空发布做成：它不判准入，固定点从这次挂载开的聚簇段里 bump");
    session.current = PoolVersion::WithFile(published);
}

/// 每块盘上两槽都空（没分配、没隔离、没扣住）的偶数起点槽对：落在这次挂载开过的聚簇段外的几对、段内的几对。逐槽扫（用例这一侧，
/// 不读分配器增量维护的计数）。
fn free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount(
    allocator: &PoolAllocator,
) -> Vec<(DeviceIdentity, u64, u64)> {
    allocator
        .devices
        .iter()
        .map(|device_map| {
            let unit_area_end =
                device_map.unit_area_start().slot().0 + device_map.unit_area_slots();
            let mut pairs_outside = 0;
            let mut pairs_inside = 0;
            let mut first_slot_of_the_pair = device_map.unit_area_start().slot().0;
            // 迭代上界是单元区槽数的一半：每一轮往后两槽。
            while first_slot_of_the_pair + 1 < unit_area_end {
                if device_map.is_free(SlotNumber(first_slot_of_the_pair))
                    && device_map.is_free(SlotNumber(first_slot_of_the_pair + 1))
                {
                    let is_inside_a_cluster_segment =
                        allocator.cluster_segments().iter().any(|segment_start| {
                            first_slot_of_the_pair >= segment_start.0
                                && first_slot_of_the_pair < segment_start.0 + CLUSTER_SEGMENT_SLOTS
                        });
                    if is_inside_a_cluster_segment {
                        pairs_inside += 1;
                    } else {
                        pairs_outside += 1;
                    }
                }
                first_slot_of_the_pair += 2;
            }
            (device_map.device, pairs_outside, pairs_inside)
        })
        .collect()
}

/// 两块盘上各一条、同样的 `available` 与 `demand`（第一版每个单元落每块盘，两块盘此刻的空闲图相同）。
fn the_same_shortage_on_both_devices(
    available: AvailableBytesOnOneDevice,
    demand: BytesOnOneDevice,
) -> Vec<DeviceShortOfDemand> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .map(|device| DeviceShortOfDemand {
            device,
            available,
            demand,
        })
        .to_vec()
}

/// 数据单元个数对应的槽数（一个数据单元两槽）。
fn slots_of_data_units(data_units: u64) -> u64 {
    data_units
        * TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile::FIRST).span_slots()
}

/// 这次挂载的会话上每块盘没挡的槽都标成已分配（位图上占着、不写记录，只供用例造盘面），`keep` 点名的槽留着。
fn take_every_free_slot_on_the_session_except(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    keep: impl Fn(u64) -> bool,
) {
    let session = pool.session.as_mut().expect("这次挂载的会话");
    for device_map in &mut session.allocator.devices {
        let first_slot = device_map.unit_area_start().slot().0;
        let past_the_last_slot = first_slot + device_map.unit_area_slots();
        // 迭代上界是单元区槽数，跨轮只改这一槽的分配位。
        for slot in first_slot..past_the_last_slot {
            if !keep(slot) && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 在这次挂载的会话上直接发一次（`publish` 收写入口、分配器、现行那一版、实例号），不经会话的准入与推抬 F：做成就换上新的一版；被拒原样交回。
fn publish_directly_on_the_session(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    publish: impl FnOnce(
        &mut PoolWriter<'_, SparseBlockDevice>,
        &mut PoolAllocator,
        &TransactionOutput,
        InstanceGeneration,
    ) -> Result<TransactionOutput, PublishError>,
) -> Result<TransactionOutput, PublishError> {
    let session = pool.session.as_mut().expect("这次挂载的会话");
    let current = session
        .current
        .version_of_trees()
        .expect("第一个文件之后现行那一版有树")
        .clone();
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let outcome = publish(
        &mut writer,
        &mut session.allocator,
        &current,
        session.instance,
    );
    if let Ok(published) = &outcome {
        session.current = PoolVersion::of_a_version_of_trees(published.clone());
    }
    outcome
}

/// 同一个盘面、同一份分配器各拷一份，在拷贝上发同一次发布，交回它重写的角色里普通分配那一半（`admission::space_budget_of_role`）的槽数：
/// 落得下那一判按这一半的角色数槽（`admission::units_of_a_publish_to_land`）。原来的盘与分配器一个字节都不动。
fn ordinary_allocation_slots_of_the_same_publish_on_a_copy(
    pool: &PoolUnderTest<SparseBlockDevice>,
    publish: impl FnOnce(
        &mut PoolWriter<'_, SparseBlockDevice>,
        &mut PoolAllocator,
        &TransactionOutput,
        InstanceGeneration,
    ) -> Result<TransactionOutput, PublishError>,
) -> u64 {
    let session = pool.session.as_ref().expect("这次挂载的会话");
    let current = session
        .current
        .version_of_trees()
        .expect("第一个文件之后现行那一版有树")
        .clone();
    let mut devices = plain_devices_on(&pool.image());
    let mut allocator = session.allocator.clone();
    let mut writer = PoolWriter::new(&pool.parameters, devices.as_mut_slice());
    let published = publish(&mut writer, &mut allocator, &current, session.instance)
        .expect("没占满的拷贝上这次发布做得成");
    published
        .rewritten
        .iter()
        .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::OrdinaryAllocation)
        .map(|role| role.span_slots())
        .sum()
}

/// 用户 2026-10-02 定 Q-T「甲」：环窗口补足之后普通写被式子先拒、走不到「式子不拦而落点拒」，改用删除类发布去撞——删除类发布
/// （意图、各批、续做批）不判式子那一判、落点那一判照判（D28（挂载期承诺量） 已定项 1）。两块单元区 384 槽的小盘，第一个文件之后崩了再挂；
/// 每块盘没挡的槽全标成已分配（只供用例造盘面），unlink 第一个文件的意图那次：普通分配那一半（inode 叶与根、`deleted_inodes` 根、
/// 墓碑容器与根）加固定点按 ckpt_cost 计，一槽都落不下——发布路径在走固定点之前交回 `SpaceAdmissionRefused`，每块盘报 0 槽 < 要的槽；
/// 报的是落得下那一判（式子这时的可用是负的、也小于要的，它不判：报的数不是它的）。两块盘逐字节不变、分配器不动。
/// 判别力：落得下那一判整判拿掉时，走固定点在分配器的拷贝上取不到落点，交回 `PlacementRefused`。
#[test]
fn an_unlink_intent_whose_commit_generated_units_find_no_unblocked_slot_is_refused_by_the_landing_judgement_before_any_write_while_the_formula_is_not_judged(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    let write_time_seconds_of_the_file = pool.write_time_seconds;
    let intent = |writer: &mut PoolWriter<'_, SparseBlockDevice>,
                  allocator: &mut PoolAllocator,
                  current: &TransactionOutput,
                  instance: InstanceGeneration| {
        publish_unlink_intent(
            writer,
            allocator,
            current,
            FIRST_FILE,
            write_time_seconds_of_the_file,
            instance,
        )
    };
    let ordinary_allocation_slots =
        ordinary_allocation_slots_of_the_same_publish_on_a_copy(&pool, intent);
    take_every_free_slot_on_the_session_except(&mut pool, |_| false);
    let allocator_before = pool.session().allocator.clone();
    let current = pool.current_file_version().clone();
    let demand_in_slots = ordinary_allocation_slots
        + checkpoint_cost_of_the_version_to_build_on(Some(&current), &allocator_before).0;
    assert!(
        admission_reading_before_a_publish(&allocator_before, Some(&current))
            .available_on_each_device()
            .into_iter()
            .all(|(_, available)| available.0
                < i128::from(BytesOnOneDevice::of_slots(demand_in_slots).0)),
        "式子那一判这时也放不下（它对删除类发布不判，报出来的不能是它的数）"
    );
    let image_before = pool.image();
    let refused = publish_directly_on_the_session(&mut pool, intent);
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &refused else {
        panic!(
            "unlink 的意图：落得下那一判先拒：{:?}",
            refused.as_ref().map(|version| version.root.checkpoint_txg)
        );
    };
    assert_eq!(
        refusal.short_devices,
        the_same_shortage_on_both_devices(
            AvailableBytesOnOneDevice(0),
            BytesOnOneDevice::of_slots(demand_in_slots),
        ),
        "每块盘没挡的槽 0 < 意图那次普通分配那一半 {ordinary_allocation_slots} 槽加 ckpt_cost"
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", pool.session().allocator) == format!("{allocator_before:?}"),
        "分配器与发布之前逐项相同"
    );
}

/// 同一条（Q-T「甲」），撞在删除的一批上：两块单元区 256 槽的小盘，第一个文件之后崩了再挂、把文件顺序写成 3 个数据单元，
/// 直接发 unlink 的意图（做成），再把每块盘没挡的槽全标成已分配；按批大小 1 发下一批（截掉尾部一个单元：重写被截掉那一片下段叶与祖先、
/// inode 叶与根）：落得下那一判交回 `SpaceAdmissionRefused`，每块盘报 0 槽 < 这一批普通分配那一半加 ckpt_cost；两块盘逐字节不变、分配器不动。
/// 判别力：落得下那一判整判拿掉时交回 `PlacementRefused`。
#[test]
fn a_deletion_batch_on_the_narrower_pool_whose_commit_generated_units_find_no_unblocked_slot_is_refused_by_the_landing_judgement_before_any_write(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf256Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    pool.sequential_write(3 * data_unit_payload_capacity())
        .expect("顺序写成 3 个数据单元");
    let write_time_seconds_of_the_file = pool.write_time_seconds;
    publish_directly_on_the_session(&mut pool, |writer, allocator, current, instance| {
        publish_unlink_intent(
            writer,
            allocator,
            current,
            FIRST_FILE,
            write_time_seconds_of_the_file,
            instance,
        )
    })
    .expect("unlink 的意图做成");
    let batch_of_one_data_unit = |writer: &mut PoolWriter<'_, SparseBlockDevice>,
                                  allocator: &mut PoolAllocator,
                                  current: &TransactionOutput,
                                  instance: InstanceGeneration| {
        publish_unlink_step(
            writer,
            allocator,
            current,
            FIRST_FILE,
            UnlinkBatchSize::CappedForTests(
                UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
            ),
            instance,
        )
    };
    let ordinary_allocation_slots =
        ordinary_allocation_slots_of_the_same_publish_on_a_copy(&pool, batch_of_one_data_unit);
    take_every_free_slot_on_the_session_except(&mut pool, |_| false);
    let allocator_before = pool.session().allocator.clone();
    let current = pool
        .session()
        .current
        .version_of_trees()
        .expect("意图那一版有树")
        .clone();
    let demand_in_slots = ordinary_allocation_slots
        + checkpoint_cost_of_the_version_to_build_on(Some(&current), &allocator_before).0;
    let image_before = pool.image();
    let refused = publish_directly_on_the_session(&mut pool, batch_of_one_data_unit);
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &refused else {
        panic!(
            "删除的一批：落得下那一判先拒：{:?}",
            refused.as_ref().map(|version| version.root.checkpoint_txg)
        );
    };
    assert_eq!(
        refusal.short_devices,
        the_same_shortage_on_both_devices(
            AvailableBytesOnOneDevice(0),
            BytesOnOneDevice::of_slots(demand_in_slots),
        ),
        "每块盘没挡的槽 0 < 这一批普通分配那一半 {ordinary_allocation_slots} 槽加 ckpt_cost"
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", pool.session().allocator) == format!("{allocator_before:?}"),
        "分配器与发布之前逐项相同"
    );
}

/// 落得下那一判的数据单元那一半（C545）在补足之后的小盘上由用例造盘面来撞：两块单元区 384 槽的小盘，第一个文件之后崩了再挂，
/// 这次挂载开过的聚簇段之外没挡的槽全标成已分配（段内的留着）。覆盖写一次：数据单元要段外成对的空槽、一对都没有——落得下那一判排在式子之前，
/// 交回 `SpaceAdmissionRefused`，每块盘报段外 0 槽 < 1 个数据单元 2 槽；两块盘逐字节不变、分配器不动。
/// （此前两格靠空发布把段外的槽对耗尽，补足 815 / 739 槽之后同样的步数段外还剩几百对；用户 2026-10-02 定 Q-T「甲」之后那两格改撞删除类发布，
/// 数据单元那一半由这一格罩着。）
/// 判别力：「落得下」那一判不判数据单元时，段内没挡的槽够提交内生块，走固定点时取不到数据单元 0 的落点，交回 `PlacementRefused`。
#[test]
fn a_write_whose_slot_pairs_outside_this_mounts_cluster_segments_are_all_taken_is_refused_for_its_data_unit_before_any_write(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    let cluster_segments: Vec<u64> = pool
        .session()
        .allocator
        .cluster_segments()
        .iter()
        .map(|segment_start| segment_start.0)
        .collect();
    take_every_free_slot_on_the_session_except(&mut pool, |slot| {
        cluster_segments.iter().any(|segment_start| {
            slot >= *segment_start && slot < segment_start + CLUSTER_SEGMENT_SLOTS
        })
    });
    let allocator_before = pool.session().allocator.clone();
    let pairs =
        free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount(&allocator_before);
    assert!(
        pairs
            .iter()
            .all(|(_, pairs_outside, pairs_inside)| *pairs_outside == 0 && *pairs_inside > 0),
        "每块盘段外 0 对、段内还有空槽对：{pairs:?}"
    );
    let image_before = pool.image();
    pool.write_time_seconds += 1;
    let content = content_of(OVERWRITE_BYTES, pool.write_time_seconds);
    let write_time_seconds = pool.write_time_seconds;
    let refused =
        publish_directly_on_the_session(&mut pool, |writer, allocator, current, instance| {
            publish_overwrite(
                writer,
                allocator,
                current,
                FirstFile {
                    content: &content,
                    write_time_seconds,
                },
                instance,
            )
        });
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &refused else {
        panic!(
            "覆盖写：准入先拒（这次的数据单元在段外落不下）：{:?}",
            refused.as_ref().map(|version| version.root.checkpoint_txg)
        );
    };
    assert_eq!(
        refusal.short_devices,
        the_same_shortage_on_both_devices(
            AvailableBytesOnOneDevice(0),
            BytesOnOneDevice::of_slots(slots_of_data_units(1)),
        ),
        "每块盘段外 0 对（0 槽）< 1 个数据单元（2 槽）"
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", pool.session().allocator) == format!("{allocator_before:?}"),
        "分配器与发布之前逐项相同"
    );
}

/// 「落得下」的另一样：数据单元落得下、提交内生块落不下。两块单元区 384 槽的小盘，第一个文件之后（mkfs 同一个进程那条会话），
/// 每块盘上除了聚簇段外最低的那一对空槽，别的没挡的槽全标成已分配（位图上占着、不写记录，只供用例造盘面）。
/// 覆盖写一次：数据单元 1 个落那一对；提交内生块要 extent 根 1 槽、inode 叶容器 2 槽、inode 根 1 槽，加固定点按 ckpt_cost 计，
/// 而数据单元取走那一对之后一槽都不剩——交回 `SpaceAdmissionRefused`，每块盘报剩下 0 槽 < 提交内生块要的槽；两块盘逐字节不变、分配器不动。
/// 判别力：「落得下」那一判不判提交内生块时，数据单元落下、extent 根取不到落点，交回 `PlacementRefused { ExtentRoot, NoFreeSlotOnAnyDevice }`。
#[test]
fn a_write_whose_data_unit_lands_but_whose_commit_generated_units_find_no_unblocked_slot_is_refused_by_the_space_admission_before_any_write(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    let pair_left_free = {
        let allocator = &pool.session().allocator;
        let device_map = &allocator.devices[0];
        let first_slot = device_map.unit_area_start().slot().0;
        let past_the_last_slot = first_slot + device_map.unit_area_slots();
        (first_slot..past_the_last_slot)
            .step_by(2)
            .find(|pair_start| {
                pair_start + 1 < past_the_last_slot
                    && allocator.devices.iter().all(|each_device_map| {
                        each_device_map.is_free(SlotNumber(*pair_start))
                            && each_device_map.is_free(SlotNumber(pair_start + 1))
                    })
                    && !allocator.cluster_segments().iter().any(|segment_start| {
                        *pair_start >= segment_start.0
                            && *pair_start < segment_start.0 + CLUSTER_SEGMENT_SLOTS
                    })
            })
            .expect("第一个文件之后段外还有空槽对")
    };
    {
        let session = pool.session.as_mut().expect("mkfs 那条会话");
        for device_map in &mut session.allocator.devices {
            let first_slot = device_map.unit_area_start().slot().0;
            let past_the_last_slot = first_slot + device_map.unit_area_slots();
            for slot in first_slot..past_the_last_slot {
                if slot != pair_left_free
                    && slot != pair_left_free + 1
                    && device_map.is_free(SlotNumber(slot))
                {
                    device_map.mark_allocated(SlotNumber(slot), 1);
                }
            }
        }
    }
    let allocator_before = pool.session().allocator.clone();
    assert_eq!(
        free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount(&allocator_before),
        vec![(DeviceIdentity(0), 1, 0), (DeviceIdentity(1), 1, 0)],
        "每块盘只剩聚簇段外那一对"
    );
    let current = pool.current_file_version().clone();
    let checkpoint_cost =
        checkpoint_cost_of_the_version_to_build_on(Some(&current), &allocator_before);
    let image_before = pool.image();
    pool.write_time_seconds += 1;
    let content = content_of(OVERWRITE_BYTES, pool.write_time_seconds);
    let refused = {
        let session = pool.session.as_mut().expect("mkfs 那条会话");
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut session.allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: pool.write_time_seconds,
            },
            session.instance,
        )
    };
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &refused else {
        panic!(
            "覆盖写：准入先拒（提交内生块落不下）：{:?}",
            refused.as_ref().map(|version| version.root.checkpoint_txg)
        );
    };
    let commit_generated_slots = TransactionUnit::ExtentRoot.span_slots()
        + TransactionUnit::InodeLeafContainer(
            singlefs_core::inode_tree::InodeLeafContainerIndexInTree::LEFTMOST,
        )
        .span_slots()
        + TransactionUnit::InodeRoot.span_slots()
        + checkpoint_cost.0;
    assert_eq!(
        refusal.short_devices,
        the_same_shortage_on_both_devices(
            AvailableBytesOnOneDevice(0),
            BytesOnOneDevice::of_slots(commit_generated_slots),
        ),
        "每块盘数据单元取走那一对之后剩 0 槽 < 提交内生块 {commit_generated_slots} 槽（普通分配 4 槽加 ckpt_cost {checkpoint_cost:?}）"
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", pool.session().allocator) == format!("{allocator_before:?}"),
        "分配器与发布之前逐项相同"
    );
}

/// 第一格那一形，会话的准入用只供测试的开关关掉（`SpaceAdmission::SkippedByTheTestOnlySwitch`：式子与落得下两判都不判）：
/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 200 次空发布之后，经会话把文件顺序写到 2、3……25 个数据单元，都一次做成；
/// 写到 26 个单元：走固定点时在分配器的拷贝上取不到数据单元 19 的落点，交回 `PlacementRefused { Data(19), NoFreeSlotOnAnyDevice }`，
/// 会话照 D16（发布语义） 已定项 1「准入」那一行（准入放行而落点取不到也推）推一串抬 F、回收之后再发，做成——推过的只这一串、
/// 推它的是那一次落点被拒；F 抬上去了，池级 checker 0 违例。
/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 26 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。
/// A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「80 次空发布、写到 6 个、第 7 个推」，与第一格同一次改（草稿探针在新几何上现跑的数）；
/// 环窗口真扣、小盘放大 815 槽（用户 2026-10-02 定「甲」）之前是「写到 6 个、数据单元 0 被拒」，放大之后 200、400 次空发布都是写到 26 个、
/// 数据单元 19 被拒（三·2 第 6 条实现员副本上的草稿探针）。
#[test]
fn with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    pool.session
        .as_mut()
        .expect("这次挂载的会话")
        .allocator
        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    for _ in 0..200 {
        publish_an_empty_version_directly_on_the_session(&mut pool);
    }
    for data_units in 2..=25 {
        let published = pool
            .sequential_write(data_units * data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("经会话顺序写到 {data_units} 个数据单元做成：{refusal:?}")
            });
        assert!(
            published.floor_raises.is_empty(),
            "顺序写到 {data_units} 个数据单元一次做成，没推抬 F"
        );
    }
    let floor_before = pool.rollback_floor();
    let published = pool
        .sequential_write(26 * data_unit_payload_capacity())
        .unwrap_or_else(|refusal| {
            panic!("经会话顺序写到 26 个数据单元，推过抬 F 之后做成：{refusal:?}")
        });
    assert!(
        matches!(
            published.refusals_that_pushed_the_floor_raises.as_slice(),
            [PublishError::PlacementRefused {
                unit: TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile(19)),
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            }]
        ),
        "推过一串抬 F，推它的是数据单元 19 的落点被拒：{:?}",
        published.refusals_that_pushed_the_floor_raises
    );
    assert!(
        pool.rollback_floor() > floor_before,
        "F 从 {floor_before:?} 抬上去了"
    );
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "推过抬 F 再写成之后池级 checker 0 违例"
    );
}
