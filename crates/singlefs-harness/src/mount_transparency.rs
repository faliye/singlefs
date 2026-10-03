//! 挂载透明（独立的第四条，不是 M3 的第三读：`research/prompts/m3-formal-r4-main-verification.md` 第四节第 3 条；
//! 逐字段表与二次崩溃、拒绝合法照 `research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条，零轮）的判法。
//!
//! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F，
//! 连同挂载续做与续做里推的抬 F：用户 2026-10-03 定「续做算挂载的一部分」），
//! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
//! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]；
//! 抬 F 的界在挂载写表里对不上位置的那一次挂载，二次崩溃整段 F 不判、按 [`FieldNotJudged`] 计数）。
//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]；崩溃注入第二截再加 C670 那一条：
//! 没注入读故障的崩溃态上判据 N-配置 复算为真，拒得不合法，[`judge_writable_mount_refusal_of_a_crash_state`]）。
//!
//! 读盘的只有 [`read_recovery_fields`] 与那两个判拒绝的函数里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
//! 只吃读好的字段与挂载交回值。checker 档的崩溃注入调它，harness 档的轻用例也调它（harness 档不依赖 checker 档）。
//! r5 判决标「推的」的几样（树 ID 水位、两个根指针、txg、扫描报告）照写进判法，每一样红的时候在交回值里单独标出它是推的还是量过的
//! （[`MountTransparencyField::basis`]）。

use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
use singlefs_core::records::{TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, effective_rollback_floor, highest_root_instance,
    highest_root_txg, instance_table_of_root, readable_roots, recover, replay_journal,
    scan_journal, user_visible_tree_root_pointers, verified_system_configuration_slots,
    walk_to_file, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
    UserVisibleTreeRootPointers,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;

use crate::memory_pool::{root_identity_written_by, RetainedWrite};
use crate::segments::StepKind;

/// 一次只读恢复读出的 POSIX 内容与存在性。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PosixContentRead {
    File(Vec<u8>),
    NoFile,
    /// 恢复或走读失败，什么都没读出来。
    NothingRead,
}

impl PosixContentRead {
    /// 一份恢复报告读出的内容（`recover` 的结局）。
    #[must_use]
    pub fn of_the_recovery_report(report: &RecoveryReport) -> Self {
        match &report.outcome {
            RecoveryOutcome::FileRead { content, .. } => PosixContentRead::File(content.clone()),
            RecoveryOutcome::NoFile { .. } => PosixContentRead::NoFile,
            RecoveryOutcome::Failed { .. } => PosixContentRead::NothingRead,
        }
    }

    /// 给人看：几个字节、没有文件、没读出来（不把整份内容打出来）。
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            PosixContentRead::File(content) => format!("文件 {} 字节", content.len()),
            PosixContentRead::NoFile => "没有文件".to_string(),
            PosixContentRead::NothingRead => "没读出内容".to_string(),
        }
    }
}

/// R2 拿哪一版来比：只挂载与二次崩溃比恢复落到的那一版；挂载加一次发布比挂载后那次发布之前那一版（r4 判决第二节 T5 那一行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparedVersion {
    TheRecoveredVersion,
    TheVersionBeforeTheRecoveredOne,
}

/// 拿来比的那一版自己的字段：根记录、读出的内容、它指着的实例表的行、树表里两棵用户可见树的根指针。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionFieldsRead {
    pub root: RootRecord,
    pub content: PosixContentRead,
    /// 实例表沿链有一片读不出、解不开时 None。
    pub instance_table_rows: Option<Vec<InstanceRow>>,
    /// 树表读不出、解不开时 None。
    pub user_visible_tree_root_pointers: Option<UserVisibleTreeRootPointers>,
}

/// 一个池上的一次只读恢复（看 journal）连同挂载透明要比的字段（[`read_recovery_fields`] 读出来的）。
/// 字段没有不变量，直接公开：harness 的轻用例改坏其中一样，证判法会红。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryFieldsRead {
    /// 那一次只读恢复的报告（`recover(reader, JournalPolicy::Consult)`）。
    pub report: RecoveryReport,
    pub compared: ComparedVersion,
    /// 拿来比的那一版；恢复没落到一条根、或要比的是发布之前那一版而根环里没有比恢复落到的那一版更旧的根时 None。
    pub compared_version: Option<VersionFieldsRead>,
    /// 择到的系统配置的不可变配置（择不到时 None）。
    pub immutable_configuration: Option<SystemImmutableConfiguration>,
    /// F 生效值（D16（发布语义） 已定项 1「生效」，`recovery::effective_rollback_floor`）。
    pub effective_rollback_floor: Option<CheckpointTxg>,
    /// 每块盘两槽里全部自证过（校验和过、fsid 与本池相同）的系统配置的实例代号的最大值（取号的 max 那一半，D18（块里携带什么信息） 已定项 11）。
    pub highest_system_configuration_instance: Option<InstanceGeneration>,
    /// 根环里全部可读根的实例代号的最大值（取号的 max 另一半）。
    pub highest_root_instance_in_the_ring: Option<InstanceGeneration>,
    /// 根环里全部可读根的 checkpoint_txg 的最大值（新实例第一次发布的 txg 下界那一半，D23（journal 的角色与格式） 已定项 14 注 3）。
    pub highest_root_txg_in_the_ring: Option<CheckpointTxg>,
    /// 环里全部自证通过的 journal 记录的 checkpoint_txg 的最大值（txg 下界另一半）。
    pub highest_record_checkpoint_txg_in_the_ring: Option<CheckpointTxg>,
}

