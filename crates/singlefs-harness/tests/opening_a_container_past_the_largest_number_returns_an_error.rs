//! C841（坏镜像上容器号加一会 panic，用户 2026-10-15 定「写路径报类型化错误」）：墓碑树、谱系树与退役树开新容器时取「最右那片的号 + 1」，
//! 最右那片的号是盘上读来的 8 字节。坏镜像把一棵容器树最右（也是唯一）那片的容器号写成 `u64::MAX`（容器头里的身份、根条目的分隔 key 与
//! 身份引用、根节点头的 key 区间一起改，各自重封、整单元校验和沿引用链补到根槽，读者核得过），可写挂载照常做成；下一次要开新容器的发布交回
//! `PublishError::NextContainerNumberPastTheTopOfItsRange`、不 panic，两块盘逐字节不变、分配器的记录不动。不判的话这一步加一溢出 panic
//! （工作区开着 overflow-checks）。三棵树各一条：墓碑树走 unlink 意图，谱系树走管理员回退，退役树走删空一片 inode 叶容器的最后一批。
//!
//! 挂载那一层（岔路单 `research/prompts/m3-b44-code-r1-forks.md` 那一行「取号之前预演」）：删除队列非空、挂载续做有一步要给
//! 退役树开新容器而最右那片的号是 `u64::MAX` 时，可写挂载在取号之前预演整个续做、交回这个错，两块盘逐字节不变；再挂一次同样。
//! 不预演的话这一错在续做里才交回，那时取号、写行那次发布与续做里排在它前面的各步已经落盘。三条：续做第一步就要开新容器；
//! 续做头两步是不删空叶的中间一批、第三步最后一批才要开新容器；不补环窗口的窄盘上续做走到一半被空间拒、推一串抬 F 再接着走，
//! 最后一批才要开新容器（预演照真发推抬 F、再试，才走得到那一批）。

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, parameters, PoolImageGeometry};
use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest,
};
use common_deletion_in_progress::{
    batches_of_one_data_unit, first_container_slot_of_the_tree_of_kind_of_the_newest_root,
    pool_with_a_three_unit_file, pool_with_new_inodes, publish_new_inodes_and_advance_the_pool,
    publish_the_intent_of, roll_back_to, root_slot_of_the_tree_of_kind_of_the_newest_root,
    unlink_to_the_end, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use common_pointer_head_corruption::{
    change_the_unit_and_propagate_its_checksum_on, data_unit_bytes, node_bytes,
    tree_table_entry_of_kind, writable_devices_of, WritableDevices,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, TreeIdentifier};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::{
    ContainerTreeNodeEntries, NextContainerNumberPastTheTopOfItsRange,
};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_deletion_switches,
    roll_back_by_a_forward_publish, DeletionQueueContinuationOutcome, MountError,
    MountSpaceAdmission, RollbackError, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::{refusal_is_short_of_space, UserChange};
use singlefs_core::records::{TREE_KIND_LINEAGE, TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE};
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, PublishError,
    TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

/// 码 3 单元头里身份的容器号（8 字节，偏移 53：类标签副本 42、出生树 43、打包记录类型 51 之后，`unit::parse_packed_unit`）。
const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
/// 容器树根（码 2，key 宽 8）：头里最小 key 在 52、最大 key 在 60，条目区从 115 + 2 × 8 = 131 起（D18（块里携带什么信息） 已定项 18）。
const CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET: usize = 52;
const CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET: usize = 60;
const CONTAINER_TREE_ROOT_ENTRIES_START: usize = 131;
/// 容器树条目 120 字节：分隔 key 0（8 字节）、身份引用 8（出生树 8、打包记录类型 2、容器号 8、容器出生代 8，容器号在条目偏移 18）、子指针 34
/// （`records::parse_inode_internal_entry_or_refusal`）。三棵容器树的内部条目同一形态（`container_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES`）。
const CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET: usize = 0;
const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET: usize = 18;
/// 只供测试的容器容量：一片装 1 条，装着一条就满了，下一条要开新容器。
const RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
/// 不补环窗口的窄盘那一格：单元区几槽、第一个文件长到几个数据单元。草稿探针（单元区 256–360 槽每 4 槽 × unlink 之前先覆盖写 0、1 次 ×
/// 长到 12、16 个单元，造法同 [`narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion`]，
/// 没改坏的镜像上逐格现挂）定的是 304 槽、不先覆盖写、16 个单元：取号之前那一判放行（写行之后不推抬 F），续做里一步被空间拒、
/// 推一串两次空发布之后接着走、16 步走到底。相邻几格多半是续做推不出空间（推满仍放不下）或一串都不推。
const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES: u64 = 304;
const DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES: usize = 16;

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_u64_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 删除开关：批大小与节点照产品，`tombstone`、`lineage`、`retirement` 三样容器容量由调用方给。
fn deletion_switches_with_container_capacities(
    tombstone_container_capacity: TombstoneContainerCapacity,
    lineage_container_capacity: LineageContainerCapacity,
    retirement_container_capacity: RetirementContainerCapacity,
) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity,
        lineage_container_capacity,
        retirement_container_capacity,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 墓碑容器一片 1 条，别的照产品。
fn deletion_switches_with_one_record_per_tombstone_container() -> DeletionSwitches {
    deletion_switches_with_container_capacities(
        TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
                .expect("1..=815"),
        ),
        LineageContainerCapacity::FromTheContainerFormat,
        RetirementContainerCapacity::FromTheContainerFormat,
    )
}

