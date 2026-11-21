//! E182（被抛弃根读不出的三条岔路）跑前登记第一段（单盘择根段）：入库装置。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`。本文件回答岔路单两行（登记 5.4 第一段）：
//! B1「被藏根的记录也读不出、单盘恢复择中它」（丙：今天；甲一：单盘择根时剩下那块盘上读得出更新实例的记录、按它的新根段读得出的
//! 实例表判所选根被抛弃就不择它；甲二：剩下那块盘系统配置记着的实例代号比所选根新就不择它），
//! B4「收严新实例第一次发布避开读不出的根槽对应的 txg」（不改实现，按状态分类）。
//!
//! 臂的名字（产物里写这些，登记里的叫法对应如下）：丙 = `today`；甲一 = `newer_instance_record`；甲二 = `configuration_instance`。
//! 岔路单 B1 的「乙」没有定义「更保守的集合」，登记不建臂。
//!
//! 几何、工作量与判据门槛一律用本文件的本地常量，值抄自跑前登记第五节并注明出处；`run_constants_and_anchors` 逐个与实现侧同名常量回比，
//! 不等就作废（登记第十一节）。不从 `crates/` 引决定工作量或判据门槛的常量。
//!
//! 模式（第一个参数）：
//!   `anchors`            常量回比、第七节锚点、闭式计数（不建历史）
//!   `list-items`         逐行列出可点名的历史名（登记修订 12.4 的读法 18 条，再接补臂 9 条，名字带 `units-kept.` 前缀）
//!   `measure <历史名>`   在这台机器上量一条历史的状态数与一百次恢复的挂钟（登记 5.5 最后一条；挂钟不进产物）
//!   `single-device [--item <历史名>]…`
//!                        第一段：建每条历史、按片判（一片 = 一条历史 × 它录制流里的一个段）、片结果按键存取、合并计数、打产物。
//!                        不带参数：剪枝跑（键相同的片直接取库里的，缺的当场判、照键写回）；`--item`：只跑点名的历史。
//!   `single-device-units-kept [--item <历史名>]…`
//!                        登记修订 12.11 的补臂：同一套判法，历史换成「实例 1 的每次覆盖写都是 s 个单元的顺序写」的 9 条（只取 s = 64 的起点），
//!                        产物里的历史名与「族 × 几何」合并标签带 `units-kept.` 前缀，config 行多一个 `overwrite_shape` 字段；与 `single-device`
//!                        并列、互不改动。
//!
//! 环境变量：`E182_THREADS`（工作线程数，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）；
//! `E182_VERDICT_LIBRARY`（片结果库目录，`single-device` 两个模式必须给：长跑脚本每一趟设成自己的中转目录、经
//! `scripts/gate/admission.py library-files` 与 prune-server 搬片结果；没设就退 2、标准错误写出路，装置没有缺省的本地库）；
//! `E182_SHARD_SHARE=<第几份>/<共几份>`（多机分片：片号对份数取余等于第几份的片归这一台，只判、只写库、不打产物）。
//!
//! 复用键 = 本文件判法入口（`record_the_history`、`judge_the_shard`）按 `singlefs-unit-graph` 算出的函数单元闭包指纹（去注释），
//! 加工具链行、历史名、段号、展开方式；文件级指纹与按 HEAD 算的指纹不进键（派发提示定，登记 5.5 的文件级 sha256 键由此替换）。
//! 闭包拒算时这一趟不复用、不入库，stderr 报原因。
//!
//! 准入写 `always` 而不写 `inputs-changed`：复跑登记（`replay.sh` 的 `E182` 两行）与长跑脚本（出错后原样重起同一片）
//! 都要在输入不变时再跑同一段、拿同一份输入核产物逐字节相同；写 `inputs-changed` 的话，第二次跑会被判「输入没变」拒绝（退出码 78），
//! 复跑与重做都跑不了。强制跑（`--force`）时产物开头多一行 `name=preflight forced=…`（`list-items` 的标准输出是历史名清单，那一行改写到标准错误），
//! 满足时不打，产物与没有这一层时逐字节相同。准入判法退 78 时装置也退 78。

// admission: always 复跑登记与长跑脚本都要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_checker::image as checker_image;
use singlefs_checker::image::{ImageReader, PoolGeometry};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::JournalRecord;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches, Mounted, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    instance_table_chain_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
    recover, root_is_abandoned_by_the_instance_table, scan_journal, JournalPolicy, PoolReader,
    RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::sha256::sha256_digest;
use singlefs_verification::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

mod constant_checks_anchors_and_closed_form_counts;
mod counters_arms_and_classification;
mod crash_states_of_a_segment;
mod device_reading_back_zeros;
mod first_instance_overwrites;
mod history_names_and_geometry;
mod omniscient_history_log;
mod positive_controls;
mod product_lines;
mod readings_of_one_crash_state;
mod second_instance_mount_and_overwrites;
mod shard_library_and_reuse_keys;
mod single_disk_recovery_reader;

