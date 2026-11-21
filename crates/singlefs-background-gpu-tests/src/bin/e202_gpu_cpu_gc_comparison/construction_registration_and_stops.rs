//! 造池登记的发布形状与停机：文件与单元的算式、登记的发布数、核心层拒打与发布形状不符时的停机。

use super::*;

/// 造数据挂载建空 inode 时一次发布至多建几个（一片 inode 叶容器装 233 条，4096 个约 18 片；文件一次建完的那一次发布太大）。
pub(super) const INODES_CREATED_BY_ONE_PUBLISH: u64 = 4096;

/// 一个活数据文件装几个数据单元 K（登记第十二节修订第 14 条「合并的个数」：一次发布写满一个文件的 16 个单元）。P 个一单元文件装不进一层 inode 树：
/// 树还不会长高（`crates/singlefs-core/src/inode_tree.rs` 的 `InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds`），一层装几条由
/// [`inode_records_one_level_tree_holds`] 照核心层现算（一片叶容器 233 条 × 一个码 2 根扇出 135 = 31455），三档的 P 是 36608、65536、131072。
/// 取 16：一个文件 16 个单元 × 2 槽正好占半个聚簇段，活数据文件与它那个交错垫片文件合起来占满一段；P 是 32 的倍数，P ÷ 2 就是 16 的倍数，
/// 顺序写那一半与交错那一半各自从文件边界起；16 不超过 extent 下段一片叶罩的 144 个单元，每个文件的下段只有一片叶。
pub(super) const LIVE_DATA_UNITS_PER_FILE: u64 = 16;

/// 一个交错垫片文件装几个单元：与活数据文件等长（补齐文件另算，装 a 个，[`alignment_units_to_the_next_segment_boundary`]）。
pub(super) const SPACER_UNITS_PER_FILE: u64 = LIVE_DATA_UNITS_PER_FILE;

/// 补齐文件的个数：一个（补齐那一步 a = 0 时它留空，照样建、照样删）。
pub(super) const ALIGNMENT_FILES: u64 = 1;

/// 活数据文件的个数 P ÷ K。
pub(super) const fn live_data_files(live_data_units: u64) -> u64 {
    live_data_units / LIVE_DATA_UNITS_PER_FILE
}

/// 活数据单元第 `unit_index` 个住第几个活数据文件（从 `lowest_live_data_inode` 数起）、是那个文件里第几个单元。
pub(super) const fn file_and_index_of_a_live_data_unit(unit_index: u64) -> (u64, u64) {
    (
        unit_index / LIVE_DATA_UNITS_PER_FILE,
        unit_index % LIVE_DATA_UNITS_PER_FILE,
    )
}

/// 交错垫片文件的个数 P ÷ 32：交错那一半每个活数据文件配一个等长的垫片文件。
pub(super) const fn interleaved_spacer_files(live_data_units: u64) -> u64 {
    live_data_units / 2 / SPACER_UNITS_PER_FILE
}

/// 垫片文件的个数 P ÷ 32 + 1：交错垫片文件加补齐文件（造碎片挂载逐个删的就是这些）。
pub(super) const fn spacer_files_of_the_construction(live_data_units: u64) -> u64 {
    interleaved_spacer_files(live_data_units) + ALIGNMENT_FILES
}

/// 造数据挂载一次建齐的 inode 个数 P ÷ 16 + P ÷ 32 + 1：活数据文件、补齐文件、交错垫片文件。
pub(super) const fn inodes_created_by_the_data_mount(live_data_units: u64) -> u64 {
    live_data_files(live_data_units) + spacer_files_of_the_construction(live_data_units)
}

/// 造一档的池 inode 树至多要装几条记录：造数据挂载之前已有的、活数据文件、垫片文件，加造碎片挂载推发布建的空 inode 至多
/// [`RELEASE_ROUNDS_AT_MOST`] 个（那时垫片文件已删掉，这里照样加上，往大里算）。
pub(super) const fn inode_records_of_the_construction_at_most(
    live_data_units: u64,
    inode_records_before_the_data_mount: u64,
) -> u64 {
    inode_records_before_the_data_mount
        + live_data_files(live_data_units)
        + spacer_files_of_the_construction(live_data_units)
        + RELEASE_ROUNDS_AT_MOST
}

