//! 多次 COW 的几条崩溃流（里程碑三一·1 的三条与多文件删除那一条的两支）怎么录：只用实现与 harness 档，不引 checker 档
//! （判法、观察者与录制器那一半在同目录的 `mod.rs`）。几条流都接在多记录流「取号 → 暖机 × 2 → A → B → C」之后（C 是第一个文件三个数据单元；
//! 前缀与多记录流、删除流、截断流同一份录法：`common_crash_points` 的 `sequential_write` 与 `record_the_multi_record_publishes`），
//! C 之后的每一步按计划定死（[`MultipleCopyOnWriteCrashStream::steps`]），不按发布的结果停：改坏实现时录流照样录完，红在判器上，不红在搭建上。
//! 崩溃点重放那一路（[`record_the_stream`]，不录崩溃点）与崩溃放量那一路（`mod.rs` 的 `record_through_the_crash_point_recorder`，录制器给
//! C 之后每一步设一个点，设点文件是 `mod.rs`）调同一个 [`StepAfterTheMultiRecordPrefix::publish`]，写表逐字节相同。
//! 按 inode 号写数据一律走按号的入口（`transaction::publish_overwrite_of_an_inode` / `publish_sequential_write_of_an_inode`），第一个文件也是：
//! 号是 1 时它们就是第一个文件那两条路。
#![allow(
    dead_code,
    reason = "几条流的测试文件各只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use std::collections::BTreeMap;

use crate::common::{
    build_pool, geometry, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_crash_points::{content_needing, sequential_write};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_deletion_switches,
    DeletionQueueContinuationOutcome, Mounted, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite_of_an_inode, publish_sequential_write_of_an_inode,
    publish_unlink_intent, publish_unlink_step, PoolWriter, PublishError, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::scenario::{deletion_history_batch_size, deletion_history_switches};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 一版的根身份：实例代号与 checkpoint_txg。
pub type RootIdentity = (InstanceGeneration, CheckpointTxg);

/// C 之后各步写进记录三个时间秒的值：比多记录流的 B、C（`FIXED_WRITE_TIME_SECONDS + 60`）晚。
pub const WRITE_TIME_SECONDS_OF_THE_STEPS_AFTER_THE_PREFIX: u64 = FIXED_WRITE_TIME_SECONDS + 600;

/// 多记录流那个进程的实例代号（新池新建文件与 B、C 都在实例 1 里）。
pub const INSTANCE_OF_THE_PREFIX: InstanceGeneration = InstanceGeneration(1);

/// 第一个文件之后建的空 inode 个数：号从水位 2 起连着发到 150（一次发布，一片 inode 叶容器装得下 233 条，不分裂）。
pub const INODES_CREATED_AFTER_THE_PREFIX: u64 = 149;

/// 落在 extent 上段第 0 片叶之外的号：上段一片叶罩 143 个号（`singlefs_format::EXTENT_TREE_UPPER_LEAF_INODES`），150 在第 1 片叶上，
/// 写它上段从层级 0 长到层级 1（同 harness 档 `writes_by_inode_number_of_several_files.rs` 那个号）。
pub const INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE: u64 = 150;

/// 与第一个文件同在上段第 0 片叶上的另一个号（建出来的第一个）。
pub const INODE_BESIDE_THE_FIRST_FILE_ON_THE_FIRST_UPPER_LEAF: u64 = 2;

/// 多记录流 C 之后第一个文件的数据单元数（`common_crash_points::record_the_multi_record_publishes`）。
pub const DATA_UNITS_OF_THE_FIRST_FILE_AFTER_THE_PREFIX: usize = 3;

/// 写 inode 150 那一次的数据单元数（多 inode 那条流与多文件删除那两支共用的前缀里的那一次）。
pub const DATA_UNITS_WRITTEN_TO_THE_INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE: usize = 2;

/// 大文件那条流：一片下段叶罩 144 个单元（`singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS`），写成 145 个下段长到两层、
/// 再覆盖写成 146 个（两层的下段整份换代）、再写回 144 个（下段缩回一片根兼叶）。
pub const LARGE_FILE_DATA_UNITS_GROWING_THE_LOWER_SEGMENT: usize = 145;
pub const LARGE_FILE_DATA_UNITS_OVERWRITING_THE_TWO_LEVELS: usize = 146;
pub const LARGE_FILE_DATA_UNITS_FILLING_ONE_LOWER_LEAF: usize = 144;

/// 几条流里的哪一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultipleCopyOnWriteCrashStream {
    /// 同一个文件覆盖写不少于 3 次、中间关闭再可写挂载 2 次（跨三个实例）：C 之后关闭再可写挂载（实例 2）、覆盖写成一个单元（三个单元缩成一个，
    /// 多出来的两个单元同一次释放）、再关闭再可写挂载（实例 3）、顺序写成两个单元（内联换成下段）。连同前缀的 B、C，第一个文件覆盖写四次。
    OneFileOverwrittenAcrossTwoWritableRemounts,
    /// 多个 inode 交错覆盖写（并行线三的形态）：建 149 个空 inode（号 2..=150），写 150（两个单元，上段长到层级 1）、写 2（两个单元）、
    /// 覆盖写第一个文件（一个单元）、覆盖写 150（一个单元）、覆盖写 2（一个单元）。
    SeveralInodesOverwrittenInTurn,
    /// 跨多个单元的大文件覆盖写（并行线一的形态）：第一个文件顺序写成 145、146、144 个单元，跨下段叶的边界来回。
    LargeFileOverwrittenAcrossLowerLeaves,
    /// 多文件删除那一条的一支（正推第 7 节「新流 1」）：建 149 个空 inode、写 150 之后，压小批大小（1）先删 150（上段降回层级 0），
    /// 再删第一个文件（有条目的号集合变空，上段成层级 0 的零条目根）。
    OtherFileDeletedBeforeTheFirstFile,
    /// 多文件删除那一条的另一支（同一前缀，正推第 7 节「新流 2」）：先删第一个文件（别的文件还在，上段留在层级 1、第 0 片叶摘掉），再删 150。
    FirstFileDeletedBeforeTheOtherFile,
}

