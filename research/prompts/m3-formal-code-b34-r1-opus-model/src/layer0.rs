//! 层 0：随机前缀（可带崩溃与重挂，造出由记录施加出来的版本）之后，接一段不崩的后缀，后缀那条录制流按层 0 的枚举域全展开
//! （`singlefs_checker_tier::crash::judge_layer0_state_range`，展开取 `full_expansion`：与 `enumerate_layer0` 同一份
//! `Layer0StatePlan`、同一个 `judge_crash_image`；状态数先用 `layer0_state_count_with_torn_in_place_overwrites` 算，跑完核对）。
//! 每个状态：判定里已有新 checker 的结果，这里再跑旧 checker 逐条比。
use singlefs_checker_tier::crash::{
    full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites,
};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;

use crate::judge::{kind_new, kind_old, Compared, OldReader};
use crate::random::{draw_op, play};
use crate::tally::Tally;

fn arg(index: usize, default: u64) -> u64 {
    std::env::args().nth(index).and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn run() {
    let (seed, prefix_steps, suffix_steps) = (arg(2, 0), arg(3, 4) as usize, arg(4, 2) as usize);
    let (threads, max_states, count_only) = (arg(5, 1).max(1), arg(6, 1_000_000), arg(7, 0) == 1);
    // 第 8 个参数：1 = 每个状态再跑旧 checker 对拍（默认），0 = 只看新 checker 的判定（省一半）。
    // 第 9、10 个：判哪一段状态序号 [起, 止)（止取 0 = 到最后一个状态）：一条流分几次调用判完，几段并起来罩住 [0, 状态数)。
    let (with_old, range_start, range_end) = (arg(8, 1) == 1, arg(9, 0), arg(10, 0));
    let (mut driver, mut rng, prefix_line) = play(0xB34_0000 + seed, prefix_steps, None);
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
    let state_count = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("{prefix_line}");
    println!(
        "B34L0 seed={seed} suffix={} segments={sizes:?} closed_form={} layer0_states={state_count}",
        suffix.join(" "),
        closed_form_state_count(&segments)
    );
    // 基镜像（后缀之前）与后缀全持久之后的镜像上，旧 checker 探针认出的两类版本（只为报告覆盖面；判红与否看下面的逐状态计数）。
    for (which, image) in [("base", base.clone()), ("end", driver.image())] {
        let compared = crate::judge::compare(&image);
        println!(
            "B34L0-PROBE seed={seed} {which} versions_applied_only_by_records={:?} next_mount_version={} new_i54={:?}",
            compared.versions_applied_only_by_records,
            compared.next_mount_version,
            compared.new.get("I-5.4").map(|k| k.0)
        );
    }
    if count_only || state_count > max_states || writes.is_empty() {
        println!("B34L0-SKIP seed={seed} states={state_count} max={max_states} count_only={count_only}");
        return;
    }
    let judged_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
    let range_end = if range_end == 0 { state_count } else { range_end.min(state_count) };
    let enumerated = range_end.saturating_sub(range_start);
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
                            use singlefs_checker_pre34::walk::{PROBE_NEXT_MOUNT_VERSION, PROBE_VERSIONS_APPLIED_ONLY_BY_RECORDS};
                            PROBE_VERSIONS_APPLIED_ONLY_BY_RECORDS.with(|cell| cell.set(Vec::new()));
                            PROBE_NEXT_MOUNT_VERSION.with(|cell| cell.set(false));
                            let old = if with_old {
                                singlefs_checker_pre34::walk::check_pool_image(&OldReader(image))
                            } else {
                                Vec::new()
                            };
                            let compared = Compared {
                                new: judgement.pool_checker.iter().map(|(n, v)| (*n, kind_new(v))).collect(),
                                old: if with_old {
                                    old.iter().map(|(n, v)| (*n, kind_old(v))).collect()
                                } else {
                                    judgement.pool_checker.iter().map(|(n, v)| (*n, kind_new(v))).collect()
                                },
                                versions_applied_only_by_records: PROBE_VERSIONS_APPLIED_ONLY_BY_RECORDS.with(|c| c.take()),
                                next_mount_version: PROBE_NEXT_MOUNT_VERSION.with(std::cell::Cell::get),
                                fl: None,
                                all_zero_tree_table_pointers: (0, 0),
                            };
                            tally.note(&format!("seed={seed} ordinal={ordinal}"), &compared);
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
    assert_eq!(judged, enumerated, "判过的状态数等于要判的状态数");
    println!(
        "B34L0-DONE seed={seed} judged={judged} range={range_start}..{range_end} of={state_count} full={} with_old={with_old} seconds={:.1}",
        judged == state_count,
        started.elapsed().as_secs_f64()
    );
    total.print(&format!("B34L0T seed={seed}"));
}
