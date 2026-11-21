//! E10（老化：结构抗老化 vs 事后整理） 第 1 次跑的第一段：岔路单 `research/prompts/m4-gc-r1-forks.md` 第 4 行（`N₀` 的值）的计数模型那一半，
//! 第 2 行（停机谓词）逐条意图的四个量顺带出。跑前登记 `research/prompts/e10-r1-prereg.md`（下称「登记」），判据、臂、取样点都在那里写死。
//!
// admission: inputs-changed research/prompts/e10-r1-prereg.md ./lib.rs arguments
// run-condition: command python3
//!
//! 准入的输入：本文件（判法自带）、跑前登记、产物行的格式器 `lib.rs`，加上这一次的参数（分片之间参数不同，各判各的）。
//!
//! ## 它答什么
//!
//! 「填满 → 删 → 填满」（FDF）循环里，合取 7 的门槛 `N₀` 取 1、2、`⌈(3S − 1) × c ÷ 64⌉ + 1` 三个候选，在 S ∈ {4, 8, 16} × c ∈ {5, 10, 13}
//! 九格、主取样点与六个方向相反的取样点上：删除类发布的提交内生块数 `c_actual` 随轮数有没有发散（登记第六节第 3 行）、删除类发布
//! 发不出去的次数（第 4 行）；附带全空段数三个口径的轨迹（第 2 行）、回落计数（第 5 行）、被拒写入（第 6 行）、纯元数据稳态流里
//! 不可开段数的峰值（第 7 行），以及每条结束的整理意图上的 `P_R`、`P_G`、`Y1`、`Y2`（第 9、10、13 行的输入）。
//!
//! ## 臂（登记第五节，逐字同义）
//!
//! - N1 / N2 / NF：合取 7 取 `N₀ = 1` / `2` / 那一格的算式值，只判前台写入类发布；整理开、选段 greedy、停机照停乙。
//! - `N₀ = 0`（只报）与上面四档各自「整理关掉」（OFF，阳性对照）。
//!
//! ## 它不答什么（跑前写死）
//!
//! 不答实现臂（`crates/` 那份代码在循环里的性质，第三段）；不答选段三臂与停不下的起点（第二段）；模型里根槽写不失败、不崩溃
//! （登记第十一节已登记差异 ⑥）；文件树节点数恒为 `c − b`（差异 ③）；合取 1 是九项式子的简化（差异 ④）。
//!
//! ## 模型是确定性的
//!
//! 随机源只有种子，同一种子跑两遍逐字节相同；证据强度来自单测里钉死的绝对值与变异，不来自轮数（`test-discipline.md`「单次观测不算数」）。

use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use e7_index_bench::Emitter;

mod lines_and_sharding;
mod pool_allocation_and_publishing;
mod pool_compaction_and_remount;
mod pool_geometry_and_state;
mod pool_slots_transactions_and_commit;
mod runs_and_growth_judgement;
#[cfg(test)]
mod tests;

use self::lines_and_sharding::*;
use self::pool_allocation_and_publishing::*;
use self::pool_geometry_and_state::*;
use self::pool_slots_transactions_and_commit::*;
use self::runs_and_growth_judgement::*;

// ---- 本地常量：值抄自 `crates/`，与 `crates/` 逐项对拍过（登记第十一节 S1、S2；对拍的命令与原样输出在执行员报告里） ----

/// 聚簇段段长（槽）。`crates/singlefs-format/src/lib.rs` 的 `CLUSTER_SEGMENT_SLOTS`。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 单元区起始槽号（默认 768 MiB 环）。同文件 `UNIT_AREA_START_SLOT`。
const UNIT_AREA_START_SLOT: u64 = 50176;
/// 根环的区数。同文件 `ROOT_RING_REGIONS`。
const ROOT_RING_REGIONS: u64 = 3;
/// 每区根槽数 S 的下界与上界。同文件 `ROOT_RING_SLOTS_PER_REGION_MINIMUM` / `_MAXIMUM`。
const ROOT_RING_SLOTS_PER_REGION_LOWER_BOUND: u64 = 4;
const ROOT_RING_SLOTS_PER_REGION_UPPER_BOUND: u64 = 16;
/// 分配记录树一片叶罩的槽数。同文件 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// 分配记录树内部节点的扇出。同文件 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`。
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 两块盘同槽（D2（RAID 条带策略） 已定项 10）：分配记录树的叶与层级 1 节点每块盘一份。
const DISK_COUNT: u64 = 2;

// ---- 登记写死的负载与判据常量（登记第五、六节） ----

