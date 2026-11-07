//! harness 用例与给定状态用例共用场景；池级 checker 镜像交给给定状态引擎两段判。
//! 删第一个文件之外的号时，删除的各批按那个号自己还剩的单元数走（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4
//! 的取样点：正推 `research/prompts/m3-multi-file-unlink-r1-sonnet-output.md` 第 6.2 节那张表「① 计划相等」一格——对 N ≠ 1 的批计划，批大小 1、2、
//! 144、145 各一——连同攻方 U4-乙「删 N 的每一批之后第一个文件读回逐字节相同」）。
//!
//! 每条用例一个场景：第一个文件三个单元，第一个文件旁边建到 inode 150（落在 extent 上段第 1 片叶，与第一个文件不在同一片上段叶里），给 150 写若干个
//! 单元，再按一种批大小删它。比三样：每一步之后 150 还剩的单元数逐步等于纯函数 `unlink::next_unlink_step` 从 150 自己的单元数算出的那一串，
//! 那一串也等于写在用例里的数；第一个文件每一步之后读回都与删之前逐字节相同、三个单元一个不少；每一版池级 checker 一条违例都没有。
//! 批大小 1、2 用只供测试的开关压小，144、145 是产品批大小（一片下段叶的 144 个）下单元数正好一批与多一个的两形。崩溃枚举那几格（同一份崩溃镜像
//! 各自续做逐字节相同、每个写点截断重开续做到底）归崩溃枚举用例，不在这里。

#![allow(dead_code, reason = "引它的测试目标各只调用其中一部分场景与帮手")]

use crate::common::{build_pool, parameters};
use crate::common_images_for_the_given_states_engine::PoolCheckerImages;
use crate::common_shared_by_harness_scenarios::{
    content_read_back, hand_out_a_clean_image,
    pseudo_random_content_seven_bytes_short_of_data_units, write_in_process,
    INSTANCE_OF_THE_PROCESS, WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unlink::{
    next_unlink_step, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize, UnlinkStep,
};

/// 删的那个号：150 落在 extent 上段第 1 片叶（一片叶罩 143 个号），写它上段长到常规层，删它的最后一批上段降回崩溃点重放。
const DELETED_INODE: u64 = 150;
/// 第一个文件的单元数：三个，删别的号的每一批之后一个不少。
const DATA_UNITS_OF_THE_FIRST_FILE: usize = 3;
/// 删除的各步至多走这么多步：走不到最后一批说明计划出错，不无限循环。
const DELETION_STEPS_OF_THE_INODE_AT_MOST: usize = 8;

/// 只供测试的开关把每批压到 `data_units` 个。
fn capped_batches_of(data_units: u64) -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(data_units).expect("批大小在 1..=144 里"),
    )
}

/// 纯函数那一串：一个还剩 `data_units` 个单元的号按 `batch_size` 一步步走到最后一批，每一步之后还剩几个（最后一批之后是 0）。
fn data_units_remaining_after_each_step_of_the_plan(
    data_units: u64,
    batch_size: UnlinkBatchSize,
) -> Vec<u64> {
    let mut remaining = data_units;
    let mut after_each_step = Vec::new();
    // 迭代上界：[`DELETION_STEPS_OF_THE_INODE_AT_MOST`] 步，每一步不是最后一批就至少放一个单元。
    for _ in 0..DELETION_STEPS_OF_THE_INODE_AT_MOST {
        match next_unlink_step(remaining, batch_size) {
            UnlinkStep::Batch {
                data_units_kept, ..
            } => {
                remaining = data_units_kept;
                after_each_step.push(remaining);
            }
            UnlinkStep::FinalBatch { .. } => {
                after_each_step.push(0);
                return after_each_step;
            }
        }
    }
    panic!("{data_units} 个单元按 {batch_size:?} 走了 {DELETION_STEPS_OF_THE_INODE_AT_MOST} 步还没到最后一批");
}

