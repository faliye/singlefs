//! E209（尖峰成因）历史用例与模型共用的纯算术：取样点表、脚本表（每个脚本是一串步）、选择子怎么落到文件、声明行的写法、项名单。
//! 不 `use` 任何 `crates/` 里的东西：inode 号写成 u64。
//!
//! 两处用同一份：入口 `e209-spike-cause-run.sh` 把它拷进副本当 `crates/singlefs-verification/tests/e209_scripts/mod.rs`，
//! 历史用例 `e209_spike_cause_histories.rs` 经 `#[path]` 引它；模型 `e209_spike_cause.rs` 经 `#[path]` 引同一份，单测在模型那个 bin 与它自己的 bin 里各跑一份：
//! `cargo test --release --bin e209-spike-cause`、`cargo test --release --bin e209-spike-cause-scripts`；变异表 `scripts/mutation/tables/e209_spike_cause_scripts.tsv` 经后一个 bin 跑。
#![allow(dead_code, reason = "历史用例与模型各用其中一部分")]

use std::collections::BTreeSet;

/// 一个取样点：单元区每盘槽数的二次幂、小文件个数、每个小文件的数据单元数（跑前登记第五节「取样点」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SamplePoint {
    pub unit_area_exponent: u32,
    pub files: u64,
    pub units_per_file: u64,
}

/// 取样点表：名字与几何，一行一个点。
pub const SAMPLE_POINTS: [(&str, SamplePoint); 8] = [
    ("files_100_units_100", SamplePoint { unit_area_exponent: 19, files: 100, units_per_file: 100 }),
    ("files_75_units_100", SamplePoint { unit_area_exponent: 19, files: 75, units_per_file: 100 }),
    ("files_200_units_100", SamplePoint { unit_area_exponent: 19, files: 200, units_per_file: 100 }),
    ("files_100_units_100_unit_area_2_pow_17", SamplePoint { unit_area_exponent: 17, files: 100, units_per_file: 100 }),
    ("files_100_units_100_unit_area_2_pow_20", SamplePoint { unit_area_exponent: 20, files: 100, units_per_file: 100 }),
    ("files_10_units_1000", SamplePoint { unit_area_exponent: 19, files: 10, units_per_file: 1000 }),
    ("files_250_units_100", SamplePoint { unit_area_exponent: 19, files: 250, units_per_file: 100 }),
    ("files_300_units_100", SamplePoint { unit_area_exponent: 19, files: 300, units_per_file: 100 }),
];

/// 训练点。
pub const TRAINING_POINT: &str = "files_100_units_100";
/// 历史里 unlink 的次数（每次删建一个空 inode 再删，建与删各记一次用户改动）。
pub const UNLINKS_OF_THE_HISTORY: u64 = 10;
/// 历史里大文件占的用户改动数（建 inode、写满，各记一次）。
pub const NOTES_OF_THE_BIG_FILE: u64 = 2;
/// 历史里大文件的数据单元数。
pub const BIG_FILE_UNITS: u64 = 10;
/// 第一个文件（inode 1）建时 3000 字节，一个数据单元以内。
pub const FIRST_FILE_UNITS: u64 = 1;
/// `cold_j` 取的冷文件池大小（`presweep_m` 的 m 至多 64，池与 m 无关，所以 `cold_j` 不随 m 变）。
pub const COLD_FILE_POOL: usize = 64;

pub fn sample_point_named(name: &str) -> Option<SamplePoint> {
    SAMPLE_POINTS.iter().find(|(candidate, _)| *candidate == name).map(|(_, point)| *point)
}

/// 历史那一段的用户改动数（产物 `name=history` 行的 `user_changes`）：大文件 2 次、每个小文件建与写各 1 次、每次删建与删各 1 次。
pub fn notes_of_the_history(files: u64) -> u64 {
    NOTES_OF_THE_BIG_FILE + 2 * files + 2 * UNLINKS_OF_THE_HISTORY
}

/// 选择子：脚本里一步覆盖写哪个文件。`live` 是历史建好的 inode 按建的次序，第 0 个是历史里的大文件，其余是小文件（`n = F`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selector {
    First,
    Old,
    Mid,
    Late,
    Young,
    YoungMinus(u64),
    Cold(u64),
}

