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
            if k.starts_with("images") {
                continue;
            }
            println!("{tag}-FIRST {k} :: {}", v.chars().take(700).collect::<String>());
        }
    }
}

/// 一张镜像：新 checker 的判定（已有的就传进来）、探针与 pre36 各跑一遍、core 那一侧算一遍，逐项记。
pub fn note_image(
    tally: &mut Tally,
    label: &str,
    reader: &(impl singlefs_checker::image::ImageReader + singlefs_core::recovery::PoolReader),
    new_given: Option<Verdicts>,
) {
    let new = new_given.unwrap_or_else(|| judge::new(reader));
    let (probe_verdicts, probe) = judge::probe(reader);
    let (pre, pre_probe) = judge::pre36(reader);
    let core = crate::core::core_view(reader);
    tally.add("images", label, String::new);
    if probe_verdicts != new {
        tally.add("SANITY probe!=new", label, || format!("{:?}", judge::differences(&new, &probe_verdicts)));
    }
    for name in judge::reds(&new) {
        tally.add(format!("new-red {name}"), label, || new[name].1.chars().take(400).collect());
    }
    for name in judge::reds(&pre) {
        tally.add(format!("pre36-red {name}"), label, || pre[name].1.chars().take(400).collect());
    }
    for (name, n, o) in judge::differences(&new, &pre) {
        tally.add(format!("new!=pre36 {name} {}>{}", o.0, n.0), label, || {
            format!("pre36=[{}] new=[{}]", o.1.chars().take(300).collect::<String>(), n.1.chars().take(300).collect::<String>())
        });
    }
    tally.add(format!("new replay-kind: {}", probe.replay_kind), label, String::new);
    if let Some((instance, txg, _)) = &probe.next_mount {
        let gap = txg.wrapping_sub(probe.newest.1);
        tally.add(
            format!("new next-mount: same-instance-as-newest={} txg-minus-newest={gap}", *instance == probe.newest.0),
            label,
            || format!("newest={:?} next=({instance}, {txg})", probe.newest),
        );
    }
    if std::env::var("B36_DEBUG").as_deref() == Ok("1")
        && probe.next_mount.as_ref().map(|v| (v.0, v.1)) != pre_probe.next_mount.as_ref().map(|v| (v.0, v.1))
    {
        println!(
            "B36-DEBUG {label} newest={:?} new={:?} pre36={:?} replay={} core={:?} diffs={:?}",
            probe.newest,
            probe.next_mount.as_ref().map(|v| (v.0, v.1)),
            pre_probe.next_mount.as_ref().map(|v| (v.0, v.1)),
            probe.replay_kind,
            core.as_ref().map(|c| (c.chosen, c.first.as_ref().map(|a| a.0), c.prefix_applied, c.verification_failed, c.refused.clone())),
            judge::differences(&new, &pre).iter().map(|d| (d.0, d.2 .0, d.1 .0)).collect::<Vec<_>>()
        );
    }
    note_versions(tally, label, "new", &probe, &core);
    note_versions(tally, label, "pre36", &pre_probe, &core);
    if let Some(core) = &core {
        if core.verification_failed > 0 {
            tally.add(format!("core verification_failed>0 first={}", core.first.is_some()), label, || format!("{core:?}").chars().take(300).collect());
        }
        if core.first.as_ref().map(|f| f.0) != core.last.as_ref().map(|l| l.0) {
            tally.add("core applied more than one publish", label, || format!("first={:?} last={:?}", core.first.as_ref().map(|f| f.0), core.last.as_ref().map(|l| l.0)));
        }
    }
}

fn note_versions(tally: &mut Tally, label: &str, which: &str, probe: &ProbeInfo, core: &Option<CoreView>) {
    let nm = probe.next_mount.as_ref().map(|(i, t, s)| (*i, *t, s.as_slice()));
    let class = classify(nm, core.as_ref().and_then(|c| c.first.as_ref()), core);
    tally.add(format!("{which} next-mount vs core-first: {class}"), label, || {
        format!(
            "checker={:?} core={:?}",
            probe.next_mount.as_ref().map(|v| (v.0, v.1)),
            core.as_ref().map(|c| (c.chosen, c.first.as_ref().map(|a| a.0), c.last.as_ref().map(|a| a.0), c.prefix_applied, c.verification_failed, c.refused.clone()))
        )
    });
    let rp = probe.replayed.as_ref().map(|(i, t, s, _)| (*i, *t, s.as_slice()));
    let class = classify(rp, core.as_ref().and_then(|c| c.last.as_ref()), core);
    tally.add(format!("{which} I-4.1 replayed vs core-last: {class}"), label, || {
        format!("checker={:?} core={:?}", probe.replayed.as_ref().map(|v| (v.0, v.1, v.3)), core.as_ref().map(|c| (c.chosen, c.last.as_ref().map(|a| a.0), c.prefix_applied)))
    });
}
