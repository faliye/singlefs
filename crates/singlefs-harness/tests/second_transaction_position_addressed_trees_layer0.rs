//! 分配记录树与 extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1–K4）的层 0：
//! 两棵按位置寻址的树每一种新写序进崩溃点。**起点镜像不枚举，只录被判的那一次发布（或那一次挂载）**，同
//! `second_transaction_supplement_two_tree_split_layer0.rs` 的切法：之前那一大截全部施加成起点镜像，录制流从那一次的第一个写切起。
//! 每个状态跑两遍恢复（看 / 不看 journal）+ 多版本 oracle + 池级 checker（两棵树逐节点按位置核 I-1.1、每个节点有分配记录 I-3.10）
//! + 记录核对器。
//!
//! 五条流：
//! - `ExtentInlineToLowerSegment`：一个数据单元的文件（上段叶条目内联，标签 2）顺序写成两个单元：下段长出一片根兼叶、
//!   上段那条条目换成下段根指针（标签 1）；
//! - `ExtentLowerSegmentBackToInline`：两个单元的文件顺序写回一个单元：下段整段换下、上段条目换回内联；
//! - `ExtentLowerSegmentGrowsToTwoLevels`：144 个单元（下段一片叶装满）顺序写成 145 个：下段两片叶上面长出第 1 层的根；
//! - `AllocationRecordTreeTwoLeavesPerDevice`：连着覆盖写到落点越过单元区起点那片叶（叶 61 罩槽 [49532, 50344)）：
//!   那一次每块盘各重写叶 61（释放）与叶 62（分配）两片、第 1 层与根；
//! - `VersionWithoutFileRowPublishWithItsOwnTree`：树表 0 条的一版上写 370 行（两片实例表）：写行那次按位置建起这一版自己的
//!   分配记录树（根、每盘第 1 层与叶 61），之后零单元暖机。录的是那一次可写挂载的整串（取号、写行、暖机）。
//!
//! **比已有的流多罩了什么**：已有的层 0 流里分配记录树恒是一个节点、extent 树恒是根兼叶装 extent 记录；这五条里一次发布写出分配记录树
//! 5–7 个节点（叶、第 1 层、根，与别的单元同一段，D8（核心索引结构） 已定项 14 不加写序步骤、不加屏障），单元写段里的子集第一次包括
//! 「新叶落了、新根没落」「一块盘的叶落了、另一块盘的没落」这一类；extent 树第一次出现上段叶条目的三种标签之间的换代与下段的长高、
//! 缩回。多跑的一步：前四条只有恢复本身；第五条多一次取号与写行、暖机（可写挂载），恢复落在树表 0 条那几代上（只许报没有文件）。
//! 与已有流的基线镜像、写表、段序列都不相同，各自全量枚举（第三条的单元写段与记录段太大、全量枚举不出来，见那条用例的注）。
//!
//! 按派活的约束这一份只要求编译通过，一条都没跑；快档与全量交 crash-verifier 跑。

mod common;
mod common_tree_split;

use common::{
    file_content, geometry, memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use common_tree_split::TreeSplitPool;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::extent_tree::{ExtentLowerNodePosition, ExtentUpperNodePosition};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    acquire_instance, publish_sequential_write, CodeTwoTreeNodeCapacities, FirstFile, PoolVersion,
    PoolWriter, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, writes_and_segments, Layer0Tally,
    MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + seed * 5 + 3) % 251).expect("小于 256"))
        .collect()
}

/// 接着现行那一版顺序写一次整份内容（产品容量），录制流切点记在这一次之前。
fn sequential_write(pool: &mut TreeSplitPool, content: &[u8]) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let operations_before = pool.stream.operations().len();
    let mut writer = PoolWriter::new(&publish_parameters, pool.devices.as_mut_slice());
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        previous.root.instance,
    )
    .expect("顺序写");
    pool.earlier_versions.push(previous);
    pool.output = output;
    pool.operations_before_the_current_version = operations_before;
}

