//! 计数：按键累计、样例行（每个配置至多几行）、panic 了的配置。全量按种子次序并起来，印成 `<名> <键> <数>`。

use std::collections::BTreeMap;

#[derive(Default)]
pub struct Tally {
    pub configurations: u64,
    pub counts: BTreeMap<String, u64>,
    pub examples: Vec<String>,
    pub panics: Vec<String>,
}

/// 一个配置至多留几行样例。
pub const EXAMPLES_PER_CONFIGURATION: usize = 8;

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
        println!(
            "{name} PROBE_RESULT configurations={} slot_pair_refusals={} vain_series_by_the_inside_term={} vain_empty_publishes={} vain_with_stuck_report={} vain_without_outlet={} vain_remount_lets_it_through={} vain_remount_still_refuses={} outlet_kept_by_the_inside_term={} outlet_lost_after_a_series={} outlet_lost_first_round={} published_where_the_old_bound_fired={} arms_differ={} stuck_reports={} c_ring_holding_at_probe={} panics={}",
            self.configurations,
            self.count("slot-pair-refusal-calls"),
            self.count("vain-series"),
            self.count("vain-empty-publishes"),
            self.count("vain-series stuck-reported"),
            self.count("vain-series no-outlet-reported"),
            self.count("vain-series remount-retry published"),
            self.count("vain-series remount-retry refused"),
            self.count("outlet-kept-by-the-inside-term"),
            self.count("outlet-lost-after-a-series remount-retry published"),
            self.count("outlet-lost-first-round remount-retry published"),
            self.count("published-where-the-old-bound-fired"),
            self.count("arms-differ"),
            self.count("stuck-reports"),
            self.count("c-ring-holding-at-probe"),
            self.panics.len(),
        );
    }
}
