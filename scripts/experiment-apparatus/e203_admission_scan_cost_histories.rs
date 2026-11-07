//! E203（准入之前扫类的步代价）的历史用例：拷进副本当 `crates/singlefs-verification/tests/e203_admission_scan_cost_histories.rs`，
//! 副本由 `e203_admission_scan_cost_counters.py` 套上计数（`--build counts`）或计时（`--build timing`）；两种构建里这一份源文件逐字相同。
//!
//! 一个取样点一条用例（跑前登记第五节「取样点」）：mkfs、取号、暖机、第一个文件 → 大文件 D 个数据单元 → F 个小文件 → u 次 unlink →
//! 同一次挂载里量 → `crash_and_mount_writable` 之后再量。同一个点的两种挂载状态共用前面那一段历史，写在同一条用例里（后一步读前一步留下的那一版）。
//! 每一轮量三种发布：覆盖写第一个文件、unlink 的意图那次与最后一批。打 `E203RESULT` 行，产物格式见模型 `e203_admission_scan_cost.rs` 的 `read_product`。
//! 环境变量 `E203_BUILD`（counts 或 timing）写进产物行；没设拒跑。
//!
//! 历史那一段（登记第五节历史第 1–4 步）经会话发布，被空间拒、推抬 F 的由会话自己推（D8（核心索引结构） 已定项 16 第 6 条），
//! 按第 8 步报每种用户改动的次数、被拒次数、抬 F 次数（`E203RESULT name=history_changes`）。会话交回的拒绝（推满仍不够，或不是空间不够、
//! 不推抬 F 的那几种，例如合取 7 全空聚簇段少于 `N₀`）登记没写怎么接着走：打一行 `E203RESULT name=history_refused`（拒的是哪一种改动、
//! 原因、那一刻的占用读数）再停，那个点没有 `item_done`。
//! 历史那一段不量，整段停着发布路径上逐步的读数（`with_the_readings_paused`，只省挂钟，判了几次与发给会话的改动不变）；被量的那几轮照旧全记。
//!
//! Y1 那一点的单元区由登记的 2¹⁴ 加大，2¹⁶、2¹⁷ 两档各一条用例（点名写明 U），用哪一档由探针定。
//! 另有一条探针用例（`probe_of_the_empty_segments_refusal_on_many_unlinks`）：不进产物、入口不点名它，单元区由环境变量给（不给取 2¹⁴），
//! 报全空聚簇段数随删的次数的轨迹与外推到 0 的那一次，判 Y1 的 u 装不装得下；打 `E203PROBE` 行。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::{AllocationRecord, UnitAreaStart};
use singlefs_core::code_two_tree::CodeTwoTreeKey;
use singlefs_core::inode_tree::leaf_containers_one_root_node_holds;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::records::AccountingEntry;
use singlefs_core::transaction::e203_scan::{self, PublishRecord};
use singlefs_core::transaction::{
    MultiLevelCodeTwoTree, PartialWriteBytes, TransactionUnit,
    ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, INODE_LEAF_RECORDS, SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS,
};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::e142_parameters;
use std::time::{Duration, Instant};

/// 6 MiB 环：单元区从槽 1408 起（同攻方模型那一档）。
const JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 一次写至多切几个数据单元：比一次发布 512 条记录的上限留出余量。
const DATA_UNITS_PER_WRITE_AT_MOST: u64 = 400;
/// 建 inode 一批至多这么多个。
const INODES_PER_BATCH: u64 = 100;
/// 被量的轮数；被量的发布判了不止一次时往后补，补满这么多轮为止。
const ROUNDS: u64 = 5;
const EXTRA_ROUNDS_AT_MOST: u64 = 5;

/// 一个取样点：(U, u, D, F)，登记第五节。
#[derive(Clone, Copy)]
struct Axes {
    unit_area_slots: u64,
    unlinks: u64,
    data_units: u64,
    files: u64,
}

const BASE: Axes = Axes {
    unit_area_slots: 1 << 17,
    unlinks: 10,
    data_units: 10,
    files: 10,
};

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

type Pool = PoolUnderTest<SparseBlockDevice>;

fn build_name() -> String {
    std::env::var("E203_BUILD").expect("E203_BUILD 要设成 counts 或 timing")
}

fn next_inode(pool: &Pool) -> InodeNumber {
    InodeNumber(pool.current_file_version().inode_number_watermark())
}

/// 建 `count` 个 inode，次数记进 `tally`；会话交回拒绝就原样交回。
fn create_inodes_counted(
    pool: &mut Pool,
    tally: &mut ChangeTally,
    count: u64,
) -> Result<Vec<InodeNumber>, RefusedChange> {
    let first = next_inode(pool).0;
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    tally.publish(
        pool,
        ChangeKind::NewInodes,
        UserChange::NewInodes {
            count,
            write_time_seconds,
        },
    )?;
    Ok((first..first + count).map(InodeNumber).collect())
}

fn overwrite_inode_counted(
    pool: &mut Pool,
    tally: &mut ChangeTally,
    inode: InodeNumber,
) -> Result<(), RefusedChange> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    tally.publish(
        pool,
        ChangeKind::OverwriteOfAnInode,
        UserChange::OverwriteOfAnInode {
            inode,
            content: &content,
            write_time_seconds,
        },
    )
}

/// 大文件：按 inode 号从偏移 0 起接着写，每次至多 400 个数据单元，凑够 `data_units` 个。
fn fill_the_big_file_counted(
    pool: &mut Pool,
    tally: &mut ChangeTally,
    inode: InodeNumber,
    data_units: u64,
) -> Result<(), RefusedChange> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷装得进 u64");
    let mut written = 0;
    while written < data_units {
        let units = (data_units - written).min(DATA_UNITS_PER_WRITE_AT_MOST);
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        let bytes = content_of(
            usize::try_from(units * payload).expect("几兆字节"),
            write_time_seconds,
        );
        tally.publish(
            pool,
            ChangeKind::WriteAtOffset,
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

fn unlink_counted(
    pool: &mut Pool,
    tally: &mut ChangeTally,
    inode: InodeNumber,
) -> Result<(), RefusedChange> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    tally.publish(
        pool,
        ChangeKind::Unlink,
        UserChange::Unlink {
            inode,
            write_time_seconds,
        },
    )
}

// 被量的那几轮（第 5、6 步）用下面三个：次数不进第 8 步的报数，会话交回拒绝就停。
fn create_inodes(pool: &mut Pool, count: u64) -> Vec<InodeNumber> {
    create_inodes_counted(pool, &mut ChangeTally::default(), count)
        .unwrap_or_else(|refused| panic!("建 inode 发成: {:?}", refused.refusal))
}

fn overwrite_inode(pool: &mut Pool, inode: InodeNumber) {
    overwrite_inode_counted(pool, &mut ChangeTally::default(), inode)
        .unwrap_or_else(|refused| panic!("按 inode 号覆盖写发成: {:?}", refused.refusal));
}

fn unlink(pool: &mut Pool, inode: InodeNumber) {
    unlink_counted(pool, &mut ChangeTally::default(), inode)
        .unwrap_or_else(|refused| panic!("unlink 发成: {:?}", refused.refusal));
}

/// 进度间隔：一个点的历史跑得超过这么久的（第二段 10⁵ 次删那一点推的要几个小时），每过一个间隔往标准错误打一行进度。
/// 那一行不以 `E203RESULT` 起头，入口写产物时不收；它只进副本里的原始输出。只在历史那一段打，被量的那几轮不经这里。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(300);

/// 历史里每做完一次用户改动就把收好的记录取走（不让几万次发布的读数堆在内存里），记判了几次；每过一个进度间隔打一行进度。
struct History {
    point: String,
    user_changes: u64,
    judgements: u64,
    last_progress: Instant,
    /// 登记第五节历史第 8 步的次数：每种用户改动交给会话几次（发成的与被拒的都算）、会话交回拒绝几次、
    /// 会话为空间推了几串抬 F、推那几串之前被拒几次（发成的改动里推的与交回的拒绝里已推的都算）。
    tally: ChangeTally,
}

#[derive(Default)]
struct ChangeTally {
    new_inode_changes: u64,
    overwrite_changes: u64,
    write_at_offset_changes: u64,
    unlink_changes: u64,
    changes_refused: u64,
    floor_raises: u64,
    refusals_that_pushed_the_floor_raises: u64,
}

/// 历史里的一种用户改动（第 8 步按种报次数）。
#[derive(Clone, Copy, Debug)]
enum ChangeKind {
    NewInodes,
    OverwriteOfAnInode,
    WriteAtOffset,
    Unlink,
}

impl ChangeKind {
    fn product_name(self) -> &'static str {
        match self {
            Self::NewInodes => "new_inodes",
            Self::OverwriteOfAnInode => "overwrite_of_an_inode",
            Self::WriteAtOffset => "write_at_offset",
            Self::Unlink => "unlink",
        }
    }
}

