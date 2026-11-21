//! 挂载透明（独立的第四条，不是 M3 的第三读：`research/prompts/m3-formal-r4-main-verification.md` 第四节第 3 条；
//! 逐字段表与二次崩溃、拒绝合法照 `research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条，零轮）的判法。
//!
//! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F，
//! 连同挂载续做与续做里推的抬 F：「续做算挂载的一部分」），
//! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
//! 只读恢复读出的与 R1 比（F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]；
//! 抬 F 的界在挂载写表里对不上位置的那一次挂载，二次崩溃整段 F 不判、按 [`FieldNotJudged`] 计数）。
//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]；崩溃注入第二、三截再加两条：
//! 没注入读故障的崩溃态上判据 N-配置 复算为真（C670），或「读阶段判完之后系统配置又读不出」那一拒的判据复算为真（改法 G4，射程只到 mkfs 返回之后的崩溃态），
//! 正确实现摆不出这份盘面，拒得不合法，[`judge_writable_mount_refusal_of_a_crash_state_of_the_pool`]）。
//!
//! 读盘的只有 [`read_recovery_fields`] 与那两个判拒绝的函数里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
//! 只吃读好的字段与挂载交回值。崩溃注入调它，harness 用例也调它。
//! r5 判决标「推的」的几样（树 ID 水位、两个根指针、txg、扫描报告）照写进判法，每一样红的时候在交回值里单独标出它是推的还是量过的
//! （[`MountTransparencyField::basis`]）。

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::verification_read::{
    choose_root, choose_system_configuration, choose_system_configuration_of_the_pool,
    chosen_system_configuration_slot_bytes, effective_rollback_floor, highest_root_instance,
    highest_root_txg, instance_table_of_root, readable_roots, root_record_bytes_of_the_version,
    scan_journal, tree_table_entry_bytes_of_root, verified_system_configuration_slots,
};
use singlefs_checker::crc32_castagnoli_table as crc32_castagnoli;
use singlefs_checker::image::{system_configuration_redundancy_report, ImageReader};
use singlefs_checker::index_node_view as parse_index_node;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish, NamedUnit};
use singlefs_core::mount::{
    MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure, OpenPoolForReadFailure};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
    TREE_KIND_RETIREMENT, TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
};
use singlefs_core::recovery::{
    recover, recover_of_the_pool, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{
    DATA_UNIT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::crash::{CrashPointReplayObserverCounts, CrashPointReplayTally};
use crate::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObservingOfTheRun,
    ReadRangeOnTheCrashImage,
};
use crate::memory_pool::{
    root_identity_written_by, writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordingBlockDevice, SharedStream};

mod floor_raises_and_continuation_root_pointers;
mod writable_mount_refusal;

pub use self::floor_raises_and_continuation_root_pointers::*;
pub use self::writable_mount_refusal::*;

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

/// R2 拿哪一版来比：只挂载与二次崩溃比恢复落到的那一版；挂载加一次发布比挂载后那次发布之前那一版（那一行）。
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
/// 字段没有不变量，直接公开：harness 的用例改坏其中一样，证判法会红。
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
            let content = match crate::verification_read::first_file_content_read_through_the_root(
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

/// 做的是哪个动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountTransparencyAction {
    /// 只挂载，不接发布：R2 比它恢复落到的那一版（v2a）。
    MountAlone(FloorRaisedByTheMount),
    /// 挂载加一次发布：R2 比挂载后那次发布之前那一版。
    MountAndOnePublish(FloorRaisedByTheMount),
    /// 挂载途中再崩一次：R2 是二次崩溃镜像上的只读恢复，比的对象是 v1。
    SecondCrashInsideTheMount(FloorRaisesAtTheSecondCrash),
}

/// 比的十八样： 那一行的逐字段表十样，加判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
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

    /// 这一样在这个动作下的关系是量过的还是推的。二次崩溃那一截的字段一半整截是推的，
    /// F 在挂载推了抬 F 时那一半（「= 挂载交回的最后一串抬到的 F」）r5 判决写的是「后半没量」。
    /// c655 那一轮补的八样（五样关系、三张字节表）一律记推的：攻方量的是副本装置列它们为零轮，入库代码上还没量过。
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

    /// 判红时记进违例的名字（崩溃注入按它分签名）：量过的与推的各一个名字，提交时的门禁一眼分得开。
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
/// （第 3 条：那张表按 Y1、Y2 改行由写回批写；这里引的行名有几行是写回批加的，见 [`MOUNT_TRANSPARENCY_TABLE_ROWS_ADDED_BY_THE_WRITE_BACK`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldByteRange {
    pub name: &'static str,
    pub bytes: std::ops::Range<usize>,
    pub class: FieldByteRangeClass,
    pub row_of_the_mount_transparency_field_table: &'static str,
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
const MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION: &str =
    "池身份与系统配置的不可变配置（本盘设备号除外）";
const MOUNT_TRANSPARENCY_TABLE_UNMOUNT_MARKER: &str = "卸载记号";
const MOUNT_TRANSPARENCY_TABLE_ROLLBACK_FLOOR: &str = "F 与 F 生效值";
const MOUNT_TRANSPARENCY_TABLE_INSTANCE_GENERATION: &str = "实例代号";
const MOUNT_TRANSPARENCY_TABLE_INSTANCE_TABLE_ROWS: &str = "实例表行";
const MOUNT_TRANSPARENCY_TABLE_CHECKPOINT_TXG: &str = "`checkpoint_txg`";
const MOUNT_TRANSPARENCY_TABLE_TREE_IDENTIFIER_WATERMARK: &str = "树 ID 水位";
/// 那张表的首格还带全角括号注「后三样是树表条目，种类 9、10、11，不是根记录字段」；行名常量写一行装得下的、去掉注的那一截
/// （门禁 doc-registries 的 mount-transparency-fields 格读 `const MOUNT_TRANSPARENCY_TABLE_…: &str = "…";`，值折到下一行的也认，比名字时两边都去掉全角括号注）。
const MOUNT_TRANSPARENCY_TABLE_TREE_ROOTS: &str =
    "树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针";
const MOUNT_TRANSPARENCY_TABLE_ALLOCATION_RECORDS: &str = "分配记录";
/// 定下、写回批加进 D13 逐字段表的几行（写回批已写进；行名照今天那张表的首格，全角括号注省掉）。
const MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS: &str =
    "magic、flags 其余位、自证校验和、整槽校验和";
/// 那张表的首格是「根槽偏移 437..466（算法、nonce、MAC）」。
const MOUNT_TRANSPARENCY_TABLE_ROOT_SLOT_RESERVED_BYTES: &str = "根槽偏移 437..466";
/// 根记录的回收门槛 t 与钉槽 p（C773（读坏与全零根槽的回收门槛钉只在内存里））；那张表的首格带注「根槽偏移 428..437」。
const MOUNT_TRANSPARENCY_TABLE_ROOT_RECORD_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT: &str =
    "回收门槛 t 与钉槽 p";
const MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_SLOT_GENERATION: &str = "系统配置槽世代号";
const MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_JOURNAL_TAIL: &str = "系统配置的 journal_tail";
const MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE: &str =
    "系统配置的 journal 实例代号";
const MOUNT_TRANSPARENCY_TABLE_MUTABLE_CONFIGURATION: &str = "系统配置的节点大小";
/// 那张表的首格带注「T_time、T_dirty、三水位」。
const MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION: &str = "系统配置的运行配置 36 字节";
const MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY: &str =
    "树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类";
/// 根记录里实例表、树表两个单元指针（不比字节，表里的行与条目另由「实例表行」与树表条目那张表比）。
const MOUNT_TRANSPARENCY_TABLE_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS: &str =
    "实例表与树表的单元指针";

/// 这几张表引的 D13 行名里，写回批之前逐字段表就有的那几行（[`MOUNT_TRANSPARENCY_TABLE_ROWS_ADDED_BY_THE_WRITE_BACK`] 之外的）。
/// 门禁 doc-registries 的 mount-transparency-fields 格把这两组与三张字段表引的行名一起按今天的 D13 表比。
pub const MOUNT_TRANSPARENCY_TABLE_ROWS_OF_TODAY: [&str; 9] = [
    MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION,
    MOUNT_TRANSPARENCY_TABLE_UNMOUNT_MARKER,
    MOUNT_TRANSPARENCY_TABLE_ROLLBACK_FLOOR,
    MOUNT_TRANSPARENCY_TABLE_INSTANCE_GENERATION,
    MOUNT_TRANSPARENCY_TABLE_INSTANCE_TABLE_ROWS,
    MOUNT_TRANSPARENCY_TABLE_CHECKPOINT_TXG,
    MOUNT_TRANSPARENCY_TABLE_TREE_IDENTIFIER_WATERMARK,
    MOUNT_TRANSPARENCY_TABLE_TREE_ROOTS,
    MOUNT_TRANSPARENCY_TABLE_ALLOCATION_RECORDS,
];

/// 这几张表引的 D13 行名里，写回批加进逐字段表的那几行。
pub const MOUNT_TRANSPARENCY_TABLE_ROWS_ADDED_BY_THE_WRITE_BACK: [&str; 10] = [
    MOUNT_TRANSPARENCY_TABLE_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS,
    MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
    MOUNT_TRANSPARENCY_TABLE_ROOT_SLOT_RESERVED_BYTES,
    MOUNT_TRANSPARENCY_TABLE_ROOT_RECORD_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT,
    MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_SLOT_GENERATION,
    MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_JOURNAL_TAIL,
    MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
    MOUNT_TRANSPARENCY_TABLE_MUTABLE_CONFIGURATION,
    MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION,
    MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
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
    row_of_the_mount_transparency_field_table: &'static str,
) -> FieldByteRange {
    FieldByteRange {
        name,
        bytes,
        class: FieldByteRangeClass::MustBeTheSame,
        row_of_the_mount_transparency_field_table,
    }
}

const fn not_compared(
    name: &'static str,
    bytes: std::ops::Range<usize>,
    judged_instead_by: &'static str,
    row_of_the_mount_transparency_field_table: &'static str,
) -> FieldByteRange {
    FieldByteRange {
        name,
        bytes,
        class: FieldByteRangeClass::NotCompared { judged_instead_by },
        row_of_the_mount_transparency_field_table,
    }
}

const fn determined_relation(
    name: &'static str,
    bytes: std::ops::Range<usize>,
    judged_by: MountTransparencyField,
    row_of_the_mount_transparency_field_table: &'static str,
) -> FieldByteRange {
    FieldByteRange {
        name,
        bytes,
        class: FieldByteRangeClass::DeterminedRelation { judged_by },
        row_of_the_mount_transparency_field_table,
    }
}

const READ_WHEN_THE_SLOT_IS_READ: &str = "读槽时判（不对这一槽不可择）";

/// 根记录 [0, `ROOT_RECORD_BYTES`) 按字段逐段（D22（单元原子性怎么合成） 已定项 7 的字段表 17 行；回收门槛 t 与钉槽 p 合一段，算法、nonce、MAC 合一段）。
pub const ROOT_RECORD_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
    not_compared(
        "magic",
        0..4,
        READ_WHEN_THE_SLOT_IS_READ,
        MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
    ),
    must_be_the_same("fsid", 4..20, MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION),
    determined_relation(
        "flags（位 0 是卸载记号；其余位非 0 读槽时拒收）",
        20..24,
        MountTransparencyField::UnmountMarker,
        MOUNT_TRANSPARENCY_TABLE_UNMOUNT_MARKER,
    ),
    determined_relation(
        "实例代号",
        ROOT_RECORD_INSTANCE,
        MountTransparencyField::InstanceGeneration,
        MOUNT_TRANSPARENCY_TABLE_INSTANCE_GENERATION,
    ),
    determined_relation(
        "checkpoint_txg",
        28..36,
        MountTransparencyField::CheckpointTxg,
        MOUNT_TRANSPARENCY_TABLE_CHECKPOINT_TXG,
    ),
    not_compared(
        "树表单元指针",
        36..122,
        "树表单元自己的落点每次发布都变、不比；它的条目按树表条目那张表逐段比",
        MOUNT_TRANSPARENCY_TABLE_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS,
    ),
    determined_relation(
        "树 ID 水位",
        122..130,
        MountTransparencyField::TreeIdentifierWatermark,
        MOUNT_TRANSPARENCY_TABLE_TREE_IDENTIFIER_WATERMARK,
    ),
    determined_relation(
        "回退下界 F",
        130..138,
        MountTransparencyField::RollbackFloor,
        MOUNT_TRANSPARENCY_TABLE_ROLLBACK_FLOOR,
    ),
    not_compared(
        "自证校验和",
        138..170,
        READ_WHEN_THE_SLOT_IS_READ,
        MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
    ),
    not_compared(
        "实例表单元指针",
        170..256,
        "实例表单元的落点不比；表里的行由「实例表行」那一样比",
        MOUNT_TRANSPARENCY_TABLE_INSTANCE_TABLE_AND_TREE_TABLE_UNIT_POINTERS,
    ),
    determined_relation(
        "中央映射树根指针",
        ROOT_RECORD_MAPPING_ROOT,
        MountTransparencyField::MappingRootEmptiness,
        MOUNT_TRANSPARENCY_TABLE_ALLOCATION_RECORDS,
    ),
    determined_relation(
        "分配记录树根指针",
        ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT,
        MountTransparencyField::AllocationRecordTreeRoot,
        MOUNT_TRANSPARENCY_TABLE_ALLOCATION_RECORDS,
    ),
    not_compared(
        "回收门槛 t、钉槽 p",
        428..437,
        "每次带单元的发布按这个进程的根环与钉现算、零单元发布照抄上一版，挂载前后不比；池级 checker 的 I-7.18、I-7.19 判（单镜像性质）",
        MOUNT_TRANSPARENCY_TABLE_ROOT_RECORD_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT,
    ),
    not_compared(
        "算法类型、nonce、MAC",
        437..466,
        "池级 checker 的 I-2.4 判全 0（单镜像性质）",
        MOUNT_TRANSPARENCY_TABLE_ROOT_SLOT_RESERVED_BYTES,
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
        must_be_the_same(
            name,
            bytes,
            MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION,
        ),
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
            MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
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
            MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION,
        ),
        SystemConfigurationTier::ImmutableConfiguration,
    ),
    immutable_and_the_same("设备数 devs", 143..147),
    system_configuration_field(
        not_compared(
            "槽世代号",
            147..155,
            "池级 checker 的 I-7.16 在 R2 的镜像上判",
            MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_SLOT_GENERATION,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    system_configuration_field(
        not_compared(
            "整槽校验和",
            155..187,
            READ_WHEN_THE_SLOT_IS_READ,
            MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
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
        must_be_the_same(
            "节点大小",
            301..305,
            MOUNT_TRANSPARENCY_TABLE_MUTABLE_CONFIGURATION,
        ),
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
            MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
    system_configuration_field(
        determined_relation(
            "T_dirty",
            437..445,
            MountTransparencyField::SystemRuntimeConfiguration,
            MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
    system_configuration_field(
        determined_relation(
            "整理水位预留",
            445..469,
            MountTransparencyField::SystemRuntimeConfiguration,
            MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
    system_configuration_field(
        determined_relation(
            "journal tail",
            SYSTEM_CONFIGURATION_JOURNAL_TAIL,
            MountTransparencyField::SystemConfigurationJournalTail,
            MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_JOURNAL_TAIL,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    system_configuration_field(
        determined_relation(
            "journal 实例代号",
            SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
            MountTransparencyField::SystemConfigurationJournalInstance,
            MOUNT_TRANSPARENCY_TABLE_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
    system_configuration_field(
        not_compared(
            "回退下界 F",
            481..489,
            "写低由池级 checker 的 I-7.12 判；F 生效值由「F 与 F 生效值」那一样比",
            MOUNT_TRANSPARENCY_TABLE_ROLLBACK_FLOOR,
        ),
        SystemConfigurationTier::RuntimeQuantities,
    ),
];

/// 树表条目 [0, 200) 按字段逐段（D8（核心索引结构） 已定项 8 的字段表：树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 +
/// `previous_snapshot_txg` 8 + 诞生 txg 8 + 头 ID 8 + 预留 76）。条目按盘上下标一一对上比（两版条数不同判红）。
pub const TREE_TABLE_ENTRY_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
    must_be_the_same("树 ID", 0..8, MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY),
    must_be_the_same(
        "条目长度",
        8..10,
        MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
    ),
    must_be_the_same(
        "种类",
        TREE_TABLE_ENTRY_KIND,
        MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
    ),
    must_be_the_same(
        "flags",
        12..14,
        MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
    ),
    determined_relation(
        "根指针",
        TREE_TABLE_ENTRY_ROOT_POINTER,
        MountTransparencyField::TreeTableEntryRootPointers,
        MOUNT_TRANSPARENCY_TABLE_TREE_ROOTS,
    ),
    must_be_the_same(
        "previous_snapshot_txg",
        100..108,
        MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
    ),
    must_be_the_same(
        "诞生 txg",
        108..116,
        MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
    ),
    must_be_the_same(
        "头 ID",
        116..124,
        MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
    ),
    FieldByteRange {
        name: "预留",
        bytes: 124..200,
        class: FieldByteRangeClass::MustBeZero,
        row_of_the_mount_transparency_field_table: MOUNT_TRANSPARENCY_TABLE_TREE_ENTRY_IDENTITY,
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
/// 盘上编码单位毫秒是，判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2，
/// `.claude/kb/layout/01-first-txn.md` 系统配置字段表「可调值 | T_time | 4」那一行由写回批写明单位毫秒、默认值 5000）。照条款写死，不取被测 core 的常量：
/// 期望取自被测者时，core 把默认值或单位改错，挂载写槽写的与期望一起错，这一行判不出
/// （判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」那一行，改法 G1）。
const BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES: u32 = 5000;
/// T_dirty 的内置默认值，单位字节：2 GiB（D16（发布语义） 已定项 5；字段表「可调值 | T_dirty | 8 | 2 GiB」那一行。挂载时的有效值另按环长夹取，
/// 不改盘上这 8 字节）。
const BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES: u64 = 2 * 1024 * 1024 * 1024;
/// 整理水位预留的三个字：全 0（字段表「可调值 | 整理水位预留 | 8 × 3 | 全 0」那一行；三条整理水位是挂载参数、不存盘，
/// D26（后台整理与放置回收） 已定项 1）。
const RESERVED_COMPACTION_WATERMARK_WORDS_OF_THE_CLAUSES: [u64; 3] = [0, 0, 0];

/// 系统运行配置那 36 字节的内置默认值：照字段表的次序（T_time 4、T_dirty 8、整理水位预留 8 × 3，小端），值照条款写死
/// （[`BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES`] 那三个常量；今天没有挂载选项通道，写槽总写内置默认）。
#[must_use]
pub fn built_in_runtime_configuration_bytes() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len());
    bytes.extend_from_slice(&BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES.to_le_bytes());
    bytes.extend_from_slice(&BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES.to_le_bytes());
    for reserved_word in RESERVED_COMPACTION_WATERMARK_WORDS_OF_THE_CLAUSES {
        bytes.extend_from_slice(&reserved_word.to_le_bytes());
    }
    assert_eq!(
        bytes.len(),
        SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len(),
        "系统运行配置一档 36 字节：T_time 4 + T_dirty 8 + 整理水位预留 8 × 3"
    );
    bytes
}

fn read_u32_in_range(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u32> {
    bytes
        .get(range)
        .map(|field| u32::from_le_bytes(field.try_into().expect("切的是 4 字节")))
}

fn read_u64_in_range(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u64> {
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

/// 挂载透明的逐字段判（两行）。`before_the_mount` 是崩溃镜像上的只读恢复（R1，比恢复落到的那一版），
/// `after_the_action` 是动作之后的只读恢复（R2：只挂载与二次崩溃比恢复落到的那一版，挂载加一次发布比发布之前那一版）。
///
/// 动作做成之后的那一版：POSIX 内容与存在性相同；不可变配置（除本盘设备号）相同；卸载记号恒为「不是卸载写的」、R1 的记号不进比较；
/// 挂载没推抬 F 时 R2 的 F 生效值与 R2 根里的 F 都 = R1 的 F 生效值，推了时 R2 的 F 生效值 = 挂载交回的最后一串抬到的 F；
/// 实例代号 = max(R1 镜像上各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1；实例表行照 D18 已定项 11「恢复写的行」；
/// 树 ID 水位相同（挂载不发第一个文件）；树表里 extent、inode、deleted_inodes、墓碑、谱系五种树的根指针相同——挂载续做过时
/// （「续做算挂载的一部分」）比挂载交回的续做那几步里 txg 不大于 R2 那一版的最近一步的
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

/// 系统配置 journal_tail 那一样（[`MountTransparencyField::SystemConfigurationJournalTail`] journal_tail 那一行取攻方的式子）：
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
            .and_then(|bytes| read_u64_in_range(bytes, SYSTEM_CONFIGURATION_JOURNAL_TAIL))
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

/// 系统配置的 journal 实例代号那一样（[`MountTransparencyField::SystemConfigurationJournalInstance`] 那一行取攻方的式子）：
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
            .and_then(|bytes| read_u32_in_range(bytes, SYSTEM_CONFIGURATION_JOURNAL_INSTANCE))
    };
    let observed = instance_of(after_the_action);
    match action {
        MountTransparencyAction::MountAlone(_) | MountTransparencyAction::MountAndOnePublish(_) => {
            let root_instance =
                read_u32_in_range(&compared.root_record_bytes, ROOT_RECORD_INSTANCE);
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
