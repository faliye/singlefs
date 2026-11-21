use super::*;

/// 登记第十二节修订第 14 条「新造法每一档预计的发布次数」那张表，三档逐格钉绝对值：P 是 36608、65536、131072；
/// 建 inode ⌈(P ÷ 16 + P ÷ 32 + 1) ÷ 4096⌉ = 1、2、4，顺序写 P ÷ 32 = 1144、2048、4096，补齐至多 1，交错 P ÷ 16 = 2288、4096、8192，
/// a > 0 时合计 3434、6147、12293，a = 0 时少补齐那 1 次。不是造数据挂载那四个阶段的，登记里没有算式。
#[test]
fn the_registered_publishes_of_every_tier_are_the_table_of_revision_fourteen() {
    for (tier, live_data_units, creating_the_inodes, sequential_half, interleaved_half, total) in [
        (
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
            36608,
            1,
            1144,
            2288,
            3434,
        ),
        (UnitAreaTier::FourGibibytes, 65536, 2, 2048, 4096, 6147),
        (UnitAreaTier::EightGibibytes, 131072, 4, 4096, 8192, 12293),
    ] {
        assert_eq!(tier.live_data_units(), live_data_units, "{}", tier.name());
        let with_alignment = RegisteredPublishesOfTheDataMount::of_the_pool(live_data_units, 30);
        assert_eq!(
            with_alignment,
            RegisteredPublishesOfTheDataMount {
                creating_the_inodes,
                sequential_half,
                alignment: 1,
                interleaved_half,
            },
            "{}",
            tier.name()
        );
        assert_eq!(with_alignment.total(), total, "{}", tier.name());
        let without_alignment = RegisteredPublishesOfTheDataMount::of_the_pool(live_data_units, 0);
        assert_eq!(without_alignment.alignment, 0, "{}", tier.name());
        assert_eq!(without_alignment.total() + 1, total, "{}", tier.name());
        for phase in [
            ConstructionPhase::MakeFilesystemAndTheFirstFile,
            ConstructionPhase::DataMount,
            ConstructionPhase::FragmentMount,
            ConstructionPhase::UnlinkingTheSpacerFiles,
            ConstructionPhase::ReleasingTheSpacerSlots,
            ConstructionPhase::ReadingTheGeometry,
        ] {
            assert_eq!(
                with_alignment.publishes_of_the_phase(phase),
                None,
                "{}",
                phase.name()
            );
        }
        assert_eq!(
            with_alignment.publishes_of_the_phase(ConstructionPhase::SequentialHalf),
            Some(sequential_half)
        );
    }
}

/// 补齐要写的单元数 a：下一个用户数据落点离下一个段边界还差几个数据单元（两槽一个），落点正在段边界上是 0。
#[test]
fn the_alignment_fills_the_units_up_to_the_next_segment_boundary() {
    let segment_start = |segment_index: u64| {
        LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
    };
    for (next_data_slot, alignment_units) in [
        (segment_start(0), 0),
        (segment_start(0) + 4, 30),
        (segment_start(0) + 62, 1),
        (segment_start(1), 0),
        (segment_start(5) + 2, 31),
    ] {
        assert_eq!(
            alignment_units_to_the_next_segment_boundary(
                next_data_slot,
                LOCAL_UNIT_AREA_START_SLOT
            ),
            alignment_units,
            "下一个落点 {next_data_slot}"
        );
    }
}

