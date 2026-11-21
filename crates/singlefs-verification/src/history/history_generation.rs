//! 按种子与比重生成一段历史：每一组比重、逐步抽操作、每一步之后预期的会话与第一个文件。

use super::*;

/// 按种子生成的一段历史：起点与操作序列。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedHistory {
    pub seed: HistorySeed,
    pub starting_point: HistoryStartingPoint,
    pub operations: Vec<HistoryOperation>,
}

/// 生成时对会话的乐观估计：只用来调各类操作的比重，不进执行——执行按真实结局走，估计错了那一步就记「前提不满足」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedSession {
    Closed,
    OpenWithoutFile,
    OpenWithFile,
    /// 这段历史 unlink 过第一个文件（之后照估计一直没有它）：只在比重写了「unlink 之后那张表」时才进这一估计
    /// （[`GenerationWeights::with_session_open_after_the_file_was_deleted`]），没写的那几组照旧按「开着、没有文件」抽。
    OpenAfterTheFileWasDeleted,
}

enum_listing_every_member! {
/// 复现入口认得的每一组比重的名字：一组一个成员，[`GenerationWeights::NAMED`] 照成员表生成。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedGenerationWeights {
    /// [`GenerationWeights::BROAD`]。
    Broad,
    /// [`GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`]。
    ReuseAfterRaisingTheFloor,
    /// [`GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR`]。
    RollbackAfterRaisingTheFloor,
    /// [`GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`]。
    TowardTheAllocationRecordWall,
    /// [`GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`]。
    TowardTheUnitAreaWall,
    /// [`GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`]。
    DeletingFilesOfSeveralLowerLeaves,
    /// [`GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP`]。
    LeavingOrphansForTheInstanceTableSweep,
    /// [`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS`]。
    NewestSystemConfigurationSlotReadingZeros,
    /// [`GenerationWeights::SNAPSHOT_OPERATIONS`]。
    SnapshotOperations,
    /// [`GenerationWeights::COMPACTION_WHILE_MOUNTED`]。
    CompactionWhileMounted,
    /// [`GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`]。
    CompactionWhileMountedWithRollback,
    /// [`GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`]。
    BroadWithCompactionWhileMounted,
    /// [`GenerationWeights::TRUNCATION`]。
    Truncation,
    /// [`GenerationWeights::WRITES_BY_INODE_NUMBER`]。
    WritesByInodeNumber,
    /// [`GenerationWeights::SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST`]。
    SizeChangesOfInodesOtherThanTheFirst,
    /// [`GenerationWeights::PARTIAL_WRITES`]。
    PartialWrites,
}
}

impl NamedGenerationWeights {
    /// 这个名字指的那一组比重。
    #[must_use]
    pub const fn weights(self) -> GenerationWeights {
        match self {
            NamedGenerationWeights::Broad => GenerationWeights::BROAD,
            NamedGenerationWeights::ReuseAfterRaisingTheFloor => {
                GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR
            }
            NamedGenerationWeights::RollbackAfterRaisingTheFloor => {
                GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR
            }
            NamedGenerationWeights::TowardTheAllocationRecordWall => {
                GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL
            }
            NamedGenerationWeights::TowardTheUnitAreaWall => {
                GenerationWeights::TOWARD_THE_UNIT_AREA_WALL
            }
            NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => {
                GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES
            }
            NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => {
                GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP
            }
            NamedGenerationWeights::NewestSystemConfigurationSlotReadingZeros => {
                GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS
            }
            NamedGenerationWeights::SnapshotOperations => GenerationWeights::SNAPSHOT_OPERATIONS,
            NamedGenerationWeights::CompactionWhileMounted => {
                GenerationWeights::COMPACTION_WHILE_MOUNTED
            }
            NamedGenerationWeights::CompactionWhileMountedWithRollback => {
                GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK
            }
            NamedGenerationWeights::BroadWithCompactionWhileMounted => {
                GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED
            }
            NamedGenerationWeights::Truncation => GenerationWeights::TRUNCATION,
            NamedGenerationWeights::WritesByInodeNumber => {
                GenerationWeights::WRITES_BY_INODE_NUMBER
            }
            NamedGenerationWeights::SizeChangesOfInodesOtherThanTheFirst => {
                GenerationWeights::SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST
            }
            NamedGenerationWeights::PartialWrites => GenerationWeights::PARTIAL_WRITES,
        }
    }

    /// 复现行与复现入口的环境变量里这组比重的取值（`HistorySamplingVariable::Weights`）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            NamedGenerationWeights::Broad => "broad",
            NamedGenerationWeights::ReuseAfterRaisingTheFloor => "reuse",
            NamedGenerationWeights::RollbackAfterRaisingTheFloor => "rollback",
            NamedGenerationWeights::TowardTheAllocationRecordWall => "wall",
            NamedGenerationWeights::TowardTheUnitAreaWall => "unit-area-wall",
            NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => "several-leaves",
            NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => "orphans",
            NamedGenerationWeights::NewestSystemConfigurationSlotReadingZeros => {
                "newest-system-configuration-slot-zeros"
            }
            NamedGenerationWeights::SnapshotOperations => "snapshots",
            NamedGenerationWeights::CompactionWhileMounted => "compaction",
            NamedGenerationWeights::CompactionWhileMountedWithRollback => "compaction-rollback",
            NamedGenerationWeights::BroadWithCompactionWhileMounted => "broad-compaction",
            NamedGenerationWeights::Truncation => "truncation",
            NamedGenerationWeights::WritesByInodeNumber => "inode-writes",
            NamedGenerationWeights::SizeChangesOfInodesOtherThanTheFirst => {
                "size-changes-of-inodes-other-than-the-first"
            }
            NamedGenerationWeights::PartialWrites => "partial-writes",
        }
    }
}

/// 生成一段历史时各样东西的比重（每张表写份数；操作那三张按百分比写，和为 100）：起点两种各占几份，三种会话估计下各类操作各占几份。
/// 抽法是 [0, 份数之和) 里取一个数、按表的次序落到哪一格：起点那张写成 1 : 1 时就是第一版的「种子的第一个数对 2 取余」，
/// broad 那一组比重下每个种子生成的历史与第一版逐项相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationWeights {
    /// 报告里的名字。
    pub name: &'static str,
    /// 复现入口认这一组用的名字（[`NamedGenerationWeights`] 的哪一个成员）：每一组比重都要写，新加一组就得给它一个成员，
    /// 成员又由 [`NamedGenerationWeights::weights`] 的穷举 `match` 找回这一组，[`GenerationWeights::NAMED`] 照成员表生成。
    pub named: NamedGenerationWeights,
    pub starting_points: &'static [(HistoryStartingPoint, u64)],
    pub with_session_closed: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_without_file: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_with_file: &'static [(HistoryOperationKind, u64)],
    /// unlink 过第一个文件之后（[`ExpectedSession::OpenAfterTheFileWasDeleted`]）各类操作各占几份；`None` 的那几组 unlink 之后照旧按
    /// 「开着、没有文件」那张表抽、之后的估计也照旧，生成的历史与加「建 inode」之前逐项相同（那几组的取样点与钉的值不动）。
    pub with_session_open_after_the_file_was_deleted:
        Option<&'static [(HistoryOperationKind, u64)]>,
    pub rollback_targets: RollbackTargetDraw,
    /// 覆盖写的内容按什么抽：只有偏向删除多片下段叶那一组抽跨几个单元的，别的几组照第一版，生成的历史不变。
    pub overwrite_contents: OverwriteContentDraw,
}