/// 一次被会话交回的拒绝：哪一种改动、会话交回的原样。
struct RefusedChange {
    kind: ChangeKind,
    refusal: UserChangeRefused,
}

/// 交回的拒绝里已经推过的抬 F 串数（推满仍不够、推的那一串自己报错、走空推过之后被别的理由拒、删完之后走空推过）。
fn floor_raises_inside_the_refusal(refusal: &UserChangeRefused) -> usize {
    match refusal {
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) => no_space.floor_raises.len(),
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
            failed.floor_raises.len()
        }
        UserChangeRefused::Publish { floor_raises, .. }
        | UserChangeRefused::NoFileVersionToChange { floor_raises, .. } => floor_raises.len(),
        _ => 0,
    }
}

/// `Debug` 文字打头的那个名字（枚举成员名）：产物行里只写原因叫什么，不把整份 `Debug` 塞进一个字段。
fn variant_name(debug_text: &str) -> &str {
    let end = debug_text
        .find(|character: char| !character.is_ascii_alphanumeric())
        .unwrap_or(debug_text.len());
    &debug_text[..end]
}

/// 拒绝的原因叫什么：发布被拒（`Publish`）取里面 `cause` 的成员名，别的取拒绝自己的成员名。
fn refusal_cause_name(refusal: &UserChangeRefused) -> String {
    match refusal {
        UserChangeRefused::Publish { cause, .. } => variant_name(&format!("{cause:?}")).to_string(),
        other => variant_name(&format!("{other:?}")).to_string(),
    }
}

impl ChangeTally {
    /// 一次用户改动：经会话发（会话为空间推抬 F，D8（核心索引结构） 已定项 16 第 6 条），按第 8 步记次数；会话交回拒绝就原样交回。
    fn publish(
        &mut self,
        pool: &mut Pool,
        kind: ChangeKind,
        change: UserChange<'_>,
    ) -> Result<(), RefusedChange> {
        match kind {
            ChangeKind::NewInodes => self.new_inode_changes += 1,
            ChangeKind::OverwriteOfAnInode => self.overwrite_changes += 1,
            ChangeKind::WriteAtOffset => self.write_at_offset_changes += 1,
            ChangeKind::Unlink => self.unlink_changes += 1,
        }
        match pool.publish(change, None) {
            Ok(published) => {
                self.floor_raises += u64::try_from(published.floor_raises.len()).expect("串数");
                self.refusals_that_pushed_the_floor_raises +=
                    u64::try_from(published.refusals_that_pushed_the_floor_raises.len())
                        .expect("次数");
                Ok(())
            }
            Err(refusal) => {
                self.changes_refused += 1;
                // 交回的拒绝里推过的每一串都是因为一次被拒才推的（两张表一一对应）。
                let raised =
                    u64::try_from(floor_raises_inside_the_refusal(&refusal)).expect("串数");
                self.floor_raises += raised;
                self.refusals_that_pushed_the_floor_raises += raised;
                Err(RefusedChange { kind, refusal })
            }
        }
    }

    fn text(&self) -> String {
        format!(
            "new_inode_changes={} overwrite_changes={} write_at_offset_changes={} unlink_changes={} changes_refused={} floor_raises={} \
             refusals_that_pushed_the_floor_raises={}",
            self.new_inode_changes,
            self.overwrite_changes,
            self.write_at_offset_changes,
            self.unlink_changes,
            self.changes_refused,
            self.floor_raises,
            self.refusals_that_pushed_the_floor_raises
        )
    }
}

impl History {
    fn new(point: &str) -> Self {
        Self {
            point: point.to_string(),
            user_changes: 0,
            judgements: 0,
            last_progress: Instant::now(),
            tally: ChangeTally::default(),
        }
    }

    fn note(&mut self) {
        self.user_changes += 1;
        self.judgements += u64::try_from(e203_scan::take_finished().len()).expect("条数");
        if self.last_progress.elapsed() >= PROGRESS_INTERVAL {
            eprintln!(
                "e203 progress point={} user_changes={} judgements={}",
                self.point, self.user_changes, self.judgements
            );
            self.last_progress = Instant::now();
        }
    }
}

fn emit_record(
    build: &str,
    point: &str,
    state: &str,
    kind: &str,
    round: u64,
    record: &PublishRecord,
) {
    for row in &record.rows {
        println!(
            "E203RESULT name=row build={build} point={point} state={state} kind={kind} round={round} segment={} row={} calls={} steps={} scale={} steps_to_the_hit={} scale_to_the_hit={} exclusive_nanos={} queries={} query_levels={}",
            row.segment, row.row, row.calls, row.steps, row.scale, row.steps_to_the_hit, row.scale_to_the_hit, row.exclusive_nanos, row.queries, row.query_levels
        );
    }
    for (segment, nanos) in &record.segment_nanos {
        println!("E203RESULT name=segment_nanos build={build} point={point} state={state} kind={kind} round={round} segment={segment} nanos={nanos}");
    }
    for delta in &record.deltas {
        println!(
            "E203RESULT name=delta build={build} point={point} state={state} kind={kind} round={round} segment={} fixed_point_round={} key={} value={}",
            delta.segment, delta.round, delta.key, delta.value
        );
    }
    println!(
        "E203RESULT name=publish build={build} point={point} state={state} kind={kind} round={round} had_caller_segment={} fixed_point_rounds={}",
        record.had_caller_segment, record.rounds
    );
}

