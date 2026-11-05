//! 攻方第二次抽样（m4-close-code-r2）的模型：n_live 与 n_记录按叶序折半找上一版那条记录（`TransactionOutput::inode_record_found_by_its_leaf`），
//! 前提是叶容器左起按 key 序、两片之间不交叠（I-9.12）；可写挂载的重建只判 I-9.4 的「记录号不小于分隔 key 与容器号」，不判两片交叠。
//! 坏镜像：两片 inode 叶（1..=233 与 234..=236），把左边那片最后一条 233 号改成 237 号（左片仍升序，根头最大 key 跟着改成 237）。
//! 写路径（`inode_tree` 逐片找）与 unlink 意图（逐条找）都找得到 234 号，按叶序折半找找不到：维护的两个数漂走，最后一批在 `expect` 上 panic。
//! 盘是内存稀疏盘，池在进程里 mkfs、取号、暖机、第一个文件、建空 inode 造出来，不建镜像文件。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::panic::{catch_unwind, AssertUnwindSafe};

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::{
    change_the_inode_record, change_the_unit_and_propagate_its_checksum,
    root_slot_of_the_tree_of_kind_of_the_newest_root,
};
use common_pointer_head_corruption::{node_bytes, writable_devices_of};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::InodeRecordCounts;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::mount_writable;
use singlefs_core::records::{InodeRecord, NLINK_OF_AN_INODE_BEING_DELETED, TREE_KIND_INODE};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 码 2 节点头里 key 区间从偏移 52 起，先最小、后最大；inode 树 key 宽 8，最大 key 住 60。
const INODE_ROOT_LARGEST_KEY_OFFSET: usize = 52 + 8;
/// 一片 inode 叶装 233 条。
const INODE_LEAF_RECORDS: u64 = 233;

fn counted_here(version: &TransactionOutput) -> InodeRecordCounts {
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

fn records_of_each_leaf(version: &TransactionOutput) -> Vec<Vec<u64>> {
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

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个文件 → 建 232 个空 inode（左片 1..=233 装满）→ 再建 3 个（右片 234..=236）。
fn pool_with_two_inode_leaves() -> (Vec<(DeviceIdentity, SparseBlockDevice)>, TransactionOutput) {
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
    (devices, two_leaves)
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

/// 从 panic 的载荷里取消息（`expect` 交 `&str` 或 `String`）。
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "（载荷不是字符串）".to_string())
}

/// 场景见文件头。每一步打一行 `attack-s2:` 开头的观测；断言的是今天冷冻副本上的行为（这条用例绿 = 缺陷在）。
#[test]
fn a_writable_mount_of_overlapping_inode_leaves_lets_the_kept_counts_drift_and_the_final_batch_panic(
) {
    let (devices, two_leaves) = pool_with_two_inode_leaves();
    let leaves = records_of_each_leaf(&two_leaves);
    assert_eq!(leaves.len(), 2, "前提：两片叶");
    assert_eq!(
        leaves[0],
        (1..=233).collect::<Vec<u64>>(),
        "前提：左片 1..=233"
    );
    assert_eq!(leaves[1], vec![234, 235, 236], "前提：右片 234..=236");
    let renamed_to = two_leaves.inode_number_watermark();
    assert_eq!(renamed_to, 237, "前提：水位 237（下一个号）");
    println!(
        "attack-s2: 造好两片叶 {:?} 片、左 {} 条、右 {:?}；水位 {renamed_to}",
        leaves.len(),
        leaves[0].len(),
        leaves[1]
    );

    let mut image = memory_pool_of(&devices);
    change_the_inode_record(&mut image, 233, |bytes| {
        let mut record = InodeRecord::parse(bytes).expect("记录解得开");
        assert_eq!(
            (record.inode, record.size),
            (233, 0),
            "改之前：233 号、size 0"
        );
        record.inode = renamed_to;
        bytes.copy_from_slice(&record.to_bytes());
    });
    let inode_root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_INODE);
    change_the_unit_and_propagate_its_checksum(&mut image, inode_root_slot, node_bytes(), |root| {
        let largest = u64::from_le_bytes(
            root[INODE_ROOT_LARGEST_KEY_OFFSET..INODE_ROOT_LARGEST_KEY_OFFSET + 8]
                .try_into()
                .expect("8 字节"),
        );
        assert_eq!(largest, 236, "改之前根头最大 key 236");
        root[INODE_ROOT_LARGEST_KEY_OFFSET..INODE_ROOT_LARGEST_KEY_OFFSET + 8]
            .copy_from_slice(&renamed_to.to_le_bytes());
    });

    let mut devices = writable_devices_of(&image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            panic!("attack-s2: 可写挂载拒了这份坏镜像（攻方的前提不成立）：{refusal:?}")
        }
    };
    println!(
        "attack-s2: 可写挂载收下：实例 {:?}、有效根 txg {:?}（造镜像那一版 txg {:?}）",
        mounted.output.instance,
        mounted.output.effective_root.checkpoint_txg,
        two_leaves.root.checkpoint_txg
    );
    let version = mounted
        .current
        .file_version()
        .expect("带文件的一版")
        .clone();
    println!(
        "attack-s2: 挂载之后的叶 {:?}",
        records_of_each_leaf(&version)
            .iter()
            .map(|leaf| (leaf.first().copied(), leaf.last().copied(), leaf.len()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        version.inode_record_counts,
        counted_here(&version),
        "挂载数起始值：与现数相等"
    );
    let by_scanning = version.inode_record_of(InodeNumber(234));
    let by_its_leaf = version.inode_record_found_by_its_leaf(InodeNumber(234));
    println!(
        "attack-s2: 234 号逐条找 {:?}；按叶序折半找 {:?}",
        by_scanning.map(|record| record.inode),
        by_its_leaf.map(|record| record.inode)
    );
    assert!(
        by_scanning.is_some() && by_its_leaf.is_none(),
        "两种找法交回的不是同一条"
    );

    let mut allocator = mounted.allocator.clone();
    let instance = mounted.output.instance;
    let publish_parameters = parameters();
    let intent = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut allocator,
            &version,
            InodeNumber(234),
            FIXED_WRITE_TIME_SECONDS,
            instance,
        )
        .expect("unlink 234 号的意图")
    };
    let kept = intent.inode_record_counts;
    let counted = counted_here(&intent);
    println!(
        "attack-s2: 意图那一版 txg {:?}：记着的 {kept:?}，现数 {counted:?}",
        intent.root.checkpoint_txg
    );
    assert_ne!(
        kept, counted,
        "意图那一版记着的两个数与现数不等（标签 6 按记着的写）"
    );

    let mut current = intent.clone();
    let walked = catch_unwind(AssertUnwindSafe(|| {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_unlink_steps_until_the_queue_is_empty(
            &mut writer,
            &mut allocator,
            &mut current,
            UnlinkBatchSize::OneExtentLowerLeaf,
            instance,
        )
    }));
    match walked {
        Ok(result) => panic!("attack-s2: 走空没有 panic：{result:?}"),
        Err(payload) => {
            let message = panic_message(payload.as_ref());
            println!("attack-s2: 走空 panic：{message}");
            assert!(
                message.contains("最后一批删的那条记录在上一版的 inode 树里"),
                "panic 在最后一批找上一版那条记录那一处：{message}"
            );
        }
    }
}