use constant_checks_anchors_and_closed_form_counts::*;
use counters_arms_and_classification::*;
use crash_states_of_a_segment::*;
use device_reading_back_zeros::*;
use first_instance_overwrites::*;
use history_names_and_geometry::*;
use omniscient_history_log::*;
use positive_controls::*;
use product_lines::*;
use readings_of_one_crash_state::*;
use second_instance_mount_and_overwrites::*;
use shard_library_and_reuse_keys::*;
use single_disk_recovery_reader::*;

// ============================================================================
// 一、本地常量：值抄自 `research/prompts/e182-preregistration.md` 第五节（"要读 `crates/` 常量的量改成本地常量加回比断言"）。
//    回比在 `run_constants_and_anchors`。
// ============================================================================

/// 落点粒度：登记第五节「槽 16384」。
const SLOT_BYTES_LOCAL: u64 = 16384;
/// 单元区起点（默认环）：登记第五节「U = 50176」。
const UNIT_AREA_START_SLOT_LOCAL: u64 = 50176;
/// 默认 journal 环字节数：登记第五节「默认环 805306368」。
const JOURNAL_RING_BYTES_LOCAL: u64 = 805_306_368;
/// 每区根环槽数 S：登记第五节「S = 8」。
const ROOT_RING_SLOTS_PER_REGION_LOCAL: u64 = 8;
/// 根环区域数：登记第五节「根环区域 = txg mod 3」。
const ROOT_RING_REGIONS_LOCAL: u64 = 3;
/// 根环区域归盘 0 / 1 / 0：登记第五节（D16 已定项 1「正常卸载」一行）。
const REGION_DEVICES_LOCAL: [u32; 3] = [0, 1, 0];
/// 可写挂载暖机零单元发布至多几次：登记第五节「暖机至多 R = 3 次」（D16 已定项 8）。
const WARM_UP_PUBLISHES_AT_MOST_LOCAL: usize = 3;
/// 段内写数不超过它的段全量展开（任意真子集）：登记第五节「全量展开的段内写数上界 16（本登记定，不取 `crates/` 的值）」。
const FULL_EXPANSION_WRITES_AT_MOST_LOCAL: usize = 16;
/// 一个数据单元的净荷字节数：登记 E180 第七节引用的 32634，与 `data_unit_payload_capacity()` 回比。
const DATA_UNIT_PAYLOAD_BYTES_LOCAL: usize = 32634;
/// 默认几何每块盘字节数：登记第一节「两块 4 GiB 盘」。
const DEVICE_BYTES_DEFAULT_LOCAL: u64 = 4 << 30;
/// 窄几何每块盘字节数：登记第一节「两块 3 GiB + 33 槽」。
const DEVICE_BYTES_NARROW_LOCAL: u64 = (3 << 30) + 33 * SLOT_BYTES_LOCAL;
/// 实例 2 挂载之后接着做的覆盖写次数：登记第一节「之后 3 次 `Overwrite`」。
const OVERWRITES_AFTER_THE_MOUNT_LOCAL: u64 = 3;
/// 固定结构槽距（系统配置槽与根环槽）：与 mkfs 默认参数一致。
const FIXED_STRUCTURE_SLOT_SPACING_LOCAL: u32 = 4096;
/// 系统配置槽字节数。
const SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL: usize = 4096;
/// 物理块字节数（内存盘的扇区）。
const PHYSICAL_BLOCK_BYTES_LOCAL: u32 = 512;
/// 一次恢复至多连着避开几条根（甲一、甲二的循环上界；根环只有 3 × 8 个槽）。
const AVOIDED_ROOTS_AT_MOST: usize = 3 * 8;
/// 固定 fsid，与别的实验装置的不同，免得两个进程各自建的镜像撞出同一份「历史上出现过」的巧合。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x45, 0x31, 0x38, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_800_000_000;

/// 产物完整性闸口径（`scripts/experiment-apparatus/replay.sh` 文件头）：每条结果行以 `E7RESULT` 起头，收尾 `E7RESULT name=done emitted=N`，
/// N = 本进程打过的 `E7RESULT` 行数（含收尾这一条）。
static EMITTED_RESULT_LINES: AtomicU64 = AtomicU64::new(0);

fn emit_result(line: &str) {
    EMITTED_RESULT_LINES.fetch_add(1, Ordering::SeqCst);
    println!("E7RESULT {line}");
}

fn emitted_result_line_count() -> u64 {
    EMITTED_RESULT_LINES.load(Ordering::SeqCst)
}

// ============================================================================
// 七、一条录好的历史：基镜像、实例 2 的写表与段、全知日志、被藏根、挂载报告。
// ============================================================================

