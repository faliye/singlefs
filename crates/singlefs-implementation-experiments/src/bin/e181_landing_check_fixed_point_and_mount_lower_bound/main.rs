//! E181（落得下那一判与挂载下界）第一段（固定点与落得下段，跑前登记 `research/prompts/e181-preregistration.md` 5.5 节第一段）：
//! 岔路单 `research/prompts/m3-close-admission-and-shadow-ledger-forks.md` 的三行——
//! 固定点上界第二半（F9：上界 P1 与今天的 P2 在中央映射树两层、三层的写上逐次比真写的固定点）、
//! 各盘同槽（F11：先拒那一道算各盘最低落点的扫描代价随单元区怎么长）、
//! 两槽的提交内生块要不要在落得下那一判按槽对数（F10：造得出让没挡的槽够、对齐的空槽对不够的可达盘面吗）。
//! 第二段（截短盘，F14）不在这个二进制里。
//!
//! 入库装置：只读、只驱动、只观测 `singlefs-core` 的公开入口，不改核心层。每个会读 `crates/` 常量的量在这里是本地常量，
//! 值抄自 kb 的登记位，注明出处，[`check_local_constants_against_the_crates`] 逐个回比。
//! 产物按 `E7RESULT` 行打印，复跑登记在 `scripts/experiment-apparatus/replay.sh`。
//!
//! 准入写 `always` 而不写 `inputs-changed`：复跑登记（`replay.sh` 的 `E181` 一行）与长跑脚本（`--full` 重做全部片、出错后原样重起同一片）
//! 都要在输入不变时再跑同一段、拿同一份输入核产物逐字节相同；写 `inputs-changed` 的话，第二次跑会被判「输入没变」拒绝（退出码 78），
//! 复跑与重做都跑不了。强制跑（`--force`）时产物开头多一行 `name=preflight forced=…`，满足时不打，产物与没有这一层时逐字节相同。
//! 退出码：0 正常；3 停机（含「边没找到」，产物里有 `name=stop`）；准入判法退 78 时照 78 退。

// admission: always 复跑登记与长跑脚本都要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::{
    accounting_tree_nodes_after_the_publish_built_on, admit_the_units_landing_on_every_device,
    central_mapping_entries_of_one_lineage_container_copy_on_write,
    central_mapping_entries_of_one_retirement_container_copy_on_write,
    checkpoint_cost_of_one_lineage_container_copy_on_write,
    checkpoint_cost_of_one_retirement_container_copy_on_write,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, space_budget_of_role,
    PlacementRoomOnOneDevice, SpaceBudgetOfARole, UnitsOfAPublishToLand,
};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, unmount};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write, warm_up,
    CodeTwoTreeNodeCapacities, FirstFile, MultiLevelCodeTwoTree, PoolWriter, PublishError,
    TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RECORD_BYTES;
use singlefs_verification::memory_pool::{SparseDevice, SECTOR_BYTES};

mod aligned_pair_shortage_walks;
mod checkpoint_cost_and_fixed_point_account;
mod data_side_fill_and_aligned_pair_shortage;
mod edge_search;
mod fixed_point_upper_bound;
mod forked_device;
mod local_constants_and_anchors;
mod registered_geometries_and_pool;
mod scan_growth_and_lowest_slot;
mod unequal_landing_states;
mod write_accounting_and_refusal_classes;

use aligned_pair_shortage_walks::*;
use checkpoint_cost_and_fixed_point_account::*;
use data_side_fill_and_aligned_pair_shortage::*;
use edge_search::*;
use fixed_point_upper_bound::*;
use forked_device::*;
use local_constants_and_anchors::*;
use registered_geometries_and_pool::*;
use scan_growth_and_lowest_slot::*;
use unequal_landing_states::*;
use write_accounting_and_refusal_classes::*;

/// 这个进程已经打出的 `E7RESULT` 行数（收尾行的 `emitted` 要等于整份产物的行数，复跑脚本的完整性闸核它）。
static EMITTED_LINES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 打一行产物：`E7RESULT <字段>`。
fn emit(fields: &str) {
    EMITTED_LINES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("E7RESULT {fields}");
}