/// 一次删除类发布至多删几个单元（登记 M-9 的 q）。
const UNITS_PER_DELETION_PUBLISH: usize = 64;
/// 一个新文件的单元数在 1..=32 里按种子均匀取（登记 M-9）。
const FILE_UNITS_UPPER_BOUND: u64 = 32;
/// FDF 跑的轮数；判「有界」的再续到加倍复核（登记 M-9 与第六节第 3 行）。
const ROUNDS_FIRST_PASS: u64 = 60;
const ROUNDS_DOUBLED_CHECK: u64 = 120;
/// 每格每臂的种子数（登记读法第 11 条）。
const SEEDS_PER_CELL: u64 = 5;
/// 一批至多搬几个单元（主取样点 K = 1，登记 M-7）。
const COMPACTION_BATCH_UNITS: usize = 1;
/// 纯元数据稳态流的空发布次数（登记第六节第 7 行）。
const METADATA_STREAM_EMPTY_PUBLISHES: u64 = 1000;
/// 抬 F 要连发的空发布次数（登记 M-6 ①，D16（发布语义） 已定项 1「抬 F 那一串」）。
const EMPTY_PUBLISHES_FOR_FLOOR_TO_TAKE_EFFECT: u64 = 2;
/// 抬 F 抬到第几新的非空根（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」）。
const RETAINED_NON_EMPTY_STATES: usize = 4;
/// 第六节第 9 行：`P_R` 判真而 `Y1` 假的占比到这个百分数就记。
const PREDICATE_FALSE_POSITIVE_SHARE_PERCENT: u64 = 5;

/// 九格的 S 与 c（登记第一节读法第 2、4 条）。
const ROOT_RING_SLOTS_PER_REGION_CHOICES: [u64; 3] = [4, 8, 16];
const COMMIT_GENERATED_BLOCKS_CHOICES: [u64; 3] = [5, 10, 13];

/// 4 GiB 与 1 GiB 盘的槽数（16 KiB 一槽）。
const FOUR_GIBIBYTE_DISK_SLOTS: u64 = 262_144;
const ONE_GIBIBYTE_DISK_SLOTS: u64 = 65_536;
/// 反向点的叶宽（登记第八节「叶宽（只在模型）」）。
const REVERSE_POINT_LEAF_SLOTS: u64 = 203;

/// 开跑时判的准入：满足时带上开跑时的输入指纹（成功跑完记它）；`--force` 强制跑时带没满足的条件摘要（写进产物、不记成功）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum PreflightOutcome {
    Met { input_fingerprint_at_start: String },
    Forced { unmet_conditions_summary: String },
}

impl PreflightOutcome {
    /// 产物里记这一次准入的行：强制跑写一行，满足不写（满足时产物与没有准入这一层时逐字节相同）。
    fn product_lines(&self) -> Vec<String> {
        match self {
            PreflightOutcome::Met { .. } => Vec::new(),
            PreflightOutcome::Forced { unmet_conditions_summary } => {
                vec![format!("name=preflight forced={}", unmet_conditions_summary.split_whitespace().collect::<Vec<_>>().join("_"))]
            }
        }
    }
}

/// 准入判法的入口与本文件：都按编译时的清单目录取绝对路径（判法要的是源文件的路径，不是二进制的）。
fn preflight_script_and_source_paths() -> (String, String) {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    (
        format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py"),
        format!("{manifest_directory}/e10_aging_compaction/main.rs"),
    )
}

/// 准入判法退 78（拒绝）时装置也退 78；别的非 0 退 1（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `main` 的第一句：直接起 `python3 preflight.py check <本文件> [--force] -- <参数…>`（不经 sh -c）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做。
fn preflight() -> PreflightOutcome {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (script, source) = preflight_script_and_source_paths();
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE)
    });
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    match output.status.code() {
        Some(0) => {}
        Some(PREFLIGHT_REFUSED_EXIT_CODE) => std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE),
        _other_exit => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").to_string();
    let last_segment = verdict_line.rsplit('\t').next().unwrap_or("").to_string();
    if verdict_line.starts_with("met") {
        PreflightOutcome::Met { input_fingerprint_at_start: last_segment }
    } else if verdict_line.starts_with("forced") {
        PreflightOutcome::Forced { unmet_conditions_summary: last_segment }
    } else {
        eprintln!("  ✗ 准入判法放行了，却没给 met 或 forced 那一行：「{verdict_line}」");
        eprintln!("  → 怎么办：查 e10_aging_compaction.rs 文件头的 admission 与 run-condition 两行写法");
        std::process::exit(1)
    }
}