impl GenerationWeights {
    /// 随机历史的默认批用的那一组（各类操作都抽）。关着的会话只抽挂载与冷启动；树表 0 条的版本上多抽第一个文件，不然从 mkfs 起的历史多半先推零单元发布、
    /// 再也接不上第一个文件；带文件的版本上零单元发布只会记「前提不满足」，少抽。起点两种各半。
    ///
    /// 崩溃恢复抛弃根（`CrashRecoveryAbandoningTheNewestRoot`，实七加）的份数从紧挨在它前面的可写挂载那一格里切出来（带文件 12 → 9 + 3，
    /// 树表 0 条 6 → 4 + 2），两张表的和照旧是 100：同一个种子抽到的数落在哪一格，只有落在切出去那几份里的从可写挂载换成它，
    /// 别的一格不动；它不再抽随机数、生成时对会话的估计与可写挂载相同，所以之后几步抽到的操作也不变。
    /// 关着的会话上不抽它：它要这个进程的会话交回最新那一版的记录（点名的单元）。
    /// unlink（三·2 第 8 条）同一个切法：带文件那张表里可写挂载那一格再切 3 份给它、紧挨在可写挂载之后（9 → 6 + 3），和照旧 100；
    /// 它也不抽随机数，落在切出去那几份里的从可写挂载换成它，之后几步按 unlink 之后的估计（有树、没有文件）抽。
    pub const BROAD: GenerationWeights = GenerationWeights {
        name: "各类操作都抽",
        named: NamedGenerationWeights::Broad,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 1),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 92),
            (HistoryOperationKind::ColdStartRecover, 8),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 60),
            (HistoryOperationKind::PublishWithoutUnits, 20),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                2,
            ),
            (HistoryOperationKind::RollBackWhileMounted, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 52),
            (HistoryOperationKind::RaiseRollbackFloor, 16),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::Unlink, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
            (HistoryOperationKind::RollBackWhileMounted, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 15),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
            (HistoryOperationKind::ColdStartRecover, 5),
            (HistoryOperationKind::PublishOverwrite, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 4),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：多从第一个文件起、多覆盖写、多抬 F、多可写挂载，
    /// 少回退与冷启动。罩住别的已回收记录要：抬 F 回收一批 1 槽的固定点单元，重开之后上一次挂载的聚簇段不再挡用户数据，
    /// 下一个 2 槽的数据单元落在两条相邻的已回收记录上。
    /// unlink 从带文件那张表的可写挂载那一格切 3 份（18 → 15 + 3，紧挨在它之后；切法同 [`GenerationWeights::BROAD`]）：
    /// 小盘上删除类发布不判式子、落点那一判照判（D28（挂载期承诺量） 已定项 1） Q-T「甲」拿它去撞落点那一判，种子照旧。
    pub const REUSE_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后的复用（第 121 行那一类的取样点）",
        named: NamedGenerationWeights::ReuseAfterRaisingTheFloor,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 55),
            (HistoryOperationKind::RaiseRollbackFloor, 22),
            (HistoryOperationKind::CloseAndMountWritable, 15),
            (HistoryOperationKind::Unlink, 3),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 25),
            (HistoryOperationKind::CloseAndMountWritable, 20),
            (HistoryOperationKind::RollBackWhileMounted, 5),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 2),
            (HistoryOperationKind::PublishFirstFile, 2),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 理想模型那一格（B2：回退到 txg = F_生效 的根被拒）的专门取样点：多从第一个文件起、多覆盖写（攒非空根，抬 F 的上限才抬得动）、
    /// 多抬 F、多回退，少冷启动；回退的目标一半取候选集的下沿（`RollbackTargetDraw::FloorRootHalfTheTime`）。broad 那组比重与抽法下
    /// 回退落到 F 那条根上一次都没有（数过：默认批 96 段、第 121 行取样点 48 段都是 0 次；只换比重、不换抽法，192 段里 2 次）。
    pub const ROLLBACK_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后回退（B2 那一格的取样点）",
        named: NamedGenerationWeights::RollbackAfterRaisingTheFloor,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 3),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 25),
            (HistoryOperationKind::RollBackWhileMounted, 25),
            (HistoryOperationKind::CloseAndMountWritable, 7),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 原分配记录墙那一格的取样点：一律从第一个文件起，带文件的版本上多半覆盖写，
    /// 夹着可写挂载、抬 F（回收之后复用改写记录，条数涨得慢）与少量回退，把分配记录攒过 812 条。分配记录树按绝对槽号按位置寻址之后
    /// （D8（核心索引结构） 已定项 14）那道墙拆了，这一段看的是越过 812 条之后多叶的树每一步都判绿、模型对得上。每一步的池级 checker 要判
    /// （第 3 条：一步都不判，攻方两条只有 checker 看得见的变异在它上面一段都不红）：那一遍归给定状态引擎两段，
    /// 每一步的镜像经钩子交出去。
    pub const TOWARD_THE_ALLOCATION_RECORD_WALL: GenerationWeights = GenerationWeights {
        name: "越过原分配记录墙（812 条那一格的取样点）",
        named: NamedGenerationWeights::TowardTheAllocationRecordWall,
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[(HistoryOperationKind::CloseAndMountWritable, 100)],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 80),
            (HistoryOperationKind::CloseAndMountWritable, 12),
            (HistoryOperationKind::RaiseRollbackFloor, 5),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 单元区墙那一格的取样点，配 `HistoryDeviceWidth::UnitAreaOf256Slots` 跑：
    /// 一律从第一个文件起，带文件的版本上绝大多数是覆盖写（每次每盘占 10 槽），夹着少量可写挂载与抬 F 与回退。
    /// 这组比重在 256 槽的小盘上只走得到用户数据那一处（覆盖写的数据单元）的落点拒绝：提交内生块能回落到任一空槽，
    /// 每块盘上一个空槽都不剩才拒，那一处由写死的用例钉（`commit_generated_units_fall_back_when_the_segment_runs_out.rs`）。
    /// unlink 从带文件那张表的可写挂载那一格切 3 份（6 → 3 + 3，紧挨在它之后；切法与 Q-T「甲」同 [`GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`]）。
    pub const TOWARD_THE_UNIT_AREA_WALL: GenerationWeights = GenerationWeights {
        name: "逼近单元区墙（小盘上落点拒绝那一格的取样点）",
        named: NamedGenerationWeights::TowardTheUnitAreaWall,
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 88),
            (HistoryOperationKind::CloseAndMountWritable, 3),
            (HistoryOperationKind::Unlink, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 4),
            (HistoryOperationKind::RollBackWhileMounted, 1),
            (HistoryOperationKind::ColdStartRecover, 1),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 删除多于一片下段叶的文件那一格（代码轮 U13：模型罩多个数据单元的文件、随机历史里出现 ≥ 2 片下段叶的删除批；
    /// N3「走空多步」）的取样点，配两块 4 GiB 的盘跑：多从第一个文件起；带文件的版本上覆盖写的内容另抽跨几个单元、
    /// 走顺序写入口的（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]，多半两片下段叶），夹着很多 unlink（一次 unlink
    /// = 意图那次 + 从尾部每批 144 个单元 + 最后一批）；unlink 之后只抽建 inode、回退、覆盖写与 unlink（不抽挂载、抬 F、冷启动：
    /// 它们写的根会把意图那一版在根环里的名次往后推），回退一半取第四新的根（建过一个 inode 之后落在意图那一版或第一个删除批那一版上，
    /// 号回到队列），再建 inode、覆盖写或 unlink（会话先把队列走空，多片的文件走空多步）。broad 那组比重不动：它的种子钉着的取样点与
    /// 崩溃那一侧、实验装置的复现都按它生成。
    pub const DELETING_FILES_OF_SEVERAL_LOWER_LEAVES: GenerationWeights = GenerationWeights {
        name: "偏向删除多于一片下段叶的文件（U13 那一格的取样点）",
        named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 45),
            (HistoryOperationKind::Unlink, 25),
            (HistoryOperationKind::RaiseRollbackFloor, 8),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::RollBackWhileMounted, 8),
            (HistoryOperationKind::ColdStartRecover, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 40),
            (HistoryOperationKind::RollBackWhileMounted, 40),
            (HistoryOperationKind::PublishOverwrite, 10),
            (HistoryOperationKind::Unlink, 10),
        ]),
        rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 实例表清扫抹头那一段（D18（块里携带什么信息） 已定项 11「回收」一条）的取样点：一律从第一个文件起，带文件的版本上
    /// 偶尔一次覆盖写掉电在第一个 journal 记录写之前、进程崩掉再挂（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]，
    /// 留下那个实例的孤儿），其余绝大多数是覆盖写（每次一条根，把孤儿那个实例的根从 24 槽的根环里轮出去），夹着少量可写挂载；
    /// 不抽回退、抬 F、冷启动与 unlink。别的几组比重不抽这一步：它们的种子钉着的取样点照旧。
    pub const LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP: GenerationWeights = GenerationWeights {
        name: "偏向留下孤儿实例（清扫抹头那一段的取样点）",
        named: NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep,
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[(HistoryOperationKind::CloseAndMountWritable, 100)],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 88),
            (
                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
                8,
            ),
            (HistoryOperationKind::CloseAndMountWritable, 4),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 崩溃注入读故障那一支的取样点（C696（崩溃注入的读故障一支在随机历史里从没触发），设计调查的候选「丁」）：与
    /// [`GenerationWeights::BROAD`] 逐格相同，只把崩溃恢复抛弃根那几格（份数照旧）换成「只有第一块盘最新那一槽系统配置读回全 0 时的可写挂载」
    /// （[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros`]）；带文件那张表与 unlink 之后那张表里
    /// 那一格各再切 1 份给紧挨在它之后的「每块盘最新那一槽系统配置读回全 0 时的可写挂载」
    /// （[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`]，3 → 2 + 1）：那一步照乙入判不出
    /// （C971（多处读坏时可写挂载写出倒挂的新实例根））拒可写、一个写都不发，模型照条款答拒，份数小是因为它不在读故障下发写、
    /// 占多了这一组取样读故障一支的步就少了。三种步都不抽随机数、生成时对会话的估计都与可写挂载相同，所以同一个种子在这组比重下生成的历史
    /// 与 broad 那一组逐步相同、只有那几步换了种类。已有各组的份数表里没有这两种步，
    /// 它们生成的历史不变（`tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs` 钉）。
    pub const NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS: GenerationWeights =
        GenerationWeights {
            name: "最新那一槽系统配置读回全 0 的可写挂载（崩溃注入读故障那一支的取样点）",
            named: NamedGenerationWeights::NewestSystemConfigurationSlotReadingZeros,
            starting_points: &[
                (HistoryStartingPoint::AfterMakeFilesystem, 1),
                (HistoryStartingPoint::AfterFirstFile, 1),
            ],
            with_session_closed: &[
                (HistoryOperationKind::CloseAndMountWritable, 92),
                (HistoryOperationKind::ColdStartRecover, 8),
            ],
            with_session_open_without_file: &[
                (HistoryOperationKind::PublishFirstFile, 60),
                (HistoryOperationKind::PublishWithoutUnits, 20),
                (HistoryOperationKind::CloseAndMountWritable, 4),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
                    2,
                ),
                (HistoryOperationKind::RollBackWhileMounted, 4),
                (HistoryOperationKind::ColdStartRecover, 4),
                (HistoryOperationKind::PublishOverwrite, 3),
                (HistoryOperationKind::RaiseRollbackFloor, 3),
            ],
            with_session_open_with_file: &[
                (HistoryOperationKind::PublishOverwrite, 52),
                (HistoryOperationKind::RaiseRollbackFloor, 16),
                (HistoryOperationKind::CloseAndMountWritable, 6),
                (HistoryOperationKind::Unlink, 3),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
                    2,
                ),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                    1,
                ),
                (HistoryOperationKind::RollBackWhileMounted, 10),
                (HistoryOperationKind::ColdStartRecover, 4),
                (HistoryOperationKind::PublishFirstFile, 4),
                (HistoryOperationKind::PublishWithoutUnits, 2),
            ],
            with_session_open_after_the_file_was_deleted: Some(&[
                (HistoryOperationKind::CreateInode, 40),
                (HistoryOperationKind::RaiseRollbackFloor, 15),
                (HistoryOperationKind::RollBackWhileMounted, 15),
                (HistoryOperationKind::CloseAndMountWritable, 10),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
                    2,
                ),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                    1,
                ),
                (HistoryOperationKind::ColdStartRecover, 5),
                (HistoryOperationKind::PublishOverwrite, 4),
                (HistoryOperationKind::PublishFirstFile, 4),
                (HistoryOperationKind::PublishWithoutUnits, 4),
            ]),
            rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
            overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
        };

    /// 建快照、读快照、删快照那一格（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7：`HistoryOperation` 加三种，
    /// 新操作进自己的权重组，已有各组生成的历史不变）的取样点，配两块 4 GiB 的盘跑：多从第一个文件起；带文件的版本上覆盖写的内容另抽跨几个单元、
    /// 走顺序写入口的（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]，多半两片下段叶：快照下覆盖写钉住整段下段、删快照分几批放），
    /// 夹着建快照、读快照、删快照，与 unlink（快照下删文件，删除批换下的单元钉进 deadlist）、回退（带快照的两版上要求拒，判决 S8）、
    /// 可写挂载、抬 F、冷启动与崩溃恢复抛弃根；unlink 之后只抽建 inode、三种快照操作、回退与可写挂载。
    pub const SNAPSHOT_OPERATIONS: GenerationWeights = GenerationWeights {
        name: "带建快照、读快照、删快照的历史（判决 m4-snapshot-r1 第三节 S7 那一格的取样点）",
        named: NamedGenerationWeights::SnapshotOperations,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::PublishWithoutUnits, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::CreateSnapshot, 4),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 28),
            (HistoryOperationKind::CreateSnapshot, 16),
            (HistoryOperationKind::ReadSnapshot, 10),
            (HistoryOperationKind::DestroySnapshot, 12),
            (HistoryOperationKind::Unlink, 5),
            (HistoryOperationKind::RollBackWhileMounted, 9),
            (HistoryOperationKind::RaiseRollbackFloor, 6),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::ColdStartRecover, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 25),
            (HistoryOperationKind::CreateSnapshot, 15),
            (HistoryOperationKind::DestroySnapshot, 15),
            (HistoryOperationKind::ReadSnapshot, 10),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::RaiseRollbackFloor, 5),
            (HistoryOperationKind::PublishOverwrite, 5),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 挂着时整理那一格（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7：`HistoryOperation` 加 `CompactWhileMounted`，
    /// 新操作进自己的权重组）的取样点，配两块 4 GiB 的盘跑：多从第一个文件起；带文件的版本上覆盖写的内容另抽跨几个单元的
    /// （[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]：数据单元铺满几段，强制选段落在装着用户数据的段上），夹着整理、建快照、读快照、
    /// 删快照（被快照钉住的单元照搬）、unlink、可写挂载与冷启动；unlink 之后抽建 inode、整理与三种快照操作。不抽回退、抬 F 与崩溃恢复抛弃根：
    /// 它们按根环里的位置点名，整理多发的那几条根会让「同一段历史去掉整理」那一份点到别的根上，两份读出的内容不再能逐字比
    /// （`tests/compaction_while_mounted_in_random_histories.rs` 拿去掉整理的那一份比读出的内容）。整理之后回退那一格另一组比重抽
    /// （[`GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`]）。
    pub const COMPACTION_WHILE_MOUNTED: GenerationWeights = GenerationWeights {
        name: "挂着时整理的历史（判决 m4-gc-r1 第三节 G7 那一格的取样点）",
        named: NamedGenerationWeights::CompactionWhileMounted,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::CompactWhileMounted, 8),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 30),
            (HistoryOperationKind::CompactWhileMounted, 25),
            (HistoryOperationKind::CreateSnapshot, 10),
            (HistoryOperationKind::ReadSnapshot, 8),
            (HistoryOperationKind::DestroySnapshot, 8),
            (HistoryOperationKind::Unlink, 5),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 25),
            (HistoryOperationKind::CompactWhileMounted, 25),
            (HistoryOperationKind::CreateSnapshot, 10),
            (HistoryOperationKind::DestroySnapshot, 10),
            (HistoryOperationKind::ReadSnapshot, 10),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishOverwrite, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 整理之后回退那一格（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F1：整理搬过单元之后回退，回退那一版的中央映射
    /// 要取目标那一版的位置；整理那一组比重不抽回退，随机历史撞不到）的取样点，配两块 4 GiB 的盘跑：在整理那一组的基础上，带文件的版本与
    /// unlink 之后各切出一份给回退与抬 F（回退按根环里的位置点名，整理多发的那几条根让它多半退到整理半路或整理之前的根上），快照操作压少
    /// （第一版有快照就拒回退，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S8），unlink 之后不抽建快照（文件集合为空时建快照
    /// 那一格归快照那一组比重）。只交给按模型逐版比的用例，不进「去掉整理那一份」的比法。
    /// 新加的一组，已有各组生成的历史不变。
    pub const COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK: GenerationWeights = GenerationWeights {
        name: "挂着时整理夹着回退的历史（判决 m4-segment-code-r1 第三节 F1 那一格的取样点）",
        named: NamedGenerationWeights::CompactionWhileMountedWithRollback,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::CompactWhileMounted, 8),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 28),
            (HistoryOperationKind::CompactWhileMounted, 25),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::RaiseRollbackFloor, 4),
            (HistoryOperationKind::CreateSnapshot, 4),
            (HistoryOperationKind::ReadSnapshot, 3),
            (HistoryOperationKind::DestroySnapshot, 4),
            (HistoryOperationKind::Unlink, 3),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 30),
            (HistoryOperationKind::CompactWhileMounted, 25),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::DestroySnapshot, 5),
            (HistoryOperationKind::ReadSnapshot, 5),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishOverwrite, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 故障注入里瞬时读错与整理一起出现那一格（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J5：故障注入用的
    /// [`GenerationWeights::BROAD`] 不抽整理，整理两条「核不过就照抄意图」的退路发出的那一版与模型的冲突在对拍历史里一次都走不到）的取样点：
    /// 起点与关着的会话那张表同 broad 那一组；开着的三张表各从份数最多的那一格切出一份给挂着时整理（[`HistoryOperation::CompactWhileMounted`]），
    /// 紧挨在切出它的那一格之后，和照旧 100：没有文件那张从第一个文件切（60 → 52 + 8），带文件那张从覆盖写切（52 → 40 + 12），
    /// unlink 之后那张从建 inode 切（40 → 30 + 10）。整理那一步照执行器的写法装只供测试的整理开关（强制批大小与选段），故障注入照常摆注入点：
    /// 落在整理那一步上的读错就是瞬时读错撞上整理。新加的一组，已有各组生成的历史不变；故障注入的默认批照旧用 broad 那一组，这一组经故障注入入口的
    /// 比重变量点名跑（`broad-compaction`）。
    /// 截断（[`HistoryOperation::Truncate`]，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8「另进带整理的那一组」）同一个切法：
    /// 带文件那张从覆盖写再切（40 → 32 + 8），unlink 之后那张从建 inode 再切（30 → 24 + 6），紧挨在切出它的那一格之后：故障注入入口点名这一组时
    /// 读错、写错、说谎的设备落得到截断的意图与各批上。
    pub const BROAD_WITH_COMPACTION_WHILE_MOUNTED: GenerationWeights = GenerationWeights {
        name: "各类操作都抽、夹着挂着时整理（判决 m4-segment-code-r3 第三节 J5 那一格的取样点，故障注入用）",
        named: NamedGenerationWeights::BroadWithCompactionWhileMounted,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 1),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 92),
            (HistoryOperationKind::ColdStartRecover, 8),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 52),
            (HistoryOperationKind::CompactWhileMounted, 8),
            (HistoryOperationKind::PublishWithoutUnits, 20),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                2,
            ),
            (HistoryOperationKind::RollBackWhileMounted, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 32),
            (HistoryOperationKind::Truncate, 8),
            (HistoryOperationKind::CompactWhileMounted, 12),
            (HistoryOperationKind::RaiseRollbackFloor, 16),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            // unlink 一格份数与次序同 broad 那一组；与前后两格各隔一行注释，`crates/mutations.tsv` 里改 broad 那一组这几格的变异原文才只在 broad 那一组里命中一次
            (HistoryOperationKind::Unlink, 3),
            // 崩溃恢复抛弃根一格份数与次序同 broad 那一组；与 unlink 那一格隔开，理由写在 unlink 那一格的注释里
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
            (HistoryOperationKind::RollBackWhileMounted, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 24),
            (HistoryOperationKind::Truncate, 6),
            (HistoryOperationKind::CompactWhileMounted, 10),
            (HistoryOperationKind::RaiseRollbackFloor, 15),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
            (HistoryOperationKind::ColdStartRecover, 5),
            (HistoryOperationKind::PublishOverwrite, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 4),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 按 inode 号写数据的取样点（里程碑「删除与多次 COW」一·1；C445（模型对拍的取样点不随写路径增长）：新入口要有对拍跑得到它的取样点）：
    /// 多从第一个文件起，第一个文件活着时多建 inode（一次至多 160 个，号越过 extent 树上段一片叶罩的 143 个）、多按 inode 号写（[`WriteOfAnInodeChoice`]：
    /// 三分之一取最大的号，上段长高与写的号落在第 0 片叶之外两形都落得到），也写第一个文件、回退、抬 F、可写挂载与冷启动（别的文件跨挂载读回）；
    /// unlink 按号选（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4，执行器选，[`inode_the_unlink_names`]：第一个文件、
    /// 带数据的别的文件、size 0 的别的文件都删得到；inode 树多于一片叶容器时不删，[`MissingPrecondition::InodeTreeSpansMoreThanOneLeafContainer`]），
    /// 带文件那张表与 unlink 之后那张表都抽（一次抽到 unlink 之后生成就按 unlink 之后那张表抽，删的可能是别的号、第一个文件还在，那张表照样接着删），
    /// 建 inode 一半只建几个（删得到删空、树留在一片叶容器里），一半至多 160 个；删完第一个文件之后按 inode 号写删之前就有的照写、写删完之后建的必拒
    /// （K-U5 的 β 时序级读法）。快照、截断、整理不抽：模型的那几臂只按第一个文件写。
    /// 只进它自己这一组，已有各组生成的历史不变。
    pub const WRITES_BY_INODE_NUMBER: GenerationWeights = GenerationWeights {
        name: "按 inode 号写数据（里程碑「删除与多次 COW」一·1 的取样点）",
        named: NamedGenerationWeights::WritesByInodeNumber,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 4),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 90),
            (HistoryOperationKind::ColdStartRecover, 10),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 75),
            (HistoryOperationKind::PublishWithoutUnits, 10),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::WriteOfAnInode, 38),
            (HistoryOperationKind::CreateInodes, 18),
            (HistoryOperationKind::PublishOverwrite, 8),
            (HistoryOperationKind::Unlink, 14),
            (HistoryOperationKind::RollBackWhileMounted, 8),
            (HistoryOperationKind::RaiseRollbackFloor, 6),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::ColdStartRecover, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInodes, 24),
            (HistoryOperationKind::WriteOfAnInode, 35),
            (HistoryOperationKind::Unlink, 26),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::RaiseRollbackFloor, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 别的 inode 带数据单元时改 size 那一格（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5 的验收：随机历史点名
    /// 别的 inode 截断、变长、快照下截断、整理时截断）的取样点：多从第一个文件起；带文件的版本上在第一个文件旁边建 inode、按 inode 号写（内容另抽跨几个
    /// 单元的，[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]：缩短要不止一批）、截断（第一个文件活着而树里有别的 inode 时一半点名别的 inode，
    /// `apply_truncate`）占大半，夹着建快照、删快照、挂着时整理、覆盖写、unlink、回退、可写挂载与冷启动；unlink 第一个文件之后接着按号写删之前就有的
    /// inode、截断与建 inode。新加的一组，已有各组生成的历史不变。
    pub const SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST: GenerationWeights = GenerationWeights {
        name:
            "别的 inode 带数据单元时改 size（判决 m4-multi-inode-size-r1 第二节 S5 那一格的取样点）",
        named: NamedGenerationWeights::SizeChangesOfInodesOtherThanTheFirst,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::CreateInodes, 12),
            (HistoryOperationKind::WriteOfAnInode, 22),
            (HistoryOperationKind::Truncate, 30),
            (HistoryOperationKind::CreateSnapshot, 6),
            (HistoryOperationKind::DestroySnapshot, 4),
            (HistoryOperationKind::CompactWhileMounted, 8),
            (HistoryOperationKind::PublishOverwrite, 4),
            (HistoryOperationKind::Unlink, 3),
            (HistoryOperationKind::RollBackWhileMounted, 3),
            (HistoryOperationKind::CloseAndMountWritable, 5),
            (HistoryOperationKind::ColdStartRecover, 3),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::WriteOfAnInode, 25),
            (HistoryOperationKind::Truncate, 30),
            (HistoryOperationKind::CreateInodes, 15),
            (HistoryOperationKind::Unlink, 10),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::RollBackWhileMounted, 10),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 按偏移写的取样点（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节；C445（模型对拍的取样点不随写路径增长）：新入口要有对拍
    /// 跑得到它的取样点）：多从第一个文件起；带文件的版本上按偏移写占四成（[`PartialWriteChoice`]：五格偏移各抽到），覆盖写的内容另抽跨几个单元的
    /// （[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]：「已有单元中间」「盖满一个单元」与「写到 o 时 o 之后一起重写」要多单元的文件），截断里的
    /// 变长留洞（「写进洞里」那一格只在有洞的文件上落得到），夹着在第一个文件旁边建 inode（按偏移写别的文件，R6）、建快照与删快照（快照下按偏移写，
    /// 换下的单元进 deadlist）、回退、抬 F、unlink、可写挂载与冷启动；unlink 之后那张表按偏移写删之前就有的 inode、截断与建 inode。
    /// 新加的一组，已有各组生成的历史不变。
    pub const PARTIAL_WRITES: GenerationWeights = GenerationWeights {
        name: "按偏移写（判决 m4-partial-write-r1 第二节那五格的取样点）",
        named: NamedGenerationWeights::PartialWrites,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::WriteAtOffset, 40),
            (HistoryOperationKind::PublishOverwrite, 12),
            (HistoryOperationKind::Truncate, 18),
            (HistoryOperationKind::CreateInodes, 3),
            (HistoryOperationKind::CreateSnapshot, 4),
            (HistoryOperationKind::DestroySnapshot, 3),
            (HistoryOperationKind::RollBackWhileMounted, 5),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
            (HistoryOperationKind::Unlink, 2),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::ColdStartRecover, 4),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::WriteAtOffset, 35),
            (HistoryOperationKind::CreateInodes, 25),
            (HistoryOperationKind::Truncate, 15),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 截断那一格（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8：`HistoryOperation` 加 `Truncate`，比重进自己那一组）的
    /// 取样点，配两块 4 GiB 的盘跑：多从第一个文件起；带文件的版本上截断占三成多（缩到 0、单元边界、单元中间、单元内与跨单元变长、变长后截回洞里、
    /// 改成现在的 size），覆盖写的内容另抽跨几个单元的（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]：多半两片下段叶，缩短要不止一批），
    /// 夹着建快照、删快照（快照下截断，K4）、unlink、回退、抬 F、可写挂载、冷启动与崩溃恢复抛弃根；回退一半取第四新的根（截断的意图与各批
    /// 一次写出好几条根，退到截断半路那一版，之后的改动先把截断走完，K1）。unlink 之后抽建 inode、截断（点名删完之后建的 inode，第一版拒）、
    /// 回退与可写挂载。新加的一组，已有各组生成的历史不变。
    pub const TRUNCATION: GenerationWeights = GenerationWeights {
        name: "带截断的历史（判决 m4-trunc-r1 第三节 K8 那一格的取样点）",
        named: NamedGenerationWeights::Truncation,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 28),
            (HistoryOperationKind::Truncate, 35),
            (HistoryOperationKind::Unlink, 4),
            (HistoryOperationKind::CreateSnapshot, 4),
            (HistoryOperationKind::DestroySnapshot, 3),
            (HistoryOperationKind::RollBackWhileMounted, 9),
            (HistoryOperationKind::RaiseRollbackFloor, 4),
            (HistoryOperationKind::CloseAndMountWritable, 7),
            (HistoryOperationKind::ColdStartRecover, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 35),
            (HistoryOperationKind::Truncate, 25),
            (HistoryOperationKind::RollBackWhileMounted, 25),
            (HistoryOperationKind::PublishOverwrite, 5),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ]),
        rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 复现入口认得的每一组比重：按 `environment_value` 找回（[`HistorySampling::from_environment`]）。
    /// 照 [`NamedGenerationWeights::ALL`] 逐个经 [`NamedGenerationWeights::weights`] 生成，不手抄常量名：
    /// 新加一组比重要一个新成员，成员不在 `weights` 的穷举 `match` 里补一臂就编不过（代码三方 test-script-align-code-r1 判决 K4）。
    pub const NAMED: [GenerationWeights; NamedGenerationWeights::ALL.len()] = {
        let mut named = [GenerationWeights::BROAD; NamedGenerationWeights::ALL.len()];
        let mut position = 0;
        while position < named.len() {
            named[position] = NamedGenerationWeights::ALL[position].weights();
            position += 1;
        }
        named
    };

    /// 复现行与复现入口的环境变量里这组比重的取值（`HistorySamplingVariable::Weights`）。
    #[must_use]
    pub const fn environment_value(&self) -> &'static str {
        self.named.environment_value()
    }

    fn for_session(&self, expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
        match expected {
            ExpectedSession::Closed => self.with_session_closed,
            ExpectedSession::OpenWithoutFile => self.with_session_open_without_file,
            ExpectedSession::OpenWithFile => self.with_session_open_with_file,
            ExpectedSession::OpenAfterTheFileWasDeleted => self
                .with_session_open_after_the_file_was_deleted
                .expect("只在这一组比重写了 unlink 之后那张表时，生成才进这一估计（`expected_session_after` 的 unlink 那一臂）"),
        }
    }
}

