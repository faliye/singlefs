//! 真发布路径上、按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`，一批 144 个数据单元）删一个罩着两片 extent 下段叶的文件
//! （D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」；代码轮 U2：入库的删除用例此前只删 3 个单元的文件、批大小 1，只改尾叶那一支没有真发布路径上的用例）。
//! 287 个数据单元：第一片叶罩单元 0–143、第二片罩 144–286；第一批从尾部删掉 144 个（143–286），留下 143 个——第一片叶的末个单元（143）
//! 正是被删掉的第一个单元，这片叶不能整片照抄、要重写（`unlink::plan_the_extent_tree_after_a_deletion_batch` 的 `is_entirely_kept`）；
//! 第二批删完。逐版判：从盘上重建出来的那一版与发布的那一版（inode 记录、队列、数据指针、extent 树、进映射的单元）逐项相同，
//! 池级 checker 无红；删完之后冷走读答「没有文件」。草稿出处：正推腿探针 `research/prompts/m3-unlink-code-r1-sonnet-model/sonnet_probe_multi_leaf_unlink.rs`。

mod common;

use common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
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

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 新池新建文件那个实例：意图与各批都在这个进程里发。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

/// 文件的数据单元数：两片下段叶（一片罩 144 个），第一批之后留下的个数（143）比第一片叶罩的少一个。
const DATA_UNITS_OF_THE_FILE: usize = 287;

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 这一版从盘上重建出来的那几样与发布的那一版逐项相同。
fn assert_rebuilt_from_the_disk_equals_the_published(
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
#[test]
fn a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut(
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
        assert_eq!(
            violations(&image),
            Vec::<String>::new(),
            "txg {:?} 之后池级 checker 无红",
            pool.output.root.checkpoint_txg
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