/// 号从 1 起连着的 `records` 条 inode 记录一次写进空树，核心层写记录那一步（`inode_tree::write_records_into_leaf_containers`）交回什么。
/// 叶只在右端满了才分裂，片数只由条数定；出生树、出生代只进容器身份，不影响片数。
fn inode_records_written_into_an_empty_tree(
    records: u64,
) -> Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal> {
    let records: Vec<InodeRecord> = (1..=records)
        .map(|inode| InodeRecord {
            inode,
            object_birth: CheckpointTxg(1),
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 0,
            change_count: 0,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
        })
        .collect();
    write_records_into_leaf_containers(&[], &records, CheckpointTxg(1), TreeIdentifier(0))
}

/// `records` 条记录在一层 inode 树里占几片叶容器；装不下（片数超过一个码 2 根的扇出）交回核心层的拒绝。
pub(super) fn inode_leaf_containers_of_a_one_level_tree(
    records: u64,
) -> Result<u64, InodeTreeWriteRefusal> {
    inode_records_written_into_an_empty_tree(records)
        .map(|after| u64::try_from(after.containers.len()).expect("片数装得进 u64"))
}

/// 一层 inode 树至多装几条记录：一片叶容器的条数（核心层写一条进空树之后那一片的条数加它的空位）乘一个码 2 根的扇出。
pub(super) fn inode_records_one_level_tree_holds() -> u64 {
    let after = inode_records_written_into_an_empty_tree(1).expect("一条记录装得进空树");
    let container = &after.containers[0];
    let records_of_a_container = u64::try_from(container.records.len()).expect("条数装得进 u64")
        + container.free_record_slots();
    records_of_a_container
        * u64::try_from(leaf_containers_one_root_node_holds()).expect("扇出装得进 u64")
}

/// 这一版 inode 树里的记录条数。
pub(super) fn inode_records_of_the_version(version: &TransactionOutput) -> u64 {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| u64::try_from(container.contents.records.len()).expect("条数装得进 u64"))
        .sum()
}

/// 补齐那一步要写的单元数 a（0 ≤ a ≤ 31）：下一个用户数据落点 `next_data_slot` 离它所在那一段的下一个段边界还差几个数据单元
/// （单元区起点在段边界上；用户数据落点是偶数槽、一个数据单元占两槽）。落点正在段边界上交 0。
pub(super) const fn alignment_units_to_the_next_segment_boundary(
    next_data_slot: u64,
    unit_area_start_slot: u64,
) -> u64 {
    let slots_into_the_segment =
        (next_data_slot - unit_area_start_slot) % LOCAL_CLUSTER_SEGMENT_SLOTS;
    let slots_to_the_boundary =
        (LOCAL_CLUSTER_SEGMENT_SLOTS - slots_into_the_segment) % LOCAL_CLUSTER_SEGMENT_SLOTS;
    slots_to_the_boundary / (LOCAL_DATA_UNIT_BYTES / LOCAL_SLOT_BYTES)
}

/// 造数据挂载各阶段登记的发布数（登记第十二节修订第 14 条「新造法每一档预计的发布次数」那张表的式子，停机条款第 8 条拿它核）：
/// 建 inode ⌈(P ÷ 16 + P ÷ 32 + 1) ÷ 4096⌉、顺序写 P ÷ 32、补齐 a > 0 时 1 次（a = 0 时 0 次）、交错 P ÷ 16。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RegisteredPublishesOfTheDataMount {
    pub(super) creating_the_inodes: u64,
    pub(super) sequential_half: u64,
    pub(super) alignment: u64,
    pub(super) interleaved_half: u64,
}

impl RegisteredPublishesOfTheDataMount {
    pub(super) fn of_the_pool(
        live_data_units: u64,
        alignment_spacer_units: u64,
    ) -> RegisteredPublishesOfTheDataMount {
        RegisteredPublishesOfTheDataMount {
            creating_the_inodes: inodes_created_by_the_data_mount(live_data_units)
                .div_ceil(INODES_CREATED_BY_ONE_PUBLISH),
            sequential_half: live_data_files(live_data_units) / 2,
            alignment: u64::from(alignment_spacer_units > 0),
            interleaved_half: live_data_files(live_data_units) / 2
                + interleaved_spacer_files(live_data_units),
        }
    }

