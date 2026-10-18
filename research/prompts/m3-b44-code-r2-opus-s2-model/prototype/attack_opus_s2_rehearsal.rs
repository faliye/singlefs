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

// ======================= 攻方腿 opus-s2 的原型（只在草稿副本里）=======================

use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, FlippedBit, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::segments::FixedGeometry;
use std::io::Write as _;

type FaultDevices = Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>;

fn fixed_geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn fault_devices_on(image: &MemoryPool, plan: &SharedFaultPlan) -> FaultDevices {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect()
}

fn fault_device_images(devices: &FaultDevices) -> Vec<SparseDevice> {
    devices
        .iter()
        .map(|(_, device)| device.wrapped_device().image.clone())
        .collect()
}

fn variant_name(text: &str) -> String {
    text.chars()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect()
}

/// 一次挂载的结局，按「盘变没变、交的什么」分类。
fn classify_the_mount(
    result: std::thread::Result<Result<singlefs_core::mount::Mounted, MountError>>,
    devices: &FaultDevices,
    before: &[SparseDevice],
    retirement_tree: TreeIdentifier,
) -> String {
    let changed = fault_device_images(devices) != before;
    match result {
        Err(_) => format!("panic changed={changed}"),
        Ok(Ok(mounted)) => format!(
            "mounted outcome={} changed={changed}",
            variant_name(&format!(
                "{:?}",
                mounted.output.deletion_queue_continuation_outcome
            ))
        ),
        Ok(Err(MountError::Publish(failed))) => {
            let c841 = is_the_container_number_past_the_top_of_its_range_in(
                &failed.cause,
                retirement_tree,
            );
            format!(
                "Publish cause={} c841={c841} persisted={} failed={} changed={changed}",
                variant_name(&format!("{:?}", failed.cause)),
                failed.writes_of_persisted_publishes.len(),
                failed.writes_of_failed_publishes.len()
            )
        }
        Ok(Err(other)) => format!(
            "{} changed={changed}",
            variant_name(&format!("{other:?}"))
        ),
    }
}

fn mount_on(
    parameters: &MakeFilesystemParameters,
    devices: &mut FaultDevices,
) -> std::thread::Result<Result<singlefs_core::mount::Mounted, MountError>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mount_writable_with_test_only_switches_and_deletion_switches(
            parameters,
            devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
        )
    }))
}

fn sweep_output_path(name: &str) -> std::path::PathBuf {
    let directory = std::env::var("ATTACK_OUTPUT_DIRECTORY").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&directory).join(name)
}

/// 窄盘 C841 镜像（续做推抬 F、最后一批开退役容器）上，整池第 n 次读注入一次故障，n 扫过挂载里的每一次读。
/// 记每个 n 的结局；「Publish c841=true persisted>0 changed=true」就是取号之前那一判被故障放过、真做在写之后交 C841。
fn sweep_one_read_fault_over_every_read_of_the_mount(fault_of: fn(u64) -> InjectedFault, tag: &str) {
    let (image, narrow, number_before) =
        narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion();
    let retirement_tree = tree_table_entry_of_kind(&image, TREE_KIND_RETIREMENT).tree;
    let mut image = image;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &narrow,
        TREE_KIND_RETIREMENT,
        number_before,
    );
    let geometry = fixed_geometry_of(&narrow.parameters);

    let measuring_plan = SharedFaultPlan::unarmed(geometry);
    let mut measuring_devices = fault_devices_on(&image, &measuring_plan);
    let before = fault_device_images(&measuring_devices);
    let started = std::time::Instant::now();
    let measured = mount_on(&narrow.parameters, &mut measuring_devices);
    let one_mount = started.elapsed();
    let reads_of_the_mount = measuring_plan.calls().reads;
    let baseline = classify_the_mount(measured, &measuring_devices, &before, retirement_tree);

    let limit: u64 = std::env::var("ATTACK_READ_LIMIT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(reads_of_the_mount);
    let start: u64 = std::env::var("ATTACK_READ_START")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let step: u64 = std::env::var("ATTACK_READ_STEP")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let mut file = std::fs::File::create(sweep_output_path(&format!(
        "sweep-{tag}-from-{start}-step-{step}.tsv"
    )))
    .expect("建扫描输出");
    writeln!(
        file,
        "# baseline\t{baseline}\treads={reads_of_the_mount}\tone_mount_ms={}",
        one_mount.as_millis()
    )
    .expect("写");
    let mut histogram: std::collections::BTreeMap<String, (u64, Vec<u64>)> =
        std::collections::BTreeMap::new();
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for ordinal in (start..=limit.min(reads_of_the_mount)).step_by(usize::try_from(step).expect("步长")) {
        let plan = SharedFaultPlan::armed(
            geometry,
            FaultSchedule::the_nth_call_across_the_pool(fault_of(ordinal), ordinal),
        );
        let mut devices = fault_devices_on(&image, &plan);
        let result = mount_on(&narrow.parameters, &mut devices);
        let outcome = classify_the_mount(result, &devices, &before, retirement_tree);
        let fired = plan.fired();
        let fired_at = fired
            .first()
            .map(|fault| format!("dev={:?} off={} len={}", fault.device, fault.offset.0, fault.length))
            .unwrap_or_else(|| "not-fired".to_string());
        writeln!(file, "{ordinal}\t{outcome}\t{fired_at}").expect("写");
        let entry = histogram.entry(outcome).or_insert((0, Vec::new()));
        entry.0 += 1;
        if entry.1.len() < 8 {
            entry.1.push(ordinal);
        }
    }
    std::panic::set_hook(previous_hook);
    println!("baseline: {baseline}; reads of the mount: {reads_of_the_mount}; one mount {one_mount:?}");
    for (outcome, (count, first)) in &histogram {
        println!("{count}\t{outcome}\tfirst ordinals {first:?}");
    }
}

