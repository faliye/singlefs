// task-server-gpu-arguments: segment1 positive-controls
// 这两个模式经 `checker_violations` 把盘交 `GivenStatesEngine` 判；s1-history 不判、不要卡。带 gpu 编好的二进制直接交统一任务服务时，
// 由服务按 task-server-gpu-arguments 的声明清场、分卡。
//! E168（删除借保留量的环窗口）入库装置：跑前登记 `research/prompts/e168-preregistration.md`（问题单
//! `research/prompts/m3-ring-window-questions.md`）。只读、只驱动与观测 `crates/` 今天的发布路径、准入读数与挂载内回收，
//! 不改 `crates/singlefs-core`、`crates/singlefs-checker`。
//!
//! 删除路径今天不在 `crates/` 里：主工作区这一份里 [`delete_path_hooks`] 是桩（登记第十二节修订第 2 条），一调就打
//! `E name=stop reason=delete_path_patch_absent` 并退非零；补丁表 `scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv`
//! 在 `cp -a` 出的副本里给 core 加只供测试的删除入口与开关，再把这个桩整个换成接上它们的那一份。`s1-history` 模式不调删除入口，
//! 两份二进制都能跑（停机条款 S1：副本开关全关时与主工作区逐字节、逐数相同）。
//!
//! 用法：`e168_delete_ring_window_borrow s1-history`；`e168_delete_ring_window_borrow segment1 [--geometry <几何名>]`；
//! 只重跑阳性对照：`e168_delete_ring_window_borrow positive-controls [--geometry <几何名>]`（W 臂账里那一份要的 W_max 现跑那个几何上
//! N-A、K = 1 那一组梯子算，那一组的行照打；登记第十二节修订第 13 条）。
//! 每一行以 `E name=` 开头；停机条款不过打 `E name=stop reason=…` 并退 4。`segment1` 与 `positive-controls` 打完 verdict 行之后，
//! 阳性对照有一格不出拒绝（V1）或 B 臂有一格的拒绝不是经会话推抬 F 那一步交回的（U12）也打停机行、退 4（登记第十二节修订第 16、17 条）。
//!
//! P1 取读法 C：5.1 第 1–3 步（写进文件、重挂载、压满）都做得成的最小盘（登记第十二节修订第 10 条）；阳性对照的 free_0 读「写进 N 个
//! 单元、压满之后」那块盘本身（修订第 11 条）。
//!
//! 梯子组与阳性对照各格按片并行（登记第十二节修订第 12 条：只改调度、不改判据）：线程数取 `E168_THREADS`，没设取
//! `SINGLEFS_THREAD_CAP`，都没设取 `.claude/heavy-thread-default` 的登记值（每台默认重活线程数）；P4（4 GiB）的片按内存上限限并发
//! （线程全打满会让内存 oom，所以另按内存限并发）：内存上限取 `E168_MEMORY_CAP`，没设取本进程控制组的
//! `memory.max`，再没有取 40 GiB；扣掉大片之外那一份常驻，按一片的峰值往下取整就是同时跑几片（至少一片）。每片打的行先攒着，主线程按片的次序整片打出，线程数不同输出逐字相同；
//! 每 5 分钟每片往标准错误打一行 `E168-progress`（不进产物）。

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, Once, OnceLock};
use std::time::Duration;

use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
    space_budget_of_role, units_of_a_publish_to_land, SpaceBudgetOfARole,
    INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE, INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
    WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemError, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, FloorRaiseStop, MountError, RaisedFloor};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_sequential_write, FirstFile, PoolVersion, PoolWriter, PublishError,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::crash_amplification::{GivenStatesEngine, GivenStatesItem};
use singlefs_verification::crash_given_states::{
    GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::sha256::sha256_hexadecimal;

thread_local! {
    /// 这条线程在跑一片时攒下的产物行（[`run_one_slice_capturing_its_lines`] 开、关）；`None` 是不在片里（主线程），行直接打。
    static LINES_OF_THE_SLICE_ON_THIS_THREAD: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    /// 这条线程在跑的那一片的进度计数（进度行读它）；`None` 是不在片里。
    static PROGRESS_OF_THE_SLICE_ON_THIS_THREAD: RefCell<Option<Arc<SliceProgress>>> = const { RefCell::new(None) };
}

/// 打一行产物：在片里攒进这一片的缓冲（主线程按片的次序整片打出），不在片里直接打到标准输出。
fn emit_line(line: String) {
    let line_not_captured =
        LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| match cell.borrow_mut().as_mut() {
            Some(lines_of_the_slice) => {
                lines_of_the_slice.push(line);
                None
            }
            None => Some(line),
        });
    if let Some(line_to_print) = line_not_captured {
        println!("{line_to_print}");
    }
}

