//! E209（尖峰成因）的历史用例：拷进副本当 `crates/singlefs-verification/tests/e209_spike_cause_histories.rs`，
//! 副本由 `e209_spike_cause_counters.py` 套上读数（`--build counts`）或一个字节不插（`--build plain`，只比盘）。
//! 读数行怎么打在副本的 `tests/e209_readings/mod.rs`（两种构建各一份，都由插计数脚本写）；声明行（`name=declared`）与脚本步行（`name=script_step`）由这里打。
//! 脚本表、选择子怎么落到文件、声明行的写法在 `scripts/experiment-apparatus/e209_spike_cause_scripts.rs`（入口拷进副本的 `tests/e209_scripts/mod.rs`，单测在模型那个 bin 里跑）。
//!
//! 一个 `<点>__<臂>__<脚本>` 一条用例：同一个点的历史逐步相同（定种子，不随脚本变），放完历史之后在同一次挂载里连着做脚本的各步，
//! 每一步经 `Measured::measure` 量一次（重挂载不是用户操作，它自己的发布另记）。次序本身就是被量的，所以历史、脚本各步、重挂载串行、不拆。
//! 臂由环境变量 `E209_ARM` 在进程开头读一次，用例核它与自己点名的相同。历史那一段每过进度间隔打一行 `e209 progress`，判结果只读 `E209RESULT` 行。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]
#![allow(non_snake_case, reason = "用例名是 item_<点>__<臂>__<脚本>，入口按这个名字单跑")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;
#[path = "e209_readings/mod.rs"]
mod readings;
/// 脚本表与选择子：入口把 `scripts/experiment-apparatus/e209_spike_cause_scripts.rs` 拷到这里。
#[path = "e209_scripts/mod.rs"]
mod scripts;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use scripts::Declared;
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::transaction::PartialWriteBytes;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::scenario::e142_parameters;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// 6 MiB 环：单元区从槽 1408 起。
const JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 一次写至多切几个数据单元。
const DATA_UNITS_PER_WRITE_AT_MOST: u64 = 400;
/// 建 inode 一批至多这么多个。
const INODES_PER_BATCH: u64 = 100;
/// 进度间隔：只在历史那一段打。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(300);
/// 比盘的那一个项：训练点上的 `walk_forward`，历史末与脚本末各取一次 `image_sha256`。
const IMAGE_POINT: &str = scripts::TRAINING_POINT;
const IMAGE_SCRIPT: &str = "walk_forward";

type Pool = PoolUnderTest<SparseBlockDevice>;

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
    let end = debug_text.find(|character: char| !character.is_ascii_alphanumeric()).unwrap_or(debug_text.len());
    &debug_text[..end]
}

/// 拒绝的原因叫什么：发布被拒取里面 `cause` 的成员名，别的取拒绝自己的成员名。
fn refusal_cause_name(refusal: &UserChangeRefused) -> String {
    match refusal {
        UserChangeRefused::Publish { cause, .. } => variant_name(&format!("{cause:?}")).to_string(),
        other => variant_name(&format!("{other:?}")).to_string(),
    }
}

#[derive(Default)]
struct Tally {
    changes: u64,
    changes_refused: u64,
    floor_raises: u64,
}

/// 经会话发一次用户改动（被空间拒、推抬 F 由会话自己推），记被拒与抬 F 次数。
fn publish_change(pool: &mut Pool, tally: &mut Tally, kind: &'static str, change: UserChange<'_>) -> Result<(), RefusedChange> {
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
    /// inode 号 → 驱动按构造记的数据单元数（建时 0，写多少记多少，覆盖写后记新内容的 1 个，unlink 删掉）。
    static UNITS_OF: RefCell<BTreeMap<u64, u64>> = RefCell::new(BTreeMap::from([(1, scripts::FIRST_FILE_UNITS)]));
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
    set_units_of(inode, Some(1));
    Ok(())
}

/// 覆盖写第一个文件（`PoolUnderTest::overwrite`）。
fn overwrite_the_first_file(pool: &mut Pool) -> Result<(), RefusedChange> {
    drop(pool.overwrite(OVERWRITE_BYTES).map_err(|refusal| RefusedChange { kind: "overwrite", refusal })?);
    set_units_of(InodeNumber(1), Some(1));
    Ok(())
}