impl MultipleCopyOnWriteCrashStream {
    /// 引擎驱动行与计数行里这条流的名字。
    #[must_use]
    pub fn flow_name(self) -> &'static str {
        match self {
            Self::OneFileOverwrittenAcrossTwoWritableRemounts => {
                "one_file_overwritten_across_two_writable_remounts"
            }
            Self::SeveralInodesOverwrittenInTurn => "several_inodes_overwritten_in_turn",
            Self::LargeFileOverwrittenAcrossLowerLeaves => {
                "large_file_overwritten_across_lower_leaves"
            }
            Self::OtherFileDeletedBeforeTheFirstFile => "other_file_deleted_before_the_first_file",
            Self::FirstFileDeletedBeforeTheOtherFile => "first_file_deleted_before_the_other_file",
        }
    }

    /// C 之后的各步（按计划定死）。
    #[must_use]
    pub fn steps(self) -> Vec<StepAfterTheMultiRecordPrefix> {
        let first_file = FIRST_INODE_NUMBER;
        match self {
            Self::OneFileOverwrittenAcrossTwoWritableRemounts => vec![
                StepAfterTheMultiRecordPrefix::CloseAndMountWritable,
                StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                    inode: first_file,
                    data_units: 1,
                    content_seed: 11,
                },
                StepAfterTheMultiRecordPrefix::CloseAndMountWritable,
                StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                    inode: first_file,
                    data_units: 2,
                    content_seed: 12,
                },
            ],
            Self::SeveralInodesOverwrittenInTurn => {
                let mut steps = steps_through_the_write_outside_the_upper_leaf_of_the_first_file();
                steps.extend([
                    StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                        inode: INODE_BESIDE_THE_FIRST_FILE_ON_THE_FIRST_UPPER_LEAF,
                        data_units: 2,
                        content_seed: 21,
                    },
                    StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                        inode: first_file,
                        data_units: 1,
                        content_seed: 13,
                    },
                    StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                        inode: INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
                        data_units: 1,
                        content_seed: 151,
                    },
                    StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                        inode: INODE_BESIDE_THE_FIRST_FILE_ON_THE_FIRST_UPPER_LEAF,
                        data_units: 1,
                        content_seed: 22,
                    },
                ]);
                steps
            }
            Self::LargeFileOverwrittenAcrossLowerLeaves => vec![
                StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                    inode: first_file,
                    data_units: LARGE_FILE_DATA_UNITS_GROWING_THE_LOWER_SEGMENT,
                    content_seed: 31,
                },
                StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                    inode: first_file,
                    data_units: LARGE_FILE_DATA_UNITS_OVERWRITING_THE_TWO_LEVELS,
                    content_seed: 32,
                },
                StepAfterTheMultiRecordPrefix::WriteOfAnInode {
                    inode: first_file,
                    data_units: LARGE_FILE_DATA_UNITS_FILLING_ONE_LOWER_LEAF,
                    content_seed: 33,
                },
            ],
            Self::OtherFileDeletedBeforeTheFirstFile => {
                let mut steps = steps_through_the_write_outside_the_upper_leaf_of_the_first_file();
                steps.extend(steps_deleting(
                    INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
                    DATA_UNITS_WRITTEN_TO_THE_INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
                ));
                steps.extend(steps_deleting(
                    first_file,
                    DATA_UNITS_OF_THE_FIRST_FILE_AFTER_THE_PREFIX,
                ));
                steps
            }
            Self::FirstFileDeletedBeforeTheOtherFile => {
                let mut steps = steps_through_the_write_outside_the_upper_leaf_of_the_first_file();
                steps.extend(steps_deleting(
                    first_file,
                    DATA_UNITS_OF_THE_FIRST_FILE_AFTER_THE_PREFIX,
                ));
                steps.extend(steps_deleting(
                    INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
                    DATA_UNITS_WRITTEN_TO_THE_INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
                ));
                steps
            }
        }
    }

    /// 观察者要读回的号：第一个文件，与这条流写过、删过的号（建出来没写过的那一百多个不读）。
    #[must_use]
    pub fn inodes_read_back(self) -> Vec<u64> {
        let mut inodes = vec![FIRST_INODE_NUMBER];
        for step in self.steps() {
            let named = match step {
                StepAfterTheMultiRecordPrefix::WriteOfAnInode { inode, .. }
                | StepAfterTheMultiRecordPrefix::UnlinkIntent { inode }
                | StepAfterTheMultiRecordPrefix::UnlinkBatch { inode } => Some(inode),
                StepAfterTheMultiRecordPrefix::CloseAndMountWritable
                | StepAfterTheMultiRecordPrefix::CreateInodes { .. } => None,
            };
            if let Some(inode) = named {
                if !inodes.contains(&inode) {
                    inodes.push(inode);
                }
            }
        }
        inodes.sort_unstable();
        inodes
    }
}

/// 多 inode 那条流与多文件删除那两支共用的前缀（C 之后）：建 149 个空 inode，写 inode 150 两个单元（上段长到层级 1）。
/// 设点文件、入口名、写出的字节都相同，几条流在这几个节点上共享判定。
#[must_use]
pub fn steps_through_the_write_outside_the_upper_leaf_of_the_first_file(
) -> Vec<StepAfterTheMultiRecordPrefix> {
    vec![
        StepAfterTheMultiRecordPrefix::CreateInodes {
            count: INODES_CREATED_AFTER_THE_PREFIX,
        },
        StepAfterTheMultiRecordPrefix::WriteOfAnInode {
            inode: INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
            data_units: DATA_UNITS_WRITTEN_TO_THE_INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE,
            content_seed: 150,
        },
    ]
}

/// 删 `inode`（这时它有 `data_units` 个数据单元）的各步：意图，再按批大小 1（只供测试的 `deletion_history_batch_size`）
/// 留 `data_units − 1`、…、留 1，最后一批——批数等于单元数。
#[must_use]
pub fn steps_deleting(inode: u64, data_units: usize) -> Vec<StepAfterTheMultiRecordPrefix> {
    std::iter::once(StepAfterTheMultiRecordPrefix::UnlinkIntent { inode })
        .chain(
            (0..data_units)
                .map(|_batch_index| StepAfterTheMultiRecordPrefix::UnlinkBatch { inode }),
        )
        .collect()
}

