//! E187：隔离落盘代价 —— 「盘上完好的隔离」那一形的三种处置各多写多少、记号写错时各是放过还是误红、
//! 落点甲借位之后旧读者把带记号的记录读成什么（岔路单 `research/prompts/m4-i315-quarantine-mark-cost-forks.md` 那一行）。
//!
//! 跑前登记：`research/prompts/e187-preregistration.md`。三条臂（`no_mark` 不落盘、`span_bit` 丙·落点甲、`quarantine_tree` 丙·落点乙）、
//! 参照臂 `plain_release`、判据、作废与停机条款都写死在那里。独立手写的计数模型，不与 `crates/` 共用代码：
//! 登记第十一节停机条款那几项的读码核写成产物里的 `name=stop_check` 行，格式常量另从 `crates/` 源码原文里逐字核一遍。
//!
//! 用法：不带参数两段都跑（产物就是这一份输出）；`segment-one` 只跑第一段，`segment-two` 只跑第二段。
//! 线程数：`E187_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取登记的重活默认并行度（`.claude/heavy-thread-default`）；
//! 第二段按历史切片并行、按历史的次序合并，产物与线程数无关（单测 `segment_two_lines_do_not_depend_on_the_thread_count`）。
//! 一段历史内部的发布逐次串行：下一次发布读上一次的树、分配器与映射，次序就是被测对象。

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::rc::Rc;

// 写成 `#[path] mod`：单元闭包的键不收条目级 `include!`（同 E178）。
#[path = "../heavy_threads_default/threads.rs"]
mod heavy_threads_default;
use heavy_threads_default::default_heavy_threads;

mod geometry_records_and_code_two_trees;
mod history_schedules_and_runs;
mod judging_old_reader_and_micro_histories;
mod pool_state_and_placement;
mod source_anchor_checks_and_micro_history_run;
mod summary_lines_and_history_cells;
mod user_intents_and_publish_planning;

use self::geometry_records_and_code_two_trees::*;
use self::history_schedules_and_runs::*;
use self::judging_old_reader_and_micro_histories::*;
use self::pool_state_and_placement::*;
use self::source_anchor_checks_and_micro_history_run::*;
use self::summary_lines_and_history_cells::*;
use self::user_intents_and_publish_planning::*;

/// 一个槽（D3（空间分配） 已定项 7 的落点粒度）与一个码 2 节点的字节数。
const SLOT_BYTES: u64 = 16384;
const NODE_BYTES: u64 = 16384;
/// 一次单元写每个槽写多少字节：设备写的整单元字节 = 这个数 × 槽数（登记第一节「字节」）。
const UNIT_WRITE_BYTES_PER_SLOT: u64 = SLOT_BYTES;
/// 池里几块盘：每个单元写到每块盘的同一槽（w = 2 镜像）。
const DEVICE_COUNT: u64 = 2;
const JOURNAL_RECORD_BYTES: u64 = 4096;
const JOURNAL_HEADER_BYTES: u64 = 496;
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 一条 journal 记录装几个点名项（`crates/singlefs-format/src/lib.rs` 的 `JOURNAL_NAMED_ENTRIES_PER_RECORD`，这里现算）。
const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;
/// 根槽宽 = physical_block_size，取 E142 那套几何的 512（`crates/singlefs-verification/src/new_pool_file_creation_regions.rs` 文件头）；
/// 每次发布只写根环所在区域的那一块盘（`transaction.rs` 的 `WriteRootRecordForceUnitAccess`）。
const ROOT_SLOT_BYTES: u64 = 512;
const ROOT_SLOT_WRITES_PER_PUBLISH: u64 = 1;
/// 系统配置槽：每次发布每块盘写一槽（`transaction.rs` 的 `RotateSystemConfigurationSlots`）。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
const UNIT_AREA_START_SLOT: u64 = 50176;
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// mkfs 那一版在单元区起点留下的两个单元（各一槽），新池新建文件之后照旧占着（只当不动的记录）。
const MAKE_FILESYSTEM_RECORD_SLOTS: [u64; 2] = [UNIT_AREA_START_SLOT, UNIT_AREA_START_SLOT + 1];
/// 分配记录树一片叶罩几个槽（W，`crates/singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_NODE_HEADER_BYTES: u64 = 135;
const ALLOCATION_RECORD_KEY_BYTES: u64 = 10;
#[expect(dead_code, reason = "D3 已定项 7 value 宽度备查")]
const ALLOCATION_RECORD_VALUE_BYTES: u64 = 10;
const ALLOCATION_RECORD_BYTES: u64 = 20;
/// 跨度段在条目里的偏移：紧接 key（D3（空间分配） 已定项 11「跨度段进 value」）。
const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: u64 = ALLOCATION_RECORD_KEY_BYTES;
const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
const MAPPING_NODE_HEADER_BYTES: u64 = 169;
const MAPPING_ENTRY_BYTES: u64 = 55;
const MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113;
/// 乙的隔离树：条目与分配记录同宽同序（key 设备 4 + 槽号 6，value 跨度 2 + 隔离代 8），节点头 86 + 2 × 10 + 29。
const QUARANTINE_TREE_NODE_HEADER_BYTES: u64 = 135;
const QUARANTINE_TREE_ENTRY_BYTES: u64 = 20;
const QUARANTINE_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
const TREE_TABLE_HEADER_BYTES: u64 = 131;
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 12;
const EXTENT_NODE_HEADER_BYTES: u64 = 163;
const EXTENT_LEAF_ENTRY_BYTES: u64 = 112;
const EXTENT_INTERNAL_ENTRY_BYTES: u64 = 110;
/// 跨度段最高位借作已释放标志（D3（空间分配） 已定项 7）。
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
/// 落点甲借的那一位：跨度段次高位（登记第五节 `span_bit` 那一行）。
const QUARANTINE_MARK_BIT: u16 = 0x4000;
/// 几何敏感性里拿来对照的低位（登记第八节：低位让多数记录照样落叶）。
const LOW_SPAN_BIT_PROBE: u16 = 0x0004;
/// 旧读者的阳性对照：`crates/singlefs-verification/src/bad_disk_input.rs` 那一条坏法的跨度段值与槽。
const OLD_READER_POSITIVE_CONTROL_SPAN_FIELD: u16 = 0x7FFF;
const OLD_READER_POSITIVE_CONTROL_SLOT: u64 = 50176;
/// 第一版合法记录的跨度只有这两种（D3（空间分配） 已定项 7「第一版恒为 1 或 2」）。
const FIRST_VERSION_SPANS: [u16; 2] = [1, 2];

