//! E168（删除借保留量的环窗口）入库装置：跑前登记 `research/prompts/e168-preregistration.md`（问题单
//! `research/prompts/m3-ring-window-questions.md`）。只读、只驱动与观测 `crates/` 今天的发布路径、准入读数与挂载内回收，
//! 不改 `crates/singlefs-core`、`crates/singlefs-checker`。
//!
//! 删除路径今天不在 `crates/` 里：主工作区这一份里 [`delete_path_hooks`] 是桩（登记第十二节修订第 2 条），一调就打
//! `E name=stop reason=delete_path_patch_absent` 并退非零；补丁表 `research/mutations/e168_delete_ring_window_borrow_delete_path.tsv`
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
//! `SINGLEFS_THREAD_CAP`，都没设取 `available_parallelism`；P4（4 GiB）的片另有一个同时跑的上限 `E168_LARGE_SLICE_THREADS`
//! （没设取 1，8G 在 P4-S4-K144 撞过 oom-kill）。每片打的行先攒着，主线程按片的次序整片打出，线程数不同输出逐字相同；
//! 每 5 分钟每片往标准错误打一行 `E168-progress`（不进产物）。

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, Once};
use std::time::Duration;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
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
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::sha256::sha256_hexadecimal;

// 下面这些常量抄自 `crates/singlefs-format/src/lib.rs` 与 `crates/singlefs-core/src/admission.rs`（行号在各自的注释里），
// 写成本地常量：装置与实现不共用一个数，两边一起错时 `local_constants_against_the_implementation` 逐个回比会红
// （登记 5.0「装置读 `crates/` 的这些常量时一律写成本地常量加回比断言」）。这一条挡不住 kb 与实现一起取错。
/// `singlefs-format/src/lib.rs:14`。
const SLOT_BYTES: u64 = 16384;
/// `singlefs-format/src/lib.rs:20`。
const DATA_UNIT_BYTES: u64 = 32768;
/// `singlefs-format/src/lib.rs:238`。
const ROOT_RING_REGIONS: u64 = 3;
/// `singlefs-format/src/lib.rs:247`。
const ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4;
/// `singlefs-format/src/lib.rs:252`。
const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16;
/// `singlefs-format/src/lib.rs:256`。
const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
/// `singlefs-format/src/lib.rs:156`；登记 5.4 的产品批大小 K = 144。
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// `singlefs-format/src/lib.rs:153`。
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// `singlefs-format/src/lib.rs:140`。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// `singlefs-format/src/lib.rs:146`。
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// `singlefs-format/src/lib.rs:209`。
const JOURNAL_RING_START_SLOT: u64 = 1024;
/// `singlefs-format/src/lib.rs:213`。
const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// `singlefs-format/src/lib.rs:271`。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// `singlefs-format/src/lib.rs:198`（一条 journal 记录的字节数）。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// `singlefs-format/src/lib.rs:228`（F：在飞记录数上限 = 环槽数 ÷ F）。
const JOURNAL_SAFETY_FACTOR: u64 = 3;
/// `singlefs-core/src/admission.rs:160`（N_switch）。
const INSTANCE_SWITCHES_PER_MOUNT: u64 = 3;
/// `singlefs-core/src/admission.rs:163`（R）。
const WARM_UP_EMPTY_PUBLISHES_PER_SWITCH: u64 = 3;
/// `singlefs-core/src/admission.rs:167`。
const INSTANCE_ROWS_PER_PAGE: u64 = 369;
/// `TransactionUnit::InstanceTable.span_slots()`（实例表一片 32768 字节）。
const INSTANCE_TABLE_PAGE_SLOTS: u64 = 2;

/// 第一版两块盘互为镜像（登记 1.2「副本数」：固定 2，不扫）。
const DEVICE_COUNT: u64 = 2;
/// 登记 5.4 只供测试的最小批。
const SMALLEST_BATCH_DATA_UNITS: u64 = 1;
/// 登记 5.7 P0 的 journal 环：1 MiB；P1 的环从它起往大取（登记第十二节修订第 15 条），不小于它。
const SMALL_JOURNAL_RING_BYTES: u64 = 1 << 20;
/// mkfs 收的 journal 环按 1 MiB 一档：单元区起点 = 1024 + ⌈环长 ÷ 16384⌉ 要落 64 槽边界（`allocator.rs` 的
/// `following_the_journal_ring`），同一档里的环长起点相同、盘的布局相同（修订第 15 条取档的上端）。
const JOURNAL_RING_STEP_BYTES: u64 = CLUSTER_SEGMENT_SLOTS * SLOT_BYTES;
/// 修订第 15 条「先定环」那一步把环放大的次数上限（放大一次就直接跳到装得下那次写的记录数的一档，再放大只在记录数随环涨时发生）。
const JOURNAL_RING_ENLARGEMENTS_AT_MOST: usize = 4;
/// 登记 5.7 P2 的 journal 环：6 MiB（`crates/singlefs-harness/src/history.rs` 窄盘先例）。
const NARROW_JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 登记 5.7 P2 的单元区槽数。
const NARROW_UNIT_AREA_SLOTS: u64 = 384;
/// 登记 5.7 P4：4 GiB。
const PRODUCT_DEVICE_BYTES: u64 = 4 << 30;
/// 盘的物理块大小：harness 的内存稀疏盘都按 512 建。
const PHYSICAL_BLOCK_BYTES: u32 = 512;
/// 第一个文件的写时刻（只要固定，取哪个数不影响任何量）。
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 停机条款不过的退出码。
const STOP_EXIT_CODE: i32 = 4;
/// 一片 panic 时装置的退出码：与 Rust 主线程 panic 的退出码相同。
const PANIC_EXIT_CODE: i32 = 101;
/// 主工作区那份不带补丁时的退出码（登记修订第 2 条）。
const DELETE_PATH_PATCH_ABSENT_EXIT_CODE: i32 = 3;
/// 梯子末端没饱和时按 ×2 续加的次数上限（登记 8.4：直到饱和或撞上沿；上沿之外再给一个有限的界，让循环有上界）。
const LADDER_EXTENSIONS_AT_MOST: usize = 6;
/// 8.3 差额敏感性只算到 δ = 3。
const DELTA_AT_MOST: u64 = 3;
/// B 臂一次准入的发布数上限（D16 已定项 1「B = 4 + 2 k_tol」，k_tol = 2）。
const PUBLISHES_IN_ONE_ADMISSION_AT_MOST: u64 = 8;

/// 本地常量逐个与 `crates/` 回比；对不上就停（第十一节「装置与 `crates/` 对不上」）。交回比过的个数。
fn local_constants_against_the_implementation() -> usize {
    let pairs: [(&str, u64, u64); 19] = [
        ("SLOT_BYTES", SLOT_BYTES, singlefs_format::SLOT_BYTES),
        (
            "DATA_UNIT_BYTES",
            DATA_UNIT_BYTES,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "ROOT_RING_REGIONS",
            ROOT_RING_REGIONS,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_MINIMUM",
            ROOT_RING_SLOTS_PER_REGION_MINIMUM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_MAXIMUM",
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM",
            ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        ),
        (
            "EXTENT_TREE_LOWER_LEAF_DATA_UNITS",
            EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        ),
        (
            "EXTENT_TREE_INTERNAL_FANOUT",
            EXTENT_TREE_INTERNAL_FANOUT,
            singlefs_format::EXTENT_TREE_INTERNAL_FANOUT,
        ),
        (
            "ALLOCATION_RECORD_TREE_LEAF_SLOTS",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        (
            "ALLOCATION_RECORD_TREE_INTERNAL_FANOUT",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        (
            "JOURNAL_RING_START_SLOT",
            JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        (
            "JOURNAL_RING_DEFAULT_BYTES",
            JOURNAL_RING_DEFAULT_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "CLUSTER_SEGMENT_SLOTS",
            CLUSTER_SEGMENT_SLOTS,
            singlefs_format::CLUSTER_SEGMENT_SLOTS,
        ),
        (
            "JOURNAL_RECORD_BYTES",
            JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "JOURNAL_SAFETY_FACTOR",
            JOURNAL_SAFETY_FACTOR,
            singlefs_format::JOURNAL_SAFETY_FACTOR,
        ),
        (
            "INSTANCE_SWITCHES_ALLOWED_PER_MOUNT",
            INSTANCE_SWITCHES_PER_MOUNT,
            INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
        ),
        (
            "WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH",
            WARM_UP_EMPTY_PUBLISHES_PER_SWITCH,
            WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
        ),
        (
            "INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE",
            INSTANCE_ROWS_PER_PAGE,
            INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
        ),
        (
            "InstanceTable.span_slots",
            INSTANCE_TABLE_PAGE_SLOTS,
            TransactionUnit::InstanceTable.span_slots(),
        ),
    ];
    for (name, local, implementation) in pairs {
        if local != implementation {
            stop_with(
                "local_constant_differs_from_the_implementation",
                &format!("constant={name} local={local} implementation={implementation}"),
            );
        }
    }
    pairs.len()
}

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
// 切片执行器（登记第十二节修订第 10 条：只改调度、不改判据；输出按片的次序，与线程数无关）
// ---------------------------------------------------------------------------------------------------------------------

/// 进度行的间隔（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」：5 分钟）。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(300);
/// 线程数：先读这个，没设读 [`SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE`]，都没设取 `available_parallelism`。
const THREADS_ENVIRONMENT_VARIABLE: &str = "E168_THREADS";
/// `research/scripts/capped.sh` 设的通用线程上限。
const SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";
/// 大片（P4）同时跑的上限。
const LARGE_SLICE_THREADS_ENVIRONMENT_VARIABLE: &str = "E168_LARGE_SLICE_THREADS";
/// 大片上限没设时取 1：P4 上一格单线程就要 8G 以上（2026-10-01 第一段 r3 在 P4-S4-K144 撞 8G oom-kill，r4 用 24G 跑完梯子）。
const LARGE_SLICE_THREADS_WHEN_UNSET: usize = 1;

/// 一片的进度：开没开跑、跑完没有、判完的格数与删除发布数（只给进度行读，不进产物）。
#[derive(Default)]
struct SliceProgress {
    is_started: AtomicBool,
    is_ended: AtomicBool,
    cells_judged: AtomicU64,
    delete_publishes: AtomicU64,
}

/// 这条线程在跑的那一片判完一格（不在片里不记）。
fn count_a_judged_cell_on_this_slice() {
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| {
        if let Some(progress) = cell.borrow().as_ref() {
            progress.cells_judged.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// 这条线程在跑的那一片发完一次删除发布（不在片里不记）。
fn count_a_delete_publish_on_this_slice() {
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| {
        if let Some(progress) = cell.borrow().as_ref() {
            progress.delete_publishes.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// 一片在调度上的样子：进度行里的区间名、算不算大片、开跑的先后（大的先跑；只排开跑次序，不排输出次序）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SliceDescription {
    label: String,
    is_large: bool,
    start_rank: u64,
}

/// 一片的结局：跑完（攒下的行与值）、停在停机条款（攒下的行与停机行）、panic（攒下的行与载荷）。
enum SliceEnd<Value> {
    Finished {
        lines: Vec<String>,
        value: Value,
    },
    Stopped {
        lines: Vec<String>,
        stop_line: String,
    },
    Panicked {
        lines: Vec<String>,
        payload: Box<dyn Any + Send>,
    },
}

/// 在这条线程上跑一片：开攒行、挂上这一片的进度计数，跑完、停机或 panic 都取回攒下的行。
fn run_one_slice_capturing_its_lines<Slice, Value>(
    slice: &Slice,
    progress: &Arc<SliceProgress>,
    run_one_slice: &(dyn Fn(&Slice) -> Value + Sync),
) -> SliceEnd<Value> {
    LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| *cell.borrow_mut() = Some(Vec::new()));
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD
        .with(|cell| *cell.borrow_mut() = Some(Arc::clone(progress)));
    progress.is_started.store(true, Ordering::SeqCst);
    let result = catch_unwind(AssertUnwindSafe(|| run_one_slice(slice)));
    let lines = LINES_OF_THE_SLICE_ON_THIS_THREAD
        .with(|cell| cell.borrow_mut().take())
        .expect("开跑时放进去的缓冲，跑的时候只有 emit_line 往里推、不取走");
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| *cell.borrow_mut() = None);
    progress.is_ended.store(true, Ordering::SeqCst);
    match result {
        Ok(value) => SliceEnd::Finished { lines, value },
        Err(payload) => match payload.downcast::<StopInsideASlice>() {
            Ok(stop) => SliceEnd::Stopped {
                lines,
                stop_line: stop.stop_line,
            },
            Err(payload) => SliceEnd::Panicked { lines, payload },
        },
    }
}

/// 调度的两个上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SliceScheduling {
    thread_count: usize,
    large_slice_thread_count: usize,
}

/// 一个工作线程池：它按次序取的片号与它的线程数。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkerPool {
    positions_in_start_order: Vec<usize>,
    thread_count: usize,
}

/// 把片分进线程池。线程数为 1：一个池、一条线程、按片的次序跑全部片（与改并行之前同一个次序）。否则大片与其余片各一个池：
/// 大片池的线程数不超过大片上限，也不吃光全部线程（还有别的片时至少给它们留一条）；池里按 `start_rank` 从大到小开跑，同值按片号。
fn worker_pools(descriptions: &[SliceDescription], scheduling: SliceScheduling) -> Vec<WorkerPool> {
    if scheduling.thread_count <= 1 {
        return vec![WorkerPool {
            positions_in_start_order: (0..descriptions.len()).collect(),
            thread_count: 1,
        }];
    }
    let positions_in_start_order_of = |wants_large: bool| -> Vec<usize> {
        let mut positions: Vec<usize> = (0..descriptions.len())
            .filter(|position| descriptions[*position].is_large == wants_large)
            .collect();
        positions.sort_by_key(|position| {
            (
                std::cmp::Reverse(descriptions[*position].start_rank),
                *position,
            )
        });
        positions
    };
    let large = positions_in_start_order_of(true);
    let small = positions_in_start_order_of(false);
    let large_cap = scheduling.large_slice_thread_count.max(1);
    let large_threads = match (large.is_empty(), small.is_empty()) {
        (true, _) => 0,
        (false, true) => large_cap.min(large.len()),
        (false, false) => large_cap.min(scheduling.thread_count - 1).min(large.len()),
    };
    let small_threads = (scheduling.thread_count - large_threads).min(small.len());
    [(large, large_threads), (small, small_threads)]
        .into_iter()
        .filter(|(positions, _)| !positions.is_empty())
        .map(|(positions_in_start_order, thread_count)| WorkerPool {
            positions_in_start_order,
            thread_count,
        })
        .collect()
}

/// 停机载荷的 unwind 不是 bug，不让默认的 panic 钩子往标准错误打「thread panicked」；别的 panic 照默认钩子打。装一次。
fn silence_the_panic_hook_for_stops_inside_slices() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            if !information.payload().is::<StopInsideASlice>() {
                default_hook(information);
            }
        }));
    });
}

/// 进度行（标准错误，不进产物）：每片报区间、状态、已判的格数与删除发布数。
fn progress_line(
    position: usize,
    slice_count: usize,
    label: &str,
    progress: &SliceProgress,
) -> String {
    let state = match (
        progress.is_started.load(Ordering::SeqCst),
        progress.is_ended.load(Ordering::SeqCst),
    ) {
        (false, _) => "waiting",
        (true, false) => "running",
        (true, true) => "ended",
    };
    format!(
        "E168-progress slice={}/{slice_count} range={label} state={state} cells_judged={} delete_publishes={}",
        position + 1,
        progress.cells_judged.load(Ordering::Relaxed),
        progress.delete_publishes.load(Ordering::Relaxed)
    )
}

/// 按片并行跑 `slices`，每片的行攒着、按片的次序整片经 [`emit_line`] 打出（线程数不同输出逐字相同）；交回每片的值（按片的次序）。
/// 按片的次序第一个停机的片：打完它前面各片与它自己攒下的行，不再取新片，对它的停机行调 [`stop_with_the_line`]（主线程不攒行时
/// 打停机行、退出进程；攒行时带着停机行 unwind，等在跑的片跑完再出这个函数）。一片 panic：打完它之前的行再把 panic 原样抛出。
/// 每过 `progress_interval` 给每片调一次 `report_progress`。
fn run_slices_in_order<Slice: Sync, Value: Send>(
    slices: &[Slice],
    describe: &dyn Fn(&Slice) -> SliceDescription,
    scheduling: SliceScheduling,
    run_one_slice: &(dyn Fn(&Slice) -> Value + Sync),
    report_progress: &(dyn Fn(&str) + Sync),
    progress_interval: Duration,
) -> Vec<Value> {
    silence_the_panic_hook_for_stops_inside_slices();
    let descriptions: Vec<SliceDescription> = slices.iter().map(describe).collect();
    let pools = worker_pools(&descriptions, scheduling);
    let progresses: Vec<Arc<SliceProgress>> = slices
        .iter()
        .map(|_| Arc::new(SliceProgress::default()))
        .collect();
    let ends: Mutex<Vec<Option<SliceEnd<Value>>>> =
        Mutex::new(slices.iter().map(|_| None).collect());
    let slice_ended = Condvar::new();
    let is_abandoned = AtomicBool::new(false);
    let is_merge_over = Mutex::new(false);
    let merge_over = Condvar::new();
    let (ends, slice_ended, is_abandoned, progresses, descriptions) = (
        &ends,
        &slice_ended,
        &is_abandoned,
        &progresses,
        &descriptions,
    );
    let (is_merge_over, merge_over) = (&is_merge_over, &merge_over);
    std::thread::scope(|scope| {
        for pool in &pools {
            let next_in_the_pool = Arc::new(AtomicUsize::new(0));
            for _ in 0..pool.thread_count {
                let next_in_the_pool = Arc::clone(&next_in_the_pool);
                scope.spawn(move || {
                    while !is_abandoned.load(Ordering::SeqCst) {
                        let taken = next_in_the_pool.fetch_add(1, Ordering::SeqCst);
                        let Some(&position) = pool.positions_in_start_order.get(taken) else {
                            break;
                        };
                        let end = run_one_slice_capturing_its_lines(
                            &slices[position],
                            &progresses[position],
                            run_one_slice,
                        );
                        ends.lock()
                            .expect("工作线程的 panic 在 catch_unwind 里接住，锁不会中毒")
                            [position] = Some(end);
                        slice_ended.notify_all();
                    }
                });
            }
        }
        scope.spawn(move || {
            let mut is_over = is_merge_over
                .lock()
                .expect("进度线程与主线程都不在持锁时 panic");
            loop {
                let (guard, _) = merge_over
                    .wait_timeout_while(is_over, progress_interval, |is_over| !*is_over)
                    .expect("进度线程与主线程都不在持锁时 panic");
                is_over = guard;
                if *is_over {
                    break;
                }
                for (position, progress) in progresses.iter().enumerate() {
                    report_progress(&progress_line(
                        position,
                        progresses.len(),
                        &descriptions[position].label,
                        progress,
                    ));
                }
            }
        });
        let merged = catch_unwind(AssertUnwindSafe(|| {
            let mut values = Vec::with_capacity(slices.len());
            for position in 0..slices.len() {
                let end = {
                    let mut ends_now = ends
                        .lock()
                        .expect("工作线程的 panic 在 catch_unwind 里接住，锁不会中毒");
                    loop {
                        if let Some(end) = ends_now[position].take() {
                            break end;
                        }
                        ends_now = slice_ended
                            .wait(ends_now)
                            .expect("工作线程的 panic 在 catch_unwind 里接住，锁不会中毒");
                    }
                };
                let (lines, ending) = match end {
                    SliceEnd::Finished { lines, value } => (lines, Ok(value)),
                    SliceEnd::Stopped { lines, stop_line } => (lines, Err(Ok(stop_line))),
                    SliceEnd::Panicked { lines, payload } => (lines, Err(Err(payload))),
                };
                for line in lines {
                    emit_line(line);
                }
                match ending {
                    Ok(value) => values.push(value),
                    Err(Ok(stop_line)) => {
                        is_abandoned.store(true, Ordering::SeqCst);
                        stop_with_the_line(stop_line);
                    }
                    Err(Err(payload)) => end_the_run_after_a_slice_panicked(payload),
                }
            }
            values
        }));
        is_abandoned.store(true, Ordering::SeqCst);
        *is_merge_over.lock().expect("进度线程不在持锁时 panic") = true;
        merge_over.notify_all();
        merged.unwrap_or_else(|payload| resume_unwind(payload))
    })
}

/// 一片 panic 之后：主线程在攒行（单测）时把 panic 原样抛出；不攒行（装置）时照 Rust panic 的退出码当场退，不等别的片跑完
/// （panic 的消息已由默认钩子在那一片的线程里打出；改并行之前单线程 panic 也是当场退）。
fn end_the_run_after_a_slice_panicked(payload: Box<dyn Any + Send>) -> ! {
    let is_capturing_lines = LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| cell.borrow().is_some());
    if is_capturing_lines {
        resume_unwind(payload);
    }
    std::process::exit(PANIC_EXIT_CODE)
}

/// 线程数：`E168_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取 `available_parallelism`；设了而不是正整数就 panic。
fn thread_count_from(
    e168_threads: Option<&str>,
    shared_thread_cap: Option<&str>,
    available_parallelism: usize,
) -> usize {
    for (variable, value) in [
        (THREADS_ENVIRONMENT_VARIABLE, e168_threads),
        (SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE, shared_thread_cap),
    ] {
        if let Some(text) = value {
            return text
                .parse::<usize>()
                .ok()
                .filter(|thread_count| *thread_count > 0)
                .unwrap_or_else(|| panic!("{variable} 要是正整数，收到的是「{text}」"));
        }
    }
    available_parallelism
}

/// 大片上限：`E168_LARGE_SLICE_THREADS`，没设取 [`LARGE_SLICE_THREADS_WHEN_UNSET`]；设了而不是正整数就 panic。
fn large_slice_thread_count_from(value: Option<&str>) -> usize {
    value.map_or(LARGE_SLICE_THREADS_WHEN_UNSET, |text| {
        text.parse::<usize>()
            .ok()
            .filter(|thread_count| *thread_count > 0)
            .unwrap_or_else(|| {
                panic!("{LARGE_SLICE_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，收到的是「{text}」")
            })
    })
}

