//! E206（发布固定点轮数的代价）的历史用例：拷进副本当 `crates/singlefs-verification/tests/e206_fixed_point_cost_histories.rs`，
//! 副本由 `e206_fixed_point_cost_counters.py` 套上读数（`--build counts`）或一个字节不插（`--build plain`，只比盘）。
//! 读数行怎么打在副本的 `tests/e206_readings/mod.rs`（两种构建各一份，都由插计数脚本写）。
//!
//! 一个 `<点>__<臂>` 一条用例（跑前登记第五节「取样点」「历史」）；同一段历史上连着量，后一步读前一步留下的那一版，轮数本身就是被量的次序效应，
//! 所以历史各步、量窗 W1–W4、两种挂载状态串行、不拆。臂由环境变量 `E206_ARM` 定（进程开头读一次），用例核它与自己点名的臂相同。
//! 历史那一段每过进度间隔打一行 `e206 progress`，判结果只读 `E206RESULT` 行。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]
#![allow(non_snake_case, reason = "用例名是 item_<点>__<臂>，入口按这个名字单跑")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;
#[path = "e206_readings/mod.rs"]
mod readings;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::transaction::PartialWriteBytes;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::scenario::e142_parameters;
use std::collections::VecDeque;
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
const SHARD_FOR_THE_COLD_FILES: usize = 32;
/// 第七节 ③ 对账点。
const RECONCILED_POINTS: [&str; 2] = ["base", "files_100"];
/// 第五节阴性对照的点。
const NEGATIVE_CONTROL_POINTS: [&str; 4] = ["base", "files_10000", "data_units_10000", "unlinks_10000"];

type Pool = PoolUnderTest<SparseBlockDevice>;

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
    match name {
        "base" => BASE,
        "files_100" => Point { files: 100, ..about(100) },
        "files_1000" => Point { files: 1_000, ..about(1_000) },
        "files_10000" => Point { files: 10_000, ..about(10_000) },
        "data_units_100" => Point { data_units: 100, ..about(50) },
        "data_units_1000" => Point { data_units: 1_000, ..about(50) },
        "data_units_10000" => Point { data_units: 10_000, ..about(75) },
        "unlinks_100" => Point { unlinks: 100, ..about(300) },
        "unlinks_1000" => Point { unlinks: 1_000, ..about(3_000) },
        "unlinks_10000" => Point { unlinks: 10_000, ..about(30_000) },
        "files_30000" => Point { unit_area_slots: power_of_two(18), files: 30_000, ..about(30_000) },
        "files_1000_units_10" => shaped(Shape::SmallFilesOfUnits(10), power_of_two(18), 1_000, 1_000),
        "files_1000_units_100" => shaped(Shape::SmallFilesOfUnits(100), power_of_two(19), 1_000, 1_000),
        "files_1000_unit_area_2_pow_14" => shaped(Shape::BigThenSmall, power_of_two(14), 1_000, 1_000),
        "files_1000_unit_area_2_pow_20" => shaped(Shape::BigThenSmall, power_of_two(20), 1_000, 1_000),
        "files_1000_unit_area_2_pow_24" => shaped(Shape::BigThenSmall, power_of_two(24), 1_000, 1_000),
        "diagonal" => Point { unit_area_slots: power_of_two(20), unlinks: 10_000, data_units: 10_000, files: 1_000, ..about(31_000) },
        "files_then_big_1000" => Point { data_units: 1_000, ..shaped(Shape::SmallThenBig, power_of_two(17), 1_000, 1_000) },
        "files_then_big_10000" => Point { data_units: 10_000, ..shaped(Shape::SmallThenBig, power_of_two(17), 10_000, 10_000) },
        "churn_random_1000" => shaped(Shape::ChurnRandom, power_of_two(20), 1_000, 2_000),
        "churn_random_10000" => shaped(Shape::ChurnRandom, power_of_two(20), 10_000, 20_000),
        "hot_then_cold_1000" => shaped(Shape::HotThenCold(1_000), power_of_two(20), 1_000, 2_000),
        "hot_then_cold_10000" => shaped(Shape::HotThenCold(10_000), power_of_two(20), 1_000, 11_000),
        "churn_half_deleted" => Point { unlinks: 5_000, ..shaped(Shape::HalfDeleted, power_of_two(20), 10_000, 30_000) },
        "alternate_create_unlink" => Point { unlinks: 10_000, ..shaped(Shape::AlternateCreateUnlink, power_of_two(20), 1_000, 41_000) },
        "snapshots_1000" => shaped(Shape::Snapshots, power_of_two(20), 1_000, 2_000),
        "remount_every_1000" => shaped(Shape::RemountEvery(1_000), power_of_two(17), 10_000, 10_000),
        "batch_of_one_files_10000" => shaped(Shape::BatchOfOne, power_of_two(17), 10_000, 20_000),
        "data_units_100000" => Point { unit_area_slots: power_of_two(19), data_units: 100_000, ..about(300) },
        "data_units_1000000" => Point { unit_area_slots: power_of_two(22), data_units: 1_000_000, ..about(2_500) },
        "unlinks_100000" => Point { unit_area_slots: power_of_two(20), unlinks: 100_000, ..about(300_000) },
        "files_100000" => Point { unit_area_slots: power_of_two(20), files: 100_000, ..about(101_000) },
        "user_scale_many_rounds" => shaped(Shape::BigThenSmall, (1 << 30) - 1408, 2_000, 2_020),
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
    )
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
    }
    Ok(())
}