/// 接在 C 之后的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepAfterTheMultiRecordPrefix {
    /// 关闭镜像（进程退出）再可写挂载：取号、写行、暖机，新实例接着发。
    CloseAndMountWritable,
    /// 建 `count` 个 size 0 的 inode（号从水位起连着发）。
    CreateInodes { count: u64 },
    /// 按 inode 号整份写 `inode` 成 `data_units` 个数据单元的内容（`content_needing(data_units, content_seed)`）：一个单元走按号覆盖写，
    /// 多个走按号顺序写。
    WriteOfAnInode {
        inode: u64,
        data_units: usize,
        content_seed: usize,
    },
    /// 删 `inode` 的意图那次发布。
    UnlinkIntent { inode: u64 },
    /// 删 `inode` 的下一批（批大小 1）；它的最后一个单元那一批就是最后一批。
    UnlinkBatch { inode: u64 },
}

impl StepAfterTheMultiRecordPrefix {
    /// 录制器设点用的入口名（节点名 `<入口>.<周期>.<步>` 的第一段）。
    #[must_use]
    pub fn entry_name(self) -> &'static str {
        match self {
            Self::CloseAndMountWritable => "mount_writable",
            Self::CreateInodes { .. } => "publish_new_inodes",
            Self::WriteOfAnInode { data_units: 1, .. } => "publish_overwrite_of_an_inode",
            Self::WriteOfAnInode { .. } => "publish_sequential_write_of_an_inode",
            Self::UnlinkIntent { .. } => "publish_unlink_intent_of_an_inode",
            Self::UnlinkBatch { .. } => "publish_unlink_step_of_an_inode",
        }
    }

    /// 恢复落到这一步发出的那几版时观察者加一的计数名。
    #[must_use]
    pub fn landed_counter_name(self) -> &'static str {
        match self {
            Self::CloseAndMountWritable => "landed_on_mount_writable",
            Self::CreateInodes { .. } => "landed_on_new_inodes",
            Self::WriteOfAnInode { data_units: 1, .. } => "landed_on_overwrite_of_an_inode",
            Self::WriteOfAnInode { .. } => "landed_on_sequential_write_of_an_inode",
            Self::UnlinkIntent { .. } => "landed_on_unlink_intent_of_an_inode",
            Self::UnlinkBatch { .. } => "landed_on_unlink_step_of_an_inode",
        }
    }

    /// 这一步写进 `inode` 的内容（写数据的那一种才有）。
    #[must_use]
    pub fn content_written(self) -> Option<(u64, Vec<u8>)> {
        match self {
            Self::WriteOfAnInode {
                inode,
                data_units,
                content_seed,
            } => Some((inode, content_needing(data_units, content_seed))),
            Self::CloseAndMountWritable
            | Self::CreateInodes { .. }
            | Self::UnlinkIntent { .. }
            | Self::UnlinkBatch { .. } => None,
        }
    }

    /// 接着池的现行那一版发这一步（实例是 `instance`；关闭再可写挂载那一步把它换成新实例），池接到它那一版上，交回这一步发出的那几版的根
    /// （可写挂载是写行与它交回的暖机那几次，别的是那一次发布）与这一步之后的现行那一版。
    ///
    /// # Panics
    /// 这一步被拒（发布路径的任何拒绝、可写挂载做不成），或可写挂载交回的现行那一版没有树、续做了什么。
    pub fn publish(self, pool: &mut BuiltPool, instance: &mut InstanceGeneration) -> RecordedStep {
        let instance_of_the_step = *instance;
        let write_time_seconds = WRITE_TIME_SECONDS_OF_THE_STEPS_AFTER_THE_PREFIX;
        let output = match self {
            Self::CloseAndMountWritable => {
                let mut devices = pool.reopen_recorded();
                let mounted = mount_writable(&parameters(), &mut devices)
                    .unwrap_or_else(|error| panic!("关闭之后再可写挂载：{error:?}"));
                pool.devices = Some(devices);
                assert!(
                    mounted.output.deletion_queue_continuation.is_empty(),
                    "关闭之前队列是空的，可写挂载不续做"
                );
                let published_roots = std::iter::once(&mounted.output.row_publish)
                    .chain(&mounted.output.warm_up_publishes)
                    .map(|version| root_identity_of(version.root()))
                    .collect();
                *instance = mounted.output.instance;
                pool.allocator = mounted.allocator;
                pool.output = mounted
                    .current
                    .into_version_of_trees()
                    .expect("关闭之前那一版带文件，可写挂载之后现行那一版有树");
                return RecordedStep {
                    step: self,
                    published_roots,
                    output: pool.output.clone(),
                    operations_after_the_step: pool.stream.operations().len(),
                };
            }
            Self::CreateInodes { count } => {
                publish_in_process(pool, |writer, allocator, previous| {
                    publish_new_inodes(
                        writer,
                        allocator,
                        previous,
                        count,
                        write_time_seconds,
                        instance_of_the_step,
                    )
                })
            }
            Self::WriteOfAnInode {
                inode, data_units, ..
            } => {
                let (_inode, content) = self.content_written().expect("写数据的那一步带内容");
                publish_in_process(pool, |writer, allocator, previous| {
                    if data_units == 1 {
                        publish_overwrite_of_an_inode(
                            writer,
                            allocator,
                            previous,
                            InodeNumber(inode),
                            &content,
                            write_time_seconds,
                            instance_of_the_step,
                        )
                    } else {
                        publish_sequential_write_of_an_inode(
                            writer,
                            allocator,
                            previous,
                            InodeNumber(inode),
                            &content,
                            write_time_seconds,
                            instance_of_the_step,
                        )
                    }
                })
            }
            Self::UnlinkIntent { inode } => {
                publish_in_process(pool, |writer, allocator, previous| {
                    publish_unlink_intent(
                        writer,
                        allocator,
                        previous,
                        InodeNumber(inode),
                        write_time_seconds,
                        instance_of_the_step,
                    )
                })
            }
            Self::UnlinkBatch { inode } => {
                publish_in_process(pool, |writer, allocator, previous| {
                    publish_unlink_step(
                        writer,
                        allocator,
                        previous,
                        InodeNumber(inode),
                        deletion_history_batch_size(),
                        instance_of_the_step,
                    )
                })
            }
        };
        RecordedStep {
            step: self,
            published_roots: vec![root_identity_of(&output.root)],
            output,
            operations_after_the_step: pool.stream.operations().len(),
        }
    }
}

fn root_identity_of(root: &singlefs_core::root_record::RootRecord) -> RootIdentity {
    (root.instance, root.checkpoint_txg)
}

