// ======== m3-w3-r2 辩方腿（Sonnet）在攻方腿的影子模块上又加了：C1b（行记恢复那一刻环里该实例 txg ≤ T 的根的水位最大值）、C2i（每实例最高水位连同第一次到它的代）、
//   C4g（谱系记录多带 (T_rb, 回退前 cur 的水位)，判水位 < W 或 (号, 出生代) 满足号 < W 且出生代 > T_rb）、C2ig = C2i 或 C4g，以及「持久镜像」计数。被攻过零轮。
// ======== m3-w3-r1 攻方腿（Opus）加的部分：W3 各候选的影子记录（只在这份原型里，被攻过零轮）========
// 候选的写规则（我的读法）：
//   C1  实例表行预留字节记「这次恢复所选那一版的水位」M：行 (i, T, W) 第一次出现时 M = 根 (i, T) 那一版的水位（读不到记 0 并计数）。
//       判：最新那条根的水位 < 它的实例表里任一行的 M ⇒ 红。
//   C2b 记账保留「每个实例最后一代」的水位行：影子(R) = 影子(父)，再把 R 的实例那一行换成 R 的水位。判：水位 < 任一行 ⇒ 红。
//   C2a 记账保留「历代」的水位行 (代, 水位)：影子(R) = 影子(父) ∪ {(R 的 txg, R 的水位)}。判：水位 < 任一行，或 inode 树里一条 (号, 出生代 b)
//       的号小于代 < b 的行里的最大水位（那个号在 b 之前已经发出去过）⇒ 红。
//   C2d 峰值连同它第一次到达的代 (峰值, 代)：判：水位 < 峰值，或 inode 树里一条号 < 峰值而出生代 > 那个代 ⇒ 红（攻方腿自己加的读法）。
//   C2c 记账另立一条峰值统计量（取最大值合并，回退那次从 cur 带）：影子(R) = max(影子(父), R 的水位)。判：水位 < 峰值 ⇒ 红。
//   C3  根下只追加的发号记录 (号, 出生代)，回退那次从 cur 带：影子(R) = 影子(父) ∪ R 的 inode 树里每条 (号, 出生代)。
//       判：同号两个出生代，或水位 ≤ 记录里的最大号 ⇒ 红。
//   C4  谱系记录 (T_rb, T_old) 多带回退前 cur 的水位：影子(R) = 影子(父)，R 是回退那次的根时并进 cur 的水位。判：水位 < 最大值 ⇒ 红。
// 父版本：R 之前见过的根里、txg 比 R 小、按最新那条根的实例表不被抛弃的那些里 (txg, 实例) 最大的那条
// （会话里的发布就是 cur，回退那次也是 cur，挂载那次是所选根）。判只读最新那条根的影子与它的实例表（单镜像）。

#[derive(Clone, Debug, Default)]
struct W3Ghost {
    own_watermark: Option<u64>,
    peak: u64,
    peak_generation: u64,
    generation_rows: BTreeMap<u64, u64>,
    per_instance_last: BTreeMap<u32, u64>,
    log: BTreeMap<u64, BTreeSet<u64>>,
    lineage_max: u64,
    /// C2i：每个实例的 (最高水位, 第一次到它的代)。
    peak_by_instance: BTreeMap<u32, (u64, u64)>,
    /// C4g：回退那次根下的 (T_rb, 回退前 cur 的水位)，随父版本带。
    lin: Vec<(u64, u64)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct W3Verdicts {
    judged: bool,
    c1: bool,
    c2a: bool,
    c2b: bool,
    c2c: bool,
    c2d: bool,
    c3: bool,
    c4: bool,
    c1b: bool,
    c2i: bool,
    c4g: bool,
    c2ig: bool,
}

const W3_COUNT: usize = 11;
const W3_NAMES: [&str; W3_COUNT] = ["c1", "c2a", "c2b", "c2c", "c2d", "c3", "c4", "c1b", "c2i", "c4g", "c2ig"];

impl W3Verdicts {
    fn flags(&self) -> [bool; W3_COUNT] {
        [self.c1, self.c2a, self.c2b, self.c2c, self.c2d, self.c3, self.c4, self.c1b, self.c2i, self.c4g, self.c2ig]
    }