/// 对 `reader` 跑一次只读恢复（看 journal），再读挂载透明要比的字段。`compared` 定拿哪一版来比（[`ComparedVersion`]）。
///
/// # Panics
/// 同一份盘、同一套读法再择一次根、再重放一次，落到的那一版与恢复报告里的不同：两次读同一份内存盘，读法是同一组函数，走不到。
#[must_use]
pub fn read_recovery_fields(
    reader: &dyn PoolReader,
    compared: ComparedVersion,
) -> RecoveryFieldsRead {
    let report = recover(reader, JournalPolicy::Consult);
    let mut fields = RecoveryFieldsRead {
        report,
        compared,
        compared_version: None,
        immutable_configuration: None,
        effective_rollback_floor: None,
        highest_system_configuration_instance: None,
        highest_root_instance_in_the_ring: None,
        highest_root_txg_in_the_ring: None,
        highest_record_checkpoint_txg_in_the_ring: None,
    };
    let Ok(system_configuration) = choose_system_configuration(reader) else {
        return fields;
    };
    let immutable = system_configuration.immutable;
    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
    fields.immutable_configuration = Some(immutable);
    fields.highest_system_configuration_instance = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                reader,
                device,
                slot_spacing,
                &immutable.filesystem_identifier,
            )
        })
        .map(|slot| slot.quantities.journal_instance)
        .max();
    fields.effective_rollback_floor = Some(effective_rollback_floor(
        reader,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    ));
    fields.highest_root_instance_in_the_ring = highest_root_instance(
        reader,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    fields.highest_root_txg_in_the_ring = highest_root_txg(
        reader,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let records = scan_journal(reader, &system_configuration);
    fields.highest_record_checkpoint_txg_in_the_ring =
        records.values().map(|record| record.checkpoint_txg).max();
    let Some(chosen) = choose_root(reader, &system_configuration) else {
        return fields;
    };
    let Ok((_, recovered_root)) = replay_journal(
        reader,
        &chosen,
        immutable.sizes.journal_ring_bytes,
        &records,
        true,
    ) else {
        return fields;
    };
    assert_eq!(
        Some((recovered_root.instance, recovered_root.checkpoint_txg)),
        fields.report.effective_root,
        "同一份盘上再择一次根、再重放一次，要落到恢复报告里那一版"
    );
    fields.compared_version = match compared {
        ComparedVersion::TheRecoveredVersion => Some(version_fields(
            reader,
            recovered_root,
            PosixContentRead::of_the_recovery_report(&fields.report),
        )),
        ComparedVersion::TheVersionBeforeTheRecoveredOne => readable_roots(
            reader,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
        )
        .into_iter()
        .filter(|root| {
            (root.checkpoint_txg, root.instance)
                < (recovered_root.checkpoint_txg, recovered_root.instance)
        })
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .map(|before| {
            let mut mapping_fallbacks = 0usize;
            let content = match walk_to_file(reader, &before, &mut mapping_fallbacks) {
                Ok(Some(content)) => PosixContentRead::File(content),
                Ok(None) => PosixContentRead::NoFile,
                Err(_failure) => PosixContentRead::NothingRead,
            };
            version_fields(reader, before, content)
        }),
    };
    fields
}

fn version_fields(
    reader: &dyn PoolReader,
    root: RootRecord,
    content: PosixContentRead,
) -> VersionFieldsRead {
    VersionFieldsRead {
        root,
        content,
        instance_table_rows: instance_table_of_root(reader, &root).map(|table| table.rows),
        user_visible_tree_root_pointers: user_visible_tree_root_pointers(reader, &root).ok(),
    }
}

/// 挂载交回值里挂载透明要的那一样：挂载推没推抬 F，推了的话最后一串抬到的 F。挂载准入在写行与暖机之后推的（`MountOutput::space_admission`）
/// 与挂载续做里为空间推的（`MountOutput::floor_raises_pushed_by_the_continuation`）都算：用户 2026-10-03 定「续做算挂载的一部分」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaisedByTheMount {
    NotRaised,
    RaisedTo(CheckpointTxg),
}

impl FloorRaisedByTheMount {
    /// 最后一串是续做里推的（推了的话）：续做接在准入推的那几串之后（`MountOutput::deletion_queue_continuation` 的契约）。
    ///
    /// # Panics
    /// 推过的一串里一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
        match output
            .space_admission
            .floor_raises()
            .iter()
            .chain(output.floor_raises_pushed_by_the_continuation.iter())
            .last()
        {
            None => FloorRaisedByTheMount::NotRaised,
            Some(last_raise) => FloorRaisedByTheMount::RaisedTo(
                last_raise
                    .publishes
                    .last()
                    .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
                    .root
                    .rollback_floor,
            ),
        }
    }
}

/// 二次崩溃镜像上，挂载推的抬 F 那几串可能已经落了一部分没有：挂载没推，或抬 F 的写一个都没持久（崩在取号、写行、暖机里，
/// 暖机那一次发布连同它末尾的系统配置轮换带的都是挂载之前的 F），一步都还没落——F 照 r5 判决 U5 前半句判不变。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaisesAtTheSecondCrash {
    NoneLandedYet,
    /// 挂载推了抬 F，二次崩溃镜像上抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起）：F 生效值落在 R1 的与抬到的之间哪一处，条款没写。
    SomeMayHaveLanded,
    /// 挂载推了抬 F，抬 F 从挂载写表的哪个写起取不出（[`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）：分不开暖机与抬 F，
    /// 这一次挂载的二次崩溃整段 F 不判。
    BoundaryNotLocatedInTheMountWrites,
}

/// 挂载那一段写流（屏障不进表，与二次崩溃切段的是同一张写表）里，挂载推的抬 F 从第几个写起。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaisesInTheMountWrites {
    NotPushed,
    /// 第一串抬 F 的第一个写的下标：那一串先把新 F 写进每块盘的系统配置、过一道屏障，之后才发带新 F 的空发布
    /// （`singlefs_core::transaction::write_the_raised_floor_into_every_system_configuration`）；这个下标之前的写都是取号、写行与暖机的。
    FirstWriteAt(usize),
    /// 挂载交回推了抬 F，那几串的写在这张写表里对不上位置（写数比表还长、界上那个写不是系统配置槽写、或某一次抬 F 发布的根槽写不在界之后）：
    /// 交来的写表与挂载交回的账对不上，界取不出。
    NotLocatedInTheMountWrites,
}

/// 挂载里一次发布交给设备的写数与它写的根身份 (实例代号, checkpoint_txg)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishInTheMountWrites {
    pub write_calls: u64,
    pub root: (InstanceGeneration, CheckpointTxg),
}

impl PublishInTheMountWrites {
    #[must_use]
    pub fn of_the_publish(publish: &TransactionOutput) -> Self {
        PublishInTheMountWrites {
            write_calls: publish.writes.total().write_calls,
            root: (publish.root.instance, publish.root.checkpoint_txg),
        }
    }
}

/// 挂载推的一串抬 F 的写账：先把新 F 写进每块盘系统配置的那几个写（`RaisedFloor::system_configuration_writes_before_the_first_publish`），
/// 再是各次带新 F 的空发布（按先后，至少一次：`mount::RaisedFloor::publishes` 的契约，构造时核）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloorRaiseInTheMountWrites {
    system_configuration_write_calls: u64,
    publishes: Vec<PublishInTheMountWrites>,
}

impl FloorRaiseInTheMountWrites {
    /// # Panics
    /// `publishes` 是空的：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes` 的契约）。
    #[must_use]
    pub fn new(
        system_configuration_write_calls: u64,
        publishes: Vec<PublishInTheMountWrites>,
    ) -> Self {
        assert!(
            !publishes.is_empty(),
            "抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）"
        );
        FloorRaiseInTheMountWrites {
            system_configuration_write_calls,
            publishes,
        }
    }

    #[must_use]
    pub fn of_the_raise(raise: &RaisedFloor) -> Self {
        FloorRaiseInTheMountWrites::new(
            raise
                .system_configuration_writes_before_the_first_publish
                .total()
                .write_calls,
            raise
                .publishes
                .iter()
                .map(PublishInTheMountWrites::of_the_publish)
                .collect(),
        )
    }

    fn write_calls(&self) -> u64 {
        self.system_configuration_write_calls
            + self
                .publishes
                .iter()
                .map(|publish| publish.write_calls)
                .sum::<u64>()
    }

    fn checkpoint_txg_of_the_first_publish(&self) -> CheckpointTxg {
        self.publishes
            .first()
            .expect("构造时核过：抬 F 那一串至少一次空发布")
            .root
            .1
    }
}

/// 一次可写挂载交回的、取抬 F 那条界要用的三样：挂载准入在写行与暖机之后推的那几串（`MountOutput::space_admission`）、
/// 续做发成的每一步（`MountOutput::deletion_queue_continuation`）与续做里为空间推的那几串（`MountOutput::floor_raises_pushed_by_the_continuation`，
/// 与续做的各步按 txg 交错）。挂载写表的末尾依次是：准入推的那几串、续做的各步与续做推的那几串。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloorRaisesOfTheMount {
    pub pushed_by_the_admission: Vec<FloorRaiseInTheMountWrites>,
    pub continuation_steps: Vec<PublishInTheMountWrites>,
    pub pushed_by_the_continuation: Vec<FloorRaiseInTheMountWrites>,
}

