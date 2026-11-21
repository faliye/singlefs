//! 写次数行、根层级与最优切分、行 3 的取样点与产物行和判定行。

use e7_index_bench::{i64_from_u64, i64_from_usize, u32_from_usize};
use super::*;

/// 参照：一次不变大的可写挂载的取号写（条款本身的写序，D22（单元原子性怎么合成） 已定项 8「取号写两轮：每轮逐盘一写、轮后一道池屏障」）。
const REFERENCE_WRITES: usize = ACQUISITION_WRITE_ROUNDS * DISK_COUNT;
const REFERENCE_BARRIERS: usize = ACQUISITION_WRITE_ROUNDS;

/// 岔路单行 2 的读数：每条臂一次变大的可写挂载写几次系统配置槽、几道池屏障，与参照比。
fn write_count_lines() -> Vec<String> {
    let mut lines = Vec::new();
    for arm in ARMS {
        let (writes, barriers) = count_writes_and_pool_barriers(&write_plan_of(arm));
        lines.push(format!(
            "name=write_count arm={} writes={writes} pool_barriers={barriers} reference_writes={REFERENCE_WRITES} \
             reference_pool_barriers={REFERENCE_BARRIERS} extra_writes={} extra_pool_barriers={} more_than_the_reference={}",
            arm_name(arm),
            i64_from_usize(writes) - i64_from_usize(REFERENCE_WRITES),
            i64_from_usize(barriers) - i64_from_usize(REFERENCE_BARRIERS),
            writes > REFERENCE_WRITES || barriers > REFERENCE_BARRIERS
        ));
    }
    // 行 2 的对照：每盘每轮一写、轮后一道屏障的四轮写序，读数必须是 8 与 4。
    let (control_writes, control_barriers) =
        count_writes_and_pool_barriers(&acquisition_write_plan(false, 4));
    lines.push(format!(
        "name=write_count_control plan=four_rounds writes={control_writes} pool_barriers={control_barriers} expected_writes=8 expected_pool_barriers=4 \
         matches={}",
        control_writes == 8 && control_barriers == 4
    ));
    lines
}

/// 层级 `level` 的一格罩多少整槽：叶罩 812 槽、每上一层乘 169（`allocation_record_tree.rs:289`–`:293` 的 `span_in_slots_at_level`）。
pub(super) fn span_in_slots(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 最小的根层级 R ≥ 1，使各盘 ⌈盘上整槽数 ÷ 层级 R−1 的一格罩的槽数⌉ 之和不超过根的扇出
/// （`allocation_record_tree.rs:96`–`:120` 的 `AllocationRecordTreeGeometry::of_devices`；这里按定义另写，不抄它）。
pub(super) fn root_level_of(slots_of_each_disk: &[u64]) -> u32 {
    let mut level = 1;
    loop {
        let cells_needed: u64 = slots_of_each_disk
            .iter()
            .map(|slots| slots.div_ceil(span_in_slots(level - 1)))
            .sum();
        if cells_needed <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return level;
        }
        level += 1;
    }
}

/// 两块等大的盘时根层级 `level` 的上界（每盘整槽数）：两盘各 ⌊扇出 ÷ 2⌋ 格。
pub(super) fn equal_disks_boundary_slots(level: u32) -> u64 {
    (ALLOCATION_RECORD_TREE_INTERNAL_FANOUT / 2) * span_in_slots(level - 1)
}

fn is_better_split(candidate: Table, current: Table) -> bool {
    let candidate_total = candidate[0] + candidate[1];
    let current_total = current[0] + current[1];
    candidate_total > current_total
        || (candidate_total == current_total
            && candidate[0].abs_diff(candidate[1]) < current[0].abs_diff(current[1]))
}