/// 第一个文件三个单元、旁边建到 inode 150、给 150 写 `data_units_of_the_deleted_inode` 个单元，再按 `batch_size` 删 150：意图那次之后一步步走到
/// 最后一批，每一步之后第一个文件读回与删之前逐字节相同、三个单元一个不少、池级 checker 全绿。交回每一步之后 150 还剩几个单元（最后一批之后
/// 它连记录都没了，记 0）。
fn data_units_of_inode_150_after_each_deletion_step(
    images: &mut PoolCheckerImages,
    tag: &str,
    data_units_of_the_deleted_inode: usize,
    batch_size: UnlinkBatchSize,
) -> Vec<u64> {
    let mut pool = build_pool(tag);
    let content_of_the_first_file =
        pseudo_random_content_seven_bytes_short_of_data_units(DATA_UNITS_OF_THE_FIRST_FILE, 0x0151);
    write_in_process(&mut pool, FIRST_INODE_NUMBER, &content_of_the_first_file);
    let publish_parameters = parameters();
    {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            DELETED_INODE - 1,
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("建 inode 2 … 150（同一片叶容器装得下）");
    }
    write_in_process(
        &mut pool,
        DELETED_INODE,
        &pseudo_random_content_seven_bytes_short_of_data_units(
            data_units_of_the_deleted_inode,
            0x1502,
        ),
    );
    {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(DELETED_INODE),
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("unlink inode 150 的意图");
    }
    let mut data_units_after_each_step = Vec::new();
    // 迭代上界：[`DELETION_STEPS_OF_THE_INODE_AT_MOST`] 步之内走到最后一批（下面断言）。
    while pool
        .output
        .deleted_inodes
        .contains(&InodeNumber(DELETED_INODE))
    {
        assert!(
            data_units_after_each_step.len() < DELETION_STEPS_OF_THE_INODE_AT_MOST,
            "删 inode 150 走了 {DELETION_STEPS_OF_THE_INODE_AT_MOST} 步还没到最后一批"
        );
        let previous = pool.output.clone();
        {
            let devices = pool.devices.as_mut().expect("镜像还开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            pool.output = publish_unlink_step(
                &mut writer,
                &mut pool.allocator,
                &previous,
                InodeNumber(DELETED_INODE),
                batch_size,
                INSTANCE_OF_THE_PROCESS,
            )
            .expect("unlink inode 150 的下一步");
        }
        let step = data_units_after_each_step.len() + 1;
        let image = pool.memory_pool();
        assert_eq!(
            content_read_back(&image, FIRST_INODE_NUMBER),
            content_of_the_first_file,
            "删 inode 150 的第 {step} 步之后第一个文件读回逐字节相同"
        );
        assert_eq!(
            pool.output.data_pointers.len(),
            DATA_UNITS_OF_THE_FIRST_FILE,
            "删 inode 150 的第 {step} 步之后第一个文件三个单元一个不少"
        );
        hand_out_a_clean_image(images, &image, &format!("删 inode 150 的第 {step} 步之后"));
        data_units_after_each_step.push(
            pool.output
                .file_other_than_the_first(InodeNumber(DELETED_INODE))
                .map_or(0, |file| {
                    u64::try_from(file.data_pointers.len()).expect("单元数装得进 u64")
                }),
        );
    }
    data_units_after_each_step
}

/// 批大小 1、inode 150 三个单元：两批各截尾部 1 个（剩 2、1），最后一批放掉剩下那 1 个。
pub fn run_the_scenario_where_inode_150_of_three_data_units_in_batches_of_one_takes_two_batches_and_a_final_batch(
    images: &mut PoolCheckerImages,
) {
    let batch_size = capped_batches_of(1);
    let planned = data_units_remaining_after_each_step_of_the_plan(3, batch_size);
    assert_eq!(planned, vec![2, 1, 0], "纯函数那一串");
    assert_eq!(
        data_units_of_inode_150_after_each_deletion_step(
            images,
            "unlink-of-inode-150-of-three-units-in-batches-of-one",
            3,
            batch_size
        ),
        planned,
        "删 inode 150 的每一步按它自己的三个单元走"
    );
}

/// 批大小 2、inode 150 七个单元：三批各截尾部 2 个（剩 5、3、1），最后一批放掉剩下那 1 个。
pub fn run_the_scenario_where_inode_150_of_seven_data_units_in_batches_of_two_takes_three_batches_and_a_final_batch(
    images: &mut PoolCheckerImages,
) {
    let batch_size = capped_batches_of(2);
    let planned = data_units_remaining_after_each_step_of_the_plan(7, batch_size);
    assert_eq!(planned, vec![5, 3, 1, 0], "纯函数那一串");
    assert_eq!(
        data_units_of_inode_150_after_each_deletion_step(
            images,
            "unlink-of-inode-150-of-seven-units-in-batches-of-two",
            7,
            batch_size
        ),
        planned,
        "删 inode 150 的每一步按它自己的七个单元走"
    );
}

/// 产品批大小（144）、inode 150 正好 144 个单元：不多于一批，意图之后就是最后一批。
pub fn run_the_scenario_where_inode_150_of_144_data_units_with_the_product_batch_goes_straight_to_the_final_batch(
    images: &mut PoolCheckerImages,
) {
    let batch_size = UnlinkBatchSize::OneExtentLowerLeaf;
    let planned = data_units_remaining_after_each_step_of_the_plan(144, batch_size);
    assert_eq!(planned, vec![0], "纯函数那一串");
    assert_eq!(
        data_units_of_inode_150_after_each_deletion_step(
            images,
            "unlink-of-inode-150-of-144-units",
            144,
            batch_size
        ),
        planned,
        "删 inode 150 的每一步按它自己的 144 个单元走"
    );
}

/// 产品批大小（144）、inode 150 有 145 个单元：先一批截掉尾部 144 个（剩 1），最后一批放掉剩下那 1 个。
pub fn run_the_scenario_where_inode_150_of_145_data_units_with_the_product_batch_cuts_144_and_then_releases_the_last_one(
    images: &mut PoolCheckerImages,
) {
    let batch_size = UnlinkBatchSize::OneExtentLowerLeaf;
    let planned = data_units_remaining_after_each_step_of_the_plan(145, batch_size);
    assert_eq!(planned, vec![1, 0], "纯函数那一串");
    assert_eq!(
        data_units_of_inode_150_after_each_deletion_step(
            images,
            "unlink-of-inode-150-of-145-units",
            145,
            batch_size
        ),
        planned,
        "删 inode 150 的每一步按它自己的 145 个单元走"
    );
}