/// 一次数据写类调用的形状（停机条款第 8 条前两半）：发成一版、写进的恰好是这个文件的 16 个单元才相符；发成两版、少写一个单元、
/// 另写进别的 inode 的单元都不相符，各报各的成员与数。
#[test]
fn a_data_write_other_than_one_publish_of_the_whole_file_differs_from_the_registration() {
    let file = WholeFileOfTheConstruction {
        inode: 9,
        units: LIVE_DATA_UNITS_PER_FILE,
        first_content_unit_index: 0,
        write_generation: WRITE_GENERATION_OF_THE_DATA_MOUNT,
    };
    let published =
        |versions_published: u64, data_units_written: &[(u64, u64)]| PublishedWhileConstructing {
            versions_published,
            floor_raises: 0,
            data_units_written: data_units_written.iter().copied().collect(),
        };
    assert_eq!(
        shape_disagreement_of_a_data_write(file, &published(1, &[(9, 16)])),
        None
    );
    assert_eq!(
        shape_disagreement_of_a_data_write(file, &published(2, &[(9, 16)])),
        Some(PublishShapeDisagreement::VersionsOfADataWrite {
            inode: 9,
            versions_published: 2,
        })
    );
    assert_eq!(
        shape_disagreement_of_a_data_write(file, &published(1, &[(9, 15)])),
        Some(PublishShapeDisagreement::UnitsOfADataWrite {
            inode: 9,
            units_written: 15,
            units_of_the_file: 16,
        })
    );
    assert_eq!(
        shape_disagreement_of_a_data_write(file, &published(1, &[(9, 16), (10, 1)])),
        Some(PublishShapeDisagreement::UnitsOfADataWrite {
            inode: 9,
            units_written: 17,
            units_of_the_file: 16,
        })
    );
}

/// 阶段的发布数（停机条款第 8 条后一半）：P = 64、a = 30 登记的是建 inode 1、顺序写 2、补齐 1、交错 4；四个阶段都相等交 `None`，
/// 按造的先后报头一个不等的阶段与两个数；不是那四个阶段的发布数多少都不判。
#[test]
fn the_first_phase_whose_publishes_differ_from_the_formula_is_reported() {
    let registered = RegisteredPublishesOfTheDataMount::of_the_pool(64, 30);
    let matching = [
        (ConstructionPhase::MakeFilesystemAndTheFirstFile, 0),
        (ConstructionPhase::DataMount, 0),
        (ConstructionPhase::CreatingTheInodes, 1),
        (ConstructionPhase::SequentialHalf, 2),
        (ConstructionPhase::AlignmentSpacer, 1),
        (ConstructionPhase::InterleavedHalf, 4),
        (ConstructionPhase::UnlinkingTheSpacerFiles, 6),
    ];
    assert_eq!(
        phase_whose_publishes_differ_from_the_registration(&matching, registered),
        None
    );
    let mut unit_by_unit = matching;
    unit_by_unit[3].1 = 32;
    unit_by_unit[4].1 = 30;
    assert_eq!(
        phase_whose_publishes_differ_from_the_registration(&unit_by_unit, registered),
        Some((ConstructionPhase::SequentialHalf, 32, 2))
    );
    let mut alignment_unit_by_unit = matching;
    alignment_unit_by_unit[4].1 = 30;
    assert_eq!(
        phase_whose_publishes_differ_from_the_registration(&alignment_unit_by_unit, registered),
        Some((ConstructionPhase::AlignmentSpacer, 30, 1))
    );
}

/// 碎片段只算交错垫片文件的单元所在的段：补齐文件（号小于头一个交错垫片文件）的单元落在别的段，那一段不进碎片段。
#[test]
fn the_fragment_segments_are_the_segments_of_the_interleaved_spacer_files_only() {
    let segment_start = |segment_index: u64| {
        LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
    };
    let alignment_file_inode = 100;
    let first_interleaved_spacer_inode = alignment_file_inode + ALIGNMENT_FILES;
    let mut spacer_slots: BTreeMap<(u64, u64), u64> = BTreeMap::new();
    for unit_in_file in 0..30 {
        spacer_slots.insert(
            (alignment_file_inode, unit_in_file),
            segment_start(0) + 4 + 2 * unit_in_file,
        );
    }
    for (interleaved_file, segment_index) in [(0, 1), (1, 2)] {
        for unit_in_file in 0..SPACER_UNITS_PER_FILE {
            spacer_slots.insert(
                (
                    first_interleaved_spacer_inode + interleaved_file,
                    unit_in_file,
                ),
                segment_start(segment_index) + 32 + 2 * unit_in_file,
            );
        }
    }
    assert_eq!(
        fragment_segment_starts_of_the_interleaved_spacers(
            &spacer_slots,
            first_interleaved_spacer_inode,
            LOCAL_UNIT_AREA_START_SLOT
        ),
        BTreeSet::from([segment_start(1), segment_start(2)])
    );
}