/// 收尾行：`name=done emitted=<行数，含这一行> loadavg1=<1 分钟负载> cpus=<在线 CPU 数>`（复跑脚本计时行的负载关读这两个字段）。
fn emit_the_done_line() {
    let emitted = EMITTED_LINES.load(std::sync::atomic::Ordering::SeqCst) + 1;
    let loadavg_text = std::fs::read_to_string("/proc/loadavg").expect("读得到 /proc/loadavg");
    let loadavg_one_minute = loadavg_text
        .split_whitespace()
        .next()
        .expect("loadavg 第一列");
    let cpuinfo_text = std::fs::read_to_string("/proc/cpuinfo").expect("读得到 /proc/cpuinfo");
    let cpus = cpuinfo_text
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();
    println!("E7RESULT name=done emitted={emitted} loadavg1={loadavg_one_minute} cpus={cpus}");
}

fn text_of_unsigned(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

/// 缩规模的变量只许往小设：没设取登记值；设的值不是十进制整数、或比登记值大，都拒（往大放宽不是登记的那一趟，也不算缩）。
fn narrowed_count_or_registered(
    variable: &str,
    value_of_the_variable: Option<&str>,
    registered: u64,
) -> Result<u64, String> {
    let Some(text) = value_of_the_variable else {
        return Ok(registered);
    };
    let narrowed = text
        .parse::<u64>()
        .map_err(|error| format!("环境变量 {variable} 要是十进制整数：{text:?}：{error}"))?;
    if narrowed > registered {
        return Err(format!(
            "环境变量 {variable} 只许往小设（登记值 {registered}），给的是 {narrowed}"
        ));
    }
    Ok(narrowed)
}

/// 准入判法退 78（拒绝）时装置也退 78；别的非 0 退 1（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `preflight` 交给 `preflight.py check` 的源文件：这个二进制的根文件，准入与运行条件的声明写在它的文件头
/// （`preflight.py` 只读给它的那一份文件头，不读子模块）。
const SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound/main.rs"
);

/// `main` 的第一句：直接起 `python3 preflight.py check <根文件> [--force] -- <参数…>`（不经 `sh -c`）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做；强制跑（判法的第一行以 `forced` 起头）时交回没满足的条件摘要，要写进产物。
fn preflight() -> Option<String> {
    let source = SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS;
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.claude/singlefs-ai-sop/scripts/preflight.py"
    );
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(script).arg("check").arg(source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE)
    });
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    match output.status.code() {
        Some(0) => {}
        Some(PREFLIGHT_REFUSED_EXIT_CODE) => std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE),
        Some(_) | None => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    forced_summary_of_the_preflight_verdict(&verdict_line)
}

/// 准入判法 stdout 第一行（制表符分段）以 `forced` 起头时，交回没满足的条件摘要（最后一段，空白换成下划线），要写进产物；
/// 满足（`met`）或没声明时是 `None`。
fn forced_summary_of_the_preflight_verdict(verdict_line: &str) -> Option<String> {
    if verdict_line.starts_with("forced") {
        let last_segment = verdict_line.rsplit('\t').next().unwrap_or("");
        Some(
            last_segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("_"),
        )
    } else {
        None
    }
}

fn main() {
    let forced_summary = preflight();
    if let Some(summary) = forced_summary {
        emit(&format!("name=preflight forced={summary}"));
    }
    let requested: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let sections: Vec<&str> = if requested.is_empty() {
        vec![
            "constants",
            "anchors",
            "selfcheck",
            "f10-controls-small",
            "f11",
            "f9",
            "f10-measure",
            "f10-controls",
            "f10-narrow",
            "f10-unequal",
            "f10-small",
        ]
    } else {
        requested.iter().map(String::as_str).collect()
    };
    let mut content = ContentBuffer::new();
    for section in sections {
        let started = Instant::now();
        let result = match section {
            "constants" => check_local_constants_against_the_crates(),
            "anchors" => run_anchors(),
            "selfcheck" => run_selfcheck(),
            "f9" => run_fixed_point_upper_bound(&mut content),
            "f11" => run_per_device_lowest_slot(&mut content),
            "f10-measure" => run_aligned_pair_shortage_measure(&mut content),
            "f10-controls" => run_aligned_pair_shortage_controls(GEOMETRY_NARROW, &mut content),
            "f10-controls-small" => {
                run_aligned_pair_shortage_controls(GEOMETRY_SMALL, &mut content)
            }
            "f10-narrow" => {
                run_aligned_pair_shortage_walks(GEOMETRY_NARROW, &mut content).map(drop)
            }
            "f10-unequal" => {
                run_aligned_pair_shortage_walks(GEOMETRY_UNEQUAL, &mut content).map(drop)
            }
            "f10-small" => run_aligned_pair_shortage_walks(GEOMETRY_SMALL, &mut content).map(drop),
            unknown => Err(format!("不认识的段 {unknown}")),
        };
        match result {
            Ok(()) => {
                // 段用时只打到标准错误：它随机器忙闲变，进产物会让登记的逐字节复跑的那几段每次不同。
                eprintln!(
                    "[e181] 段 {section} 用了 {:.1} 秒",
                    started.elapsed().as_secs_f64()
                );
                emit(&format!("name=section_done section={section}"));
            }
            Err(reason) => {
                emit(&format!("name=stop section={section} reason={reason:?}"));
                std::process::exit(3);
            }
        }
    }
    emit_the_done_line();
}