/// 谱系容器一片 1 条，别的照产品。
fn deletion_switches_with_one_record_per_lineage_container() -> DeletionSwitches {
    deletion_switches_with_container_capacities(
        TombstoneContainerCapacity::FromTheContainerFormat,
        LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
                .expect("1..=1359"),
        ),
        RetirementContainerCapacity::FromTheContainerFormat,
    )
}

/// 只供测试的退役容器容量：一片 1 条。
fn one_record_per_retirement_container() -> RetirementContainerCapacity {
    RetirementContainerCapacity::CappedForTests(
        RetirementRecordsPerContainerCappedForTests::new(RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
            .expect("1..=959"),
    )
}

/// 退役容器一片 1 条，别的照产品。
fn deletion_switches_with_one_record_per_retirement_container() -> DeletionSwitches {
    deletion_switches_with_container_capacities(
        TombstoneContainerCapacity::FromTheContainerFormat,
        LineageContainerCapacity::FromTheContainerFormat,
        one_record_per_retirement_container(),
    )
}

/// 一批放掉一个数据单元、退役容器一片 1 条，别的照产品。
fn deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(
) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity: one_record_per_retirement_container(),
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 种类 `kind` 那棵容器树最右（也是唯一）那片容器的号从 `number_before` 改成 `u64::MAX`，读者核的几处一起改：容器头里的身份、
/// 根条目的分隔 key 与身份引用、根节点头的最小与最大 key（根只有这一条条目）。容器先改（它的新校验和补进根条目的子指针），
/// 根再改（新校验和补进树表、根槽）。映射 key 不含容器号（`records::mapping_key_for_node`），映射不用动。
/// `geometry` 是镜像所在那一组盘的几何：补校验和按它找单元与 journal 环。
fn move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
    image: &mut MemoryPool,
    geometry: &PoolImageGeometry,
    kind: u16,
    number_before: u64,
) {
    let container_slot = first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        geometry,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(
                u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET),
                number_before,
                "种类 {kind} 那棵容器树唯一那片容器头里的号"
            );
            set_u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, u64::MAX);
        },
    );
    let root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        geometry,
        root_slot,
        node_bytes(),
        |root| {
            let entry = CONTAINER_TREE_ROOT_ENTRIES_START;
            for offset in [
                CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET,
                CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET,
                entry + CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET,
                entry + CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET,
            ] {
                assert_eq!(
                    u64_at(root, offset),
                    number_before,
                    "种类 {kind} 那棵容器树根偏移 {offset} 那 8 字节是唯一那片容器的号"
                );
                set_u64_at(root, offset, u64::MAX);
            }
        },
    );
}

/// 有树的现行那一版（带文件或有树没文件）。
fn version_of_trees(current: &PoolVersion) -> TransactionOutput {
    match current {
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            version.clone()
        }
        PoolVersion::WithoutFile(_) => panic!("这几份镜像的树表不是 0 条：现行那一版有树"),
    }
}

