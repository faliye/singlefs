//! 挂载透明（独立的第四条，不是 M3 的第三读：`research/prompts/m3-formal-r4-main-verification.md` 第四节第 3 条；
//! 逐字段表与二次崩溃、拒绝合法照 `research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条，零轮）的判法。
//!
//! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F，
//! 连同挂载续做与续做里推的抬 F：用户 2026-10-03 定「续做算挂载的一部分」），
//! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
//! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]；
//! 抬 F 的界在挂载写表里对不上位置的那一次挂载，二次崩溃整段 F 不判、按 [`FieldNotJudged`] 计数）。
//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]；崩溃注入第二、三截再加两条：
//! 没注入读故障的崩溃态上判据 N-配置 复算为真（C670），或「读阶段判完之后系统配置又读不出」那一拒的判据复算为真（改法 G4，射程只到 mkfs 返回之后的崩溃态），
//! 正确实现摆不出这份盘面，拒得不合法，[`judge_writable_mount_refusal_of_a_crash_state`]）。
//!
//! 读盘的只有 [`read_recovery_fields`] 与那两个判拒绝的函数里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
//! 只吃读好的字段与挂载交回值。checker 档的崩溃注入调它，harness 档的快档用例也调它（harness 档不依赖 checker 档）。
//! r5 判决标「推的」的几样（树 ID 水位、两个根指针、txg、扫描报告）照写进判法，每一样红的时候在交回值里单独标出它是推的还是量过的
//! （[`MountTransparencyField::basis`]）。

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::verification_read::{
    choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,
    effective_rollback_floor, highest_root_instance, highest_root_txg, instance_table_of_root,
    readable_roots, root_record_bytes_of_the_version, scan_journal, tree_table_entry_bytes_of_root,
    verified_system_configuration_slots,
};
use singlefs_checker::crc32_castagnoli_table as crc32_castagnoli;
use singlefs_checker::image::{system_configuration_redundancy_report, ImageReader};
use singlefs_checker::index_node_view as parse_index_node;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish, NamedUnit};
use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
    TREE_KIND_RETIREMENT, TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{
    DATA_UNIT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};

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

/// 拿来比的那一版自己的字段：根记录（解出来的与原字节）、读出的内容、它指着的实例表的行、树表条目的原字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionFieldsRead {
    pub root: RootRecord,
    /// 根记录 [0, `ROOT_RECORD_BYTES`) 的原字节（验证侧独立读取根环里那一槽，
    /// 由记录施加出来、根槽没落盘的那一版是按字段表编出来的）。[`ROOT_RECORD_FIELD_BYTE_RANGES`] 按它逐段比。
    pub root_record_bytes: Vec<u8>,
    pub content: PosixContentRead,
    /// 实例表沿链有一片读不出、解不开时 None。
    pub instance_table_rows: Option<Vec<InstanceRow>>,
    /// 验证侧独立解析的树表条目原字节，按盘上次序；树表读不出、解不开时 None。
    /// [`TREE_TABLE_ENTRY_FIELD_BYTE_RANGES`] 按它逐条逐段比，树表条目的根指针那一样也从这里取。
    pub tree_table_entries: Option<Vec<Vec<u8>>>,
}

/// 一个池上的一次只读恢复（看 journal）连同挂载透明要比的字段（[`read_recovery_fields`] 读出来的）。
/// 字段没有不变量，直接公开：harness 的快档用例改坏其中一样，证判法会红。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryFieldsRead {
    /// 那一次只读恢复的报告（`recover(reader, JournalPolicy::Consult)`）。
    pub report: RecoveryReport,
    pub compared: ComparedVersion,
    /// 拿来比的那一版；恢复没落到一条根、或要比的是发布之前那一版而根环里没有比恢复落到的那一版更旧的根时 None。
    pub compared_version: Option<VersionFieldsRead>,
    /// 择到的系统配置的不可变配置（择不到时 None）。
    pub immutable_configuration: Option<SystemImmutableConfiguration>,
    /// 验证侧复算的 F 生效值（D16（发布语义） 已定项 1「生效」）。
    pub effective_rollback_floor: Option<CheckpointTxg>,
    /// 每块盘两槽里全部自证过（校验和过、fsid 与本池相同）的系统配置的实例代号的最大值（取号的 max 那一半，D18（块里携带什么信息） 已定项 11）。
    pub highest_system_configuration_instance: Option<InstanceGeneration>,
    /// 根环里全部可读根的实例代号的最大值（取号的 max 另一半）。
    pub highest_root_instance_in_the_ring: Option<InstanceGeneration>,
    /// 根环里全部可读根的 checkpoint_txg 的最大值（新实例第一次发布的 txg 下界那一半，D23（journal 的角色与格式） 已定项 14 注 3）。
    pub highest_root_txg_in_the_ring: Option<CheckpointTxg>,
    /// 环里全部自证通过的 journal 记录的 checkpoint_txg 的最大值（txg 下界另一半）。
    pub highest_record_checkpoint_txg_in_the_ring: Option<CheckpointTxg>,
    /// 环里全部自证通过的 journal 记录的计数器的最大值（系统配置 journal_tail 那一样的上界）；环里一条都没有时 None。
    pub highest_record_counter_in_the_ring: Option<u64>,
    /// 验证侧择到的系统配置槽 [0, `SYSTEM_CONFIGURATION_BYTES`) 的原字节；择不到时 None。
    /// [`SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES`] 按它逐段比，journal_tail、journal 实例代号、系统运行配置三样也从这里取。
    pub chosen_system_configuration_bytes: Option<Vec<u8>>,
}

/// 对 `reader` 跑一次只读恢复（看 journal），再读挂载透明要比的字段。`compared` 定拿哪一版来比（[`ComparedVersion`]）。
///
/// 恢复报告只提供实际落到的版本身份和内容；配置、根、journal、实例表、树表字段由验证侧读法独立解析。
/// 这里观察被测恢复的结果，不把再调一次实现的重放/判退当作独立参照。
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
        highest_record_counter_in_the_ring: None,
        chosen_system_configuration_bytes: None,
    };
    let Ok(system_configuration) = choose_system_configuration(reader) else {
        return fields;
    };
    let system_configuration_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("系统配置字段表 489 字节装得进 usize");
    fields.chosen_system_configuration_bytes = chosen_system_configuration_slot_bytes(reader)
        .map(|slot| slot[..system_configuration_bytes].to_vec());
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
    fields.highest_record_counter_in_the_ring = records.values().map(|record| record.counter).max();
    let Some(recovered_root) = fields.report.effective_root.and_then(|version| {
        crate::verification_read::root_of_observed_version(reader, &system_configuration, version)
    }) else {
        return fields;
    };
    fields.compared_version = match compared {
        ComparedVersion::TheRecoveredVersion => Some(version_fields(
            reader,
            &immutable,
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
            let content = match crate::verification_read::first_file_content(
                reader,
                &before,
                &immutable.sizes,
            ) {
                Some(Some(content)) => PosixContentRead::File(content),
                Some(None) => PosixContentRead::NoFile,
                None => PosixContentRead::NothingRead,
            };
            version_fields(reader, &immutable, before, content)
        }),
    };
    fields
}

fn version_fields(
    reader: &dyn PoolReader,
    immutable: &SystemImmutableConfiguration,
    root: RootRecord,
    content: PosixContentRead,
) -> VersionFieldsRead {
    VersionFieldsRead {
        root,
        root_record_bytes: root_record_bytes_of_the_version(
            reader,
            &immutable.region_devices,
            &immutable.sizes,
            &root,
        ),
        content,
        instance_table_rows: instance_table_of_root(reader, &root).map(|table| table.rows),
        tree_table_entries: tree_table_entry_bytes_of_root(reader, &root).ok(),
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

/// 树表里只由挂载续做改根的那六种树（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1：墓碑、谱系、
/// deleted_inodes 三根并进「树表里 inode 树、extent 树的根指针」那一行，同一条续做规则）：extent、inode、deleted_inodes、墓碑树、谱系树，
/// 与退役树（续做的最后一批身份消失时追加，挂载本身不改它；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条，
/// 正推 K1.2 那一行）。
pub const TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES: [u16; 6] = [
    TREE_KIND_EXTENT,
    TREE_KIND_INODE,
    TREE_KIND_DELETED_INODES,
    TREE_KIND_TOMBSTONE,
    TREE_KIND_LINEAGE,
    TREE_KIND_RETIREMENT,
];

/// 树表里根恒全零、挂载前后必须相同的那四种树（同上判决 Y1「树表种类 6、7、8、12 的根」那一行）：livelist、稀疏旁表、deadlist、logged_ops。
/// 种类 3、4（分配记录树、记账树）的根每一版都重写，不比，归「分配记录」那一行（池级 checker 判 R2 的镜像）。
pub const TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS: [u16; 4] = [
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_LOGGED_OPERATIONS,
];

/// 一版树表里按种类取的条目根指针（条目偏移 14 起 86 字节的盘上原样）；同一种有几条时取盘上先出现的那条（同一种两条由池级 checker 的
/// I-9.16 ③ 判，这里不另判）。只装调用方点名的那几种。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeTableEntryRootPointersByKind {
    pub root_pointer_of_each_kind: BTreeMap<u16, Vec<u8>>,
}

impl TreeTableEntryRootPointersByKind {
    /// 从树表条目的原字节取 `kinds` 那几种的根指针。条目宽不到种类那 2 字节或根指针那 86 字节的条目跳过（宽度由树表字段表那一段比）。
    #[must_use]
    pub fn of_the_entries(entries: &[Vec<u8>], kinds: &[u16]) -> Self {
        let mut root_pointer_of_each_kind = BTreeMap::new();
        for entry in entries {
            let (Some(kind_bytes), Some(root_pointer)) = (
                entry.get(TREE_TABLE_ENTRY_KIND.clone()),
                entry.get(TREE_TABLE_ENTRY_ROOT_POINTER.clone()),
            ) else {
                continue;
            };
            let kind = u16::from_le_bytes(kind_bytes.try_into().expect("切的是种类那 2 字节"));
            if kinds.contains(&kind) {
                root_pointer_of_each_kind
                    .entry(kind)
                    .or_insert_with(|| root_pointer.to_vec());
            }
        }
        TreeTableEntryRootPointersByKind {
            root_pointer_of_each_kind,
        }
    }
}

/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里只由续做改根的那六种树
/// （[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）的根指针（用户 2026-10-03 定「续做算挂载的一部分」）：
/// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的；续做的每一步改了它们，之后挂载写的版本
/// （续做推的抬 F 的空发布）照抄最近一步的。挂载没续做时一步都没有，挂载写的每一版都比 R1 那一版。
/// 名字里的「用户可见」是只装 inode 树、extent 树两种时起的：checker 档的崩溃注入（`crash_injection.rs`）照这个名字调它，改名随那一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserVisibleTreeRootPointersOfTheContinuation {
    /// (那一步的实例代号与 checkpoint_txg, 五种树的根指针)，按 txg 升序（构造时排）。
    steps: Vec<(
        (InstanceGeneration, CheckpointTxg),
        TreeTableEntryRootPointersByKind,
    )>,
}

impl UserVisibleTreeRootPointersOfTheContinuation {
    /// 挂载没续做（`MountOutput::deletion_queue_continuation` 空）。
    pub const NOT_CONTINUED: Self = Self { steps: Vec::new() };