/// 读环境变量定调度，并往标准错误报一行（线程数不进产物：产物与线程数无关）。
fn slice_scheduling_from_the_environment() -> SliceScheduling {
    let read = |variable: &str| std::env::var(variable).ok();
    let scheduling = SliceScheduling {
        thread_count: thread_count_from(
            read(THREADS_ENVIRONMENT_VARIABLE).as_deref(),
            read(SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
            std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        ),
        large_slice_thread_count: large_slice_thread_count_from(
            read(LARGE_SLICE_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        ),
    };
    eprintln!(
        "E168-progress threads={} large_slice_threads={}",
        scheduling.thread_count, scheduling.large_slice_thread_count
    );
    scheduling
}

/// 装置里的切片执行：产物行经 [`emit_line`] 打到标准输出，进度行写标准错误，停机就退 [`STOP_EXIT_CODE`]。交回每片的值（按片的次序）。
fn run_slices_of_the_apparatus<Slice: Sync, Value: Send>(
    slices: &[Slice],
    describe: &dyn Fn(&Slice) -> SliceDescription,
    scheduling: SliceScheduling,
    run_one_slice: &(dyn Fn(&Slice) -> Value + Sync),
) -> Vec<Value> {
    run_slices_in_order(
        slices,
        describe,
        scheduling,
        run_one_slice,
        &|line: &str| eprintln!("{line}"),
        PROGRESS_INTERVAL,
    )
}

// ---------------------------------------------------------------------------------------------------------------------
// 纯算术（第七节 B 类：只用格式常量与条款式子，不调 `crates/` 的函数；与 `crates/` 的读数只在停机条款 S3、S4 里逐格比）
// ---------------------------------------------------------------------------------------------------------------------

/// 环长（根环里的根数）= 区数 × 每区槽数（第七节 A3）。
fn ring_length_in_roots(slots_per_region: u64) -> u64 {
    ROOT_RING_REGIONS * slots_per_region
}

/// 分配记录树层级 L 上一个位置罩的槽数 S_L = 812 × 169^L。
fn allocation_record_tree_span_in_slots_at_level(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 分配记录树的根层级（`allocation_record_tree.rs:92` 的规则）：最小的 R 使 Σ_盘 ⌈盘绝对槽数 ÷ S_(R−1)⌉ ≤ 169。
fn allocation_record_tree_root_level(absolute_slots_of_each_device: &[u64]) -> u32 {
    let mut root_level = 1;
    loop {
        let span_below_the_root = allocation_record_tree_span_in_slots_at_level(root_level - 1);
        let positions_under_the_root: u64 = absolute_slots_of_each_device
            .iter()
            .map(|absolute_slots| absolute_slots.div_ceil(span_below_the_root))
            .sum();
        if positions_under_the_root <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return root_level;
        }
        root_level += 1;
    }
}

/// 一块盘的单元区 [起点, 盘末)，按绝对槽号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitAreaOnOneDevice {
    start_slot: u64,
    end_slot_exclusive: u64,
}

/// D28 已定项 4 的 K1：1 + Σ_盘 Σ_{L = 0 … 根层级 − 1}（⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1）。交回 (根层级, K1)。
fn allocation_record_tree_positions_by_the_clause(
    unit_areas: &[UnitAreaOnOneDevice],
) -> (u32, u64) {
    let absolute_slots: Vec<u64> = unit_areas
        .iter()
        .map(|area| area.end_slot_exclusive)
        .collect();
    let root_level = allocation_record_tree_root_level(&absolute_slots);
    let positions_below_the_root: u64 = unit_areas
        .iter()
        .map(|area| {
            (0..root_level)
                .map(|level| {
                    let span = allocation_record_tree_span_in_slots_at_level(level);
                    (area.end_slot_exclusive - 1) / span - area.start_slot / span + 1
                })
                .sum::<u64>()
        })
        .sum();
    (root_level, 1 + positions_below_the_root)
}

/// 单元区起点 = journal 环起点 + ⌈环长 ÷ 16384⌉（`journal.rs:185`）。
fn unit_area_start_slot(journal_ring_bytes: u64) -> u64 {
    JOURNAL_RING_START_SLOT + journal_ring_bytes.div_ceil(SLOT_BYTES)
}

/// 一次发布的 journal 记录数上限 = ⌊环长 ÷ 4096 ÷ 3⌋（`system_configuration.rs` 的 `journal_in_flight_record_limit`，拒在
/// `transaction.rs` 的 `refuse_a_publish_of_more_journal_records_than_the_limit`）；本地按格式常量算、不调 core，单测与 core 回比。
fn journal_record_limit_by_the_format(journal_ring_bytes: u64) -> u64 {
    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}

/// 登记第十二节修订第 15 条：装得下一次发布 `records_of_the_publish` 条 journal 记录的最小合法环。合法 = mkfs 收：环长按
/// [`JOURNAL_RING_STEP_BYTES`]（1 MiB）一档，取在飞上限 ≥ 记录数的最小一档、档的上端；不小于 [`SMALL_JOURNAL_RING_BYTES`]。
fn smallest_legal_journal_ring_holding(records_of_the_publish: u64) -> u64 {
    let bytes_holding_the_records =
        records_of_the_publish * JOURNAL_RECORD_BYTES * JOURNAL_SAFETY_FACTOR;
    let steps = bytes_holding_the_records
        .div_ceil(JOURNAL_RING_STEP_BYTES)
        .max(SMALL_JOURNAL_RING_BYTES / JOURNAL_RING_STEP_BYTES);
    steps * JOURNAL_RING_STEP_BYTES
}

/// mkfs 允许的最小盘（第七节 B3）：单元区要装得下 mkfs 写的实例表（2 槽）与树表（1 槽），且环 ≤ 盘 ÷ 4。
fn make_filesystem_smallest_device_bytes(journal_ring_bytes: u64) -> u64 {
    let unit_area_holds_the_make_filesystem_units =
        (unit_area_start_slot(journal_ring_bytes) + 3) * SLOT_BYTES;
    unit_area_holds_the_make_filesystem_units.max(4 * journal_ring_bytes)
}

/// 单元区恰好 `unit_area_slots` 槽的盘的字节数。
fn device_bytes_with_unit_area(journal_ring_bytes: u64, unit_area_slots: u64) -> u64 {
    (unit_area_start_slot(journal_ring_bytes) + unit_area_slots) * SLOT_BYTES
}

/// 第七节 A2：发布路径上实例表链一片装得下时，每盘保留量 = (N_switch + 1) × (一片 2 槽 + R × c) + c（c = ckpt_cost，
/// ckpt 池 c × 16384 × 副本数按全部副本之和记、摊到每块盘 c 槽）= 8 + 13c。
fn reserve_slots_by_the_clauses(checkpoint_cost: u64) -> u64 {
    let one_switch =
        INSTANCE_TABLE_PAGE_SLOTS + WARM_UP_EMPTY_PUBLISHES_PER_SWITCH * checkpoint_cost;
    (INSTANCE_SWITCHES_PER_MOUNT + 1) * one_switch + checkpoint_cost
}

/// 5.3 压满：把式子可用压进 (−16384, 0] 要再标掉的槽数。
fn slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(available_bytes: i128) -> u64 {
    if available_bytes <= 0 {
        return 0;
    }
    let slot_bytes = i128::from(SLOT_BYTES);
    u64::try_from((available_bytes + slot_bytes - 1) / slot_bytes).expect("可用的槽数装得进 u64")
}

/// 压满之后的区间断言 (−16384, 0]（登记 1.2「满池」）。
fn available_is_within_one_slot_at_or_below_zero(available_bytes: i128) -> bool {
    available_bytes <= 0 && available_bytes > -i128::from(SLOT_BYTES)
}

/// M1：顶点 − 顶点处的保留量 ≤ 0 判「罩得住（顶点）」。
fn reserve_covers_the_apex(apex_slots: i128, reserve_at_the_apex_slots: i128) -> bool {
    apex_slots - reserve_at_the_apex_slots <= 0
}

/// M7：(环长 + 1) × W_max。
fn ring_window_bound_slots(
    slots_per_region: u64,
    largest_new_slots_of_a_delete_publish: u64,
) -> u64 {
    (ring_length_in_roots(slots_per_region) + 1) * largest_new_slots_of_a_delete_publish
}

/// 8.3：使 apex + (3S + 1) × δ > reserve_apex 的最小 δ（只算 δ ≤ 3）；罩不住的格（δ = 0 就越过）交回 `Some(0)`。
fn smallest_unbuilt_delta_that_flips_the_cover(
    apex_slots: i128,
    reserve_at_the_apex_slots: i128,
    slots_per_region: u64,
) -> Option<u64> {
    let roots_plus_one = i128::from(ring_length_in_roots(slots_per_region) + 1);
    (0..=DELTA_AT_MOST)
        .find(|delta| apex_slots + roots_plus_one * i128::from(*delta) > reserve_at_the_apex_slots)
}

/// M3 的判定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ApexSaturation {
    /// 批数 ≥ 3S + 2 的梯子点里，最后两点的 apex ≤ 它们之前那一点的 apex。
    Saturated,
    /// 有至少三个那样的点，最后两点里有一点比之前那一点高。
    GrowingWithoutSaturation,
    /// 那样的点不到三个，判不了。
    FewerThanThreePointsPastTheRing,
}

/// M3：`apex_by_delete_publishes` 按梯子次序给 (这条删除的发布次数, apex)。
fn apex_saturation(
    apex_by_delete_publishes: &[(u64, i128)],
    slots_per_region: u64,
) -> ApexSaturation {
    let past_the_ring: Vec<i128> = apex_by_delete_publishes
        .iter()
        .filter(|(publishes, _)| *publishes >= ring_length_in_roots(slots_per_region) + 2)
        .map(|(_, apex)| *apex)
        .collect();
    let [.., before_the_last_two, second_last, last] = past_the_ring.as_slice() else {
        return ApexSaturation::FewerThanThreePointsPastTheRing;
    };
    if *second_last <= *before_the_last_two && *last <= *before_the_last_two {
        ApexSaturation::Saturated
    } else {
        ApexSaturation::GrowingWithoutSaturation
    }
}

/// M5：W_max(N) 在梯子最后三点逐点严格增 ⇒ 判「随文件大小涨」。
fn largest_new_slots_grow_with_file_size(largest_new_slots_by_ladder_point: &[u64]) -> bool {
    matches!(largest_new_slots_by_ladder_point, [.., first, second, third] if first < second && second < third)
}

/// 删除的发布次数 = 1（意图那次）+ ⌈N ÷ K⌉。
fn delete_publish_count(data_units: u64, batch_data_units: u64) -> u64 {
    1 + data_units.div_ceil(batch_data_units)
}

/// 8.4 文件大小梯子（数据单元数 N），从小到大、去重。K = 144：N = 144 × b；K = 1：N 取同一组 b 再加 8 × 3S。
fn ladder_of_file_sizes(batch_data_units: u64, slots_per_region: u64) -> Vec<u64> {
    let ring = ring_length_in_roots(slots_per_region);
    let mut batches_per_file = vec![
        1,
        2,
        4,
        8,
        ring - 1,
        ring,
        ring + 1,
        ring + 2,
        ring + 5,
        2 * ring + 2,
        4 * ring,
    ];
    if batch_data_units == SMALLEST_BATCH_DATA_UNITS {
        batches_per_file.push(8 * ring);
    }
    let mut data_units: Vec<u64> = batches_per_file
        .into_iter()
        .map(|batches| batches * batch_data_units)
        .collect();
    data_units.sort_unstable();
    data_units.dedup();
    data_units
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
// 几何、臂与池
// ---------------------------------------------------------------------------------------------------------------------

/// 登记 5.7 的盘大小取样点（第一段用到的四个）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeviceSizePoint {
    /// P0：mkfs 字面最小盘（journal 环 1 MiB，单元区 3 槽）。
    MakeFilesystemSmallest,
    /// P1(S, N)：对每个 N 找的最小可用盘（journal 环 1 MiB）。
    SmallestUsable,
    /// P2：单元区 384 槽、journal 环 6 MiB。
    Narrow,
    /// P4：4 GiB、产品默认环。
    Product,
}

impl DeviceSizePoint {
    fn label(self) -> &'static str {
        match self {
            DeviceSizePoint::MakeFilesystemSmallest => "P0",
            DeviceSizePoint::SmallestUsable => "P1",
            DeviceSizePoint::Narrow => "P2",
            DeviceSizePoint::Product => "P4",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Geometry {
    point: DeviceSizePoint,
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
}

impl Geometry {
    fn label(&self) -> String {
        format!("{}-S{}", self.point.label(), self.slots_per_region)
    }

    fn unit_area_slots(&self) -> u64 {
        self.device_bytes / SLOT_BYTES - unit_area_start_slot(self.journal_ring_bytes)
    }

    fn unit_areas(&self) -> Vec<UnitAreaOnOneDevice> {
        let area = UnitAreaOnOneDevice {
            start_slot: unit_area_start_slot(self.journal_ring_bytes),
            end_slot_exclusive: self.device_bytes / SLOT_BYTES,
        };
        vec![area; usize::try_from(DEVICE_COUNT).expect("盘数")]
    }
}

/// 登记 5.2 的账那一条轴。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerArm {
    /// N：待删占用 = 0、已承诺预留 = 0。
    NothingSetAside,
    /// W：已承诺预留 = (3S + 1) × W_max × 16384 × 2。
    RingWindowSetAside,
}

/// 登记 5.2 的被拒处置那一条轴。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RefusalHandling {
    /// A：那一批直接交回拒绝。
    Direct,
    /// B：经会话，落点被拒先推抬 F 的空发布再判，推满仍拒才算拒。
    ThroughTheSession,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Arm {
    ledger: LedgerArm,
    refusal: RefusalHandling,
}

impl Arm {
    fn label(self) -> &'static str {
        match (self.ledger, self.refusal) {
            (LedgerArm::NothingSetAside, RefusalHandling::Direct) => "N-A",
            (LedgerArm::NothingSetAside, RefusalHandling::ThroughTheSession) => "N-B",
            (LedgerArm::RingWindowSetAside, RefusalHandling::Direct) => "W-A",
            (LedgerArm::RingWindowSetAside, RefusalHandling::ThroughTheSession) => "W-B",
        }
    }
}

const ALL_ARMS: [Arm; 4] = [
    Arm {
        ledger: LedgerArm::NothingSetAside,
        refusal: RefusalHandling::Direct,
    },
    Arm {
        ledger: LedgerArm::NothingSetAside,
        refusal: RefusalHandling::ThroughTheSession,
    },
    Arm {
        ledger: LedgerArm::RingWindowSetAside,
        refusal: RefusalHandling::Direct,
    },
    Arm {
        ledger: LedgerArm::RingWindowSetAside,
        refusal: RefusalHandling::ThroughTheSession,
    },
];

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn make_filesystem_parameters(geometry: &Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"_SFS-E168-RINGW\0",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                geometry.slots_per_region,
            )
            .expect("每区槽数只取 4、8、16，都在挂载判的区间里"),
        },
    }
}

fn blank_devices(device_bytes: u64) -> Devices {
    (0..DEVICE_COUNT)
        .map(|device_number| {
            (
                DeviceIdentity(u32::try_from(device_number).expect("盘号")),
                SparseBlockDevice::new(
                    device_bytes,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                ),
            )
        })
        .collect()
}

/// `data_units` 个数据单元的内容（最后一个单元只写一半净荷）。
fn content_needing(data_units: u64, seed: u64) -> Vec<u8> {
    let payload_bytes = u64::try_from(data_unit_payload_capacity()).expect("净荷字节数");
    (0..(data_units - 1) * payload_bytes + payload_bytes / 2)
        .map(|byte_position| {
            u8::try_from((byte_position * 31 + seed * 7 + 1) % 251).expect("小于 251")
        })
        .collect()
}

/// 挂着的池：A 臂直接拿会话的分配器与现行那一版发，B 臂经会话的 `publish_user_change` 发。
struct MountedPool {
    geometry: Geometry,
    parameters: MakeFilesystemParameters,
    devices: Devices,
    session: MountedSession,
}

impl MountedPool {
    fn current_file_version(&self) -> &TransactionOutput {
        match &self.session.current {
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
            PoolVersion::WithoutFile(_) => stop_with(
                "current_version_has_no_file",
                &format!("geometry={}", self.geometry.label()),
            ),
        }
    }

    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: self.geometry.device_bytes,
        }
    }

    fn free_slots_of_each_device(&self) -> Vec<u64> {
        self.session
            .allocator
            .devices
            .iter()
            .map(|map| map.free_slots())
            .collect()
    }
}

/// 5.1 第 1、2 步的结局：写成了，或在哪一步被拒。
enum PoolWithAFile {
    Written(Box<MountedPool>),
    Refused {
        step: &'static str,
        detail: String,
        /// 被拒的是那次发布的 journal 记录数超过上限（`PublishError::JournalRecordsOfThePublishExceedTheLimit`）时带着条数与上限，
        /// 别的拒绝是 None（登记第十二节修订第 15 条：P1 的环按它放大，找不到盘时它进原因字段）。
        journal_records_over_the_limit: Option<JournalRecordsOverTheLimit>,
    },
}

/// 一次发布的 journal 记录条数与这个环的上限（前者大于后者才会被拒）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct JournalRecordsOverTheLimit {
    records_of_the_publish: u64,
    record_limit: u64,
}

fn journal_records_over_the_limit_of(error: &PublishError) -> Option<JournalRecordsOverTheLimit> {
    if let PublishError::JournalRecordsOfThePublishExceedTheLimit {
        records_of_the_publish,
        record_limit,
    } = error
    {
        Some(JournalRecordsOverTheLimit {
            records_of_the_publish: *records_of_the_publish,
            record_limit: *record_limit,
        })
    } else {
        None
    }
}

/// 5.1 第 1、2 步：mkfs → 可写挂载（新实例、写行、暖机）→ 第一个文件（一个单元）→（N > 1 时）顺序写成 N 个数据单元 →
/// 关进程、重开可写挂载一次（分配器装上从盘上读来的根环表）。式子那一判开着。
fn pool_with_a_file(geometry: Geometry, data_units: u64) -> PoolWithAFile {
    let parameters = make_filesystem_parameters(&geometry);
    let mut devices = blank_devices(geometry.device_bytes);
    if let Err(error) = make_filesystem(&parameters, &mut devices) {
        return PoolWithAFile::Refused {
            step: "make_filesystem",
            detail: format!("{error:?}"),
            journal_records_over_the_limit: None,
        };
    }
    let mut mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            return PoolWithAFile::Refused {
                step: "first_mount",
                detail: format!("{error:?}"),
                journal_records_over_the_limit: None,
            }
        }
    };
    let instance = mounted.output.instance;
    let (root, record_bytes) = match &mounted.current {
        PoolVersion::WithoutFile(version) => (version.root, version.record_bytes.clone()),
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => stop_with(
            "new_pool_already_has_a_file",
            &format!("geometry={}", geometry.label()),
        ),
    };
    let first_content = content_needing(1, 1);
    let written = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let first = publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root,
            FirstFile {
                content: &first_content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &record_bytes,
        );
        match first {
            Err(error) => Err(("first_file", error)),
            Ok(first) if data_units == 1 => Ok(first),
            Ok(first) => {
                let content = content_needing(data_units, 2);
                publish_sequential_write(
                    &mut writer,
                    &mut mounted.allocator,
                    &first,
                    FirstFile {
                        content: &content,
                        write_time_seconds: WRITE_TIME_SECONDS + 60,
                    },
                    instance,
                )
                .map_err(|error| ("sequential_write", error))
            }
        }
    };
    if let Err((step, error)) = written {
        return PoolWithAFile::Refused {
            step,
            detail: format!("{error:?}"),
            journal_records_over_the_limit: journal_records_over_the_limit_of(&error),
        };
    }
    drop(mounted);
    match mount_writable(&parameters, &mut devices) {
        Ok(remounted) => {
            let (_, session) = MountedSession::of_the_mount(remounted);
            PoolWithAFile::Written(Box::new(MountedPool {
                geometry,
                parameters,
                devices,
                session,
            }))
        }
        Err(error) => PoolWithAFile::Refused {
            step: "remount",
            detail: format!("{error:?}"),
            journal_records_over_the_limit: None,
        },
    }
}

/// 一块盘镜像的 sha256：按写过的扇区号次序，每段「扇区号 + 内容的 sha256」拼起来再取一次 sha256。
fn device_image_digest(device: &SparseBlockDevice) -> String {
    let mut digests = String::new();
    for (sector, bytes) in device.image.written_sectors() {
        digests.push_str(&format!("{sector}:{};", sha256_hexadecimal(bytes)));
    }
    sha256_hexadecimal(digests.as_bytes())
}

/// 池级 checker 判红的不变量（停机条款 S7）。
fn checker_violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------------------------------
// 读数与压满
// ---------------------------------------------------------------------------------------------------------------------

/// 每块盘的式子可用（字节），账取此刻线程局部里那一对。
fn available_bytes_of_each_device(pool: &MountedPool) -> Vec<i128> {
    admission_reading_before_a_publish(&pool.session.allocator, Some(pool.current_file_version()))
        .available_on_each_device()
        .into_iter()
        .map(|(_, available)| available.0)
        .collect()
}

/// 这一刻的读数：保留量（`crates/` 读数，逐盘槽）与 ckpt_cost。保留量 = 账置 0 时「空闲槽 − 式子可用 ÷ 16384」（登记 1.2），
/// 读完把账换回 `ledger_of_the_arm`；与本地按条款算的 8 + 13c 逐盘比（停机条款 S3），被抛弃根独占量不为 0 也在这里对不上。
fn reserve_reading(
    pool: &MountedPool,
    hooks: &DeletePathHooks,
    ledger_of_the_arm: Option<(u64, u64)>,
) -> (Vec<i128>, u64) {
    (hooks.set_ledger)(Some((0, 0)));
    let available = available_bytes_of_each_device(pool);
    (hooks.set_ledger)(ledger_of_the_arm);
    let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(
        Some(pool.current_file_version()),
        &pool.session.allocator,
    )
    .0;
    let reserve_by_the_clauses = i128::from(reserve_slots_by_the_clauses(checkpoint_cost));
    let slot_bytes = i128::from(SLOT_BYTES);
    let reserve: Vec<i128> = pool
        .free_slots_of_each_device()
        .into_iter()
        .zip(available)
        .map(|(free_slots, available_bytes)| {
            if available_bytes % slot_bytes != 0 {
                stop_with(
                    "reserve_reading_not_whole_slots",
                    &format!(
                        "geometry={} available_bytes={available_bytes}",
                        pool.geometry.label()
                    ),
                );
            }
            i128::from(free_slots) - available_bytes / slot_bytes
        })
        .collect();
    for (device_position, reserve_on_the_device) in reserve.iter().enumerate() {
        if *reserve_on_the_device != reserve_by_the_clauses {
            stop_with(
                "s3_reserve_differs_from_the_clauses",
                &format!(
                    "geometry={} device={device_position} implementation={reserve_on_the_device} clauses={reserve_by_the_clauses} checkpoint_cost={checkpoint_cost}",
                    pool.geometry.label()
                ),
            );
        }
    }
    (reserve, checkpoint_cost)
}

/// 停机条款 S4 与第七节 B4：本地 K1 与 `crates/` ckpt_cost 里分配记录树那一项比，单元区槽数与分配器读数比。交回 (根层级, K1)。
fn allocation_record_tree_positions_checked(
    pool: &MountedPool,
    hooks: &DeletePathHooks,
) -> (u32, u64) {
    for map in &pool.session.allocator.devices {
        if map.unit_area_slots() != pool.geometry.unit_area_slots() {
            stop_with(
                "b4_unit_area_slots_differ",
                &format!(
                    "geometry={} allocator={} local={}",
                    pool.geometry.label(),
                    map.unit_area_slots(),
                    pool.geometry.unit_area_slots()
                ),
            );
        }
    }
    let implementation = (hooks.allocation_record_tree_positions)(
        pool.current_file_version(),
        &pool.session.allocator,
    );
    let (root_level, by_the_clause) =
        allocation_record_tree_positions_by_the_clause(&pool.geometry.unit_areas());
    if implementation != by_the_clause {
        stop_with(
            "s4_allocation_record_tree_positions_differ",
            &format!("geometry={} implementation={implementation} local={by_the_clause} root_level={root_level}", pool.geometry.label()),
        );
    }
    (root_level, by_the_clause)
}

/// W_t：这次发布写出的分配记录里落在这块盘、分配代 = 这次 txg、未释放的跨度之和（登记 1.2）。
fn new_slots_on_device(output: &TransactionOutput, device: DeviceIdentity) -> u64 {
    output
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == device
                && !record.is_released
                && record.generation == output.root.checkpoint_txg
        })
        .map(|record| u64::from(record.span_slots))
        .sum()
}