/// 碎片段里活槽的排法：前 32 槽一整片活、后 32 槽空的段（已释放的记录不算活）算一片、算「恰好前半」；活槽在后半的算一片、不算前半；
/// 每隔一个单元一个活的段是 8 片。
#[test]
fn the_live_slot_layout_counts_the_runs_and_the_segments_live_in_exactly_the_front_half() {
    let segment_start = |segment_index: u64| {
        LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
    };
    let record_at = |slot: u64, is_released: bool| AllocationRecord {
        device: DeviceIdentity(0),
        slot: SlotNumber(slot),
        span_slots: 2,
        generation: singlefs_core::address::CheckpointTxg(7),
        is_released,
        quarantine_mark: singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
    };
    let mut records = data_unit_records_in_the_segment(0, 16);
    records
        .extend((0..16).map(|unit_index| record_at(segment_start(0) + 32 + 2 * unit_index, true)));
    records
        .extend((0..16).map(|unit_index| record_at(segment_start(1) + 32 + 2 * unit_index, false)));
    records.extend((0..8).map(|unit_index| record_at(segment_start(2) + 4 * unit_index, false)));
    let layout = live_slot_layout_of_the_fragment_segments(
        &records,
        &BTreeSet::from([segment_start(0), segment_start(1), segment_start(2)]),
    );
    assert_eq!(
        layout,
        LiveSlotLayoutOfTheFragmentSegments {
            live_slot_runs_histogram: BTreeMap::from([(1, 2), (8, 1)]),
            segments_live_in_exactly_the_front_half: 1,
        }
    );
}

/// 推满也放不下那一形（`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）读出每块被判短的盘是哪一判判的、推抬 F 至多放得出几个字节，按核心层交回的盘的次序；
/// 拒绝的成员名照成员写。合成的拒绝：盘 0 被槽对那一判判短（聚簇段把空间吃光那一形的拒因）、盘 1 被式子那一判判短。
#[test]
fn the_refusal_reads_the_devices_judged_short_even_with_everything_a_floor_raise_could_release() {
    let device_short =
        |device: u32,
         judged_short_by: singlefs_core::admission::JudgementThatFoundTheDeviceShort,
         available: i128| {
            DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
                device: DeviceIdentity(device),
                judged_short_by,
                most_a_floor_raise_could_make_available:
                    singlefs_core::admission::AvailableBytesOnOneDevice(available),
                demand: singlefs_core::admission::BytesOnOneDevice(32768),
            }
        };
    let refusal = UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(Box::new(
        singlefs_core::mounted_session::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
            devices_short: vec![
                device_short(
                    0,
                    singlefs_core::admission::JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
                    -65536,
                ),
                device_short(
                    1,
                    singlefs_core::admission::JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                    0,
                ),
            ],
            empty_cluster_segments_below_the_floor: None,
            last_refusal: singlefs_core::transaction::PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish: 1,
                record_limit: 0,
            },
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        },
    ));
    assert_eq!(
        refusal_read_while_constructing(&refusal),
        RefusalReadWhileConstructing {
            name_of_the_refusal: "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease",
            judged_short_by: vec![
                "SlotPairsOutsideTheClusterSegmentsForDataUnits".to_string(),
                "FormulaAvailableAgainstDemand".to_string(),
            ],
            most_a_floor_raise_could_make_available: vec![-65536, 0],
        }
    );
}