/// 每块盘此刻的字节。
fn images_of(devices: &WritableDevices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

fn assert_every_device_image_is_unchanged(
    devices: &WritableDevices,
    images_before: &[(DeviceIdentity, SparseDevice)],
    what: &str,
) {
    for (identity, image_before) in images_before {
        let (_, device) = devices
            .iter()
            .find(|(device_identity, _)| device_identity == identity)
            .expect("交进来的每块盘都还在");
        assert!(
            device.image == *image_before,
            "{what}：盘 {identity:?} 逐字节不变"
        );
    }
}

/// 报的是种类 `tree` 那棵树开新容器时容器号到顶，最右那片的号 `u64::MAX`。
fn is_the_container_number_past_the_top_of_its_range_in(
    refusal: &PublishError,
    tree: TreeIdentifier,
) -> bool {
    matches!(
        refusal,
        PublishError::NextContainerNumberPastTheTopOfItsRange(
            NextContainerNumberPastTheTopOfItsRange {
                tree: refused_tree,
                rightmost_container: u64::MAX,
            }
        ) if *refused_tree == tree
    )
}

/// 场景：新池新建文件之后建两个空 inode（2、3 号），把 2 号 unlink 到底（墓碑树一片容器、号 1、装一条墓碑），墓碑树唯一那片的号改成 `u64::MAX`；
/// 挂载之后写入口把墓碑容器压到一片 1 条，发 3 号的 unlink 意图。
/// 预期：意图那次发布交回 `NextContainerNumberPastTheTopOfItsRange`（墓碑树、`u64::MAX`），两块盘逐字节不变、分配器的记录不动。
#[test]
fn an_unlink_intent_that_opens_a_tombstone_container_after_one_numbered_at_the_top_of_its_range_is_refused_before_any_write(
) {
    let mut pool = pool_with_new_inodes("container-number-at-the-top-of-its-range", 2);
    unlink_to_the_end(&mut pool, 2);
    let number_before = pool
        .output
        .tombstone_containers
        .last()
        .expect("墓碑树一片容器")
        .contents
        .identity
        .container;
    let mut image = pool.memory_pool();
    let tombstone_tree = tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).tree;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        TREE_KIND_TOMBSTONE,
        number_before,
    );

    let mut devices = writable_devices_of(&image);
    let mut mounted = mount_writable(&parameters(), &mut devices)
        .expect("读者核得过容器号 u64::MAX 那片墓碑容器：可写挂载照常做成");
    let current = version_of_trees(&mounted.current);
    assert_eq!(
        current
            .tombstone_containers
            .last()
            .map(|container| container.contents.identity.container),
        Some(u64::MAX),
        "从盘上重建出来的墓碑树最右那片的号是 u64::MAX"
    );
    let images_after_the_mount = images_of(&devices);
    let allocator_records_after_the_mount = mounted.allocator.records().to_vec();

    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(
        deletion_switches_with_one_record_per_tombstone_container().container_tree_capacities(),
    );
    let refusal = publish_unlink_intent(
        &mut writer,
        &mut mounted.allocator,
        &current,
        InodeNumber(3),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        mounted.output.instance,
    )
    .expect_err("墓碑容器压到 1 条、唯一那片已满：3 号的墓碑要开新容器，号 u64::MAX 加一装不下");
    assert!(
        is_the_container_number_past_the_top_of_its_range_in(&refusal, tombstone_tree),
        "报的是墓碑树开新容器时容器号到顶：{refusal:?}"
    );
    drop(writer);
    assert_every_device_image_is_unchanged(
        &devices,
        &images_after_the_mount,
        "意图那次发布在任何写之前交回",
    );
    assert_eq!(
        mounted.allocator.records(),
        allocator_records_after_the_mount.as_slice(),
        "分配器的记录不动"
    );
}