struct RecordedHistory {
    specification: HistorySpecification,
    pool_geometry: PoolGeometry,
    /// 实例 2 起头的镜像（实例 1 的盘，已按确认档截好）。
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    roots: BTreeMap<RootKey, RootLog>,
    /// 全知日志认的被抛弃根（带藏根的族是 R_h，对照族没有）。
    hidden_root: Option<RootKey>,
    fault_ranges: Vec<ReadsZeroRange>,
    mount: MountReport,
    /// 实例 2 的第一次根槽写（写行那次发布）在写表里的下标。
    row_publish_root_write: Option<usize>,
    /// 自检里没过的（录制流重建的镜像与盘上真字节不同……），空才算这条历史可信。
    consistency_problems: Vec<String>,
}

fn independent_geometry(pool: &MemoryPool) -> Result<PoolGeometry, String> {
    let chosen = checker_image::chosen_system_configurations(pool);
    let mut geometries = Vec::new();
    for (device, entry) in &chosen {
        match entry {
            Some((_, geometry)) => geometries.push(*geometry),
            None => return Err(format!("设备 {device} 没有自证的系统配置")),
        }
    }
    let first = *geometries.first().ok_or_else(|| "池里没有盘".to_string())?;
    if geometries.iter().any(|other| other != &first) {
        return Err("两盘系统配置解出的几何不一致".to_string());
    }
    Ok(first)
}

/// 实例 2 起头的镜像（实例 1 的盘，按确认档截好）与被藏根读回全零的字节区间。
fn base_and_fault_ranges(
    specification: &HistorySpecification,
    first: &FirstInstance,
) -> Result<(MemoryPool, Vec<ReadsZeroRange>), String> {
    let fixed = fixed_geometry_of_the_pool();
    let final_operations = &first.operations[first.last_publish_start..];
    let root_write_position = final_operations
        .iter()
        .position(|operation| {
            is_a_root_slot_write(operation, &fixed)
                && root_written_by(operation).map(|root| root_key_of(&root))
                    == Some(first.last_root)
        })
        .ok_or_else(|| "被藏根那次发布的录制流里没有它的根槽写".to_string())?;
    let applied_final = match specification.family.confirmation() {
        Confirmation::Unconfirmed => &final_operations[..=root_write_position],
        Confirmation::Confirmed => final_operations,
    };
    let mut base = first.pool_before_the_last_publish.clone();
    base.apply(applied_final);
    let fault_ranges =
        fault_ranges_of_the_last_publish(specification.family, applied_final, &fixed);
    Ok((base, fault_ranges))
}

/// 判法入口之一（函数单元闭包指纹的入口，见文件头）：建一条历史。
fn record_the_history(specification: &HistorySpecification) -> Result<RecordedHistory, String> {
    let first = run_the_first_instance(specification)?;
    let fixed = fixed_geometry_of_the_pool();
    let (base, fault_ranges) = base_and_fault_ranges(specification, &first)?;
    let second = run_the_second_instance(specification, &base, fault_ranges.clone());
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&second.operations, &fixed);
    let mut roots = first.roots;
    roots.extend(log_the_roots_of_a_stream(&base, &second.operations));
    let row_publish_root_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua);
    let mut consistency_problems = Vec::new();
    if !second.replay_equals_the_devices {
        consistency_problems.push("基镜像加录制流重建的镜像与盘上真字节不同".to_string());
    }
    let pool_geometry = independent_geometry(&base)?;
    Ok(RecordedHistory {
        specification: *specification,
        pool_geometry,
        base,
        writes,
        segments,
        roots,
        hidden_root: specification
            .family
            .hides_a_root()
            .then_some(first.last_root),
        fault_ranges,
        mount: second.mount,
        row_publish_root_write,
        consistency_problems,
    })
}

/// 判法入口之一（函数单元闭包指纹的入口，见文件头）：判一片 = 一条历史的一个段（段号等于段数时是整条流末尾那个全持久的状态）。
fn judge_the_shard(history: &RecordedHistory, shard_index: usize) -> Result<Counters, String> {
    let mut counters = Counters::default();
    for_each_state_of_segment(
        &history.writes,
        &history.segments,
        shard_index,
        &mut |persisted| evaluate_one_state(history, persisted, &mut counters).map(|_trace| ()),
    )?;
    Ok(counters)
}

// ============================================================================
// 十六、模式：建历史、按片判、合并、打产物。
// ============================================================================

/// 并行建历史（每条历史自己的实例 1、实例 2 与全知日志）；按 `specs` 的次序交回。
fn build_the_histories(
    specifications: &[HistorySpecification],
) -> Result<Vec<RecordedHistory>, String> {
    use std::sync::Mutex;
    let next_specification = std::sync::atomic::AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Result<RecordedHistory, String>>>> =
        Mutex::new((0..specifications.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads().min(specifications.len().max(1)) {
            scope.spawn(|| loop {
                let index = next_specification.fetch_add(1, Ordering::SeqCst);
                let Some(specification) = specifications.get(index) else {
                    break;
                };
                let built = record_the_history(specification);
                slots.lock().expect("历史表的锁没被毒化")[index] = Some(built);
            });
        }
    });
    slots
        .into_inner()
        .expect("历史表的锁没被毒化")
        .into_iter()
        .zip(specifications)
        .map(|(built, specification)| {
            built
                .expect("每个下标都有线程建过")
                .map_err(|reason| format!("{}：{reason}", history_name(specification)))
        })
        .collect()
}

