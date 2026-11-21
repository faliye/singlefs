//! E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 第一段（登记 `research/prompts/e183-preregistration.md` 第五节 5.5「第一段」）：
//! 回答问题单 `research/prompts/m3-close-cost-measurement-questions.md` 的 H4 一行——最小能 mkfs 并可写挂载的盘上，准入式子（D28（挂载期承诺量）
//! 已定项 1）里除容量、已分配、不可回收之外的六项各多大、谁占大头。
//!
// admission: always 每跑一次都在今天这份 crates 上现量一遍代价数（H4 的读数、H2 的耗时与堆量、H5 的挂载历史），计时每次都是新观测，上一次跑完不替这一次作保
// run-condition: command python3
//!
//! 这一段做的事：本地常量与 `crates/` 回比（登记表头常量表）、计数的全局分配器与 PC-堆、按区间数读写的块设备包装（E31）、
//! 几何锚点的本地移植（A30、A31、A33）、H4 的最小盘搜索（X14：三个最小盘、单调核）、准入读数逐项报（X15）、PC-准入、K30–K32 与 G7 的对读数的核。
//!
//! 第二段（登记 5.5「第二段」，问题单 H2）：E33 的容量格与记录格（X4–X10，文件后端稀疏镜像，每格每轮一个进程、一片结果文件）与 E37 里 H2 的判定
//! （X10 的斜率与类、5 轮同类才写类、PC-线性、PC-平、几何敏感性、判别力自证 ①）。入口 `h2 list-shards`、`h2 shard`、`h2 merge`、`h2 cells-beyond-the-threshold`。
//! 第二段的读法与跑法照 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 第 2 次重跑登记 `research/prompts/e183-r2-prereg.md`：
//! X22（每格每轮读进程自报的内存高水位 H，与登记写死的阈值 T 比，分四类）、X24（台与镜像目录的文件系统写进 `h2_config`）、
//! X26（服务撞到两台最大时执行员写的 `<片名>.service-outcome`）、PC-高水位、A43 几何钉值、一轮一台（G10）、R2f 按台、阈值敏感性与判别力自证 ④。
//! 不套内存包装：内存上限归统一任务服务，跑法脚本 `scripts/experiment-apparatus/e183-capacity-cells.sh` 照服务的分片三步起本装置。
//! 第三段（登记 5.5「第三段」，问题单 H5）：E36 的清扫历史（X16–X21，24 格加「交给后台」对照 1 格，一格一片）、锚点模型的本地移植（A34、A35）、
//! K33、K34、A36、PC-不扫、PC-差分，与 E37 里 H5 的判定（R5b 甲、R5c、几何敏感性、判别力自证 ②）。入口 `h5 list-shards`、`h5 shard`、`h5 merge`。
//! 第四段（H3 读缓存）这一段不做。
//!
//! 只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的任何文件；产物按 `E7RESULT` 行打印到标准输出，每项耗时打到标准错误（产物逐字节可复跑）。
//! 第一段入口：`--list-items`、`--item <项>`、`all`（输出与第一段的入库产物逐字节相同，单测 `segment_one_all_output_equals_the_registered_product_byte_for_byte` 证）。

#![deny(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::{
    admission_reading_before_a_publish, admission_reading_of_a_writable_mount, AdmissionReading,
    SpaceAdmission,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, FileBackedBlockDevice, PhysicalBlockSizeInBytes,
    WriteDurability,
};
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration,
    mount_writable_with_space_admission, unmount, MountSpaceAdmission, Mounted, ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{instance_table_chain_of_root, readable_roots_with_ring_slots};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolVersion, PoolWriter, TransactionUnit,
};
use singlefs_verification::memory_pool::SparseBlockDevice;

mod admission_readings;
mod capacity_and_record_cells;
mod capacity_merge;
mod capacity_slope_verdict;
mod constant_and_anchor_comparisons;
mod counting_allocator_and_heap_control;
mod counting_block_device;
mod item_runs;
mod local_geometry_anchors;
mod mount_attempts;
mod pool_construction;
mod product_lines;
mod shard_files;
mod smallest_device_search;
mod sweep_histories;
mod sweep_merge;

use admission_readings::*;
use capacity_and_record_cells::*;
use capacity_merge::*;
use capacity_slope_verdict::*;
use constant_and_anchor_comparisons::*;
use counting_allocator_and_heap_control::*;
use counting_block_device::*;
use item_runs::*;
use local_geometry_anchors::*;
use mount_attempts::*;
use pool_construction::*;
use product_lines::*;
use shard_files::*;
use smallest_device_search::*;
use sweep_histories::*;
use sweep_merge::*;