/// 按 inode 号从偏移 0 起接着写，每次至多 400 个数据单元，凑够 `data_units` 个。
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

/// 历史那一段的簿记：每做完一次用户改动取走读数（不让几百次发布的记录堆在内存里），每过一个进度间隔打一行进度。
struct History<'item> {
    item: &'item str,
    publishes_about: u64,
    user_changes: u64,
    last_progress: Instant,
    lines: readings::HistoryLines,
    tally: Tally,
}

impl History<'_> {
    fn note(&mut self) {
        self.user_changes += 1;
        self.take_readings();
        if self.last_progress.elapsed() >= PROGRESS_INTERVAL {
            eprintln!(
                "e209 progress item={} publishes_done={} of={} peak_rounds={}",
                self.item, self.lines.settle_calls, self.publishes_about, self.lines.peak_rounds
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

/// 历史（与 E207 的 `SmallFilesOfUnits` 点同一份）：大文件（建、写满）、F 个小文件（各建一次、写 K 个单元）、10 次「建空 inode 再删」。
/// 交回历史建好的 inode：第 0 个是大文件，其余是小文件按建的次序。
fn run_the_history(pool: &mut Pool, history: &mut History<'_>, files: u64, units_per_file: u64) -> Result<Vec<InodeNumber>, RefusedChange> {
    let big = create_inodes(pool, &mut history.tally, 1)?[0];
    history.note();
    fill_a_file(pool, &mut history.tally, big, scripts::BIG_FILE_UNITS)?;
    history.note();
    let mut live = vec![big];
    let mut remaining = files;
    while remaining > 0 {
        let count = remaining.min(INODES_PER_BATCH);
        for inode in create_inodes(pool, &mut history.tally, count)? {
            history.note();
            fill_a_file(pool, &mut history.tally, inode, units_per_file)?;
            history.note();
            live.push(inode);
        }
        remaining -= count;
    }
    for _ in 0..scripts::UNLINKS_OF_THE_HISTORY {
        let inode = create_inodes(pool, &mut history.tally, 1)?[0];
        history.note();
        unlink_inode(pool, &mut history.tally, inode)?;
        history.note();
    }
    Ok(live)
}

/// 历史被会话交回拒绝：打一行 `history_refused`，收完这一段的读数再停，这个项没有 `item_done`。
fn stop_on_a_refusal_in_the_history(history: &mut History<'_>, refused: &RefusedChange) -> ! {
    history.take_readings();
    for line in history.lines.close_the_bucket(history.item) {
        println!("{line}");
    }
    println!(
        "E209RESULT name=history_refused item={} change={} cause={} user_changes={} settle_calls={} changes={} changes_refused={} floor_raises={}",
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

/// 脚本里一次被量的操作的定位：读数标签 `phase=script mount_epoch=<序号> operation=<步> operation_index=<第几步>`。
struct Measured<'item> {
    item: &'item str,
}

impl Measured<'_> {
    fn labels(mount_epoch: u64, operation_name: &str, step_index: u64) -> String {
        format!("phase=script mount_epoch={mount_epoch} operation={operation_name} operation_index={step_index}")
    }

    /// 量一次操作：之前打声明行与规模读数，做完取走固定点读数、节点读数与 E203 的逐步读数；被拒照报、这个项没有 `item_done`。
    fn measure(
        &self,
        pool: &mut Pool,
        labels: &str,
        declared: Declared,
        change: impl FnOnce(&mut Pool, &mut Tally) -> Result<(), RefusedChange>,
    ) {
        println!("{}", scripts::declared_line(self.item, labels, declared));
        let state_line = readings::state_line(self.item, labels, pool);
        if !state_line.is_empty() {
            println!("{state_line}");
        }
        let outcome = change(&mut *pool, &mut Tally::default());
        let mut unused_history = readings::HistoryLines::default();
        for line in readings::take_settle_lines(self.item, labels, true, &mut unused_history) {
            println!("{line}");
        }
        for line in readings::take_node_lines() {
            println!("{line}");
        }
        let publish_prefix = labels.split(' ').filter(|word| !word.starts_with("phase=")).map(|word| word.split_once('=').map_or("", |pair| pair.1)).collect::<Vec<_>>().join("/");
        for line in readings::take_record_lines(self.item, labels, &publish_prefix) {
            println!("{line}");
        }
        if let Err(refused) = outcome {
            println!(
                "E209RESULT name=window_refused item={} {labels} change={} cause={}",
                self.item,
                refused.kind,
                refusal_cause_name(&refused.refusal)
            );
            panic!("脚本里 {} 被会话拒了：{:?}", refused.kind, refused.refusal);
        }
    }

    /// 重挂载不是用户操作：崩溃后可写挂载，它自己的发布（写行一次、暖机若干次）逐个打节点读数；E203 的逐步读数不要。
    fn remount(&self, pool: &mut Pool, mount_epoch: u64, step_index: u64) {
        pool.crash_and_mount_writable().expect("崩溃后可写挂载");
        readings::mount_changed();
        let labels = format!("phase=remount mount_epoch={mount_epoch} operation=remount operation_index={step_index}");
        let mut unused_history = readings::HistoryLines::default();
        for line in readings::take_settle_lines(self.item, &labels, true, &mut unused_history) {
            println!("{line}");
        }
        for line in readings::take_node_lines() {
            println!("{line}");
        }
        drop(readings::take_record_lines(self.item, &labels, "-"));
    }

    /// 脚本各步：每步先打 `script_step` 行；重挂载把后面各步的挂载序号加一。
    fn run_the_script(&self, pool: &mut Pool, live: &[InodeNumber], steps: &[scripts::Step]) {
        let live_numbers: Vec<u64> = live.iter().map(|inode| inode.0).collect();
        let mut mount_epoch = 0;
        for (position, step) in steps.iter().copied().enumerate() {
            let step_index = u64::try_from(position).expect("下标") + 1;
            println!("{}", scripts::script_step_line(self.item, step_index, step, mount_epoch));
            let name = step.text();
            let labels = Self::labels(mount_epoch, &name, step_index);
            match step {
                scripts::Step::Overwrite(selector) => {
                    let inode = InodeNumber(scripts::resolve_selector(selector, &live_numbers));
                    let declared = Declared::overwrite(units_of(inode));
                    if inode.0 == 1 {
                        self.measure(pool, &labels, declared, |pool, _| overwrite_the_first_file(pool));
                    } else {
                        self.measure(pool, &labels, declared, |pool, tally| overwrite_inode(pool, tally, inode));
                    }
                }
                scripts::Step::CreateOne => {
                    self.measure(pool, &labels, Declared::create(1), |pool, tally| create_inodes(pool, tally, 1).map(|_| ()));
                }
                scripts::Step::TruncateToTheSameSize => {
                    // 阴性对照：没有普通分配的发布。把第一个文件截成它现在的大小（只改记录）。
                    self.measure(pool, &labels, Declared::truncate_to_the_same_size(), |pool, tally| {
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
                scripts::Step::Remount => {
                    mount_epoch += 1;
                    self.remount(pool, mount_epoch, step_index);
                }
            }
        }
    }
}

fn replay_the_history_and_run_the_script(point_name: &str, arm: &str, script_name: &str) {
    assert_eq!(std::env::var("E209_ARM").as_deref(), Ok(arm), "E209_ARM 要与用例点名的臂相同：入口按臂分进程起");
    assert_eq!(arm == "plain", readings::build_name() == "plain", "plain 这条对照只在 --build plain 的副本里跑");
    let item = scripts::item_name(point_name, arm, script_name);
    let point = scripts::sample_point_named(point_name).unwrap_or_else(|| panic!("取样点表里没有 {point_name}"));
    let steps = scripts::script_steps(script_name).unwrap_or_else(|| panic!("脚本表里没有 {script_name}"));
    let unit_area_slots = 1_u64 << point.unit_area_exponent;
    println!(
        "E209RESULT name=point item={item} point={point_name} arm={arm} script={script_name} build={} unit_area_slots={unit_area_slots} unlinks={} data_units={} files={} units_per_small_file={} shape=small_files_of_units",
        readings::build_name(),
        scripts::UNLINKS_OF_THE_HISTORY,
        scripts::BIG_FILE_UNITS,
        point.files,
        point.units_per_file
    );
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + unit_area_slots) * SLOT_BYTES,
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
        publishes_about: scripts::notes_of_the_history(point.files),
        user_changes: 0,
        last_progress: Instant::now(),
        lines: readings::HistoryLines::default(),
        tally: Tally::default(),
    };
    let live = match run_the_history(&mut pool, &mut history, point.files, point.units_per_file) {
        Ok(live) => live,
        Err(refused) => stop_on_a_refusal_in_the_history(&mut history, &refused),
    };
    history.take_readings();
    for line in history.lines.close_the_bucket(&item) {
        println!("{line}");
    }
    println!(
        "E209RESULT name=history item={item} user_changes={} settle_calls={} peak_rounds={} peak_round_work={} changes={} changes_refused={} floor_raises={}",
        history.user_changes,
        history.lines.settle_calls,
        history.lines.peak_rounds,
        history.lines.peak_round_work,
        history.tally.changes,
        history.tally.changes_refused,
        history.tally.floor_raises
    );
    let compared_by_the_disk = point_name == IMAGE_POINT && script_name == IMAGE_SCRIPT;
    if compared_by_the_disk {
        println!("E209RESULT name=image_sha256 item={item} when=history_end sha256={}", image_sha256(&pool.image()));
    }
    let carried_line = readings::history_carried_line(&item);
    if !carried_line.is_empty() {
        println!("{carried_line}");
    }
    readings::pause_readings(false);
    readings::set_window(true);
    Measured { item: &item }.run_the_script(&mut pool, &live, &steps);
    readings::set_window(false);
    if compared_by_the_disk {
        println!("E209RESULT name=image_sha256 item={item} when=script_end sha256={}", image_sha256(&pool.image()));
    }
    println!("E209RESULT name=item_done item={item}");
}

// ---- 下面每个 `<点>__<臂>__<脚本>` 一条用例；名单与入口 e209-spike-cause-run.sh 的 items_table 同一份（入口自证核两边相同）----

#[test]
fn item_files_100_units_100__head__walk_forward() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "walk_forward");
}

#[test]
fn item_files_100_units_100__head__walk_backward() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "walk_backward");
}