/// 一次发布按空间归哪一格拆的槽数（`space_budget_of_role`）与落点那一判的需求（M6 的 W_land）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PublishShape {
    ordinary_slots: u64,
    fixed_point_slots: u64,
    switch_reserve_slots: u64,
    landing_demand_slots: u64,
}

fn shape_of_the_publish(output: &TransactionOutput, checkpoint_cost_before: u64) -> PublishShape {
    let slots_of = |budget: SpaceBudgetOfARole| -> u64 {
        output
            .rewritten
            .iter()
            .filter(|role| space_budget_of_role(**role) == budget)
            .map(|role| role.span_slots())
            .sum()
    };
    let ordinary_roles: Vec<TransactionUnit> = output
        .rewritten
        .iter()
        .copied()
        .filter(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
        .collect();
    let landing = units_of_a_publish_to_land(
        &ordinary_roles,
        singlefs_core::admission::MetadataBlocks(checkpoint_cost_before),
    );
    PublishShape {
        ordinary_slots: slots_of(SpaceBudgetOfARole::OrdinaryAllocation),
        fixed_point_slots: slots_of(SpaceBudgetOfARole::CheckpointReservePool),
        switch_reserve_slots: slots_of(SpaceBudgetOfARole::InstanceSwitchReserve),
        landing_demand_slots: landing.commit_generated_slots
            + landing.data_units * (DATA_UNIT_BYTES / SLOT_BYTES),
    }
}

/// 5.3 压满的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
enum FillOutcome {
    /// 每块盘的式子可用都落在 (−16384, 0]；`free_slots` 是压满之后每块盘的空闲槽，`filled_slots_of_each_device` 是每块盘标掉的槽号
    /// （停机条款 S7 判 checker 之前要撤掉它们，见 [`check_the_finished_delete`]）。
    WithinOneSlotAtOrBelowZero {
        free_slots: Vec<u64>,
        filled_slots_of_each_device: Vec<Vec<u64>>,
    },
    /// 压满之前式子可用就已经 ≤ −16384（这个几何上写进文件、重挂载之后保留量已经超出空闲），造不出登记的「满池」。
    AlreadyBelowTheIntervalBeforeFilling { available_bytes: Vec<i128> },
}

/// 5.3 F-end：从单元区末尾往前把没挡的空闲槽标成已分配（只动分配器，不写盘），直到式子可用（按这条臂此刻的账）落进 (−16384, 0]。
/// 标掉的槽不改式子里别的项，一轮就该落进区间；不落就停（压满的算术与式子对不上）。
fn fill_from_the_unit_area_end(pool: &mut MountedPool) -> FillOutcome {
    let available_before = available_bytes_of_each_device(pool);
    if available_before
        .iter()
        .any(|available| *available <= -i128::from(SLOT_BYTES))
    {
        return FillOutcome::AlreadyBelowTheIntervalBeforeFilling {
            available_bytes: available_before,
        };
    }
    let filled_slots_of_each_device: Vec<Vec<u64>> = pool
        .session
        .allocator
        .devices
        .iter_mut()
        .zip(available_before)
        .map(|(map, available)| {
            mark_free_slots_from_the_unit_area_end(
                map,
                slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(available),
            )
        })
        .collect();
    let available_after = available_bytes_of_each_device(pool);
    if !available_after
        .iter()
        .all(|available| available_is_within_one_slot_at_or_below_zero(*available))
    {
        stop_with(
            "fill_did_not_land_in_the_interval",
            &format!(
                "geometry={} available_after={available_after:?}",
                pool.geometry.label()
            ),
        );
    }
    FillOutcome::WithinOneSlotAtOrBelowZero {
        free_slots: pool.free_slots_of_each_device(),
        filled_slots_of_each_device,
    }
}

/// 从单元区末尾往前把至多 `slots_to_mark` 个空闲槽一槽一槽标成已分配，交回标掉的槽号（从高到低）。
fn mark_free_slots_from_the_unit_area_end(map: &mut DeviceFreeMap, slots_to_mark: u64) -> Vec<u64> {
    let mut filled_slots = Vec::new();
    let start_slot = map.unit_area_start().slot().0;
    let mut slot = start_slot + map.unit_area_slots();
    while u64::try_from(filled_slots.len()).expect("标掉的槽数") < slots_to_mark
        && slot > start_slot
    {
        slot -= 1;
        if map.is_free(SlotNumber(slot)) {
            map.mark_allocated(SlotNumber(slot), 1);
            filled_slots.push(slot);
        }
    }
    filled_slots
}

/// 撤掉压满：把 [`mark_free_slots_from_the_unit_area_end`] 标掉的槽逐槽放回空闲。分配器没有「已分配直接回空闲」的入口，
/// 照它自己的两步走——`mark_released` 挪进待释放、`mark_reclaimed` 从待释放回空闲——待释放一加一减，已分配、空闲、空闲段数、
/// 每段占用与没挡的槽数都回到压满之前。压满的槽从没进过分配记录，也就不在 `PoolAllocator` 的记录与 defer 队列里，只动空闲图。
fn unmark_the_filled_slots(
    device_maps: &mut [DeviceFreeMap],
    filled_slots_of_each_device: &[Vec<u64>],
) {
    assert_eq!(
        device_maps.len(),
        filled_slots_of_each_device.len(),
        "压满逐盘标过，撤也逐盘撤"
    );
    for (map, filled_slots) in device_maps.iter_mut().zip(filled_slots_of_each_device) {
        for slot in filled_slots {
            map.mark_released(SlotNumber(*slot), 1);
            map.mark_reclaimed(SlotNumber(*slot), 1);
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 删除（5.1 第 4、5 步）
// ---------------------------------------------------------------------------------------------------------------------

/// 删除阶段回收开不开（阳性对照 PC-noreclaim 关）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReclaimDuringTheDelete {
    AsToday,
    SwitchedOff,
}

/// 一次删除发布（意图那次记 0）被拒的种类（登记 1.2「被落点那一判拒」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeleteRefusal {
    /// `PublishError::SpaceAdmissionRefused`：删除各批式子那一判关着（调用计数核过），只能来自落点那一判。
    Landing,
    /// `PublishError::PlacementRefused`。
    Placement,
    /// B 臂：推满抬 F 之后仍拒。
    NoSpaceAfterRaisingTheFloor,
}

impl DeleteRefusal {
    fn label(self) -> &'static str {
        match self {
            DeleteRefusal::Landing => "landing",
            DeleteRefusal::Placement => "placement",
            DeleteRefusal::NoSpaceAfterRaisingTheFloor => "no_space_after_raising_the_floor",
        }
    }
}

/// 一次删除发布的读数（逐盘的量按盘的次序）。
#[derive(Clone, Debug)]
struct DeletePublishReading {
    /// 第几次删除发布，意图那次记 0。
    position: u64,
    kept_data_units: u64,
    reserve_slots: Vec<i128>,
    checkpoint_cost: u64,
    free_before_slots: Vec<u64>,
    /// W_t；被拒的那一次是 0。
    new_slots: Vec<u64>,
    /// 这一次里推的抬 F 空发布（B 臂）的发布数与它们新占的槽。
    floor_raise_publishes: u64,
    floor_raise_new_slots: Vec<u64>,
    /// 被拒的那一次里会话不再推抬 F 的原因（[`FloorRaisesOfTheStep::stop_label`]；A 臂与发成的那几次是 `none`）。
    floor_raise_stop: String,
    /// 谷值钩子在会话的分配器上记根时读到的每块盘最小空闲槽（抬 F 预演在拷贝上记的不算，[`hook_readings_on_the_allocator_of_the_session`]）；
    /// 一次都没读到（被拒且没有一串抬 F 落盘）时取发布之前的空闲槽。
    valley_free_slots: Vec<u64>,
    /// 谷值与 free_before − W_t 逐盘比过（S5）；推了抬 F 的那几次不比，谷值直接取钩子读数（第十二节修订）。
    is_valley_checked_against_the_new_slots: bool,
    shape: Option<PublishShape>,
    refusal: Option<DeleteRefusal>,
}

impl DeletePublishReading {
    fn borrow_at_the_valley(&self) -> Vec<i128> {
        self.reserve_slots
            .iter()
            .zip(&self.valley_free_slots)
            .map(|(reserve, valley)| reserve - i128::from(*valley))
            .collect()
    }

    fn borrow_between_publishes(&self) -> Vec<i128> {
        self.reserve_slots
            .iter()
            .zip(&self.free_before_slots)
            .map(|(reserve, free)| reserve - i128::from(*free))
            .collect()
    }

    fn line(&self, cell_label: &str) -> String {
        let per_device = |name: &str, values: &[String]| -> String {
            values
                .iter()
                .enumerate()
                .map(|(device, value)| format!("{name}_device{device}={value}"))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let strings_of =
            |values: &[i128]| values.iter().map(ToString::to_string).collect::<Vec<_>>();
        let strings_of_unsigned =
            |values: &[u64]| values.iter().map(ToString::to_string).collect::<Vec<_>>();
        let shape = self.shape.map_or_else(
            || "ordinary_slots=none fixed_point_slots=none switch_reserve_slots=none landing_demand_slots=none w_exceeds_landing=none".to_owned(),
            |shape| {
                format!(
                    "ordinary_slots={} fixed_point_slots={} switch_reserve_slots={} landing_demand_slots={} w_exceeds_landing={}",
                    shape.ordinary_slots,
                    shape.fixed_point_slots,
                    shape.switch_reserve_slots,
                    shape.landing_demand_slots,
                    u8::from(self.new_slots.iter().any(|new_slots| *new_slots > shape.landing_demand_slots))
                )
            },
        );
        format!(
            "E name=publish cell={cell_label} t={} kept_data_units={} checkpoint_cost={} {} {} {} {} {} {} {} floor_raise_publishes={} floor_raise_stop={} valley_checked={} {} refused={}",
            self.position,
            self.kept_data_units,
            self.checkpoint_cost,
            per_device("reserve", &strings_of(&self.reserve_slots)),
            per_device("free_before", &strings_of_unsigned(&self.free_before_slots)),
            per_device("new_slots", &strings_of_unsigned(&self.new_slots)),
            per_device("floor_raise_new_slots", &strings_of_unsigned(&self.floor_raise_new_slots)),
            per_device("valley_free", &strings_of_unsigned(&self.valley_free_slots)),
            per_device("borrow_valley", &strings_of(&self.borrow_at_the_valley())),
            per_device("borrow_between", &strings_of(&self.borrow_between_publishes())),
            self.floor_raise_publishes,
            self.floor_raise_stop,
            u8::from(self.is_valley_checked_against_the_new_slots),
            shape,
            self.refusal.map_or("none", DeleteRefusal::label),
        )
    }
}

/// 删除的每一步：(保留的数据单元数 M, 删后 size)。意图那次留全部、size 不变；之后从尾部每批截 K 个，直到 0。
fn plan_of_the_delete(current: &TransactionOutput, batch_data_units: u64) -> Vec<(u64, u64)> {
    let payload_bytes = u64::try_from(data_unit_payload_capacity()).expect("净荷字节数");
    let mut kept = u64::try_from(current.data_pointers.len()).expect("单元数");
    let mut steps = vec![(
        kept,
        current
            .inode_record
            .expect("删之前那一版有第一个文件那条记录")
            .size,
    )];
    while kept > 0 {
        kept = kept.saturating_sub(batch_data_units);
        steps.push((kept, kept * payload_bytes));
    }
    steps
}

fn new_slots_of_each_device(output: &TransactionOutput) -> Vec<u64> {
    (0..DEVICE_COUNT)
        .map(|device_number| {
            new_slots_on_device(
                output,
                DeviceIdentity(u32::try_from(device_number).expect("盘号")),
            )
        })
        .collect()
}

fn summed_new_slots_of_the_floor_raises(floor_raise_outputs: &[&TransactionOutput]) -> Vec<u64> {
    let mut summed = vec![0; usize::try_from(DEVICE_COUNT).expect("盘数")];
    for output in floor_raise_outputs {
        for (total, new_slots) in summed.iter_mut().zip(new_slots_of_each_device(output)) {
            *total += new_slots;
        }
    }
    summed
}

/// 一步删除发布的结局：成了的新一版（与推的抬 F 空发布），或被拒的种类（与推的抬 F 空发布）。
struct StepOutcome {
    published: Option<TransactionOutput>,
    floor_raise_outputs: Vec<TransactionOutput>,
    floor_raises: FloorRaisesOfTheStep,
    refusal: Option<DeleteRefusal>,
}

/// 一步删除发布里会话推抬 F 的样子（B 臂；A 臂与没推的是 [`FloorRaisesOfTheStep::none`]）。谷值钩子在每次记根时读空闲槽，而抬 F
/// 那一串先在分配器的一份拷贝上整串预演（`mount::rehearse_the_publishes_raising_the_floor`，预演也走记根）、预演过了才真发：落了盘的
/// 一串 n 次空发布，钩子先读到拷贝上的 n 次、再读到真发的 n 次；在预演里第 k 次被拒的那一串，钩子读到拷贝上的 k − 1 次，一次都没真发。
/// 拷贝上的读数不是会话分配器的空闲槽，不进谷值（登记 1.2 的 free_t 是分配器这块盘此刻的空闲槽；第十二节修订第 16 条）。
/// 2026-10-02 第一段 r7 冒烟里 P1-S16-K1-Lseq-Fend-N-B-N88-noreclaim 的 t = 7 就是这样：落点拒（9 槽 < 11），抬 F 那一串（两次各 6 槽）
/// 在预演里第 2 次被拒，钩子读到拷贝上第 1 次之后的 3 槽，谷值被记成 3、borrow 记成 83，而会话分配器的空闲一直是 9。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FloorRaisesOfTheStep {
    /// 落了盘的每一串的空发布数，按先后。
    publishes_of_each_landed_chain: Vec<usize>,
    /// 最后推的那一串在预演里第 k 次被拒时它在拷贝上记过的根数 k − 1；没有这样一串是 0。
    rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: usize,
    /// 被拒那一次会话不再推的原因；发成的、A 臂的是 `none`。
    stop_label: String,
}

impl FloorRaisesOfTheStep {
    fn none() -> Self {
        Self {
            publishes_of_each_landed_chain: Vec::new(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
            stop_label: "none".to_owned(),
        }
    }

    /// 发成的那一次：推过的几串都落了盘，不再推。
    fn of_a_published_step(floor_raises: &[RaisedFloor]) -> Self {
        Self {
            publishes_of_each_landed_chain: floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .collect(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
            stop_label: "none".to_owned(),
        }
    }

    /// 推满仍拒的那一次（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`）：落了盘的几串与不再推的原因。抬 F 那一串真发途中失败
    /// （`MountError::RaiseFloorSequencePublishFailed`）时，前面已落盘的几次不在 `floor_raises` 里，抬 F 新占的槽与 B5 的等式都数不全，
    /// 交回 Err（调用方停机）。
    fn of_a_refused_step(
        floor_raises: &[RaisedFloor],
        stop: &FloorRaiseStop,
    ) -> Result<Self, String> {
        let (stop_label, rehearsal_root_recordings) = match stop {
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => {
                ("publishes_per_admission_would_be_exceeded".to_owned(), 0)
            }
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {
                ("floor_already_at_the_ceiling".to_owned(), 0)
            }
            FloorRaiseStop::FloorRaiseRefusedForSpace { refusal } => {
                let MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    ..
                } = refusal.as_ref()
                else {
                    return Err(format!("{refusal:?}"));
                };
                (
                    format!("floor_raise_refused_for_space_in_the_rehearsal_at_publish_{refused_publish_in_the_sequence}_of_{publishes_in_the_sequence}"),
                    refused_publish_in_the_sequence - 1,
                )
            }
        };
        Ok(Self {
            publishes_of_each_landed_chain: floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .collect(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal:
                rehearsal_root_recordings,
            stop_label,
        })
    }
}

/// 这一步里钩子在会话的分配器上读到的那几条（按先后）：每一串落了盘的抬 F 先丢掉它在拷贝上预演的 n 条、留下真发的 n 条，发成的这一次留 1 条，
/// 最后在预演里被拒的那一串丢掉它在拷贝上的 k − 1 条（[`FloorRaisesOfTheStep`]）。钩子读到的条数与这一拆法对不上交回 None（调用方停机：
/// 有一处记根不在这套账里，谷值读的就不知道是哪个分配器）。
fn hook_readings_on_the_allocator_of_the_session(
    hook_readings: Vec<Vec<u64>>,
    floor_raises: &FloorRaisesOfTheStep,
    is_published: bool,
) -> Option<Vec<Vec<u64>>> {
    let mut remaining = hook_readings.into_iter();
    let mut on_the_allocator = Vec::new();
    for publishes_of_the_chain in &floor_raises.publishes_of_each_landed_chain {
        for _rehearsed_on_a_copy in 0..*publishes_of_the_chain {
            remaining.next()?;
        }
        for _published_on_the_allocator in 0..*publishes_of_the_chain {
            on_the_allocator.push(remaining.next()?);
        }
    }
    if is_published {
        on_the_allocator.push(remaining.next()?);
    }
    for _rehearsed_on_a_copy_then_refused in
        0..floor_raises.rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal
    {
        remaining.next()?;
    }
    if remaining.next().is_some() {
        return None;
    }
    Some(on_the_allocator)
}

/// A 臂：直接在会话的分配器与现行那一版上发这一批，被拒原样交回。
fn publish_the_step_directly(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    kept_data_units: u64,
    size_after_the_batch: u64,
) -> StepOutcome {
    let previous = pool.current_file_version().clone();
    let result = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        (hooks.publish_unlink_batch)(
            &mut writer,
            &mut pool.session.allocator,
            &previous,
            kept_data_units,
            size_after_the_batch,
            pool.session.instance,
        )
    };
    match result {
        Ok(output) => {
            pool.session.current = PoolVersion::WithFile(output.clone());
            StepOutcome {
                published: Some(output),
                floor_raise_outputs: Vec::new(),
                floor_raises: FloorRaisesOfTheStep::none(),
                refusal: None,
            }
        }
        Err(error) => {
            let refusal = if matches!(error, PublishError::SpaceAdmissionRefused(_)) {
                DeleteRefusal::Landing
            } else if matches!(error, PublishError::PlacementRefused { .. }) {
                DeleteRefusal::Placement
            } else {
                stop_with(
                    "unexpected_publish_error",
                    &format!("geometry={} error={error:?}", pool.geometry.label()),
                )
            };
            StepOutcome {
                published: None,
                floor_raise_outputs: Vec::new(),
                floor_raises: FloorRaisesOfTheStep::none(),
                refusal: Some(refusal),
            }
        }
    }
}

/// B 臂：经会话发（`MountedSession::publish_user_change`），落点被拒由会话推抬 F 再判。
fn publish_the_step_through_the_session(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    kept_data_units: u64,
    size_after_the_batch: u64,
) -> StepOutcome {
    let change = (hooks.unlink_batch_change)(kept_data_units, size_after_the_batch);
    match pool.session.publish_user_change(&mut pool.devices, change) {
        Ok(published) => StepOutcome {
            published: Some(pool.current_file_version().clone()),
            floor_raises: FloorRaisesOfTheStep::of_a_published_step(&published.floor_raises),
            floor_raise_outputs: published
                .floor_raises
                .into_iter()
                .flat_map(|raised| raised.publishes)
                .collect(),
            refusal: None,
        },
        Err(refusal) => {
            let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
                stop_with(
                    "unexpected_user_change_refusal",
                    &format!("geometry={} refusal={refusal:?}", pool.geometry.label()),
                )
            };
            let floor_raises =
                FloorRaisesOfTheStep::of_a_refused_step(&no_space.floor_raises, &no_space.stop)
                    .unwrap_or_else(|refusal_of_the_raise| {
                        stop_with(
                            "floor_raise_publishes_landed_outside_the_refusal",
                            &format!(
                                "geometry={} refusal_of_the_raise={refusal_of_the_raise}",
                                pool.geometry.label()
                            ),
                        )
                    });
            StepOutcome {
                published: None,
                floor_raise_outputs: no_space
                    .floor_raises
                    .into_iter()
                    .flat_map(|raised| raised.publishes)
                    .collect(),
                floor_raises,
                refusal: Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor),
            }
        }
    }
}

/// 删除期间谷值钩子（S5）的取数口。钩子在 `record_root_written_by_this_process` 每次记根时都读（建池时的挂载暖机、写文件、重挂载与此前
/// 各格的发布都记根；压满只标槽、不记根），读数攒在线程里等装置取走；只有 [`ValleyHookDuringTheDelete::opened_at_the_start_of_the_delete`] 造得出这个取数口，而它先把
/// 删除开始之前攒下的读数取走丢掉，删除各步之间再读到的才是「删除发布之外」的读数（登记第十二节修订第 8 条）。
struct ValleyHookDuringTheDelete {
    take_free_slots_recorded_at_each_root: fn() -> Vec<Vec<u64>>,
}

impl ValleyHookDuringTheDelete {
    fn opened_at_the_start_of_the_delete(
        take_free_slots_recorded_at_each_root: fn() -> Vec<Vec<u64>>,
    ) -> Self {
        drop(take_free_slots_recorded_at_each_root());
        Self {
            take_free_slots_recorded_at_each_root,
        }
    }

    /// 自开口或上一次取走以来每次记根读到的每块盘空闲槽。
    fn take_readings_since_the_last_take(&self) -> Vec<Vec<u64>> {
        (self.take_free_slots_recorded_at_each_root)()
    }
}

/// 删除类发布一律不判式子那一判、落点那一判照判（登记 1.2「被落点那一判拒」、5.1 第 4 步「删除各批（含意图那次）不判式子那一判」）：
/// 意图那次、各批，与停机条款 S7 判 checker 之前那一次「续做一遍发现无事可做」的删除发布（登记第十二节修订第 9 条）都经它发。
/// 只有 [`FormulaJudgementSkippedForDeletePublishes::opened`] 造得出它：开口时关掉式子那一判、记下那一刻的判数；
/// [`FormulaJudgementSkippedForDeletePublishes::closed`] 把式子那一判开回来，交回其间真判了几次（该是 0）。
/// 2026-10-02 第一段 r5 停在 P1-S4-K1-Lseq-Fend-N-A-N1 的 S7 那一次发布，就是那一次没经它、式子那一判开着把它拒了（第十二节修订第 14 条）。
struct FormulaJudgementSkippedForDeletePublishes {
    set_formula_judgement_skipped: fn(bool),
    formula_judgements_made: fn() -> u64,
    formula_judgements_when_opened: u64,
}

impl FormulaJudgementSkippedForDeletePublishes {
    fn opened(hooks: &DeletePathHooks) -> Self {
        (hooks.set_formula_judgement_skipped)(true);
        Self {
            set_formula_judgement_skipped: hooks.set_formula_judgement_skipped,
            formula_judgements_made: hooks.formula_judgements_made,
            formula_judgements_when_opened: (hooks.formula_judgements_made)(),
        }
    }

    fn closed(self) -> u64 {
        (self.set_formula_judgement_skipped)(false);
        (self.formula_judgements_made)() - self.formula_judgements_when_opened
    }
}

/// 一条删除的结局：逐次读数（被拒那一次在最后），删完与否。
struct DeleteRun {
    readings: Vec<DeletePublishReading>,
    is_completed: bool,
}

