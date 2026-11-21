//! 常量与计数两份报告，以及站点读数与式子的对账：关系、对账行、调用处读到的表长、逐格对账与对账报告。

use super::*;

fn constants_report(product: &Product, out: &mut String) {
    let local: [(&str, u64); 9] = [
        ("slot_bytes", SLOT_BYTES),
        ("segment_slots", SEGMENT_SLOTS),
        ("unit_area_start_slot", UNIT_AREA_START_SLOT),
        ("disks", DISKS),
        ("inode_leaf_records", INODE_LEAF_RECORDS),
        ("inode_leaves_at_most", INODE_LEAVES_AT_MOST),
        ("tombstone_container_records", TOMBSTONE_CONTAINER_RECORDS),
        ("settling_rounds_at_most", SETTLING_ROUNDS_AT_MOST),
        ("overwrite_bytes", OVERWRITE_BYTES),
    ];
    for (key, value) in local {
        let measured = number(&product.constants, key);
        let _ = writeln!(
            out,
            "name=constant_check key={key} model={value} implementation={} match={}",
            measured.map_or("NOT_READ".to_string(), |maintenance| maintenance
                .to_string()),
            measured == Some(value)
        );
    }
}

/// Q1、Q2 与第七节 ③1：计数产物里每个点每个状态每种发布的扫类步数，每根轴的 α。
pub(super) fn counts_report(product: &Product, out: &mut String) {
    constants_report(product, out);
    for state in STATES {
        for kind in KINDS {
            for axis in AXES {
                let quantity = |point: &str| {
                    scan_steps(product, point, state, kind, &SEGMENTS_BEFORE_ADMISSION)
                };
                let reading = read_axis(product, axis, &quantity);
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q2 axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                }
                for (row, _) in all_rows() {
                    let by_row = |point: &str| {
                        product.steps_of(
                            &round_key(point, state, kind, 1),
                            &SEGMENTS_BEFORE_ADMISSION,
                            row,
                        )
                    };
                    let by_row_reading = read_axis(product, axis, &by_row);
                    if matches!(by_row_reading, AxisReading::Alpha(..)) {
                        let _ = writeln!(
                            out,
                            "name=q1 axis={axis} state={state} kind={kind} row={row} {}",
                            reading_text(&by_row_reading)
                        );
                    }
                }
            }
        }
    }
    // 第七节 ③1：整份拷贝每份的元素数里只由 U 定的那一部分，与站点读到的整份元素数比（站点读数 ≥ U 那一部分，多出的是记录等小表）。
    let mut disagreements = 0_u64;
    for ((point, state, kind, round, segment, row), cell) in &product.rows {
        let copy_row = row == "whole_allocator_copy_before_the_judgement"
            || row == "whole_allocator_copy_in_a_settling_round";
        let (true, Some(axes)) = (copy_row && cell.calls > 0, product.points.get(point)) else {
            continue;
        };
        let per_copy = cell.scale / cell.calls;
        let unit_area_part = allocator_copy_elements_from_the_unit_area(axes.unit_area_slots);
        let _ = writeln!(
            out,
            "name=copy_elements point={point} state={state} kind={kind} round={round} segment={segment} row={row} calls={} elements_per_copy={per_copy} from_the_unit_area={unit_area_part} other_tables={}",
            cell.calls,
            per_copy.saturating_sub(unit_area_part)
        );
        if per_copy < unit_area_part || cell.scale % cell.calls != 0 {
            disagreements += 1;
        }
    }
    let _ = writeln!(
        out,
        "name=verdict quantity=three_one_copy_elements disagreements={disagreements}"
    );
    let unlisted: u64 = product
        .rows
        .iter()
        .filter(|(key, _)| {
            key.5 == UNLISTED_ROW && SEGMENTS_BEFORE_ADMISSION.contains(&key.4.as_str())
        })
        .map(|(_, cell)| cell.steps)
        .sum();
    let _ = writeln!(out, "name=unlisted_steps total_over_the_product={unlisted}");
}

