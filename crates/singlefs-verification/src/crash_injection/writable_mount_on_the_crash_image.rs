//! 崩溃镜像上的可写挂载：挂载、挂完再发一次布，挂载被拒时判拒绝合不合法，续做那几次写的界线。

use super::*;

/// 只起一次可写挂载（取号、写行、暖机，不接发布）的结局。
#[derive(Debug)]
pub enum WritableMountAlone {
    Mounted {
        instance: InstanceGeneration,
        pool_after: MemoryPool,
    },
    /// 被拒：成员、它说的理由、那个错误本身（拒得合不合法按它判）与拒之后的池（盘变没变按它比）。
    Refused {
        member: String,
        reason: ObservedRefusalReason,
        refusal: Box<MountError>,
        pool_after: MemoryPool,
    },
}

/// 在 `pool` 的副本上只起一次可写挂载（建池那一份参数、给定的空间准入开关），不接发布；交回挂载之后的池，或拒的理由、错误与拒之后的池。
/// 挂载末尾的实例表清扫取产品路径的配置（`InstanceTableSweepConfiguration::PRODUCT`）。
#[must_use]
pub fn writable_mount_alone_on(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
) -> WritableMountAlone {
    writable_mount_alone_with_the_instance_table_sweep_configuration_on(
        pool,
        parameters,
        space_admission,
        InstanceTableSweepConfiguration::PRODUCT,
    )
}

/// 同 [`writable_mount_alone_on`]，挂载末尾那一判实例表清扫的开关与阈值由调用方给（`mount::mount_writable_with_instance_table_sweep_configuration`；
/// 影子账开着，与产品路径同一臂）。
#[must_use]
pub fn writable_mount_alone_with_the_instance_table_sweep_configuration_on(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    instance_table_sweep: InstanceTableSweepConfiguration,
) -> WritableMountAlone {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                pool.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let mounted_instance = match mount_writable_with_instance_table_sweep_configuration(
        parameters,
        &mut devices,
        space_admission,
        ShadowLedger::On,
        instance_table_sweep,
    ) {
        Err(error) => Err(error),
        Ok(mounted) => Ok(mounted.output.instance),
    };
    let pool_after = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: pool.device_size_in_bytes,
    };
    match mounted_instance {
        Err(error) => WritableMountAlone::Refused {
            member: format!("{error:?}"),
            reason: refusal_reason_of_mount_error(&error),
            refusal: Box::new(error),
            pool_after,
        },
        Ok(instance) => WritableMountAlone::Mounted {
            instance,
            pool_after,
        },
    }
}

/// 没崩过的那一次可写挂载之后的池：挂载那一段的写全落在第一次崩溃之后的镜像上（不接之后那一次发布）。I-8.4 的「做一次」与
/// 挂载透明的 v2a 都在它上面只读恢复。挂载被拒时没有（None）。
pub(super) fn pool_after_the_uninterrupted_mount(
    crash_image_materialized: &MemoryPool,
    mount_run: &WritableMountOnTheCrashImage,
    mount_writes: &[RetainedWrite],
) -> Option<MemoryPool> {
    match mount_run.outcome {
        WritableMountOutcome::MountRefused { .. } => None,
        WritableMountOutcome::PublishRefused { .. } | WritableMountOutcome::Published => {
            let mut pool_after_the_mount = crash_image_materialized.clone();
            pool_after_the_mount.apply_writes(mount_writes);
            Some(pool_after_the_mount)
        }
    }
}

/// 崩溃后镜像上那次可写挂载与之后那一次发布的结局。
#[derive(Debug)]
pub enum WritableMountOutcome {
    /// 挂载被拒：成员、它说的理由与那个错误本身（拒的理由在不在 D23 已定项 14 那张单子上按它判）。
    MountRefused {
        member: String,
        reason: ObservedRefusalReason,
        refusal: Box<MountError>,
    },
    /// 挂载做成、之后那一次发布被拒。
    PublishRefused {
        member: String,
        reason: ObservedRefusalReason,
    },
    /// 挂载与那一次发布都做成了。
    Published,
}

/// 在一份崩溃后镜像上跑一次可写挂载加一次发布：交回结局、挂载那一段录制流（取号、写行、暖机与挂载推的抬 F；被拒时是拒之前写出的）、
/// 之后那一次发布的录制流、挂载与发布之后的池。
pub struct WritableMountOnTheCrashImage {
    pub outcome: WritableMountOutcome,
    pub mount_operations: Vec<RetainedOperation>,
    /// 挂载做成之后那一次发布写出的那一段录制流（挂载被拒时是空的）：`pool_after` 上有它，第二截交给记录核对器的写表也要有它。
    pub publish_operations: Vec<RetainedOperation>,
    pub pool_after: MemoryPool,
    /// 挂载交回值里推没推抬 F、最后抬到的 F（挂载透明要它）；挂载被拒时 None。
    pub floor_raised_by_the_mount: Option<FloorRaisedByTheMount>,
    /// 挂载续做之前这次挂载发了几次（写行、暖机、推的抬 F 的每一次空发布）：挂载那一段录制流里第几个根槽写之后起是续做；
    /// 挂载交回 `MountError::Publish` 或 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 时按错里交回的已落盘发布写账划
    /// （[`publishes_before_the_continuation_of_a_mount_that_failed`]；划不出续做、别的错都是 None：写行之后都算暖机）。
    pub publishes_before_the_deletion_queue_continuation: Option<usize>,
    /// 挂载末尾清扫写了东西（抹了头或发了清扫发布）时，清扫之前这次挂载发了几次（写行、暖机、推的抬 F、续做每一步与续做里推的抬 F）：
    /// 挂载那一段录制流里第几个根槽写之后起是清扫抹头；没清扫、或清扫一个字节都没写、或挂载被拒时 None。
    pub publishes_before_the_instance_table_sweep: Option<usize>,
    /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分；
    /// 挂载准入推的与续做里推的都认，界按它们在写表里的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
    pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
    /// 挂载续做每一步那一版的两棵用户可见树的根指针（挂载透明那一样按它比「续做算挂载的一部分」）；挂载被拒时 None。
    pub user_visible_tree_root_pointers_of_the_continuation:
        Option<UserVisibleTreeRootPointersOfTheContinuation>,
    /// 挂载末尾的清扫发布删了行时：清扫发布那一版是哪条根、删了哪几行、抹了几个头（挂载透明的实例表行那一样按它放回被删的行再比，
    /// C704（挂载透明不认清扫发布））；没清扫、清扫没删行、或挂载被拒时 None。
    pub rows_deleted_by_the_instance_table_sweep: Option<RowsDeletedByTheInstanceTableSweepPublish>,
}

/// 崩溃后镜像上那次可写挂载末尾的清扫发布删了行（`InstanceTableSweepOutcome::RowsDeleted`）：清扫发布那一版的根身份、删掉的行、
/// 删行之前抹了几个头。清扫是挂载交回之前最后一件事（`mount::mount_writable` 文档「可写挂载末尾」），清扫发布那一版就是挂载交回的现行版本。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowsDeletedByTheInstanceTableSweepPublish {
    pub sweep_publish: (InstanceGeneration, CheckpointTxg),
    pub rows_deleted: Vec<InstanceRow>,
    pub heads_erased: usize,
}

impl RowsDeletedByTheInstanceTableSweepPublish {
    /// 挂载交回值里清扫删了行时交回删了什么；没删行（没扫、扫了没删、清扫发布没发成）时 None。
    #[must_use]
    pub fn of_the_mount(mounted: &Mounted) -> Option<Self> {
        match &mounted.output.instance_table_sweep {
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::RowsDeleted {
                    rows_deleted,
                    heads_erased,
                },
            ) => Some(Self {
                sweep_publish: (
                    mounted.current.root().instance,
                    mounted.current.root().checkpoint_txg,
                ),
                rows_deleted: rows_deleted.clone(),
                heads_erased: heads_erased.len(),
            }),
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
            | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
            | InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::NoRowDeleted(_)
                | InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
                | InstanceTableSweepOutcome::SweepPublishFailed { .. }
                | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. },
            ) => None,
        }
    }
}

