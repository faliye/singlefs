use e7_index_bench::usize_from_u64;
use super::*;

/// A3：单元区容量。
#[test]
fn unit_area_capacity_matches_registered_value() {
    assert_eq!(UNIT_AREA_CAPACITY_SLOTS, 211968);
}

/// A1：分配记录树每节点条目数。
#[test]
fn records_per_node_matches_registered_value() {
    assert_eq!(RECORDS_PER_NODE_SINGLE_GENERATION, 812);
    assert_eq!(RECORDS_PER_NODE_DUAL_GENERATION, 580);
}

/// K3：根环坐标公式与暖机 txg。
#[test]
fn root_ring_coordinates_match_registered_formula() {
    assert_eq!(root_ring_region(3), 0);
    assert_eq!(root_ring_device(3), Device::Disk0);
    assert_eq!(root_ring_slot_in_region(3, 8), 1);
    let world = new_pool_file_creation_world(BASE_GEOMETRY);
    assert_eq!(world.pool.current_txg, 3, "首次挂载暖机恰 2 次（txg 1、2），新池新建文件 txg 3");
}

/// A2：W 的 txg mod 3 决定暖机次数。
#[test]
fn warm_up_publish_count_depends_on_txg_mod_three() {
    assert_eq!(ROOT_RING_REGION_DEVICE[0], Device::Disk0);
    assert_eq!(ROOT_RING_REGION_DEVICE[1], Device::Disk1);
    assert_eq!(ROOT_RING_REGION_DEVICE[2], Device::Disk0);
    // 模拟：从 txg u 开始暖机需要几次才覆盖两块盘（A2：0/1 → 1 次，2 → 2 次）。
    for (start_txg_mod_three, expected_warm_ups) in [(0u64, 1u64), (1, 1), (2, 2)] {
        let mut covered: BTreeSet<Device> = BTreeSet::new();
        covered.insert(ROOT_RING_REGION_DEVICE[usize_from_u64(start_txg_mod_three)]);
        let mut count = 0u64;
        let mut txg = start_txg_mod_three;
        while covered.len() < 2 {
            txg += 1;
            covered.insert(ROOT_RING_REGION_DEVICE[(txg % 3) as usize]);
            count += 1;
        }
        assert_eq!(count, expected_warm_ups, "start_txg_mod_three={start_txg_mod_three}");
    }
}