/// 照 `println!` 的写法打一行产物（经 [`emit_line`]）。
macro_rules! emit_product_line {
    ($($format_arguments:tt)*) => {
        emit_line(format!($($format_arguments)*))
    };
}

mod cell_driver;
mod delete_run;
mod delete_summary;
mod geometry_arms_and_pool;
mod geometry_points_smallest_usable_disk_search_and_independent_arithmetic_anchors;
#[cfg(test)]
mod helpers_shared_by_the_tests;
mod ladder_groups_and_positive_controls;
mod local_constants;
mod pure_arithmetic;
mod readings_and_fill;
mod slice_executor;

use cell_driver::*;
use delete_run::*;
use delete_summary::*;
use geometry_arms_and_pool::*;
use geometry_points_smallest_usable_disk_search_and_independent_arithmetic_anchors::*;
#[cfg(test)]
use helpers_shared_by_the_tests::*;
use ladder_groups_and_positive_controls::*;
use local_constants::*;
use pure_arithmetic::*;
use readings_and_fill::*;
use slice_executor::*;

/// 停机停在一片里时随 unwind 交给切片执行器的载荷：执行器按片的次序打到这一片时，先打它攒下的行、再打停机行，再停。
struct StopInsideASlice {
    stop_line: String,
}

/// 停机：打一行 `E name=stop` 并退 [`STOP_EXIT_CODE`]（经 [`stop_with_the_line`]）。
fn stop_with(reason: &str, detail: &str) -> ! {
    stop_with_the_line(format!("E name=stop reason={reason} {detail}"))
}

/// 打停机行并退 [`STOP_EXIT_CODE`]。这条线程在攒行（在一片里，或单测把整段跑在攒行里）时不当场退：带着停机行 unwind 出去，
/// 由攒行的那一方接住——切片执行器按片的次序打完它前面各片与它自己攒下的行之后，再对停机行调一次这个函数（单线程跑到同一处
/// 打出的就是这些行）。
fn stop_with_the_line(stop_line: String) -> ! {
    let is_capturing_lines = LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| cell.borrow().is_some());
    if is_capturing_lines {
        std::panic::panic_any(StopInsideASlice { stop_line });
    }
    println!("{stop_line}");
    std::process::exit(STOP_EXIT_CODE)
}

// ---------------------------------------------------------------------------------------------------------------------
// 删除路径的入口（补丁表在副本里接上；主工作区这份是桩）
// ---------------------------------------------------------------------------------------------------------------------

/// 删除的一批：(写入口, 分配器, 上一版, 保留的数据单元数 M, 删后 size, 实例) → 新的一版。
type PublishUnlinkBatch = fn(
    &mut PoolWriter<'_, SparseBlockDevice>,
    &mut PoolAllocator,
    &TransactionOutput,
    u64,
    u64,
    InstanceGeneration,
) -> Result<TransactionOutput, PublishError>;

/// 补丁表在副本里给 core 加的只供测试的入口与开关，装置只经这几个函数指针用它们。
#[allow(
    dead_code,
    reason = "主工作区那份里只有桩 delete_path_hooks、不构造它；补丁表在副本里把桩换成构造它的那一份（登记第十二节修订第 2 条）"
)]
struct DeletePathHooks {
    /// 删除的一批：从尾部截到前 M 个数据单元、size 改成删后的值（`transaction::publish_unlink_batch_for_tests`）。
    publish_unlink_batch: PublishUnlinkBatch,
    /// 同一批经会话发（B 臂）：`UserChange::UnlinkBatchForTests`。
    unlink_batch_change: fn(u64, u64) -> UserChange<'static>,
    /// 这个 inode 的记录写 nlink = 0；`None` 恢复恒写 1。
    set_inode_written_with_zero_links: fn(Option<u64>),
    /// 式子那一判关掉（落点那一判照判）。
    set_formula_judgement_skipped: fn(bool),
    /// 式子那一判至今真判了几次。
    formula_judgements_made: fn() -> u64,
    /// 回收一律不放槽回空闲（阳性对照 PC-noreclaim）。
    set_reclaim_switched_off: fn(bool),
    /// 取走自上次以来每次 `record_root_written_by_this_process` 入口那一刻每块盘的空闲槽（谷值钩子，S5）。
    take_free_slots_recorded_at_each_root: fn() -> Vec<Vec<u64>>,
    /// 账：`Some((待删占用, 已承诺预留))`，按全部副本之和记的字节；`None` 照第一版的两个 0。
    set_ledger: fn(Option<(u64, u64)>),
    /// `crates/` ckpt_cost 里分配记录树那一项（K1，根层级从这一版的根节点头现读）。
    allocation_record_tree_positions: fn(&TransactionOutput, &PoolAllocator) -> u64,
}