/// 这个装置的固定 fsid：与别的 `eNNN` 装置各自独立取一份。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x38, 0x33, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING_BYTES: u32 = 4096;
const MEBIBYTE: u64 = 1 << 20;
const GIBIBYTE: u64 = 1 << 30;

// ============================================================================================
// 本地常量（登记表头常量表）：值抄自下面各行点名的 kb 登记位，不从 `crates/` 引；`constants_against_the_implementation` 逐个回比。
// ============================================================================================

/// 每片实例表数据行：D28（挂载期承诺量） 已定项 3「行宽 88 ⇒ 369 行」；D18（块里携带什么信息） 已定项 11「数据行 369」。
const INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE: u64 = 369;
/// N_switch、暖机至多 R：`28-挂载期承诺量.md` 已定项 3。
const INSTANCE_SWITCHES_ALLOWED_PER_MOUNT: u64 = 3;
const WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH: u64 = 3;
/// 实例表一片的字节：`28-挂载期承诺量.md` 已定项 3「32768 × max(1, …)」。
const INSTANCE_TABLE_PAGE_BYTES: u64 = 32768;
/// 清扫阈值：`.claude/kb/checks-owed.md` C690（清扫触发阈值的默认值没定） 那一行「阈值默认 365」；
/// 名字照实现那一份（`singlefs_core::instance_table_sweep::InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`）。
const INSTANCE_TABLE_ROW_COUNT_THRESHOLD_DEFAULT_ROWS: u64 = 365;
/// journal 记录宽、安全系数、默认环长：D23（journal 的角色与格式） 已定项 18、19（环 ≤ 容量 ÷ 4，默认 768 MiB）。
const JOURNAL_RECORD_BYTES: u64 = 4096;
const JOURNAL_SAFETY_FACTOR: u64 = 3;
const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 分配记录树叶罩槽数、内部扇出、删除批 K：D8（核心索引结构） 已定项 14、16；D28（挂载期承诺量） 已定项 1「环窗口」一条。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
const DELETION_BATCH_DATA_UNITS: u64 = 144;
/// 默认环长下的单元区起点：D3（空间分配） 已定项 10 ④（环末尾的下一个槽）。
const UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING: u64 = 50176;
/// mkfs 默认每区槽数 S：`22-单元原子性怎么合成.md` 已定项 1 字段表；`layout/01-first-txn.md` mkfs 写 8。
const SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
/// 几何锚点（`anchors_e183.py`）与读写计数包装要的布局常量：槽宽 16384、journal 环从槽 1024 起（`23-journal的角色与格式.md` 已定项 2 的布局）、
/// 根环三个区域从槽 64 起每区域隔 3 个 1 MiB chunk（`22-单元原子性怎么合成.md` 已定项 2）、系统配置两槽。
const SLOT_BYTES: u64 = 16384;
const JOURNAL_RING_START_SLOT: u64 = 1024;
const ROOT_RING_REGIONS: u64 = 3;
const ROOT_RING_BASE_SLOT: u64 = 64;
const ROOT_RING_CHUNK_BYTES: u64 = 1_048_576;
const ROOT_RING_PRIME_STEP: u64 = 3;
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;
/// 池是两块盘（第一个可运行目标，D2（RAID 条带策略） 已定项 9）。
const POOL_DEVICE_COUNT: u64 = 2;

/// 合并的收尾：停机就打停机行、退 3；否则打完成行、退 0。
fn finish_a_merge(emitter: &mut Emitter, end: MergeEnd) -> i32 {
    match end {
        MergeEnd::Complete => {
            emitter.finish();
            0
        }
        MergeEnd::Stopped { reason } => {
            emitter.emit(&format!("name=stop reason={reason}"));
            emitter.finish();
            3
        }
    }
}

/// 这个进程自己的程序文件字节数；读不到是 none，不当 0。
/// 程序文件取起这个进程的命令行里写的程序路径（跑法脚本写绝对路径起它）：函数单元键（`scripts/gate/lib_function_units.py`）
/// 把 `current_exe()` 只问路径的写法拒算，实验键就算不出、「输入没变不重跑」用不上。
fn own_program_file_bytes() -> Option<u64> {
    std::env::args_os()
        .next()
        .and_then(|program| std::fs::metadata(program).ok())
        .map(|metadata| metadata.len())
}

