//! E207（严格判据下的扫类步数）的历史用例：拷进副本当 `crates/singlefs-verification/tests/e207_strict_scan_steps_histories.rs`，
//! 副本由 `e207_strict_scan_steps_counters.py` 套上读数（`--build counts`）或一个字节不插（`--build plain`，只比盘）。
//! 读数行怎么打在副本的 `tests/e207_readings/mod.rs`（两种构建各一份，都由插计数脚本写）；第 3 组声明行（`name=declared`）由这里按驱动自己的单元记账打。
//! 量窗的取法（声明行的式子与写法、W2 冷文件池、探针前缀、扩展窗口、阴性对照的点）在入口一并拷进副本的 `tests/e207_window_choices/mod.rs`，
//! 源文件是 `scripts/experiment-apparatus/e207_strict_scan_steps_window_choices.rs`（那里另编成 bin 跑单测与变异）。
//!
//! 一个 `<点>__<臂>`（探针项 `<点>__<臂>__<探针>`）一条用例（跑前登记第五节「取样点」「量窗」）；同一段历史上连着量，后一步读前一步留下的那一版，
//! 次序效应（`first` 对 `later`）本身就是被量的，所以历史各步、量窗 W1–W7、两种挂载状态串行、不拆。臂由环境变量 `E207_ARM`、探针由 `E207_PROBE`
//! 定（进程开头读一次），用例核它们与自己点名的相同。历史那一段每过进度间隔打一行 `e207 progress`，判结果只读 `E207RESULT` 行。
//! 弄坏开关（只给入口的 `--break-history` 用，平常跑时入口不设）：`E207_RUN_BREAK=units_of_not_refreshed` 覆盖写之后单元记账不刷新成 1（登记第九节 M13）；
//! `slots_placed_undercounted` 对账行打的 `slots_placed` 减 1、`image_witness_flipped` 只在 `plain` 构建里把 `remounted` W2 第 5 次那一行的镜像翻一位再取哈希（登记修订 3 的阳性对照 ① ②）。
//! 内存：单元区里的扇区按内容只存一份（`e207_compact_device/mod.rs`，源文件 `e207_strict_scan_steps_compact_device.rs`），要取镜像的点（`IMAGE_POINTS`、`PER_OPERATION_IMAGE_POINTS`）仍用整块稀疏盘；
//! 每片在几个时点往标准错误打一行 `e207 memory …`（进程常驻与峰值、写过的扇区数、其中不同内容的份数），不进产物。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]
#![allow(non_snake_case, reason = "用例名是 item_<点>__<臂>，入口按这个名字单跑")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;
#[path = "e207_readings/mod.rs"]
mod readings;
/// 量窗的取法（声明行、冷文件池、探针前缀、扩展窗口、阴性对照）：入口把 `scripts/experiment-apparatus/e207_strict_scan_steps_window_choices.rs` 拷到这里，单测与变异在那一份上跑。
#[path = "e207_window_choices/mod.rs"]
mod choices;
/// 紧凑盘：单元区里内容相同的扇区只存一份（本实验大点的内存办法，文件头）。
#[path = "e207_compact_device/mod.rs"]
mod compact_device;

use common_admission_pool::{content_of, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES};
use compact_device::CompactDevice;
use singlefs_core::address::{DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::transaction::PartialWriteBytes;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_verification::scenario::e142_parameters;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

/// 6 MiB 环：单元区从槽 1408 起（同 E203（准入之前扫类的步代价））。
const JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 一次写至多切几个数据单元（同 E203）。
const DATA_UNITS_PER_WRITE_AT_MOST: u64 = 400;
/// 建 inode 一批至多这么多个（同 E203）。
const INODES_PER_BATCH: u64 = 100;
/// 进度间隔（`.claude/rules/implementation-workflow.md`「进度间隔」）：只在历史那一段打。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(300);
/// `churn_random_*` 的随机源种子，打进产物。
const CHURN_SEED: u64 = 1;
/// 读法第 5 条的量窗次数。
const FIRST_FILE_OVERWRITES: u64 = 32;
const COLD_FILE_OVERWRITES: u64 = 32;
const CREATE_WRITE_UNLINK_ROUNDS: u64 = 8;
const FIRST_FILE_OVERWRITES_AFTERWARDS: u64 = 8;
/// 历史末取 `image_sha256` 的点（作废条款第 3 条；大点不取，第五节「大点可达核过」③）。
const IMAGE_POINTS: [&str; 2] = ["base", "files_1000"];
/// 每个量窗操作做完之后取镜像哈希（`image_after_the_operation`）、重挂载前后各取一次（`image_at_the_remount`）的点，`head` 与 `plain` 两个构建都打；
/// `head` 构建另在这两点每个量窗操作上打 `reconciliation` 行（盘上变了几个单元槽对计数记的落点槽数；登记修订 3）。取镜像要整块稀疏盘，所以这两点不用紧凑盘。
const PER_OPERATION_IMAGE_POINTS: [&str; 2] = ["base", "files_1000"];
/// 扩展窗口：W5 覆盖写的冷文件数、W6 新建的文件数与每个写的单元数、W6 一批建几个、W7 冷文件与新文件各几个。
const EXTENDED_COLD_OVERWRITES: usize = 200;
const EXTENDED_NEW_FILES: u64 = 100;
const EXTENDED_UNITS_PER_NEW_FILE: u64 = 100;
const EXTENDED_FILES_PER_BATCH: u64 = 10;
const EXTENDED_LAST_PICKS: usize = 5;

type Pool = PoolUnderTest<CompactDevice>;

impl DeviceOverASparseImage for CompactDevice {
    fn sparse_image(&self) -> &SparseDevice {
        CompactDevice::sparse_image(self)
    }
}

/// 这个点要不要用紧凑盘：取镜像的点不用（紧凑盘不交整份稀疏盘），别的点都用。
fn uses_the_compact_device(point_name: &str) -> bool {
    !IMAGE_POINTS.contains(&point_name) && !PER_OPERATION_IMAGE_POINTS.contains(&point_name)
}

/// 进程常驻与峰值（`/proc/self/status` 的 VmRSS、VmHWM，单位 KiB）；读不到交 0。
fn process_memory_kib() -> (u64, u64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let value_of = |key: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(0)
    };
    (value_of("VmRSS:"), value_of("VmHWM:"))
}

/// 一行内存读数（只往标准错误打、不进产物；入口把 `e207 memory` 开头的行也转到任务日志）：这一刻进程的常驻与峰值，两块盘写过的扇区数与其中不同内容的份数。
fn print_memory_line(item: &str, stage: &str, user_changes: u64, pool: &Pool) {
    let (resident, peak) = process_memory_kib();
    let (written_sectors, distinct_contents) = pool.devices.iter().fold((0, 0), |(written, distinct), (_, device)| {
        let stored = device.stored();
        (written + stored.written_sectors, distinct + stored.distinct_contents)
    });
    eprintln!(
        "e207 memory item={item} stage={stage} user_changes={user_changes} vm_rss_kib={resident} vm_hwm_kib={peak} written_sectors={written_sectors} distinct_contents={distinct_contents}"
    );
}

