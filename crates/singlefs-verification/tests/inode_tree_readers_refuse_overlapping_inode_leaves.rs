//! 读回判写者的前提（判决 `research/prompts/m4-close-code-r2-main-verification.md` 第二节 T4、第三节取改法甲）：写者
//! `inode_tree::write_records_into_leaf_containers` 的 `requires` 要全部 inode 叶的记录按叶序连起来按 inode 号严格升序（两片之间不交叠，I-9.12），
//! 按叶序折半找 `TransactionOutput::inode_record_found_by_its_leaf` 也靠它。读 inode 树的四个读者——可写挂载重建、冷走读、只读挂载打开池
//! （`mounted_read::open_pool_for_read`）与快照第一个文件的冷读（`recovery::read_the_first_file_of_a_snapshot`）——照这一条判，交叠就拒，
//! 同一张坏镜像四个读者一个结局。
//! 坏镜像取攻方第二次抽样那一份（`research/prompts/m4-close-code-r2-opus-s2-model/tests/` 下）：两片 inode 叶（1..=233 与 234..=236），
//! 把左片最后一条 233 号改成 237 号（左片自己仍升序，根头最大 key 跟着改成 237，I-9.4 与根的子树覆盖区间都照旧成立）。
//! 可写挂载收下它的话，之后按叶序折半找漏找 234 号、维护的计数漂走、unlink 走空在 `expect` 上 panic；所以可写挂载在任何写之前拒。
//! 合法的两片叶照旧收：删左片里的一条，记着的计数等于现数。盘是内存稀疏盘，池在进程里 mkfs、取号、暖机、第一个文件、建空 inode 造出来。
//!
//! 分隔 key 那两句（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 1、2 条，攻方两份模型用例
//! `research/prompts/m4-close-code-r3-opus-model/tests/inode_readers_judge_separator_keys_against_the_left_leaf.rs` 挪进来、断言翻成修完之后的期望）：
//! 同一片两片叶的池（另建一个只读快照，它的 inode 根就是活头那一棵）把右片那条内部条目的分隔 key 从容器号 234 改成别的值、根头 key 区间不动。
//! 改成 200（不大于左片最大号 233）：池级 checker 判 I-9.12 红，四个读者都拒在 I-9.12（`recovery::INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT`）；
//! 改成 235（大于右片最小号 234）：池级 checker 判 I-9.12 红，四个读者都拒在 I-9.4（容器号不超最小 key），即记录号小于分隔 key。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::{
    change_the_inode_record, change_the_unit_and_propagate_its_checksum,
    root_slot_of_the_tree_of_kind_of_the_newest_root,
};
use common_pointer_head_corruption::{
    newest_root, node_bytes, recovery_failure, writable_mount_refused_before_any_write,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration, TreeIdentifier};
use singlefs_core::admission::InodeRecordCounts;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{open_pool_for_read, OpenPoolForReadFailure};
use singlefs_core::records::{
    InodeRecord, NLINK_OF_AN_INODE_BEING_DELETED, TREE_KIND_INODE, TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, RecoveryFailure,
    INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
    INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
    INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
};
use singlefs_core::transaction::{
    acquire_instance, publish_create_snapshot, publish_first_file, publish_new_inodes,
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile,
    PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use std::sync::Arc;

/// 码 2 节点头里 key 区间从偏移 52 起，先最小、后最大；inode 树 key 宽 8，最大 key 住 60。
const INODE_ROOT_LARGEST_KEY_OFFSET: usize = 52 + 8;
/// 一片 inode 叶装 233 条。
const INODE_LEAF_RECORDS: u64 = 233;

/// 这一版 inode 树里的两个数现数一遍（对照维护的那两个）。
fn counted_over_the_tree(version: &TransactionOutput) -> InodeRecordCounts {
    let mut counts = InodeRecordCounts {
        live_objects: 0,
        records: 0,
    };
    for container in &version.inode_leaf_containers {
        for record in &container.contents.records {
            counts.records += 1;
            if record.nlink != NLINK_OF_AN_INODE_BEING_DELETED {
                counts.live_objects += 1;
            }
        }
    }
    counts
}

fn inode_numbers_of_each_leaf(version: &TransactionOutput) -> Vec<Vec<u64>> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| {
            container
                .contents
                .records
                .iter()
                .map(|record| record.inode)
                .collect()
        })
        .collect()
}