/// 5.1 第 4、5 步：意图那次与各批，删除各批不判式子那一判、落点那一判照判；每次发布之前读 reserve_t、free_t，之后读 W_t 与谷值。
/// 被拒就停在那里（A 臂直接交回，B 臂经会话推满仍拒）。删除期间 inode 1 记录写 nlink = 0；停机条款 S5、S6 与「式子那一判调用计数为 0」在这里判。
fn run_the_delete(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    arm: Arm,
    batch_data_units: u64,
    ledger_of_the_arm: Option<(u64, u64)>,
    reclaim: ReclaimDuringTheDelete,
) -> DeleteRun {
    let steps = plan_of_the_delete(pool.current_file_version(), batch_data_units);
    (hooks.set_inode_written_with_zero_links)(Some(1));
    let formula_judgement_skipped_for_the_delete =
        FormulaJudgementSkippedForDeletePublishes::opened(hooks);
    (hooks.set_reclaim_switched_off)(reclaim == ReclaimDuringTheDelete::SwitchedOff);
    let valley_hook = ValleyHookDuringTheDelete::opened_at_the_start_of_the_delete(
        hooks.take_free_slots_recorded_at_each_root,
    );
    let mut readings = Vec::new();
    let mut is_completed = true;
    for (position, (kept_data_units, size_after_the_batch)) in (0u64..).zip(steps) {
        let (reserve_slots, checkpoint_cost) = reserve_reading(pool, hooks, ledger_of_the_arm);
        let free_before_slots = pool.free_slots_of_each_device();
        let stale_hook_readings = valley_hook.take_readings_since_the_last_take();
        if !stale_hook_readings.is_empty() {
            stop_with(
                "valley_hook_read_outside_a_delete_publish",
                &format!(
                    "geometry={} t={position} readings={}",
                    pool.geometry.label(),
                    stale_hook_readings.len()
                ),
            );
        }
        let outcome = match arm.refusal {
            RefusalHandling::Direct => {
                publish_the_step_directly(pool, hooks, kept_data_units, size_after_the_batch)
            }
            RefusalHandling::ThroughTheSession => publish_the_step_through_the_session(
                pool,
                hooks,
                kept_data_units,
                size_after_the_batch,
            ),
        };
        count_a_delete_publish_on_this_slice();
        let hook_readings = valley_hook.take_readings_since_the_last_take();
        let before_the_step = ReadingBeforeTheStep {
            position,
            kept_data_units,
            reserve_slots,
            checkpoint_cost,
            free_before_slots,
        };
        let reading = reading_of_the_step(pool, outcome, hook_readings, before_the_step);
        let is_refused = reading.refusal.is_some();
        readings.push(reading);
        if is_refused {
            is_completed = false;
            break;
        }
    }
    (hooks.set_inode_written_with_zero_links)(None);
    let formula_judgements_during_the_delete = formula_judgement_skipped_for_the_delete.closed();
    (hooks.set_reclaim_switched_off)(false);
    if formula_judgements_during_the_delete != 0 {
        stop_with(
            "formula_judgement_made_during_the_delete",
            &format!(
                "geometry={} judgements={formula_judgements_during_the_delete}",
                pool.geometry.label()
            ),
        );
    }
    DeleteRun {
        readings,
        is_completed,
    }
}

/// 一步删除发布之前读到的量。
struct ReadingBeforeTheStep {
    position: u64,
    kept_data_units: u64,
    reserve_slots: Vec<i128>,
    checkpoint_cost: u64,
    free_before_slots: Vec<u64>,
}

/// 一步的读数，停机条款 S5（谷值 = free_before − W_t）、S6（两盘 W 相等）与 V4（成了的发布钩子一次都没读到）在这里判。
fn reading_of_the_step(
    pool: &MountedPool,
    outcome: StepOutcome,
    hook_readings: Vec<Vec<u64>>,
    before: ReadingBeforeTheStep,
) -> DeletePublishReading {
    let floor_raise_references: Vec<&TransactionOutput> =
        outcome.floor_raise_outputs.iter().collect();
    let floor_raise_new_slots = summed_new_slots_of_the_floor_raises(&floor_raise_references);
    let floor_raise_publishes =
        u64::try_from(outcome.floor_raise_outputs.len()).expect("抬 F 的发布数");
    let new_slots = outcome.published.as_ref().map_or_else(
        || vec![0; floor_raise_new_slots.len()],
        new_slots_of_each_device,
    );
    if new_slots.windows(2).any(|pair| pair[0] != pair[1]) {
        stop_with(
            "s6_new_slots_differ_between_devices",
            &format!(
                "geometry={} t={} new_slots={new_slots:?}",
                pool.geometry.label(),
                before.position
            ),
        );
    }
    let hook_readings_count = hook_readings.len();
    let Some(hook_readings) = hook_readings_on_the_allocator_of_the_session(
        hook_readings,
        &outcome.floor_raises,
        outcome.published.is_some(),
    ) else {
        stop_with(
            "valley_hook_readings_do_not_match_the_root_recordings_of_the_step",
            &format!(
                "geometry={} t={} hook_readings={hook_readings_count} floor_raises={:?} published={}",
                pool.geometry.label(),
                before.position,
                outcome.floor_raises,
                outcome.published.is_some()
            ),
        )
    };
    if outcome.published.is_some() && hook_readings.is_empty() {
        stop_with(
            "v4_valley_hook_read_nothing",
            &format!("geometry={} t={}", pool.geometry.label(), before.position),
        );
    }
    let valley_free_slots: Vec<u64> = (0..before.free_before_slots.len())
        .map(|device_position| {
            hook_readings
                .iter()
                .map(|reading| reading[device_position])
                .min()
                .unwrap_or(before.free_before_slots[device_position])
        })
        .collect();
    let is_valley_checked_against_the_new_slots =
        outcome.published.is_some() && floor_raise_publishes == 0;
    if is_valley_checked_against_the_new_slots {
        let expected: Vec<u64> = before
            .free_before_slots
            .iter()
            .zip(&new_slots)
            .map(|(free, new)| free - new)
            .collect();
        if hook_readings.len() != 1 || hook_readings[0] != expected {
            stop_with(
                "s5_valley_differs_from_free_before_minus_new_slots",
                &format!(
                    "geometry={} t={} hook={hook_readings:?} expected={expected:?}",
                    pool.geometry.label(),
                    before.position
                ),
            );
        }
    }
    DeletePublishReading {
        position: before.position,
        kept_data_units: before.kept_data_units,
        reserve_slots: before.reserve_slots,
        checkpoint_cost: before.checkpoint_cost,
        free_before_slots: before.free_before_slots,
        new_slots,
        floor_raise_publishes,
        floor_raise_new_slots,
        floor_raise_stop: outcome.floor_raises.stop_label,
        valley_free_slots,
        is_valley_checked_against_the_new_slots,
        shape: outcome
            .published
            .as_ref()
            .map(|output| shape_of_the_publish(output, before.checkpoint_cost)),
        refusal: outcome.refusal,
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 一格的汇总（第六节 M1–M6、8.1 轨迹、8.3 差额敏感性）
// ---------------------------------------------------------------------------------------------------------------------

/// 一条删除的汇总，全取自逐次读数（两块盘里取大的那一块；S6 已核两盘 W 相等）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeleteSummary {
    publishes: u64,
    is_completed: bool,
    /// 被拒那一次：(第几次发布, 种类, 发布之前的 borrow)。
    refusal: Option<(u64, DeleteRefusal, i128)>,
    apex: i128,
    apex_position: u64,
    reserve_at_the_apex: i128,
    reserve_minimum: i128,
    reserve_maximum: i128,
    reserve_at_the_start: i128,
    reserve_at_the_last_publish: i128,
    apex_between: i128,
    apex_between_position: u64,
    publishes_with_a_positive_valley_borrow: u64,
    publishes_with_a_positive_between_borrow: u64,
    borrow_before_the_last_publish: i128,
    intent_new_slots: u64,
    largest_batch_new_slots: u64,
    largest_new_slots: u64,
    publishes_whose_new_slots_exceed_the_landing_demand: u64,
    largest_new_slots_per_mille_of_the_landing_demand: u64,
}

fn largest_over_devices(values: &[i128]) -> i128 {
    values.iter().copied().max().expect("至少一块盘")
}

fn summary_of_the_delete(run: &DeleteRun) -> DeleteSummary {
    let readings = &run.readings;
    let first = readings.first().expect("至少意图那一次");
    let last = readings.last().expect("至少意图那一次");
    let valley_borrow: Vec<i128> = readings
        .iter()
        .map(|reading| largest_over_devices(&reading.borrow_at_the_valley()))
        .collect();
    let between_borrow: Vec<i128> = readings
        .iter()
        .map(|reading| largest_over_devices(&reading.borrow_between_publishes()))
        .collect();
    let reserve: Vec<i128> = readings
        .iter()
        .map(|reading| largest_over_devices(&reading.reserve_slots))
        .collect();
    let (apex_index, apex) = valley_borrow
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_, borrow)| *borrow)
        .expect("有");
    let (apex_between_index, apex_between) = between_borrow
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_, borrow)| *borrow)
        .expect("有");
    let new_slots_of = |reading: &DeletePublishReading| {
        reading.new_slots.iter().copied().max().expect("至少一块盘")
    };
    let landing_ratios: Vec<(u64, u64)> = readings
        .iter()
        .filter_map(|reading| {
            reading
                .shape
                .map(|shape| (new_slots_of(reading), shape.landing_demand_slots))
        })
        .collect();
    DeleteSummary {
        publishes: u64::try_from(readings.len()).expect("发布数"),
        is_completed: run.is_completed,
        refusal: last.refusal.map(|refusal| {
            (
                last.position,
                refusal,
                largest_over_devices(&last.borrow_between_publishes()),
            )
        }),
        apex,
        apex_position: readings[apex_index].position,
        reserve_at_the_apex: reserve[apex_index],
        reserve_minimum: reserve.iter().copied().min().expect("有"),
        reserve_maximum: reserve.iter().copied().max().expect("有"),
        reserve_at_the_start: reserve[0],
        reserve_at_the_last_publish: *reserve.last().expect("有"),
        apex_between,
        apex_between_position: readings[apex_between_index].position,
        publishes_with_a_positive_valley_borrow: u64::try_from(
            valley_borrow.iter().filter(|borrow| **borrow > 0).count(),
        )
        .expect("次数"),
        publishes_with_a_positive_between_borrow: u64::try_from(
            between_borrow.iter().filter(|borrow| **borrow > 0).count(),
        )
        .expect("次数"),
        borrow_before_the_last_publish: *between_borrow.last().expect("有"),
        intent_new_slots: new_slots_of(first),
        largest_batch_new_slots: readings[1..].iter().map(new_slots_of).max().unwrap_or(0),
        largest_new_slots: readings.iter().map(new_slots_of).max().expect("有"),
        publishes_whose_new_slots_exceed_the_landing_demand: u64::try_from(
            landing_ratios
                .iter()
                .filter(|(new_slots, demand)| new_slots > demand)
                .count(),
        )
        .expect("次数"),
        largest_new_slots_per_mille_of_the_landing_demand: landing_ratios
            .iter()
            .filter(|(_, demand)| *demand > 0)
            .map(|(new_slots, demand)| new_slots * 1000 / demand)
            .max()
            .unwrap_or(0),
    }
}

fn summary_fields(summary: &DeleteSummary, slots_per_region: u64) -> String {
    let refusal = summary.refusal.map_or_else(
        || "refused=none refused_at=none borrow_at_refusal=none".to_owned(),
        |(position, refusal, borrow)| {
            format!(
                "refused={} refused_at={position} borrow_at_refusal={borrow}",
                refusal.label()
            )
        },
    );
    let delta = smallest_unbuilt_delta_that_flips_the_cover(
        summary.apex,
        summary.reserve_at_the_apex,
        slots_per_region,
    )
    .map_or_else(|| "none".to_owned(), |delta| delta.to_string());
    format!(
        "publishes={} completed={} {refusal} apex={} apex_at={} reserve_at_apex={} apex_minus_reserve_at_apex={} apex_minus_reserve_minimum={} m1_reserve_covers_apex={} delta_flip={delta} reserve_minimum={} reserve_maximum={} reserve_at_start={} reserve_at_last_publish={} apex_between={} apex_between_at={} positive_valley_publishes={} positive_between_publishes={} borrow_before_last_publish={} intent_new_slots={} largest_batch_new_slots={} largest_new_slots={} w_exceeds_landing_publishes={} largest_w_per_mille_of_landing={}",
        summary.publishes,
        summary.is_completed,
        summary.apex,
        summary.apex_position,
        summary.reserve_at_the_apex,
        summary.apex - summary.reserve_at_the_apex,
        summary.apex - summary.reserve_minimum,
        reserve_covers_the_apex(summary.apex, summary.reserve_at_the_apex),
        summary.reserve_minimum,
        summary.reserve_maximum,
        summary.reserve_at_the_start,
        summary.reserve_at_the_last_publish,
        summary.apex_between,
        summary.apex_between_position,
        summary.publishes_with_a_positive_valley_borrow,
        summary.publishes_with_a_positive_between_borrow,
        summary.borrow_before_the_last_publish,
        summary.intent_new_slots,
        summary.largest_batch_new_slots,
        summary.largest_new_slots,
        summary.publishes_whose_new_slots_exceed_the_landing_demand,
        summary.largest_new_slots_per_mille_of_the_landing_demand,
    )
}

// ---------------------------------------------------------------------------------------------------------------------
// 一格的驱动
// ---------------------------------------------------------------------------------------------------------------------

/// 一格的设定（第六节「格」；布局 L-seq、压满方向 F-end 在第一段固定）。
#[derive(Clone, Copy, Debug)]
struct CellSetting {
    geometry: Geometry,
    batch_data_units: u64,
    arm: Arm,
    data_units: u64,
    /// W 臂账里那一份用的 W_max（同一几何上 N-A 量到的）；N 臂不用。
    set_aside_largest_new_slots: u64,
    reclaim: ReclaimDuringTheDelete,
}

impl CellSetting {
    fn label(&self) -> String {
        let reclaim = match self.reclaim {
            ReclaimDuringTheDelete::AsToday => "",
            ReclaimDuringTheDelete::SwitchedOff => "-noreclaim",
        };
        format!(
            "{}-K{}-Lseq-Fend-{}-N{}{reclaim}",
            self.geometry.label(),
            self.batch_data_units,
            self.arm.label(),
            self.data_units
        )
    }

    fn ledger(&self) -> Option<(u64, u64)> {
        ledger_of_the_arm(
            self.arm,
            self.geometry.slots_per_region,
            self.set_aside_largest_new_slots,
        )
    }
}

/// 一条臂的账（按全部副本之和记的字节）：N 臂两项都是 0；W 臂已承诺预留 = (3S + 1) × W_max × 16384 × 2。
fn ledger_of_the_arm(
    arm: Arm,
    slots_per_region: u64,
    set_aside_largest_new_slots: u64,
) -> Option<(u64, u64)> {
    match arm.ledger {
        LedgerArm::NothingSetAside => Some((0, 0)),
        LedgerArm::RingWindowSetAside => Some((
            0,
            ring_window_bound_slots(slots_per_region, set_aside_largest_new_slots)
                * SLOT_BYTES
                * DEVICE_COUNT,
        )),
    }
}

/// 压满那一步（5.3）按哪一份账压：账与设账的入口（装置里是补丁表接上的 `set_ledger`；单测不经补丁，账恒为第一版的两个 0，
/// 只测 N 臂，设账入口什么都不做）。
#[derive(Clone, Copy)]
struct LedgerForTheFill {
    ledger: Option<(u64, u64)>,
    set_ledger: fn(Option<(u64, u64)>),
}

