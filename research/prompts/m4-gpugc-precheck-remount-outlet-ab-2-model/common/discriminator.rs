//! 卸载重挂出路的判别子那一臂（两族共用这一份，拷进各族探针 crate 的 src/）。
//!
//! 产品那一臂每次发布之前开记、发布之后取最后一条（抬 F 之前先算判出放不下那一刻读分配器算的，连同在只进内存的盘上走的那一遍重挂，
//! `singlefs_core::mounted_session::remount_outlet_probe`）。判出的每一格，拿它与同一格重挂那一臂（正常卸载、再可写挂载、发同一个改动）
//! 写没写进对成四格表：判别子真且写进、真而没写进、假而写进、假且没写进。重挂那一臂卸载或挂载就失败的算没写进。
//! 四格表每个候选、每一判各一行（`series_before=all`），再按判出之前这次调用推过串没有拆两行（`0`、`1-or-more`）。
//! 计数键：`discriminator <候选> by=<判> cell=<true|false>-<published|not>`，拆的那两行另带 `series-before=<0|1-or-more>`；
//! 合取 7 那一判另记罩着记录的槽对最少的整段的对数分档：`discriminator fewest-blocked-pairs by=<判> pairs=<档> remount=<published|not>`；
//! 丁那一遍的结局与重挂那一臂的结局对成交叉表：`discriminator walked-in-memory by=<判> series-before=<…> walked=<结局> remount=<结局>`。
//! 走那一遍的挂钟收进全进程一份的表，主跑之后按判、按每块盘的单元区槽数印中位数与最大（`<名> WALK_COST …`）。
//! 自证：把核心层的判别子一律读成真、一律读成假各跑一小批，四格表里「假」那两列与「真」那两列各自归零、另两列之和等于判出的格数。

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use singlefs_core::mounted_session::remount_outlet_probe::{
    self as core_probe, DiscriminatorsAtTheRefusal,
};

use crate::tally::Tally;

/// 候选判别子：输出里的名字、派发提示里的名字。
pub const CANDIDATES: &[(&str, &str)] = &[
    ("slot-pairs-once-cluster-segments-forgotten", "甲一"),
    ("slot-pairs-and-formula-bound", "甲二"),
    ("opened-cluster-segment-empty-by-records", "乙一"),
    ("nearly-empty-full-segment", "乙二"),
    ("formula-bound-and-empty-segments-at-the-floor", "丙一"),
    ("and-the-unmount-walked-in-memory-goes-through", "丙二"),
    ("remount-walked-in-memory", "丁"),
];

/// 列四格表的判（`trigger_of` 交回的名字）：槽对、式子、合取 7 三判总列（判出 0 格也列一行），两样都有与两个成员同时空、同时有的三形判出过才列。
const JUDGEMENTS_ALWAYS_LISTED: &[&str] = &["slot-pairs", "formula", "conjunct-before-any-series"];
const JUDGEMENTS_LISTED_WHEN_FIRED: &[&str] = &["mixed", "both-empty", "both-set"];

/// 四格表的行：不拆（`all`），按判出之前这次调用推过串没有拆的两行。
const SERIES_BEFORE_SPLITS: &[&str] = &["all", "0", "1-or-more"];

/// 乙一、乙二只对合取 7 那一判列。
fn candidate_applies_to(candidate: &str, trigger: &str) -> bool {
    match candidate {
        "opened-cluster-segment-empty-by-records" | "nearly-empty-full-segment" => {
            trigger == "conjunct-before-any-series"
        }
        _ => true,
    }
}