/// 两片 inode 叶的池：盘、接着发布要用的分配器与实例代号、现行那一版。
struct PoolWithTwoInodeLeaves {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    two_leaves: TransactionOutput,
}

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个文件 → 建 232 个空 inode（左片 1..=233 装满）→ 再建 3 个（右片 234..=236）。
fn pool_with_two_inode_leaves() -> (Vec<(DeviceIdentity, SparseBlockDevice)>, TransactionOutput) {
    let pool = pool_with_two_inode_leaves_keeping_the_allocator();
    (pool.devices, pool.two_leaves)
}

/// 同 [`pool_with_two_inode_leaves`]，分配器与实例代号留着给接着的发布用。
fn pool_with_two_inode_leaves_keeping_the_allocator() -> PoolWithTwoInodeLeaves {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = file_content();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(
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
    let left_full = publish_new_inodes(
        &mut writer,
        &mut allocator,
        &first,
        INODE_LEAF_RECORDS - 1,
        FIXED_WRITE_TIME_SECONDS,
        instance,
    )
    .expect("建 232 个空 inode");
    let two_leaves = publish_new_inodes(
        &mut writer,
        &mut allocator,
        &left_full,
        3,
        FIXED_WRITE_TIME_SECONDS,
        instance,
    )
    .expect("再建 3 个空 inode");
    drop(writer);
    let leaves = inode_numbers_of_each_leaf(&two_leaves);
    assert_eq!(
        leaves,
        vec![(1..=233).collect::<Vec<u64>>(), vec![234, 235, 236]],
        "前提：左片 1..=233、右片 234..=236"
    );
    PoolWithTwoInodeLeaves {
        devices,
        allocator,
        instance,
        two_leaves,
    }
}

fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 两片叶的池上把左片最后一条 233 号改成水位那个号 237（比右片头一条 234 大），根头最大 key 236 跟着改成 237；改单元的那几处校验和沿引用链补到根槽。
fn image_whose_left_inode_leaf_reaches_past_the_right_leaf() -> MemoryPool {
    let (devices, two_leaves) = pool_with_two_inode_leaves();
    let mut image = memory_pool_of(&devices);
    make_the_left_inode_leaf_reach_past_the_right_leaf(&mut image, &two_leaves);
    image
}

/// 在 `image` 上做 [`image_whose_left_inode_leaf_reaches_past_the_right_leaf`] 那一改：改的是最新那条根的 inode 树（叶与根），引着这两个单元的
/// 每一处位置条目（快照的码 14 条目照抄活头那一棵时也在内）跟着补校验和。`two_leaves` 是两片叶那一版（取它的水位）。
fn make_the_left_inode_leaf_reach_past_the_right_leaf(
    image: &mut MemoryPool,
    two_leaves: &TransactionOutput,
) {
    let renamed_to = two_leaves.inode_number_watermark();
    assert_eq!(renamed_to, 237, "前提：水位 237（下一个号）");
    change_the_inode_record(image, 233, |bytes| {
        let mut record = InodeRecord::parse(bytes).expect("记录解得开");
        assert_eq!(
            (record.inode, record.size),
            (233, 0),
            "改之前：233 号、size 0"
        );
        record.inode = renamed_to;
        bytes.copy_from_slice(&record.to_bytes());
    });
    let inode_root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_INODE);
    change_the_unit_and_propagate_its_checksum(image, inode_root_slot, node_bytes(), |root| {
        let largest = u64::from_le_bytes(
            root[INODE_ROOT_LARGEST_KEY_OFFSET..INODE_ROOT_LARGEST_KEY_OFFSET + 8]
                .try_into()
                .expect("8 字节"),
        );
        assert_eq!(largest, 236, "改之前根头最大 key 236");
        root[INODE_ROOT_LARGEST_KEY_OFFSET..INODE_ROOT_LARGEST_KEY_OFFSET + 8]
            .copy_from_slice(&renamed_to.to_le_bytes());
    });
}

/// 两片叶的池上再建一个只读快照（建快照照抄 inode 树：快照的 inode 根就是活头那一棵），再做 [`make_the_left_inode_leaf_reach_past_the_right_leaf`]
/// 那一改：快照指着的 inode 树两片叶交叠。交回镜像与快照的树号（码 14 条目的树号）。
fn image_whose_snapshot_has_a_left_inode_leaf_reaching_past_the_right_leaf(
) -> (MemoryPool, TreeIdentifier) {
    let mut pool = pool_with_two_inode_leaves_keeping_the_allocator();
    let parameters = parameters();
    let with_the_snapshot = {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &pool.two_leaves,
            pool.instance,
        )
        .expect("建快照")
    };
    let entry_of_kind = |kind: u16| {
        *with_the_snapshot
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind)
            .expect("树表里有这种条目")
    };
    let snapshot_entry = entry_of_kind(TREE_KIND_SNAPSHOT_INODE_ROOT);
    assert_eq!(
        snapshot_entry.root.locations,
        entry_of_kind(TREE_KIND_INODE).root.locations,
        "前提：快照的 inode 根就是活头那一棵（建快照照抄 inode 树）"
    );
    let mut image = memory_pool_of(&pool.devices);
    make_the_left_inode_leaf_reach_past_the_right_leaf(&mut image, &pool.two_leaves);
    (image, snapshot_entry.tree)
}