#[test]
fn item_files_100_units_100__head__walk_from_middle() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "walk_from_middle");
}

#[test]
fn item_files_100_units_100__head__first_repeated() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "first_repeated");
}

#[test]
fn item_files_100_units_100__head__create_then_first() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "create_then_first");
}

#[test]
fn item_files_100_units_100__head__filler_create_1() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_create_1");
}

#[test]
fn item_files_100_units_100__head__filler_create_2() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_create_2");
}

#[test]
fn item_files_100_units_100__head__filler_create_4() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_create_4");
}

#[test]
fn item_files_100_units_100__head__filler_create_8() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_create_8");
}

#[test]
fn item_files_100_units_100__head__filler_create_16() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_create_16");
}

#[test]
fn item_files_100_units_100__head__filler_young_1() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_young_1");
}

#[test]
fn item_files_100_units_100__head__filler_young_4() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_young_4");
}

#[test]
fn item_files_100_units_100__head__filler_young_8() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "filler_young_8");
}

#[test]
fn item_files_100_units_100__head__presweep_1() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_1");
}

#[test]
fn item_files_100_units_100__head__presweep_2() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_2");
}

#[test]
fn item_files_100_units_100__head__presweep_4() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_4");
}

#[test]
fn item_files_100_units_100__head__presweep_8() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_8");
}