/// 场景：新池新建文件之后建一个空 inode，再挂着回退到第一个文件那一版（谱系树一片容器、装一条谱系记录），谱系树唯一那片的号改成 `u64::MAX`；
/// 挂载之后分配器装上谱系容器一片 1 条的开关（回退照分配器上的开关取容量），再回退到第一个文件那一版。
/// 预期：回退交回 `RollbackError::Publish`，原因 `NextContainerNumberPastTheTopOfItsRange`（谱系树、`u64::MAX`），两块盘逐字节不变、
/// 分配器的记录不动。
#[test]
fn a_rollback_that_opens_a_lineage_container_after_one_numbered_at_the_top_of_its_range_is_refused_before_any_write(
) {
    let mut pool = build_pool("lineage-container-number-at-the-top-of-its-range");
    let first_file_version = pool.output.clone();
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    roll_back_to(&mut pool, &first_file_version);
    let number_before = pool
        .output
        .lineage_containers
        .last()
        .expect("谱系树一片容器")
        .contents
        .identity
        .container;
    let mut image = pool.memory_pool();
    let lineage_tree = tree_table_entry_of_kind(&image, TREE_KIND_LINEAGE).tree;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        TREE_KIND_LINEAGE,
        number_before,
    );

    let mut devices = writable_devices_of(&image);
    let mut mounted = mount_writable(&parameters(), &mut devices)
        .expect("读者核得过容器号 u64::MAX 那片谱系容器：可写挂载照常做成");
    let mut current = version_of_trees(&mounted.current);
    assert_eq!(
        current
            .lineage_containers
            .last()
            .map(|container| container.contents.identity.container),
        Some(u64::MAX),
        "从盘上重建出来的谱系树最右那片的号是 u64::MAX"
    );
    mounted
        .allocator
        .set_deletion_switches(deletion_switches_with_one_record_per_lineage_container());
    let images_after_the_mount = images_of(&devices);
    let allocator_records_after_the_mount = mounted.allocator.records().to_vec();

    let refusal = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut mounted.allocator,
        &mut current,
        RollbackTarget {
            instance: first_file_version.root.instance,
            checkpoint_txg: first_file_version.root.checkpoint_txg,
        },
    )
    .expect_err(
        "谱系容器压到 1 条、唯一那片已满：这次回退的谱系记录要开新容器，号 u64::MAX 加一装不下",
    );
    assert!(
        matches!(
            &refusal,
            RollbackError::Publish(failed)
                if is_the_container_number_past_the_top_of_its_range_in(&failed.cause, lineage_tree)
        ),
        "报的是谱系树开新容器时容器号到顶：{refusal:?}"
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_after_the_mount,
        "回退那次发布在任何写之前交回",
    );
    assert_eq!(
        mounted.allocator.records(),
        allocator_records_after_the_mount.as_slice(),
        "分配器的记录不动"
    );
}

/// 新池新建文件之后把第一个文件 unlink 到底（inode 树唯一那片叶容器删空、身份消失，退役树一片容器、装一条退役记录），再建一个空 inode（2 号，
/// 新开一片叶容器）。
fn pool_with_one_leaf_retired_and_inode_two_in_a_new_leaf(tag: &str) -> common::BuiltPool {
    let mut pool = build_pool(tag);
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    assert_eq!(
        pool.output
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .collect::<Vec<_>>(),
        [1],
        "退役树一片容器、一条退役记录"
    );
    pool
}

/// 退役树唯一那片容器的号（改成 `u64::MAX` 之前）。
fn number_of_the_only_retirement_container(pool: &common::BuiltPool) -> u64 {
    pool.output
        .retirement_containers
        .last()
        .expect("退役树一片容器")
        .contents
        .identity
        .container
}

/// 场景：新池新建文件之后把第一个文件 unlink 到底（退役树一片容器、装一条退役记录），再建 2 号；退役树唯一那片的号改成 `u64::MAX`。
/// 挂载之后写入口把退役容器压到一片 1 条，发 2 号的 unlink 意图（不碰退役树），再发它的最后一批（2 号那片叶删空、身份消失，要追加一条退役记录）。
/// 预期：最后一批交回 `NextContainerNumberPastTheTopOfItsRange`（退役树、`u64::MAX`），两块盘与意图那次之后逐字节相同、分配器的记录不动。
#[test]
fn an_unlink_final_batch_that_opens_a_retirement_container_after_one_numbered_at_the_top_of_its_range_is_refused_before_any_write(
) {
    let pool = pool_with_one_leaf_retired_and_inode_two_in_a_new_leaf(
        "retirement-container-number-at-the-top-of-its-range",
    );
    let number_before = number_of_the_only_retirement_container(&pool);
    let mut image = pool.memory_pool();
    let retirement_tree = tree_table_entry_of_kind(&image, TREE_KIND_RETIREMENT).tree;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        TREE_KIND_RETIREMENT,
        number_before,
    );

    let mut devices = writable_devices_of(&image);
    let mut mounted = mount_writable(&parameters(), &mut devices)
        .expect("读者核得过容器号 u64::MAX 那片退役容器：可写挂载照常做成");
    let current = version_of_trees(&mounted.current);
    assert_eq!(
        current
            .retirement_containers
            .last()
            .map(|container| container.contents.identity.container),
        Some(u64::MAX),
        "从盘上重建出来的退役树最右那片的号是 u64::MAX"
    );
    let publish_parameters = parameters();
    let capacities =
        deletion_switches_with_one_record_per_retirement_container().container_tree_capacities();
    let intent = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        writer.set_container_tree_capacities(capacities);
        publish_unlink_intent(
            &mut writer,
            &mut mounted.allocator,
            &current,
            InodeNumber(2),
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            mounted.output.instance,
        )
        .expect("意图那次不退役叶容器、不碰退役树")
    };
    let images_after_the_intent = images_of(&devices);
    let allocator_records_after_the_intent = mounted.allocator.records().to_vec();

    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let refusal = publish_unlink_step(
        &mut writer,
        &mut mounted.allocator,
        &intent,
        InodeNumber(2),
        UnlinkBatchSize::OneExtentLowerLeaf,
        mounted.output.instance,
    )
    .expect_err(
        "退役容器压到 1 条、唯一那片已满：2 号那片叶的退役记录要开新容器，号 u64::MAX 加一装不下",
    );
    assert!(
        is_the_container_number_past_the_top_of_its_range_in(&refusal, retirement_tree),
        "报的是退役树开新容器时容器号到顶：{refusal:?}"
    );
    drop(writer);
    assert_every_device_image_is_unchanged(
        &devices,
        &images_after_the_intent,
        "最后一批在任何写之前交回",
    );
    assert_eq!(
        mounted.allocator.records(),
        allocator_records_after_the_intent.as_slice(),
        "分配器的记录不动"
    );
}