fn unlink_inode(pool: &mut Pool, tally: &mut Tally, inode: InodeNumber) -> Result<(), RefusedChange> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    publish_change(pool, tally, "unlink", UserChange::Unlink { inode, write_time_seconds })
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
            eprintln!(
                "e206 progress item={} publishes_done={} of={} peak_rounds={} peak_W={}",
                self.item, self.lines.settle_calls, self.publishes_about, self.lines.peak_rounds, self.lines.peak_round_work
            );
            self.last_progress = Instant::now();
        }
    }

    fn take_readings(&mut self) {
        for line in readings::take_settle_lines(self.item, "phase=history", false, &mut self.lines) {
            println!("{line}");
        }
        drop(readings::take_record_lines(self.item, "phase=history", "-"));
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
    for line in history.lines.close_the_bucket(history.item) {
        println!("{line}");
    }
    println!(
        "E206RESULT name=history_refused item={} change={} cause={} user_changes={} settle_calls={} changes={} changes_refused={} floor_raises={}",
        history.item,
        refused.kind,
        refusal_cause_name(&refused.refusal),
        history.user_changes,
        history.lines.settle_calls,
        history.tally.changes,
        history.tally.changes_refused,
        history.tally.floor_raises
    );
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
        let mut working_state = self.state;
        for index in 0..64 {
            let compression_sigma_one = working_state[4].rotate_right(6) ^ working_state[4].rotate_right(11) ^ working_state[4].rotate_right(25);
            let choose = (working_state[4] & working_state[5]) ^ (!working_state[4] & working_state[6]);
            let first = working_state[7].wrapping_add(compression_sigma_one).wrapping_add(choose).wrapping_add(SHA256_ROUND_CONSTANTS[index]).wrapping_add(schedule[index]);
            let compression_sigma_zero = working_state[0].rotate_right(2) ^ working_state[0].rotate_right(13) ^ working_state[0].rotate_right(22);
            let majority = (working_state[0] & working_state[1]) ^ (working_state[0] & working_state[2]) ^ (working_state[1] & working_state[2]);
            let second = compression_sigma_zero.wrapping_add(majority);
            working_state = [first.wrapping_add(second), working_state[0], working_state[1], working_state[2], working_state[3].wrapping_add(first), working_state[4], working_state[5], working_state[6]];
        }
        for (word, added) in self.state.iter_mut().zip(working_state) {
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

/// 一份池的镜像（每块盘写过的扇区，按盘、按扇区号）的 sha256。
fn image_sha256(image: &MemoryPool) -> String {
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
    pool.overwrite(OVERWRITE_BYTES)
        .map(|_| ())
        .map_err(|refusal| RefusedChange { kind: "overwrite", refusal })
}

/// 一次被量的操作的定位（第 2 组读数的标签）：同一条产物里 `publish=<状态>/<操作>/<第几次>/<第几次判>` 认一次发布。
struct Measured<'item> {
    item: &'item str,
    state: &'item str,
    point_name: &'item str,
}

impl Measured<'_> {
    /// 量一次操作：之前打规模读数，做完取走第 1–7 组与 E203 的逐步读数；`.expect` 不用：量窗被拒同样照报、这个项没有 `item_done`。
    fn measure(&self, pool: &mut Pool, operation: &str, index: u64, change: impl FnOnce(&mut Pool, &mut Tally) -> Result<(), RefusedChange>) {
        let labels = format!("phase=window state={} op={operation} op_index={index}", self.state);
        let state_line = readings::state_line(self.item, &labels, pool);
        if !state_line.is_empty() {
            println!("{state_line}");
        }
        let reconciled = index == 1
            && matches!(operation, "W1" | "W2" | "W3")
            && self.state == "same_mount"
            && RECONCILED_POINTS.contains(&self.point_name);
        let before = reconciled.then(|| pool.image());
        drop(readings::slots_placed_since_taken());
        let outcome = change(&mut *pool, &mut Tally::default());
        let mut unused_history = readings::HistoryLines::default();
        for line in readings::take_settle_lines(self.item, &labels, true, &mut unused_history) {
            println!("{line}");
        }
        for line in readings::take_record_lines(self.item, &labels, &format!("{}/{operation}/{index}", self.state)) {
            println!("{line}");
        }
        if let Err(refused) = outcome {
            println!(
                "E206RESULT name=window_refused item={} {labels} change={} cause={}",
                self.item,
                refused.kind,
                refusal_cause_name(&refused.refusal)
            );
            panic!("量窗里 {} 被会话拒了：{:?}", refused.kind, refused.refusal);
        }
        if let Some(before) = before {
            println!(
                "E206RESULT name=reconciliation item={} {labels} image_changed_unit_slots={} slots_placed={}",
                self.item,
                image_changed_unit_slots(&before, &pool.image()),
                readings::slots_placed_since_taken()
            );
        }
    }

    /// 量窗（历史第 3 条）：W1 覆盖写第一个文件 32 次；W2 覆盖写冷文件 32 次；W3「建 Y、覆盖写 Y、unlink Y」8 次；W4 再覆盖写第一个文件 8 次。
    fn window(&self, pool: &mut Pool, live: &[InodeNumber], point: Point) {
        let first = InodeNumber(1);
        for index in 1..=FIRST_FILE_OVERWRITES {
            self.measure(pool, "W1", index, |pool, _| overwrite_the_first_file(pool));
        }
        for index in 1..=COLD_FILE_OVERWRITES {
            let share = usize::try_from(index - 1).expect("下标");
            let cold = live[share * live.len() / SHARD_FOR_THE_COLD_FILES % live.len()];
            self.measure(pool, "W2", index, |pool, tally| overwrite_inode(pool, tally, cold));
        }
        for index in 1..=CREATE_WRITE_UNLINK_ROUNDS {
            self.measure(pool, "W3", index, |pool, tally| {
                let inode = create_inodes(pool, tally, 1)?[0];
                overwrite_inode(pool, tally, inode)?;
                unlink_inode(pool, tally, inode)
            });
        }
        for index in 1..=FIRST_FILE_OVERWRITES_AFTERWARDS {
            self.measure(pool, "W4", index, |pool, _| overwrite_the_first_file(pool));
        }
        if point.shape == Shape::Snapshots {
            let snapshot = pool.current_file_version().snapshot_entries()[0].inode_root.tree;
            self.measure(pool, "destroy_snapshot", 1, |pool, tally| {
                publish_change(pool, tally, "destroy_snapshot", UserChange::DestroySnapshot { snapshot })
            });
        }
        if NEGATIVE_CONTROL_POINTS.contains(&self.point_name) {
            // 阴性对照：没有普通分配的发布。把第一个文件截成它现在的大小（N = S 只改记录）。
            self.measure(pool, "negative_control", 1, |pool, tally| {
                pool.write_time_seconds += 1;
                let write_time_seconds = pool.write_time_seconds;
                publish_change(
                    pool,
                    tally,
                    "truncate_to_the_same_size",
                    UserChange::Truncate { inode: first, new_size: u64::try_from(OVERWRITE_BYTES).expect("字节数"), write_time_seconds },
                )
            });
        }
    }
}

