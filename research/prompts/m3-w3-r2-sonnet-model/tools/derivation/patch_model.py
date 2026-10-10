#!/usr/bin/env python3
"""从 m3-w3-r1 攻方腿的 timeline-model/src/main.rs 生成辩方腿的模型：加墓碑与谱系树、最强形态的候选、「现有检查」口径的并集计数。
每处旧串必须恰好命中一次，否则退 3。用法：patch_model.py <原 main.rs> <输出 main.rs>"""
import sys
src, out = sys.argv[1], sys.argv[2]
t = open(src, encoding="utf-8").read()
def once(old, new):
    global t
    n = t.count(old)
    if n != 1:
        sys.stderr.write(f"旧串命中 {n} 次：{old[:90]!r}\n"); sys.exit(3)
    t = t.replace(old, new)

once("//! m3-w3-r1 攻方腿（Opus）的抽象时间线模型。只照条文走，不读 crates/：",
     "//! m3-w3-r2 辩方腿（Sonnet）在 m3-w3-r1 攻方腿的抽象时间线模型上加了：墓碑（跟现行那一版走、不回收）、谱系树 (T_rb, T_old)、\n//! 被第一轮判出局的候选的最强形态（C1b、C2i、C4g 等）、「现有检查」（I-9.29 含谱系上的墓碑、I-9.6 含墓碑那一半）口径的并集计数。\n//! 原文件头：m3-w3-r1 攻方腿（Opus）的抽象时间线模型。只照条文走，不读 crates/：")

# Row: 加 m_ring / m_ring_true
once("""    m_memory: u64,
    m_true: u64,
}""", """    m_memory: u64,
    m_true: u64,
    /// C1b：写这一行那一刻，根环里读得出的、该实例且 txg ≤ T 的根的水位最大值（Sonnet 正推的 C1b）。
    m_ring: u64,
    m_ring_true: u64,
}""")
# Records: 加字段
once("""    /// C2ay：谱系记录多带的 (cur 的 txg, 回退前水位)。谱系树随现行那一版，所以只在 true_records 里带；值按变异写（攻方腿提的改法，零轮）。
    lineage_entries: Vec<(u64, u64)>,
}""", """    /// C2ay：谱系记录多带的 (cur 的 txg, 回退前水位)。谱系树随现行那一版，所以只在 true_records 里带；值按变异写（攻方腿提的改法，零轮）。
    lineage_entries: Vec<(u64, u64)>,
    /// C2i：每个实例的 (实例, 最高水位, 第一次到它的代)。
    peak_by_instance: Vec<(u32, u64, u64)>,
    /// C4g：谱系记录多带 (T_rb, 回退前 cur 的水位)，判时 A 半句水位 < W、B 半句 (号, 出生代) 满足号 < W 且出生代 > T_rb。随现行那一版（谱系树的载体），回退那次追加一条。
    lin: Vec<(u64, u64)>,
    /// C1bk：清扫删掉的行也留着的 (实例, T, m_ring)，不是盘上的东西（真相侧）。
    rows_ever_ring: Vec<(u32, u64, u64)>,
}""")
# Version: 加 tombs / lineage_tree
once("""    /// 真相：这条时间线上发过的 (号, 出生代)。
    issued: Vec<(u64, u64)>,
}""", """    /// 真相：这条时间线上发过的 (号, 出生代)。
    issued: Vec<(u64, u64)>,
    /// 墓碑树：(号, 出生代, 死亡 txg)。随现行那一版、第一版不回收（D18 已定项 24 第 4、5 条）。
    tombs: Vec<(u64, u64, u64)>,
    /// 谱系树：(T_rb, T_old)，随现行那一版、只增不删（D18 已定项 24 第 5 条）。
    lineage_tree: Vec<(u64, u64)>,
}""")
# extend_records: peak_by_instance
once("""    if watermark > records.peak {
        records.peak = watermark;
        records.peak_pair = (watermark, txg);
    }
    records.log = union_sorted(&records.log, tree);
    records
}""", """    if watermark > records.peak {
        records.peak = watermark;
        records.peak_pair = (watermark, txg);
    }
    match records.peak_by_instance.iter_mut().find(|(i, _, _)| *i == instance) {
        Some(entry) => {
            if watermark > entry.1 {
                entry.1 = watermark;
                entry.2 = txg;
            }
        }
        None => records.peak_by_instance.push((instance, watermark, txg)),
    }
    records.log = union_sorted(&records.log, tree);
    records
}""")
once("""            rows_ever: Vec::new(),
            lineage_entries: Vec::new(),
        },
        Mutation::HistoryNotWritten => Records::default(),""", """            rows_ever: Vec::new(),
            lineage_entries: Vec::new(),
            peak_by_instance: vec![(instance, watermark, txg)],
            lin: Vec::new(),
            rows_ever_ring: Vec::new(),
        },
        Mutation::HistoryNotWritten => Records::default(),""")