struct CommandLine {
    items: Vec<String>,
    /// 调用方给的片结果库目录（[`verdict_library_directory_given_by`]）。
    verdict_library_directory: std::path::PathBuf,
}

fn parse_the_command_line(
    arguments: &[String],
    verdict_library_directory: std::path::PathBuf,
) -> Result<CommandLine, String> {
    let mut command_line = CommandLine {
        items: Vec::new(),
        verdict_library_directory,
    };
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        match argument.as_str() {
            "--item" => command_line.items.push(
                remaining
                    .next()
                    .ok_or_else(|| "--item 后面缺历史名".to_string())?
                    .clone(),
            ),
            unknown => return Err(format!("认不得的参数 {unknown}")),
        }
    }
    Ok(command_line)
}

fn selected_specifications(
    items: &[String],
    shape: OverwriteShape,
) -> Result<Vec<HistorySpecification>, String> {
    let every_specification = history_specifications_of_the_shape(shape);
    if items.is_empty() {
        return Ok(every_specification);
    }
    let mut selected = Vec::new();
    for item in items {
        let specification = every_specification
            .iter()
            .find(|specification| &history_name(specification) == item)
            .ok_or_else(|| {
                format!(
                    "没有历史叫 {item}；可点名的：{}",
                    every_specification
                        .iter()
                        .map(history_name)
                        .collect::<Vec<_>>()
                        .join("、")
                )
            })?;
        if !selected.contains(specification) {
            selected.push(*specification);
        }
    }
    Ok(selected)
}

/// 按片判完（或取库）的一批历史：合并后每条历史的计数，按历史次序。
struct JudgedHistories {
    counters_by_history: Vec<Counters>,
    judged: u64,
    reused: u64,
}

