use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// Q3.1（通用实现）在 P = 1 一格必须与 7.3 手算格逐字节相同：`U_3 = 250880/344576`。
#[test]
fn whole_unit_rewrite_general_implementation_matches_the_hand_calculated_cell_at_file_count_1() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let share = whole_unit_rewrite_counterfactual_share(shape, PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES);
    assert!((share - 250_880.0 / 344_576.0).abs() < 1e-9, "share={share}");
}

/// Q2.1–Q2.4：P = 1 一格四样固定点各自只贡献 16384×2 字节（B2 锚点逐项相同），
/// 抑制其中任何一样时其余树仍是单节点、没有连带效应，四个上界应该逐字节相等。
#[test]
fn fixed_point_upper_bounds_are_equal_at_file_count_1_because_every_fixed_point_is_one_node() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let expected = 32_768.0 / 344_576.0;
    let accounting = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { accounting: true, ..Default::default() });
    let allocation = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { allocation: true, ..Default::default() });
    let mapping = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { mapping: true, ..Default::default() });
    let tree_table = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { tree_table: true, ..Default::default() });
    for (name, value) in [("accounting", accounting), ("allocation", allocation), ("mapping", mapping), ("tree_table", tree_table)] {
        assert!((value - expected).abs() < 1e-9, "{name}={value} expected={expected}");
    }
    // 钉住这一格的实际读数（0.0951 < 0.10）：P=1 一格四样固定点各自都在 10% 门槛**之下**，
    // 这是这一格的真实结果，不是判据——门槛怎么判随 P 变化，见产物里的 P 轴轨迹。
    assert!(expected < 0.10, "P=1 一格：{expected}");
}

/// Q2 的抑制不能让脏节点数变负或让 W 变负——抑制掉的树在极小 P 上仍应收敛。
#[test]
fn suppressing_any_fixed_point_still_converges_and_stays_nonnegative() {
    for suppressed in [
        SuppressedFixedPoints { accounting: true, ..Default::default() },
        SuppressedFixedPoints { allocation: true, ..Default::default() },
        SuppressedFixedPoints { mapping: true, ..Default::default() },
        SuppressedFixedPoints { tree_table: true, ..Default::default() },
    ] {
        for file_count in [1u64, 145, 10_000, 1_000_000] {
            let shape = shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Far);
            let outcome = solve_jia_publish_with_suppressed_fixed_points(shape, suppressed);
            assert!(outcome.fixed_point_iterations < MAXIMUM_FIXED_POINT_ITERATIONS, "file_count={file_count} suppressed={suppressed:?}");
            assert!(outcome.total_bytes() > 0, "file_count={file_count} suppressed={suppressed:?}");
        }
    }
}

/// Q2.5：树表条目数 T 从 7 涨到 82 时树表本身长到 2 层（`tree_layers` 与其它树同一个公式），
/// 上界跟着变大（更多节点可能被这次持久化碰到）。
#[test]
fn tree_table_reaches_two_layers_at_82_entries_and_upper_bound_does_not_shrink() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let share_at_7 = tree_table_counterfactual_share(shape, 7);
    let share_at_81 = tree_table_counterfactual_share(shape, 81);
    let share_at_82 = tree_table_counterfactual_share(shape, 82);
    assert!((share_at_7 - share_at_81).abs() < 1e-9, "T=7 与 T=81 都还是 1 层，上界应该相同：{share_at_7} vs {share_at_81}");
    assert!(share_at_82 >= share_at_81, "T=82 长到 2 层，上界不该变小：{share_at_82} vs {share_at_81}");
}

/// B8（第七节 7.2）：第 4 行每单元每盘省下的字节，Q4.1 反事实上界用它对拍。
#[test]
fn data_unit_rounding_per_unit_savings_match_the_anchor() {
    let cases: [(u64, u64, u64); 8] = [
        (100, 512, 32_256),
        (100, 4096, 28_672),
        (4100, 512, 28_160),
        (4100, 4096, 24_576),
        (16_000, 512, 16_384),
        (16_000, 4096, 16_384),
        (32_634, 512, 0),
        (32_634, 4096, 0),
    ];
    for (content_bytes, pbs, expected_saving_per_unit_per_device) in cases {
        let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
        let saving = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
        assert_eq!(saving, expected_saving_per_unit_per_device, "content_bytes={content_bytes} pbs={pbs}");
    }
}

/// M18 的防等价取样点：s = 4000 是唯一让数据单元头 134 改变取整的一档（B8 附注）。
#[test]
fn content_size_4000_is_header_sensitive_unlike_the_other_sampled_sizes() {
    for (content_bytes, pbs) in [(100u64, 512u64), (4100, 512), (16_000, 512), (32_634, 512)] {
        let with_header = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
        let without_header = content_bytes.div_ceil(pbs) * pbs;
        assert_eq!(with_header, without_header, "content_bytes={content_bytes}：这几档不该对头字节敏感");
    }
    let with_header = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + 4000).div_ceil(512) * 512;
    let without_header = 4000u64.div_ceil(512) * 512;
    assert_ne!(with_header, without_header, "s=4000 应该是防等价的那一档");
}

/// Q4.1（防等价 M18）：s = 4000、pbs = 512 一档必须真的算头（`DATA_UNIT_HEADER_AND_TRAILER_BYTES`），
/// 不算头会把取整从 4608 变成 4096，省下的字节从 28160 变成 28672——这一档就是防这个的。
#[test]
fn data_unit_rounding_at_content_size_4000_accounts_for_the_data_unit_header() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let baseline = solve_jia_publish(PoolShape { geometry: Geometry { physical_block_size_bytes: 512, ..shape.geometry }, ..shape }).total_bytes();
    let share = data_unit_rounding_counterfactual_share(shape, 4000, 512);
    let expected = (2 * 28_160) as f64 / baseline as f64;
    assert!((share - expected).abs() < 1e-12, "share={share} expected={expected}");
}

/// Q4.1：F8A 一格每次 fsync 改 8 个数据单元，反事实上界应该正好是 F1 同一格的 8 倍（都用 d×D×省下的字节）。
#[test]
fn data_unit_rounding_scales_linearly_with_data_units_touched_per_fsync() {
    let shape_one_data_unit_per_file = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shape_f8a = PoolShape { family: Family::EightAdjacentDataUnitsPerFsync, ..shape_one_data_unit_per_file };
    let baseline_one_data_unit_per_file = solve_jia_publish(shape_one_data_unit_per_file).total_bytes() as f64;
    let baseline_f8a = solve_jia_publish(shape_f8a).total_bytes() as f64;
    let share_one_data_unit_per_file = data_unit_rounding_counterfactual_share(shape_one_data_unit_per_file, 4100, 512);
    let share_f8a = data_unit_rounding_counterfactual_share(shape_f8a, 4100, 512);
    let numerator_one_data_unit_per_file = share_one_data_unit_per_file * baseline_one_data_unit_per_file;
    let numerator_f8a = share_f8a * baseline_f8a;
    assert!(
        (numerator_f8a / numerator_one_data_unit_per_file - 8.0).abs() < 1e-6,
        "numerator_one_data_unit_per_file={numerator_one_data_unit_per_file} numerator_f8a={numerator_f8a}"
    );
}
