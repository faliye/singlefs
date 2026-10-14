//! harness 测试文件 `crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::MemoryPool;

pub const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 新池新建文件那个实例：意图与各批都在这个进程里发。
pub const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

/// 文件的数据单元数：两片下段叶（一片罩 144 个），第一批之后留下的个数（143）比第一片叶罩的少一个。
pub const DATA_UNITS_OF_THE_FILE: usize = 287;

/// 这一版从盘上重建出来的那几样与发布的那一版逐项相同。
pub fn assert_rebuilt_from_the_disk_equals_the_published(
    image: &MemoryPool,
    version: &TransactionOutput,
) {
    let rebuilt = rebuild_version(image, &version.root, Some(version.record.clone()))
        .unwrap_or_else(|failure| {
            panic!(
                "txg {:?} 从盘上重建：{failure:?}",
                version.root.checkpoint_txg
            )
        });
    let (RebuiltVersion::WithFile(rebuilt) | RebuiltVersion::WithTreesAndNoFile(rebuilt)) = rebuilt
    else {
        panic!("txg {:?} 有树", version.root.checkpoint_txg);
    };
    assert_eq!(
        (
            &rebuilt.inode_record,
            &rebuilt.deleted_inodes,
            &rebuilt.data_pointers,
            &rebuilt.extent_tree,
            &rebuilt.mapped_units,
        ),
        (
            &version.inode_record,
            &version.deleted_inodes,
            &version.data_pointers,
            &version.extent_tree,
            &version.mapped_units,
        ),
        "txg {:?}：从盘上重建出来的与发布的逐项相同",
        version.root.checkpoint_txg
    );
}

/// 判别力：`is_entirely_kept` 把「末个单元等于留下的个数」也算整片留下（`<` 改 `<=`）时，第一批之后那一版的第一片叶照抄、
/// 还指着被删掉的单元 143，重建或池级 checker 那一项红。
pub fn a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut(
    images: &mut PoolCheckerImages,
) {
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(DATA_UNITS_OF_THE_FILE - 1) * payload_capacity
        + payload_capacity / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool("unlink-two-extent-lower-leaves-in-product-batches");
    let grown =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), DATA_UNITS_OF_THE_FILE);
    let publish_parameters = parameters();
    let mut published: Vec<TransactionOutput> = Vec::new();
    {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FIRST_FILE,
            previous
                .inode_record
                .expect("意图之前第一个文件那条记录在")
                .write_time_seconds,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("意图");
    }
    published.push(pool.output.clone());
    assert_rebuilt_from_the_disk_equals_the_published(&pool.memory_pool(), &pool.output);
    // 迭代上界：每一步至少删一个单元，文件 287 个单元。
    while !pool.output.deleted_inodes.is_empty() {
        assert!(published.len() <= DATA_UNITS_OF_THE_FILE, "删除步数有界");
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_step(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FIRST_FILE,
            UnlinkBatchSize::OneExtentLowerLeaf,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("删除的下一步");
        published.push(pool.output.clone());
        let image = pool.memory_pool();
        assert_rebuilt_from_the_disk_equals_the_published(&image, &pool.output);
        images.hand_out(
            &format!("txg {:?} 之后", pool.output.root.checkpoint_txg),
            || image.clone(),
            PoolCheckerExpectation::no_invariant_violated(),
        );
    }
    assert_eq!(
        published
            .iter()
            .map(|version| version.data_pointers.len())
            .collect::<Vec<_>>(),
        vec![DATA_UNITS_OF_THE_FILE, 143, 0],
        "意图那一版单元都在；第一批从尾部删掉 144 个、留 143 个；第二批删完"
    );
    assert!(
        matches!(
            recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
            RecoveryOutcome::NoFile { .. }
        ),
        "删完之后冷走读答「没有文件」"
    );
}