/// 一条流：被录的那一次是什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PositionAddressedStream {
    ExtentInlineToLowerSegment,
    ExtentLowerSegmentBackToInline,
    ExtentLowerSegmentGrowsToTwoLevels,
    AllocationRecordTreeTwoLeavesPerDevice,
    VersionWithoutFileRowPublishWithItsOwnTree,
}

impl PositionAddressedStream {
    /// 单元写段与记录段都装得下全量枚举的那几条（第三条那一次 145 个数据单元：单元写段约 300 写、记录段 290 写）。
    const ENUMERABLE_IN_FULL: [PositionAddressedStream; 4] = [
        PositionAddressedStream::ExtentInlineToLowerSegment,
        PositionAddressedStream::ExtentLowerSegmentBackToInline,
        PositionAddressedStream::AllocationRecordTreeTwoLeavesPerDevice,
        PositionAddressedStream::VersionWithoutFileRowPublishWithItsOwnTree,
    ];

    const ALL: [PositionAddressedStream; 5] = [
        PositionAddressedStream::ExtentInlineToLowerSegment,
        PositionAddressedStream::ExtentLowerSegmentBackToInline,
        PositionAddressedStream::ExtentLowerSegmentGrowsToTwoLevels,
        PositionAddressedStream::AllocationRecordTreeTwoLeavesPerDevice,
        PositionAddressedStream::VersionWithoutFileRowPublishWithItsOwnTree,
    ];
}

struct PreparedStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 被录那一次之前与之后两版带文件的版本（内容按顺序写的那一份），与那一次录下的写表、段、被判的根槽写。
fn prepared_from_the_current_publish(
    pool: &TreeSplitPool,
    content_before: Vec<u8>,
    content_after: Vec<u8>,
) -> PreparedStream {
    let before = pool.version_before_the_current_one();
    let after = &pool.output;
    let operations = pool.retained_operations();
    let (writes, segments) = writes_and_segments(
        &operations[pool.operations_before_the_current_version..],
        &geometry(),
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 1, "只录了一次发布：一次根槽写");
    PreparedStream {
        base: pool.memory_pool_before_the_current_version(),
        writes,
        segments,
        judged_root_index: root_indexes[0],
        versions: vec![
            PublishedVersion {
                instance: before.root.instance,
                checkpoint_txg: before.root.checkpoint_txg,
                content: content_before,
            },
            PublishedVersion {
                instance: after.root.instance,
                checkpoint_txg: after.root.checkpoint_txg,
                content: content_after,
            },
        ],
    }
}

fn lower_segment_shape(pool: &TreeSplitPool) -> Vec<ExtentLowerNodePosition> {
    pool.output
        .extent_tree
        .lower_nodes
        .iter()
        .map(|(position, _)| *position)
        .collect()
}

fn prepare_an_extent_stream(stream: PositionAddressedStream) -> PreparedStream {
    let mut pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let lower_leaf = |index| ExtentLowerNodePosition { level: 0, index };
    let (content_before, content_after, shape_after) = match stream {
        PositionAddressedStream::ExtentInlineToLowerSegment => {
            let two_units = content_needing(2, 1);
            sequential_write(&mut pool, &two_units);
            (file_content(), two_units, vec![lower_leaf(0)])
        }
        PositionAddressedStream::ExtentLowerSegmentBackToInline => {
            let two_units = content_needing(2, 1);
            sequential_write(&mut pool, &two_units);
            let one_unit = content_needing(1, 2);
            sequential_write(&mut pool, &one_unit);
            (two_units, one_unit, Vec::new())
        }
        PositionAddressedStream::ExtentLowerSegmentGrowsToTwoLevels => {
            let full_leaf = content_needing(144, 3);
            sequential_write(&mut pool, &full_leaf);
            assert_eq!(lower_segment_shape(&pool), vec![lower_leaf(0)]);
            let one_more = content_needing(145, 4);
            sequential_write(&mut pool, &one_more);
            (
                full_leaf,
                one_more,
                vec![
                    lower_leaf(0),
                    lower_leaf(1),
                    ExtentLowerNodePosition { level: 1, index: 0 },
                ],
            )
        }
        PositionAddressedStream::AllocationRecordTreeTwoLeavesPerDevice
        | PositionAddressedStream::VersionWithoutFileRowPublishWithItsOwnTree => {
            unreachable!("这两条不是 extent 树那几条流")
        }
    };
    assert_eq!(
        lower_segment_shape(&pool),
        shape_after,
        "{stream:?}：被录那一次之后下段的样子"
    );
    assert_eq!(
        pool.output
            .extent_tree
            .upper_nodes
            .iter()
            .map(|(position, _)| *position)
            .collect::<Vec<_>>(),
        vec![ExtentUpperNodePosition { level: 0, index: 0 }],
        "{stream:?}：上段只有 inode 1 所在那片根兼叶"
    );
    prepared_from_the_current_publish(&pool, content_before, content_after)
}

