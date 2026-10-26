//! 第一个文件之外的 inode 截断三条崩溃流的录法（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第五节「别的 inode 的截断崩溃流」，
//! 截断判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K9 那三条推到别的 inode）：公共前缀是 `common::build_pool` 那一段
//! （mkfs → 取号 → 暖机 × 2 → 新池新建文件 A，第一个文件 3000 字节），接着在第一个文件旁边建 inode 2、按 inode 号把它顺序写成三个数据单元
//! （两个半净荷，同多记录流 C 那三个单元的样子）；之后按流截 inode 2，截断的各批按只供测试的批大小 1 放（同 K9）：
//! - 缩到 0：截断意图（inode 2 的 size 写成 0，单元 0 重写成声明长度 0，单元 1、2 在 size 之外挂着、截断条目点名 inode 2）→ 一批（放单元 2）
//!   → 最后一批（放单元 1、同一次发布删截断条目，inode 2 那一条上段叶条目换成内联那一个单元）；
//! - 缩到单元中间再变长：缩到一个半净荷的意图（单元 1 重写成半个净荷，单元 2 在 size 之外挂着）→ 最后一批（放单元 2、删条目）
//!   → 变长到三个半净荷（一次发布：单元 1 补零重写成一整个净荷，之后到 size 是洞，标签 3，判决 K5）；
//! - 快照下缩：建快照 S → 缩到半个净荷的意图（单元 0 重写，被快照引用、换下的单元进 deadlist、映射条目留存，判决 K4）→ 一批 → 最后一批
//!   （放掉的尾部单元被快照引用，同样进 deadlist、不释放）。
//!
//! 第一个文件从 A 起一个字节都不动：崩溃之后读者读到的第一个文件恒是 A 的内容，inode 2 读到截前或截后（K2「先落目标」推到别的 inode）。
//! 这一份只引 harness 档与实现，不引 checker 档：崩溃点重放的判法、观察者与计数都在 checker 档那一份
//! （`tests/common_truncation_of_an_inode_other_than_the_first_crash_streams/mod.rs`）。用这一份的测试文件同时声明 `mod common;`、
//! `mod common_deletion_in_progress;`、`mod common_pointer_head_corruption;`、`mod common_snapshot_prefix;` 与 `mod common_truncation;`（都在 crate 根）。
#![allow(
    dead_code,
    reason = "三条流的测试文件各只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use crate::common::{build_pool, file_content, geometry, parameters, BuiltPool};
use crate::common_deletion_in_progress::{
    publish_new_inodes_and_advance_the_pool, INSTANCE_OF_THE_PUBLISHES,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use crate::common_truncation::{
    batches_of_one, payload_capacity, publish_size_change_of_and_advance,
    publish_the_next_truncation_step_and_advance, truncation_entry_of,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
use singlefs_core::extent_tree::{ExtentUpperLeafEntry, ExtentUpperLeafTarget};
use singlefs_core::transaction::{
    publish_sequential_write_of_an_inode, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 被截的那个 inode：新池新建文件之后在第一个文件旁边建的第一个号。
pub const THE_TRUNCATED_INODE: u64 = 2;

/// 三条流里的哪一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncationOfAnInodeOtherThanTheFirstCrashStream {
    /// 缩到 0。
    DownToZero,
    /// 缩到单元中间再变长。
    IntoAUnitThenExtended,
    /// 快照下缩。
    UnderASnapshot,
}

/// 按号写完 inode 2 之后的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncationOfAnInodeOtherThanTheFirstStreamStep {
    /// 建快照 S（快照下缩那一条）。
    CreateSnapshot,
    /// inode 2 的截断意图：一次发布写它那条记录的 size = 目标、签截断条目、按需重写它的尾单元（判决 K2、K3 推到别的 inode）。
    TruncationIntent { new_size_in_bytes: u64 },
    /// 不是最后一批的一批：从 inode 2 尾部放一个单元，截断条目留着。
    TruncationBatch,
    /// 最后一批：放完 inode 2 size 之外剩下的、同一次发布删截断条目（判决 K2、K11）。
    TruncationLastBatch,
    /// inode 2 变长：一次发布做完，不立意图（判决 K5）。
    Extension { new_size_in_bytes: u64 },
}

impl TruncationOfAnInodeOtherThanTheFirstStreamStep {
    /// 观察者落到这一版时加一的计数名。
    #[must_use]
    pub fn landed_counter_name(self) -> &'static str {
        match self {
            Self::CreateSnapshot => "landed_on_create_snapshot",
            Self::TruncationIntent { .. } => "landed_on_truncation_intent",
            Self::TruncationBatch => "landed_on_truncation_batch",
            Self::TruncationLastBatch => "landed_on_truncation_last_batch",
            Self::Extension { .. } => "landed_on_extension",
        }
    }

    /// 接着池的现行那一版发这一步，池接到它那一版上。
    pub fn publish(self, pool: &mut BuiltPool) -> TransactionOutput {
        match self {
            Self::CreateSnapshot => {
                let created = crate::common_snapshot_prefix::create_snapshot(pool);
                pool.output = created.clone();
                created
            }
            Self::TruncationIntent { new_size_in_bytes }
            | Self::Extension { new_size_in_bytes } => publish_size_change_of_and_advance(
                pool,
                InodeNumber(THE_TRUNCATED_INODE),
                new_size_in_bytes,
            )
            .unwrap_or_else(|refusal| {
                panic!("inode {THE_TRUNCATED_INODE} 改成 {new_size_in_bytes} 字节：{refusal:?}")
            }),
            Self::TruncationBatch | Self::TruncationLastBatch => {
                publish_the_next_truncation_step_and_advance(pool, batches_of_one())
            }
        }
    }

    /// 这一步之后 inode 2 读出的内容（前一版是 `content_before`）：改 size 的截到它或补 0 到它，别的不动内容。
    fn content_after(self, content_before: &[u8]) -> Vec<u8> {
        match self {
            Self::TruncationIntent { new_size_in_bytes }
            | Self::Extension { new_size_in_bytes } => {
                let new_size =
                    usize::try_from(new_size_in_bytes).expect("这几条流的 size 装得进 usize");
                let mut content = content_before[..new_size.min(content_before.len())].to_vec();
                content.resize(new_size, 0);
                content
            }
            Self::CreateSnapshot | Self::TruncationBatch | Self::TruncationLastBatch => {
                content_before.to_vec()
            }
        }
    }
}