/// 主工作区这份是桩：不带补丁表就没有删除入口，打一行停机就退（登记第十二节修订第 2 条）。
fn delete_path_hooks() -> DeletePathHooks {
    println!("E name=stop reason=delete_path_patch_absent");
    std::process::exit(DELETE_PATH_PATCH_ABSENT_EXIT_CODE)
}

// ---------------------------------------------------------------------------------------------------------------------
// 第一段、S1 与入口
// ---------------------------------------------------------------------------------------------------------------------

const SEGMENT_ONE_POINTS: [DeviceSizePoint; 3] = [
    DeviceSizePoint::SmallestUsable,
    DeviceSizePoint::Narrow,
    DeviceSizePoint::Product,
];
const SEGMENT_ONE_SLOTS_PER_REGION: [u64; 2] = [
    ROOT_RING_SLOTS_PER_REGION_MINIMUM,
    ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
];
const SEGMENT_ONE_BATCHES: [u64; 2] =
    [EXTENT_TREE_LOWER_LEAF_DATA_UNITS, SMALLEST_BATCH_DATA_UNITS];

/// 第一段的一片：一组梯子（盘大小点, S, K, 臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LadderGroupSlice {
    point: DeviceSizePoint,
    slots_per_region: u64,
    batch_data_units: u64,
    arm: Arm,
}

/// 第一段的梯子组，按改并行之前那几层循环的次序（几何点、S、K、N-A 与 N-B）；`filter` 只留这一个几何。
fn ladder_group_slices(filter: Option<&str>) -> Vec<LadderGroupSlice> {
    let mut slices = Vec::new();
    for point in SEGMENT_ONE_POINTS {
        for slots_per_region in SEGMENT_ONE_SLOTS_PER_REGION {
            if filter.is_some_and(|wanted| wanted != geometry_label_of(point, slots_per_region)) {
                continue;
            }
            for batch_data_units in SEGMENT_ONE_BATCHES {
                for arm in &ALL_ARMS[..2] {
                    slices.push(LadderGroupSlice {
                        point,
                        slots_per_region,
                        batch_data_units,
                        arm: *arm,
                    });
                }
            }
        }
    }
    slices
}

/// P4 的组算大片；开跑先后按梯子基本点的删除发布次数之和（续加的点不算），大的先跑。
fn describe_ladder_group(slice: &LadderGroupSlice) -> SliceDescription {
    SliceDescription {
        label: format!(
            "{}-K{}-{}",
            geometry_label_of(slice.point, slice.slots_per_region),
            slice.batch_data_units,
            slice.arm.label()
        ),
        is_large: slice.point == DeviceSizePoint::Product,
        start_rank: ladder_of_file_sizes(slice.batch_data_units, slice.slots_per_region)
            .into_iter()
            .map(|data_units| delete_publish_count(data_units, slice.batch_data_units))
            .sum(),
    }
}

/// W 臂账里那一份要的 W_max：每个几何上 N-A、K = 1 那一组量到的（登记 5.2 W-A「W_max 取同一几何上 N-A 量到的值」）。
fn largest_new_slots_for_the_set_aside(
    slices: &[LadderGroupSlice],
    tallies: &[GroupTally],
) -> BTreeMap<String, u64> {
    slices
        .iter()
        .zip(tallies)
        .filter(|(slice, _)| {
            slice.arm.refusal == RefusalHandling::Direct
                && slice.batch_data_units == SMALLEST_BATCH_DATA_UNITS
        })
        .map(|(slice, tally)| {
            (
                geometry_label_of(slice.point, slice.slots_per_region),
                tally.largest_new_slots,
            )
        })
        .collect()
}

/// 跑一组梯子组（按片并行），交回各组的计数（按片的次序）。
fn run_the_ladder_groups(
    slices: &[LadderGroupSlice],
    hooks: &DeletePathHooks,
    scheduling: SliceScheduling,
) -> Vec<GroupTally> {
    run_slices_of_the_apparatus(slices, &describe_ladder_group, scheduling, &|slice| {
        run_the_ladder_group(
            slice.point,
            slice.slots_per_region,
            slice.batch_data_units,
            slice.arm,
            0,
            hooks,
        )
    })
}