fn value_of(candidate: &str, discriminators: &DiscriminatorsAtTheRefusal) -> bool {
    match candidate {
        "slot-pairs-once-cluster-segments-forgotten" => {
            discriminators.slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand
        }
        "slot-pairs-and-formula-bound" => discriminators.and_the_formula_bound_covers_the_demand,
        "opened-cluster-segment-empty-by-records" => {
            discriminators.an_opened_cluster_segment_is_empty_by_the_records
        }
        "nearly-empty-full-segment" => {
            discriminators.a_nearly_empty_full_segment_on_every_device_below_the_floor
        }
        "formula-bound-and-empty-segments-at-the-floor" => {
            discriminators.formula_bound_and_empty_segments_at_the_floor
        }
        "and-the-unmount-walked-in-memory-goes-through" => {
            discriminators.and_the_unmount_walked_in_memory_goes_through
        }
        "remount-walked-in-memory" => discriminators.remount_walked_in_memory_publishes,
        other => panic!("认不出的候选 {other}"),
    }
}

/// 产品那一臂发布之前调：清掉这个线程上一次留下的，开记。
pub fn before_the_product_publish() {
    core_probe::arm_and_clear_the_noted();
}

/// 产品那一臂发布之后调：关记，交回这次发布里最后一次判出放不下时记下的判别子。
pub fn after_the_product_publish() -> Option<DiscriminatorsAtTheRefusal> {
    core_probe::disarm_and_take_the_noted().pop()
}

fn pairs_bucket(pairs: u64) -> &'static str {
    match pairs {
        0 => "0",
        1 => "1",
        2 => "2",
        3..=4 => "3-4",
        5..=8 => "5-8",
        9..=16 => "9-16",
        _ => "17-32",
    }
}

fn series_before_split(series_before: usize) -> &'static str {
    if series_before == 0 {
        "0"
    } else {
        "1-or-more"
    }
}

/// 走那一遍的挂钟：一格一条。
struct WalkCost {
    trigger: String,
    unit_area_slots: String,
    walk_nanoseconds: u64,
    unmount_nanoseconds: u64,
}

/// 全进程一份：主跑各线程判出的每一格都往里放，主跑之后 [`print_walk_costs`] 取空。
static WALK_COSTS: Mutex<Vec<WalkCost>> = Mutex::new(Vec::new());

/// 判出的一格：`trigger` 是判出它的那一判，`series_before` 是判出之前这次调用推成了几串，
/// `discriminators` 是产品那一臂判出那一刻记下的，`remount_outcome` 是重挂那一臂的结局
/// （`published`、`failed:unmount …`、`failed:mount …`、被拒的成员名：卸载或再挂载就失败时同一次写没发、算没写进）。
pub fn tally_the_cell(
    tally: &mut Tally,
    trigger: &str,
    series_before: usize,
    discriminators: Option<DiscriminatorsAtTheRefusal>,
    remount_outcome: &str,
) {
    let remount_published = remount_outcome == "published";
    let remount_failed = remount_outcome.starts_with("failed:");
    let remount = if remount_published { "published" } else { "not" };
    let split = series_before_split(series_before);
    tally.add(format!("discriminator fired by={trigger} series-before={split}"), 1);
    let Some(discriminators) = discriminators else {
        tally.add(format!("discriminator missing by={trigger}"), 1);
        return;
    };
    tally.add(format!("discriminator noted by={trigger}"), 1);
    for (candidate, _) in CANDIDATES {
        if !candidate_applies_to(candidate, trigger) {
            continue;
        }
        let value = value_of(candidate, &discriminators);
        tally.add(
            format!("discriminator {candidate} by={trigger} cell={value}-{remount}"),
            1,
        );
        tally.add(
            format!("discriminator {candidate} by={trigger} series-before={split} cell={value}-{remount}"),
            1,
        );
        if !remount_published {
            // 没写进的再分：卸载或挂载就失败（failed），与挂上之后同一次写被拒（refused）。
            let not_because = if remount_failed { "failed" } else { "refused" };
            tally.add(
                format!("discriminator {candidate} by={trigger} cell={value}-not remount={not_because}"),
                1,
            );
        }
    }
    let devices = match discriminators.devices_judged {
        0 => "0",
        1 => "1",
        _ => "2-or-more",
    };
    tally.add(
        format!("discriminator devices-judged by={trigger} devices={devices} remount={remount}"),
        1,
    );
    if let Some(pairs) = discriminators.fewest_blocked_pairs_in_a_full_segment {
        tally.add(
            format!(
                "discriminator fewest-blocked-pairs by={trigger} pairs={} remount={remount}",
                pairs_bucket(pairs)
            ),
            1,
        );
    }
    // 丙二读的那一步：只进内存的卸载做成了（`went-through`）还是报了什么错。
    tally.add(
        format!(
            "discriminator unmount-walked-in-memory by={trigger} series-before={split} unmount={} remount={remount}",
            discriminators.unmount_walked_in_memory
        ),
        1,
    );
    tally.add(
        format!(
            "discriminator walked-in-memory by={trigger} series-before={split} walked={} remount={remount_outcome}",
            discriminators.remount_walked_in_memory
        ),
        1,
    );
    let agrees = if discriminators.remount_walked_in_memory == remount_outcome {
        "yes"
    } else {
        "no"
    };
    tally.add(
        format!("discriminator walked-in-memory-agrees by={trigger} series-before={split} agrees={agrees}"),
        1,
    );
    let unit_area_slots: Vec<String> = discriminators
        .unit_area_slots_of_each_device
        .iter()
        .map(u64::to_string)
        .collect();
    WALK_COSTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(WalkCost {
            trigger: trigger.to_string(),
            unit_area_slots: unit_area_slots.join("/"),
            walk_nanoseconds: discriminators.walk_nanoseconds,
            unmount_nanoseconds: discriminators.unmount_nanoseconds,
        });
}

