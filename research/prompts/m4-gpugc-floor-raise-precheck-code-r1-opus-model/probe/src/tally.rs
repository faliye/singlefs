//! 一条或几条历史的计数：每一次经会话的调用怎么收场、先算判了什么、臂（旁路先算的那一份拷贝）怎么收场、
//! 每一串抬 F 推成前后的读数对不对得上上界、同一次调用里先后几次被拒的需求变没变。

use std::collections::BTreeMap;

use singlefs_core::mounted_session::floor_raise_precheck_probe::{PrecheckObserved, RaiseObserved};
use singlefs_format::SLOT_BYTES;

/// 一份计数里每类逐行留几条例子（命中照全留）。
const EXAMPLES_KEPT: usize = 12;

#[derive(Default)]
pub struct Tally {
    pub histories: u64,
    pub steps: u64,
    pub setup_refused: BTreeMap<String, u64>,
    pub mount_refused: BTreeMap<String, u64>,
    pub panics: Vec<String>,
    pub calls: BTreeMap<String, u64>,
    pub first_round_fired: u64,
    pub later_round_fired: u64,
    pub oracle: BTreeMap<String, u64>,
    pub hits: Vec<String>,
    pub examples_fired: Vec<String>,
    pub slack_of_the_pushed: BTreeMap<String, u64>,
    pub deficit_of_the_fired: BTreeMap<String, u64>,
    pub raises: u64,
    pub publishes_per_raise: BTreeMap<usize, u64>,
    pub formula_margin_minimum_slots: Option<i128>,
    pub pairs_margin_minimum_slots: Option<i128>,
    pub unsound_formula: u64,
    pub unsound_pairs: u64,
    pub examples_unsound: Vec<String>,
    pub checkpoint_cost_delta: BTreeMap<i128, u64>,
    pub demand_delta: BTreeMap<String, u64>,
    pub pushed_then_refused_calls: u64,
    pub pushed_then_published_calls: u64,
    pub stuck_reported_with_the_precheck: u64,
}

pub fn add<K: Ord + Clone>(map: &mut BTreeMap<K, u64>, key: K, count: u64) {
    *map.entry(key).or_insert(0) += count;
}

fn bucket(slots: i128) -> &'static str {
    match slots {
        i128::MIN..=-1 => "<0",
        0 => "0",
        1 => "1",
        2..=3 => "2-3",
        4..=7 => "4-7",
        8..=15 => "8-15",
        16..=63 => "16-63",
        _ => "64+",
    }
}

fn slots_of(bytes: i128) -> i128 {
    bytes.div_euclid(i128::from(SLOT_BYTES))
}

fn keep_minimum(slot: &mut Option<i128>, value: i128) {
    *slot = Some(slot.map_or(value, |kept| kept.min(value)));
}

impl Tally {
    pub fn merge(&mut self, other: Tally) {
        self.histories += other.histories;
        self.steps += other.steps;
        for (key, count) in other.setup_refused {
            add(&mut self.setup_refused, key, count);
        }
        for (key, count) in other.mount_refused {
            add(&mut self.mount_refused, key, count);
        }
        self.panics.extend(other.panics);
        for (key, count) in other.calls {
            add(&mut self.calls, key, count);
        }
        self.first_round_fired += other.first_round_fired;
        self.later_round_fired += other.later_round_fired;
        for (key, count) in other.oracle {
            add(&mut self.oracle, key, count);
        }
        self.hits.extend(other.hits);
        self.examples_fired.extend(other.examples_fired);
        for (key, count) in other.slack_of_the_pushed {
            add(&mut self.slack_of_the_pushed, key, count);
        }
        for (key, count) in other.deficit_of_the_fired {
            add(&mut self.deficit_of_the_fired, key, count);
        }
        self.raises += other.raises;
        for (key, count) in other.publishes_per_raise {
            add(&mut self.publishes_per_raise, key, count);
        }
        if let Some(value) = other.formula_margin_minimum_slots {
            keep_minimum(&mut self.formula_margin_minimum_slots, value);
        }
        if let Some(value) = other.pairs_margin_minimum_slots {
            keep_minimum(&mut self.pairs_margin_minimum_slots, value);
        }
        self.unsound_formula += other.unsound_formula;
        self.unsound_pairs += other.unsound_pairs;
        self.examples_unsound.extend(other.examples_unsound);
        for (key, count) in other.checkpoint_cost_delta {
            add(&mut self.checkpoint_cost_delta, key, count);
        }
        for (key, count) in other.demand_delta {
            add(&mut self.demand_delta, key, count);
        }
        self.pushed_then_refused_calls += other.pushed_then_refused_calls;
        self.pushed_then_published_calls += other.pushed_then_published_calls;
        self.stuck_reported_with_the_precheck += other.stuck_reported_with_the_precheck;
    }

