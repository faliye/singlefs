//! 删除线 checker 设计轮第二轮攻方腿（Opus）原型：九格与两问的各候选读法，在合法历史（用户决定的几步放开扫）与改坏的镜像上逐版判。
//! 只跑 harness 档：不起 checker 档的测试目标与 bin，只把 singlefs-checker-tier 当库调（层 0 枚举函数）。副本专用（walk.rs 打了 attack_stash 补丁）。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;

mod cand;
mod driver;
mod plans;
mod crashes;
mod bad;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::{attack_stash, check_pool_image};

pub fn threads() -> usize {
    std::env::var("M3DCR2_THREADS").ok().and_then(|n| n.parse().ok()).unwrap_or(1)
}

pub struct Judged {
    pub checker_reds: Vec<(&'static str, String)>,
    pub candidates: Vec<(&'static str, cand::V)>,
    pub versions: usize,
}

pub fn judge(image: &dyn ImageReader) -> Judged {
    let report = check_pool_image(image);
    let checker_reds = report
        .iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*name, detail.chars().take(200).collect())),
            _ => None,
        })
        .collect();
    let facts = attack_stash::take();
    match facts {
        Some(facts) => Judged { checker_reds, candidates: cand::evaluate(&facts), versions: facts.versions.len() },
        None => Judged { checker_reds, candidates: Vec::new(), versions: 0 },
    }
}

/// 计数：候选 → [成立, 红, 不适用]，每格第一个红的例子。
#[derive(Default)]
pub struct Tally {
    pub counts: BTreeMap<String, [u64; 3]>,
    pub first_red: BTreeMap<String, String>,
    pub checker_reds: BTreeMap<String, u64>,
    pub first_checker_red: BTreeMap<String, String>,
    pub states: u64,
    pub lines: Vec<String>,
}

impl Tally {
    pub fn note(&mut self, place: &str, judged: &Judged, context: &str) {
        self.states += 1;
        let mut reds = Vec::new();
        for (name, verdict) in &judged.candidates {
            let key = format!("{place}\t{name}");
            let slot = self.counts.entry(key.clone()).or_default();
            match verdict {
                cand::V::Holds => slot[0] += 1,
                cand::V::Red(detail) => {
                    slot[1] += 1;
                    reds.push(*name);
                    self.first_red.entry(key).or_insert_with(|| format!("{context} :: {detail}"));
                }
                cand::V::Na => slot[2] += 1,
            }
        }
        for (name, detail) in &judged.checker_reds {
            let key = format!("{place}\t{name}");
            *self.checker_reds.entry(key.clone()).or_default() += 1;
            self.first_checker_red.entry(key).or_insert_with(|| format!("{context} :: {detail}"));
        }
        if place == "step" {
            let checker: Vec<&str> = judged.checker_reds.iter().map(|(name, _)| *name).collect();
            self.lines.push(format!("STEPLINE {context} versions={} checker_red={checker:?} cand_red={reds:?}", judged.versions));
        }
    }
    pub fn absorb(&mut self, other: Tally) {
        for (key, value) in other.counts {
            let slot = self.counts.entry(key).or_default();
            for index in 0..3 {
                slot[index] += value[index];
            }
        }
        for (key, value) in other.first_red {
            self.first_red.entry(key).or_insert(value);
        }
        for (key, value) in other.checker_reds {
            *self.checker_reds.entry(key).or_default() += value;
        }
        for (key, value) in other.first_checker_red {
            self.first_checker_red.entry(key).or_insert(value);
        }
        self.states += other.states;
        self.lines.extend(other.lines);
    }
    pub fn print(&self, tag: &str, print_lines: bool) {
        println!("SUMMARY {tag} states={}", self.states);
        for (key, [holds, red, na]) in &self.counts {
            println!("COUNT {tag}\t{key}\tholds={holds}\tred={red}\tna={na}");
        }
        for (key, example) in &self.first_red {
            println!("FIRSTRED {tag}\t{key}\t{example}");
        }
        for (key, count) in &self.checker_reds {
            println!("CHECKERRED {tag}\t{key}\t{count}\t{}", self.first_checker_red.get(key).cloned().unwrap_or_default());
        }
        if print_lines {
            for line in &self.lines {
                println!("{line}");
            }
        }
    }
}

