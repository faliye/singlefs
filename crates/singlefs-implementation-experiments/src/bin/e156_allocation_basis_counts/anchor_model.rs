use std::collections::{BTreeMap, BTreeSet};

/// 区域设备 [0, 1, 0]（D22（单元原子性怎么合成） 已定项 16 第 4 句）。
const REGION_DEVICE_NUMBERS: [u64; 3] = [0, 1, 0];
/// 造洞之后隔几次工作负载发布再造下一个（登记表头第 4 行）。
const WORKLOAD_PUBLISHES_BETWEEN_HOLES: u64 = 2;
const NEW_POOL_FILE_CREATION_TXG: u64 = 3;
/// 「前」位置第一个洞最早落的 txg（C380 逐字的 txg 5）。
const FRONT_HOLE_EARLIEST_TXG: u64 = 5;
const ZERO_UNIT_WARM_UP_TXGS: [u64; 2] = [1, 2];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnchorPublishRole {
    MakeFilesystemWarmUpOrNewPoolFileCreation,
    Workload,
    Crash,
    InstanceRow,
    WarmUpAfterRecovery,
}

#[derive(Clone, Copy, Debug)]
pub struct AnchorRow {
    pub txg: u64,
    pub floor: u64,
    pub tree_table_lower_bound: u64,
    pub interval_rule_tree_table_over_withheld: u64,
    pub missing_txgs: u64,
    pub live_holes: u64,
    pub live_holes_closed_form: u64,
    pub missing_txgs_closed_form: u64,
    /// 第 7 次重跑登记 A23：这一刻的 txg 减门槛（脚本 `--dump` 的 `lag`）。
    pub lag: u64,
    /// 第 7 次重跑登记 A24：这一刻还钉着的全零槽数（脚本 `--dump` 的 `pinned`）。
    pub pinned_slots: u64,
    pub is_observed: bool,
}

#[derive(Clone, Debug)]
pub struct AnchorCell {
    pub rows: Vec<AnchorRow>,
    pub holes: Vec<u64>,
    pub recovery_ends: Vec<u64>,
    pub fits: bool,
    pub end: u64,
}

impl AnchorCell {
    pub fn observed_row(&self, txg: u64) -> Option<&AnchorRow> {
        self.rows
            .iter()
            .find(|row| row.txg == txg && row.is_observed)
    }
}

fn device_of_txg(txg: u64) -> u64 {
    REGION_DEVICE_NUMBERS[usize::try_from(txg % 3).expect("对 3 取模落在 usize 里")]
}

/// 造洞的两种规则（第 7 次重跑登记 W29）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoleSchedule {
    /// 造洞之后隔 2 次工作负载发布再造下一个，一直造到 `hole_count` 个（Hh、Hc）。
    ConsecutiveHoles { front: bool, hole_count: u64 },
    /// 洞钉在同一个环槽：第 j 个洞的 txg = 第一个洞最早的 txg + j × 3S（Hs）；两个洞之间的发布全是工作负载。
    HolesOnTheSameRingSlot { front: bool, hole_count: u64 },
}

struct HistoryUnderConstruction {
    roles: BTreeMap<u64, AnchorPublishRole>,
    holes: Vec<u64>,
    recovery_ends: Vec<u64>,
    latest_txg: u64,
    workload_publishes_since_the_last_hole: u64,
}

impl HistoryUnderConstruction {
    fn publish_workload(&mut self) {
        self.latest_txg += 1;
        self.roles
            .insert(self.latest_txg, AnchorPublishRole::Workload);
        self.workload_publishes_since_the_last_hole = if self.holes.is_empty() {
            0
        } else {
            self.workload_publishes_since_the_last_hole + 1
        };
    }

    fn gap_is_long_enough(&self) -> bool {
        self.holes.is_empty()
            || self.workload_publishes_since_the_last_hole >= WORKLOAD_PUBLISHES_BETWEEN_HOLES
    }