fn merge_the_report(
    histories: &[RecordedHistory],
    jobs: &[ShardJob<'_>],
    report: &RunReport,
) -> JudgedHistories {
    let names: Vec<String> = histories
        .iter()
        .map(|history| history_name(&history.specification))
        .collect();
    let mut counters_by_history = vec![Counters::default(); histories.len()];
    let (mut judged, mut reused) = (0u64, 0u64);
    for (job, result) in jobs.iter().zip(&report.results) {
        let Some(result) = result else {
            continue;
        };
        if result.reused {
            reused += 1;
        } else {
            judged += 1;
        }
        let position = names
            .iter()
            .position(|name| name == &job.history_name)
            .expect("片属于建好的某条历史");
        counters_by_history[position].merge(&result.counters);
    }
    JudgedHistories {
        counters_by_history,
        judged,
        reused,
    }
}

/// 建好 `specs` 的历史、按片判（或取库）、合并。`Err` 里是该停下交回的原因。
fn judge_the_histories(
    specifications: &[HistorySpecification],
    command_line: &CommandLine,
    sentinel_check: bool,
) -> Result<(Vec<RecordedHistory>, JudgedHistories), String> {
    let histories = build_the_histories(specifications)?;
    let fingerprint = match closure_fingerprint() {
        Ok(fingerprint) => Some(fingerprint),
        Err(reason) => {
            eprintln!("reuse_disabled: {reason}");
            None
        }
    };
    let library = if fingerprint.is_some() {
        match create_the_verdict_library_directory(&command_line.verdict_library_directory) {
            Ok(()) => Some(command_line.verdict_library_directory.as_path()),
            Err(reason) => {
                eprintln!("reuse_disabled: {reason}");
                None
            }
        }
    } else {
        None
    };
    let share = shard_share_from_the_environment()?;
    let jobs = plan_the_shards(&histories);
    let options = RunOptions {
        library,
        fingerprint: fingerprint.as_ref().filter(|_| library.is_some()),
        share,
    };
    let report = run_the_shards(&jobs, &options);
    if let Some(reason) = &report.stop {
        return Err(reason.clone());
    }
    let judged_histories = merge_the_report(&histories, &jobs, &report);
    eprintln!(
        "现判 {}、复用 {}（片 {} 个，这一份 {:?}）",
        judged_histories.judged,
        judged_histories.reused,
        jobs.len(),
        share
    );
    if sentinel_check && share.is_none() {
        let checked = recheck_the_sentinels(&jobs, &report)?;
        eprintln!("哨兵重判 {checked} 片，与库里的逐名相同");
    }
    drop(jobs);
    Ok((histories, judged_histories))
}

fn emit_counts(history_label: &str, counters: &Counters) {
    let fields: String = counters
        .0
        .iter()
        .map(|(name, amount)| format!(" {name}={amount}"))
        .collect();
    emit_result(&format!("name=counts history={history_label}{fields}"));
}

/// 同一族同一几何的历史合起来的计数（"hidden-root-unconfirmed.default" 这样的名字）。
fn aggregate_by_family_and_geometry(
    histories: &[RecordedHistory],
    counters_by_history: &[Counters],
) -> BTreeMap<String, Counters> {
    let mut aggregates: BTreeMap<String, Counters> = BTreeMap::new();
    for (history, counters) in histories.iter().zip(counters_by_history) {
        aggregates
            .entry(format!(
                "{}{}.{}",
                history.specification.overwrite_shape.label_prefix(),
                history.specification.family.name(),
                history.specification.geometry.name()
            ))
            .or_default()
            .merge(counters);
    }
    aggregates
}

const ARM_WITNESS_HISTORY_NAME: &str = "hidden-root-unconfirmed.default.units-64.overwrites-10";

/// 甲一、甲二判法对照挑状态用的那条历史：两个覆盖写形状各有一条（补臂的带 `units-kept.` 前缀）。
fn arm_witness_history_name(shape: OverwriteShape) -> String {
    format!("{}{ARM_WITNESS_HISTORY_NAME}", shape.label_prefix())
}

/// 登记 5.1 路径真跑到的补跑：Q1 在 G默认 合计是 0 时，同一组起点、影子账关着、记录读得出再跑一遍，报「择中而被盖」合计。
fn emit_the_shadow_ledger_off_control(
    command_line: &CommandLine,
    shape: OverwriteShape,
) -> Result<u64, String> {
    let specifications = shadow_ledger_off_history_specifications(shape);
    let (histories, judged) = judge_the_histories(&specifications, command_line, false)?;
    let mut covered_total = 0u64;
    for (history, counters) in histories.iter().zip(&judged.counters_by_history) {
        emit_history_summary(history);
        emit_counts(&history_name(&history.specification), counters);
        emit_questions("history", &history_name(&history.specification), counters);
        covered_total += sum_over_kept_disks(counters, &|device| {
            format!("today.kept{device}.pick.hidden_root_covered")
        });
    }
    emit_result(&format!(
        "name=positive_control control=shadow_ledger_off_covered_total histories={} hidden_root_covered_total={covered_total} status={}",
        histories.len(),
        if covered_total > 0 {
            "covered_states_exist_without_the_protection"
        } else {
            "no_covered_state_even_without_the_protection"
        }
    ));
    Ok(covered_total)
}

fn emit_the_product(
    histories: &[RecordedHistory],
    judged: &JudgedHistories,
    command_line: &CommandLine,
    shape: OverwriteShape,
) -> Result<(), String> {
    // 登记修订 12.4 的读法不加字段（第一段产物的 config 行逐字不变）；补臂（修订 12.11）加一个字段说明覆盖写的形状。
    let shape_field = match shape {
        OverwriteShape::OneUnit => "",
        OverwriteShape::KeepingTheFirstFileUnits => {
            " overwrite_shape=replacing_writes_keep_the_first_file_units_before_the_hidden_publish"
        }
    };
    emit_result(&format!(
        "name=config histories={} disk_geometries=default:{}+narrow:{} journal_ring_bytes={JOURNAL_RING_BYTES_LOCAL} root_ring_slots_per_region={ROOT_RING_SLOTS_PER_REGION_LOCAL} full_expansion_writes_at_most={FULL_EXPANSION_WRITES_AT_MOST_LOCAL} overwrites_after_the_mount={OVERWRITES_AFTER_THE_MOUNT_LOCAL} arms=today,newer_instance_record,configuration_instance{shape_field}",
        histories.len(),
        DEVICE_BYTES_DEFAULT_LOCAL,
        DEVICE_BYTES_NARROW_LOCAL
    ));
    let mut mismatches = run_constants_and_anchors();
    let (mut consistency_problems, mut refused_mounts, mut detection_failures) = (0u64, 0u64, 0u64);
    let mut confirmed_none_covered_control = ConfirmedNoneCoveredControlTotal::default();
    for history in histories {
        emit_history_summary(history);
        consistency_problems += u64::from(!history.consistency_problems.is_empty());
        refused_mounts += u64::from(history.mount.refused.is_some());
        for reading in [
            anchor_root_slot_writes_land_by_the_ring_rule(history),
            anchor_warm_up_publishes_are_at_most_three(history),
            anchor_first_publish_txg_follows_the_newest_readable(history),
        ] {
            mismatches += u64::from(reading.applicable && !reading.holds);
            emit_anchor_reading(&reading);
        }
        detection_failures += u64::from(!emit_detection_control(history));
        // 登记修订 12.12：B4 的类 a 到类 b 对照，只在已确认档带藏根的历史上跑（类 a 只出在这一族）。
        if matches!(
            history.specification.family,
            Family::HiddenRoot(Confirmation::Confirmed)
        ) {
            emit_the_confirmed_none_covered_control(history, &mut confirmed_none_covered_control);
        }
    }
    if confirmed_none_covered_control.histories > 0 {
        emit_the_confirmed_none_covered_control_total(&confirmed_none_covered_control);
        // 对照没过（violated、停机）或一次都没试成，都算进判决行的「检测对照失败」条数；全过时这个数不变。
        detection_failures += confirmed_none_covered_control.failures();
    }
    let witness_name = arm_witness_history_name(shape);
    let witness = histories
        .iter()
        .find(|history| history_name(&history.specification) == witness_name)
        .map(arm_witness_control);
    let witness_status = match &witness {
        None => "not_run",
        Some(reading) if !reading.applicable => "not_run",
        Some(reading) if reading.holds => "holds",
        Some(_) => "violated",
    };
    if let Some(reading) = &witness {
        emit_anchor_reading(reading);
    }
    for (history, counters) in histories.iter().zip(&judged.counters_by_history) {
        emit_counts(&history_name(&history.specification), counters);
        emit_questions("history", &history_name(&history.specification), counters);
    }
    let aggregates = aggregate_by_family_and_geometry(histories, &judged.counters_by_history);
    for (label, counters) in &aggregates {
        emit_counts(label, counters);
        emit_questions("aggregate", label, counters);
    }
    let covered_on_the_default_geometry: u64 = aggregates
        .iter()
        .filter(|(label, _)| {
            label
                .strip_prefix(shape.label_prefix())
                .is_some_and(|rest| rest.starts_with("hidden-root-") && rest.ends_with(".default"))
        })
        .map(|(_, counters)| {
            sum_over_kept_disks(counters, &|device| {
                format!("today.kept{device}.pick.hidden_root_covered")
            })
        })
        .sum();
    let default_unconfirmed_label =
        format!("{}hidden-root-unconfirmed.default", shape.label_prefix());
    let hidden_root_picks_on_the_default_unconfirmed: Option<u64> =
        aggregates.get(&default_unconfirmed_label).map(|counters| {
            sum_over_kept_disks(counters, &|device| {
                format!("today.kept{device}.pick.hidden_root_covered")
            }) + sum_over_kept_disks(counters, &|device| {
                format!("today.kept{device}.pick.hidden_root_intact")
            })
        });
    let off_switch_covered = if covered_on_the_default_geometry == 0
        && hidden_root_picks_on_the_default_unconfirmed.is_some()
    {
        Some(emit_the_shadow_ledger_off_control(command_line, shape)?)
    } else {
        None
    };
    let slot_hit_states: u64 = aggregates
        .values()
        .map(|counters| {
            counters.count_named(
                "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot",
            )
        })
        .sum();
    emit_result(&format!(
        "name=verdict constants_and_anchor_mismatches={mismatches} histories_with_consistency_problems={consistency_problems} histories_whose_mount_was_refused={refused_mounts} detection_control_failures={detection_failures} arm_witness_control={witness_status} path_reached_on_the_default_unconfirmed_histories={} hidden_root_covered_states_on_the_default_geometry={covered_on_the_default_geometry} shadow_ledger_off_covered_states={} first_publish_txg_hits_the_hidden_root_slot_states={slot_hit_states}",
        hidden_root_picks_on_the_default_unconfirmed.map_or("not_run".to_string(), |picks| (picks > 0).to_string()),
        off_switch_covered.map_or("not_run".to_string(), |covered| covered.to_string()),
    ));
    Ok(())
}

/// `library_variable_value` 是 `E182_VERDICT_LIBRARY` 的值（`main` 现读环境交进来）：没给就在建任何历史之前退
/// [`NO_VERDICT_LIBRARY_EXIT_CODE`]。
fn run_single_device(
    arguments: &[String],
    shape: OverwriteShape,
    library_variable_value: Option<&str>,
) -> i32 {
    let verdict_library_directory = match verdict_library_directory_given_by(library_variable_value)
    {
        Ok(directory) => directory,
        Err(refusal) => {
            eprintln!("用法错：{refusal}");
            return NO_VERDICT_LIBRARY_EXIT_CODE;
        }
    };
    let command_line = match parse_the_command_line(arguments, verdict_library_directory) {
        Ok(command_line) => command_line,
        Err(reason) => {
            eprintln!("用法错：{reason}");
            return 2;
        }
    };
    let specifications = match selected_specifications(&command_line.items, shape) {
        Ok(specifications) => specifications,
        Err(reason) => {
            eprintln!("用法错：{reason}");
            return 2;
        }
    };
    let (histories, judged) = match judge_the_histories(&specifications, &command_line, true) {
        Ok(judged_histories) => judged_histories,
        Err(reason) => {
            eprintln!("STOP: {reason}");
            return 3;
        }
    };
    if matches!(shard_share_from_the_environment(), Ok(Some(_))) {
        return 0;
    }
    if let Err(reason) = emit_the_product(&histories, &judged, &command_line, shape) {
        eprintln!("STOP: {reason}");
        return 3;
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    0
}

fn run_anchors_mode() -> i32 {
    let mismatches = run_constants_and_anchors();
    match closure_fingerprint() {
        Ok(fingerprint) => emit_result(&format!(
            "name=closure_fingerprint status=keyed key={} toolchain_lines={}",
            fingerprint.key_hex,
            fingerprint.toolchain_lines.len()
        )),
        Err(reason) => emit_result(&format!(
            "name=closure_fingerprint status=refused detail={}",
            token_of(&reason.chars().take(400).collect::<String>())
        )),
    }
    emit_result(&format!(
        "name=verdict constants_and_anchor_mismatches={mismatches}"
    ));
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    i32::from(mismatches > 0)
}

fn run_list_items() -> i32 {
    for specification in every_history_specification() {
        println!("{}", history_name(&specification));
    }
    0
}

/// 登记 5.5 最后一条：建全部历史报状态数（闭式），再在点名的历史上抽 100 个状态量完整判一个状态与单独一次 `recover` 的挂钟，按实测外推。
/// 挂钟不进产物，这一个模式的输出不入库。
fn run_measure(arguments: &[String]) -> i32 {
    let measured_name = arguments
        .first()
        .map_or(ARM_WITNESS_HISTORY_NAME, String::as_str);
    let specifications = main_history_specifications();
    let started = std::time::Instant::now();
    let histories = match build_the_histories(&specifications) {
        Ok(histories) => histories,
        Err(reason) => {
            eprintln!("建历史失败：{reason}");
            return 2;
        }
    };
    eprintln!(
        "建 {} 条历史用了 {:.1} 秒",
        histories.len(),
        started.elapsed().as_secs_f64()
    );
    let mut total_states = 0u64;
    for history in &histories {
        let states: u64 = (0..=history.segments.len())
            .map(|shard_index| {
                history.segments.get(shard_index).map_or(1, |segment| {
                    state_count_of_segment(&history.writes, segment).unwrap_or(0)
                })
            })
            .sum();
        total_states += states;
        emit_result(&format!(
            "name=measure_history history={} segments={} writes={} states_closed_form={states} mount_refused={}",
            history_name(&history.specification),
            history.segments.len(),
            history.writes.len(),
            history.mount.refused.is_some()
        ));
    }
    let Some(measured) = histories
        .iter()
        .find(|history| history_name(&history.specification) == measured_name)
    else {
        eprintln!("没有历史叫 {measured_name}");
        return 2;
    };
    let measured_states: u64 = (0..=measured.segments.len())
        .map(|shard_index| {
            measured.segments.get(shard_index).map_or(1, |segment| {
                state_count_of_segment(&measured.writes, segment).unwrap_or(0)
            })
        })
        .sum();
    let step = (measured_states / 100).max(1);
    let (mut visited, mut sampled) = (0u64, 0u64);
    let (mut evaluate_seconds, mut recover_seconds) = (0f64, 0f64);
    for shard_index in 0..=measured.segments.len() {
        let walked = for_each_state_of_segment(
            &measured.writes,
            &measured.segments,
            shard_index,
            &mut |persisted| {
                visited += 1;
                if visited % step != 0 || sampled >= 100 {
                    return Ok(());
                }
                sampled += 1;
                let image = CrashImage {
                    base: &measured.base,
                    writes: &measured.writes,
                    persisted: persisted.to_vec(),
                };
                let recover_started = std::time::Instant::now();
                let _report = recover(
                    &kept_disk(&image, DeviceIdentity(0)),
                    JournalPolicy::Consult,
                );
                recover_seconds += recover_started.elapsed().as_secs_f64();
                let evaluate_started = std::time::Instant::now();
                evaluate_one_state(measured, persisted, &mut Counters::default())?;
                evaluate_seconds += evaluate_started.elapsed().as_secs_f64();
                Ok(())
            },
        );
        if let Err(reason) = walked {
            eprintln!("STOP: {reason}");
            return 3;
        }
    }
    let seconds_per_state =
        evaluate_seconds / f64::from(u32::try_from(sampled.max(1)).expect("抽样数装得进 u32"));
    #[allow(
        clippy::cast_precision_loss,
        reason = "外推只要量级，状态总数远小于 2^52"
    )]
    let single_thread_seconds = total_states as f64 * seconds_per_state;
    let threads = worker_threads();
    #[allow(clippy::cast_precision_loss, reason = "线程数远小于 2^52")]
    let spread_seconds = single_thread_seconds / threads as f64;
    emit_result(&format!(
        "name=measure_timing history={measured_name} states_of_this_history={measured_states} sampled_states={sampled} evaluate_seconds_for_the_sample={evaluate_seconds:.3} one_recover_seconds_for_the_sample={recover_seconds:.3} seconds_per_evaluated_state={seconds_per_state:.6}"
    ));
    emit_result(&format!(
        "name=measure_extrapolation states_all_histories={total_states} single_thread_seconds={single_thread_seconds:.0} threads={threads} seconds_spread_over_the_threads={spread_seconds:.0} hours_spread_over_the_threads={:.2}",
        spread_seconds / 3600.0
    ));
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    0
}