/// 一格的结局。
#[derive(Clone, Debug)]
enum CellOutcome {
    /// 5.1 第 1、2 步在 `step` 被拒（梯子的上沿；P0 预期如此）。
    FileNotWritten { step: &'static str },
    /// 压满之前式子可用就 ≤ −16384，造不出满池。
    NotFillable,
    /// 删过了；`free_after_the_fill` 是压满之后每块盘的空闲槽（阳性对照拿它与 free_0 比）。
    Deleted {
        summary: Box<DeleteSummary>,
        floor_raise_publishes: u64,
        /// 最后一次删除发布里会话不再推抬 F 的原因（[`FloorRaisesOfTheStep::stop_label`]；阳性对照的行报它）。
        floor_raise_stop_of_the_last_publish: String,
        free_after_the_fill: Vec<u64>,
    },
}

/// 一格（[`run_the_cell_uncounted`]），判完记进这一片的进度（进度行读，不进产物）。
fn run_the_cell(setting: &CellSetting, hooks: &DeletePathHooks) -> CellOutcome {
    let outcome = run_the_cell_uncounted(setting, hooks);
    count_a_judged_cell_on_this_slice();
    outcome
}

/// 一格：建池、核 S4 / B4（P4 上另核 A1）、按臂的账压满、删、核 S7（回收照今天的格）与 B5（回收关着的格），逐次行与格行都打出来。
fn run_the_cell_uncounted(setting: &CellSetting, hooks: &DeletePathHooks) -> CellOutcome {
    let label = setting.label();
    let mut pool = match pool_with_a_file(setting.geometry, setting.data_units) {
        PoolWithAFile::Written(pool) => *pool,
        PoolWithAFile::Refused { step, detail, .. } => {
            emit_product_line!(
                "E name=cell cell={label} first_file=refused step={step} detail={detail:?}"
            );
            return CellOutcome::FileNotWritten { step };
        }
    };
    let (root_level, allocation_record_tree_positions) =
        allocation_record_tree_positions_checked(&pool, hooks);
    if setting.geometry.point == DeviceSizePoint::Product {
        let holds = allocation_record_tree_positions == 529;
        emit_product_line!("E name=anchor_a anchor=A1 cell={label} allocation_record_tree_positions={allocation_record_tree_positions} expected=529 holds={holds}");
        if !holds {
            stop_with("f7_anchor_a1_does_not_hold", &format!("cell={label}"));
        }
    }
    let ledger = setting.ledger();
    (hooks.set_ledger)(ledger);
    let (free_after_the_fill, filled_slots_of_each_device) = match fill_from_the_unit_area_end(
        &mut pool,
    ) {
        FillOutcome::WithinOneSlotAtOrBelowZero {
            free_slots,
            filled_slots_of_each_device,
        } => (free_slots, filled_slots_of_each_device),
        FillOutcome::AlreadyBelowTheIntervalBeforeFilling { available_bytes } => {
            (hooks.set_ledger)(None);
            emit_product_line!("E name=cell cell={label} first_file=written fill=already_below_the_interval available_bytes={available_bytes:?}");
            return CellOutcome::NotFillable;
        }
    };
    let run = run_the_delete(
        &mut pool,
        hooks,
        setting.arm,
        setting.batch_data_units,
        ledger,
        setting.reclaim,
    );
    (hooks.set_ledger)(None);
    for reading in &run.readings {
        emit_product_line!("{}", reading.line(&label));
    }
    let admissions_over_the_limit = run
        .readings
        .iter()
        .filter(|reading| reading.floor_raise_publishes + 1 > PUBLISHES_IN_ONE_ADMISSION_AT_MOST)
        .count();
    if setting.arm.refusal == RefusalHandling::ThroughTheSession {
        emit_product_line!("E name=anchor_a anchor=A4 cell={label} admissions_over_eight_publishes={admissions_over_the_limit} holds={}", admissions_over_the_limit == 0);
    }
    match setting.reclaim {
        ReclaimDuringTheDelete::SwitchedOff => check_the_positive_control_equation(&run, &label),
        ReclaimDuringTheDelete::AsToday => {
            if run.is_completed {
                check_the_finished_delete(
                    &mut pool,
                    hooks,
                    setting,
                    &run,
                    &label,
                    &filled_slots_of_each_device,
                );
            }
        }
    }
    let summary = summary_of_the_delete(&run);
    let floor_raise_publishes: u64 = run
        .readings
        .iter()
        .map(|reading| reading.floor_raise_publishes)
        .sum();
    emit_product_line!(
        "E name=cell cell={label} first_file=written root_level={root_level} allocation_record_tree_positions={allocation_record_tree_positions} unit_area_slots={} free_after_fill={free_after_the_fill:?} floor_raise_publishes={floor_raise_publishes} {}",
        setting.geometry.unit_area_slots(),
        summary_fields(&summary, setting.geometry.slots_per_region)
    );
    CellOutcome::Deleted {
        summary: Box::new(summary),
        floor_raise_publishes,
        floor_raise_stop_of_the_last_publish: run
            .readings
            .last()
            .map_or_else(|| "none".to_owned(), |last| last.floor_raise_stop.clone()),
        free_after_the_fill,
    }
}

/// 第七节 B5（回收关着时）：空闲槽逐次只减不加，free_before_t = free_before_0 − Σ_{s<t}（W_s + 那一次推的抬 F 新占的槽），逐盘逐次断言等号。
/// 登记写的是 borrow_t = borrow_0 + Σ W_s；保留量 reserve_t 随 ckpt_cost 逐次变时那一形会对不上而测量没错，所以两形都算：
/// 空闲槽那一形不成立就停（V1），borrow 那一形对不上的次数照报（`borrow_form_mismatches`），V1 按哪一形判交主 agent。
fn check_the_positive_control_equation(run: &DeleteRun, label: &str) {
    let first = run.readings.first().expect("至少意图那一次");
    let mut expected_free = first.free_before_slots.clone();
    let mut expected_borrow = first.borrow_between_publishes();
    let mut borrow_form_mismatches = 0u64;
    for reading in &run.readings {
        if reading.free_before_slots != expected_free {
            stop_with(
                "v1_b5_positive_control_equation_broken",
                &format!(
                    "cell={label} t={} free_before={:?} expected={expected_free:?}",
                    reading.position, reading.free_before_slots
                ),
            );
        }
        borrow_form_mismatches += u64::from(reading.borrow_between_publishes() != expected_borrow);
        for (((free, borrow), new_slots), raise_slots) in expected_free
            .iter_mut()
            .zip(expected_borrow.iter_mut())
            .zip(&reading.new_slots)
            .zip(&reading.floor_raise_new_slots)
        {
            *free -= new_slots + raise_slots;
            *borrow += i128::from(new_slots + raise_slots);
        }
    }
    emit_product_line!(
        "E name=anchor_b anchor=B5 cell={label} publishes={} free_form_holds=true borrow_form_mismatches={borrow_form_mismatches}",
        run.readings.len()
    );
}

/// 停机条款 S7 判 checker 之前那一次「续做一遍发现无事可做」的删除发布（保留 0 个单元、inode 1 写 nlink = 0）：它是删除类发布，
/// 照意图那次与各批的规矩发——式子那一判关着、落点那一判照判（[`FormulaJudgementSkippedForDeletePublishes`]）。交回这一次的结局与
/// 其间式子那一判真判的次数（该是 0）。
fn publish_the_ledger_rewrite_as_a_delete_publish(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
) -> (StepOutcome, u64) {
    let formula_judgement_skipped_for_the_ledger_rewrite =
        FormulaJudgementSkippedForDeletePublishes::opened(hooks);
    (hooks.set_inode_written_with_zero_links)(Some(1));
    let ledger_rewrite = publish_the_step_directly(pool, hooks, 0, 0);
    (hooks.set_inode_written_with_zero_links)(None);
    (
        ledger_rewrite,
        formula_judgement_skipped_for_the_ledger_rewrite.closed(),
    )
}

/// 停机条款 S7：删完之后 inode 1 的数据单元为 0、删除发布次数 = 1 + ⌈N ÷ K⌉、池级 checker 在删完的镜像上不判红。
///
/// 压满（登记 5.3）只在分配器上标槽、不写盘：标掉的槽进了分配器的「已分配」，删除各次发布照 `transaction.rs` 把分配器的已分配写进
/// 记账行，盘上却没有任何东西引用它们；不撤掉就判，I-3.1、I-3.11 两条差的恰好是压满标掉的槽（2026-10-01 第一段停在
/// P2-S4-K1-Lseq-Fend-N-A-N1：盘 0 记账 292 槽、走读 49 槽，登记第十二节修订第 9 条）。所以判 checker 之前先撤掉压满，再发一次
/// 「续做一遍发现无事可做」的删除发布（保留 0 个单元、inode 1 照旧写 nlink = 0，D8 已定项 5 三态表第三行），最新根的记账按撤掉之后的
/// 分配器重写，checker 判这一版；删除各次的读数在这之前已经取完，这一次发布不进任何量。
fn check_the_finished_delete(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    setting: &CellSetting,
    run: &DeleteRun,
    label: &str,
    filled_slots_of_each_device: &[Vec<u64>],
) {
    let remaining_data_units = pool.current_file_version().data_pointers.len();
    let expected_publishes = delete_publish_count(setting.data_units, setting.batch_data_units);
    let publishes = u64::try_from(run.readings.len()).expect("发布数");
    let filled_slots: Vec<usize> = filled_slots_of_each_device.iter().map(Vec::len).collect();
    let allocated_slots_before_the_unmark: Vec<u64> = pool
        .session
        .allocator
        .devices
        .iter()
        .map(DeviceFreeMap::allocated_slots)
        .collect();
    unmark_the_filled_slots(
        &mut pool.session.allocator.devices,
        filled_slots_of_each_device,
    );
    let free_before_the_ledger_rewrite = pool.free_slots_of_each_device();
    let (ledger_rewrite, formula_judgements_during_the_ledger_rewrite) =
        publish_the_ledger_rewrite_as_a_delete_publish(pool, hooks);
    if formula_judgements_during_the_ledger_rewrite != 0 {
        stop_with(
            "formula_judgement_made_during_the_s7_ledger_rewrite",
            &format!("cell={label} judgements={formula_judgements_during_the_ledger_rewrite}"),
        );
    }
    if let Some(refusal) = ledger_rewrite.refusal {
        stop_with(
            "s7_ledger_rewrite_publish_after_the_unmark_refused",
            &format!(
                "cell={label} refusal={} filled_slots={filled_slots:?} free_before_the_ledger_rewrite={free_before_the_ledger_rewrite:?}",
                refusal.label()
            ),
        );
    }
    let violations = checker_violations(&pool.image());
    if remaining_data_units != 0 || publishes != expected_publishes || !violations.is_empty() {
        stop_with(
            "s7_finished_delete_is_not_what_the_patch_should_leave",
            &format!(
                "cell={label} remaining_data_units={remaining_data_units} publishes={publishes} expected_publishes={expected_publishes} filled_slots={filled_slots:?} allocated_slots_before_the_unmark={allocated_slots_before_the_unmark:?} checker_violations={violations:?}"
            ),
        );
    }
    emit_product_line!(
        "E name=s7_finished_delete cell={label} remaining_data_units={remaining_data_units} publishes={publishes} filled_slots={filled_slots:?} allocated_slots_before_the_unmark={allocated_slots_before_the_unmark:?} checker_violations=0"
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// 几何取样点与第七节 B 类锚点
// ---------------------------------------------------------------------------------------------------------------------

/// 文件大小的装置上限（数据单元）：内容在内存里成一整份、P1 每个 N 另找一块盘；梯子续加到这里还没饱和的，照「撞上沿」报
/// `ladder_ceiling=apparatus_file_size_limit`（第十二节修订：登记 8.4 的上沿只写了「写不进」，没写装置的内存界）。
const LARGEST_FILE_DATA_UNITS: u64 = 32768;
/// P1 搜索的单元区上限（槽）：倍增到这里仍做不成就报找不到。
const SMALLEST_USABLE_SEARCH_LIMIT_SLOTS: u64 = 3 + 2 * (1 << 20);

fn make_filesystem_smallest_geometry(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::MakeFilesystemSmallest,
        device_bytes: make_filesystem_smallest_device_bytes(SMALL_JOURNAL_RING_BYTES),
        journal_ring_bytes: SMALL_JOURNAL_RING_BYTES,
        slots_per_region,
    }
}

/// P1 的一块候选盘：journal 环 `journal_ring_bytes`（修订第 15 条先定下来的那一个）、单元区 `unit_area_slots` 槽。
fn smallest_usable_candidate(
    slots_per_region: u64,
    journal_ring_bytes: u64,
    unit_area_slots: u64,
) -> Geometry {
    Geometry {
        point: DeviceSizePoint::SmallestUsable,
        device_bytes: device_bytes_with_unit_area(journal_ring_bytes, unit_area_slots),
        journal_ring_bytes,
        slots_per_region,
    }
}

fn narrow_geometry(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::Narrow,
        device_bytes: device_bytes_with_unit_area(
            NARROW_JOURNAL_RING_BYTES,
            NARROW_UNIT_AREA_SLOTS,
        ),
        journal_ring_bytes: NARROW_JOURNAL_RING_BYTES,
        slots_per_region,
    }
}

fn product_geometry(slots_per_region: u64) -> Geometry {
    Geometry {
        point: DeviceSizePoint::Product,
        device_bytes: PRODUCT_DEVICE_BYTES,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        slots_per_region,
    }
}

/// 5.1 第 1、2 步（写进 N 个单元的文件、重挂载）做不做得成。
fn pool_is_written(geometry: Geometry, data_units: u64) -> bool {
    matches!(
        pool_with_a_file(geometry, data_units),
        PoolWithAFile::Written(_)
    )
}

/// 写进 N 个单元、重挂载、按 `ledger_for_the_fill` 压满（5.3）之后每块盘空闲槽的最大值；写不进或压不满（压之前式子可用就
/// ≤ −16384）交回 None。
fn free_slots_after_the_fill(
    geometry: Geometry,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> Option<u64> {
    let PoolWithAFile::Written(mut pool) = pool_with_a_file(geometry, data_units) else {
        return None;
    };
    (ledger_for_the_fill.set_ledger)(ledger_for_the_fill.ledger);
    let outcome = fill_from_the_unit_area_end(&mut pool);
    (ledger_for_the_fill.set_ledger)(None);
    match outcome {
        FillOutcome::WithinOneSlotAtOrBelowZero { free_slots, .. } => free_slots.into_iter().max(),
        FillOutcome::AlreadyBelowTheIntervalBeforeFilling { .. } => None,
    }
}

/// 5.1 第 1–3 步（写进、重挂载、压满）做不做得成：读法 C 的 P1 要的那一格（登记第十二节修订第 10 条）。
fn pool_is_written_and_fillable(
    geometry: Geometry,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> bool {
    free_slots_after_the_fill(geometry, data_units, ledger_for_the_fill).is_some()
}

/// P1 搜索的一步：单元区 3 + 2 × 步数 槽（从 P0 的 3 槽起按 2 槽一步）。
fn smallest_usable_unit_area_slots_at_step(step: u64) -> u64 {
    3 + 2 * step
}

/// 从 `first_step` 起往大找 `works` 做得成的最小步数：先倍增离 `first_step` 的距离找做得成的上界，再二分；二分收口时比它小一步
/// （小 2 槽）那一格已经判过做不成（`first_step` 本身做得成时，小一步那一格不在这次搜索里）。单元区超过
/// [`SMALLEST_USABLE_SEARCH_LIMIT_SLOTS`] 仍做不成交回 None。交回 (最小步数, 判过的格数)。
fn smallest_step_that_works(first_step: u64, works: &dyn Fn(u64) -> bool) -> (Option<u64>, u64) {
    let mut judged = 1;
    if works(first_step) {
        return (Some(first_step), judged);
    }
    let mut known_unusable = first_step;
    let mut distance = 1;
    let mut known_usable = loop {
        let candidate = first_step + distance;
        if smallest_usable_unit_area_slots_at_step(candidate) > SMALLEST_USABLE_SEARCH_LIMIT_SLOTS {
            return (None, judged);
        }
        judged += 1;
        if works(candidate) {
            break candidate;
        }
        known_unusable = candidate;
        distance *= 2;
    };
    while known_usable - known_unusable > 1 {
        let middle = known_unusable + (known_usable - known_unusable) / 2;
        judged += 1;
        if works(middle) {
            known_usable = middle;
        } else {
            known_unusable = middle;
        }
    }
    (Some(known_usable), judged)
}

/// 修订第 15 条「先定环」那一步，在搜索上沿那块盘（单元区 [`SMALLEST_USABLE_SEARCH_LIMIT_SLOTS`] 槽）上试一个环的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingProbeOutcome {
    /// 5.1 第 1、2 步做成了：这个环装得下第 1 步那次写。
    Written,
    /// 第 1 步那次写被 journal 记录数上限拒，带着那次发布的记录条数。
    RecordsOverTheLimit(u64),
    /// 在这一步被别的原因拒（不是记录数上限）。
    RefusedAt(&'static str),
}

/// P1 找不到盘的原因（`E name=smallest_usable` 那一行的 `not_found_reason`；只加输出、不改判据）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmallestUsableNotFoundReason {
    /// 搜索上沿那块盘上第 1 步那次写被 journal 记录数上限拒，而环已经放大满 [`JOURNAL_RING_ENLARGEMENTS_AT_MOST`] 次。
    JournalRecordsOfOnePublishExceedTheLimit(JournalRecordsOverTheLimit),
    /// 搜索上沿那块盘上第 1、2 步在这一步被别的原因拒。
    FileNotWrittenOnTheLargestDiskSearched { step: &'static str },
    /// 第 1、2 步找得到盘，第 1–3 步到搜索上沿都做不成（压不满）。
    NotFillableOnAnyDiskSearched,
}

impl SmallestUsableNotFoundReason {
    fn label(&self) -> String {
        match self {
            SmallestUsableNotFoundReason::JournalRecordsOfOnePublishExceedTheLimit(_) => {
                "journal_records_of_one_publish_exceed_the_limit".to_owned()
            }
            SmallestUsableNotFoundReason::FileNotWrittenOnTheLargestDiskSearched { step } => {
                format!("file_not_written_on_the_largest_disk_searched_step_{step}")
            }
            SmallestUsableNotFoundReason::NotFillableOnAnyDiskSearched => {
                "not_fillable_on_any_disk_searched".to_owned()
            }
        }
    }
}

/// 修订第 15 条「先定环」：P1(S, N) 的 journal 环。
#[derive(Clone, Debug, PartialEq, Eq)]
struct JournalRingOfTheSmallestUsableDisk {
    /// 定下来的环；定不下来时是最后试的那一个。
    journal_ring_bytes: u64,
    /// 试过的每个环与它在搜索上沿那块盘上的结局，按试的次序。
    tried: Vec<(u64, RingProbeOutcome)>,
    /// 定不下来的原因；None 是定下来了（最后那个环在搜索上沿那块盘上第 1、2 步做得成）。
    not_settled: Option<SmallestUsableNotFoundReason>,
}

/// 修订第 15 条「先定环」：从 1 MiB 起，在搜索上沿那块盘上写进 N 个单元；第 1 步那次写被 journal 记录数上限拒，就把环换成装得下
/// 那次发布记录条数的最小合法环（[`smallest_legal_journal_ring_holding`]）再试，至多放大 `enlargements_at_most` 次。搜索上沿那块盘
/// 是找盘那一步（[`smallest_step_that_works`]）最后会判的那一格，空间不是这一步的约束。
fn journal_ring_of_the_smallest_usable_disk(
    slots_per_region: u64,
    data_units: u64,
    enlargements_at_most: usize,
) -> JournalRingOfTheSmallestUsableDisk {
    let mut journal_ring_bytes = SMALL_JOURNAL_RING_BYTES;
    let mut tried = Vec::new();
    let mut enlargements = 0;
    loop {
        let at_the_search_limit = smallest_usable_candidate(
            slots_per_region,
            journal_ring_bytes,
            SMALLEST_USABLE_SEARCH_LIMIT_SLOTS,
        );
        let not_settled = match pool_with_a_file(at_the_search_limit, data_units) {
            PoolWithAFile::Written(_) => {
                tried.push((journal_ring_bytes, RingProbeOutcome::Written));
                None
            }
            PoolWithAFile::Refused {
                journal_records_over_the_limit: Some(over_the_limit),
                ..
            } => {
                if over_the_limit.record_limit
                    != journal_record_limit_by_the_format(journal_ring_bytes)
                {
                    stop_with(
                        "journal_record_limit_differs_from_the_format",
                        &format!(
                            "journal_ring_bytes={journal_ring_bytes} implementation={} local={}",
                            over_the_limit.record_limit,
                            journal_record_limit_by_the_format(journal_ring_bytes)
                        ),
                    );
                }
                tried.push((
                    journal_ring_bytes,
                    RingProbeOutcome::RecordsOverTheLimit(over_the_limit.records_of_the_publish),
                ));
                let holding =
                    smallest_legal_journal_ring_holding(over_the_limit.records_of_the_publish);
                if enlargements < enlargements_at_most && holding > journal_ring_bytes {
                    journal_ring_bytes = holding;
                    enlargements += 1;
                    continue;
                }
                Some(
                    SmallestUsableNotFoundReason::JournalRecordsOfOnePublishExceedTheLimit(
                        over_the_limit,
                    ),
                )
            }
            PoolWithAFile::Refused {
                step,
                journal_records_over_the_limit: None,
                ..
            } => {
                tried.push((journal_ring_bytes, RingProbeOutcome::RefusedAt(step)));
                Some(SmallestUsableNotFoundReason::FileNotWrittenOnTheLargestDiskSearched { step })
            }
        };
        return JournalRingOfTheSmallestUsableDisk {
            journal_ring_bytes,
            tried,
            not_settled,
        };
    }
}

/// P1(S, N) 的搜索结局（登记 5.7；读法 C 见第十二节修订第 10 条；环见修订第 15 条）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SmallestUsableSearch {
    /// 先定下来的环（修订第 15 条）；下面两个最小盘都在这个环上找。
    journal_ring: JournalRingOfTheSmallestUsableDisk,
    /// 原定义：5.1 第 1、2 步做得成的最小盘（从 P0 起找）。
    written: Option<Geometry>,
    judged_for_written: u64,
    /// 读法 C：第 1–3 步做得成的最小盘（从原定义那一格起往大找）；P1 取它。
    written_and_fillable: Option<Geometry>,
    judged_for_fillable: u64,
    /// 环大于 1 MiB 时，小一档的环在读法 C 那块盘的单元区上第 1 步那次写被 journal 记录数上限拒（修订第 15 条的最小性；不拒就停机）；
    /// None 是没核（环就是 1 MiB，或没找到盘）。
    smaller_journal_ring_refused_by_the_record_limit: Option<bool>,
    /// 找不到盘的原因；None 是找到了。
    not_found_reason: Option<SmallestUsableNotFoundReason>,
}

/// P1(S, N)：先定环（修订第 15 条），再在那个环上照原定义从 P0 起找第 1、2 步做得成的最小盘，再从那一格起往大找第 1–3 步做得成的
/// 最小盘（读法 C）。压满按 `ledger_for_the_fill` 那一份账压（N 臂两个 0，W 臂另记那一份）。
fn smallest_usable_geometry(
    slots_per_region: u64,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> SmallestUsableSearch {
    smallest_usable_geometry_with_the_ring_enlarged_at_most(
        slots_per_region,
        data_units,
        ledger_for_the_fill,
        JOURNAL_RING_ENLARGEMENTS_AT_MOST,
    )
}

/// [`smallest_usable_geometry`]，环至多放大 `ring_enlargements_at_most` 次（单测拿 0 造「环不许放大」那一格的找不到）。
fn smallest_usable_geometry_with_the_ring_enlarged_at_most(
    slots_per_region: u64,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
    ring_enlargements_at_most: usize,
) -> SmallestUsableSearch {
    let journal_ring = journal_ring_of_the_smallest_usable_disk(
        slots_per_region,
        data_units,
        ring_enlargements_at_most,
    );
    if let Some(reason) = journal_ring.not_settled {
        return SmallestUsableSearch {
            journal_ring,
            written: None,
            judged_for_written: 0,
            written_and_fillable: None,
            judged_for_fillable: 0,
            smaller_journal_ring_refused_by_the_record_limit: None,
            not_found_reason: Some(reason),
        };
    }
    let journal_ring_bytes = journal_ring.journal_ring_bytes;
    let candidate = |step: u64| {
        smallest_usable_candidate(
            slots_per_region,
            journal_ring_bytes,
            smallest_usable_unit_area_slots_at_step(step),
        )
    };
    let (written_step, judged_for_written) =
        smallest_step_that_works(0, &|step| pool_is_written(candidate(step), data_units));
    let (fillable_step, judged_for_fillable) = match written_step {
        Some(first_step) => smallest_step_that_works(first_step, &|step| {
            pool_is_written_and_fillable(candidate(step), data_units, ledger_for_the_fill)
        }),
        None => stop_with(
            "p1_search_misses_the_written_search_limit",
            &format!("S={slots_per_region} data_units={data_units} journal_ring_bytes={journal_ring_bytes}"),
        ),
    };
    let mut search = SmallestUsableSearch {
        journal_ring,
        written: written_step.map(candidate),
        judged_for_written,
        written_and_fillable: fillable_step.map(candidate),
        judged_for_fillable,
        smaller_journal_ring_refused_by_the_record_limit: None,
        not_found_reason: None,
    };
    match search.written_and_fillable {
        None => {
            search.not_found_reason =
                Some(SmallestUsableNotFoundReason::NotFillableOnAnyDiskSearched);
        }
        Some(found) if journal_ring_bytes > SMALL_JOURNAL_RING_BYTES => {
            let one_step_smaller = smallest_usable_candidate(
                slots_per_region,
                journal_ring_bytes - JOURNAL_RING_STEP_BYTES,
                found.unit_area_slots(),
            );
            let is_refused_by_the_record_limit = matches!(
                pool_with_a_file(one_step_smaller, data_units),
                PoolWithAFile::Refused {
                    journal_records_over_the_limit: Some(_),
                    ..
                }
            );
            if !is_refused_by_the_record_limit {
                stop_with(
                    "p1_journal_ring_one_step_smaller_holds_the_write",
                    &format!("S={slots_per_region} data_units={data_units} journal_ring_bytes={journal_ring_bytes} unit_area_slots={}", found.unit_area_slots()),
                );
            }
            search.smaller_journal_ring_refused_by_the_record_limit = Some(true);
        }
        Some(_) => {}
    }
    search
}

/// `E name=smallest_usable` 那一行：原定义（第 1、2 步）与读法 C（第 1–3 步）两个最小盘、各判过几格、搜索上沿（登记 F8）；
/// 修订第 15 条加环、试过的环、小一档的环核没核、找不到的原因四个字段（只加输出）。
fn smallest_usable_line(
    slots_per_region: u64,
    data_units: u64,
    search: &SmallestUsableSearch,
) -> String {
    let unit_area_slots_or_not_found = |geometry: Option<Geometry>| {
        geometry.map_or_else(
            || "not_found".to_owned(),
            |geometry| geometry.unit_area_slots().to_string(),
        )
    };
    format!(
        "E name=smallest_usable S={slots_per_region} data_units={data_units} judged_steps_1_2={} unit_area_slots_steps_1_2={} judged_steps_1_3={} unit_area_slots={} search_limit_unit_area_slots={SMALLEST_USABLE_SEARCH_LIMIT_SLOTS} above_the_narrow_point={} journal_ring_bytes={} journal_ring_tried={:?} smaller_journal_ring_refused_by_the_record_limit={} not_found_reason={}",
        search.judged_for_written,
        unit_area_slots_or_not_found(search.written),
        search.judged_for_fillable,
        unit_area_slots_or_not_found(search.written_and_fillable),
        search
            .written_and_fillable
            .is_none_or(|geometry| geometry.unit_area_slots() > NARROW_UNIT_AREA_SLOTS),
        search.journal_ring.journal_ring_bytes,
        search.journal_ring.tried,
        search
            .smaller_journal_ring_refused_by_the_record_limit
            .map_or_else(|| "not_checked".to_owned(), |is_refused| is_refused.to_string()),
        search
            .not_found_reason
            .as_ref()
            .map_or_else(|| "none".to_owned(), SmallestUsableNotFoundReason::label),
    )
}

/// 第七节 B1、B2、B3：独立算术（B1、B2）与 mkfs 的边（B3）；不成立就作废（V3）。交回核过的条数。
fn anchors_of_class_b() -> usize {
    let device_slots = |bytes: u64| bytes / SLOT_BYTES;
    let two_devices = |start_slot: u64, end_slot_exclusive: u64| {
        vec![
            UnitAreaOnOneDevice {
                start_slot,
                end_slot_exclusive
            };
            2
        ]
    };
    let expected: [(&str, Vec<UnitAreaOnOneDevice>, u32, u64); 5] = [
        (
            "4GiB_default_ring",
            two_devices(
                unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES),
                device_slots(PRODUCT_DEVICE_BYTES),
            ),
            2,
            529,
        ),
        (
            "unit_area_240_ring_6MiB",
            two_devices(1408, 1408 + 240),
            1,
            5,
        ),
        (
            "unit_area_384_ring_6MiB",
            two_devices(1408, 1408 + 384),
            1,
            5,
        ),
        (
            "1GiB_ring_1MiB",
            two_devices(1088, device_slots(1 << 30)),
            1,
            161,
        ),
        (
            "256GiB_default_ring",
            two_devices(
                unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES),
                device_slots(256 << 30),
            ),
            3,
            41451,
        ),
    ];
    let mut checked = 0;
    for (name, unit_areas, expected_root_level, expected_positions) in expected {
        let (root_level, positions) = allocation_record_tree_positions_by_the_clause(&unit_areas);
        let holds = root_level == expected_root_level && positions == expected_positions;
        emit_product_line!("E name=anchor_b anchor=B1_B2 geometry={name} root_level={root_level} allocation_record_tree_positions={positions} expected_root_level={expected_root_level} expected_positions={expected_positions} holds={holds}");
        if !holds {
            stop_with("v3_anchor_b1_b2_does_not_hold", &format!("geometry={name}"));
        }
        checked += 1;
    }
    let smallest = make_filesystem_smallest_device_bytes(SMALL_JOURNAL_RING_BYTES);
    let default_smallest = make_filesystem_smallest_device_bytes(JOURNAL_RING_DEFAULT_BYTES);
    let make_filesystem_on = |device_bytes: u64, journal_ring_bytes: u64| {
        let geometry = Geometry {
            point: DeviceSizePoint::MakeFilesystemSmallest,
            device_bytes,
            journal_ring_bytes,
            slots_per_region: ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        };
        make_filesystem(
            &make_filesystem_parameters(&geometry),
            &mut blank_devices(device_bytes),
        )
    };
    let edges = [
        (
            "ring_1MiB_smallest",
            smallest,
            make_filesystem_on(smallest, SMALL_JOURNAL_RING_BYTES).is_ok(),
        ),
        (
            "ring_1MiB_smallest_minus_512",
            smallest - 512,
            matches!(
                make_filesystem_on(smallest - 512, SMALL_JOURNAL_RING_BYTES),
                Err(MakeFilesystemError::UnitAreaBeyondDevice { .. })
            ),
        ),
        (
            "default_ring_smallest",
            default_smallest,
            make_filesystem_on(default_smallest, JOURNAL_RING_DEFAULT_BYTES).is_ok(),
        ),
        (
            "default_ring_smallest_minus_512",
            default_smallest - 512,
            matches!(
                make_filesystem_on(default_smallest - 512, JOURNAL_RING_DEFAULT_BYTES),
                Err(MakeFilesystemError::JournalRingTooLargeForDevice { .. })
            ),
        ),
    ];
    for (name, device_bytes, holds) in edges {
        emit_product_line!(
            "E name=anchor_b anchor=B3 case={name} device_bytes={device_bytes} holds={holds}"
        );
        if !holds {
            stop_with("v3_anchor_b3_does_not_hold", &format!("case={name}"));
        }
        checked += 1;
    }
    checked
}

// ---------------------------------------------------------------------------------------------------------------------
// 梯子组（同一 (盘大小点, S, K, 臂) 下扫 N）与阳性对照
// ---------------------------------------------------------------------------------------------------------------------

/// 一组梯子的结局，供判决行数。
#[derive(Clone, Debug, Default)]
struct GroupTally {
    cells_deleted: u64,
    cells_refused: u64,
    cells_apex_over_reserve: u64,
    cells_apex_over_ring_window: u64,
    cells_not_fillable: u64,
    is_saturated: bool,
    is_w_growing_with_file_size: bool,
    publishes_whose_w_exceeds_landing: u64,
    largest_new_slots: u64,
}

/// 一个盘大小点在这个 S、这个 N 上的几何；P1 先定环（修订第 15 条）、再照读法 C 现找（压满按 `ledger_for_the_fill`），找不到交回
/// None。P1 每找一次打一行 `E name=smallest_usable`（[`smallest_usable_line`]）。
fn geometry_of_the_point(
    point: DeviceSizePoint,
    slots_per_region: u64,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> Option<Geometry> {
    match point {
        DeviceSizePoint::MakeFilesystemSmallest => {
            Some(make_filesystem_smallest_geometry(slots_per_region))
        }
        DeviceSizePoint::SmallestUsable => {
            let search =
                smallest_usable_geometry(slots_per_region, data_units, ledger_for_the_fill);
            emit_product_line!(
                "{}",
                smallest_usable_line(slots_per_region, data_units, &search)
            );
            search.written_and_fillable
        }
        DeviceSizePoint::Narrow => Some(narrow_geometry(slots_per_region)),
        DeviceSizePoint::Product => Some(product_geometry(slots_per_region)),
    }
}

/// 一组梯子：8.4 的 N 依次跑，写不进那一点是上沿；末端不满足 M3 的饱和规则就按 ×2 续加，直到饱和、撞上沿或续加满
/// [`LADDER_EXTENSIONS_AT_MOST`] 次。每格的 M7 用这一组的 W_max，组行报 M3、M5、W_max。
fn run_the_ladder_group(
    point: DeviceSizePoint,
    slots_per_region: u64,
    batch_data_units: u64,
    arm: Arm,
    set_aside_largest_new_slots: u64,
    hooks: &DeletePathHooks,
) -> GroupTally {
    let mut file_sizes: Vec<u64> = ladder_of_file_sizes(batch_data_units, slots_per_region)
        .into_iter()
        .filter(|data_units| *data_units <= LARGEST_FILE_DATA_UNITS)
        .collect();
    let mut deleted: Vec<(CellSetting, DeleteSummary)> = Vec::new();
    let mut tally = GroupTally::default();
    let mut ceiling = "none".to_owned();
    let mut extensions = 0;
    let mut next = 0;
    loop {
        if next == file_sizes.len() {
            let apex_series: Vec<(u64, i128)> = deleted
                .iter()
                .map(|(_, summary)| (summary.publishes, summary.apex))
                .collect();
            if apex_saturation(&apex_series, slots_per_region) == ApexSaturation::Saturated
                || extensions == LADDER_EXTENSIONS_AT_MOST
            {
                break;
            }
            let doubled = file_sizes.last().expect("梯子至少一点") * 2;
            if doubled > LARGEST_FILE_DATA_UNITS {
                ceiling = format!("apparatus_file_size_limit_{LARGEST_FILE_DATA_UNITS}");
                break;
            }
            file_sizes.push(doubled);
            extensions += 1;
        }
        let data_units = file_sizes[next];
        next += 1;
        let ledger_for_the_fill = LedgerForTheFill {
            ledger: ledger_of_the_arm(arm, slots_per_region, set_aside_largest_new_slots),
            set_ledger: hooks.set_ledger,
        };
        let Some(geometry) =
            geometry_of_the_point(point, slots_per_region, data_units, ledger_for_the_fill)
        else {
            ceiling = format!("smallest_usable_not_found_at_{data_units}");
            break;
        };
        let setting = CellSetting {
            geometry,
            batch_data_units,
            arm,
            data_units,
            set_aside_largest_new_slots,
            reclaim: ReclaimDuringTheDelete::AsToday,
        };
        match run_the_cell(&setting, hooks) {
            CellOutcome::FileNotWritten { step } => {
                ceiling = format!("file_not_written_at_{data_units}_step_{step}");
                break;
            }
            CellOutcome::NotFillable => tally.cells_not_fillable += 1,
            CellOutcome::Deleted { summary, .. } => deleted.push((setting, *summary)),
        }
    }
    let largest_new_slots = deleted
        .iter()
        .map(|(_, summary)| summary.largest_new_slots)
        .max()
        .unwrap_or(0);
    let bound = ring_window_bound_slots(slots_per_region, largest_new_slots);
    for (setting, summary) in &deleted {
        let apex_minus_bound = summary.apex - i128::from(bound);
        emit_product_line!(
            "E name=ring_window cell={} apex={} ring_window_bound={bound} apex_minus_bound={apex_minus_bound} m7_within_bound={}",
            setting.label(),
            summary.apex,
            apex_minus_bound <= 0
        );
        tally.cells_apex_over_ring_window += u64::from(apex_minus_bound > 0);
        tally.cells_apex_over_reserve += u64::from(!reserve_covers_the_apex(
            summary.apex,
            summary.reserve_at_the_apex,
        ));
        tally.cells_refused += u64::from(summary.refusal.is_some());
        tally.publishes_whose_w_exceeds_landing +=
            summary.publishes_whose_new_slots_exceed_the_landing_demand;
    }
    let apex_series: Vec<(u64, i128)> = deleted
        .iter()
        .map(|(_, summary)| (summary.publishes, summary.apex))
        .collect();
    let largest_by_point: Vec<u64> = deleted
        .iter()
        .map(|(_, summary)| summary.largest_new_slots)
        .collect();
    let saturation = apex_saturation(&apex_series, slots_per_region);
    tally.cells_deleted = u64::try_from(deleted.len()).expect("格数");
    tally.is_saturated = saturation == ApexSaturation::Saturated;
    tally.is_w_growing_with_file_size = largest_new_slots_grow_with_file_size(&largest_by_point);
    tally.largest_new_slots = largest_new_slots;
    emit_product_line!(
        "E name=group point={} S={slots_per_region} K={batch_data_units} arm={} cells_deleted={} cells_not_fillable={} cells_refused={} cells_apex_over_reserve={} cells_apex_over_ring_window={} w_max={largest_new_slots} w_max_intent={} w_max_batches={} ring_window_bound={bound} m3_saturation={saturation:?} m5_w_grows_with_file_size={} w_exceeds_landing_publishes={} ladder_extensions={extensions} ladder_ceiling={ceiling} apex_series={apex_series:?} w_max_series={largest_by_point:?}",
        point.label(),
        arm.label(),
        tally.cells_deleted,
        tally.cells_not_fillable,
        tally.cells_refused,
        tally.cells_apex_over_reserve,
        tally.cells_apex_over_ring_window,
        deleted.iter().map(|(_, summary)| summary.intent_new_slots).max().unwrap_or(0),
        deleted.iter().map(|(_, summary)| summary.largest_batch_new_slots).max().unwrap_or(0),
        tally.is_w_growing_with_file_size,
        tally.publishes_whose_w_exceeds_landing,
    );
    if point == DeviceSizePoint::SmallestUsable {
        emit_product_line!(
            "{}",
            group_line_on_the_one_mebibyte_ring_cells(
                slots_per_region,
                batch_data_units,
                arm,
                &deleted
            )
        );
    }
    tally
}

/// 修订第 15 条记一次输、只许收严那一半：P1 一组里环仍是 1 MiB 的那几格（就是改之前的形态在这一组上跑得到的那一段梯子：环随 N 只增，
/// 1 MiB 的格是梯子的前缀），照组行同一套算法另算 M3、M5、W_max 与 M7，打一行 `E name=group_one_mebibyte_ring_cells`。组行的判定
/// 不动；这一行让放大环之后多出来的格不能把改之前判得出的「没饱和」「W 随文件大小涨」「越过环窗口界」抹掉。
fn group_line_on_the_one_mebibyte_ring_cells(
    slots_per_region: u64,
    batch_data_units: u64,
    arm: Arm,
    deleted: &[(CellSetting, DeleteSummary)],
) -> String {
    let on_the_one_mebibyte_ring: Vec<&DeleteSummary> = deleted
        .iter()
        .filter(|(setting, _)| setting.geometry.journal_ring_bytes == SMALL_JOURNAL_RING_BYTES)
        .map(|(_, summary)| summary)
        .collect();
    let largest_new_slots = on_the_one_mebibyte_ring
        .iter()
        .map(|summary| summary.largest_new_slots)
        .max()
        .unwrap_or(0);
    let bound = ring_window_bound_slots(slots_per_region, largest_new_slots);
    let cells_apex_over_ring_window = on_the_one_mebibyte_ring
        .iter()
        .filter(|summary| summary.apex - i128::from(bound) > 0)
        .count();
    let apex_series: Vec<(u64, i128)> = on_the_one_mebibyte_ring
        .iter()
        .map(|summary| (summary.publishes, summary.apex))
        .collect();
    let largest_by_point: Vec<u64> = on_the_one_mebibyte_ring
        .iter()
        .map(|summary| summary.largest_new_slots)
        .collect();
    format!(
        "E name=group_one_mebibyte_ring_cells point=P1 S={slots_per_region} K={batch_data_units} arm={} cells_deleted={} cells_refused={} cells_apex_over_reserve={} cells_apex_over_ring_window={cells_apex_over_ring_window} w_max={largest_new_slots} ring_window_bound={bound} m3_saturation={:?} m5_w_grows_with_file_size={} apex_series={apex_series:?} w_max_series={largest_by_point:?}",
        arm.label(),
        on_the_one_mebibyte_ring.len(),
        on_the_one_mebibyte_ring
            .iter()
            .filter(|summary| summary.refusal.is_some())
            .count(),
        on_the_one_mebibyte_ring
            .iter()
            .filter(|summary| !reserve_covers_the_apex(summary.apex, summary.reserve_at_the_apex))
            .count(),
        apex_saturation(&apex_series, slots_per_region),
        largest_new_slots_grow_with_file_size(&largest_by_point),
    )
}

/// 阳性对照里 N 与盘的迭代次数上限。
const POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST: usize = 8;

/// 阳性对照定下来的盘、N 与 free_0（N = free_0 + 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlFileSize {
    geometry: Geometry,
    data_units: u64,
    free_after_the_fill_slots: u64,
}

/// 阳性对照迭代过的每一次：(N, 那块盘压满之后读到的空闲；取不到盘或压不满是 None)。
type PositiveControlIterations = Vec<(u64, Option<u64>)>;

/// 阳性对照的盘、文件大小 N 与 free_0（登记 5.5：N = free_0 + 2，free_0 = 压满之后那块盘的空闲槽；第十二节修订第 11 条）。N 与盘
/// 互相定——P1 的盘随 N 找，压满之后的空闲随 N 写出的树变——所以从 N = 1 起迭代：按 N 取盘（`geometry_for`）、在那块盘上写进 N 个
/// 单元、压满、读空闲（`free_after_the_fill`），读到的 + 2 = N 就是它，否则 N 取读到的 + 2 再来；至多
/// [`POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST`] 次。交回 (盘, N, free_0)；不收敛、或某一次取不到盘、压不满，交回 Err(迭代过的
/// (N, 读到的空闲))。
fn positive_control_file_size_on_the_filled_disk(
    geometry_for: &dyn Fn(u64) -> Option<Geometry>,
    free_after_the_fill: &dyn Fn(Geometry, u64) -> Option<u64>,
) -> Result<PositiveControlFileSize, PositiveControlIterations> {
    let mut tried = Vec::new();
    let mut data_units = 1;
    for _ in 0..POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST {
        let Some(geometry) = geometry_for(data_units) else {
            tried.push((data_units, None));
            return Err(tried);
        };
        let free_slots_read = free_after_the_fill(geometry, data_units);
        tried.push((data_units, free_slots_read));
        let Some(free_after_the_fill_slots) = free_slots_read else {
            return Err(tried);
        };
        if free_after_the_fill_slots + 2 == data_units {
            return Ok(PositiveControlFileSize {
                geometry,
                data_units,
                free_after_the_fill_slots,
            });
        }
        data_units = free_after_the_fill_slots + 2;
    }
    Err(tried)
}

/// 5.5 阳性对照的两个几何：最小可用盘（P1）× S = 16，与 4 GiB（P4）× S = 4。
const POSITIVE_CONTROL_GEOMETRIES: [(DeviceSizePoint, u64); 2] = [
    (
        DeviceSizePoint::SmallestUsable,
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
    ),
    (DeviceSizePoint::Product, ROOT_RING_SLOTS_PER_REGION_MINIMUM),
];

/// 几何点与 S 的名字（`P1-S16`），`--geometry` 与 W_max 的键都用它。
fn geometry_label_of(point: DeviceSizePoint, slots_per_region: u64) -> String {
    format!("{}-S{slots_per_region}", point.label())
}

/// 阳性对照的一片：一个几何上的一条臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlSlice {
    point: DeviceSizePoint,
    slots_per_region: u64,
    arm: Arm,
}