    fn record_a_crash_hole_and_its_recovery(&mut self) {
        self.latest_txg += 1;
        self.roles.insert(self.latest_txg, AnchorPublishRole::Crash);
        self.holes.push(self.latest_txg);
        self.latest_txg += 1;
        self.roles
            .insert(self.latest_txg, AnchorPublishRole::InstanceRow);
        let mut covered_devices = BTreeSet::from([device_of_txg(self.latest_txg)]);
        while covered_devices.len() < 2 {
            self.latest_txg += 1;
            self.roles
                .insert(self.latest_txg, AnchorPublishRole::WarmUpAfterRecovery);
            covered_devices.insert(device_of_txg(self.latest_txg));
        }
        self.recovery_ends.push(self.latest_txg);
        self.workload_publishes_since_the_last_hole = 0;
    }
}

/// 第一个洞最早落的 txg：前位置是 C380 逐字的 txg 5，后位置是环转过一圈之后的 3S + 4。
fn first_hole_earliest_txg(slots_per_region: u64, front: bool) -> u64 {
    if front {
        FRONT_HOLE_EARLIEST_TXG
    } else {
        3 * slots_per_region + NEW_POOL_FILE_CREATION_TXG + 1
    }
}

fn build_history(
    slots_per_region: u64,
    schedule: HoleSchedule,
) -> (BTreeMap<u64, AnchorPublishRole>, Vec<u64>, Vec<u64>) {
    let ring_length = 3 * slots_per_region;
    let mut roles = BTreeMap::new();
    for setup_txg in 0..=NEW_POOL_FILE_CREATION_TXG {
        roles.insert(
            setup_txg,
            AnchorPublishRole::MakeFilesystemWarmUpOrNewPoolFileCreation,
        );
    }
    let mut history = HistoryUnderConstruction {
        roles,
        holes: Vec::new(),
        recovery_ends: Vec::new(),
        latest_txg: NEW_POOL_FILE_CREATION_TXG,
        workload_publishes_since_the_last_hole: 0,
    };
    match schedule {
        HoleSchedule::ConsecutiveHoles { front, hole_count } => {
            let first_hole_at = first_hole_earliest_txg(slots_per_region, front);
            while u64::try_from(history.holes.len()).expect("洞数落在 u64 内") < hole_count {
                if history.latest_txg + 1 >= first_hole_at && history.gap_is_long_enough() {
                    history.record_a_crash_hole_and_its_recovery();
                } else {
                    history.publish_workload();
                }
            }
        }
        HoleSchedule::HolesOnTheSameRingSlot { front, hole_count } => {
            let first_hole_at = first_hole_earliest_txg(slots_per_region, front);
            for hole_index in 0..hole_count {
                let hole_txg = first_hole_at + hole_index * ring_length;
                while history.latest_txg + 1 < hole_txg {
                    history.publish_workload();
                }
                assert!(
                    history.latest_txg + 1 == hole_txg && history.gap_is_long_enough(),
                    "同槽叠洞：下一个洞 txg {hole_txg} 距上一次挂载做完不足 2 次工作负载发布（E20：下一个洞 ≥ 上一次挂载做完 + 3）"
                );
                history.record_a_crash_hole_and_its_recovery();
            }
        }
    }
    (history.roles, history.holes, history.recovery_ends)
}

/// 脚本 `fits` 的两句：同槽的两个洞相隔不到一圈就撞上；有洞的前面隔整一圈还有一个洞也算撞上（第 4 次 W5）。
fn holes_fit_the_ring(holes: &[u64], ring_length: u64) -> bool {
    let pairs_do_not_collide = holes.iter().enumerate().all(|(index, first)| {
        holes[index + 1..].iter().all(|second| {
            !(first % ring_length == second % ring_length && first.abs_diff(*second) < ring_length)
        })
    });
    let no_hole_one_ring_before_another = holes
        .iter()
        .filter(|hole| **hole >= ring_length)
        .all(|hole| !holes.contains(&(hole - ring_length)));
    pairs_do_not_collide && no_hole_one_ring_before_another
}

/// 第 4 次重跑脚本的 `run_the_anchor_hole_script(S, k, pos, end)`：间距不变造 `hole_count` 个洞（[`HoleSchedule::ConsecutiveHoles`]）。
pub fn run_the_anchor_hole_script(
    slots_per_region: u64,
    hole_count: u64,
    front: bool,
    end_override: Option<u64>,
) -> AnchorCell {
    run_the_anchor_schedule(
        slots_per_region,
        HoleSchedule::ConsecutiveHoles { front, hole_count },
        end_override,
    )
}