/// 可写挂载重建这一版时拒在 I-9.12 上（`recovery::inode_records_of_the_leaf_ascend_after_the_leaves_before`），盘上逐字节不变。
/// 判别力：重建那一处不调这一判（`crates/mutations.tsv` 一行），可写挂载收下这张镜像，`writable_mount_refused_before_any_write` 当场 panic。
#[test]
fn a_writable_mount_refuses_inode_leaves_whose_left_leaf_reaches_past_the_right_leaf_before_any_write(
) {
    let image = image_whose_left_inode_leaf_reaches_past_the_right_leaf();
    let refusal = writable_mount_refused_before_any_write(&image);
    let described = format!("{refusal:?}");
    assert!(
        described.contains("I-9.12")
            && described.contains(INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM),
        "可写挂载拒在两片 inode 叶交叠上：{refusal:?}"
    );
}

/// 冷走读同一判：拒在 I-9.12 上。判别力：冷走读那一处不调这一判（`crates/mutations.tsv` 一行），冷走读读通这一版、`recovery_failure` 当场 panic。
#[test]
fn the_cold_walk_refuses_inode_leaves_whose_left_leaf_reaches_past_the_right_leaf() {
    let image = image_whose_left_inode_leaf_reaches_past_the_right_leaf();
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::InvariantViolated {
            invariant: "I-9.12",
            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
        },
        "冷走读拒在两片 inode 叶交叠上"
    );
}

/// 只读挂载打开池（`mounted_read::open_pool_for_read`）同一判：拒在 I-9.12 上。判别力：只读挂载那一处不调这一判（`crates/mutations.tsv` 一行），
/// 打开照收这一版，`expect` 当场 panic。
#[test]
fn the_read_only_open_refuses_inode_leaves_whose_left_leaf_reaches_past_the_right_leaf() {
    let image = image_whose_left_inode_leaf_reaches_past_the_right_leaf();
    let refusal = open_pool_for_read(&image, &newest_root(&image))
        .err()
        .expect("只读挂载打开池不收两片交叠的一版");
    assert_eq!(
        refusal,
        OpenPoolForReadFailure::Walk(RecoveryFailure::InvariantViolated {
            invariant: "I-9.12",
            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
        }),
        "只读挂载拒在两片 inode 叶交叠上"
    );
}

/// 快照第一个文件的冷读（`recovery::read_the_first_file_of_a_snapshot`）同一判：快照指着的 inode 树两片叶交叠，拒在 I-9.12 上。
/// 判别力：快照冷读那一处不调这一判（`crates/mutations.tsv` 一行），读出第一个文件、`assert_eq` 红。
#[test]
fn the_cold_read_of_a_snapshot_refuses_inode_leaves_whose_left_leaf_reaches_past_the_right_leaf() {
    let (image, snapshot) =
        image_whose_snapshot_has_a_left_inode_leaf_reaching_past_the_right_leaf();
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &newest_root(&image), snapshot),
        Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.12",
            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
        }),
        "快照冷读拒在两片 inode 叶交叠上"
    );
}