#[test]
fn attack_s2_read_fails_swept_over_every_read_of_the_mount() {
    sweep_one_read_fault_over_every_read_of_the_mount(|_| InjectedFault::ReadFails, "read-fails");
}

#[test]
fn attack_s2_read_returns_corrupted_bytes_swept_over_every_read_of_the_mount() {
    sweep_one_read_fault_over_every_read_of_the_mount(
        |_| InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit { byte_index: 0, bit_index: 0 },
        },
        "read-corrupted",
    );
}

// ---- 带读序号与「是不是只进内存那一趟」标记的盘：按读序号让一段窗口里的读报错 ----

use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes};

#[derive(Default)]
struct ReadLog {
    ordinal: u64,
    /// (读序号, 是不是只进内存那一趟, 盘, 偏移, 长度)
    reads: Vec<(u64, bool, u32, u64, u64)>,
    failing: std::collections::BTreeSet<u64>,
    fired: Vec<(u64, bool)>,
}

#[derive(Clone, Default)]
struct SharedReadLog(std::rc::Rc<std::cell::RefCell<ReadLog>>);

struct LoggingDevice {
    inner: SparseBlockDevice,
    device: DeviceIdentity,
    log: SharedReadLog,
}

impl BlockDevice for LoggingDevice {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let mut log = self.log.0.borrow_mut();
        log.ordinal += 1;
        let ordinal = log.ordinal;
        let in_walk = singlefs_core::mount::ATTACK_MEMORY_WALK_DEPTH
            .load(std::sync::atomic::Ordering::SeqCst)
            > 0;
        log.reads.push((
            ordinal,
            in_walk,
            self.device.0,
            offset.0,
            u64::try_from(buffer.len()).expect("长度"),
        ));
        if log.failing.contains(&ordinal) {
            log.fired.push((ordinal, in_walk));
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "攻方注入的读错",
            )));
        }
        drop(log);
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
    fn journal_record_offsets_hint(
        &self,
        ring_start: singlefs_core::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<singlefs_core::address::DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

type LoggingDevices = Vec<(DeviceIdentity, LoggingDevice)>;

fn logging_devices_on(image: &MemoryPool, log: &SharedReadLog) -> LoggingDevices {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                LoggingDevice {
                    inner: device,
                    device: identity,
                    log: log.clone(),
                },
            )
        })
        .collect()
}

fn logging_device_images(devices: &LoggingDevices) -> Vec<SparseDevice> {
    devices.iter().map(|(_, device)| device.inner.image.clone()).collect()
}

fn mount_on_logging(
    parameters: &MakeFilesystemParameters,
    devices: &mut LoggingDevices,
) -> std::thread::Result<Result<singlefs_core::mount::Mounted, MountError>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mount_writable_with_test_only_switches_and_deletion_switches(
            parameters,
            devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            deletion_switches_in_batches_of_one_data_unit_with_one_record_per_retirement_container(),
        )
    }))
}

