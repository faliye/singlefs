#!/usr/bin/env python3
"""从攻方腿的 w3_module.rs（影子记录模块）生成辩方腿的 w3r2_module.rs：加 C1b、C2i、C4g（及 C2ig 组合）三种最强形态的影子，
并加「持久镜像」口径的计数（事件之后的镜像里被重发的对象还活着，各候选红不红；与现有 I-9.29 / I-9.6 的并集）。
用法：patch_module.py <w3_module.rs> <输出>"""
import sys
src, out = sys.argv[1], sys.argv[2]
t = open(src, encoding="utf-8").read()
def once(old, new):
    global t
    n = t.count(old)
    if n != 1:
        sys.stderr.write(f"旧串命中 {n} 次：{old[:90]!r}\n"); sys.exit(3)
    t = t.replace(old, new)

once("// ======== m3-w3-r1 攻方腿（Opus）加的部分", "// ======== m3-w3-r2 辩方腿（Sonnet）在攻方腿的影子模块上又加了：C1b（行记恢复那一刻环里该实例 txg ≤ T 的根的水位最大值）、C2i（每实例最高水位连同第一次到它的代）、\n//   C4g（谱系记录多带 (T_rb, 回退前 cur 的水位)，判水位 < W 或 (号, 出生代) 满足号 < W 且出生代 > T_rb）、C2ig = C2i 或 C4g，以及「持久镜像」计数。被攻过零轮。\n// ======== m3-w3-r1 攻方腿（Opus）加的部分")
once("""    lineage_max: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct W3Verdicts {""", """    lineage_max: u64,
    /// C2i：每个实例的 (最高水位, 第一次到它的代)。
    peak_by_instance: BTreeMap<u32, (u64, u64)>,
    /// C4g：回退那次根下的 (T_rb, 回退前 cur 的水位)，随父版本带。
    lin: Vec<(u64, u64)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct W3Verdicts {""")
once("""    c3: bool,
    c4: bool,
}

const W3_COUNT: usize = 7;
const W3_NAMES: [&str; W3_COUNT] = ["c1", "c2a", "c2b", "c2c", "c2d", "c3", "c4"];""", """    c3: bool,
    c4: bool,
    c1b: bool,
    c2i: bool,
    c4g: bool,
    c2ig: bool,
}

const W3_COUNT: usize = 11;
const W3_NAMES: [&str; W3_COUNT] = ["c1", "c2a", "c2b", "c2c", "c2d", "c3", "c4", "c1b", "c2i", "c4g", "c2ig"];""")
once("""        [self.c1, self.c2a, self.c2b, self.c2c, self.c2d, self.c3, self.c4]""",
     """        [self.c1, self.c2a, self.c2b, self.c2c, self.c2d, self.c3, self.c4, self.c1b, self.c2i, self.c4g, self.c2ig]""")
once("""    row_maximum: BTreeMap<(u32, u64, u64), u64>,
    steps_judged: u64,""", """    row_maximum: BTreeMap<(u32, u64, u64), u64>,
    /// C1b：行第一次出现那一刻，根环里读得出的、该实例且 txg ≤ T 的根的水位最大值。
    row_ring: BTreeMap<(u32, u64, u64), u64>,
    /// 事件（同一条时间线上的重发）发出去的 (号, 出生代)：之后的镜像里它还活着，就是「持久镜像」。
    reissued: BTreeSet<(u64, u64)>,
    /// 现有检查（I-9.29 与 I-9.6）在事件那一步都不红的事件数，与各候选在这些事件上红的数。
    hole_events: u64,
    hole_events_red: [u64; W3_COUNT],
    /// 持久镜像：事件之后、被重发的对象还活着的步数；各候选红的步数；其中现有检查都不红的步数与各候选红的步数。
    residual_steps: u64,
    residual_red: [u64; W3_COUNT],
    residual_hole_steps: u64,
    residual_hole_red: [u64; W3_COUNT],
    steps_judged: u64,""")
once("""                ghost.per_instance_last.insert(facts.identity.0, watermark);
            }""", """                ghost.per_instance_last.insert(facts.identity.0, watermark);
                let entry = ghost.peak_by_instance.entry(facts.identity.0).or_insert((0, 0));
                if watermark > entry.0 {
                    *entry = (watermark, facts.identity.1);
                }
            }""")
once("""                if let Some(before) = parent_watermark {
                    ghost.lineage_max = ghost.lineage_max.max(before);
                }""", """                if let Some(before) = parent_watermark {
                    ghost.lineage_max = ghost.lineage_max.max(before);
                    ghost.lin.push((facts.identity.1, before));
                }""")
