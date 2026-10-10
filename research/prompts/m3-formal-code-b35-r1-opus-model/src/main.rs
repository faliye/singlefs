//! 第 35 批代码轮第一轮攻方原型（`research/prompts/m3-formal-code-b35-r1-opus-model/`）。用法见 `tools/rerun.sh`。
mod core;
mod craft;
mod crafted;
mod drive;
mod judge;
mod layer0;
mod random;
mod smallring;
mod tally;

fn arg(index: usize, default: u64) -> u64 {
    std::env::args().nth(index).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    match mode.as_str() {
        "random" => {
            let (first, count, steps) = (arg(2, 0), arg(3, 4), arg(4, 12) as usize);
            let threads = arg(5, 1) as usize;
            let seeds: Vec<u64> = (first..first + count).collect();
            let chunks: Vec<Vec<u64>> = seeds.chunks(seeds.len().div_ceil(threads).max(1)).map(<[u64]>::to_vec).collect();
            let started = std::time::Instant::now();
            let tallies: Vec<tally::Tally> = std::thread::scope(|scope| {
                let handles: Vec<_> = chunks
                    .into_iter()
                    .map(|chunk| {
                        scope.spawn(move || {
                            let mut tally = tally::Tally::default();
                            for seed in chunk {
                                let t0 = std::time::Instant::now();
                                let (_, _, _, line) = random::play(0xB35_0000 + seed, steps, Some(&mut tally));
                                println!("{line} seconds={:.1}", t0.elapsed().as_secs_f64());
                            }
                            tally
                        })
                    })
                    .collect();
                handles.into_iter().map(|h| h.join().expect("线程")).collect()
            });
            let mut total = tally::Tally::default();
            for t in tallies {
                total.absorb(t);
            }
            println!("B35R-DONE first={first} count={count} steps={steps} seconds={:.1}", started.elapsed().as_secs_f64());
            total.print("B35R");
        }
        "layer0" => layer0::run(),
        "crafted" => crafted::run(),
        "smallring" => smallring::run(),
        _ => eprintln!("用法：b35probe random <首个种子> <段数> <步数> <线程> | layer0 … | crafted <线程> | smallring <环记录数> <线程>"),
    }
}