impl FloorRaisesOfTheMount {
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
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

impl FloorRaisesInTheMountWrites {
    /// 按位置取界（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条）：挂载准入推的与续做里推的
    /// 抬 F 那几串都认（[`FloorRaisesOfTheMount`]），界 = 写表长度减去从第一串抬 F 起到写表末尾的写数（每一串是先把新 F 写进每块盘系统配置的
    /// 那几个写再加各次空发布自己的写，续做的各步按 txg 落在第一串之后的那几步也在这一截里）。崩溃注入与 harness 档的用例取界都经这一份。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput, mount_writes: &[RetainedWrite]) -> Self {
        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(
            &FloorRaisesOfTheMount::of_the_mount_output(output),
            mount_writes,
        )
    }

    /// 不按写进系统配置的 F 认：系统配置里写的是抬到的 F 与盘上已生效的 F 的大者（`singlefs_core::transaction` 写系统配置槽那一处），
    /// 盘上已生效的 F 更高时不等于根里的 F，挂载里更早的系统配置写也可能恰带同一个 F。
    /// 取出的界再核两样：界上那个写是系统配置槽写（每一串先把新 F 写进系统配置），每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；
    /// 对不上的交 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
    #[must_use]
    pub fn of_the_floor_raises_of_the_mount(
        floor_raises: &FloorRaisesOfTheMount,
        mount_writes: &[RetainedWrite],
    ) -> Self {
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
        let roots_written_from_the_boundary: Vec<(InstanceGeneration, CheckpointTxg)> =
            mount_writes[first_write..]
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

    /// 一个二次崩溃状态上抬 F 那几串落没落（`persisted_in_the_mount` 与挂载那一段写表逐条对应）：
    /// 从抬 F 的第一个写起有一个持久的，就算可能落了一部分。
    ///
    /// # Panics
    /// `persisted_in_the_mount` 比抬 F 第一个写的下标还短：持久集合与写表对不上。
    #[must_use]
    pub fn at_a_second_crash(self, persisted_in_the_mount: &[bool]) -> FloorRaisesAtTheSecondCrash {
        match self {
            FloorRaisesInTheMountWrites::NotPushed => FloorRaisesAtTheSecondCrash::NoneLandedYet,
            FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites => {
                FloorRaisesAtTheSecondCrash::BoundaryNotLocatedInTheMountWrites
            }
            FloorRaisesInTheMountWrites::FirstWriteAt(first_write) => {
                let persisted_from_the_floor_raises = persisted_in_the_mount
                    .get(first_write..)
                    .expect("持久集合与挂载那一段写表逐条对应，抬 F 的第一个写在表里");
                if persisted_from_the_floor_raises.contains(&true) {
                    FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
                } else {
                    FloorRaisesAtTheSecondCrash::NoneLandedYet
                }
            }
        }
    }
}

/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里两棵用户可见树的根指针（用户 2026-10-03 定「续做算挂载的一部分」）：
/// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的两棵树；续做的每一步改了它们，之后挂载写的版本
/// （续做推的抬 F 的空发布）照抄最近一步的。挂载没续做时一步都没有，挂载写的每一版都比 R1 那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserVisibleTreeRootPointersOfTheContinuation {
    /// (那一步的实例代号与 checkpoint_txg, 两根指针)，按 txg 升序（构造时排）。
    steps: Vec<(
        (InstanceGeneration, CheckpointTxg),
        UserVisibleTreeRootPointers,
    )>,
}

impl UserVisibleTreeRootPointersOfTheContinuation {
    /// 挂载没续做（`MountOutput::deletion_queue_continuation` 空）。
    pub const NOT_CONTINUED: Self = Self { steps: Vec::new() };

    /// 两根指针取续做每一步交回的树表条目（`TransactionOutput::tree_table_entries`）里那 86 字节，与只读恢复读盘上树表的取法
    /// （`recovery::user_visible_tree_root_pointers`）同一段字节。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
        let mut steps: Vec<(
            (InstanceGeneration, CheckpointTxg),
            UserVisibleTreeRootPointers,
        )> = output
            .deletion_queue_continuation
            .iter()
            .map(|step| {
                (
                    (step.root.instance, step.root.checkpoint_txg),
                    user_visible_tree_root_pointers_of_a_published_version(step),
                )
            })
            .collect();
        steps.sort_by_key(|(version, _)| (version.1, version.0));
        UserVisibleTreeRootPointersOfTheContinuation { steps }
    }

    /// 挂载写的 `version` 那一版该带的两根指针：同一个实例里 txg 不大于它的最近一步续做那一版的；它在续做之前（或挂载没续做）时 None，
    /// 调用方比 R1 那一版。
    fn of_the_last_step_at_or_before(
        &self,
        version: (InstanceGeneration, CheckpointTxg),
    ) -> Option<&UserVisibleTreeRootPointers> {
        self.steps
            .iter()
            .filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)
            .map(|(_, pointers)| pointers)
            .next_back()
    }
}

/// 一次发布交回的树表条目里 inode 树、extent 树两条的根指针盘上字节；树表里没有那棵树的条目时 None。
fn user_visible_tree_root_pointers_of_a_published_version(
    version: &TransactionOutput,
) -> UserVisibleTreeRootPointers {
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    for entry in &version.tree_table_entries {
        let root_pointer = Some(TreeTableEntry::root_pointer_bytes(&entry.to_bytes()).to_vec());
        if entry.kind == TREE_KIND_INODE {
            pointers.inode_tree = root_pointer;
        } else if entry.kind == TREE_KIND_EXTENT {
            pointers.extent_tree = root_pointer;
        }
    }
    pointers
}

/// 做的是哪个动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountTransparencyAction {
    /// 只挂载，不接发布：R2 比它恢复落到的那一版（v2a）。
    MountAlone(FloorRaisedByTheMount),
    /// 挂载加一次发布：R2 比挂载后那次发布之前那一版。
    MountAndOnePublish(FloorRaisedByTheMount),
    /// 挂载途中再崩一次：R2 是二次崩溃镜像上的只读恢复，比的对象是 v1（r5 判决第二节 U6）。
    SecondCrashInsideTheMount(FloorRaisesAtTheSecondCrash),
}

/// 比的十样（r5 判决第二节 U5 那一行的逐字段表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MountTransparencyField {
    /// POSIX 内容与存在性。
    PosixContent,
    /// 系统配置的不可变配置（本盘设备号除外）。
    ImmutableConfigurationWithoutThisDevice,
    UnmountMarker,
    /// F：根里的与生效值。
    RollbackFloor,
    InstanceGeneration,
    InstanceTableRows,
    TreeIdentifierWatermark,
    /// 树表里 inode 树、extent 树两个根指针。
    UserVisibleTreeRootPointers,
    CheckpointTxg,
    /// 扫描报告（施加了几条、水位之上几条）。
    JournalScanReport,
}

/// 一样字段的关系是攻方在副本装置上量过（主 agent 复跑坐实）的，还是 r5 判决标「推的」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldBasis {
    MeasuredByTheSecondRoundAttack,
    InferredNotMeasured,
}