/// 这一次发布在两块盘上都重写了叶 61 与叶 62。
fn rewrites_two_leaves_on_every_device(rewritten: &[TransactionUnit]) -> bool {
    DISKS.iter().all(|device| {
        [61, 62].iter().all(|index_in_device| {
            rewritten.contains(&TransactionUnit::AllocationTreeNodeBelowTheRoot(
                AllocationRecordTreeNodePosition {
                    level: 0,
                    device: *device,
                    index_in_device: *index_in_device,
                },
            ))
        })
    })
}

fn prepare_the_allocation_record_tree_stream() -> PreparedStream {
    let mut pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let mut content_before = file_content();
    let mut content_after = file_content();
    // 迭代上界 20 次覆盖写（每次每盘占 14 槽，叶 61 在单元区里只剩 168 槽，十几次之内就越过去）；跨轮携带的是现行那一版的内容。
    for overwrite_number in 0..20 {
        content_before = content_after;
        content_after = content_needing(1, 10 + overwrite_number);
        pool.overwrite(&content_after, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
        if rewrites_two_leaves_on_every_device(&pool.output.rewritten) {
            break;
        }
    }
    assert!(
        rewrites_two_leaves_on_every_device(&pool.output.rewritten),
        "二十次覆盖写之内有一次在两块盘上都重写了叶 61 与叶 62：{:?}",
        pool.output.rewritten
    );
    assert!(
        pool.output
            .allocation_record_tree
            .pointer_of(AllocationRecordTreeNode::Root)
            .is_some(),
        "这一版的分配记录树有根"
    );
    prepared_from_the_current_publish(&pool, content_before, content_after)
}

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn sparse_devices(stream: &SharedStream) -> Devices {
    DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// mkfs → 可写挂载一次（实例 1，零单元写行）→「取号之后崩溃」369 次 → 被录的那一次可写挂载：写 [1, 371) 共 370 行、两片实例表，
/// 写行那次按位置建起这一版自己的分配记录树，之后零单元暖机。树表 0 条的一版一个文件都没有：`versions` 为空（oracle 只许报没有文件）。
fn prepare_the_version_without_file_stream() -> PreparedStream {
    let stream = SharedStream::retaining_contents();
    let mut devices = sparse_devices(&stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        for _ in 0..369 {
            acquire_instance(&mut writer).expect("取号之后崩溃");
        }
    }
    let operations_before_the_mount = stream.operations().len();
    let base = {
        let mut base = MemoryPool::with_devices(&DISKS, IMAGE_BYTES);
        base.apply(&stream.retained_operations()[..operations_before_the_mount]);
        base
    };
    assert_eq!(
        base,
        memory_pool_of_sparse_devices(&devices),
        "起点镜像就是挂载之前的盘面"
    );
    let mounted = mount_writable(&parameters(), &mut devices).expect("树表 0 条的一版上写两片");
    let PoolVersion::WithoutFile(row_publish) = &mounted.output.row_publish else {
        panic!("树表 0 条的一版上写行，写出来的仍是没有文件版本的一版")
    };
    assert!(
        row_publish
            .rewritten
            .iter()
            .filter(|role| matches!(role, TransactionUnit::AllocationTreeNodeBelowTheRoot(_)))
            .count()
            >= 2,
        "写行那次建起这一版自己的分配记录树，根之下每块盘至少一个节点：{:?}",
        row_publish.rewritten
    );
    let operations = stream.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[operations_before_the_mount..], &geometry());
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    PreparedStream {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("写行与暖机各一条根槽写"),
        versions: Vec::new(),
    }
}

fn prepare(stream: PositionAddressedStream) -> PreparedStream {
    match stream {
        PositionAddressedStream::ExtentInlineToLowerSegment
        | PositionAddressedStream::ExtentLowerSegmentBackToInline
        | PositionAddressedStream::ExtentLowerSegmentGrowsToTwoLevels => {
            prepare_an_extent_stream(stream)
        }
        PositionAddressedStream::AllocationRecordTreeTwoLeavesPerDevice => {
            prepare_the_allocation_record_tree_stream()
        }
        PositionAddressedStream::VersionWithoutFileRowPublishWithItsOwnTree => {
            prepare_the_version_without_file_stream()
        }
    }
}

fn assert_clean(stream: PositionAddressedStream, tally: &Layer0Tally) {
    assert_eq!(
        tally.violations, 0,
        "{stream:?} oracle：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "{stream:?} 不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "{stream:?} 走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "{stream:?} 记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{stream:?} {invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 枚举一条流：`expand` 为真的段展开任意子集，别的段只以整段持久进入后面的状态。
fn enumerate(
    stream: PositionAddressedStream,
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    let prepared = prepare(stream);
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .enumerate()
        .filter(|(index, segment)| expand(*index, segment))
        .map(|(_, segment)| segment.clone())
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "{stream:?}：展开的段按闭式数"
    );
    println!(
        "LAYER0_POSITION_ADDRESSED stream={stream:?} segments={:?} states={} closed_form_of_every_segment={} root_persisted_states={} journal_differing_states={} verification_ran_states={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        prepared.segments.iter().map(Vec::len).collect::<Vec<_>>(),
        tally.states,
        closed_form_state_count(&prepared.segments),
        tally.root_persisted_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(stream, &tally);
    tally
}

/// 平时跑的那一份：每条流写数不到 10 的段全展开（记录段、根槽、系统配置槽轮换，第五条的取号与暖机那几段），
/// 单元写段只以整段持久进入后面的状态——「单元全落了、记录落了一份、根没落」这一类每条流都跑到。
#[test]
fn every_position_addressed_tree_stream_recovers_cleanly_in_every_state_of_its_small_segments() {
    for stream in PositionAddressedStream::ALL {
        let tally = enumerate(stream, &|_segment_index, segment| segment.len() < 10);
        assert!(tally.states > 1, "{stream:?}：至少展开了一段");
    }
}

/// 全量：单元写段的全部子集也展开（分配记录树那一条单元写段约 2 × 14 写，状态数上亿级；两条 extent 流约 2 × 10 写）。
/// 第三条（145 个数据单元）不在里面：单元写段约 300 写、记录段 290 写，全量枚举不出来——那一格要按段内取样，条款与装置都没有，交回里写明。
#[test]
#[ignore = "单元写段全展开，状态数上亿、每个两遍恢复 + checker；交 crash-verifier 在 release 下跑"]
fn full_enumeration_of_every_enumerable_position_addressed_tree_stream_is_exhaustive_and_clean() {
    for stream in PositionAddressedStream::ENUMERABLE_IN_FULL {
        let tally = enumerate(stream, &|_segment_index, _segment| true);
        assert!(tally.states > 1, "{stream:?}");
    }
}