# initial
once("""            true_records: records,
            issued: tree,
        };""", """            true_records: records,
            issued: tree,
            tombs: Vec::new(),
            lineage_tree: Vec::new(),
        };""")
# next_version: carry
once("""        &self,
        config: &Config,
        instance: u32,
        txg: u64,
        watermark: u64,
        tree: Vec<(u64, u64)>,
        floor: u64,
        table: Vec<Row>,
        record_base: &Records,
        true_base: &Records,
        issued: Vec<(u64, u64)>,
    ) -> Version {""", """        &self,
        config: &Config,
        carry: &Version,
        instance: u32,
        txg: u64,
        watermark: u64,
        tree: Vec<(u64, u64)>,
        floor: u64,
        table: Vec<Row>,
        record_base: &Records,
        true_base: &Records,
        issued: Vec<(u64, u64)>,
    ) -> Version {""")
once("""            let key = (row.instance, row.selected_txg, row.m_true);
            if !true_records.rows_ever.contains(&key) {
                true_records.rows_ever.push(key);
            }
        }
        Version { instance, txg, watermark, tree, floor, table, records, true_records, issued }""",
"""            let key = (row.instance, row.selected_txg, row.m_true);
            if !true_records.rows_ever.contains(&key) {
                true_records.rows_ever.push(key);
            }
            let key_ring = (row.instance, row.selected_txg, row.m_ring_true);
            if !true_records.rows_ever_ring.contains(&key_ring) {
                true_records.rows_ever_ring.push(key_ring);
            }
        }
        Version {
            instance, txg, watermark, tree, floor, table, records, true_records, issued,
            tombs: carry.tombs.clone(),
            lineage_tree: carry.lineage_tree.clone(),
        }""")
# 各调用点
once("config, instance, txg, watermark, tree, base.floor, table, &base.records, &base.true_records, issued,",
     "config, base, instance, txg, watermark, tree, base.floor, table, &base.records, &base.true_records, issued,")
once("config, base.instance, txg, base.watermark, base.tree.clone(), base.floor, kept, &base.records,",
     "config, &base, base.instance, txg, base.watermark, base.tree.clone(), base.floor, kept, &base.records,")
once("config, new_instance, txg, selected.watermark, selected.tree.clone(), selected.floor, table.clone(),",
     "config, &selected, new_instance, txg, selected.watermark, selected.tree.clone(), selected.floor, table.clone(),")
once("config, current.instance, next_txg, current.watermark, tree, current.floor, current.table.clone(),",
     "config, &current, current.instance, next_txg, current.watermark, tree, current.floor, current.table.clone(),")
once("config, current.instance, next_txg, watermark, target.tree.clone(), current.floor,",
     "config, &current, current.instance, next_txg, watermark, target.tree.clone(), current.floor,")
once("config, current.instance, next_txg, current.watermark, current.tree.clone(), current.txg,",
     "config, &current, current.instance, next_txg, current.watermark, current.tree.clone(), current.txg,")