impl MountTransparencyField {
    pub const ALL: [MountTransparencyField; 10] = [
        MountTransparencyField::PosixContent,
        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
        MountTransparencyField::UnmountMarker,
        MountTransparencyField::RollbackFloor,
        MountTransparencyField::InstanceGeneration,
        MountTransparencyField::InstanceTableRows,
        MountTransparencyField::TreeIdentifierWatermark,
        MountTransparencyField::UserVisibleTreeRootPointers,
        MountTransparencyField::CheckpointTxg,
        MountTransparencyField::JournalScanReport,
    ];

    /// 这一样在这个动作下的关系是量过的还是推的。二次崩溃那一截的字段一半整截是推的（r5 判决第二节 U6、攻方 U6-1），
    /// F 在挂载推了抬 F 时那一半（「= 挂载交回的最后一串抬到的 F」）r5 判决写的是「后半没量」。
    #[must_use]
    pub fn basis(self, action: MountTransparencyAction) -> FieldBasis {
        match (action, self) {
            (MountTransparencyAction::SecondCrashInsideTheMount(_), _)
            | (
                MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(_))
                | MountTransparencyAction::MountAndOnePublish(FloorRaisedByTheMount::RaisedTo(_)),
                MountTransparencyField::RollbackFloor,
            )
            | (
                MountTransparencyAction::MountAlone(_)
                | MountTransparencyAction::MountAndOnePublish(_),
                MountTransparencyField::TreeIdentifierWatermark
                | MountTransparencyField::UserVisibleTreeRootPointers
                | MountTransparencyField::CheckpointTxg
                | MountTransparencyField::JournalScanReport,
            ) => FieldBasis::InferredNotMeasured,
            (
                MountTransparencyAction::MountAlone(_)
                | MountTransparencyAction::MountAndOnePublish(_),
                MountTransparencyField::PosixContent
                | MountTransparencyField::ImmutableConfigurationWithoutThisDevice
                | MountTransparencyField::UnmountMarker
                | MountTransparencyField::RollbackFloor
                | MountTransparencyField::InstanceGeneration
                | MountTransparencyField::InstanceTableRows,
            ) => FieldBasis::MeasuredByTheSecondRoundAttack,
        }
    }

    /// 判红时记进违例的名字（崩溃注入按它分签名）：量过的与推的各一个名字，提交时的快档一眼分得开。
    #[must_use]
    pub fn violation_name(self, basis: FieldBasis) -> &'static str {
        match (self, basis) {
            (MountTransparencyField::PosixContent, FieldBasis::MeasuredByTheSecondRoundAttack) => {
                "挂载透明：POSIX 内容（量过）"
            }
            (MountTransparencyField::PosixContent, FieldBasis::InferredNotMeasured) => {
                "挂载透明：POSIX 内容（推的）"
            }
            (
                MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：不可变配置（量过）",
            (
                MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
                FieldBasis::InferredNotMeasured,
            ) => "挂载透明：不可变配置（推的）",
            (MountTransparencyField::UnmountMarker, FieldBasis::MeasuredByTheSecondRoundAttack) => {
                "挂载透明：卸载记号（量过）"
            }
            (MountTransparencyField::UnmountMarker, FieldBasis::InferredNotMeasured) => {
                "挂载透明：卸载记号（推的）"
            }
            (MountTransparencyField::RollbackFloor, FieldBasis::MeasuredByTheSecondRoundAttack) => {
                "挂载透明：F（量过）"
            }
            (MountTransparencyField::RollbackFloor, FieldBasis::InferredNotMeasured) => {
                "挂载透明：F（推的）"
            }
            (
                MountTransparencyField::InstanceGeneration,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：实例代号（量过）",
            (MountTransparencyField::InstanceGeneration, FieldBasis::InferredNotMeasured) => {
                "挂载透明：实例代号（推的）"
            }
            (
                MountTransparencyField::InstanceTableRows,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：实例表行（量过）",
            (MountTransparencyField::InstanceTableRows, FieldBasis::InferredNotMeasured) => {
                "挂载透明：实例表行（推的）"
            }
            (
                MountTransparencyField::TreeIdentifierWatermark,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：树 ID 水位（量过）",
            (MountTransparencyField::TreeIdentifierWatermark, FieldBasis::InferredNotMeasured) => {
                "挂载透明：树 ID 水位（推的）"
            }
            (
                MountTransparencyField::UserVisibleTreeRootPointers,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：两棵树的根指针（量过）",
            (
                MountTransparencyField::UserVisibleTreeRootPointers,
                FieldBasis::InferredNotMeasured,
            ) => "挂载透明：两棵树的根指针（推的）",
            (MountTransparencyField::CheckpointTxg, FieldBasis::MeasuredByTheSecondRoundAttack) => {
                "挂载透明：txg（量过）"
            }
            (MountTransparencyField::CheckpointTxg, FieldBasis::InferredNotMeasured) => {
                "挂载透明：txg（推的）"
            }
            (
                MountTransparencyField::JournalScanReport,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：扫描报告（量过）",
            (MountTransparencyField::JournalScanReport, FieldBasis::InferredNotMeasured) => {
                "挂载透明：扫描报告（推的）"
            }
        }
    }
}

/// 某一样为什么没判：条款没写这一格该是什么，不替它定红绿，只计数（交主 agent）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldNotJudged {
    /// 挂载推了抬 F、二次崩溃镜像上抬 F 的写有持久的（[`FloorRaisesAtTheSecondCrash::SomeMayHaveLanded`]）：F 生效值落在哪一处条款没写。
    FloorOfASecondCrashAfterTheFloorRaisesBegan,
    /// 挂载推了抬 F、抬 F 的界在挂载写表里取不出（[`FloorRaisesAtTheSecondCrash::BoundaryNotLocatedInTheMountWrites`]）：
    /// 分不开暖机与抬 F，这一次挂载的二次崩溃整段 F 不判。
    FloorOfASecondCrashInAMountWhoseFloorRaisesWereNotLocated,
    /// 二次崩溃镜像上的恢复可以合法地由记录重建暖机那一版（施加条数不为 0）：扫描报告的式子只写到挂载做成之后。
    JournalScanReportOfASecondCrashImage,
}

impl FieldNotJudged {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            FieldNotJudged::FloorOfASecondCrashAfterTheFloorRaisesBegan => {
                "二次崩溃崩在抬 F 开始之后的 F"
            }
            FieldNotJudged::FloorOfASecondCrashInAMountWhoseFloorRaisesWereNotLocated => {
                "抬 F 的界在挂载写表里取不出的那次挂载，二次崩溃的 F"
            }
            FieldNotJudged::JournalScanReportOfASecondCrashImage => "二次崩溃镜像上的扫描报告",
        }
    }
}

/// 一样字段比下来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldOutcome {
    Holds,
    Violated { expected: String, observed: String },
    NotJudged(FieldNotJudged),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldVerdict {
    pub field: MountTransparencyField,
    pub basis: FieldBasis,
    pub outcome: FieldOutcome,
}

/// 整个动作没判的理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MountTransparencyNotJudged {
    /// R1 没落到一条根、或落到了却没读出内容：射程只到挂载做成的状态，崩溃镜像本身那一截已经把它判红了。
    NothingWasRecoveredFromTheCrashImage,
}

impl MountTransparencyNotJudged {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MountTransparencyNotJudged::NothingWasRecoveredFromTheCrashImage => {
                "崩溃镜像上的只读恢复没落到一版"
            }
        }
    }
}