fn classify_logging(
    result: std::thread::Result<Result<singlefs_core::mount::Mounted, MountError>>,
    changed: bool,
    retirement_tree: TreeIdentifier,
) -> String {
    match result {
        Err(_) => format!("panic changed={changed}"),
        Ok(Ok(mounted)) => format!(
            "mounted outcome={} changed={changed}",
            variant_name(&format!("{:?}", mounted.output.deletion_queue_continuation_outcome))
        ),
        Ok(Err(MountError::Publish(failed))) => format!(
            "Publish cause={} c841={} persisted={} failed={} changed={changed}",
            variant_name(&format!("{:?}", failed.cause)),
            is_the_container_number_past_the_top_of_its_range_in(&failed.cause, retirement_tree),
            failed.writes_of_persisted_publishes.len(),
            failed.writes_of_failed_publishes.len()
        ),
        Ok(Err(other)) => format!("{} changed={changed}", variant_name(&format!("{other:?}"))),
    }
}

/// 窄盘 C841 镜像：先不注入挂一次，记下每次读的序号与它在不在只进内存那一趟里；再对那一趟里的每个读序号 n，
/// 让 n 起连续 `ATTACK_WINDOW` 个读序号（只取落在那一趟里的）报读错，挂一次，记结局与盘变没变。
#[test]
fn attack_s2_read_fault_window_inside_the_memory_walk() {
    let (image, narrow, number_before) =
        narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion();
    let retirement_tree = tree_table_entry_of_kind(&image, TREE_KIND_RETIREMENT).tree;
    let mut image = image;
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &narrow,
        TREE_KIND_RETIREMENT,
        number_before,
    );
    let window: u64 = std::env::var("ATTACK_WINDOW").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let start: usize = std::env::var("ATTACK_READ_START").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let step: usize = std::env::var("ATTACK_READ_STEP").ok().and_then(|v| v.parse().ok()).unwrap_or(1);

    let log = SharedReadLog::default();
    let mut devices = logging_devices_on(&image, &log);
    let before = logging_device_images(&devices);
    let measured = mount_on_logging(&narrow.parameters, &mut devices);
    let changed = logging_device_images(&devices) != before;
    let baseline = classify_logging(measured, changed, retirement_tree);
    let reads = log.0.borrow().reads.clone();
    let walk_ordinals: Vec<u64> = reads.iter().filter(|read| read.1).map(|read| read.0).collect();
    let mut measure = std::fs::File::create(sweep_output_path("reads-of-the-refused-mount.tsv")).expect("建");
    for read in &reads {
        writeln!(measure, "{}\t{}\t{}\t{}\t{}", read.0, read.1, read.2, read.3, read.4).expect("写");
    }
    println!(
        "baseline: {baseline}; reads {}; reads inside the memory walk {} (ordinals {:?}..={:?})",
        reads.len(),
        walk_ordinals.len(),
        walk_ordinals.first(),
        walk_ordinals.last()
    );
    let mut file = std::fs::File::create(sweep_output_path(&format!(
        "window-{window}-from-{start}-step-{step}.tsv"
    )))
    .expect("建");
    writeln!(file, "# baseline\t{baseline}\treads={}\twalk={}", reads.len(), walk_ordinals.len()).expect("写");
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut histogram: std::collections::BTreeMap<String, (u64, Vec<u64>)> = std::collections::BTreeMap::new();
    let count: usize = std::env::var("ATTACK_READ_COUNT").ok().and_then(|v| v.parse().ok()).unwrap_or(usize::MAX);
    for &first in walk_ordinals.iter().skip(start).step_by(step).take(count) {
        let faulted_log = SharedReadLog::default();
        faulted_log.0.borrow_mut().failing = (first..first + window).collect();
        let mut devices = logging_devices_on(&image, &faulted_log);
        let result = mount_on_logging(&narrow.parameters, &mut devices);
        let changed = logging_device_images(&devices) != before;
        let outcome = classify_logging(result, changed, retirement_tree);
        let fired = faulted_log.0.borrow().fired.clone();
        let all_fired_inside_the_walk = fired.iter().all(|(_, in_walk)| *in_walk);
        let first_read = reads.iter().find(|read| read.0 == first).copied();
        writeln!(
            file,
            "{first}\t{outcome}\tfired={}\tall_inside_walk={all_fired_inside_the_walk}\tread={first_read:?}",
            fired.len()
        )
        .expect("写");
        let entry = histogram
            .entry(format!("{outcome} all_inside_walk={all_fired_inside_the_walk}"))
            .or_insert((0, Vec::new()));
        entry.0 += 1;
        if entry.1.len() < 8 {
            entry.1.push(first);
        }
    }
    std::panic::set_hook(previous_hook);
    for (outcome, (count, first)) in &histogram {
        println!("{count}\t{outcome}\tfirst ordinals {first:?}");
    }
}