/// 这一刻的规模读数（登记第五节共用项表第三行）的一行，不打：被量的发布做之前取（那是它扫的那一版），做完再跟着它的记录一起打。
/// 两个构建各打一行，两边逐字段相同。树高从各树根节点的码 2 头里现读（`height_read_from_the_root_node_header`）。
fn state_text(
    build: &str,
    point: &str,
    state: &str,
    kind: &str,
    round: u64,
    pool: &Pool,
) -> String {
    let version = pool.current_file_version();
    let allocator = &pool.session().allocator;
    let heights = version.position_addressed_tree_heights_read_from_the_root_node_headers();
    format!(
        "E203RESULT name=state build={build} point={point} state={state} kind={kind} round={round} units={} mapped_units={} allocation_records={} cluster_segments={} \
         other_files={} tombstone_containers={} inode_leaf_containers={} accounting_rows={} devices={} whole_copy_elements={} deadlist_entries={} extent_upper_nodes={} \
         extent_upper_levels={} accounting_tree_levels={} deadlist_tree_levels={} allocation_tree_levels={} central_mapping_levels={}",
        version.units.len(),
        version.mapped_units.len(),
        allocator.records().len(),
        allocator.e203_cluster_segments_opened(),
        version.files_other_than_the_first.len(),
        version.tombstone_containers.len(),
        version.inode_leaf_containers.len(),
        version.accounting_entries.len(),
        allocator.devices.len(),
        allocator.e203_whole_copy_elements(),
        version.deadlist_entries.len(),
        version.extent_tree.upper_nodes.len(),
        heights.extent_tree_upper_segment,
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::Accounting),
        // deadlist 树没有条目时是零根、没有根单元可读：高记 0（它进出的 key 数也是 0）。
        if version.deadlist_entries.is_empty() {
            0
        } else {
            version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::Deadlist)
        },
        heights.allocation_record_tree,
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping),
    )
}

/// 一块盘上全空聚簇段数，从分配记录现数：单元区里每 64 槽一段（只数凑满 64 槽的整段），段里没有一条罩到它的未释放记录就算全空。
/// 与分配器增量维护的那一个（`DeviceFreeMap::empty_segments`，合取 7 读的就是它）同一个记录口径：defer 里已释放的槽不算占用。
/// `records` 是这块盘上每条记录的（起点槽，跨度，已释放）；`unit_area_start_slot` 是单元区第一个槽。
fn empty_segments_counted_from_the_records(
    records: impl Iterator<Item = (u64, u64, bool)>,
    unit_area_start_slot: u64,
    unit_area_slots: u64,
) -> u64 {
    let segments = unit_area_slots / CLUSTER_SEGMENT_SLOTS;
    let mut held = vec![false; usize::try_from(segments).expect("段数")];
    for (slot, span, released) in records {
        if released {
            continue;
        }
        let first = (slot - unit_area_start_slot) / CLUSTER_SEGMENT_SLOTS;
        let last = (slot + span.max(1) - 1 - unit_area_start_slot) / CLUSTER_SEGMENT_SLOTS;
        for segment in first..=last.min(segments.saturating_sub(1)) {
            held[usize::try_from(segment).expect("段号")] = true;
        }
    }
    u64::try_from(held.iter().filter(|segment_held| !**segment_held).count()).expect("段数")
}

/// 占用读数（Y1 那一点被合取 7 拒时要的那几样）：每块盘（按分配器里的次序，逗号隔）的全空聚簇段数——分配器增量维护的那一个与从记录现数的那一个——、
/// 有槽占着的段数（含 defer 里的槽，开段看的是它）、未释放记录数、已分配槽数、defer 槽数、未释放记录里落点不是这一版 `units` 里任何一个单元的那几条；
/// 再加这一版的单元数、映射里的单元数、墓碑容器数、记账行数、inode 号水位的台阶行数、deadlist 里还没释放的条数、inode 号水位、分配记录总条数、
/// 提交内生块回落到最低空槽的次数。只读，不动会话。
fn occupancy_text(pool: &Pool) -> String {
    let version = pool.current_file_version();
    let allocator = &pool.session().allocator;
    let unit_slots: std::collections::BTreeSet<u64> =
        version.units.iter().map(|unit| unit.slot.0).collect();
    let two_ways = empty_segments_two_ways(pool);
    let by_the_allocator: Vec<String> = two_ways
        .iter()
        .map(|(counted_by_the_allocator, _)| counted_by_the_allocator.to_string())
        .collect();
    let from_the_records: Vec<String> = two_ways
        .iter()
        .map(|(_, recount)| recount.to_string())
        .collect();
    let mut segments_with_a_slot_taken = Vec::new();
    let mut unreleased_records = Vec::new();
    let mut allocated_slots = Vec::new();
    let mut deferred_slots = Vec::new();
    let mut unreleased_records_not_at_a_unit_slot = Vec::new();
    for device_map in &allocator.devices {
        let device = device_map.device;
        let of_this_device = move || {
            allocator
                .records()
                .iter()
                .filter(move |record| record.device == device)
        };
        let segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
        segments_with_a_slot_taken.push(
            (0..usize::try_from(segments).expect("段数"))
                .filter(|segment| !device_map.segment_is_empty(*segment))
                .count()
                .to_string(),
        );
        unreleased_records.push(
            of_this_device()
                .filter(|record| !record.is_released)
                .count()
                .to_string(),
        );
        allocated_slots.push(device_map.allocated_slots().to_string());
        deferred_slots.push(device_map.deferred_slots().to_string());
        unreleased_records_not_at_a_unit_slot.push(
            of_this_device()
                .filter(|record| !record.is_released && !unit_slots.contains(&record.slot.0))
                .count()
                .to_string(),
        );
    }
    format!(
        "empty_segments_by_the_allocator={} empty_segments_from_the_records={} segments_with_a_slot_taken={} unreleased_records={} \
         allocated_slots={} deferred_slots={} unreleased_records_not_at_a_unit_slot={} units={} mapped_units={} tombstone_containers={} \
         accounting_rows={} inode_watermark_steps={} deadlist_entries={} inode_watermark={} allocation_records={} commit_generated_fallbacks={}",
        by_the_allocator.join(","),
        from_the_records.join(","),
        segments_with_a_slot_taken.join(","),
        unreleased_records.join(","),
        allocated_slots.join(","),
        deferred_slots.join(","),
        unreleased_records_not_at_a_unit_slot.join(","),
        version.units.len(),
        version.mapped_units.len(),
        version.tombstone_containers.len(),
        version.accounting_entries.len(),
        version.inode_number_watermark_steps().len(),
        version.deadlist_entries.len(),
        version.inode_number_watermark(),
        allocator.records().len(),
        allocator.commit_generated_placements_that_fell_back_to_the_lowest_free_slot()
    )
}

/// 量一轮覆盖写（第一个文件）；判了不止一次的那一轮不算（登记第五节历史第 8 步），返回 `None`。
fn measure_an_overwrite(pool: &mut Pool) -> Option<PublishRecord> {
    drop(e203_scan::take_finished());
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
    let mut records = e203_scan::take_finished();
    (records.len() == 1).then(|| records.remove(0))
}