impl Selector {
    pub fn text(self) -> String {
        match self {
            Self::First => "first".to_string(),
            Self::Old => "old".to_string(),
            Self::Mid => "mid".to_string(),
            Self::Late => "late".to_string(),
            Self::Young => "young".to_string(),
            Self::YoungMinus(distance) => format!("young_minus_{distance}"),
            Self::Cold(number) => format!("cold_{number}"),
        }
    }
}

/// 脚本里的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Overwrite(Selector),
    CreateOne,
    Remount,
    TruncateToTheSameSize,
}

impl Step {
    /// 声明行与 `script_step` 行里 `operation=` 的写法。
    pub fn text(self) -> String {
        match self {
            Self::Overwrite(selector) => format!("overwrite_{}", selector.text()),
            Self::CreateOne => "create_one".to_string(),
            Self::Remount => "remount".to_string(),
            Self::TruncateToTheSameSize => "truncate_to_the_same_size".to_string(),
        }
    }

    /// 是被量的操作，即有声明行与固定点读数的用户操作：重挂载不是。
    pub fn is_a_user_operation(self) -> bool {
        !matches!(self, Self::Remount)
    }

    /// 选择子的名字：覆盖写是选择子，新建是 `new_file`，截断是 `truncate`，重挂载是 `remount`。
    pub fn selector_text(self) -> String {
        match self {
            Self::Overwrite(selector) => selector.text(),
            Self::CreateOne => "new_file".to_string(),
            Self::Remount => "remount".to_string(),
            Self::TruncateToTheSameSize => "truncate".to_string(),
        }
    }
}

fn overwrites(selectors: &[Selector]) -> Vec<Step> {
    selectors.iter().map(|selector| Step::Overwrite(*selector)).collect()
}

fn twice(steps: Vec<Step>) -> Vec<Step> {
    let mut doubled = steps.clone();
    doubled.extend(steps);
    doubled
}

/// 脚本名里带的数（`filler_create_4` 的 4）。
fn number_after(name: &str, prefix: &str) -> Option<u64> {
    name.strip_prefix(prefix)?.parse().ok()
}

/// 脚本表（跑前登记第五节「脚本」）：脚本名 → 步。认不得的名字交 `None`。
pub fn script_steps(name: &str) -> Option<Vec<Step>> {
    use Selector::{Cold, First, Late, Mid, Old, Young, YoungMinus};
    let steps = match name {
        "walk_forward" => twice(overwrites(&[First, Old, Mid, Late, Young])),
        "walk_backward" => twice(overwrites(&[Young, Late, Mid, Old, First])),
        "walk_from_middle" => twice(overwrites(&[Mid, First, Young, Old, Late])),
        "first_repeated" => {
            let mut steps = overwrites(&[First; 6]);
            steps.push(Step::TruncateToTheSameSize);
            steps
        }
        "create_then_first" => {
            let mut steps = vec![Step::CreateOne];
            steps.extend(overwrites(&[First; 3]));
            steps
        }
        "mount_then_first" => {
            let mut steps = vec![Step::Remount];
            steps.extend(overwrites(&[First, Old, Mid, Late, Young]));
            steps
        }
        "first_mount_first" => {
            let mut steps = overwrites(&[First, Old]);
            steps.push(Step::Remount);
            steps.extend(overwrites(&[First, Old, Mid]));
            steps
        }
        "mount_twice_first" => {
            let mut steps = vec![Step::Remount, Step::Remount];
            steps.extend(overwrites(&[First, Old, Mid]));
            steps
        }
        "young_mount_first" => {
            let mut steps = overwrites(&[Young, Late]);
            steps.push(Step::Remount);
            steps.extend(overwrites(&[First, Old, Mid]));
            steps
        }
        "old_mount_old" => {
            let mut steps = overwrites(&[Old]);
            steps.push(Step::Remount);
            steps.extend(overwrites(&[Old, First]));
            steps
        }
        other => {
            let tail = overwrites(&[First, Old, Mid]);
            if let Some(count) = number_after(other, "filler_create_") {
                let mut steps = vec![Step::CreateOne; usize::try_from(count).ok()?];
                steps.extend(tail);
                steps
            } else if let Some(count) = number_after(other, "filler_young_") {
                let mut steps: Vec<Step> = (0..count).map(|distance| Step::Overwrite(YoungMinus(distance))).collect();
                steps.extend(tail);
                steps
            } else {
                let count = number_after(other, "presweep_")?;
                let mut steps: Vec<Step> = (1..=count).map(|number| Step::Overwrite(Cold(number))).collect();
                steps.extend(tail);
                steps
            }
        }
    };
    Some(steps)
}