/// 照 `parameters` 装 `deletion_switches` 这组删除开关做可写挂载，挂载被拒：交回拒的那个错。
fn writable_mount_under_deletion_switches_refused(
    parameters: &MakeFilesystemParameters,
    devices: &mut WritableDevices,
    deletion_switches: DeletionSwitches,
) -> MountError {
    match mount_writable_with_test_only_switches_and_deletion_switches(
        parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches,
    ) {
        Ok(mounted) => panic!(
            "续做有一步要开的退役容器号装不下，可写挂载却做成了：实例 {:?}",
            mounted.output.instance
        ),
        Err(refusal) => refusal,
    }
}

/// 拒的是 C841（退役树、`u64::MAX`），而且一个写都没发（三份写账都是空的）。
fn assert_refused_before_acquisition_by_the_retirement_container_number(
    refusal: &MountError,
    retirement_tree: TreeIdentifier,
) {
    let MountError::Publish(failed) = refusal else {
        panic!("可写挂载交回 C841 那个错（MountError::Publish）：{refusal:?}");
    };
    assert!(
        is_the_container_number_past_the_top_of_its_range_in(&failed.cause, retirement_tree),
        "原因是退役树开新容器时容器号到顶：{refusal:?}"
    );
    assert!(
        failed.writes_of_persisted_publishes.is_empty()
            && failed.writes_of_failed_publishes.is_empty(),
        "取号之前交回：这次挂载一次发布都没落盘、没有失败的落盘：{refusal:?}"
    );
}

/// 场景：新池新建文件之后把第一个文件 unlink 到底（退役树一片容器、装一条退役记录），再建 2 号、发 2 号的 unlink 意图（删除队列 {2}）；
/// 退役树唯一那片的号改成 `u64::MAX`。装退役容器一片 1 条的删除开关做可写挂载：续做第一步就是 2 号的最后一批，2 号那片叶删空、要给退役树开新容器。
/// 预期：可写挂载在取号之前预演续做、第一步就交回 `MountError::Publish`（原因 `NextContainerNumberPastTheTopOfItsRange`，退役树、`u64::MAX`，
/// 写账全空），两块盘逐字节不变；在同一组盘上再挂一次，同样拒、盘仍不变。
#[test]
fn a_writable_mount_whose_deletion_continuation_opens_a_retirement_container_after_one_numbered_at_the_top_of_its_range_is_refused_before_acquisition(
) {
    let mut pool = pool_with_one_leaf_retired_and_inode_two_in_a_new_leaf(
        "mount-continuation-retirement-container-at-the-top-of-its-range",
    );
    publish_the_intent_of(&mut pool, 2);
    assert_eq!(
        pool.output.deleted_inodes,
        [InodeNumber(2)],
        "删除队列 {{2}}：挂载续做第一步是 2 号的最后一批"
    );
    let number_before = number_of_the_only_retirement_container(&pool);
    let mut image = pool.memory_pool();
    let retirement_tree = tree_table_entry_of_kind(&image, TREE_KIND_RETIREMENT).tree;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        TREE_KIND_RETIREMENT,
        number_before,
    );
    let mut devices = writable_devices_of(&image);
    let images_before_the_mounts = images_of(&devices);

    let first_refusal = writable_mount_under_deletion_switches_refused(
        &parameters(),
        &mut devices,
        deletion_switches_with_one_record_per_retirement_container(),
    );
    assert_refused_before_acquisition_by_the_retirement_container_number(
        &first_refusal,
        retirement_tree,
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_before_the_mounts,
        "可写挂载在取号之前拒",
    );

    let second_refusal = writable_mount_under_deletion_switches_refused(
        &parameters(),
        &mut devices,
        deletion_switches_with_one_record_per_retirement_container(),
    );
    assert_refused_before_acquisition_by_the_retirement_container_number(
        &second_refusal,
        retirement_tree,
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_before_the_mounts,
        "再挂一次同样在取号之前拒",
    );
}

