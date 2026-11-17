//! 乙甲代码轮第二轮攻方腿的探针（只在冻结副本的拷贝里编，副本里的核心层拼了 `refusal_precheck_oracle_probe`）：
//! 填满再打碎的那一族历史（`shapes.rs` 文件头），换一族再取一份乙在槽对那一判上、甲的误拒样本，另核增量计数与按记录重建的一致。
//!
//! 用法：refusal-precheck-oracle-probe --configurations <个数> --seed-base <起始种子> --steps <每个配置的步数> --registration <.claude/heavy-thread-default 的路径>
//! 线程数：SINGLEFS_THREAD_CAP 设了照它（服务给的），没设取登记文件第一条非注释行开头的正整数。
//!   另可给 --cost-tier-extra-slots <槽数> --cost-tier-configurations <个数>：主跑与自证之后另跑一档大盘（两块盘各多这么几槽），
//!   只量走那一遍重挂的挂钟（个数 0 或不给就不跑）。
//! 先跑一批（与线程数一样多的配置）估全量挂钟，估出来超过 100 分钟就停、退 3、不跑余下的。
//! 每个配置跑完往标准错误打一行 `PROGRESS`。
//! 输出：`oracle COUNT …`、`oracle EXAMPLE …`、末尾 `oracle PROBE_RESULT …`，再是自证几行 `name=self_check …` 与 `name=self_checks bad=<数>`。
//! 判别子那一臂（`discriminator.rs`）：主跑之后印 `oracle DISCRIMINATOR_TABLE …`（每个候选、每一判一行四格，再按判出之前推过串没有拆两行）
//! 与 `oracle WALK_COST …`（在只进内存的盘上走一遍重挂的挂钟），再挑主跑里判出过的种子
//! （每一判头 3 个）把判别子一律读成真、一律读成假各跑一小批，印 `forced-true DISCRIMINATOR_TABLE …`、`forced-false DISCRIMINATOR_TABLE …`，
//! 归零与否进自证；估时把这两小批各算一批。代价那一档印 `cost-tier DISCRIMINATOR_TABLE …`、`cost-tier WALK_COST …` 与 `cost-tier RESULT …`，
//! 不进自证；跑之前剩下的挂钟不够三批（推的：大盘一批按主跑一批的三倍算）就不跑、印 `cost-tier SKIPPED`。
//! 退出码：0 跑完且自证全过；1 自证有不合；2 用法错；3 估时超过 100 分钟。

mod device;
mod discriminator;
mod pool;
mod shapes;
mod tally;

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use shapes::{run_configuration, Options};
use tally::Tally;

const WALL_CLOCK_LIMIT_SECONDS: f64 = 100.0 * 60.0;

fn usage(why: &str) -> ! {
    eprintln!("  ✗ {why}");
    eprintln!("用法：refusal-precheck-oracle-probe --configurations <个数> --seed-base <起始种子> --steps <步数> --registration <登记文件> [--cost-tier-extra-slots <槽数> --cost-tier-configurations <个数>]");
    std::process::exit(2);
}

fn registered_threads(path: &str) -> usize {
    if let Ok(value) = std::env::var("SINGLEFS_THREAD_CAP") {
        return value
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| usage("SINGLEFS_THREAD_CAP 不是正整数"));
    }
    let text = std::fs::read_to_string(path).unwrap_or_else(|_| usage("读不到登记文件"));
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or_else(|| usage("登记文件里没有非注释行"));
    line.split_whitespace()
        .next()
        .and_then(|word| word.parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .unwrap_or_else(|| usage("登记文件第一条非注释行开头不是正整数"))
}

/// 自证那两小批里每一判至多取几个主跑里判出过它的种子。
const SEEDS_PER_JUDGEMENT_IN_A_FORCED_BATCH: usize = 3;

/// 跑点名的种子：线程各取下一个，一个配置 panic 了记下种子与消息、接着跑别的。
/// 交回按种子次序并起来的计数，与每个种子判出过哪几判（自证那两小批照它挑种子）。
fn run_seeds(
    seeds: &[u64],
    threads: usize,
    options: &Options,
    finished_before: u64,
    total: u64,
) -> (Tally, BTreeMap<u64, Vec<&'static str>>) {
    let next = AtomicU64::new(0);
    let count = seeds.len() as u64;
    let merged = Mutex::new(Vec::<(u64, Tally)>::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= count {
                    break;
                }
                let seed = seeds[usize::try_from(index).expect("下标")];
                let started = Instant::now();
                let tally = match catch_unwind(AssertUnwindSafe(|| run_configuration(seed, options))) {
                    Ok(tally) => tally,
                    Err(payload) => {
                        let message = payload
                            .downcast_ref::<&str>()
                            .map(|text| (*text).to_string())
                            .or_else(|| payload.downcast_ref::<String>().cloned())
                            .unwrap_or_default();
                        Tally {
                            configurations: 1,
                            panics: vec![format!("seed={seed} {message}")],
                            ..Tally::default()
                        }
                    }
                };
                let mut finished = merged.lock().expect("计数的锁里不 panic");
                finished.push((seed, tally));
                eprintln!(
                    "PROGRESS seed={seed} seconds={:.1} finished={}/{total}",
                    started.elapsed().as_secs_f64(),
                    finished_before + finished.len() as u64
                );
            });
        }
    });
    let mut per_seed = merged.into_inner().expect("计数的锁里不 panic");
    per_seed.sort_by_key(|(seed, _)| *seed);
    let mut total_tally = Tally::default();
    let mut triggers_of_each_seed = BTreeMap::new();
    for (seed, tally) in per_seed {
        triggers_of_each_seed.insert(seed, discriminator::triggers_fired_in(&tally));
        total_tally.merge(tally);
    }
    (total_tally, triggers_of_each_seed)
}