/// 一个动作的挂载透明判下来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MountTransparency {
    /// 逐样判过，按 [`MountTransparencyField::ALL`] 的次序（R2 没有可比的那一版时只有 POSIX 内容一样，判红）。
    Judged(Vec<FieldVerdict>),
    NotJudged(MountTransparencyNotJudged),
}

impl MountTransparency {
    /// 判红的那几样。
    #[must_use]
    pub fn violated_fields(&self) -> Vec<&FieldVerdict> {
        match self {
            MountTransparency::Judged(verdicts) => verdicts
                .iter()
                .filter(|verdict| match verdict.outcome {
                    FieldOutcome::Violated { .. } => true,
                    FieldOutcome::Holds | FieldOutcome::NotJudged(_) => false,
                })
                .collect(),
            MountTransparency::NotJudged(_) => Vec::new(),
        }
    }

    /// 这一样判下来是什么（没判到这一样时 None）。
    #[must_use]
    pub fn outcome_of(&self, field: MountTransparencyField) -> Option<&FieldOutcome> {
        match self {
            MountTransparency::Judged(verdicts) => verdicts
                .iter()
                .find(|verdict| verdict.field == field)
                .map(|verdict| &verdict.outcome),
            MountTransparency::NotJudged(_) => None,
        }
    }
}

/// R2 那一版与 R1 是哪一种关系：动作做成之后的那一版（实例是新取的号），还是二次崩溃之后落回 v1 那一版自己。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relation {
    WrittenByTheMount(FloorRelation),
    TheVersionOfTheFirstRecovery(FloorRelation),
}

/// F 那一样按哪一句判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FloorRelation {
    /// 挂载没推抬 F（或二次崩溃崩在抬 F 落盘之前）：生效值不变、R2 根里的 F = R1 的生效值。
    Unchanged,
    /// 挂载推了：R2 的 F 生效值 = 最后一串抬到的 F。
    RaisedTo(CheckpointTxg),
    NotJudged(FieldNotJudged),
}

fn held_or_violated(holds: bool, expected: String, observed: String) -> FieldOutcome {
    if holds {
        FieldOutcome::Holds
    } else {
        FieldOutcome::Violated { expected, observed }
    }
}

/// 不可变配置除本盘设备号之外逐项相同（本盘设备号是择到哪块盘的那一槽，不是池级字段）。按字段拆开写全：新加一个字段而这里没比，编译不过。
fn immutable_configuration_is_the_same_except_this_device(
    left: &SystemImmutableConfiguration,
    right: &SystemImmutableConfiguration,
) -> bool {
    let SystemImmutableConfiguration {
        filesystem_identifier: left_filesystem_identifier,
        this_device: _,
        device_count: left_device_count,
        region_devices: left_region_devices,
        sizes: left_sizes,
        journal_ring_start_slot: left_journal_ring_start_slot,
        root_ring_base_slot: left_root_ring_base_slot,
    } = left;
    let SystemImmutableConfiguration {
        filesystem_identifier: right_filesystem_identifier,
        this_device: _,
        device_count: right_device_count,
        region_devices: right_region_devices,
        sizes: right_sizes,
        journal_ring_start_slot: right_journal_ring_start_slot,
        root_ring_base_slot: right_root_ring_base_slot,
    } = right;
    (
        left_filesystem_identifier,
        left_device_count,
        left_region_devices,
        left_sizes,
        left_journal_ring_start_slot,
        left_root_ring_base_slot,
    ) == (
        right_filesystem_identifier,
        right_device_count,
        right_region_devices,
        right_sizes,
        right_journal_ring_start_slot,
        right_root_ring_base_slot,
    )
}

/// D18（块里携带什么信息） 已定项 11「恢复写的行」：恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行，
/// 所选根那个实例写 (i, 重放之后那个根的 checkpoint_txg, 这次重放施加的属于它的最大事务号)，严格介于中间的写 (i, 0, 0)；后写覆盖。
/// 重放不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1），所以所选根的实例就是 R1 那一版的实例。交回按实例排好的行；
/// R1 那一版的表读不出时 None。
fn instance_table_rows_written_by_the_recovery(
    first_recovery: &VersionFieldsRead,
    maximum_applied_transaction: u64,
    new_instance: InstanceGeneration,
) -> Option<Vec<InstanceRow>> {
    let mut rows: BTreeMap<InstanceGeneration, InstanceRow> = first_recovery
        .instance_table_rows
        .as_ref()?
        .iter()
        .map(|row| (row.instance, *row))
        .collect();
    let chosen_instance = first_recovery.root.instance;
    for instance in chosen_instance.0.max(1)..new_instance.0 {
        let instance = InstanceGeneration(instance);
        let row = if instance == chosen_instance {
            InstanceRow {
                instance,
                selected_root_txg: first_recovery.root.checkpoint_txg,
                applied_transaction_high_water: maximum_applied_transaction,
            }
        } else {
            InstanceRow {
                instance,
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
            }
        };
        rows.insert(instance, row);
    }
    Some(rows.into_values().collect())
}

fn rows_sorted_by_instance(rows: &[InstanceRow]) -> Vec<InstanceRow> {
    let mut sorted = rows.to_vec();
    sorted.sort_by_key(|row| row.instance);
    sorted
}

