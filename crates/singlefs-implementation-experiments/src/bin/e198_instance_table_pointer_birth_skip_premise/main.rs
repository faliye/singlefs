//! E198（取号跳过认出生代号的前提）：入库装置。跑前登记 `research/prompts/e198-preregistration.md`。
//!
//! 回答问题单 `research/prompts/m3-close-b19-c699-forks.md` 两行：第 1 行（加第四条件「R 的实例表指针出生 txg 等于 R.txg」之后，
//! 「行 + R 的实例表指针出生代号」能不能把取号跳过与存在过的版本分全）与第 2 行（C698 两组镜像 nrb、rs1 上改后的 I-3.5 一半与今天的 I-3.9）。
//!
//! 只读、只驱动、只观测：只调 `crates/` 的公开入口造历史（`singlefs_core` 的 mkfs、可写挂载、各条发布与回退）与读盘
//! （`singlefs_core::recovery` 的读根环、读实例表、扫环，`singlefs_checker` 的系统配置槽、根环读法与 `check_pool_image`），
//! 不改 `crates/singlefs-core`、`crates/singlefs-checker`。两条判法（今天只认行的 `today`、加第四条件的 `with_pointer_birth`）、
//! 候选集、洞对与真值账本都写在本文件里，不调、不抄 `walk.rs` 的私有函数（它们没有公开入口，登记第三节）。
//!
//! 臂的名字（产物里写这些，登记里的叫法对应如下）：H1-A = `today`；甲 = `with_pointer_birth`。
//!
//! 几何、工作量与判据门槛一律用本文件的本地常量，值抄自跑前登记第五节并注明出处；`constant_checks` 逐个与实现侧回比，
//! 不等就是停机条款 S4（登记第十一节），整趟不出结论。
//!
//! 模式（第一个参数）：
//!   `checks`                 本地常量回比（S4）与不建历史的锚点（B1–B6 的闭式张数、B8），不建历史
//!   `segment-one`            第一段（登记第五节「分段」）：nrb、rs1 与几何敏感性 G4，逐张判两条臂的分类、跑 `check_pool_image` 取 I-3.9 与 I-3.5
//!   `segment-two`            第二段：ss、skr、sw-crash、sw-read 与几何敏感性 G1–G3，算前提 P1–P3、E1 / E2、清扫触发计数与洞对覆盖
//!   `segment-three`          第三段（附带）：sw-none（问题单两行都够判时不跑，登记第五节）
//!   `stop-clauses <段名>`    只建那一段的历史、只判停机条款 S1、S2、S3、S5 与 S4（登记 3b：停机条款在产物之前跑），不打分类
//!   `list-items <段名>`      逐行列出那一段的历史名，不建历史
//!
//! 环境变量：`E198_THREADS`（工作线程数，没设取每台默认重活线程数：task-server 起任务时设好的 `SINGLEFS_THREAD_CAP`，
//! 再没设取 `.claude/heavy-thread-default` 的登记值）。
//!
//! 产物按 `E7RESULT` 行打到标准输出：历史全部建完、判完之后按历史名的次序一次打出（与线程数无关，逐字节可比）；进度与段用时打到标准错误。
//!
//! 准入写 `always`：复跑登记（`replay.sh` 的 `driver_e198` 两行）要在输入不变时再跑同一段、核产物逐字节相同。

// admission: always 复跑登记要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use singlefs_checker::image::{verified_system_configuration_slots, InvariantVerdict};
use singlefs_checker::walk::{
    check_pool_image, journal_records_as_the_recovery_reads_them,
    root_ring_the_ring_judgements_read,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::journal::{back_chain_of, JournalRecordPlaceInPublish};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration,
    roll_back_by_a_forward_publish, MountOutput, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_of_root, readable_roots_with_ring_slots,
    scan_journal,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, publish_sequential_write,
    publish_version, FirstFile, InstanceTablePlan, PoolVersion, PoolWriter, PublishPlan,
    TransactionOutput,
};
use singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

mod constant_checks_and_closed_form_anchors;
mod group_builds;
mod history_building;
mod judging_side_and_arms;
mod one_image_against_the_ledger;
mod parallel_histories;
mod pool_building_parts;
mod premises;
mod sampling_domain;
mod totals_and_product_lines;
mod truth_ledger;
mod verdict_lines_and_clauses;

use constant_checks_and_closed_form_anchors::*;
use group_builds::*;
use history_building::*;
use judging_side_and_arms::*;
use one_image_against_the_ledger::*;
use parallel_histories::*;
use pool_building_parts::*;
use premises::*;
use sampling_domain::*;
use totals_and_product_lines::*;
use truth_ledger::*;
use verdict_lines_and_clauses::*;