/// 阳性对照各片，按登记 5.5 的次序（几何在外、四条臂在里）；`filter` 只留这一个几何。
fn positive_control_slices(filter: Option<&str>) -> Vec<PositiveControlSlice> {
    POSITIVE_CONTROL_GEOMETRIES
        .into_iter()
        .filter(|(point, slots_per_region)| {
            filter.is_none_or(|wanted| wanted == geometry_label_of(*point, *slots_per_region))
        })
        .flat_map(|(point, slots_per_region)| {
            ALL_ARMS.map(|arm| PositiveControlSlice {
                point,
                slots_per_region,
                arm,
            })
        })
        .collect()
}

fn describe_positive_control(slice: &PositiveControlSlice) -> SliceDescription {
    SliceDescription {
        label: format!(
            "PC-{}-{}",
            geometry_label_of(slice.point, slice.slots_per_region),
            slice.arm.label()
        ),
        is_large: slice.point == DeviceSizePoint::Product,
        start_rank: 0,
    }
}

/// 5.5 阳性对照的一格：K = 1、N = free_0 + 2（free_0 读写进 N 个单元、压满之后那块盘本身，见
/// [`positive_control_file_size_on_the_filled_disk`]）、删除阶段回收关着。交回这一格的两条判定（[`PositiveControlCellVerdict`]）：
/// 出现拒绝（V1），B 臂另要拒绝是经会话推抬 F 那一步交回的（U12）。
fn run_one_positive_control(
    slice: &PositiveControlSlice,
    set_aside_largest_new_slots: u64,
    hooks: &DeletePathHooks,
) -> PositiveControlCellVerdict {
    let PositiveControlSlice {
        point,
        slots_per_region,
        arm,
    } = *slice;
    let group_label = geometry_label_of(point, slots_per_region);
    let ledger_for_the_fill = LedgerForTheFill {
        ledger: ledger_of_the_arm(arm, slots_per_region, set_aside_largest_new_slots),
        set_ledger: hooks.set_ledger,
    };
    let settled = positive_control_file_size_on_the_filled_disk(
        &|data_units| {
            geometry_of_the_point(point, slots_per_region, data_units, ledger_for_the_fill)
        },
        &|geometry, data_units| {
            free_slots_after_the_fill(geometry, data_units, ledger_for_the_fill)
        },
    );
    let PositiveControlFileSize {
        geometry,
        data_units,
        free_after_the_fill_slots,
    } = match settled {
        Ok(settled) => settled,
        Err(tried) => stop_with(
            "v1_positive_control_file_size_does_not_settle",
            &format!("geometry={group_label} arm={} tried={tried:?}", arm.label()),
        ),
    };
    let setting = CellSetting {
        geometry,
        batch_data_units: SMALLEST_BATCH_DATA_UNITS,
        arm,
        data_units,
        set_aside_largest_new_slots,
        reclaim: ReclaimDuringTheDelete::SwitchedOff,
    };
    let (refusal, floor_raise_publishes, floor_raise_stop) = match run_the_cell(&setting, hooks) {
        CellOutcome::Deleted {
            summary,
            floor_raise_publishes,
            floor_raise_stop_of_the_last_publish,
            free_after_the_fill,
        } => {
            if free_after_the_fill.iter().copied().max() != Some(free_after_the_fill_slots) {
                stop_with(
                    "v1_positive_control_free_0_differs_from_the_filled_cell",
                    &format!(
                        "cell={} free_0={free_after_the_fill_slots} cell_free_after_fill={free_after_the_fill:?}",
                        setting.label()
                    ),
                );
            }
            (
                summary.refusal.map(|(_position, refusal, _borrow)| refusal),
                floor_raise_publishes,
                floor_raise_stop_of_the_last_publish,
            )
        }
        CellOutcome::FileNotWritten { .. } | CellOutcome::NotFillable => {
            (None, 0, "none".to_owned())
        }
    };
    let verdict = PositiveControlCellVerdict::of_the_cell(arm.refusal, refusal);
    emit_product_line!(
        "E name=positive_control cell={} free_0={free_after_the_fill_slots} set_aside_w_max={set_aside_largest_new_slots} refused={} refusal={} floor_raise_publishes={floor_raise_publishes} floor_raise_stop={floor_raise_stop} u12_refused_with_no_space_after_raising_the_floor={} holds={}",
        setting.label(),
        verdict.is_refused,
        refusal.map_or("none", DeleteRefusal::label),
        verdict.u12_label(),
        verdict.holds()
    );
    verdict
}

/// 阳性对照一格的两条判定，各记各的（登记第十一节 V1：出现拒绝；第九节 U12：B 臂被拒之前至少推过一串抬 F）。U12 按读法乙判（用户
/// 2026-10-02 定案，登记第十二节修订第 17 条）：拒绝是经会话推抬 F 那一步交回的（[`DeleteRefusal::NoSpaceAfterRaisingTheFloor`]）即成立，
/// 推的那一串在预演里就被拒、一次都没落盘的也算；抬 F 落没落盘（[`DeletePublishReading::floor_raise_publishes`]）不再是条件。B 臂不经会话
/// 时拒绝种类是 [`DeleteRefusal::Landing`]（或 `Placement`），U12 不成立。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlCellVerdict {
    is_refused: bool,
    /// B 臂：拒绝种类是 `no_space_after_raising_the_floor`；A 臂没有这一条（None）。
    is_refused_with_no_space_after_raising_the_floor: Option<bool>,
}

impl PositiveControlCellVerdict {
    fn of_the_cell(refusal_handling: RefusalHandling, refusal: Option<DeleteRefusal>) -> Self {
        Self {
            is_refused: refusal.is_some(),
            is_refused_with_no_space_after_raising_the_floor: match refusal_handling {
                RefusalHandling::Direct => None,
                RefusalHandling::ThroughTheSession => {
                    Some(refusal == Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor))
                }
            },
        }
    }

    fn u12_label(self) -> &'static str {
        match self.is_refused_with_no_space_after_raising_the_floor {
            None => "not_applicable",
            Some(true) => "true",
            Some(false) => "false",
        }
    }

    fn holds(self) -> bool {
        self.is_refused && self.is_refused_with_no_space_after_raising_the_floor != Some(false)
    }
}

/// 阳性对照各格的计数，每个数只数一个量（verdict 行逐个报）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PositiveControlTally {
    cells: u64,
    /// 不出拒绝的格（V1）。
    cells_without_refusal: u64,
    /// B 臂出了拒绝、而拒绝种类不是 `no_space_after_raising_the_floor` 的格（U12，读法乙见 [`PositiveControlCellVerdict`]）。
    session_cells_refused_other_than_no_space_after_raising_the_floor: u64,
    /// 两条里有一条不合的格（`holds=false`）。
    cells_not_holding: u64,
}

impl PositiveControlTally {
    fn of_the_cells(verdicts: &[PositiveControlCellVerdict]) -> Self {
        let count = |judged: fn(&PositiveControlCellVerdict) -> bool| -> u64 {
            u64::try_from(verdicts.iter().filter(|verdict| judged(verdict)).count()).expect("格数")
        };
        Self {
            cells: u64::try_from(verdicts.len()).expect("格数"),
            cells_without_refusal: count(|verdict| !verdict.is_refused),
            session_cells_refused_other_than_no_space_after_raising_the_floor: count(|verdict| {
                verdict.is_refused
                    && verdict.is_refused_with_no_space_after_raising_the_floor == Some(false)
            }),
            cells_not_holding: count(|verdict| !verdict.holds()),
        }
    }

    /// verdict 行里阳性对照那几个字段。
    fn verdict_fields(self) -> String {
        format!(
            "positive_control_cells={} positive_control_cells_without_refusal={} positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor={} positive_control_cells_not_holding={}",
            self.cells,
            self.cells_without_refusal,
            self.session_cells_refused_other_than_no_space_after_raising_the_floor,
            self.cells_not_holding
        )
    }

    /// 有一格不合就停（打在 verdict 行之后）：不出拒绝按 V1 作废这一段（`v1_positive_control_cells_without_refusal`）；
    /// 都出了拒绝而 B 臂有一格的拒绝不是经会话推抬 F 那一步交回的，U12 那条断言不成立（`u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor`）。
    fn stop_reason(self) -> Option<&'static str> {
        if self.cells_without_refusal > 0 {
            Some("v1_positive_control_cells_without_refusal")
        } else if self.session_cells_refused_other_than_no_space_after_raising_the_floor > 0 {
            Some("u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor")
        } else {
            None
        }
    }
}

/// verdict 行打完之后：阳性对照有一格不合就打停机行、退 [`STOP_EXIT_CODE`]（[`PositiveControlTally::stop_reason`]），不打 `name=done`。
fn stop_when_a_positive_control_does_not_hold(tally: PositiveControlTally) {
    if let Some(reason) = tally.stop_reason() {
        stop_with(reason, &tally.verdict_fields());
    }
}