/// 乙「记到边界为止」在两块盘上的摊法（登记第一节末列的第 2 条）：每盘记录值不低于旧记录、不高于探测值、根层级不变
/// （各盘格数之和不超过扇出）的前提下取整槽总数最大；总数相同取两盘之差最小；再相同取先遇到的（盘 0 格数少的那一组）。
pub(super) fn best_total_split(old_records: Table, new_probe: Table) -> Table {
    let span = span_in_slots(root_level_of(&old_records) - 1);
    let mut best: Option<Table> = None;
    for cells_of_disk_zero in old_records[0].div_ceil(span)..=ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    {
        let cells_of_disk_one = ALLOCATION_RECORD_TREE_INTERNAL_FANOUT - cells_of_disk_zero;
        if cells_of_disk_one < old_records[1].div_ceil(span) {
            break;
        }
        let candidate = [
            new_probe[0].min(cells_of_disk_zero * span),
            new_probe[1].min(cells_of_disk_one * span),
        ];
        if candidate[0] < old_records[0] || candidate[1] < old_records[1] {
            continue;
        }
        best = match best {
            Some(current) if !is_better_split(candidate, current) => Some(current),
            _ => Some(candidate),
        };
    }
    best.expect(
        "when the probe is not below the old records, the old records themselves are a candidate",
    )
}

/// 乙按盘号先后贪心的变体（登记第一节末列的第 2 条附带并列报）：盘 0 先取满（扣下盘 1 旧记录要的格数），盘 1 保旧记录。
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "只在单测里钉登记第七节乙的逐盘按盘号贪心变体与它的变异"
    )
)]
pub(super) fn greedy_split(old_records: Table, new_probe: Table) -> Table {
    let span = span_in_slots(root_level_of(&old_records) - 1);
    let cells_of_disk_zero = ALLOCATION_RECORD_TREE_INTERNAL_FANOUT - old_records[1].div_ceil(span);
    [new_probe[0].min(cells_of_disk_zero * span), old_records[1]]
}

/// 一个行 3 取样点：旧记录与新探测（每盘整槽数）下两种做法变大之后用得上的整槽数。
pub(super) struct RootLevelPoint {
    old_records: Table,
    new_probe: Table,
    pub(super) old_level: u32,
    pub(super) new_level: u32,
    /// 甲「一点不记」：根层级变了就保持旧记录，没变用新探测。
    pub(super) chain_total: u64,
    /// 乙「记到边界为止」：根层级变了就记总数最大的一组（不超过边界），没变用新探测。
    pub(super) highest_split: Table,
}

pub(super) fn root_level_point(old_records: Table, new_probe: Table) -> RootLevelPoint {
    let old_level = root_level_of(&old_records);
    let new_level = root_level_of(&new_probe);
    let level_changes = old_level != new_level;
    RootLevelPoint {
        old_records,
        new_probe,
        old_level,
        new_level,
        chain_total: if level_changes {
            old_records[0] + old_records[1]
        } else {
            new_probe[0] + new_probe[1]
        },
        highest_split: if level_changes {
            best_total_split(old_records, new_probe)
        } else {
            new_probe
        },
    }
}

/// 单元区整槽数：表项（绝对整槽数）减单元区起点。
pub(super) fn unit_area_slots(absolute_slots: u64) -> u64 {
    absolute_slots - DISK_COUNT as u64 * UNIT_AREA_START_SLOT
}
/// 行 3 取样点：边界之下、跨一道、跨两道（登记第六节），加行 3 的合成对照点（答案手算得出）与变异要的两个边界加一点。
/// 旧尺寸每盘 60000 槽，根层级 1，单元区起点之外有 9824 槽。
pub(super) const ROOT_LEVEL_POINTS: [(&str, Table, Table); 6] = [
    ("below_boundary", [60000, 60000], [68208, 68208]),
    ("crosses_one_boundary", [60000, 60000], [262144, 262144]),
    (
        "crosses_two_boundaries",
        [60000, 60000],
        [67108864, 67108864],
    ),
    (
        "synthetic_control_crossing",
        [68208, 68208],
        [262144, 262144],
    ),
    (
        "one_over_the_boundary_on_one_disk",
        [60000, 60000],
        [68209, 68208],
    ),
    (
        "one_over_the_boundary_on_both_disks",
        [60000, 60000],
        [68209, 68209],
    ),
];