/// 钉绝对值的断言与对拍（登记第七节 A1–A4、B1–B3，加装置自己的判法）。写成普通函数，单测（`tests` 模块）与产物里的 `selfcheck` 段各调一遍：
/// 单测在实验包里跑，`selfcheck` 在装置里直接跑，红在哪一条看产物里的 `name=selfcheck` 行。
mod claims;

/// 每一条断言的名字与函数（`selfcheck` 段按这张表逐条跑）。
const CLAIMS: [(&str, fn()); 28] = [
    ("allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three", claims::check_that_allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three),
    ("allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two", claims::check_that_allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two),
    ("central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples", claims::check_that_central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples),
    ("sparse_layers_share_reads_and_keep_writes_apart", claims::check_that_sparse_layers_share_reads_and_keep_writes_apart),
    ("zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask", claims::check_that_zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask),
    ("the_first_file_lands_at_50180_and_its_records_end_at_50253", claims::check_that_the_first_file_lands_at_50180_and_its_records_end_at_50253),
    ("the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths", claims::check_that_the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths),
    ("the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured", claims::check_that_the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured),
    ("a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role", claims::check_that_a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role),
    ("a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost", claims::check_that_a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost),
    ("extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree", claims::check_that_extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree),
    ("the_judges_flip_when_the_threshold_moves_between_two_readings", claims::check_that_the_judges_flip_when_the_threshold_moves_between_two_readings),
    ("slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere", claims::check_that_slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere),
    ("marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182", claims::check_that_marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182),
    ("aligned_free_pairs_count_the_open_segment_too", claims::check_that_aligned_free_pairs_count_the_open_segment_too),
    ("the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds", claims::check_that_the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds),
    ("refusals_are_named_by_the_gate_that_refused", claims::check_that_refusals_are_named_by_the_gate_that_refused),
    ("the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half", claims::check_that_the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half),
    ("a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit", claims::check_that_a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit),
    ("the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count", claims::check_that_the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count),
    ("the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points", claims::check_that_the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points),
    ("a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly", claims::check_that_a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly),
    ("an_edge_that_no_observation_brackets_is_reported_as_not_found", claims::check_that_an_edge_that_no_observation_brackets_is_reported_as_not_found),
    ("a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm", claims::check_that_a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm),
    ("the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units", claims::check_that_the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units),
    ("a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section", claims::check_that_a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section),
    ("the_preflight_verdict_line_gives_a_summary_only_when_forced", claims::check_that_the_preflight_verdict_line_gives_a_summary_only_when_forced),
    ("local_constants_agree_with_the_crates", claims::check_that_local_constants_agree_with_the_crates),
];

/// 逐条跑 [`CLAIMS`]，每条一行产物；有一条红就返回 `Err`（红在哪一条看产物行）。
fn run_selfcheck() -> Result<(), String> {
    let mut failed = Vec::new();
    for (name, claim) in CLAIMS {
        let outcome = std::panic::catch_unwind(claim);
        emit(&format!(
            "name=selfcheck item={name} passed={}",
            outcome.is_ok()
        ));
        if outcome.is_err() {
            failed.push(name);
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("自证红了：{failed:?}"))
    }
}

#[cfg(test)]
mod tests;
