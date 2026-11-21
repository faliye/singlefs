//! 第 7 次重跑（登记 `research/prompts/e156-r7-prereg.md`）：洞族的锚点格与 Python 标签。

use super::*;

// ============================================================================================
// 第 7 次重跑（登记 `research/prompts/e156-r7-prereg.md`）：洞上多扣的槽随洞数长不长、有没有上界（问题单 H1）。
// 族表（W29、W31）、档序列的判定（W28）、每个统计单位的上界数（R7e）、PC-档序列与 PC-同槽。
// ============================================================================================

/// 一个族在一个 S 下的锚点格：族标签、注入的洞数（Hh 的 k、Hc 的 n、Hs 的 m）、位置、锚点模型给的洞 txg 与终点。
pub(super) struct HoleFamilyCell {
    pub(super) family: &'static str,
    pub(super) hole_count: u64,
    pub(super) position: HolePosition,
    pub(super) anchor: anchor_model::AnchorCell,
}

/// 一个 S 下第 7 次重跑的全部格：k = 0 的基准族单列，其余按 Hh 前、Hh 后、Hc、Hs 的次序（与脚本输出同序）。
pub(super) struct HoleFamilyAnchorCellsOfOneSlotsPerRegion {
    pub(super) front_hole_count_inside_the_first_ring: u64,
    front_hole_count_that_fits: u64,
    back_hole_count_that_fits: u64,
    pub(super) without_holes: anchor_model::AnchorCell,
    pub(super) cells: Vec<HoleFamilyCell>,
}

fn hole_family_table_value(table: &[(u64, u64); 3], slots_per_region: u64) -> u64 {
    table
        .iter()
        .find(|(slots, _)| *slots == slots_per_region)
        .map(|(_, count)| *count)
        .expect("登记的 S 都在 A20 的本地常量表里")
}

fn hole_family_cell(
    slots_per_region: u64,
    family: &'static str,
    hole_count: u64,
    position: HolePosition,
    schedule: anchor_model::HoleSchedule,
) -> HoleFamilyCell {
    HoleFamilyCell {
        family,
        hole_count,
        position,
        anchor: anchor_model::run_the_anchor_schedule(slots_per_region, schedule, None),
    }
}

pub(super) fn hole_family_anchor_cells_of(
    slots_per_region: u64,
) -> HoleFamilyAnchorCellsOfOneSlotsPerRegion {
    let front_hole_count_inside_the_first_ring =
        anchor_model::largest_front_hole_count_inside_the_first_ring(slots_per_region);
    let front_hole_count_that_fits =
        anchor_model::largest_hole_count_that_fits(slots_per_region, true);
    let back_hole_count_that_fits =
        anchor_model::largest_hole_count_that_fits(slots_per_region, false);
    assert_eq!(
        (
            front_hole_count_inside_the_first_ring,
            front_hole_count_that_fits,
            back_hole_count_that_fits
        ),
        (
            hole_family_table_value(
                &E156_HOLE_FAMILY_FRONT_HOLE_COUNT_INSIDE_THE_FIRST_RING,
                slots_per_region
            ),
            hole_family_table_value(&E156_HOLE_FAMILY_FRONT_HOLE_COUNT_THAT_FITS, slots_per_region),
            hole_family_table_value(&E156_HOLE_FAMILY_BACK_HOLE_COUNT_THAT_FITS, slots_per_region),
        ),
        "停机 S20：锚点模型算的 k_满、前位置排得下的最大洞数、后位置排得下的最大洞数与登记第七节 A20 的本地常量表不同"
    );
    assert_eq!(
        front_hole_count_that_fits,
        front_hole_count_inside_the_first_ring + 1,
        "停机 S20：前位置排得下的最大洞数应比 k_满 多一档（多的那一档是第一个落进第二圈的洞，W26）"
    );
    let mut cells = Vec::new();
    for hole_count in 1..=front_hole_count_that_fits {
        cells.push(hole_family_cell(
            slots_per_region,
            E156_FAMILY_HOLE_COUNT_SWEEP,
            hole_count,
            HolePosition::Front,
            anchor_model::HoleSchedule::ConsecutiveHoles {
                front: true,
                hole_count,
            },
        ));
    }
    for hole_count in 1..=back_hole_count_that_fits {
        cells.push(hole_family_cell(
            slots_per_region,
            E156_FAMILY_HOLE_COUNT_SWEEP,
            hole_count,
            HolePosition::Back,
            anchor_model::HoleSchedule::ConsecutiveHoles {
                front: false,
                hole_count,
            },
        ));
    }
    for hole_count in [
        2 * front_hole_count_that_fits,
        3 * front_hole_count_that_fits,
    ] {
        cells.push(hole_family_cell(
            slots_per_region,
            E156_FAMILY_CONTINUED_SWEEP,
            hole_count,
            HolePosition::Front,
            anchor_model::HoleSchedule::ConsecutiveHoles {
                front: true,
                hole_count,
            },
        ));
    }
    for position in [HolePosition::Front, HolePosition::Back] {
        for hole_count in [1u64, 2, 3] {
            cells.push(hole_family_cell(
                slots_per_region,
                E156_FAMILY_SAME_RING_SLOT,
                hole_count,
                position,
                anchor_model::HoleSchedule::HolesOnTheSameRingSlot {
                    front: position == HolePosition::Front,
                    hole_count,
                },
            ));
        }
    }
    let end_of_the_longest_family = cells
        .iter()
        .map(|cell| cell.anchor.end)
        .max()
        .expect("至少一个族");
    let without_holes = anchor_model::run_the_anchor_schedule(
        slots_per_region,
        anchor_model::HoleSchedule::ConsecutiveHoles {
            front: false,
            hole_count: 0,
        },
        Some(end_of_the_longest_family),
    );
    HoleFamilyAnchorCellsOfOneSlotsPerRegion {
        front_hole_count_inside_the_first_ring,
        front_hole_count_that_fits,
        back_hole_count_that_fits,
        without_holes,
        cells,
    }
}