/// 登记第七节第二张表里这六个点的（甲总数，乙总数）：表里的数由独立的算术脚本算出（甲 = 层级变了取旧记录总数）。
const DRAFT_TOTALS_OF_ROOT_LEVEL_POINTS: [(u64, u64); 6] = [
    (136416, 136416),
    (120000, 137228),
    (120000, 137228),
    (136416, 137228),
    (136417, 136417),
    (120000, 136417),
];

/// 登记第三节第 8 条：`allocation_record_tree.rs` 的单元测试断言的根层级（每盘整槽数，两块等大盘 → 根层级，测试名）。
pub(super) const CRATES_ROOT_LEVEL_READINGS: [(u64, u32, &str); 4] = [
    (
        262144,
        2,
        "two_four_gibibyte_devices_give_a_tree_of_height_three",
    ),
    (
        67108864,
        3,
        "two_one_tebibyte_devices_give_a_tree_of_height_four",
    ),
    (
        68208,
        1,
        "half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary",
    ),
    (
        68209,
        2,
        "half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary",
    ),
];

const DRAFT_LEVEL_SPANS: [u64; 4] = [812, 137228, 23191532, 3919368908];
const DRAFT_EQUAL_DISK_BOUNDARIES: [u64; 3] = [68208, 11527152, 1948088688];

fn root_level_point_line(label: &str, point: &RootLevelPoint) -> String {
    let old_total = point.old_records[0] + point.old_records[1];
    let new_total = point.new_probe[0] + point.new_probe[1];
    let highest_total = point.highest_split[0] + point.highest_split[1];
    let level_changes = point.old_level != point.new_level;
    format!(
        "name=root_level_point label={label} old_records={} new_probe={} old_root_level={} new_root_level={} root_level_changes={level_changes} \
         chain_total_slots={} highest_total_slots={highest_total} highest_split={} highest_minus_chain_slots={} \
         chain_total_unit_area_slots={} highest_total_unit_area_slots={} highest_minus_chain_unit_area_slots={} \
         growth_available_slots={} highest_keeps_fraction_of_the_growth={:.4} diverges={}",
        describe_table(point.old_records),
        describe_table(point.new_probe),
        point.old_level,
        point.new_level,
        point.chain_total,
        describe_table(point.highest_split),
        i64_from_u64(highest_total) - i64_from_u64(point.chain_total),
        unit_area_slots(point.chain_total),
        unit_area_slots(highest_total),
        i64_from_u64(highest_total) - i64_from_u64(point.chain_total),
        new_total - old_total,
        (i64_from_u64(highest_total) - i64_from_u64(point.chain_total)) as f64 / (new_total - old_total) as f64,
        highest_total > point.chain_total
    )
}

/// 岔路单行 2 与行 3 的读数（命令行参数 `write-counts-and-root-level`）。
pub(super) fn write_counts_and_root_level_lines(threads: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config segment=write-counts-and-root-level configured_threads={threads} root_level_point_phase_threads={} root_level_points={}",
        threads_used_for(ROOT_LEVEL_POINTS.len(), threads),
        ROOT_LEVEL_POINTS.len()
    )];
    lines.extend(write_count_lines());
    for (level, span) in DRAFT_LEVEL_SPANS.iter().enumerate() {
        lines.push(format!(
            "name=level_span level={level} slots={} draft_slots={span}",
            span_in_slots(u32_from_usize(level))
        ));
    }
    for (boundary_index, draft_boundary) in DRAFT_EQUAL_DISK_BOUNDARIES.iter().enumerate() {
        let level = u32_from_usize(boundary_index) + 1;
        lines.push(format!(
            "name=level_boundary root_level={level} equal_disks_up_to_slots={} draft_slots={draft_boundary} bytes_per_disk={}",
            equal_disks_boundary_slots(level),
            equal_disks_boundary_slots(level) * SLOT_BYTES
        ));
    }
    for (slots, crates_level, crates_test) in CRATES_ROOT_LEVEL_READINGS {
        let level = root_level_of(&[slots, slots]);
        lines.push(format!(
            "name=crates_cross_check per_disk_slots={slots} root_level={level} crates_root_level={crates_level} crates_test={crates_test} matches={}",
            level == crates_level
        ));
    }
    let points = parallel_map(&ROOT_LEVEL_POINTS, threads, |point| {
        root_level_point(point.1, point.2)
    });
    for (sampling_point, point) in ROOT_LEVEL_POINTS.iter().zip(points.iter()) {
        lines.push(root_level_point_line(sampling_point.0, point));
    }
    lines.push(
        "name=not_run_after_sufficient item=publication_rotation_write_count_after_each_publish"
            .to_string(),
    );
    lines.push("name=not_run_after_sufficient item=abandon_branch_write_count_of_lowest_to_read_highest_to_probe".to_string());
    lines.push("name=not_run_after_sufficient item=highest_greedy_per_disk_variant_and_the_one_disk_grows_across_one_boundary_point".to_string());
    lines.extend(write_counts_and_root_level_verdict_lines(&points));
    lines
}

