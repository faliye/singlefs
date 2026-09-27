//! 里程碑「第二个事务」收尾实现批次实五：准入不够时先推抬 F 的空发布再判（D16（发布语义） 已定项 1「准入」那一行，
//! C283（准入失败时不先推发布就报 ENOSPC）），挂载那一处写行之后也推，推满仍不够怎么收尾（C565（挂载处推满仍不够怎么收尾没定），
//! 实现员提的那一种），发布路径的切换预留按下一次挂载的 rows0（D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」），
//! 与算抬 F 上限时根槽这一次读坏（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，C562（抬 F 算上限时根槽读坏当没有根））。
//!
//! 用例的形状照安全设计轮第三轮攻方（`research/prompts/m2-safety-r3-opus-output.md`）：会话里准入放行而落点取不到、删掉之后同样大小的写回、
//! 只崩了再挂的窄池、实例表行数逼近 369 的整数倍；445 那一格照实四丙的构造（种子基 + 339 第 1 步）。
//! 盘都是内存里的稀疏盘，两块、单元区 240 / 256 / 384 槽（`HistoryDeviceWidth` 那几档），一个进程里 mkfs、取号、暖机、第一个文件起步。
//! 崩在准入推的那一串空发布中间那一形在 `second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`（层 0 的枚举，单独一个测试二进制）。

mod common_admission;