once("config, current.instance, next_txg, current.watermark, current.tree.clone(), current.floor,",
     "config, &current, current.instance, next_txg, current.watermark, current.tree.clone(), current.floor,")
once("config, new_instance, next_txg, selected.watermark, selected.tree.clone(), selected.floor, table,",
     "config, &selected, new_instance, next_txg, selected.watermark, selected.tree.clone(), selected.floor, table,")
# Delete: 墓碑
once("""                let tree: Vec<(u64, u64)> = current.tree.iter().copied().filter(|pair| *pair != victim).collect();
                let version = self.next_version(""", """                let tree: Vec<(u64, u64)> = current.tree.iter().copied().filter(|pair| *pair != victim).collect();
                let mut version = self.next_version(""")
once("""                    &current.records, &current.true_records, current.issued.clone(),
                );
                self.publish(version);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::Rollback(k) => {""", """                    &current.records, &current.true_records, current.issued.clone(),
                );
                version.tombs.push((victim.0, victim.1, next_txg));
                self.publish(version);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::Rollback(k) => {""")
# Rollback: 谱系树与 lin
once("""                if config.mutation != Mutation::HistoryNotWritten {
                    version.records.lineage_max = current.records.lineage_max.max(watermark_before);
                }""", """                if config.mutation != Mutation::HistoryNotWritten {
                    version.records.lineage_max = current.records.lineage_max.max(watermark_before);
                    version.records.lin.push((next_txg, watermark_before));
                }
                version.lineage_tree.push((next_txg, target.txg));
                version.true_records.lin.push((next_txg, current.watermark));""")
# 行：m_ring
once("""        let mut table: Vec<Row> = selected.table.clone();
        for instance in selected.instance.max(1)..new_instance {
            table.retain(|row| row.instance != instance);
            let row = if instance == selected.instance {
                Row {
                    instance,
                    selected_txg: selected.txg,
                    m_disk: if config.mutation == Mutation::HistoryNotWritten { 0 } else { selected.watermark },
                    m_memory: memory_watermark,
                    m_true: selected.watermark,
                }
            } else {
                Row { instance, selected_txg: 0, m_disk: 0, m_memory: 0, m_true: 0 }
            };""", """        let mut table: Vec<Row> = selected.table.clone();
        let ring_maximum = self
            .readable_versions()
            .iter()
            .filter(|version| version.instance == selected.instance && version.txg <= selected.txg)
            .map(|version| version.watermark)
            .max()
            .unwrap_or(0)
            .max(selected.watermark);
        for instance in selected.instance.max(1)..new_instance {
            table.retain(|row| row.instance != instance);
            let row = if instance == selected.instance {
                Row {
                    instance,
                    selected_txg: selected.txg,
                    m_disk: if config.mutation == Mutation::HistoryNotWritten { 0 } else { selected.watermark },
                    m_memory: memory_watermark,
                    m_true: selected.watermark,
                    m_ring: if config.mutation == Mutation::HistoryNotWritten { 0 } else { ring_maximum },
                    m_ring_true: ring_maximum,
                }
            } else {
                Row { instance, selected_txg: 0, m_disk: 0, m_memory: 0, m_true: 0, m_ring: 0, m_ring_true: 0 }
            };""")

# 候选表
once("""const N: usize = 21;
const CANDIDATES: [&str; N] = [
    "I-9.29", "I-9.6", "C1", "C1m", "C1s", "C2a", "C2as", "C2b", "C2c", "C2cs", "C2d", "C3r", "C3rs", "C3f", "C3fe", "C3fb", "C4",
    "C1k", "C2ax", "C3rx", "C2ay",
];""", """const N: usize = 25;
const CANDIDATES: [&str; N] = [
    "I-9.29", "I-9.6", "I-9.29t", "I-9.6t", "C1", "C1b", "C1bk", "C1k", "C2a", "C2b", "C2c", "C2d", "C2i", "C3r", "C4", "C4g",
    "C2ax", "C3rx", "C2ay", "C2dg", "C3rg", "C2ag", "C2ig", "C1bg", "C4gk",
];
const IDX_I929: usize = 0;
const IDX_I929T: usize = 2;
const IDX_I96T: usize = 3;
const IDX_C1: usize = 4;
const IDX_C1K: usize = 7;""")