/// 这一条流按号写完 inode 2 之后的各步（按计划定死，不按「截断条目删了没有」停：最后一批漏删条目时录流照样录完，红在判器上）。
#[must_use]
pub fn steps_of(
    stream: TruncationOfAnInodeOtherThanTheFirstCrashStream,
) -> Vec<TruncationOfAnInodeOtherThanTheFirstStreamStep> {
    let payload = payload_capacity();
    match stream {
        TruncationOfAnInodeOtherThanTheFirstCrashStream::DownToZero => vec![
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationIntent {
                new_size_in_bytes: 0,
            },
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationBatch,
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationLastBatch,
        ],
        TruncationOfAnInodeOtherThanTheFirstCrashStream::IntoAUnitThenExtended => vec![
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationIntent {
                new_size_in_bytes: payload + payload / 2,
            },
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationLastBatch,
            TruncationOfAnInodeOtherThanTheFirstStreamStep::Extension {
                new_size_in_bytes: 3 * payload + payload / 2,
            },
        ],
        TruncationOfAnInodeOtherThanTheFirstCrashStream::UnderASnapshot => vec![
            TruncationOfAnInodeOtherThanTheFirstStreamStep::CreateSnapshot,
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationIntent {
                new_size_in_bytes: payload / 2,
            },
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationBatch,
            TruncationOfAnInodeOtherThanTheFirstStreamStep::TruncationLastBatch,
        ],
    }
}

/// inode 2 按号写进去的内容：两个半净荷（三个数据单元，同多记录流 C 的单元数），一个 0 都没有（读回里的 0 只来自截断之后的补零与洞）。
#[must_use]
pub fn content_of_the_truncated_inode() -> Vec<u8> {
    let length = 2 * payload_capacity() + payload_capacity() / 2;
    (0..length)
        .map(|index| u8::try_from((index * 13 + 5) % 251 + 1).expect("1 到 251"))
        .collect()
}

/// 流里一版：根身份、第一个文件那一刻的内容、inode 2 那一刻读出的内容（还没建它的那一版是 `None`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheStream {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub first_file_content: Vec<u8>,
    pub truncated_inode_content: Option<Vec<u8>>,
}

/// 录完的一条流：池（录制流在它里面）、建 inode 2 与按号写它之前那一段（A、建 inode、按号写三版），之后每一步与它的输出。
pub struct RecordedTruncationOfAnInodeOtherThanTheFirst {
    pub pool: BuiltPool,
    pub prefix: Vec<VersionOfTheStream>,
    pub steps: Vec<(
        TruncationOfAnInodeOtherThanTheFirstStreamStep,
        TransactionOutput,
    )>,
}

fn version_of(
    output: &TransactionOutput,
    first_file_content: &[u8],
    truncated_inode_content: Option<&[u8]>,
) -> VersionOfTheStream {
    VersionOfTheStream {
        version: (output.root.instance, output.root.checkpoint_txg),
        first_file_content: first_file_content.to_vec(),
        truncated_inode_content: truncated_inode_content.map(<[u8]>::to_vec),
    }
}