/// 第 7 次重跑脚本的 `run(S, schedule, end)`：`end_override` 给了就跑到它，没给就跑到最后一个洞 + 3S + 2。
pub fn run_the_anchor_schedule(
    slots_per_region: u64,
    schedule: HoleSchedule,
    end_override: Option<u64>,
) -> AnchorCell {
    let ring_length = 3 * slots_per_region;
    let (mut roles, holes, recovery_ends) = build_history(slots_per_region, schedule);
    let last_txg = *roles.keys().next_back().expect("至少有 mkfs 那一条");
    let end = end_override
        .unwrap_or_else(|| holes.last().expect("没给终点的格至少有一个洞") + ring_length + 2);
    for padding_txg in last_txg + 1..=end {
        roles.insert(padding_txg, AnchorPublishRole::Workload);
    }
    let mut ring: BTreeMap<u64, u64> = BTreeMap::new();
    let mut tree_tables: BTreeMap<u64, Option<u64>> = BTreeMap::from([(0, None)]);
    let mut live_tree_table = 0u64;
    let mut rows = Vec::new();
    let mount_first_txgs: BTreeSet<u64> = holes.iter().map(|hole| hole + 1).collect();
    let mut pins: BTreeSet<u64> = BTreeSet::new();
    for txg in 0..=end {
        let role = roles[&txg];
        if !ZERO_UNIT_WARM_UP_TXGS.contains(&txg) && txg > 0 {
            tree_tables.insert(live_tree_table, Some(txg));
            tree_tables.insert(txg, None);
            live_tree_table = txg;
        }
        if mount_first_txgs.contains(&txg) {
            pins = (0..ring_length.min(txg))
                .filter(|slot| !ring.contains_key(slot))
                .collect();
        }
        if role != AnchorPublishRole::Crash {
            ring.insert(txg % ring_length, txg);
        }
        if role == AnchorPublishRole::Crash || txg < NEW_POOL_FILE_CREATION_TXG {
            continue;
        }
        let oldest = *ring.values().min().expect("环里至少有 mkfs 那一条根");
        let floor = pins
            .iter()
            .copied()
            .filter(|slot| !ring.contains_key(slot))
            .fold(oldest, u64::min);
        let roots: BTreeSet<u64> = ring.values().copied().collect();
        let is_referenced = |allocation: u64, release: Option<u64>| match release {
            None => true,
            Some(release) => roots
                .iter()
                .any(|root| allocation <= *root && *root < release),
        };
        let withheld: BTreeSet<u64> = tree_tables
            .iter()
            .filter(|(_, release)| match release {
                None => true,
                Some(release) => *release > floor,
            })
            .map(|(allocation, _)| *allocation)
            .collect();
        let referenced: BTreeSet<u64> = tree_tables
            .iter()
            .filter(|(allocation, release)| is_referenced(**allocation, **release))
            .map(|(allocation, _)| *allocation)
            .collect();
        let interval_rule_withheld = referenced.clone();
        let missing_txgs = (oldest..=txg)
            .filter(|candidate| !roots.contains(candidate))
            .count();
        let live_holes = holes
            .iter()
            .filter(|hole| {
                oldest <= **hole
                    && **hole <= txg
                    && !roots.contains(hole)
                    && ring
                        .get(&(**hole % ring_length))
                        .is_none_or(|occupant| *occupant < **hole)
            })
            .count();
        let live_closed_form: Vec<u64> = holes
            .iter()
            .copied()
            .filter(|hole| {
                *hole <= txg && txg < hole + ring_length - u64::from(*hole < ring_length)
            })
            .collect();
        let first_live_hole_after_one_ring = live_closed_form
            .iter()
            .copied()
            .filter(|hole| *hole >= ring_length)
            .min();
        let missing_txgs_closed_form = match first_live_hole_after_one_ring {
            Some(hole) => txg - hole + 1,
            None => u64::try_from(live_closed_form.len()).expect("洞数落在 u64 内"),
        };
        rows.push(AnchorRow {
            txg,
            floor,
            tree_table_lower_bound: u64::try_from(withheld.difference(&referenced).count())
                .expect("落在 u64 内"),
            interval_rule_tree_table_over_withheld: u64::try_from(
                interval_rule_withheld.difference(&referenced).count(),
            )
            .expect("落在 u64 内"),
            missing_txgs: u64::try_from(missing_txgs).expect("落在 u64 内"),
            live_holes: u64::try_from(live_holes).expect("落在 u64 内"),
            live_holes_closed_form: u64::try_from(live_closed_form.len()).expect("落在 u64 内"),
            missing_txgs_closed_form,
            lag: txg - floor,
            pinned_slots: u64::try_from(
                pins.iter()
                    .copied()
                    .filter(|slot| !ring.contains_key(slot))
                    .count(),
            )
            .expect("落在 u64 内"),
            is_observed: txg == NEW_POOL_FILE_CREATION_TXG
                || role == AnchorPublishRole::Workload
                || recovery_ends.contains(&txg),
        });
    }
    let fits = holes_fit_the_ring(&holes, ring_length);
    AnchorCell {
        rows,
        holes,
        recovery_ends,
        fits,
        end,
    }
}