// ---- 第七节 ③2：B1 式子与量的逐格对账 ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relation {
    /// 站点读到的步数恰好等于式子值（式子吃入口、出口读数与调用次数）。
    Equals,
    /// 修订 4，整行是全表扫：步数逐格等于各次调用那一刻读的表长之和；走到命中为止那一部分没有，它的步与表长都该是 0。
    WholeTableAtTheCallSite,
    /// 修订 4，整行走到命中为止：步数全记在那一部分、不超过各次调用那一刻读的表长之和；全表扫那一部分没有，表长该是 0。
    UpToTheHitAtTheCallSite,
    /// 修订 4，两部分都有：全表扫那一部分（步数减走到命中为止的步）逐格等于它的表长之和，走到命中为止那一部分不超过它的表长之和。
    WholeTableAndUpToTheHitAtTheCallSite,
    /// 这一行没有表长可比（s37 只记步，没有 `scale`）：只核「调过就有步、没调过就没有步」，逐格对。
    EveryCallTakesAStep,
}

impl Relation {
    /// 有走到命中为止、只核上界的那一部分（汇总行按这个分两类数行）。
    fn has_a_bounded_part(self) -> bool {
        matches!(
            self,
            Relation::UpToTheHitAtTheCallSite | Relation::WholeTableAndUpToTheHitAtTheCallSite
        )
    }
}

struct ReconciledRow {
    row: &'static str,
    relation: Relation,
    /// 对账的发布种类：入口、出口与调用处的读数都是计数构建在这一次发布里读的，三种发布（含最后一批）都对。
    kinds: &'static [&'static str],
    segments: &'static [&'static str],
}

/// 三种被量的发布都对账（入口、出口读数是计数构建在这一次发布里读的，最后一批也有自己的那一版）。
const EVERY_KIND: &[&str] = &["overwrite", "unlink_intent", "unlink_final"];

/// 记账行：池级三行（`transaction.rs:7941` `PoolWideAccountingRow::ALL`），每块盘六行（`transaction.rs:7981` `PerDeviceAccountingRow::ALL`）。
const POOL_WIDE_ACCOUNTING_ROWS: u64 = 3;
const PER_DEVICE_ACCOUNTING_ROWS: u64 = 6;