    /// 造数据挂载四个阶段登记的发布数之和（修订第 14 条那张表的「数据挂载发布合计」）。
    pub(super) const fn total(self) -> u64 {
        self.creating_the_inodes + self.sequential_half + self.alignment + self.interleaved_half
    }

    /// 这一阶段登记的发布数；不是造数据挂载那四个阶段的交 `None`（登记里没有它们的算式）。
    pub(super) const fn publishes_of_the_phase(self, phase: ConstructionPhase) -> Option<u64> {
        match phase {
            ConstructionPhase::CreatingTheInodes => Some(self.creating_the_inodes),
            ConstructionPhase::SequentialHalf => Some(self.sequential_half),
            ConstructionPhase::AlignmentSpacer => Some(self.alignment),
            ConstructionPhase::InterleavedHalf => Some(self.interleaved_half),
            ConstructionPhase::MakeFilesystemAndTheFirstFile
            | ConstructionPhase::DataMount
            | ConstructionPhase::FragmentMount
            | ConstructionPhase::UnlinkingTheSpacerFiles
            | ConstructionPhase::ReleasingTheSpacerSlots
            | ConstructionPhase::ReadingTheGeometry => None,
        }
    }
}

/// 一个阶段发成的版数与登记的算式不等：（阶段，那一阶段发成的版数，登记的版数）。
pub(super) type PhasePublishesDifferingFromTheRegistration = (ConstructionPhase, u64, u64);

/// 按造的先后，头一个发成的版数与登记的算式不等的阶段；都相等（或不是有算式的那四个阶段）交 `None`。`publishes_of_each_phase` 是（阶段，那一阶段发成的版数）。
pub(super) fn phase_whose_publishes_differ_from_the_registration(
    publishes_of_each_phase: &[(ConstructionPhase, u64)],
    registered: RegisteredPublishesOfTheDataMount,
) -> Option<PhasePublishesDifferingFromTheRegistration> {
    publishes_of_each_phase
        .iter()
        .find_map(|&(phase, publishes)| {
            registered
                .publishes_of_the_phase(phase)
                .filter(|registered_publishes| *registered_publishes != publishes)
                .map(|registered_publishes| (phase, publishes, registered_publishes))
        })
}

/// 造池停下的那一刻（登记第十二节修订第 14 条 `name=construction_refused` 那一行的字段）：哪一阶段、那一阶段正在做第几步（做完的步数加一；
/// 阶段的发布数对不上时是那一阶段做完的步数）、造池到这一刻一共发成几版、这次挂载开过的聚簇段数与单元区的整段数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MomentOfTheConstruction {
    pub(super) phase: ConstructionPhase,
    pub(super) step: u64,
    pub(super) publishes: u64,
    pub(super) cluster_segments: ClusterSegmentsOfTheMount,
}

fn moment_of_the_construction(
    clock: &ConstructionClock<'_>,
    session: &MountedSession,
    publishes: u64,
) -> MomentOfTheConstruction {
    let (phase, step) = clock.phase_and_step_in_progress();
    MomentOfTheConstruction {
        phase,
        step,
        publishes,
        cluster_segments: cluster_segments_of_the_mount(session),
    }
}

/// 核心层拒了造池里一次发布时读到的：拒绝是 `UserChangeRefused` 的哪一个成员；推满也放不下那一形（`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）
/// 每块被判短的盘是哪一判判的、推抬 F 全放出来至多放得出几个字节，按核心层交回的盘的次序（别的成员两样都是空的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RefusalReadWhileConstructing {
    pub(super) name_of_the_refusal: &'static str,
    pub(super) judged_short_by: Vec<String>,
    pub(super) most_a_floor_raise_could_make_available: Vec<i128>,
}