/// k = 0 的终点：同一 S 下 k ∈ {1, 2, 4} × {前, 后} 各族终点的最大值（脚本的 `k0_end`，排不下的格也算进去）。
pub fn without_holes_end(slots_per_region: u64) -> u64 {
    [false, true]
        .iter()
        .flat_map(|front| {
            [1u64, 2, 4].map(|hole_count| {
                run_the_anchor_hole_script(slots_per_region, hole_count, *front, None).end
            })
        })
        .max()
        .expect("六个格")
}

/// 脚本的 `a10`：每个树表单元第一次能被分配的那次发布的 txg − 它的释放 txg。
pub fn tree_table_reclaim_lags(slots_per_region: u64, end: u64) -> BTreeSet<u64> {
    let ring_length = 3 * slots_per_region;
    let mut lags = BTreeSet::new();
    let mut ring: BTreeMap<u64, u64> = BTreeMap::new();
    let mut released: BTreeMap<u64, u64> = BTreeMap::new();
    let mut live_tree_table = 0u64;
    let mut seen: BTreeSet<(u64, u64)> = BTreeSet::new();
    for txg in 0..=end {
        if !ZERO_UNIT_WARM_UP_TXGS.contains(&txg) && txg > 0 {
            released.insert(live_tree_table, txg);
            live_tree_table = txg;
        }
        ring.insert(txg % ring_length, txg);
        let floor = *ring.values().min().expect("环里有根");
        for (allocation, release) in &released {
            if *release <= floor && seen.insert((*allocation, *release)) {
                lags.insert(txg + 1 - release);
            }
        }
    }
    lags
}