fn run_point_under_arm(point_name: &str, arm: &str) {
    assert_eq!(std::env::var("E206_ARM").as_deref(), Ok(arm), "E206_ARM 要与用例点名的臂相同：入口按臂分进程起");
    assert_eq!(arm == "plain", readings::build_name() == "plain", "plain 这条对照只在 --build plain 的副本里跑");
    let item = format!("{point_name}__{arm}");
    let point = point_named(point_name);
    println!(
        "E206RESULT name=point item={item} point={point_name} arm={arm} build={} unit_area_slots={} unlinks={} data_units={} files={} shape={} churn_seed={CHURN_SEED}",
        readings::build_name(),
        point.unit_area_slots,
        point.unlinks,
        point.data_units,
        point.files,
        shape_tag(point.shape)
    );
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + point.unit_area_slots) * SLOT_BYTES,
        plain_device,
    );
    for line in [readings::constants_line(&item, &pool), readings::sizes_line(&item)] {
        if !line.is_empty() {
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
    };
    let live = match run_the_history(&mut pool, &mut history, point) {
        Ok(live) => live,
        Err(refused) => stop_on_a_refusal_in_the_history(&mut history, &refused),
    };
    history.take_readings();
    for line in history.lines.close_the_bucket(&item) {
        println!("{line}");
    }
    println!(
        "E206RESULT name=history item={item} user_changes={} settle_calls={} peak_rounds={} peak_round_work={} changes={} changes_refused={} floor_raises={}",
        history.user_changes,
        history.lines.settle_calls,
        history.lines.peak_rounds,
        history.lines.peak_round_work,
        history.tally.changes,
        history.tally.changes_refused,
        history.tally.floor_raises
    );
    println!("E206RESULT name=image_sha256 item={item} when=history_end sha256={}", image_sha256(&pool.image()));
    readings::pause_readings(false);
    let measured = Measured { item: &item, state: "same_mount", point_name };
    readings::set_window(true);
    measured.window(&mut pool, &live, point);
    readings::set_window(false);
    readings::pause_readings(true);
    pool.crash_and_mount_writable().expect("崩溃后可写挂载");
    readings::mount_changed();
    drop(readings::take_settle_lines(&item, "phase=remount", false, &mut readings::HistoryLines::default()));
    drop(readings::take_record_lines(&item, "phase=remount", "-"));
    readings::pause_readings(false);
    readings::set_window(true);
    Measured { item: &item, state: "remounted", point_name }.window(&mut pool, &live, point);
    println!("E206RESULT name=item_done item={item}");
}

