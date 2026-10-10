//! 层 0：随机前缀（可带崩溃与重挂）之后，接一段不崩的后缀，后缀那条录制流按层 0 的枚举域全展开
//! （`singlefs_checker_tier::crash::judge_layer0_state_range`，展开取 `full_expansion`：与 `enumerate_layer0` 同一份 `Layer0StatePlan`、
//! 同一个 `judge_crash_image`，段内写的整写子集逐个枚举，含撕裂的原地覆盖写）。状态数先用 `closed_form_state_count` 与
//! `layer0_state_count_with_torn_in_place_overwrites` 算，超过上限不跑；跑完核对判过的个数。每个状态：新 checker 的判定取自层 0 那一份，
//! 再跑探针、pre36 与 core 那一侧逐项记（`tally::note_image`）。前缀、后缀的流都在这里现造，不从任何层 0 用例拷。
use singlefs_checker_tier::crash::{full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;

use crate::judge::kinds_of_new;
use crate::random::{draw_op, play};
use crate::tally::{note_image, Tally};

fn arg(index: usize, default: u64) -> u64 {
    std::env::args().nth(index).and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn run() {
    let (seed, prefix_steps, suffix_steps) = (arg(2, 0), arg(3, 4) as usize, arg(4, 2) as usize);
    let (threads, max_states, count_only) = (arg(5, 1).max(1), arg(6, 1_000_000), arg(7, 0) == 1);
    let (mut driver, mut rng, prefix_line) = play(0xB36_0000 + seed, prefix_steps, None);
    let base: MemoryPool = driver.image();
    let start = driver.stream.operation_count();
    let mut suffix = Vec::new();
    for _ in 0..suffix_steps {
        let op = draw_op(driver.state(), &mut rng);
        let outcome = driver.execute(op);
        suffix.push(format!("{op:?}[{}]", outcome.split_whitespace().take(2).collect::<Vec<_>>().join(" ")));
    }
    let ops = driver.ops_since(start);
    let geometry = driver.geometry();
    let (writes, segments) = writes_and_segments(&ops, &geometry);
    let closed_form = closed_form_state_count(&segments);
    let state_count = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    let record_writes = writes.iter().filter(|w| w.kind == StepKind::JournalRecord).count();
    println!("{prefix_line}");
    println!(
        "B36L0 seed={seed} suffix={} segments={sizes:?} record_writes={record_writes} closed_form={closed_form} layer0_states={state_count}",
        suffix.join(" "),
    );
    if count_only || state_count > max_states || writes.is_empty() {
        println!("B36L0-SKIP seed={seed} states={state_count} max={max_states} count_only={count_only}");
        return;
    }
    // 第 8、9 个参数：判哪一段状态序号 [起, 止)（止取 0 = 到最后一个状态），一条流太大时分几条命令跑。
    let range_start = arg(8, 0).min(state_count);
    let range_end = match arg(9, 0) {
        0 => state_count,
        end => end.min(state_count),
    };
    let enumerated = range_end.saturating_sub(range_start);
    let judged_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
    let per = enumerated.div_ceil(threads);
    let started = std::time::Instant::now();
    let tallies: Vec<(Tally, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (base, writes, segments) = (&base, &writes, &segments);
                scope.spawn(move || {
                    let mut tally = Tally::default();
                    let range = range_start + (t * per).min(enumerated)..range_start + ((t + 1) * per).min(enumerated);
                    let mut judged = 0u64;
                    let _ = judge_layer0_state_range(
                        base, writes, segments, judged_root_index, &[], &full_expansion, range,
                        &mut |ordinal, image, judgement| {
                            let label = format!("seed={seed} ordinal={ordinal}");
                            note_image(&mut tally, &label, image, Some(kinds_of_new(&judgement.pool_checker)));
                            judged += 1;
                        },
                    );
                    (tally, judged)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("线程")).collect()
    });
    let mut total = Tally::default();
    let mut judged = 0;
    for (tally, count) in tallies {
        total.absorb(tally);
        judged += count;
    }
    assert_eq!(judged, enumerated, "判过的状态数等于要判的那一段");
    println!(
        "B36L0-DONE seed={seed} judged={judged} range={range_start}..{range_end} of={state_count} full={} seconds={:.1}",
        judged == state_count,
        started.elapsed().as_secs_f64()
    );
    total.print(&format!("B36L0T seed={seed}"));
}
