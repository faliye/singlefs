//! E207（B8严格判据下的步数）历史用例里量窗的几样取法：第 3 组声明行、W2 冷文件池按挂载状态劈开、探针前缀 P1、扩展窗口 W5–W7
//! 在哪几个点上做与取哪几个文件、阴性对照在哪几个点上做（跑前登记 `research/prompts/e207-preregistration.md` 第一节读法第 2 条、
//! 第三节第 3 组、第五节「量窗」与「阳性对照」）。纯算术：inode 号写成 u64，不 `use` 任何 `crates/` 里的东西。
//!
//! 两处用同一份：入口 `e207-strict-scan-steps-run.sh` 把它拷进副本当 `crates/singlefs-verification/tests/e207_window_choices/mod.rs`，
//! 历史用例 `e207_strict_scan_steps_histories.rs` 经 `#[path]` 引它；这里另编成 bin `e207-strict-scan-steps-window-choices`，只为单测与变异：
//! `cargo test --release --bin e207-strict-scan-steps-window-choices`，变异表 `scripts/mutation/tables/e207_strict_scan_steps_window_choices.tsv`
//! 经 `scripts/mutation/mutate.sh` 跑。bin 的入口什么都不做。
#![allow(dead_code, reason = "历史用例与单测各用其中一部分；当 bin 编时入口什么都不调")]

use std::collections::BTreeSet;

/// W2 第 i 次取池里第 `(i − 1) × 池长 ÷ 32 % 池长` 个。
pub const SHARD_FOR_THE_COLD_FILES: usize = 32;
/// 量窗 W1–W4 与 E206（发布固定点轮数的代价）逐字节相同取法的对账点（第五节「量窗」第 1 条）；别的点 W2 的冷文件池按挂载状态劈开。
pub const RECONCILED_WINDOW_POINTS: [&str; 3] = ["base", "files_1000", "files_10000"];
/// 第五节阴性对照的点。
pub const NEGATIVE_CONTROL_POINTS: [&str; 3] = ["base", "files_1000", "files_10000"];
/// 带扩展窗口 W5–W7 的点（第五节「量窗」第 3 条）。
pub const EXTENDED_WINDOW_POINTS: [&str; 2] = ["files_1000_units_100", "files_30000"];
/// 复量段 `hundred_units_per_file_repeat` 新加的四个点（登记设计员修订 2 的点表）：量窗是标准的 W1–W4，不进上面三张名单，W2 冷文件池按挂载状态劈开。
pub const HUNDRED_UNITS_PER_FILE_REPEAT_POINTS: [&str; 4] = ["files_56_units_100", "files_178_units_100", "files_316_units_100", "files_562_units_100"];
/// 第一个文件（inode 1）建时 3000 字节，一个数据单元以内。
pub const FIRST_FILE_UNITS: u64 = 1;

/// 第 3 组声明（读法第 2 条）：这一次操作点名的数据单元数，驱动按构造记账算，不读实现。一次操作含几次用户改动的（W3、W6 一批）取各次之和。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Declared {
    pub user_units: u64,
    pub old_units: u64,
    pub new_units: u64,
}

impl Declared {
    /// 覆盖写：换下的旧内容单元数（驱动记的 `old_units`）与新内容单元数（1 个）的较大者。
    pub fn overwrite(old_units: u64) -> Self {
        Self { user_units: old_units.max(1), old_units, new_units: 1 }
    }
    /// 建 inode：每个记 1，一批取各个之和。
    pub fn create(count: u64) -> Self {
        Self { user_units: count, old_units: 0, new_units: 0 }
    }
    /// 接着写：写进去的单元数。
    pub fn fill(old_units: u64, units: u64) -> Self {
        Self { user_units: units, old_units, new_units: old_units + units }
    }
    /// 截断到同样大小：|旧 − 新|，下限 1。
    pub fn truncate_to_the_same_size() -> Self {
        Self { user_units: 1, old_units: FIRST_FILE_UNITS, new_units: FIRST_FILE_UNITS }
    }
    /// W3 一次：建一个（1）、覆盖写这个空 inode（max(0, 1) = 1）、unlink 它（1 个单元），三次之和。
    pub fn create_write_unlink() -> Self {
        Self { user_units: 3, old_units: 0, new_units: 0 }
    }
}

/// 第 3 组声明行（模型 `e207_strict_scan_steps.rs` 按 `name=declared` 读回）。
pub fn declared_line(item: &str, labels: &str, declared: Declared) -> String {
    format!(
        "E207RESULT name=declared item={item} {labels} user_units={} old_units={} new_units={}",
        declared.user_units, declared.old_units, declared.new_units
    )
}