/// 同一个进程里接着池的现行那一版发一次，池接到它那一版上。
///
/// # Panics
/// 这一次被拒。
fn publish_in_process(
    pool: &mut BuiltPool,
    publish: impl FnOnce(
        &mut PoolWriter<'_, Recorded>,
        &mut PoolAllocator,
        &TransactionOutput,
    ) -> Result<TransactionOutput, PublishError>,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish(&mut writer, &mut pool.allocator, &previous).unwrap_or_else(|refusal| {
        panic!(
            "接在 txg {} 之后那一次被拒：{refusal:?}",
            previous.root.checkpoint_txg.0
        )
    });
    pool.output = output.clone();
    output
}

/// 录下的一步：哪一步、它发出的那几版的根、它之后的现行那一版、它发完时录制流有几步（整条流施加到这里就是这一步之后的盘面）。
#[derive(Clone, Debug)]
pub struct RecordedStep {
    pub step: StepAfterTheMultiRecordPrefix,
    pub published_roots: Vec<RootIdentity>,
    pub output: TransactionOutput,
    pub operations_after_the_step: usize,
}

/// 多记录流的 A、B、C 三版（`build_pool` 里那一版与两次顺序写）：输出与第一个文件那一刻的内容。
#[derive(Clone, Debug)]
pub struct MultiRecordPrefixOfTheStream {
    pub outputs: Vec<TransactionOutput>,
    pub contents: Vec<Vec<u8>>,
}

impl MultiRecordPrefixOfTheStream {
    /// A、B、C 的输出与内容（C 之后第一个文件三个数据单元）。
    ///
    /// # Panics
    /// C 之后第一个文件不是三个数据单元。
    #[must_use]
    pub fn of_the_three_publishes(
        new_pool_file_creation: TransactionOutput,
        write_of_two_data_units: TransactionOutput,
        content_of_two_data_units: Vec<u8>,
        write_of_three_data_units: TransactionOutput,
        content_of_three_data_units: Vec<u8>,
    ) -> Self {
        assert_eq!(
            write_of_three_data_units.data_pointers.len(),
            DATA_UNITS_OF_THE_FIRST_FILE_AFTER_THE_PREFIX,
            "C 之后第一个文件三个数据单元"
        );
        Self {
            outputs: vec![
                new_pool_file_creation,
                write_of_two_data_units,
                write_of_three_data_units,
            ],
            contents: vec![
                crate::common::file_content(),
                content_of_two_data_units,
                content_of_three_data_units,
            ],
        }
    }
}

/// 录完的一条流：池（录制流在它里面）、C 之前那一段、C 之后每一步。
pub struct RecordedMultipleCopyOnWriteStream {
    pub stream: MultipleCopyOnWriteCrashStream,
    pub pool: BuiltPool,
    pub prefix: MultiRecordPrefixOfTheStream,
    pub steps: Vec<RecordedStep>,
}

/// 不录崩溃点那一路：`build_pool`（取号、暖机两次、A）、B、C（与多记录流同一份写法），再按计划发 C 之后的各步。
#[must_use]
pub fn record_the_stream(
    tag: &str,
    stream: MultipleCopyOnWriteCrashStream,
) -> RecordedMultipleCopyOnWriteStream {
    let mut pool = build_pool(tag);
    let new_pool_file_creation = pool.output.clone();
    let content_of_two_data_units = content_needing(2, 1);
    let write_of_two_data_units = sequential_write(&mut pool, &content_of_two_data_units);
    let content_of_three_data_units =
        content_needing(DATA_UNITS_OF_THE_FIRST_FILE_AFTER_THE_PREFIX, 2);
    let write_of_three_data_units = sequential_write(&mut pool, &content_of_three_data_units);
    let prefix = MultiRecordPrefixOfTheStream::of_the_three_publishes(
        new_pool_file_creation,
        write_of_two_data_units,
        content_of_two_data_units,
        write_of_three_data_units,
        content_of_three_data_units,
    );
    record_the_steps_after_the_prefix(stream, pool, prefix, |step, pool, instance| {
        step.publish(pool, instance)
    })
}

/// 接着 C 按计划发这条流 C 之后的每一步：每一步交 `publish_a_step`（崩溃放量那一路在这里包一层录制器设点，崩溃点重放那一路直接发）。
pub fn record_the_steps_after_the_prefix(
    stream: MultipleCopyOnWriteCrashStream,
    mut pool: BuiltPool,
    prefix: MultiRecordPrefixOfTheStream,
    mut publish_a_step: impl FnMut(
        StepAfterTheMultiRecordPrefix,
        &mut BuiltPool,
        &mut InstanceGeneration,
    ) -> RecordedStep,
) -> RecordedMultipleCopyOnWriteStream {
    let mut instance = INSTANCE_OF_THE_PREFIX;
    let steps = stream
        .steps()
        .into_iter()
        .map(|step| publish_a_step(step, &mut pool, &mut instance))
        .collect();
    RecordedMultipleCopyOnWriteStream {
        stream,
        pool,
        prefix,
        steps,
    }
}

/// 第一个文件之外的一个号在某一版里读回什么：`Some(内容)` 是打得开、读出这些字节（建出来没写过的读出 0 字节），`None` 是没有这个 inode
/// （还没建、已经删完，或删除中：从意图那次起读者答没有）。
pub type ContentOfAnInode = Option<Vec<u8>>;

/// 流里的一版与可写挂载那个观察者另读的东西：根身份、恢复落到它时加一的计数名、落到它之后可写挂载的续做还要发几次、续做走完之后每个要读的号读回什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheStream {
    pub root: RootIdentity,
    pub landed_counter_name: &'static str,
    pub deletion_publishes_left: usize,
    pub contents_after_the_continuation: BTreeMap<u64, ContentOfAnInode>,
}

/// 落到 A、B、C 那三版时加一的计数名。
pub const LANDED_ON_THE_MULTI_RECORD_PREFIX_COUNTER: &str = "landed_on_the_multi_record_prefix";

/// 落到 A 之前（mkfs 或两次暖机）时加一的计数名。
pub const LANDED_BEFORE_THE_FIRST_FILE_COUNTER: &str = "landed_before_the_first_file";