/// 下中位数与最大（`values` 非空）。
fn lower_median_and_maximum(values: &mut [u64]) -> (u64, u64) {
    values.sort_unstable();
    (values[(values.len() - 1) / 2], values[values.len() - 1])
}

/// 印走那一遍的挂钟并取空：每一判、每一档单元区槽数一行，每一判不分档一行（`unit_area_slots=all`），不分判、按档各一行（`by=all`）：
/// `<名> WALK_COST by=… unit_area_slots=… cells=… walk_median_ns=… walk_max_ns=… unmount_median_ns=… unmount_max_ns=…`。
pub fn print_walk_costs(name: &str) {
    let costs = std::mem::take(&mut *WALK_COSTS.lock().unwrap_or_else(PoisonError::into_inner));
    let mut grouped: BTreeMap<(String, String), (Vec<u64>, Vec<u64>)> = BTreeMap::new();
    for cost in &costs {
        for key in [
            (cost.trigger.clone(), cost.unit_area_slots.clone()),
            (cost.trigger.clone(), "all".to_string()),
            ("all".to_string(), cost.unit_area_slots.clone()),
            ("all".to_string(), "all".to_string()),
        ] {
            let (walks, unmounts) = grouped.entry(key).or_default();
            walks.push(cost.walk_nanoseconds);
            unmounts.push(cost.unmount_nanoseconds);
        }
    }
    for ((trigger, unit_area_slots), (mut walks, mut unmounts)) in grouped {
        let cells = walks.len();
        let (walk_median, walk_maximum) = lower_median_and_maximum(&mut walks);
        let (unmount_median, unmount_maximum) = lower_median_and_maximum(&mut unmounts);
        println!(
            "{name} WALK_COST by={trigger} unit_area_slots={unit_area_slots} cells={cells} walk_median_ns={walk_median} walk_max_ns={walk_maximum} unmount_median_ns={unmount_median} unmount_max_ns={unmount_maximum}"
        );
    }
}

/// 丢掉收着的挂钟（自证那两小批跑的那几格不进代价）。
pub fn discard_walk_costs() {
    WALK_COSTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
}

/// 四格表的一行。
pub struct TableRow {
    pub candidate: &'static str,
    pub named: &'static str,
    pub trigger: &'static str,
    pub series_before: &'static str,
    pub fired: u64,
    pub true_published: u64,
    pub true_not: u64,
    pub false_published: u64,
    pub false_not: u64,
}

