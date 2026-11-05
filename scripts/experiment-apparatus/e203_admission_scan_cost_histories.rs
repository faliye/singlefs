//! E203（准入之前扫类的步代价）的历史用例：拷进副本当 `crates/singlefs-verification/tests/e203_admission_scan_cost_histories.rs`，
//! 副本由 `e203_admission_scan_cost_counters.py` 套上计数（`--build counts`）或计时（`--build timing`）；两种构建里这一份源文件逐字相同。
//!
//! 一个取样点一条用例（跑前登记第五节「取样点」）：mkfs、取号、暖机、第一个文件 → 大文件 D 个数据单元 → F 个小文件 → u 次 unlink →
//! 同一次挂载里量 → `crash_and_mount_writable` 之后再量。同一个点的两种挂载状态共用前面那一段历史，写在同一条用例里（后一步读前一步留下的那一版）。
//! 每一轮量三种发布：覆盖写第一个文件、unlink 的意图那次与最后一批。打 `E203RESULT` 行，产物格式见模型 `e203_admission_scan_cost.rs` 的 `read_product`。
//! 环境变量 `E203_BUILD`（counts 或 timing）写进产物行；没设拒跑。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::{AllocationRecord, UnitAreaStart};
use singlefs_core::code_two_tree::CodeTwoTreeKey;
use singlefs_core::inode_tree::leaf_containers_one_root_node_holds;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
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

/// 6 MiB 环：单元区从槽 1408 起（同第三轮攻方模型那一档）。
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

fn create_inodes(pool: &mut Pool, count: u64) -> Vec<InodeNumber> {
    let first = next_inode(pool).0;
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        UserChange::NewInodes {
            count,
            write_time_seconds,
        },
        None,
    )
    .expect("建 inode 发成");
    (first..first + count).map(InodeNumber).collect()
}

fn overwrite_inode(pool: &mut Pool, inode: InodeNumber) {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    pool.publish(
        UserChange::OverwriteOfAnInode {
            inode,
            content: &content,
            write_time_seconds,
        },
        None,
    )
    .expect("按 inode 号覆盖写发成");
}

/// 大文件：按 inode 号从偏移 0 起接着写，每次至多 400 个数据单元，凑够 `data_units` 个。
fn fill_the_big_file(pool: &mut Pool, inode: InodeNumber, data_units: u64) {
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
        pool.publish(
            UserChange::WriteAtOffset {
                inode,
                offset: FileOffsetInBytes(written * payload),
                bytes: PartialWriteBytes::new(&bytes).expect("写的字节不是空的"),
                write_time_seconds,
            },
            None,
        )
        .expect("按偏移写发成");
        written += units;
    }
}

fn unlink(pool: &mut Pool, inode: InodeNumber) {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        UserChange::Unlink {
            inode,
            write_time_seconds,
        },
        None,
    )
    .expect("unlink 发成");
}

/// 历史里每做完一次用户改动就把收好的记录取走（不让几万次发布的读数堆在内存里），记判了几次。
struct History {
    user_changes: u64,
    judgements: u64,
}

impl History {
    fn note(&mut self) {
        self.user_changes += 1;
        self.judgements += u64::try_from(e203_scan::take_finished().len()).expect("条数");
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
            "E203RESULT name=row build={build} point={point} state={state} kind={kind} round={round} segment={} row={} calls={} steps={} scale={} exclusive_nanos={} queries={} query_levels={}",
            row.segment, row.row, row.calls, row.steps, row.scale, row.exclusive_nanos, row.queries, row.query_levels
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

/// 量一轮覆盖写（第一个文件）；判了不止一次的那一轮不算（登记第五节历史第 8 步），返回 `None`。
fn measure_an_overwrite(pool: &mut Pool) -> Option<PublishRecord> {
    drop(e203_scan::take_finished());
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
    let mut records = e203_scan::take_finished();
    (records.len() == 1).then(|| records.remove(0))
}

/// 量一轮 unlink：建 Y、按 inode 号覆盖写 Y 一个单元（不量）、unlink Y（量）。交回 unlink 的各次发布。
fn measure_an_unlink(pool: &mut Pool) -> Vec<PublishRecord> {
    let inode = create_inodes(pool, 1)[0];
    overwrite_inode(pool, inode);
    drop(e203_scan::take_finished());
    unlink(pool, inode);
    e203_scan::take_finished()
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
        let scanned_by_the_intent = state_text(build, point, state, "unlink_intent", round, pool);
        let records = measure_an_unlink(pool);
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

fn run_the_point(point: &str, axes: Axes) {
    let build = build_name();
    println!(
        "E203RESULT name=point build={build} point={point} unit_area_slots={} unlinks={} data_units={} files={}",
        axes.unit_area_slots, axes.unlinks, axes.data_units, axes.files
    );
    let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        (unit_area_start_slot() + axes.unit_area_slots) * SLOT_BYTES,
        plain_device,
    );
    println!(
        "E203RESULT name=constants build={build} point={point} slot_bytes={SLOT_BYTES} segment_slots={CLUSTER_SEGMENT_SLOTS} unit_area_start_slot={} disks={} \
         inode_leaf_records={INODE_LEAF_RECORDS} inode_leaves_at_most={} tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} \
         settling_rounds_at_most={ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT} overwrite_bytes={OVERWRITE_BYTES}",
        unit_area_start_slot(),
        pool.session().allocator.devices.len(),
        leaf_containers_one_root_node_holds()
    );
    emit_sizes(&build, point);
    let mut history = History {
        user_changes: 0,
        judgements: 0,
    };
    if axes.data_units > 0 {
        let big = create_inodes(&mut pool, 1)[0];
        history.note();
        fill_the_big_file(&mut pool, big, axes.data_units);
        history.note();
    }
    let mut remaining = axes.files;
    while remaining > 0 {
        let batch = remaining.min(INODES_PER_BATCH);
        for inode in create_inodes(&mut pool, batch) {
            history.note();
            overwrite_inode(&mut pool, inode);
            history.note();
        }
        remaining -= batch;
    }
    let mut judgements_per_unlink: Vec<u64> = Vec::new();
    for _ in 0..axes.unlinks {
        let inode = create_inodes(&mut pool, 1)[0];
        history.note();
        let before = history.judgements;
        unlink(&mut pool, inode);
        history.note();
        judgements_per_unlink.push(history.judgements - before);
    }
    println!(
        "E203RESULT name=history build={build} point={point} user_changes={} judgements={}",
        history.user_changes, history.judgements
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
#[test]
fn point_many_unlinks_small_unit_area() {
    run_the_point(
        "many_unlinks_small_unit_area",
        Axes {
            unit_area_slots: unit_area(14),
            unlinks: 100_000,
            ..BASE
        },
    );
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