// ============================================================================
// 一、本地常量：值抄自 `research/prompts/e198-preregistration.md` 第五节「本地常量表」与「几何」一段（登记修订见第十二节）。
//    回比在 `constant_checks`（停机条款 S4）。
// ============================================================================

/// 根记录里实例表指针的偏移：登记第五节本地常量表第 1 行（`crates/singlefs-core/src/root_record.rs` 里 `specification_decode_at(bytes, 170)` 那一处）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
/// 指针里出生 txg 的偏移：登记第五节本地常量表第 2 行（`crates/singlefs-checker/src/image.rs` 里 `read_u64(bytes, 42)` 那一处）。
const POINTER_BIRTH_TXG_OFFSET: usize = 42;
/// 出生 txg 字段的字节数（小端 u64）：登记第七节 B7「[212, 220)」。
const POINTER_BIRTH_TXG_BYTES: usize = 8;
/// 根环区域数：登记第五节本地常量表第 3 行（`singlefs_format::ROOT_RING_REGIONS`）。
const ROOT_RING_REGIONS: u64 = 3;
/// 区域到盘：登记第五节「几何」一段「区域到盘 [0, 1, 0]」。
const REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// 清扫阈值的产品默认：365（登记第五节本地常量表第 4 行、登记第十二节修订），
/// 与 `InstanceTableSweepConfiguration::PRODUCT` 的阈值回比，不等即 S4 停机（C690（清扫触发阈值的默认值没定））。
const SWEEP_THRESHOLD_PRODUCT_DEFAULT: u64 = 365;
/// 清扫阈值「有一行就扫」：登记第五节本地常量表第 5 行（`InstanceTableRowCountThreshold::ANY_ROW`）。
const SWEEP_THRESHOLD_ANY_ROW: u64 = 0;
/// 一片实例表的记录数：登记第五节本地常量表第 6 行（`singlefs_format::INSTANCE_TABLE_PAGE_RECORDS`）。
const INSTANCE_TABLE_PAGE_RECORDS: u64 = 370;
/// mkfs 那一版的 txg：登记第五节本地常量表第 7 行（`GENESIS_CHECKPOINT_TXG`）。
const GENESIS_TXG: u64 = 0;
/// 读坏一个根槽时取反的字节：登记第五节本地常量表第 8 行「槽内第 40 字节取反」。
const CORRUPTED_BYTE_IN_THE_ROOT_SLOT: u64 = 40;

/// 几何（登记第五节「几何」一段）：两块盘、每盘 256 MiB、物理块与最小 I/O 512 字节、固定结构槽距 4096。
const DEVICE_BYTES: u64 = 256 << 20;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
/// journal 环长：主取样点 1 MiB，几何敏感性 G2 改成 4 MiB（登记第五节取样域表与第八节）。
const SHORT_JOURNAL_RING_BYTES: u64 = 1 << 20;
const LONG_JOURNAL_RING_BYTES: u64 = 4 << 20;
/// 内容：默认 3000 字节覆盖写；顺序写内容长 c × 524 288 字节（登记第五节「几何」一段）；nrb 顺序写那一组 2 MiB（第五节取样域表）。
const OVERWRITE_CONTENT_BYTES: usize = 3000;
const SEQUENTIAL_WRITE_CHUNK_BYTES: usize = 524_288;
const NEWEST_ROW_BAD_SEQUENTIAL_CONTENT_BYTES: usize = 2 * 1024 * 1024;
/// 写时刻固定（登记第五节「几何」一段「写时刻固定」）；fsid 自取 16 字节。
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"E198-birth-skip!";
/// G1 攒行时实例数的上限：阈值 365 要 366 行（实例 367 的挂载才扫），留余量；到上限还没扫就照实报「这一形没造出来」。
const SWEEP_ROW_ACCUMULATION_INSTANCES_AT_MOST: u32 = 400;

// ============================================================================
// 十四、入口：参数、准入、打印。
// ============================================================================