/// 5.5 阳性对照：四条臂 × 两个几何，各格按片并行。W 臂账里那一份取 `largest_new_slots_of_each_geometry` 里那个几何的 W_max（没有记 0）。
fn run_the_positive_controls(
    largest_new_slots_of_each_geometry: &BTreeMap<String, u64>,
    filter: Option<&str>,
    hooks: &DeletePathHooks,
    scheduling: SliceScheduling,
) -> PositiveControlTally {
    let slices = positive_control_slices(filter);
    let verdicts =
        run_slices_of_the_apparatus(&slices, &describe_positive_control, scheduling, &|slice| {
            let set_aside = largest_new_slots_of_each_geometry
                .get(&geometry_label_of(slice.point, slice.slots_per_region))
                .copied()
                .unwrap_or(0);
            run_one_positive_control(slice, set_aside, hooks)
        });
    PositiveControlTally::of_the_cells(&verdicts)
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
    emit_product_line!("E name=anchors_b checked={}", anchors_of_class_b());
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
        sum_of(None, |tally| u64::from(tally.is_w_growing_with_file_size)),
        sum_of(None, |tally| tally.publishes_whose_w_exceeds_landing),
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
    emit_product_line!("E name=anchors_b checked={}", anchors_of_class_b());
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
fn run_the_s1_history() {
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
        let PoolWithAFile::Written(mut pool) = pool_with_a_file(geometry, data_units) else {
            stop_with(
                "s1_history_file_not_written",
                &format!("geometry={}", geometry.label()),
            );
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
            run_the_s1_history();
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
mod tests {
    use super::*;

    #[test]
    fn ring_length_is_three_regions_times_the_slots_per_region() {
        assert_eq!(ring_length_in_roots(4), 12, "S = 4 时环长 12（第七节 A3）");
        assert_eq!(ring_length_in_roots(8), 24, "S = 8 时环长 24");
        assert_eq!(ring_length_in_roots(16), 48, "S = 16 时环长 48");
    }

    #[test]
    fn allocation_record_tree_root_level_steps_up_right_past_the_registered_boundaries() {
        assert_eq!(
            allocation_record_tree_root_level(&[68208, 68208]),
            1,
            "两块盘各 68208 槽：2 × 84 = 168 ≤ 169，根层级 1（第七节 B2）"
        );
        assert_eq!(
            allocation_record_tree_root_level(&[68209, 68209]),
            2,
            "各 68209 槽：2 × 85 = 170 > 169，根层级 2"
        );
        assert_eq!(
            allocation_record_tree_root_level(&[11_527_152, 11_527_152]),
            2,
            "各 11527152 槽：根层级 2"
        );
        assert_eq!(
            allocation_record_tree_root_level(&[11_527_153, 11_527_153]),
            3,
            "各 11527153 槽：根层级 3"
        );
    }

    #[test]
    fn allocation_record_tree_positions_match_the_registered_anchors() {
        let two_devices = |start_slot: u64, end_slot_exclusive: u64| {
            vec![
                UnitAreaOnOneDevice {
                    start_slot,
                    end_slot_exclusive
                };
                2
            ]
        };
        assert_eq!(
            allocation_record_tree_positions_by_the_clause(&two_devices(50176, 262_144)),
            (2, 529),
            "4 GiB 默认环（D28 已定项 4、第七节 A1 / B1）"
        );
        assert_eq!(
            allocation_record_tree_positions_by_the_clause(&two_devices(1408, 1408 + 240)),
            (1, 5),
            "单元区 240 槽、环 6 MiB"
        );
        assert_eq!(
            allocation_record_tree_positions_by_the_clause(&two_devices(1408, 1408 + 384)),
            (1, 5),
            "单元区 384 槽、环 6 MiB（P2）"
        );
        assert_eq!(
            allocation_record_tree_positions_by_the_clause(&two_devices(1088, 65536)),
            (1, 161),
            "1 GiB、环 1 MiB"
        );
        assert_eq!(
            allocation_record_tree_positions_by_the_clause(&two_devices(50176, 16_777_216)),
            (3, 41451),
            "256 GiB 默认环"
        );
    }

    #[test]
    fn make_filesystem_smallest_device_matches_the_registered_anchors() {
        assert_eq!(
            make_filesystem_smallest_device_bytes(1 << 20),
            17_874_944,
            "环 1 MiB：(1088 + 3) × 16384（第七节 B3）"
        );
        assert_eq!(
            make_filesystem_smallest_device_bytes(1_032_193),
            17_874_944,
            "环 1032193 字节的单元区起点同为槽 1088"
        );
        assert_eq!(
            make_filesystem_smallest_device_bytes(805_306_368),
            3_221_225_472,
            "默认环：环 ≤ 盘 ÷ 4 先卡住"
        );
        assert_eq!(
            unit_area_start_slot(805_306_368),
            50176,
            "默认环的单元区起点"
        );
        assert_eq!(
            device_bytes_with_unit_area(6 << 20, 384),
            (1408 + 384) * 16384,
            "P2：环 6 MiB、单元区 384 槽"
        );
    }

    #[test]
    fn reserve_by_the_clauses_is_eight_plus_thirteen_checkpoint_costs() {
        for checkpoint_cost in [1, 9, 161, 529] {
            assert_eq!(
                reserve_slots_by_the_clauses(checkpoint_cost),
                8 + 13 * checkpoint_cost,
                "c = {checkpoint_cost} 时每盘保留量 8 + 13c（第七节 A2）"
            );
        }
    }

    #[test]
    fn filling_lands_the_available_bytes_within_one_slot_at_or_below_zero() {
        assert!(
            available_is_within_one_slot_at_or_below_zero(0),
            "0 在区间里"
        );
        assert!(
            available_is_within_one_slot_at_or_below_zero(-16383),
            "−16383 在区间里"
        );
        assert!(
            !available_is_within_one_slot_at_or_below_zero(-16384),
            "−16384 不在区间里"
        );
        assert!(
            !available_is_within_one_slot_at_or_below_zero(1),
            "1 不在区间里"
        );
        assert_eq!(
            slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(-5),
            0,
            "已经 ≤ 0 不标"
        );
        for available in [1i128, 16383, 16384, 16385, 3 * 16384 + 7] {
            let marked = i128::from(
                slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(available),
            );
            assert!(
                available_is_within_one_slot_at_or_below_zero(available - marked * 16384),
                "可用 {available} 标 {marked} 槽之后落进 (−16384, 0]"
            );
        }
    }

    #[test]
    fn m1_counts_an_apex_equal_to_the_reserve_as_covered() {
        assert!(
            reserve_covers_the_apex(125, 125),
            "顶点 = 保留量判罩得住（M1 的门槛是 ≤ 0）"
        );
        assert!(
            !reserve_covers_the_apex(126, 125),
            "顶点比保留量多 1 槽判越过"
        );
    }

    #[test]
    fn m3_is_not_saturated_when_the_second_last_point_rises_and_the_last_falls_back() {
        let rises_then_falls_back = [(14, 10), (15, 12), (16, 10)];
        assert_eq!(
            apex_saturation(&rises_then_falls_back, 4),
            ApexSaturation::GrowingWithoutSaturation,
            "最后两点里一点比之前那一点高，不算饱和"
        );
        assert_eq!(
            apex_saturation(&[(14, 12), (15, 12), (16, 11)], 4),
            ApexSaturation::Saturated,
            "最后两点都不高于之前那一点"
        );
        assert_eq!(
            apex_saturation(&[(13, 50), (14, 12), (15, 12)], 4),
            ApexSaturation::FewerThanThreePointsPastTheRing,
            "批数 ≥ 3S + 2 = 14 的点只有两个"
        );
    }

    #[test]
    fn m5_needs_three_strictly_rising_last_points() {
        assert!(
            largest_new_slots_grow_with_file_size(&[5, 1, 2, 3]),
            "最后三点 1 < 2 < 3"
        );
        assert!(
            !largest_new_slots_grow_with_file_size(&[1, 2, 2]),
            "最后三点有一对相等"
        );
        assert!(!largest_new_slots_grow_with_file_size(&[2, 3]), "不到三点");
    }

    #[test]
    fn smallest_unbuilt_delta_counts_the_ring_window_per_slot() {
        assert_eq!(
            smallest_unbuilt_delta_that_flips_the_cover(100, 125, 4),
            Some(2),
            "S = 4：100 + 13 × 2 = 126 > 125"
        );
        assert_eq!(
            smallest_unbuilt_delta_that_flips_the_cover(126, 125, 4),
            Some(0),
            "已经越过"
        );
        assert_eq!(
            smallest_unbuilt_delta_that_flips_the_cover(0, 125, 4),
            None,
            "δ ≤ 3 翻不了"
        );
        assert_eq!(ring_window_bound_slots(4, 10), 130, "(12 + 1) × 10");
    }

    #[test]
    fn ladder_of_file_sizes_crosses_the_ring_for_both_batch_sizes() {
        assert_eq!(
            ladder_of_file_sizes(144, 4),
            vec![144, 288, 576, 1152, 1584, 1728, 1872, 2016, 2448, 3744, 6912],
            "K = 144、S = 4：N = 144 × {{1, 2, 4, 8, 11, 12, 13, 14, 17, 26, 48}}"
        );
        assert_eq!(
            ladder_of_file_sizes(1, 4),
            vec![1, 2, 4, 8, 11, 12, 13, 14, 17, 26, 48, 96],
            "K = 1、S = 4 另加 8 × 3S"
        );
        assert_eq!(delete_publish_count(144, 144), 2, "意图那次加一批");
        assert_eq!(delete_publish_count(145, 144), 3, "多一个单元多一批");
        assert_eq!(delete_publish_count(1, 1), 2, "一个单元、K = 1");
    }

    thread_local! {
        /// 假的谷值钩子存放处：代替副本里 `allocator::FREE_SLOTS_AT_ROOT_RECORDING_FOR_TESTS`，测试往里推「记根时读到的空闲槽」。
        static FAKE_FREE_SLOTS_RECORDED_AT_EACH_ROOT: std::cell::RefCell<Vec<Vec<u64>>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn take_the_fake_free_slots_recorded_at_each_root() -> Vec<Vec<u64>> {
        FAKE_FREE_SLOTS_RECORDED_AT_EACH_ROOT.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
    }

    fn record_a_fake_root(free_slots_of_each_device: [u64; 2]) {
        FAKE_FREE_SLOTS_RECORDED_AT_EACH_ROOT
            .with(|cell| cell.borrow_mut().push(free_slots_of_each_device.to_vec()));
    }

    #[test]
    fn valley_hook_drops_the_roots_recorded_before_the_delete_and_keeps_those_recorded_after() {
        for free_slots in [[40, 40], [30, 30], [3, 3]] {
            record_a_fake_root(free_slots);
        }
        let valley_hook = ValleyHookDuringTheDelete::opened_at_the_start_of_the_delete(
            take_the_fake_free_slots_recorded_at_each_root,
        );
        assert_eq!(
            valley_hook.take_readings_since_the_last_take(),
            Vec::<Vec<u64>>::new(),
            "建池与此前各格记的三条根在开口时丢掉，第一步之前读到 0 条（2026-10-01 第一段停在 P2-S4、readings=4035 就是没丢）"
        );
        record_a_fake_root([2, 2]);
        assert_eq!(
            valley_hook.take_readings_since_the_last_take(),
            vec![vec![2, 2]],
            "开口之后记的根照取：删除各步之间多出来的读数仍会停机"
        );
        assert_eq!(
            valley_hook.take_readings_since_the_last_take(),
            Vec::<Vec<u64>>::new(),
            "取走之后清空"
        );
    }

    /// 空闲图的几个计数：(已分配, 空闲, 待释放, 空闲段数, 全空的聚簇段数, 没挡的槽数)。
    fn counts_of_the_device_free_map(map: &DeviceFreeMap) -> [u64; 6] {
        [
            map.allocated_slots(),
            map.free_slots(),
            map.deferred_slots(),
            map.free_runs(),
            map.empty_segments(),
            map.unblocked_slots(),
        ]
    }

    #[test]
    fn unmarking_the_filled_slots_gives_back_every_count_the_fill_took() {
        let mut device_maps = vec![DeviceFreeMap::new(DeviceIdentity(0), PRODUCT_DEVICE_BYTES)];
        let start_slot = device_maps[0].unit_area_start().slot().0;
        let end_slot = start_slot + device_maps[0].unit_area_slots();
        device_maps[0].mark_allocated(SlotNumber(start_slot), 3);
        device_maps[0].mark_allocated(SlotNumber(end_slot - 70), 1);
        let counts_before_the_fill = counts_of_the_device_free_map(&device_maps[0]);
        let filled_slots = mark_free_slots_from_the_unit_area_end(&mut device_maps[0], 130);
        assert_eq!(filled_slots.len(), 130, "单元区末尾往前有 130 个空闲槽可标");
        assert!(!filled_slots.contains(&(end_slot - 70)), "已经占着的槽跳过");
        assert_eq!(
            device_maps[0].allocated_slots(),
            3 + 1 + 130,
            "压满标掉的槽进了已分配（2026-10-01 第一段 S7 停机：记账比走读多的就是它们）"
        );
        unmark_the_filled_slots(&mut device_maps, std::slice::from_ref(&filled_slots));
        assert_eq!(
            counts_of_the_device_free_map(&device_maps[0]),
            counts_before_the_fill,
            "撤掉压满之后六个计数都回到压满之前"
        );
        assert!(
            filled_slots
                .iter()
                .all(|slot| device_maps[0].is_free(SlotNumber(*slot))),
            "标掉的槽逐个回到空闲"
        );
    }

    #[test]
    fn a_publish_after_the_filled_slots_are_unmarked_leaves_a_pool_the_checker_holds() {
        let PoolWithAFile::Written(mut pool) =
            pool_with_a_file(narrow_geometry(ROOT_RING_SLOTS_PER_REGION_MINIMUM), 1)
        else {
            panic!("P2、S = 4 上写得进一个单元的文件（2026-10-01 第一段在 P2-S4-K1-Lseq-Fend-N-A-N1 上删过）");
        };
        let FillOutcome::WithinOneSlotAtOrBelowZero {
            filled_slots_of_each_device,
            ..
        } = fill_from_the_unit_area_end(&mut pool)
        else {
            panic!("P2、S = 4、一个单元的池压得满（那一格 free_after_fill = 112）");
        };
        assert!(
            filled_slots_of_each_device
                .iter()
                .all(|filled_slots| !filled_slots.is_empty()),
            "两块盘都标了槽"
        );
        unmark_the_filled_slots(
            &mut pool.session.allocator.devices,
            &filled_slots_of_each_device,
        );
        let content = content_needing(1, 3);
        let overwrite = UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS + 120,
        });
        if let Err(refusal) = pool
            .session
            .publish_user_change(&mut pool.devices, overwrite)
        {
            panic!("撤掉压满之后式子可用回到压满之前，覆盖写一个单元放得进：{refusal:?}");
        }
        assert_eq!(
            checker_violations(&pool.image()),
            Vec::<String>::new(),
            "撤掉压满之后再发一次，记账按分配器重写，checker 不判红（不撤就是 S7 那两条 I-3.1、I-3.11）"
        );
    }

    thread_local! {
        /// 假的删除入口用的开关与计数：代替副本里补丁 R11 的 `FORMULA_JUDGEMENT_SKIPPED_FOR_TESTS`、R7 的判数与 R2 的 nlink 开关。
        static FAKE_FORMULA_JUDGEMENT_SKIPPED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        static FAKE_FORMULA_JUDGEMENTS_MADE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
        static FAKE_INODE_WRITTEN_WITH_ZERO_LINKS: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
        /// 假的删除入口每被调一次，记下那一刻 (式子那一判关没关, 写 nlink = 0 的 inode)。
        static FAKE_SWITCHES_SEEN_BY_EACH_UNLINK_BATCH: RefCell<Vec<(bool, Option<u64>)>> = const { RefCell::new(Vec::new()) };
    }

    /// 假的删除入口，照副本里补丁 R6、R7 的样子：式子那一判开着时真判一次并拒（r5 停机那一格删完之后每盘空闲 66 槽、保留量 86 槽，
    /// 式子可用为负），关着时不判、交回上一版（落点那一判不在假入口里）。
    fn fake_publish_unlink_batch(
        _writer: &mut PoolWriter<'_, SparseBlockDevice>,
        _allocator: &mut PoolAllocator,
        previous: &TransactionOutput,
        _kept_data_units: u64,
        _size_after_the_batch: u64,
        _instance: InstanceGeneration,
    ) -> Result<TransactionOutput, PublishError> {
        let is_formula_judgement_skipped =
            FAKE_FORMULA_JUDGEMENT_SKIPPED.with(std::cell::Cell::get);
        let inode_written_with_zero_links =
            FAKE_INODE_WRITTEN_WITH_ZERO_LINKS.with(std::cell::Cell::get);
        FAKE_SWITCHES_SEEN_BY_EACH_UNLINK_BATCH.with(|cell| {
            cell.borrow_mut()
                .push((is_formula_judgement_skipped, inode_written_with_zero_links));
        });
        if is_formula_judgement_skipped {
            Ok(previous.clone())
        } else {
            FAKE_FORMULA_JUDGEMENTS_MADE.with(|cell| cell.set(cell.get() + 1));
            Err(PublishError::SpaceAdmissionRefused(
                singlefs_core::admission::AdmissionRefusedOnSomeDevices {
                    short_devices: Vec::new(),
                },
            ))
        }
    }

    fn fake_unlink_batch_change(
        _kept_data_units: u64,
        _size_after_the_batch: u64,
    ) -> UserChange<'static> {
        panic!("S7 那一次直发（A 臂的发法），不经会话");
    }

    fn fake_set_formula_judgement_skipped(is_skipped: bool) {
        FAKE_FORMULA_JUDGEMENT_SKIPPED.with(|cell| cell.set(is_skipped));
    }

    fn fake_formula_judgements_made() -> u64 {
        FAKE_FORMULA_JUDGEMENTS_MADE.with(std::cell::Cell::get)
    }

    fn fake_set_inode_written_with_zero_links(inode: Option<u64>) {
        FAKE_INODE_WRITTEN_WITH_ZERO_LINKS.with(|cell| cell.set(inode));
    }

    fn fake_set_reclaim_switched_off(_is_switched_off: bool) {}

    fn fake_allocation_record_tree_positions(
        _version: &TransactionOutput,
        _allocator: &PoolAllocator,
    ) -> u64 {
        panic!("S7 那一次发布不读 K1");
    }

    fn fake_delete_path_hooks() -> DeletePathHooks {
        DeletePathHooks {
            publish_unlink_batch: fake_publish_unlink_batch,
            unlink_batch_change: fake_unlink_batch_change,
            set_inode_written_with_zero_links: fake_set_inode_written_with_zero_links,
            set_formula_judgement_skipped: fake_set_formula_judgement_skipped,
            formula_judgements_made: fake_formula_judgements_made,
            set_reclaim_switched_off: fake_set_reclaim_switched_off,
            take_free_slots_recorded_at_each_root: take_the_fake_free_slots_recorded_at_each_root,
            set_ledger: leave_the_ledger_as_it_is,
            allocation_record_tree_positions: fake_allocation_record_tree_positions,
        }
    }

    #[test]
    fn the_s7_ledger_rewrite_is_published_with_the_formula_judgement_skipped_like_every_delete_publish(
    ) {
        let PoolWithAFile::Written(mut pool) =
            pool_with_a_file(narrow_geometry(ROOT_RING_SLOTS_PER_REGION_MINIMUM), 1)
        else {
            panic!("P2、S = 4 上写得进一个单元的文件");
        };
        let hooks = fake_delete_path_hooks();
        let (ledger_rewrite, formula_judgements) =
            publish_the_ledger_rewrite_as_a_delete_publish(&mut pool, &hooks);
        assert_eq!(
            FAKE_SWITCHES_SEEN_BY_EACH_UNLINK_BATCH
                .with(|cell| std::mem::take(&mut *cell.borrow_mut())),
            vec![(true, Some(1))],
            "S7 那一次删除发布只发一次，发的那一刻式子那一判关着、inode 1 写 nlink = 0（登记 5.1 第 4 步：删除类发布不判式子那一判）"
        );
        assert_eq!(formula_judgements, 0, "其间式子那一判一次都没判");
        assert_eq!(
            ledger_rewrite.refusal, None,
            "式子那一判关着就不被它拒（2026-10-02 第一段 r5 那一行 `refusal=landing filled_slots=[0, 0]` 是式子那一判开着拒的，被记成了落点）"
        );
        assert!(
            !FAKE_FORMULA_JUDGEMENT_SKIPPED.with(std::cell::Cell::get),
            "发完把式子那一判开回来"
        );
        assert_eq!(
            FAKE_INODE_WRITTEN_WITH_ZERO_LINKS.with(std::cell::Cell::get),
            None,
            "发完 nlink 开关恢复"
        );
    }

    /// 在这条线程上攒行跑 `run`（与切片执行器给一片开的攒行同一个缓冲）：交回攒下的行与停机行（跑完是 None）。
    fn lines_and_stop_line_of(run: impl FnOnce()) -> (Vec<String>, Option<String>) {
        silence_the_panic_hook_for_stops_inside_slices();
        LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| *cell.borrow_mut() = Some(Vec::new()));
        let result = catch_unwind(AssertUnwindSafe(run));
        let lines = LINES_OF_THE_SLICE_ON_THIS_THREAD
            .with(|cell| cell.borrow_mut().take())
            .expect("开跑时放进去的缓冲");
        let stop_line = match result {
            Ok(()) => None,
            Err(payload) => match payload.downcast::<StopInsideASlice>() {
                Ok(stop) => Some(stop.stop_line),
                Err(payload) => resume_unwind(payload),
            },
        };
        (lines, stop_line)
    }

    fn describe_a_fake_slice(slice: &u64) -> SliceDescription {
        SliceDescription {
            label: format!("fake-{slice}"),
            is_large: false,
            start_rank: 0,
        }
    }

    /// 六片假片按 `scheduling` 跑：第 k 片睡 60 × (5 − k) 毫秒（前面的片后跑完）、打两行、交回 10k；`stopping_slice` 那一片打完两行停机。
    fn fake_slices_on(
        scheduling: SliceScheduling,
        stopping_slice: Option<u64>,
    ) -> (Vec<String>, Option<String>, Vec<u64>) {
        let slices: Vec<u64> = (0..6).collect();
        let mut values = Vec::new();
        let (lines, stop_line) = lines_and_stop_line_of(|| {
            values = run_slices_in_order(
                &slices,
                &describe_a_fake_slice,
                scheduling,
                &|slice: &u64| {
                    std::thread::sleep(Duration::from_millis(60 * (5 - *slice)));
                    emit_product_line!("E name=fake slice={slice} line=first");
                    emit_product_line!("E name=fake slice={slice} line=second");
                    if stopping_slice == Some(*slice) {
                        stop_with("fake_stop", &format!("slice={slice}"));
                    }
                    *slice * 10
                },
                &|_progress_line: &str| {},
                PROGRESS_INTERVAL,
            );
        });
        (lines, stop_line, values)
    }

    #[test]
    fn slices_on_four_threads_print_the_lines_of_one_thread_in_slice_order_and_stop_at_the_first_stopping_slice(
    ) {
        let one_thread = SliceScheduling {
            thread_count: 1,
            large_slice_thread_count: 1,
        };
        let four_threads = SliceScheduling {
            thread_count: 4,
            large_slice_thread_count: 1,
        };
        let expected_lines = |slice_count: u64| -> Vec<String> {
            (0..slice_count)
                .flat_map(|slice| {
                    [
                        format!("E name=fake slice={slice} line=first"),
                        format!("E name=fake slice={slice} line=second"),
                    ]
                })
                .collect()
        };
        for scheduling in [one_thread, four_threads] {
            assert_eq!(
                fake_slices_on(scheduling, None),
                (expected_lines(6), None, vec![0, 10, 20, 30, 40, 50]),
                "{scheduling:?}：六片的行按片的次序整片打出（后开跑的片先跑完也排在后面），值按片的次序"
            );
            assert_eq!(
                fake_slices_on(scheduling, Some(4)),
                (
                    expected_lines(5),
                    Some("E name=stop reason=fake_stop slice=4".to_owned()),
                    Vec::new()
                ),
                "{scheduling:?}：第 4 片停机，打完第 0–4 片的行再交停机行，第 5 片的行不打（单线程跑到同一处打出的就是这些）"
            );
        }
    }

    #[test]
    fn thread_count_reads_e168_threads_first_then_the_shared_cap_then_the_available_parallelism() {
        assert_eq!(
            thread_count_from(Some("3"), Some("8"), 16),
            3,
            "E168_THREADS 先读"
        );
        assert_eq!(
            thread_count_from(None, Some("8"), 16),
            8,
            "没设 E168_THREADS 读 SINGLEFS_THREAD_CAP"
        );
        assert_eq!(
            thread_count_from(None, None, 16),
            16,
            "都没设取 available_parallelism"
        );
        assert_eq!(large_slice_thread_count_from(None), 1, "大片上限没设取 1");
        assert_eq!(
            large_slice_thread_count_from(Some("2")),
            2,
            "大片上限照设的取"
        );
    }

    #[test]
    fn large_slices_run_at_most_their_own_thread_cap_and_leave_a_thread_for_the_rest() {
        let description = |is_large: bool, start_rank: u64| SliceDescription {
            label: String::new(),
            is_large,
            start_rank,
        };
        let descriptions = [
            description(false, 5),
            description(true, 10),
            description(false, 9),
            description(true, 30),
            description(true, 10),
            description(false, 1),
        ];
        let pools_of = |thread_count: usize, large_slice_thread_count: usize| {
            worker_pools(
                &descriptions,
                SliceScheduling {
                    thread_count,
                    large_slice_thread_count,
                },
            )
        };
        let pool = |positions_in_start_order: Vec<usize>, thread_count: usize| WorkerPool {
            positions_in_start_order,
            thread_count,
        };
        assert_eq!(
            pools_of(8, 1),
            vec![pool(vec![3, 1, 4], 1), pool(vec![2, 0, 5], 3)],
            "大片（P4）同时只跑 1 片（E168_LARGE_SLICE_THREADS 没设），按删除发布数从大到小开跑，同值按片号"
        );
        assert_eq!(
            pools_of(3, 8),
            vec![pool(vec![3, 1, 4], 2), pool(vec![2, 0, 5], 1)],
            "大片上限比线程数大时，给其余片留一条线程"
        );
        assert_eq!(
            pools_of(1, 1),
            vec![pool(vec![0, 1, 2, 3, 4, 5], 1)],
            "一条线程：一个池、按片的次序（与改并行之前同一个次序）"
        );
    }

    /// 单测不经补丁的设账入口：什么都不做（账恒为第一版的两个 0，即 N 臂的账）。
    fn leave_the_ledger_as_it_is(_ledger: Option<(u64, u64)>) {}

    const LEDGER_OF_THE_FIRST_VERSION: LedgerForTheFill = LedgerForTheFill {
        ledger: None,
        set_ledger: leave_the_ledger_as_it_is,
    };

    #[test]
    fn smallest_usable_disk_under_reading_c_fills_while_the_steps_one_and_two_disk_and_the_disk_two_slots_smaller_do_not(
    ) {
        for slots_per_region in [
            ROOT_RING_SLOTS_PER_REGION_MINIMUM,
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        ] {
            let search = smallest_usable_geometry(slots_per_region, 1, LEDGER_OF_THE_FIRST_VERSION);
            let written = search
                .written
                .expect("一个单元的文件在原定义下找得到最小可用盘");
            let fillable = search
                .written_and_fillable
                .expect("读法 C 的最小盘在搜索上沿之内找得到");
            assert!(
                free_slots_after_the_fill(fillable, 1, LEDGER_OF_THE_FIRST_VERSION).is_some(),
                "S = {slots_per_region}：读法 C 找到的盘（单元区 {} 槽）第 3 步压得满",
                fillable.unit_area_slots()
            );
            assert!(
                !pool_is_written_and_fillable(written, 1, LEDGER_OF_THE_FIRST_VERSION),
                "S = {slots_per_region}：原定义的 P1（单元区 {} 槽）压不满（2026-10-01 第一段 r4 原定义下 P1 的 40 格全 not_fillable）——读法 C 真把盘往大挪了",
                written.unit_area_slots()
            );
            let two_slots_smaller = smallest_usable_candidate(
                slots_per_region,
                fillable.journal_ring_bytes,
                fillable.unit_area_slots() - 2,
            );
            assert!(
                !pool_is_written_and_fillable(two_slots_smaller, 1, LEDGER_OF_THE_FIRST_VERSION),
                "S = {slots_per_region}：小 2 槽那一格第 1–3 步做不成（登记 5.7「找到之后再核小 2 槽那一格做不成」）"
            );
        }
    }

    #[test]
    fn p1_journal_ring_for_a_file_of_88_data_units_is_two_mebibytes_and_one_mebibyte_refuses_the_write(
    ) {
        for (records_of_the_publish, expected_ring_bytes) in [
            (0, 1_048_576),
            (1, 1_048_576),
            (85, 1_048_576),
            (86, 2_097_152),
            (88, 2_097_152),
            (170, 2_097_152),
            (171, 3_145_728),
        ] {
            assert_eq!(
                smallest_legal_journal_ring_holding(records_of_the_publish),
                expected_ring_bytes,
                "{records_of_the_publish} 条记录：1 MiB 的上限 ⌊1048576 ÷ 12288⌋ = 85、2 MiB 的 170，环按 1 MiB 一档取装得下的最小一档、不小于 1 MiB（修订第 15 条）"
            );
        }
        for journal_ring_bytes in [1 << 20, 2 << 20, 3 << 20, JOURNAL_RING_DEFAULT_BYTES] {
            assert_eq!(
                journal_record_limit_by_the_format(journal_ring_bytes),
                singlefs_core::system_configuration::journal_in_flight_record_limit(
                    journal_ring_bytes
                ),
                "本地的记录数上限与 core 的 journal_in_flight_record_limit 在环 {journal_ring_bytes} 字节上相同"
            );
        }
        let ring = journal_ring_of_the_smallest_usable_disk(
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            88,
            JOURNAL_RING_ENLARGEMENTS_AT_MOST,
        );
        assert_eq!(
            ring,
            JournalRingOfTheSmallestUsableDisk {
                journal_ring_bytes: 2_097_152,
                tried: vec![
                    (1_048_576, RingProbeOutcome::RecordsOverTheLimit(88)),
                    (2_097_152, RingProbeOutcome::Written),
                ],
                not_settled: None,
            },
            "P1-S16、N = 88：1 MiB 的环上那次顺序写切 88 条记录、被上限 85 拒（第九个执行员 probe2.out 那几行），换成 2 MiB 写得进"
        );
        let search = smallest_usable_geometry(
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            88,
            LEDGER_OF_THE_FIRST_VERSION,
        );
        assert_eq!(
            search
                .written_and_fillable
                .map(|geometry| geometry.journal_ring_bytes),
            Some(2_097_152),
            "读法 C 的 P1 在 2 MiB 的环上找得到"
        );
        assert_eq!(
            search.smaller_journal_ring_refused_by_the_record_limit,
            Some(true),
            "小一档（1 MiB）的环在同一单元区上被记录数上限拒：2 MiB 是最小合法环"
        );
        assert_eq!(search.not_found_reason, None);
    }

    #[test]
    fn smallest_usable_line_names_the_journal_record_limit_when_the_ring_may_not_grow() {
        let search = smallest_usable_geometry_with_the_ring_enlarged_at_most(
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            88,
            LEDGER_OF_THE_FIRST_VERSION,
            0,
        );
        assert_eq!(
            search.not_found_reason,
            Some(
                SmallestUsableNotFoundReason::JournalRecordsOfOnePublishExceedTheLimit(
                    JournalRecordsOverTheLimit {
                        records_of_the_publish: 88,
                        record_limit: 85,
                    }
                )
            ),
            "环不许放大（改之前的形态）时 N = 88 找不到盘，原因是记录数上限，不是盘的大小"
        );
        let line = smallest_usable_line(ROOT_RING_SLOTS_PER_REGION_MAXIMUM, 88, &search);
        for expected_field in [
            "unit_area_slots=not_found",
            "above_the_narrow_point=true",
            "journal_ring_bytes=1048576",
            "smaller_journal_ring_refused_by_the_record_limit=not_checked",
            "not_found_reason=journal_records_of_one_publish_exceed_the_limit",
        ] {
            assert!(
                line.split(' ').any(|field| field == expected_field),
                "`E name=smallest_usable` 那一行有 `{expected_field}`：{line}"
            );
        }
    }

    #[test]
    fn positive_control_reads_free_0_on_the_disk_it_fills_with_the_n_unit_file() {
        let geometry_for = |data_units: u64| {
            Some(smallest_usable_candidate(
                ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
                SMALL_JOURNAL_RING_BYTES,
                64 + 2 * (data_units / 10),
            ))
        };
        let free_after_the_fill =
            |geometry: Geometry, _data_units: u64| Some(geometry.unit_area_slots() / 4 + 30);
        let PositiveControlFileSize {
            geometry,
            data_units,
            free_after_the_fill_slots,
        } = positive_control_file_size_on_the_filled_disk(&geometry_for, &free_after_the_fill)
            .expect("盘随 N 涨得慢，迭代收敛");
        assert_eq!(
            (geometry.unit_area_slots(), data_units, free_after_the_fill_slots),
            (74, 50, 48),
            "N = 1 的盘（64 槽）读到 46 ⇒ N = 48 的盘（72 槽）读到 48 ⇒ N = 50 的盘（74 槽）读到 48 = N − 2，停；照 N = 1 那块盘的 46 取 N = 48 就是改之前的读法"
        );
        assert_eq!(
            free_after_the_fill(geometry_for(data_units).expect("有盘"), data_units),
            Some(data_units - 2),
            "free_0 读的是写进 N 个单元、压满之后那块盘本身（登记 5.5）"
        );
        let never_settles = |_geometry: Geometry, file_data_units: u64| Some(file_data_units);
        assert_eq!(
            positive_control_file_size_on_the_filled_disk(&geometry_for, &never_settles)
                .expect_err("N 每次多 2，不收敛")
                .len(),
            POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST,
            "不收敛时迭代满上限交回迭代过的每一次"
        );
    }

    #[test]
    #[ignore = "要补丁表副本：主工作区那份的 delete_path_hooks 是桩，一调就打停机行退 3；在套过补丁表的副本里 `-- --ignored` 跑"]
    fn segment_one_on_p2_s4_prints_the_same_lines_on_one_thread_and_on_eight_threads() {
        let lines_and_stop_line_on = |thread_count: usize| {
            lines_and_stop_line_of(|| {
                run_segment_one(
                    Some("P2-S4"),
                    SliceScheduling {
                        thread_count,
                        large_slice_thread_count: 1,
                    },
                );
            })
        };
        let on_one_thread = lines_and_stop_line_on(1);
        assert!(
            on_one_thread
                .0
                .iter()
                .any(|line| line.starts_with("E name=group point=P2 S=4 ")),
            "P2-S4 的梯子组跑到了组行（不是一开头就停）"
        );
        assert_eq!(
            lines_and_stop_line_on(8),
            on_one_thread,
            "八条线程与一条线程打出的行逐行相同、停机行（有的话）相同（登记第十二节修订第 12 条：只改调度、不改判据）"
        );
    }

    #[test]
    fn valley_keeps_the_roots_recorded_on_the_session_allocator_and_drops_those_the_floor_raise_rehearsal_recorded_on_a_copy(
    ) {
        let free_on_both_devices = |free_slots: u64| vec![free_slots, free_slots];
        assert_eq!(
            hook_readings_on_the_allocator_of_the_session(
                vec![free_on_both_devices(75)],
                &FloorRaisesOfTheStep::none(),
                true
            ),
            Some(vec![free_on_both_devices(75)]),
            "没推抬 F、发成的一次：钩子读到的那一条就是会话分配器上的"
        );
        let refused_at_the_second_rehearsed_publish = FloorRaisesOfTheStep {
            publishes_of_each_landed_chain: Vec::new(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 1,
            stop_label: "floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2"
                .to_owned(),
        };
        assert_eq!(
            hook_readings_on_the_allocator_of_the_session(
                vec![free_on_both_devices(3)],
                &refused_at_the_second_rehearsed_publish,
                false
            ),
            Some(Vec::new()),
            "r7 冒烟 P1-S16-K1-Lseq-Fend-N-B-N88-noreclaim 的 t = 7：抬 F 那一串在预演第 2 次被拒，钩子读到的 3 槽是拷贝上第 1 次之后的，\
             不进谷值（谷值退回发布之前的 9 槽，borrow 是 86 − 9 = 77，不是 86 − 3 = 83）"
        );
        let one_landed_chain_of_two = FloorRaisesOfTheStep {
            publishes_of_each_landed_chain: vec![2],
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
            stop_label: "none".to_owned(),
        };
        assert_eq!(
            hook_readings_on_the_allocator_of_the_session(
                vec![
                    free_on_both_devices(40),
                    free_on_both_devices(34),
                    free_on_both_devices(41),
                    free_on_both_devices(35),
                    free_on_both_devices(24),
                ],
                &one_landed_chain_of_two,
                true
            ),
            Some(vec![
                free_on_both_devices(41),
                free_on_both_devices(35),
                free_on_both_devices(24)
            ]),
            "落了盘的一串两次：丢掉预演在拷贝上的前两条，留下真发的两条与发成的那一次"
        );
        let landed_then_refused_in_the_rehearsal = FloorRaisesOfTheStep {
            publishes_of_each_landed_chain: vec![2],
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 1,
            stop_label: "floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2"
                .to_owned(),
        };
        assert_eq!(
            hook_readings_on_the_allocator_of_the_session(
                vec![
                    free_on_both_devices(20),
                    free_on_both_devices(14),
                    free_on_both_devices(21),
                    free_on_both_devices(15),
                    free_on_both_devices(9),
                ],
                &landed_then_refused_in_the_rehearsal,
                false
            ),
            Some(vec![free_on_both_devices(21), free_on_both_devices(15)]),
            "落了一串、再推的一串在预演里被拒：只留第一串真发的两条"
        );
        assert_eq!(
            hook_readings_on_the_allocator_of_the_session(
                vec![free_on_both_devices(3), free_on_both_devices(3)],
                &refused_at_the_second_rehearsed_publish,
                false
            ),
            None,
            "钩子多读了一条：这一套账对不上，交回 None（调用方停机）"
        );
        assert_eq!(
            hook_readings_on_the_allocator_of_the_session(
                Vec::new(),
                &FloorRaisesOfTheStep::none(),
                true
            ),
            None,
            "发成而钩子一条都没读到：对不上"
        );
    }

    #[test]
    fn floor_raises_of_a_refused_step_count_the_rehearsal_roots_and_refuse_a_chain_that_landed_outside_the_refusal(
    ) {
        use singlefs_core::address::CheckpointTxg;
        use singlefs_core::mount::PublishSequenceFailed;
        use singlefs_core::write_accounting::WritesByStructureKind;
        let some_publish_error = || PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
            checkpoint_txg: CheckpointTxg(9),
            instance: InstanceGeneration(1),
        };
        assert_eq!(
            FloorRaisesOfTheStep::of_a_refused_step(
                &[],
                &FloorRaiseStop::FloorAlreadyAtTheCeiling {
                    floor: CheckpointTxg(5),
                    ceiling: CheckpointTxg(5)
                }
            ),
            Ok(FloorRaisesOfTheStep {
                publishes_of_each_landed_chain: Vec::new(),
                rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
                stop_label: "floor_already_at_the_ceiling".to_owned(),
            })
        );
        assert_eq!(
            FloorRaisesOfTheStep::of_a_refused_step(
                &[],
                &FloorRaiseStop::FloorRaiseRefusedForSpace {
                    refusal: Box::new(
                        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                            refused_publish_in_the_sequence: 2,
                            publishes_in_the_sequence: 2,
                            cause: some_publish_error(),
                        }
                    )
                }
            ),
            Ok(FloorRaisesOfTheStep {
                publishes_of_each_landed_chain: Vec::new(),
                rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 1,
                stop_label: "floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2"
                    .to_owned(),
            }),
            "预演第 2 次被拒：拷贝上记过 1 条根"
        );
        let landed_one_then_failed = FloorRaiseStop::FloorRaiseRefusedForSpace {
            refusal: Box::new(MountError::RaiseFloorSequencePublishFailed(Box::new(
                PublishSequenceFailed {
                    cause: some_publish_error(),
                    writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
                    writes_of_persisted_publishes: vec![WritesByStructureKind::NOTHING_WRITTEN],
                    writes_of_failed_publishes: Vec::new(),
                },
            ))),
        };
        assert!(
            FloorRaisesOfTheStep::of_a_refused_step(&[], &landed_one_then_failed).is_err(),
            "真发途中失败：落了盘的那一次不在 floor_raises 里，交回 Err（调用方停机），不当成没推"
        );
    }

    #[test]
    fn positive_control_u12_holds_for_a_session_refusal_whose_floor_raise_was_refused_in_the_rehearsal_and_not_for_a_landing_refusal(
    ) {
        let like_the_r7_session_cell_with_no_chain_landed = PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::ThroughTheSession,
            Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor),
        );
        assert_eq!(
            like_the_r7_session_cell_with_no_chain_landed,
            PositiveControlCellVerdict {
                is_refused: true,
                is_refused_with_no_space_after_raising_the_floor: Some(true),
            },
            "r7 冒烟 P1-S16 N-B 那一格：会话推了抬 F、那一串在预演第 2 次被拒、零落盘（floor_raise_publishes=0），拒绝种类 no_space_after_raising_the_floor；读法乙下 U12 成立"
        );
        assert!(like_the_r7_session_cell_with_no_chain_landed.holds());
        assert_eq!(
            like_the_r7_session_cell_with_no_chain_landed.u12_label(),
            "true"
        );
        for refusal_without_the_session in [DeleteRefusal::Landing, DeleteRefusal::Placement] {
            let session_arm_refused_without_the_session = PositiveControlCellVerdict::of_the_cell(
                RefusalHandling::ThroughTheSession,
                Some(refusal_without_the_session),
            );
            assert_eq!(
                session_arm_refused_without_the_session,
                PositiveControlCellVerdict {
                    is_refused: true,
                    is_refused_with_no_space_after_raising_the_floor: Some(false),
                },
                "B 臂不经会话（变异 U12）：拒绝种类是 {}，U12 不成立",
                refusal_without_the_session.label()
            );
            assert!(!session_arm_refused_without_the_session.holds());
            assert_eq!(session_arm_refused_without_the_session.u12_label(), "false");
        }
        let session_arm_not_refused =
            PositiveControlCellVerdict::of_the_cell(RefusalHandling::ThroughTheSession, None);
        assert_eq!(
            session_arm_not_refused,
            PositiveControlCellVerdict {
                is_refused: false,
                is_refused_with_no_space_after_raising_the_floor: Some(false),
            }
        );
        assert!(!session_arm_not_refused.holds());
        let direct_refused = PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::Direct,
            Some(DeleteRefusal::Landing),
        );
        assert_eq!(
            direct_refused,
            PositiveControlCellVerdict {
                is_refused: true,
                is_refused_with_no_space_after_raising_the_floor: None,
            }
        );
        assert!(direct_refused.holds());
        assert_eq!(direct_refused.u12_label(), "not_applicable");
        let direct_not_refused =
            PositiveControlCellVerdict::of_the_cell(RefusalHandling::Direct, None);
        assert!(!direct_not_refused.holds());
        assert_eq!(direct_not_refused.u12_label(), "not_applicable");
    }

    #[test]
    fn positive_control_verdict_counts_the_refusal_and_the_session_floor_raise_refusal_apart_and_stops_on_either(
    ) {
        let direct_refused = PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::Direct,
            Some(DeleteRefusal::Landing),
        );
        let direct_not_refused =
            PositiveControlCellVerdict::of_the_cell(RefusalHandling::Direct, None);
        let session_refused_after_raising_the_floor = PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::ThroughTheSession,
            Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor),
        );
        let session_refused_by_the_landing = PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::ThroughTheSession,
            Some(DeleteRefusal::Landing),
        );
        let session_not_refused =
            PositiveControlCellVerdict::of_the_cell(RefusalHandling::ThroughTheSession, None);
        let like_the_r7_smoke = PositiveControlTally::of_the_cells(&[
            direct_refused,
            session_refused_after_raising_the_floor,
            direct_refused,
            session_refused_after_raising_the_floor,
        ]);
        assert_eq!(
            like_the_r7_smoke,
            PositiveControlTally {
                cells: 4,
                cells_without_refusal: 0,
                session_cells_refused_other_than_no_space_after_raising_the_floor: 0,
                cells_not_holding: 0,
            },
            "r7 冒烟那四格（N-A、N-B、W-A、W-B）：四格都出了拒绝，两条 B 臂的拒绝都是 no_space_after_raising_the_floor（N-B 零落盘），读法乙下都成立"
        );
        assert_eq!(like_the_r7_smoke.stop_reason(), None);
        assert_eq!(
            lines_and_stop_line_of(|| stop_when_a_positive_control_does_not_hold(
                like_the_r7_smoke
            )),
            (Vec::new(), None),
            "四格都合：不停"
        );
        let session_arm_without_the_session = PositiveControlTally::of_the_cells(&[
            direct_refused,
            session_refused_by_the_landing,
            direct_refused,
            session_refused_after_raising_the_floor,
        ]);
        assert_eq!(
            session_arm_without_the_session,
            PositiveControlTally {
                cells: 4,
                cells_without_refusal: 0,
                session_cells_refused_other_than_no_space_after_raising_the_floor: 1,
                cells_not_holding: 1,
            },
            "一条 B 臂被落点那一判直接拒（不经会话）：四格都出了拒绝，U12 那个数是 1"
        );
        assert_eq!(
            session_arm_without_the_session.verdict_fields(),
            "positive_control_cells=4 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=1 positive_control_cells_not_holding=1",
            "不出拒绝的格数报在 without_refusal 下，U12 不合的格数报在自己的名字下"
        );
        assert_eq!(
            session_arm_without_the_session.stop_reason(),
            Some("u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor")
        );
        let (lines, stop_line) = lines_and_stop_line_of(|| {
            stop_when_a_positive_control_does_not_hold(session_arm_without_the_session)
        });
        assert!(lines.is_empty());
        assert_eq!(
            stop_line.as_deref(),
            Some("E name=stop reason=u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor positive_control_cells=4 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=1 positive_control_cells_not_holding=1"),
            "有一格不合就打停机行（退 4），不当成跑完"
        );
        let none_refused =
            PositiveControlTally::of_the_cells(&[direct_not_refused, session_not_refused]);
        assert_eq!(
            none_refused,
            PositiveControlTally {
                cells: 2,
                cells_without_refusal: 2,
                session_cells_refused_other_than_no_space_after_raising_the_floor: 0,
                cells_not_holding: 2,
            },
            "B 臂没出拒绝只算进 without_refusal（V1），不算进 U12 那个数"
        );
        assert_eq!(
            none_refused.stop_reason(),
            Some("v1_positive_control_cells_without_refusal"),
            "不出拒绝的格先按 V1 停"
        );
        let all_holding = PositiveControlTally::of_the_cells(&[
            direct_refused,
            session_refused_after_raising_the_floor,
            direct_refused,
            session_refused_after_raising_the_floor,
            direct_refused,
            session_refused_after_raising_the_floor,
            direct_refused,
            session_refused_after_raising_the_floor,
        ]);
        assert_eq!(
            all_holding.verdict_fields(),
            "positive_control_cells=8 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=0 positive_control_cells_not_holding=0",
            "八格都合：r8 脚本第六步按这一串判"
        );
        assert_eq!(
            lines_and_stop_line_of(|| stop_when_a_positive_control_does_not_hold(all_holding)),
            (Vec::new(), None),
            "八格都合：不停"
        );
    }
}