/// 历史怎么走（登记第五节取样点表「形状」一列）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    BigThenSmall,
    SmallThenBig,
    SmallFilesOfUnits(u64),
    BatchOfOne,
    ChurnRandom,
    HotThenCold(u64),
    HalfDeleted,
    AlternateCreateUnlink,
    Snapshots,
    RemountEvery(u64),
}

/// 一个取样点：(U, u, D, F) 加形状，`publishes_about` 只给进度行当分母（推的）。
#[derive(Clone, Copy)]
struct Point {
    unit_area_slots: u64,
    unlinks: u64,
    data_units: u64,
    files: u64,
    shape: Shape,
    publishes_about: u64,
}

const BASE: Point = Point {
    unit_area_slots: 1 << 17,
    unlinks: 10,
    data_units: 10,
    files: 10,
    shape: Shape::BigThenSmall,
    publishes_about: 50,
};

/// 登记第五节「取样点」表，一行一个点。
fn point_named(name: &str) -> Point {
    let power_of_two = |exponent: u32| 1_u64 << exponent;
    let about = |publishes_about: u64| Point { publishes_about, ..BASE };
    let shaped = |shape: Shape, unit_area_slots: u64, files: u64, publishes_about: u64| Point {
        unit_area_slots,
        files,
        shape,
        publishes_about,
        ..BASE
    };
    let units = |data_units_per_file: u64, exponent: u32, files: u64, publishes_about: u64| {
        shaped(Shape::SmallFilesOfUnits(data_units_per_file), power_of_two(exponent), files, publishes_about)
    };
    match name {
        "base" => BASE,
        "files_1000" => Point { files: 1_000, ..about(1_000) },
        "files_10000" => Point { files: 10_000, ..about(10_000) },
        "files_30000" => Point { unit_area_slots: power_of_two(18), files: 30_000, ..about(30_000) },
        "files_1000_units_100" => units(100, 19, 1_000, 1_000),
        "files_56_units_100" => units(100, 19, 56, 134),
        "files_100_units_100" => units(100, 19, 100, 200),
        "files_178_units_100" => units(100, 19, 178, 378),
        "files_316_units_100" => units(100, 19, 316, 654),
        "files_562_units_100" => units(100, 19, 562, 1_146),
        "files_10_units_1000" => units(1_000, 19, 10, 40),
        "files_100_units_1000" => units(1_000, 19, 100, 300),
        "files_10_units_10000" => units(10_000, 19, 10, 250),
        "files_then_big_1000" => Point { data_units: 1_000, ..shaped(Shape::SmallThenBig, power_of_two(17), 1_000, 1_000) },
        "files_then_big_10000" => Point { data_units: 10_000, ..shaped(Shape::SmallThenBig, power_of_two(17), 10_000, 10_000) },
        "files_10_units_1000_unit_area_2_pow_17" => units(1_000, 17, 10, 40),
        "files_100_units_1000_unit_area_2_pow_20" => units(1_000, 20, 100, 300),
        "files_10_units_1000_unit_area_2_pow_21" => units(1_000, 21, 10, 40),
        "files_100_units_1000_unit_area_2_pow_21" => units(1_000, 21, 100, 300),
        "files_1000_units_1000" => units(1_000, 21, 1_000, 4_000),
        other => panic!("取样点表里没有 {other}"),
    }
}

fn shape_tag(shape: Shape) -> &'static str {
    match shape {
        Shape::BigThenSmall => "big_then_small",
        Shape::SmallThenBig => "small_then_big",
        Shape::SmallFilesOfUnits(_) => "small_files_of_units",
        Shape::BatchOfOne => "batch_of_one",
        Shape::ChurnRandom => "churn_random",
        Shape::HotThenCold(_) => "hot_then_cold",
        Shape::HalfDeleted => "half_deleted",
        Shape::AlternateCreateUnlink => "alternate_create_unlink",
        Shape::Snapshots => "snapshots",
        Shape::RemountEvery(_) => "remount_every",
    }
}

fn parameters() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES;
    parameters
}

fn unit_area_start_slot() -> u64 {
    UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES)
        .expect("6 MiB 的环末尾落在聚簇段边界上")
        .slot()
        .0
}

fn next_inode(pool: &Pool) -> InodeNumber {
    InodeNumber(pool.current_file_version().inode_number_watermark())
}

/// 一次被会话交回的拒绝：哪一种改动、会话交回的原样。
struct RefusedChange {
    kind: &'static str,
    refusal: UserChangeRefused,
}

/// `Debug` 文字打头的那个名字（枚举成员名）。
fn variant_name(debug_text: &str) -> &str {
    let end = debug_text
        .find(|character: char| !character.is_ascii_alphanumeric())
        .unwrap_or(debug_text.len());
    &debug_text[..end]
}

/// 拒绝的原因叫什么：发布被拒取里面 `cause` 的成员名，别的取拒绝自己的成员名。
fn refusal_cause_name(refusal: &UserChangeRefused) -> String {
    match refusal {
        UserChangeRefused::Publish { cause, .. } => variant_name(&format!("{cause:?}")).to_string(),
        other => variant_name(&format!("{other:?}")).to_string(),
    }
}

/// 经会话发一次用户改动（被空间拒、推抬 F 由会话自己推，历史第 5 条），记被拒与抬 F 次数。
fn publish_change(
    pool: &mut Pool,
    tally: &mut Tally,
    kind: &'static str,
    change: UserChange<'_>,
) -> Result<(), RefusedChange> {
    tally.changes += 1;
    match pool.publish(change, None) {
        Ok(published) => {
            tally.floor_raises += u64::try_from(published.floor_raises.len()).expect("串数");
            Ok(())
        }
        Err(refusal) => {
            tally.changes_refused += 1;
            Err(RefusedChange { kind, refusal })
        }
    }
}

thread_local! {
    /// inode 号 → 驱动按构造记的数据单元数（读法第 2 条：建时 0，写多少记多少，覆盖写后记新内容的 1 个，unlink 删掉）。
    static UNITS_OF: RefCell<BTreeMap<u64, u64>> = RefCell::new(BTreeMap::from([(1, choices::FIRST_FILE_UNITS)]));
}

/// 驱动记的这个 inode 的数据单元数（没有记 0）。
fn units_of(inode: InodeNumber) -> u64 {
    UNITS_OF.with(|cell| cell.borrow().get(&inode.0).copied().unwrap_or(0))
}

fn set_units_of(inode: InodeNumber, units: Option<u64>) {
    UNITS_OF.with(|cell| {
        let mut map = cell.borrow_mut();
        match units {
            Some(units) => map.insert(inode.0, units),
            None => map.remove(&inode.0),
        };
    });
}