// ---- J1：extent 上段叶里别的 inode 那一条（照抄写回）的指针不在重建时 I-2.5 那一判的清单里 ----

/// 第一个文件三个数据单元那一版的 extent 上段根（叶）里追加一条 2 号 inode 的条目：照抄 1 号那一条，inode 号改成 2，
/// 指针的两条位置条目设备身份对调（逆序）；头里条目数、声明长度、最大 key 一起改，重封、校验和沿引用链补到根槽。
fn append_a_descending_entry_of_inode_two_to_the_extent_upper_leaf(image: &mut MemoryPool) {
    let slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, singlefs_core::records::TREE_KIND_EXTENT);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
        slot,
        node_bytes(),
        |node| {
            let key_width = usize::from(node[51]);
            assert_eq!(node[50], 0, "extent 上段根就是叶（层级 0）");
            let header_end = 86 + 2 * key_width;
            let entries_start = header_end + 29;
            let declared = usize::from(u16::from_le_bytes([node[8], node[9]]));
            let entry_count = usize::from(u16::from_le_bytes([node[header_end - 4], node[header_end - 3]]));
            let entry_width = usize::from(u16::from_le_bytes([node[header_end - 2], node[header_end - 1]]));
            assert_eq!(entry_count, 1, "只有第一个文件那一条");
            assert_eq!(declared, entry_width);
            let mut entry = node[entries_start..entries_start + entry_width].to_vec();
            assert_eq!(u64_at(&entry, 8), FIRST_INODE_NUMBER, "key 里 inode 号在偏移 8");
            set_u64_at(&mut entry, 8, 2);
            let pointer = key_width + 1;
            let first_device = pointer + 50;
            let second_device = first_device + 14;
            let a = entry[first_device..first_device + 4].to_vec();
            let b = entry[second_device..second_device + 4].to_vec();
            assert_ne!(a, b, "前提：两条位置条目在两块盘上");
            entry[first_device..first_device + 4].copy_from_slice(&b);
            entry[second_device..second_device + 4].copy_from_slice(&a);
            node[entries_start + declared..entries_start + declared + entry_width].copy_from_slice(&entry);
            let new_declared = u16::try_from(declared + entry_width).expect("宽");
            node[8..10].copy_from_slice(&new_declared.to_le_bytes());
            node[header_end - 4..header_end - 2].copy_from_slice(&2u16.to_le_bytes());
            // 头里的 key 区间是这片叶的位置罩的那一段（读者按位置核），不跟着条目改。
        },
    );
}

#[test]
fn attack_s2_descending_pointer_in_an_extent_upper_leaf_entry_of_another_inode() {
    let pool = pool_with_a_three_unit_file("attack-s2-extent-leaf-entry-of-inode-two");
    let mut image = pool.memory_pool();
    drop(pool);
    append_a_descending_entry_of_inode_two_to_the_extent_upper_leaf(&mut image);
    let mut devices = writable_devices_of(&image);
    let mounted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mount_writable(&parameters(), &mut devices)
    }));
    let mut mounted = match mounted {
        Err(_) => {
            println!("J1 extent leaf: mount=panic");
            return;
        }
        Ok(Err(refusal)) => {
            println!("J1 extent leaf: mount=refused {refusal:?}");
            return;
        }
        Ok(Ok(mounted)) => mounted,
    };
    println!("J1 extent leaf: mount=ok instance={:?}", mounted.output.instance);
    let current = version_of_trees(&mounted.current);
    let instance = mounted.output.instance;
    let parameters = parameters();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let intent = publish_unlink_intent(
            &mut writer,
            &mut mounted.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES + 10,
            instance,
        )
        .map_err(|error| format!("intent {error:?}"))?;
        let mut version = intent;
        let mut steps = 0;
        while !version.deleted_inodes.is_empty() {
            version = publish_unlink_step(
                &mut writer,
                &mut mounted.allocator,
                &version,
                InodeNumber(FIRST_INODE_NUMBER),
                UnlinkBatchSize::OneExtentLowerLeaf,
                instance,
            )
            .map_err(|error| format!("step {steps} {error:?}"))?;
            steps += 1;
        }
        Ok::<usize, String>(steps)
    }));
    match outcome {
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_default();
            println!("J1 extent leaf: unlink=panic {message}");
        }
        Ok(result) => println!("J1 extent leaf: unlink={result:?}"),
    }
}
