//! 分配记录树根层级的写侧与读侧（D8（核心索引结构） 已定项 14「实现取值」：根层取最小的 R ≥ 1，
//! 使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169）。代码三方 `research/prompts/m2-final-code-r3-main-verification.md`
//! 第三节「算术」那一格、第四节第 3 条：写侧（`AllocationRecordTreeGeometry::of_allocator`，发布时装树）与读侧
//! （`of_reader`，恢复与挂载时读树）此前各算一份「盘上绝对槽数」——写侧「单元区起点 + 单元区槽数」、读侧「盘字节数 ÷ 16384」，
//! 没有一条断言把两份连起来。今天收成一个函数：两侧同走 `of_device_sizes_in_bytes`，盘上绝对槽数只有
//! `allocator::absolute_slot_count_of_device` 一处定义，两侧只差字节数从哪来（分配器建空闲图时记下的、读者报的）。
//!
//! 取样点压在根层级的分界上：每块盘 812 × 84 = 68208 个整槽再多半个槽——字节数不是 16384 的整数倍，盘尾那半个槽也不在单元区里。
//! 按整槽数读，两盘各 84 格、根在第 1 层；把盘尾那半个槽算成一槽就是各 85 格、根在第 2 层。
//! 两侧里只要有一侧换了读法，盘上写下的树就与读的一侧对不上层级：这里先比两侧的几何，再让恢复与可写挂载真去读写侧装的树
//! （读侧核根头里的层级，对不上按 I-1.1 拒，不静默取错根层）。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocation_record_tree::AllocationRecordTreeGeometry;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::{ALLOCATION_RECORD_TREE_LEAF_SLOTS, SLOT_BYTES};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::scenario::e142_parameters;

/// 每块盘 84 片叶那么多个整槽：两盘各 84 格、168 ≤ 169，根在第 1 层的最后一档。
const SLOTS_OF_EIGHTY_FOUR_LEAVES: u64 = ALLOCATION_RECORD_TREE_LEAF_SLOTS * 84;
/// 每块盘的字节数：84 片叶那么多个整槽，再多半个槽。
const DEVICE_BYTES_WITH_HALF_A_SLOT_AT_THE_END: u64 =
    SLOTS_OF_EIGHTY_FOUR_LEAVES * SLOT_BYTES + SLOT_BYTES / 2;
/// journal 环：mkfs 要求不超过设备容量的四分之一（约 1066 MiB 的盘上默认 768 MiB 放不进），取小盘那一档的 128 MiB。
const JOURNAL_RING_BYTES_ON_THESE_DEVICES: u64 = 128 << 20;
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;

fn parameters() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES_ON_THESE_DEVICES;
    parameters
}

fn two_devices_with_half_a_slot_at_the_end() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(
                    DEVICE_BYTES_WITH_HALF_A_SLOT_AT_THE_END,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect()
}

fn content_of(length_in_bytes: usize, seed: usize) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| u8::try_from((index * 13 + seed) % 251).expect("小于 256"))
        .collect()
}

/// 写侧按分配器、读侧按这几块盘算出来的几何逐项相等，根都在第 1 层（盘尾那半个槽不算）。
fn assert_the_writer_and_the_reader_agree_on_the_first_root_level(
    allocator: &PoolAllocator,
    devices: &dyn PoolReader,
    when: &str,
) {
    let of_the_writer = AllocationRecordTreeGeometry::of_allocator(allocator);
    let of_the_reader = AllocationRecordTreeGeometry::of_reader(devices);
    assert_eq!(
        of_the_writer, of_the_reader,
        "{when}：写侧与读侧的几何（每块盘的绝对槽数与根层级）逐项相等"
    );
    assert_eq!(
        of_the_writer.root_level(),
        1,
        "{when}：盘尾那半个槽不算，两盘各 84 格，根在第 1 层"
    );
}

/// 这一版的文件冷走读读回来（读侧按自己的几何读写侧装的分配记录树之外，冷走读还择根、施加前缀、走 extent 树）。
fn assert_the_file_reads_back_cold(
    devices: &dyn PoolReader,
    version: &TransactionOutput,
    content: &[u8],
    when: &str,
) {
    let report = recover(devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (version.root.instance, version.root.checkpoint_txg),
            content: content.to_vec(),
        },
        "{when}：冷走读读回这一版的文件"
    );
}

/// mkfs 同一个进程里写第一个文件（写侧装树），冷走读读回；可写挂载从盘上整棵读回那棵树重建分配器（读侧核根头层级）、
/// 挂载之后写侧与读侧仍逐项相等；挂载之后的覆盖写再装一次树，冷走读读回。每一步两侧的根都在第 1 层，写下的根头也是第 1 层。
#[test]
fn half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back(
) {
    let parameters = parameters();
    let mut devices = two_devices_with_half_a_slot_at_the_end();
    let genesis = make_filesystem(&parameters, &mut devices)
        .expect("字节数不是 16384 整数倍的盘，mkfs 照常接受");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    assert_eq!(
        allocator
            .devices
            .iter()
            .map(|device_map| device_map.device_size_in_bytes())
            .collect::<Vec<u64>>(),
        vec![DEVICE_BYTES_WITH_HALF_A_SLOT_AT_THE_END; 2],
        "分配器建空闲图时记下的就是这两块盘的字节数"
    );
    assert_the_writer_and_the_reader_agree_on_the_first_root_level(
        &allocator,
        &devices,
        "mkfs 之后",
    );

    let first_content = content_of(5000, 1);
    let first = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件")
    };
    assert_eq!(
        first
            .position_addressed_tree_heights_read_from_the_root_node_headers()
            .allocation_record_tree,
        2,
        "写侧装的分配记录树根头自述第 1 层（树高 2）"
    );
    assert_the_file_reads_back_cold(&devices, &first, &first_content, "第一个文件之后");

    let mounted = mount_writable(&parameters, &mut devices)
        .expect("可写挂载：读侧按同一个根层级整棵读回写侧装的分配记录树");
    let instance = mounted.output.instance;
    let mut allocator_of_the_mount = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("第一个文件之后重开，现行那一版带文件");
    assert_the_writer_and_the_reader_agree_on_the_first_root_level(
        &allocator_of_the_mount,
        &devices,
        "可写挂载之后",
    );

    let second_content = content_of(7000, 2);
    let second = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        publish_overwrite(
            &mut writer,
            &mut allocator_of_the_mount,
            &current,
            FirstFile {
                content: &second_content,
                write_time_seconds: WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("挂载之后覆盖写")
    };
    assert_eq!(
        second
            .position_addressed_tree_heights_read_from_the_root_node_headers()
            .allocation_record_tree,
        2,
        "挂载之后写侧装的分配记录树根头仍自述第 1 层"
    );
    assert_the_file_reads_back_cold(&devices, &second, &second_content, "挂载之后覆盖写");
}