/// 量一轮 unlink：建 Y、按 inode 号覆盖写 Y 一个单元（不量）、unlink Y（量）。交回意图那次的规模读数与 unlink 的各次发布。
/// 规模读数在 Y 建好、写好之后、unlink 之前取：意图那次扫的就是这一版（Y 已是一个别的文件，写 Y 的那几次发布可能已新开聚簇段）。
fn measure_an_unlink(
    build: &str,
    point: &str,
    state: &str,
    round: u64,
    pool: &mut Pool,
) -> (String, Vec<PublishRecord>) {
    let inode = create_inodes(pool, 1)[0];
    overwrite_inode(pool, inode);
    let scanned_by_the_intent = state_text(build, point, state, "unlink_intent", round, pool);
    drop(e203_scan::take_finished());
    unlink(pool, inode);
    (scanned_by_the_intent, e203_scan::take_finished())
}

fn measure(build: &str, point: &str, state: &str, pool: &mut Pool) {
    let mut taken = 0;
    let mut attempts = 0;
    while taken < ROUNDS && attempts < ROUNDS + EXTRA_ROUNDS_AT_MOST {
        attempts += 1;
        // 规模读数取被量的发布做之前那一版的（它扫的就是那一版）；这一次被拒重判的不算，丢掉。
        let scanned = state_text(build, point, state, "overwrite", taken + 1, pool);
        match measure_an_overwrite(pool) {
            Some(record) => {
                taken += 1;
                emit_record(build, point, state, "overwrite", taken, &record);
                println!("{scanned}");
            }
            None => println!("E203RESULT name=not_measurable build={build} point={point} state={state} kind=overwrite attempt={attempts}"),
        }
    }
    println!("E203RESULT name=rounds_taken build={build} point={point} state={state} kind=overwrite taken={taken} attempts={attempts}");
    for round in 1..=ROUNDS {
        let (scanned_by_the_intent, records) = measure_an_unlink(build, point, state, round, pool);
        println!("E203RESULT name=unlink_publishes build={build} point={point} state={state} round={round} publishes={}", records.len());
        if let (Some(intent), Some(last)) = (records.first(), records.last()) {
            emit_record(build, point, state, "unlink_intent", round, intent);
            emit_record(build, point, state, "unlink_final", round, last);
            println!("{scanned_by_the_intent}");
            // 最后一批扫的是意图那次发完的一版；两次发布在一次 unlink 调用里连着做，中间那一版取不到，
            // 这里取整个 unlink 做完的一版，与它相差意图那次与最后一批各改的几条记录（登记第五节共用项表只比两边同一个读法）。
            println!(
                "{}",
                state_text(build, point, state, "unlink_final", round, pool)
            );
        }
    }
}

/// 大小读数（登记第五节共用项表第四行）：类型大小，B8 常驻内存式子吃它们。
fn emit_sizes(build: &str, point: &str) {
    println!(
        "E203RESULT name=sizes build={build} point={point} unit_bytes={} key_bytes={} accounting_row_bytes={} allocation_record_bytes={}",
        std::mem::size_of::<TransactionUnit>(),
        std::mem::size_of::<CodeTwoTreeKey>(),
        std::mem::size_of::<AccountingEntry>(),
        std::mem::size_of::<AllocationRecord>()
    );
}

/// 第 1 步之后的池：mkfs、取号、暖机、第一个文件，盘字节数 (1408 + U) × 16384（登记读法第 5 条）。
fn pool_of_the_point(axes: Axes) -> Pool {
    PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + axes.unit_area_slots) * SLOT_BYTES,
        plain_device,
    )
}

/// 第 2、3 步：大文件 D 个数据单元、F 个小文件（每批至多 100 个建 inode，各覆盖写 2999 字节）。
/// 进度行与「判了几次」的读数由 `history.note()` 的调用位置定。
fn history_before_the_unlinks(
    pool: &mut Pool,
    history: &mut History,
    axes: Axes,
) -> Result<(), RefusedChange> {
    if axes.data_units > 0 {
        let big = create_inodes_counted(pool, &mut history.tally, 1)?[0];
        history.note();
        fill_the_big_file_counted(pool, &mut history.tally, big, axes.data_units)?;
        history.note();
    }
    let mut remaining = axes.files;
    while remaining > 0 {
        let batch = remaining.min(INODES_PER_BATCH);
        for inode in create_inodes_counted(pool, &mut history.tally, batch)? {
            history.note();
            overwrite_inode_counted(pool, &mut history.tally, inode)?;
            history.note();
        }
        remaining -= batch;
    }
    Ok(())
}

/// 第 4 步里的一次删：建一个空 inode、立刻 unlink 它（走空 + 意图 + 最后一批）。交回 unlink 那一次判了几次。
fn one_unlink_of_the_history(pool: &mut Pool, history: &mut History) -> Result<u64, RefusedChange> {
    let inode = create_inodes_counted(pool, &mut history.tally, 1)?[0];
    history.note();
    let before = history.judgements;
    unlink_counted(pool, &mut history.tally, inode)?;
    history.note();
    Ok(history.judgements - before)
}

/// 历史里会话交回了拒绝：登记第五节历史第 8 步只写了被空间拒、推抬 F 的照会话走，交回之后怎么接着走没写，所以照报、停下。
/// 打一行 `history_refused`（拒的是哪一种改动、原因、删了几次、第 8 步的次数、那一刻的占用读数），那个点没有 `item_done`。
fn stop_on_a_refusal_in_the_history(
    build: &str,
    point: &str,
    history: &History,
    unlinks_done: u64,
    refused: &RefusedChange,
    pool: &Pool,
) -> ! {
    let judgements_of_the_refused_change = e203_scan::take_finished().len();
    println!(
        "E203RESULT name=history_refused build={build} point={point} change={} cause={} unlinks_done={unlinks_done} user_changes={} judgements={} \
         judgements_of_the_refused_change={judgements_of_the_refused_change} {} {}",
        refused.kind.product_name(),
        refusal_cause_name(&refused.refusal),
        history.user_changes,
        history.judgements,
        history.tally.text(),
        occupancy_text(pool)
    );
    panic!(
        "历史里 {} 被会话拒了，登记第五节历史第 8 步没写这之后怎么走：{:?}",
        refused.kind.product_name(),
        refused.refusal
    );
}

/// 不量的那一段（历史第 1–4 步）停掉发布路径上逐步的读数（插计数脚本模块里的 `pause_readings`）：计数构建不记步数、表长与差量、不算规模读数的式子，
/// 计时构建的守卫不计时；段照旧换、记录照旧开与收，所以「判了几次」照数，发给会话的改动、次序与时间秒一样不变。做完恢复，被量的那几轮照旧全记。
/// 只为省挂钟：历史那一段的记录本来就只取条数、丢掉内容（`History::note`）。
fn with_the_readings_paused<Outcome>(unmeasured: impl FnOnce() -> Outcome) -> Outcome {
    e203_scan::pause_readings(true);
    let outcome = unmeasured();
    e203_scan::pause_readings(false);
    outcome
}