/// 弄坏开关打开了没有（只由入口的 `--break-history` 设；平常不设）。
fn run_break_is(switch: &str) -> bool {
    std::env::var("E207_RUN_BREAK").as_deref() == Ok(switch)
}

/// 弄坏开关 M13：覆盖写之后不刷新成 1（只由入口的 `--break-history` 设）。
fn units_of_is_refreshed_after_an_overwrite() -> bool {
    !run_break_is("units_of_not_refreshed")
}

/// 覆盖写之后的记账：`OVERWRITE_BYTES` 在一个数据单元以内，新内容记 1 个单元。
fn note_an_overwrite(inode: InodeNumber) {
    if units_of_is_refreshed_after_an_overwrite() {
        set_units_of(inode, Some(1));
    }
}

#[derive(Default)]
struct Tally {
    changes: u64,
    changes_refused: u64,
    floor_raises: u64,
}

fn create_inodes(pool: &mut Pool, tally: &mut Tally, count: u64) -> Result<Vec<InodeNumber>, RefusedChange> {
    let first = next_inode(pool).0;
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    publish_change(pool, tally, "new_inodes", UserChange::NewInodes { count, write_time_seconds })?;
    for inode in first..first + count {
        set_units_of(InodeNumber(inode), Some(0));
    }
    Ok((first..first + count).map(InodeNumber).collect())
}

fn overwrite_inode(pool: &mut Pool, tally: &mut Tally, inode: InodeNumber) -> Result<(), RefusedChange> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    publish_change(
        pool,
        tally,
        "overwrite_of_an_inode",
        UserChange::OverwriteOfAnInode { inode, content: &content, write_time_seconds },
    )?;
    note_an_overwrite(inode);
    Ok(())
}

/// 按 inode 号从偏移 0 起接着写，每次至多 400 个数据单元，凑够 `data_units` 个。交回用了几次用户改动。
fn fill_a_file(pool: &mut Pool, tally: &mut Tally, inode: InodeNumber, data_units: u64) -> Result<(), RefusedChange> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷装得进 u64");
    let mut written = 0;
    while written < data_units {
        let units = (data_units - written).min(DATA_UNITS_PER_WRITE_AT_MOST);
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        let bytes = content_of(usize::try_from(units * payload).expect("几兆字节"), write_time_seconds);
        publish_change(
            pool,
            tally,
            "write_at_offset",
            UserChange::WriteAtOffset {
                inode,
                offset: FileOffsetInBytes(written * payload),
                bytes: PartialWriteBytes::new(&bytes).expect("写的字节不是空的"),
                write_time_seconds,
            },
        )?;
        written += units;
        set_units_of(inode, Some(units_of(inode) + units));
    }
    Ok(())
}

fn unlink_inode(pool: &mut Pool, tally: &mut Tally, inode: InodeNumber) -> Result<(), RefusedChange> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    publish_change(pool, tally, "unlink", UserChange::Unlink { inode, write_time_seconds })?;
    set_units_of(inode, None);
    Ok(())
}

/// 分片（登记第十二节执行员修订 4）：环境变量 `E207_SHARD=<第几片>/<共几片>`（片号 0 起；入口的服务分片步骤设，平常不设）。
/// 一片管历史里用户改动序号 `(from, to]` 的读数：每一片都从头放历史，序号不超过 `from` 的读数照取（读数模块的状态与不分片时逐步相同）、不打；
/// 在区间里的照打，与不分片时同一行；不是最后一片的，序号到 `to` 打完就收尾退出（后面的历史、量窗与重挂载归后面的片）。最后一片接着做完历史、量窗与重挂载。
/// 一个进程只跑一片的一项（入口用 `--exact` 点名一条用例），所以收尾直接退出进程。
#[derive(Clone, Copy)]
struct ShardRange {
    index: u64,
    count: u64,
    from: u64,
    to: u64,
    total: u64,
}

impl ShardRange {
    fn is_last(self) -> bool {
        self.index + 1 == self.count
    }

    /// 序号 `note` 的读数归这一片打。
    fn prints_at(self, note: u64) -> bool {
        note > self.from && note <= self.to
    }
}

/// 这一趟的分片；没设 `E207_SHARD` 是不分片（None）。分片只认三种形状（`choices::notes_of_the_history` 的式子只对它们成立）。
fn shard_of_this_run(point: Point) -> Option<ShardRange> {
    let text = std::env::var("E207_SHARD").ok()?;
    let (index, count) = choices::parse_shard(&text).unwrap_or_else(|| panic!("E207_SHARD={text} 不是 <第几片>/<共几片>"));
    assert!(
        matches!(point.shape, Shape::BigThenSmall | Shape::SmallThenBig | Shape::SmallFilesOfUnits(_)),
        "分片只认 BigThenSmall、SmallThenBig、SmallFilesOfUnits 三种形状：用户改动数的式子只对它们成立"
    );
    let total = choices::notes_of_the_history(point.files, point.unlinks);
    let (from, to) = choices::shard_interval(total, index, count);
    assert!(to > from, "{total} 个用户改动分 {count} 片，第 {index} 片是空的");
    Some(ShardRange { index, count, from, to, total })
}

/// 历史那一段的簿记：每做完一次用户改动取走读数（不让几万次发布的记录堆在内存里），每过一个进度间隔打一行进度。
struct History<'item> {
    item: &'item str,
    publishes_about: u64,
    remount_every: Option<u64>,
    user_changes: u64,
    last_progress: Instant,
    lines: readings::HistoryLines,
    tally: Tally,
    shard: Option<ShardRange>,
    /// 序号 `from` 的读数取走之后，固定点已做过的次数（这一片的发布从它的下一次起算）。
    settle_calls_at_the_shard_start: u64,
}