    /// 根指针取续做每一步交回的树表条目（`TransactionOutput::tree_table_entries`）编成盘上字节之后那 86 字节，与只读恢复读盘上树表条目
    /// 的取法（[`TreeTableEntryRootPointersByKind::of_the_entries`]）同一段字节。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
        let mut steps: Vec<(
            (InstanceGeneration, CheckpointTxg),
            TreeTableEntryRootPointersByKind,
        )> = output
            .deletion_queue_continuation
            .iter()
            .map(|step| {
                (
                    (step.root.instance, step.root.checkpoint_txg),
                    root_pointers_the_continuation_rewrites_in_a_published_version(step),
                )
            })
            .collect();
        steps.sort_by_key(|(version, _)| (version.1, version.0));
        UserVisibleTreeRootPointersOfTheContinuation { steps }
    }

    /// 挂载写的 `version` 那一版该带的五种树的根指针：同一个实例里 txg 不大于它的最近一步续做那一版的；它在续做之前（或挂载没续做）时 None，
    /// 调用方比 R1 那一版。
    fn of_the_last_step_at_or_before(
        &self,
        version: (InstanceGeneration, CheckpointTxg),
    ) -> Option<&TreeTableEntryRootPointersByKind> {
        self.steps
            .iter()
            .filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)
            .map(|(_, pointers)| pointers)
            .next_back()
    }
}

/// 一次发布交回的树表条目里只由续做改根的那五种树的根指针盘上字节；树表里没有那种树的条目时那一种不在表里。
fn root_pointers_the_continuation_rewrites_in_a_published_version(
    version: &TransactionOutput,
) -> TreeTableEntryRootPointersByKind {
    let entries: Vec<Vec<u8>> = version
        .tree_table_entries
        .iter()
        .map(crate::verification_read::encoded_tree_table_entry)
        .collect();
    TreeTableEntryRootPointersByKind::of_the_entries(
        &entries,
        &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
    )
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

/// 比的十八样：r5 判决第二节 U5 那一行的逐字段表十样，加判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
/// 第二节 Y1、Y2 补的五样关系与三张按字节区间的字段表。
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
    /// 树表条目的根指针：种类 1、2、9、10、11、13（[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）照续做规则，
    /// 种类 6、7、8、12（[`TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS`]）与 R1 相同，种类 3、4 不比。
    TreeTableEntryRootPointers,
    CheckpointTxg,
    /// 扫描报告（施加了几条、水位之上几条）。
    JournalScanReport,
    /// 根记录里中央映射树根指针（偏移 256 起 86 字节）：挂载写的那一版 R2 全零 ⇔ R1 全零；落回 v1 时逐字节相同。
    MappingRootEmptiness,
    /// 根记录里树表 0 条那一版的分配记录树根（偏移 342 起 86 字节）：挂载写的那一版 R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒
    /// R2 非零 ⇔ 这次挂载取到的新实例号 ≥ 2；落回 v1 时逐字节相同。
    AllocationRecordTreeRoot,
    /// 系统配置 journal_tail（偏移 469）：R1 的 ≤ R2 的 ≤ R2 环里自证记录的最大计数器，挂载做完（只挂载、挂载加一次发布）取等。
    SystemConfigurationJournalTail,
    /// 系统配置的 journal 实例代号（偏移 477）：挂载做完 = R2 那一版根的实例代号；二次崩溃 ∈ {R1 的, 新实例}。
    SystemConfigurationJournalInstance,
    /// 系统运行配置 36 字节（偏移 433 起）：R2 = 内置默认（今天没有挂载选项通道，写槽总写默认）。
    SystemRuntimeConfiguration,
    /// 根记录按字节区间（[`ROOT_RECORD_FIELD_BYTE_RANGES`]）：「必须相同」「必须为 0」的那几段。
    RootRecordBytes,
    /// 系统配置按字节区间（[`SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES`]）：「必须相同」「必须为 0」的那几段。
    SystemConfigurationBytes,
    /// 树表条目按字节区间（[`TREE_TABLE_ENTRY_FIELD_BYTE_RANGES`]），逐条：「必须相同」「必须为 0」的那几段。
    TreeTableEntryBytes,
}

/// 一样字段的关系是攻方在副本装置上量过（主 agent 复跑坐实）的，还是 r5 判决标「推的」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldBasis {
    MeasuredByTheSecondRoundAttack,
    InferredNotMeasured,
}

impl MountTransparencyField {
    pub const ALL: [MountTransparencyField; 18] = [
        MountTransparencyField::PosixContent,
        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
        MountTransparencyField::UnmountMarker,
        MountTransparencyField::RollbackFloor,
        MountTransparencyField::InstanceGeneration,
        MountTransparencyField::InstanceTableRows,
        MountTransparencyField::TreeIdentifierWatermark,
        MountTransparencyField::TreeTableEntryRootPointers,
        MountTransparencyField::CheckpointTxg,
        MountTransparencyField::JournalScanReport,
        MountTransparencyField::MappingRootEmptiness,
        MountTransparencyField::AllocationRecordTreeRoot,
        MountTransparencyField::SystemConfigurationJournalTail,
        MountTransparencyField::SystemConfigurationJournalInstance,
        MountTransparencyField::SystemRuntimeConfiguration,
        MountTransparencyField::RootRecordBytes,
        MountTransparencyField::SystemConfigurationBytes,
        MountTransparencyField::TreeTableEntryBytes,
    ];