fn run_the_point(point: &str, axes: Axes) {
    let build = build_name();
    println!(
        "E203RESULT name=point build={build} point={point} unit_area_slots={} unlinks={} data_units={} files={}",
        axes.unit_area_slots, axes.unlinks, axes.data_units, axes.files
    );
    let mut pool = pool_of_the_point(axes);
    println!(
        "E203RESULT name=constants build={build} point={point} slot_bytes={SLOT_BYTES} segment_slots={CLUSTER_SEGMENT_SLOTS} unit_area_start_slot={} disks={} \
         inode_leaf_records={INODE_LEAF_RECORDS} inode_leaves_at_most={} tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} \
         settling_rounds_at_most={ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT} overwrite_bytes={OVERWRITE_BYTES}",
        unit_area_start_slot(),
        pool.session().allocator.devices.len(),
        leaf_containers_one_root_node_holds()
    );
    emit_sizes(&build, point);
    let mut history = History::new(point);
    let mut judgements_per_unlink: Vec<u64> = Vec::new();
    let outcome = with_the_readings_paused(|| -> Result<(), (u64, RefusedChange)> {
        history_before_the_unlinks(&mut pool, &mut history, axes)
            .map_err(|refused| (0, refused))?;
        for _ in 0..axes.unlinks {
            let judgements =
                one_unlink_of_the_history(&mut pool, &mut history).map_err(|refused| {
                    (
                        u64::try_from(judgements_per_unlink.len()).expect("次数"),
                        refused,
                    )
                })?;
            judgements_per_unlink.push(judgements);
        }
        Ok(())
    });
    if let Err((done, refused)) = outcome {
        stop_on_a_refusal_in_the_history(&build, point, &history, done, &refused, &pool);
    }
    println!(
        "E203RESULT name=history build={build} point={point} user_changes={} judgements={}",
        history.user_changes, history.judgements
    );
    println!(
        "E203RESULT name=history_changes build={build} point={point} {}",
        history.tally.text()
    );
    println!(
        "E203RESULT name=history_unlinks build={build} point={point} count={} fewest={} most={}",
        judgements_per_unlink.len(),
        judgements_per_unlink.iter().copied().min().unwrap_or(0),
        judgements_per_unlink.iter().copied().max().unwrap_or(0)
    );
    measure(&build, point, "same_mount", &mut pool);
    pool.crash_and_mount_writable().expect("崩溃后可写挂载");
    drop(e203_scan::take_finished());
    measure(&build, point, "remounted", &mut pool);
    println!("E203RESULT name=item_done build={build} point={point}");
}

fn unit_area(exponent: u32) -> u64 {
    1_u64 << exponent
}

#[test]
fn point_unit_area_2_pow_14() {
    run_the_point(
        "unit_area_2_pow_14",
        Axes {
            unit_area_slots: unit_area(14),
            ..BASE
        },
    );
}
#[test]
fn point_base() {
    run_the_point("base", BASE);
}
#[test]
fn point_unit_area_2_pow_20() {
    run_the_point(
        "unit_area_2_pow_20",
        Axes {
            unit_area_slots: unit_area(20),
            ..BASE
        },
    );
}
#[test]
fn point_unit_area_2_pow_24() {
    run_the_point(
        "unit_area_2_pow_24",
        Axes {
            unit_area_slots: unit_area(24),
            ..BASE
        },
    );
}
#[test]
fn point_unlinks_1() {
    run_the_point("unlinks_1", Axes { unlinks: 1, ..BASE });
}
#[test]
fn point_unlinks_100() {
    run_the_point(
        "unlinks_100",
        Axes {
            unlinks: 100,
            ..BASE
        },
    );
}
#[test]
fn point_unlinks_1000() {
    run_the_point(
        "unlinks_1000",
        Axes {
            unlinks: 1000,
            ..BASE
        },
    );
}
#[test]
fn point_unlinks_10000() {
    run_the_point(
        "unlinks_10000",
        Axes {
            unlinks: 10_000,
            ..BASE
        },
    );
}
#[test]
fn point_data_units_1() {
    run_the_point(
        "data_units_1",
        Axes {
            data_units: 1,
            ..BASE
        },
    );
}
#[test]
fn point_data_units_100() {
    run_the_point(
        "data_units_100",
        Axes {
            data_units: 100,
            ..BASE
        },
    );
}
#[test]
fn point_data_units_1000() {
    run_the_point(
        "data_units_1000",
        Axes {
            data_units: 1000,
            ..BASE
        },
    );
}
#[test]
fn point_data_units_10000() {
    run_the_point(
        "data_units_10000",
        Axes {
            data_units: 10_000,
            ..BASE
        },
    );
}
#[test]
fn point_files_1() {
    run_the_point("files_1", Axes { files: 1, ..BASE });
}
#[test]
fn point_files_100() {
    run_the_point("files_100", Axes { files: 100, ..BASE });
}
#[test]
fn point_files_1000() {
    run_the_point(
        "files_1000",
        Axes {
            files: 1000,
            ..BASE
        },
    );
}
#[test]
fn point_files_10000() {
    run_the_point(
        "files_10000",
        Axes {
            files: 10_000,
            ..BASE
        },
    );
}
#[test]
fn point_diagonal() {
    run_the_point(
        "diagonal",
        Axes {
            unit_area_slots: unit_area(20),
            unlinks: 10_000,
            data_units: 10_000,
            files: 1000,
        },
    );
}
/// Y1 = (U, 10⁵, 10, 10)（登记第五节「实现量与分段」第二段）。登记写的 U = 2¹⁴ 装不下 10⁵ 次删（第四万多次删时建 inode 被合取 7 拒），
/// U 加大到 2¹⁶ 或 2¹⁷，u、D、F 照留；两档各一条用例，点名里写明 U，用哪一档由探针定（不两档都进产物）。取样点与探针共用这一份几何。
fn many_unlinks_on_a_unit_area_of(exponent: u32) -> Axes {
    Axes {
        unit_area_slots: unit_area(exponent),
        unlinks: 100_000,
        ..BASE
    }
}
#[test]
fn point_many_unlinks_unit_area_2_pow_16() {
    run_the_point(
        "many_unlinks_unit_area_2_pow_16",
        many_unlinks_on_a_unit_area_of(16),
    );
}
#[test]
fn point_many_unlinks_unit_area_2_pow_17() {
    run_the_point(
        "many_unlinks_unit_area_2_pow_17",
        many_unlinks_on_a_unit_area_of(17),
    );
}

/// 探针一趟删多少次（建空 inode、unlink 它算一次）：环境变量 `E203_PROBE_UNLINKS` 给了照它，没给取这个数。
const PROBE_UNLINKS_BY_DEFAULT: u64 = 10_000;
/// 探针的单元区是 2 的几次方：环境变量 `E203_PROBE_UNIT_AREA_EXPONENT` 给了照它，没给取 14（登记原来的 Y1）。
const PROBE_UNIT_AREA_EXPONENT_BY_DEFAULT: u32 = 14;
/// 探针每删这么多次打一行占用读数。
const PROBE_SAMPLE_EVERY: u64 = 1000;
/// Y1 要删的次数：探针判「装得下」比的就是它。
const UNLINKS_OF_THE_MANY_UNLINKS_POINT: u64 = 100_000;
/// 后半段每千次删少的全空段数，至多是前半段的这么多倍（分子 ÷ 分母）才算「直」：更陡的说明越删越快，照直线外推不可信。
const STRAIGHT_ENOUGH_NUMERATOR: u64 = 5;
const STRAIGHT_ENOUGH_DENOMINATOR: u64 = 4;