/// 锚点脚本 `anchors_e156_r7.py` 里这一族的名字（族标签、位置、洞数前缀 k / n / m）。
fn hole_family_python_label(cell: &HoleFamilyCell) -> String {
    let count_prefix = if cell.family == E156_FAMILY_CONTINUED_SWEEP {
        "n"
    } else if cell.family == E156_FAMILY_SAME_RING_SLOT {
        "m"
    } else {
        "k"
    };
    format!(
        "{}-{}-{count_prefix}{}",
        cell.family,
        cell.position.label(),
        cell.hole_count
    )
}

/// 锚点模型的输出，格式与 `anchors_e156_r7.py`（带与不带 `--dump`）逐字相同，各加一个前缀：`name=anchor_r7_summary` 去掉前缀与脚本输出（58 行）
/// 逐行比，`name=anchor_r7_dump` 去掉前缀与脚本 `--dump` 的 `DUMP` 行（3567 行）逐行比（停机 S20 的报告命令）。k = 0 的基准族脚本没有，
/// 另打 `name=anchor_r7_k0_summary` / `name=anchor_r7_k0_dump`（标签 `Hh-none-k0`），由执行员的包装脚本按同一份 `run` 在同一终点上算一份来比。
/// 摘要行有五个键名与冻结在登记里的脚本不同（已照命名纪律改名，欠账 C430；登记里的脚本不回改）：`k_full_front` → `largest_front_hole_count_that_fits`、
/// `k_full_back` → `largest_back_hole_count_that_fits`、`max_h_que` → `missing_txgs_peak`、`max_h_dong` → `live_holes_peak`、`max_pinned` → `pinned_slots_peak`；
/// 拿登记里的脚本重算来比时，先把脚本输出照这五对换名。`research/results/e156-alloc-basis-counts-2026-10-18-support/` 下的两份脚本已照新名改过。
pub(super) fn emit_hole_family_anchor_cells(
    emitter: &mut Emitter,
    slots_per_region: u64,
    cells: &HoleFamilyAnchorCellsOfOneSlotsPerRegion,
) {
    emitter.emit(&format!(
        "name=anchor_r7_summary S={slots_per_region} largest_front_hole_count_that_fits={} largest_back_hole_count_that_fits={}",
        cells.front_hole_count_that_fits, cells.back_hole_count_that_fits
    ));
    for cell in &cells.cells {
        let label = hole_family_python_label(cell);
        emitter.emit(&format!(
            "name=anchor_r7_summary {}",
            anchor_model::summary_line(slots_per_region, &label, &cell.anchor)
        ));
        for line in anchor_model::dump_lines_of_one_family(slots_per_region, &label, &cell.anchor) {
            emitter.emit(&format!("name=anchor_r7_dump {line}"));
        }
    }
    emitter.emit(&format!(
        "name=anchor_r7_k0_summary {}",
        anchor_model::summary_line(slots_per_region, "Hh-none-k0", &cells.without_holes)
    ));
    for line in
        anchor_model::dump_lines_of_one_family(slots_per_region, "Hh-none-k0", &cells.without_holes)
    {
        emitter.emit(&format!("name=anchor_r7_k0_dump {line}"));
    }
}

#[cfg(test)]
mod tests;