#[test]
fn item_files_100_units_100__head__presweep_16() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_16");
}

#[test]
fn item_files_100_units_100__head__presweep_32() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_32");
}

#[test]
fn item_files_100_units_100__head__presweep_64() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "presweep_64");
}

#[test]
fn item_files_100_units_100__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "mount_then_first");
}

#[test]
fn item_files_100_units_100__head__first_mount_first() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "first_mount_first");
}

#[test]
fn item_files_100_units_100__head__mount_twice_first() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "mount_twice_first");
}

#[test]
fn item_files_100_units_100__head__young_mount_first() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "young_mount_first");
}

#[test]
fn item_files_100_units_100__head__old_mount_old() {
    replay_the_history_and_run_the_script("files_100_units_100", "head", "old_mount_old");
}

#[test]
fn item_files_100_units_100__plain__walk_forward() {
    replay_the_history_and_run_the_script("files_100_units_100", "plain", "walk_forward");
}

#[test]
fn item_files_75_units_100__head__walk_forward() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "walk_forward");
}

#[test]
fn item_files_75_units_100__head__walk_backward() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "walk_backward");
}

#[test]
fn item_files_75_units_100__head__first_repeated() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "first_repeated");
}

#[test]
fn item_files_75_units_100__head__filler_create_4() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "filler_create_4");
}