impl RecordedMultipleCopyOnWriteStream {
    /// 删除那几步之后这一个号的删除还剩几次发布（批与最后一批）：意图与不是最后一批的那一批数到同一个号连着的最后一批为止（含），
    /// 最后一批与别的步是 0。
    fn deletion_publishes_left_after(&self, step_index: usize) -> usize {
        let inode_being_deleted = match self.steps[step_index].step {
            StepAfterTheMultiRecordPrefix::UnlinkIntent { inode }
            | StepAfterTheMultiRecordPrefix::UnlinkBatch { inode } => inode,
            StepAfterTheMultiRecordPrefix::CloseAndMountWritable
            | StepAfterTheMultiRecordPrefix::CreateInodes { .. }
            | StepAfterTheMultiRecordPrefix::WriteOfAnInode { .. } => return 0,
        };
        self.steps[step_index + 1..]
            .iter()
            .take_while(|recorded| {
                recorded.step
                    == StepAfterTheMultiRecordPrefix::UnlinkBatch {
                        inode: inode_being_deleted,
                    }
            })
            .count()
    }

    /// 流里的每一版（A、B、C 与 C 之后每一步发出的那几版），按发布的次序。
    #[must_use]
    pub fn versions_of_the_stream(&self) -> Vec<VersionOfTheStream> {
        let inodes_read_back = self.stream.inodes_read_back();
        let mut contents: BTreeMap<u64, ContentOfAnInode> = inodes_read_back
            .iter()
            .map(|inode| (*inode, None))
            .collect();
        let mut versions = Vec::new();
        for (output, content) in self.prefix.outputs.iter().zip(&self.prefix.contents) {
            contents.insert(FIRST_INODE_NUMBER, Some(content.clone()));
            versions.push(VersionOfTheStream {
                root: root_identity_of(&output.root),
                landed_counter_name: LANDED_ON_THE_MULTI_RECORD_PREFIX_COUNTER,
                deletion_publishes_left: 0,
                contents_after_the_continuation: contents.clone(),
            });
        }
        let mut next_inode_number = FIRST_INODE_NUMBER + 1;
        for (step_index, recorded) in self.steps.iter().enumerate() {
            let deletion_publishes_left = self.deletion_publishes_left_after(step_index);
            let mut inode_whose_deletion_the_continuation_finishes = None;
            match recorded.step {
                StepAfterTheMultiRecordPrefix::CloseAndMountWritable => {}
                StepAfterTheMultiRecordPrefix::CreateInodes { count } => {
                    for inode in next_inode_number..next_inode_number + count {
                        if contents.contains_key(&inode) {
                            contents.insert(inode, Some(Vec::new()));
                        }
                    }
                    next_inode_number += count;
                }
                StepAfterTheMultiRecordPrefix::WriteOfAnInode { .. } => {
                    let (inode, content) = recorded
                        .step
                        .content_written()
                        .expect("写数据的那一步带内容");
                    contents.insert(inode, Some(content));
                }
                StepAfterTheMultiRecordPrefix::UnlinkIntent { inode }
                | StepAfterTheMultiRecordPrefix::UnlinkBatch { inode } => {
                    if deletion_publishes_left == 0 {
                        contents.insert(inode, None);
                    } else {
                        inode_whose_deletion_the_continuation_finishes = Some(inode);
                    }
                }
            }
            let mut contents_after_the_continuation = contents.clone();
            if let Some(inode) = inode_whose_deletion_the_continuation_finishes {
                contents_after_the_continuation.insert(inode, None);
            }
            for root in &recorded.published_roots {
                versions.push(VersionOfTheStream {
                    root: *root,
                    landed_counter_name: recorded.step.landed_counter_name(),
                    deletion_publishes_left,
                    contents_after_the_continuation: contents_after_the_continuation.clone(),
                });
            }
        }
        versions
    }

    /// oracle 认的版本表：第一个文件那一刻读得出的那几版（内容），与从删第一个文件的意图那次起读者答没有文件的那几版。
    #[must_use]
    pub fn version_tables(&self) -> (Vec<PublishedVersion>, Vec<PublishedVersionWithoutAFile>) {
        let mut with_a_file = Vec::new();
        let mut without_a_file = Vec::new();
        for (output, content) in self.prefix.outputs.iter().zip(&self.prefix.contents) {
            with_a_file.push(PublishedVersion {
                instance: output.root.instance,
                checkpoint_txg: output.root.checkpoint_txg,
                content: content.clone(),
            });
        }
        let mut first_file_content = self.prefix.contents.last().cloned();
        for recorded in &self.steps {
            match recorded.step {
                StepAfterTheMultiRecordPrefix::WriteOfAnInode { inode, .. }
                    if inode == FIRST_INODE_NUMBER =>
                {
                    first_file_content =
                        recorded.step.content_written().map(|(_, content)| content);
                }
                StepAfterTheMultiRecordPrefix::UnlinkIntent { inode }
                    if inode == FIRST_INODE_NUMBER =>
                {
                    first_file_content = None;
                }
                StepAfterTheMultiRecordPrefix::CloseAndMountWritable
                | StepAfterTheMultiRecordPrefix::CreateInodes { .. }
                | StepAfterTheMultiRecordPrefix::WriteOfAnInode { .. }
                | StepAfterTheMultiRecordPrefix::UnlinkIntent { .. }
                | StepAfterTheMultiRecordPrefix::UnlinkBatch { .. } => {}
            }
            for (instance, checkpoint_txg) in &recorded.published_roots {
                match &first_file_content {
                    Some(content) => with_a_file.push(PublishedVersion {
                        instance: *instance,
                        checkpoint_txg: *checkpoint_txg,
                        content: content.clone(),
                    }),
                    None => without_a_file.push(PublishedVersionWithoutAFile {
                        instance: *instance,
                        checkpoint_txg: *checkpoint_txg,
                    }),
                }
            }
        }
        (with_a_file, without_a_file)
    }

    /// 每一步之后的形状（录流的形状，判器判完之后核：改坏实现时先红在判器上）。
    #[must_use]
    pub fn shapes_of_the_steps(&self) -> Vec<ShapeAfterAStep> {
        self.steps
            .iter()
            .map(|recorded| ShapeAfterAStep::of_the_version(&recorded.output))
            .collect()
    }
}

/// 一版的形状：extent 上段每个节点的（层级，序号）、有条目的号集合、第一个文件的数据单元数与下段节点数、第一个文件之外有数据单元的号与各自的
/// 数据单元数、删除队列里的号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeAfterAStep {
    pub upper_nodes: Vec<(u8, u64)>,
    pub inodes_with_an_upper_leaf_entry: Vec<u64>,
    pub first_file_data_units: usize,
    pub first_file_lower_nodes: usize,
    pub data_units_of_the_files_other_than_the_first: Vec<(u64, usize)>,
    pub inodes_being_deleted: Vec<u64>,
}