fn draw_weighted<Choice: Copy>(
    source: &mut SeededRandomSource,
    weights: &[(Choice, u64)],
) -> Choice {
    let total: u64 = weights.iter().map(|(_, weight)| weight).sum();
    let mut remaining = source.below(total);
    for (choice, weight) in weights {
        if remaining < *weight {
            return *choice;
        }
        remaining -= weight;
    }
    unreachable!("remaining < total = 各比重之和，循环里一定有一项接住")
}

fn draw_content(source: &mut SeededRandomSource) -> ContentChoice {
    let selector = source.next_word();
    let length = draw_weighted(
        source,
        &[
            (ContentLength::Empty, 10),
            (ContentLength::InsideOneDataUnit { selector }, 60),
            (ContentLength::OneByteBelowDataUnitPayloadCapacity, 8),
            (ContentLength::ExactlyDataUnitPayloadCapacity, 8),
            (ContentLength::OneByteAboveDataUnitPayloadCapacity, 7),
            (ContentLength::ExactlyDataUnitBytes, 7),
        ],
    );
    ContentChoice {
        length,
        fill_seed: source.next_word(),
    }
}

/// 跨几个数据单元的内容（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]）：与 `draw_content` 抽的随机数一样多（选择子、
/// 长度种类、填充种子各一个），五分之一在一个单元以内（走覆盖写），其余走顺序写——一片下段叶以内的几个单元（2–16 个）、两片
/// （145–152 个单元，删除要一个删除批加最后一批）、三片（289–292 个单元，两个删除批加最后一批）。单元数取得窄是为了每一步的池级 checker 读得快。
fn draw_content_spanning_several_data_units(source: &mut SeededRandomSource) -> ContentChoice {
    let selector = source.next_word();
    let length = draw_weighted(
        source,
        &[
            (ContentLength::InsideOneDataUnit { selector }, 20),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 2 + selector % 15,
                    selector,
                },
                15,
            ),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 8,
                    selector,
                },
                55,
            ),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 4,
                    selector,
                },
                10,
            ),
        ],
    );
    ContentChoice {
        length,
        fill_seed: source.next_word(),
    }
}