/// 训练点上的 25 个脚本（跑前登记第五节「项数」）。
pub fn training_scripts() -> Vec<String> {
    let mut names: Vec<String> = ["walk_forward", "walk_backward", "walk_from_middle", "first_repeated", "create_then_first"].map(str::to_string).to_vec();
    names.extend([1, 2, 4, 8, 16].map(|count| format!("filler_create_{count}")));
    names.extend([1, 4, 8].map(|count| format!("filler_young_{count}")));
    names.extend([1, 2, 4, 8, 16, 32, 64].map(|count| format!("presweep_{count}")));
    names.extend(["mount_then_first", "first_mount_first", "mount_twice_first", "young_mount_first", "old_mount_old"].map(str::to_string));
    names
}

/// 检验点上的脚本：`files_10_units_1000` 去掉两个 `presweep`（冷文件不够，分母 0，照报）。
pub fn test_geometry_scripts(point: &str) -> Vec<String> {
    let mut names: Vec<String> = [
        "walk_forward",
        "walk_backward",
        "first_repeated",
        "filler_create_4",
        "filler_young_4",
        "presweep_8",
        "presweep_32",
        "mount_then_first",
        "first_mount_first",
        "mount_twice_first",
    ]
    .map(str::to_string)
    .to_vec();
    if point == "files_10_units_1000" {
        names.retain(|name| !name.starts_with("presweep_"));
    }
    names
}

/// 登记里写死的检验点（第三段）。
pub const TEST_GEOMETRY_POINTS: [&str; 5] = [
    "files_75_units_100",
    "files_200_units_100",
    "files_100_units_100_unit_area_2_pow_17",
    "files_100_units_100_unit_area_2_pow_20",
    "files_10_units_1000",
];

/// 项名：`<点>__<臂>__<脚本>`，臂是 `head`（今天的实现，带读数）或 `plain`（只拷不插，只比盘）。
pub fn item_name(point: &str, arm: &str, script: &str) -> String {
    format!("{point}__{arm}__{script}")
}

/// 拆项名；认不得交 `None`。
pub fn split_item_name(item: &str) -> Option<(&str, &str, &str)> {
    let (point, rest) = item.split_once("__")?;
    let (arm, script) = rest.split_once("__")?;
    (matches!(arm, "head" | "plain") && sample_point_named(point).is_some() && script_steps(script).is_some()).then_some((point, arm, script))
}

/// 全部登记的项（项名、所属段）：第一段、第二段、第三段、第四段与第一段的闸。
pub fn registered_items() -> Vec<(String, &'static str)> {
    let mut items = Vec::new();
    let first_segment = ["walk_forward", "first_repeated", "mount_then_first", "first_mount_first", "mount_twice_first"];
    for script in training_scripts() {
        let segment = if first_segment.contains(&script.as_str()) { "first" } else { "second" };
        items.push((item_name(TRAINING_POINT, "head", &script), segment));
    }
    items.push((item_name(TRAINING_POINT, "plain", "walk_forward"), "first"));
    for point in TEST_GEOMETRY_POINTS {
        for script in test_geometry_scripts(point) {
            let gate = point == "files_200_units_100" && script == "walk_forward";
            items.push((item_name(point, "head", &script), if gate { "gate" } else { "third" }));
        }
    }
    for point in ["files_250_units_100", "files_300_units_100"] {
        for script in ["walk_forward", "mount_then_first"] {
            items.push((item_name(point, "head", script), "fourth"));
        }
    }
    items
}

/// 把 `live` 里去掉大文件之后的小文件（大文件是第 0 个）。
pub fn small_files(live: &[u64]) -> &[u64] {
    &live[1..]
}