/// 第三节清单里扫类的 27 行（两行整份拷走第七节 ③1，不在这里；第 27 行 s37 只核步）全对账，三种发布、准入之前三段：
/// 16 行吃计数构建在这一次发布进判之前读上一版（`entry_` 起头的差量）、发完之后读这一次那一版（`exit_` 起头的差量）的表长，
/// 与发布行的固定点轮数、这一行的调用次数，逐格相等（登记第十二节修订 3 ②；s5、s7、s14 是修订 4 换吃入口读数的）；
/// 修订 4 那十行吃插计数脚本在每次调用那一刻读的表长，全表扫那一部分逐格相等、走到命中为止那一部分不超过上界（拆开记的，没有整行走上界的）。
const RECONCILED_ROWS: [ReconciledRow; 27] = [
    reconciled_on_every_kind("files_other_than_the_first_plan"),
    reconciled_on_every_kind("inodes_with_an_extent_entry_set"),
    reconciled_on_every_kind("cluster_segment_sum"),
    read_at_the_call_site(
        "file_other_than_the_first_linear_find",
        Relation::UpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site(
        "extent_upper_segment_plan",
        Relation::WholeTableAndUpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site("root_header_linear_find", Relation::UpToTheHitAtTheCallSite),
    read_at_the_call_site(
        "allocation_record_tree_plan",
        Relation::WholeTableAtTheCallSite,
    ),
    read_at_the_call_site(
        "allocator_release_linear_find",
        Relation::UpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site(
        "user_data_slot_linear_scan",
        Relation::UpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site(
        "record_intersection_filter",
        Relation::WholeTableAndUpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site(
        "empty_segment_linear_scan",
        Relation::UpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site(
        "fallback_slot_linear_scan",
        Relation::UpToTheHitAtTheCallSite,
    ),
    read_at_the_call_site("records_of_each_leaf", Relation::WholeTableAtTheCallSite),
    reconciled_on_every_kind("inode_record_linear_find"),
    reconciled_on_every_kind("extent_roles_rewritten_scan"),
    reconciled_on_every_kind("checkpoint_cost_accounting_tree_plan"),
    reconciled_on_every_kind("allocation_record_tree_node_set"),
    reconciled_on_every_kind("inode_number_watermark_steps"),
    reconciled_on_every_kind("deadlist_changes_indexes"),
    reconciled_on_every_kind("multi_level_tree_planning"),
    reconciled_on_every_kind("mapped_units_of_container_trees"),
    reconciled_on_every_kind("moved_entries_path_tail_filter"),
    reconciled_on_every_kind("release_judgement_version_indexes"),
    reconciled_on_every_kind("accounting_entries_of_this_publish"),
    reconciled_on_every_kind("instance_table_chain"),
    reconciled_on_every_kind("mapped_units_whole_table"),
    ReconciledRow {
        row: ROW_WITHOUT_A_BY_DELTA_FORMULA,
        relation: Relation::EveryCallTakesAStep,
        kinds: EVERY_KIND,
        segments: &["caller", "before_landing", "between"],
    },
];

/// 逐格相等、三种发布、准入之前三段：吃入口、出口读数的 16 行一律这样对。
const fn reconciled_on_every_kind(row: &'static str) -> ReconciledRow {
    ReconciledRow {
        row,
        relation: Relation::Equals,
        kinds: EVERY_KIND,
        segments: &["caller", "before_landing", "between"],
    }
}

/// 修订 4 那十行：吃调用处读的表长，三种发布、准入之前三段。
const fn read_at_the_call_site(row: &'static str, relation: Relation) -> ReconciledRow {
    ReconciledRow {
        row,
        relation,
        kinds: EVERY_KIND,
        segments: &["caller", "before_landing", "between"],
    }
}

/// 修订 4 那十行一格的读数（准入之前三段合起来）。
struct CallSiteCell {
    pub(super) steps: u64,
    pub(super) steps_to_the_hit: u64,
    /// 全表扫那一部分各次调用处表长之和（行读数的 `scale`）。
    whole_table_lengths: u64,
    /// 走到命中为止那一部分各次调用处表长之和（行读数的 `scale_to_the_hit`）。
    lengths_to_the_hit: u64,
}

/// 一格的调用处读数；有一条行读数没带走到命中为止那两样时读不到（不拿 0 补）。
fn call_site_cell(
    product: &Product,
    at: &RoundKey,
    segments: &[&str],
    row: &str,
) -> Option<CallSiteCell> {
    let mut cell = CallSiteCell {
        steps: 0,
        steps_to_the_hit: 0,
        whole_table_lengths: 0,
        lengths_to_the_hit: 0,
    };
    for segment in segments {
        let read = product.row_cell(&at.0, &at.1, &at.2, at.3, segment, row);
        if read.lines_without_the_part_to_the_hit > 0 {
            return None;
        }
        cell.steps += read.steps;
        cell.steps_to_the_hit += read.steps_to_the_hit;
        cell.whole_table_lengths += read.scale;
        cell.lengths_to_the_hit += read.scale_to_the_hit;
    }
    Some(cell)
}

/// 修订 4 的判法：(这一格对不对, 走到命中为止那一部分超没超上界)。超上界也算不对（停机条款第 2 条的不符）。
fn call_site_cell_agrees(relation: Relation, cell: &CallSiteCell) -> (bool, bool) {
    let Some(whole_table_steps) = cell.steps.checked_sub(cell.steps_to_the_hit) else {
        return (false, false);
    };
    let over_the_bound = cell.steps_to_the_hit > cell.lengths_to_the_hit;
    let absent_part_reads_nothing = match relation {
        Relation::WholeTableAtTheCallSite => cell.lengths_to_the_hit == 0,
        Relation::UpToTheHitAtTheCallSite => cell.whole_table_lengths == 0,
        Relation::Equals
        | Relation::WholeTableAndUpToTheHitAtTheCallSite
        | Relation::EveryCallTakesAStep => true,
    };
    let agrees = whole_table_steps == cell.whole_table_lengths
        && !over_the_bound
        && absent_part_reads_nothing;
    (agrees, over_the_bound)
}

/// 计数构建在被量的发布进判之前读的上一版（`entry_`）或发完之后读的这一版（`exit_`）：一次发布恰好一条，多一条少一条都读不到。
fn reading_of_the_publish(product: &Product, at: &RoundKey, key: &str) -> Option<u64> {
    let segment = if key.starts_with("entry_") {
        "before_landing"
    } else {
        "after"
    };
    match product.deltas_of(at, key, &[segment]).as_slice() {
        [(0, value)] => Some(*value),
        _ => None,
    }
}

/// 这一次发布的固定点走了几轮（发布行）。
fn fixed_point_rounds_of(product: &Product, at: &RoundKey) -> Option<u64> {
    number(product.publishes.get(at)?, "fixed_point_rounds")
}

/// 固定点每一轮的分配记录树节点数都等于发出来的那一版的：这时每一轮的映射单元数、中央映射树的 key 与节点，就是出口读到的那一份
/// （它们只经分配记录树的节点随轮变；节点数一轮一条，取自 s16 那一处的入参，不是 s20、s36 那一步的结果）。不满足的那一格读不到。
fn allocation_nodes_stay_put_in_every_round(
    product: &Product,
    at: &RoundKey,
    rounds: u64,
) -> Option<()> {
    let after = reading_of_the_publish(product, at, "exit_allocation_record_tree_nodes")?;
    let per_round = product.deltas_of(at, "allocation_plan_nodes_after", &["between"]);
    let every_round_once = u64::try_from(per_round.len()).ok()? == rounds
        && (1..=rounds).all(|round| {
            per_round
                .iter()
                .filter(|(fixed_point_round, _)| *fixed_point_round == round)
                .count()
                == 1
        });
    (every_round_once && per_round.iter().all(|(_, nodes)| *nodes == after)).then_some(())
}

/// 码 2 规划一次记的步：上一版的 key、这一版的 key、这一版的节点与它们的出处各一遍、上一版的节点（`code_two_tree.rs:637` 规划函数末尾那一处）。
fn code_two_plan_steps(
    previous_keys: u64,
    keys_after: u64,
    nodes_after: u64,
    previous_nodes: u64,
) -> u64 {
    previous_keys + keys_after + 2 * nodes_after + previous_nodes
}

/// 修订 3 ② 补上的 13 行的式子（读码写的，出处是主工作区 `crates/singlefs-core/src/` 的 文件:行，登记第三节「一步是什么」与插计数脚本那一处的记法）。
fn expected_steps_of_the_rows_read_at_the_entry_and_the_exit(
    row: &str,
    calls: u64,
    state: &Fields,
    product: &Product,
    at: &RoundKey,
) -> Option<u64> {
    let entry = |key: &str| reading_of_the_publish(product, at, &format!("entry_{key}"));
    let exit = |key: &str| reading_of_the_publish(product, at, &format!("exit_{key}"));
    match row {
        // s5 `transaction.rs:7686`：规划别的文件时上一版的别的文件各看一遍（修订 4 换吃入口读数）
        "files_other_than_the_first_plan" => Some(calls * entry("files_other_than_the_first")?),
        // s7 `transaction.rs:4116`：第一个文件一步，加上一版的别的文件各一步（修订 4 换吃入口读数）
        "inodes_with_an_extent_entry_set" => {
            Some(calls * (1 + entry("files_other_than_the_first")?))
        }
        // s14 `allocator.rs:574`：每块盘调一次、每次走完这一刻的聚簇段（修订 4 收严成相等，吃入口聚簇段数）
        "cluster_segment_sum" => Some(calls * entry("cluster_segments")?),
        // s4 `transaction.rs:4202`：find 走到改 size 的那条记录为止；记录按号升序，走过的条数 = 号更小的条数 + 1（入口按号数，不走 find）
        "inode_record_linear_find" => {
            Some(calls * entry("position_of_the_inode_whose_size_changes")?)
        }
        // s11 `transaction.rs:7873`：这一次规划出的 extent 树下段、上段与别的文件的下段节点各看一遍；规划出的就是发出来的那一版
        "extent_roles_rewritten_scan" => Some(calls * exit("extent_tree_nodes")?),
        // s13 `admission.rs:1325`：一次 ckpt 记两次调用（入口一次、码 2 规划一次）；这一版的记账行、接在它后面那次的 key（与这一次发布同一组 key：
        // 池级三行加每盘六行、照抄的台阶行，只差代段）、码 2 规划（上一版就是这一版的记账树，规划出的节点数与这一次发出来的记账树相同）
        "checkpoint_cost_accounting_tree_plan" => {
            if !calls.is_multiple_of(2) {
                return None;
            }
            let keys_after = exit("accounting_tree_keys")?;
            let per_judgement = entry("accounting_rows")?
                + keys_after
                + code_two_plan_steps(
                    entry("accounting_tree_keys")?,
                    keys_after,
                    exit("accounting_tree_nodes")?,
                    entry("accounting_tree_nodes")?,
                );
            Some(calls / 2 * per_judgement)
        }
        // s15 `transaction.rs:13312` 起：上一版分配记录树的节点收成集合，又拷一份当这一版的起点，每个节点两步
        "allocation_record_tree_node_set" => {
            Some(calls * 2 * entry("allocation_record_tree_nodes")?)
        }
        // s18 `transaction.rs:4269`：过滤与找这一版那一行各记一遍记账行；台阶行（水位、代段小于这一版的 txg）按记账行的次序收进来就是代段升序，
        // 稳定排序在已排好的输入上恰好比 n − 1 次（标准库插入排序与找已有的段都是相邻两两比一遍）
        "inode_number_watermark_steps" => Some(
            calls
                * (2 * entry("accounting_rows")?
                    + entry("watermark_rows_before_its_txg")?.saturating_sub(1)),
        ),
        // s19 `transaction.rs:6566`：每轮一次；上一版的 deadlist 条目与留存的映射条目各收一遍、按角色建上一版映射单元的索引；
        // 换下的角色里有第一个文件之外一个文件的单元时再建一份按号的文件索引（多出的那一次调用）
        "deadlist_changes_indexes" => {
            let rounds = fixed_point_rounds_of(product, at)?;
            let every_round = entry("deadlist_entries")?
                + entry("kept_mapping_entries")?
                + entry("mapped_units")?;
            match calls.checked_sub(2 * rounds)? {
                0 => Some(rounds * every_round),
                extra if extra == rounds => {
                    Some(rounds * (every_round + entry("files_other_than_the_first")?))
                }
                _ => None,
            }
        }
        // s20 `transaction.rs:9002`：每轮三棵码 2 树各收 key 一遍、规划一次；deadlist 树一把 key 都没有时走零根那一支，只走上一版的节点
        "multi_level_tree_planning" => {
            let rounds = fixed_point_rounds_of(product, at)?;
            allocation_nodes_stay_put_in_every_round(product, at, rounds)?;
            let accounting_keys = exit("accounting_tree_keys")?;
            let accounting = accounting_keys
                + code_two_plan_steps(
                    entry("accounting_tree_keys")?,
                    accounting_keys,
                    exit("accounting_tree_nodes")?,
                    entry("accounting_tree_nodes")?,
                );
            let deadlist_keys = exit("deadlist_tree_keys")?;
            let deadlist = deadlist_keys
                + if deadlist_keys == 0 {
                    entry("deadlist_tree_nodes")?
                } else {
                    code_two_plan_steps(
                        entry("deadlist_tree_keys")?,
                        deadlist_keys,
                        exit("deadlist_tree_nodes")?,
                        entry("deadlist_tree_nodes")?,
                    )
                };
            let mapping_keys = exit("central_mapping_tree_keys")?;
            let mapping = mapping_keys
                + code_two_plan_steps(
                    entry("central_mapping_tree_keys")?,
                    mapping_keys,
                    exit("central_mapping_tree_nodes")?,
                    entry("central_mapping_tree_nodes")?,
                );
            Some(rounds * (accounting + deadlist + mapping))
        }
        // s21 `transaction.rs:6938`：每轮三棵容器树各一次，每片容器、每个码 2 节点出一条
        "mapped_units_of_container_trees" => {
            Some(fixed_point_rounds_of(product, at)? * exit("container_tree_units")?)
        }
        // s22 `transaction.rs:9469`：每轮一次，尾部过滤看上一版中央映射树的每个节点
        "moved_entries_path_tail_filter" => Some(calls * entry("central_mapping_tree_nodes")?),
        // s23 `transaction.rs:4628`：每轮一次；留存给快照的角色建一份索引（这一次新进 deadlist 的条目数，量的历史里没有删快照那一步，
        // 不放出条目）；第一次按角色查映射时建上一版映射单元与单元两份索引（多出两次调用），查到第一个文件之外一个文件的单元时再建按号的文件索引（再多一次）
        "release_judgement_version_indexes" => {
            let rounds = fixed_point_rounds_of(product, at)?;
            let kept = exit("deadlist_entries")?.checked_sub(entry("deadlist_entries")?)?;
            let by_role = entry("mapped_units")? + entry("units")?;
            match calls.checked_sub(2 * rounds)? {
                0 => Some(rounds * kept),
                extra if extra == 2 * rounds => Some(rounds * (kept + by_role)),
                extra if extra == 3 * rounds => {
                    Some(rounds * (kept + by_role + entry("files_other_than_the_first")?))
                }
                _ => None,
            }
        }
        // s31 `transaction.rs:8006`：照抄台阶行之前每出一行记一步，池级三行加每盘六行
        "accounting_entries_of_this_publish" => Some(
            calls
                * (POOL_WIDE_ACCOUNTING_ROWS
                    + PER_DEVICE_ACCOUNTING_ROWS * number(state, "devices")?),
        ),
        // s32 `transaction.rs:4724`：整条实例表链重写时，被换下的那条旧链每一片走一步（链取自这一次的计划）
        "instance_table_chain" => Some(calls * entry("instance_table_chain_replaced")?),
        // s36 `transaction.rs:9002` 那一段：每轮按角色建上一版映射单元的索引，再出这一版的映射单元表
        "mapped_units_whole_table" => {
            let rounds = fixed_point_rounds_of(product, at)?;
            allocation_nodes_stay_put_in_every_round(product, at, rounds)?;
            Some(rounds * (entry("mapped_units")? + exit("mapped_units")?))
        }
        _ => None,
    }
}

/// 对账的合计：核了几格、不符几格（超上界的格也在里面）、其中超上界几格、读不到几格。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ReconciliationTotals {
    cells: u64,
    pub(super) disagreements: u64,
    pub(super) over_the_bound: u64,
    unread: u64,
}

/// 一行一格比的结果：(对不对, 超没超上界, 不符时那一行要打的两边值)；读不到交回 `None`。
fn reconcile_one_cell(
    entry: &ReconciledRow,
    product: &Product,
    at: &RoundKey,
    state: &Fields,
    calls: u64,
) -> Option<(bool, bool, String)> {
    if entry.relation == Relation::EveryCallTakesAStep {
        // s37 每次调用至少走一片叶（重写集里的叶或单元区罩到的叶），所以调过 ⇔ 有步
        let steps = product.steps_of(at, entry.segments, entry.row);
        return Some((
            (calls > 0) == (steps > 0),
            false,
            format!("steps={steps} calls={calls}"),
        ));
    }
    if entry.relation == Relation::Equals {
        let steps = product.steps_of(at, entry.segments, entry.row);
        let expected = expected_steps_of_the_rows_read_at_the_entry_and_the_exit(
            entry.row, calls, state, product, at,
        )?;
        return Some((
            steps == expected,
            false,
            format!("steps={steps} formula={expected}"),
        ));
    }
    let cell = call_site_cell(product, at, entry.segments, entry.row)?;
    let (agrees, over_the_bound) = call_site_cell_agrees(entry.relation, &cell);
    let sides = format!(
        "steps={} steps_to_the_hit={} whole_table_lengths={} lengths_to_the_hit={}",
        cell.steps, cell.steps_to_the_hit, cell.whole_table_lengths, cell.lengths_to_the_hit
    );
    Some((agrees, over_the_bound, sides))
}

/// 对账：每个量过的点、状态、种类、轮上，`RECONCILED_ROWS` 每行的站点读数与式子值（或调用处表长）比。
pub(super) fn reconciliation_report(product: &Product, out: &mut String) -> ReconciliationTotals {
    let mut totals = ReconciliationTotals::default();
    for entry in &RECONCILED_ROWS {
        let (mut row_cells, mut row_disagreements, mut row_over) = (0_u64, 0_u64, 0_u64);
        for (at, state) in &product.states {
            if !entry.kinds.contains(&at.2.as_str()) || !product.points.contains_key(&at.0) {
                continue;
            }
            let calls = product.calls_of(at, entry.segments, entry.row);
            let Some((agrees, over_the_bound, sides)) =
                reconcile_one_cell(entry, product, at, state, calls)
            else {
                totals.unread += 1;
                let _ = writeln!(
                    out,
                    "name=three_two_unread point={} state={} kind={} round={} row={} steps={} calls={calls}",
                    at.0, at.1, at.2, at.3, entry.row, product.steps_of(at, entry.segments, entry.row)
                );
                continue;
            };
            row_cells += 1;
            row_over += u64::from(over_the_bound);
            if !agrees {
                row_disagreements += 1;
                let _ = writeln!(
                    out,
                    "name=three_two_disagreement point={} state={} kind={} round={} row={} relation={:?} {sides} over_the_bound={over_the_bound} calls={calls}",
                    at.0, at.1, at.2, at.3, entry.row, entry.relation
                );
            }
        }
        totals.cells += row_cells;
        totals.disagreements += row_disagreements;
        totals.over_the_bound += row_over;
        let _ = writeln!(
            out,
            "name=three_two_row row={} relation={:?} cells_checked={row_cells} disagreements={row_disagreements} over_the_bound={row_over}",
            entry.row, entry.relation
        );
    }
    let not_reconciled: Vec<&str> = scan_rows()
        .into_iter()
        .filter(|row| {
            !RECONCILED_ROWS.iter().any(|entry| entry.row == *row)
                && *row != "whole_allocator_copy_before_the_judgement"
                && *row != "whole_allocator_copy_in_a_settling_round"
        })
        .collect();
    let _ = writeln!(
        out,
        "name=three_two_rows_not_reconciled count={} rows={}",
        not_reconciled.len(),
        not_reconciled.join(",")
    );
    let rows_with_a_bounded_part = RECONCILED_ROWS
        .iter()
        .filter(|entry| entry.relation.has_a_bounded_part())
        .count();
    let rows_steps_only = RECONCILED_ROWS
        .iter()
        .filter(|entry| entry.relation == Relation::EveryCallTakesAStep)
        .count();
    let _ = writeln!(
        out,
        "name=verdict quantity=three_two cells_checked={} disagreements={} cells_over_the_bound={} cells_with_the_scale_unread={} rows_reconciled={} rows_equal_only={} rows_with_a_bounded_part={rows_with_a_bounded_part} rows_steps_only={rows_steps_only} rows_not_reconciled={}",
        totals.cells,
        totals.disagreements,
        totals.over_the_bound,
        totals.unread,
        RECONCILED_ROWS.len(),
        RECONCILED_ROWS.len() - rows_with_a_bounded_part - rows_steps_only,
        not_reconciled.len()
    );
    totals
}

#[cfg(test)]
#[path = "reconciliation_of_site_readings_tests.rs"]
mod tests;
