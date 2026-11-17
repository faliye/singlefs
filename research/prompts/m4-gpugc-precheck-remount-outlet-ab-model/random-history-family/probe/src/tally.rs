//! 计数：按键累计、样例行（每个配置至多几行）、panic 了的配置。全量按种子次序并起来，
//! 印成 `<名> COUNT <键> <数>`、`<名> EXAMPLE <行>`、`<名> PANIC <行>`，末尾一行 `<名> PROBE_RESULT <名字>=<数> …`。

use std::collections::BTreeMap;

#[derive(Default)]
pub struct Tally {
    pub configurations: u64,
    pub counts: BTreeMap<String, u64>,
    pub examples: Vec<String>,
    pub panics: Vec<String>,
}

/// 一个配置至多留几行样例。
pub const EXAMPLES_PER_CONFIGURATION: usize = 12;

/// 汇总行的名字与计数键，按这个次序印。
pub const RESULT_KEYS: &[(&str, &str)] = &[
    ("steps", "steps"),
    ("changes", "changes"),
    ("fired", "fired"),
    ("fired_by_slot_pairs", "fired by=slot-pairs"),
    ("fired_by_formula", "fired by=formula"),
    ("fired_by_mixed", "fired by=mixed"),
    ("fired_by_conjunct_before_any_series", "fired by=conjunct-before-any-series"),
    ("fired_both_members_empty", "fired by=both-empty"),
    ("fired_both_members_set", "fired by=both-set"),
    ("conjunct_fired_with_versions_published", "fired by=conjunct-before-any-series versions-published=yes"),
    ("misrefused_slot_pairs", "misrefused by=slot-pairs"),
    ("misrefused_formula", "misrefused by=formula"),
    ("misrefused_mixed", "misrefused by=mixed"),
    ("misrefused_conjunct_before_any_series", "misrefused by=conjunct-before-any-series"),
    ("remount_published_slot_pairs", "remount published by=slot-pairs"),
    ("remount_published_formula", "remount published by=formula"),
    ("remount_published_mixed", "remount published by=mixed"),
    ("remount_published_conjunct_before_any_series", "remount published by=conjunct-before-any-series"),
    ("oracle_pushed_more_series_than_the_product", "oracle pushed-more-series-than-the-product"),
    ("base_published_after_a_series", "base published-after-a-series"),
    ("conjunct_before_any_series_judged", "conjunct-before-any-series judged"),
    ("conjunct_before_any_series_found", "conjunct-before-any-series found=true"),
    ("series_product", "series arm=product"),
    ("series_product_raised", "series arm=product empty-segments=raised"),
    ("series_product_mixed", "series arm=product empty-segments=mixed"),
    ("series_product_crossed_the_floor", "series arm=product crossed-the-floor"),
    ("series_oracle", "series arm=oracle"),
    ("series_oracle_raised", "series arm=oracle empty-segments=raised"),
    ("series_oracle_mixed", "series arm=oracle empty-segments=mixed"),
    ("series_oracle_crossed_the_floor", "series arm=oracle crossed-the-floor"),
    ("configs_fired_slot_pairs", "configs fired by=slot-pairs"),
    ("configs_fired_formula", "configs fired by=formula"),
    ("configs_fired_conjunct", "configs fired by=conjunct-before-any-series"),
    ("slot_pairs_oracle_series_0", "fired by=slot-pairs oracle-series=0"),
    ("slot_pairs_oracle_series_1", "fired by=slot-pairs oracle-series=1"),
    ("slot_pairs_oracle_series_2", "fired by=slot-pairs oracle-series=2"),
    ("slot_pairs_oracle_series_3_or_more", "fired by=slot-pairs oracle-series=3"),
    ("formula_oracle_series_0", "fired by=formula oracle-series=0"),
    ("conjunct_oracle_series_0", "fired by=conjunct-before-any-series oracle-series=0"),
    ("slot_pairs_isolated_before_some", "fired by=slot-pairs isolated-before=some"),
    ("formula_isolated_before_some", "fired by=formula isolated-before=some"),
    ("slot_pairs_oracle_wide_protection_stuck", "fired by=slot-pairs oracle-wide-protection=stuck"),
    ("slot_pairs_oracle_wide_protection_not", "fired by=slot-pairs oracle-wide-protection=not"),
    ("remount_published_slot_pairs_wide_protection_stuck", "remount published by=slot-pairs oracle-wide-protection=stuck"),
    ("remount_published_slot_pairs_wide_protection_not", "remount published by=slot-pairs oracle-wide-protection=not"),
    ("remount_published_slot_pairs_oracle_series_0", "remount published by=slot-pairs oracle-series=0"),
    ("slot_pairs_count_below_floor_yes", "fired by=slot-pairs count-below-floor=yes"),
    ("formula_count_below_floor_yes", "fired by=formula count-below-floor=yes"),
    ("series_product_short_before", "series arm=product short-before"),
    ("series_product_short_before_raised", "series arm=product short-before empty-segments=raised"),
    ("series_oracle_short_before", "series arm=oracle short-before"),
    ("series_oracle_short_before_raised", "series arm=oracle short-before empty-segments=raised"),
    ("conjunct_first_refused", "conjunct-first refused"),
    ("configs_conjunct_first_refused", "configs conjunct-first refused"),
    ("conjunct_first_forced_published", "conjunct-first forced=published"),
    ("conjunct_first_forced_series_0", "conjunct-first forced-series=0"),
    ("series_forced", "series arm=forced"),
    ("series_forced_raised", "series arm=forced empty-segments=raised"),
    ("series_forced_lowered", "series arm=forced empty-segments=lowered"),
    ("series_forced_short_before", "series arm=forced short-before"),
    ("series_forced_short_before_raised", "series arm=forced short-before empty-segments=raised"),
    ("series_forced_crossed_the_floor", "series arm=forced crossed-the-floor"),
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
            "{name} PROBE_RESULT configurations={} {} panics={}",
            self.configurations,
            fields.join(" "),
            self.panics.len()
        );
    }
}