/// 读拒绝的成员名与被判短的盘：不整个格式化拒绝（被拒之前发成的各版带在里面，格式化出来是整份版本）。
fn refusal_read_while_constructing(refusal: &UserChangeRefused) -> RefusalReadWhileConstructing {
    let name_of_the_refusal = match refusal {
        UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. } => {
            "DeviceTableOtherThanTheOneOfTheMount"
        }
        UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
            ..
        } => "PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration",
        UserChangeRefused::NoFileVersionToChange { .. } => "NoFileVersionToChange",
        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => {
            "DevicesWithoutASelfVerifiedSystemConfiguration"
        }
        UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. } => {
            "OwnDeviceNumbersDifferFromTheDeviceTable"
        }
        UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => {
            "DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot"
        }
        UserChangeRefused::Publish { .. } => "Publish",
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => "NoSpaceAfterRaisingTheFloor",
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_) => {
            "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease"
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {
            "FloorRaiseFailedWhilePushingForSpace"
        }
    };
    let devices_short: &[DeviceShortEvenWithEverythingAFloorRaiseCouldRelease] =
        if let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(short) = refusal
        {
            &short.devices_short
        } else {
            &[]
        };
    RefusalReadWhileConstructing {
        name_of_the_refusal,
        judged_short_by: devices_short
            .iter()
            .map(|device_short| format!("{:?}", device_short.judged_short_by))
            .collect(),
        most_a_floor_raise_could_make_available: devices_short
            .iter()
            .map(|device_short| device_short.most_a_floor_raise_could_make_available.0)
            .collect(),
    }
}

/// 造池里数据写类发布的形状、或一个阶段的发布数，与登记第十二节修订第 14 条读到的不符（停机条款第 8 条：装置与 `crates/` 对不上，两边都查）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PublishShapeDisagreement {
    /// 一次数据写类调用发成的版数（推的抬 F 之外）不是 1。
    VersionsOfADataWrite { inode: u64, versions_published: u64 },
    /// 一次数据写类调用写进的数据单元不全是这个文件的、或个数不等于这个文件的单元数（`units_written` 是这次调用写进的全部数据单元，各 inode 加起来）。
    UnitsOfADataWrite {
        inode: u64,
        units_written: u64,
        units_of_the_file: u64,
    },
    /// 一个阶段发成的版数与登记的算式不等。
    PublishesOfThePhase {
        publishes: u64,
        registered_publishes: u64,
    },
}

/// 造池停在哪（登记第十二节修订第 14 条追加的停机条款；停机时产物照打、退 3）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ConstructionStopped {
    /// 第 7 条前一半：造池里一次发布被核心层拒（打 `name=construction_refused`）。装箱：不装箱这个成员就大过 clippy `result_large_err` 的 128 字节。
    PublishRefused {
        moment: MomentOfTheConstruction,
        refusal: Box<RefusalReadWhileConstructing>,
    },
    /// 第 8 条。
    PublishShapeDiffersFromTheRegistration {
        moment: MomentOfTheConstruction,
        disagreement: PublishShapeDisagreement,
    },
}

fn stop_of_the_refusal(
    clock: &ConstructionClock<'_>,
    session: &MountedSession,
    publishes: u64,
    refusal: &UserChangeRefused,
) -> ConstructionStopped {
    ConstructionStopped::PublishRefused {
        moment: moment_of_the_construction(clock, session, publishes),
        refusal: Box::new(refusal_read_while_constructing(refusal)),
    }
}

/// 造池里一次写满的一个文件（登记第十二节修订第 14 条「新造法」：数据写类的发布一律一次 `WriteAtOffset`、偏移 0、字节是这个文件的全部单元）：
/// inode 号、单元数；文件里第 j 个单元的内容是第 `first_content_unit_index + j` 个单元第 `write_generation` 次写的
/// （[`fill_the_content_of_a_data_unit`]；活数据文件写代数 [`WRITE_GENERATION_OF_THE_DATA_MOUNT`]，垫片文件 [`WRITE_GENERATION_OF_THE_SPACER_FILE`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WholeFileOfTheConstruction {
    pub(super) inode: u64,
    pub(super) units: u64,
    pub(super) first_content_unit_index: u64,
    pub(super) write_generation: u64,
}