/// 新池新建文件之后把第一个文件顺序写成三个数据单元，把它 unlink 到底（三个单元在一片 extent 下段叶里，产品批大小下意图之后一批就是最后一批；
/// inode 树唯一那片叶容器删空、身份消失，退役树一片容器、一条退役记录），再挂着回退到三个单元那一版（inode 树与 extent 树取那一版的，
/// 退役树随现行那一版），发第一个文件的 unlink 意图：删除队列 {1}，第一个文件三个数据单元都在、独占一片叶容器。
fn pool_with_one_leaf_retired_and_the_three_unit_first_file_queued_for_deletion(
    tag: &str,
) -> common::BuiltPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    let three_unit_version = pool.output.clone();
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    roll_back_to(&mut pool, &three_unit_version);
    publish_the_intent_of(&mut pool, FIRST_INODE_NUMBER);
    assert_eq!(
        pool.output.deleted_inodes,
        [InodeNumber(FIRST_INODE_NUMBER)],
        "删除队列 {{1}}"
    );
    assert_eq!(
        pool.output.data_pointers.len(),
        3,
        "第一个文件三个数据单元都在：批大小 1 下续做头两步是不删空叶的中间一批"
    );
    assert_eq!(
        pool.output
            .inode_leaf_container_contents()
            .iter()
            .map(|container| container.records.len())
            .collect::<Vec<_>>(),
        [1],
        "inode 树一片叶容器、只装第一个文件那一条：最后一批删空它、要追加一条退役记录"
    );
    assert_eq!(
        pool.output
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .collect::<Vec<_>>(),
        [1],
        "退役树一片容器、一条退役记录：退役容器压到一片 1 条时它已满"
    );
    pool
}

/// 场景：第一个文件三个数据单元、删除队列 {1}、退役树一片容器装一条退役记录（[`pool_with_one_leaf_retired_and_the_three_unit_first_file_queued_for_deletion`]）；
/// 退役树唯一那片的号改成 `u64::MAX`。装一批一个单元、退役容器一片 1 条的删除开关做可写挂载：续做头两步各放掉尾部一个单元
/// （不删空叶、不碰退役树），第三步是最后一批，第一个文件那片叶删空、要给退役树开新容器。
/// 预期：可写挂载在取号之前预演整个续做、在最后一批报 C841，交回 `MountError::Publish`（原因 `NextContainerNumberPastTheTopOfItsRange`，
/// 退役树、`u64::MAX`，写账全空），两块盘逐字节不变；在同一组盘上再挂一次，同样拒、盘仍不变。
#[test]
fn a_writable_mount_whose_deletion_continuation_opens_a_retirement_container_only_at_its_final_batch_is_refused_before_acquisition(
) {
    let pool = pool_with_one_leaf_retired_and_the_three_unit_first_file_queued_for_deletion(
        "mount-continuation-final-batch-retirement-container-at-the-top-of-its-range",
    );
    let number_before = number_of_the_only_retirement_container(&pool);
    let mut image = pool.memory_pool();
    let retirement_tree = tree_table_entry_of_kind(&image, TREE_KIND_RETIREMENT).tree;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        TREE_KIND_RETIREMENT,
        number_before,
    );
    let mut devices = writable_devices_of(&image);
    let images_before_the_mounts = images_of(&devices);

    let first_refusal = writable_mount_under_deletion_switches_refused(
        &parameters(),
        &mut devices,
        deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
    );
    assert_refused_before_acquisition_by_the_retirement_container_number(
        &first_refusal,
        retirement_tree,
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_before_the_mounts,
        "可写挂载在取号之前拒",
    );

    let second_refusal = writable_mount_under_deletion_switches_refused(
        &parameters(),
        &mut devices,
        deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
    );
    assert_refused_before_acquisition_by_the_retirement_container_number(
        &second_refusal,
        retirement_tree,
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_before_the_mounts,
        "再挂一次同样在取号之前拒",
    );
}