/// 每块盘的全空聚簇段数，两样并排：分配器增量维护的那一个（合取 7 读它）与从分配记录现数的那一个。
fn empty_segments_two_ways(pool: &Pool) -> Vec<(u64, u64)> {
    let allocator = &pool.session().allocator;
    allocator
        .devices
        .iter()
        .map(|device_map| {
            let recount = empty_segments_counted_from_the_records(
                allocator
                    .records()
                    .iter()
                    .filter(|record| record.device == device_map.device)
                    .map(|record| {
                        (
                            record.slot.0,
                            u64::from(record.span_slots),
                            record.is_released,
                        )
                    }),
                device_map.unit_area_start().slot().0,
                device_map.unit_area_slots(),
            );
            (device_map.empty_segments(), recount)
        })
        .collect()
}

/// 照一串（删了几次，那一刻各盘里最少的全空段数）的头尾两点连直线，推到全空段数为 0 时删了几次（推的，不是量的）；
/// 尾点不比头点少、或只有一点，交回 `None`。
fn extrapolated_unlinks_until_no_empty_segment(samples: &[(u64, u64)]) -> Option<u64> {
    let (&(first_unlinks, first_empty), &(last_unlinks, last_empty)) =
        (samples.first()?, samples.last()?);
    if last_unlinks <= first_unlinks || last_empty >= first_empty {
        return None;
    }
    Some(last_unlinks + last_empty * (last_unlinks - first_unlinks) / (first_empty - last_empty))
}

/// 每 10⁴ 次删的口径：一串（删了几次，全空段数）头尾两点之间每 10⁴ 次删少几段（取整往下）；不到两个点、或头尾删的次数一样，交回 `None`。
fn segments_lost_per_ten_thousand_unlinks(samples: &[(u64, u64)]) -> Option<u64> {
    let (&(first_unlinks, first_empty), &(last_unlinks, last_empty)) =
        (samples.first()?, samples.last()?);
    (last_unlinks > first_unlinks)
        .then(|| first_empty.saturating_sub(last_empty) * 10_000 / (last_unlinks - first_unlinks))
}

/// 探针末行的外推口径（推的，不是量的）：取样串从中间那一点切成前后两半（中间那一点两半都算）。
/// ① 头尾两点连直线推到全空段数为 0 的那一次删；② 后半段两端连直线推的那一次（越删越快时它比 ① 早）；
/// ③ 前后两半每 10⁴ 次删少几段；后半段至多是前半段的 5/4 倍才算「直」。
/// 「装得下」：取样期间全空段数没少（照这一段看删不完），或 ① ② 两个外推都推到 `UNLINKS_OF_THE_MANY_UNLINKS_POINT` 次之后。直不直另报，不并进装得下。
fn probe_extrapolation_text(samples: &[(u64, u64)]) -> String {
    let middle = samples.len() / 2;
    let whole = extrapolated_unlinks_until_no_empty_segment(samples);
    let second_half = samples
        .get(middle..)
        .and_then(extrapolated_unlinks_until_no_empty_segment);
    let first_rate = samples
        .get(..=middle)
        .and_then(segments_lost_per_ten_thousand_unlinks);
    let second_rate = samples
        .get(middle..)
        .and_then(segments_lost_per_ten_thousand_unlinks);
    let straight_enough = match (first_rate, second_rate) {
        (Some(first), Some(second)) => {
            second * STRAIGHT_ENOUGH_DENOMINATOR <= first * STRAIGHT_ENOUGH_NUMERATOR
        }
        _ => false,
    };
    let never_lost = match (samples.first(), samples.last()) {
        (Some((_, first_empty)), Some((_, last_empty))) => last_empty >= first_empty,
        _ => false,
    };
    let fits = samples.len() >= 3
        && (never_lost
            || [whole, second_half].into_iter().all(|until| {
                until.is_some_and(|unlinks| unlinks > UNLINKS_OF_THE_MANY_UNLINKS_POINT)
            }));
    let shown =
        |value: Option<u64>| value.map_or_else(|| "none".to_string(), |number| number.to_string());
    format!(
        "extrapolated_unlinks_until_no_empty_segment={} extrapolated_from_the_second_half={} segments_lost_per_ten_thousand_unlinks_first_half={} \
         segments_lost_per_ten_thousand_unlinks_second_half={} straight_enough={straight_enough} fits_the_unlinks_of_the_point={fits}",
        shown(whole),
        shown(second_half),
        shown(first_rate),
        shown(second_rate)
    )
}

fn number_from_the_environment(name: &str, by_default: u64) -> u64 {
    std::env::var(name).map_or(by_default, |text| {
        text.parse::<u64>()
            .unwrap_or_else(|_| panic!("{name} 要是非负整数"))
    })
}

