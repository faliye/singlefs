//! 计数：按键累计、样例行（每个配置至多几行）、panic 了的配置。全量按种子次序并起来，印成 `<名> COUNT <键> <数>`，末行 `<名> PROBE_RESULT …`。

use std::collections::BTreeMap;

#[derive(Default)]
pub struct Tally {
    pub configurations: u64,
    pub counts: BTreeMap<String, u64>,
    pub examples: Vec<String>,
    pub panics: Vec<String>,
}

/// 一个配置至多留几行样例。
pub const EXAMPLES_PER_CONFIGURATION: usize = 10;

/// 末行逐项印的键（名字 = 计数键），按这个次序。
pub const RESULT_KEYS: [(&str, &str); 28] = [
    ("cells", "cells"),
    ("c_published", "c published"),
    ("c_white", "c-white"),
    ("c_white_after_ceiling_no_outlet", "c-white final=after-ceiling"),
    ("c_white_after_budget", "c-white final=after-budget"),
    ("c_white_after_raise_refused", "c-white final=after-raise-refused-for-space"),
    ("c_white_stuck", "c-white final=stuck"),
    ("c_white_fired_after_a_series", "c-white final=fired-after-a-series"),
    ("c_white_publish_refusal", "c-white final=publish-refusal"),
    ("c_white_conjunct7_short_before_the_series", "c-white publish-refusal conjunct7-short-at-probe"),
    ("c_white_first_slot_pairs", "c-white first=slot-pairs"),
    ("c_white_first_formula", "c-white first=formula"),
    ("c_white_first_other", "c-white first=other"),
    ("c_white_slot_pairs_bound_loose", "c-white slot-pairs-bound-loose"),
    ("c_white_formula_unreachable_defer", "c-white formula-unreachable-defer"),
    ("o_white", "o-white"),
    ("oracle_published_after_a_series", "oracle published-after-a-series"),
    ("misrefused", "misrefused"),
    ("precheck_saved_a_vain_series", "precheck-saved-a-vain-series"),
    ("fired_first_with_versions_published", "fired-first-with-versions-published"),
    ("stuck_followed", "outlet stuck followed"),
    ("stuck_remount_published", "outlet stuck remount published"),
    ("stuck_push_ring_admitted", "outlet stuck push-ring-rehearsal admitted"),
    ("lost_first_followed", "outlet lost-first followed"),
    ("lost_first_remount_published", "outlet lost-first remount published"),
    ("lost_after_followed", "outlet lost-after followed"),
    ("lost_after_remount_published", "outlet lost-after remount published"),
    ("arms_differ", "arms-differ"),
];

impl Tally {
    pub fn add(&mut self, key: impl Into<String>, count: u64) {
        *self.counts.entry(key.into()).or_insert(0) += count;
    }

    pub fn count(&self, key: &str) -> u64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    pub fn example(&mut self, line: String) {
        if self.examples.len() < EXAMPLES_PER_CONFIGURATION {
            self.examples.push(line);
        }
    }

    pub fn merge(&mut self, other: Tally) {
        self.configurations += other.configurations;
        for (key, count) in other.counts {
            *self.counts.entry(key).or_insert(0) += count;
        }
        self.examples.extend(other.examples);
        self.panics.extend(other.panics);
    }

    pub fn print(&self, name: &str) {
        for (key, count) in &self.counts {
            println!("{name} COUNT {key} {count}");
        }
        for line in &self.examples {
            println!("{name} EXAMPLE {line}");
        }
        for line in &self.panics {
            println!("{name} PANIC {line}");
        }
        let fields: Vec<String> = RESULT_KEYS
            .iter()
            .map(|(field, key)| format!("{field}={}", self.count(key)))
            .collect();
        println!(
            "{name} PROBE_RESULT configurations={} {} c_ring_holding_at_probe={} queue_waiting_at_probe={} panics={}",
            self.configurations,
            fields.join(" "),
            self.count("c-ring-holding-at-probe"),
            self.count("queue-waiting-at-probe"),
            self.panics.len(),
        );
    }
}