    /// 这一样在这个动作下的关系是量过的还是推的。二次崩溃那一截的字段一半整截是推的（r5 判决第二节 U6、攻方 U6-1），
    /// F 在挂载推了抬 F 时那一半（「= 挂载交回的最后一串抬到的 F」）r5 判决写的是「后半没量」。
    /// c655 那一轮补的八样（五样关系、三张字节表）一律记推的：攻方量的是副本装置，判决第五节列它们为零轮，入库代码上还没量过。
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
                | MountTransparencyField::TreeTableEntryRootPointers
                | MountTransparencyField::CheckpointTxg
                | MountTransparencyField::JournalScanReport
                | MountTransparencyField::MappingRootEmptiness
                | MountTransparencyField::AllocationRecordTreeRoot
                | MountTransparencyField::SystemConfigurationJournalTail
                | MountTransparencyField::SystemConfigurationJournalInstance
                | MountTransparencyField::SystemRuntimeConfiguration
                | MountTransparencyField::RootRecordBytes
                | MountTransparencyField::SystemConfigurationBytes
                | MountTransparencyField::TreeTableEntryBytes,
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
                MountTransparencyField::TreeTableEntryRootPointers,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：树表条目的根指针（量过）",
            (
                MountTransparencyField::TreeTableEntryRootPointers,
                FieldBasis::InferredNotMeasured,
            ) => "挂载透明：树表条目的根指针（推的）",
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
            (
                MountTransparencyField::MappingRootEmptiness,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：中央映射树根的空性（量过）",
            (MountTransparencyField::MappingRootEmptiness, FieldBasis::InferredNotMeasured) => {
                "挂载透明：中央映射树根的空性（推的）"
            }
            (
                MountTransparencyField::AllocationRecordTreeRoot,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：根记录里的分配记录树根（量过）",
            (MountTransparencyField::AllocationRecordTreeRoot, FieldBasis::InferredNotMeasured) => {
                "挂载透明：根记录里的分配记录树根（推的）"
            }
            (
                MountTransparencyField::SystemConfigurationJournalTail,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：系统配置 journal_tail（量过）",
            (
                MountTransparencyField::SystemConfigurationJournalTail,
                FieldBasis::InferredNotMeasured,
            ) => "挂载透明：系统配置 journal_tail（推的）",
            (
                MountTransparencyField::SystemConfigurationJournalInstance,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：系统配置的 journal 实例代号（量过）",
            (
                MountTransparencyField::SystemConfigurationJournalInstance,
                FieldBasis::InferredNotMeasured,
            ) => "挂载透明：系统配置的 journal 实例代号（推的）",
            (
                MountTransparencyField::SystemRuntimeConfiguration,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：系统运行配置（量过）",
            (
                MountTransparencyField::SystemRuntimeConfiguration,
                FieldBasis::InferredNotMeasured,
            ) => "挂载透明：系统运行配置（推的）",
            (
                MountTransparencyField::RootRecordBytes,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：根记录按字节区间（量过）",
            (MountTransparencyField::RootRecordBytes, FieldBasis::InferredNotMeasured) => {
                "挂载透明：根记录按字节区间（推的）"
            }
            (
                MountTransparencyField::SystemConfigurationBytes,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：系统配置按字节区间（量过）",
            (MountTransparencyField::SystemConfigurationBytes, FieldBasis::InferredNotMeasured) => {
                "挂载透明：系统配置按字节区间（推的）"
            }
            (
                MountTransparencyField::TreeTableEntryBytes,
                FieldBasis::MeasuredByTheSecondRoundAttack,
            ) => "挂载透明：树表条目按字节区间（量过）",
            (MountTransparencyField::TreeTableEntryBytes, FieldBasis::InferredNotMeasured) => {
                "挂载透明：树表条目按字节区间（推的）"
            }
        }
    }
}

/// 一段字节区间挂载前后怎么比（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y2「字段清单从哪取」那一行：
/// 每段带归类，判官按表比原字节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldByteRangeClass {
    /// R2 的这几字节与 R1 的逐字节相同。
    MustBeTheSame,
    /// R2 的这几字节全是 0。
    MustBeZero,
    /// 不比：值随这次挂载合法地变，或读槽那一刻已经判过；`judged_instead_by` 写谁替它判。
    NotCompared { judged_instead_by: &'static str },
    /// 挂载前后有一条确定的关系，由 `judged_by` 那一样判（它从同一份原字节里取这几字节）。
    DeterminedRelation { judged_by: MountTransparencyField },
}

/// 字段表里的一段：名字、字节区间、归类，与它引的 D13（验证路线） 已定项 1 挂载透明逐字段表那一行的行名
/// （判决第三节第 3 条：那张表按 Y1、Y2 改行由写回批写；这里引的行名有几行是写回批加的，见 [`D13_ROWS_ADDED_BY_THE_WRITE_BACK`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldByteRange {
    pub name: &'static str,
    pub bytes: std::ops::Range<usize>,
    pub class: FieldByteRangeClass,
    pub row_of_the_d13_field_table: &'static str,
}

/// 系统配置字段表的四档（D22（单元原子性怎么合成） 已定项 9 / 已定项 26：系统不可变配置 389、系统可变配置 4、系统运行配置 36、系统运行量 60）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SystemConfigurationTier {
    ImmutableConfiguration,
    MutableConfiguration,
    RuntimeConfiguration,
    RuntimeQuantities,
}

/// 系统配置字段表里的一段，连同它归哪一档。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationFieldByteRange {
    pub range: FieldByteRange,
    pub tier: SystemConfigurationTier,
}

/// D13（验证路线） 已定项 1 挂载透明逐字段表今天的行名（`.claude/kb/decisions/13-验证路线.md`）。
const D13_IMMUTABLE_CONFIGURATION: &str = "池身份与系统配置的不可变配置（本盘设备号除外）";
const D13_UNMOUNT_MARKER: &str = "卸载记号";
const D13_ROLLBACK_FLOOR: &str = "F 与 F 生效值";
const D13_INSTANCE_GENERATION: &str = "实例代号";
const D13_INSTANCE_TABLE_ROWS: &str = "实例表行";
const D13_CHECKPOINT_TXG: &str = "`checkpoint_txg`";
const D13_TREE_IDENTIFIER_WATERMARK: &str = "树 ID 水位";
/// 那张表的首格还带全角括号注「后三样是树表条目，种类 9、10、11，不是根记录字段」；行名常量写一行装得下的、去掉注的那一截
/// （门禁 doc-registries 的 mount-transparency-fields 格逐行读 `const D13_…: &str = "…";`，比名字时两边都去掉全角括号注）。
const D13_TREE_ROOTS: &str = "树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针";
const D13_ALLOCATION_RECORDS: &str = "分配记录";
/// 判决第二节 Y1、Y2 定下、写回批加进 D13 逐字段表的几行（写回批已写进；行名照今天那张表的首格，全角括号注省掉）。
const D13_STRUCTURAL_FIELDS: &str = "magic、flags 其余位、自证校验和、整槽校验和";
/// 那张表的首格是「根槽偏移 437..466（算法、nonce、MAC）」。
const D13_ROOT_SLOT_RESERVED_BYTES: &str = "根槽偏移 437..466";
/// 根记录的回收门槛 t 与钉槽 p（C773（读坏与全零根槽的回收门槛钉只在内存里），用户 2026-10-20 定）；那张表的首格带注「根槽偏移 428..437」。
const D13_ROOT_RECORD_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT: &str = "回收门槛 t 与钉槽 p";
const D13_SYSTEM_CONFIGURATION_SLOT_GENERATION: &str = "系统配置槽世代号";
const D13_SYSTEM_CONFIGURATION_JOURNAL_TAIL: &str = "系统配置的 journal_tail";
const D13_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE: &str = "系统配置的 journal 实例代号";
const D13_MUTABLE_CONFIGURATION: &str = "系统配置的节点大小";
/// 那张表的首格带注「T_time、T_dirty、三水位」。
const D13_RUNTIME_CONFIGURATION: &str = "系统配置的运行配置 36 字节";
const D13_TREE_ENTRY_IDENTITY: &str = "树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类";
/// 根记录里实例表、树表两个单元指针（不比字节，表里的行与条目另由「实例表行」与树表条目那张表比）。
const D13_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS: &str = "实例表与树表的单元指针";

/// 这几张表引的 D13 行名里，写回批之前逐字段表就有的那几行（[`D13_ROWS_ADDED_BY_THE_WRITE_BACK`] 之外的）。
/// 门禁 doc-registries 的 mount-transparency-fields 格把这两组与三张字段表引的行名一起按今天的 D13 表比。
pub const D13_ROWS_OF_TODAY: [&str; 9] = [
    D13_IMMUTABLE_CONFIGURATION,
    D13_UNMOUNT_MARKER,
    D13_ROLLBACK_FLOOR,
    D13_INSTANCE_GENERATION,
    D13_INSTANCE_TABLE_ROWS,
    D13_CHECKPOINT_TXG,
    D13_TREE_IDENTIFIER_WATERMARK,
    D13_TREE_ROOTS,
    D13_ALLOCATION_RECORDS,
];

/// 这几张表引的 D13 行名里，写回批加进逐字段表的那几行（判决第三节第 3 条）。
pub const D13_ROWS_ADDED_BY_THE_WRITE_BACK: [&str; 10] = [
    D13_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS,
    D13_STRUCTURAL_FIELDS,
    D13_ROOT_SLOT_RESERVED_BYTES,
    D13_ROOT_RECORD_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT,
    D13_SYSTEM_CONFIGURATION_SLOT_GENERATION,
    D13_SYSTEM_CONFIGURATION_JOURNAL_TAIL,
    D13_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
    D13_MUTABLE_CONFIGURATION,
    D13_RUNTIME_CONFIGURATION,
    D13_TREE_ENTRY_IDENTITY,
];

/// 根记录里几样字段的字节区间（D22（单元原子性怎么合成） 已定项 7 的字段表：行序即盘上顺序，偏移按行累加）。
const ROOT_RECORD_INSTANCE: std::ops::Range<usize> = 24..28;
const ROOT_RECORD_MAPPING_ROOT: std::ops::Range<usize> = 256..342;
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;

/// 系统配置里几样字段的字节区间（`.claude/kb/layout/01-first-txn.md`「系统配置字段表的偏移」）。
const SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION: std::ops::Range<usize> = 433..469;
const SYSTEM_CONFIGURATION_JOURNAL_TAIL: std::ops::Range<usize> = 469..477;
const SYSTEM_CONFIGURATION_JOURNAL_INSTANCE: std::ops::Range<usize> = 477..481;

/// 树表条目里种类与根指针的字节区间（D8（核心索引结构） 已定项 8：树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86）。
const TREE_TABLE_ENTRY_KIND: std::ops::Range<usize> = 10..12;
const TREE_TABLE_ENTRY_ROOT_POINTER: std::ops::Range<usize> = 14..100;

const fn must_be_the_same(
    name: &'static str,
    bytes: std::ops::Range<usize>,
    row_of_the_d13_field_table: &'static str,
) -> FieldByteRange {
    FieldByteRange {
        name,
        bytes,
        class: FieldByteRangeClass::MustBeTheSame,
        row_of_the_d13_field_table,
    }
}

const fn not_compared(
    name: &'static str,
    bytes: std::ops::Range<usize>,
    judged_instead_by: &'static str,
    row_of_the_d13_field_table: &'static str,
) -> FieldByteRange {
    FieldByteRange {
        name,
        bytes,
        class: FieldByteRangeClass::NotCompared { judged_instead_by },
        row_of_the_d13_field_table,
    }
}

const fn determined_relation(
    name: &'static str,
    bytes: std::ops::Range<usize>,
    judged_by: MountTransparencyField,
    row_of_the_d13_field_table: &'static str,
) -> FieldByteRange {
    FieldByteRange {
        name,
        bytes,
        class: FieldByteRangeClass::DeterminedRelation { judged_by },
        row_of_the_d13_field_table,
    }
}

const READ_WHEN_THE_SLOT_IS_READ: &str = "读槽时判（不对这一槽不可择）";

/// 根记录 [0, `ROOT_RECORD_BYTES`) 按字段逐段（D22（单元原子性怎么合成） 已定项 7 的字段表 17 行；回收门槛 t 与钉槽 p 合一段，算法、nonce、MAC 合一段）。
pub const ROOT_RECORD_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
    not_compared(
        "magic",
        0..4,
        READ_WHEN_THE_SLOT_IS_READ,
        D13_STRUCTURAL_FIELDS,
    ),
    must_be_the_same("fsid", 4..20, D13_IMMUTABLE_CONFIGURATION),
    determined_relation(
        "flags（位 0 是卸载记号；其余位非 0 读槽时拒收）",
        20..24,
        MountTransparencyField::UnmountMarker,
        D13_UNMOUNT_MARKER,
    ),
    determined_relation(
        "实例代号",
        ROOT_RECORD_INSTANCE,
        MountTransparencyField::InstanceGeneration,
        D13_INSTANCE_GENERATION,
    ),
    determined_relation(
        "checkpoint_txg",
        28..36,
        MountTransparencyField::CheckpointTxg,
        D13_CHECKPOINT_TXG,
    ),
    not_compared(
        "树表单元指针",
        36..122,
        "树表单元自己的落点每次发布都变、不比；它的条目按树表条目那张表逐段比",
        D13_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS,
    ),
    determined_relation(
        "树 ID 水位",
        122..130,
        MountTransparencyField::TreeIdentifierWatermark,
        D13_TREE_IDENTIFIER_WATERMARK,
    ),
    determined_relation(
        "回退下界 F",
        130..138,
        MountTransparencyField::RollbackFloor,
        D13_ROLLBACK_FLOOR,
    ),
    not_compared(
        "自证校验和",
        138..170,
        READ_WHEN_THE_SLOT_IS_READ,
        D13_STRUCTURAL_FIELDS,
    ),
    not_compared(
        "实例表单元指针",
        170..256,
        "实例表单元的落点不比；表里的行由「实例表行」那一样比",
        D13_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS,
    ),
    determined_relation(
        "中央映射树根指针",
        ROOT_RECORD_MAPPING_ROOT,
        MountTransparencyField::MappingRootEmptiness,
        D13_ALLOCATION_RECORDS,
    ),
    determined_relation(
        "分配记录树根指针",
        ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT,
        MountTransparencyField::AllocationRecordTreeRoot,
        D13_ALLOCATION_RECORDS,
    ),
    not_compared(
        "回收门槛 t、钉槽 p",
        428..437,
        "每次带单元的发布按这个进程的根环与钉现算、零单元发布照抄上一版，挂载前后不比；池级 checker 的 I-7.18、I-7.19 判（单镜像性质）",
        D13_ROOT_RECORD_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT,
    ),
    not_compared(
        "算法类型、nonce、MAC",
        437..466,
        "池级 checker 的 I-2.4 判全 0（单镜像性质）",
        D13_ROOT_SLOT_RESERVED_BYTES,
    ),
];

const fn system_configuration_field(
    range: FieldByteRange,
    tier: SystemConfigurationTier,
) -> SystemConfigurationFieldByteRange {
    SystemConfigurationFieldByteRange { range, tier }
}

const fn immutable_and_the_same(
    name: &'static str,
    bytes: std::ops::Range<usize>,
) -> SystemConfigurationFieldByteRange {
    system_configuration_field(
        must_be_the_same(name, bytes, D13_IMMUTABLE_CONFIGURATION),
        SystemConfigurationTier::ImmutableConfiguration,
    )
}

/// 系统配置 [0, `SYSTEM_CONFIGURATION_BYTES`) 按字段逐段（`.claude/kb/layout/01-first-txn.md`「系统配置字段表」46 行，一行一段）。
pub const SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES: &[SystemConfigurationFieldByteRange] = &[
    system_configuration_field(
        not_compared(
            "magic",
            0..4,
            READ_WHEN_THE_SLOT_IS_READ,
            D13_STRUCTURAL_FIELDS,
        ),
        SystemConfigurationTier::ImmutableConfiguration,
    ),
    immutable_and_the_same("格式版本", 4..6),
    immutable_and_the_same("feature bits", 6..102),
    immutable_and_the_same("fsid", 102..118),
    immutable_and_the_same("写入者身份", 118..138),
    immutable_and_the_same("校验和算法标识", 138..139),
    system_configuration_field(
        not_compared(
            "本盘设备号",
            139..143,
            "择到哪块盘的那一槽就是哪块盘的号；池级 checker 的 I-7.14 判它等于盘在池里的身份",
            D13_IMMUTABLE_CONFIGURATION,
        ),
        SystemConfigurationTier::ImmutableConfiguration,
    ),
    immutable_and_the_same("设备数 devs", 143..147),
    system_configuration_field(
        not_compared(
            "槽世代号",
            147..155,
            "池级 checker 的 I-7.16 在 R2 的镜像上判",
            D13_SYSTEM_CONFIGURATION_SLOT_GENERATION,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    system_configuration_field(
        not_compared(
            "整槽校验和",
            155..187,
            READ_WHEN_THE_SLOT_IS_READ,
            D13_STRUCTURAL_FIELDS,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    immutable_and_the_same("系统配置自身 MAC", 187..203),
    immutable_and_the_same("nonce 水位", 203..215),
    immutable_and_the_same("KDF 标识", 215..219),
    immutable_and_the_same("加密类型", 219..220),
    immutable_and_the_same("MAC 长度声明", 220..221),
    immutable_and_the_same("主密钥槽", 221..301),
    system_configuration_field(
        must_be_the_same("节点大小", 301..305, D13_MUTABLE_CONFIGURATION),
        SystemConfigurationTier::MutableConfiguration,
    ),
    immutable_and_the_same("单元大小", 305..309),
    immutable_and_the_same("落点粒度", 309..313),
    immutable_and_the_same("位置条目宽度", 313..317),
    immutable_and_the_same("mkfs 时的 physical_block_size", 317..321),
    immutable_and_the_same("扩展点声明值 N", 321..325),
    immutable_and_the_same("journal 环起点", 325..333),
    immutable_and_the_same("journal 环长度", 333..341),
    immutable_and_the_same("journal 最大记录字节", 341..345),
    immutable_and_the_same("journal 在飞记录数上限", 345..349),
    immutable_and_the_same("journal 最坏占用", 349..357),
    immutable_and_the_same("journal 安全系数 F", 357..361),
    immutable_and_the_same("根环区域数 R", 361..362),
    immutable_and_the_same("每区槽数 S", 362..363),
    immutable_and_the_same("素数步长 P", 363..367),
    immutable_and_the_same("根环 chunk", 367..371),
    immutable_and_the_same("根环起点", 371..379),
    immutable_and_the_same("根环逐区域设备身份", 379..391),
    immutable_and_the_same("条带宽度上界 w_max", 391..392),
    immutable_and_the_same("组大小 g", 392..393),
    immutable_and_the_same("映射来源", 393..417),
    immutable_and_the_same("单元区起始槽号", 417..425),
    immutable_and_the_same("mkfs 时的 io_min", 425..429),
    immutable_and_the_same("固定结构槽距", 429..433),
    system_configuration_field(
        determined_relation(
            "T_time",
            433..437,
            MountTransparencyField::SystemRuntimeConfiguration,
            D13_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
    system_configuration_field(
        determined_relation(
            "T_dirty",
            437..445,
            MountTransparencyField::SystemRuntimeConfiguration,
            D13_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
    system_configuration_field(
        determined_relation(
            "整理低水位 / 高水位 / 停止线",
            445..469,
            MountTransparencyField::SystemRuntimeConfiguration,
            D13_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
    system_configuration_field(
        determined_relation(
            "journal tail",
            SYSTEM_CONFIGURATION_JOURNAL_TAIL,
            MountTransparencyField::SystemConfigurationJournalTail,
            D13_SYSTEM_CONFIGURATION_JOURNAL_TAIL,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    system_configuration_field(
        determined_relation(
            "journal 实例代号",
            SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
            MountTransparencyField::SystemConfigurationJournalInstance,
            D13_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    system_configuration_field(
        not_compared(
            "回退下界 F",
            481..489,
            "写低由池级 checker 的 I-7.12 判；F 生效值由「F 与 F 生效值」那一样比",
            D13_ROLLBACK_FLOOR,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
];

/// 树表条目 [0, 200) 按字段逐段（D8（核心索引结构） 已定项 8 的字段表：树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 +
/// `previous_snapshot_txg` 8 + 诞生 txg 8 + 头 ID 8 + 预留 76）。条目按盘上下标一一对上比（两版条数不同判红）。
pub const TREE_TABLE_ENTRY_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
    must_be_the_same("树 ID", 0..8, D13_TREE_ENTRY_IDENTITY),
    must_be_the_same("条目长度", 8..10, D13_TREE_ENTRY_IDENTITY),
    must_be_the_same("种类", TREE_TABLE_ENTRY_KIND, D13_TREE_ENTRY_IDENTITY),
    must_be_the_same("flags", 12..14, D13_TREE_ENTRY_IDENTITY),
    determined_relation(
        "根指针",
        TREE_TABLE_ENTRY_ROOT_POINTER,
        MountTransparencyField::TreeTableEntryRootPointers,
        D13_TREE_ROOTS,
    ),
    must_be_the_same("previous_snapshot_txg", 100..108, D13_TREE_ENTRY_IDENTITY),
    must_be_the_same("诞生 txg", 108..116, D13_TREE_ENTRY_IDENTITY),
    must_be_the_same("头 ID", 116..124, D13_TREE_ENTRY_IDENTITY),
    FieldByteRange {
        name: "预留",
        bytes: 124..200,
        class: FieldByteRangeClass::MustBeZero,
        row_of_the_d13_field_table: D13_TREE_ENTRY_IDENTITY,
    },
];

/// 三张字节表按字节比的那几段（「必须相同」「必须为 0」）里，`after` 对不上的段名。`before` / `after` 短于那一段的末尾也算对不上。
fn byte_ranges_that_do_not_hold<'table>(
    table: impl IntoIterator<Item = &'table FieldByteRange>,
    before: &[u8],
    after: &[u8],
) -> Vec<&'static str> {
    table
        .into_iter()
        .filter(|range| match range.class {
            FieldByteRangeClass::MustBeTheSame => {
                before.get(range.bytes.clone()) != after.get(range.bytes.clone())
                    || after.get(range.bytes.clone()).is_none()
            }
            FieldByteRangeClass::MustBeZero => !after
                .get(range.bytes.clone())
                .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0)),
            FieldByteRangeClass::NotCompared { .. }
            | FieldByteRangeClass::DeterminedRelation { .. } => false,
        })
        .map(|range| range.name)
        .collect()
}

/// T_time 的内置默认值，单位毫秒：5 s 写 5000（D16（发布语义） 已定项 5「`T_time` 默认值 5 s」、可设区间 [200 ms, 5 s]；
/// 盘上编码单位毫秒是用户 2026-10-05 定的，判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2，
/// `.claude/kb/layout/01-first-txn.md` 系统配置字段表「可调值 | T_time | 4」那一行由写回批写明单位毫秒、默认值 5000）。照条款写死，不取被测 core 的常量：
/// 期望取自被测者时，core 把默认值或单位改错，挂载写槽写的与期望一起错，这一行判不出
/// （判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」那一行，改法 G1）。
const BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES: u32 = 5000;
/// T_dirty 的内置默认值，单位字节：2 GiB（D16（发布语义） 已定项 5；字段表「可调值 | T_dirty | 8 | 2 GiB」那一行。挂载时的有效值另按环长夹取，
/// 不改盘上这 8 字节）。
const BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES: u64 = 2 * 1024 * 1024 * 1024;
/// 整理低水位 / 高水位 / 停止线的内置默认值：0 = 内置默认（字段表「可调值 | 整理低水位 / 高水位 / 停止线 | 8 × 3 | 0 = 内置默认」那一行，
/// D26（后台整理与放置回收） 已定项 1）。
const BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES: [u64; 3] = [0, 0, 0];

/// 系统运行配置那 36 字节的内置默认值：照字段表的次序（T_time 4、T_dirty 8、整理三水位 8 × 3，小端），值照条款写死
/// （[`BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES`] 那三个常量；今天没有挂载选项通道，写槽总写内置默认）。
#[must_use]
pub fn built_in_runtime_configuration_bytes() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len());
    bytes.extend_from_slice(&BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES.to_le_bytes());
    bytes.extend_from_slice(&BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES.to_le_bytes());
    for watermark in BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES {
        bytes.extend_from_slice(&watermark.to_le_bytes());
    }
    assert_eq!(
        bytes.len(),
        SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len(),
        "系统运行配置一档 36 字节：T_time 4 + T_dirty 8 + 整理三水位 8 × 3"
    );
    bytes
}

fn read_u32_at(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u32> {
    bytes
        .get(range)
        .map(|field| u32::from_le_bytes(field.try_into().expect("切的是 4 字节")))
}

fn read_u64_at(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u64> {
    bytes
        .get(range)
        .map(|field| u64::from_le_bytes(field.try_into().expect("切的是 8 字节")))
}

fn is_all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
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
        inode_record_truncation_fields: left_inode_record_truncation_fields,
    } = left;
    let SystemImmutableConfiguration {
        filesystem_identifier: right_filesystem_identifier,
        this_device: _,
        device_count: right_device_count,
        region_devices: right_region_devices,
        sizes: right_sizes,
        journal_ring_start_slot: right_journal_ring_start_slot,
        root_ring_base_slot: right_root_ring_base_slot,
        inode_record_truncation_fields: right_inode_record_truncation_fields,
    } = right;
    (
        left_filesystem_identifier,
        left_device_count,
        left_region_devices,
        left_sizes,
        left_journal_ring_start_slot,
        left_root_ring_base_slot,
        left_inode_record_truncation_fields,
    ) == (
        right_filesystem_identifier,
        right_device_count,
        right_region_devices,
        right_sizes,
        right_journal_ring_start_slot,
        right_root_ring_base_slot,
        right_inode_record_truncation_fields,
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
/// 树 ID 水位相同（挂载不发第一个文件）；树表里 extent、inode、deleted_inodes、墓碑、谱系五种树的根指针相同——挂载续做过时
/// （用户 2026-10-03 定「续做算挂载的一部分」）比挂载交回的续做那几步里 txg 不大于 R2 那一版的最近一步的
/// （`continuation`，[`UserVisibleTreeRootPointersOfTheContinuation`]），livelist、稀疏旁表、deadlist、logged_ops 四种的根相同；
/// txg ≥ max(根环全部根的 txg, 环里全部自证记录的 checkpoint_txg) + 1（只判下界：kb 允许跳号）；扫描报告施加 0 条、水位之上 0 条。
/// 二次崩溃之后落回 v1 那一版自己（实例与 txg 都是 R1 那一版的）：各样与 R1 那一版相同（推的）；落到这次挂载写的某一版：照上一段，
/// 扫描报告不判，崩在抬 F 开始之后的 F 不判。
/// 判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1、Y2 补的几样（[`MountTransparencyField`] 的后八个成员）：
/// 中央映射树根的空性、根记录里的分配记录树根、系统配置 journal_tail 与 journal 实例代号、系统运行配置 = 内置默认，
/// 再加根记录、系统配置、树表条目三张按字节区间的字段表里「必须相同」「必须为 0」的那几段；这几样都从 [`read_recovery_fields`]
/// 读进来的原字节里取（[`VersionFieldsRead::root_record_bytes`]、[`VersionFieldsRead::tree_table_entries`]、
/// [`RecoveryFieldsRead::chosen_system_configuration_bytes`]），签名不变。
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
        MountTransparencyField::TreeTableEntryRootPointers => {
            tree_table_entry_root_pointers_outcome(first_recovery, compared, relation, continuation)
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
        MountTransparencyField::MappingRootEmptiness => {
            let before = &first_recovery.root_record_bytes[ROOT_RECORD_MAPPING_ROOT];
            let after = &compared.root_record_bytes[ROOT_RECORD_MAPPING_ROOT];
            match relation {
                Relation::WrittenByTheMount(_) => held_or_violated(
                    is_all_zero(after) == is_all_zero(before),
                    format!("R2 全零 ⇔ R1 全零（R1 全零：{}）", is_all_zero(before)),
                    format!("R2 全零：{}", is_all_zero(after)),
                ),
                Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
                    after == before,
                    format!("与 R1 逐字节相同：{before:?}"),
                    format!("{after:?}"),
                ),
            }
        }
        MountTransparencyField::AllocationRecordTreeRoot => {
            allocation_record_tree_root_outcome(first_recovery, compared, relation, new_instance)
        }
        MountTransparencyField::SystemConfigurationJournalTail => {
            system_configuration_journal_tail_outcome(before_the_mount, after_the_action, action)
        }
        MountTransparencyField::SystemConfigurationJournalInstance => {
            system_configuration_journal_instance_outcome(
                before_the_mount,
                after_the_action,
                compared,
                new_instance,
                action,
            )
        }
        MountTransparencyField::SystemRuntimeConfiguration => {
            let built_in = built_in_runtime_configuration_bytes();
            let observed = after_the_action
                .chosen_system_configuration_bytes
                .as_deref()
                .and_then(|bytes| bytes.get(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION));
            held_or_violated(
                observed == Some(built_in.as_slice()),
                format!("内置默认 {built_in:?}"),
                format!("{observed:?}"),
            )
        }
        MountTransparencyField::RootRecordBytes => {
            let ranges_that_do_not_hold = byte_ranges_that_do_not_hold(
                ROOT_RECORD_FIELD_BYTE_RANGES,
                &first_recovery.root_record_bytes,
                &compared.root_record_bytes,
            );
            held_or_violated(
                ranges_that_do_not_hold.is_empty(),
                "根记录里「必须相同」「必须为 0」的那几段都对得上".to_string(),
                format!("这几段不符：{ranges_that_do_not_hold:?}"),
            )
        }
        MountTransparencyField::SystemConfigurationBytes => match (
            &before_the_mount.chosen_system_configuration_bytes,
            &after_the_action.chosen_system_configuration_bytes,
        ) {
            (Some(before), Some(after)) => {
                let ranges_that_do_not_hold = byte_ranges_that_do_not_hold(
                    SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES
                        .iter()
                        .map(|system_configuration_field| &system_configuration_field.range),
                    before,
                    after,
                );
                held_or_violated(
                    ranges_that_do_not_hold.is_empty(),
                    "系统配置里「必须相同」「必须为 0」的那几段都对得上".to_string(),
                    format!("这几段不符：{ranges_that_do_not_hold:?}"),
                )
            }
            (before, after) => FieldOutcome::Violated {
                expected: "R1、R2 都择得到系统配置".to_string(),
                observed: format!(
                    "R1 择到：{}；R2 择到：{}",
                    before.is_some(),
                    after.is_some()
                ),
            },
        },
        MountTransparencyField::TreeTableEntryBytes => {
            tree_table_entry_bytes_outcome(first_recovery, compared)
        }
    }
}

/// 树表条目的根指针那一样（[`MountTransparencyField::TreeTableEntryRootPointers`]）：只由续做改根的五种照续做规则
/// （挂载写的那一版比续做里 txg 不大于它的最近一步，没有那一步比 R1），根恒全零的四种与 R1 相同，种类 3、4 不比。
/// 两版任一版树表读不出判红。
fn tree_table_entry_root_pointers_outcome(
    first_recovery: &VersionFieldsRead,
    compared: &VersionFieldsRead,
    relation: Relation,
    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
) -> FieldOutcome {
    let (Some(entries_before), Some(entries_after)) = (
        &first_recovery.tree_table_entries,
        &compared.tree_table_entries,
    ) else {
        return FieldOutcome::Violated {
            expected: "R1、R2 两版的树表都读得出".to_string(),
            observed: format!(
                "R1 读得出：{}；R2 读得出：{}",
                first_recovery.tree_table_entries.is_some(),
                compared.tree_table_entries.is_some()
            ),
        };
    };
    let rewritten_only_by_the_continuation_before =
        TreeTableEntryRootPointersByKind::of_the_entries(
            entries_before,
            &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
        );
    let expected_rewritten_only_by_the_continuation = match relation {
        Relation::WrittenByTheMount(_) => continuation
            .of_the_last_step_at_or_before((compared.root.instance, compared.root.checkpoint_txg))
            .unwrap_or(&rewritten_only_by_the_continuation_before),
        Relation::TheVersionOfTheFirstRecovery(_) => &rewritten_only_by_the_continuation_before,
    };
    let observed_rewritten_only_by_the_continuation =
        TreeTableEntryRootPointersByKind::of_the_entries(
            entries_after,
            &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
        );
    let kept_before = TreeTableEntryRootPointersByKind::of_the_entries(
        entries_before,
        &TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS,
    );
    let kept_after = TreeTableEntryRootPointersByKind::of_the_entries(
        entries_after,
        &TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS,
    );
    held_or_violated(
        observed_rewritten_only_by_the_continuation == *expected_rewritten_only_by_the_continuation
            && kept_after == kept_before,
        format!(
            "续做改根的五种 {expected_rewritten_only_by_the_continuation:?}；根恒全零的四种 {kept_before:?}"
        ),
        format!(
            "续做改根的五种 {observed_rewritten_only_by_the_continuation:?}；根恒全零的四种 {kept_after:?}"
        ),
    )
}

/// 根记录里的分配记录树根那一样（[`MountTransparencyField::AllocationRecordTreeRoot`]）。R1 树表读不出（认不出带不带树）、
/// 或 R1 树表 0 条而新实例号算不出时判红。
fn allocation_record_tree_root_outcome(
    first_recovery: &VersionFieldsRead,
    compared: &VersionFieldsRead,
    relation: Relation,
    new_instance: Option<InstanceGeneration>,
) -> FieldOutcome {
    let before = &first_recovery.root_record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT];
    let after = &compared.root_record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT];
    match relation {
        Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
            after == before,
            format!("与 R1 逐字节相同：{before:?}"),
            format!("{after:?}"),
        ),
        Relation::WrittenByTheMount(_) => {
            match (first_recovery.tree_table_entries.as_deref(), new_instance) {
                (Some([_, ..]), _) => held_or_violated(
                    is_all_zero(after),
                    "R1 带树 ⇒ R2 全零".to_string(),
                    format!("R2 全零：{}", is_all_zero(after)),
                ),
                (Some([]), Some(new_instance)) => {
                    let expected_non_zero = new_instance.0 >= 2;
                    held_or_violated(
                        !is_all_zero(after) == expected_non_zero,
                        format!(
                            "R1 树表 0 条 ⇒ R2 非零 ⇔ 新实例号 {} ≥ 2（该非零：{expected_non_zero}）",
                            new_instance.0
                        ),
                        format!("R2 非零：{}", !is_all_zero(after)),
                    )
                }
                (Some([]), None) => FieldOutcome::Violated {
                    expected: "R1 树表 0 条：要新实例号定 R2 该不该非零".to_string(),
                    observed: "新实例号算不出".to_string(),
                },
                (None, _) => FieldOutcome::Violated {
                    expected: "R1 那一版的树表读得出（认得出带不带树）".to_string(),
                    observed: "R1 那一版的树表读不出".to_string(),
                },
            }
        }
    }
}

/// 系统配置 journal_tail 那一样（[`MountTransparencyField::SystemConfigurationJournalTail`]，判决第二节 Y1 journal_tail 那一行取攻方的式子）：
/// R1 的 ≤ R2 的 ≤ R2 环里自证记录的最大计数器（环里一条都没有按 0），挂载做完（只挂载、挂载加一次发布）取等。
/// 二次崩溃落回 v1 那一版时照这一行判，不按与 R1 相同判（判决表下那句：系统配置的运行量不是那一版自己的）。
/// 两次恢复任一次择不到系统配置判红。
fn system_configuration_journal_tail_outcome(
    before_the_mount: &RecoveryFieldsRead,
    after_the_action: &RecoveryFieldsRead,
    action: MountTransparencyAction,
) -> FieldOutcome {
    let tail_of = |fields: &RecoveryFieldsRead| {
        fields
            .chosen_system_configuration_bytes
            .as_deref()
            .and_then(|bytes| read_u64_at(bytes, SYSTEM_CONFIGURATION_JOURNAL_TAIL))
    };
    let (Some(tail_before), Some(tail_after)) =
        (tail_of(before_the_mount), tail_of(after_the_action))
    else {
        return FieldOutcome::Violated {
            expected: "R1、R2 都择得到系统配置".to_string(),
            observed: format!(
                "R1 的 tail {:?}；R2 的 tail {:?}",
                tail_of(before_the_mount),
                tail_of(after_the_action)
            ),
        };
    };
    let highest_record_counter = after_the_action
        .highest_record_counter_in_the_ring
        .unwrap_or(0);
    let mount_completed = match action {
        MountTransparencyAction::MountAlone(_) | MountTransparencyAction::MountAndOnePublish(_) => {
            true
        }
        MountTransparencyAction::SecondCrashInsideTheMount(_) => false,
    };
    held_or_violated(
        tail_before <= tail_after
            && tail_after <= highest_record_counter
            && (!mount_completed || tail_after == highest_record_counter),
        format!(
            "R1 的 tail {tail_before} ≤ R2 的 tail ≤ R2 环里自证记录的最大计数器 {highest_record_counter}{}",
            if mount_completed { "，挂载做完取等" } else { "" }
        ),
        format!("R2 的 tail {tail_after}"),
    )
}

/// 系统配置的 journal 实例代号那一样（[`MountTransparencyField::SystemConfigurationJournalInstance`]，判决第二节 Y1 那一行取攻方的式子）：
/// 挂载做完 = R2 那一版根的实例代号（从根记录原字节取）；二次崩溃 ∈ {R1 的, 新实例}。择不到系统配置判红。
fn system_configuration_journal_instance_outcome(
    before_the_mount: &RecoveryFieldsRead,
    after_the_action: &RecoveryFieldsRead,
    compared: &VersionFieldsRead,
    new_instance: Option<InstanceGeneration>,
    action: MountTransparencyAction,
) -> FieldOutcome {
    let instance_of = |fields: &RecoveryFieldsRead| {
        fields
            .chosen_system_configuration_bytes
            .as_deref()
            .and_then(|bytes| read_u32_at(bytes, SYSTEM_CONFIGURATION_JOURNAL_INSTANCE))
    };
    let observed = instance_of(after_the_action);
    match action {
        MountTransparencyAction::MountAlone(_) | MountTransparencyAction::MountAndOnePublish(_) => {
            let root_instance = read_u32_at(&compared.root_record_bytes, ROOT_RECORD_INSTANCE);
            held_or_violated(
                observed.is_some() && observed == root_instance,
                format!("R2 那一版根的实例代号 {root_instance:?}"),
                format!("{observed:?}"),
            )
        }
        MountTransparencyAction::SecondCrashInsideTheMount(_) => {
            let before = instance_of(before_the_mount);
            let new_instance = new_instance.map(|instance| instance.0);
            held_or_violated(
                observed.is_some() && (observed == before || observed == new_instance),
                format!("R1 的 {before:?} 或新实例 {new_instance:?}"),
                format!("{observed:?}"),
            )
        }
    }
}

/// 树表条目按字节区间那一样（[`MountTransparencyField::TreeTableEntryBytes`]）：两版条目按盘上下标一一对上，逐条按
/// [`TREE_TABLE_ENTRY_FIELD_BYTE_RANGES`] 比「必须相同」「必须为 0」那几段。条数不同、任一版树表读不出判红。
fn tree_table_entry_bytes_outcome(
    first_recovery: &VersionFieldsRead,
    compared: &VersionFieldsRead,
) -> FieldOutcome {
    let (Some(entries_before), Some(entries_after)) = (
        &first_recovery.tree_table_entries,
        &compared.tree_table_entries,
    ) else {
        return FieldOutcome::Violated {
            expected: "R1、R2 两版的树表都读得出".to_string(),
            observed: format!(
                "R1 读得出：{}；R2 读得出：{}",
                first_recovery.tree_table_entries.is_some(),
                compared.tree_table_entries.is_some()
            ),
        };
    };
    if entries_before.len() != entries_after.len() {
        return FieldOutcome::Violated {
            expected: format!("树表 {} 条", entries_before.len()),
            observed: format!("树表 {} 条", entries_after.len()),
        };
    }
    let entries_that_do_not_hold: Vec<String> = entries_before
        .iter()
        .zip(entries_after)
        .enumerate()
        .filter_map(|(index, (before, after))| {
            let ranges =
                byte_ranges_that_do_not_hold(TREE_TABLE_ENTRY_FIELD_BYTE_RANGES, before, after);
            (!ranges.is_empty()).then(|| format!("第 {index} 条：{ranges:?}"))
        })
        .collect();
    held_or_violated(
        entries_that_do_not_hold.is_empty(),
        "树表每一条里「必须相同」「必须为 0」的那几段都对得上".to_string(),
        format!("这几条不符：{}", entries_that_do_not_hold.join("；")),
    )
}

/// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 判不出改取 R2，用户 2026-10-03 定）上的拒可写理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalReasonOnTheD23List {
    /// 读阶段判据 N-配置 为真（系统配置见证判不出按真：R2，有槽设备报读错而没有一块盘两槽都自证过），重读一次仍为真。
    NewerPublishWitnessedBySystemConfiguration,
    /// 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次仍读不出。
    InstanceTableOfTheNewestRootUnreadable,
    /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置判不出（R2）或某块盘一份自证过的都没有
    /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`；C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里报）。
    SystemConfigurationUnreadableAfterTheReadStage,
    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有
    /// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`；用户 2026-10-05 定 Q3「另起拒因、单子加一行」，
    /// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md`；之前报 `Recovery(NoValidRoot)`、与读阶段择不到根同名，不在单子上；
    /// 两遍按槽并照用户 2026-10-05 定 R1，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节：第一遍读出过一条有效根的不拒）。
    NoValidRootInTheRootRingOfTheRebuiltAllocator,
    /// 「记录有缺口时拒可写」（F1 乙，用户 2026-10-07 定；判据照判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行，`F1_ANY`）：
    /// 水位之上一次没施加的发布，读得出的那几条记录的本次发布内序号不是 1..N 连着或缺末条，而它们点名的单元里有一个两份副本都验不过，
    /// 重读一次仍这样（`StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`）。
    PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
    /// 可写挂载的 C 读一次没施加的发布（水位之上那几条记录拼的根，或比所选那一版新、按实例表有效、读得出的根）的账读不出、那次发布读得出的记录
    /// 不齐、从中央映射也现算不成，重读一次仍这样（`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
    /// 第二节 K2 W1 / W2 H1、K2 W2 H3 两行；`StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`，
    /// 用户 2026-10-08 定「乙：core 另起错误成员」、列进单子）。`unapplied_publish` 是那一拒点名的 (实例代号, checkpoint_txg)，复算按它找那次发布。
    AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
        unapplied_publish: (InstanceGeneration, CheckpointTxg),
    },
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
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                    Some(RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)
                }
                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                    unapplied_publish,
                } => Some(
                    RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                        unapplied_publish: (
                            unapplied_publish.instance,
                            unapplied_publish.checkpoint_txg,
                        ),
                    },
                ),
                // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
            }
        }
        MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
            Some(RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage)
        }
        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
            Some(RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator)
        }
        // 恢复报的错（读阶段择不到根那一拒在内，只读挂载同样挂不上）：不在单子上。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
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

