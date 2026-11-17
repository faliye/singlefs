//! 乙甲代码轮第三轮攻方腿的探针（第二轮攻方那一份拷来改；只在冻结副本的拷贝里编，副本里的核心层拼了 `refusal_precheck_oracle_probe`）：
//! 填满再打碎的那一族历史（`shapes.rs` 文件头），在今天这棵树上重取乙在槽对那一判上、甲的误拒样本，另核增量计数与按记录重建的一致、
//! 乙的上界与落得下那一判是不是同一把尺（`ruler.rs`），判出放不下的格按被拒的那一次是哪一种分。
//!
//! 用法：refusal-precheck-oracle-probe --configurations <个数> --seed-base <起始种子> --steps <每个配置的步数> --registration <.claude/heavy-thread-default 的路径>
//! 线程数：SINGLEFS_THREAD_CAP 设了照它（服务给的），没设取登记文件第一条非注释行开头的正整数。
//! 先跑一批（与线程数一样多的配置）估全量挂钟，估出来超过 100 分钟就停、退 3、不跑余下的。
//! 每个配置跑完往标准错误打一行 `PROGRESS`。
//! 输出：`oracle COUNT …`、`oracle EXAMPLE …`、末尾 `oracle PROBE_RESULT …`，再是自证几行 `name=self_check …` 与 `name=self_checks bad=<数>`。
//! 退出码：0 跑完且自证全过；1 自证有不合；2 用法错；3 估时超过 100 分钟。

mod device;
mod pool;
mod ruler;
mod shapes;
mod tally;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use shapes::{run_configuration, Options};
use tally::Tally;

const WALL_CLOCK_LIMIT_SECONDS: f64 = 100.0 * 60.0;

fn usage(why: &str) -> ! {
    eprintln!("  ✗ {why}");
    eprintln!("用法：refusal-precheck-oracle-probe --configurations <个数> --seed-base <起始种子> --steps <步数> --registration <登记文件>");
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

/// 跑种子 [first, first + count)：线程各取下一个，一个配置 panic 了记下种子与消息、接着跑别的。
fn run_seeds(first: u64, count: u64, threads: usize, options: &Options, finished_before: u64, total: u64) -> Tally {
    let next = AtomicU64::new(0);
    let merged = Mutex::new(Vec::<(u64, Tally)>::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= count {
                    break;
                }
                let seed = first + index;
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
    for (_, tally) in per_seed {
        total_tally.merge(tally);
    }
    total_tally
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut configurations: Option<u64> = None;
    let mut seed_base: Option<u64> = None;
    let mut steps: Option<usize> = None;
    let mut registration: Option<String> = None;
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
    let mut total = run_seeds(seed_base, pilot_count, threads, &options, 0, configurations);
    let pilot_seconds = started.elapsed().as_secs_f64();
    let batches = configurations.div_ceil(threads as u64);
    let estimate_seconds = pilot_seconds * batches as f64;
    println!(
        "oracle ESTIMATE threads={threads} pilot_configurations={pilot_count} pilot_seconds={pilot_seconds:.1} configurations={configurations} batches={batches} estimate_seconds={estimate_seconds:.0}"
    );
    if estimate_seconds > WALL_CLOCK_LIMIT_SECONDS {
        println!(
            "oracle ESTIMATE_EXCEEDS_THE_WALL_CLOCK_LIMIT limit_seconds={WALL_CLOCK_LIMIT_SECONDS:.0}：先缩配置数或步数再跑"
        );
        total.print("oracle-pilot");
        std::process::exit(3);
    }
    if configurations > pilot_count {
        let rest = run_seeds(
            seed_base + pilot_count,
            configurations - pilot_count,
            threads,
            &options,
            pilot_count,
            configurations,
        );
        total.merge(rest);
    }
    println!(
        "oracle WALL_SECONDS {:.1}",
        started.elapsed().as_secs_f64()
    );
    total.print("oracle");
    let oracle_reported_no_space_even: u64 = total
        .counts
        .iter()
        .filter(|(key, _)| key.contains("oracle=NoSpaceEvenWithEverythingAFloorRaiseCouldRelease"))
        .map(|(_, count)| *count)
        .sum();
    let checks: [(&str, bool); 5] = [
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
        (
            "同一把尺那一核跑到过（ruler checked 大于 0）",
            total.count("ruler checked") > 0,
        ),
    ];
    let mut bad = 0;
    for (what, holds) in checks {
        if holds {
            println!("name=self_check ok {what}");
        } else {
            bad += 1;
            println!("  ✗ name=self_check {what}");
        }
    }
    if total.count("fired by=conjunct-before-any-series") == 0 {
        println!("name=conjunct_fired_zero 这一族里推串之前合取 7 那一判一次都没判出：甲的误拒没有分母");
    }
    if total.count("base published-after-a-series") == 0 {
        println!("name=misrefusal_base_zero 这一族里产品那一臂推过一串之后一次都没写进：测不出误拒");
    }
    println!("name=self_checks bad={bad}");
    if bad > 0 {
        std::process::exit(1);
    }
}