# 谱系链
once("""fn two_births(pairs: &[(u64, u64)]) -> bool {""", """/// 死亡 txg `death` 在不在这一版的谱系父链上：从这一版的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg 的父是 T_old，其余是 txg − 1（I-9.18 的定义）。
fn on_the_lineage(version: &Version, death: u64) -> bool {
    let mut txg = version.txg;
    loop {
        if txg == death {
            return true;
        }
        if txg < death || txg <= 1 {
            return false;
        }
        txg = match version.lineage_tree.iter().find(|(rollback, _)| *rollback == txg) {
            Some((_, old)) => *old,
            None => txg - 1,
        };
    }
}

fn pairs_with_tombstones_on_the_lineage(version: &Version) -> Vec<(u64, u64)> {
    let mut out = version.tree.clone();
    out.extend(version.tombs.iter().filter(|(_, _, death)| on_the_lineage(version, *death)).map(|(n, b, _)| (*n, *b)));
    out
}

fn two_births(pairs: &[(u64, u64)]) -> bool {""")

# verdicts 整体替换
start = t.index("/// 单镜像上各候选的判定（true = 红）。")
end = t.index("/// 每个候选的记录这一格被变异改过没有")
new_verdicts = r'''/// 单镜像上各候选的判定（true = 红）。只读：最新那条根、它的实例表与它下面的记录、环里读得出的根。
fn verdicts(state: &State) -> [bool; N] {
    let newest = state.newest_readable();
    let records = &newest.records;
    let watermark = newest.watermark;
    // 现在的 I-9.29 / I-9.6（Opus 口径，只看树）：候选集（按最新那条根的实例表滤、txg ≥ F）各版的 inode 树合起来。
    let mut candidate_pairs: Vec<(u64, u64)> = newest.tree.clone();
    let mut candidate_pairs_t: Vec<(u64, u64)> = pairs_with_tombstones_on_the_lineage(newest);
    for version in state.readable_versions() {
        if !abandoned_by(&newest.table, version.instance, version.txg) && version.txg >= newest.floor {
            candidate_pairs.extend(version.tree.iter().copied());
            candidate_pairs_t.extend(pairs_with_tombstones_on_the_lineage(version));
        }
    }
    candidate_pairs.sort_unstable();
    candidate_pairs.dedup();
    candidate_pairs_t.sort_unstable();
    candidate_pairs_t.dedup();
    let i9_29 = two_births(&candidate_pairs);
    let i9_6 = candidate_pairs.iter().any(|(inode, _)| *inode >= watermark);
    let i9_29t = two_births(&candidate_pairs_t);
    let i9_6t = i9_6 || newest.tombs.iter().any(|(inode, _, _)| *inode >= watermark);
    let c1 = newest.table.iter().any(|row| row.m_disk > watermark);
    let readable = state.readable_versions();
    // C1b：A 半句水位 < M_ring；B 半句 (号, 出生代) 满足号 < M_ring 且出生代 > T（Sonnet 正推的判法 A、B）。
    let c1b_row = |instance_row: &Row, m: u64| -> bool {
        instance_row.selected_txg > 0
            && (m > watermark || newest.tree.iter().any(|&(inode, birth)| inode < m && birth > instance_row.selected_txg))
    };
    let c1b = newest.table.iter().any(|row| c1b_row(row, row.m_ring));
    let c1bk = c1b
        || newest.true_records.rows_ever_ring.iter().any(|&(_, t_pub, m)| {
            t_pub > 0 && (m > watermark || newest.tree.iter().any(|&(inode, birth)| inode < m && birth > t_pub))
        });
    let rows_max = records.generation_rows.iter().map(|(_, value)| *value).max().unwrap_or(0);
    let generation_reissue = newest.tree.iter().any(|&(inode, birth)| {
        records.generation_rows.iter().filter(|(generation, _)| *generation < birth).map(|(_, value)| *value).max().is_some_and(|before| inode < before)
    });
    let c2a = rows_max > watermark || generation_reissue;
    let c2b = records.per_instance_last.iter().any(|(_, value)| *value > watermark);
    let c2c = records.peak > watermark;
    let (peak, peak_generation) = records.peak_pair;
    let c2d = peak > watermark || newest.tree.iter().any(|&(inode, birth)| inode < peak && birth > peak_generation);
    let c2i = records.peak_by_instance.iter().any(|&(_, high, generation)| {
        high > watermark || newest.tree.iter().any(|&(inode, birth)| inode < high && birth > generation)
    });
    let c3r = two_births(&records.log) || records.log.iter().any(|(inode, _)| *inode >= watermark);
    let c4 = records.lineage_max > watermark;
    let c4g = records.lin.iter().any(|&(t_rb, w)| {
        w > watermark || newest.tree.iter().any(|&(inode, birth)| inode < w && birth > t_rb)
    });
    // C1k：清扫删掉的行也留着（看「行回收丢了 M」那一格用）。
    let c1k = newest.true_records.rows_ever.iter().any(|(_, _, m)| *m > watermark);
    // C2ax / C3rx：再拿环里读得出、按实例表不被抛弃的每一条根交叉核。
    let on_the_timeline: Vec<&Version> =
        readable.iter().copied().filter(|version| !abandoned_by(&newest.table, version.instance, version.txg)).collect();
    let c2as = c2a || !records.generation_rows.contains(&(newest.txg, watermark));
    let c3rs = c3r || newest.tree.iter().any(|pair| !records.log.contains(pair));
    let c2ax = c2as || on_the_timeline.iter().any(|version| !records.generation_rows.contains(&(version.txg, version.watermark)));
    let c3rx = c3rs || on_the_timeline.iter().any(|version| version.tree.iter().any(|pair| !records.log.contains(pair)));
    let c2ay = c2ax || newest.true_records.lineage_entries.iter().any(|entry| !records.generation_rows.contains(entry));
    // 组合：两处各自存的历史，红一处即红（OR）。
    let c2dg = c2d || c4g;
    let c3rg = c3r || c4g;
    let c2ag = c2a || c4g;
    let c2ig = c2i || c4g;
    let c1bg = c1b || c4g;
    let c4gk = c4g || c1bk;
    [
        i9_29, i9_6, i9_29t, i9_6t, c1, c1b, c1bk, c1k, c2a, c2b, c2c, c2d, c2i, c3r, c4, c4g, c2ax, c3rx, c2ay, c2dg, c3rg, c2ag, c2ig,
        c1bg, c4gk,
    ]
}

'''
t = t[:start] + new_verdicts + t[end:]

