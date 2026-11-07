//! harness 测试文件 `crates/singlefs-verification/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_given_states_engine/mod.rs`）。那份 harness 测试文件与给定状态那一份
//! `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common_admission::{start_plain, PoolUnderTest};
use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PlacementRefusal;
use singlefs_core::journal::back_chain_of;
use singlefs_core::transaction::{
    publish_version, InstanceTablePlan, PoolVersion, PoolWriter, PublishError, PublishPlan,
    TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::history::HistoryDeviceWidth;
use singlefs_verification::memory_pool::SparseBlockDevice;
use std::sync::Arc;

/// 这次挂载的会话上直接发一次空发布（暖机、抬 F 与卸载推的那一种：不写文件内容、实例表照抄），不经会话的准入与推抬 F。
pub fn publish_an_empty_version_directly_on_the_session(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
) {
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
    session.current = PoolVersion::WithFile(Arc::new(published));
}

/// 第一格那一形，会话的准入用只供测试的开关关掉（`SpaceAdmission::SkippedByTheTestOnlySwitch`：式子与落得下两判都不判）：
/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 200 次空发布之后，经会话把文件顺序写到 2、3……29 个数据单元，都一次做成；
/// 写到 30 个单元：走固定点时在分配器的拷贝上取不到数据单元 19 的落点，交回 `PlacementRefused { Data(19), NoFreeSlotOnAnyDevice }`，
/// 会话照 D16（发布语义） 已定项 1「准入」那一行（准入放行而落点取不到也推）推一串抬 F、回收之后再发，做成——推过的只这一串、
/// 推它的是那一次落点被拒；F 抬上去了，池级 checker 0 违例。
/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 30 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。
/// 环窗口真扣、小盘放大补足 1007 槽（含退役树那几项）：200、400 次空发布都是写到 30 个、
/// 数据单元 19 被拒
/// （29 个之前都一次做成）。
pub fn run_the_scenario_where_with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write(
    images: &mut PoolCheckerImages,
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
    for data_units in 2..=29 {
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
        .sequential_write(30 * data_unit_payload_capacity())
        .unwrap_or_else(|refusal| {
            panic!("经会话顺序写到 30 个数据单元，推过抬 F 之后做成：{refusal:?}")
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
    images.hand_out(
        "推过抬 F 再写成之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