/// 脚本 `--dump` 的那几行（`DUMP` 换成调用方加的前缀），格序照脚本：先按位置标签（`-` < `back` < `front`）、再按 k。
pub fn dump_lines(slots_per_region: u64) -> Vec<String> {
    let without_holes = run_the_anchor_hole_script(
        slots_per_region,
        0,
        false,
        Some(without_holes_end(slots_per_region)),
    );
    let mut cells: Vec<(u64, &'static str, AnchorCell)> = vec![(0, "-", without_holes)];
    for (front, label) in [(false, "back"), (true, "front")] {
        for hole_count in [1u64, 2, 4] {
            cells.push((
                hole_count,
                label,
                run_the_anchor_hole_script(slots_per_region, hole_count, front, None),
            ));
        }
    }
    let mut lines = Vec::new();
    for (hole_count, label, cell) in &cells {
        let fits_label = if cell.fits { "True" } else { "False" };
        for row in cell.rows.iter().filter(|row| row.is_observed) {
            lines.push(format!(
                "S={slots_per_region} k={hole_count} pos={label} fits={fits_label} txg={} floor={} h_que={} h_dong={} q1a_tree_table_lower_bound={}",
                row.txg, row.floor, row.missing_txgs, row.live_holes, row.tree_table_lower_bound
            ));
        }
    }
    lines
}

/// 脚本的 `k_full(S, pos)`：`Hh(k + 1, 位置)` 开始排不下（`fits = False`）的那个 k，从 1 往上数。
pub fn largest_hole_count_that_fits(slots_per_region: u64, front: bool) -> u64 {
    let ring_length = 3 * slots_per_region;
    let mut hole_count = 1u64;
    loop {
        let (_, holes, _) = build_history(
            slots_per_region,
            HoleSchedule::ConsecutiveHoles {
                front,
                hole_count: hole_count + 1,
            },
        );
        if !holes_fit_the_ring(&holes, ring_length) {
            return hole_count;
        }
        hole_count += 1;
    }
}

/// `k_满(S)`（登记 W26 读法甲）：前位置的族 `Hh(k, 前)` 里，全部洞都落在第一圈（txg < 3S）的最大 k。
pub fn largest_front_hole_count_inside_the_first_ring(slots_per_region: u64) -> u64 {
    let ring_length = 3 * slots_per_region;
    let mut hole_count = 1u64;
    loop {
        let (_, holes, _) = build_history(
            slots_per_region,
            HoleSchedule::ConsecutiveHoles {
                front: true,
                hole_count: hole_count + 1,
            },
        );
        if holes.iter().any(|hole| *hole >= ring_length) {
            return hole_count;
        }
        hole_count += 1;
    }
}

/// 登记第十三节命令一 `anchors_e156_r7.py`（不带 `--dump`）每族一行的摘要，除三个键名照 [`crate::emit_hole_family_anchor_cells`] 文档里那张对照换过之外
/// 格式逐字相同（Python 列表的写法与 `{:?}` 一致）。
pub fn summary_line(slots_per_region: u64, label: &str, cell: &AnchorCell) -> String {
    let observed: Vec<&AnchorRow> = cell.rows.iter().filter(|row| row.is_observed).collect();
    let peak = observed
        .iter()
        .map(|row| row.tree_table_lower_bound)
        .max()
        .expect("至少一个观测点");
    let positive_points = observed
        .iter()
        .filter(|row| row.tree_table_lower_bound > 0)
        .count();
    let final_value = observed
        .last()
        .expect("至少一个观测点")
        .tree_table_lower_bound;
    let lag_peak = observed
        .iter()
        .map(|row| row.lag)
        .max()
        .expect("至少一个观测点");
    let maximum_missing = observed
        .iter()
        .map(|row| row.missing_txgs)
        .max()
        .expect("至少一个观测点");
    let maximum_live_holes = observed
        .iter()
        .map(|row| row.live_holes)
        .max()
        .expect("至少一个观测点");
    let maximum_pinned = observed
        .iter()
        .map(|row| row.pinned_slots)
        .max()
        .expect("至少一个观测点");
    format!(
        "S={slots_per_region} family={label} holes={:?} recovery_ends={:?} end={} fits={} observations={} q1a_tt_peak={peak} q1a_tt_positive_points={positive_points} q1a_tt_final={final_value} lag_peak={lag_peak} missing_txgs_peak={maximum_missing} live_holes_peak={maximum_live_holes} pinned_slots_peak={maximum_pinned}",
        cell.holes,
        cell.recovery_ends,
        cell.end,
        if cell.fits { "True" } else { "False" },
        observed.len(),
    )
}

/// 脚本 `--dump` 每个观测点一行（`DUMP` 换成调用方加的前缀）。
pub fn dump_lines_of_one_family(
    slots_per_region: u64,
    label: &str,
    cell: &AnchorCell,
) -> Vec<String> {
    cell.rows
        .iter()
        .filter(|row| row.is_observed)
        .map(|row| {
            format!(
                "S={slots_per_region} family={label} txg={} floor={} lag={} h_que={} h_dong={} pinned={} q1a_tree_table_lower_bound={}",
                row.txg,
                row.floor,
                row.lag,
                row.missing_txgs,
                row.live_holes,
                row.pinned_slots,
                row.tree_table_lower_bound
            )
        })
        .collect()
}