impl ShapeAfterAStep {
    #[must_use]
    pub fn of_the_version(output: &TransactionOutput) -> Self {
        Self {
            upper_nodes: output
                .extent_tree
                .upper_nodes
                .iter()
                .map(|(position, _pointer)| (position.level, position.index))
                .collect(),
            inodes_with_an_upper_leaf_entry: output
                .inodes_with_an_extent_upper_leaf_entry()
                .into_iter()
                .collect(),
            first_file_data_units: output.data_pointers.len(),
            first_file_lower_nodes: output.extent_tree.lower_nodes.len(),
            data_units_of_the_files_other_than_the_first: output
                .files_other_than_the_first
                .iter()
                .map(|file| (file.inode.0, file.data_pointers.len()))
                .collect(),
            inodes_being_deleted: output.deleted_inodes.iter().map(|inode| inode.0).collect(),
        }
    }
}

/// 上段只有第 0 片根兼叶（有条目的号都在 0..143 里，或一个都没有）。
const UPPER_ROOT_LEAF_ONLY: &[(u8, u64)] = &[(0, 0)];
/// 上段长到层级 1、第 0 片与第 1 片叶都在（有条目的号里有第一个文件或 2，又有 150）。
const UPPER_LEVEL_ONE_WITH_BOTH_LEAVES: &[(u8, u64)] = &[(0, 0), (0, 1), (1, 0)];
/// 上段在层级 1、只有第 1 片叶（有条目的号只剩 150：第 0 片叶摘掉，不留空叶）。
const UPPER_LEVEL_ONE_WITHOUT_THE_LEAF_OF_THE_FIRST_FILE: &[(u8, u64)] = &[(0, 1), (1, 0)];

fn shape(
    upper_nodes: &[(u8, u64)],
    inodes_with_an_upper_leaf_entry: &[u64],
    (first_file_data_units, first_file_lower_nodes): (usize, usize),
    data_units_of_the_files_other_than_the_first: &[(u64, usize)],
    inodes_being_deleted: &[u64],
) -> ShapeAfterAStep {
    ShapeAfterAStep {
        upper_nodes: upper_nodes.to_vec(),
        inodes_with_an_upper_leaf_entry: inodes_with_an_upper_leaf_entry.to_vec(),
        first_file_data_units,
        first_file_lower_nodes,
        data_units_of_the_files_other_than_the_first: data_units_of_the_files_other_than_the_first
            .to_vec(),
        inodes_being_deleted: inodes_being_deleted.to_vec(),
    }
}

impl MultipleCopyOnWriteCrashStream {
    /// 每一步之后该有的形状（上段规范形：判决 m3-multi-file-unlink-r1 第三节 M2；下段一片叶罩 144 个单元：D8（核心索引结构） 已定项 14）。
    #[must_use]
    pub fn expected_shapes_of_the_steps(self) -> Vec<ShapeAfterAStep> {
        let first_file = FIRST_INODE_NUMBER;
        let outside_inode = INODE_OUTSIDE_THE_UPPER_LEAF_OF_THE_FIRST_FILE;
        let beside = INODE_BESIDE_THE_FIRST_FILE_ON_THE_FIRST_UPPER_LEAF;
        // 第一个文件三个单元：下段一片根兼叶；一个单元：内联在上段叶条目里、没有下段；两个单元：下段一片根兼叶。
        let three_units = (3, 1);
        let two_units = (2, 1);
        let one_unit = (1, 0);
        let deleted = (0, 0);
        let prefix_through_the_write_outside_the_upper_leaf_of_the_first_file = [
            shape(UPPER_ROOT_LEAF_ONLY, &[first_file], three_units, &[], &[]),
            shape(
                UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                &[first_file, outside_inode],
                three_units,
                &[(outside_inode, 2)],
                &[],
            ),
        ];
        match self {
            Self::OneFileOverwrittenAcrossTwoWritableRemounts => vec![
                shape(UPPER_ROOT_LEAF_ONLY, &[first_file], three_units, &[], &[]),
                shape(UPPER_ROOT_LEAF_ONLY, &[first_file], one_unit, &[], &[]),
                shape(UPPER_ROOT_LEAF_ONLY, &[first_file], one_unit, &[], &[]),
                shape(UPPER_ROOT_LEAF_ONLY, &[first_file], two_units, &[], &[]),
            ],
            Self::SeveralInodesOverwrittenInTurn => {
                let every_inode = [first_file, beside, outside_inode];
                let mut shapes =
                    prefix_through_the_write_outside_the_upper_leaf_of_the_first_file.to_vec();
                shapes.extend([
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &every_inode,
                        three_units,
                        &[(beside, 2), (outside_inode, 2)],
                        &[],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &every_inode,
                        one_unit,
                        &[(beside, 2), (outside_inode, 2)],
                        &[],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &every_inode,
                        one_unit,
                        &[(beside, 2), (outside_inode, 1)],
                        &[],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &every_inode,
                        one_unit,
                        &[(beside, 1), (outside_inode, 1)],
                        &[],
                    ),
                ]);
                shapes
            }
            // 145、146 个单元：下段两片叶上面一个第 1 层的根；144 个：一片根兼叶。
            Self::LargeFileOverwrittenAcrossLowerLeaves => vec![
                shape(
                    UPPER_ROOT_LEAF_ONLY,
                    &[first_file],
                    (LARGE_FILE_DATA_UNITS_GROWING_THE_LOWER_SEGMENT, 3),
                    &[],
                    &[],
                ),
                shape(
                    UPPER_ROOT_LEAF_ONLY,
                    &[first_file],
                    (LARGE_FILE_DATA_UNITS_OVERWRITING_THE_TWO_LEVELS, 3),
                    &[],
                    &[],
                ),
                shape(
                    UPPER_ROOT_LEAF_ONLY,
                    &[first_file],
                    (LARGE_FILE_DATA_UNITS_FILLING_ONE_LOWER_LEAF, 1),
                    &[],
                    &[],
                ),
            ],
            // 删 150：意图、留 1 的一批照旧在层级 1；最后一批去掉它的条目、上段降回层级 0。再删第一个文件：每一批截它的下段，最后一批之后
            // 有条目的号集合空了，上段是层级 0 的零条目根兼叶。
            Self::OtherFileDeletedBeforeTheFirstFile => {
                let both = [first_file, outside_inode];
                let mut shapes =
                    prefix_through_the_write_outside_the_upper_leaf_of_the_first_file.to_vec();
                shapes.extend([
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &both,
                        three_units,
                        &[(outside_inode, 2)],
                        &[outside_inode],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &both,
                        three_units,
                        &[(outside_inode, 1)],
                        &[outside_inode],
                    ),
                    shape(UPPER_ROOT_LEAF_ONLY, &[first_file], three_units, &[], &[]),
                    shape(
                        UPPER_ROOT_LEAF_ONLY,
                        &[first_file],
                        three_units,
                        &[],
                        &[first_file],
                    ),
                    shape(
                        UPPER_ROOT_LEAF_ONLY,
                        &[first_file],
                        two_units,
                        &[],
                        &[first_file],
                    ),
                    shape(
                        UPPER_ROOT_LEAF_ONLY,
                        &[first_file],
                        one_unit,
                        &[],
                        &[first_file],
                    ),
                    shape(UPPER_ROOT_LEAF_ONLY, &[], deleted, &[], &[]),
                ]);
                shapes
            }
            // 先删第一个文件：最后一批之后上段留在层级 1、第 0 片叶摘掉；再删 150：最后一批之后集合空了，降成层级 0 的零条目根兼叶。
            Self::FirstFileDeletedBeforeTheOtherFile => {
                let both = [first_file, outside_inode];
                let mut shapes =
                    prefix_through_the_write_outside_the_upper_leaf_of_the_first_file.to_vec();
                shapes.extend([
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &both,
                        three_units,
                        &[(outside_inode, 2)],
                        &[first_file],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &both,
                        two_units,
                        &[(outside_inode, 2)],
                        &[first_file],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITH_BOTH_LEAVES,
                        &both,
                        one_unit,
                        &[(outside_inode, 2)],
                        &[first_file],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITHOUT_THE_LEAF_OF_THE_FIRST_FILE,
                        &[outside_inode],
                        deleted,
                        &[(outside_inode, 2)],
                        &[],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITHOUT_THE_LEAF_OF_THE_FIRST_FILE,
                        &[outside_inode],
                        deleted,
                        &[(outside_inode, 2)],
                        &[outside_inode],
                    ),
                    shape(
                        UPPER_LEVEL_ONE_WITHOUT_THE_LEAF_OF_THE_FIRST_FILE,
                        &[outside_inode],
                        deleted,
                        &[(outside_inode, 1)],
                        &[outside_inode],
                    ),
                    shape(UPPER_ROOT_LEAF_ONLY, &[], deleted, &[], &[]),
                ]);
                shapes
            }
        }
    }
}