/// 第 `live_data_file` 个活数据文件（从 `lowest_live_data_inode` 数起）：K 个单元，装活数据单元第 `live_data_file × K` 起的那 K 个。
pub(super) const fn live_data_file_of_the_construction(
    lowest_live_data_inode: u64,
    live_data_file: u64,
) -> WholeFileOfTheConstruction {
    WholeFileOfTheConstruction {
        inode: lowest_live_data_inode + live_data_file,
        units: LIVE_DATA_UNITS_PER_FILE,
        first_content_unit_index: live_data_file * LIVE_DATA_UNITS_PER_FILE,
        write_generation: WRITE_GENERATION_OF_THE_DATA_MOUNT,
    }
}

/// 第 `interleaved_file` 个交错垫片文件：K 个单元。垫片单元按写的次序连着编内容下标，补齐文件那 a 个在前，这个文件装第 `a + interleaved_file × K` 起的那 K 个。
pub(super) const fn interleaved_spacer_file_of_the_construction(
    first_interleaved_spacer_inode: u64,
    interleaved_file: u64,
    alignment_spacer_units: u64,
) -> WholeFileOfTheConstruction {
    WholeFileOfTheConstruction {
        inode: first_interleaved_spacer_inode + interleaved_file,
        units: SPACER_UNITS_PER_FILE,
        first_content_unit_index: alignment_spacer_units + interleaved_file * SPACER_UNITS_PER_FILE,
        write_generation: WRITE_GENERATION_OF_THE_SPACER_FILE,
    }
}

/// 一次 `WriteAtOffset`、偏移 0 写满 `file`（一次发布）；核心层拒了交回它的拒绝。
fn write_the_whole_file(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    file: WholeFileOfTheConstruction,
    write_time_seconds: u64,
) -> Result<PublishedWhileConstructing, UserChangeRefused> {
    let unit_payload_bytes = data_unit_payload_capacity();
    let mut file_bytes = vec![
        0u8;
        usize::try_from(file.units).expect("一个文件的单元数装得进 usize")
            * unit_payload_bytes
    ];
    for (unit_in_file, unit_bytes) in (0_u64..).zip(file_bytes.chunks_mut(unit_payload_bytes)) {
        fill_the_content_of_a_data_unit(
            file.first_content_unit_index + unit_in_file,
            file.write_generation,
            unit_bytes,
        );
    }
    publish_while_constructing(
        session,
        devices,
        UserChange::WriteAtOffset {
            inode: InodeNumber(file.inode),
            offset: FileOffsetInBytes(0),
            bytes: PartialWriteBytes::new(&file_bytes).expect("造池写的文件至少一个单元"),
            write_time_seconds,
        },
    )
}

/// 一次数据写类调用交回的形状与登记的不符（停机条款第 8 条前两半）：发成的版数不是 1，或写进的数据单元不是恰好这个文件的全部单元。相符交 `None`。
fn shape_disagreement_of_a_data_write(
    file: WholeFileOfTheConstruction,
    published: &PublishedWhileConstructing,
) -> Option<PublishShapeDisagreement> {
    if published.versions_published != 1 {
        return Some(PublishShapeDisagreement::VersionsOfADataWrite {
            inode: file.inode,
            versions_published: published.versions_published,
        });
    }
    if published.data_units_written != BTreeMap::from([(file.inode, file.units)]) {
        return Some(PublishShapeDisagreement::UnitsOfADataWrite {
            inode: file.inode,
            units_written: published.data_units_written.values().sum(),
            units_of_the_file: file.units,
        });
    }
    None
}

/// 造池里发一次不是数据写类的改动（建 inode、删文件）；核心层拒了交回停在这一刻的读数（停机条款第 7 条）。
pub(super) fn publish_or_the_stop(
    clock: &ConstructionClock<'_>,
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    publishes: u64,
    change: UserChange<'_>,
) -> Result<PublishedWhileConstructing, ConstructionStopped> {
    publish_while_constructing(session, devices, change)
        .map_err(|refusal| stop_of_the_refusal(clock, session, publishes, &refusal))
}