/// 核心层拒了造池里一次数据写类发布（写一个不在 inode 树里的号，`WriteToAnInodeThatIsNotInTheTree`）：交回停机条款第 7 条那一形，
/// 带正在做的阶段与步、调用方给的发布数、这次挂载此刻的聚簇段数与拒绝的成员名，打得出 `name=construction_refused` 那一行；不 panic。
#[test]
fn a_data_write_the_core_refuses_while_constructing_stops_with_the_moment_of_the_refusal() {
    let mut devices = new_pool_with_the_first_file(
        smallest_device_bytes_the_make_filesystem_accepts(),
        ChunkStorage::SharedWhenTheBytesAreEqual,
    );
    let mounted =
        mount_writable(&make_filesystem_parameters_of_the_pool(), &mut devices).expect("可写挂载");
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    let mut progress_dropped = |_line: &str| {};
    let mut clock = ConstructionClock::new(
        "拒绝".to_string(),
        CONSTRUCTION_PROGRESS_INTERVAL,
        &mut progress_dropped,
    );
    clock.begin(ConstructionPhase::SequentialHalf, 1, 0);
    let inode_not_in_the_tree = version_of_trees(&session).inode_number_watermark() + 1000;
    let stopped = write_the_whole_file_or_the_stop(
        &clock,
        &mut session,
        &mut devices,
        5,
        live_data_file_of_the_construction(inode_not_in_the_tree, 0),
        FIXED_WRITE_TIME_SECONDS + 1,
    );
    let (moment, refusal) = match &stopped {
        Err(ConstructionStopped::PublishRefused { moment, refusal }) => (*moment, refusal.clone()),
        other => panic!("写不在树里的号要交回被拒那一形：{other:?}"),
    };
    assert_eq!(
        moment,
        MomentOfTheConstruction {
            phase: ConstructionPhase::SequentialHalf,
            step: 1,
            publishes: 5,
            cluster_segments: cluster_segments_of_the_mount(&session),
        }
    );
    assert_eq!(refusal.name_of_the_refusal, "Publish");
    assert!(refusal.judged_short_by.is_empty());
    let lines = construction_stop_lines(
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
        64,
        &ConstructionStopped::PublishRefused { moment, refusal },
    );
    assert_eq!(lines[0].0, "construction_refused");
}

/// 三档造池要的 inode 记录都装得进一层 inode 树（登记第十二节修订第 14 条的造法；树还不会长高）：一层装几条、每档要几条、
/// 占几片叶容器，都照核心层写记录那一步现算（`inode_tree::write_records_into_leaf_containers` 与 `leaf_containers_one_root_node_holds`），
/// 再拿另外手算的绝对值钉住：一片 233 条 × 扇出 135 = 31455 条；活数据文件 P ÷ 16，垫片文件 P ÷ 32 + 1（交错垫片文件加补齐文件），
/// 加造数据挂载之前那 1 条与造碎片挂载推发布的至多 16 个空 inode，片数 ⌈条数 ÷ 233⌉。
/// 判别力：P 个一单元文件（K = 1）三档都装不下，核心层拒成「叶容器多于一个码 2 根装得下的」，同三档 `stage-three-cost` 在建池那一步撞上的拒绝。
#[test]
fn the_construction_of_every_tier_fits_a_one_level_inode_tree() {
    let one_level_tree_holds = inode_records_one_level_tree_holds();
    assert_eq!(one_level_tree_holds, 233 * 135);
    let containers_one_root_node_holds =
        u64::try_from(leaf_containers_one_root_node_holds()).expect("扇出装得进 u64");
    for (tier, files, spacer_files, containers) in [
        (
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
            2288,
            1145,
            15,
        ),
        (UnitAreaTier::FourGibibytes, 4096, 2049, 27),
        (UnitAreaTier::EightGibibytes, 8192, 4097, 53),
    ] {
        let live_data_units = tier.live_data_units();
        assert_eq!(live_data_files(live_data_units), files, "{}", tier.name());
        assert_eq!(
            spacer_files_of_the_construction(live_data_units),
            spacer_files,
            "{}",
            tier.name()
        );
        assert_eq!(
            inodes_created_by_the_data_mount(live_data_units),
            files + spacer_files,
            "{}",
            tier.name()
        );
        let records = inode_records_of_the_construction_at_most(
            live_data_units,
            INODE_RECORDS_BEFORE_THE_DATA_MOUNT,
        );
        assert_eq!(records, 1 + files + spacer_files + 16, "{}", tier.name());
        assert!(records <= one_level_tree_holds, "{}", tier.name());
        assert_eq!(
            inode_leaf_containers_of_a_one_level_tree(records),
            Ok(containers),
            "{}",
            tier.name()
        );
        assert!(
            containers <= containers_one_root_node_holds,
            "{}",
            tier.name()
        );
    }
    for tier in UnitAreaTier::EVERY_TIER {
        let records_of_one_unit_files =
            INODE_RECORDS_BEFORE_THE_DATA_MOUNT + tier.live_data_units() + 1;
        assert!(
            records_of_one_unit_files > one_level_tree_holds,
            "{}",
            tier.name()
        );
        assert!(
            matches!(
                inode_leaf_containers_of_a_one_level_tree(records_of_one_unit_files),
                Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { .. })
            ),
            "{}",
            tier.name()
        );
    }
}
