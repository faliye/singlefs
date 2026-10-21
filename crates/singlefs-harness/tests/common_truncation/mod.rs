//! 截断与变长用例的搭建（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1–K5、K10–K12）：在同一个进程里接着 `pool.output`
//! 发改 size 的那一次与之后的各批、读回第一个文件（挂载态读者与冷走读两条路）、取上段叶条目的标签与 logged_ops 里的截断条目。
//! 用这一份的测试文件同时声明 `mod common;` 与 `mod common_deletion_in_progress;`（后者要 `mod common_pointer_head_corruption;`）。
#![allow(dead_code, reason = "几份测试文件各自只用到其中一部分")]

use singlefs_core::address::{CheckpointTxg, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::extent_tree::{ExtentUpperLeafEntry, ExtentUpperLeafTarget};
use singlefs_core::logged_operations::{live_truncation_intent_of_the_entries, TruncationIntent};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::recovery::walk_to_file;
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_size_change, publish_truncation_step, walk_the_deletion_queue_until_it_is_empty,
    PoolWriter, PublishError, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::history::{
    ContentChoice, ContentLength, GeneratedHistory, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, TruncationChoice, TruncationTarget,
};
use singlefs_harness::memory_pool::MemoryPool;

use crate::common::{parameters, BuiltPool};

/// 改 size 那几次都在新池新建文件那个实例里发。
pub const INSTANCE_OF_THE_SIZE_CHANGES: InstanceGeneration = InstanceGeneration(1);
/// 第一个文件。
pub const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
/// 改 size 那次写进记录三个时间秒的值。
pub const WRITE_TIME_SECONDS_OF_THE_SIZE_CHANGE: u64 = 1_788_000_200;

/// 数据单元的净荷容量（32634）。
pub fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 接着 `pool.output` 发一次 `inode` 的改 size（N = S 只改记录、变长一次发布、截断是意图那次）；发成了池接到它那一版上。
pub fn publish_size_change_of_and_advance(
    pool: &mut BuiltPool,
    inode: InodeNumber,
    new_size: u64,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let published = publish_size_change(
        &mut writer,
        &mut pool.allocator,
        &previous,
        inode,
        new_size,
        WRITE_TIME_SECONDS_OF_THE_SIZE_CHANGE,
        INSTANCE_OF_THE_SIZE_CHANGES,
    )?;
    pool.output = published.clone();
    Ok(published)
}

/// 第一个文件改 size，发不成就停下。
pub fn change_the_size_of_the_first_file_and_advance(
    pool: &mut BuiltPool,
    new_size: u64,
) -> TransactionOutput {
    publish_size_change_of_and_advance(pool, FIRST_FILE, new_size)
        .unwrap_or_else(|refusal| panic!("第一个文件改成 {new_size} 字节：{refusal:?}"))
}

/// 接着 `pool.output` 发截断的下一步（一批或最后一批）。
pub fn publish_the_next_truncation_step_and_advance(
    pool: &mut BuiltPool,
    batch_size: UnlinkBatchSize,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let published = publish_truncation_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        batch_size,
        INSTANCE_OF_THE_SIZE_CHANGES,
    )
    .unwrap_or_else(|refusal| panic!("截断的下一步：{refusal:?}"));
    pool.output = published.clone();
    published
}

/// 共用走空走到底（`deleted_inodes` → 销毁中的快照 → 截断），交回发成的每一版；池接到最后一版上。
pub fn walk_the_queues_and_advance(
    pool: &mut BuiltPool,
    batch_size: UnlinkBatchSize,
) -> Vec<TransactionOutput> {
    let publish_parameters = parameters();
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        batch_size,
        INSTANCE_OF_THE_SIZE_CHANGES,
    );
    walk.stopped_by.expect("走空走到底");
    pool.output = current;
    walk.steps_published
}

/// 这一版 logged_ops 里那一条截断条目（没有交回 `None`）。
pub fn truncation_entry_of(
    version: &TransactionOutput,
) -> Option<(CheckpointTxg, TruncationIntent)> {
    live_truncation_intent_of_the_entries(&version.logged_operations)
}