#[test]
fn item_files_75_units_100__head__filler_young_4() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "filler_young_4");
}

#[test]
fn item_files_75_units_100__head__presweep_8() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "presweep_8");
}

#[test]
fn item_files_75_units_100__head__presweep_32() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "presweep_32");
}

#[test]
fn item_files_75_units_100__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "mount_then_first");
}

#[test]
fn item_files_75_units_100__head__first_mount_first() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "first_mount_first");
}

#[test]
fn item_files_75_units_100__head__mount_twice_first() {
    replay_the_history_and_run_the_script("files_75_units_100", "head", "mount_twice_first");
}

#[test]
fn item_files_200_units_100__head__walk_forward() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "walk_forward");
}

#[test]
fn item_files_200_units_100__head__walk_backward() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "walk_backward");
}

#[test]
fn item_files_200_units_100__head__first_repeated() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "first_repeated");
}

#[test]
fn item_files_200_units_100__head__filler_create_4() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "filler_create_4");
}

#[test]
fn item_files_200_units_100__head__filler_young_4() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "filler_young_4");
}

#[test]
fn item_files_200_units_100__head__presweep_8() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "presweep_8");
}

#[test]
fn item_files_200_units_100__head__presweep_32() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "presweep_32");
}

#[test]
fn item_files_200_units_100__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "mount_then_first");
}

#[test]
fn item_files_200_units_100__head__first_mount_first() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "first_mount_first");
}

#[test]
fn item_files_200_units_100__head__mount_twice_first() {
    replay_the_history_and_run_the_script("files_200_units_100", "head", "mount_twice_first");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__walk_forward() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "walk_forward");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__walk_backward() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "walk_backward");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__first_repeated() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "first_repeated");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__filler_create_4() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "filler_create_4");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__filler_young_4() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "filler_young_4");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__presweep_8() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "presweep_8");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__presweep_32() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "presweep_32");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "mount_then_first");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__first_mount_first() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "first_mount_first");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_17__head__mount_twice_first() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_17", "head", "mount_twice_first");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__walk_forward() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "walk_forward");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__walk_backward() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "walk_backward");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__first_repeated() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "first_repeated");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__filler_create_4() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "filler_create_4");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__filler_young_4() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "filler_young_4");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__presweep_8() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "presweep_8");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__presweep_32() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "presweep_32");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "mount_then_first");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__first_mount_first() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "first_mount_first");
}

#[test]
fn item_files_100_units_100_unit_area_2_pow_20__head__mount_twice_first() {
    replay_the_history_and_run_the_script("files_100_units_100_unit_area_2_pow_20", "head", "mount_twice_first");
}

#[test]
fn item_files_10_units_1000__head__walk_forward() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "walk_forward");
}

#[test]
fn item_files_10_units_1000__head__walk_backward() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "walk_backward");
}

#[test]
fn item_files_10_units_1000__head__first_repeated() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "first_repeated");
}

#[test]
fn item_files_10_units_1000__head__filler_create_4() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "filler_create_4");
}

#[test]
fn item_files_10_units_1000__head__filler_young_4() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "filler_young_4");
}

#[test]
fn item_files_10_units_1000__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "mount_then_first");
}

#[test]
fn item_files_10_units_1000__head__first_mount_first() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "first_mount_first");
}

