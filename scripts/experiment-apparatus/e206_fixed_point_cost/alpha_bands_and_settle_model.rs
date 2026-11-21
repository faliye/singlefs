//! 增长阶与档位、树的几何、稳定模型的结果与逐轮结算、释放链与跨越的结果。

use super::*;

// ---- 增长阶与判定带 ----
pub(super) fn alpha(low_quantity: f64, high_quantity: f64, low_position: f64, high_position: f64) -> f64 {
    (high_quantity / low_quantity).ln() / (high_position / low_position).ln()
}

/// 读法第 6 条的 α：端点取 10 → 10⁴。
pub(super) fn alpha_on_the_axis(low_quantity: f64, high_quantity: f64) -> f64 {
    alpha(low_quantity, high_quantity, AXIS_LOW, AXIS_HIGH)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Band {
    DoesNotGrow,
    Between,
    Grows,
}

impl Band {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::DoesNotGrow => "does_not_grow",
            Self::Between => "between",
            Self::Grows => "grows",
        }
    }
}

const BAND_DOES_NOT_GROW_AT_MOST: f64 = 0.25;
const BAND_GROWS_AT_LEAST: f64 = 0.5;

pub(super) fn band(growth: f64) -> Band {
    if growth >= BAND_GROWS_AT_LEAST {
        Band::Grows
    } else if growth <= BAND_DOES_NOT_GROW_AT_MOST {
        Band::DoesNotGrow
    } else {
        Band::Between
    }
}

/// Q1 的三档：不多、多、撞上界（第六节表）。
pub(super) fn rounds_verdict(peak_rounds: u64, did_not_settle: bool) -> &'static str {
    if did_not_settle || peak_rounds >= SETTLING_ROUNDS_LIMIT as u64 {
        "hits_the_limit"
    } else if peak_rounds > MANY_ROUNDS_ABOVE {
        "many"
    } else {
        "not_many"
    }
}

pub(super) fn lg(value: u64) -> u64 {
    u64::from(u64::BITS - value.leading_zeros())
}

/// 每层 64 叉的分层位图要几层：最小的 l ≥ 1 使 64ˡ ≥ n（E203 同一个式子）。
pub(super) fn levels_64(value: u64) -> u64 {
    let mut level = 1_u32;
    while 64_u64.pow(level) < value {
        level += 1;
    }
    u64::from(level)
}

// ---- 几何（第七节 ③，`calc/anchors.py` 照 `allocation_record_tree.rs:97`–`:119` 的找法）----
/// (每盘叶数, 根层级, 全树节点上界)。
pub(super) fn tree_geometry(unit_area_slots: u64) -> (u64, u32, u64) {
    let slots = UNIT_AREA_START_SLOT + unit_area_slots;
    let leaves = slots.div_ceil(LEAF_SLOTS);
    let mut level = 1_u32;
    loop {
        let span = LEAF_SLOTS * INTERNAL_FANOUT.pow(level - 1);
        if DISKS * slots.div_ceil(span) <= INTERNAL_FANOUT {
            break;
        }
        level += 1;
    }
    let below_the_root: u64 = (0..level)
        .map(|layer| DISKS * slots.div_ceil(LEAF_SLOTS * INTERNAL_FANOUT.pow(layer)))
        .sum();
    (leaves, level, 1 + below_the_root)
}

// ---- 抽象固定点（第七节 ③）：节点是叶，`g[n]` 是节点旧落点那条记录所在的叶，次序照 `transaction.rs:13378`–`:13457` ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModelArm {
    Today,
    Closure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ModelOutcome {
    /// 定下那一轮的序号；走满上界没定是 `None`。
    pub(super) rounds: Option<u64>,
    pub(super) per_round: Vec<u64>,
}

impl ModelOutcome {
    /// 轮功 W = Σ_r A_r。
    pub(super) fn round_work(&self) -> u64 {
        self.per_round.iter().sum()
    }

    /// 有效轮数 W ÷ A_final（读法第 4 条）。
    pub(super) fn effective_rounds(&self) -> f64 {
        self.round_work() as f64 / *self.per_round.last().unwrap_or(&1) as f64
    }

    /// P3 的轮间差之和（第五节 P3 式子的抽象形：每轮只碰比上一轮多出来的角色）。
    pub(super) fn sum_of_increments(&self) -> u64 {
        let mut before = 0;
        let mut sum = 0;
        for roles in &self.per_round {
            sum += roles.abs_diff(before);
            before = *roles;
        }
        sum
    }
}

fn close_over_the_release_chain(graph: &BTreeMap<u64, u64>, set: &mut BTreeSet<u64>) {
    let mut frontier: Vec<u64> = set.iter().copied().collect();
    while let Some(node) = frontier.pop() {
        let next = graph[&node];
        if set.insert(next) {
            frontier.push(next);
        }
    }
}

fn settle_model(
    graph: &BTreeMap<u64, u64>,
    dirtied_by_the_ordinary_roles: &BTreeSet<u64>,
    ordinary_roles: u64,
    leaf_of_the_new_placement: &dyn Fn(u64) -> u64,
    arm: ModelArm,
    limit: usize,
) -> ModelOutcome {
    let mut rewritten: BTreeSet<u64> = BTreeSet::new();
    let mut per_round = Vec::new();
    for round in 1..=limit {
        let plan = rewritten.clone();
        let roles = ordinary_roles + plan.len() as u64;
        per_round.push(roles);
        let mut release_dirty = dirtied_by_the_ordinary_roles.clone();
        release_dirty.extend(plan.iter().map(|node| graph[node]));
        if !release_dirty.is_subset(&plan) {
            rewritten.extend(release_dirty.iter().copied());
            if arm == ModelArm::Closure {
                close_over_the_release_chain(graph, &mut rewritten);
            }
            continue;
        }
        let mut changed = release_dirty;
        changed.extend((0..roles).map(leaf_of_the_new_placement));
        if changed.is_subset(&plan) {
            return ModelOutcome {
                rounds: Some(round as u64),
                per_round,
            };
        }
        rewritten.extend(changed);
        if arm == ModelArm::Closure {
            close_over_the_release_chain(graph, &mut rewritten);
        }
    }
    ModelOutcome {
        rounds: None,
        per_round,
    }
}

/// 第七节 ③ 的链：k 片叶，第 i 片叶的旧落点在第 i + 1 片，末片在自己；普通角色弄脏第 0 片，新落点全在链尾。
pub(super) fn chain_outcome(leaves: u64, ordinary_roles: u64, arm: ModelArm) -> ModelOutcome {
    let graph: BTreeMap<u64, u64> = (0..leaves)
        .map(|leaf| (leaf, (leaf + 1).min(leaves - 1)))
        .collect();
    settle_model(
        &graph,
        &BTreeSet::from([0]),
        ordinary_roles,
        &|_| leaves - 1,
        arm,
        SETTLING_ROUNDS_LIMIT,
    )
}

/// 第七节 ③ 的跨叶图：`g = {0:1, 1:2, 2:2, 3:4, 4:4}`，m = 2，前 4 个新落点在叶 2、之后在叶 3。
pub(super) fn crossing_outcome(arm: ModelArm) -> ModelOutcome {
    let graph = BTreeMap::from([(0, 1), (1, 2), (2, 2), (3, 4), (4, 4)]);
    settle_model(
        &graph,
        &BTreeSet::from([0]),
        2,
        &|index| if index < 4 { 2 } else { 3 },
        arm,
        SETTLING_ROUNDS_LIMIT,
    )
}