// ---- 下面每个 `<点>__<臂>` 一条用例；名单与入口 e206-fixed-point-cost-run.sh 的 items_table 同一份（入口自证核两边相同）----

#[test]
fn item_base__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("base", "p1");
}

#[test]
fn item_base__p2a() {
    run_point_under_arm("base", "p2a");
}

#[test]
fn item_files_100__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_100", "p1");
}

#[test]
fn item_files_100__p2a() {
    run_point_under_arm("files_100", "p2a");
}

#[test]
fn item_files_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_1000", "p1");
}

#[test]
fn item_files_1000__p2a() {
    run_point_under_arm("files_1000", "p2a");
}

#[test]
fn item_files_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_10000", "p1");
}

#[test]
fn item_files_10000__p2a() {
    run_point_under_arm("files_10000", "p2a");
}

#[test]
fn item_data_units_100__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("data_units_100", "p1");
}

#[test]
fn item_data_units_100__p2a() {
    run_point_under_arm("data_units_100", "p2a");
}

#[test]
fn item_data_units_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("data_units_1000", "p1");
}

#[test]
fn item_data_units_1000__p2a() {
    run_point_under_arm("data_units_1000", "p2a");
}

#[test]
fn item_data_units_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("data_units_10000", "p1");
}

#[test]
fn item_data_units_10000__p2a() {
    run_point_under_arm("data_units_10000", "p2a");
}

#[test]
fn item_unlinks_100__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("unlinks_100", "p1");
}

#[test]
fn item_unlinks_100__p2a() {
    run_point_under_arm("unlinks_100", "p2a");
}

#[test]
fn item_unlinks_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("unlinks_1000", "p1");
}

#[test]
fn item_unlinks_1000__p2a() {
    run_point_under_arm("unlinks_1000", "p2a");
}

#[test]
fn item_unlinks_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("unlinks_10000", "p1");
}

#[test]
fn item_unlinks_10000__p2a() {
    run_point_under_arm("unlinks_10000", "p2a");
}

#[test]
fn item_files_30000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_30000", "p1");
}

#[test]
fn item_files_30000__p2a() {
    run_point_under_arm("files_30000", "p2a");
}

#[test]
fn item_files_1000_units_10__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_1000_units_10", "p1");
}

#[test]
fn item_files_1000_units_10__p2a() {
    run_point_under_arm("files_1000_units_10", "p2a");
}

#[test]
fn item_files_1000_units_100__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_1000_units_100", "p1");
}

#[test]
fn item_files_1000_units_100__p2a() {
    run_point_under_arm("files_1000_units_100", "p2a");
}