#[test]
fn item_files_10_units_1000__head__mount_twice_first() {
    replay_the_history_and_run_the_script("files_10_units_1000", "head", "mount_twice_first");
}

#[test]
fn item_files_250_units_100__head__walk_forward() {
    replay_the_history_and_run_the_script("files_250_units_100", "head", "walk_forward");
}

#[test]
fn item_files_250_units_100__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_250_units_100", "head", "mount_then_first");
}

#[test]
fn item_files_300_units_100__head__walk_forward() {
    replay_the_history_and_run_the_script("files_300_units_100", "head", "walk_forward");
}

#[test]
fn item_files_300_units_100__head__mount_then_first() {
    replay_the_history_and_run_the_script("files_300_units_100", "head", "mount_then_first");
}

// ---- 钉绝对值的断言：数由登记与格式常量另外算出，不从上面的表与函数读回来；各条抄自哪里、怎么算的写在断言旁边 ----

/// 6 MiB 环的单元区起点：环从槽 1024 起（`singlefs-format` 的 `JOURNAL_RING_START_SLOT`），6 MiB / 16 KiB（`SLOT_BYTES`）= 384 槽，1024 + 384 = 1408；
/// 1408 = 22 × 64，落在聚簇段边界上。
#[test]
fn the_six_mebibyte_ring_puts_the_unit_area_at_slot_1408() {
    assert_eq!(JOURNAL_RING_BYTES, 6_291_456);
    assert_eq!(parameters().geometry.journal_ring_bytes, 6_291_456);
    assert_eq!(unit_area_start_slot(), 1408);
}

/// 训练点 `files_100_units_100` 的几何，抄登记：U = 2¹⁹，F = 100，每个小文件 100 个数据单元；历史的用户改动序号数 = 大文件 2 + 小文件 2 × 100 + 删 2 × 10 = 222。
#[test]
fn the_training_point_carries_the_registered_geometry_and_note_count() {
    let point = scripts::sample_point_named("files_100_units_100").expect("训练点在取样点表里");
    assert_eq!(point.unit_area_exponent, 19);
    assert_eq!(point.files, 100);
    assert_eq!(point.units_per_file, 100);
    assert_eq!(scripts::notes_of_the_history(point.files), 222);
}

/// 一段小历史（10 个小文件、每个 1000 个数据单元）各步的用户改动数，另外数：
/// 大文件 = 建 1 个 inode + 一次写（10 个单元不超过一次至多 400 个）= 2 次；小文件 = 一批建 10 个 inode（至多 100 个一批）1 次，
/// 各写 1000 个单元要 400 + 400 + 200 = 3 次写，10 × 3 = 30 次；10 次删各建一个空 inode 再 unlink = 20 次；合 2 + 31 + 20 = 53 次发布用户改动，一次都没被拒、没抬 F。
/// `History::note` 每个 inode 建好与做完各叫一次：大文件 2、小文件 10 × 2 = 20、删 10 × 2 = 20，合 42 次。驱动自己记的数据单元数：大文件 10，每个小文件 1000。
#[test]
fn small_history_makes_fifty_three_user_changes_notes_forty_two_steps_and_keeps_the_units_it_wrote() {
    let item = "pin__head__walk_forward";
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + (1_u64 << 17)) * SLOT_BYTES,
        plain_device,
    );
    readings::set_window(false);
    readings::pause_readings(true);
    let mut history = History {
        item,
        publishes_about: 42,
        user_changes: 0,
        last_progress: Instant::now(),
        lines: readings::HistoryLines::default(),
        tally: Tally::default(),
    };
    let live = match run_the_history(&mut pool, &mut history, 10, 1_000) {
        Ok(live) => live,
        Err(refused) => panic!("小历史被会话拒了：{}", refusal_cause_name(&refused.refusal)),
    };
    assert_eq!(history.tally.changes, 53);
    assert_eq!(history.user_changes, 42);
    assert_eq!(history.tally.changes_refused, 0);
    assert_eq!(history.tally.floor_raises, 0);
    assert_eq!(live.len(), 11);
    assert_eq!(units_of(live[0]), 10);
    assert_eq!(units_of(live[1]), 1_000);
    assert_eq!(units_of(live[10]), 1_000);
}