impl History<'_> {
    fn note(&mut self, pool: &mut Pool) {
        self.user_changes += 1;
        self.take_readings();
        if self.remount_every.is_some_and(|every| self.user_changes % every == 0) {
            pool.crash_and_mount_writable().expect("历史里崩溃后可写挂载");
            readings::mount_changed();
            self.take_readings();
        }
        if self.last_progress.elapsed() >= PROGRESS_INTERVAL {
            let shard_text = self.shard.map_or_else(String::new, |shard| {
                format!(" shard={}/{} stage={}", shard.index, shard.count, if self.user_changes <= shard.from { "replay" } else { "measure" })
            });
            eprintln!(
                "e207 progress item={} publishes_done={} of={} peak_rounds={}{shard_text}",
                self.item, self.lines.settle_calls, self.publishes_about, self.lines.peak_rounds
            );
            print_memory_line(self.item, "progress", self.user_changes, pool);
            self.last_progress = Instant::now();
        }
        if let Some(shard) = self.shard {
            if self.user_changes == shard.from {
                self.settle_calls_at_the_shard_start = self.lines.settle_calls;
            }
            if self.user_changes == shard.to && !shard.is_last() {
                print_memory_line(self.item, "shard_end", self.user_changes, pool);
                self.end_the_shard(shard);
            }
        }
    }

    /// 此刻取走的读数归不归这一片打（不分片总打）。
    fn prints(&self) -> bool {
        self.shard.is_none_or(|shard| shard.prints_at(self.user_changes))
    }

    fn take_readings(&mut self) {
        let prints = self.prints();
        for line in readings::take_settle_lines(self.item, "phase=history", false, &mut self.lines) {
            if prints {
                println!("{line}");
            }
        }
        drop(readings::take_record_lines(self.item, "phase=history", "-"));
    }

    /// 片尾标记（不是 `E207RESULT` 行，不进产物）：这一片管的序号与固定点次数，合并那一步核各片首尾相接、总数对得上。
    fn shard_marker(&self, shard: ShardRange) -> String {
        format!(
            "E207SHARD name=shard_done item={} shard={}/{} notes_from={} notes_to={} settle_calls_from={} settle_calls_to={} notes_total={}",
            self.item, shard.index, shard.count, shard.from, shard.to, self.settle_calls_at_the_shard_start, self.lines.settle_calls, shard.total
        )
    }

    /// 不是最后一片：序号到 `to`，打片尾标记就退出进程（一个进程只跑一片的一项）。
    fn end_the_shard(&self, shard: ShardRange) -> ! {
        println!("{}", self.shard_marker(shard));
        std::io::Write::flush(&mut std::io::stdout()).expect("标准输出刷得出去");
        std::process::exit(0)
    }
}

/// 第 2、3 步里 F 个小文件（历史第 1 条）：每批至多 100 个建 inode（`batch_of_one_files_10000` 一次 1 个），各覆盖写 2999 字节；
/// `files_1000_units_*` 各写 N 个数据单元。交回新建的号。
fn small_files(pool: &mut Pool, history: &mut History<'_>, point: Point, files: u64) -> Result<Vec<InodeNumber>, RefusedChange> {
    let batch = if point.shape == Shape::BatchOfOne { 1 } else { INODES_PER_BATCH };
    let mut created = Vec::new();
    let mut remaining = files;
    while remaining > 0 {
        let count = remaining.min(batch);
        for inode in create_inodes(pool, &mut history.tally, count)? {
            history.note(pool);
            if let Shape::SmallFilesOfUnits(data_units) = point.shape {
                fill_a_file(pool, &mut history.tally, inode, data_units)?;
            } else {
                overwrite_inode(pool, &mut history.tally, inode)?;
            }
            history.note(pool);
            created.push(inode);
        }
        remaining -= count;
    }
    Ok(created)
}

fn big_file(pool: &mut Pool, history: &mut History<'_>, data_units: u64) -> Result<InodeNumber, RefusedChange> {
    let big = create_inodes(pool, &mut history.tally, 1)?[0];
    history.note(pool);
    fill_a_file(pool, &mut history.tally, big, data_units)?;
    history.note(pool);
    Ok(big)
}

/// 第 4 步里的一次删：建一个空 inode、立刻 unlink 它。
fn one_unlink_of_the_history(pool: &mut Pool, history: &mut History<'_>) -> Result<(), RefusedChange> {
    let inode = create_inodes(pool, &mut history.tally, 1)?[0];
    history.note(pool);
    unlink_inode(pool, &mut history.tally, inode)?;
    history.note(pool);
    Ok(())
}

/// 定种子的 xorshift（`churn_random_*`），种子打进产物。
fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// 历史第 1–4 步与形状那一列多出来的步。交回还活着的文件（量窗 W2 取冷文件用，第一个文件不在内）。
fn run_the_history(pool: &mut Pool, history: &mut History<'_>, point: Point) -> Result<Vec<InodeNumber>, RefusedChange> {
    let mut live = if point.shape == Shape::SmallThenBig {
        let mut small = small_files(pool, history, point, point.files)?;
        small.push(big_file(pool, history, point.data_units)?);
        small
    } else {
        let mut big = vec![big_file(pool, history, point.data_units)?];
        big.extend(small_files(pool, history, point, point.files)?);
        big
    };
    if !matches!(point.shape, Shape::HalfDeleted | Shape::AlternateCreateUnlink) {
        for _ in 0..point.unlinks {
            one_unlink_of_the_history(pool, history)?;
        }
    }
    match point.shape {
        Shape::ChurnRandom => {
            let mut state = CHURN_SEED;
            for _ in 0..point.files {
                let inode = live[usize::try_from(next_random(&mut state) % u64::try_from(live.len()).expect("个数")).expect("下标")];
                overwrite_inode(pool, &mut history.tally, inode)?;
                history.note(pool);
            }
        }
        Shape::HotThenCold(overwrites) => {
            for index in 0..overwrites {
                let inode = live[usize::try_from(index % 10).expect("下标") + 1];
                overwrite_inode(pool, &mut history.tally, inode)?;
                history.note(pool);
            }
        }
        Shape::HalfDeleted => {
            let (even, odd): (Vec<InodeNumber>, Vec<InodeNumber>) = live.iter().copied().partition(|inode| inode.0 % 2 == 0);
            for inode in even {
                unlink_inode(pool, &mut history.tally, inode)?;
                history.note(pool);
            }
            live = odd;
            live.extend(small_files(pool, history, point, point.unlinks)?);
        }
        Shape::AlternateCreateUnlink => {
            let mut queue: VecDeque<InodeNumber> = live.iter().copied().collect();
            for _ in 0..point.unlinks {
                queue.extend(create_inodes(pool, &mut history.tally, 1)?);
                history.note(pool);
                let oldest = queue.pop_front().expect("队列不空");
                unlink_inode(pool, &mut history.tally, oldest)?;
                history.note(pool);
            }
            live = queue.into_iter().collect();
        }
        Shape::Snapshots => {
            for snapshot in 0..10_u64 {
                for index in 0..100_u64 {
                    let position = usize::try_from((snapshot * 100 + index) % u64::try_from(live.len()).expect("个数")).expect("下标");
                    overwrite_inode(pool, &mut history.tally, live[position])?;
                    history.note(pool);
                }
                publish_change(pool, &mut history.tally, "create_snapshot", UserChange::CreateSnapshot)?;
                history.note(pool);
            }
        }
        _ => {}
    }
    Ok(live)
}