/// 这一版 extent 树上段里第一个文件那一条叶条目的载荷（第一版上段只有根兼叶）。
pub fn upper_leaf_target_of_the_first_file(version: &TransactionOutput) -> ExtentUpperLeafTarget {
    let leaf = parse_index_node(&version.unit(TransactionUnit::ExtentRoot).bytes)
        .expect("这一版的 extent 树根解得开");
    leaf.entries
        .iter()
        .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("上段叶条目解得开"))
        .find(|entry| entry.inode == FIRST_INODE_NUMBER)
        .expect("上段叶里有第一个文件那一条")
        .target
}

/// 上段叶条目的标签字节（0 到 4）。
pub fn upper_leaf_tag_of_the_first_file(version: &TransactionOutput) -> u8 {
    match upper_leaf_target_of_the_first_file(version) {
        ExtentUpperLeafTarget::NoDataUnit => 0,
        ExtentUpperLeafTarget::LowerSegmentRoot(_) => 1,
        ExtentUpperLeafTarget::InlineDataUnit(_) => 2,
        ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_) => 3,
        ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_) => 4,
    }
}

/// 挂载态读者（`mounted_read`）按 `root` 打开第一个文件、从 0 读到 size。
pub fn first_file_read_by_the_mounted_reader(image: &MemoryPool, root: &RootRecord) -> Vec<u8> {
    let mounted = open_pool_for_read(image, root).expect("打开挂载态");
    let file = mounted
        .open_file(image, FIRST_FILE)
        .expect("打开第一个文件");
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size")
        .bytes
}

/// 冷走读（`recovery::walk_to_file`）按 `root` 读第一个文件。
pub fn first_file_read_by_the_cold_walk(image: &MemoryPool, root: &RootRecord) -> Vec<u8> {
    let mut mapping_fallbacks = 0;
    walk_to_file(image, root, &mut mapping_fallbacks)
        .expect("冷走读")
        .expect("有第一个文件")
}

/// 两条读路径读回的第一个文件都等于 `expected`。
pub fn assert_both_readers_read(
    image: &MemoryPool,
    root: &RootRecord,
    expected: &[u8],
    what: &str,
) {
    let mounted = first_file_read_by_the_mounted_reader(image, root);
    assert!(
        mounted == expected,
        "{what}：挂载态读者读出 {} 字节，期望 {} 字节（首个不同在 {:?}）",
        mounted.len(),
        expected.len(),
        mounted
            .iter()
            .zip(expected)
            .position(|(read, wanted)| read != wanted)
    );
    let cold = first_file_read_by_the_cold_walk(image, root);
    assert!(
        cold == expected,
        "{what}：冷走读读出 {} 字节，期望 {} 字节（首个不同在 {:?}）",
        cold.len(),
        expected.len(),
        cold.iter()
            .zip(expected)
            .position(|(read, wanted)| read != wanted)
    );
}

/// 只供测试的删除开关，每批一个数据单元（截断与 unlink 共用 `UnlinkBatchSize`，判决 m4-trunc-r1 K2）。
pub fn deletion_switches_with_batches_of_one() -> singlefs_core::unlink::DeletionSwitches {
    singlefs_core::unlink::DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            singlefs_core::unlink::UnlinkBatchDataUnitsCappedForTests::new(1)
                .expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity:
            singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity:
            singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries:
            singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 只供测试的批大小 1。
pub fn batches_of_one() -> UnlinkBatchSize {
    deletion_switches_with_batches_of_one().batch_size
}

/// 关掉镜像、重开并可写挂载（每批一个数据单元的删除开关）：挂载续做照共用走空走完承诺类队列。
pub fn remount_with_batches_of_one(
    pool: &mut BuiltPool,
) -> (
    singlefs_core::mount::Mounted,
    Vec<(
        singlefs_core::address::DeviceIdentity,
        crate::common::Recorded,
    )>,
) {
    let mut devices = pool.reopen_recorded();
    let mounted =
        singlefs_core::mount::mount_writable_with_test_only_switches_and_deletion_switches(
            &parameters(),
            &mut devices,
            singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
            singlefs_core::mount::ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
        .expect("重开可写挂载");
    (mounted, devices)
}

/// 随机历史里的一次截断（`selector` 照 [`TruncationChoice`] 的读法取目标 size）。
pub fn truncate_in_a_history(target: TruncationTarget, selector: u64) -> HistoryOperation {
    HistoryOperation::Truncate(TruncationChoice { target, selector })
}

/// 走顺序写入口的 `data_units` 个单元的内容：前面的单元装满净荷，最后一个装 1 + `selector` 字节（`selector` 小于净荷）。
pub fn content_of_data_units_in_a_history(data_units: u64, selector: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units,
            selector,
        },
        fill_seed: 5,
    }
}