/// 挂载透明的逐字段判（r5 判决第二节 U5、U6 两行）。`before_the_mount` 是崩溃镜像上的只读恢复（R1，比恢复落到的那一版），
/// `after_the_action` 是动作之后的只读恢复（R2：只挂载与二次崩溃比恢复落到的那一版，挂载加一次发布比发布之前那一版）。
///
/// 动作做成之后的那一版：POSIX 内容与存在性相同；不可变配置（除本盘设备号）相同；卸载记号恒为「不是卸载写的」、R1 的记号不进比较；
/// 挂载没推抬 F 时 R2 的 F 生效值与 R2 根里的 F 都 = R1 的 F 生效值，推了时 R2 的 F 生效值 = 挂载交回的最后一串抬到的 F；
/// 实例代号 = max(R1 镜像上各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1；实例表行照 D18 已定项 11「恢复写的行」；
/// 树 ID 水位相同（挂载不发第一个文件）；两棵用户可见树的根指针相同——挂载续做过时（用户 2026-10-03 定「续做算挂载的一部分」）
/// 比挂载交回的续做那几步里 txg 不大于 R2 那一版的最近一步的（`continuation`，[`UserVisibleTreeRootPointersOfTheContinuation`]）；
/// txg ≥ max(根环全部根的 txg, 环里全部自证记录的 checkpoint_txg) + 1（只判下界：kb 允许跳号）；扫描报告施加 0 条、水位之上 0 条。
/// 二次崩溃之后落回 v1 那一版自己（实例与 txg 都是 R1 那一版的）：各样与 R1 那一版相同（推的）；落到这次挂载写的某一版：照上一段，
/// 扫描报告不判，崩在抬 F 开始之后的 F 不判。
///
/// # Panics
/// 调用方交错了比的版本：R1 不是比恢复落到的那一版，或 R2 拿的那一版与动作对不上。
#[must_use]
pub fn judge_mount_transparency(
    before_the_mount: &RecoveryFieldsRead,
    after_the_action: &RecoveryFieldsRead,
    action: MountTransparencyAction,
    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
) -> MountTransparency {
    assert_eq!(
        before_the_mount.compared,
        ComparedVersion::TheRecoveredVersion,
        "R1 比的是崩溃镜像上恢复落到的那一版"
    );
    let compared_for_the_action = match action {
        MountTransparencyAction::MountAlone(_)
        | MountTransparencyAction::SecondCrashInsideTheMount(_) => {
            ComparedVersion::TheRecoveredVersion
        }
        MountTransparencyAction::MountAndOnePublish(_) => {
            ComparedVersion::TheVersionBeforeTheRecoveredOne
        }
    };
    assert_eq!(
        after_the_action.compared, compared_for_the_action,
        "R2 拿来比的那一版要与动作对上：挂载加一次发布比发布之前那一版，别的比恢复落到的那一版"
    );
    let Some(first_recovery) = &before_the_mount.compared_version else {
        return MountTransparency::NotJudged(
            MountTransparencyNotJudged::NothingWasRecoveredFromTheCrashImage,
        );
    };
    if first_recovery.content == PosixContentRead::NothingRead {
        return MountTransparency::NotJudged(
            MountTransparencyNotJudged::NothingWasRecoveredFromTheCrashImage,
        );
    }
    let Some(compared) = &after_the_action.compared_version else {
        return MountTransparency::Judged(vec![FieldVerdict {
            field: MountTransparencyField::PosixContent,
            basis: MountTransparencyField::PosixContent.basis(action),
            outcome: FieldOutcome::Violated {
                expected: first_recovery.content.render(),
                observed: format!(
                    "动作之后的只读恢复没有可比的那一版（{:?}）",
                    after_the_action.report.outcome
                ),
            },
        }]);
    };
    let relation = match action {
        MountTransparencyAction::MountAlone(floor)
        | MountTransparencyAction::MountAndOnePublish(floor) => {
            Relation::WrittenByTheMount(match floor {
                FloorRaisedByTheMount::NotRaised => FloorRelation::Unchanged,
                FloorRaisedByTheMount::RaisedTo(raised) => FloorRelation::RaisedTo(raised),
            })
        }
        MountTransparencyAction::SecondCrashInsideTheMount(floor_raises) => {
            let floor = match floor_raises {
                FloorRaisesAtTheSecondCrash::NoneLandedYet => FloorRelation::Unchanged,
                FloorRaisesAtTheSecondCrash::SomeMayHaveLanded => FloorRelation::NotJudged(
                    FieldNotJudged::FloorOfASecondCrashAfterTheFloorRaisesBegan,
                ),
                FloorRaisesAtTheSecondCrash::BoundaryNotLocatedInTheMountWrites => {
                    FloorRelation::NotJudged(
                        FieldNotJudged::FloorOfASecondCrashInAMountWhoseFloorRaisesWereNotLocated,
                    )
                }
            };
            if (compared.root.instance, compared.root.checkpoint_txg)
                == (
                    first_recovery.root.instance,
                    first_recovery.root.checkpoint_txg,
                )
            {
                Relation::TheVersionOfTheFirstRecovery(floor)
            } else {
                Relation::WrittenByTheMount(floor)
            }
        }
    };
    let new_instance = before_the_mount
        .highest_system_configuration_instance
        .max(before_the_mount.highest_root_instance_in_the_ring)
        .and_then(|highest| highest.0.checked_add(1).map(InstanceGeneration));
    let verdicts = MountTransparencyField::ALL
        .into_iter()
        .map(|field| FieldVerdict {
            field,
            basis: field.basis(action),
            outcome: judge_one_field(
                field,
                before_the_mount,
                first_recovery,
                after_the_action,
                compared,
                relation,
                new_instance,
                action,
                continuation,
            ),
        })
        .collect();
    MountTransparency::Judged(verdicts)
}