/// 登记第五节 5.8 第一段：S2–S7 随各格判；B 类锚点；P0；N-A、N-B 在 P1(S = 4 / 16)、P2(S = 4 / 16)、P4(S = 4 / 16) 上，
/// K = 144 与 K = 1、L-seq、F-end，梯子扫到饱和；阳性对照八格。`filter` 只跑这一个几何（如 `P2-S4`）的 P0 / 梯子组 / 阳性对照。
/// 梯子组与阳性对照各格按片并行（`scheduling`），输出次序与线程数无关。
fn run_segment_one(filter: Option<&str>, scheduling: SliceScheduling) {
    let hooks = delete_path_hooks();
    emit_product_line!(
        "E name=constants compared={}",
        local_constants_against_the_implementation()
    );
    emit_product_line!(
        "E name=anchors_b checked={}",
        anchors_of_the_independent_arithmetic_and_make_filesystem_edge()
    );
    for slots_per_region in SEGMENT_ONE_SLOTS_PER_REGION {
        let geometry = make_filesystem_smallest_geometry(slots_per_region);
        if filter.is_some_and(|wanted| wanted != geometry.label()) {
            continue;
        }
        match pool_with_a_file(geometry, 1) {
            PoolWithAFile::Written(_) => emit_product_line!(
                "E name=p0 geometry={} device_bytes={} first_file=written",
                geometry.label(),
                geometry.device_bytes
            ),
            PoolWithAFile::Refused { step, detail, .. } => {
                emit_product_line!("E name=p0 geometry={} device_bytes={} first_file=refused step={step} detail={detail:?}", geometry.label(), geometry.device_bytes);
            }
        }
    }
    let group_slices = ladder_group_slices(filter);
    let tallies = run_the_ladder_groups(&group_slices, &hooks, scheduling);
    let largest_new_slots_of_each_geometry =
        largest_new_slots_for_the_set_aside(&group_slices, &tallies);
    let positive_controls = run_the_positive_controls(
        &largest_new_slots_of_each_geometry,
        filter,
        &hooks,
        scheduling,
    );
    let sum_of = |batch: Option<u64>, field: fn(&GroupTally) -> u64| -> u64 {
        group_slices
            .iter()
            .zip(&tallies)
            .filter(|(slice, _)| {
                batch.is_none_or(|only_this_batch| only_this_batch == slice.batch_data_units)
            })
            .map(|(_, tally)| field(tally))
            .sum()
    };
    emit_product_line!(
        "E name=verdict groups={} cells_deleted={} cells_not_fillable={} q1_cells_refused={} q1_cells_apex_over_reserve_k144={} q1_cells_apex_over_reserve_k1={} q1_groups_unsaturated={} q2_cells_apex_over_ring_window={} q2_groups_w_growing={} q2_w_exceeds_landing_publishes={} {}",
        tallies.len(),
        sum_of(None, |tally| tally.cells_deleted),
        sum_of(None, |tally| tally.cells_not_fillable),
        sum_of(None, |tally| tally.cells_refused),
        sum_of(Some(EXTENT_TREE_LOWER_LEAF_DATA_UNITS), |tally| tally.cells_apex_over_reserve),
        sum_of(Some(SMALLEST_BATCH_DATA_UNITS), |tally| tally.cells_apex_over_reserve),
        sum_of(None, |tally| u64::from(!tally.is_saturated)),
        sum_of(None, |tally| tally.cells_apex_over_ring_window),
        sum_of(None, |tally| u64::from(tally.are_the_largest_new_slots_growing_with_file_size)),
        sum_of(None, |tally| tally.publishes_whose_new_slots_exceed_the_landing_demand),
        positive_controls.verdict_fields(),
    );
    stop_when_a_positive_control_does_not_hold(positive_controls);
}