/// 一个码 2 节点装几条：(16384 − 头) ÷ 条目宽 的整数部分（`crates/singlefs-core/src/unit.rs` 的 `index_node_entry_capacity`）。
const fn entries_per_node(header_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - header_bytes) / entry_bytes
}
const ALLOCATION_RECORD_TREE_LEAF_ENTRIES: u64 = entries_per_node(ALLOCATION_RECORD_TREE_NODE_HEADER_BYTES, ALLOCATION_RECORD_BYTES);
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
const MAPPING_LEAF_ENTRIES: u64 = entries_per_node(MAPPING_NODE_HEADER_BYTES, MAPPING_ENTRY_BYTES);
const MAPPING_INTERNAL_ENTRIES: u64 = entries_per_node(MAPPING_NODE_HEADER_BYTES, MAPPING_INTERNAL_ENTRY_BYTES);
const QUARANTINE_TREE_LEAF_ENTRIES: u64 = entries_per_node(QUARANTINE_TREE_NODE_HEADER_BYTES, QUARANTINE_TREE_ENTRY_BYTES);
const QUARANTINE_TREE_INTERNAL_ENTRIES: u64 =
    entries_per_node(QUARANTINE_TREE_NODE_HEADER_BYTES, QUARANTINE_TREE_INTERNAL_ENTRY_BYTES);
const TREE_TABLE_UNIT_ENTRIES: u64 = entries_per_node(TREE_TABLE_HEADER_BYTES, TREE_TABLE_ENTRY_BYTES);
const EXTENT_LEAF_ENTRIES: u64 = entries_per_node(EXTENT_NODE_HEADER_BYTES, EXTENT_LEAF_ENTRY_BYTES);
const EXTENT_INTERNAL_ENTRIES: u64 = entries_per_node(EXTENT_NODE_HEADER_BYTES, EXTENT_INTERNAL_ENTRY_BYTES);

/// 树 ID（`crates/singlefs-format/src/lib.rs` 的 `TREE_IDENTIFIER_*`）。
const EXTENT_TREE: u64 = 65536;
const INODE_TREE: u64 = 65537;
const ALLOCATION_RECORD_TREE: u64 = 65538;
const ACCOUNTING_TREE: u64 = 65539;
/// 乙的隔离树取池里下一个没用过的树 ID（新池新建文件之后的树 ID 水位 `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`）：
/// 比别的树都大，它的节点在映射 key 的次序里排在别的码 2 树之后。
const QUARANTINE_TREE: u64 = 65549;
/// 映射 key 的类标签（`crates/singlefs-core/src/unit.rs` 的 `UNIT_CLASS_*`）。
const UNIT_CLASS_DATA: u8 = 1;
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const UNIT_CLASS_PACKED: u8 = 3;
const WRITE_ORDER_INSTANCE: u32 = 1;