/// 样本模式：在点名的历史上取三个状态（什么都没持久、前一半的段全持久、整条流全持久），逐个报两块盘都在与各只留一块盘时 `recover` 的结局
/// （失败成员原样），好看单盘恢复为什么成功或失败。输出不入库。
fn run_outcome_samples(arguments: &[String]) -> i32 {
    let name = arguments
        .first()
        .map_or(ARM_WITNESS_HISTORY_NAME, String::as_str);
    let Some(specification) = every_history_specification()
        .into_iter()
        .find(|specification| history_name(specification) == name)
    else {
        eprintln!("没有历史叫 {name}");
        return 2;
    };
    let history = match record_the_history(&specification) {
        Ok(history) => history,
        Err(reason) => {
            eprintln!("建历史失败：{reason}");
            return 2;
        }
    };
    for (label, persisted) in [
        "nothing_persisted",
        "first_half_of_the_segments",
        "everything_persisted",
    ]
    .into_iter()
    .zip(three_states_for_the_controls(&history))
    {
        let image = CrashImage {
            base: &history.base,
            writes: &history.writes,
            persisted,
        };
        let both = recover(&image, JournalPolicy::Consult);
        let mut reports = vec![("both_disks".to_string(), both)];
        for device_number in [0u32, 1u32] {
            reports.push((
                format!("only_disk_{device_number}"),
                recover(
                    &kept_disk(&image, DeviceIdentity(device_number)),
                    JournalPolicy::Consult,
                ),
            ));
        }
        for (kept, report) in reports {
            let failure = match &report.outcome {
                RecoveryOutcome::Failed { failure, .. } => format!("{failure:?}"),
                RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => {
                    "none".to_string()
                }
            };
            emit_result(&format!(
                "name=outcome_sample history={name} state={label} kept={kept} class={} root={} effective_root={} failure={} journal={}",
                outcome_class(&report.outcome),
                token_of(&format!("{:?}", outcome_root(&report.outcome))),
                token_of(&format!(
                    "{:?}",
                    report.effective_root.map(|(instance, txg)| (instance.0, txg.0))
                )),
                token_of(&failure),
                token_of(&format!("{:?}", report.journal))
            ));
        }
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    0
}

/// 准入判法退 78（拒绝）时装置也退 78；别的非 0 退 1（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `main` 的第一句：直接起 `python3 preflight.py check <本文件> [--force] -- <参数…>`（不经 `sh -c`）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做；强制跑（判法的第一行以 `forced` 起头）时交回没满足的条件摘要，要写进产物。
fn preflight() -> Option<String> {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/e182_unreadable_abandoned_root_forks/main.rs"
    );
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.claude/singlefs-ai-sop/scripts/preflight.py"
    );
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(script).arg("check").arg(source);
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
        Some(_) | None => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    forced_summary_of_the_preflight_verdict(&verdict_line)
}

