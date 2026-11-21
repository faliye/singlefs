//! 取样点与每点的输出行。

use super::*;

// ── 取样点（第八节：G0、逐一扫、事件点、等高点、G-small） ──

pub(super) fn sampling_points() -> Vec<(String, Knobs)> {
    let main = Knobs::main_geometry();
    let mut points = vec![("main_geometry".to_string(), main)];
    for tombstone_records in [0, 10, 1000, 10_000, 100_000, 10_000_000, 100_000_000] {
        points.push((format!("tombstone_records_{tombstone_records}"), Knobs { tombstone_records, ..main }));
    }
    for tombstones_added in [0, 8150, 81_500, 815_000] {
        points.push((format!("tombstones_added_{tombstones_added}"), Knobs { tombstones_added, ..main }));
    }
    for settle_rounds in [1, 3, 64] {
        points.push((format!("settle_rounds_{settle_rounds}"), Knobs { settle_rounds, ..main }));
    }
    for retirement_records in [0, 1_000_000] {
        points.push((format!("retirement_records_{retirement_records}"), Knobs { retirement_records, ..main }));
    }
    for retirements_added in [1, 959] {
        points.push((format!("retirements_added_{retirements_added}"), Knobs { retirements_added, ..main }));
    }
    points.push(("lineage_records_0".to_string(), Knobs { lineage_records: 0, ..main }));
    points.push(("lineage_records_added_1".to_string(), Knobs { lineage_records_added: 1, ..main }));
    for live_inodes in [1, 1_000_000] {
        points.push((format!("live_inodes_{live_inodes}"), Knobs { live_inodes, ..main }));
    }
    points.push(("file_data_units_32768".to_string(), Knobs { file_data_units: 32768, ..main }));
    points.push(("mapping_leaves_half_full".to_string(), Knobs { mapping_leaves_half_full: true, ..main }));
    points.push(("touched_leaves_focused".to_string(), Knobs { touched_leaves: TouchedLeaves::Focused, ..main }));
    for persistent_fanout in [16, 32, 135] {
        for persistent_leaf_block in [8, 64, 815] {
            if (persistent_fanout, persistent_leaf_block) != (main.persistent_fanout, main.persistent_leaf_block) {
                let label = format!("persistent_fanout_{persistent_fanout}_leaf_block_{persistent_leaf_block}");
                points.push((label, Knobs { persistent_fanout, persistent_leaf_block, ..main }));
            }
        }
    }
    // 事件点：开新片（815 × 1227 = 1000005，fill = 815、room = 0）与长高（815 × 135 = 110025，追加 1 条 C' = 136）。
    points.push(("open_new_container".to_string(), Knobs { tombstone_records: 1_000_005, ..main }));
    points.push(("tree_grows_a_level".to_string(), Knobs { tombstone_records: 110_025, ..main }));
    // 等高点（集中档）：C = 6401 与 C = 16001。
    for tombstone_records in [5_216_400, 13_040_400] {
        points.push((format!("equal_height_{tombstone_records}"), Knobs { tombstone_records, touched_leaves: TouchedLeaves::Focused, ..main }));
    }
    points.push(("small_geometry".to_string(), Knobs::small_geometry()));
    // 第三份第七节「新补两处逐项」那一点：u = 10⁸、a_T = 0。
    points.push(("tombstone_records_100000000_not_appending".to_string(), Knobs { tombstone_records: 100_000_000, tombstones_added: 0, ..main }));
    points
}

/// 判定那几格在 G0 之外重跑的点（第八节「几何敏感性」）与只报数的两点（第三份：a_T = 0、D = 32768）。
pub(super) const SENSITIVITY_POINTS: [&str; 6] =
    ["small_geometry", "settle_rounds_1", "tombstones_added_815000", "live_inodes_1", "mapping_leaves_half_full", "touched_leaves_focused"];
pub(super) const REPORTED_ONLY_POINTS: [&str; 2] = ["tombstones_added_0", "file_data_units_32768"];

fn levels_text(levels: &[u64]) -> String {
    levels.iter().map(u64::to_string).collect::<Vec<_>>().join("/")
}

/// 一个取样点的全部产物行：几何、各臂四样字节、共有的几样、读路径跳数、常驻峰值、每格比值。
pub(super) fn point_lines(label: &str, geometry: &Geometry) -> Vec<String> {
    let tombstones = &geometry.tombstones;
    let mut lines = vec![format!(
        "name=geometry point={label} tombstone_containers={} tombstone_levels={} tombstone_fill={} open_container_rewritten={} new_containers={} \
         rewritten_index_nodes={} tombstone_containers_after={} tombstone_levels_after={} inode_leaves={} core_rewritten_units={} core_new_units={} \
         slots_per_device={} allocation_levels={} allocation_levels_after={} allocation_changed_records={} allocation_touched_leaves={} \
         allocation_rewritten_nodes={} allocation_carried_nodes={} mapping_leaf_entries={} mapping_entries={} mapping_entries_after={} \
         mapping_levels={} mapping_levels_after={} mapping_rewritten_units={} mapping_new_entries={} mapping_changed_entries={} \
         mapping_touched_leaves={} mapping_rewritten_nodes={} mapping_carried_nodes={} units={} units_after={} rewritten_units={} new_units={} \
         touched_leaves={}",
        tombstones.containers,
        levels_text(&tombstones.levels),
        tombstones.fill,
        tombstones.open_container_rewritten,
        tombstones.new_containers,
        tombstones.rewritten_index_nodes,
        tombstones.containers_after,
        levels_text(&tombstones.levels_after),
        geometry.inode_leaves,
        geometry.core_rewritten_units,
        geometry.core_new_units,
        geometry.slots_per_device,
        levels_text(&geometry.allocation_levels),
        levels_text(&geometry.allocation_levels_after),
        geometry.allocation_changed_records,
        geometry.allocation_touched_leaves,
        geometry.allocation_rewritten_nodes,
        geometry.allocation_carried_nodes,
        geometry.mapping_leaf_entries,
        geometry.mapping_entries,
        geometry.mapping_entries_after,
        levels_text(&geometry.mapping_levels),
        levels_text(&geometry.mapping_levels_after),
        geometry.mapping_rewritten_units,
        geometry.mapping_new_entries,
        geometry.mapping_changed_entries,
        geometry.mapping_touched_leaves,
        geometry.mapping_rewritten_nodes,
        geometry.mapping_carried_nodes,
        geometry.units,
        geometry.units_after,
        geometry.rewritten_units,
        geometry.new_units,
        geometry.knobs.touched_leaves.label(),
    )];
    for arm in Arm::EVERY_ARM {
        let bytes = arm_bytes(geometry, arm);
        lines.push(format!(
            "name=arm_bytes point={label} arm={} copied_before={} copied_after={} recomputed_before={} recomputed_after={}",
            arm.label(),
            bytes.copied_before,
            bytes.copied_after,
            bytes.recomputed_before,
            bytes.recomputed_after
        ));
    }
    lines.extend(shared_lines(label, geometry));
    for cell in judged_cells() {
        let forms: Vec<String> = cell.forms(geometry).iter().map(|&(numerator, denominator)| ratio_text(numerator, denominator)).collect();
        lines.push(format!("name=ratio point={label} cell={} forms={} side_at_two={}", cell.label(), forms.join("/"), cell.verdict(geometry, JUDGED_THRESHOLD)));
    }
    lines
}

