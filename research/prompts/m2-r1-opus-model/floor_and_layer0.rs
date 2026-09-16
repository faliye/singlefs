//! 第二部分：抬 F 的上限、回退候选集、E 时分配器发哪个落点，以及层 0 全量的状态数。
//! 与 fixed_script.rs 同一条脚本（那份跑出来的表逐行抄进 SCRIPT）。只用 std。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind { UserVisible, Empty, Rollback, WriteRow }

/// (txg, 实例, 单元数/盘, 种类, 这次发布之前刚取过号)
const SCRIPT: &[(u64, u32, usize, Kind, bool)] = &[
    (1, 1, 0, Kind::Empty, true),
    (2, 1, 0, Kind::Empty, false),
    (3, 1, 8, Kind::UserVisible, false),
    (4, 1, 8, Kind::UserVisible, false),
    (5, 2, 8, Kind::WriteRow, true),
    (6, 2, 5, Kind::Empty, false),
    (7, 2, 5, Kind::Empty, false),
    (8, 2, 8, Kind::UserVisible, false),
    (9, 3, 8, Kind::Rollback, true),
    (10, 3, 5, Kind::Empty, false),
    (11, 3, 8, Kind::UserVisible, false),
    (12, 3, 8, Kind::UserVisible, false),
    (13, 3, 8, Kind::UserVisible, false),
    (14, 3, 8, Kind::UserVisible, false),
    (15, 3, 5, Kind::Empty, false),
    (16, 3, 5, Kind::Empty, false),
    (17, 3, 8, Kind::UserVisible, false),
];

fn device_of_txg(txg: u64) -> u32 { match txg % 3 { 0 => 0, 1 => 1, _ => 0 } }

/// D23 已定项 14 逐字：(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti。
/// 回退之后表上有回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)；实例 0（mkfs 的种子根）无行。
fn still_valid_after_rollback(instance: u32, txg: u64) -> bool {
    match instance {
        0 => true,                 // 无行
        1 => txg <= 3,             // 回退行 (1, 3, 0)
        2 => txg <= 0,             // 中间实例行 (2, 0, 0) ⇒ 一条都选不中
        _ => true,                 // 实例 3 无行
    }
}

fn main() {
    // ---- 抬 F：那次空发布在 txg 15，此刻持久的根是 txg ≤ 14 ----
    let raise_at = 15u64;
    let mut roots: Vec<(u64, u32, Kind)> = SCRIPT.iter()
        .filter(|(txg, ..)| *txg < raise_at)
        .map(|(txg, instance, _, kind, _)| (*txg, *instance, *kind))
        .collect();
    roots.push((0, 0, Kind::UserVisible)); // mkfs 的种子根，区域 0 槽 0 到 txg 24 才被覆写
    let valid: Vec<_> = roots.iter().copied()
        .filter(|(txg, instance, _)| still_valid_after_rollback(*instance, *txg)).collect();
    println!("== 三、抬 F 那一刻（txg {raise_at} 的空发布之前）根环里按实例表判仍然有效的根");
    for (txg, instance, kind) in &valid {
        println!("   txg {txg:>2} 实例 {instance} 盘 {} {:?}", device_of_txg(*txg), kind);
    }
    let newest_on = |device: u32| valid.iter().filter(|(txg, ..)| device_of_txg(*txg) == device)
        .map(|(txg, ..)| *txg).max().unwrap();
    let per_device_min = newest_on(0).min(newest_on(1));
    println!("   每块盘上最新的持久有效根：盘 0 = {}，盘 1 = {} ⇒ min = {per_device_min}", newest_on(0), newest_on(1));

    // 「4」只数改过用户可见状态的根；三种读法都算一遍，看结论稳不稳（mutation-sampling.md 第六类）
    for (label, counts) in [
        ("只数 UserVisible", &[Kind::UserVisible][..]),
        ("UserVisible + 回退那次", &[Kind::UserVisible, Kind::Rollback][..]),
        ("除空发布之外全算（含写行那次）", &[Kind::UserVisible, Kind::Rollback, Kind::WriteRow][..]),
    ] {
        let mut non_empty: Vec<u64> = valid.iter()
            .filter(|(_, _, kind)| counts.contains(kind)).map(|(txg, ..)| *txg).collect();
        non_empty.sort_unstable(); non_empty.reverse();
        let fourth = non_empty.get(3).copied();
        println!("   非空有效根（{label}）由新到旧 {non_empty:?} ⇒ 第 4 新 = {fourth:?}");
    }
    let fourth_newest_non_empty = 11u64;
    let release_generation_of_50180 = 11u64;
    let ceiling = per_device_min.min(fourth_newest_non_empty);
    let target = release_generation_of_50180.min(fourth_newest_non_empty);
    println!("   ⇒ 抬 F 的上限 = min({per_device_min}, {fourth_newest_non_empty}) = {ceiling}；一次处置的目标 = min({release_generation_of_50180}, {fourth_newest_non_empty}) = {target}");
    assert!(target <= ceiling, "目标超过上限，步 5 走不下去");
}