/// 按 inode 号写的内容（[`HistoryOperation::WriteOfAnInode`]）：与 `draw_content` 抽的随机数一样多（选择子、长度种类、填充种子各一个）。
/// 一半在一个数据单元以内或刚越过（走覆盖写入口，刚越过的必拒），其余走顺序写——一片下段叶以内的几个单元（2–16 个）、两片（145–148 个单元）。
fn draw_content_of_a_write_by_inode_number(source: &mut SeededRandomSource) -> ContentChoice {
    let selector = source.next_word();
    let length = draw_weighted(
        source,
        &[
            (ContentLength::Empty, 6),
            (ContentLength::InsideOneDataUnit { selector }, 34),
            (ContentLength::ExactlyDataUnitPayloadCapacity, 5),
            (ContentLength::OneByteAboveDataUnitPayloadCapacity, 5),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 2 + selector % 15,
                    selector,
                },
                40,
            ),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 4,
                    selector,
                },
                10,
            ),
        ],
    );
    ContentChoice {
        length,
        fill_seed: source.next_word(),
    }
}

fn draw_operation(
    source: &mut SeededRandomSource,
    kind: HistoryOperationKind,
    rollback_targets: RollbackTargetDraw,
    overwrite_contents: OverwriteContentDraw,
) -> HistoryOperation {
    match kind {
        HistoryOperationKind::PublishFirstFile => {
            HistoryOperation::PublishFirstFile(draw_content(source))
        }
        HistoryOperationKind::PublishOverwrite => {
            HistoryOperation::PublishOverwrite(match overwrite_contents {
                OverwriteContentDraw::WithinOneDataUnitOrJustBeyond => draw_content(source),
                OverwriteContentDraw::AlsoSpanningSeveralDataUnits => {
                    draw_content_spanning_several_data_units(source)
                }
            })
        }
        HistoryOperationKind::PublishWithoutUnits => HistoryOperation::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable => HistoryOperation::CloseAndMountWritable,
        HistoryOperationKind::RollBackWhileMounted => {
            match rollback_targets {
                RollbackTargetDraw::RecentUniformOrBeyond => {}
                RollbackTargetDraw::FloorRootHalfTheTime => {
                    if source.below(2) == 0 {
                        return HistoryOperation::RollBackWhileMounted(
                            RollbackTargetChoice::RingRootAtTheNewestFloor,
                        );
                    }
                }
                RollbackTargetDraw::FourthNewestRootHalfTheTime => {
                    if source.below(2) == 0 {
                        return HistoryOperation::RollBackWhileMounted(
                            RollbackTargetChoice::RingRoot {
                                index_from_newest: 3,
                            },
                        );
                    }
                }
            }
            // 一半取最近的几条（多半在候选集里），一半在整个根环里均匀取（被抛弃的、F 之下的、暖机那两条树表 0 条的都落得到），
            // 余下的取根环之外。
            let target = match source.below(5) {
                0 | 1 => RollbackTargetChoice::RingRoot {
                    index_from_newest: source.below(4),
                },
                2 | 3 => RollbackTargetChoice::RingRoot {
                    index_from_newest: source.next_word(),
                },
                _ => RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: source.below(3),
                },
            };
            HistoryOperation::RollBackWhileMounted(target)
        }
        HistoryOperationKind::RaiseRollbackFloor => {
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: source.next_word(),
            })
        }
        HistoryOperationKind::ColdStartRecover => HistoryOperation::ColdStartRecover,
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
        }
        HistoryOperationKind::Unlink => HistoryOperation::Unlink,
        HistoryOperationKind::CreateInode => HistoryOperation::CreateInode,
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
        }
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => {
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
        }
        HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
                match overwrite_contents {
                    OverwriteContentDraw::WithinOneDataUnitOrJustBeyond => draw_content(source),
                    OverwriteContentDraw::AlsoSpanningSeveralDataUnits => {
                        draw_content_spanning_several_data_units(source)
                    }
                },
            )
        }
        HistoryOperationKind::CreateSnapshot => HistoryOperation::CreateSnapshot,
        HistoryOperationKind::ReadSnapshot => HistoryOperation::ReadSnapshot(SnapshotChoice {
            index_among_the_snapshots: source.next_word(),
        }),
        HistoryOperationKind::DestroySnapshot => HistoryOperation::DestroySnapshot(SnapshotChoice {
            index_among_the_snapshots: source.next_word(),
        }),
        HistoryOperationKind::CompactWhileMounted => {
            let batch_units = 1 + source.below(COMPACTION_BATCH_UNITS_DRAWN_AT_MOST);
            let region_selector = source.next_word();
            // 三分之一推够几批就崩掉、可写挂载（批间崩溃）；其余推到停。
            let ending = if source.below(3) == 0 {
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable {
                    batches: 1 + source.below(COMPACTION_BATCHES_BEFORE_THE_CRASH_DRAWN_AT_MOST),
                }
            } else {
                CompactionRunEnding::RunsUntilItStops
            };
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units,
                region_selector,
                ending,
            })
        }
        HistoryOperationKind::Truncate => {
            let target = draw_weighted(
                source,
                &[
                    (TruncationTarget::ToZero, 15),
                    (TruncationTarget::ToADataUnitBoundary, 15),
                    (TruncationTarget::IntoTheMiddleOfADataUnit, 20),
                    (TruncationTarget::GrowingWithinTheLastDataUnit, 15),
                    (TruncationTarget::GrowingAcrossDataUnits, 15),
                    (TruncationTarget::BackIntoTheHole, 15),
                    (TruncationTarget::ToTheCurrentSize, 5),
                ],
            );
            HistoryOperation::Truncate(TruncationChoice {
                target,
                selector: source.next_word(),
            })
        }
        // 一半只建几个（删得到删空、树留在一片叶容器里，unlink 才删得到），一半至多 160 个（号越过上段第 0 片叶）。
        HistoryOperationKind::CreateInodes => {
            let drawn_at_most = if source.below(2) == 0 {
                INODES_CREATED_AT_ONCE_IN_A_SMALL_DRAW_AT_MOST
            } else {
                INODES_CREATED_AT_ONCE_DRAWN_AT_MOST
            };
            HistoryOperation::CreateInodes(InodeCountChoice {
                count_drawn: 1 + source.below(drawn_at_most),
            })
        }
        HistoryOperationKind::WriteOfAnInode => {
            let inode_selector = source.next_word();
            HistoryOperation::WriteOfAnInode(WriteOfAnInodeChoice {
                inode_selector,
                content: draw_content_of_a_write_by_inode_number(source),
            })
        }
        // 五格各占一份（那五格）。
        HistoryOperationKind::WriteAtOffset => {
            let inode_selector = source.next_word();
            let cell = draw_weighted(
                source,
                &[
                    (PartialWriteOffsetCell::InsideAnExistingDataUnit, 1),
                    (PartialWriteOffsetCell::CoveringAWholeExistingDataUnit, 1),
                    (PartialWriteOffsetCell::AcrossTheEnd, 1),
                    (PartialWriteOffsetCell::PastTheEndLeavingAGap, 1),
                    (PartialWriteOffsetCell::IntoTheHole, 1),
                ],
            );
            HistoryOperation::WriteAtOffset(PartialWriteChoice {
                inode_selector,
                cell,
                selector: source.next_word(),
                fill_seed: source.next_word(),
            })
        }
    }
}

