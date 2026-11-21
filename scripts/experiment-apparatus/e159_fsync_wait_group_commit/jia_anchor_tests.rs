use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// B2（第七节 7.2）：甲，P=1、F1、seq、主几何，三种位置策略都一样；R1 之后分配记录 402 条
/// （第一次装置的口径 354，见 Q6.11）；总字节不变（P=1 时分配记录树仍是单节点）。
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
        assert_eq!(outcome.allocation_layers, vec![1], "policy={policy:?}");
    }
    let legacy = legacy::legacy_solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(legacy.total_bytes(), 344_576, "legacy 总字节应与 R1 之后相同（P=1 仍是单节点）");
}

/// B7（第七节 7.2）：甲，P=145、F1、seq、主几何 —— extent 树长到 2 层，甲每次持久化写字节 377 344。
#[test]
fn jia_at_file_count_145_matches_the_anchor() {
    let outcome = solve_jia_publish(shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(outcome.extent_layers, vec![2, 1], "extent 树 2 层（2 叶 + 1 根）");
    assert_eq!(outcome.inode_layers, vec![1, 1], "inode 树仍是 1 容器 + 1 根");
    assert_eq!(outcome.allocation_layers, vec![1], "分配记录 742 条（R1）≤ 812，仍 1 节点");
    assert_eq!(outcome.mapping_layers, vec![1], "映射 152 条 ≤ 294，仍 1 节点");
    assert_eq!(outcome.total_bytes(), 377_344);
}

/// B10（第七节 7.2，第三节 3.2 ①）：R1 之后分配记录树长到 2 层的门槛 P* = 181（P=180 恰 812 条），
/// legacy（R1 之前）门槛是 205。
#[test]
fn allocation_tree_height_threshold_moves_from_205_to_181_after_counting_data_units_in_backlog() {
    let at_180 = solve_jia_publish(shape_at(180, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    let at_181 = solve_jia_publish(shape_at(181, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(at_180.allocation_layers.len(), 1, "P=180 分配记录树仍是 1 层（R1 之后）");
    assert_eq!(at_181.allocation_layers.len(), 2, "P=181 分配记录树长到 2 层（R1 之后）");

    let legacy_at_204 = legacy::legacy_solve_jia_publish(shape_at(204, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    let legacy_at_205 = legacy::legacy_solve_jia_publish(shape_at(205, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(legacy_at_204.allocation_layers.len(), 1, "legacy：P=204 仍是 1 层");
    assert_eq!(legacy_at_205.allocation_layers.len(), 2, "legacy：P=205 长到 2 层（第一次装置的口径）");
}

/// A7（第七节 7.1）：甲的持久顺序每次 fsync 恰好两道屏障、根槽一次 FUA。
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

/// A8（第七节 7.1）：F8 各格，extent 树每次 fsync 第 1 层起每层期望脏节点 ≤ 2。
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

/// 不动点收敛（Q6.9 支撑量）：一格算完，迭代次数应远小于上限 1000（R1 让 W 更大，仍要收敛）。
/// 这一条只覆盖 F8A；F1 在同样规模上会撞上限，见 `fixed_point_iteration_ceiling_triggers_at_file_count_1e8_one_data_unit_per_file_random_balanced`——
/// 两条并存是如实的（F2 触发不等于模型错，登记第十节）。
#[test]
fn fixed_point_converges_well_below_the_iteration_ceiling() {
    for file_count in [1u64, 100, 10_000, 1_000_000, 100_000_000] {
        let outcome = solve_jia_publish(shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Balanced));
        assert!(outcome.fixed_point_iterations < MAXIMUM_FIXED_POINT_ITERATIONS, "file_count={file_count} 撞到了迭代上限，F2 应该触发");
    }
}

/// F2 第一半（登记第十节，追加要求）：P=10⁸、F1、rand、Lbalanced 一格
/// 迭代到上限仍未收敛（`row6_grid` 里这一格 `fixed_point_iterations=1000`），`jia_convergence_and_saturation_status` 要把它
/// 判成第一半触发；这一格第二半不该触发（extent/alloc/mapping 第 0 层脏节点占比都远低于 50%）。
#[test]
fn fixed_point_iteration_ceiling_triggers_at_file_count_1e8_one_data_unit_per_file_random_balanced() {
    let outcome = solve_jia_publish(shape_at(100_000_000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Balanced));
    assert_eq!(outcome.fixed_point_iterations, MAXIMUM_FIXED_POINT_ITERATIONS, "这一格应该撞满 1000 次迭代");
    let (half1, half2) = jia_convergence_and_saturation_status(&outcome);
    assert!(half1, "F2 第一半应该触发");
    assert!(half2.is_empty(), "这一格第二半不该触发：{half2:?}");
}

/// F2 第二半（登记第十节）：P=10⁶、F1、rand、Lfar 一格分配记录树第 0 层节点数 ≥ 64 且脏节点占比
/// ≥ 50%，`jia_convergence_and_saturation_status` 要把它判成第二半触发（触发的树含 `allocation`）；同一个 P、族、落点换成
/// Lbalanced 时不触发（占比只有约 39.7%，见运行记录），两者对照证明这条判据不是恒真恒假。
#[test]
fn layer_zero_saturation_triggers_on_allocation_tree_at_file_count_1e6_one_data_unit_per_file_random_far() {
    let far_outcome = solve_jia_publish(shape_at(1_000_000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Far));
    let (far_half1, far_half2) = jia_convergence_and_saturation_status(&far_outcome);
    assert!(!far_half1, "这一格应该收敛");
    assert!(far_half2.contains(&"allocation"), "L远 一格分配记录树应该触发第二半：{far_half2:?}");

    let balanced_outcome = solve_jia_publish(shape_at(1_000_000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Balanced));
    let (balanced_half1, balanced_half2) = jia_convergence_and_saturation_status(&balanced_outcome);
    assert!(!balanced_half1, "这一格应该收敛");
    assert!(balanced_half2.is_empty(), "同一个 P 换 Lbalanced 不该触发第二半：{balanced_half2:?}");
}

/// V2（作废条款）：甲自身不产生负的或超界的脏节点数。
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