/// 只重跑阳性对照（登记第十二节修订第 13 条）：常量回比与 B 类锚点照打；W 臂账里那一份要的 W_max 现跑每个阳性对照几何上 N-A、
/// K = 1 那一组梯子算（那一组的行照打，与第一段同一个函数），再跑阳性对照各格。`filter` 只留这一个几何（如 `P1-S16`）。各片按片并行。
fn run_the_positive_controls_only(filter: Option<&str>, scheduling: SliceScheduling) {
    let hooks = delete_path_hooks();
    emit_product_line!(
        "E name=constants compared={}",
        local_constants_against_the_implementation()
    );
    emit_product_line!(
        "E name=anchors_b checked={}",
        anchors_of_the_independent_arithmetic_and_make_filesystem_edge()
    );
    let group_slices: Vec<LadderGroupSlice> = POSITIVE_CONTROL_GEOMETRIES
        .into_iter()
        .filter(|(point, slots_per_region)| {
            filter.is_none_or(|wanted| wanted == geometry_label_of(*point, *slots_per_region))
        })
        .map(|(point, slots_per_region)| LadderGroupSlice {
            point,
            slots_per_region,
            batch_data_units: SMALLEST_BATCH_DATA_UNITS,
            arm: ALL_ARMS[0],
        })
        .collect();
    let tallies = run_the_ladder_groups(&group_slices, &hooks, scheduling);
    let largest_new_slots_of_each_geometry =
        largest_new_slots_for_the_set_aside(&group_slices, &tallies);
    let positive_controls = run_the_positive_controls(
        &largest_new_slots_of_each_geometry,
        filter,
        &hooks,
        scheduling,
    );
    emit_product_line!(
        "E name=verdict mode=positive-controls w_max_groups={} w_max_of_each_geometry={largest_new_slots_of_each_geometry:?} {}",
        tallies.len(),
        positive_controls.verdict_fields()
    );
    stop_when_a_positive_control_does_not_hold(positive_controls);
}

/// 停机条款 S1 的那一段不删的历史（不调删除入口，主工作区与副本两份二进制都跑）：mkfs → 可写挂载 → 写一个文件（P2 上 32 个单元、
/// P4 上 300 个单元）→ 重挂载（[`pool_with_a_file`] 的第 2 步）→ 覆盖写成 1 个单元 → 重挂载；打盘镜像的 sha256 与分配器三个计数。
fn run_the_history_without_deletes() {
    for (geometry, data_units) in [
        (
            narrow_geometry(ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM),
            32,
        ),
        (
            product_geometry(ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM),
            300,
        ),
    ] {
        let mut pool = match pool_with_a_file(geometry, data_units) {
            PoolWithAFile::Written(pool) => pool,
            PoolWithAFile::Refused { step, detail, .. } => {
                stop_with(
                    "s1_history_file_not_written",
                    &format!("geometry={} step={step} detail={detail}", geometry.label()),
                );
            }
        };
        let content = content_needing(1, 3);
        let overwrite = UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS + 120,
        });
        if let Err(refusal) = pool
            .session
            .publish_user_change(&mut pool.devices, overwrite)
        {
            stop_with(
                "s1_history_overwrite_refused",
                &format!("geometry={} refusal={refusal:?}", geometry.label()),
            );
        }
        let remounted = match mount_writable(&pool.parameters, &mut pool.devices) {
            Ok(remounted) => remounted,
            Err(error) => stop_with(
                "s1_history_remount_refused",
                &format!("geometry={} error={error:?}", geometry.label()),
            ),
        };
        let counts: Vec<String> = remounted
            .allocator
            .devices
            .iter()
            .enumerate()
            .map(|(device, map)| {
                format!(
                    "device{device}_allocated={} device{device}_free={} device{device}_deferred={}",
                    map.allocated_slots(),
                    map.free_slots(),
                    map.deferred_slots()
                )
            })
            .collect();
        let digests: Vec<String> = pool
            .devices
            .iter()
            .enumerate()
            .map(|(device, (_, sparse))| {
                format!("device{device}_sha256={}", device_image_digest(sparse))
            })
            .collect();
        emit_product_line!(
            "E name=s1_history geometry={} data_units={data_units} {} {}",
            geometry.label(),
            digests.join(" "),
            counts.join(" ")
        );
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match arguments.as_slice() {
        ["s1-history"] => {
            emit_product_line!("E name=config mode=s1-history");
            run_the_history_without_deletes();
        }
        ["segment1"] => {
            emit_product_line!("E name=config mode=segment1 geometry=all");
            run_segment_one(None, slice_scheduling_from_the_environment());
        }
        ["segment1", "--geometry", label] => {
            emit_product_line!("E name=config mode=segment1 geometry={label}");
            run_segment_one(Some(label), slice_scheduling_from_the_environment());
        }
        ["positive-controls"] => {
            emit_product_line!("E name=config mode=positive-controls geometry=all");
            run_the_positive_controls_only(None, slice_scheduling_from_the_environment());
        }
        ["positive-controls", "--geometry", label] => {
            emit_product_line!("E name=config mode=positive-controls geometry={label}");
            run_the_positive_controls_only(Some(label), slice_scheduling_from_the_environment());
        }
        _ => {
            eprintln!("用法：e168_delete_ring_window_borrow s1-history | segment1 [--geometry <几何名，如 P2-S4>] | positive-controls [--geometry <几何名，如 P1-S16>]");
            std::process::exit(2);
        }
    }
    emit_product_line!("E name=done");
}

#[cfg(test)]
mod tests;