/// 整理一批至多搬几个单元，生成时在 1 到这么多里抽：一段 64 槽至多 32 个数据单元，一批取得小，一段要推好几批，批间崩溃才落得在意图半路。
const COMPACTION_BATCH_UNITS_DRAWN_AT_MOST: u64 = 3;

/// 批间崩溃之前至多推几批，生成时在 1 到这么多里抽。
const COMPACTION_BATCHES_BEFORE_THE_CRASH_DRAWN_AT_MOST: u64 = 3;

/// 生成时对「这段历史的第一个文件」的估计（只调比重、不进执行）：还没写过、写过、unlink 过之后一直没有（只在比重写了 unlink 之后那张表时进）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedFirstFile {
    NotWrittenYet,
    Written,
    DeletedAndFollowedByItsOwnTable,
}

fn expected_session_after(
    expected: ExpectedSession,
    first_file: ExpectedFirstFile,
    deletion_is_followed_by_its_own_table: bool,
    kind: HistoryOperationKind,
) -> ExpectedSession {
    match kind {
        // 崩溃恢复抛弃根也是一次可写挂载：落到的前一条根带不带文件，生成时照「这段历史发过文件没有」估，与可写挂载同一个估法；
        // 掉电在 journal 记录之前的那次覆盖写之后也是一次可写挂载，落到的是那次改动之前的一版；
        // 最新那一槽系统配置读回全 0 的那两种（每块盘都藏、只藏第一块盘）也是可写挂载，同一个估法。
        HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => {
            match first_file {
                ExpectedFirstFile::Written => ExpectedSession::OpenWithFile,
                ExpectedFirstFile::NotWrittenYet => ExpectedSession::OpenWithoutFile,
                ExpectedFirstFile::DeletedAndFollowedByItsOwnTable => {
                    ExpectedSession::OpenAfterTheFileWasDeleted
                }
            }
        }
        HistoryOperationKind::ColdStartRecover => ExpectedSession::Closed,
        HistoryOperationKind::PublishFirstFile => match expected {
            ExpectedSession::OpenWithoutFile => ExpectedSession::OpenWithFile,
            // 删完之后第一个文件照条款被拒（树表不是 0 条），估计不变。
            ExpectedSession::Closed
            | ExpectedSession::OpenWithFile
            | ExpectedSession::OpenAfterTheFileWasDeleted => expected,
        },
        HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::CreateInode
        | HistoryOperationKind::CreateSnapshot
        | HistoryOperationKind::ReadSnapshot
        | HistoryOperationKind::DestroySnapshot
        // 整理不改用户看得见的东西；批间崩溃之后那次可写挂载落到的就是最后那一批，有没有文件与整理之前相同。
        | HistoryOperationKind::CompactWhileMounted
        // 截断只改文件的长度，有没有文件不变（删完之后点名别的 inode 的那一次被拒）。
        | HistoryOperationKind::Truncate
        // 建 inode、按 inode 号写不改第一个文件在不在（删完之后按 inode 号写被拒）。
        | HistoryOperationKind::CreateInodes
        | HistoryOperationKind::WriteOfAnInode
        // 按偏移写同样不改第一个文件在不在（删完之后按偏移写删完之后才建的 inode 被拒）。
        | HistoryOperationKind::WriteAtOffset => expected,
        // unlink 之后那一版有树而文件集合为空：比重写了 unlink 之后那张表的按它抽，没写的照旧按「开着、没有文件」抽
        // （第一个文件在它上面照条款被拒）。
        HistoryOperationKind::Unlink => match expected {
            ExpectedSession::OpenWithFile => {
                if deletion_is_followed_by_its_own_table {
                    ExpectedSession::OpenAfterTheFileWasDeleted
                } else {
                    ExpectedSession::OpenWithoutFile
                }
            }
            ExpectedSession::Closed
            | ExpectedSession::OpenWithoutFile
            | ExpectedSession::OpenAfterTheFileWasDeleted => expected,
        },
    }
}

