//! 代码轮 m4-close-code-r3 攻方腿 T1 的模型（只在冻结副本的拷贝上跑，不进仓）：读 inode 树的四个读者——可写挂载重建、冷走读、
//! 只读挂载打开池（`mounted_read::open_pool_for_read`）、快照第一个文件的冷读——今天判的「两片叶交叠」
//! （`recovery::inode_records_of_the_leaf_ascend_after_the_leaves_before`，拒因写 I-9.12）只比记录号，不比内部条目的分隔 key；
//! I-9.12 的字面（`crates/singlefs-checker/invariants/09-inode树结构.md` 那一行）是分隔 key 与相邻孩子的 key 区间。
//! 两张坏镜像都从合法的两片叶池（左片 1..=233、右片 234..=236，外加一个只读快照：它的 inode 根就是活头那一棵）改一处：
//! 右片那条内部条目的分隔 key 从它的容器号 234 改成别的值，根头 key 区间 [1, 236] 不动，改单元的校验和沿引用链补到根槽。
//! - 分隔 key 改成 200：不大于左片的最大号 233，池级 checker 判 I-9.12 红；四个读者都收下（记录号仍按叶序严格升序、都不小于分隔 key 与容器号）。
//! - 分隔 key 改成 235：大于右片的最小号 234，池级 checker 判 I-9.12 红；可写挂载重建、冷走读、快照冷读拒在 I-9.4（记录号小于分隔 key），
//!   只读挂载打开池不看分隔 key、收下——同一张镜像四个读者两种结局。
//! 每条用例断言的是今天的行为：绿 = 攻方说的那一形在冻结树上成立。观测行以 `attack-r3:` 开头。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, root_slot_of_the_tree_of_kind_of_the_newest_root,
};
use common_pointer_head_corruption::{newest_root, node_bytes, writable_devices_of};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, TreeIdentifier};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::records::{TREE_KIND_INODE, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_create_snapshot, publish_first_file, publish_new_inodes, warm_up,
    FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 一片 inode 叶装 233 条。
const INODE_LEAF_RECORDS: u64 = 233;
/// 码 2 节点头 131 字节，条目 120 字节、头 8 字节是分隔 key（`checker_known_bad_images.rs` 的 I-9.12 那一格改的是 131 起那 8 字节，第 0 条）：
/// 第 1 条（右片）的分隔 key 从 131 + 120 起。
const SEPARATOR_OF_THE_RIGHT_LEAF_OFFSET: usize = 131 + 120;

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

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个文件 → 建 232 个空 inode（左片 1..=233）→ 再建 3 个（右片 234..=236）→ 建一个只读快照。
/// 交回内存镜像与快照的树号。
fn image_of_two_inode_leaves_with_a_snapshot() -> (MemoryPool, TreeIdentifier) {
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
    let with_the_snapshot = {
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
        assert_eq!(
            inode_numbers_of_each_leaf(&two_leaves),
            vec![(1..=233).collect::<Vec<u64>>(), vec![234, 235, 236]],
            "前提：左片 1..=233、右片 234..=236"
        );
        publish_create_snapshot(&mut writer, &mut allocator, &two_leaves, instance)
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
        "前提：快照的 inode 根就是活头那一棵"
    );
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    (image, snapshot_entry.tree)
}

/// 把最新那条根的 inode 树根（活头与快照共用那一个节点）右片那条条目的分隔 key 从 234 改成 `separator`，根头 key 区间不动，校验和沿引用链补到根槽。
fn set_the_separator_of_the_right_leaf(image: &mut MemoryPool, separator: u64) {
    let inode_root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_INODE);
    change_the_unit_and_propagate_its_checksum(image, inode_root_slot, node_bytes(), |root| {
        let range = SEPARATOR_OF_THE_RIGHT_LEAF_OFFSET..SEPARATOR_OF_THE_RIGHT_LEAF_OFFSET + 8;
        let before = u64::from_le_bytes(root[range.clone()].try_into().expect("8 字节"));
        assert_eq!(before, 234, "改之前右片的分隔 key 是它的容器号 234");
        root[range].copy_from_slice(&separator.to_le_bytes());
    });
}

fn verdict_of_i_9_12(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-9.12")
        .map(|(_, verdict)| verdict)
        .expect("池级 checker 报 I-9.12")
}

/// 四个读者各自的结局：收下是 `Ok(())`，拒是拒因的 Debug。
struct FourReaders {
    writable_mount: Result<(), String>,
    cold_walk: Result<(), String>,
    read_only_open: Result<(), String>,
    snapshot_cold_read: Result<(), String>,
}