/// 一块盘两个系统配置槽在崩溃镜像上各直接读一次：自证过、fsid 与本池相同的那几份，与设备报读错（`PoolReader::read` 交空）的槽数。
/// 读得出而自证不过的槽（撕裂的轮换、没写过的槽）两边都不算。
struct SystemConfigurationSlotsReadOnTheCrashImage {
    self_verified: Vec<SystemConfiguration>,
    slots_reporting_a_read_error: u64,
}

fn system_configuration_slots_read_on_the_crash_image(
    crash_image: &dyn PoolReader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> SystemConfigurationSlotsReadOnTheCrashImage {
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节装得进 usize");
    let mut slots = SystemConfigurationSlotsReadOnTheCrashImage {
        self_verified: Vec::new(),
        slots_reporting_a_read_error: 0,
    };
    // 迭代上界是一块盘的两个系统配置槽；跨轮携带的只有两样计数；没有提前出口。
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes);
        match crash_image.read(device, offset, slot_bytes) {
            None => slots.slots_reporting_a_read_error += 1,
            Some(bytes) => {
                if let Ok(system_configuration) =
                    crate::verification_read::parse_system_configuration(&bytes)
                {
                    if system_configuration.immutable.filesystem_identifier
                        == *filesystem_identifier
                    {
                        slots.self_verified.push(system_configuration);
                    }
                }
            }
        }
    }
    slots
}