/// 这一步之后对第一个文件的估计：unlink 之后比重写了那张表的一直记「删过」，没写的回到「还没写过」（与加「建 inode」之前的估法逐项相同）；
/// 别的操作之后，写过的仍是写过，估计开着带文件的也记写过。
fn expected_first_file_after(
    first_file: ExpectedFirstFile,
    deletion_is_followed_by_its_own_table: bool,
    kind: HistoryOperationKind,
    expected_after: ExpectedSession,
) -> ExpectedFirstFile {
    if kind == HistoryOperationKind::Unlink {
        return if deletion_is_followed_by_its_own_table {
            ExpectedFirstFile::DeletedAndFollowedByItsOwnTable
        } else {
            ExpectedFirstFile::NotWrittenYet
        };
    }
    match first_file {
        ExpectedFirstFile::DeletedAndFollowedByItsOwnTable | ExpectedFirstFile::Written => {
            first_file
        }
        ExpectedFirstFile::NotWrittenYet => {
            if expected_after == ExpectedSession::OpenWithFile {
                ExpectedFirstFile::Written
            } else {
                ExpectedFirstFile::NotWrittenYet
            }
        }
    }
}

/// 按种子生成一段 `operation_count` 步的历史，比重取 `GenerationWeights::BROAD`：同一个种子、同一个步数，逐项相同。
#[must_use]
pub fn generate_history(seed: HistorySeed, operation_count: usize) -> GeneratedHistory {
    generate_history_with_weights(seed, operation_count, &GenerationWeights::BROAD)
}