impl TableRow {
    fn sum(&self) -> u64 {
        self.true_published + self.true_not + self.false_published + self.false_not
    }
}

pub fn table_rows(tally: &Tally) -> Vec<TableRow> {
    let mut rows = Vec::new();
    let listed = JUDGEMENTS_ALWAYS_LISTED.iter().copied().chain(
        JUDGEMENTS_LISTED_WHEN_FIRED
            .iter()
            .copied()
            .filter(|trigger| tally.count(&format!("fired by={trigger}")) > 0),
    );
    for trigger in listed {
        for &series_before in SERIES_BEFORE_SPLITS {
            for &(candidate, named) in CANDIDATES {
                if !candidate_applies_to(candidate, trigger) {
                    continue;
                }
                let split_part = if series_before == "all" {
                    String::new()
                } else {
                    format!(" series-before={series_before}")
                };
                let cell = |value: bool, remount: &str| {
                    tally.count(&format!(
                        "discriminator {candidate} by={trigger}{split_part} cell={value}-{remount}"
                    ))
                };
                let fired = if series_before == "all" {
                    tally.count(&format!("fired by={trigger}"))
                } else {
                    tally.count(&format!(
                        "discriminator fired by={trigger} series-before={series_before}"
                    ))
                };
                rows.push(TableRow {
                    candidate,
                    named,
                    trigger,
                    series_before,
                    fired,
                    true_published: cell(true, "published"),
                    true_not: cell(true, "not"),
                    false_published: cell(false, "published"),
                    false_not: cell(false, "not"),
                });
            }
        }
    }
    rows
}

/// 印四格表：`<名> DISCRIMINATOR_TABLE candidate=… named=… by=… series_before=… fired=… true_published=… true_not=… false_published=… false_not=… sum=…`。
pub fn print_tables(name: &str, tally: &Tally) {
    for row in table_rows(tally) {
        println!(
            "{name} DISCRIMINATOR_TABLE candidate={} named={} by={} series_before={} fired={} true_published={} true_not={} false_published={} false_not={} sum={}",
            row.candidate,
            row.named,
            row.trigger,
            row.series_before,
            row.fired,
            row.true_published,
            row.true_not,
            row.false_published,
            row.false_not,
            row.sum()
        );
    }
}

/// 主跑那一份的自证：每一行四格之和等于那一判（那一拆）判出的格数，拆的两行判出的格数加起来等于那一判判出的格数，判出的每一格都记到了判别子。
pub fn checks_of_the_main_run(tally: &Tally) -> Vec<(String, bool)> {
    let missing: u64 = tally
        .counts
        .iter()
        .filter(|(key, _)| key.starts_with("discriminator missing by="))
        .map(|(_, count)| *count)
        .sum();
    let rows = table_rows(tally);
    let mismatched: Vec<String> = rows
        .iter()
        .filter(|row| row.sum() != row.fired)
        .map(|row| format!("{}/{}/{}", row.candidate, row.trigger, row.series_before))
        .collect();
    let splits_not_adding_up: Vec<&str> = JUDGEMENTS_ALWAYS_LISTED
        .iter()
        .chain(JUDGEMENTS_LISTED_WHEN_FIRED)
        .copied()
        .filter(|trigger| {
            let split_sum: u64 = ["0", "1-or-more"]
                .iter()
                .map(|split| {
                    tally.count(&format!(
                        "discriminator fired by={trigger} series-before={split}"
                    ))
                })
                .sum();
            split_sum != tally.count(&format!("fired by={trigger}"))
        })
        .collect();
    vec![
        (
            format!("判出的每一格都在判出那一刻记到了判别子（discriminator missing 共 {missing}）"),
            missing == 0,
        ),
        (
            format!(
                "每个候选、每一判、每一拆的四格之和等于那一判（那一拆）判出的格数（{} 行，对不上的：{mismatched:?}）",
                rows.len()
            ),
            mismatched.is_empty(),
        ),
        (
            format!("按推过串没有拆的两行判出的格数加起来等于那一判判出的格数（对不上的判：{splits_not_adding_up:?}）"),
            splits_not_adding_up.is_empty(),
        ),
    ]
}