# mutated_components 替换
start = t.index("/// 每个候选的记录这一格被变异改过没有")
end = t.index("/// 真相：这条时间线上有没有同号两个出生代")
new_mut = r'''/// 每个候选的记录这一格被变异改过没有（与正确写法比）。
fn mutated_components(state: &State) -> [bool; N] {
    let newest = state.newest_readable();
    let (records, truth) = (&newest.records, &newest.true_records);
    let rows = newest.table.iter().any(|row| row.m_disk != row.m_true);
    let rows_ring = newest.table.iter().any(|row| row.m_ring != row.m_ring_true);
    let generation = records.generation_rows != truth.generation_rows;
    let per_instance = records.per_instance_last != truth.per_instance_last;
    let peak = records.peak != truth.peak;
    let pair = records.peak_pair != truth.peak_pair;
    let by_instance = records.peak_by_instance != truth.peak_by_instance;
    let log = records.log != truth.log;
    let lineage = records.lineage_max != truth.lineage_max;
    let lin = records.lin != truth.lin;
    [
        false, false, false, false, rows, rows_ring, rows_ring, rows, generation, per_instance, peak, pair, by_instance, log, lineage,
        lin, generation, log, generation, pair || lin, log || lin, generation || lin, by_instance || lin, rows_ring || lin,
        lin || rows_ring,
    ]
}

'''
t = t[:start] + new_mut + t[end:]