/// 跑一段：建全部历史、判停机条款；`print_the_classification` 时另打每段历史、每张镜像、合计、轨迹与判定（产物），否则只打停机条款（登记 3b）。
fn run_the_segment(segment: Segment, print_the_classification: bool) -> Vec<String> {
    let specifications = segment.history_specifications();
    let threads = worker_threads();
    let constants = constant_checks();
    let started = std::time::Instant::now();
    eprintln!(
        "  {}：{} 段历史，{threads} 个线程",
        segment.name(),
        specifications.len()
    );
    let results = run_the_histories(&specifications, threads);
    eprintln!(
        "  {}：历史全部判完，用了 {:.0} 秒",
        segment.name(),
        started.elapsed().as_secs_f64()
    );
    let mut lines = vec![format!(
        "name=config experiment=e198 segment={} mode={} histories={} history_threads={threads} judging_threads={threads} output_threads=1",
        segment.name(),
        if print_the_classification { "products" } else { "stop_clauses" },
        specifications.len()
    )];
    for check in &constants {
        lines.push(format!(
            "name=constant constant={} local={} implementation={} equal={}",
            check.name,
            check.local,
            check.implementation,
            yes_or_no(check.holds())
        ));
    }
    let stops = stop_clauses(&results, &constants);
    lines.extend(stops.lines.iter().cloned());
    if print_the_classification {
        for result in &results {
            lines.extend(history_lines(result));
            lines.extend(history_trajectory_lines(result));
            for image in &result.images {
                lines.push(image_line(result, image));
            }
        }
        for group in GROUP_ORDER {
            for point in POINT_ORDER {
                lines.extend(group_lines(&results, group, point));
            }
        }
        lines.extend(match segment {
            Segment::One => judgement_lines_of_segment_one(&specifications, &results, &stops),
            Segment::Two => judgement_lines_of_segment_two(&specifications, &results, &stops),
            Segment::Three => judgement_lines_of_segment_three(&results, &stops),
        });
    } else {
        lines.push(format!(
            "name=verdict stop_clauses_triggered={} status={}",
            if stops.triggered.is_empty() {
                "none".to_string()
            } else {
                stops.triggered.join(",")
            },
            if stops.triggered.is_empty() {
                "products_may_run"
            } else {
                "stopped_hand_to_the_main_agent"
            }
        ));
    }
    lines
}

/// `checks`：常量回比与闭式张数锚点，不建历史（编完先跑它，S4 不过就不跑产物）。
fn run_the_checks() -> Vec<String> {
    let mut lines = Vec::new();
    for check in constant_checks() {
        lines.push(format!(
            "name=constant constant={} local={} implementation={} equal={}",
            check.name,
            check.local,
            check.implementation,
            yes_or_no(check.holds())
        ));
    }
    let mut every_specification = Segment::One.history_specifications();
    every_specification.extend(Segment::Two.history_specifications());
    for (anchor, group, point, expected) in ATTEMPTED_IMAGE_ANCHORS {
        let observed = attempted_images_of(&every_specification, group, point)
            .map_or("unknown".to_string(), |count| count.to_string());
        lines.push(anchor_line(
            anchor,
            "independent",
            &format!("attempted_images.{}.{}", group.name(), point.name()),
            &expected.to_string(),
            &observed,
        ));
    }
    lines
}

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
fn preflight() -> Option<String> {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/bin/e198_instance_table_pointer_birth_skip_premise/main.rs");
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
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
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn usage() {
    eprintln!("用法：e198_instance_table_pointer_birth_skip_premise <checks | segment-one | segment-two | segment-three | stop-clauses <段名> | list-items <段名>> [--force]");
}

fn print_result_lines(lines: &[String]) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for line in lines {
        writeln!(handle, "E7RESULT {line}").expect("标准输出写得进");
    }
    // emitted 含收尾行自己，与 replay.sh 闸 2 的逐段核口径一致。
    writeln!(handle, "E7RESULT name=done emitted={}", lines.len() + 1).expect("标准输出写得进");
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let Some(summary) = forced_summary {
        if words.first() == Some(&"list-items") {
            eprintln!("name=preflight forced={summary}");
        } else {
            println!("E7RESULT name=preflight forced={summary}");
        }
    }
    let lines = match words.as_slice() {
        ["checks"] => run_the_checks(),
        [segment_name] if Segment::from_name(segment_name).is_some() => run_the_segment(
            Segment::from_name(segment_name).expect("match 守卫判过认得出"),
            true,
        ),
        ["stop-clauses", segment_name] if Segment::from_name(segment_name).is_some() => {
            run_the_segment(
                Segment::from_name(segment_name).expect("match 守卫判过认得出"),
                false,
            )
        }
        ["list-items", segment_name] if Segment::from_name(segment_name).is_some() => {
            for specification in Segment::from_name(segment_name)
                .expect("match 守卫判过认得出")
                .history_specifications()
            {
                println!("{}", history_name(&specification));
            }
            return;
        }
        _ => {
            usage();
            std::process::exit(2);
        }
    };
    print_result_lines(&lines);
}

// ============================================================================
// 十五、单测（登记第五节 PC-单测、第七节 C1 / C2、第九节变异的必红断言）。纯判法各喂一张已知答案表；
//     造历史的几条各建一段真历史（nrb、skr、ss、sw-crash、sw-read、rs1 各一段，G1 那一段要攒 366 行、不进单测）。
// ============================================================================

#[cfg(test)]
mod helpers_shared_by_the_tests;