use common_admission::{
    checker_violations_on, content_of, space_admission_of_a_crash_remount_on_a_copy, start_plain,
    DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on, SpaceAdmission,
};
use singlefs_core::allocator::{PlacementRefusal, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling, ring_slots_known_to_hold_a_root_by,
    rollback_floor_ceiling, unmount, FloorRaiseStop, MountError, MountSpaceAdmission,
    RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::UserChangeRefused;
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, BadRootRingSlotReading,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::transaction::{
    publish_sequential_write, publish_version, FirstFile, InstanceTablePlan, PoolVersion,
    PoolWriter, PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
use singlefs_harness::crash::{SparseBlockDevice, SparseDevice};
use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::{
    execute_history_with_faults, generate_history_with_weights, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    StepOutcome, StepPosition,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 会话里准入放行、落点取不到（攻方「会话里写」那一形）：两块单元区 384 槽的小盘，第一个文件之后崩了再挂，在这次挂载里每次覆盖写 2999 字节。
/// 第 62 次覆盖写：数据单元取不到落点（发布路径算定这次的形状时就在分配器的拷贝上取落点，取不到在准入与任何写之前交回 `PlacementRefused`）——
/// defer 堆到数据单元的候选段里没有成对的空槽。会话先推一串抬 F 的空发布（D16（发布语义） 已定项 1「准入」那一行：准入放行而落点取不到
/// 也推），F 抬到上限、回收之后再发，这次覆盖写做成；70 次覆盖写全做成，被落点拒之后推过抬 F 的第一次是第 62 次，
/// 最后的镜像池级 checker 0 违例。ckpt_cost 按最坏情况计（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定）之后第 18、32、47 次是式子先拒、
/// 推过再发成，落点那一道第一次走到是第 62 次（实审 A4 按每块盘一条叶路径计的那一版在同一份树上是第 35 次）。
/// 判别力：只在准入拒时推（落点被拒原样交回）时，第 62 次报 `UserChangeRefused::Publish { PlacementRefused }`。
#[test]
fn an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    let mut overwrites_that_pushed_after_a_placement_refusal: Vec<usize> = Vec::new();
    for overwrite_index in 1..=70 {
        let floor_before = pool.rollback_floor();
        let published = pool
            .overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
        let pushed_after_a_placement_refusal = published
            .refusals_that_pushed_the_floor_raises
            .iter()
            .any(|refusal| matches!(refusal, PublishError::PlacementRefused { .. }));
        if pushed_after_a_placement_refusal {
            assert!(
                pool.rollback_floor() > floor_before,
                "第 {overwrite_index} 次：推过抬 F，F 从 {floor_before:?} 抬上去了"
            );
            overwrites_that_pushed_after_a_placement_refusal.push(overwrite_index);
        }
    }
    assert_eq!(
        overwrites_that_pushed_after_a_placement_refusal.first(),
        Some(&62),
        "第一次被落点拒、推过抬 F 再做成的是第 62 次覆盖写：{overwrites_that_pushed_after_a_placement_refusal:?}"
    );
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "70 次覆盖写之后池级 checker 0 违例"
    );
}

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

/// 每块盘上两槽都空（没分配、没隔离、没扣住）的偶数起点槽对：落在这次挂载开过的聚簇段外的几对、段内的几对。
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

/// C545（空间准入罩不住分裂与聚簇段层）那一格：用户数据落得下的空槽对全在这次挂载开过的聚簇段里、段外一对都没有，而按字节算的式子放得下这次的需求。
/// 两块单元区 384 槽的小盘，第一个文件之后崩了再挂；这次挂载里直接推 80 次空发布（固定点在这次挂载开的聚簇段里 bump，根环转过之后回收，
/// 段里空出来的槽照旧只给提交内生块，D3（空间分配） 已定项 8 第 2 条），再直接把文件顺序写到 6 个数据单元，都做成。
/// 接着顺序写到 7 个单元：每块盘段外成对的空槽 0 对、段内 79 对；按字节算的式子（`admission_reading_before_a_publish`）每块盘可用 49 槽，
/// 放得下这次需求的上界（6 个单元那次新写的槽 + 多一个数据单元 2 槽 + 一次空发布的 ckpt_cost）。发布路径算定这次的形状时就在分配器的拷贝上
/// 取落点（`transaction::settle_the_allocation_record_tree`），数据单元 0 取不到，在准入与任何写之前交回
/// `PlacementRefused { Data(0), NoFreeSlotOnAnyDevice }`，两块盘逐字节不变。
/// 判别力：用户数据的候选落点不排除这次挂载开过的聚簇段时，第 7 个单元落进段里、这次写做成。
#[test]
fn a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_placement_before_any_write_while_the_byte_formula_admits(
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
    let allocator = pool.session().allocator.clone();
    assert_eq!(
        free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount(&allocator),
        vec![(DeviceIdentity(0), 0, 79), (DeviceIdentity(1), 0, 79)],
        "每块盘成对的空槽：这次挂载开过的聚簇段外 0 对、段内 79 对"
    );
    let demand_upper_bound_in_slots = last_write_that_landed
        .rewritten
        .iter()
        .map(|role| role.span_slots())
        .sum::<u64>()
        + TransactionUnit::Data(DataUnitIndexInFile(0)).span_slots()
        + checkpoint_cost_of_the_version_to_build_on(Some(&last_write_that_landed), &allocator).0;
    let available_in_slots: Vec<(DeviceIdentity, i128)> =
        admission_reading_before_a_publish(&allocator, Some(&last_write_that_landed))
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
    assert!(
        matches!(
            &refused,
            Err(PublishError::PlacementRefused {
                unit: TransactionUnit::Data(DataUnitIndexInFile(0)),
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            })
        ),
        "顺序写到 {} 个单元：数据单元 0 取不到落点：{:?}",
        DATA_UNITS_THAT_STILL_LAND + 1,
        refused.as_ref().map(|version| version.root.checkpoint_txg)
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
}

/// 删掉之后同样大小的写回（D3（空间分配） 已定项 9 第 2 条：删掉 s 字节之后同样大小的写在 3 次改变用户可见状态的发布内做成）：
/// 两块单元区 256 槽的小盘，崩了再挂之后先覆盖写 20 次，再用顺序写把文件一个数据单元一个数据单元地长大，长到会话报空间不够（6 个单元：
/// 第 7 个单元落点取不到，推抬 F 推到上限仍取不到）；截断成 0 字节（删），再写回 6 个单元。写回被拒时发一次改变用户可见状态的小发布
/// （建一个空 inode）再试：第 1、2、3 次写回被拒（推抬 F 推到上限仍不够），各发一次建 inode 之后，第 4 次写回做成——
/// 删与写回之间夹 3 次改变用户可见状态的发布，在界 3 之内（条款逐字：删掉的块「要再发生 3 次这样的发布才回可分配集合」，写回接在那之后）。
/// ckpt_cost 按最坏情况计（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定）之前这一格夹 1 到 2 次；之后保留池与切换预留多扣，
/// 写回等到第 3 次之后。写回之后池级 checker 0 违例。
/// 判别力：落点被拒不推时，写回一次都做不成（推不出被钉着的块之外的空间）。
#[test]
fn after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes() {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf256Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for overwrite_index in 1..=20 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    let unit_bytes = data_unit_payload_capacity();
    let mut units = 0;
    let growth_refusal = loop {
        match pool.sequential_write((units + 1) * unit_bytes) {
            Ok(_) => units += 1,
            Err(refusal) => break refusal,
        }
        assert!(units < 100, "256 槽的盘长不到 100 个单元");
    };
    assert_eq!(units, 6, "长到 6 个单元，第 7 个装不下");
    assert!(
        matches!(
            &growth_refusal,
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)
                if matches!(no_space.last_refusal, PublishError::PlacementRefused { .. })
                    && matches!(no_space.stop, FloorRaiseStop::FloorAlreadyAtTheCeiling { .. })
        ),
        "长大被拒：落点取不到、推抬 F 推到上限仍取不到：{growth_refusal:?}"
    );
    pool.overwrite(0).expect("截断成 0 字节（删）做成");
    let mut user_visible_publishes_after_the_truncation = 0;
    let mut refused_write_backs = 0;
    loop {
        let write_back = pool.sequential_write(units * unit_bytes);
        match write_back {
            Ok(_) => {
                user_visible_publishes_after_the_truncation += 1;
                break;
            }
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => {
                refused_write_backs += 1;
                assert!(refused_write_backs <= 7, "写回在 8 次之内做成");
                pool.create_one_inode()
                    .expect("建一个空 inode（改变用户可见状态的小发布）做成");
                user_visible_publishes_after_the_truncation += 1;
            }
            Err(refusal) => panic!("写回只会因为空间不够被拒：{refusal:?}"),
        }
    }
    assert_eq!(
        (
            refused_write_backs,
            user_visible_publishes_after_the_truncation
        ),
        (3, 4),
        "写回被拒三次、建三次 inode，删之后第 4 次改变用户可见状态的发布（写回本身）做成"
    );
    let user_visible_publishes_between_the_truncation_and_the_write_back =
        user_visible_publishes_after_the_truncation - 1;
    assert!(
        user_visible_publishes_between_the_truncation_and_the_write_back <= 3,
        "D3（空间分配） 已定项 9 第 2 条的界：删掉的块要再发生 3 次改变用户可见状态的发布才回可分配集合，删与写回之间至多夹 3 次"
    );
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "写回之后池级 checker 0 违例"
    );
}