/// 合法历史上的对照：两片 inode 叶（1..=233 与 234..=236）可写挂载照收，把左片里的 5 号 unlink 到底，意图那一版与走空之后那一版
/// 记着的两个数都等于现数（取自攻方第二次抽样那一份的对照用例）。判别力：叶序折半找改成「恒取最右那片」时意图那一版的数漂走，这一条红。
#[test]
fn unlinking_a_record_of_the_left_leaf_of_two_keeps_the_counts_equal_to_the_counts_over_the_tree() {
    let (mut devices, two_leaves) = pool_with_two_inode_leaves();
    assert_eq!(
        two_leaves.inode_record_counts,
        counted_over_the_tree(&two_leaves),
        "建完 inode 那一版"
    );
    let parameters = parameters();
    let mounted = mount_writable(&parameters, &mut devices).expect("合法的两片叶池可写挂载");
    let version = mounted
        .current
        .file_version()
        .expect("带文件的一版")
        .clone();
    assert_eq!(
        version.inode_record_counts,
        counted_over_the_tree(&version),
        "挂载数起始值"
    );
    let mut allocator = mounted.allocator.clone();
    let instance = mounted.output.instance;
    let intent = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut allocator,
            &version,
            InodeNumber(5),
            FIXED_WRITE_TIME_SECONDS,
            instance,
        )
        .expect("unlink 5 号的意图")
    };
    assert_eq!(
        intent.inode_record_counts,
        counted_over_the_tree(&intent),
        "意图那一版：5 号在左片，找得到上一版那条"
    );
    let mut current = Arc::new(intent.clone());
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        instance,
    )
    .expect("走空");
    assert_eq!(
        current.inode_record_counts,
        counted_over_the_tree(&current),
        "走空之后：最后一批删掉的那条在左片"
    );
}

/// 码 2 节点头 131 字节，内部条目 120 字节、头 8 字节是分隔 key：右片那条（第 1 条）的分隔 key 从 131 + 120 起。
const SEPARATOR_OF_THE_RIGHT_LEAF_OFFSET: usize = 131 + 120;
/// 右片那条内部条目的分隔 key 改成不大于左片最大号 233 的那个值（攻方 `separator_200`）。
const SEPARATOR_NOT_ABOVE_THE_LEFT_LEAF: u64 = 200;
/// 右片那条内部条目的分隔 key 改成大于右片最小号 234 的那个值（攻方 `separator_235`）。
const SEPARATOR_ABOVE_THE_FIRST_RECORD_OF_THE_RIGHT_LEAF: u64 = 235;

/// 两片叶的池上建一个只读快照（快照的 inode 根就是活头那一棵），再把最新那条根的 inode 树根右片那条条目的分隔 key 从 234 改成 `separator`，
/// 根头 key 区间不动，改单元的校验和沿引用链补到根槽（快照的码 14 条目跟着补）。交回镜像与快照的树号；改完的镜像上池级 checker 判 I-9.12 红（前提）。
fn image_with_a_snapshot_whose_right_inode_leaf_separator_is(
    separator: u64,
) -> (MemoryPool, TreeIdentifier) {
    let mut pool = pool_with_two_inode_leaves_keeping_the_allocator();
    let parameters = parameters();
    let with_the_snapshot = {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &pool.two_leaves,
            pool.instance,
        )
        .expect("建快照")
    };
    let snapshot_entry = *with_the_snapshot
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("树表里有快照的码 14 条目");
    let mut image = memory_pool_of(&pool.devices);
    let inode_root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_INODE);
    change_the_unit_and_propagate_its_checksum(&mut image, inode_root_slot, node_bytes(), |root| {
        let range = SEPARATOR_OF_THE_RIGHT_LEAF_OFFSET..SEPARATOR_OF_THE_RIGHT_LEAF_OFFSET + 8;
        let before = u64::from_le_bytes(root[range.clone()].try_into().expect("8 字节"));
        assert_eq!(before, 234, "改之前右片的分隔 key 是它的容器号 234");
        root[range].copy_from_slice(&separator.to_le_bytes());
    });
    let verdict_of_i_9_12 = check_pool_image(&image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-9.12")
        .map(|(_, verdict)| verdict)
        .expect("池级 checker 报 I-9.12");
    assert!(
        matches!(verdict_of_i_9_12, InvariantVerdict::Violated(_)),
        "前提：分隔 key {separator} 那一版池级 checker 判 I-9.12 红：{verdict_of_i_9_12:?}"
    );
    (image, snapshot_entry.tree)
}

/// 分隔 key 不大于左片最大号那一形：四个读者都拒成的那一个失败。
fn separator_not_above_the_left_leaf() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-9.12",
        detail: INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
    }
}

/// 分隔 key 大于右片最小号那一形：四个读者都拒成的那一个失败。
fn record_below_the_separator() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-9.4",
        detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
    }
}

