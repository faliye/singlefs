//! 正推腿探针（m3-unlink-code-r1，Sonnet）K6.c：unlink 切进三张比重表之后，别的组（取样点）生成的历史有多少变了。
//! 只生成、不执行：旧表 = 新表去掉 Unlink、把它的份数还给 CloseAndMountWritable、去掉「删完之后」那张表（`None`）。
//! 对每个取样点（种子区间、步数照 `tests/random_histories.rs` 写死的常量）数：新表生成的历史里有 unlink 的几段、与旧表生成的不同的几段、
//! 第一处不同落在第几步；另核两个种子钉住的步（4000000045 第 27 步、4000000001 第 30 步，比重 REUSE_AFTER_RAISING_THE_FLOOR、60 步）之前有没有 unlink。
use singlefs_harness::history::{
    generate_history_with_weights, GenerationWeights, HistoryOperation, HistoryOperationKind, HistorySeed,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};

fn without_unlink(table: &'static [(HistoryOperationKind, u64)]) -> &'static [(HistoryOperationKind, u64)] {
    let unlink_share: u64 = table.iter().filter(|(kind, _)| *kind == HistoryOperationKind::Unlink).map(|(_, share)| *share).sum();
    let rebuilt: Vec<(HistoryOperationKind, u64)> = table
        .iter()
        .filter(|(kind, _)| *kind != HistoryOperationKind::Unlink)
        .map(|(kind, share)| {
            if *kind == HistoryOperationKind::CloseAndMountWritable { (*kind, share + unlink_share) } else { (*kind, *share) }
        })
        .collect();
    Box::leak(rebuilt.into_boxed_slice())
}

fn old_of(new: GenerationWeights) -> GenerationWeights {
    GenerationWeights {
        with_session_open_with_file: without_unlink(new.with_session_open_with_file),
        with_session_open_after_the_file_was_deleted: None,
        ..new
    }
}

fn compare(name: &str, new: GenerationWeights, first_seed: u64, seeds: u64, operations: usize) {
    let old = old_of(new);
    let (mut with_unlink, mut differing, mut first_difference_steps) = (0, 0, Vec::new());
    for seed in first_seed..first_seed + seeds {
        let new_history = generate_history_with_weights(HistorySeed(seed), operations, &new);
        let old_history = generate_history_with_weights(HistorySeed(seed), operations, &old);
        if new_history.operations.iter().any(|op| *op == HistoryOperation::Unlink) { with_unlink += 1; }
        if new_history.operations != old_history.operations {
            differing += 1;
            first_difference_steps.push(new_history.operations.iter().zip(&old_history.operations).position(|(a, b)| a != b).unwrap_or(0));
        }
    }
    first_difference_steps.sort_unstable();
    println!("{name}：{seeds} 段 × {operations} 步，新表里有 unlink 的 {with_unlink} 段，与旧表生成的不同的 {differing} 段，第一处不同的步（升序）{first_difference_steps:?}");
}

#[test]
fn how_many_histories_the_unlink_tables_changed() {
    let base = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    compare("快档 BROAD", GenerationWeights::BROAD, base, 96, 30);
    compare("大档默认 BROAD", GenerationWeights::BROAD, base, 8, 60);
    compare("复用取样点 REUSE_AFTER_RAISING_THE_FLOOR", GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR, base, 48, 30);
    compare("单元区墙取样点 TOWARD_THE_UNIT_AREA_WALL", GenerationWeights::TOWARD_THE_UNIT_AREA_WALL, base, 32, 150);
    for (seed, step) in [(4_000_000_045_u64, 27_usize), (4_000_000_001, 30)] {
        let new_history = generate_history_with_weights(HistorySeed(seed), 60, &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR);
        let old_history = generate_history_with_weights(HistorySeed(seed), 60, &old_of(GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR));
        let first_unlink = new_history.operations.iter().position(|op| *op == HistoryOperation::Unlink);
        let first_difference = new_history.operations.iter().zip(&old_history.operations).position(|(a, b)| a != b);
        println!("种子 {seed}（REUSE，60 步）：第一个 unlink 在 {first_unlink:?}，新旧第一处不同在 {first_difference:?}，钉住的步 {step}");
    }
}