/// 只崩了再挂、一个字节不写（攻方「只崩了再挂」那一形）：两块单元区 240 槽的小盘，第一个文件之后连着崩了再挂 15 次。
/// 每次挂载写行与暖机，换下的固定点进 defer，崩了再挂不抬 F，环里留着约 6 次挂载的 defer；取号之前那一判从第 8 次起不够。
/// 不推的话第 8 次在取号之前被拒、之后每一次判的都是同一份盘，挂死在那里。可写挂载准入不够时写行之后推抬 F 的空发布再判：
/// ckpt_cost 按最坏情况计（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定；窄盘更早报 ENOSPC 用户认了）之后，第 8 次起每次各推两串
/// （一次准入的预算做满）仍不够、挂载照样做成（C565（挂载处推满仍不够怎么收尾没定） 那一种收尾，D2（RAID 条带策略） 已定项 13 那一格的例外），
/// 这块盘上再也回不到「够」。15 次挂载全做成，每次挂载之后池级 checker 0 违例，F 只升不降。
/// 判别力：挂载那一处不推时，第 8 次在取号之前被拒，之后每一次都被拒。
#[test]
fn crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf240Slots);
    let mut admissions: Vec<&'static str> = Vec::new();
    let mut floor_before = CheckpointTxg(0);
    for mount_index in 1..=15 {
        let output = pool
            .crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
        admissions.push(match output.space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition => "取号之前就够",
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => "推了再判够了",
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => "推满仍不够",
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "没判",
        });
        assert!(
            pool.rollback_floor() >= floor_before,
            "第 {mount_index} 次：F 只升不降（{floor_before:?} → {:?}）",
            pool.rollback_floor()
        );
        floor_before = pool.rollback_floor();
        assert_eq!(
            checker_violations_on(&pool.image()),
            Vec::<String>::new(),
            "第 {mount_index} 次挂载之后池级 checker 0 违例"
        );
    }
    assert_eq!(
        admissions,
        [["取号之前就够"; 7].as_slice(), &["推满仍不够"; 8]].concat(),
        "第 8 次起每次推满仍不够、挂载照样做成"
    );
}