/// 等间隔取 `count` 个：第 k 个取 `from[k × len ÷ count]`；不够 `count` 个交 `None`。
pub fn evenly_spaced(from: &[u64], count: usize) -> Option<Vec<u64>> {
    (from.len() >= count).then(|| (0..count).map(|position| from[position * from.len() / count]).collect())
}

/// 选择子落到哪个 inode：`live` 第 0 个是大文件，`first` 是 inode 1。
pub fn resolve_selector(selector: Selector, live: &[u64]) -> u64 {
    let small = small_files(live);
    let files = small.len();
    match selector {
        Selector::First => 1,
        Selector::Old => small[files / 8],
        Selector::Mid => small[files / 2],
        Selector::Late => small[7 * files / 8],
        Selector::Young => small[files - 1],
        Selector::YoungMinus(distance) => small[files - 1 - usize::try_from(distance).expect("距离")],
        Selector::Cold(number) => {
            let taken: BTreeSet<u64> = [Selector::Old, Selector::Mid, Selector::Late, Selector::Young].map(|chosen| resolve_selector(chosen, live)).into_iter().collect();
            let mut rest: Vec<u64> = small.iter().copied().filter(|inode| !taken.contains(inode)).collect();
            rest.sort_unstable();
            let pool = evenly_spaced(&rest, COLD_FILE_POOL).unwrap_or_else(|| panic!("去掉四个选择子之后只剩 {} 个冷文件，不够 {COLD_FILE_POOL} 个", rest.len()));
            pool[usize::try_from(number).expect("序号") - 1]
        }
    }
}

/// 声明（读法：驱动按构造记账，不读实现）：这一次操作点名的数据单元数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Declared {
    pub user_units: u64,
    pub old_units: u64,
    pub new_units: u64,
}

impl Declared {
    /// 覆盖写：换下的旧内容单元数与新内容单元数（1 个）的较大者。
    pub fn overwrite(old_units: u64) -> Self {
        Self { user_units: old_units.max(1), old_units, new_units: 1 }
    }
    /// 建 inode。
    pub fn create(count: u64) -> Self {
        Self { user_units: count, old_units: 0, new_units: 0 }
    }
    /// 截断到同样大小：|旧 − 新|，下限 1。
    pub fn truncate_to_the_same_size() -> Self {
        Self { user_units: 1, old_units: FIRST_FILE_UNITS, new_units: FIRST_FILE_UNITS }
    }
}

/// 声明行（模型按 `name=declared` 读回）。
pub fn declared_line(item: &str, labels: &str, declared: Declared) -> String {
    format!(
        "E209RESULT name=declared item={item} {labels} user_units={} old_units={} new_units={}",
        declared.user_units, declared.old_units, declared.new_units
    )
}

