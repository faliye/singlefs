//! 抬 F 先算代码轮第二轮攻方腿的探针（只在冻结副本的拷贝里编，副本里的核心层拼了 `floor_raise_precheck_r2_probe`）。
//!
//! 用法：floor-raise-precheck-r2-probe --configurations <个数> --seed-base <起始种子> --registration <.claude/heavy-thread-default 的路径>
//!                                    [--arm-a as-coded|zero|huge]
//!   --arm-a as-coded  全量：A 照冻结副本的上界（算聚簇段里那一截），B 那一截读成 0。
//!   --arm-a zero      探针自证：A 也读成 0，A 与 B 每一格都该相同（arms_differ=0），不然分类或拷贝有毛病。
//!   --arm-a huge      探针自证：A 那一截读成大数，槽对那一判永远照推；有槽对那一判拒的格就该数出白推（vain_series_by_the_inside_term>0）。
//! 线程数：SINGLEFS_THREAD_CAP 设了照它（服务给的），没设取登记文件第一条非注释行开头的正整数。
//! 先跑一批（与线程数一样多的配置）估全量挂钟，估出来超过 40 分钟就停、退 3、不跑余下的。
//! 输出：逐行 `<名> <类> …`，末行 `<名> PROBE_RESULT …`。退出码：0 跑完；2 用法错；3 估时超过 40 分钟。

mod device;
mod pool;
mod shapes;
mod tally;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use shapes::{run_configuration, Options};
use singlefs_core::mounted_session::floor_raise_precheck_r2_probe::InsideTerm;
use tally::Tally;

const WALL_CLOCK_LIMIT_SECONDS: f64 = 40.0 * 60.0;

fn usage(why: &str) -> ! {
    eprintln!("  ✗ {why}");
    eprintln!("用法：floor-raise-precheck-r2-probe --configurations <个数> --seed-base <起始种子> --registration <登记文件> [--arm-a as-coded|zero|huge]");
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
    let mut configurations: Option<u64> = None;
    let mut seed_base: Option<u64> = None;
    let mut registration: Option<String> = None;
    let mut arm_a = InsideTerm::AsCoded;
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
            "--registration" => registration = Some(value()),
            "--arm-a" => {
                arm_a = match value().as_str() {
                    "as-coded" => InsideTerm::AsCoded,
                    "zero" => InsideTerm::Zero,
                    "huge" => InsideTerm::Huge,
                    other => usage(&format!("--arm-a 认不出 {other}")),
                }
            }
            other => usage(&format!("认不出参数 {other}")),
        }
        index += 2;
    }
    let configurations = configurations
        .filter(|count| *count > 0)
        .unwrap_or_else(|| usage("缺 --configurations 或不是正整数"));
    let seed_base = seed_base.unwrap_or_else(|| usage("缺 --seed-base"));
    let threads = registered_threads(&registration.unwrap_or_else(|| usage("缺 --registration")));
    let options = Options { arm_a };
    let name = match arm_a {
        InsideTerm::AsCoded => "probe",
        InsideTerm::Zero => "self-check-arm-a-zero",
        InsideTerm::Huge => "self-check-arm-a-huge",
    };
    let pilot = configurations.min(threads as u64);
    let started = Instant::now();
    let mut total = run_seeds(seed_base, pilot, threads, &options);
    let pilot_seconds = started.elapsed().as_secs_f64();
    let batches = configurations.div_ceil(threads as u64);
    let estimate_seconds = pilot_seconds * batches as f64;
    println!("{name} ESTIMATE threads={threads} pilot_configurations={pilot} pilot_seconds={pilot_seconds:.1} configurations={configurations} batches={batches} estimate_seconds={estimate_seconds:.0}");
    if estimate_seconds > WALL_CLOCK_LIMIT_SECONDS {
        println!("{name} ESTIMATE_EXCEEDS_FORTY_MINUTES: 缩配置数再跑；先跑那一批的计数照印");
        total.print(name);
        std::process::exit(3);
    }
    if configurations > pilot {
        total.merge(run_seeds(
            seed_base + pilot,
            configurations - pilot,
            threads,
            &options,
        ));
    }
    println!("{name} WALL_SECONDS {:.1}", started.elapsed().as_secs_f64());
    total.print(name);
}