/// 推满仍不够怎么收尾（C565（挂载处推满仍不够怎么收尾没定）；实现员提的那一种，交主 agent 定）：挂载照样做成——实例已取、行已写、
/// 暖机与推的那几串都已落盘，交回 `MountSpaceAdmission::StillShortAfterTheFloorRaises`；之后的发布照发布路径的准入判，不够就报空间不够；
/// 正常卸载照常可走（它抬 F 到现行那一版、不判上限），卸载之后再挂取号之前就够。
/// 这一格：两块单元区 240 槽的小盘，第一个文件之后连着崩了再挂，第 10 次取号之前每块盘短 36 槽；写行与暖机之后推两串（F 抬到 20、再到 23，
/// 各 3 次空发布），再判每块盘仍短 34 槽，再推一串就是 6 + 3 + 1 = 10 次发布、超过一次准入的 8 次，不再推（ckpt_cost 按最坏情况计，
/// D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定：第 8、9 次挂载已经推满仍不够、各推两串，F 在第 10 次之前已到 14）。这次挂载的会话里覆盖写一次：
/// 推两串（又做满预算）仍不够，报空间不够（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`）。正常卸载做成，卸载之后再挂取号之前就够、
/// 池级 checker 0 违例。
/// 判别力：换成另一种收尾（推满仍不够就报可写挂载被拒），第 10 次挂载交回错误，用例在那一步就红。
#[test]
fn mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf240Slots);
    for mount_index in 1..=9 {
        pool.crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
    }
    let output = pool
        .crash_and_mount_writable()
        .expect("第 10 次：推满仍不够，挂载照样做成");
    let MountSpaceAdmission::StillShortAfterTheFloorRaises {
        refusal_before_acquisition,
        floor_raises,
        last_refusal,
        stop,
    } = &output.space_admission
    else {
        panic!("第 10 次推满仍不够：{:?}", output.space_admission);
    };
    let slots_short = |refusal: &singlefs_core::admission::AdmissionRefusedOnSomeDevices| {
        refusal
            .short_devices
            .iter()
            .map(|short| {
                (
                    short.device,
                    short.available.0 / i128::from(singlefs_format::SLOT_BYTES),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        slots_short(refusal_before_acquisition),
        vec![(DeviceIdentity(0), -36), (DeviceIdentity(1), -36)],
        "取号之前每块盘短 36 槽"
    );
    assert_eq!(
        floor_raises
            .iter()
            .map(|raised| (raised.ceiling, raised.publishes.len()))
            .collect::<Vec<_>>(),
        vec![(CheckpointTxg(20), 3), (CheckpointTxg(23), 3)],
        "写行与暖机之后推两串：F 抬到 20、再到 23"
    );
    assert_eq!(
        slots_short(last_refusal),
        vec![(DeviceIdentity(0), -34), (DeviceIdentity(1), -34)],
        "推两串之后再判每块盘仍短 34 槽"
    );
    assert!(
        matches!(
            stop,
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
                publishes_pushed: 6,
                publishes_of_the_next_raise: 3,
            }
        ),
        "再推一串就超过一次准入的 8 次发布：{stop:?}"
    );
    assert_eq!(pool.rollback_floor(), CheckpointTxg(23), "现行那一版带新 F");
    let write = pool.overwrite(OVERWRITE_BYTES);
    assert!(
        matches!(
            &write,
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space))
                if matches!(no_space.last_refusal, PublishError::SpaceAdmissionRefused(_))
                    && matches!(
                        no_space.stop,
                        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
                    )
        ),
        "这次挂载的会话里写：推满仍不够，报空间不够：{write:?}"
    );
    let mut session = pool.session.take().expect("第 10 次挂载的会话还在");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("正常卸载做成");
    assert!(
        matches!(unmounted, Unmounted::FloorRaisedToTheCurrentVersion(_)),
        "卸载推了一串带卸载记号的空发布"
    );
    let after_unmount = pool.crash_and_mount_writable().expect("卸载之后再挂");
    assert!(
        matches!(
            after_unmount.space_admission,
            MountSpaceAdmission::AdmittedBeforeAcquisition
        ),
        "卸载之后再挂取号之前就够：{:?}",
        after_unmount.space_admission
    );
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "卸载之后再挂的镜像池级 checker 0 违例"
    );
}

