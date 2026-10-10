// 本轮腿的小模型：把 crash_injection.rs 里两个纯函数原样抄出来（冻结副本 /tmp/claude-1000/m3-formal-code-b20-r1/tree/ 里
// stream_index_of_the_last_write_of_the_crash_window 在第 1054 行、exactly_one_second_crash_point 在第 2738 行），
// WritableMountPhase 与 SecondCrashPoint 换成同形的桩，只为量它们的行为；函数体一个字不改（占位符 //MUTATE_F5// 只在变异副本里换）。
#![allow(dead_code)]
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum WritableMountPhase { RowPublish, WarmUp, Continuation }

#[derive(Clone, Debug, PartialEq, Eq)]
struct SecondCrashPoint {
    segment_index: usize,
    persisted_within_the_segment: Vec<bool>,
    phase: WritableMountPhase,
}

fn stream_index_of_the_last_write_of_the_crash_window(
    stream_indexes: &[usize],
    first_write_of_the_segment: usize,
    writes_from_the_segment_on: usize,
) -> usize {
    let last_write_within_the_crash_window = writes_from_the_segment_on
        .checked_sub(1)
        .expect("崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数在摆的时候拒 0");
    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]
}

fn exactly_one_second_crash_point(
    mount_segments: &[Vec<usize>],
    phase_of_each_write: &[WritableMountPhase],
    segment_index: usize,
    writes_from_the_segment_on: usize,
    withheld_among_them: &[usize],
) -> SecondCrashPoint {
    let first_write_of_the_segment = mount_segments
        .get(segment_index)
        .and_then(|segment| segment.first().copied())
        .unwrap_or_else(|| {
            panic!(
                "靶向用例写死的挂载第 {segment_index} 段不在挂载写出的 {} 段里",
                mount_segments.len()
            )
        });
    assert!(
        writes_from_the_segment_on > 0
            && first_write_of_the_segment + writes_from_the_segment_on
                <= phase_of_each_write.len(),
        "靶向用例从挂载第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过挂载写表末尾（{} 个写）",
        phase_of_each_write.len()
    );
    let first_withheld = withheld_among_them
        .iter()
        .min()
        .copied()
        .filter(|_| {
            withheld_among_them
                .iter()
                .all(|withheld| *withheld < writes_from_the_segment_on)
        })
        .unwrap_or_else(|| {
            panic!(
                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从挂载第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写里"
            )
        });
    SecondCrashPoint {
        segment_index,
        persisted_within_the_segment: (0..writes_from_the_segment_on)
            .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
            .collect(),
        phase: phase_of_each_write[first_write_of_the_segment + first_withheld],
    }
}

fn panics<R>(f: impl FnOnce() -> R) -> bool {
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(f)).is_err();
    let _ = std::panic::take_hook();
    r
}

fn main() {
    // ---- F5 内联单测（crash_injection.rs 第 5498、5503、5508 行那三条断言）
    let stream_indexes = [0, 1, 3, 4, 6, 9];
    let f5 = [
        (2, 3, 6usize), (4, 2, 9), (3, 1, 4),
    ];
    let mut f5_ok = true;
    for (first, n, want) in f5 {
        let got = stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, first, n);
        println!("F5 first={first} n={n} 得 {got} 要 {want} {}", if got == want { "绿" } else { "红" });
        f5_ok &= got == want;
    }
    println!("F5 单测整体：{}", if f5_ok { "绿" } else { "红" });
    println!("F5 n=0 panic：{}", panics(|| stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 2, 0)));

    // ---- 靶向二次崩溃：用探针量到的真挂载写表（38 个写、9 段）
    let segments: Vec<Vec<usize>> = vec![
        (0..2).collect(), (2..16).collect(), (16..18).collect(), vec![18], (19..21).collect(),
        (21..33).collect(), (33..35).collect(), vec![35], (36..38).collect(),
    ];
    let phases: Vec<WritableMountPhase> = (0..38)
        .map(|w| if w <= 20 { WritableMountPhase::RowPublish } else { WritableMountPhase::WarmUp })
        .collect();
    let f2 = exactly_one_second_crash_point(&segments, &phases, 3, 3, &[0]);
    println!("F2 用例入参 (段 3, 窗口 3, 扣下 [0]) → {f2:?}");
    println!("F2 越过段尾：窗口 [18,19,20] 里 18 落段 3、19、20 是段 4 的写；持久 {:?}", f2.persisted_within_the_segment);
    // 扣下的不在窗口第一个：崩在挂载哪一段按第一个被扣下的写取（这里造一个能区分的相位表）
    let mut phases2 = phases.clone();
    phases2[19] = WritableMountPhase::Continuation;
    let g = exactly_one_second_crash_point(&segments, &phases2, 3, 3, &[1, 2]);
    println!("F2 扣下 [1,2]（写 19、20），相位表里 19 记 Continuation → phase={:?}（应取第一个被扣下的写 19）persisted={:?}", g.phase, g.persisted_within_the_segment);
    let h = exactly_one_second_crash_point(&segments, &phases2, 3, 3, &[2, 1]);
    println!("F2 扣下 [2,1]（无序）→ phase={:?}（min 取 1 → 写 19）", h.phase);
    // panic 清单（文档说的几种）
    println!("F2 panic：段号越界(9) {}", panics(|| exactly_one_second_crash_point(&segments, &phases, 9, 1, &[0])));
    println!("F2 panic：窗口 0 {}", panics(|| exactly_one_second_crash_point(&segments, &phases, 3, 0, &[0])));
    println!("F2 panic：越过写表末尾（段 8 起数 3） {}", panics(|| exactly_one_second_crash_point(&segments, &phases, 8, 3, &[0])));
    println!("F2 panic：恰好到写表末尾（段 8 起数 2）不应 panic {}", !panics(|| exactly_one_second_crash_point(&segments, &phases, 8, 2, &[0])));
    println!("F2 panic：扣下为空 {}", panics(|| exactly_one_second_crash_point(&segments, &phases, 3, 3, &[])));
    println!("F2 panic：扣下越界（3 在 3 个写里） {}", panics(|| exactly_one_second_crash_point(&segments, &phases, 3, 3, &[3])));
    println!("F2 panic：扣下一个在内一个越界 [0,3] {}", panics(|| exactly_one_second_crash_point(&segments, &phases, 3, 3, &[0, 3])));
    // 整段持久：窗口等于段长、扣下一个，等价于段内真子集一支
    let w = exactly_one_second_crash_point(&segments, &phases, 1, 14, &[0, 5]);
    println!("F2 段内真子集形态（段 1，14 个写，扣下 [0,5]）→ persisted 假的下标 {:?}", w.persisted_within_the_segment.iter().enumerate().filter(|(_, p)| !**p).map(|(i, _)| i).collect::<Vec<_>>());
}
