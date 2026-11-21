//! 第 7 次重跑（登记 `research/prompts/e156-r7-prereg.md`）：洞族的锚点格与 Python 标签的单测。

use super::hole_family_anchor_cells_of;

/// 族表（第 7 次重跑登记 5.2）：每个 ρ 上 S = 4 / 8 / 16 各 14 / 18 / 26 格（含 k = 0 的基准族），k = 0 的终点取各族终点的最大值（Hc 最长一条：66 / 114 / 210）。
/// M53（族表少 Hc 两档）在这里红。
#[test]
fn family_table_has_fourteen_eighteen_and_twenty_six_cells_and_ends_where_the_longest_family_ends()
{
    for (slots_per_region, cells_per_density, longest_end) in
        [(4u64, 14usize, 66u64), (8, 18, 114), (16, 26, 210)]
    {
        let table = hole_family_anchor_cells_of(slots_per_region);
        assert_eq!(
            table.cells.len() + 1,
            cells_per_density,
            "S = {slots_per_region} 的格数（k = 0 单列，所以 +1）"
        );
        assert_eq!(
            table.without_holes.end, longest_end,
            "S = {slots_per_region} 的 k = 0 终点"
        );
        assert!(table.without_holes.holes.is_empty());
    }
}