/// 成功跑完、退出之前记下开跑时判的那份输入指纹（`preflight.py record`）；记不下只在 stderr 说为什么，不改这一次的退出码。
fn preflight_record_success(input_fingerprint_at_start: &str, arguments: &[String]) {
    let (script, source) = preflight_script_and_source_paths();
    let recorded = std::process::Command::new("python3")
        .arg(&script)
        .arg("record")
        .arg(&source)
        .arg("--fingerprint")
        .arg(input_fingerprint_at_start)
        .arg("--")
        .args(arguments)
        .status();
    match recorded {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("  ! 没记下这一次的输入指纹：preflight.py record 退 {status}"),
        Err(error) => eprintln!("  ! 没记下这一次的输入指纹：起不了 python3（{error}）"),
    }
}

/// 三种用法：不带参数跑全部并出全部判定（产物与复跑就是这一种）；`--point <取样点> [--root-ring-slots-per-region <S>]` 只跑一个取样点
/// （或其中一个 S）、出一份分片；`--merge <分片…>` 把按（取样点, S）次序给的分片拼成与不带参数逐字节相同的产物（正式跑的次序与条数逐条核）。
/// 三种都可再带 `--force`：准入不满足也照跑（产物开头记一行 `name=preflight forced=…`，不记成功）。
fn main() {
    let preflight_outcome = preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let preflight_lines = preflight_outcome.product_lines();
    let started = std::time::Instant::now();
    let shard_of = |name: &str, root_ring_slots_per_region: Option<u64>| {
        let point = SAMPLING_POINTS.iter().find(|point| point.name == name).unwrap_or_else(|| panic!("不认得的取样点 {name}"));
        assert!(
            root_ring_slots_per_region.is_none_or(|wanted| ROOT_RING_SLOTS_PER_REGION_CHOICES.contains(&wanted)),
            "S 只取 4 / 8 / 16"
        );
        ShardSelection { point: point.name, root_ring_slots_per_region }
    };
    match arguments.as_slice() {
        [] => {
            let mut lines = header_lines();
            let (measured, summaries) = measured_lines(None);
            lines.extend(measured);
            lines.extend(cell_and_verdict_lines(&summaries, true));
            print_with_emitter(&preflight_lines, &lines);
        }
        [flag, name] if flag == "--point" => print_with_emitter(&preflight_lines, &measured_lines(Some(shard_of(name, None))).0),
        [point_flag, name, root_ring_slots_flag, root_ring_slots_per_region]
            if point_flag == "--point" && root_ring_slots_flag == "--root-ring-slots-per-region" =>
        {
            let root_ring_slots_per_region: u64 = root_ring_slots_per_region.parse().expect("S 是正整数");
            print_with_emitter(&preflight_lines, &measured_lines(Some(shard_of(name, Some(root_ring_slots_per_region)))).0);
        }
        [flag, shard_paths @ ..] if flag == "--merge" => {
            let shards: Vec<Vec<String>> = shard_paths.iter().map(|path| read_shard(path)).collect();
            let mut lines = header_lines();
            for shard in &shards {
                lines.extend(shard.iter().filter(|line| line.starts_with("name=metadata_stream ")).cloned());
            }
            let mut summaries = Vec::new();
            for shard in &shards {
                for line in shard.iter().filter(|line| !line.starts_with("name=metadata_stream ")) {
                    summaries.extend(run_summary_of_line(line));
                    lines.push(line.clone());
                }
            }
            let merged_keys: Vec<RunKey> = summaries.iter().map(|summary| summary.key).collect();
            assert_eq!(merged_keys, all_run_keys(None), "分片拼起来的正式跑与全跑逐条同序、一条不缺");
            assert_eq!(
                lines.iter().filter(|line| line.starts_with("name=metadata_stream ")).count(),
                metadata_stream_lines(None).len(),
                "第 7 行一格不缺"
            );
            lines.extend(cell_and_verdict_lines(&summaries, true));
            print_with_emitter(&preflight_lines, &lines);
        }
        other => panic!(
            "用法：e10-aging-compaction [--force] [--point <取样点> [--root-ring-slots-per-region <S>] | --merge <分片…>]，收到 {other:?}"
        ),
    }
    eprintln!("e10-aging-compaction {arguments:?}：{:.1} 秒", started.elapsed().as_secs_f64());
    if let PreflightOutcome::Met { input_fingerprint_at_start } = &preflight_outcome {
        preflight_record_success(input_fingerprint_at_start, &arguments);
    }
}