/// 探针：Y1 的 u、D、F 与同一段历史（第 1–4 步），单元区取 2 的 `E203_PROBE_UNIT_AREA_EXPONENT` 次方（不给取 14），只删 `E203_PROBE_UNLINKS` 次（不给取 10⁴），
/// 每 1000 次打一行占用读数与从起池到这一刻的挂钟秒数（`E203PROBE name=probe_trajectory`）；会话交回拒绝就打拒的那一刻（`probe_refused`）。
/// 末行 `probe_verdict`：被拒了，必须与 Y1 一样是建 inode 被合取 7 拒（不是就红）；删满没被拒，记 `reproduced=false`；两样都另报分配器口径与现数口径
/// 对不上的取样数与外推口径（`probe_extrapolation_text`，推的）。逐步的读数整趟停着（不量），`E203_PROBE_KEEP_READINGS=1` 时不停（对照停读数省了多少挂钟）。
/// 不进产物：入口只点名 `point_<点>`，这一条不在里面；打的行以 `E203PROBE` 起头，入口写产物时不收。
#[test]
fn probe_of_the_empty_segments_refusal_on_many_unlinks() {
    let unlinks = number_from_the_environment("E203_PROBE_UNLINKS", PROBE_UNLINKS_BY_DEFAULT);
    let exponent = u32::try_from(number_from_the_environment(
        "E203_PROBE_UNIT_AREA_EXPONENT",
        u64::from(PROBE_UNIT_AREA_EXPONENT_BY_DEFAULT),
    ))
    .expect("指数装得进 u32");
    let keep_readings = std::env::var("E203_PROBE_KEEP_READINGS").is_ok_and(|text| text == "1");
    let axes = many_unlinks_on_a_unit_area_of(exponent);
    let point = format!("probe_many_unlinks_unit_area_2_pow_{exponent}");
    println!(
        "E203PROBE name=probe_geometry point={point} unit_area_slots={} segments_per_device={} unlinks_to_do={unlinks} readings_paused={}",
        axes.unit_area_slots,
        axes.unit_area_slots / CLUSTER_SEGMENT_SLOTS,
        !keep_readings
    );
    e203_scan::pause_readings(!keep_readings);
    let started = Instant::now();
    let mut pool = pool_of_the_point(axes);
    let mut history = History::new(&point);
    if let Err(refused) = history_before_the_unlinks(&mut pool, &mut history, axes) {
        panic!("删之前那一段历史就被拒：{:?}", refused.refusal);
    }
    let mut samples: Vec<(u64, u64)> = Vec::new();
    let mut samples_disagreeing = 0_u64;
    let mut done = 0_u64;
    let refused = loop {
        if done % PROBE_SAMPLE_EVERY == 0 {
            let two_ways = empty_segments_two_ways(&pool);
            samples_disagreeing += u64::from(
                two_ways
                    .iter()
                    .any(|(by_the_allocator, recount)| by_the_allocator != recount),
            );
            samples.push((
                done,
                two_ways
                    .iter()
                    .map(|(by_the_allocator, _)| *by_the_allocator)
                    .min()
                    .expect("两块盘"),
            ));
            println!(
                "E203PROBE name=probe_trajectory point={point} unlinks_done={done} elapsed_seconds={} user_changes={} judgements={} {} {}",
                started.elapsed().as_secs(),
                history.user_changes,
                history.judgements,
                history.tally.text(),
                occupancy_text(&pool)
            );
        }
        if done == unlinks {
            break None;
        }
        match one_unlink_of_the_history(&mut pool, &mut history) {
            Ok(_) => done += 1,
            Err(refused) => break Some(refused),
        }
    };
    e203_scan::pause_readings(false);
    let extrapolated = probe_extrapolation_text(&samples);
    let Some(refused) = refused else {
        println!(
            "E203PROBE name=probe_verdict point={point} reproduced=false unlinks_done={done} elapsed_seconds={} samples={} \
             samples_where_the_two_counts_disagree={samples_disagreeing} {extrapolated}",
            started.elapsed().as_secs(),
            samples.len()
        );
        return;
    };
    let cause = refusal_cause_name(&refused.refusal);
    println!(
        "E203PROBE name=probe_refused point={point} change={} cause={cause} unlinks_done={done} user_changes={} judgements={} judgements_of_the_refused_change={} {} {}",
        refused.kind.product_name(),
        history.user_changes,
        history.judgements,
        e203_scan::take_finished().len(),
        history.tally.text(),
        occupancy_text(&pool)
    );
    println!(
        "E203PROBE name=probe_verdict point={point} reproduced=true unlinks_done={done} elapsed_seconds={} change={} cause={cause} samples={} \
         samples_where_the_two_counts_disagree={samples_disagreeing} {extrapolated}",
        started.elapsed().as_secs(),
        refused.kind.product_name(),
        samples.len()
    );
    assert!(
        matches!(refused.kind, ChangeKind::NewInodes)
            && cause == "EmptyClusterSegmentsBelowTheFloor",
        "Y1 那一点是建 inode 被合取 7 拒（全空聚簇段少于 N₀）；探针被拒的不是这一形：{:?} {:?}",
        refused.kind,
        refused.refusal
    );
}

/// 第 8 步的次数照历史的结构逐种钉死（独立于计数那一处，从取样点的四根轴算）：2¹⁴ 单元区、D = 10、F = 10、u = 10 那一段历史
/// 建 inode 1（大文件）+ 1（F 个小文件一批）+ 10（每次删一个）= 12 次、覆盖写 10 次、按偏移写 ⌈10 / 400⌉ = 1 次、unlink 10 次、会话交回拒绝 0 次；
/// 推的抬 F 串数与推之前被拒的次数一一对应。另核占用读数里全空聚簇段数两种数法逐盘相同（探针与 `history_refused` 那一行靠它）。
#[test]
fn history_changes_count_every_kind_of_change_on_a_small_history() {
    let axes = Axes {
        unit_area_slots: 1 << 14,
        ..BASE
    };
    let mut pool = pool_of_the_point(axes);
    let mut history = History::new("history_changes_count");
    if let Err(refused) = history_before_the_unlinks(&mut pool, &mut history, axes) {
        panic!("删之前那一段历史被拒：{:?}", refused.refusal);
    }
    for _ in 0..axes.unlinks {
        if let Err(refused) = one_unlink_of_the_history(&mut pool, &mut history) {
            panic!("删的那一段被拒：{:?}", refused.refusal);
        }
    }
    let tally = &history.tally;
    assert_eq!(tally.new_inode_changes, 1 + 1 + 10);
    assert_eq!(tally.overwrite_changes, 10);
    assert_eq!(tally.write_at_offset_changes, 1);
    assert_eq!(tally.unlink_changes, 10);
    assert_eq!(tally.changes_refused, 0);
    assert_eq!(
        tally.floor_raises,
        tally.refusals_that_pushed_the_floor_raises
    );
    let two_ways = empty_segments_two_ways(&pool);
    assert_eq!(two_ways.len(), 2);
    for (counted_by_the_allocator, recount) in two_ways {
        assert_eq!(counted_by_the_allocator, recount);
    }
}

#[test]
fn empty_segments_counted_from_the_records_pins_hand_counted_segments() {
    // 单元区从槽 1408 起、256 槽 = 4 段：段 0 = 1408..1471，段 1 = 1472..1535，段 2 = 1536..1599，段 3 = 1600..1663。
    // 一条罩段 0 的未释放记录；一条从 1534 起跨 4 槽、罩到段 1 与段 2 的未释放记录；段 3 里只有一条已释放的。只剩段 3 全空。
    let records = [(1408, 2, false), (1534, 4, false), (1600, 1, true)];
    assert_eq!(
        empty_segments_counted_from_the_records(records.into_iter(), 1408, 256),
        1
    );
    // 全是已释放的（defer 里的槽不算占用）：4 段全空。
    let released = [(1408, 2, true), (1534, 4, true), (1600, 1, true)];
    assert_eq!(
        empty_segments_counted_from_the_records(released.into_iter(), 1408, 256),
        4
    );
    // 单元区尾上凑不满 64 槽的那一截不算段：300 槽 = 4 个整段，罩在尾上 1664 那一截的记录不让任何一段占着。
    let tail = [(1664, 2, false)];
    assert_eq!(
        empty_segments_counted_from_the_records(tail.into_iter(), 1408, 300),
        4
    );
}

#[test]
fn variant_name_takes_the_member_name_before_the_payload() {
    assert_eq!(
        variant_name("EmptyClusterSegmentsBelowTheFloor(EmptyClusterSegmentsBelowTheFloorOnSomeDevices { floor_in_segments: 1 })"),
        "EmptyClusterSegmentsBelowTheFloor"
    );
    assert_eq!(
        variant_name("Publish { cause: SpaceAdmissionRefused }"),
        "Publish"
    );
    assert_eq!(variant_name("NotNeeded"), "NotNeeded");
}