/// 不补环窗口的窄盘（`common_admission::start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off`）
/// 那一组盘的几何：改坏单元之后补校验和、抄 journal 记录按窄盘自己的盘宽与环长找单元区与 journal 环。
fn geometry_of_the_narrow_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolImageGeometry {
    PoolImageGeometry {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
    }
}

/// 不补环窗口的窄盘（[`UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES`] 槽）上第一个文件顺序写长到 [`DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES`]
/// 个数据单元，经会话 unlink 到底（inode 树唯一那片叶容器删空、身份消失，退役树一片容器、一条退役记录），再回退到长到那么多单元的那一版
/// （退役树随现行那一版），在这个进程里发第一个文件的 unlink 意图，然后进程退出（会话丢掉）：删除队列 {1}，第一个文件的数据单元都在、
/// 独占一片叶容器。交回这一刻的镜像、窄盘的几何与退役树唯一那片容器的号。
fn narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion(
) -> (MemoryPool, PoolImageGeometry, u64) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES,
        );
    pool.sequential_write(
        (DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES - 1) * data_unit_payload_capacity()
            + 100,
    )
    .unwrap_or_else(|refusal| panic!("顺序写把第一个文件长到 16 个单元：{refusal:?}"));
    let version_with_the_grown_file = pool.current_file_version().clone();
    pool.write_time_seconds += 1;
    let write_time_seconds_of_the_unlink = pool.write_time_seconds;
    pool.publish(
        UserChange::Unlink {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds: write_time_seconds_of_the_unlink,
        },
        None,
    )
    .unwrap_or_else(|refusal| panic!("经会话把第一个文件 unlink 到底：{refusal:?}"));
    let geometry = geometry_of_the_narrow_pool(&pool);
    let PoolUnderTest {
        session,
        devices,
        parameters,
        write_time_seconds,
        ..
    } = &mut pool;
    let session = session.as_mut().expect("这一刻有可写会话");
    let mut current = session
        .current
        .version_of_trees()
        .expect("unlink 到底之后有树")
        .clone();
    roll_back_by_a_forward_publish(
        parameters,
        devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_with_the_grown_file.root.instance,
            checkpoint_txg: version_with_the_grown_file.root.checkpoint_txg,
        },
    )
    .unwrap_or_else(|refusal| panic!("挂着回退到长到 16 个单元那一版：{refusal:?}"));
    let intent = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图那次：回退之后第一个文件活着、队列空")
    };
    assert_eq!(
        intent.deleted_inodes,
        [InodeNumber(FIRST_INODE_NUMBER)],
        "删除队列 {{1}}"
    );
    assert_eq!(
        intent.data_pointers.len(),
        DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES,
        "第一个文件的数据单元都在：批大小 1 下续做前面 15 步是不删空叶的中间一批"
    );
    assert_eq!(
        intent
            .inode_leaf_container_contents()
            .iter()
            .map(|container| container.records.len())
            .collect::<Vec<_>>(),
        [1],
        "inode 树一片叶容器、只装第一个文件那一条：最后一批删空它、要追加一条退役记录"
    );
    assert_eq!(
        intent
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .collect::<Vec<_>>(),
        [1],
        "退役树一片容器、一条退役记录：退役容器压到一片 1 条时它已满"
    );
    let number_of_the_only_retirement_container = intent
        .retirement_containers
        .last()
        .expect("退役树一片容器")
        .contents
        .identity
        .container;
    pool.session = None;
    (
        pool.image(),
        geometry,
        number_of_the_only_retirement_container,
    )
}

