//! 抬 F 先算代码轮攻方腿的探针（只在冻结副本的拷贝里编，副本里的核心层拼了 `floor_raise_precheck_probe`）。
//!
//! 用法：floor-raise-precheck-probe --histories <条数> --seed-base <起始种子> --registration <.claude/heavy-thread-default 的路径>
//!                                  [--bound-without-deferred] [--soundness-without-deferred]
//!   --bound-without-deferred    探针自证：副本里先算的上界不算 defer（真池那一臂照它判），旁路先算的拷贝写得进的该成命中。
//!   --soundness-without-deferred 探针自证：对上界那一判不算 defer，推成一串回收过槽就该判出超出。
//! 线程数：SINGLEFS_THREAD_CAP 设了照它，没设取登记文件第一条非注释行开头的正整数。
//! 先跑一批（与线程数一样多的历史）估全量挂钟，估出来超过 40 分钟就停、退 3、不跑余下的。
//! 输出：逐行 `<名> 键=值`，末行 `<名> PROBE_RESULT …`。退出码：0 跑完；2 用法错；3 估时超过 40 分钟。

mod history;
mod memory_device;
mod pool;
mod tally;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use history::{run_history, Options};
use tally::Tally;

const WALL_CLOCK_LIMIT_SECONDS: f64 = 40.0 * 60.0;

fn usage(why: &str) -> ! {
    eprintln!("  ✗ {why}");
    eprintln!("用法：floor-raise-precheck-probe --histories <条数> --seed-base <起始种子> --registration <登记文件> [--bound-without-deferred] [--soundness-without-deferred]");
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

/// 跑种子 [first, first + count)：线程各取下一条，一条历史 panic 了记下种子与消息、接着跑别的。
fn run_seeds(first: u64, count: u64, threads: usize, options: &Options) -> Tally {
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
                let tally = match catch_unwind(AssertUnwindSafe(|| run_history(seed, options))) {
                    Ok(tally) => tally,
                    Err(payload) => {
                        let message = payload
                            .downcast_ref::<&str>()
                            .map(|text| (*text).to_string())
                            .or_else(|| payload.downcast_ref::<String>().cloned())
                            .unwrap_or_default();
                        Tally {
                            histories: 1,
                            panics: vec![format!("seed={seed} {message}")],
                            ..Tally::default()
                        }
                    }
                };
                merged
                    .lock()
                    .expect("计数的锁里不 panic")
                    .push((seed, tally));
            });
        }
    });
    let mut per_seed = merged.into_inner().expect("计数的锁里不 panic");
    per_seed.sort_by_key(|(seed, _)| *seed);
    let mut total = Tally::default();
    for (_, tally) in per_seed {
        total.merge(tally);
    }
    total
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut histories: Option<u64> = None;
    let mut seed_base: Option<u64> = None;
    let mut registration: Option<String> = None;
    let mut options = Options {
        bound_without_deferred: false,
        soundness_without_deferred: false,
    };
    let mut index = 0;
    while index < arguments.len() {
        let value = || {
            arguments
                .get(index + 1)
                .cloned()
                .unwrap_or_else(|| usage("参数后面缺值"))
        };
        match arguments[index].as_str() {
            "--histories" => {
                histories = value().parse().ok();
                index += 2;
            }
            "--seed-base" => {
                seed_base = value().parse().ok();
                index += 2;
            }
            "--registration" => {
                registration = Some(value());
                index += 2;
            }
            "--bound-without-deferred" => {
                options.bound_without_deferred = true;
                index += 1;
            }
            "--soundness-without-deferred" => {
                options.soundness_without_deferred = true;
                index += 1;
            }
            other => usage(&format!("认不出参数 {other}")),
        }
    }
    let histories = histories
        .filter(|count| *count > 0)
        .unwrap_or_else(|| usage("缺 --histories 或不是正整数"));
    let seed_base = seed_base.unwrap_or_else(|| usage("缺 --seed-base"));
    let threads = registered_threads(&registration.unwrap_or_else(|| usage("缺 --registration")));
    let name = if options.bound_without_deferred {
        "self-check-bound-without-deferred"
    } else if options.soundness_without_deferred {
        "self-check-soundness-without-deferred"
    } else {
        "probe"
    };
    let pilot = histories.min(threads as u64);
    let started = Instant::now();
    let mut total = run_seeds(seed_base, pilot, threads, &options);
    let pilot_seconds = started.elapsed().as_secs_f64();
    let batches = histories.div_ceil(threads as u64);
    let estimate_seconds = pilot_seconds * batches as f64;
    println!("{name} ESTIMATE threads={threads} pilot_histories={pilot} pilot_seconds={pilot_seconds:.1} histories={histories} batches={batches} estimate_seconds={estimate_seconds:.0}");
    if estimate_seconds > WALL_CLOCK_LIMIT_SECONDS {
        println!("{name} ESTIMATE_EXCEEDS_FORTY_MINUTES: 缩条数再跑；先跑那一批的计数照印");
        total.print(name);
        std::process::exit(3);
    }
    if histories > pilot {
        total.merge(run_seeds(
            seed_base + pilot,
            histories - pilot,
            threads,
            &options,
        ));
    }
    println!("{name} WALL_SECONDS {:.1}", started.elapsed().as_secs_f64());
    total.print(name);
}
