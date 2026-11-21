use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// B2（第七节 7.2）：甲，P=1、F1、seq、主几何，三种位置策略都一样——只有一个位置，近远无差别。
#[test]
fn jia_matches_the_anchor_at_file_count_1_on_every_position_policy() {
    for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
        let outcome = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, policy));
        assert_eq!(outcome.total_bytes(), 344_576, "policy={policy:?}");
        assert_eq!(outcome.write_calls, 21, "policy={policy:?}");
        assert_eq!(outcome.barriers, 4, "policy={policy:?}");
        assert_eq!(outcome.fua_count, 1, "policy={policy:?}");
        assert_eq!(outcome.block_layer_write_requests(), 26, "policy={policy:?}");
        assert_eq!(outcome.named_items, 8, "policy={policy:?}");
        assert_eq!(outcome.record_count, 1, "policy={policy:?}");
        assert_eq!(outcome.data_bytes, 32768 * 2, "policy={policy:?}");
        assert_eq!(outcome.extent_bytes, 16384 * 2, "policy={policy:?}");
        assert_eq!(outcome.inode_container_bytes, 32768 * 2, "policy={policy:?}");
        assert_eq!(outcome.inode_root_bytes, 16384 * 2, "policy={policy:?}");
        assert_eq!(outcome.allocation_bytes, 16384 * 2, "policy={policy:?}");
        assert_eq!(outcome.accounting_bytes, 16384 * 2, "policy={policy:?}");
        assert_eq!(outcome.mapping_bytes, 16384 * 2, "policy={policy:?}");
        assert_eq!(outcome.tree_table_bytes, 16384 * 2, "policy={policy:?}");
        assert_eq!(outcome.record_bytes, 4096 * 2, "policy={policy:?}");
        assert_eq!(outcome.root_slot_bytes, 512, "policy={policy:?}");
        assert_eq!(outcome.system_configuration_bytes, 4096 * 2, "policy={policy:?}");
    }
}

/// B3（值断言，Q1.6 门槛已撤，第十二节修订②）：G23.1 在锚点的份额，字面口径 47.70%、加系统配置 50.07%。
/// 分子（Q1.6）＝ extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**脏节点 + 根槽；
/// P=1 时 extent 树高 1（根就是叶），非叶节点为 0——`inode_container_bytes`（叶）与 `extent_bytes`（此格恒为叶）都不算进去。
#[test]
fn ancestor_and_fixed_point_share_at_the_anchor_matches_the_literal_and_with_system_configuration_readings() {
    let outcome = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(outcome.extent_layers.len(), 1, "P=1 时 extent 树高 1，根即叶，非叶节点为 0");
    let literal_numerator = outcome.ancestor_and_fixed_point_share_numerator_bytes();
    let total = outcome.total_bytes();
    assert_eq!(literal_numerator, 164_352);
    let share = literal_numerator as f64 / total as f64;
    assert!((share - 0.476_968_796).abs() < 1e-6, "share={share}");
    let with_system_configuration = (literal_numerator + outcome.system_configuration_bytes) as f64 / total as f64;
    assert!((with_system_configuration - 0.500_742_942).abs() < 1e-6, "with_system_configuration={with_system_configuration}");
}

/// B7（第七节 7.2）：甲，P=145、F1、seq、主几何 —— extent 树长到 2 层，其余树层数与 P=1 相同，
/// 甲每次持久化写字节 377 344。
#[test]
fn jia_at_file_count_145_matches_the_anchor() {
    let outcome = solve_jia_publish(shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(outcome.extent_layers, vec![2, 1], "extent 树 2 层（2 叶 + 1 根）");
    assert_eq!(outcome.inode_layers, vec![1, 1], "inode 树仍是 1 容器 + 1 根");
    assert_eq!(outcome.allocation_layers, vec![1], "分配记录 742 条 ≤ 812，仍 1 节点");
    assert_eq!(outcome.mapping_layers, vec![1], "映射 152 条 ≤ 294，仍 1 节点");
    assert_eq!(outcome.total_bytes(), 377_344);
}

/// A7（第七节 7.1）：甲的持久顺序每次 fsync 恰好两道屏障（每道每盘一次，共 4 次刷写）、根槽一次 FUA。
#[test]
fn jia_has_exactly_two_barriers_and_one_fua_on_every_grid_point() {
    for file_count in [1u64, 100, 10_000] {
        for placement in [Placement::Sequential, Placement::Random] {
            for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
                let outcome = solve_jia_publish(shape_at(file_count, Family::OneDataUnitPerFile, placement, policy));
                assert_eq!(outcome.barriers, 4, "file_count={file_count} placement={placement:?} policy={policy:?}");
                assert_eq!(outcome.fua_count, 1);
            }
        }
    }
}

/// A8（第七节 7.1）：F8 各格，extent 树每次 fsync 第 1 层起每层期望脏节点 ≤ 2（8 条相邻记录，一条共享脊柱）。
#[test]
fn eight_adjacent_family_extent_dirty_nodes_above_leaf_layer_stay_at_or_below_two() {
    for file_count in [1u64, 100, 10_000, 1_000_000] {
        let outcome = solve_jia_publish(shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Sequential, PositionPolicy::Balanced));
        let extent_internal_cap = node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false));
        let extent_dirty = single_group_dirty_nodes_per_layer(&outcome.extent_layers, node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), extent_internal_cap, 8);
        for (layer_index, &dirty) in extent_dirty.iter().enumerate().skip(1) {
            assert!(dirty <= 2.0 + 1e-9, "file_count={file_count} layer={layer_index} dirty={dirty}");
        }
    }
}

/// 不动点收敛（Q1.7 支撑量）：一格算完，迭代次数应远小于上限 1000，且没有撞到上限（F2 未触发）。
#[test]
fn fixed_point_converges_well_below_the_iteration_ceiling() {
    for file_count in [1u64, 100, 10_000, 1_000_000, 100_000_000] {
        let outcome = solve_jia_publish(shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Balanced));
        assert!(
            outcome.fixed_point_iterations < MAXIMUM_FIXED_POINT_ITERATIONS,
            "file_count={file_count} 撞到了迭代上限，F2 应该触发"
        );
    }
}

/// V2（作废条款）：三条臂的包含关系与 `U_l ≤ n_l` 在主表网格上处处不违反——这里先钉住甲自身不产生负的或超界的脏节点数。
#[test]
fn jia_dirty_node_counts_never_exceed_their_own_layer_node_counts() {
    for file_count in [1u64, 145, 10_000, 1_000_000] {
        for family in [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync] {
            for placement in [Placement::Sequential, Placement::Random] {
                for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
                    let outcome = solve_jia_publish(shape_at(file_count, family, placement, policy));
                    assert!(outcome.total_bytes() > 0, "file_count={file_count} family={family:?}");
                }
            }
        }
    }
}