/// 自证那两小批：判别子一律读成真、一律读成假各跑一次主跑里挑出的种子，印各自的四格表，交回自证格。
fn forced_batches(
    seeds: &[u64],
    threads: usize,
    options: &Options,
    main: &Tally,
) -> Vec<(String, bool)> {
    let mut checks = Vec::new();
    for forced in [true, false] {
        eprintln!("PROGRESS forced-batch every-discriminator={forced} seeds={seeds:?}");
        discriminator::force_every_discriminator(Some(forced));
        let (batch, _) = run_seeds(seeds, threads, options, 0, seeds.len() as u64);
        discriminator::force_every_discriminator(None);
        let name = if forced { "forced-true" } else { "forced-false" };
        discriminator::print_tables(name, &batch);
        checks.extend(discriminator::checks_of_a_forced_batch(forced, &batch, main));
    }
    checks
}

/// 代价那一档：两块盘在 1007 + 384 槽之外各多 `extra` 槽（盘的几何只这一档，文件大小两档照种子取），跑 `count` 个配置
/// （种子从 `seed_base` 起），只印四格表、走那一遍的挂钟与汇总。`seconds_after_it_at_least` 是跑完它至少到第几秒（推的），超过挂钟上限就不跑。
fn run_the_cost_tier(
    extra: u64,
    count: u64,
    seed_base: u64,
    steps: usize,
    threads: usize,
    seconds_after_it_at_least: f64,
) {
    if seconds_after_it_at_least > WALL_CLOCK_LIMIT_SECONDS {
        println!(
            "cost-tier SKIPPED extra_slots={extra} configurations={count} seconds_after_it_at_least={seconds_after_it_at_least:.0} limit_seconds={WALL_CLOCK_LIMIT_SECONDS:.0}"
        );
        return;
    }
    eprintln!("PROGRESS cost-tier extra_slots={extra} configurations={count}");
    let tier_started = Instant::now();
    let seeds: Vec<u64> = (seed_base..seed_base + count).collect();
    let options = Options::with_one_geometry(steps, [extra, extra]);
    let (tier, _) = run_seeds(&seeds, threads, &options, 0, count);
    discriminator::print_tables("cost-tier", &tier);
    discriminator::print_walk_costs("cost-tier");
    println!(
        "cost-tier RESULT extra_slots={extra} configurations={} fired={} panics={} seconds={:.1}",
        tier.configurations,
        tier.count("fired"),
        tier.panics.len(),
        tier_started.elapsed().as_secs_f64()
    );
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut configurations: Option<u64> = None;
    let mut seed_base: Option<u64> = None;
    let mut steps: Option<usize> = None;
    let mut registration: Option<String> = None;
    let mut cost_tier_extra_slots: Option<u64> = None;
    let mut cost_tier_configurations: Option<u64> = None;
    let mut index = 0;
    while index < arguments.len() {
        let value = || {
            arguments
                .get(index + 1)
                .cloned()
                .unwrap_or_else(|| usage("参数后面缺值"))
        };
        match arguments[index].as_str() {
            "--configurations" => configurations = value().parse().ok(),
            "--seed-base" => seed_base = value().parse().ok(),
            "--steps" => steps = value().parse().ok(),
            "--registration" => registration = Some(value()),
            "--cost-tier-extra-slots" => {
                cost_tier_extra_slots = Some(
                    value()
                        .parse()
                        .unwrap_or_else(|_| usage("--cost-tier-extra-slots 不是非负整数")),
                )
            }
            "--cost-tier-configurations" => {
                cost_tier_configurations = Some(
                    value()
                        .parse()
                        .unwrap_or_else(|_| usage("--cost-tier-configurations 不是非负整数")),
                )
            }
            other => usage(&format!("认不出参数 {other}")),
        }
        index += 2;
    }
    let configurations = configurations
        .filter(|count| *count > 0)
        .unwrap_or_else(|| usage("缺 --configurations 或不是正整数"));
    let seed_base = seed_base.unwrap_or_else(|| usage("缺 --seed-base"));
    let steps = steps
        .filter(|count| *count > 0)
        .unwrap_or_else(|| usage("缺 --steps 或不是正整数"));
    let registration = registration.unwrap_or_else(|| usage("缺 --registration"));
    let threads = registered_threads(&registration);
    let options = Options::new(steps);
    let started = Instant::now();
    let pilot_count = (threads as u64).min(configurations);
    let pilot_seeds: Vec<u64> = (seed_base..seed_base + pilot_count).collect();
    let (mut total, mut triggers_of_each_seed) =
        run_seeds(&pilot_seeds, threads, &options, 0, configurations);
    let pilot_seconds = started.elapsed().as_secs_f64();
    let batches = configurations.div_ceil(threads as u64);
    let estimate_seconds = pilot_seconds * batches as f64;
    // 自证那两小批各至多一批的挂钟（种子数不过线程数时并排跑完）。
    let estimate_with_the_forced_batches_seconds = pilot_seconds * (batches + 2) as f64;
    println!(
        "oracle ESTIMATE threads={threads} pilot_configurations={pilot_count} pilot_seconds={pilot_seconds:.1} configurations={configurations} batches={batches} estimate_seconds={estimate_seconds:.0} estimate_with_the_forced_batches_seconds={estimate_with_the_forced_batches_seconds:.0}"
    );
    if estimate_with_the_forced_batches_seconds > WALL_CLOCK_LIMIT_SECONDS {
        println!(
            "oracle ESTIMATE_EXCEEDS_THE_WALL_CLOCK_LIMIT limit_seconds={WALL_CLOCK_LIMIT_SECONDS:.0}：先缩配置数或步数再跑"
        );
        total.print("oracle-pilot");
        std::process::exit(3);
    }
    if configurations > pilot_count {
        let rest_seeds: Vec<u64> = (seed_base + pilot_count..seed_base + configurations).collect();
        let (rest, rest_triggers) = run_seeds(
            &rest_seeds,
            threads,
            &options,
            pilot_count,
            configurations,
        );
        total.merge(rest);
        triggers_of_each_seed.extend(rest_triggers);
    }
    println!(
        "oracle WALL_SECONDS {:.1}",
        started.elapsed().as_secs_f64()
    );
    total.print("oracle");
    discriminator::print_tables("oracle", &total);
    discriminator::print_walk_costs("oracle");
    let forced_seeds = discriminator::seeds_for_the_forced_batches(
        &triggers_of_each_seed,
        SEEDS_PER_JUDGEMENT_IN_A_FORCED_BATCH,
    );
    let forced_checks = forced_batches(&forced_seeds, threads, &options, &total);
    discriminator::discard_walk_costs();
    println!(
        "oracle WALL_SECONDS_WITH_THE_FORCED_BATCHES {:.1} forced_seeds={forced_seeds:?}",
        started.elapsed().as_secs_f64()
    );
    if let (Some(extra), Some(count)) = (cost_tier_extra_slots, cost_tier_configurations) {
        if count > 0 {
            run_the_cost_tier(
                extra,
                count,
                seed_base,
                steps,
                threads,
                started.elapsed().as_secs_f64() + pilot_seconds * 3.0,
            );
        }
    }
    let oracle_reported_no_space_even: u64 = total
        .counts
        .iter()
        .filter(|(key, _)| key.contains("oracle=NoSpaceEvenWithEverythingAFloorRaiseCouldRelease"))
        .map(|(_, count)| *count)
        .sum();
    let checks_of_the_family: [(&str, bool); 4] = [
        (
            "推串之前合取 7 那一判的探针钩子跑到过（conjunct-before-any-series judged 大于 0）",
            total.count("conjunct-before-any-series judged") > 0,
        ),
        (
            "推成一串的探针钩子跑到过（series arm=product 大于 0）",
            total.count("series arm=product") > 0,
        ),
        (
            "O 那一臂先算关得掉（O 那一臂一次都不报 NoSpaceEvenWithEverythingAFloorRaiseCouldRelease）",
            oracle_reported_no_space_even == 0,
        ),
        ("没有配置 panic", total.panics.is_empty()),
    ];
    let mut checks: Vec<(String, bool)> = checks_of_the_family
        .into_iter()
        .map(|(what, holds)| (what.to_string(), holds))
        .collect();
    checks.extend(discriminator::checks_of_the_main_run(&total));
    checks.extend(forced_checks);
    let mut bad = 0;
    for (what, holds) in checks {
        if holds {
            println!("name=self_check ok {what}");
        } else {
            bad += 1;
            println!("  ✗ name=self_check {what}");
        }
    }
    if total.count("base published-after-a-series") == 0 {
        println!("name=misrefusal_base_zero 这一族里产品那一臂推过一串之后一次都没写进：测不出误拒");
    }
    if total.count("fired by=conjunct-before-any-series") == 0 {
        println!("name=conjunct_fired_zero 这一族里推串之前合取 7 那一判一次都没判出：甲的误拒没有分母");
    }
    println!("name=self_checks bad={bad}");
    if bad > 0 {
        std::process::exit(1);
    }
}