fn four_readers_on(image: &MemoryPool, snapshot: TreeIdentifier) -> FourReaders {
    let mut devices = writable_devices_of(image);
    let writable_mount = mount_writable(&parameters(), &mut devices)
        .map(|_mounted| ())
        .map_err(|refusal| format!("{refusal:?}"));
    let cold_walk = match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::Failed { failure, .. } => Err(format!("{failure:?}")),
        RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => Ok(()),
    };
    let read_only_open = open_pool_for_read(image, &newest_root(image))
        .map(|_mounted| ())
        .map_err(|refusal| format!("{refusal:?}"));
    let snapshot_cold_read = read_the_first_file_of_a_snapshot(image, &newest_root(image), snapshot)
        .map(|_reading| ())
        .map_err(|failure| format!("{failure:?}"));
    FourReaders {
        writable_mount,
        cold_walk,
        read_only_open,
        snapshot_cold_read,
    }
}

fn print_the_observations(shape: &str, image: &MemoryPool, readers: &FourReaders) {
    println!("attack-r3: shape={shape} checker_i_9_12={:?}", verdict_of_i_9_12(image));
    println!("attack-r3: shape={shape} writable_mount={:?}", readers.writable_mount);
    println!("attack-r3: shape={shape} cold_walk={:?}", readers.cold_walk);
    println!("attack-r3: shape={shape} read_only_open={:?}", readers.read_only_open);
    println!("attack-r3: shape={shape} snapshot_cold_read={:?}", readers.snapshot_cold_read);
}

/// 对照：没改过的两片叶池（带快照）I-9.12 成立，四个读者都收下。
#[test]
fn the_untouched_two_leaf_pool_holds_i_9_12_and_every_reader_accepts_it() {
    let (image, snapshot) = image_of_two_inode_leaves_with_a_snapshot();
    let readers = four_readers_on(&image, snapshot);
    print_the_observations("untouched", &image, &readers);
    assert_eq!(verdict_of_i_9_12(&image), InvariantVerdict::Holds, "合法池 I-9.12 成立");
    assert_eq!(readers.writable_mount, Ok(()), "可写挂载收下合法池");
    assert_eq!(readers.cold_walk, Ok(()), "冷走读收下合法池");
    assert_eq!(readers.read_only_open, Ok(()), "只读挂载收下合法池");
    assert_eq!(readers.snapshot_cold_read, Ok(()), "快照冷读收下合法池");
}

/// 分隔 key 200（≤ 左片最大号 233）：池级 checker 判 I-9.12 红，四个读者都收下——读者拒因写着 I-9.12 的那一判不判 I-9.12 字面的分隔 key 那两句。
#[test]
fn a_right_leaf_separator_not_above_the_left_leaf_is_red_on_i_9_12_and_accepted_by_every_reader() {
    let (mut image, snapshot) = image_of_two_inode_leaves_with_a_snapshot();
    set_the_separator_of_the_right_leaf(&mut image, 200);
    let readers = four_readers_on(&image, snapshot);
    print_the_observations("separator_200", &image, &readers);
    assert!(
        matches!(verdict_of_i_9_12(&image), InvariantVerdict::Violated(_)),
        "池级 checker 判 I-9.12 红"
    );
    assert_eq!(readers.writable_mount, Ok(()), "可写挂载收下 I-9.12 红的一版");
    assert_eq!(readers.cold_walk, Ok(()), "冷走读收下 I-9.12 红的一版");
    assert_eq!(readers.read_only_open, Ok(()), "只读挂载收下 I-9.12 红的一版");
    assert_eq!(readers.snapshot_cold_read, Ok(()), "快照冷读收下 I-9.12 红的一版");
}

/// 分隔 key 235（> 右片最小号 234）：池级 checker 判 I-9.12 红；三个读者拒在 I-9.4，只读挂载打开池收下——同一张镜像四个读者两种结局。
#[test]
fn a_right_leaf_separator_above_its_first_record_splits_the_four_readers() {
    let (mut image, snapshot) = image_of_two_inode_leaves_with_a_snapshot();
    set_the_separator_of_the_right_leaf(&mut image, 235);
    let readers = four_readers_on(&image, snapshot);
    print_the_observations("separator_235", &image, &readers);
    assert!(
        matches!(verdict_of_i_9_12(&image), InvariantVerdict::Violated(_)),
        "池级 checker 判 I-9.12 红"
    );
    for (reader, outcome) in [
        ("可写挂载", &readers.writable_mount),
        ("冷走读", &readers.cold_walk),
        ("快照冷读", &readers.snapshot_cold_read),
    ] {
        assert!(
            outcome.as_ref().is_err_and(|refusal| refusal.contains("I-9.4")),
            "{reader}拒在 I-9.4（记录号小于分隔 key）：{outcome:?}"
        );
    }
    assert_eq!(
        readers.read_only_open,
        Ok(()),
        "只读挂载打开池不看分隔 key，收下"
    );
}