/// 上一版树表 0 条（只做过 mkfs、还没有文件）的可写挂载准入不够：抬 F 要带文件的现行版本、这一版上没有可退的带文件版本，推不出空间，
/// 照旧在取号之前拒（`MountError::SpaceAdmissionRefusedBeforeAcquisition`），一个写都没发、两块盘逐字节不变（两块盘系统配置里的实例代号还是 0）。
/// 这一格：两块单元区只有 16 槽的盘上 mkfs（实例表一片 2 槽、树表 1 槽），挂载判实例切换的预留（每块盘 4 × (2 + 3 × c_max) 槽）拿不到。
/// 判别力：树表 0 条这一格也照带文件的那一格写行之后推，挂载就取了号、写了盘。
#[test]
fn writable_mount_on_a_version_without_file_short_of_its_reserve_is_refused_before_acquisition_with_the_disk_unchanged(
) {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let device_bytes = (singlefs_format::UNIT_AREA_START_SLOT + 16) * singlefs_format::SLOT_BYTES;
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("单元区 16 槽的盘上 mkfs 做得成");
    let operations_after_mkfs = stream.operation_count();
    let image_after_mkfs: Vec<SparseDevice> = devices
        .iter()
        .map(|(_, device)| device.sparse_image().clone())
        .collect();
    let refused = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {
        instance_to_acquire,
        refusal,
    }) = refused
    else {
        panic!(
            "树表 0 条的一版上准入不够在取号之前拒：{:?}",
            refused.as_ref().err()
        );
    };
    assert_eq!(instance_to_acquire, InstanceGeneration(1));
    assert_eq!(refusal.short_devices.len(), 2, "两块盘都短");
    assert_eq!(
        stream.operation_count(),
        operations_after_mkfs,
        "一个写、一道屏障都没发"
    );
    assert!(
        devices
            .iter()
            .zip(&image_after_mkfs)
            .all(|((_, device), before)| device.sparse_image() == before),
        "两块盘逐字节不变"
    );
}