/// `h2 merge`：45 片按片的次序接成产物（没做完的片记结局、不接行），接着打判定。
/// 合并这个进程自己的程序文件字节数给「贴着阈值」与 X27 用（各片与合并是同一份二进制，sha256 由跑法脚本核）；读不到记 none，不当 0。
fn run_capacity_merge(directory: &Path, binary_sha256: &str) -> i32 {
    let binary_bytes = own_program_file_bytes();
    let mut emitter = Emitter::to_standard_output();
    emitter.emit(&format!(
        "name=config experiment=e183 segment=2 scope=h2 shards={} binary_sha256={binary_sha256} memory_threshold_bytes={MEMORY_THRESHOLD_BYTES} binary_bytes={}",
        capacity_shards().len(),
        optional_number_text(binary_bytes),
    ));
    for (shard, _, _) in capacity_shards() {
        for line in finished_shard_lines(directory, &shard).unwrap_or_default() {
            let body = line.strip_prefix("E7RESULT ").unwrap_or(&line).to_string();
            emitter.emit(&body);
        }
    }
    let statuses = read_capacity_statuses(directory, binary_sha256);
    let end = emit_capacity_summary(&mut emitter, &statuses, binary_bytes);
    finish_a_merge(&mut emitter, end)
}

/// `h2 cells-beyond-the-threshold`：运行目录里已有一轮在阈值 T 之外的格，一行一个格名（跑法脚本照它跳过同一格别的轮，X26）。
fn run_capacity_cells_beyond_the_threshold(directory: &Path, binary_sha256: &str) -> i32 {
    let statuses = read_capacity_statuses(directory, binary_sha256);
    for cell in cells_beyond_the_threshold(&statuses, MEMORY_THRESHOLD_BYTES) {
        println!("{}", cell.name());
    }
    0
}

/// 一片跑完之后：做完 ⇒ 原子地写片文件、标准输出报一行 `shard_written`、退 0；停机 ⇒ 片文件不写，收着的行与停机行打到标准输出、退 3；
/// 片文件写不成退 1。耗时打到标准错误。
fn finish_a_shard(
    shard: &str,
    output: &Path,
    emitter: Emitter,
    end: ShardEnd,
    started: Instant,
) -> i32 {
    let lines = emitter.into_collected_lines();
    let exit_code = match end {
        ShardEnd::Finished => match write_shard_file_atomically(output, shard, &lines) {
            Ok(()) => {
                println!(
                    "E7RESULT name=shard_written shard={shard} lines={}",
                    lines.len() + 1
                );
                0
            }
            Err(error) => {
                eprintln!("片文件写不成：{} {error}", output.display());
                1
            }
        },
        ShardEnd::Stopped { reason } => {
            for line in &lines {
                println!("{line}");
            }
            println!("E7RESULT name=stop shard={shard} reason={reason}");
            3
        }
    };
    eprintln!(
        "name=item_done item={shard} exit={exit_code} judged={} red={} reused=不复用 seconds={:.1}",
        lines.len(),
        i32::from(exit_code != 0),
        started.elapsed().as_secs_f64()
    );
    exit_code
}

fn run_capacity_shard_command(
    shard: &str,
    output: &Path,
    image_directory: &Path,
    context: &CapacityShardContext,
) -> i32 {
    let Some((cell, round)) = capacity_shard_by_name(shard) else {
        eprintln!("认不得的 H2 片名：{shard}（`h2 list-shards` 列全部片）");
        return 2;
    };
    let started = Instant::now();
    let mut emitter = Emitter::collecting_lines();
    let end = run_capacity_cell(shard, cell, round, image_directory, context, &mut emitter);
    finish_a_shard(shard, output, emitter, end, started)
}

fn run_sweep_shard_command(shard: &str, output: &Path, binary_sha256: &str) -> i32 {
    let Some(cell) = sweep_shards()
        .into_iter()
        .find(|(name, _)| name == shard)
        .map(|(_, cell)| cell)
    else {
        eprintln!("认不得的 H5 片名：{shard}（`h5 list-shards` 列全部片）");
        return 2;
    };
    let started = Instant::now();
    let mut emitter = Emitter::collecting_lines();
    let end = run_sweep_history_shard(shard, cell, binary_sha256, &mut emitter);
    finish_a_shard(shard, output, emitter, end, started)
}