/// 准入判法 stdout 第一行（制表符分段）以 `forced` 起头时，交回没满足的条件摘要（最后一段，空白换成下划线），要写进产物；
/// 满足（`met`）或没声明时是 `None`。
fn forced_summary_of_the_preflight_verdict(verdict_line: &str) -> Option<String> {
    if verdict_line.starts_with("forced") {
        let last_segment = verdict_line.rsplit('\t').next().unwrap_or("");
        Some(
            last_segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("_"),
        )
    } else {
        None
    }
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    if let Some(summary) = forced_summary {
        if arguments.first().map(String::as_str) == Some("list-items") {
            eprintln!("name=preflight forced={summary}");
        } else {
            emit_result(&format!("name=preflight forced={summary}"));
        }
    }
    let exit_code = match arguments.first().map(String::as_str) {
        Some("anchors") => run_anchors_mode(),
        Some("list-items") => run_list_items(),
        Some("measure") => run_measure(&arguments[1..]),
        Some("outcome-samples") => run_outcome_samples(&arguments[1..]),
        Some("single-device") => run_single_device(
            &arguments[1..],
            OverwriteShape::OneUnit,
            std::env::var(LIBRARY_ENVIRONMENT_VARIABLE).ok().as_deref(),
        ),
        Some("single-device-units-kept") => run_single_device(
            &arguments[1..],
            OverwriteShape::KeepingTheFirstFileUnits,
            std::env::var(LIBRARY_ENVIRONMENT_VARIABLE).ok().as_deref(),
        ),
        Some(_) | None => {
            eprintln!("用法：e182_unreadable_abandoned_root_forks anchors | list-items | measure [历史名] | outcome-samples [历史名] | single-device [--item <历史名>]… | single-device-units-kept [--item <历史名>]…");
            2
        }
    };
    std::process::exit(exit_code);
}

// ============================================================================
// 单测：钉绝对值的断言（登记第七节）、每条臂与每个检测各一条会红的用例（变异表 `crates/mutations.tsv` 里 E182 的行点名这里的测试名）。
// ============================================================================

#[cfg(test)]
mod helpers_shared_by_the_tests;
#[cfg(test)]
mod tests;