/// 系统配置见证判不出（R2，D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」）在崩溃镜像上独立写的一份，
/// 不调实现的 `recovery::system_configuration_witness_is_undecidable`：判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。
/// 撕裂的轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。
fn system_configuration_witness_undecidable_on_the_crash_image(
    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
) -> bool {
    let some_slot_reported_a_read_error = slots_of_every_device
        .iter()
        .any(|slots| slots.slots_reporting_a_read_error > 0);
    let some_device_self_verified_both_slots = slots_of_every_device.iter().any(|slots| {
        u64::try_from(slots.self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
    });
    some_slot_reported_a_read_error && !some_device_self_verified_both_slots
}

/// 某块盘两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1：这块盘不「可见」，D18（块里携带什么信息） 已定项 11）。
/// R2 之下这一形不并进判不出（一块盘两槽都撕裂、另一块盘两槽都自证过时 R2 判得出）；实现在读阶段判完之后的逐盘核与取号里拒它
/// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，与判据 N-配置 那一拒分开报，C664（挂载拒绝合法复算把两种拒因报成一个））。
fn some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
) -> bool {
    slots_of_every_device
        .iter()
        .any(|slots| slots.self_verified.is_empty())
}

/// 「读阶段判完之后系统配置又读不出」（D23（journal 的角色与格式） 已定项 14：取号那一刻按同一个判不出（R2）判，某块盘一份自证过的系统配置
/// 都没有时也拒）在崩溃镜像上复算：每块盘两槽各直接读一次，判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]）或某块盘
/// 一份本池自证过的都没有（[`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`]）⇒ 真；择不到系统配置也按真。
/// 两遍读之间的瞬时读错在崩溃镜像上复算不出（C664（挂载拒绝合法复算把两种拒因报成一个）），那一形怎么判见
/// [`judge_writable_mount_refusal_of_a_crash_state`]。
fn system_configuration_unreadable_after_the_read_stage_on_the_crash_image(
    crash_image: &dyn PoolReader,
) -> bool {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return true;
    };
    let immutable = system_configuration.immutable;
    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_image
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_read_on_the_crash_image(
                crash_image,
                device,
                slot_spacing,
                &immutable.filesystem_identifier,
            )
        })
        .collect();
    system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)
        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
            &slots_of_every_device,
        )
}