/// 场景：不补环窗口的窄盘上第一个文件 16 个数据单元、删除队列 {1}、退役树一片容器装一条退役记录
/// （[`narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion`]）。装一批一个单元、退役容器一片 1 条的删除开关做可写挂载。
/// 前提（没改坏的那一份上真挂一次）：取号之前那一判放行、写行之后不推抬 F；续做里一步被拒是空间不够、推一串抬 F，之后接着走、16 步走到底，
/// 推的那一串在最后一批之前；最后一批给退役树开了新容器。改坏的那一份把退役树唯一那片的号改成 `u64::MAX`：续做只有最后一批要开新容器，
/// 而分配器拷贝上那一趟预演在推抬 F 那一步之前就因空间不够接不上一版。
/// 预期：可写挂载在取号之前把抬 F 也预演进去、走到最后一批报 C841，交回 `MountError::Publish`（原因 `NextContainerNumberPastTheTopOfItsRange`，
/// 退役树、`u64::MAX`，写账全空），两块盘逐字节不变；在同一组盘上再挂一次，同样拒、盘仍不变。
/// 判别力：预演遇到空间不够就停时，挂载取号、写行、续做推抬 F、走到最后一批才交 C841，写账不空、盘变了。
#[test]
fn a_writable_mount_whose_deletion_continuation_raises_the_floor_before_its_final_batch_opens_a_retirement_container_is_refused_before_acquisition(
) {
    let (image, narrow, number_before) =
        narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion();
    let retirement_tree = tree_table_entry_of_kind(&image, TREE_KIND_RETIREMENT).tree;

    let mut devices_of_the_image_not_changed = plain_devices_on(&image);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &narrow.parameters,
        &mut devices_of_the_image_not_changed,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
    )
    .unwrap_or_else(|refusal| {
        panic!("没改坏的那一份上可写挂载做成、续做推过抬 F 走到底：{refusal:?}")
    });
    let output = &mounted.output;
    assert!(
        matches!(
            output.space_admission,
            MountSpaceAdmission::AdmittedBeforeAcquisition
        ),
        "取号之前那一判放行、写行之后不推抬 F：{:?}",
        output.space_admission
    );
    let continuation_txgs: Vec<CheckpointTxg> = output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.root.checkpoint_txg)
        .collect();
    let floor_raise_txgs: Vec<CheckpointTxg> = output
        .floor_raises_pushed_by_the_continuation
        .iter()
        .flat_map(|raised| {
            raised
                .publishes
                .iter()
                .map(|publish| publish.root.checkpoint_txg)
        })
        .collect();
    assert!(
        matches!(
            output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ) && continuation_txgs.len() == DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES
            && !output.floor_raises_pushed_by_the_continuation.is_empty()
            && output
                .refusals_that_pushed_the_continuation_floor_raises
                .iter()
                .all(refusal_is_short_of_space),
        "续做里一步被拒是空间不够、推了抬 F，之后 16 步走到底：续做 {continuation_txgs:?}，推的 {floor_raise_txgs:?}，推它的 {:?}，结局 {:?}",
        output.refusals_that_pushed_the_continuation_floor_raises,
        output.deletion_queue_continuation_outcome
    );
    let final_batch_txg = *continuation_txgs.last().expect("续做 16 步");
    assert!(
        floor_raise_txgs
            .iter()
            .all(|txg| continuation_txgs[0] < *txg && *txg < final_batch_txg),
        "推的抬 F 插在续做的两步之间、在最后一批之前：续做 {continuation_txgs:?}，推的 {floor_raise_txgs:?}"
    );
    assert_eq!(
        version_of_trees(&mounted.current)
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .collect::<Vec<_>>(),
        [1, 1],
        "最后一批给退役树开了新容器（一片 1 条，原来那片已满）"
    );
    drop(mounted);

    let mut image = image;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &narrow,
        TREE_KIND_RETIREMENT,
        number_before,
    );
    let mut devices = plain_devices_on(&image);
    let images_before_the_mounts = images_of(&devices);

    let first_refusal = writable_mount_under_deletion_switches_refused(
        &narrow.parameters,
        &mut devices,
        deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
    );
    assert_refused_before_acquisition_by_the_retirement_container_number(
        &first_refusal,
        retirement_tree,
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_before_the_mounts,
        "可写挂载在取号之前拒",
    );

    let second_refusal = writable_mount_under_deletion_switches_refused(
        &narrow.parameters,
        &mut devices,
        deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
    );
    assert_refused_before_acquisition_by_the_retirement_container_number(
        &second_refusal,
        retirement_tree,
    );
    assert_every_device_image_is_unchanged(
        &devices,
        &images_before_the_mounts,
        "再挂一次同样在取号之前拒",
    );
}