/// 一个取样点上四条臂共有的几样，与只给甲报的附带量、甲的分项、第三份补进的四项、读路径跳数、常驻峰值。
fn shared_lines(label: &str, geometry: &Geometry) -> Vec<String> {
    let settle_rounds = geometry.knobs.settle_rounds;
    let (installed_before, installed_after) = installed_bytes(geometry);
    let rounds = jia_after_patch_copied_before_admission(geometry);
    let after = jia_after_patch_copied_after_admission(geometry);
    let added = items_added_by_the_third_registration(geometry);
    let jia_after_patch_before = rounds.total(settle_rounds);
    let (yi2_record_hops, yi2_unit_hops) = yi2_extra_hops(geometry);
    let jia_peak = jia_resident_peak(geometry);
    let bing_peak = bing_resident_peak(geometry);
    let yi1_rounds = yi1_copied_before_admission(geometry, ReferenceSharing::Shared);
    vec![
        // 乙1（附带，只报数、不判；第四份第五节「乙1 的照抄字节」拼法）。
        format!(
            "name=yi1_copied point={label} arm=yi1_reference_counted copied_before={} copied_after={} ordinary_allocation_pass={} settle_round={} outside_the_rounds={} judged=no",
            yi1_rounds.total(settle_rounds),
            yi1_copied_after_admission(geometry, ReferenceSharing::Shared),
            yi1_rounds.ordinary_allocation_pass,
            yi1_rounds.settle_round,
            yi1_rounds.outside_the_rounds,
        ),
        format!(
            "name=shared_bytes point={label} installed_before={installed_before} installed_after={installed_after} \
             installed_unit_bytes_second_copy={} installed_unit_bytes_publish_writes_copy={} version_bytes={} version_bytes_after={} \
             jia_today_minus_jia_after_patch={}",
            installed_unit_bytes(geometry),
            installed_unit_bytes(geometry),
            version_bytes(geometry),
            version_bytes_after(geometry),
            jia_today_copied_before_admission(geometry) - jia_after_patch_before,
        ),
        format!(
            "name=jia_after_patch_parts point={label} ordinary_allocation_pass={} settle_round={} outside_the_rounds={} inode_leaves_again={} \
             mapping_carried_nodes_each_copy={} multi_level_shapes={} item_8640={} item_8965={} item_14640={} item_14643={}",
            rounds.ordinary_allocation_pass,
            rounds.settle_round,
            rounds.outside_the_rounds,
            after.inode_leaves_again,
            after.mapping_carried_nodes_each_copy,
            after.multi_level_shapes,
            added.tombstone_shape_when_not_appending,
            added.lineage_shape_when_not_appending,
            added.carried_file_data_units,
            added.carried_file_data_pointers,
        ),
        format!(
            "name=read_hops point={label} jia_extra=0 bing_extra=0 yi2_record_extra={yi2_record_hops} yi2_unit_extra={yi2_unit_hops} \
             yi1_record_extra=1 yi1_unit_extra=1 ding_extra=layers_since_compaction_0_to_{}",
            DING_MAIN_PERIOD - 1
        ),
        format!(
            "name=resident_peak point={label} jia_after_patch={jia_peak} bing_by_value={bing_peak} bing_over_jia={} bing_side_at_one={}",
            ratio_text(bing_peak, jia_peak),
            resident_peak_side(bing_peak, jia_peak)
        ),
    ]
}

/// 常驻峰值那一行：臂 ÷ 甲 ≥ 1 判「不比甲省」，< 1 判「比甲省」（门槛 1）。
pub(super) fn resident_peak_side(arm_peak: u64, jia_peak: u64) -> &'static str {
    match side_of(arm_peak, jia_peak, (1, 1)) {
        Side::AtOrAboveThreshold => "not_less_than_jia",
        Side::BelowThreshold => "less_than_jia",
    }
}

pub(super) fn geometry_of_point(points: &[(String, Knobs)], label: &str) -> Geometry {
    let knobs = points.iter().find(|(name, _)| name == label).map(|(_, knobs)| *knobs).expect("点名的取样点在取样点表里");
    Geometry::new(knobs)
}