/// 一段历史逐步跑完，判每一步之后的整池镜像。
pub fn run_published(label: &str, ops: &[driver::Op], batch_units: u64) -> (Tally, driver::Pool) {
    let mut tally = Tally::default();
    let mut pool = driver::Pool::build(4 << 30, batch_units);
    for op in ops {
        pool.run(op);
    }
    for (index, step) in pool.steps.iter().enumerate() {
        let judged = judge(&step.after);
        tally.note("step", &judged, &format!("{label} #{index}:{} [{}]", step.label, step.outcome));
    }
    (tally, pool)
}

fn arg(args: &[String], index: usize) -> Option<usize> {
    args.get(index).and_then(|n| n.parse().ok())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let started = std::time::Instant::now();
    match args.get(1).map(String::as_str) {
        Some("sweep") => {
            let family = args.get(2).cloned().unwrap_or_else(|| "all".into());
            let plan: Vec<_> = plans::plan(&family);
            let from = arg(&args, 3).unwrap_or(0);
            let to = arg(&args, 4).unwrap_or(usize::MAX).min(plan.len());
            let lines = std::env::var("M3DCR2_LINES").is_ok_and(|value| value == "1");
            let plan = &plan[from.min(to)..to];
            let next = AtomicUsize::new(0);
            let total = Mutex::new(Tally::default());
            std::thread::scope(|scope| {
                for _ in 0..threads() {
                    scope.spawn(|| loop {
                        let index = next.fetch_add(1, Ordering::SeqCst);
                        let Some((label, ops, batch)) = plan.get(index) else { break };
                        let (tally, _) = run_published(label, ops, *batch);
                        total.lock().expect("锁").absorb(tally);
                    });
                }
            });
            let total = total.into_inner().expect("锁");
            println!("SWEEP family={family} histories={} from={from} to={to} elapsed={:.1}s", plan.len(), started.elapsed().as_secs_f64());
            total.print(&format!("sweep-{family}"), lines);
        }
        Some("show") => {
            // 一段历史：每一步的结局与每一版的事实（调试与报告引用）。
            let ops = plans::parse(args.get(2).expect("历史，如 C2,U2,R3"));
            let batch = arg(&args, 3).unwrap_or(1) as u64;
            let (tally, pool) = run_published("show", &ops, batch);
            for (index, step) in pool.steps.iter().enumerate() {
                println!("STEP #{index} {} -> {}", step.label, step.outcome);
                let _ = check_pool_image(&step.after);
                if let Some(facts) = attack_stash::take() {
                    println!("  newest=inst{}/txg{} wm={:?} maxkey={:?} rows={:?}", facts.newest_instance, facts.newest_txg, facts.newest_watermark, facts.largest_key_walked, facts.instance_table_rows);
                    for version in &facts.versions {
                        let records: Vec<_> = version.inode_records.iter().map(|record| (record.inode, record.nlink, record.birth)).collect();
                        let tombs: Vec<_> = cand::tombs(version).iter().map(|tomb| (tomb.object, tomb.birth, tomb.death, tomb.published)).collect();
                        println!(
                            "  V {} txg={} newest={} rec={} q={:?} wm={:?} recs={records:?} tombs={tombs:?} lineage={:?} gap={:?}",
                            version.name, version.txg, version.is_newest, version.from_records, version.queue, version.watermark, cand::lineage(version), version.gap
                        );
                    }
                }
            }
            tally.print("show", true);
        }
        Some("crash") => crashes::run(&args[2..]),
        Some("bad") => bad::run(&args[2..]),
        other => {
            eprintln!("用法：m3dcr2-opus-proto <sweep <family> [from to]|show <ops> [batch]|crash …|bad>，收到 {other:?}");
            std::process::exit(2);
        }
    }
    println!("ELAPSED {:.1}s", started.elapsed().as_secs_f64());
}
