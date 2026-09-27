//! 实审 A4d（C545（空间准入罩不住分裂与聚簇段层），用户 2026-09-27 定「准入先拒」）：发布路径在走分配记录树的固定点、在分配器的拷贝上
//! 取落点之前，先判这次的单元落不落得下（`singlefs_core::admission::admit_the_units_landing_on_every_device`）——数据单元要这次挂载开过的
//! 聚簇段之外成对的空槽，提交内生块要没挡的槽——不够交回 `PublishError::SpaceAdmissionRefused`，在任何写之前，两块盘逐字节不变、分配器不动。
//! 实审 A4c 造出来的两格（报告 `research/prompts/m2-rev-a4c-implementer-report.md` 第三节）此前交回 `PlacementRefused`，这里钉成准入先拒；
//! 另加一格数据单元落得下、提交内生块落不下的；再加一格对照：准入用只供测试的开关关掉时落点那一道照样兜着，会话推过抬 F 再发成。
//! 盘都是内存里的稀疏盘，两块、单元区 256 / 384 槽（`HistoryDeviceWidth` 那两档）。

mod common_admission;

use common_admission::{
    checker_violations_on, content_of, start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceShortOfDemand, SpaceAdmission,
};
use singlefs_core::allocator::{PlacementRefusal, PoolAllocator};
use singlefs_core::journal::back_chain_of;
use singlefs_core::transaction::{
    publish_overwrite, publish_sequential_write, publish_version, FirstFile, InstanceTablePlan,
    PoolVersion, PoolWriter, PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

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

/// 这次挂载的会话上直接调发布路径把文件顺序写成 `data_units` 个数据单元（`transaction::publish_sequential_write`），不经会话的推抬 F：
/// 做成就换上新的一版；被拒原样交回。
fn write_the_file_sequentially_directly_on_the_session(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    data_units: usize,
) -> Result<TransactionOutput, PublishError> {
    pool.write_time_seconds += 1;
    let content = content_of(
        data_units * data_unit_payload_capacity(),
        pool.write_time_seconds,
    );
    let session = pool.session.as_mut().expect("这次挂载的会话");
    let PoolVersion::WithFile(current) = &session.current else {
        panic!("第一个文件之后现行那一版带文件");
    };
    let current = current.clone();
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let outcome = publish_sequential_write(
        &mut writer,
        &mut session.allocator,
        &current,
        FirstFile {
            content: &content,
            write_time_seconds: pool.write_time_seconds,
        },
        session.instance,
    );
    if let Ok(published) = &outcome {
        session.current = PoolVersion::WithFile(published.clone());
    }
    outcome
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

/// C545 那一格（实审 A4c 造出来的，报告第三节）：用户数据落得下的空槽对全在这次挂载开过的聚簇段里、段外一对都没有，而按字节算的式子放得下这次的需求。
/// 两块单元区 384 槽的小盘，第一个文件之后崩了再挂；这次挂载里直接推 80 次空发布（固定点在这次挂载开的聚簇段里 bump，根环转过之后回收，
/// 段里空出来的槽照旧只给提交内生块，D3（空间分配） 已定项 8 第 2 条），再直接把文件顺序写到 6 个数据单元，都做成。
/// 接着顺序写到 7 个单元：每块盘段外成对的空槽 0 对、段内 79 对；按字节算的式子（`admission_reading_before_a_publish`）每块盘可用 49 槽，
/// 放得下这次需求的上界（6 个单元那次新写的槽 + 多一个数据单元 2 槽 + 一次空发布的 ckpt_cost）。发布路径在走固定点之前判「这次的单元落得下」：
/// 7 个数据单元要 7 对、段外 0 对，交回 `SpaceAdmissionRefused`，每块盘报段外 0 槽 < 数据单元 14 槽；两块盘逐字节不变、分配器不动。
/// 判别力：「落得下」那一判不判数据单元（或整判拿掉）时，走固定点在分配器的拷贝上取不到数据单元 0 的落点，交回 `PlacementRefused`（实审 A4c 钉的那个）。
#[test]
fn a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_the_space_admission_before_any_write_while_the_byte_formula_admits(
) {
    const EMPTY_PUBLISHES_IN_THIS_MOUNT: usize = 80;
    const DATA_UNITS_THAT_STILL_LAND: usize = 6;
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for _ in 0..EMPTY_PUBLISHES_IN_THIS_MOUNT {
        publish_an_empty_version_directly_on_the_session(&mut pool);
    }
    let mut last_write_that_landed = None;
    for data_units in 2..=DATA_UNITS_THAT_STILL_LAND {
        last_write_that_landed = Some(
            write_the_file_sequentially_directly_on_the_session(&mut pool, data_units)
                .unwrap_or_else(|refusal| {
                    panic!("顺序写到 {data_units} 个数据单元做成：{refusal:?}")
                }),
        );
    }
    let last_write_that_landed = last_write_that_landed.expect("至少顺序写过一次");
    let allocator_before = pool.session().allocator.clone();
    assert_eq!(
        free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount(&allocator_before),
        vec![(DeviceIdentity(0), 0, 79), (DeviceIdentity(1), 0, 79)],
        "每块盘成对的空槽：这次挂载开过的聚簇段外 0 对、段内 79 对"
    );
    let demand_upper_bound_in_slots = last_write_that_landed
        .rewritten
        .iter()
        .map(|role| role.span_slots())
        .sum::<u64>()
        + slots_of_data_units(1)
        + checkpoint_cost_of_the_version_to_build_on(
            Some(&last_write_that_landed),
            &allocator_before,
        )
        .0;
    let available_in_slots: Vec<(DeviceIdentity, i128)> =
        admission_reading_before_a_publish(&allocator_before, Some(&last_write_that_landed))
            .available_on_each_device()
            .into_iter()
            .map(|(device, available)| (device, available.0 / i128::from(SLOT_BYTES)))
            .collect();
    assert!(
        available_in_slots
            .iter()
            .all(|(_, available)| *available >= i128::from(demand_upper_bound_in_slots)),
        "按字节算的式子每块盘都放得下这次需求的上界 {demand_upper_bound_in_slots} 槽：{available_in_slots:?}"
    );
    let image_before = pool.image();
    let refused = write_the_file_sequentially_directly_on_the_session(
        &mut pool,
        DATA_UNITS_THAT_STILL_LAND + 1,
    );
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &refused else {
        panic!(
            "顺序写到 {} 个单元：准入先拒（这次的数据单元在段外落不下）：{:?}",
            DATA_UNITS_THAT_STILL_LAND + 1,
            refused.as_ref().map(|version| version.root.checkpoint_txg)
        );
    };
    assert_eq!(
        refusal.short_devices,
        the_same_shortage_on_both_devices(
            AvailableBytesOnOneDevice(0),
            BytesOnOneDevice::of_slots(slots_of_data_units(7)),
        ),
        "每块盘段外 0 对（0 槽）< 7 个数据单元（14 槽）"
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", pool.session().allocator) == format!("{allocator_before:?}"),
        "分配器与发布之前逐项相同"
    );
}

/// C545 的另一格（实审 A4c 第三节：式子可用 −14 槽那一格，此前式子也轮不到判、交回的是 `PlacementRefused`）：两块单元区 256 槽的小盘，
/// 第一个文件之后崩了再挂，这次挂载里直接推 40 次空发布；每块盘段外成对的空槽 0 对、段内 48 对。把文件顺序写到 2 个单元：
/// 发布路径在走固定点之前判「这次的单元落得下」，交回 `SpaceAdmissionRefused`，每块盘报段外 0 槽 < 数据单元 4 槽——报的是落得下那一判，
/// 不是式子（式子那一判每块盘可用 −14 槽，报的会是它）；两块盘逐字节不变、分配器不动。
/// 判别力：「落得下」那一判不判数据单元时，走固定点取不到数据单元 0 的落点，交回 `PlacementRefused`。
#[test]
fn a_write_on_the_narrower_pool_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_for_its_data_units_before_any_write(
) {
    const EMPTY_PUBLISHES_IN_THIS_MOUNT: usize = 40;
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf256Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for _ in 0..EMPTY_PUBLISHES_IN_THIS_MOUNT {
        publish_an_empty_version_directly_on_the_session(&mut pool);
    }
    let allocator_before = pool.session().allocator.clone();
    let current = pool.current_file_version().clone();
    assert_eq!(
        free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount(&allocator_before),
        vec![(DeviceIdentity(0), 0, 48), (DeviceIdentity(1), 0, 48)],
        "每块盘成对的空槽：这次挂载开过的聚簇段外 0 对、段内 48 对"
    );
    assert_eq!(
        admission_reading_before_a_publish(&allocator_before, Some(&current))
            .available_on_each_device()
            .into_iter()
            .map(|(device, available)| (device, available.0 / i128::from(SLOT_BYTES)))
            .collect::<Vec<_>>(),
        vec![(DeviceIdentity(0), -14), (DeviceIdentity(1), -14)],
        "按字节算的式子每块盘可用 −14 槽"
    );
    let image_before = pool.image();
    let refused = write_the_file_sequentially_directly_on_the_session(&mut pool, 2);
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &refused else {
        panic!(
            "顺序写到 2 个单元：准入先拒（这次的数据单元在段外落不下）：{:?}",
            refused.as_ref().map(|version| version.root.checkpoint_txg)
        );
    };
    assert_eq!(
        refusal.short_devices,
        the_same_shortage_on_both_devices(
            AvailableBytesOnOneDevice(0),
            BytesOnOneDevice::of_slots(slots_of_data_units(2)),
        ),
        "每块盘段外 0 对（0 槽）< 2 个数据单元（4 槽），不是式子的可用 −14 槽"
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
/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 80 次空发布之后，经会话把文件顺序写到 2、3……6 个数据单元，都一次做成；
/// 写到 7 个单元：走固定点时在分配器的拷贝上取不到数据单元 0 的落点，交回 `PlacementRefused { Data(0), NoFreeSlotOnAnyDevice }`，
/// 会话照 D16（发布语义） 已定项 1「准入」那一行（准入放行而落点取不到也推）推一串抬 F、回收之后再发，做成——推过的只这一串、
/// 推它的是那一次落点被拒；F 抬上去了，池级 checker 0 违例。
/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 7 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。
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
    for _ in 0..80 {
        publish_an_empty_version_directly_on_the_session(&mut pool);
    }
    for data_units in 2..=6 {
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
        .sequential_write(7 * data_unit_payload_capacity())
        .unwrap_or_else(|refusal| {
            panic!("经会话顺序写到 7 个数据单元，推过抬 F 之后做成：{refusal:?}")
        });
    assert!(
        matches!(
            published.refusals_that_pushed_the_floor_raises.as_slice(),
            [PublishError::PlacementRefused {
                unit: TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile::FIRST),
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            }]
        ),
        "推过一串抬 F，推它的是数据单元 0 的落点被拒：{:?}",
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