/// 一步开始之前打的一行（重挂载也打）：模型拿它与脚本表逐步比。
pub fn script_step_line(item: &str, step_index: u64, step: Step, mount_epoch: u64) -> String {
    format!(
        "E209RESULT name=script_step item={item} step_index={step_index} operation={} mount_epoch={mount_epoch}",
        step.text()
    )
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn live_with_the_big_file_first(small_files: u64) -> Vec<u64> {
        std::iter::once(1000).chain(1..=small_files).collect()
    }

    #[test]
    fn the_history_notes_are_two_plus_twice_the_files_plus_twenty() {
        let expected = [(10, 42), (75, 172), (100, 222), (200, 422), (300, 622)];
        for (files, notes) in expected {
            assert_eq!(notes_of_the_history(files), notes, "F = {files}");
        }
    }

    #[test]
    fn the_training_point_has_twenty_five_scripts_and_the_test_points_ten_or_eight() {
        assert_eq!(training_scripts().len(), 25);
        assert_eq!(test_geometry_scripts("files_75_units_100").len(), 10);
        assert_eq!(test_geometry_scripts("files_10_units_1000").len(), 8);
    }

    #[test]
    fn the_registry_has_seventy_three_head_items_before_the_fourth_segment_and_one_plain_item() {
        let items = registered_items();
        let head_items_of_the_registry = items.iter().filter(|(name, segment)| name.contains("__head__") && *segment != "fourth").count();
        assert_eq!(head_items_of_the_registry, 25 + 48, "训练点 25 项加检验点 48 项（含闸那一项）");
        assert_eq!(items.iter().filter(|(name, _)| name.contains("__plain__")).count(), 1);
        let names: BTreeSet<&String> = items.iter().map(|(name, _)| name).collect();
        assert_eq!(names.len(), items.len(), "项名没有重复");
        assert!(items.iter().all(|(name, _)| split_item_name(name).is_some()));
    }

    #[test]
    fn every_script_has_the_registered_number_of_steps() {
        let lengths = [
            ("walk_forward", 10),
            ("walk_backward", 10),
            ("walk_from_middle", 10),
            ("first_repeated", 7),
            ("create_then_first", 4),
            ("filler_create_16", 19),
            ("filler_young_8", 11),
            ("presweep_64", 67),
            ("mount_then_first", 6),
            ("first_mount_first", 6),
            ("mount_twice_first", 5),
            ("young_mount_first", 6),
            ("old_mount_old", 4),
        ];
        for (name, length) in lengths {
            assert_eq!(script_steps(name).expect(name).len(), length, "{name}");
        }
        assert_eq!(script_steps("no_such_script"), None);
    }

    #[test]
    fn walk_backward_is_walk_forward_reversed_in_one_round() {
        let forward: Vec<String> = script_steps("walk_forward").unwrap().iter().map(|step| step.text()).collect();
        let backward: Vec<String> = script_steps("walk_backward").unwrap().iter().map(|step| step.text()).collect();
        assert_eq!(forward[..5], ["overwrite_first", "overwrite_old", "overwrite_mid", "overwrite_late", "overwrite_young"]);
        assert_eq!(backward[..5], ["overwrite_young", "overwrite_late", "overwrite_mid", "overwrite_old", "overwrite_first"]);
        assert_eq!(forward[..5], forward[5..]);
    }

    #[test]
    fn the_selectors_land_on_the_registered_small_files() {
        let live = live_with_the_big_file_first(100);
        assert_eq!(resolve_selector(Selector::First, &live), 1);
        assert_eq!(resolve_selector(Selector::Old, &live), 13, "small[100 ÷ 8] = small[12]，小文件号从 1 起所以是 13");
        assert_eq!(resolve_selector(Selector::Mid, &live), 51);
        assert_eq!(resolve_selector(Selector::Late, &live), 88);
        assert_eq!(resolve_selector(Selector::Young, &live), 100);
        assert_eq!(resolve_selector(Selector::YoungMinus(3), &live), 97);
    }

    #[test]
    fn the_cold_files_skip_the_four_named_ones_and_do_not_depend_on_how_many_are_taken() {
        let live = live_with_the_big_file_first(100);
        let named = [13, 51, 88, 100];
        let cold: Vec<u64> = (1..=64).map(|number| resolve_selector(Selector::Cold(number), &live)).collect();
        assert!(cold.iter().all(|inode| !named.contains(inode)));
        assert_eq!(cold.iter().collect::<BTreeSet<_>>().len(), 64, "64 个互不相同");
        assert_eq!(resolve_selector(Selector::Cold(1), &live), 1, "去掉四个之后第一个是小文件 1，等间隔取第 0 个");
        assert!(cold.windows(2).all(|pair| pair[0] < pair[1]), "按下标升序");
    }

    #[test]
    fn the_declared_line_carries_the_three_unit_counts() {
        let line = declared_line("files_10_units_1000__head__walk_forward", "phase=script mount_epoch=0 operation=overwrite_old operation_index=2", Declared::overwrite(1000));
        assert_eq!(
            line,
            "E209RESULT name=declared item=files_10_units_1000__head__walk_forward phase=script mount_epoch=0 operation=overwrite_old operation_index=2 user_units=1000 old_units=1000 new_units=1"
        );
        assert_eq!(Declared::overwrite(0), Declared { user_units: 1, old_units: 0, new_units: 1 });
        assert_eq!(Declared::truncate_to_the_same_size(), Declared { user_units: 1, old_units: 1, new_units: 1 });
    }

    #[test]
    fn the_script_step_line_names_the_step_and_the_mount() {
        assert_eq!(
            script_step_line("a__head__b", 3, Step::Remount, 1),
            "E209RESULT name=script_step item=a__head__b step_index=3 operation=remount mount_epoch=1"
        );
    }

    #[test]
    fn the_filler_scripts_name_their_fillers_before_the_tail_steps() {
        let texts = |name: &str| -> Vec<String> { script_steps(name).expect(name).iter().map(|step| step.text()).collect() };
        assert_eq!(texts("presweep_2"), ["overwrite_cold_1", "overwrite_cold_2", "overwrite_first", "overwrite_old", "overwrite_mid"]);
        assert_eq!(texts("filler_young_2"), ["overwrite_young_minus_0", "overwrite_young_minus_1", "overwrite_first", "overwrite_old", "overwrite_mid"]);
        assert_eq!(texts("filler_create_2"), ["create_one", "create_one", "overwrite_first", "overwrite_old", "overwrite_mid"]);
        assert_eq!(texts("first_repeated").last().map(String::as_str), Some("truncate_to_the_same_size"));
    }

    #[test]
    fn the_cold_files_are_spaced_evenly_over_what_is_left() {
        let live = live_with_the_big_file_first(100);
        assert_eq!(resolve_selector(Selector::Cold(2), &live), 2, "剩 96 个，第 2 个取下标 1");
        assert_eq!(resolve_selector(Selector::Cold(33), &live), 50, "下标 32 × 96 ÷ 64 = 48，去掉 13 之后第 48 位是 50");
        assert_eq!(resolve_selector(Selector::Cold(64), &live), 98, "下标 63 × 96 ÷ 64 = 94，去掉四个之后第 94 位是 98");
    }

    #[test]
    fn the_registry_segments_hold_the_registered_item_counts() {
        let items = registered_items();
        let in_segment = |segment: &str| items.iter().filter(|(_, name)| *name == segment).count();
        assert_eq!(in_segment("first"), 6, "训练点 5 项加 plain 1 项");
        assert_eq!(in_segment("second"), 20);
        assert_eq!(in_segment("third"), 47);
        assert_eq!(in_segment("gate"), 1);
        assert_eq!(in_segment("fourth"), 4);
        assert_eq!(items.iter().find(|(_, segment)| *segment == "gate").map(|(name, _)| name.as_str()), Some("files_200_units_100__head__walk_forward"));
        assert_eq!(split_item_name("files_100_units_100__head__walk_forward"), Some(("files_100_units_100", "head", "walk_forward")));
        assert_eq!(split_item_name("files_100_units_100__tail__walk_forward"), None);
    }

    #[test]
    fn the_sample_points_carry_the_registered_geometry() {
        let wanted = [
            ("files_100_units_100", 19, 100, 100),
            ("files_75_units_100", 19, 75, 100),
            ("files_200_units_100", 19, 200, 100),
            ("files_100_units_100_unit_area_2_pow_17", 17, 100, 100),
            ("files_100_units_100_unit_area_2_pow_20", 20, 100, 100),
            ("files_10_units_1000", 19, 10, 1000),
            ("files_250_units_100", 19, 250, 100),
            ("files_300_units_100", 19, 300, 100),
        ];
        for (name, unit_area_exponent, files, units_per_file) in wanted {
            assert_eq!(sample_point_named(name), Some(SamplePoint { unit_area_exponent, files, units_per_file }), "{name}");
        }
        assert_eq!(sample_point_named("files_1_units_1"), None);
    }

    #[test]
    fn the_create_declaration_names_the_count_and_the_remount_is_not_a_user_operation() {
        assert_eq!(Declared::create(3), Declared { user_units: 3, old_units: 0, new_units: 0 });
        assert!(!Step::Remount.is_a_user_operation());
        assert!(Step::CreateOne.is_a_user_operation() && Step::TruncateToTheSameSize.is_a_user_operation() && Step::Overwrite(Selector::First).is_a_user_operation());
        assert_eq!(Step::CreateOne.selector_text(), "new_file");
        assert_eq!(Step::TruncateToTheSameSize.selector_text(), "truncate");
    }
}