/// 按种子与比重生成一段 `operation_count` 步的历史：同一个种子、同一个步数、同一组比重，逐项相同。
#[must_use]
pub fn generate_history_with_weights(
    seed: HistorySeed,
    operation_count: usize,
    weights: &GenerationWeights,
) -> GeneratedHistory {
    let mut source = SeededRandomSource::from_seed(seed.0);
    let starting_point = draw_weighted(&mut source, weights.starting_points);
    let mut expected = match starting_point {
        HistoryStartingPoint::AfterMakeFilesystem => ExpectedSession::Closed,
        HistoryStartingPoint::AfterFirstFile => ExpectedSession::OpenWithFile,
    };
    let mut first_file = match starting_point {
        HistoryStartingPoint::AfterMakeFilesystem => ExpectedFirstFile::NotWrittenYet,
        HistoryStartingPoint::AfterFirstFile => ExpectedFirstFile::Written,
    };
    let deletion_is_followed_by_its_own_table = weights
        .with_session_open_after_the_file_was_deleted
        .is_some();
    let mut operations = Vec::with_capacity(operation_count);
    for _ in 0..operation_count {
        let kind = draw_weighted(&mut source, weights.for_session(expected));
        operations.push(draw_operation(
            &mut source,
            kind,
            weights.rollback_targets,
            weights.overwrite_contents,
        ));
        expected = expected_session_after(
            expected,
            first_file,
            deletion_is_followed_by_its_own_table,
            kind,
        );
        first_file = expected_first_file_after(
            first_file,
            deletion_is_followed_by_its_own_table,
            kind,
            expected,
        );
    }
    GeneratedHistory {
        seed,
        starting_point,
        operations,
    }
}