/// 池级 checker 读崩溃镜像的口子（只交给它数系统配置槽，[`system_configuration_redundancy_report`]）：盘、盘的字节数与读都转给崩溃镜像，
/// 读不出（`PoolReader::read` 交空：设备报读错）的那几块盘记下来。系统配置槽的读法不扫单元区与 journal 环，两样扫描候选交空。
struct CrashImageReadByThePoolChecker<'image> {
    crash_image: &'image dyn PoolReader,
    devices_with_a_read_reporting_an_error: RefCell<BTreeSet<u32>>,
}

impl ImageReader for CrashImageReadByThePoolChecker<'_> {
    fn devices(&self) -> Vec<u32> {
        self.crash_image
            .device_identities()
            .into_iter()
            .map(|device| device.0)
            .collect()
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        self.crash_image
            .device_size_in_bytes(DeviceIdentity(device))
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let bytes =
            self.crash_image
                .read(DeviceIdentity(device), DeviceOffsetInBytes(offset), length);
        if bytes.is_none() {
            self.devices_with_a_read_reporting_an_error
                .borrow_mut()
                .insert(device);
        }
        bytes
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

/// 「读阶段判完之后系统配置又读不出」那一拒的判据在崩溃镜像上复算，判槽自证与逐盘数读池级 checker（改法 G4 那一臂用，
/// 用户 2026-10-22 定「改读池级 checker」，`research/prompts/m3-close-g4-rewrite-r3-main-verification.md`「用户定案（2026-10-22，弹窗）」）：
/// 每块盘几槽自证过取 [`system_configuration_redundancy_report`]，不调核心层的 `SystemConfiguration::parse_slot` 与
/// `choose_system_configuration`（`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」）。
/// 为真 ⟺ 某块盘一个自证过的槽都没有，或判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过）。
/// 设备报读错按池级 checker 读系统配置槽时读不出的盘记（全池没有可用的槽 0 时它逐档读槽 1，那几档里读不出的也记）。
/// 与 U6 复算那一份（[`system_configuration_unreadable_after_the_read_stage_on_the_crash_image`]，核心层口径）的口径差：池级 checker 的
/// 「自证过」不分 fsid、自证过而不可择的槽也算，择不到系统配置不另按真；分得开的只有两盘全废与带别的池的槽那几形，
/// 两盘全废由 I-8.5 与 oracle 另判（同一份判决第二节 Y1 那一行理由 ③）。
fn system_configuration_unreadable_after_the_read_stage_judged_by_the_pool_checker_counts_on_the_crash_image(
    crash_image: &dyn PoolReader,
) -> bool {
    let reader = CrashImageReadByThePoolChecker {
        crash_image,
        devices_with_a_read_reporting_an_error: RefCell::new(BTreeSet::new()),
    };
    let report = system_configuration_redundancy_report(&reader);
    let device_count = reader.devices().len();
    let some_device_without_any_self_verified_slot =
        !report.devices_without_a_self_verified_slot.is_empty();
    let no_device_self_verified_both_slots =
        report.devices_with_fewer_than_two_self_verified_slots.len() == device_count;
    let some_slot_reported_a_read_error = !reader
        .devices_with_a_read_reporting_an_error
        .borrow()
        .is_empty();
    some_device_without_any_self_verified_slot
        || (some_slot_reported_a_read_error && no_device_self_verified_both_slots)
}

/// 判据 N-配置（D23 已定项 14「可写挂载读到的更新状态读不出」那一段）在崩溃镜像上自己复算一遍，不经实现的 `mount::newer_publish_witness`：
/// 系统配置见证判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]，R2）⇒ 按真；否则全部自证过的槽的 journal tail
/// 取最大为 c_见证（撕裂的槽跳过），0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录
/// 读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
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
    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_image
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_read_on_the_crash_image(
                crash_image,
                device,
                slot_spacing,
                &immutable.filesystem_identifier,
            )
        })
        .collect();
    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {
        return true;
    }
    let witnessed_journal_counter = slots_of_every_device
        .iter()
        .flat_map(|slots| slots.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
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
/// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过——
/// 与 [`newer_publish_witnessed_on_the_crash_image`] 改取 R2 之后在撕裂的轮换上同一个取法（某块盘两槽都撕裂、一份自证过的都没有的那一形
/// 两边都照常比：实现拒它报的是另一个拒因，C664（挂载拒绝合法复算把两种拒因报成一个））。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
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

/// 「重建分配器读根环整环重读一次、两遍逐槽并起来仍一条不被抛弃的自证有效根都没有」（Q3，并两遍照用户 2026-10-05 定 R1）在崩溃镜像上复算：
/// 按最新那条根（`choose_root`）指着的实例表，根环里一条不被抛弃的自证有效根都没有。并两遍之后判据照旧只读一遍：崩溃镜像上没有瞬时读错，
/// 两遍读出的逐槽相同，并起来就是那一遍（任一遍自证过的根留着，两遍都没有才算没有）。被抛弃照 D23（journal 的角色与格式） 已定项 14 候选集那一格在这里另写一份：表里有那条根的实例的行
/// (i, Ti, Wi) 且它的 txg > Ti（不调实现的 `recovery::root_is_abandoned_by_the_instance_table`）。择不到系统配置、择不到根都按真；
/// 最新那条根的实例表读不出时没有「被抛弃」可判，不按表滤（读得出的根都算有效，与 F 生效值那一处的取法同向），择得到根就为假。
/// 没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例：写行只给比要取的号小的实例写，`mount.rs` 的
/// `instance_rows_to_write`），所以择得到根的崩溃镜像上这条判据为假；「按表判被抛弃」那半句在合法盘面上不改判据的值，等价性留档在
/// `writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs`
/// 的 `a_refusal_for_no_valid_root_on_a_crash_image_holding_an_abandoned_root_is_illegal_because_the_newest_root_is_never_abandoned_by_its_own_table`。
fn no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
    crash_image: &dyn PoolReader,
) -> bool {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return true;
    };
    let Some(newest) = choose_root(crash_image, &system_configuration) else {
        return true;
    };
    let Some(table_of_the_newest_root) = instance_table_of_root(crash_image, &newest) else {
        return false;
    };
    let immutable = &system_configuration.immutable;
    !readable_roots(
        crash_image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .iter()
    .any(|root| {
        !table_of_the_newest_root
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    })
}

