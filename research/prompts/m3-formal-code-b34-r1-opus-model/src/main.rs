//! 第 34 批代码轮第一轮攻方原型（`research/prompts/m3-formal-code-b34-r1-opus-model/`）。用法见 `tools/rerun.sh`。
mod craft;
mod drive;
mod judge;
mod layer0;
mod p2;
mod random;
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
            let results: Vec<(tally::Tally, Vec<String>)> = std::thread::scope(|scope| {
                let handles: Vec<_> = chunks
                    .into_iter()
                    .map(|chunk| {
                        scope.spawn(move || {
                            let mut tally = tally::Tally::default();
                            let lines: Vec<String> = chunk
                                .iter()
                                .map(|seed| {
                                    let started = std::time::Instant::now();
                                    let line = random::run_history(0xB34_0000 + seed, steps, &mut tally);
                                    println!("{line} seconds={:.1}", started.elapsed().as_secs_f64());
                                    line
                                })
                                .collect();
                            (tally, lines)
                        })
                    })
                    .collect();
                handles.into_iter().map(|h| h.join().expect("线程")).collect()
            });
            let mut total = tally::Tally::default();
            for (tally, lines) in results {
                let _ = lines;
                total.absorb(tally);
            }
            total.print("B34R");
        }
        "layer0" => layer0::run(),
        "crafted" => p2::run(),
        _ => eprintln!("用法：b34probe random <首个种子> <段数> <步数> <线程> | layer0 <名字>"),
    }
}