/// [`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`] 里掉电之后的那次可写挂载（实例 2）之后覆盖写几次。
/// 根环 24 槽（`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 每区 8 槽、三区）里实例 1 的根要全被轮出去：实例 2 的写行、两次暖机与
/// 之后 24 次覆盖写一共 27 条根；第 25 次是崩溃注入摆崩溃状态的那一步。
pub const OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE: u64 = 25;

/// 留下实例 1 的孤儿、再把实例 1 的根全轮出根环的一段写死的历史：从第一个文件起（实例 1 的会话开着），一次覆盖写掉电在它的第一个
/// journal 记录写之前、进程崩掉再可写挂载成实例 2（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]：
/// 写行把实例 1 那一行写进表，那次覆盖写写下的单元按这一行判未发布），再覆盖写 [`OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE`] 次。
/// 跑完之后（或崩在最后一次覆盖写之前）的盘面上，有一行就扫的可写挂载末尾清扫删实例 1 那一行、先抹它的孤儿的头
/// （D18（块里携带什么信息） 已定项 11「回收」一条）：抹头那一段从这里走得到。
#[must_use]
pub fn history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(
) -> GeneratedHistory {
    let mut operations = vec![
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
            ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 0 },
                fill_seed: 0,
            },
        ),
    ];
    operations.extend(
        (1..=OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE).map(|overwrite| {
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: overwrite,
                },
                fill_seed: overwrite,
            })
        }),
    );
    GeneratedHistory {
        seed: HistorySeed(3),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 留孤儿那一组比重抽得到「覆盖写掉电在 journal 记录之前再挂」，别的每一组比重一次都抽不到（它们的种子钉着的取样点照旧）：
    /// 32 个种子、每段 40 步。判别力：留孤儿那一组里这一步的份数是 0 时第一条红。
    #[test]
    fn the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does() {
        let kind =
            HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable;
        let steps_drawn_with = |weights: &GenerationWeights| {
            (0..32u64)
                .map(|seed_offset| {
                    generate_history_with_weights(
                        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
                        40,
                        weights,
                    )
                    .operations
                    .iter()
                    .filter(|operation| operation.kind() == kind)
                    .count()
                })
                .sum::<usize>()
        };
        assert!(
            steps_drawn_with(&GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP) >= 1,
            "留孤儿那一组比重 32 段里一次都没抽到 {kind:?}"
        );
        for weights in GenerationWeights::NAMED {
            if weights.named == NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep {
                continue;
            }
            assert_eq!(
                steps_drawn_with(&weights),
                0,
                "{} 那一组比重抽到了 {kind:?}",
                weights.name
            );
        }
    }

    /// 每一组有名字的比重：名字找回的那一组自报的也是这个名字（两组比重报同一个名字时，有一组复现入口永远找不回），
    /// [`GenerationWeights::NAMED`] 里同一个位置上就是它，复现入口按它的取值读回的也是它。
    #[test]
    fn every_named_generation_weights_member_finds_its_own_weights_and_reads_back_from_its_value() {
        let defaults = HistorySampling {
            operations_per_history: 1,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        for (position, member) in NamedGenerationWeights::ALL.into_iter().enumerate() {
            assert_eq!(
                member.weights().named,
                member,
                "{member:?} 找回的那一组比重自报的名字不是它"
            );
            assert_eq!(
                GenerationWeights::NAMED[position],
                member.weights(),
                "NAMED 第 {position} 个不是 {member:?} 那一组"
            );
            let read_back = HistorySampling::from_environment(
                RANDOM_HISTORY_VARIABLE_PREFIX,
                &|variable: &str| {
                    (variable == "SINGLEFS_RANDOM_HISTORY_WEIGHTS")
                        .then(|| member.environment_value().to_string())
                },
                &defaults,
            )
            .map(|sampling| sampling.weights);
            assert_eq!(
                read_back,
                Ok(member.weights()),
                "WEIGHTS={} 读回的不是 {member:?} 那一组",
                member.environment_value()
            );
        }
    }

    /// 同一个种子、同一个步数生成的历史逐项相同；换一个种子就不同。
    #[test]
    fn the_same_seed_generates_the_same_history_and_another_seed_does_not() {
        let first = generate_history(HistorySeed(42), 40);
        assert_eq!(first, generate_history(HistorySeed(42), 40));
        assert_ne!(
            first.operations,
            generate_history(HistorySeed(43), 40).operations,
            "只比操作序列：历史里带着种子，整个比的话种子不同就恒不等"
        );
        assert_eq!(
            first.operations[..20],
            generate_history(HistorySeed(42), 20).operations[..],
            "步数只截尾，不改前面几步"
        );
    }

    /// 截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）进自己那一组比重与带整理的那一组：两组的带文件那张表与
    /// unlink 之后那张表都有截断、份数不为 0；别的每一组比重的每张表都没有截断（已有各组生成的历史不变）。
    /// 判别力：带整理那一组带文件那张表里截断的份数改成 0（`crates/mutations.tsv` 一行）时这一条红。
    #[test]
    fn the_truncation_weights_and_the_broad_compaction_weights_draw_truncations_and_no_other_weights_do(
    ) {
        let truncation_weight_of = |table: &[(HistoryOperationKind, u64)]| {
            table
                .iter()
                .filter(|(kind, _)| *kind == HistoryOperationKind::Truncate)
                .map(|(_, weight)| *weight)
                .sum::<u64>()
        };
        for weights in GenerationWeights::NAMED {
            let tables = [
                weights.with_session_closed,
                weights.with_session_open_without_file,
                weights.with_session_open_with_file,
                weights
                    .with_session_open_after_the_file_was_deleted
                    .unwrap_or(&[]),
            ];
            let truncation_weights: Vec<u64> =
                tables.into_iter().map(truncation_weight_of).collect();
            match weights.named {
                NamedGenerationWeights::Truncation
                | NamedGenerationWeights::BroadWithCompactionWhileMounted
                | NamedGenerationWeights::SizeChangesOfInodesOtherThanTheFirst
                // 按偏移写那一组抽截断里的变长留洞（「写进洞里」那一格要有洞）。
                | NamedGenerationWeights::PartialWrites => {
                    assert!(
                        truncation_weights[2] > 0 && truncation_weights[3] > 0,
                        "{} 的带文件那张表与 unlink 之后那张表都抽截断：{truncation_weights:?}",
                        weights.name
                    );
                }
                NamedGenerationWeights::Broad
                | NamedGenerationWeights::ReuseAfterRaisingTheFloor
                | NamedGenerationWeights::RollbackAfterRaisingTheFloor
                | NamedGenerationWeights::TowardTheAllocationRecordWall
                | NamedGenerationWeights::TowardTheUnitAreaWall
                | NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves
                | NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep
                | NamedGenerationWeights::NewestSystemConfigurationSlotReadingZeros
                | NamedGenerationWeights::SnapshotOperations
                | NamedGenerationWeights::CompactionWhileMounted
                | NamedGenerationWeights::CompactionWhileMountedWithRollback
                | NamedGenerationWeights::WritesByInodeNumber => {
                    assert_eq!(
                        truncation_weights,
                        vec![0, 0, 0, 0],
                        "{} 不抽截断",
                        weights.name
                    );
                }
            }
        }
    }
}