#[test]
fn item_files_1000_unit_area_2_pow_14__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_1000_unit_area_2_pow_14", "p1");
}

#[test]
fn item_files_1000_unit_area_2_pow_14__p2a() {
    run_point_under_arm("files_1000_unit_area_2_pow_14", "p2a");
}

#[test]
fn item_files_1000_unit_area_2_pow_20__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_1000_unit_area_2_pow_20", "p1");
}

#[test]
fn item_files_1000_unit_area_2_pow_20__p2a() {
    run_point_under_arm("files_1000_unit_area_2_pow_20", "p2a");
}

#[test]
fn item_files_1000_unit_area_2_pow_24__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_1000_unit_area_2_pow_24", "p1");
}

#[test]
fn item_files_1000_unit_area_2_pow_24__p2a() {
    run_point_under_arm("files_1000_unit_area_2_pow_24", "p2a");
}

#[test]
fn item_base__single_step() {
    run_point_under_arm("base", "single_step");
}

#[test]
fn item_base__closure_off() {
    run_point_under_arm("base", "closure_off");
}

#[test]
fn item_files_100__single_step() {
    run_point_under_arm("files_100", "single_step");
}

#[test]
fn item_files_100__closure_off() {
    run_point_under_arm("files_100", "closure_off");
}

#[test]
fn item_files_1000__single_step() {
    run_point_under_arm("files_1000", "single_step");
}

#[test]
fn item_files_1000__closure_off() {
    run_point_under_arm("files_1000", "closure_off");
}

#[test]
fn item_base__plain() {
    run_point_under_arm("base", "plain");
}

#[test]
fn item_files_1000__plain() {
    run_point_under_arm("files_1000", "plain");
}

// ---- 第二段 ----

#[test]
fn item_diagonal__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("diagonal", "p1");
}

#[test]
fn item_diagonal__p2a() {
    run_point_under_arm("diagonal", "p2a");
}

#[test]
fn item_files_then_big_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_then_big_1000", "p1");
}

#[test]
fn item_files_then_big_1000__p2a() {
    run_point_under_arm("files_then_big_1000", "p2a");
}

#[test]
fn item_files_then_big_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_then_big_10000", "p1");
}

#[test]
fn item_files_then_big_10000__p2a() {
    run_point_under_arm("files_then_big_10000", "p2a");
}

#[test]
fn item_churn_random_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("churn_random_1000", "p1");
}

#[test]
fn item_churn_random_1000__p2a() {
    run_point_under_arm("churn_random_1000", "p2a");
}

#[test]
fn item_churn_random_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("churn_random_10000", "p1");
}

#[test]
fn item_churn_random_10000__p2a() {
    run_point_under_arm("churn_random_10000", "p2a");
}

#[test]
fn item_hot_then_cold_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("hot_then_cold_1000", "p1");
}

#[test]
fn item_hot_then_cold_1000__p2a() {
    run_point_under_arm("hot_then_cold_1000", "p2a");
}

#[test]
fn item_hot_then_cold_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("hot_then_cold_10000", "p1");
}

#[test]
fn item_hot_then_cold_10000__p2a() {
    run_point_under_arm("hot_then_cold_10000", "p2a");
}

#[test]
fn item_churn_half_deleted__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("churn_half_deleted", "p1");
}

#[test]
fn item_churn_half_deleted__p2a() {
    run_point_under_arm("churn_half_deleted", "p2a");
}

#[test]
fn item_alternate_create_unlink__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("alternate_create_unlink", "p1");
}

#[test]
fn item_alternate_create_unlink__p2a() {
    run_point_under_arm("alternate_create_unlink", "p2a");
}

#[test]
fn item_snapshots_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("snapshots_1000", "p1");
}

#[test]
fn item_snapshots_1000__p2a() {
    run_point_under_arm("snapshots_1000", "p2a");
}

#[test]
fn item_remount_every_1000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("remount_every_1000", "p1");
}

#[test]
fn item_remount_every_1000__p2a() {
    run_point_under_arm("remount_every_1000", "p2a");
}

#[test]
fn item_batch_of_one_files_10000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("batch_of_one_files_10000", "p1");
}

#[test]
fn item_batch_of_one_files_10000__p2a() {
    run_point_under_arm("batch_of_one_files_10000", "p2a");
}

#[test]
fn item_files_10000__p2b16() {
    run_point_under_arm("files_10000", "p2b16");
}

#[test]
fn item_files_10000__p2b256() {
    run_point_under_arm("files_10000", "p2b256");
}