#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：判哪一样、两次恢复与各自比的那一版、两版的关系、R1 推出的新实例、动作、挂载续做那几步的两根指针"
)]
fn judge_one_field(
    field: MountTransparencyField,
    before_the_mount: &RecoveryFieldsRead,
    first_recovery: &VersionFieldsRead,
    after_the_action: &RecoveryFieldsRead,
    compared: &VersionFieldsRead,
    relation: Relation,
    new_instance: Option<InstanceGeneration>,
    action: MountTransparencyAction,
    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
) -> FieldOutcome {
    match field {
        MountTransparencyField::PosixContent => held_or_violated(
            compared.content == first_recovery.content,
            first_recovery.content.render(),
            compared.content.render(),
        ),
        MountTransparencyField::ImmutableConfigurationWithoutThisDevice => {
            match (
                &before_the_mount.immutable_configuration,
                &after_the_action.immutable_configuration,
            ) {
                (Some(before), Some(after)) => held_or_violated(
                    immutable_configuration_is_the_same_except_this_device(before, after),
                    format!("{before:?}"),
                    format!("{after:?}"),
                ),
                (before, after) => FieldOutcome::Violated {
                    expected: format!("{before:?}"),
                    observed: format!("{after:?}"),
                },
            }
        }
        MountTransparencyField::UnmountMarker => match relation {
            Relation::WrittenByTheMount(_) => held_or_violated(
                compared.root.unmount_marker == UnmountMarker::NotWrittenByTheUnmountSequence,
                format!("{:?}", UnmountMarker::NotWrittenByTheUnmountSequence),
                format!("{:?}", compared.root.unmount_marker),
            ),
            Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
                compared.root.unmount_marker == first_recovery.root.unmount_marker,
                format!("{:?}", first_recovery.root.unmount_marker),
                format!("{:?}", compared.root.unmount_marker),
            ),
        },
        MountTransparencyField::RollbackFloor => {
            let effective_floor_before = before_the_mount.effective_rollback_floor;
            // 挂载写的那一版根里的 F 照 R1 的生效值写；落回 v1 那一版自己时根就是 v1 的根。
            let (floor, expected_floor_in_the_root) = match relation {
                Relation::WrittenByTheMount(floor) => (floor, effective_floor_before),
                Relation::TheVersionOfTheFirstRecovery(floor) => {
                    (floor, Some(first_recovery.root.rollback_floor))
                }
            };
            match floor {
                FloorRelation::NotJudged(reason) => FieldOutcome::NotJudged(reason),
                FloorRelation::Unchanged => held_or_violated(
                    after_the_action.effective_rollback_floor == effective_floor_before
                        && Some(compared.root.rollback_floor) == expected_floor_in_the_root,
                    format!(
                        "F 生效值是 R1 的 {effective_floor_before:?}，R2 根里的 F 是 {expected_floor_in_the_root:?}"
                    ),
                    format!(
                        "R2 的 F 生效值 {:?}、R2 根里的 F {:?}",
                        after_the_action.effective_rollback_floor, compared.root.rollback_floor
                    ),
                ),
                FloorRelation::RaisedTo(raised) => held_or_violated(
                    after_the_action.effective_rollback_floor == Some(raised),
                    format!("F 生效值是挂载交回的最后一串抬到的 F {raised:?}"),
                    format!("R2 的 F 生效值 {:?}", after_the_action.effective_rollback_floor),
                ),
            }
        }
        MountTransparencyField::InstanceGeneration => match relation {
            Relation::WrittenByTheMount(_) => held_or_violated(
                Some(compared.root.instance) == new_instance,
                format!(
                    "max(系统配置 {:?}, 根环 {:?}) + 1 = {new_instance:?}",
                    before_the_mount.highest_system_configuration_instance,
                    before_the_mount.highest_root_instance_in_the_ring
                ),
                format!("{:?}", compared.root.instance),
            ),
            Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
                compared.root.instance == first_recovery.root.instance,
                format!("{:?}", first_recovery.root.instance),
                format!("{:?}", compared.root.instance),
            ),
        },
        MountTransparencyField::InstanceTableRows => {
            let expected = match relation {
                Relation::WrittenByTheMount(_) => new_instance.and_then(|new_instance| {
                    instance_table_rows_written_by_the_recovery(
                        first_recovery,
                        before_the_mount.report.journal.maximum_applied_transaction,
                        new_instance,
                    )
                }),
                Relation::TheVersionOfTheFirstRecovery(_) => first_recovery
                    .instance_table_rows
                    .as_deref()
                    .map(rows_sorted_by_instance),
            };
            let observed = compared
                .instance_table_rows
                .as_deref()
                .map(rows_sorted_by_instance);
            held_or_violated(
                expected.is_some() && expected == observed,
                format!("{expected:?}"),
                format!("{observed:?}"),
            )
        }
        MountTransparencyField::TreeIdentifierWatermark => held_or_violated(
            compared.root.tree_identifier_watermark
                == first_recovery.root.tree_identifier_watermark,
            first_recovery.root.tree_identifier_watermark.to_string(),
            compared.root.tree_identifier_watermark.to_string(),
        ),
        MountTransparencyField::UserVisibleTreeRootPointers => {
            let expected = match relation {
                Relation::WrittenByTheMount(_) => continuation
                    .of_the_last_step_at_or_before((
                        compared.root.instance,
                        compared.root.checkpoint_txg,
                    ))
                    .or(first_recovery.user_visible_tree_root_pointers.as_ref()),
                Relation::TheVersionOfTheFirstRecovery(_) => {
                    first_recovery.user_visible_tree_root_pointers.as_ref()
                }
            };
            held_or_violated(
                expected.is_some() && compared.user_visible_tree_root_pointers.as_ref() == expected,
                format!("{expected:?}"),
                format!("{:?}", compared.user_visible_tree_root_pointers),
            )
        }
        MountTransparencyField::CheckpointTxg => match relation {
            Relation::WrittenByTheMount(_) => {
                let lower_bound = before_the_mount
                    .highest_root_txg_in_the_ring
                    .max(before_the_mount.highest_record_checkpoint_txg_in_the_ring)
                    .and_then(|highest| highest.0.checked_add(1).map(CheckpointTxg));
                held_or_violated(
                    lower_bound
                        .is_some_and(|lower_bound| compared.root.checkpoint_txg >= lower_bound),
                    format!(
                        "≥ max(根环 {:?}, 记录 {:?}) + 1",
                        before_the_mount.highest_root_txg_in_the_ring,
                        before_the_mount.highest_record_checkpoint_txg_in_the_ring
                    ),
                    format!("{:?}", compared.root.checkpoint_txg),
                )
            }
            Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
                compared.root.checkpoint_txg == first_recovery.root.checkpoint_txg,
                format!("{:?}", first_recovery.root.checkpoint_txg),
                format!("{:?}", compared.root.checkpoint_txg),
            ),
        },
        MountTransparencyField::JournalScanReport => match action {
            MountTransparencyAction::SecondCrashInsideTheMount(_) => {
                FieldOutcome::NotJudged(FieldNotJudged::JournalScanReportOfASecondCrashImage)
            }
            MountTransparencyAction::MountAlone(_)
            | MountTransparencyAction::MountAndOnePublish(_) => {
                let journal = after_the_action.report.journal;
                held_or_violated(
                    journal.prefix_applied == 0 && journal.above_water == 0,
                    "施加 0 条、水位之上 0 条".to_string(),
                    format!(
                        "施加 {} 条、水位之上 {} 条",
                        journal.prefix_applied, journal.above_water
                    ),
                )
            }
        },
    }
}

/// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 取甲，用户 2026-09-28 定）上的拒可写理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalReasonOnTheD23List {
    /// 读阶段判据 N-配置 为真（有一槽读不出或本池自证不过按真；取号那一刻有一槽读不出也报成它），重读一次仍为真。
    NewerPublishWitnessedBySystemConfiguration,
    /// 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次仍读不出。
    InstanceTableOfTheNewestRootUnreadable,
}

/// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
#[must_use]
pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReasonOnTheD23List> {
    match error {
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
                    Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)
                }
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
                    Some(RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable)
                }
                // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
            }
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
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
        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => None,
    }
}

/// 判据 N-配置（D23 已定项 14「可写挂载读到的更新状态读不出」那一段）在崩溃镜像上自己复算一遍，不经实现的 `mount::newer_publish_witness`：
/// 每块盘两槽有一槽读不出或本池自证不过 ⇒ 判不出、按真；全部槽的 journal tail 取最大为 c_见证，0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒
/// c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
/// `selected_version` 是 R1 落到的那一版（与可写挂载读阶段同一份盘、同一套择根与重放）；没有时判据无从复算，交回假。
fn newer_publish_witnessed_on_the_crash_image(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> bool {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return true;
    };
    let immutable = system_configuration.immutable;
    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
    let mut witnessed_journal_counter = 0u64;
    for device in crash_image.device_identities() {
        let slots = verified_system_configuration_slots(
            crash_image,
            device,
            slot_spacing,
            &immutable.filesystem_identifier,
        );
        if u64::try_from(slots.len()).expect("一块盘的系统配置槽数装得进 u64")
            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
        {
            return true;
        }
        witnessed_journal_counter = slots
            .iter()
            .map(|slot| slot.quantities.journal_tail)
            .fold(witnessed_journal_counter, u64::max);
    }
    witnessed_counter_holds_against_the_selected_version(
        crash_image,
        &system_configuration,
        witnessed_journal_counter,
        selected_version,
    )
    .is_some()
}

/// 判据 N-配置 在 c_见证 与所选那一版比出来为真的那一支（D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」那一段的后半）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessedCounterBranchThatHolds {
    /// 所选那一版那次发布带末条标志的记录读得出，c_见证 大于它的计数器。
    WitnessedCounterAboveTheLastRecordOfTheSelectedVersion,
    /// 所选那一版的末条读不出，计数器等于 c_见证 的记录读得出、它的 (实例代号, checkpoint_txg) 大于所选那一版。
    RecordAtTheWitnessedCounterNewerThanTheSelectedVersion,
    /// 两条都读不出，按真。
    NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable,
}