/// 合法历史上的对照（判别力补在多片叶上）：两片 inode 叶（1..=233 与 234..=236），把左片里的 5 号 unlink 到底，意图那一版与走空之后那一版
/// 记着的两个数都等于现数。`inode_record_counts_kept_with_each_version.rs` 的随机历史一段至多 30 步、建 inode 一步一个，树里不到一片叶装满，
/// 按叶序折半找的跨片那一半走不到；把它改成「恒取最右那片」，那边照绿、这条红。
#[test]
fn unlinking_a_record_of_the_left_leaf_of_two_keeps_the_counts_equal_to_the_counts_over_the_tree() {
    let (mut devices, two_leaves) = pool_with_two_inode_leaves();
    assert_eq!(records_of_each_leaf(&two_leaves).len(), 2, "前提：两片叶");
    assert_eq!(
        two_leaves.inode_record_counts,
        counted_here(&two_leaves),
        "建完 inode 那一版"
    );
    let parameters = parameters();
    let mut allocator = {
        // 造池那一段的分配器没交出来：照同一份盘重开一次可写挂载，取它交回的分配器与现行那一版（数起始值那一条路也走一遍）。
        let mounted = mount_writable(&parameters, &mut devices).expect("合法的两片叶池可写挂载");
        let version = mounted
            .current
            .file_version()
            .expect("带文件的一版")
            .clone();
        assert_eq!(
            version.inode_record_counts,
            counted_here(&version),
            "挂载数起始值"
        );
        (mounted.allocator.clone(), version, mounted.output.instance)
    };
    let (allocator_of_the_mount, version, instance) =
        (&mut allocator.0, allocator.1.clone(), allocator.2);
    let intent = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            allocator_of_the_mount,
            &version,
            InodeNumber(5),
            FIXED_WRITE_TIME_SECONDS,
            instance,
        )
        .expect("unlink 5 号的意图")
    };
    println!(
        "attack-s2: 合法两片叶 unlink 5 号意图：记着的 {:?}，现数 {:?}",
        intent.inode_record_counts,
        counted_here(&intent)
    );
    assert_eq!(
        intent.inode_record_counts,
        counted_here(&intent),
        "意图那一版：5 号在左片，找得到上一版那条"
    );
    let mut current = intent.clone();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        allocator_of_the_mount,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        instance,
    )
    .expect("走空");
    println!(
        "attack-s2: 合法两片叶走空之后：记着的 {:?}，现数 {:?}",
        current.inode_record_counts,
        counted_here(&current)
    );
    assert_eq!(
        current.inode_record_counts,
        counted_here(&current),
        "走空之后：最后一批删掉的那条在左片"
    );
}