/// 历史被会话交回拒绝：照 F8 打一行 `history_refused`（F1 也靠它的 `cause` 认），收完这一段的读数再停，这个项没有 `item_done`。
fn stop_on_a_refusal_in_the_history(history: &mut History<'_>, refused: &RefusedChange) -> ! {
    history.take_readings();
    // 被拒的那一次是序号 user_changes + 1：分片时只有管它的那一片打这几行，别的片放历史时遇到同样的拒绝只停、不打
    let prints = history.shard.is_none_or(|shard| shard.prints_at(history.user_changes + 1));
    for line in history.lines.close_the_bucket(history.item) {
        if prints {
            println!("{line}");
        }
    }
    if prints {
        println!(
            "E207RESULT name=history_refused item={} change={} cause={} user_changes={} settle_calls={} changes={} changes_refused={} floor_raises={}",
            history.item,
            refused.kind,
            refusal_cause_name(&refused.refusal),
            history.user_changes,
            history.lines.settle_calls,
            history.tally.changes,
            history.tally.changes_refused,
            history.tally.floor_raises
        );
    }
    panic!("历史里 {} 被会话拒了：{:?}", refused.kind, refused.refusal);
}

/// SHA-256（FIPS 180-4）：验证包不依赖 sha2，这里写一份，只算第五节 `image_sha256` 比盘那一个数。
struct Sha256 {
    state: [u32; 8],
    pending: Vec<u8>,
    length: u64,
}

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be,
    0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa,
    0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85,
    0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f,
    0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19],
            pending: Vec::with_capacity(64),
            length: 0,
        }
    }

    fn block(&mut self, block: &[u8]) {
        let mut schedule = [0_u32; 64];
        for (word, bytes) in schedule.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().expect("四个字节"));
        }
        for index in 16..64 {
            let schedule_sigma_zero = schedule[index - 15].rotate_right(7) ^ schedule[index - 15].rotate_right(18) ^ (schedule[index - 15] >> 3);
            let schedule_sigma_one = schedule[index - 2].rotate_right(17) ^ schedule[index - 2].rotate_right(19) ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16].wrapping_add(schedule_sigma_zero).wrapping_add(schedule[index - 7]).wrapping_add(schedule_sigma_one);
        }
        let mut working = self.state;
        for index in 0..64 {
            let round_sigma_one = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choose = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let first = working[7].wrapping_add(round_sigma_one).wrapping_add(choose).wrapping_add(SHA256_ROUND_CONSTANTS[index]).wrapping_add(schedule[index]);
            let round_sigma_zero = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let second = round_sigma_zero.wrapping_add(majority);
            working = [first.wrapping_add(second), working[0], working[1], working[2], working[3].wrapping_add(first), working[4], working[5], working[6]];
        }
        for (word, added) in self.state.iter_mut().zip(working) {
            *word = word.wrapping_add(added);
        }
    }

    fn absorb_bytes(&mut self, bytes: &[u8]) {
        self.length += u64::try_from(bytes.len()).expect("长度");
        self.pending.extend_from_slice(bytes);
        let whole = self.pending.len() / 64 * 64;
        let pending = std::mem::take(&mut self.pending);
        for block in pending[..whole].chunks_exact(64) {
            self.block(block);
        }
        self.pending = pending[whole..].to_vec();
    }

    fn hex(mut self) -> String {
        let bits = self.length * 8;
        let mut padding = vec![0x80_u8];
        padding.resize(1 + (119 - usize::try_from(self.length % 64).expect("余数")) % 64, 0);
        padding.extend_from_slice(&bits.to_be_bytes());
        self.absorb_bytes(&padding);
        self.state.iter().map(|word| format!("{word:08x}")).collect()
    }
}

/// 一份池的镜像（每块盘写过的扇区，按盘、按扇区号）的 sha256。`flip_one_bit` 只给弄坏开关 `image_witness_flipped` 用：
/// 取哈希之前把第一块盘第一个写过的扇区的第一位翻掉（在一份拷贝上翻，不动盘）。
fn image_sha256(image: &MemoryPool, flip_one_bit: bool) -> String {
    let mut flipped;
    let image = if flip_one_bit {
        flipped = image.clone();
        let device = flipped.devices.values_mut().next().expect("镜像里有盘");
        let (sector, bytes) = device.written_sectors().iter().next().map(|(sector, bytes)| (*sector, bytes.clone())).expect("镜像里写过扇区");
        let mut changed = bytes;
        changed[0] ^= 1;
        device.write(DeviceOffsetInBytes(sector * 512), &changed);
        &flipped
    } else {
        image
    };
    let mut digest = Sha256::new();
    for (device, sectors) in &image.devices {
        digest.absorb_bytes(&device.0.to_le_bytes());
        for (sector, bytes) in sectors.written_sectors() {
            digest.absorb_bytes(&sector.to_le_bytes());
            digest.absorb_bytes(bytes);
        }
    }
    digest.hex()
}

/// 两份镜像之间单元区里内容变了的槽数（第七节 ③：看盘这一路，不读记录）。扇区在后一份里写过且不同的算；每盘各算一份。
fn image_changed_unit_slots(before: &MemoryPool, after: &MemoryPool) -> u64 {
    let sectors_per_slot = SLOT_BYTES / 512;
    let start = unit_area_start_slot();
    let mut changed = std::collections::BTreeSet::new();
    for (device, sectors) in &after.devices {
        let old = before.devices.get(device).expect("两份镜像同样几块盘").written_sectors();
        for (sector, bytes) in sectors.written_sectors() {
            let slot = sector / sectors_per_slot;
            if slot >= start && old.get(sector) != Some(bytes) {
                changed.insert((device.0, slot));
            }
        }
    }
    u64::try_from(changed.len()).expect("槽数")
}

/// 覆盖写第一个文件（E203（准入之前扫类的步代价）量的那一种：`PoolUnderTest::overwrite`）。
fn overwrite_the_first_file(pool: &mut Pool) -> Result<(), RefusedChange> {
    drop(pool.overwrite(OVERWRITE_BYTES).map_err(|refusal| RefusedChange { kind: "overwrite", refusal })?);
    note_an_overwrite(InodeNumber(1));
    Ok(())
}

/// 第 3 组声明（读法第 2 条）的式子在 `choices`；旧单元数取驱动自己的记账 `units_of`，不读实现。
use choices::Declared;

/// inode 号换成 `choices` 吃的 u64。
fn inode_numbers(live: &[InodeNumber]) -> Vec<u64> {
    live.iter().map(|inode| inode.0).collect()
}

/// 一次被量的操作的定位（第 2 组读数的标签）：同一条产物里 `publish=<状态>/<操作>/<第几次>/<第几次判>` 认一次发布。
struct Measured<'item> {
    item: &'item str,
    state: &'item str,
    point_name: &'item str,
    probe: &'item str,
}