/// `live` 去掉大文件之后的小文件：先小后大的形状大文件是最后一个，别的形状是第 0 个。
pub fn small_files(live: &[u64], small_then_big: bool) -> &[u64] {
    if small_then_big {
        &live[..live.len() - 1]
    } else {
        &live[1..]
    }
}

/// W2 第 `index` 次（1 起）的冷文件。对账点照 E206 的取法（`live` 含大文件）；别的点把小文件池按挂载状态劈开（第五节「量窗」第 2 条）：
/// `same_mount` 取前 ⌈n ÷ 2⌉ 个、`remounted` 取其余，第 i 次取 `pool[(i − 1) × len ÷ 32 % len]`。
pub fn cold_file(live: &[u64], point_name: &str, small_then_big: bool, same_mount: bool, index: u64) -> u64 {
    let share = usize::try_from(index - 1).expect("下标");
    if RECONCILED_WINDOW_POINTS.contains(&point_name) {
        return live[share * live.len() / SHARD_FOR_THE_COLD_FILES % live.len()];
    }
    let small = small_files(live, small_then_big);
    let half = small.len().div_ceil(2);
    let pool: &[u64] = if same_mount { &small[..half] } else { &small[half..] };
    assert!(!pool.is_empty(), "{point_name} 的冷文件池是空的");
    pool[share * pool.len() / SHARD_FOR_THE_COLD_FILES % pool.len()]
}

/// 探针前缀 P1 做什么（第五节「量窗」第 4 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbePrefix {
    Nothing,
    Overwrite(u64),
    Create,
}

/// `cold_first` 覆盖写 `live[len ÷ 2]`（中间的冷文件）；`create_first` 建 1 个新 inode；`young_first` 覆盖写 `live` 里最后一个
/// （历史里最后建的小文件）；`standard` 不加。认不得的探针当场停。
pub fn probe_prefix(probe: &str, live: &[u64]) -> ProbePrefix {
    match probe {
        "standard" => ProbePrefix::Nothing,
        "cold_first" => ProbePrefix::Overwrite(live[live.len() / 2]),
        "create_first" => ProbePrefix::Create,
        "young_first" => ProbePrefix::Overwrite(*live.last().expect("有小文件")),
        other => panic!("探针认不得：{other}"),
    }
}

/// 扩展窗口只在带它的点、同一次挂载那一段做（接在 W4 之后、重挂载之前）。
pub fn extended_window_runs(state: &str, point_name: &str) -> bool {
    state == "same_mount" && EXTENDED_WINDOW_POINTS.contains(&point_name)
}

/// 阴性对照只在三个对账点上做。
pub fn negative_control_runs(point_name: &str) -> bool {
    NEGATIVE_CONTROL_POINTS.contains(&point_name)
}

/// 等间隔取 `count` 个：第 k 个取 `from[k × len ÷ count]`；不够 `count` 个交 `None`。
pub fn evenly_spaced(from: &[u64], count: usize) -> Option<Vec<u64>> {
    (from.len() >= count).then(|| (0..count).map(|position| from[position * from.len() / count]).collect())
}

/// W5 取文件的那一池：W2 没碰过的小文件，按号升序。
pub fn untouched_small_files(live: &[u64], small_then_big: bool, touched_by_the_cold_overwrites: &BTreeSet<u64>) -> Vec<u64> {
    let mut untouched: Vec<u64> = small_files(live, small_then_big).iter().copied().filter(|inode| !touched_by_the_cold_overwrites.contains(inode)).collect();
    untouched.sort_unstable();
    untouched
}

/// 历史那一段里大文件占的用户改动数（建 inode、写满，各记一次）。
pub const NOTES_OF_THE_BIG_FILE: u64 = 2;

/// 历史那一段的用户改动数（`History::note` 被叫的次数，产物 `name=history` 行的 `user_changes`）：大文件 2 次、每个小文件建与写各 1 次、
/// 每次删建与删各 1 次。只对 `BigThenSmall`、`SmallThenBig`、`SmallFilesOfUnits` 三种形状成立（登记第五节取样点表里的 21 项都是这三种）。
pub fn notes_of_the_history(files: u64, unlinks: u64) -> u64 {
    NOTES_OF_THE_BIG_FILE + 2 * files + 2 * unlinks
}