    fn render(&self) -> String {
        if !self.judged {
            return "w3=not_judged".to_string();
        }
        W3_NAMES
            .iter()
            .zip(self.flags())
            .map(|(name, red)| format!("w3_{name}={}", u8::from(red)))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug, Default)]
struct W3Account {
    trace_enabled: bool,
    trace: Vec<String>,
    ghosts: BTreeMap<RootIdentity, W3Ghost>,
    row_maximum: BTreeMap<(u32, u64, u64), u64>,
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
    steps_judged: u64,
    steps_without_watermark: u64,
    red_steps: [u64; W3_COUNT],
    q1_events: u64,
    q1_events_red: [u64; W3_COUNT],
    gone_events: u64,
    gone_events_red: [u64; W3_COUNT],
    parent_missing: u64,
    row_root_unknown: u64,
    first_red: [Option<String>; W3_COUNT],
}

fn w3_row_key(row: &singlefs_core::instance_table::InstanceRow) -> (u32, u64, u64) {
    (row.instance.0, row.selected_root_txg.0, row.applied_transaction_high_water)
}

impl W3Account {
    #[allow(clippy::too_many_arguments)]
    fn step(
        &mut self,
        readable: &[RootFacts],
        table: Option<&InstanceTableRecords>,
        rolled_back: bool,
        position: &str,
        operation: &str,
        i9_29: VerdictClass,
        i9_6: VerdictClass,
        events: &mut [ReissueEvent],
    ) -> W3Verdicts {
        let newest_identity = readable.last().map(|facts| facts.identity);
        let new_roots: Vec<RootFacts> = readable
            .iter()
            .filter(|facts| !self.ghosts.contains_key(&facts.identity))
            .cloned()
            .collect();
        for facts in &new_roots {
            let parent: Option<(RootIdentity, W3Ghost)> = self
                .ghosts
                .iter()
                .filter(|(identity, _)| identity.1 < facts.identity.1)
                .filter(|(identity, _)| !table.is_some_and(|table| abandoned_by_the_table(**identity, table)))
                .max_by_key(|(identity, _)| (identity.1, identity.0))
                .map(|(identity, ghost)| (*identity, ghost.clone()));
            let parent_watermark = parent.as_ref().and_then(|(_, ghost)| ghost.own_watermark);
            let mut ghost = match parent {
                Some((_, ghost)) => ghost,
                None => {
                    if !self.ghosts.is_empty() {
                        self.parent_missing += 1;
                    }
                    W3Ghost::default()
                }
            };
            ghost.own_watermark = facts.watermark();
            if let Some(watermark) = facts.watermark() {
                if watermark > ghost.peak {
                    ghost.peak_generation = facts.identity.1;
                }
                ghost.generation_rows.insert(facts.identity.1, watermark);
                ghost.peak = ghost.peak.max(watermark);
                ghost.per_instance_last.insert(facts.identity.0, watermark);
                let entry = ghost.peak_by_instance.entry(facts.identity.0).or_insert((0, 0));
                if watermark > entry.0 {
                    *entry = (watermark, facts.identity.1);
                }
            }
            for &(inode, birth) in facts.inode_births() {
                ghost.log.entry(inode).or_default().insert(birth);
            }
            if rolled_back && Some(facts.identity) == newest_identity {
                if let Some(before) = parent_watermark {
                    ghost.lineage_max = ghost.lineage_max.max(before);
                    ghost.lin.push((facts.identity.1, before));
                }
            }
            self.ghosts.insert(facts.identity, ghost);
        }
        if let Some(table) = table {
            for row in &table.rows {
                let key = w3_row_key(row);
                if !self.row_maximum.contains_key(&key) {
                    let value = self.ghosts.get(&(key.0, key.1)).and_then(|ghost| ghost.own_watermark);
                    if value.is_none() && key.1 != 0 {
                        self.row_root_unknown += 1;
                    }
                    self.row_maximum.insert(key, value.unwrap_or(0));
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
        }
        let newest = readable.last();
        let mut verdicts = W3Verdicts::default();
        if let Some((facts, watermark)) = newest.and_then(|facts| facts.watermark().map(|watermark| (facts, watermark))) {
            let ghost = &self.ghosts[&facts.identity];
            let row_max = table.map_or(0, |table| {
                table.rows.iter().map(|row| self.row_maximum[&w3_row_key(row)]).max().unwrap_or(0)
            });
            let births = facts.inode_births();
            let generation_row_reissue = births.iter().any(|&(inode, birth)| {
                ghost.generation_rows.range(..birth).map(|(_, watermark)| *watermark).max().is_some_and(|before| inode < before)
            });
            let ring_row_red = table.is_some_and(|table| {
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
                c2ig: c2i || c4g,
                c1: row_max > watermark,
                c2a: ghost.generation_rows.values().copied().max().unwrap_or(0) > watermark || generation_row_reissue,
                c2d: ghost.peak > watermark
                    || births.iter().any(|&(inode, birth)| inode < ghost.peak && birth > ghost.peak_generation),
                c2b: ghost.per_instance_last.values().copied().max().unwrap_or(0) > watermark,
                c2c: ghost.peak > watermark,
                c3: ghost.log.values().any(|births| births.len() >= 2)
                    || ghost.log.keys().next_back().is_some_and(|largest| *largest >= watermark),
                c4: ghost.lineage_max > watermark,
            };
        }
        if verdicts.judged {
            self.steps_judged += 1;
            for (index, red) in verdicts.flags().iter().enumerate() {
                if *red {
                    self.red_steps[index] += 1;
                    if self.first_red[index].is_none() {
                        self.first_red[index] = Some(position.to_string());
                    }
                }
            }
        } else {
            self.steps_without_watermark += 1;
        }
        let existing_silent = i9_29 != VerdictClass::Violated && i9_6 != VerdictClass::Violated;
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
            event.w3 = verdicts;
            if event.same_timeline {
                self.q1_events += 1;
                for (index, red) in verdicts.flags().iter().enumerate() {
                    if *red {
                        self.q1_events_red[index] += 1;
                    }
                }
                if event.q9 == OlderBirthStillInTheCandidateSet::Gone {
                    self.gone_events += 1;
                    for (index, red) in verdicts.flags().iter().enumerate() {
                        if *red {
                            self.gone_events_red[index] += 1;
                        }
                    }
                }
            }
        }
        if self.trace_enabled {
            let ghost = newest.and_then(|facts| self.ghosts.get(&facts.identity)).cloned().unwrap_or_default();
            let rows: Vec<String> = table
                .map(|table| {
                    table
                        .rows
                        .iter()
                        .map(|row| {
                            let key = w3_row_key(row);
                            format!("{}:{}:M{}", key.0, key.1, self.row_maximum[&key])
                        })
                        .collect()
                })
                .unwrap_or_default();
            let ring: Vec<String> = readable
                .iter()
                .map(|facts| {
                    format!(
                        "{}:{}:F{}:wm{}",
                        facts.identity.0,
                        facts.identity.1,
                        facts.rollback_floor,
                        facts.watermark().map_or("-".to_string(), |watermark| watermark.to_string())
                    )
                })
                .collect();
            let newest_text = newest.map_or("none".to_string(), |facts| format!("{}:{}", facts.identity.0, facts.identity.1));
            let watermark_text = newest
                .and_then(RootFacts::watermark)
                .map_or("-".to_string(), |watermark| watermark.to_string());
            let births: Vec<String> = newest
                .map(|facts| facts.inode_births().iter().map(|(inode, birth)| format!("{inode}@{birth}")).collect())
                .unwrap_or_default();
            self.trace.push(format!(
                "position={position} op={operation} newest={newest_text} wm={watermark_text} peak={} lineage={} lin={:?} peak_by_instance={:?} per_instance={} rows=[{}] births=[{}] ring=[{}] i9_29={} i9_6={} {}",
                ghost.peak,
                ghost.lineage_max,
                ghost.lin,
                ghost.peak_by_instance,
                format!("{:?}", ghost.per_instance_last).replace(' ', ""),
                rows.join(","),
                births.join(","),
                ring.join(","),
                i9_29.label(),
                i9_6.label(),
                verdicts.render(),
            ));
        }
        verdicts
    }

    fn render(&self) -> String {
        let mut parts = vec![
            format!("w3_hole_events={}", self.hole_events),
            format!("w3_residual_steps={}", self.residual_steps),
            format!("w3_residual_hole_steps={}", self.residual_hole_steps),
            format!("w3_steps_judged={}", self.steps_judged),
            format!("w3_steps_without_watermark={}", self.steps_without_watermark),
            format!("w3_q1_events={}", self.q1_events),
            format!("w3_gone_events={}", self.gone_events),
            format!("w3_parent_missing={}", self.parent_missing),
            format!("w3_row_root_unknown={}", self.row_root_unknown),
        ];
        for (index, name) in W3_NAMES.iter().enumerate() {
            parts.push(format!("w3_red_steps_{name}={}", self.red_steps[index]));
            parts.push(format!("w3_q1_red_{name}={}", self.q1_events_red[index]));
            parts.push(format!("w3_gone_red_{name}={}", self.gone_events_red[index]));
            parts.push(format!("w3_first_red_{name}={}", self.first_red[index].as_deref().unwrap_or("none")));
            parts.push(format!("w3_hole_red_{name}={}", self.hole_events_red[index]));
            parts.push(format!("w3_residual_red_{name}={}", self.residual_red[index]));
            parts.push(format!("w3_residual_hole_red_{name}={}", self.residual_hole_red[index]));
        }
        parts.join(" ")
    }
}