#[test]
fn item_files_10000__whole() {
    run_point_under_arm("files_10000", "whole");
}

#[test]
fn item_data_units_10000__p2b16() {
    run_point_under_arm("data_units_10000", "p2b16");
}

#[test]
fn item_data_units_10000__p2b256() {
    run_point_under_arm("data_units_10000", "p2b256");
}

#[test]
fn item_data_units_10000__whole() {
    run_point_under_arm("data_units_10000", "whole");
}

#[test]
fn item_unlinks_10000__p2b16() {
    run_point_under_arm("unlinks_10000", "p2b16");
}

#[test]
fn item_unlinks_10000__p2b256() {
    run_point_under_arm("unlinks_10000", "p2b256");
}

#[test]
fn item_unlinks_10000__whole() {
    run_point_under_arm("unlinks_10000", "whole");
}

#[test]
fn item_base__p2b_never() {
    run_point_under_arm("base", "p2b_never");
}

#[test]
fn item_files_100__p2b_never() {
    run_point_under_arm("files_100", "p2b_never");
}

#[test]
fn item_files_1000__whole() {
    run_point_under_arm("files_1000", "whole");
}

// ---- 全量才跑的点（第三段）----

#[test]
fn item_data_units_100000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("data_units_100000", "p1");
}

#[test]
fn item_data_units_100000__p2a() {
    run_point_under_arm("data_units_100000", "p2a");
}

#[test]
fn item_data_units_1000000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("data_units_1000000", "p1");
}

#[test]
fn item_data_units_1000000__p2a() {
    run_point_under_arm("data_units_1000000", "p2a");
}

#[test]
fn item_unlinks_100000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("unlinks_100000", "p1");
}

#[test]
fn item_unlinks_100000__p2a() {
    run_point_under_arm("unlinks_100000", "p2a");
}

#[test]
fn item_files_100000__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("files_100000", "p1");
}

#[test]
fn item_files_100000__p2a() {
    run_point_under_arm("files_100000", "p2a");
}

#[test]
fn item_user_scale_many_rounds__p1() { // naming-lint:external 臂名 p1 是 E206（发布固定点轮数的代价）跑前登记定的，入口与产物照它点名
    run_point_under_arm("user_scale_many_rounds", "p1");
}

#[test]
fn item_user_scale_many_rounds__p2a() {
    run_point_under_arm("user_scale_many_rounds", "p2a");
}

// ---- 钉绝对值的断言：数由登记与格式常量另外算出，不从上面的表与函数读回来；各条抄自哪里、怎么算的写在断言旁边 ----

/// 6 MiB 环的单元区起点：环从槽 1024 起（`singlefs-format` 的 `JOURNAL_RING_START_SLOT`），6 MiB / 16 KiB（`SLOT_BYTES`）= 384 槽，1024 + 384 = 1408
/// （E203（准入之前扫类的步代价）装置同一个数，也是计数模型 `e206_fixed_point_cost.rs` 的 `UNIT_AREA_START_SLOT`）；1408 = 22 × 64，落在聚簇段边界上。
#[test]
fn the_six_mebibyte_ring_puts_the_unit_area_at_slot_1408() {
    assert_eq!(JOURNAL_RING_BYTES, 6_291_456);
    assert_eq!(parameters().geometry.journal_ring_bytes, 6_291_456);
    assert_eq!(unit_area_start_slot(), 1408);
}

/// 登记第五节「取样点」表里几个点的几何，逐项抄登记：基点 U = 2¹⁷ = 131072 槽；`unlinks_100000` 与 `files_100000` 的 U = 2²⁰ = 1048576；
/// `data_units_1000000` 的 U = 2²² = 4194304；`user_scale_many_rounds` 的 U = 2³⁰ − 1408 = 1073741824 − 1408 = 1073740416。
#[test]
fn the_registered_points_carry_the_registered_geometry() {
    assert_eq!(point_named("base").unit_area_slots, 131_072);
    assert_eq!(point_named("unlinks_100000").unit_area_slots, 1_048_576);
    assert_eq!(point_named("files_100000").unit_area_slots, 1_048_576);
    assert_eq!(point_named("data_units_1000000").unit_area_slots, 4_194_304);
    assert_eq!(point_named("user_scale_many_rounds").unit_area_slots, 1_073_740_416);
    assert_eq!(point_named("files_10000").files, 10_000);
    assert_eq!(point_named("unlinks_1000").unlinks, 1_000);
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
        plain_device,
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