/// 工作负载（登记第五节「模型」）。
const OVERWRITES_PER_PUBLISH: usize = 4;
const NORMAL_PUBLISHES_PER_HISTORY: u64 = 10_000;
/// 密度 d（每千次发布 d 次）在 10000 次普通发布上排 10d 次隔离事件。
const QUARANTINE_EVENTS_PER_DENSITY_STEP: u64 = NORMAL_PUBLISHES_PER_HISTORY / 1000;
const ROLLBACKS_AT_RATE_ONE: u64 = 10;
const DENSITIES_PER_THOUSAND_PUBLISHES: [u64; 4] = [0, 1, 3, 10];
const ROLLBACK_RATES_PER_THOUSAND_PUBLISHES: [u64; 2] = [0, 1];
const FILE_SIZES_IN_DATA_UNITS: [u32; 2] = [200, 20000];
const CANDIDATE_WINDOWS: [u64; 2] = [8, 64];
const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
/// 固定点最多走几轮（同 `transaction.rs` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`）。
const SETTLING_ROUNDS_LIMIT: usize = 64;

/// 份额的档界（登记第六节 `extra_write_share`）：档一 < 0.1%，档二 [0.1%, 1%)，档三 ≥ 1%；按十亿分之几写（判别力自证要挪到两点之间）。
const SHARE_TIER_BOUNDS_PARTS_PER_BILLION: ShareTierBounds = ShareTierBounds { tier_two_from: 1_000_000, tier_three_from: 10_000_000 };
const PARTS_PER_BILLION: i128 = 1_000_000_000;

/// 线程数：`E187_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn threads_from_the_environment() -> usize {
    std::env::var("E187_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads >= 1)
        .unwrap_or_else(|| default_heavy_threads(include_str!("../../../.claude/heavy-thread-default")))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RunMode {
    BothSegments,
    SegmentOne,
    SegmentTwo,
}

fn run_mode_of(arguments: &[String]) -> Option<RunMode> {
    match arguments {
        [] => Some(RunMode::BothSegments),
        [mode] if mode == "segment-one" => Some(RunMode::SegmentOne),
        [mode] if mode == "segment-two" => Some(RunMode::SegmentTwo),
        _unrecognized => None,
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(mode) = run_mode_of(&arguments) else {
        eprintln!("用法：e187-quarantine-mark-cost [segment-one | segment-two]（不带参数两段都跑）");
        std::process::exit(2);
    };
    let threads = threads_from_the_environment();
    let runs_segment_one = mode != RunMode::SegmentTwo;
    let runs_segment_two = mode != RunMode::SegmentOne;
    let mut lines = vec![format!(
        "name=config experiment=e187_quarantine_mark_cost model=counting_model_written_by_hand registration=research/prompts/e187-preregistration.md \
         segment_one={runs_segment_one} segment_two={runs_segment_two} threads_segment_one=1 threads_segment_two={threads} \
         slot_bytes={SLOT_BYTES} devices={DEVICE_COUNT} journal_record_bytes={JOURNAL_RECORD_BYTES} root_slot_bytes={ROOT_SLOT_BYTES} root_slot_writes_per_publish={ROOT_SLOT_WRITES_PER_PUBLISH} \
         system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} unit_area_start={UNIT_AREA_START_SLOT} segment_slots={CLUSTER_SEGMENT_SLOTS} \
         allocation_leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} allocation_fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} mapping_leaf={MAPPING_LEAF_ENTRIES} mapping_internal={MAPPING_INTERNAL_ENTRIES} \
         quarantine_tree_leaf={QUARANTINE_TREE_LEAF_ENTRIES} extent_leaf={EXTENT_LEAF_ENTRIES} journal_names_per_record={JOURNAL_NAMED_ENTRIES_PER_RECORD} \
         quarantine_tree_identifier={QUARANTINE_TREE} share_tier_bounds_ppb={}/{} bytes_reported=both_devices_with_single_device_alongside",
        SHARE_TIER_BOUNDS_PARTS_PER_BILLION.tier_two_from,
        SHARE_TIER_BOUNDS_PARTS_PER_BILLION.tier_three_from
    )];
    let mut stop_checks_hold = true;
    if runs_segment_one {
        let (segment_one_lines, judgements) = run_segment_one();
        stop_checks_hold = judgements.stop_checks_hold;
        lines.extend(segment_one_lines);
    }
    if runs_segment_two {
        if stop_checks_hold {
            lines.extend(run_segment_two(threads));
        } else {
            lines.push("name=segment_two_not_run reason=stop_check_failed".to_string());
        }
    }
    let standard_output = std::io::stdout();
    let mut output = std::io::BufWriter::new(standard_output.lock());
    for line in &lines {
        writeln!(output, "E7RESULT {line}").expect("写得进标准输出");
    }
    writeln!(output, "E7RESULT name=done emitted={}", lines.len() + 1).expect("写得进标准输出");
}

#[cfg(test)]
mod tests;
