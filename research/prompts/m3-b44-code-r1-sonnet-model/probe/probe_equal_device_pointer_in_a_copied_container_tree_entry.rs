//! C841（坏镜像上容器号加一会 panic，用户 2026-10-15 定「写路径报类型化错误」）：墓碑树、谱系树与退役树开新容器时取「最右那片的号 + 1」，
//! 最右那片的号是盘上读来的 8 字节。坏镜像把墓碑树最右那片的容器号写成 `u64::MAX`（容器头里的身份、根条目的分隔 key 与身份引用、
//! 根节点头的 key 区间一起改，各自重封、整单元校验和沿引用链补到根槽，读者核得过），可写挂载照常做成；下一次要开新容器的发布交回
//! `PublishError::NextContainerNumberPastTheTopOfItsRange`、不 panic，两块盘逐字节不变。改之前这一步加一溢出 panic（工作区开着 overflow-checks）。
//! 造法：新池新建文件之后建两个空 inode（2、3 号），把 2 号 unlink 到底（墓碑树一片容器、号 1、装一条墓碑），再改坏；
//! 挂载之后写入口把墓碑容器压到一片 1 条，发 3 号的 unlink 意图。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::parameters;
use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, first_tombstone_container_slot_of_the_newest_root,
    pool_with_new_inodes, root_slot_of_the_tree_of_kind_of_the_newest_root, unlink_to_the_end,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use common_pointer_head_corruption::{
    data_unit_bytes, node_bytes, tree_table_entry_of_kind, writable_devices_of,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, NextContainerNumberPastTheTopOfItsRange,
};
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::mount_writable;
use singlefs_core::records::TREE_KIND_TOMBSTONE;
use singlefs_core::retirement_tree::RetirementContainerCapacity;
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{publish_unlink_intent, PoolVersion, PoolWriter, PublishError};

/// 码 3 单元头里身份的容器号（8 字节，偏移 53：类标签副本 42、出生树 43、打包记录类型 51 之后，`unit::parse_packed_unit`）。
const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
/// 墓碑树根（码 2，key 宽 8）：头里最小 key 在 52、最大 key 在 60，条目区从 115 + 2 × 8 = 131 起（D18（块里携带什么信息） 已定项 18）。
const CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET: usize = 52;
const CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET: usize = 60;
const CONTAINER_TREE_ROOT_ENTRIES_START: usize = 131;
/// 容器树条目 120 字节：分隔 key 0（8 字节）、身份引用 8（出生树 8、打包记录类型 2、容器号 8、容器出生代 8，容器号在条目偏移 18）、子指针 34
/// （`records::parse_inode_internal_entry_or_refusal`）。
const CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET: usize = 0;
const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET: usize = 18;
/// 2 号 unlink 到底之后墓碑树唯一那片容器的号：零根时第一片的号是 1（D18（块里携带什么信息） 已定项 24 第 1 条）。
const CONTAINER_NUMBER_OF_THE_FIRST_TOMBSTONE_CONTAINER: u64 = 1;

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_u64_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 墓碑容器压到一片 1 条，别的照产品：开放容器装着一条墓碑就满了，下一条要开新容器。
fn tombstone_containers_of_one_record() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container: RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 探针（m3-b44-code-r1 正推腿 K4）：墓碑树根条目里指向第一片容器的子指针，两条位置条目改成同一块盘（槽、校验和不动，两份内容相同，读得通）。
/// 挂载之后发 3 号的 unlink 意图（容器压到 1 条，要开第二片），根被重写、第一片的子指针从盘上读来的值照抄写回。
/// 打印结局：mount=ok|err、publish=ok|err|panic。
#[test]
fn probe_equal_device_pointer_in_a_copied_container_tree_entry() {
    let mut pool = pool_with_new_inodes("probe-equal-device-pointer", 2);
    unlink_to_the_end(&mut pool, 2);
    let mut image = pool.memory_pool();
    let root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_TOMBSTONE);
    change_the_unit_and_propagate_its_checksum(&mut image, root_slot, node_bytes(), |root| {
        let location_zero_device = CONTAINER_TREE_ROOT_ENTRIES_START + 34 + 50;
        let location_one_device = location_zero_device + 14;
        let zero = root[location_zero_device..location_zero_device + 4].to_vec();
        let one = root[location_one_device..location_one_device + 4].to_vec();
        assert_ne!(zero, one, "前提：改之前两条位置条目在两块盘上");
        root[location_one_device..location_one_device + 4].copy_from_slice(&zero);
    });
    let mut devices = writable_devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices);
    let mut mounted = match mounted {
        Ok(mounted) => {
            println!("probe mount=ok");
            mounted
        }
        Err(refusal) => {
            println!("probe mount=err {refusal:?}");
            return;
        }
    };
    let current = match &mounted.current {
        PoolVersion::WithFile(version) => version.clone(),
        PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => panic!("带文件"),
    };
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(tombstone_containers_of_one_record());
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        publish_unlink_intent(
            &mut writer,
            &mut mounted.allocator,
            &current,
            InodeNumber(3),
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            mounted.output.instance,
        )
        .map(|_| ())
    }));
    match outcome {
        Ok(Ok(())) => println!("probe publish=ok"),
        Ok(Err(refusal)) => println!("probe publish=err {refusal:?}"),
        Err(payload) => {
            let text = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_default();
            println!("probe publish=panic {text}");
        }
    }
}