/// 实例表行数逼近 369 的整数倍（攻方「挂载路径的 rows0」那一形，D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」）：两块单元区 384 槽的小盘，
/// 第一个文件之后连着崩了再挂 366 次（mkfs 那个进程的实例 1 不写行，每次挂载写一行），把这次挂载的 rows0 推到 366（⌈(366 + 3) ÷ 369⌉ = 1 片），下一次挂载写一行、rows0 = 367，
/// 链要两片、切换预留每块盘多 8 槽。在这次挂载里一直覆盖写到会话报空间不够（至多 200 次）：发布路径的读数按下一次挂载的 367 行留份额，
/// 每一次覆盖写做成之后，在盘面的拷贝上只崩了再挂，取号之前就够（一次都不用推）。
/// 判别力：发布路径的切换预留照旧按这次挂载的 366 行算时，会话放行得更靠满，有一次覆盖写之后崩了再挂取号之前不够。
#[test]
fn at_366_instance_rows_every_admitted_overwrite_leaves_the_next_mounts_row_so_a_crash_remount_is_admitted_before_acquisition(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    for mount_index in 1..=366 {
        pool.crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
    }
    assert_eq!(
        pool.session()
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        366,
        "这次挂载的 rows0"
    );
    let mut admitted_overwrites = 0;
    for overwrite_index in 1..=200 {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(_) => admitted_overwrites += 1,
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => break,
            Err(refusal) => {
                panic!("第 {overwrite_index} 次覆盖写只会因为空间不够被拒：{refusal:?}")
            }
        }
        let remount = space_admission_of_a_crash_remount_on_a_copy(&pool);
        assert!(
            matches!(remount, Ok(MountSpaceAdmission::AdmittedBeforeAcquisition)),
            "第 {overwrite_index} 次覆盖写做成之后崩了再挂，取号之前就够：{remount:?}"
        );
    }
    assert!(
        admitted_overwrites >= 1,
        "这次挂载里至少做成一次覆盖写（不然这一格什么都没判）"
    );
}

/// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）用的池：mkfs 同一个进程里取号、暖机、第一个文件，
/// 盘外面包一层注入故障的（计划先不开）。根环这时：(区域 0, 槽 0) 是 mkfs 的第 0 代根（txg 0，区域 1、2 的槽 0 已被暖机那两条盖掉），
/// 第一个文件那条根在 (区域 0, 槽 1)。有效根里非空的不足 4 个，上限取最旧的有效根 = 0。
fn pool_for_the_ceiling_reading() -> (
    PoolUnderTest<FaultInjectingBlockDevice<SparseBlockDevice>>,
    SharedFaultPlan,
) {
    let plan = SharedFaultPlan::unarmed(HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry());
    let plan_for_the_devices = plan.clone();
    let pool = PoolUnderTest::start_after_the_first_file(
        HistoryDeviceWidth::UnitAreaOf384Slots,
        move |identity, device| {
            FaultInjectingBlockDevice::new(identity, device, plan_for_the_devices.clone())
        },
    );
    (pool, plan)
}

/// 根环 (区域 0, 槽 0) 在盘 0 上的偏移（第一版区域归属 0 / 1 / 0）。
fn offset_of_the_genesis_root_slot_on_device_zero(
    parameters: &MakeFilesystemParameters,
) -> DeviceOffsetInBytes {
    slot_offset(
        RootRingSlot { region: 0, slot: 0 },
        parameters.geometry.fixed_structure_slot_spacing,
    )
}

/// 只在盘 0 那个根槽的读上注入 `fault`，从第 `first_occurrence` 次命中起（`every_time_from_then_on` 为真时之后每次都注入）。
fn schedule_on_the_genesis_root_slot(
    parameters: &MakeFilesystemParameters,
    occurrence: FaultOccurrence,
) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
        placement: FaultPlacement::OffsetExactly(offset_of_the_genesis_root_slot_on_device_zero(
            parameters,
        )),
        counting: FaultCounting::AcrossThePool,
        occurrence,
    }
}

/// 算上限的输入：系统配置、现行那一版的实例表、这个进程知道住着根的根环槽。
fn ceiling_of(
    pool: &PoolUnderTest<FaultInjectingBlockDevice<SparseBlockDevice>>,
) -> Result<CheckpointTxg, MountError> {
    let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置读得出");
    let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
        .expect("实例表读得出")
        .records;
    rollback_floor_ceiling(
        &pool.devices,
        &system_configuration,
        pool.rollback_floor(),
        &table,
        &ring_slots_known_to_hold_a_root_by(&pool.session().allocator),
    )
}