impl Measured<'_> {
    /// 量一次操作：之前打声明行与规模读数，做完取走第 1、2、4 组与 E203 的逐步读数；量窗被拒同样照报、这个项没有 `item_done`。
    fn measure(
        &self,
        pool: &mut Pool,
        operation_name: &str,
        index: u64,
        declared: Declared,
        change: impl FnOnce(&mut Pool, &mut Tally) -> Result<(), RefusedChange>,
    ) {
        let labels = format!("phase=window state={} operation={operation_name} operation_index={index}", self.state);
        println!("{}", choices::declared_line(self.item, &labels, declared));
        let state_line = readings::state_line(self.item, &labels, pool);
        if !state_line.is_empty() {
            println!("{state_line}");
        }
        let per_operation_image = PER_OPERATION_IMAGE_POINTS.contains(&self.point_name);
        let counted = readings::build_name() == "counts";
        let before = (per_operation_image && counted).then(|| pool.image());
        drop(readings::slots_placed_since_taken());
        let outcome = change(&mut *pool, &mut Tally::default());
        let mut unused_history = readings::HistoryLines::default();
        for line in readings::take_settle_lines(self.item, &labels, true, &mut unused_history) {
            println!("{line}");
        }
        for line in readings::take_record_lines(self.item, &labels, &format!("{}/{operation_name}/{index}", self.state)) {
            println!("{line}");
        }
        if per_operation_image {
            let after = pool.image();
            let witness_is_flipped = run_break_is("image_witness_flipped") && !counted && self.state == "remounted" && operation_name == "W2" && index == 5;
            println!(
                "E207RESULT name=image_after_the_operation item={} {labels} image_sha256={}",
                self.item,
                image_sha256(&after, witness_is_flipped)
            );
            if let Some(before) = before {
                let placed = readings::slots_placed_since_taken();
                println!(
                    "E207RESULT name=reconciliation item={} {labels} image_changed_unit_slots={} slots_placed={}",
                    self.item,
                    image_changed_unit_slots(&before, &after),
                    if run_break_is("slots_placed_undercounted") { placed.saturating_sub(1) } else { placed }
                );
            }
        }
        if let Err(refused) = outcome {
            println!(
                "E207RESULT name=window_refused item={} {labels} change={} cause={}",
                self.item,
                refused.kind,
                refusal_cause_name(&refused.refusal)
            );
            panic!("量窗里 {} 被会话拒了：{:?}", refused.kind, refused.refusal);
        }
    }

    fn overwrite(&self, pool: &mut Pool, operation_name: &str, index: u64, inode: InodeNumber) {
        self.measure(pool, operation_name, index, Declared::overwrite(units_of(inode)), |pool, tally| overwrite_inode(pool, tally, inode));
    }

    fn overwrite_the_first(&self, pool: &mut Pool, operation_name: &str, index: u64) {
        self.measure(pool, operation_name, index, Declared::overwrite(units_of(InodeNumber(1))), |pool, _| overwrite_the_first_file(pool));
    }

    /// W2 第 `index` 次的冷文件（取法在 `choices::cold_file`）。
    fn cold_file(&self, live: &[InodeNumber], point: Point, index: u64) -> InodeNumber {
        InodeNumber(choices::cold_file(&inode_numbers(live), self.point_name, point.shape == Shape::SmallThenBig, self.state == "same_mount", index))
    }

    /// 探针前缀 P1，只在同一次挂载那一段、W1 之前（取法在 `choices::probe_prefix`）。
    fn probe_prefix(&self, pool: &mut Pool, live: &[InodeNumber]) {
        match choices::probe_prefix(self.probe, &inode_numbers(live)) {
            choices::ProbePrefix::Nothing => {}
            choices::ProbePrefix::Overwrite(inode) => self.overwrite(pool, "P1", 1, InodeNumber(inode)),
            choices::ProbePrefix::Create => self.measure(pool, "P1", 1, Declared::create(1), |pool, tally| create_inodes(pool, tally, 1).map(|_| ())),
        }
    }

    /// 量窗（第五节「量窗」）：W1 覆盖写第一个文件 32 次；W2 覆盖写冷文件 32 次；W3「建 Y、覆盖写 Y、unlink Y」8 次；W4 再覆盖写第一个文件 8 次；
    /// 带扩展窗口的点在同一次挂载里接 W5–W7；对照点末尾一次阴性对照。
    fn window(&self, pool: &mut Pool, live: &[InodeNumber], point: Point) {
        if self.state == "same_mount" {
            self.probe_prefix(pool, live);
        }
        for index in 1..=FIRST_FILE_OVERWRITES {
            self.overwrite_the_first(pool, "W1", index);
        }
        let mut touched = std::collections::BTreeSet::new();
        for index in 1..=COLD_FILE_OVERWRITES {
            let cold = self.cold_file(live, point, index);
            touched.insert(cold.0);
            self.overwrite(pool, "W2", index, cold);
        }
        for index in 1..=CREATE_WRITE_UNLINK_ROUNDS {
            self.measure(pool, "W3", index, Declared::create_write_unlink(), |pool, tally| {
                let inode = create_inodes(pool, tally, 1)?[0];
                overwrite_inode(pool, tally, inode)?;
                unlink_inode(pool, tally, inode)
            });
        }
        for index in 1..=FIRST_FILE_OVERWRITES_AFTERWARDS {
            self.overwrite_the_first(pool, "W4", index);
        }
        if choices::extended_window_runs(self.state, self.point_name) {
            self.extended_window(pool, live, point, &touched);
        }
        if choices::negative_control_runs(self.point_name) {
            // 阴性对照：没有普通分配的发布。把第一个文件截成它现在的大小（N = S 只改记录）。
            self.measure(pool, "negative_control", 1, Declared::truncate_to_the_same_size(), |pool, tally| {
                pool.write_time_seconds += 1;
                let write_time_seconds = pool.write_time_seconds;
                publish_change(
                    pool,
                    tally,
                    "truncate_to_the_same_size",
                    UserChange::Truncate { inode: InodeNumber(1), new_size: u64::try_from(OVERWRITE_BYTES).expect("字节数"), write_time_seconds },
                )
            });
        }
    }

    /// 扩展窗口（第五节「量窗」第 3 条）：W5 覆盖写 200 个 W1–W4 没碰过的冷文件（没碰过的按号升序取等间隔的 200 个）；
    /// W6 建 100 个新 inode、各写 100 个单元（10 批，每批先建 10 个、再逐个写）；W7 覆盖写第一个文件 1 次、W5 没碰过的冷文件 5 个、W6 新建的 5 个。
    fn extended_window(&self, pool: &mut Pool, live: &[InodeNumber], point: Point, touched_by_the_cold_overwrites: &std::collections::BTreeSet<u64>) {
        let untouched: Vec<InodeNumber> =
            choices::untouched_small_files(&inode_numbers(live), point.shape == Shape::SmallThenBig, touched_by_the_cold_overwrites).into_iter().map(InodeNumber).collect();
        let evenly = |from: &[InodeNumber], count: usize| -> Vec<InodeNumber> {
            choices::evenly_spaced(&inode_numbers(from), count)
                .unwrap_or_else(|| panic!("{} 里没碰过的冷文件不够 {count} 个（只有 {}）", self.point_name, from.len()))
                .into_iter()
                .map(InodeNumber)
                .collect()
        };
        let extended_cold_files = evenly(&untouched, EXTENDED_COLD_OVERWRITES);
        for (position, inode) in extended_cold_files.iter().enumerate() {
            self.overwrite(pool, "W5", u64::try_from(position).expect("下标") + 1, *inode);
        }
        let mut index = 0;
        let mut created = Vec::new();
        for _ in 0..EXTENDED_NEW_FILES / EXTENDED_FILES_PER_BATCH {
            let first = next_inode(pool).0;
            index += 1;
            self.measure(pool, "W6", index, Declared::create(EXTENDED_FILES_PER_BATCH), |pool, tally| {
                create_inodes(pool, tally, EXTENDED_FILES_PER_BATCH).map(|_| ())
            });
            for inode in (first..first + EXTENDED_FILES_PER_BATCH).map(InodeNumber) {
                index += 1;
                self.measure(pool, "W6", index, Declared::fill(units_of(inode), EXTENDED_UNITS_PER_NEW_FILE), |pool, tally| {
                    fill_a_file(pool, tally, inode, EXTENDED_UNITS_PER_NEW_FILE)
                });
                created.push(inode);
            }
        }
        let still_untouched: Vec<InodeNumber> = untouched.iter().copied().filter(|inode| !extended_cold_files.contains(inode)).collect();
        self.overwrite_the_first(pool, "W7", 1);
        let mut index = 1;
        for inode in evenly(&still_untouched, EXTENDED_LAST_PICKS).into_iter().chain(evenly(&created, EXTENDED_LAST_PICKS)) {
            index += 1;
            self.overwrite(pool, "W7", index, inode);
        }
    }
}