    /// 先算记下的几次：头一次被拒没判出放不下时，按被点名的盘里上界减需求最小的那一块记余量（槽），连同这次调用收没收下；
    /// 判出放不下时记差额。同一次调用里先后两次被拒都点名同一块盘、同一判时，记需求变了几槽。
    pub fn note_prechecks(&mut self, prechecks: &[PrecheckObserved], published: bool, label: &str) {
        let outcome = if published { "published" } else { "refused" };
        if let Some(first) = prechecks.first() {
            if first.refusal_kind == "space-admission" && !first.fired {
                let slack = first
                    .short_devices
                    .iter()
                    .filter_map(|short| {
                        short
                            .bound
                            .map(|bound| (short.judgement, bound - i128::from(short.demand)))
                    })
                    .min_by_key(|(_, slack)| *slack);
                if let Some((judgement, slack)) = slack {
                    add(
                        &mut self.slack_of_the_pushed,
                        format!("{judgement} {outcome} {}", bucket(slots_of(slack))),
                        1,
                    );
                }
            }
        }
        for precheck in prechecks.iter().filter(|precheck| precheck.fired) {
            for short in &precheck.short_devices {
                if let Some(bound) = short.bound {
                    if bound < i128::from(short.demand) {
                        add(
                            &mut self.deficit_of_the_fired,
                            format!(
                                "{} {}",
                                short.judgement,
                                bucket(slots_of(i128::from(short.demand) - bound))
                            ),
                            1,
                        );
                    }
                }
            }
            if self.examples_fired.len() < EXAMPLES_KEPT {
                self.examples_fired
                    .push(format!("{label} short={}", describe(precheck)));
            }
        }
        for pair in prechecks.windows(2) {
            for later in &pair[1].short_devices {
                if let Some(earlier) = pair[0].short_devices.iter().find(|earlier| {
                    earlier.device == later.device && earlier.judgement == later.judgement
                }) {
                    let delta = slots_of(i128::from(later.demand) - i128::from(earlier.demand));
                    add(
                        &mut self.demand_delta,
                        format!("{} {delta}", later.judgement),
                        1,
                    );
                }
            }
        }
    }

    /// 每一串推成前后的读数对上界：式子那一判的可用涨了多少不许超过 (defer + 隔离) 的字节，段外槽对涨了多少不许超过
    /// defer + 隔离 + 空闲而被挡的槽数（先算用的两个上界）。`without_deferred` 是探针自证：上界里不算 defer，推成一串回收过槽就该判出超出。
    pub fn note_raises(&mut self, raises: &[RaiseObserved], without_deferred: bool, label: &str) {
        for raise in raises {
            self.raises += 1;
            add(&mut self.publishes_per_raise, raise.publishes, 1);
            for before in &raise.before {
                let Some(after) = raise
                    .after
                    .iter()
                    .find(|after| after.device == before.device)
                else {
                    continue;
                };
                let deferred = if without_deferred { 0 } else { before.deferred };
                add(
                    &mut self.checkpoint_cost_delta,
                    i128::from(after.checkpoint_cost) - i128::from(before.checkpoint_cost),
                    1,
                );
                if let (Some(available_before), Some(available_after)) =
                    (before.formula_available, after.formula_available)
                {
                    let gain = available_after - available_before;
                    let bound_gain =
                        i128::from(deferred + before.isolated) * i128::from(SLOT_BYTES);
                    keep_minimum(
                        &mut self.formula_margin_minimum_slots,
                        slots_of(bound_gain - gain),
                    );
                    if gain > bound_gain {
                        self.unsound_formula += 1;
                        if self.examples_unsound.len() < EXAMPLES_KEPT {
                            self.examples_unsound.push(format!(
                                "UNSOUND_FORMULA {label} device={} gain_bytes={gain} bound_gain_bytes={bound_gain} checkpoint_cost={}->{} deferred={} isolated={} publishes={}",
                                before.device, before.checkpoint_cost, after.checkpoint_cost, before.deferred, before.isolated, raise.publishes
                            ));
                        }
                    }
                }
                let pairs_gain = i128::from(after.pairs_outside) - i128::from(before.pairs_outside);
                let pairs_bound = i128::from(deferred + before.isolated + before.free_but_blocked);
                keep_minimum(
                    &mut self.pairs_margin_minimum_slots,
                    pairs_bound - pairs_gain,
                );
                if pairs_gain > pairs_bound {
                    self.unsound_pairs += 1;
                    if self.examples_unsound.len() < EXAMPLES_KEPT {
                        self.examples_unsound.push(format!(
                            "UNSOUND_PAIRS {label} device={} pairs={}->{} deferred={} isolated={} free_but_blocked={}",
                            before.device, before.pairs_outside, after.pairs_outside, before.deferred, before.isolated, before.free_but_blocked
                        ));
                    }
                }
            }
        }
    }

