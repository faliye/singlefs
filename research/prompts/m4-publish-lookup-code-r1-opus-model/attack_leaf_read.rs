//! m4-publish-lookup-code-r1 攻方腿 T2：只拷进改后那棵树。造一版中央映射树不止一片叶的池，再把叶的字节换着装（两片叶互换、一片叶抄进另一片、
//! 同一角色在 `units` 里多一项放前放后、删掉一片叶的单元），对每一把能想到的 key（进映射的、任一片叶里有的、全零、全 FF、短一字节、长一字节、空）
//! 比 `CentralMappingOfAVersionReadLeafByLeaf`（同一个读者跨 key 复用）与逐 key 的 `central_mapping_locations_in_the_version`，panic 也比（文案逐字）。
mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::CompactionSwitches;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable_with_compaction_switches;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
    CentralMappingOfAVersionReadLeafByLeaf, FirstFile, PoolWriter, PublishedUnit, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_harness::memory_pool::SparseBlockDevice;

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "（非字符串 panic）".to_owned())
}

fn pool_whose_mapping_tree_has_several_leaves(files: u64, units_per_file: usize) -> TransactionOutput {
    let parameters = common::parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_file, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, CompactionSwitches::PRODUCT).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let first = session.current.version_of_trees().expect("有树").inode_number_watermark();
    let mut time = common::FIXED_WRITE_TIME_SECONDS + 10;
    session
        .publish_user_change(&mut devices, UserChange::NewInodes { count: files, write_time_seconds: time })
        .expect("建 inode");
    for inode in first..first + files {
        time += 1;
        let content: Vec<u8> = (0..units_per_file * data_unit_payload_capacity() - 5)
            .map(|index| u8::try_from((index as u64 * 31 + inode) % 251).expect("小于 256"))
            .collect();
        session
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWriteOfAnInode { inode: InodeNumber(inode), content: &content, write_time_seconds: time },
            )
            .expect("写文件");
    }
    session.current.version_of_trees().expect("有树").clone()
}

/// 一版里中央映射树的叶（解得开、层级 0 的映射树节点单元）的下标。
fn leaf_positions(version: &TransactionOutput) -> Vec<usize> {
    version
        .units
        .iter()
        .enumerate()
        .filter(|(_, unit)| matches!(unit.identity, TransactionUnit::MappingTree | TransactionUnit::MappingTreeNodeBelowTheRoot(_)))
        .filter(|(_, unit)| parse_index_node(&unit.bytes).is_ok_and(|node| node.level == 0))
        .map(|(position, _)| position)
        .collect()
}

/// 这一版上每一把 key 两种查法的答案逐个比（panic 比文案）；交回比了几把、几把两边都 panic、几把查到。
fn compare_every_key(name: &str, version: &TransactionOutput, keys: &[Vec<u8>]) -> (usize, usize, usize) {
    let mut reader = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let (mut panics, mut found) = (0, 0);
    for key in keys {
        let batched = catch_unwind(AssertUnwindSafe(|| reader.locations_of_key(key))).map_err(|payload| panic_text(payload.as_ref()));
        let per_key = catch_unwind(AssertUnwindSafe(|| central_mapping_locations_in_the_version(version, key))).map_err(|payload| panic_text(payload.as_ref()));
        assert_eq!(batched, per_key, "{name}：key {key:02x?} 两种查法不同");
        panics += usize::from(per_key.is_err());
        found += usize::from(matches!(per_key, Ok(singlefs_core::transaction::MappingLookup::Found(_))));
    }
    println!("M4T2 version={name} keys={} both_panicked={panics} found={found}", keys.len());
    (keys.len(), panics, found)
}

#[test]
fn leaf_by_leaf_read_agrees_with_the_per_key_lookup_on_rearranged_leaves() {
    let version = pool_whose_mapping_tree_has_several_leaves(48, 8);
    let leaves = leaf_positions(&version);
    println!("M4T2 mapping_tree_nodes={} leaves={}", version.central_mapping_tree.shape.nodes().len(), leaves.len());
    assert!(leaves.len() >= 2, "要不止一片叶");
    let mut keys: Vec<Vec<u8>> = version.mapped_units.iter().map(|(_, key)| key.clone()).collect();
    for position in &leaves {
        for entry in parse_index_node(&version.units[*position].bytes).expect("叶").entries {
            keys.push(entry[..27].to_vec());
        }
    }
    let sample = keys[keys.len() / 2].clone();
    keys.extend([vec![0u8; 27], vec![0xff; 27], sample[..26].to_vec(), [sample.clone(), vec![0]].concat(), Vec::new()]);
    let mut totals = (0, 0, 0);
    let mut add = |result: (usize, usize, usize)| {
        totals.0 += result.0;
        totals.1 += result.1;
        totals.2 += result.2;
    };
    add(compare_every_key("original", &version, &keys));
    for (left_index, left) in leaves.iter().enumerate() {
        for right in &leaves[left_index + 1..] {
            let mut swapped = version.clone();
            let left_bytes = swapped.units[*left].bytes.clone();
            swapped.units[*left].bytes = swapped.units[*right].bytes.clone();
            swapped.units[*right].bytes = left_bytes;
            add(compare_every_key(&format!("swap {left} {right}"), &swapped, &keys));
            let mut copied = version.clone();
            copied.units[*right].bytes = copied.units[*left].bytes.clone();
            add(compare_every_key(&format!("copy {left} into {right}"), &copied, &keys));
        }
    }
    for position in &leaves {
        let duplicate = PublishedUnit { slot: version.units[*position].slot, identity: version.units[*position].identity, bytes: vec![0; version.units[*position].bytes.len()] };
        let mut appended = version.clone();
        appended.units.push(duplicate.clone());
        add(compare_every_key(&format!("zeroed duplicate after {position}"), &appended, &keys));
        let mut prepended = version.clone();
        prepended.units.insert(0, duplicate);
        add(compare_every_key(&format!("zeroed duplicate before {position}"), &prepended, &keys));
        let mut removed = version.clone();
        removed.units.remove(*position);
        add(compare_every_key(&format!("leaf {position} removed"), &removed, &keys));
    }
    println!("M4T2 total keys_compared={} both_panicked={} found={}", totals.0, totals.1, totals.2);
}