/// 一个点的形状里每个小文件写几个数据单元（读法第 2 条：建时按构造）。
fn units_per_small_file(point: Point) -> u64 {
    if let Shape::SmallFilesOfUnits(data_units) = point.shape { data_units } else { 1 }
}

fn run_point(point_name: &str, arm: &str, probe: &str) {
    assert_eq!(std::env::var("E207_ARM").as_deref(), Ok(arm), "E207_ARM 要与用例点名的臂相同：入口按臂与探针分进程起");
    assert_eq!(std::env::var("E207_PROBE").as_deref(), Ok(probe), "E207_PROBE 要与用例点名的探针相同：入口按臂与探针分进程起");
    assert_eq!(arm == "plain", readings::build_name() == "plain", "plain 这条对照只在 --build plain 的副本里跑");
    let item = if probe == "standard" { format!("{point_name}__{arm}") } else { format!("{point_name}__{arm}__{probe}") };
    let point = point_named(point_name);
    let shard = shard_of_this_run(point);
    let first_or_whole = shard.is_none_or(|shard| shard.index == 0);
    if first_or_whole {
        println!(
            "E207RESULT name=point item={item} point={point_name} arm={arm} probe={probe} build={} unit_area_slots={} unlinks={} data_units={} files={} units_per_small_file={} shape={} churn_seed={CHURN_SEED}",
            readings::build_name(),
            point.unit_area_slots,
            point.unlinks,
            point.data_units,
            point.files,
            units_per_small_file(point),
            shape_tag(point.shape)
        );
    }
    let compact = uses_the_compact_device(point_name);
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + point.unit_area_slots) * SLOT_BYTES,
        |_identity, device: SparseBlockDevice| CompactDevice::new(device, compact.then(unit_area_start_slot)),
    );
    print_memory_line(&item, "pool_started", 0, &pool);
    for line in [readings::constants_line(&item, &pool), readings::sizes_line(&item)] {
        if !line.is_empty() && first_or_whole {
            println!("{line}");
        }
    }
    readings::set_window(false);
    readings::pause_readings(true);
    let mut history = History {
        item: &item,
        publishes_about: point.publishes_about,
        remount_every: if let Shape::RemountEvery(every) = point.shape { Some(every) } else { None },
        user_changes: 0,
        last_progress: Instant::now(),
        lines: readings::HistoryLines::default(),
        tally: Tally::default(),
        shard,
        settle_calls_at_the_shard_start: 0,
    };
    let live = match run_the_history(&mut pool, &mut history, point) {
        Ok(live) => live,
        Err(refused) => stop_on_a_refusal_in_the_history(&mut history, &refused),
    };
    history.take_readings();
    if let Some(shard) = shard {
        assert_eq!(history.user_changes, shard.total, "历史里用户改动的个数要等于 choices::notes_of_the_history 的式子，不然各片的区间对不上");
    }
    for line in history.lines.close_the_bucket(&item) {
        println!("{line}");
    }
    println!(
        "E207RESULT name=history item={item} user_changes={} settle_calls={} peak_rounds={} peak_round_work={} changes={} changes_refused={} floor_raises={}",
        history.user_changes,
        history.lines.settle_calls,
        history.lines.peak_rounds,
        history.lines.peak_round_work,
        history.tally.changes,
        history.tally.changes_refused,
        history.tally.floor_raises
    );
    print_memory_line(&item, "history_end", history.user_changes, &pool);
    if IMAGE_POINTS.contains(&point_name) {
        println!("E207RESULT name=image_sha256 item={item} when=history_end sha256={}", image_sha256(&pool.image(), false));
    }
    readings::pause_readings(false);
    readings::set_window(true);
    Measured { item: &item, state: "same_mount", point_name, probe }.window(&mut pool, &live, point);
    readings::set_window(false);
    readings::pause_readings(true);
    print_memory_line(&item, "same_mount_window_end", history.user_changes, &pool);
    if PER_OPERATION_IMAGE_POINTS.contains(&point_name) {
        println!("E207RESULT name=image_at_the_remount item={item} when=before_the_crash image_sha256={}", image_sha256(&pool.image(), false));
    }
    pool.crash_and_mount_writable().expect("崩溃后可写挂载");
    if PER_OPERATION_IMAGE_POINTS.contains(&point_name) {
        println!("E207RESULT name=image_at_the_remount item={item} when=after_the_mount image_sha256={}", image_sha256(&pool.image(), false));
    }
    readings::mount_changed();
    drop(readings::take_settle_lines(&item, "phase=remount", false, &mut readings::HistoryLines::default()));
    drop(readings::take_record_lines(&item, "phase=remount", "-"));
    readings::pause_readings(false);
    readings::set_window(true);
    Measured { item: &item, state: "remounted", point_name, probe }.window(&mut pool, &live, point);
    print_memory_line(&item, "item_done", history.user_changes, &pool);
    println!("E207RESULT name=item_done item={item}");
    if let Some(shard) = shard {
        println!("{}", history.shard_marker(shard));
    }
}

// ---- 下面每个 `<点>__<臂>`（探针项 `<点>__<臂>__<探针>`）一条用例；名单与入口 e207-strict-scan-steps-run.sh 的 items_table 同一份（入口自证核两边相同）----