/// 第二、三段的入口（`h2 …`、`h5 …`）；不是这两段的参数交回 `None`，由第一段的入口接着认。
fn run_segment_two_or_three(arguments: &[&str]) -> Option<i32> {
    Some(match arguments {
        ["h2", "list-shards"] => {
            for (shard, _, round) in capacity_shards_in_running_order() {
                println!("{shard}\t{round}");
            }
            0
        }
        ["h2", "shard", shard, "--shard-output", output, "--image-directory", images, "--binary-sha256", sha256, "--host", host, "--service-attempt", attempt, "--image-filesystem", filesystem] => {
            match CapacityShardContext::parse(sha256, host, attempt, filesystem) {
                Some(context) => run_capacity_shard_command(
                    shard,
                    Path::new(output),
                    Path::new(images),
                    &context,
                ),
                None => {
                    eprintln!("h2 shard 的 --host 只认「本机」「第二台」，--service-attempt 写 <片号>/<片数>:<第几次>，--image-filesystem 写一个不带空白的词");
                    2
                }
            }
        }
        ["h2", "shard", ..] => {
            eprintln!("h2 shard 要写全：<片名> --shard-output <片文件> --image-directory <镜像目录> --binary-sha256 <…> --host <本机|第二台> --service-attempt <片号>/<片数>:<第几次> --image-filesystem <类型>");
            2
        }
        ["h2", "merge", directory, "--binary-sha256", sha256] => {
            run_capacity_merge(Path::new(directory), sha256)
        }
        ["h2", "cells-beyond-the-threshold", directory, "--binary-sha256", sha256] => {
            run_capacity_cells_beyond_the_threshold(Path::new(directory), sha256)
        }
        ["h5", "list-shards"] => {
            for (shard, _) in sweep_shards() {
                println!("{shard}\t1");
            }
            0
        }
        ["h5", "shard", shard, "--shard-output", output, "--binary-sha256", sha256] => {
            run_sweep_shard_command(shard, Path::new(output), sha256)
        }
        ["h5", "merge", directory, "--binary-sha256", sha256] => {
            run_sweep_merge(Path::new(directory), sha256)
        }
        _ => return None,
    })
}

const ITEM_NAMES: [&str; 10] = [
    "constants",
    "positive-control-heap",
    "anchors",
    "h4-small-disk-slots-per-region-4",
    "h4-small-disk-slots-per-region-8",
    "h4-small-disk-slots-per-region-16",
    "h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes",
    "h4-default-disk-slots-per-region-4",
    "h4-default-disk-slots-per-region-8",
    "h4-default-disk-slots-per-region-16",
];

/// 项名到它的运行；认不得的项名交回 `None`。
fn measure_and_emit_the_item_named(emitter: &mut Emitter, item: &str) -> Option<ItemOutcome> {
    let device_condition_only = |invalid_device_conditions| ItemOutcome {
        dominance_points: Vec::new(),
        invalid_device_conditions,
    };
    Some(match item {
        "constants" => device_condition_only(run_constants_item(emitter)),
        "positive-control-heap" => device_condition_only(run_positive_control_heap_item(emitter)),
        "anchors" => device_condition_only(run_anchors_item(emitter)),
        "h4-small-disk-slots-per-region-4" => {
            run_small_disk_item(emitter, item, SMALL_DISK_JOURNAL_RING_BYTES, 4)
        }
        "h4-small-disk-slots-per-region-8" => {
            run_small_disk_item(emitter, item, SMALL_DISK_JOURNAL_RING_BYTES, 8)
        }
        "h4-small-disk-slots-per-region-16" => {
            run_small_disk_item(emitter, item, SMALL_DISK_JOURNAL_RING_BYTES, 16)
        }
        "h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes" => {
            run_small_disk_item(emitter, item, ALTERNATIVE_JOURNAL_RING_BYTES, 8)
        }
        "h4-default-disk-slots-per-region-4" => run_default_disk_item(emitter, item, 4),
        "h4-default-disk-slots-per-region-8" => run_default_disk_item(emitter, item, 8),
        "h4-default-disk-slots-per-region-16" => run_default_disk_item(emitter, item, 16),
        _ => return None,
    })
}

