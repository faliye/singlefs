//! 计数：每张判过的镜像记一笔；键是一句短的分类，第一处的原文留下。
use std::collections::BTreeMap;

use crate::core::{classify, CoreView};
use crate::judge::{self, ProbeInfo, Verdicts};

#[derive(Default)]
pub struct Tally {
    pub counts: BTreeMap<String, u64>,
    pub first: BTreeMap<String, String>,
}

impl Tally {
    pub fn add(&mut self, key: impl Into<String>, label: &str, detail: impl FnOnce() -> String) {
        let key = key.into();
        *self.counts.entry(key.clone()).or_default() += 1;
        if !self.first.contains_key(&key) {
            self.first.insert(key, format!("{label} :: {}", detail()));
        }
    }
    pub fn absorb(&mut self, other: Tally) {
        for (k, v) in other.counts {
            *self.counts.entry(k).or_default() += v;
        }
        for (k, v) in other.first {
            self.first.entry(k).or_insert(v);
        }
    }
    pub fn print(&self, tag: &str) {
        for (k, v) in &self.counts {
            println!("{tag} {k} = {v}");
        }
        for (k, v) in &self.first {
            if k.starts_with("images") || k.starts_with("ok ") {
                continue;
            }
            println!("{tag}-FIRST {k} :: {}", v.chars().take(700).collect::<String>());
        }
    }
}

/// 由记录施加出来的那几版，core 当初施加出来的新根段（历史里每次挂载之前照恢复算的，键 (实例, txg)）。
pub type Ledger = BTreeMap<(u32, u64), Vec<u8>>;

/// 一张镜像：新 checker 的判定（已有的就传进来）、探针与 pre35 各跑一遍、core 那一侧算一遍，逐项记。
pub fn note_image(
    tally: &mut Tally,
    label: &str,
    reader: &(impl singlefs_checker::image::ImageReader + singlefs_core::recovery::PoolReader),
    new_given: Option<Verdicts>,
    ledger: &Ledger,
) {
    let new = new_given.unwrap_or_else(|| judge::new(reader));
    let (probe_verdicts, probe) = judge::probe(reader);
    let (pre, pre_probe) = judge::pre35(reader);
    let core = crate::core::core_view(reader);
    tally.add("images", label, String::new);
    if probe_verdicts != new {
        tally.add("SANITY probe!=new", label, || format!("{:?}", judge::differences(&new, &probe_verdicts)));
    }
    for name in judge::reds(&new) {
        tally.add(format!("new-red {name}"), label, || new[name].1.chars().take(400).collect());
    }
    for (name, n, o) in judge::differences(&new, &pre) {
        tally.add(format!("new!=pre35 {name} {}>{}", o.0, n.0), label, || {
            format!("pre35=[{}] new=[{}]", o.1.chars().take(300).collect::<String>(), n.1.chars().take(300).collect::<String>())
        });
    }
    note_versions(tally, label, "new", &probe, &core, ledger);
    note_versions(tally, label, "pre35", &pre_probe, &core, ledger);
    let zeros = judge::all_zero_tree_table_pointers(reader);
    if zeros != (0, 0) {
        tally.add(format!("all-zero roots={} records={}", zeros.0, zeros.1), label, String::new);
    }
    if let Some((_, reason)) = new.get("I-5.5") {
        tally.add(format!("I-5.5 {}:{}", new["I-5.5"].0, reason.chars().take(20).collect::<String>()), label, String::new);
    }
}

fn note_versions(tally: &mut Tally, label: &str, which: &str, probe: &ProbeInfo, core: &Option<CoreView>, ledger: &Ledger) {
    let nm = probe.next_mount.as_ref().map(|(i, t, s)| (*i, *t, s.as_slice()));
    let class = classify(nm, core);
    tally.add(format!("{which} next-mount vs core: {class}"), label, || {
        format!("checker={:?} core={:?}", probe.next_mount.as_ref().map(|v| (v.0, v.1)), core.as_ref().map(|c| (c.chosen, c.applied.as_ref().map(|a| (a.0, a.2)), c.refused.clone())))
    });
    let rp = probe.replayed.as_ref().map(|(i, t, s, _)| (*i, *t, s.as_slice()));
    let class = classify(rp, core);
    tally.add(format!("{which} I-4.1 replayed vs core: {class}"), label, || {
        format!("checker={:?} core={:?}", probe.replayed.as_ref().map(|v| (v.0, v.1, v.3)), core.as_ref().map(|c| (c.chosen, c.applied.as_ref().map(|a| (a.0, a.2)), c.refused.clone())))
    });
    for (instance, txg, segment) in &probe.versions {
        let class = match ledger.get(&(*instance, *txg)) {
            None => "unledgered",
            Some(core_segment) if core_segment == segment => "same-bytes-as-core-applied",
            Some(_) => "DIFFERENT-bytes-from-core-applied",
        };
        tally.add(format!("{which} version-applied-only-by-records: {class}"), label, || format!("({instance}, {txg})"));
    }
    for ((instance, txg), from_last) in &probe.from_last {
        if !from_last {
            tally.add(format!("{which} version taken from a record without the last flag (退路)"), label, || format!("({instance}, {txg})"));
        }
    }
}
