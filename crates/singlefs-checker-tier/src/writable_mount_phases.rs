//! 可写挂载的分段与崩溃镜像落成池：挂载那一段录制流里每一次写落在挂载的哪一段（[`WritableMountPhase`]）、清扫两段各是哪几个写、
//! 清扫之前这次挂载发了几次，与把一份崩溃镜像落成一个池。
//!
//! 从 `crash_injection` 挪出来：实例表清扫的崩溃点用例（`tests/crash_points_of_the_instance_table_sweep.rs`）只用这几样、不拿随机历史当输入，
//! 经这一份引就不把随机历史脚手架带进它的判法闭包（`crash_identity` 里登记的崩溃枚举用例那一核，按模块引用收文件）。
//! 所以这一份不引随机历史（`singlefs_harness::history`），也不引崩溃注入那一份；崩溃注入经它引这几样、原样转出。

use std::collections::BTreeMap;

use singlefs_core::instance_table_sweep::{
    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
};
use singlefs_core::mount::MountOutput;
use singlefs_harness::memory_pool::{CrashImage, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;

/// 可写挂载的六段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机、
/// D8（核心索引结构） 已定项 16 第 6 条挂载续做、D18 已定项 11「回收」一条的清扫抹头与清扫发布）：挂载那一段录制流里的一次写落在哪一段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WritableMountPhase {
    /// 取号：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写。
    Acquisition,
    /// 写行那次发布：到它的根槽写、连同紧跟着的系统配置槽轮换为止。
    RowPublish,
    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F：摆二次崩溃时两者算一段，F 那一样按抬 F 的写有没有持久分开判，
    /// 崩溃注入里 `FloorRaisesInTheMountWrites::at_a_second_crash`）。
    WarmUp,
    /// 挂载续做：暖机与推的抬 F 之后、交回之前把所选那一版的 `deleted_inodes` 走空的那几步删除发布（所选那一版有删除中的文件时才有）。
    Continuation,
    /// 实例表清扫抹头：续做之后、清扫发布之前，FUA 写零那几个未发布单元的头、每块盘一道屏障（`singlefs_core::instance_table_sweep`；
    /// 挂载末尾清扫了才有）。
    ErasingHeadsOfTheInstanceTableSweep,
    /// 实例表清扫发布：删行的那一次专用发布（单元、记录、根槽、轮换）。
    InstanceTableSweepPublish,
}

impl WritableMountPhase {
    pub const ALL: [WritableMountPhase; 6] = [
        WritableMountPhase::Acquisition,
        WritableMountPhase::RowPublish,
        WritableMountPhase::WarmUp,
        WritableMountPhase::Continuation,
        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
        WritableMountPhase::InstanceTableSweepPublish,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            WritableMountPhase::Acquisition => "acquisition",
            WritableMountPhase::RowPublish => "row_publish",
            WritableMountPhase::WarmUp => "warm_up",
            WritableMountPhase::Continuation => "continuation",
            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep => {
                "erasing_heads_of_the_instance_table_sweep"
            }
            WritableMountPhase::InstanceTableSweepPublish => "instance_table_sweep_publish",
        }
    }
}

/// 把一份崩溃镜像（基线 + 这一段里持久了的写）落成一个池：之后的可写挂载在它的副本上写。
#[must_use]
pub fn materialized_crash_image(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

/// 挂载末尾清扫写了东西时，清扫之前这次挂载发了几次（崩溃注入里那次可写挂载交回的 `publishes_before_the_instance_table_sweep` 取的就是它）。
#[must_use]
pub fn publishes_before_the_instance_table_sweep_of_the_mount_output(
    output: &MountOutput,
) -> Option<usize> {
    let sweep_wrote_something = match &output.instance_table_sweep {
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
        | InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(_)) => {
            false
        }
        InstanceTableSweepAtTheEndOfTheMount::Swept(
            InstanceTableSweepOutcome::RowsDeleted { .. }
            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
        ) => true,
        InstanceTableSweepAtTheEndOfTheMount::Swept(
            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { heads_erased, .. }
            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                heads_erased, ..
            },
        ) => !heads_erased.is_empty(),
    };
    sweep_wrote_something.then(|| {
        1 + output.warm_up_publishes.len()
            + output
                .space_admission
                .floor_raises()
                .iter()
                .chain(&output.floor_raises_pushed_by_the_continuation)
                .map(|raised| raised.publishes.len())
                .sum::<usize>()
            + output.deletion_queue_continuation.len()
    })
}

/// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布；
/// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
/// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）；第 `publishes_before_the_instance_table_sweep`
/// 个根槽写之后起是清扫：其中抹头那一段只有 FUA 写零头所在物理块的单元写（与每块盘一道屏障），第一个别的写（不带 FUA 的单元写、
/// journal 记录写……）起是清扫发布——一个头都没抹时清扫发布从清扫之前最后一次发布的轮换之后第一个写起，抹头那一段一个写都没有
/// （没清扫时 None）。
pub(crate) fn writable_mount_phase_of_each_write(
    writes: &[RetainedWrite],
    publishes_before_the_deletion_queue_continuation: Option<usize>,
    publishes_before_the_instance_table_sweep: Option<usize>,
) -> Vec<WritableMountPhase> {
    let phase_of_the_publish_after =
        |root_records_written: usize, phase_so_far: WritableMountPhase, write: &RetainedWrite| {
            if publishes_before_the_instance_table_sweep.is_some_and(|publishes_before_the_sweep| {
                root_records_written > 0 && root_records_written >= publishes_before_the_sweep
            }) {
                // 已经在抹头里：往清扫发布换段由下面那几条臂按写的种类判。刚从清扫之前那几段进清扫：头一个写是 FUA 写零（或屏障）才进抹头，
                // 别的写说明一个头都没抹、直接是清扫发布。
                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {
                    return WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep;
                }
                return match (write.kind, write.is_force_unit_access) {
                    (StepKind::UnitWrite, true) | (StepKind::Barrier, _) => {
                        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
                    }
                    (StepKind::UnitWrite, false)
                    | (
                        StepKind::JournalRecord
                        | StepKind::RootRecordFua
                        | StepKind::ZeroFill
                        | StepKind::SystemConfigurationSlot,
                        _,
                    ) => WritableMountPhase::InstanceTableSweepPublish,
                };
            }
            match (
                root_records_written,
                publishes_before_the_deletion_queue_continuation,
            ) {
                (0, _) => WritableMountPhase::RowPublish,
                (publishes_landed, Some(publishes_before_the_continuation))
                    if publishes_landed >= publishes_before_the_continuation =>
                {
                    WritableMountPhase::Continuation
                }
                (_, Some(_) | None) => WritableMountPhase::WarmUp,
            }
        };
    let mut phase = WritableMountPhase::Acquisition;
    let mut root_records_written = 0;
    // 迭代上界是挂载那一段的写数；跨轮带的是这一次写落在哪一段与已写的根槽数；没有提前出口。
    writes
        .iter()
        .map(|write| {
            phase = match (phase, write.kind) {
                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {
                    WritableMountPhase::Acquisition
                }
                (
                    WritableMountPhase::RowPublish
                    | WritableMountPhase::WarmUp
                    | WritableMountPhase::Continuation
                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
                    | WritableMountPhase::InstanceTableSweepPublish,
                    StepKind::SystemConfigurationSlot,
                ) => phase,
                (
                    WritableMountPhase::InstanceTableSweepPublish,
                    StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                )
                | (
                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                    StepKind::JournalRecord,
                ) => WritableMountPhase::InstanceTableSweepPublish,
                (WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep, StepKind::UnitWrite)
                    if !write.is_force_unit_access =>
                {
                    WritableMountPhase::InstanceTableSweepPublish
                }
                (
                    WritableMountPhase::Acquisition
                    | WritableMountPhase::RowPublish
                    | WritableMountPhase::WarmUp
                    | WritableMountPhase::Continuation
                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                    StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                ) => phase_of_the_publish_after(root_records_written, phase, write),
            };
            if write.kind == StepKind::RootRecordFua {
                root_records_written += 1;
            }
            phase
        })
        .collect()
}

/// 清扫两段（抹头、清扫发布）在挂载那一段写表里各是哪几个写（给 checker 档清扫的用例划段用；划法同 [`writable_mount_phase_of_each_write`]）。
#[must_use]
pub fn writes_of_the_instance_table_sweep_phases(
    writes: &[RetainedWrite],
    publishes_before_the_instance_table_sweep: usize,
) -> BTreeMap<WritableMountPhase, Vec<usize>> {
    let mut by_phase: BTreeMap<WritableMountPhase, Vec<usize>> = BTreeMap::new();
    for (write_index, phase) in writable_mount_phase_of_each_write(
        writes,
        None,
        Some(publishes_before_the_instance_table_sweep),
    )
    .into_iter()
    .enumerate()
    {
        by_phase.entry(phase).or_default().push(write_index);
    }
    by_phase
}
