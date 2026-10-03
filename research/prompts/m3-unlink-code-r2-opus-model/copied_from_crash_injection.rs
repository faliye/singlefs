// 从冻结副本 crates/singlefs-checker-tier/src/crash_injection.rs 原样抄出（第 1732–1973 行、第 2476–2529 行），只为在 harness 档的原型里调；
// 抄来的私有函数不改一字，外面只补 use 与 WritableMountPhase 的定义。
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{MountOutput, RaisedFloor};
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
use singlefs_harness::memory_pool::{root_identity_written_by, RetainedWrite};
use singlefs_harness::mount_transparency::FloorRaisesInTheMountWrites;
use singlefs_harness::segments::StepKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WritableMountPhase { Acquisition, RowPublish, WarmUp, Continuation }

/// 挂载里一次发布交给设备的写数与它写的根身份 (实例代号, checkpoint_txg)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PublishInTheMountWrites {
    write_calls: u64,
    root: (InstanceGeneration, CheckpointTxg),
}

impl PublishInTheMountWrites {
    fn of_the_publish(publish: &TransactionOutput) -> Self {
        PublishInTheMountWrites {
            write_calls: publish.writes.total().write_calls,
            root: (publish.root.instance, publish.root.checkpoint_txg),
        }
    }
}

/// 挂载推的一串抬 F 的写账：先把新 F 写进每块盘系统配置的那几个写（`RaisedFloor::system_configuration_writes_before_the_first_publish`），
/// 再是各次带新 F 的空发布（按先后）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FloorRaiseInTheMountWrites {
    system_configuration_write_calls: u64,
    publishes: Vec<PublishInTheMountWrites>,
}

impl FloorRaiseInTheMountWrites {
    fn of_the_raise(raise: &RaisedFloor) -> Self {
        FloorRaiseInTheMountWrites {
            system_configuration_write_calls: raise
                .system_configuration_writes_before_the_first_publish
                .total()
                .write_calls,
            publishes: raise
                .publishes
                .iter()
                .map(PublishInTheMountWrites::of_the_publish)
                .collect(),
        }
    }

    fn write_calls(&self) -> u64 {
        self.system_configuration_write_calls
            + self
                .publishes
                .iter()
                .map(|publish| publish.write_calls)
                .sum::<u64>()
    }

    /// # Panics
    /// 这一串一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
    fn checkpoint_txg_of_the_first_publish(&self) -> CheckpointTxg {
        self.publishes
            .first()
            .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
            .root
            .1
    }
}

/// 一次可写挂载交回的、取抬 F 那条界要用的三样：挂载准入在写行与暖机之后推的那几串（`MountOutput::space_admission`）、
/// 续做发成的每一步（`MountOutput::deletion_queue_continuation`）与续做里为空间推的那几串（`MountOutput::floor_raises_pushed_by_the_continuation`，
/// 与续做的各步按 txg 交错）。挂载写表的末尾依次是：准入推的那几串、续做的各步与续做推的那几串。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FloorRaisesOfTheMount {
    pushed_by_the_admission: Vec<FloorRaiseInTheMountWrites>,
    continuation_steps: Vec<PublishInTheMountWrites>,
    pushed_by_the_continuation: Vec<FloorRaiseInTheMountWrites>,
}

impl FloorRaisesOfTheMount {
    fn of_the_mount_output(output: &MountOutput) -> Self {
        FloorRaisesOfTheMount {
            pushed_by_the_admission: output
                .space_admission
                .floor_raises()
                .iter()
                .map(FloorRaiseInTheMountWrites::of_the_raise)
                .collect(),
            continuation_steps: output
                .deletion_queue_continuation
                .iter()
                .map(PublishInTheMountWrites::of_the_publish)
                .collect(),
            pushed_by_the_continuation: output
                .floor_raises_pushed_by_the_continuation
                .iter()
                .map(FloorRaiseInTheMountWrites::of_the_raise)
                .collect(),
        }
    }

    /// 从第一串抬 F 的第一个写起、到挂载写表末尾一共几个写，连同这一截里每一次抬 F 发布的根身份；一串都没推时 None。
    /// 第一串是准入推的（推了的话）：它之后是准入推的别的串、续做的每一步与续做推的每一串。准入一串都没推、续做里推了的：
    /// 第一串是续做推的第一串，它之后是续做推的每一串与 txg 大于它第一次空发布的那几步续做（之前的那几步在界之前）。
    fn writes_from_the_first_floor_raise_on(
        &self,
    ) -> Option<(u64, Vec<(InstanceGeneration, CheckpointTxg)>)> {
        let writes_of = |raises: &[FloorRaiseInTheMountWrites]| {
            raises
                .iter()
                .map(FloorRaiseInTheMountWrites::write_calls)
                .sum::<u64>()
        };
        let roots_of = |raises: &[FloorRaiseInTheMountWrites]| {
            raises
                .iter()
                .flat_map(|raise| raise.publishes.iter().map(|publish| publish.root))
                .collect::<Vec<_>>()
        };
        let continuation_steps_after = |checkpoint_txg: Option<CheckpointTxg>| {
            self.continuation_steps
                .iter()
                .filter(|step| checkpoint_txg.is_none_or(|boundary| step.root.1 > boundary))
                .map(|step| step.write_calls)
                .sum::<u64>()
        };
        if !self.pushed_by_the_admission.is_empty() {
            let mut roots = roots_of(&self.pushed_by_the_admission);
            roots.extend(roots_of(&self.pushed_by_the_continuation));
            return Some((
                writes_of(&self.pushed_by_the_admission)
                    + continuation_steps_after(None)
                    + writes_of(&self.pushed_by_the_continuation),
                roots,
            ));
        }
        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;
        Some((
            writes_of(&self.pushed_by_the_continuation)
                + continuation_steps_after(Some(
                    first_pushed_by_the_continuation.checkpoint_txg_of_the_first_publish(),
                )),
            roots_of(&self.pushed_by_the_continuation),
        ))
    }
}

