//! PC2c、PC4c：去掉一个位置再算键（登记 5.3）。

use super::*;

// ───────────────────────────── PC2c、PC4c：去掉一个位置再算键（5.3） ─────────────────────────────

/// 对照样本上一条臂的一个状态：原来的键、结局、按位置可去掉的元素（读集按次序，集合按位置排序）。
struct ControlArmState {
    pub(super) key: Digest128,
    pub(super) outcome: Digest128,
    /// 集合臂的所选根；读集臂不用。
    pub(super) root: Option<Digest128>,
    /// （位置摘要，元素摘要）。
    pub(super) elements: Vec<(Digest128, Digest128)>,
    /// 这个状态出现过的位置，排好序去重（判「含不含 p」用）。
    pub(super) positions: Vec<Digest128>,
}

fn control_arm_states(
    control: &[ControlStateData],
    arm: Arm,
) -> (
    Vec<ControlArmState>,
    BTreeMap<Digest128, PositionDescription>,
) {
    let mut descriptions = BTreeMap::new();
    let states = control
        .iter()
        .map(|data| {
            let (root, elements, outcome): (
                Option<Digest128>,
                Vec<(Digest128, Digest128)>,
                Digest128,
            ) = match arm {
                Arm::RecoveryConsult | Arm::RecoveryIgnore | Arm::FullReadSet => {
                    let (calls, outcome) = match arm {
                        Arm::RecoveryConsult => (&data.consult_calls, data.consult_outcome),
                        Arm::RecoveryIgnore => (&data.ignore_calls, data.ignore_outcome),
                        Arm::FullReadSet | Arm::RecoveryPair | Arm::WalkUnits | Arm::AllUnits => {
                            (&data.full_calls, data.verdict_outcome)
                        }
                    };
                    for call in calls {
                        descriptions.insert(call.position, call.description);
                    }
                    (
                        None,
                        calls
                            .iter()
                            .map(|call| (call.position, call.digest))
                            .collect(),
                        outcome,
                    )
                }
                Arm::WalkUnits | Arm::AllUnits | Arm::RecoveryPair => {
                    let set = match arm {
                        Arm::WalkUnits => &data.walk_elements,
                        Arm::AllUnits
                        | Arm::RecoveryPair
                        | Arm::RecoveryConsult
                        | Arm::RecoveryIgnore
                        | Arm::FullReadSet => &data.unit_elements,
                    };
                    for (position, _element, description) in set {
                        descriptions.insert(*position, *description);
                    }
                    (
                        Some(data.root),
                        set.iter()
                            .map(|(position, element, _description)| (*position, *element))
                            .collect(),
                        data.verdict_outcome,
                    )
                }
            };
            let key = key_without(root, &elements, None);
            let mut positions: Vec<Digest128> =
                elements.iter().map(|(position, _)| *position).collect();
            positions.sort_unstable();
            positions.dedup();
            ControlArmState {
                key,
                outcome,
                root,
                elements,
                positions,
            }
        })
        .collect();
    (states, descriptions)
}

/// 去掉位置 `removed` 上的全部元素再算键；读集臂与 [`sequence_key`] 同一个算法，集合臂与 [`set_key`] 同一个。
fn key_without(
    root: Option<Digest128>,
    elements: &[(Digest128, Digest128)],
    removed: Option<Digest128>,
) -> Digest128 {
    let kept = || {
        elements
            .iter()
            .filter(move |(position, _)| Some(*position) != removed)
            .map(|(_, element)| *element)
    };
    let count = kept().count();
    match root {
        Some(root) => set_key_of(root, count, kept()),
        None => {
            let mut builder = DigestBuilder::new(DIGEST_TAG_KEY_SEQUENCE);
            builder.feed_word(u64::try_from(count).expect("个数"));
            for element in kept() {
                builder.feed_digest(element);
            }
            builder.finish()
        }
    }
}

/// 不一致数：状态按序号次序，同键里第一个是参照。
fn inconsistent_count(keys_and_outcomes: impl Iterator<Item = (Digest128, Digest128)>) -> u64 {
    let mut reference: HashMap<Digest128, Digest128> = HashMap::new();
    let mut inconsistent = 0;
    for (key, outcome) in keys_and_outcomes {
        let first = reference.entry(key).or_insert(outcome);
        if *first != outcome {
            inconsistent += 1;
        }
    }
    inconsistent
}

/// PC2c / PC4c 的结果：位置总数、使不一致数 ≥ 1 的位置个数与前十个（按位置原样排序）。
pub(super) struct DiscriminatingPositions {
    pub(super) positions: usize,
    pub(super) discriminating: Vec<PositionDescription>,
    pub(super) baseline_inconsistent: u64,
}

pub(super) fn discriminating_positions(
    control: &[ControlStateData],
    arm: Arm,
    threads: usize,
) -> DiscriminatingPositions {
    let (states, descriptions) = control_arm_states(control, arm);
    let baseline_inconsistent =
        inconsistent_count(states.iter().map(|state| (state.key, state.outcome)));
    let all_positions: Vec<Digest128> = descriptions.keys().copied().collect();
    let chunk = all_positions.len().div_ceil(threads.max(1)).max(1);
    let found = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for positions in all_positions.chunks(chunk) {
            let states = &states;
            let found = &found;
            scope.spawn(move || {
                let mut local = Vec::new();
                for removed in positions {
                    let inconsistent = inconsistent_count(states.iter().map(|state| {
                        let key = if state.positions.binary_search(removed).is_ok() {
                            key_without(state.root, &state.elements, Some(*removed))
                        } else {
                            state.key
                        };
                        (key, state.outcome)
                    }));
                    if inconsistent >= 1 {
                        local.push(*removed);
                    }
                }
                found.lock().expect("收集判别位置的锁").extend(local);
            });
        }
    });
    let mut discriminating: Vec<PositionDescription> = found
        .into_inner()
        .expect("收集判别位置的锁")
        .iter()
        .map(|position| descriptions[position])
        .collect();
    discriminating.sort_unstable();
    DiscriminatingPositions {
        positions: all_positions.len(),
        discriminating,
        baseline_inconsistent,
    }
}