fn write_counts_and_root_level_verdict_lines(points: &[RootLevelPoint]) -> Vec<String> {
    let arms_that_write_more: Vec<&str> = ARMS
        .iter()
        .filter(|arm| {
            let (writes, barriers) = count_writes_and_pool_barriers(&write_plan_of(**arm));
            writes > REFERENCE_WRITES || barriers > REFERENCE_BARRIERS
        })
        .map(|arm| arm_name(*arm))
        .collect();
    let crates_reading_mismatches = CRATES_ROOT_LEVEL_READINGS
        .iter()
        .filter(|(slots, crates_level, _)| root_level_of(&[*slots, *slots]) != *crates_level)
        .count();
    let span_mismatches = (0..DRAFT_LEVEL_SPANS.len())
        .filter(|level| span_in_slots(u32_from_usize(*level)) != DRAFT_LEVEL_SPANS[*level])
        .count();
    let boundary_mismatches = (0..DRAFT_EQUAL_DISK_BOUNDARIES.len())
        .filter(|index| {
            equal_disks_boundary_slots(u32_from_usize(*index) + 1) != DRAFT_EQUAL_DISK_BOUNDARIES[*index]
        })
        .count();
    let total_mismatches = points
        .iter()
        .zip(DRAFT_TOTALS_OF_ROOT_LEVEL_POINTS.iter())
        .filter(|(point, draft)| {
            point.chain_total != draft.0
                || point.highest_split[0] + point.highest_split[1] != draft.1
        })
        .count();
    let synthetic_control = &points[3];
    let synthetic_control_total =
        synthetic_control.highest_split[0] + synthetic_control.highest_split[1];
    let synthetic_control_matches_hand_values = synthetic_control.chain_total == 136416
        && synthetic_control_total == 137228
        && synthetic_control_total - synthetic_control.chain_total == 812;
    let non_crossing_points_have_zero_difference = points
        .iter()
        .filter(|point| point.old_level == point.new_level)
        .all(|point| point.highest_split[0] + point.highest_split[1] == point.chain_total);
    let (control_writes, control_barriers) =
        count_writes_and_pool_barriers(&acquisition_write_plan(false, 4));
    vec![
        format!(
            "name=verdict check=write_counts reference_writes={REFERENCE_WRITES} reference_pool_barriers={REFERENCE_BARRIERS} \
             four_round_control_matches={} arms_that_write_more_than_the_reference={}",
            control_writes == 8 && control_barriers == 4,
            if arms_that_write_more.is_empty() { "none".to_string() } else { arms_that_write_more.join(",") }
        ),
        format!("name=verdict check=root_level_cross_check crates_reading_mismatches={}", zero_or_count(crates_reading_mismatches)),
        format!(
            "name=verdict check=independent_arithmetic draft_mismatches={}",
            zero_or_count(span_mismatches + boundary_mismatches + total_mismatches)
        ),
        format!(
            "name=verdict check=root_level_points synthetic_control_matches_hand_values={synthetic_control_matches_hand_values} \
             non_crossing_points_have_zero_difference={non_crossing_points_have_zero_difference}"
        ),
    ]
}