/// 分片（登记第十二节执行员修订 4）：第 `index` 片（0 起，共 `count` 片）管用户改动序号 `(from, to]`，`from = index × total ÷ count`，`to = (index + 1) × total ÷ count`；
/// 各片首尾相接、不重不漏，最后一片的 `to` 就是 `total`。
pub fn shard_interval(total: u64, index: u64, count: u64) -> (u64, u64) {
    assert!(count >= 1 && index < count, "片号 {index}/{count} 不成立");
    (index * total / count, (index + 1) * total / count)
}

/// `E207_SHARD` 的写法 `<第几片>/<共几片>`，片号 0 起；写坏的交 `None`。
pub fn parse_shard(text: &str) -> Option<(u64, u64)> {
    let (index, count) = text.split_once('/')?;
    let (index, count) = (index.parse::<u64>().ok()?, count.parse::<u64>().ok()?);
    (count >= 1 && index < count).then_some((index, count))
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 大文件 100 在第 0 个，十个小文件 1–10（`SmallFilesOfUnits` 的次序）。
    fn live_with_the_big_file_first() -> Vec<u64> {
        std::iter::once(100).chain(1..=10).collect()
    }

    #[test]
    fn an_overwrite_declares_the_larger_of_the_old_and_new_units() {
        assert_eq!(Declared::overwrite(1000), Declared { user_units: 1000, old_units: 1000, new_units: 1 });
        assert_eq!(Declared::overwrite(0), Declared { user_units: 1, old_units: 0, new_units: 1 }, "空 inode 覆盖写记新内容的 1 个单元");
        assert_eq!(Declared::fill(1, 100), Declared { user_units: 100, old_units: 1, new_units: 101 });
        assert_eq!(Declared::truncate_to_the_same_size(), Declared { user_units: 1, old_units: 1, new_units: 1 });
    }

    #[test]
    fn one_batch_declares_the_sum_of_its_changes() {
        assert_eq!(Declared::create_write_unlink().user_units, 1 + 1 + 1, "W3 一次建、写、删各 1");
        assert_eq!(Declared::create(10).user_units, 10, "W6 一批建 10 个");
    }

    #[test]
    fn the_declared_line_carries_the_three_unit_counts() {
        let line = declared_line("files_10_units_1000__head", "phase=window state=same_mount operation=W2 operation_index=1", Declared::overwrite(1000));
        assert_eq!(
            line,
            "E207RESULT name=declared item=files_10_units_1000__head phase=window state=same_mount operation=W2 operation_index=1 user_units=1000 old_units=1000 new_units=1"
        );
    }

    #[test]
    fn the_cold_pool_is_split_by_the_mount_state() {
        let live = live_with_the_big_file_first();
        let picks = |same_mount: bool| -> BTreeSet<u64> { (1..=32).map(|index| cold_file(&live, "files_10_units_1000", false, same_mount, index)).collect() };
        assert_eq!(picks(true), (1..=5).collect::<BTreeSet<u64>>(), "同一次挂载取前一半，每个都碰到");
        assert_eq!(picks(false), (6..=10).collect::<BTreeSet<u64>>(), "重挂载之后取后一半");
        let small_then_big: Vec<u64> = (1..=10).chain(std::iter::once(100)).collect();
        assert_eq!(cold_file(&small_then_big, "files_then_big_1000", true, false, 1), 6, "先小后大的大文件在最后、不进池：十个小文件劈两半，重挂载那一半从 6 起");
        assert_eq!(cold_file(&live, "base", false, true, 1), 100, "对账点照 E206 的取法，第 1 次取 live[0]");
        assert_eq!(cold_file(&live, "base", false, false, 4), live[3 * 11 / 32], "对账点两种状态取法相同");
    }

    #[test]
    fn the_probe_prefix_picks_the_registered_target() {
        let live = live_with_the_big_file_first();
        assert_eq!(probe_prefix("standard", &live), ProbePrefix::Nothing);
        assert_eq!(probe_prefix("cold_first", &live), ProbePrefix::Overwrite(5), "live[11 ÷ 2] = live[5]");
        assert_eq!(probe_prefix("create_first", &live), ProbePrefix::Create);
        assert_eq!(probe_prefix("young_first", &live), ProbePrefix::Overwrite(10), "最后建的小文件");
    }

    #[test]
    fn the_extended_window_runs_on_two_points_in_the_same_mount() {
        assert!(extended_window_runs("same_mount", "files_1000_units_100"));
        assert!(extended_window_runs("same_mount", "files_30000"));
        assert!(!extended_window_runs("remounted", "files_1000_units_100"), "重挂载那一段不加");
        assert!(!extended_window_runs("same_mount", "files_100_units_100"));
    }

    #[test]
    fn the_extended_window_takes_evenly_spaced_untouched_files() {
        let ten: Vec<u64> = (1..=10).collect();
        assert_eq!(evenly_spaced(&ten, 5), Some(vec![1, 3, 5, 7, 9]));
        assert_eq!(evenly_spaced(&[1, 2], 5), None, "不够就交 None，历史用例当场停");
        assert_eq!(untouched_small_files(&[100, 3, 1, 2], false, &BTreeSet::from([1])), vec![2, 3], "去掉大文件与 W2 碰过的，按号升序");
    }

    #[test]
    fn the_negative_control_runs_on_the_three_reconciled_points() {
        assert!(negative_control_runs("base") && negative_control_runs("files_1000") && negative_control_runs("files_10000"));
        assert!(!negative_control_runs("files_1000_units_100"));
    }

    #[test]
    fn the_history_notes_match_the_registered_products() {
        assert_eq!(notes_of_the_history(10, 10), 42, "base 的产物 user_changes=42");
        assert_eq!(notes_of_the_history(100, 10), 222, "files_100_units_1000 的产物 user_changes=222");
        assert_eq!(notes_of_the_history(1_000, 10), 2_022, "files_1000 的产物 user_changes=2022");
        assert_eq!(notes_of_the_history(10_000, 10), 20_022, "files_10000 的产物 user_changes=20022");
    }

    #[test]
    fn the_hundred_units_per_file_repeat_points_run_the_standard_window_with_a_split_cold_pool() {
        for point in HUNDRED_UNITS_PER_FILE_REPEAT_POINTS {
            assert!(!RECONCILED_WINDOW_POINTS.contains(&point), "{point} 的 W2 冷文件池要劈开，不照 E206 的取法");
            assert!(!EXTENDED_WINDOW_POINTS.contains(&point), "{point} 没有扩展窗口（F 不到 250 凑不出 200 个没碰过的冷文件）");
            assert!(!NEGATIVE_CONTROL_POINTS.contains(&point), "{point} 没有阴性对照");
            assert!(!extended_window_runs("same_mount", point) && !negative_control_runs(point));
        }
        // F = 56：大文件在 live[0]，小文件 live[1..=56]；同一次挂载取前 28 个，重挂载取后 28 个，各自取满 32 次里的不同文件
        let live: Vec<u64> = std::iter::once(100).chain(1..=56).collect();
        let picks = |same_mount: bool| -> BTreeSet<u64> { (1..=32).map(|index| cold_file(&live, "files_56_units_100", false, same_mount, index)).collect() };
        assert!(picks(true).iter().all(|inode| (1..=28).contains(inode)) && picks(false).iter().all(|inode| (29..=56).contains(inode)));
        assert_eq!(picks(true).len(), 28, "28 个文件的池 32 次取，每个文件都碰到");
        // 发布数的式子对四个点给出登记写的约数
        assert_eq!([56, 178, 316, 562].map(|files| notes_of_the_history(files, 10)), [134, 378, 654, 1_146]);
    }

    #[test]
    fn the_shard_intervals_meet_end_to_end_without_gap_or_overlap() {
        for total in [42, 222, 2_022, 20_022] {
            for count in 1..=7 {
                let mut next_from = 0;
                for index in 0..count {
                    let (from, to) = shard_interval(total, index, count);
                    assert_eq!(from, next_from, "{total} 分 {count} 片，第 {index} 片该从上一片的末尾起");
                    assert!(to > from, "{total} 分 {count} 片，第 {index} 片不该是空的");
                    next_from = to;
                }
                assert_eq!(next_from, total, "{total} 分 {count} 片，最后一片该到总数");
            }
        }
        assert_eq!(shard_interval(2_022, 1, 3), (674, 1_348), "手推：2022 ÷ 3 = 674");
    }

    #[test]
    fn the_shard_text_is_index_over_count() {
        assert_eq!(parse_shard("0/1"), Some((0, 1)));
        assert_eq!(parse_shard("2/3"), Some((2, 3)));
        assert_eq!(parse_shard("3/3"), None, "片号 0 起，等于片数不成立");
        assert_eq!(parse_shard("0/0"), None);
        assert_eq!(parse_shard("1"), None);
        assert_eq!(parse_shard("a/b"), None);
    }
}