#[test]
fn extrapolated_unlinks_until_no_empty_segment_pins_the_straight_line() {
    // 头点 (0, 250)、尾点 (10000, 190)：每删 10000 次少 60 段，还剩 190 段要再删 190 × 10000 / 60 = 31666 次（取整往下），到 41666。
    assert_eq!(
        extrapolated_unlinks_until_no_empty_segment(&[(0, 250), (5000, 222), (10_000, 190)]),
        Some(41_666)
    );
    // 头点不在 0：(1000, 250) 到 (11000, 190)，同样每 10000 次少 60 段，推到 11000 + 31666 = 42666。
    assert_eq!(
        extrapolated_unlinks_until_no_empty_segment(&[(1000, 250), (11_000, 190)]),
        Some(42_666)
    );
    assert_eq!(
        extrapolated_unlinks_until_no_empty_segment(&[(0, 250), (10_000, 250)]),
        None
    );
    assert_eq!(
        extrapolated_unlinks_until_no_empty_segment(&[(0, 250)]),
        None
    );
}

#[test]
fn probe_extrapolation_text_reads_the_two_halves() {
    // 直：每 10⁴ 次删前后两半都少 60 段；两种外推都推到 10000 + 940 × 10000 / 60 = 166666 次，过 10⁵，装得下。
    assert_eq!(
        probe_extrapolation_text(&[(0, 1000), (5000, 970), (10_000, 940)]),
        "extrapolated_unlinks_until_no_empty_segment=166666 extrapolated_from_the_second_half=166666 segments_lost_per_ten_thousand_unlinks_first_half=60 \
         segments_lost_per_ten_thousand_unlinks_second_half=60 straight_enough=true fits_the_unlinks_of_the_point=true"
    );
    // 越删越快：前半 32 段、后半 80 段（超过 5/4 倍，不直）；头尾推 10000 + 200 × 10000 / 56 = 45714，后半推 10000 + 200 × 5000 / 40 = 35000。
    assert_eq!(
        probe_extrapolation_text(&[(0, 256), (5000, 240), (10_000, 200)]),
        "extrapolated_unlinks_until_no_empty_segment=45714 extrapolated_from_the_second_half=35000 segments_lost_per_ten_thousand_unlinks_first_half=32 \
         segments_lost_per_ten_thousand_unlinks_second_half=80 straight_enough=false fits_the_unlinks_of_the_point=false"
    );
    // 头尾推得过 10⁵（10000 + 1850 × 10000 / 150 = 133333），后半推不过（10000 + 1850 × 5000 / 145 = 73793）：只看头尾会误判装得下。
    assert_eq!(
        probe_extrapolation_text(&[(0, 2000), (5000, 1995), (10_000, 1850)]),
        "extrapolated_unlinks_until_no_empty_segment=133333 extrapolated_from_the_second_half=73793 segments_lost_per_ten_thousand_unlinks_first_half=10 \
         segments_lost_per_ten_thousand_unlinks_second_half=290 straight_enough=false fits_the_unlinks_of_the_point=false"
    );
    // 全空段数一段没少：推不到 0，装得下。
    assert_eq!(
        probe_extrapolation_text(&[(0, 1024), (5000, 1024), (10_000, 1024)]),
        "extrapolated_unlinks_until_no_empty_segment=none extrapolated_from_the_second_half=none segments_lost_per_ten_thousand_unlinks_first_half=0 \
         segments_lost_per_ten_thousand_unlinks_second_half=0 straight_enough=true fits_the_unlinks_of_the_point=true"
    );
    // 不到三个取样点不判装得下。
    assert!(probe_extrapolation_text(&[(0, 1024), (10_000, 1024)])
        .ends_with(" fits_the_unlinks_of_the_point=false"));
}

/// 2¹⁴ 单元区、D = F = u = 10 那一段历史（第 1–4 步）在一个新池上做一遍；`paused` 为真时整段停着读数做（`with_the_readings_paused`）。
fn small_history_on_a_pool(paused: bool) -> (Pool, History) {
    let axes = Axes {
        unit_area_slots: 1 << 14,
        ..BASE
    };
    let mut pool = pool_of_the_point(axes);
    let mut history = History::new("pause_check");
    let history_steps = |pool: &mut Pool, history: &mut History| {
        history_before_the_unlinks(pool, history, axes)
            .unwrap_or_else(|refused| panic!("删之前那一段被拒：{:?}", refused.refusal));
        for _ in 1..=axes.unlinks {
            one_unlink_of_the_history(pool, history)
                .unwrap_or_else(|refused| panic!("删的那一段被拒：{:?}", refused.refusal));
        }
    };
    if paused {
        with_the_readings_paused(|| history_steps(&mut pool, &mut history));
    } else {
        history_steps(&mut pool, &mut history);
    }
    (pool, history)
}

/// 停读数只省挂钟：同一段历史停着做与不停做，判了几次、第 8 步的次数、这一版的规模读数与占用读数逐字相同；停着的时候发布照收记录、记录里没有行与差量；
/// 做完恢复，下一次被量的覆盖写照旧有行（两种构建都过：计数构建的行有步数，计时构建的行有独占计时）。
#[test]
fn pausing_the_readings_in_the_history_changes_neither_the_version_nor_the_judgements() {
    let (mut paused_pool, paused_history) = small_history_on_a_pool(true);
    assert!(!e203_scan::readings_paused(), "做完历史那一段要恢复读数");
    let (unpaused_pool, unpaused_history) = small_history_on_a_pool(false);
    assert_eq!(paused_history.judgements, unpaused_history.judgements);
    assert_eq!(paused_history.user_changes, unpaused_history.user_changes);
    assert_eq!(paused_history.tally.text(), unpaused_history.tally.text());
    assert_eq!(
        state_text(
            "check",
            "pause_check",
            "same_mount",
            "overwrite",
            1,
            &paused_pool
        ),
        state_text(
            "check",
            "pause_check",
            "same_mount",
            "overwrite",
            1,
            &unpaused_pool
        )
    );
    assert_eq!(occupancy_text(&paused_pool), occupancy_text(&unpaused_pool));
    assert!(with_the_readings_paused(e203_scan::readings_paused));
    let records_while_paused = with_the_readings_paused(|| {
        drop(e203_scan::take_finished());
        paused_pool
            .overwrite(OVERWRITE_BYTES)
            .expect("停着读数时覆盖写发成");
        e203_scan::take_finished()
    });
    assert!(
        !records_while_paused.is_empty(),
        "停着读数也照收记录（判了几次照数）"
    );
    assert!(
        records_while_paused
            .iter()
            .all(|record| record.rows.is_empty() && record.deltas.is_empty()),
        "停着读数的那一次发布，记录里不该有行与差量"
    );
    let measured = measure_an_overwrite(&mut paused_pool).expect("恢复之后这一次覆盖写只判一次");
    assert!(!measured.rows.is_empty(), "恢复之后被量的那一次要有行");
}
#[test]
fn point_user_scale_unit_area() {
    run_the_point(
        "user_scale_unit_area",
        Axes {
            unit_area_slots: (1 << 30) - 1408,
            ..BASE
        },
    );
}