/// 一律读成真（`forced` 为真）或一律读成假的那一小批的自证：每一行该归零的两列归零、另两列之和等于判出的格数，
/// 主跑里判出过的每一判这一批里也判出过（这一批的种子照主跑挑，见 [`seeds_for_the_forced_batches`]）。
pub fn checks_of_a_forced_batch(
    forced: bool,
    batch: &Tally,
    main: &Tally,
) -> Vec<(String, bool)> {
    let rows = table_rows(batch);
    let wrong: Vec<String> = rows
        .iter()
        .filter(|row| {
            let (zeroed, kept) = if forced {
                (
                    row.false_published + row.false_not,
                    row.true_published + row.true_not,
                )
            } else {
                (
                    row.true_published + row.true_not,
                    row.false_published + row.false_not,
                )
            };
            zeroed != 0 || kept != row.fired
        })
        .map(|row| format!("{}/{}/{}", row.candidate, row.trigger, row.series_before))
        .collect();
    let uncovered: Vec<&str> = JUDGEMENTS_ALWAYS_LISTED
        .iter()
        .chain(JUDGEMENTS_LISTED_WHEN_FIRED)
        .copied()
        .filter(|trigger| {
            main.count(&format!("fired by={trigger}")) > 0
                && batch.count(&format!("fired by={trigger}")) == 0
        })
        .collect();
    let fired_in_the_batch = batch.count("fired");
    let (zeroed_columns, kept_columns) = if forced {
        ("假而写进、假且没写进", "真那两列")
    } else {
        ("真且写进、真而没写进", "假那两列")
    };
    vec![(
        format!(
            "判别子一律读成{}的那一小批（{} 个配置、判出 {fired_in_the_batch} 格）：{zeroed_columns}两列归零、{kept_columns}之和等于判出的格数，主跑判出过的每一判这一批也判出过（不合的行：{wrong:?}；这一批没判出的判：{uncovered:?}）",
            if forced { "真" } else { "假" },
            batch.configurations
        ),
        fired_in_the_batch > 0 && wrong.is_empty() && uncovered.is_empty(),
    )]
}

/// 主跑里每个种子判出过哪几判：`run_seeds` 收集，交 [`seeds_for_the_forced_batches`] 挑种子。
pub fn triggers_fired_in(tally: &Tally) -> Vec<&'static str> {
    JUDGEMENTS_ALWAYS_LISTED
        .iter()
        .chain(JUDGEMENTS_LISTED_WHEN_FIRED)
        .copied()
        .filter(|trigger| tally.count(&format!("fired by={trigger}")) > 0)
        .collect()
}

/// 自证那两小批的种子：每一判取主跑里判出过它的头 `per_judgement` 个种子，并起来按种子排。
pub fn seeds_for_the_forced_batches(
    triggers_of_each_seed: &BTreeMap<u64, Vec<&'static str>>,
    per_judgement: usize,
) -> Vec<u64> {
    let mut chosen = std::collections::BTreeSet::new();
    let mut taken: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (seed, triggers) in triggers_of_each_seed {
        for trigger in triggers {
            let count = taken.entry(*trigger).or_insert(0);
            if *count < per_judgement {
                *count += 1;
                chosen.insert(*seed);
            }
        }
    }
    chosen.into_iter().collect()
}

/// 自证那两小批前后调：`Some(真假)` 把核心层的判别子一律读成它，`None` 回到照算。
pub fn force_every_discriminator(forced: Option<bool>) {
    core_probe::force_every_discriminator(forced);
}