/// 崩溃注入里那次可写挂载（与每个二次崩溃镜像上的再挂）末尾的实例表清扫：有一行就扫（开关关着、阈值
/// [`InstanceTableRowCountThreshold::ANY_ROW`]、清扫发布照挂载的分配器取落点）。产品路径的阈值占位 369 行（C690（清扫触发阈值的默认值没定）），
/// 短历史到不了；要摆到清扫那两段（[`WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep`]、[`WritableMountPhase::InstanceTableSweepPublish`]）
/// 的用例用这一份（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支都能被测试强制进入）。
pub const INSTANCE_TABLE_SWEPT_ON_ANY_ROW: InstanceTableSweepConfiguration =
    InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    };

/// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &CrashPoint) -> Vec<u8> {
    let length = 97 + crash_point.segment_index % 64;
    let fill = seed.0.to_le_bytes()[0];
    (0..length)
        .map(|index| fill ^ u8::try_from(index % 251).expect("小于 251"))
        .collect()
}

/// 在 `crash_image_materialized` 的副本上起可写挂载（建池那一份参数、那一段历史的空间准入开关），做成了就接着发一次布：
/// 挂载交回的现行版本带文件、第一个文件那条记录还活着时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），
/// 有树而文件集合为空（删完那一版）时、或带文件而第一个文件已删完或正在删（unlink 之后再建过 inode）时建一个 size 0 的 inode
/// （D8（核心索引结构） 已定项 16 第 4、5 条：写入口拒不在树里与正在删的 inode）。挂载末尾的实例表清扫取产品路径的配置
/// （`InstanceTableSweepConfiguration::PRODUCT`）。
#[must_use]
pub fn mount_and_publish_once_on_the_crash_image(
    crash_image_materialized: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    execution: HistoryExecution,
    seed: HistorySeed,
    crash_point: &CrashPoint,
) -> WritableMountOnTheCrashImage {
    mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
        crash_image_materialized,
        parameters,
        execution,
        seed,
        crash_point,
        InstanceTableSweepConfiguration::PRODUCT,
    )
}

/// 同 [`mount_and_publish_once_on_the_crash_image`]，挂载末尾那一判实例表清扫的开关与阈值由调用方给
/// （`mount::mount_writable_with_instance_table_sweep_configuration`，影子账开着、与产品路径同一臂）。
/// 可写挂载与之后那一次发布真跑实现，只能在 CPU 上（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount 那一行）；
/// 挂完的盘上的池级 checker 与记录核对器不在这里，经给定状态表交判器判（第二截，[`GivenStatesAfterTheFirstCrash::judge_the_pool_after_the_writable_mount`]）。
// gpu-exempt: writable-mount
#[must_use]
pub fn mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
    crash_image_materialized: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    execution: HistoryExecution,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    instance_table_sweep: InstanceTableSweepConfiguration,
) -> WritableMountOnTheCrashImage {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        crash_image_materialized
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(
                    crash_image_materialized.device_size_in_bytes,
                    PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
                );
                device.image = sectors.clone();
                (
                    *identity,
                    RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
                )
            })
            .collect();
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        parameters,
        &mut devices,
        execution.space_admission,
        ShadowLedger::On,
        instance_table_sweep,
    );
    let mount_operation_count = stream.operation_count();
    let rows_deleted_by_the_instance_table_sweep = mounted
        .as_ref()
        .ok()
        .and_then(RowsDeletedByTheInstanceTableSweepPublish::of_the_mount);
    let floor_raised_by_the_mount = mounted
        .as_ref()
        .ok()
        .map(|mounted| FloorRaisedByTheMount::of_the_mount_output(&mounted.output));
    let publishes_before_the_deletion_queue_continuation = match &mounted {
        Ok(mounted) => Some(
            1 + mounted.output.warm_up_publishes.len()
                + mounted
                    .output
                    .space_admission
                    .floor_raises()
                    .iter()
                    .map(|raised| raised.publishes.len())
                    .sum::<usize>(),
        ),
        Err(error) => publishes_before_the_continuation_of_a_mount_that_failed(error),
    };
    let publishes_before_the_instance_table_sweep = mounted.as_ref().ok().and_then(|mounted| {
        publishes_before_the_instance_table_sweep_of_the_mount_output(&mounted.output)
    });
    let user_visible_tree_root_pointers_of_the_continuation =
        mounted.as_ref().ok().map(|mounted| {
            UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output)
        });
    // 录制流此刻只有挂载那一段：按建池那一份参数的几何切出写表，与调用方切二次崩溃的写表同一张。抬 F 的界按位置取（挂载准入推的与续做里推的
    // 那几串都认，`FloorRaisesInTheMountWrites::of_the_mount_output`），取不出时不 panic、交 NotLocatedInTheMountWrites，
    // 这一次挂载的二次崩溃整段 F 不判、按理由计进 `mount_transparency_fields_not_judged`。
    let floor_raises_in_the_mount_writes = mounted.as_ref().ok().map(|mounted| {
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        };
        let (mount_writes, _mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)
    });
    let outcome = match mounted {
        Err(error) => WritableMountOutcome::MountRefused {
            member: format!("{error:?}"),
            reason: refusal_reason_of_mount_error(&error),
            refusal: Box::new(error),
        },
        Ok(mounted) => {
            let content = content_of_the_publish_after_the_crash(seed, crash_point);
            let file = FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            };
            let mut allocator = mounted.allocator;
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            let published = match &mounted.current {
                PoolVersion::WithFile(current) if first_file_is_a_live_record(current) => {
                    publish_overwrite(
                        &mut writer,
                        &mut allocator,
                        current,
                        file,
                        mounted.output.instance,
                    )
                    .map(|_| ())
                }
                // 带文件的一版而第一个文件已删完（unlink 之后再建过 inode：树里只剩别的 inode）或正在删（nlink = 0）：覆盖写只写第一个文件，
                // 在这一版上被拒成写一个不在树里、或正在删的 inode（D8（核心索引结构） 已定项 16 第 4 条）；与删完那一版一样建一个 size 0 的 inode。
                PoolVersion::WithFile(current) => publish_new_inodes(
                    &mut writer,
                    &mut allocator,
                    current,
                    1,
                    FIXED_WRITE_TIME_SECONDS,
                    mounted.output.instance,
                )
                .map(|_| ()),
                // 删完那一版（D8（核心索引结构） 已定项 16 第 5 条：之后只许建 inode、写行与空发布）：建一个 size 0 的 inode
                // （随机历史的「建 inode」经会话建的也是一个）；覆盖写在这一版上被拒成写一个不在树里的 inode。
                PoolVersion::WithTreesAndNoFile(current) => publish_new_inodes(
                    &mut writer,
                    &mut allocator,
                    current,
                    1,
                    FIXED_WRITE_TIME_SECONDS,
                    mounted.output.instance,
                )
                .map(|_| ()),
                PoolVersion::WithoutFile(current) => publish_first_file(
                    &mut writer,
                    &mut allocator,
                    &current.root,
                    file,
                    mounted.output.instance,
                    &current.record_bytes,
                )
                .map(|_| ()),
            };
            match published {
                Ok(()) => WritableMountOutcome::Published,
                Err(error) => WritableMountOutcome::PublishRefused {
                    member: format!("{error:?}"),
                    reason: refusal_reason_of_publish_error(&error),
                },
            }
        }
    };
    let mut mount_operations = stream.retained_operations();
    let publish_operations = mount_operations.split_off(mount_operation_count);
    let pool_after = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.image))
            .collect(),
        device_size_in_bytes: crash_image_materialized.device_size_in_bytes,
    };
    WritableMountOnTheCrashImage {
        outcome,
        mount_operations,
        publish_operations,
        pool_after,
        floor_raised_by_the_mount,
        publishes_before_the_deletion_queue_continuation,
        publishes_before_the_instance_table_sweep,
        floor_raises_in_the_mount_writes,
        user_visible_tree_root_pointers_of_the_continuation,
        rows_deleted_by_the_instance_table_sweep,
    }
}

/// 带文件的一版里第一个文件那条记录还活着（在树里、nlink 不是 0）：覆盖写只写第一个文件，写入口拒删完与正在删的
/// （`transaction::live_record_of_the_first_file`，D8（核心索引结构） 已定项 16 第 4 条）。unlink 之后再建过 inode 的一版仍归带文件的一版
/// （`TransactionOutput::is_a_version_with_trees_and_no_file` 为假），它的 `inode_record` 是 None。
fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {
    version
        .inode_record
        .as_ref()
        .is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
}

/// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
/// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（在副本上量的：崩在意图之后重挂，
/// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
/// 退役树只在身份消失的那次最后一批写，挂载里只可能是续做那一步写的，同样认。
/// 五种里 `DataUnit` 与 `DeletedInodesTreeNode` 在那次探针上空转（续做那一步没写它们）、仍留着（V6）：这一判认的是
/// 「写行、暖机与抬 F 的空发布都不写的结构」，不是「今天的续做那一步写了的结构」，两种都属于前者——那几种发布不碰文件数据，也不碰
/// `deleted_inodes`（队列只由删除类发布改），所以认它们不会把写行或暖机认成续做；去掉它们，划段就随续做那一步今天的写法变：
/// 第一版只有一个文件、走空那一步删 key 时 `deleted_inodes` 根指针置零（D8（核心索引结构） 已定项 16 第 5 条）而不写它的节点，
/// 队列里有不止一条 key 时删一条要重写那一片（推的，没量）；删除批只释放数据单元、不写（已定项 16 第 2 条），以后某一步续做写了数据单元，
/// 它也只可能是续做。
fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKind) -> bool {
    WrittenStructureKind::IN_REPORT_ORDER.iter().any(|kind| {
        let only_the_continuation_writes_it = match kind {
            WrittenStructureKind::DataUnit
            | WrittenStructureKind::ExtentTreeNode
            | WrittenStructureKind::InodeTreeLeafContainer
            | WrittenStructureKind::InodeTreeRoot
            | WrittenStructureKind::DeletedInodesTreeNode
            // logged_ops 根只在整理签发、删意图的发布里写（判决 m4-gc-r1 第三节 G1）：挂载里只可能是续做整理那一步写的。
            | WrittenStructureKind::LoggedOperationsTreeNode
            | WrittenStructureKind::RetirementTreeNode
            // deadlist 树只在杀了被活快照引用的块的发布与删快照的批里写（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
            // 第三节 S3、S4）：挂载里只可能是续做删快照那一步写的。
            | WrittenStructureKind::DeadlistTreeNode => true,
            WrittenStructureKind::AllocationRecordTreeNode
            | WrittenStructureKind::AccountingTreeNode
            | WrittenStructureKind::TombstoneTreeNode
            | WrittenStructureKind::LineageTreeNode
            | WrittenStructureKind::CentralMappingTreeNode
            | WrittenStructureKind::TreeTableUnit
            | WrittenStructureKind::InstanceTableUnit
            | WrittenStructureKind::JournalRecord
            | WrittenStructureKind::RootSlot
            | WrittenStructureKind::SystemConfigurationSlot => false,
        };
        only_the_continuation_writes_it && writes.of(*kind).write_calls > 0
    })
}

/// 挂载交回错时续做从第几次发布起（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节；V1，
/// `research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：带着这次挂载已落盘发布写账的两个成员按同一个划法划
/// （[`publishes_before_the_continuation_of_a_mount_that_handed_back_an_error`]），不并进暖机段——`Publish`（写行、暖机或续做里的一步没发成）
/// 连同失败那一次的写账；`FloorRaiseFailedAfterTheMountsPublishes`（准入或续做里推的抬 F 自己报了不是空间不够的错）按它的
/// `writes_of_persisted_publishes`，抬 F 那一串自己的写账在 `cause` 里，都是空发布、不写只有续做才写的结构，不交进来。
/// 别的错在写行之前（取号之前的判、取号自己）就交回，没有续做：None。
fn publishes_before_the_continuation_of_a_mount_that_failed(error: &MountError) -> Option<usize> {
    match error {
        MountError::Publish(failed) => {
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &failed.writes_of_persisted_publishes,
                &failed.writes_of_failed_publishes,
            )
        }
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &failed.writes_of_persisted_publishes,
                &[],
            )
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
    }
}

/// 挂载在写行之后交回 `MountError::Publish` 时续做之前发了几次（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
/// `writes_of_persisted_publishes` 是已落盘的各次发布的写账（按先后，写行那次在最前，`mount::PublishSequenceFailed`），
/// `writes_of_failed_publishes` 是落盘阶段失败的那一次的。写行之后第一个写了只有续做才写的结构
/// （[`writes_a_structure_only_the_continuation_changes`]）的那一次起是续做；已落盘的都没写、失败的那一次写了，续做从失败那一次起
/// （已落盘的那几次之后）；都没写（错出在写行、暖机或准入推的抬 F 里，或续做的第一步落盘之前就被拒、一个写都没发）是 None，写行之后都算暖机。
/// 续做开头先被空间拒、先推的那一串抬 F 归暖机：错里分不出它是准入推的还是续做推的。
fn publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
    writes_of_persisted_publishes: &[WritesByStructureKind],
    writes_of_failed_publishes: &[WritesByStructureKind],
) -> Option<usize> {
    let first_persisted_continuation_step = writes_of_persisted_publishes
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, writes)| writes_a_structure_only_the_continuation_changes(writes))
        .map(|(publish_index, _)| publish_index);
    match first_persisted_continuation_step {
        Some(publish_index) => Some(publish_index),
        None if !writes_of_persisted_publishes.is_empty()
            && writes_of_failed_publishes
                .iter()
                .any(writes_a_structure_only_the_continuation_changes) =>
        {
            Some(writes_of_persisted_publishes.len())
        }
        None => None,
    }
}

impl HistoryThenWritableMountRecords {
    /// `first_write_after_each_power_loss_before_the_crash` 是第一次崩溃之前每次掉电之后第一个写在历史写表里的下标，从小到大
    /// （历史在那次崩溃之前没掉过电的给空表）。
    ///
    /// # Panics
    /// `persisted_in_the_history` 与 `history_writes` 不一样长；有一道掉电接缝越过历史写表。
    #[must_use]
    pub fn new(
        history_writes: &[RetainedWrite],
        persisted_in_the_history: &[bool],
        first_write_after_each_power_loss_before_the_crash: &[usize],
        version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
        mount_writes: &[RetainedWrite],
        writes_of_the_publish_after_the_mount: &[RetainedWrite],
    ) -> Self {
        assert_eq!(
            persisted_in_the_history.len(),
            history_writes.len(),
            "历史那一段的持久集合与历史录制流逐条对应"
        );
        assert!(
            first_write_after_each_power_loss_before_the_crash
                .iter()
                .all(|first_write_after_the_power_loss| {
                    *first_write_after_the_power_loss <= history_writes.len()
                }),
            "掉电接缝落在历史写表里：{first_write_after_each_power_loss_before_the_crash:?}，历史写表 {} 项",
            history_writes.len()
        );
        let mut seams = power_loss_seams(first_write_after_each_power_loss_before_the_crash);
        seams.push(RecordStreamSeam::AfterACrash {
            first_write_after_the_crash: history_writes.len(),
            version_landed_on_after_the_crash: version_landed_on_after_the_first_crash,
        });
        Self {
            writes: history_writes
                .iter()
                .chain(mount_writes)
                .chain(writes_of_the_publish_after_the_mount)
                .cloned()
                .collect(),
            persisted_in_the_history: persisted_in_the_history.to_vec(),
            writes_of_the_mount: mount_writes.len(),
            version_landed_on_after_the_first_crash,
            seams,
        }
    }

    /// 挂载那一段的写（二次崩溃按它切段、摆状态）。
    #[must_use]
    pub fn mount_writes(&self) -> &[RetainedWrite] {
        let first_write_of_the_mount = self.persisted_in_the_history.len();
        &self.writes[first_write_of_the_mount..first_write_of_the_mount + self.writes_of_the_mount]
    }

