//! 提交内生块回落先取这次发布分配记录树重写集里的叶、不够取空槽最多的那片叶（D3（空间分配） 已定项 8 第 2 条），分配记录树重写集的固定点
//! （`transaction::settle_the_allocation_record_tree`）在一块锻造的盘上几轮就定下。锻造的形状：单元区低处的叶占满，往上 120 片叶每片只留两个空槽，
//! 再往上每段只占段首一槽。回落若取槽号最小的空槽，这次发布的提交内生块一个接一个落进那 120 片稀疏的叶，每进一片新叶就弄脏它、下一轮再进一片，
//! 走满 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`（64）轮也定不下来；先取重写集的叶、不够取空槽最多的叶，落点集中在上面那片空叶里。
//! 盘在进程里造，不读活树；锻造只改分配器的空闲图（不写分配记录），所以这块池上记账与记录对不上，这里不跑池级 checker。

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

/// 6 MiB 环：单元区从槽 1408 起。
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const UNIT_AREA_SLOTS: u64 = 1 << 18;
/// 锻造之前在空盘上覆盖写几次：上一版的提交内生块落在单元区低处的开放段里（叶 1 与叶 2），锻造把它们所在的叶占满。
const OVERWRITES_BEFORE_THE_FORGING: usize = 4;
/// 叶序号小于它的叶整片占满；它这片叶只留叶首 `FREE_SLOTS_AT_THE_START_OF_THE_FIRST_SPARSE_LEAF` 个槽。
const FIRST_SPARSE_LEAF: u64 = 12;
const FREE_SLOTS_AT_THE_START_OF_THE_FIRST_SPARSE_LEAF: u64 = 14;
/// 第一片稀疏叶之后这么多片叶各只留叶内偏移 400、401 两个空槽。
const SPARSE_LEAVES_WITH_TWO_FREE_SLOTS: u64 = 120;
const OFFSETS_OF_THE_TWO_FREE_SLOTS_IN_A_SPARSE_LEAF: std::ops::Range<u64> = 400..402;
/// 先取重写集的叶、不够取空槽最多的叶，这一形上固定点走的轮数（锻造盘重量探针 `prefer_then_richest_leaf` 臂在叶首留 14 槽那一探量到 4 轮；
/// 回落取槽号最小的空槽，同一探走满 64 轮、交回 `AllocationRecordTreeRewriteSetDidNotSettle`）。
const ROUNDS_OF_THE_SETTLING_ON_THE_FORGED_DISK: usize = 4;
/// 锻造之后那次覆盖写的写入时刻比上一次写晚多少秒（与重量探针同一个数，覆盖写的字节逐个相同）。
const SECONDS_FROM_THE_LAST_WRITE_TO_THE_OVERWRITE: u64 = 100;
/// 这一形上覆盖写一次发布走的栈比测试线程的缺省栈深：在一条栈够大的线程上跑。
const STACK_BYTES_OF_THE_PUBLISHING_THREAD: usize = 64 << 20;

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

fn device_bytes() -> u64 {
    (unit_area_start_slot() + UNIT_AREA_SLOTS) * SLOT_BYTES
}

/// 锻造：叶 `FIRST_SPARSE_LEAF` 之前的叶占满；那片叶只留叶首几槽；之后 120 片叶各只留两个空槽；再往上每段只占段首一槽（一个全空段都不剩）。
/// 只动空闲图，不写分配记录。
fn forge_the_sparse_leaves(allocator: &mut PoolAllocator) {
    let start = unit_area_start_slot();
    for device_map in &mut allocator.devices {
        for slot in start..start + device_map.unit_area_slots() {
            let leaf_index = slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
            let offset_in_the_leaf = slot % ALLOCATION_RECORD_TREE_LEAF_SLOTS;
            let is_left_free = if leaf_index < FIRST_SPARSE_LEAF {
                false
            } else if leaf_index == FIRST_SPARSE_LEAF {
                offset_in_the_leaf < FREE_SLOTS_AT_THE_START_OF_THE_FIRST_SPARSE_LEAF
            } else if leaf_index <= FIRST_SPARSE_LEAF + SPARSE_LEAVES_WITH_TWO_FREE_SLOTS {
                OFFSETS_OF_THE_TWO_FREE_SLOTS_IN_A_SPARSE_LEAF.contains(&offset_in_the_leaf)
            } else {
                !(slot - start).is_multiple_of(CLUSTER_SEGMENT_SLOTS)
            };
            if !is_left_free && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 锻造盘上覆盖写第一个文件一次：固定点几轮就定下（之后被合取 7 拒，锻造的盘上一个全空段都没有，那是固定点定下之后的准入，不是这里要钉的）。
/// 判别力：`crates/mutations.tsv` 里回落不取空槽最多的叶那一行（回落退回槽号最小的空槽）让这一次走满 64 轮、交回 `AllocationRecordTreeRewriteSetDidNotSettle`。
#[test]
fn an_overwrite_on_a_disk_whose_free_slots_sit_two_per_leaf_settles_in_a_few_rounds_falling_back_into_the_leaf_with_the_most_free_slots(
) {
    let publishing_thread = std::thread::Builder::new()
        .stack_size(STACK_BYTES_OF_THE_PUBLISHING_THREAD)
        .spawn(|| {
            let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
                parameters(),
                device_bytes(),
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
            forge_the_sparse_leaves(&mut session.allocator);
            for device_map in &session.allocator.devices {
                assert_eq!(
                    device_map.lowest_empty_segment(),
                    None,
                    "锻造的盘上一个全空段都不剩：提交内生块走回落"
                );
            }
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
            assert!(
                matches!(
                    published,
                    Err(PublishError::EmptyClusterSegmentsBelowTheFloor(_))
                ),
                "固定点定下之后被合取 7 拒（锻造的盘上没有全空段），不是轮数用完：{:?}",
                published.as_ref().err()
            );
            assert_eq!(
                writer.rounds_of_the_last_allocation_record_tree_settling(),
                ROUNDS_OF_THE_SETTLING_ON_THE_FORGED_DISK
            );
        })
        .expect("起发布线程");
    publishing_thread.join().expect("发布线程跑完");
}