/// 根槽这一次读坏、重读读出来（C562（抬 F 算上限时根槽读坏当没有根） 的正方向）：(区域 0, 槽 0) 是 mkfs 写下、这个进程的根环表里记着的槽，
/// 算上限那一次读它报一次读错，重读读得出，上限照常是 0（最旧的有效根就是它）。注入正好打中一次。
/// 判别力：读坏就当没有根（改之前的做法）时，最旧的有效根成了暖机那条 txg 1，上限算成 1，F 会被抬过条款上限（实四丙查出的那一格）。
#[test]
fn known_root_slot_that_reads_bad_once_is_reread_and_the_ceiling_is_unchanged() {
    let (pool, plan) = pool_for_the_ceiling_reading();
    assert_eq!(
        ceiling_of(&pool).expect("不注入时算得出上限"),
        CheckpointTxg(0)
    );
    assert!(
        ring_slots_known_to_hold_a_root_by(&pool.session().allocator)
            .contains(&RootRingSlot { region: 0, slot: 0 }),
        "mkfs 写下的第 0 代根那一槽，这个进程知道住着根"
    );
    plan.arm(schedule_on_the_genesis_root_slot(
        &pool.parameters,
        FaultOccurrence::TheNthMatchingCall(1),
    ));
    let ceiling = ceiling_of(&pool);
    plan.disarm();
    assert_eq!(plan.fired_count(), 1, "注入正好打中算上限那一读");
    assert_eq!(
        ceiling.expect("重读读得出，上限照常算"),
        CheckpointTxg(0),
        "重读读出第 0 代根，上限还是 0"
    );
}

/// 根槽这一次读坏、重读仍坏（C562 的反方向）：同一个池上抬 F 到 1，(区域 0, 槽 0) 的每一次读都报错：这个进程知道那里住着根，
/// 重读一次仍坏就拒这次抬 F，报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，不按有根或没根猜：
/// 一个写都没发（两块盘逐字节不变）、分配器与抬 F 之前逐项相同、现行那一版不动。
/// 判别力：读坏就当没有根时，上限算成 1、F 抬到 1、推了空发布——抬过了条款上限 0（checker 的 I-7.9 判红那一格）。
#[test]
fn known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any_write() {
    let (mut pool, plan) = pool_for_the_ceiling_reading();
    let image_before = pool.image();
    let mut session = pool.session.take().expect("mkfs 那条会话");
    let allocator_before = format!("{:?}", session.allocator);
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("第一个文件之后现行那一版带文件");
    };
    let current_before = current.clone();
    plan.arm(schedule_on_the_genesis_root_slot(
        &pool.parameters,
        FaultOccurrence::EVERY_MATCHING_CALL,
    ));
    let raised = raise_rollback_floor(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        CheckpointTxg(1),
        ShadowLedger::On,
    );
    plan.disarm();
    let Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
        ring_slot,
        first_reading,
        reread,
    }) = raised
    else {
        panic!(
            "重读仍坏就拒这次抬 F：{:?}",
            raised.as_ref().map(|raised| raised.ceiling).err()
        );
    };
    assert_eq!(
        (ring_slot, first_reading, reread),
        (
            RootRingSlot { region: 0, slot: 0 },
            BadRootRingSlotReading::Unreadable,
            BadRootRingSlotReading::Unreadable
        )
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", session.allocator) == allocator_before,
        "分配器与抬 F 之前逐项相同"
    );
    assert!(*current == current_before, "现行那一版不动");
}