/// 「记录有缺口时拒可写」（D23（journal 的角色与格式） 已定项 14，F1 乙）在崩溃镜像上复算，不调实现的
/// `mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`：水位（`selected_version`，R1 落到的那一版）之上
/// 按 (实例代号, checkpoint_txg) 分组的每一次发布，读得出的记录的本次发布内序号排好之后不是从 1 起逐个加一、或其中没有一条带末条标志，
/// 而它们点名的单元里有一个两份副本都验不过（[`named_unit_has_a_copy_that_verifies_on_the_crash_image`] 为假）——有这样一次发布即为真。
/// 崩溃镜像上没有瞬时读错，重读一次读出的与这一遍相同，判据只算一遍。择不到系统配置按真（与这一族别的复算同向）；
/// 没有所选那一版时水位无从取，交回假。
fn publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> bool {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return true;
    };
    let Some(selected_version) = selected_version else {
        return false;
    };
    let mut records_of_each_publish: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<JournalRecord>,
    > = BTreeMap::new();
    for record in scan_journal(crash_image, &system_configuration).into_values() {
        let publish = (record.instance, record.checkpoint_txg);
        if publish > selected_version {
            records_of_each_publish
                .entry(publish)
                .or_default()
                .push(record);
        }
    }
    records_of_each_publish
        .values()
        .any(|records_of_the_publish| {
            let mut ordinals: Vec<u32> = records_of_the_publish
                .iter()
                .map(|record| record.ordinal_within_publish.0)
                .collect();
            ordinals.sort_unstable();
            let ordinals_run_from_one_without_a_gap = ordinals
                .iter()
                .zip(1u32..)
                .all(|(ordinal, expected_ordinal)| *ordinal == expected_ordinal);
            let carries_the_last_record_of_the_publish =
                records_of_the_publish.iter().any(|record| {
                    record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
                });
            let some_named_unit_fails_on_both_copies = records_of_the_publish
                .iter()
                .flat_map(|record| &record.named)
                .any(|named| {
                    !named_unit_has_a_copy_that_verifies_on_the_crash_image(crash_image, named)
                });
            !(ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish)
                && some_named_unit_fails_on_both_copies
        })
}

/// 一条记录点名的一个单元在崩溃镜像上至少有一份副本验得过：某一条位置条目指的那一份读得出，按单元种类的长度（数据单元、打包单元 32 KiB，
/// 索引节点 16 KiB）读回的整单元 CRC-32C 等于条目里的。种类不认识的点名项（坏盘上的记录）哪一份都不算验得过。
fn named_unit_has_a_copy_that_verifies_on_the_crash_image(
    crash_image: &dyn PoolReader,
    named: &NamedUnit,
) -> bool {
    let unit_length_in_bytes = if named.unit_class == UNIT_CLASS_INDEX_NODE {
        NODE_BYTES
    } else if named.unit_class == UNIT_CLASS_DATA || named.unit_class == UNIT_CLASS_PACKED {
        DATA_UNIT_BYTES
    } else {
        return false;
    };
    let unit_bytes =
        usize::try_from(unit_length_in_bytes).expect("单元字节数 16384 或 32768 装得进 usize");
    named.locations.iter().any(|location| {
        crash_image
            .read(
                location.device,
                location.slot.to_device_offset(),
                unit_bytes,
            )
            .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
    })
}

/// 「C 读一次没施加的发布的账读不出、读得出的记录不齐、从中央映射也现算不成」（`C_MAPPING` 收窄）那一拒在崩溃镜像上复算，不调实现的
/// `mount::placements_in_the_accounts_of_unapplied_publishes` 与 `mount::placements_in_the_accounts_of_newer_readable_roots`。
/// 判据由主 agent 定（2026-10-08，被攻过零轮），三样都成立即为真：
/// 1. 那一拒点名的 `unapplied_publish` 是水位（`selected_version`，R1 落到的那一版，按实例代号为主比）之上没施加的发布——崩溃镜像上读得出它的记录；
///    或是根环里比所选那一版新（txg 为主、实例代号破平局）、按最新那条根的实例表有效、读得出的根。被抛弃照 D23（journal 的角色与格式） 已定项 14
///    候选集那一格另写一份（表里有那条根的实例的行 (i, Ti, Wi) 且它的 txg > Ti）；最新那条根的实例表读不出时不按表滤，与
///    [`no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image`] 同一取法。
/// 2. 那次发布在崩溃镜像上读得出的记录不齐：本次发布内序号排好之后不是从 1 起逐个加一，或没有一条带末条标志（一条都读不出也算没有末条）。
/// 3. 它的账在崩溃镜像上读不出：按新根段里的树表指针找到的账根两份副本都验不过、或账根读不到（[`account_unreadable_on_the_crash_image`]）——
///    读得出的每条记录的新根段，与第 1 条里那条较新的根的根记录，各找一次，每一份都读不出才算。
///
/// 这一格只判必要条件：实现拒之前还要「从中央映射现算不成」，那一半崩溃镜像上不复算——复算为真不说明实现非拒不可，只说拒的理由在镜像上站得住。
/// 账只看到账根那一个节点：实现读账还读分配记录树根以下的节点，只有那几样读不出时实现照拒、这里复算为假（判不合法）。
/// 崩溃镜像上没有瞬时读错，重读一次读出的与这一遍相同，判据只算一遍。择不到系统配置按真（与这一族别的复算同向）；没有所选那一版时水位无从取，交回假。
fn account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    unapplied_publish: (InstanceGeneration, CheckpointTxg),
) -> bool {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        return true;
    };
    let Some(selected_version) = selected_version else {
        return false;
    };
    let readable_records_of_the_publish: Vec<JournalRecord> =
        scan_journal(crash_image, &system_configuration)
            .into_values()
            .filter(|record| (record.instance, record.checkpoint_txg) == unapplied_publish)
            .collect();
    let is_an_unapplied_publish_above_the_water_mark =
        unapplied_publish > selected_version && !readable_records_of_the_publish.is_empty();
    let newer_valid_readable_root = newer_valid_readable_root_on_the_crash_image(
        crash_image,
        &system_configuration,
        selected_version,
        unapplied_publish,
    );
    if !is_an_unapplied_publish_above_the_water_mark && newer_valid_readable_root.is_none() {
        return false;
    }
    let mut ordinals: Vec<u32> = readable_records_of_the_publish
        .iter()
        .map(|record| record.ordinal_within_publish.0)
        .collect();
    ordinals.sort_unstable();
    let ordinals_run_from_one_without_a_gap = ordinals
        .iter()
        .zip(1u32..)
        .all(|(ordinal, expected_ordinal)| *ordinal == expected_ordinal);
    let carries_the_last_record_of_the_publish =
        readable_records_of_the_publish.iter().any(|record| {
            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        });
    if ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish {
        return false;
    }
    readable_records_of_the_publish
        .iter()
        .map(|record| {
            (
                record.new_tree_table,
                record.new_allocation_record_tree_root,
            )
        })
        .chain(
            newer_valid_readable_root
                .iter()
                .map(|root| (root.tree_table, root.allocation_record_tree_root)),
        )
        .all(
            |(tree_table, allocation_record_tree_root_in_the_new_root_segment)| {
                account_unreadable_on_the_crash_image(
                    crash_image,
                    &tree_table,
                    &allocation_record_tree_root_in_the_new_root_segment,
                )
            },
        )
}