/// 不录崩溃点那一路：`build_pool`（取号、暖机两次、A），建 inode 2，按号把它写成三个单元，再按计划发之后的各步。
#[must_use]
pub fn record_the_truncation_of_an_inode_other_than_the_first(
    tag: &str,
    stream: TruncationOfAnInodeOtherThanTheFirstCrashStream,
) -> RecordedTruncationOfAnInodeOtherThanTheFirst {
    let mut pool = build_pool(tag);
    let first_file = file_content();
    let first = pool.output.clone();
    let created = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let content = content_of_the_truncated_inode();
    let written = sequential_write_of_the_truncated_inode_and_advance(&mut pool, &content);
    assert_eq!(
        data_units_of_the_truncated_inode(&written),
        3,
        "按号写完之后 inode 2 三个数据单元"
    );
    let prefix = vec![
        version_of(&first, &first_file, None),
        version_of(&created, &first_file, Some(&[])),
        version_of(&written, &first_file, Some(&content)),
    ];
    let steps = steps_of(stream)
        .into_iter()
        .map(|step| (step, step.publish(&mut pool)))
        .collect();
    RecordedTruncationOfAnInodeOtherThanTheFirst {
        pool,
        prefix,
        steps,
    }
}

/// 同一个进程里按 inode 号把 inode 2 顺序写成 `content`，池接到它那一版上。
fn sequential_write_of_the_truncated_inode_and_advance(
    pool: &mut BuiltPool,
    content: &[u8],
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let written = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(THE_TRUNCATED_INODE),
        content,
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("按 inode 号写 inode {THE_TRUNCATED_INODE}：{refusal:?}"));
    pool.output = written.clone();
    written
}

impl RecordedTruncationOfAnInodeOtherThanTheFirst {
    /// A、建 inode、按号写三版与之后的每一版：第一个文件恒是 A 的内容，inode 2 各批的内容同意图那一版（意图那次已把 size 写成目标）。
    #[must_use]
    pub fn versions(&self) -> Vec<VersionOfTheStream> {
        let mut versions = self.prefix.clone();
        let first_file = file_content();
        let mut content = content_of_the_truncated_inode();
        for (step, output) in &self.steps {
            content = step.content_after(&content);
            versions.push(version_of(output, &first_file, Some(&content)));
        }
        versions
    }

    /// oracle 认的版本表：每一版第一个文件的内容。
    #[must_use]
    pub fn first_file_versions(&self) -> Vec<PublishedVersion> {
        self.versions()
            .into_iter()
            .map(|version| PublishedVersion {
                instance: version.version.0,
                checkpoint_txg: version.version.1,
                content: version.first_file_content,
            })
            .collect()
    }

    /// 按号写完 inode 2 之后每一步那一版的根身份与落到它时加一的计数名（观察者另读的输入）。
    #[must_use]
    pub fn landed_versions(&self) -> Vec<((InstanceGeneration, CheckpointTxg), &'static str)> {
        self.steps
            .iter()
            .map(|(step, output)| {
                (
                    (output.root.instance, output.root.checkpoint_txg),
                    step.landed_counter_name(),
                )
            })
            .collect()
    }

    /// 每一步之后截断条目在不在、inode 2 还剩几个数据单元、它那一条上段叶条目的标签（录流的形状，判器判完之后核：改坏实现时先红在判器上）。
    #[must_use]
    pub fn shape_of_the_steps(&self) -> Vec<(bool, usize, Option<u8>)> {
        self.steps
            .iter()
            .map(|(_step, output)| {
                (
                    truncation_entry_of(output).is_some(),
                    data_units_of_the_truncated_inode(output),
                    upper_leaf_tag_of_the_truncated_inode(output),
                )
            })
            .collect()
    }
}

/// 这一版里 inode 2 的数据单元数（上段里没有它为 0）。
#[must_use]
pub fn data_units_of_the_truncated_inode(version: &TransactionOutput) -> usize {
    version
        .file_other_than_the_first(InodeNumber(THE_TRUNCATED_INODE))
        .map_or(0, |file| file.data_pointers.len())
}

/// 这一版 extent 树上段（只有根兼叶：号都在第 0 片叶里）里 inode 2 那一条的标签（0 到 4）；没有条目交 `None`。
#[must_use]
pub fn upper_leaf_tag_of_the_truncated_inode(version: &TransactionOutput) -> Option<u8> {
    let leaf = parse_index_node(&version.unit(TransactionUnit::ExtentRoot).bytes)
        .expect("这一版的 extent 树根解得开");
    leaf.entries
        .iter()
        .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("上段叶条目解得开"))
        .find(|entry| entry.inode == THE_TRUNCATED_INODE)
        .map(|entry| match entry.target {
            ExtentUpperLeafTarget::NoDataUnit => 0,
            ExtentUpperLeafTarget::LowerSegmentRoot(_) => 1,
            ExtentUpperLeafTarget::InlineDataUnit(_) => 2,
            ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_) => 3,
            ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_) => 4,
        })
}

/// 崩溃点重放那一路要的写表几样：起点镜像（mkfs 之后）、写表、段、每个写在录制流里的下标、被判的根槽写（最后一次根槽写）。
pub struct WritesOfTheRecordedStream {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
}

/// 录完的流切段。
#[must_use]
pub fn writes_of_the_recorded_stream(
    recorded: &RecordedTruncationOfAnInodeOtherThanTheFirst,
) -> WritesOfTheRecordedStream {
    let pool = &recorded.pool;
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("流里有根槽写");
    WritesOfTheRecordedStream {
        base,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
    }
}