/// 录流的形状与计划的逐步相同（判器判完之后核：改坏实现时先红在判器上）。
///
/// # Panics
/// 有一步对不上。
pub fn assert_the_shape_of_the_steps(recorded: &RecordedMultipleCopyOnWriteStream) {
    assert_eq!(
        recorded.shapes_of_the_steps(),
        recorded.stream.expected_shapes_of_the_steps(),
        "{:?} 每一步之后的形状（上段节点、有条目的号、第一个文件的单元数与下段节点数、别的文件的单元数、删除队列）",
        recorded.stream
    );
}

/// 崩溃点重放那一路要的几样：起点镜像、写表、段、被判的根槽写、版本表与流里的每一版。
pub struct PreparedMultipleCopyOnWriteCrashStream {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    pub versions_of_the_stream: Vec<VersionOfTheStream>,
    pub recorded: RecordedMultipleCopyOnWriteStream,
}

impl PreparedMultipleCopyOnWriteCrashStream {
    /// 每段的写数（用例核段形）。
    #[must_use]
    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segments.iter().map(Vec::len).collect()
    }

    /// 写数小于 `writes_below` 的那几段。
    #[must_use]
    pub fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }
}

/// 录完的流切段：起点镜像是 mkfs 之后，写表从 mkfs 之后起，被判的根槽写是最后一条根槽写。
///
/// # Panics
/// 流里没有根槽写。
#[must_use]
pub fn prepared_from_the_recorded_stream(
    recorded: RecordedMultipleCopyOnWriteStream,
) -> PreparedMultipleCopyOnWriteCrashStream {
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
    let (versions_with_a_file, versions_without_a_file) = recorded.version_tables();
    let versions_of_the_stream = recorded.versions_of_the_stream();
    PreparedMultipleCopyOnWriteCrashStream {
        base,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        versions_of_the_stream,
        recorded,
    }
}

/// 录一条流（不录崩溃点）、切段。
#[must_use]
pub fn prepare_the_crash_stream(
    tag: &str,
    stream: MultipleCopyOnWriteCrashStream,
) -> PreparedMultipleCopyOnWriteCrashStream {
    prepared_from_the_recorded_stream(record_the_stream(tag, stream))
}

/// 这一版落在流里哪一项（按根身份）；不在流里（A 之前）是 `None`。
#[must_use]
pub fn version_of_the_stream_at(
    versions: &[VersionOfTheStream],
    root: RootIdentity,
) -> Option<&VersionOfTheStream> {
    versions.iter().find(|version| version.root == root)
}

// ============================== 可写挂载续做、读回每个文件（观察者的本体，判法那一半在 `mod.rs`） ==============================

/// 可写挂载那个观察者看的状态落到 A 之前时加一的计数名（与落点观察者的 [`LANDED_BEFORE_THE_FIRST_FILE_COUNTER`] 分开：崩溃点重放那一路
/// 几个观察者的计数并在一份里）。
pub const MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER: &str = "mount_landed_before_the_first_file";

/// 续做之后读回的每个号都对上流里那一版（续做走完之后）时加一的计数名。
pub const FILES_READ_BACK_AFTER_THE_CONTINUATION_COUNTER: &str =
    "files_read_back_as_the_version_after_the_continuation";

/// 落到删除中的那几版、同一张崩溃镜像在两份副本上各自可写挂载续做，挂完的盘逐字节相同时加一的计数名（幂等判据 ①b）。
pub const CONTINUATION_REPEATED_BYTE_IDENTICAL_COUNTER: &str =
    "continuation_repeated_on_the_same_crash_image_byte_identical";

