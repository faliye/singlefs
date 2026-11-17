//! 攻方腿模型（d3-fallback-richest-leaf-code-r1，攻击面第 2 条「固定点终止」与第 1 条「越位」）：锻造的盘上覆盖写第一个文件一次，
//! 看分配记录树重写集的固定点（`transaction::settle_the_allocation_record_tree`）定不定得下、走几轮、还是交回别的拒绝。
//! 锻造照实现员的新目标 `allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf` 的写法（只动空闲图、不写分配记录）。
//! 三条臂由 `tools/patch_the_copy.py` 在副本里改分配器：as-coded（今天的实现）、prefer-then-lowest（先重写集、不够退回各盘槽号最小的空槽）、
//! lowest-only（改前的政策：各盘槽号最小的空槽）。设备在进程里造，不读活树。每条用例打一行 `PROBE_RESULT …`，不断言结局（结局就是要量的东西）。

mod common_admission_pool;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{InodeNumber, SlotNumber};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::transaction::{
    publish_overwrite_of_an_inode, PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_format::{ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::e142_parameters;

type Pool = PoolUnderTest<SparseBlockDevice>;

const JOURNAL_RING_BYTES: u64 = 6 << 20;
const OVERWRITES_BEFORE_THE_FORGING: usize = 4;
const FIRST_SPARSE_LEAF: u64 = 12;
const FREE_SLOTS_AT_THE_START_OF_THE_FIRST_SPARSE_LEAF: u64 = 14;
const SPARSE_LEAVES_WITH_TWO_FREE_SLOTS: u64 = 120;
const OFFSET_OF_THE_FIRST_FREE_SLOT_IN_A_SPARSE_LEAF: u64 = 400;
const SECONDS_FROM_THE_LAST_WRITE_TO_THE_OVERWRITE: u64 = 100;
const STACK_BYTES_OF_THE_PUBLISHING_THREAD: usize = 64 << 20;
const QUARTER_MEBI_SLOTS: u64 = 1 << 18;
const MEBI_SLOTS: u64 = 1 << 20;

/// 锻造的形状。叶序号小于 [`FIRST_SPARSE_LEAF`] 的叶在每种形状里都整片占满（上一版的节点住在那里）。
#[derive(Clone, Copy, Debug)]
enum ForgedShape {
    /// 实现员新目标的形状：叶 12 留叶首 14 槽、之后 120 片叶各留偏移 400–401、再往上每段只占段首一槽。
    SparseLeavesBelowARichRegion,
    /// 叶 12 起到单元区末尾，每片叶只留偏移 400–401 一对。没有空槽多的叶。
    AnAlignedPairInEveryLeaf,
    /// 叶 12 起每片叶只留偏移 401 一个单槽（凑不成对）；单元区里最后一片整叶另留偏移 400–403 两对（数据单元一对、两槽的 inode 叶容器一对）。
    OneUnalignedSlotInEveryLeafAndTwoPairsInTheTopLeaf,
    /// 叶 12 只留偏移 400–401 一对（给数据单元）；叶 13 留 20 个奇数偏移的单槽（401、403 … 439）；叶 14 留偏移 400–401 一对再加 21 个奇数偏移的单槽
    /// （403 … 443），共 23 槽、是空槽最多的叶，而它最低的空槽正是全池（数据单元取走叶 12 那对之后）唯一一对的起点。别的叶都占满。
    PairAtTheBottomOfTheRichestLeaf,
    /// 上一形的镜像：叶 12 只留偏移 400–401 一对（给数据单元）；叶 13 只留偏移 400–401 一对（数据单元取走叶 12 那对之后，它是全池最低的空槽、也是唯一一对）；
    /// 叶 14 留 20 个奇数偏移的单槽（401、403 … 439），是空槽最多的叶。别的叶都占满。
    PairAtTheBottomOfTheLowestLeaf,
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

fn is_left_free(shape: ForgedShape, slot: u64, start: u64, end: u64) -> bool {
    let leaf_index = slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
    let offset = slot % ALLOCATION_RECORD_TREE_LEAF_SLOTS;
    let first = OFFSET_OF_THE_FIRST_FREE_SLOT_IN_A_SPARSE_LEAF;
    let is_odd_offset_from = |low: u64, high: u64| offset % 2 == 1 && (low..=high).contains(&offset);
    if leaf_index < FIRST_SPARSE_LEAF {
        return false;
    }
    match shape {
        ForgedShape::SparseLeavesBelowARichRegion => {
            if leaf_index == FIRST_SPARSE_LEAF {
                offset < FREE_SLOTS_AT_THE_START_OF_THE_FIRST_SPARSE_LEAF
            } else if leaf_index <= FIRST_SPARSE_LEAF + SPARSE_LEAVES_WITH_TWO_FREE_SLOTS {
                (first..first + 2).contains(&offset)
            } else {
                !(slot - start).is_multiple_of(CLUSTER_SEGMENT_SLOTS)
            }
        }
        ForgedShape::AnAlignedPairInEveryLeaf => (first..first + 2).contains(&offset),
        ForgedShape::OneUnalignedSlotInEveryLeafAndTwoPairsInTheTopLeaf => {
            let top_leaf = (end - 1) / ALLOCATION_RECORD_TREE_LEAF_SLOTS - 1;
            if leaf_index == top_leaf {
                (first..first + 4).contains(&offset)
            } else {
                offset == first + 1
            }
        }
        ForgedShape::PairAtTheBottomOfTheRichestLeaf => match leaf_index - FIRST_SPARSE_LEAF {
            0 => (first..first + 2).contains(&offset),
            1 => is_odd_offset_from(first + 1, first + 39),
            2 => (first..first + 2).contains(&offset) || is_odd_offset_from(first + 3, first + 43),
            _ => false,
        },
        ForgedShape::PairAtTheBottomOfTheLowestLeaf => match leaf_index - FIRST_SPARSE_LEAF {
            0 | 1 => (first..first + 2).contains(&offset),
            2 => is_odd_offset_from(first + 1, first + 39),
            _ => false,
        },
    }
}

/// 只动空闲图，不写分配记录（同实现员的锻造）。
fn forge(allocator: &mut PoolAllocator, shape: ForgedShape) {
    let start = unit_area_start_slot();
    for device_map in &mut allocator.devices {
        let end = start + device_map.unit_area_slots();
        for slot in start..end {
            if !is_left_free(shape, slot, start, end) && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

fn outcome_of<Published>(published: &Result<Published, PublishError>) -> String {
    match published {
        Ok(_) => "published".to_owned(),
        Err(PublishError::EmptyClusterSegmentsBelowTheFloor(_)) => {
            "settled-then-refused-by-the-empty-segments-floor".to_owned()
        }
        Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle { rounds }) => {
            format!("did-not-settle-after-{rounds}-rounds")
        }
        Err(PublishError::PlacementRefused { unit, refusal }) => {
            let refusal_kind = format!("{refusal:?}");
            format!(
                "placement-refused unit={unit:?} refusal={}",
                refusal_kind
                    .split(|character: char| !character.is_alphanumeric())
                    .next()
                    .unwrap_or("")
            )
        }
        Err(other) => {
            let text = format!("{other:?}");
            format!("other:{}", text.chars().take(160).collect::<String>())
        }
    }
}

fn run_the_shape(name: &'static str, shape: ForgedShape, unit_area_slots: u64) {
    let publishing_thread = std::thread::Builder::new()
        .stack_size(STACK_BYTES_OF_THE_PUBLISHING_THREAD)
        .spawn(move || {
            let started = std::time::Instant::now();
            let device_bytes = (unit_area_start_slot() + unit_area_slots) * SLOT_BYTES;
            let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
                parameters(),
                device_bytes,
                plain_device,
            );
            for _ in 0..OVERWRITES_BEFORE_THE_FORGING {
                pool.overwrite(OVERWRITE_BYTES).expect("空盘上覆盖写做得成");
            }
            let PoolUnderTest {
                parameters,
                devices,
                session,
                write_time_seconds,
                ..
            } = &mut pool;
            let session = session.as_mut().expect("这一刻有可写会话");
            forge(&mut session.allocator, shape);
            let every_device_lacks_an_empty_segment = session
                .allocator
                .devices
                .iter()
                .all(|device_map| device_map.lowest_empty_segment().is_none());
            let start = unit_area_start_slot();
            let common_free_slots = (start..start + unit_area_slots)
                .filter(|slot| {
                    session
                        .allocator
                        .devices
                        .iter()
                        .all(|device_map| device_map.is_free(SlotNumber(*slot)))
                })
                .count();
            let previous = session
                .current
                .file_version()
                .expect("现行那一版带文件")
                .clone();
            let overwrite_time_seconds =
                *write_time_seconds + SECONDS_FROM_THE_LAST_WRITE_TO_THE_OVERWRITE;
            let content = content_of(OVERWRITE_BYTES, overwrite_time_seconds);
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            let published = publish_overwrite_of_an_inode(
                &mut writer,
                &mut session.allocator,
                &previous,
                InodeNumber(FIRST_INODE_NUMBER),
                &content,
                overwrite_time_seconds,
                session.instance,
            );
            let rounds = match &published {
                Ok(_) | Err(PublishError::EmptyClusterSegmentsBelowTheFloor(_)) => writer
                    .rounds_of_the_last_allocation_record_tree_settling()
                    .to_string(),
                Err(_) => "none".to_owned(),
            };
            println!(
                "PROBE_RESULT shape={name} unit_area_slots={unit_area_slots} every_device_lacks_an_empty_segment={every_device_lacks_an_empty_segment} common_free_slots={common_free_slots} rounds={rounds} wall_ms={} outcome={}",
                started.elapsed().as_millis(),
                outcome_of(&published)
            );
        })
        .expect("起发布线程");
    publishing_thread.join().expect("发布线程跑完");
}

/// 对照：实现员新目标的形状（as-coded 臂上它钉 4 轮）。
#[test]
fn sparse_leaves_below_a_rich_region() {
    run_the_shape(
        "sparse-leaves-below-a-rich-region",
        ForgedShape::SparseLeavesBelowARichRegion,
        QUARTER_MEBI_SLOTS,
    );
}

#[test]
fn an_aligned_pair_in_every_leaf() {
    run_the_shape(
        "an-aligned-pair-in-every-leaf",
        ForgedShape::AnAlignedPairInEveryLeaf,
        QUARTER_MEBI_SLOTS,
    );
}

#[test]
fn one_unaligned_slot_in_every_leaf_and_two_pairs_in_the_top_leaf() {
    run_the_shape(
        "one-unaligned-slot-in-every-leaf-and-two-pairs-in-the-top-leaf",
        ForgedShape::OneUnalignedSlotInEveryLeafAndTwoPairsInTheTopLeaf,
        MEBI_SLOTS,
    );
}

#[test]
fn pair_at_the_bottom_of_the_richest_leaf() {
    run_the_shape(
        "pair-at-the-bottom-of-the-richest-leaf",
        ForgedShape::PairAtTheBottomOfTheRichestLeaf,
        QUARTER_MEBI_SLOTS,
    );
}

#[test]
fn pair_at_the_bottom_of_the_lowest_leaf() {
    run_the_shape(
        "pair-at-the-bottom-of-the-lowest-leaf",
        ForgedShape::PairAtTheBottomOfTheLowestLeaf,
        QUARTER_MEBI_SLOTS,
    );
}
