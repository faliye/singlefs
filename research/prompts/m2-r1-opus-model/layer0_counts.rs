//! 第三部分：层 0 全量状态数。闭式与 crates/singlefs-harness/src/crash.rs 的
//! `closed_form_state_count` 逐字同型：1 + Σ(2^|段| − 1)（D13 已定项 4 切段）。
//! 四个臂：里程碑步 0 自己的模型 / 空发布按 D16 已定项 9 写 c_max 单元 / 再加上
//! first-txn-layout.md 八末尾那条 ⚠️ 的超级块合并 / 并行线一那次 70 单元的发布。

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { UserVisible, Empty, Rollback, WriteRow }
use Kind::*;

/// (txg, 单元数/盘, 种类, 这次发布之前刚取过号)
const SCRIPT: &[(u64, usize, Kind, bool)] = &[
    (1, 0, Empty, true), (2, 0, Empty, false), (3, 8, UserVisible, false),
    (4, 8, UserVisible, false), (5, 8, WriteRow, true), (6, 5, Empty, false),
    (7, 5, Empty, false), (8, 8, UserVisible, false), (9, 8, Rollback, true),
    (10, 5, Empty, false), (11, 8, UserVisible, false), (12, 8, UserVisible, false),
    (13, 8, UserVisible, false), (14, 8, UserVisible, false), (15, 5, Empty, false),
    (16, 5, Empty, false), (17, 8, UserVisible, false),
];

fn closed_form(segments: &[usize]) -> u128 {
    1 + segments.iter().map(|n| (1u128 << n) - 1).sum::<u128>()
}

/// `empty_writes_units`：空发布写不写 c_max 个单元（D16 已定项 9）。
/// `merge_superblock`：上一次发布的超级块槽写与这次的单元写落不落同一段
/// （first-txn-layout.md 八末尾那条 ⚠️；里程碑步 0 的「基线 = 这次发布之前的全部写都持久」等于 false）。
fn segments(empty_writes_units: bool, merge_superblock: bool) -> Vec<usize> {
    let mut out = Vec::new();
    let mut carried = 0usize;
    for (_, units_per_device, kind, after_acquire) in SCRIPT {
        let mut writes = units_per_device * 2;
        if !empty_writes_units && matches!(kind, Empty) { writes = 0; }
        if *after_acquire { out.push(carried + 2); carried = 0; }   // [取号超级块 × 2] 屏障
        if writes == 0 {
            if carried > 0 { out.push(carried); carried = 0; }       // 零单元空发布开头有屏障
        } else {
            if merge_superblock {
                out.push(carried + writes);
            } else {
                if carried > 0 { out.push(carried); }
                out.push(writes);
            }
            carried = 0;
        }
        out.push(2);  // [记录 × 2 盘] 屏障
        out.push(1);  // [根 FUA]
        carried = 2;  // [超级块槽 × 2]
    }
    if carried > 0 { out.push(carried); }
    out
}

fn report(label: &str, segments: &[usize]) {
    let states = closed_form(segments);
    let shown: Vec<String> = segments.iter().map(|n| n.to_string()).collect();
    println!("{label}\n  段序列 {}\n  闭式状态数 {states}（第一个事务 262165 的 {:.1} 倍；按 22.7 s / 262165 个状态折算 {:.0} s）",
        shown.join("+"), states as f64 / 262165.0, states as f64 / 262165.0 * 22.7);
}

fn main() {
    println!("== 四、整条固定脚本的层 0 全量状态数（闭式 1 + Σ(2^|段| − 1)）\n");
    report("臂 A　里程碑步 0 自己的模型（空发布 2+1+2、每次发布独立基线 ⇒ 不并超级块）",
        &segments(false, false));
    report("臂 B　空发布按 D16 已定项 9 写 c_max = 5 个单元，仍不并超级块",
        &segments(true, false));
    report("臂 C　臂 B 再加上登记表末尾那条 ⚠️ 的超级块合并（与第一个事务 262165 同一种读法）",
        &segments(true, true));
    println!("\n  对照：第一个事务那一条流 2+2+1+2+2+1+18+2+1+2 ⇒ {}（E142 产物逐字 262165）",
        closed_form(&[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]));

    println!("\n== 五、并行线一「一次发布写 70 个单元（两条记录）进层 0」");
    // 段长超过 127 时 u128 的移位会回绕，大数一律走 f64 的 2^n
    let states_of = |unit_segment: u32| -> f64 {
        1.0 + (2f64.powi(unit_segment as i32) - 1.0) + 3.0 + 1.0 + 3.0
    };
    for units in [8u32, 16, 32, 67, 68, 70] {
        let unit_segment = units * 2;              // w ≥ 2，每个单元两次物理写
        let states = states_of(unit_segment);
        println!("  {units:>3} 个单元 ⇒ 单元段 {unit_segment} 次写 ⇒ 这一次发布的层 0 状态数 {states:.3e}\
　折算挂钟 {:.3e} 秒", states / 262165.0 * 22.7);
    }
}