    fn continuity(&self) -> RecordStreamContinuity<'_> {
        RecordStreamContinuity::BrokenAtSeams(&self.seams)
    }

    /// 历史那一段的持久集合，接上挂载那一段的（`persisted_in_the_mount`，与挂载那一段逐条对应），
    /// 再接上挂载之后那一次发布的（`publish_after_the_mount`）。
    fn persisted_with_the_mount(
        &self,
        persisted_in_the_mount: &[bool],
        publish_after_the_mount: PublishAfterTheMountInTheState,
    ) -> Vec<bool> {
        assert_eq!(
            persisted_in_the_mount.len(),
            self.writes_of_the_mount,
            "挂载那一段的持久集合与挂载那一段的写逐条对应"
        );
        let writes_of_the_publish_after_the_mount =
            self.writes.len() - self.persisted_in_the_history.len() - self.writes_of_the_mount;
        let publish_after_the_mount_persisted = match publish_after_the_mount {
            PublishAfterTheMountInTheState::Landed => true,
            PublishAfterTheMountInTheState::NotWrittenYet => false,
        };
        self.persisted_in_the_history
            .iter()
            .chain(persisted_in_the_mount)
            .copied()
            .chain(std::iter::repeat_n(
                publish_after_the_mount_persisted,
                writes_of_the_publish_after_the_mount,
            ))
            .collect()
    }

    /// 第二截：崩溃态镜像是第一次崩溃的那份崩溃镜像（`crash_image`，判择根与前缀），实现恢复后的镜像是挂载与之后那一次发布之后的池
    /// （`pool_after_the_mount`，判在不在，D13（验证路线） 已定项 7「崩溃后镜像是两份」）；挂载那一段与之后那次发布全落了。
    /// 恢复自称的那一版取挂载与之后那次发布写出的最新那条根（写行、暖机、之后那次发布都是这个池上的版本）；一条根都没写出
    /// （取号之前就被拒）时取第一次崩溃之后恢复落到的那一版。
    #[must_use]
    pub fn record_check_after_the_writable_mount(
        &self,
        crash_image: &CrashImage<'_>,
        pool_after_the_mount: &MemoryPool,
    ) -> RecordCheck {
        check_records_against(
            crash_image,
            pool_after_the_mount,
            &self.writes,
            &self.persisted_with_the_mount(
                &vec![true; self.writes_of_the_mount],
                PublishAfterTheMountInTheState::Landed,
            ),
            self.continuity(),
            newest_root_written_after_the_crash(
                &self.writes[self.persisted_in_the_history.len()..],
                self.version_landed_on_after_the_first_crash,
            ),
        )
    }

    /// 第三截：两份崩溃后镜像都是二次崩溃镜像（`second_crash_image`，只读恢复不改盘）；挂载那一段的持久集合按二次崩溃
    /// （`persisted_in_the_mount`，与整条挂载流逐条对应：更早的段整段持久、当前段按子集、更晚的段一个都没持久），
    /// 挂载之后那一次发布一个都没写到。
    #[must_use]
    pub fn record_check_after_a_second_crash(
        &self,
        persisted_in_the_mount: &[bool],
        second_crash_image: &CrashImage<'_>,
        effective_root_after_the_second_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    ) -> RecordCheck {
        check_records_against(
            second_crash_image,
            second_crash_image,
            &self.writes,
            &self.persisted_with_the_mount(
                persisted_in_the_mount,
                PublishAfterTheMountInTheState::NotWrittenYet,
            ),
            self.continuity(),
            effective_root_after_the_second_crash,
        )
    }
}

/// 挂载之后那一次发布的写在一个状态里落没落：第二截的池上整次落了；第二截那张表里第一次崩溃那份镜像上、第三截崩在挂载途中时，它一个写都还没发。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PublishAfterTheMountInTheState {
    Landed,
    NotWrittenYet,
}

/// 第二截、第三截被拒的那次挂载都起在崩溃镜像（第一次崩溃的、二次崩溃的）上，盘是不带故障层的内存盘：读到的就是镜像上的字节，
/// 瞬时读错不会发生——崩溃态注入过读故障也一样（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
/// C3 / J2-a 那一行，改法 G3：判官恒交「否」）。
pub(super) const READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE: ReadFaultsUnderTheRefusedMount =
    ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults;

/// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
/// 做没做成都在之后的池上跑池级 checker；`record_check` 是第二截记录核对器的结论（第一条判据读第一次崩溃那份镜像、第二条读之后的池，
/// [`ConclusionsAfterTheWritableMount::record_check`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
/// 那张单子上的，判拒绝本身（`singlefs_verification::mount_transparency::judge_writable_mount_refusal_of_a_crash_state_of_the_pool`，被拒的那次挂载恒交不带故障层：判据在崩溃镜像上复算为真、
/// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670，「读阶段判完之后系统配置又读不出」
/// 那一拒的判据复算为真的同样另判不合法，改法 G4），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
/// 模型对可写挂载与之后那一次发布（覆盖写活着的第一个文件、第一个文件、建 inode）从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、挂载那一趟与它之后的记录核对、判拒绝要的崩溃镜像那一份、这一个崩溃状态落在哪"
)]
// gpu-exempt: writable-mount
pub(super) fn judge_the_writable_mount_on_the_crash_image(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    mount_run: &WritableMountOnTheCrashImage,
    record_check: RecordCheck,
    pool_checker_violations: Vec<(&'static str, String)>,
    refusal_on_the_crash_image: RefusalOnTheCrashImage<'_>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    tally.writable_mounts_after_the_crash += 1;
    let refusal = match &mount_run.outcome {
        WritableMountOutcome::MountRefused {
            member,
            reason,
            refusal: refusal_error,
        } => {
            let disk =
                if mount_run.pool_after == *refusal_on_the_crash_image.crash_image_materialized {
                    DiskAfterTheRefusal::ByteForByteUnchanged
                } else {
                    DiskAfterTheRefusal::Changed
                };
            match judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
                refusal_on_the_crash_image.crash_image_materialized,
                refusal_on_the_crash_image.filesystem_identifier_of_the_pool,
                refusal_on_the_crash_image.version_landed_on_after_the_crash,
                refusal_error,
                disk,
                refusal_on_the_crash_image.read_only_read_back,
                refusal_on_the_crash_image.read_faults,
                READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
            ) {
                WritableMountRefusalVerdict::Legal(_) => {
                    tally.writable_mount_refusals_on_the_legal_refusal_list_legal += 1;
                    None
                }
                WritableMountRefusalVerdict::Illegal {
                    reason: reason_on_the_list,
                    conditions_not_met,
                } => {
                    tally.writable_mount_refusals_on_the_legal_refusal_list_illegal += 1;
                    Some(format!(
                        "崩溃后镜像上的可写挂载拒了，理由 {reason_on_the_list:?} 在 D23 已定项 14 那张单子上，拒得不合法（{conditions_not_met:?}）：{member}"
                    ))
                }
                WritableMountRefusalVerdict::ReasonNotOnTheLegalRefusalList => {
                    if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall)
                    {
                        tally.writable_mounts_after_the_crash_refused_at_the_unit_area_wall += 1;
                        None
                    } else {
                        tally.writable_mounts_after_the_crash_refused += 1;
                        Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))
                    }
                }
            }
        }
        WritableMountOutcome::PublishRefused { member, reason } => {
            tally.writable_mounts_after_the_crash_succeeded += 1;
            if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) {
                tally.publishes_after_the_writable_mount_refused_at_the_unit_area_wall += 1;
                None
            } else {
                tally.publishes_after_the_writable_mount_refused += 1;
                Some(format!("崩溃后可写挂载之后的那一次发布拒了：{member}"))
            }
        }
        WritableMountOutcome::Published => {
            tally.writable_mounts_after_the_crash_succeeded += 1;
            tally.publishes_after_the_writable_mount += 1;
            None
        }
    };
    tally.checker_runs_after_the_writable_mount += 1;
    let violations = pool_checker_violations;
    if violations.is_empty() && refusal.is_none() && record_check == RecordCheck::default() {
        return;
    }
    let model_disagreement = refusal.map(|implementation_answer| {
        ModelDisagreement::new(
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            "崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）与之后那一次发布（第一个文件还活着时覆盖写、树表 0 条时第一个文件、第一个文件已删完或正在删时建一个 inode）都该成（单元区墙之外模型不许拒）"
                .to_string(),
            implementation_answer,
        )
    });
    let image = CrashImage {
        base: &mount_run.pool_after,
        writes: &[],
        persisted: Vec::new(),
    };
    let observation = crash_state_observation(
        &image,
        crash_point,
        violations,
        model_disagreement,
        record_check,
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::AfterTheWritableMountAndOnePublish,
            observation,
            read_back: format!("{:?}", mount_run.outcome),
            image: &image,
        },
        image_directory,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mount_transparency::WitnessedCounterBranchThatHolds;
    use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;

    /// 挂载续做（Q-J，D8（核心索引结构） 已定项 16 第 6 条）：挂载那一段的写里，写行之后、第「续做之前发了几次」个根槽写之后起的写
    /// 归续做那一段，而且摆二次崩溃时那一段也摆到一个。造的写表：取号的系统配置槽写；写行那次（单元、记录、根槽、轮换的系统配置槽）；
    /// 一次暖机（单元、根槽）；续做那一步（单元、记录、根槽、轮换）——续做之前发了 2 次。
    /// 判别力：`WritableMountPhase::ALL` 里没有 `Continuation` 时这一段一个二次崩溃都摆不到；按写的种类（不按根槽数）分段时续做的写归暖机。
    #[test]
    fn the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point(
    ) {
        let write_of = |kind: StepKind| RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(0),
            contents: crate::memory_pool::WrittenContents::Bytes(vec![0; 512]),
        };
        let kinds = [
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
            StepKind::RootRecordFua,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
        ];
        let writes: Vec<RetainedWrite> = kinds.iter().map(|kind| write_of(*kind)).collect();
        let phases = writable_mount_phase_of_each_write(&writes, Some(2), None);
        assert_eq!(
            phases,
            [
                WritableMountPhase::Acquisition,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::WarmUp,
                WritableMountPhase::WarmUp,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
            ],
            "取号、写行（到它的根槽与轮换）、暖机一次、续做一步"
        );
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: vec![false],
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let one_write_per_segment: Vec<Vec<usize>> = (0..writes.len())
            .map(|write_index| vec![write_index])
            .collect();
        let second_crash_points = draw_second_crash_points(
            HistorySeed(7),
            &crash_point,
            &one_write_per_segment,
            &phases,
        );
        assert_eq!(
            second_crash_points
                .iter()
                .filter(|point| point.phase == WritableMountPhase::Continuation)
                .count(),
            1,
            "续做那一段摆一个二次崩溃：{second_crash_points:?}"
        );
    }

    /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
    /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
    /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
    /// 判别力：`WritableMountPhase::ALL` 里没有这两段时一个二次崩溃都摆不到；不按 FUA 分开时清扫发布的写归抹头。
    #[test]
    fn the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point(
    ) {
        let write_of = |kind: StepKind, is_force_unit_access: bool| RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access,
            offset: DeviceOffsetInBytes(0),
            contents: crate::memory_pool::WrittenContents::Bytes(vec![0; 512]),
        };
        let writes = vec![
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::UnitWrite, true),
            write_of(StepKind::UnitWrite, true),
            write_of(StepKind::Barrier, false),
            write_of(StepKind::Barrier, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::Barrier, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
        ];
        let phases = writable_mount_phase_of_each_write(&writes, None, Some(2));
        assert_eq!(
            phases[7..],
            [
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
            ],
            "抹两个头与两道屏障归抹头，清扫发布从它第一个不带 FUA 的单元写起"
        );
        assert_eq!(
            phases[..7],
            [
                WritableMountPhase::Acquisition,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::WarmUp,
                WritableMountPhase::WarmUp,
            ],
            "清扫之前那几段照旧"
        );
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: vec![false],
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let one_write_per_segment: Vec<Vec<usize>> = (0..writes.len())
            .map(|write_index| vec![write_index])
            .collect();
        let second_crash_points = draw_second_crash_points(
            HistorySeed(7),
            &crash_point,
            &one_write_per_segment,
            &phases,
        );
        for phase in [
            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
            WritableMountPhase::InstanceTableSweepPublish,
        ] {
            assert_eq!(
                second_crash_points
                    .iter()
                    .filter(|point| point.phase == phase)
                    .count(),
                1,
                "{} 那一段摆一个二次崩溃：{second_crash_points:?}",
                phase.name()
            );
        }
    }

    /// 清扫一个头都没抹（只删行）：清扫之前最后一次发布的轮换之后，第一个写就是清扫发布的单元写，抹头那一段一个写都没有。
    /// 把清扫发布的第一个写记进抹头那一段的，二次崩溃会说崩在了没写过一个字节的抹头里（假绿）。
    #[test]
    fn sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish(
    ) {
        let write_of = |kind: StepKind, is_force_unit_access: bool| RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access,
            offset: DeviceOffsetInBytes(0),
            contents: crate::memory_pool::WrittenContents::Bytes(vec![0; 512]),
        };
        let writes = vec![
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
        ];
        let phases = writable_mount_phase_of_each_write(&writes, None, Some(1));
        assert_eq!(
            phases,
            [
                WritableMountPhase::Acquisition,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
            ],
            "写行那次的轮换之后没有 FUA 写零：清扫发布从它第一个单元写起，抹头那一段是空的"
        );
    }

    /// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
    fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
        let mut bytes = vec![0u8; 512];
        if let Some((instance, checkpoint_txg)) = root {
            bytes[24..28].copy_from_slice(&instance.to_le_bytes());
            bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
        }
        RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(0),
            contents: crate::memory_pool::WrittenContents::Bytes(bytes),
        }
    }

    fn writes_by_structure_kind_of(kinds: &[WrittenStructureKind]) -> WritesByStructureKind {
        let mut writes = WritesByStructureKind::NOTHING_WRITTEN;
        for kind in kinds {
            writes.count_write_call(*kind, &[0u8; 512]);
        }
        writes
    }

    /// U12（`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：挂载在续做里交回 `MountError::Publish` 时，
    /// 按错里交回的已落盘发布写账划出续做段。写账照在副本上量到的种类造：写行（实例表、树表、分配记录树……）、
    /// 暖机（分配记录树、记账树……）、续做那一步（extent 树、inode 树根……）。
    /// 判别力：错的那一支交 None 时续做的写全归暖机段（攻方 A4）；不认续做的种类时同样交 None。
    #[test]
    fn mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes(
    ) {
        let row_publish = writes_by_structure_kind_of(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::InstanceTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let warm_up = writes_by_structure_kind_of(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::AccountingTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let continuation_step = writes_by_structure_kind_of(&[
            WrittenStructureKind::ExtentTreeNode,
            WrittenStructureKind::InodeTreeRoot,
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &[
                    row_publish.clone(),
                    warm_up.clone(),
                    warm_up.clone(),
                    continuation_step.clone(),
                    continuation_step.clone()
                ],
                &[],
            ),
            Some(3),
            "写行、两次暖机之后的第一步续做起是续做"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &[row_publish.clone(), warm_up.clone()],
                std::slice::from_ref(&continuation_step),
            ),
            Some(2),
            "续做第一步落盘阶段失败：续做从失败那一次起"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&warm_up),
            ),
            None,
            "错出在暖机里：没有续做，写行之后都算暖机"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &[],
                std::slice::from_ref(&row_publish),
            ),
            None,
            "写行那次就失败：没有续做"
        );
        let phases_of_a_mount_that_failed_in_the_second_continuation_step =
            writable_mount_phase_of_each_write(
                &[
                    synthetic_write(StepKind::SystemConfigurationSlot, None),
                    synthetic_write(StepKind::UnitWrite, None),
                    synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
                    synthetic_write(StepKind::UnitWrite, None),
                    synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
                    synthetic_write(StepKind::UnitWrite, None),
                    synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
                    synthetic_write(StepKind::UnitWrite, None),
                ],
                publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                    &[row_publish, warm_up, continuation_step.clone()],
                    std::slice::from_ref(&continuation_step),
                ),
                None,
            );
        assert_eq!(
            phases_of_a_mount_that_failed_in_the_second_continuation_step[5..],
            [
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation
            ],
            "续做那两步（落盘的一步与失败那一步写出的单元）的写落在续做段，不并进暖机段"
        );
    }

    /// V1（`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：挂载交回 `MountError::FloorRaiseFailedAfterTheMountsPublishes`
    /// （续做里推的抬 F 自己报了不是空间不够的错）时，按它的 `writes_of_persisted_publishes` 划续做段，与 `MountError::Publish` 那一臂同一个划法：
    /// 写账照 U12 那一条的种类造（写行、暖机、续做那一步），续做从第 2 次发布起；只有写行与暖机的（准入推的抬 F 报错，今天就交这个成员）没有续做。
    /// 判别力：这个成员归「别的错」交 None 时续做的写全归暖机段。
    #[test]
    fn mount_whose_floor_raise_failed_after_a_continuation_step_marks_the_continuation_by_the_persisted_publishes(
    ) {
        let row_publish = writes_by_structure_kind_of(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::InstanceTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let warm_up = writes_by_structure_kind_of(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::AccountingTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let continuation_step = writes_by_structure_kind_of(&[
            WrittenStructureKind::ExtentTreeNode,
            WrittenStructureKind::InodeTreeRoot,
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let floor_raise_failed_after =
            |writes_of_persisted_publishes: Vec<WritesByStructureKind>| {
                MountError::FloorRaiseFailedAfterTheMountsPublishes(Box::new(
                    FloorRaiseFailedAfterTheMountsPublishes {
                        cause: MountError::InstanceTableMalformed,
                        writes_of_persisted_publishes,
                        floor_raises: Vec::new(),
                    },
                ))
            };
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_failed(&floor_raise_failed_after(
                vec![row_publish.clone(), warm_up.clone(), continuation_step]
            )),
            Some(2),
            "写行、一次暖机之后的那一步续做起是续做"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_failed(&floor_raise_failed_after(
                vec![row_publish, warm_up]
            )),
            None,
            "准入推的抬 F 报错（只落了写行与暖机）：没有续做，写行之后都算暖机"
        );
    }

    /// D8（核心索引结构） 已定项 16 第 5 条：删完那一版上只许建 inode、写行与空发布。第一个文件之后经挂着的会话 unlink 它跑完（意图那次 + 最后一批），
    /// 整份盘面当崩溃镜像：可写挂载交回的现行那一版是有树而文件集合为空的一版，之后那一次发布建一个 size 0 的 inode、做成。
    /// 判别力：那一版上照旧调覆盖写时被拒成 `WriteToAnInodeThatIsNotInTheTree`、结局是「发布拒了」。
    #[test]
    fn the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published()
    {
        use crate::history::{HistoryDeviceWidth, PerStepChecker};
        let width = HistoryDeviceWidth::UnitAreaOf384Slots;
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::Unlink],
        };
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(&history, execution, &stream, &mut |_observation| {});
        assert_eq!(
            run.outcomes.len(),
            history.operations.len(),
            "unlink 那一步要跑完：{:?}",
            run.ending
        );
        let mut crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            width.device_bytes(),
        );
        crash_image.apply(&stream.retained_operations());
        assert!(
            matches!(
                recover(&crash_image, JournalPolicy::Consult).outcome,
                RecoveryOutcome::NoFile { .. }
            ),
            "前提：unlink 跑完之后的盘面上只读恢复读出「没有文件」"
        );
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: Vec::new(),
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image,
            &width.parameters(),
            execution,
            HistorySeed(0),
            &crash_point,
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "删完那一版上挂载之后那一次发布建 inode、做成：{:?}",
            mount_run.outcome
        );
    }

    /// unlink 之后再建过 inode 的一版归带文件的一版、第一个文件那条记录不在（删除与并发那条线读码诊断；攻方随机历史种子 32768、245760 两段那一形，
    /// 判决 `research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节「越出攻击面」那一行）：第一个文件之后 unlink 它、再建一个 inode，
    /// 崩在建 inode 那次发布里——根槽落了、紧跟着的系统配置轮换没落，只读恢复落在建 inode 那一版（这段历史最后提交的一版）。
    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现；发布之后那份盘只读挂载读回的 inode 记录是崩溃那一版的
    /// 每一条原样加新建的那一条，新建那一条整条等于第二截那一支的调法该写出的（改法 a，`research/prompts/m3-formal-code-b20-r1-main-verification.md`
    /// 第二节 K2 H1；比整条是改法 A1，`research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节「K2 改法 a 判别力」与第三节第 2 条）。
    /// 判别力：带文件那一臂改回「一律覆盖写」（`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、
    /// 记一条第二截的新发现，这一条红；那一臂建 0 个 inode、新建那一条 nlink = 0、出生代取上一版、size 4096、写入时间 0
    /// （`crates/mutations.tsv` 各一行；攻方量过后四种池级 checker 照样绿，`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt`），
    /// 红在比整条记录那一条。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
    #[test]
    fn the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published(
    ) {
        use crate::history::{HistoryDeviceWidth, PerStepChecker};
        use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
        use singlefs_core::transaction::FIRST_INODE_NUMBER;
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode],
        };
        let stream = SharedStream::retaining_contents();
        let mut operations_after_the_unlink = None;
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            if observation.position == StepPosition::Operation(0) {
                operations_after_the_unlink = Some(stream.operation_count());
            }
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "前提：unlink 与建 inode 都跑完"
        );
        let operations_after_the_unlink =
            operations_after_the_unlink.expect("unlink 那一步跑完时登记过一次");
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let root_of_the_inode_creation = (0..writes.len())
            .rev()
            .find(|write| {
                stream_indexes[*write] >= operations_after_the_unlink
                    && writes[*write].kind == StepKind::RootRecordFua
            })
            .expect("前提：建 inode 那次发布写了根槽");
        assert_eq!(
            writes
                .get(root_of_the_inode_creation + 1)
                .map(|write| write.kind),
            Some(StepKind::SystemConfigurationSlot),
            "前提：建 inode 那次发布的根槽 FUA 之后紧跟着轮换系统配置"
        );
        let segment_of_the_root =
            segment_containing_the_write(&segments, root_of_the_inode_creation);
        let first_write_of_the_segment = segments[segment_of_the_root][0];
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: segment_of_the_root,
                writes_from_the_segment_on: root_of_the_inode_creation + 2
                    - first_write_of_the_segment,
                withheld_among_them: vec![
                    root_of_the_inode_creation + 1 - first_write_of_the_segment,
                ]
                .leak(),
            },
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        let tally = &injection.tally;
        assert_eq!(
            injection
                .crash_points
                .iter()
                .map(|crash_point| crash_point.operation_kind)
                .collect::<Vec<_>>(),
            [Some(HistoryOperationKind::CreateInode)],
            "前提：只摆写死的那一个崩溃态，崩在建 inode 那次发布里"
        );
        assert_eq!(
            tally.crash_points_that_land_before_the_last_committed_version,
            0,
            "前提：建 inode 那次发布的根落了，崩溃态落在这段历史最后提交的那一版上：{}",
            tally.render()
        );
        assert_eq!(
            tally.writable_mounts_after_the_crash_succeeded,
            1,
            "前提：崩溃后镜像上的可写挂载做成：{}",
            tally.render()
        );
        let findings_after_the_writable_mount: Vec<String> = injection
            .new_findings
            .iter()
            .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
            .map(CrashPointFinding::render)
            .collect();
        assert_eq!(
            (
                tally.publishes_after_the_writable_mount,
                tally.publishes_after_the_writable_mount_refused
            ),
            (1, 0),
            "第一个文件已删完的带文件一版上，挂载之后那一次发布建一个 inode、做成：{findings_after_the_writable_mount:?}"
        );
        assert!(
            findings_after_the_writable_mount.is_empty(),
            "第二截没有新发现：{findings_after_the_writable_mount:?}"
        );
        // 崩溃态是写表前缀到建 inode 那次的根槽为止（段内摆到 root + 1、扣下 root + 1，之后一个都没落），在同一份盘面上照第二截那一种调法
        // 再挂一次加一次发布，比发布之后那份池与崩溃那一份上只读挂载读回的 inode 号。
        let mut crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        crash_image.apply_writes(&writes[..=root_of_the_inode_creation]);
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image,
            &execution.device_width.parameters(),
            execution,
            history.seed,
            &injection.crash_points[0],
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "前提：同一份盘面上再挂一次加一次发布，照样做成：{:?}",
            mount_run.outcome
        );
        let read_only_mount = |pool: &MemoryPool| {
            singlefs_core::mounted_read::mount_read_only(pool)
                .expect("崩溃那一版与发布之后那一版都是整次落盘的版本，只读挂载做得成")
        };
        let crashed_version = read_only_mount(&crash_image);
        let version_after_the_publish = read_only_mount(&mount_run.pool_after);
        // 第一个文件取 1 号（已删完），unlink 之后历史里建的那一个取 2 号：崩溃那一版树里只剩 2 号，inode 号水位 3。
        assert_eq!(
            crashed_version
                .mounted
                .inode_records()
                .iter()
                .map(|record| record.inode)
                .collect::<Vec<_>>(),
            [FIRST_INODE_NUMBER + 1],
            "前提：崩溃那一版树里只剩历史里建的那一个 inode"
        );
        // 照第二截那一支的调法 `publish_new_inodes(…, 1, FIXED_WRITE_TIME_SECONDS, 挂载的实例)` 该写出的那一条：号从水位 3 起发，
        // 对象出生代与改动计数都是这次发布的 checkpoint_txg（发布之后那一版的根），活着（nlink = 1），长度 0（blocks 由 size 现算，跟着是 0）。
        let txg_of_the_publish = version_after_the_publish.effective_root.checkpoint_txg;
        let mut expected_records_after_the_publish =
            crashed_version.mounted.inode_records().to_vec();
        expected_records_after_the_publish.push(InodeRecord {
            inode: FIRST_INODE_NUMBER + 2,
            object_birth: txg_of_the_publish,
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 0,
            change_count: txg_of_the_publish.0,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            // 新建的 inode 没被截断过：截断字段是 0（判决 m4-trunc-r2 R2）。
            truncation_field: singlefs_core::records::InodeRecordTruncationField::NOT_WRITTEN,
        });
        assert_eq!(
            version_after_the_publish.mounted.inode_records(),
            expected_records_after_the_publish.as_slice(),
            "发布之后只读挂载读回的 inode 记录 = 崩溃那一版的每一条原样 + 新建的那一条（号、出生代、nlink、size、改动计数、写入时间逐个对上）"
        );
    }

    /// 崩溃后那次可写挂载末尾的清扫发布删了行（C704（挂载透明不认清扫发布） 的验收）：挂载做成、删了实例 1 那一行、一个头都没抹；
    /// 只挂载与挂载加一次发布两个比法拿到的 R2 都是清扫发布那一版，实例表行那一样把删的行放回去再比，挂载透明一样都没红；
    /// 清扫发布那一段摆了二次崩溃，再挂判了 I-8.4、挂载透明判了；抹头那一段一个二次崩溃都没摆（没抹头时那一段一个写都没有）；
    /// 一条新发现都没有。判别力：挂载透明不放回删的行时只挂载那一个比法判红（`crates/mutations.tsv` 里点名这条的行）。
    #[test]
    fn writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field(
    ) {
        let injection =
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                given_states_engine_for("crash_injection_unit_tests").engine(),
            );
        let tally = &injection.tally;
        let rendered = tally.render();
        assert_eq!(
            (
                tally.crash_points,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.publishes_after_the_writable_mount
            ),
            (1, 1, 1),
            "前提：摆了一个崩溃状态，那次可写挂载与之后那一次发布都做成：{rendered}"
        );
        assert_eq!(
            (
                tally.writable_mounts_after_the_crash_deleting_instance_table_rows,
                tally.heads_erased_by_the_instance_table_sweep_after_the_crash
            ),
            (1, 0),
            "前提：那次挂载末尾的清扫发布删了行、一个头都没抹（这段历史每次发布都做完了，没有孤儿）：{rendered}"
        );
        assert_eq!(
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 2,
            "只挂载与挂载加一次发布两个比法拿到的 R2 都是清扫发布那一版，各放回一次删的行再比：{rendered}"
        );
        let phase_count = |counts: &BTreeMap<WritableMountPhase, u64>, phase| {
            counts.get(&phase).copied().unwrap_or(0)
        };
        assert_eq!(
            phase_count(
                &tally.second_crash_points_by_phase,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
            ),
            0,
            "一个头都没抹时抹头那一段一个写都没有，不摆二次崩溃：{rendered}"
        );
        for (what, counts) in [
            ("摆了二次崩溃", &tally.second_crash_points_by_phase),
            ("再挂判了 I-8.4", &tally.remount_idempotence_judged_by_phase),
            (
                "判了挂载透明",
                &tally.mount_transparency_judged_on_second_crashes_by_phase,
            ),
        ] {
            assert!(
                phase_count(counts, WritableMountPhase::InstanceTableSweepPublish) >= 1,
                "清扫发布那一段{what}：{rendered}"
            );
        }
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "挂载透明判红的字段 {:?}：{}",
            tally.mount_transparency_violations_by_field,
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
        assert_eq!(
            tally.remounts_after_a_second_crash_reading_different_content, 0,
            "二次崩溃之后再挂读出的内容与没崩过的那一次不同：{rendered}"
        );
        assert!(
            injection.new_findings.is_empty(),
            "新发现：{}",
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
    }

    /// 崩溃后那次可写挂载末尾的清扫先抹孤儿的头、再发清扫发布删行（D18（块里携带什么信息） 已定项 11「回收」一条，抹头那一段）：
    /// 挂载做成、删了行、抹了至少一个头；抹头那一段摆了二次崩溃、再挂判了 I-8.4、判了挂载透明；挂载透明一样都没红、二次崩溃之后再挂
    /// 读出的内容与没崩过的那一次相同、一条新发现都没有。判别力：历史开头那次覆盖写掉电不截写（`singlefs_verification::history` 里
    /// 掉电那一截的判法）时盘上没有孤儿，「抹了至少一个头」那条先红。
    #[test]
    fn writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field(
    ) {
        let injection =
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                given_states_engine_for("crash_injection_unit_tests").engine(),
            );
        let tally = &injection.tally;
        let rendered = tally.render();
        assert_eq!(
            (
                tally.crash_points,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.writable_mounts_after_the_crash_deleting_instance_table_rows
            ),
            (1, 1, 1),
            "前提：摆了一个崩溃状态，那次可写挂载做成、末尾的清扫发布删了行：{rendered}"
        );
        assert!(
            tally.heads_erased_by_the_instance_table_sweep_after_the_crash >= 1,
            "前提：那次挂载末尾的清扫抹了孤儿的头：{rendered}"
        );
        let phase_count = |counts: &BTreeMap<WritableMountPhase, u64>, phase| {
            counts.get(&phase).copied().unwrap_or(0)
        };
        for (what, counts) in [
            ("摆了二次崩溃", &tally.second_crash_points_by_phase),
            ("再挂判了 I-8.4", &tally.remount_idempotence_judged_by_phase),
            (
                "判了挂载透明",
                &tally.mount_transparency_judged_on_second_crashes_by_phase,
            ),
        ] {
            assert!(
                phase_count(
                    counts,
                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
                ) >= 1,
                "抹头那一段{what}：{rendered}"
            );
        }
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "挂载透明判红的字段 {:?}：{rendered}",
            tally.mount_transparency_violations_by_field
        );
        assert_eq!(
            tally.remounts_after_a_second_crash_reading_different_content, 0,
            "二次崩溃之后再挂读出的内容与没崩过的那一次不同：{rendered}"
        );
        assert!(
            injection.new_findings.is_empty(),
            "新发现：{}",
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
    }

    /// 崩溃后镜像上那次可写挂载推了抬 F 时，交回抬 F 从挂载写表的哪个写起——一个系统配置槽写，
    /// 前面紧挨着暖机末尾的系统配置轮换；二次崩溃上 F 判不判按它分（分法两侧的判定由验证包的
    /// `mount_transparency_judged_field_by_field_after_a_writable_mount` 钉）。盘面照攻方 fms12 的造法：单元区 256 槽的小盘、
    /// 关掉空间准入跑第一个文件之后重挂与 12 次覆盖写，再判准入挂载一次。
    #[test]
    fn the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up() {
        use crate::history::{HistoryDeviceWidth, PerStepChecker};
        let width = HistoryDeviceWidth::UnitAreaOf256Slots;
        let mut operations = vec![HistoryOperation::CloseAndMountWritable];
        operations.extend(std::iter::repeat_n(
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 200 },
                fill_seed: 11,
            }),
            12,
        ));
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        };
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(
            &history,
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: width,
                space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
            },
            &stream,
            &mut |_observation| {},
        );
        assert_eq!(
            run.outcomes.len(),
            history.operations.len(),
            "写死的那段历史每一步都要跑完：{:?}",
            run.ending
        );
        let mut crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            width.device_bytes(),
        );
        crash_image.apply(&stream.retained_operations());
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: Vec::new(),
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image,
            &width.parameters(),
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: width,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
            HistorySeed(0),
            &crash_point,
        );
        assert!(
            matches!(
                mount_run.floor_raised_by_the_mount,
                Some(FloorRaisedByTheMount::RaisedTo(_))
            ),
            "前提：这次挂载在写行与暖机之后推了抬 F：{:?}",
            mount_run.floor_raised_by_the_mount
        );
        let Some(FloorRaisesInTheMountWrites::FirstWriteAt(first_floor_raise_write)) =
            mount_run.floor_raises_in_the_mount_writes
        else {
            panic!(
                "挂载推了抬 F，交回的却不是抬 F 从哪个写起：{:?}",
                mount_run.floor_raises_in_the_mount_writes
            );
        };
        let (mount_writes, _mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &width.fixed_geometry(),
            );
        assert_eq!(
            (
                mount_writes[first_floor_raise_write - 1].kind,
                mount_writes[first_floor_raise_write].kind
            ),
            (
                StepKind::SystemConfigurationSlot,
                StepKind::SystemConfigurationSlot
            ),
            "抬 F 从把新 F 写进系统配置的那个写起，前面紧挨着暖机末尾的系统配置轮换"
        );
    }

    /// C670 判别力（E171（三oracle合谋测试） 第 242、269 行）：起点（取号、暖机、第一个文件）加一次关掉重挂的历史，短段整段枚举；
    /// 历史里没有带读故障的步，每个崩溃态都判 C670 那一条，第二截一个拒得不合法的都没有、没有新发现。
    /// 系统配置轮换挪到根槽 FUA 之前时（`crates/mutations.tsv` 里「发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前」那条改法），暖机那一段摆得出
    /// 「轮换落了、根没落」：记录不跨实例边界，所选那一版停在上一个实例那一版，见证值比它新，正确实现在这份盘面上永远拒可写——这一条红；
    /// 取号写的见证值取成槽世代号时（「见证值取成槽世代号」那条），重挂取号写落了、暖机的根没落的那几态同样红。
    // crash-case-check:not-a-crash-case 起点加一步的合成历史、短段整段枚举，不调全量枚举，崩溃态数以百计（推的，没量过）
    #[test]
    fn no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version(
    ) {
        use crate::history::{HistoryDeviceWidth, PerStepChecker};
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::CloseAndMountWritable],
        };
        let injection = inject_crashes_into_history(
            &history,
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 4,
                sampled_in_longer_segments: 2,
            },
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        let tally = &injection.tally;
        assert!(tally.crash_points > 0, "一个崩溃态都没摆出来：{tally:?}");
        assert_eq!(
            tally.writable_mounts_after_the_crash, tally.crash_points,
            "每个崩溃态都起了一次可写挂载：{tally:?}"
        );
        assert_eq!(
            tally.crash_points_after_a_step_reading_under_injected_faults, 0,
            "这段历史没有带读故障的步，每个崩溃态都判 C670 那一条：{tally:?}"
        );
        assert!(
            injection
                .crash_points
                .iter()
                .any(|crash_point| crash_point.step == StepPosition::StartingPoint)
                && injection.crash_points.iter().any(|crash_point| {
                    crash_point.operation_kind == Some(HistoryOperationKind::CloseAndMountWritable)
                }),
            "起点与关掉重挂那一步都要摆到崩溃态：{:?}",
            injection
                .crash_points
                .iter()
                .map(|crash_point| (crash_point.step, crash_point.operation_kind))
                .collect::<Vec<_>>()
        );
        let new_findings: Vec<String> = injection
            .new_findings
            .iter()
            .map(CrashPointFinding::render)
            .collect();
        assert_eq!(
            tally.writable_mount_refusals_on_the_legal_refusal_list_illegal, 0,
            "没注入读故障的崩溃态上可写挂载以见证判据为由拒、拒得不合法：{new_findings:?}"
        );
        assert!(
            new_findings.is_empty(),
            "没注入读故障的崩溃态上有新发现：{new_findings:?}"
        );
    }

    /// C670 的接线（攻方改法 P2，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-e）：经崩溃注入的入口摆一个靶向崩溃态——
    /// 第一个文件之后一次覆盖写，那次发布只落了系统配置轮换，单元、记录与根都没落（盘面照
    /// `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的造法；越过了根槽 FUA 与轮换的先后，今天的持久顺序下段内真子集摆不出来）。
    /// 历史里没有带读故障的步；第二截的可写挂载以判据 N-配置 拒，盘没变、只读读出在允许集合里，U6 四条都满足，C670 那一条判不合法，计 1。
    /// 判别力：调用点把读故障的认定恒交 `Injected`、第二截换回不判 C670 的旧判官 `judge_writable_mount_refusal`（`crates/mutations.tsv` 两行），
    /// 这一拒都判成合法，这一条红；调用点把所选那一版交成 None 时计数照样 (0, 1)，不合法改由「判据复算为假」给出，红在条件那两条断言上
    /// （攻方改法 F1）。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
    #[test]
    fn crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let writes_of_the_overwrite = &writes[first_write_of_the_overwrite..];
        assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
            writes_of_the_overwrite,
        );
        let segment_index = segments
            .iter()
            .position(|segment| segment[0] == first_write_of_the_overwrite)
            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
        let withheld_among_them: Vec<usize> = writes_of_the_overwrite
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind != StepKind::SystemConfigurationSlot)
            .map(|(within_the_overwrite, _)| within_the_overwrite)
            .collect();
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index,
                writes_from_the_segment_on: writes_of_the_overwrite.len(),
                withheld_among_them: withheld_among_them.leak(),
            },
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        let tally = &injection.tally;
        assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃态");
        assert_eq!(
            tally.crash_points_after_a_step_reading_under_injected_faults,
            0,
            "这段历史没有带读故障的步：{}",
            tally.render()
        );
        assert_eq!(
            tally.model_contents_compared,
            1,
            "前提：只读恢复读出第一个文件那一版，在模型的允许集合里（U6 只读读出那一条满足）：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.writable_mount_refusals_on_the_legal_refusal_list_legal,
                tally.writable_mount_refusals_on_the_legal_refusal_list_illegal
            ),
            (0, 1),
            "第二截的可写挂载以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：拒得不合法，计 1：{}",
            tally.render()
        );
        // 不合法是哪一条给的（攻方改法 F1，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：调用点把所选那一版交成 None 时，
        // C670 那一条不成立、U6「判据复算为假」那一条顶上来，计数照样是 (0, 1)；只有看判官列出的没满足的条件才分得开。
        let conditions_not_met = conditions_not_met_named_by_the_one_illegal_writable_mount_refusal(
            &injection.new_findings,
        );
        let witness_criterion_true_without_read_faults = format!(
            "{:?}",
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
                WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
            )
        )
        .split('(')
        .next()
        .expect("split 至少交回一段")
        .to_string();
        let criterion_false_when_recomputed = format!(
            "{:?}",
            RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
        );
        assert!(
            conditions_not_met.contains(&witness_criterion_true_without_read_faults),
            "拒得不合法要由 C670 那一条（{witness_criterion_true_without_read_faults}）给出：判官列出的没满足的条件 {conditions_not_met}"
        );
        assert!(
            !conditions_not_met.contains(&criterion_false_when_recomputed),
            "判据在崩溃镜像上复算为真（U6 那一条满足），不许列出 {criterion_false_when_recomputed}：判官列出的没满足的条件 {conditions_not_met}"
        );
    }

    /// 第二截那一条拒得不合法的发现里，判官列出的没满足的条件（[`judge_the_writable_mount_on_the_crash_image`] 写进实现那一半的
    /// 「拒得不合法（…）：」那一段，`Vec<RefusalConditionNotMet>` 的 Debug 文本）。
    ///
    /// # Panics
    /// 第二截的新发现不是恰好一条、它没有模型对不上那一半、或那一半里没有「拒得不合法（…）：」那一段。
    fn conditions_not_met_named_by_the_one_illegal_writable_mount_refusal(
        new_findings: &[CrashPointFinding],
    ) -> String {
        let findings_after_the_writable_mount: Vec<&CrashPointFinding> = new_findings
            .iter()
            .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
            .collect();
        let [finding] = findings_after_the_writable_mount.as_slice() else {
            panic!(
                "第二截要恰好一条新发现（那一次拒得不合法）：{:?}",
                findings_after_the_writable_mount
                    .iter()
                    .map(|finding| finding.render())
                    .collect::<Vec<_>>()
            );
        };
        let implementation_answer = &finding
            .observation
            .model_disagreement
            .as_ref()
            .unwrap_or_else(|| panic!("拒得不合法记成模型对不上：{}", finding.render()))
            .implementation_answer;
        implementation_answer
            .split_once("拒得不合法（")
            .and_then(|(_, after)| after.split_once("）："))
            .map(|(conditions, _)| conditions.to_string())
            .unwrap_or_else(|| {
                panic!("实现那一半里没有「拒得不合法（…）：」那一段：{implementation_answer}")
            })
    }
}