/// 跑给定的项，`with_summary` 时在末尾跨点汇总；返回退出码（设备有效性的红 > 0 退 1）。
fn measure_the_named_items_returning_the_exit_code(items: &[&str], with_summary: bool) -> i32 {
    let mut emitter = Emitter::to_standard_output();
    emitter.emit(&format!(
        "name=config experiment=e183 segment=1 scope=h4 items={} slots_per_region_sampling_points={:?} default_device_mebibytes={} small_disk_journal_ring_bytes={SMALL_DISK_JOURNAL_RING_BYTES} alternative_journal_ring_bytes={ALTERNATIVE_JOURNAL_RING_BYTES}",
        items.join(","),
        SLOTS_PER_REGION_SAMPLING_POINTS,
        DEFAULT_DEVICE_BYTES / MEBIBYTE,
    ));
    let mut invalid_device_conditions = 0;
    let mut dominance_points: Vec<DominancePoint> = Vec::new();
    for item in items {
        let started = Instant::now();
        let lines_before = emitter.emitted;
        let outcome =
            measure_and_emit_the_item_named(&mut emitter, item).expect("项名在入口处核过");
        invalid_device_conditions += outcome.invalid_device_conditions;
        dominance_points.extend(outcome.dominance_points);
        emitter.emit(&format!("name=shard_done item={item}"));
        eprintln!(
            "name=item_done item={item} exit={} judged={} red={} reused=不复用 seconds={:.1}",
            i32::from(outcome.invalid_device_conditions > 0),
            emitter.emitted - lines_before,
            outcome.invalid_device_conditions,
            started.elapsed().as_secs_f64(),
        );
        let is_a_device_check = matches!(*item, "constants" | "positive-control-heap" | "anchors");
        if is_a_device_check && outcome.invalid_device_conditions > 0 {
            emitter.emit(&format!(
                "name=stop item={item} reason=device_check_failed invalid_device_conditions={invalid_device_conditions}"
            ));
            emitter.finish();
            return 1;
        }
    }
    if with_summary {
        let small_points: Vec<DominancePoint> = dominance_points
            .iter()
            .filter(|point| point.item.starts_with("h4-small"))
            .cloned()
            .collect();
        let default_points: Vec<DominancePoint> = dominance_points
            .iter()
            .filter(|point| point.item.starts_with("h4-default"))
            .cloned()
            .collect();
        let small_stable = emit_dominance_across_points(&mut emitter, "small_disks", &small_points);
        let default_stable =
            emit_dominance_across_points(&mut emitter, "default_disks", &default_points);
        let self_proof_holds = emit_discrimination_self_proof(&mut emitter, &dominance_points);
        emitter.emit(&format!(
            "name=verdict item=h4_summary dominant_item_stable_across_small_disk_points={small_stable} dominant_item_stable_across_default_disk_points={default_stable} discrimination_self_proof_holds={self_proof_holds} invalid_device_conditions={invalid_device_conditions}"
        ));
    }
    emitter.finish();
    i32::from(invalid_device_conditions > 0)
}

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
///
/// 路径不写成「清单目录往上两级」或 `format!` 拼的串：函数单元键（`scripts/gate/lib_function_units.py`）把清单目录拼出的目录
/// 当运行期路径拒算；点名清单目录里这份源文件时它的字节进键，仓根从它往上取。
fn preflight() -> Option<String> {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin")
        .join("e183_capacity_read_cache_reserve_and_sweep_costs")
        .join("main.rs");
    let repository_root = source
        .ancestors()
        .nth(6)
        .expect("源文件在 <仓根>/crates/singlefs-implementation-experiments/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs/ 下");
    let script = repository_root
        .join(".claude")
        .join("singlefs-ai-sop")
        .join("scripts")
        .join("preflight.py");
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
    forced_summary_of(&first_line)
}

/// `preflight.py check` 打的头一行以 `forced` 起头时，把后面的摘要当成强制跑的标记（空格换下划线，免得一行里拆成几个字段）。
fn forced_summary_of(first_line: &str) -> Option<String> {
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let usage = || {
        eprintln!("用法：e183_capacity_read_cache_reserve_and_sweep_costs --list-items | --item <项> | all");
        eprintln!("项：{}", ITEM_NAMES.join(" "));
        eprintln!("第二段：h2 list-shards | h2 shard <片名> --shard-output <片文件> --image-directory <镜像目录> --binary-sha256 <程序文件 sha256> --host <本机|第二台> --service-attempt <片号>/<片数>:<第几次> --image-filesystem <类型> | h2 merge <运行目录> --binary-sha256 <…> | h2 cells-beyond-the-threshold <运行目录> --binary-sha256 <…>");
        eprintln!("第三段：h5 list-shards | h5 shard <片名> --shard-output <片文件> --binary-sha256 <…> | h5 merge <片目录> --binary-sha256 <…>");
    };
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let Some(exit_code) = run_segment_two_or_three(&words) {
        std::process::exit(exit_code);
    }
    let exit_code = match words.as_slice() {
        ["--list-items"] => {
            for item in ITEM_NAMES {
                println!("{item}");
            }
            0
        }
        ["--item", item] if ITEM_NAMES.contains(item) => {
            measure_the_named_items_returning_the_exit_code(&[*item], false)
        }
        ["all"] => measure_the_named_items_returning_the_exit_code(&ITEM_NAMES, true),
        _ => {
            usage();
            2
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod helpers_shared_by_the_tests;
#[cfg(test)]
mod tests;