/// 挂载推的抬 F 从挂载写表的哪个写起（[`FloorRaisesInTheMountWrites`]；`singlefs_harness::mount_transparency` 那一份
/// `FloorRaisesInTheMountWrites::of_the_mount_output` 只认挂载准入推的、按「抬 F 那几串在写表末尾」取界，挂载有续做时取不出、
/// 续做里推的看不见）：挂载准入推的与续做里推的都认（[`FloorRaisesOfTheMount::writes_from_the_first_floor_raise_on`]），
/// 界 = 写表长度减去从第一串抬 F 起的写数。取出的界再核两样：界上那个写是系统配置槽写（每一串先把新 F 写进系统配置），
/// 每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；对不上的交 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
fn floor_raises_in_the_mount_writes_counting_the_continuation(
    output: &MountOutput,
    mount_writes: &[RetainedWrite],
) -> FloorRaisesInTheMountWrites {
    floor_raises_located_in_the_mount_writes(
        &FloorRaisesOfTheMount::of_the_mount_output(output),
        mount_writes,
    )
}

fn floor_raises_located_in_the_mount_writes(
    floor_raises: &FloorRaisesOfTheMount,
    mount_writes: &[RetainedWrite],
) -> FloorRaisesInTheMountWrites {
    let Some((writes_from_the_first_floor_raise_on, floor_raise_roots)) =
        floor_raises.writes_from_the_first_floor_raise_on()
    else {
        return FloorRaisesInTheMountWrites::NotPushed;
    };
    let Some(first_write) = usize::try_from(writes_from_the_first_floor_raise_on)
        .ok()
        .and_then(|writes| mount_writes.len().checked_sub(writes))
    else {
        return FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites;
    };
    let the_boundary_is_a_system_configuration_slot_write = mount_writes
        .get(first_write)
        .is_some_and(|write| write.kind == StepKind::SystemConfigurationSlot);
    let roots_written_from_the_boundary: Vec<(InstanceGeneration, CheckpointTxg)> = mount_writes
        [first_write..]
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .filter_map(|write| write.bytes().map(root_identity_written_by))
        .collect();
    let every_floor_raise_root_is_written_from_the_boundary = floor_raise_roots
        .iter()
        .all(|root| roots_written_from_the_boundary.contains(root));
    if the_boundary_is_a_system_configuration_slot_write
        && every_floor_raise_root_is_written_from_the_boundary
    {
        FloorRaisesInTheMountWrites::FirstWriteAt(first_write)
    } else {
        FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites
    }
}

/// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
/// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
/// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKind) -> bool {
    WrittenStructureKind::IN_REPORT_ORDER.iter().any(|kind| {
        let only_the_continuation_writes_it = match kind {
            WrittenStructureKind::DataUnit
            | WrittenStructureKind::ExtentTreeNode
            | WrittenStructureKind::InodeTreeLeafContainer
            | WrittenStructureKind::InodeTreeRoot
            | WrittenStructureKind::DeletedInodesTreeNode => true,
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

/// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
/// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）。
fn writable_mount_phase_of_each_write(
    writes: &[RetainedWrite],
    publishes_before_the_deletion_queue_continuation: Option<usize>,
) -> Vec<WritableMountPhase> {
    let phase_of_the_publish_after = |root_records_written: usize| match (
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
                    | WritableMountPhase::Continuation,
                    StepKind::SystemConfigurationSlot,
                ) => phase,
                (
                    WritableMountPhase::Acquisition
                    | WritableMountPhase::RowPublish
                    | WritableMountPhase::WarmUp
                    | WritableMountPhase::Continuation,
                    StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                ) => phase_of_the_publish_after(root_records_written),
            };
            if write.kind == StepKind::RootRecordFua {
                root_records_written += 1;
            }
            phase
        })
        .collect()
}

// ---- 以下是原型加的出口（上面抄来的私有函数原样不动，这里只转一手给原型的测试文件调） ----
pub fn located_counting_the_continuation(
    output: &MountOutput,
    mount_writes: &[RetainedWrite],
) -> FloorRaisesInTheMountWrites {
    floor_raises_in_the_mount_writes_counting_the_continuation(output, mount_writes)
}

pub fn partition_of_a_mount_that_handed_back_an_error(
    writes_of_persisted_publishes: &[WritesByStructureKind],
    writes_of_failed_publishes: &[WritesByStructureKind],
) -> Option<usize> {
    publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
        writes_of_persisted_publishes,
        writes_of_failed_publishes,
    )
}

pub fn phases(writes: &[RetainedWrite], before: Option<usize>) -> Vec<WritableMountPhase> {
    writable_mount_phase_of_each_write(writes, before)
}

pub fn only_the_continuation_writes(writes: &WritesByStructureKind) -> bool {
    writes_a_structure_only_the_continuation_changes(writes)
}