/// 写死的那一段走遍截断目标每一类的历史（起点是第一个文件；P 是净荷 32634 字节）。harness 档与理想模型逐版比的那一条
/// （`tests/truncation_random_histories_against_the_model.rs`）与 checker 档每一步镜像经判定库过池级 checker 的那一条
/// （`crates/singlefs-checker-tier/tests/crash_given_states_of_every_step_of_the_fixed_truncation_history.rs`）跑的都是这一段：
/// 0 顺序写 300 个单元（299 P + 78 字节）；1 缩到单元边界 P（单元 0 已满不重写，299 个挂在 size 之外：意图、两批各 144 个、最后一批 11 个）；
/// 2 跨单元变长到 3 P + 1（单元 0 已满不重写，标签 2 换成 4：只重写上段根）；3 截回洞里、恰在单元 0 末尾 P（洞整个截掉：意图那次只为换标签
/// 重写上段根，标签 4 换回 2，最后一批不放单元，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第六节 S1）；4 变长到 P + 6（跨进单元 1 留洞，标签 4）；
/// 5 截到单元 0 中间 1 字节（重写单元 0、换回标签 2，最后一批不放单元）；6 顺序写 150 个单元（149 P + 10）；7 截到单元 143 中间（143 P + 21：重写单元 143，
/// 留下 144 个单元，罩着它的那片下段叶到根重写；6 个挂在外面，最后一批放掉）；8 回退到根环里次新的那条（第 7 步的意图那一版：截断半路，截断条目取回退目标的，
/// K1）；9 缩到 0（会话先把回退带回来的那次截断走完、再发意图：单元 0 重写成声明长度 0，143 个挂在外面，最后一批放掉）；10 改成现在的 size 0
/// （只改记录，K12）；
/// 截断字段那两步（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R5、R6 必红世界）：11 顺序写 5 个单元（4 P + 1，整份写清零）；
/// 12 缩到单元边界 P（放 4 个，意图那次写字段）；13 回退到根环里第三新的那条（第 11 步那一版：写字段之前，字段随它是 0，R5）；14 跨单元变长到
/// 5 P + 1（照抄 0：回退若把第 12 步的字段并回来，扫描重建判死单元 1–3）；15 缩到单元边界 P（放 4 个，写字段）；16 跨单元变长到 4 P；17 截回洞里、
/// 选择子 2 P 落在洞里更靠后（3 P：一个不放，照抄第 15 步的字段；写成「最近一次」的（17, 3 P）时扫描重建认回单元 1、2，攻方
/// `research/prompts/m4-trunc-r2-opus-output.md`「打中一」那一形）；18 跨单元变长到 5 P；
/// 19 unlink；20 建一个 inode；21 截断点名那个 inode（size 0 改成 6：第一版拒，调度记录「截甲交回」第 ① 处）。
pub fn history_through_every_truncation_target() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(content_of_data_units_in_a_history(300, 77)),
            truncate_in_a_history(TruncationTarget::ToADataUnitBoundary, 0),
            truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, 4),
            truncate_in_a_history(TruncationTarget::BackIntoTheHole, 0),
            truncate_in_a_history(TruncationTarget::GrowingWithinTheLastDataUnit, 5),
            truncate_in_a_history(TruncationTarget::IntoTheMiddleOfADataUnit, 3),
            HistoryOperation::PublishOverwrite(content_of_data_units_in_a_history(150, 9)),
            truncate_in_a_history(TruncationTarget::IntoTheMiddleOfADataUnit, 143),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 1,
            }),
            truncate_in_a_history(TruncationTarget::ToZero, 0),
            truncate_in_a_history(TruncationTarget::ToTheCurrentSize, 0),
            HistoryOperation::PublishOverwrite(content_of_data_units_in_a_history(5, 0)),
            truncate_in_a_history(TruncationTarget::ToADataUnitBoundary, 0),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, 0),
            truncate_in_a_history(TruncationTarget::ToADataUnitBoundary, 0),
            truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, 2),
            truncate_in_a_history(TruncationTarget::BackIntoTheHole, 2 * payload_capacity()),
            truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, 1),
            HistoryOperation::Unlink,
            HistoryOperation::CreateInode,
            truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, 5),
        ],
    }
}