/// 续做发了几次的计数名：`deletion_continuation_publishes_<次数>`。
#[must_use]
pub fn deletion_continuation_counter(publishes: usize) -> String {
    format!("deletion_continuation_publishes_{publishes}")
}

/// 一次删除至多还剩几次发布（第一个文件三个单元：意图之后三批）。
pub const DELETION_CONTINUATION_PUBLISHES_AT_MOST: usize =
    DATA_UNITS_OF_THE_FIRST_FILE_AFTER_THE_PREFIX;

/// 在一张崩溃镜像（落成整盘的那一份）的副本上可写挂载一次之后的样子：挂完的盘、挂载发的写与读（录在这条录制流上）、要加一的计数名。
pub struct MountedOnACopyOfTheCrashImage {
    pub disks_after_the_mount: MemoryPool,
    pub stream_of_the_mount: SharedStream,
    pub counters: Vec<String>,
}

/// 在 `crash_image` 的一份副本上可写挂载一次（批大小 1，同录流），挂载发的写与读录在一条新的录制流上；挂载之前的盘不动。
///
/// # Panics
/// 可写挂载做不成，或续做没把队列走到空。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
fn mount_writable_on_a_copy_of(crash_image: &MemoryPool) -> (Mounted, MemoryPool, SharedStream) {
    let mount_parameters = parameters();
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = crash_image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                crash_image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &mount_parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_history_switches(),
    )
    .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "续做把队列走到空：{:?}",
        mounted.output.deletion_queue_continuation_outcome
    );
    let disks_after_the_mount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: crash_image.device_size_in_bytes,
    };
    (mounted, disks_after_the_mount, stream)
}

/// 只读挂载 `image`（扫环只这一次），逐个打开 `inodes` 里的号、各从偏移 0 读整个文件：打不开的是 `None`。
///
/// # Panics
/// 只读挂载做不成，或打开了却读不出整个文件。
fn contents_read_back(image: &MemoryPool, inodes: &[u64]) -> BTreeMap<u64, ContentOfAnInode> {
    let mounted = mount_read_only(image)
        .unwrap_or_else(|failure| panic!("可写挂载之后的盘只读挂载得上：{failure:?}"));
    inodes
        .iter()
        .map(|inode| {
            let content = mounted
                .mounted
                .open_file(image, InodeNumber(*inode))
                .ok()
                .map(|file| {
                    let size = file.inode_record().size;
                    file.read_at(image, FileOffsetInBytes(0), size)
                        .unwrap_or_else(|failure| {
                            panic!("打开了 inode {inode}，读它的 {size} 字节：{failure:?}")
                        })
                        .bytes
                });
            (*inode, content)
        })
        .collect()
}

/// 可写挂载续做观察者在一个崩溃状态上做的事（`mod.rs` 的观察者转调它；不录崩溃点的探针也调它）：在崩溃镜像的副本上可写挂载一次（批大小 1，同录流），
/// 续做把队列走到空，续做发的次数等于恢复落到的那一版之后录流里这一个号的删除还剩几次发布；落到流里一版的，挂完的盘只读挂载、读回的每个号
/// （第一个文件与这条流写过、删过的号）都等于那一版续做走完之后的样子（幂等判据 ③：终态逻辑内容与不崩溃的终态相同）；落到删除中那几版的，
/// 同一张崩溃镜像在另一份副本上再挂一次，两份挂完的盘逐字节相同（幂等判据 ①b）。落到 A 之前的挂完照旧没有树。
/// 交回第一次挂完的盘、挂载那一趟的录制流与要加一的计数名：判段在挂完的盘上跑池级 checker（`mod.rs`）。
///
/// # Panics
/// 可写挂载做不成，或上面任何一样对不上。
#[must_use]
// gpu-exempt: writable-mount
// gpu-exempt: continuation
pub fn mount_continue_and_read_back_every_file(
    versions: &[VersionOfTheStream],
    inodes_read_back: &[u64],
    landed_root: RootIdentity,
    crash_image: &MemoryPool,
) -> MountedOnACopyOfTheCrashImage {
    let landed = version_of_the_stream_at(versions, landed_root);
    let (mounted, disks_after_the_mount, stream_of_the_mount) =
        mount_writable_on_a_copy_of(crash_image);
    let expected_publishes = landed.map_or(0, |version| version.deletion_publishes_left);
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        expected_publishes,
        "恢复落到 {landed_root:?}：录流在这一版之后这一个号的删除还剩几次发布，续做就发几次"
    );
    let mut counters = vec![deletion_continuation_counter(expected_publishes)];
    match landed {
        Some(version) => {
            assert!(
                mounted.current.version_of_trees().is_some(),
                "恢复落到 {landed_root:?}（A 起的一版）：挂完有树"
            );
            let read_back = contents_read_back(&disks_after_the_mount, inodes_read_back);
            for (inode, expected) in &version.contents_after_the_continuation {
                let found = read_back.get(inode).expect("要读的号都读过");
                assert!(
                    found == expected,
                    "恢复落到 {landed_root:?}：续做之后 inode {inode} 读回 {:?} 字节，那一版续做走完之后是 {:?} 字节（None 是没有这个 inode）",
                    found.as_ref().map(Vec::len),
                    expected.as_ref().map(Vec::len)
                );
            }
            counters.push(FILES_READ_BACK_AFTER_THE_CONTINUATION_COUNTER.to_string());
            if expected_publishes > 0 {
                let (_mounted_again, disks_after_the_repeated_mount, _stream_of_the_repeated_mount) =
                    mount_writable_on_a_copy_of(crash_image);
                assert!(
                    disks_after_the_repeated_mount == disks_after_the_mount,
                    "恢复落到 {landed_root:?}：同一张崩溃镜像在两份副本上各自续做，挂完的盘逐字节相同"
                );
                counters.push(CONTINUATION_REPEATED_BYTE_IDENTICAL_COUNTER.to_string());
            }
        }
        None => {
            assert!(
                mounted.current.version_of_trees().is_none(),
                "恢复落到 {landed_root:?}（A 之前）：挂完照旧没有树"
            );
            counters.push(MOUNT_LANDED_BEFORE_THE_FIRST_FILE_COUNTER.to_string());
        }
    }
    MountedOnACopyOfTheCrashImage {
        disks_after_the_mount,
        stream_of_the_mount,
        counters,
    }
}