/// 分隔 key 200：可写挂载重建这一版时拒在 I-9.12 第三句，盘上逐字节不变。判别力：重建那一处不比分隔 key（`crates/mutations.tsv` 一行），
/// 可写挂载收下这张镜像，`writable_mount_refused_before_any_write` 当场 panic。
#[test]
fn a_writable_mount_refuses_a_right_leaf_separator_not_above_the_left_leaf_before_any_write() {
    let (image, _snapshot) = image_with_a_snapshot_whose_right_inode_leaf_separator_is(
        SEPARATOR_NOT_ABOVE_THE_LEFT_LEAF,
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    let described = format!("{refusal:?}");
    assert!(
        described.contains("I-9.12")
            && described.contains(INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT),
        "可写挂载拒在分隔 key 不大于左片最大号上：{refusal:?}"
    );
}

/// 分隔 key 200：冷走读拒在 I-9.12 第三句。判别力：两片叶之间不比分隔 key（`crates/mutations.tsv` 一行），冷走读读通、`recovery_failure` 当场 panic。
#[test]
fn the_cold_walk_refuses_a_right_leaf_separator_not_above_the_left_leaf() {
    let (image, _snapshot) = image_with_a_snapshot_whose_right_inode_leaf_separator_is(
        SEPARATOR_NOT_ABOVE_THE_LEFT_LEAF,
    );
    assert_eq!(
        recovery_failure(&image),
        separator_not_above_the_left_leaf(),
        "冷走读拒在分隔 key 不大于左片最大号上"
    );
}

/// 分隔 key 200：只读挂载打开池拒在 I-9.12 第三句。判别力：只读挂载那一处交进去的分隔 key 换成右片头一条的号（`crates/mutations.tsv` 一行），打开照收。
#[test]
fn the_read_only_open_refuses_a_right_leaf_separator_not_above_the_left_leaf() {
    let (image, _snapshot) = image_with_a_snapshot_whose_right_inode_leaf_separator_is(
        SEPARATOR_NOT_ABOVE_THE_LEFT_LEAF,
    );
    assert_eq!(
        open_pool_for_read(&image, &newest_root(&image)).err(),
        Some(OpenPoolForReadFailure::Walk(
            separator_not_above_the_left_leaf()
        )),
        "只读挂载拒在分隔 key 不大于左片最大号上"
    );
}

/// 分隔 key 200：快照第一个文件的冷读拒在 I-9.12 第三句（快照的 inode 根就是改过的那一棵）。
#[test]
fn the_cold_read_of_a_snapshot_refuses_a_right_leaf_separator_not_above_the_left_leaf() {
    let (image, snapshot) = image_with_a_snapshot_whose_right_inode_leaf_separator_is(
        SEPARATOR_NOT_ABOVE_THE_LEFT_LEAF,
    );
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &newest_root(&image), snapshot),
        Err(separator_not_above_the_left_leaf()),
        "快照冷读拒在分隔 key 不大于左片最大号上"
    );
}

/// 分隔 key 235：只读挂载打开池拒在 I-9.4（右片头一条 234 小于分隔 key），与另三个读者同一个失败。判别力：只读挂载那一处不逐条判记录号不小于分隔 key
/// （`crates/mutations.tsv` 一行），打开照收这一版（修之前就是这样，攻方 `separator_235`）。
#[test]
fn the_read_only_open_refuses_a_right_leaf_separator_above_its_first_record_on_the_container_number_invariant(
) {
    let (image, _snapshot) = image_with_a_snapshot_whose_right_inode_leaf_separator_is(
        SEPARATOR_ABOVE_THE_FIRST_RECORD_OF_THE_RIGHT_LEAF,
    );
    assert_eq!(
        open_pool_for_read(&image, &newest_root(&image)).err(),
        Some(OpenPoolForReadFailure::Walk(record_below_the_separator())),
        "只读挂载拒在记录号小于分隔 key 上"
    );
}

/// 分隔 key 235：另三个读者——可写挂载重建、冷走读、快照冷读——拒成与只读挂载同一个失败，I-9.4（容器号不超最小 key）。同一张镜像上的三个读者，一个场景。
#[test]
fn the_other_three_readers_refuse_a_right_leaf_separator_above_its_first_record_with_the_same_failure(
) {
    let (image, snapshot) = image_with_a_snapshot_whose_right_inode_leaf_separator_is(
        SEPARATOR_ABOVE_THE_FIRST_RECORD_OF_THE_RIGHT_LEAF,
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        format!("{refusal:?}").contains(&format!("{:?}", record_below_the_separator())),
        "可写挂载拒在记录号小于分隔 key 上：{refusal:?}"
    );
    assert_eq!(
        recovery_failure(&image),
        record_below_the_separator(),
        "冷走读拒在记录号小于分隔 key 上"
    );
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &newest_root(&image), snapshot),
        Err(record_below_the_separator()),
        "快照冷读拒在记录号小于分隔 key 上"
    );
}