once("""                    self.row_maximum.insert(key, value.unwrap_or(0));
                }
            }
        }""", """                    self.row_maximum.insert(key, value.unwrap_or(0));
                    let ring_maximum = readable
                        .iter()
                        .filter(|facts| facts.identity.0 == key.0 && facts.identity.1 <= key.1)
                        .filter_map(RootFacts::watermark)
                        .max()
                        .unwrap_or(0)
                        .max(value.unwrap_or(0));
                    self.row_ring.insert(key, ring_maximum);
                }
            }
        }""")
once("""            verdicts = W3Verdicts {
                judged: true,""", """            let ring_row_red = table.is_some_and(|table| {
                table.rows.iter().any(|row| {
                    let key = w3_row_key(row);
                    let m = self.row_ring.get(&key).copied().unwrap_or(0);
                    key.1 > 0 && (m > watermark || births.iter().any(|&(inode, birth)| inode < m && birth > key.1))
                })
            });
            let c2i = ghost.peak_by_instance.values().any(|&(high, generation)| {
                high > watermark || births.iter().any(|&(inode, birth)| inode < high && birth > generation)
            });
            let c4g = ghost.lin.iter().any(|&(t_rb, w)| w > watermark || births.iter().any(|&(inode, birth)| inode < w && birth > t_rb));
            verdicts = W3Verdicts {
                judged: true,
                c1b: ring_row_red,
                c2i,
                c4g,
                c2ig: c2i || c4g,""")
# 计数：事件之后与持久镜像
once("""        for event in events.iter_mut() {
            event.w3 = verdicts;""", """        let existing_silent = i9_29 != VerdictClass::Violated && i9_6 != VerdictClass::Violated;
        let event_now = events.iter().any(|event| event.same_timeline);
        if verdicts.judged && !event_now {
            let alive = newest.is_some_and(|facts| facts.inode_births().iter().any(|pair| self.reissued.contains(pair)));
            if alive {
                self.residual_steps += 1;
                if existing_silent {
                    self.residual_hole_steps += 1;
                }
                for (index, red) in verdicts.flags().iter().enumerate() {
                    if *red {
                        self.residual_red[index] += 1;
                        if existing_silent {
                            self.residual_hole_red[index] += 1;
                        }
                    }
                }
            }
        }
        for event in events.iter() {
            if event.same_timeline {
                self.reissued.insert((event.inode, event.new_birth));
                if existing_silent {
                    self.hole_events += 1;
                    for (index, red) in verdicts.flags().iter().enumerate() {
                        if *red {
                            self.hole_events_red[index] += 1;
                        }
                    }
                }
            }
        }
        for event in events.iter_mut() {
            event.w3 = verdicts;""")
# 逐步打印：加 lin 与 per-instance 峰值
once("""                "position={position} op={operation} newest={newest_text} wm={watermark_text} peak={} lineage={} per_instance={} rows=[{}] births=[{}] ring=[{}] i9_29={} i9_6={} {}",
                ghost.peak,
                ghost.lineage_max,""", """                "position={position} op={operation} newest={newest_text} wm={watermark_text} peak={} lineage={} lin={:?} peak_by_instance={:?} per_instance={} rows=[{}] births=[{}] ring=[{}] i9_29={} i9_6={} {}",
                ghost.peak,
                ghost.lineage_max,
                ghost.lin,
                ghost.peak_by_instance,""")
once("""        let mut parts = vec![
            format!("w3_steps_judged={}", self.steps_judged),""", """        let mut parts = vec![
            format!("w3_hole_events={}", self.hole_events),
            format!("w3_residual_steps={}", self.residual_steps),
            format!("w3_residual_hole_steps={}", self.residual_hole_steps),
            format!("w3_steps_judged={}", self.steps_judged),""")
once("""            parts.push(format!("w3_first_red_{name}={}", self.first_red[index].as_deref().unwrap_or("none")));""",
     """            parts.push(format!("w3_first_red_{name}={}", self.first_red[index].as_deref().unwrap_or("none")));
            parts.push(format!("w3_hole_red_{name}={}", self.hole_events_red[index]));
            parts.push(format!("w3_residual_red_{name}={}", self.residual_red[index]));
            parts.push(format!("w3_residual_hole_red_{name}={}", self.residual_hole_red[index]));""")
open(out, "w", encoding="utf-8").write(t)