/// 造池里一次写满 `file`（数据写类）：被拒交回停机条款第 7 条那一形，发成而形状与登记的不符交回第 8 条那一形。
pub(super) fn write_the_whole_file_or_the_stop(
    clock: &ConstructionClock<'_>,
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    publishes: u64,
    file: WholeFileOfTheConstruction,
    write_time_seconds: u64,
) -> Result<PublishedWhileConstructing, ConstructionStopped> {
    let published = write_the_whole_file(session, devices, file, write_time_seconds)
        .map_err(|refusal| stop_of_the_refusal(clock, session, publishes, &refusal))?;
    match shape_disagreement_of_a_data_write(file, &published) {
        Some(disagreement) => Err(
            ConstructionStopped::PublishShapeDiffersFromTheRegistration {
                moment: moment_of_the_construction(
                    clock,
                    session,
                    publishes + published.versions_published,
                ),
                disagreement,
            },
        ),
        None => Ok(published),
    }
}

/// 碎片段（登记第十二节修订第 14 条「几何核」：交错那一半里交错垫片文件的单元所在的段，去重；补齐文件的段不算）的起点。
/// `spacer_slots` 是垫片文件每个单元的（inode 号，文件里的单元序号）→ 槽；交错垫片文件是号不小于 `first_interleaved_spacer_inode` 的那些。
pub(super) fn fragment_segment_starts_of_the_interleaved_spacers(
    spacer_slots: &BTreeMap<(u64, u64), u64>,
    first_interleaved_spacer_inode: u64,
    unit_area_start_slot: u64,
) -> BTreeSet<u64> {
    spacer_slots
        .iter()
        .filter(|&(&(inode, _unit_in_file), _slot)| inode >= first_interleaved_spacer_inode)
        .map(|(_key, slot)| segment_start_of(*slot, unit_area_start_slot))
        .collect()
}

/// 碎片段里活槽怎么排（登记第十二节修订第 14 条「几何核」追加的读数，只报、不进门槛）：每段活槽的极大连续片数 → 段数，与活槽恰好是段的前半
/// （前 32 槽）的段数。一次发布写满一个文件、活数据文件先于它那个交错垫片文件写，碎片段是前半一整片活、后半空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LiveSlotLayoutOfTheFragmentSegments {
    pub(super) live_slot_runs_histogram: BTreeMap<u64, u64>,
    pub(super) segments_live_in_exactly_the_front_half: u64,
}

/// 盘 0 的记录上读碎片段（起点 `fragment_segment_starts`）里活槽（未释放记录罩住的槽）怎么排。
pub(super) fn live_slot_layout_of_the_fragment_segments(
    records: &[AllocationRecord],
    fragment_segment_starts: &BTreeSet<u64>,
) -> LiveSlotLayoutOfTheFragmentSegments {
    let live_slots = slots_under_an_unreleased_record_on_device_zero(records);
    let half_segment_slots =
        usize::try_from(LOCAL_CLUSTER_SEGMENT_SLOTS / 2).expect("半段的槽数装得进 usize");
    let mut live_slot_runs_histogram: BTreeMap<u64, u64> = BTreeMap::new();
    let mut segments_live_in_exactly_the_front_half = 0_u64;
    for &segment_start in fragment_segment_starts {
        let is_each_slot_live: Vec<bool> = (segment_start
            ..segment_start + LOCAL_CLUSTER_SEGMENT_SLOTS)
            .map(|slot| live_slots.contains(&slot))
            .collect();
        let runs = is_each_slot_live
            .iter()
            .zip(std::iter::once(&false).chain(is_each_slot_live.iter()))
            .filter(|&(&is_this_slot_live, &is_the_slot_before_live)| {
                is_this_slot_live && !is_the_slot_before_live
            })
            .count();
        *live_slot_runs_histogram
            .entry(u64::try_from(runs).expect("片数装得进 u64"))
            .or_insert(0) += 1;
        if is_each_slot_live[..half_segment_slots]
            .iter()
            .all(|&is_live| is_live)
            && !is_each_slot_live[half_segment_slots..]
                .iter()
                .any(|&is_live| is_live)
        {
            segments_live_in_exactly_the_front_half += 1;
        }
    }
    LiveSlotLayoutOfTheFragmentSegments {
        live_slot_runs_histogram,
        segments_live_in_exactly_the_front_half,
    }
}

#[cfg(test)]
mod tests;