/// c_见证 已经取好之后的那一半：0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录读得出 ⇒
/// 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。为真交回是哪一支，为假（或没有所选那一版、判据无从复算）交回 `None`。
fn witnessed_counter_holds_against_the_selected_version(
    crash_image: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    witnessed_journal_counter: u64,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<WitnessedCounterBranchThatHolds> {
    if witnessed_journal_counter == 0 {
        return None;
    }
    let selected_version = selected_version?;
    let records = scan_journal(crash_image, system_configuration);
    let last_record_counter_of_the_selected_version = records
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == selected_version
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max();
    if let Some(last_record_counter) = last_record_counter_of_the_selected_version {
        return (witnessed_journal_counter > last_record_counter).then_some(
            WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion,
        );
    }
    let record_at_the_witnessed_counter = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .map(|record| (record.instance, record.checkpoint_txg))
        .max();
    match record_at_the_witnessed_counter {
        Some(record) => (record > selected_version).then_some(
            WitnessedCounterBranchThatHolds::RecordAtTheWitnessedCounterNewerThanTheSelectedVersion,
        ),
        None => Some(
            WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable,
        ),
    }
}

/// C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：判据 N-配置 在没注入读故障的崩溃镜像上复算，为真交回是哪一支。
/// 没有读故障时设备不报读错，见证读判得出（R2，D22（单元原子性怎么合成） 已定项 8 射程：判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过；
/// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过。
/// 撕裂系统配置那一支因此不落在这里（C664（挂载拒绝合法复算把两种拒因报成一个）、C331（择根倒挂压过已确认的写））：
/// [`newer_publish_witnessed_on_the_crash_image`] 照甲把它判成按真，那一支的拒照 U6 判。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
fn witnessed_counter_holds_on_a_crash_image_without_read_faults(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<WitnessedCounterBranchThatHolds> {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return None;
    };
    let slot_spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let witnessed_journal_counter = crash_image
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                crash_image,
                device,
                slot_spacing,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    witnessed_counter_holds_against_the_selected_version(
        crash_image,
        &system_configuration,
        witnessed_journal_counter,
        selected_version,
    )
}

/// 「重建分配器判抛弃用的最新那条根的实例表读不出」在崩溃镜像上复算：根环里择到的最新那条根指着的实例表沿链读不出、解不开（择不到根也算）。
fn instance_table_of_the_newest_root_unreadable_on_the_crash_image(
    crash_image: &dyn PoolReader,
) -> bool {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return true;
    };
    choose_root(crash_image, &system_configuration)
        .is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())
}

/// 拒之后盘上逐字节同不同（调用方比拒之前与拒之后的整份镜像）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskAfterTheRefusal {
    ByteForByteUnchanged,
    Changed,
}

/// 崩溃镜像上只读恢复读出的，在不在允许集合里（调用方拿理想模型判的那一截的结论交进来）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadOnlyReadBackOfTheCrashImage {
    InsideTheAllowedSet,
    OutsideTheAllowedSet,
}

/// 崩溃态注没注入过读故障（D13（验证路线） 已定项 1：挂载透明的射程只管不带故障的状态；C670（无读故障的崩溃态上可写挂载见证判据为真没有判官））。
/// 没注入：崩溃镜像读得出的就是盘上的字节，造出这份盘面的历史里也没有一步在读故障下发过写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFaultsInTheCrashState {
    NoneInjected,
    Injected,
}

/// 单子上的理由，合法还差哪一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalConditionNotMet {
    /// 那一条判据在崩溃镜像上复算为假：拒的理由是错的。
    CriterionFalseWhenRecomputedOnTheCrashImage,
    DiskChangedByTheRefusedMount,
    ReadOnlyReadBackOutsideTheAllowedSet,
    /// C670：崩溃态没注入读故障，判据 N-配置 在崩溃镜像上（撕裂的系统配置槽跳过）复算仍为真——正确实现的可写挂载在这份盘面上永远拒，
    /// 一次普通断电之后池只剩只读挂载；这样的崩溃态不许出现，拒是对的、造出这份盘面的那一段不对。带的是判据为真的那一支。
    WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterBranchThatHolds),
}

/// 可写挂载被拒，拒得合不合法（r5 判决第二节 U6：拒绝合法 ⇔ 理由在 D23 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、
/// 盘上逐字节不变、只读读出在允许集合里）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WritableMountRefusalVerdict {
    Legal(RefusalReasonOnTheD23List),
    Illegal {
        reason: RefusalReasonOnTheD23List,
        conditions_not_met: Vec<RefusalConditionNotMet>,
    },
    /// 理由不在单子上：照崩溃注入第二截今天的写法判（单元区墙计数，别的报红）。
    ReasonNotOnTheD23List,
}

/// 判一次拒可写合不合法，只判 r5 判决 U6 那四条，不判 C670 那一条（崩溃态知道注没注入读故障的调用方用
/// [`judge_writable_mount_refusal_of_a_crash_state`]）。拒的判据在崩溃镜像（`crash_image`）上自己复算（不经 O1 判官，也不经实现判拒的那一段）；
/// `selected_version` 是崩溃镜像上只读恢复落到的那一版（R1 恢复报告里的 `effective_root`）。
#[must_use]
pub fn judge_writable_mount_refusal(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    refusal: &MountError,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
) -> WritableMountRefusalVerdict {
    let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {
        return WritableMountRefusalVerdict::ReasonNotOnTheD23List;
    };
    verdict_of_a_refusal_on_the_d23_list(
        reason,
        conditions_not_met_by_a_refusal_on_the_d23_list(
            crash_image,
            selected_version,
            reason,
            disk,
            read_only_read_back,
        ),
    )
}

/// 判崩溃注入第二截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
/// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
/// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态不判这一条
/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
#[must_use]
pub fn judge_writable_mount_refusal_of_a_crash_state(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    refusal: &MountError,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    read_faults: ReadFaultsInTheCrashState,
) -> WritableMountRefusalVerdict {
    let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {
        return WritableMountRefusalVerdict::ReasonNotOnTheD23List;
    };
    let mut conditions_not_met = conditions_not_met_by_a_refusal_on_the_d23_list(
        crash_image,
        selected_version,
        reason,
        disk,
        read_only_read_back,
    );
    match (reason, read_faults) {
        (
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            ReadFaultsInTheCrashState::NoneInjected,
        ) => {
            if let Some(branch) = witnessed_counter_holds_on_a_crash_image_without_read_faults(
                crash_image,
                selected_version,
            ) {
                conditions_not_met.push(
                    RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
                        branch,
                    ),
                );
            }
        }
        (
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            ReadFaultsInTheCrashState::Injected,
        )
        | (
            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
        ) => {}
    }
    verdict_of_a_refusal_on_the_d23_list(reason, conditions_not_met)
}

/// 理由在单子上（U6 第一条）之后，U6 另三条里没满足的：判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。
fn conditions_not_met_by_a_refusal_on_the_d23_list(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    reason: RefusalReasonOnTheD23List,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
) -> Vec<RefusalConditionNotMet> {
    let criterion_holds_on_the_crash_image = match reason {
        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration => {
            newer_publish_witnessed_on_the_crash_image(crash_image, selected_version)
        }
        RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable => {
            instance_table_of_the_newest_root_unreadable_on_the_crash_image(crash_image)
        }
    };
    let mut conditions_not_met = Vec::new();
    if !criterion_holds_on_the_crash_image {
        conditions_not_met
            .push(RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);
    }
    match disk {
        DiskAfterTheRefusal::ByteForByteUnchanged => {}
        DiskAfterTheRefusal::Changed => {
            conditions_not_met.push(RefusalConditionNotMet::DiskChangedByTheRefusedMount);
        }
    }
    match read_only_read_back {
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet => {}
        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet => {
            conditions_not_met.push(RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet);
        }
    }
    conditions_not_met
}

fn verdict_of_a_refusal_on_the_d23_list(
    reason: RefusalReasonOnTheD23List,
    conditions_not_met: Vec<RefusalConditionNotMet>,
) -> WritableMountRefusalVerdict {
    if conditions_not_met.is_empty() {
        WritableMountRefusalVerdict::Legal(reason)
    } else {
        WritableMountRefusalVerdict::Illegal {
            reason,
            conditions_not_met,
        }
    }
}