#[test]
fn item_base__head() {
    run_point("base", "head", "standard");
}

#[test]
fn item_files_1000__head() {
    run_point("files_1000", "head", "standard");
}

#[test]
fn item_files_10000__head() {
    run_point("files_10000", "head", "standard");
}

#[test]
fn item_files_1000_units_100__head() {
    run_point("files_1000_units_100", "head", "standard");
}

#[test]
fn item_files_100_units_100__head() {
    run_point("files_100_units_100", "head", "standard");
}

#[test]
fn item_files_56_units_100__head() {
    run_point("files_56_units_100", "head", "standard");
}

#[test]
fn item_files_178_units_100__head() {
    run_point("files_178_units_100", "head", "standard");
}

#[test]
fn item_files_316_units_100__head() {
    run_point("files_316_units_100", "head", "standard");
}

#[test]
fn item_files_562_units_100__head() {
    run_point("files_562_units_100", "head", "standard");
}

#[test]
fn item_files_10_units_1000__head() {
    run_point("files_10_units_1000", "head", "standard");
}

#[test]
fn item_files_100_units_1000__head() {
    run_point("files_100_units_1000", "head", "standard");
}

#[test]
fn item_files_10_units_10000__head() {
    run_point("files_10_units_10000", "head", "standard");
}

#[test]
fn item_files_then_big_1000__head() {
    run_point("files_then_big_1000", "head", "standard");
}

#[test]
fn item_files_then_big_10000__head() {
    run_point("files_then_big_10000", "head", "standard");
}

#[test]
fn item_files_10_units_1000_unit_area_2_pow_17__head() {
    run_point("files_10_units_1000_unit_area_2_pow_17", "head", "standard");
}

#[test]
fn item_files_100_units_1000_unit_area_2_pow_20__head() {
    run_point("files_100_units_1000_unit_area_2_pow_20", "head", "standard");
}

#[test]
fn item_base__plain() {
    run_point("base", "plain", "standard");
}

#[test]
fn item_files_1000__plain() {
    run_point("files_1000", "plain", "standard");
}

#[test]
fn item_files_1000_units_100__head__cold_first() {
    run_point("files_1000_units_100", "head", "cold_first");
}

#[test]
fn item_files_1000_units_100__head__create_first() {
    run_point("files_1000_units_100", "head", "create_first");
}

#[test]
fn item_files_1000_units_100__head__young_first() {
    run_point("files_1000_units_100", "head", "young_first");
}

#[test]
fn item_files_30000__head() {
    run_point("files_30000", "head", "standard");
}

#[test]
fn item_files_10_units_1000_unit_area_2_pow_21__head() {
    run_point("files_10_units_1000_unit_area_2_pow_21", "head", "standard");
}

#[test]
fn item_files_100_units_1000_unit_area_2_pow_21__head() {
    run_point("files_100_units_1000_unit_area_2_pow_21", "head", "standard");
}

#[test]
fn item_files_1000_units_1000__head() {
    run_point("files_1000_units_1000", "head", "standard");
}

// ---- 钉绝对值的断言：数由登记与格式常量另外算出，不从上面的表与函数读回来；各条抄自哪里、怎么算的写在断言旁边 ----

/// 6 MiB 环的单元区起点：环从槽 1024 起（`singlefs-format` 的 `JOURNAL_RING_START_SLOT`），6 MiB / 16 KiB（`SLOT_BYTES`）= 384 槽，1024 + 384 = 1408
/// （E203（准入之前扫类的步代价）装置同一个数）；1408 = 22 × 64，落在聚簇段边界上。
#[test]
fn the_six_mebibyte_ring_puts_the_unit_area_at_slot_1408() {
    assert_eq!(JOURNAL_RING_BYTES, 6_291_456);
    assert_eq!(parameters().geometry.journal_ring_bytes, 6_291_456);
    assert_eq!(unit_area_start_slot(), 1408);
}

/// 登记第五节「取样点」表里几个点的几何，逐项抄登记：基点 U = 2¹⁷ = 131072 槽、每个小文件 1 个数据单元；
/// `files_1000_units_1000` 的 U = 2²¹ = 2097152 槽、每个小文件 1000 个数据单元；`files_10_units_1000` 的 U = 2¹⁹ = 524288 槽。
#[test]
fn the_registered_points_carry_the_registered_geometry() {
    assert_eq!(point_named("base").unit_area_slots, 131_072);
    assert_eq!(units_per_small_file(point_named("base")), 1);
    assert_eq!(point_named("files_1000_units_1000").unit_area_slots, 2_097_152);
    assert_eq!(point_named("files_1000_units_1000").files, 1_000);
    assert_eq!(units_per_small_file(point_named("files_1000_units_1000")), 1_000);
    assert_eq!(point_named("files_10_units_1000").unit_area_slots, 524_288);
    assert_eq!(point_named("files_10000").files, 10_000);
}

/// 基点历史（登记历史第 1–4 步：一个 10 数据单元的大文件、10 个小文件、10 次删）各步的用户改动数，另外数：
/// 大文件 = 建 1 个 inode + 一次写（10 个单元不超过一次至多 400 个）= 2 次；小文件 = 一批建 10 个 inode（至多 100 个一批）1 次 + 各覆盖写 10 次 = 11 次；
/// 10 次删各建一个空 inode 再 unlink = 20 次；合 33 次发布用户改动，一次都没被拒、没抬 F。`History::note` 每个 inode 建好与做完各叫一次：
/// 大文件 2 次、小文件 10 × 2 = 20 次、删 10 × 2 = 20 次，合 42 次。
#[test]
fn the_base_history_makes_thirty_three_user_changes_and_notes_forty_two_steps() {
    let item = "base__pin";
    let point = point_named("base");
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + point.unit_area_slots) * SLOT_BYTES,
        |_identity, device: SparseBlockDevice| CompactDevice::new(device, None),
    );
    readings::set_window(false);
    readings::pause_readings(true);
    let mut history = History {
        item,
        publishes_about: point.publishes_about,
        remount_every: None,
        user_changes: 0,
        last_progress: Instant::now(),
        lines: readings::HistoryLines::default(),
        tally: Tally::default(),
        shard: None,
        settle_calls_at_the_shard_start: 0,
    };
    let live = match run_the_history(&mut pool, &mut history, point) {
        Ok(live) => live,
        Err(refused) => panic!("基点历史被会话拒了：{}", refusal_cause_name(&refused.refusal)),
    };
    assert_eq!(history.tally.changes, 33);
    assert_eq!(history.user_changes, 42);
    assert_eq!(history.tally.changes_refused, 0);
    assert_eq!(history.tally.floor_raises, 0);
    // 活着的文件 = 1 个大文件 + 10 个小文件（删掉的是另建的空 inode）。
    assert_eq!(live.len(), 11);
}