    pub fn print(&self, name: &str) {
        println!(
            "{name} histories={} steps={} panics={}",
            self.histories,
            self.steps,
            self.panics.len()
        );
        for panic in &self.panics {
            println!("{name} PANIC {panic}");
        }
        for (key, count) in &self.setup_refused {
            println!("{name} setup_refused {key} {count}");
        }
        for (key, count) in &self.mount_refused {
            println!("{name} mount_refused {key} {count}");
        }
        for (key, count) in &self.calls {
            println!("{name} call {key} {count}");
        }
        println!(
            "{name} precheck first_round_fired={} later_round_fired={} pushed_then_refused_calls={} pushed_then_published_calls={} stuck_reported_with_the_precheck={}",
            self.first_round_fired, self.later_round_fired, self.pushed_then_refused_calls, self.pushed_then_published_calls, self.stuck_reported_with_the_precheck
        );
        for (key, count) in &self.oracle {
            println!("{name} oracle {key} {count}");
        }
        for (key, count) in &self.slack_of_the_pushed {
            println!("{name} slack_of_the_pushed {key} {count}");
        }
        for (key, count) in &self.deficit_of_the_fired {
            println!("{name} deficit_of_the_fired {key} {count}");
        }
        println!(
            "{name} raises={} unsound_formula={} unsound_pairs={} formula_margin_minimum_slots={:?} pairs_margin_minimum_slots={:?}",
            self.raises, self.unsound_formula, self.unsound_pairs, self.formula_margin_minimum_slots, self.pairs_margin_minimum_slots
        );
        for (key, count) in &self.publishes_per_raise {
            println!("{name} publishes_per_raise {key} {count}");
        }
        for (key, count) in &self.checkpoint_cost_delta {
            println!("{name} checkpoint_cost_delta {key} {count}");
        }
        for (key, count) in &self.demand_delta {
            println!("{name} demand_delta_slots {key} {count}");
        }
        for line in &self.examples_unsound {
            println!("{name} {line}");
        }
        for line in self.examples_fired.iter().take(EXAMPLES_KEPT * 2) {
            println!("{name} FIRED {line}");
        }
        for line in &self.hits {
            println!("{name} {line}");
        }
        println!(
            "{name} PROBE_RESULT histories={} hits={} unsound_formula={} unsound_pairs={} first_round_fired={} later_round_fired={} panics={}",
            self.histories, self.hits.len(), self.unsound_formula, self.unsound_pairs, self.first_round_fired, self.later_round_fired, self.panics.len()
        );
    }
}

/// 一次先算点名的盘：盘号、哪一判、上界与需求（槽，上界没有写 none）。
pub fn describe(precheck: &PrecheckObserved) -> String {
    precheck
        .short_devices
        .iter()
        .map(|short| {
            let bound = short
                .bound
                .map_or_else(|| "none".to_string(), |bound| slots_of(bound).to_string());
            format!(
                "{}:{}:available={}:bound={}:demand={}",
                short.device,
                short.judgement,
                slots_of(short.available),
                bound,
                slots_of(i128::from(short.demand))
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