/// 一版的账在崩溃镜像上读不出：账根两份副本都验不过（[`node_has_a_copy_that_verifies_on_the_crash_image`] 为假），或连账根在哪都读不到。
/// 账根按那一版的树表找（`mount::placements_referenced_by_root` 读账的同一条路，这里另写一份、不调它）：树表两份都验不过、解不开 ⇒
/// 读不到账根，算读不出；树表 0 条（没写过文件的一版）⇒ 账根是新根段里那条分配记录树根（空指针算读得出：mkfs 的第 0 代与照抄它的暖机根没有这棵树）；
/// 树表有条目 ⇒ 账根是 `TREE_KIND_ALLOCATION` 那一条的根指针（没有那一条算读得出：账里一条记录都没有）。
/// 主 agent 定的判据写的是「新根段里的分配记录树根」；带文件的一版，根记录与新根段里那一条恒是空指针（它的分配记录树住树表条目，
/// D8（核心索引结构） 已定项 8；`transaction.rs` 发布带文件的一版时那一项写 `NodePointer::empty_root()`），不是它的账——
/// 只看那一条，带文件的一版永远判不成读不出。这里按「新根段里的树表指针找到的账根」读，交主 agent 定（第 43 批报告）。
/// 只看账根那一个节点：账根以下的节点读不出时实现照拒、这里判读得出。
fn account_unreadable_on_the_crash_image(
    crash_image: &dyn PoolReader,
    tree_table: &NodePointer,
    allocation_record_tree_root_in_the_new_root_segment: &NodePointer,
) -> bool {
    let node_bytes = usize::try_from(NODE_BYTES).expect("节点字节数 16384 装得进 usize");
    let Some(tree_table_bytes) = tree_table.locations.iter().find_map(|location| {
        crash_image
            .read(
                location.device,
                location.slot.to_device_offset(),
                node_bytes,
            )
            .filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum)
    }) else {
        return true;
    };
    let Ok(tree_table_node) = parse_index_node(&tree_table_bytes) else {
        return true;
    };
    let account_root = if tree_table_node.entries.is_empty() {
        *allocation_record_tree_root_in_the_new_root_segment
    } else {
        match TreeTableEntryRootPointersByKind::of_the_entries(
            &tree_table_node.entries,
            &[TREE_KIND_ALLOCATION],
        )
        .root_pointer_of_each_kind
        .get(&TREE_KIND_ALLOCATION)
        {
            Some(root_pointer_bytes) => {
                match crate::verification_read::node_pointer(root_pointer_bytes) {
                    Some(pointer) => pointer,
                    None => return true,
                }
            }
            None => return false,
        }
    };
    account_root != NodePointer::empty_root()
        && !node_has_a_copy_that_verifies_on_the_crash_image(crash_image, &account_root)
}

/// 根环里那条 (实例代号, checkpoint_txg) 等于 `publish` 的根，它比所选那一版新（择根的序：txg 为主、实例代号破平局）、读得出、按最新那条根
/// （`choose_root`）的实例表不被抛弃（表里有它的实例的行 (i, Ti, Wi) 且它的 txg > Ti 才算被抛弃，不调实现的
/// `recovery::root_is_abandoned_by_the_instance_table`；那张表读不出时不按表滤）：交回那条根，没有交 `None`。
fn newer_valid_readable_root_on_the_crash_image(
    crash_image: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    selected_version: (InstanceGeneration, CheckpointTxg),
    publish: (InstanceGeneration, CheckpointTxg),
) -> Option<RootRecord> {
    let table_of_the_newest_root = choose_root(crash_image, system_configuration)
        .and_then(|newest| instance_table_of_root(crash_image, &newest));
    let immutable = &system_configuration.immutable;
    readable_roots(
        crash_image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        let is_abandoned = table_of_the_newest_root.as_ref().is_some_and(|table| {
            table.rows.iter().any(|row| {
                row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg
            })
        });
        (root.instance, root.checkpoint_txg) == publish
            && (root.checkpoint_txg, root.instance) > (selected_version.1, selected_version.0)
            && !is_abandoned
    })
}

/// 一条码 2 / 码 3 指针指的节点在崩溃镜像上至少有一份副本验得过：某一条位置条目指的那一份读得出，16 KiB 读回的整单元 CRC-32C 等于条目里的。
fn node_has_a_copy_that_verifies_on_the_crash_image(
    crash_image: &dyn PoolReader,
    pointer: &NodePointer,
) -> bool {
    let node_bytes = usize::try_from(NODE_BYTES).expect("节点字节数 16384 装得进 usize");
    pointer.locations.iter().any(|location| {
        crash_image
            .read(
                location.device,
                location.slot.to_device_offset(),
                node_bytes,
            )
            .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
    })
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

/// 被拒的这次可写挂载自己在不在读故障下跑（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
/// 「注入读故障的态上『新拒因复算为假不算不合法』（C3 / J2-a）」那一行，改法 G3）。与 [`ReadFaultsInTheCrashState`] 是两件事：
/// 那一样说造出崩溃镜像的历史里有没有读故障步，这一样说被拒的那次挂载读盘时有没有故障层——崩溃注入第二、三截那次挂载起在崩溃镜像上、
/// 盘是不带故障层的内存盘，历史里注入过读故障也不会在这次挂载里出瞬时读错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFaultsUnderTheRefusedMount {
    /// 被拒的那次挂载读的就是崩溃镜像上的字节、不带故障层（崩溃注入第二、三截，E171 装置）：瞬时读错不会发生，
    /// 拒的判据在崩溃镜像上复算为假就是拒错了。
    TheRefusedMountReadTheCrashImageWithoutFaults,
    /// 被拒的那次挂载的读经故障层、读错可以是瞬时的：两遍读之间的瞬时读错在崩溃镜像上复算不出
    /// （C664（挂载拒绝合法复算把两种拒因报成一个）判别力自证里取号那一刻瞬时读错的态）。
    TheRefusedMountReadThroughInjectedFaults,
}

impl ReadFaultsUnderTheRefusedMount {
    /// 「读阶段判完之后系统配置又读不出」与「重建分配器读根环整环重读一次仍一条有效根都没有」（Q3）那两拒的判据在崩溃镜像上复算为假，
    /// 能不能是被拒那次挂载里一次瞬时读错（复算不出）。
    fn can_hide_a_transient_read_error_from_the_crash_image(self) -> bool {
        match self {
            Self::TheRefusedMountReadTheCrashImageWithoutFaults => false,
            Self::TheRefusedMountReadThroughInjectedFaults => true,
        }
    }
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
    /// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）：
    /// 崩溃态没注入读故障，拒因是「读阶段判完之后系统配置又读不出」，那一拒的判据在崩溃镜像上复算为真（按池级 checker 的逐盘数，某块盘
    /// 一个自证过的系统配置槽都没有，或见证判不出；`system_configuration_unreadable_after_the_read_stage_judged_by_the_pool_checker_counts_on_the_crash_image`）——mkfs 返回之后没有读故障的历史留不下这份盘面，正确实现在它上面永远拒可写；与 C670 那一条同形，拒是对的、造出这份盘面的那一段不对。
    /// 射程只到 mkfs 返回之后的崩溃态（D13（验证路线） 已定项 6）：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，mkfs 返回之前的崩溃态留得下这份盘面，
    /// 不归这一条判；崩溃注入第二、三截的候选段都取在 mkfs 之后。
    SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
    /// 与改法 G4 同形（用户 2026-10-05 定 Q3，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）：崩溃态没注入读故障，
    /// 拒因是「重建分配器读根环整环重读一次、两遍并起来仍一条不被抛弃的自证有效根都没有」，那一拒的判据在崩溃镜像上复算为真（按最新那条根的实例表，
    /// 环里一条不被抛弃的自证有效根都没有）——没有读故障的历史留不下这份盘面（读阶段择得到根、它按自己的实例表不被抛弃），正确实现在它上面
    /// 永远拒可写；拒是对的、造出这份盘面的那一段不对。
    NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
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

/// 判一次拒可写合不合法，只判 r5 判决 U6 那四条，不判 C670 与改法 G4 那两条（崩溃态知道注没注入读故障的调用方用
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

/// 同 [`judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`]，被拒的那次挂载读的就是崩溃镜像、
/// 不带故障层（[`ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults`]：在崩溃镜像上起一次挂载判它的调用方，
/// 如 E171 装置）。
#[must_use]
pub fn judge_writable_mount_refusal_of_a_crash_state(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    refusal: &MountError,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    read_faults: ReadFaultsInTheCrashState,
) -> WritableMountRefusalVerdict {
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
        crash_image,
        selected_version,
        refusal,
        disk,
        read_only_read_back,
        read_faults,
        ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
    )
}

/// 判崩溃注入第二、三截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
/// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
/// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）；与它同形的改法 G4——崩溃态没注入读故障、
/// 理由是「读阶段判完之后系统配置又读不出」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
/// （[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）；与 G4 同形的 Q3——
/// 崩溃态没注入读故障、理由是「重建分配器读根环整环重读一次、两遍并起来仍一条不被抛弃的自证有效根都没有」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
/// （[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态
/// 这三条都不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」或 Q3 那一拒而被拒的那次挂载
/// 自己在读故障下跑的（`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算
/// 把两种拒因报成一个））；被拒的那次挂载不带故障层的，崩溃态注没注入过读故障都照 U6 复算（判据为假即不合法，改法 G3）。
#[must_use]
pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
    crash_image: &dyn PoolReader,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    refusal: &MountError,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    read_faults: ReadFaultsInTheCrashState,
    read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,
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
        // 改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在 mkfs 返回之后没有读故障的历史里留不下它。
        // mkfs 返回之前的崩溃态不在这一条的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，那一段留得下两槽都自证不过的盘
        // （D13（验证路线） 已定项 6）；调这一臂的崩溃注入第二、三截，候选段都取在 mkfs 之后。判槽自证读池级 checker 的逐盘数
        // （用户 2026-10-22 定「改读池级 checker」），不用核心层的解析。
        (
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
            ReadFaultsInTheCrashState::NoneInjected,
        ) => {
            if system_configuration_unreadable_after_the_read_stage_judged_by_the_pool_checker_counts_on_the_crash_image(
                crash_image,
            ) {
                conditions_not_met.push(
                    RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
                );
            }
        }
        // Q3 与 G4 同形：没有读故障时设备不报读错，读阶段择得到的根按它自己的实例表不被抛弃——这一拒的判据为真只能是盘面本身如此。
        (
            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            ReadFaultsInTheCrashState::NoneInjected,
        ) => {
            if no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
                crash_image,
            ) {
                conditions_not_met.push(
                    RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
                );
            }
        }
        (
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
            | RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
            | RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            ReadFaultsInTheCrashState::Injected,
        )
        | (
            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
            | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
            | RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                ..
            },
            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
        ) => {}
    }
    // 读阶段之后系统配置又读不出、被拒的那次挂载自己在读故障下跑：两遍读之间的瞬时读错在崩溃镜像上复算不出，判据复算为假不算拒得不合法
    // （C664（挂载拒绝合法复算把两种拒因报成一个）的还清条件：带读故障的档里取号那一刻瞬时读错的态，分开报之后判合法）；
    // 盘上逐字节不变与只读读出照判。被拒的那次挂载不带故障层的照 U6 复算（判据为假即不合法）。
    // 重建分配器读根环重读仍一条有效根都没有（Q3）同一个办法：重建那一遍与它的重读读坏是这次挂载里的瞬时读错，崩溃镜像上复算不出。
    match reason {
        RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
        | RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator => {
            if read_faults_under_the_refused_mount
                .can_hide_a_transient_read_error_from_the_crash_image()
            {
                conditions_not_met.retain(|condition| {
                    *condition
                        != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
                });
            }
        }
        // 「记录有缺口时拒可写」那一拒没有这一条宽限：被拒的那次挂载在读故障下跑、判据在崩溃镜像上复算为假的照判不合法
        // （D23 已定项 14 那一段只写判据、没写读故障下这一拒怎么判；取不放宽的一边：复算不出的报红，不藏）。
        // `C_MAPPING` 那一拒同一个取法（主 agent 2026-10-08 定，被攻过零轮）。
        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
        | RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
        | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
        | RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            ..
        } => {}
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
        RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage => {
            system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)
        }
        RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator => {
            no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
                crash_image,
            )
        }
        RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify => {
            publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image(
                crash_image,
                selected_version,
            )
        }
        RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            unapplied_publish,
        } => account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image(
            crash_image,
            selected_version,
            unapplied_publish,
        ),
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