/// 挂载那一刻就坏的槽照旧当没有根、不挡抬 F（C562 那一行的前半句）：同一个池进程退出，(区域 0, 槽 0) 从这时起每一次读都报错，
/// 再可写挂载——挂载读根环时它就读不出，这个进程的根环表里没有它。之后抬 F 到上限：算上限时它读不出、当没有根，最旧的有效根是暖机那条 txg 1，
/// 上限 1，F 抬到 1（`RaiseToTheAdmissionCeiling::Raised`），不因为这一槽被拒。
/// 判别力：读坏的槽不分挂载时读不读得出、一律重读仍坏就拒时，这次抬 F 被拒。
#[test]
fn root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise() {
    let (mut pool, plan) = pool_for_the_ceiling_reading();
    plan.arm(schedule_on_the_genesis_root_slot(
        &pool.parameters,
        FaultOccurrence::EVERY_MATCHING_CALL,
    ));
    pool.crash_and_mount_writable()
        .expect("那一槽读不出照样挂得上");
    assert!(
        !ring_slots_known_to_hold_a_root_by(&pool.session().allocator)
            .contains(&RootRingSlot { region: 0, slot: 0 }),
        "挂载那一刻读不出的槽不在这个进程的根环表里"
    );
    let mut session = pool.session.take().expect("挂载的会话");
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("挂载之后现行那一版带文件");
    };
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        ShadowLedger::On,
    );
    plan.disarm();
    let Ok(RaiseToTheAdmissionCeiling::Raised(raised)) = raised else {
        panic!("挂载时就坏的槽不挡抬 F：{raised:?}");
    };
    assert_eq!(
        raised.ceiling,
        CheckpointTxg(1),
        "最旧的有效根是暖机那条 txg 1"
    );
    assert!(
        raised
            .publishes
            .iter()
            .all(|publish| publish.root.rollback_floor == CheckpointTxg(1)),
        "这一串的根都带新 F = 1"
    );
}

/// 实四丙的构造做成用例（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行）：故障注入的种子基 + 339 那段历史
/// （宽泛的比重、两块 4 GiB 的盘、每一步之后跑池级 checker），第 1 步抬 F 到 1、条款上限 0（非空状态不足 4 个，取最旧的有效根 (0, 0)）。
/// 只在这一步里，对盘 0 上 (区域 0, 槽 0) 那个根槽第 2 次读注一次读错——第 1 次是现算 F 生效值那一遍读根环，第 2 次才是算上限那一读。
/// 这一槽这个进程知道住着根：重读读得出，上限照常是 0，这次抬 F 被上限拒（`RollbackFloorAboveCeiling`，与模型同一个结论），
/// 历史跑完、没有新发现（I-7.9 不红）。注入正好打中一次。
/// 判别力：读坏就当没有根时，实现接受 F = 1，模型判「该拒、实现做成了」，同一步 checker 判 I-7.9 红。
#[test]
fn the_seeded_history_whose_ceiling_read_of_the_genesis_root_slot_fails_once_refuses_the_raise_above_the_ceiling(
) {
    const SEED_OFFSET_OF_THE_CONSTRUCTION: u64 = 339;
    const STEP_WHOSE_READ_FAILS: usize = 1;
    const OPERATIONS: usize = 2;
    let history = generate_history_with_weights(
        HistorySeed(
            SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_CONSTRUCTION),
        ),
        OPERATIONS,
        &GenerationWeights::BROAD,
    );
    assert!(
        matches!(
            history.operations[STEP_WHOSE_READ_FAILS],
            HistoryOperation::RaiseRollbackFloor(_)
        ),
        "这段历史的第 1 步是抬 F：{:?}",
        history.operations
    );
    let execution = HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let plan = SharedFaultPlan::unarmed(execution.device_width.fixed_geometry());
    let parameters = execution.device_width.parameters();
    let schedule =
        schedule_on_the_genesis_root_slot(&parameters, FaultOccurrence::TheNthMatchingCall(2));
    let mut outcome_of_the_step = None;
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(STEP_WHOSE_READ_FAILS - 1) {
                plan.arm(schedule);
            }
            if observation.position == StepPosition::Operation(STEP_WHOSE_READ_FAILS) {
                plan.disarm();
                outcome_of_the_step = observation.outcome.cloned();
            }
        },
    );
    assert_eq!(plan.fired_count(), 1, "注入正好打中一次");
    assert_eq!(run.ending, HistoryEnding::Completed, "历史跑完、没有新发现");
    assert_eq!(
        outcome_of_the_step,
        Some(StepOutcome::Refused {
            member: "MountError::RollbackFloorAboveCeiling".to_string()
        }),
        "重读读得出，上限照常是 0，抬 F 到 1 被上限拒"
    );
}