# KINDS
once("""const KINDS: [&str; 6] = [""", """const KINDS: [&str; 8] = [""")
once("""    "event_missed_only_because_the_sweep_deleted_the_row",
];""", """    "event_missed_only_because_the_sweep_deleted_the_row",
    "event_missed_where_existing_checks_silent",
    "residual_missed_where_existing_checks_silent",
];""")
once("""    events_where_i9_29_holds: u64,
    residual_images: u64,""", """    events_where_i9_29_holds: u64,
    events_where_existing_silent: u64,
    residual_images: u64,
    residual_where_existing_silent: u64,""")
once("""        if event {
            self.events += 1;
            if !red[0] {
                self.events_where_i9_29_holds += 1;
            }
        }""", """        let existing_silent = !red[IDX_I929T] && !red[IDX_I96T];
        if event {
            self.events += 1;
            if !red[IDX_I929] {
                self.events_where_i9_29_holds += 1;
            }
            if existing_silent {
                self.events_where_existing_silent += 1;
            }
        }""")
once("""        if residual {
            self.residual_images += 1;
        }""", """        if residual {
            self.residual_images += 1;
            if existing_silent {
                self.residual_where_existing_silent += 1;
            }
        }""")
once("""            if event && !red[candidate] {
                self.hit(candidate, 1, sequence);
                if !red[0] {
                    self.hit(candidate, 2, sequence);
                }
            }
            if residual && !red[candidate] {
                self.hit(candidate, 3, sequence);
            }
            if event && !red[candidate] && CANDIDATES[candidate] == "C1" && red[17] {
                self.hit(candidate, 5, sequence);
            }""", """            if event && !red[candidate] {
                self.hit(candidate, 1, sequence);
                if !red[IDX_I929] {
                    self.hit(candidate, 2, sequence);
                }
                if existing_silent {
                    self.hit(candidate, 6, sequence);
                }
            }
            if residual && !red[candidate] {
                self.hit(candidate, 3, sequence);
                if existing_silent {
                    self.hit(candidate, 7, sequence);
                }
            }
            if event && !red[candidate] && candidate == IDX_C1 && red[IDX_C1K] {
                self.hit(candidate, 5, sequence);
            }""")
once("""    counts: [[u64; 6]; N],""", """    counts: [[u64; 8]; N],""")
once("""events_where_I-9.29_holds={} residual_images={} reissues_across_abandoned_roots={}",
        tally.images, tally.legal_images, tally.events, tally.events_where_i9_29_holds, tally.residual_images,
        tally.reissues_across_abandoned_roots""", """events_where_I-9.29_holds={} events_where_existing_checks_silent={} residual_images={} residual_where_existing_checks_silent={} reissues_across_abandoned_roots={}",
        tally.images, tally.legal_images, tally.events, tally.events_where_i9_29_holds, tally.events_where_existing_silent,
        tally.residual_images, tally.residual_where_existing_silent, tally.reissues_across_abandoned_roots""")
# render_version: 加 lin
once("""peak={} peak_pair={}@{} lineage={} c3r_log=[{}]",""", """peak={} peak_pair={}@{} lineage={} lin=[{}] tombs=[{}] c3r_log=[{}]",""")
once("""        version.records.lineage_max,
        log.join(","),
    )""", """        version.records.lineage_max,
        version.records.lin.iter().map(|(t, w)| format!("{t}:{w}")).collect::<Vec<_>>().join(","),
        version.tombs.iter().map(|(n, b, d)| format!("{n}@{b}d{d}")).collect::<Vec<_>>().join(","),
        log.join(","),
    )""")
# 配置：main 组加一个环 6 的 V2 与 V1（保持原有），再加辩方自己的组
open(out, "w", encoding="utf-8").write(t)
