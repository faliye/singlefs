//! 造起步镜像：照登记的造法在内存盘上造池，按阶段计时。

use super::*;

/// 造一档的起步镜像（登记第五节「负载」第 1 步原样，第 2、3 步照第十二节修订第 14 条「新造法，逐步定义」，第 4 步几何核照原文）。
/// 记 K = [`LIVE_DATA_UNITS_PER_FILE`] = 16；数据写类的发布一律是一次 `WriteAtOffset`、偏移 0、字节是这个文件的全部单元
/// （[`write_the_whole_file`]），每次调用交回的发成版数必须恰好 1、写进的单元恰好是这个文件的全部单元（停机条款第 8 条）：
/// 1. 新池新建文件（两块内存盘、`parameters()`、换 fsid、取号暖机）；
/// 2. 造数据挂载（一次可写挂载，交错那一半写完才丢会话）：
///    - 建 inode：本挂载要用的 inode 一次建齐，每次发布至多 [`INODES_CREATED_BY_ONE_PUBLISH`] 个，号从水位起连着发：先 P ÷ 16 个活数据文件，
///      再 1 个补齐文件，再 P ÷ 32 个交错垫片文件；
///    - 顺序写那一半：活数据文件 P ÷ 32 到 P ÷ 16 − 1，按文件号从小到大，每个文件一次发布写满 16 个单元；
///    - 补齐：读下一个用户数据落点，算它离下一个段边界还差 a 个单元（[`alignment_units_to_the_next_segment_boundary`]）；a > 0 用一次发布给补齐文件
///      写满 a 个单元，a = 0 不发布、补齐文件留空；
///    - 交错那一半：文件序号 f 从 0 到 P ÷ 32 − 1，先一次发布写满活数据文件 f，再一次发布写满第 f 个交错垫片文件；一个活数据文件占段的前 32 槽、
///      它那个垫片文件占同段的后 32 槽，一个碎片段就是这一对；
///    - 四个阶段的发布数与登记的算式（[`RegisteredPublishesOfTheDataMount`]）不等的，停（停机条款第 8 条）；
/// 3. 造碎片挂载：重挂，删全部垫片文件（P ÷ 32 + 1 个，含补齐文件，空的也删），抬 F 到准入上限直到垫片的旧落点按记录都已释放
///    （迭代上界 [`RELEASE_ROUNDS_AT_MOST`]，每轮没放完再发一次建一个 inode 的改动推一次发布），会话丢掉；这一步不写用户数据；
/// 4. 几何核：碎片段（[`fragment_segment_starts_of_the_interleaved_spacers`]）活槽恰好 32 的占比，强制选段后备点名的那一段，greedy 照核心层重放要选的那一段只报；
///    另读碎片段里活槽怎么排（[`live_slot_layout_of_the_fragment_segments`]）。
///
/// 造池里任一发布被核心层拒交回 [`ConstructionStopped::PublishRefused`]（停机条款第 7 条），形状对不上交回
/// [`ConstructionStopped::PublishShapeDiffersFromTheRegistration`]；几何没造成照样交回镜像，由调用方照 [`GeometryReading::is_formed`] 停。
///
/// 不用「按偏移覆盖写一个大文件里下标为奇数的单元」：核心层按偏移写把偏移所在的单元到文件末尾一起重写（R4），覆盖写第 u 个单元
/// 要重写 P − u 个，段里留不下一半活槽；前台用户改动同理只覆盖写一个活数据文件的最后一个单元（[`publish_the_user_change`]）。
/// 不用 P 个一单元文件：三档的 P 都超过一层 inode 树装得下的文件数（[`inode_records_one_level_tree_holds`]），建池那一步核心层拒；
/// 开造之前先照核心层算一遍这一档至多要几条 inode 记录（[`inode_records_of_the_construction_at_most`]）装不装得下。
///
/// 关掉（不卸载）交回两块内存盘；量挂载那一侧再可写挂载，这样造出来的段不是那次挂载开过的段（`choose_region` 的资格）。
pub(super) fn construct_the_starting_image(
    tier: UnitAreaTier,
) -> Result<StartingImage, ConstructionStopped> {
    construct_a_starting_image_of_the_size(tier, tier.device_bytes(), tier.live_data_units())
}

/// 同 [`construct_the_starting_image`]，盘大小与活数据单元数由调用方给（同包测试在小池上走一遍整条路）。块照共用的存法存，
/// 每 [`CONSTRUCTION_PROGRESS_INTERVAL`] 往标准错误打一行进度。
pub(super) fn construct_a_starting_image_of_the_size(
    tier: UnitAreaTier,
    device_bytes: u64,
    live_data_units: u64,
) -> Result<StartingImage, ConstructionStopped> {
    let mut progress_to_the_standard_error = |line: &str| eprintln!("{line}");
    let mut clock = ConstructionClock::new(
        format!("{}-P{live_data_units}", tier.name()),
        CONSTRUCTION_PROGRESS_INTERVAL,
        &mut progress_to_the_standard_error,
    );
    construct_a_starting_image_on_the_clock(
        tier,
        device_bytes,
        live_data_units,
        ChunkStorage::SharedWhenTheBytesAreEqual,
        &mut clock,
    )
}

/// 同 [`construct_a_starting_image_of_the_size`]，块照 `storage` 存，分阶段计时与进度行走 `clock`（各阶段的读数取走放进
/// [`ConstructionReport::phases`]）。两种存法发的改动、读写的字节逐个相同。
pub(super) fn construct_a_starting_image_on_the_clock(
    tier: UnitAreaTier,
    device_bytes: u64,
    live_data_units: u64,
    storage: ChunkStorage,
    clock: &mut ConstructionClock<'_>,
) -> Result<StartingImage, ConstructionStopped> {
    assert_eq!(
        live_data_units % data_units_of_a_cluster_segment(),
        0,
        "P 是 32 的倍数：交错那一半一个活数据文件配一个等长的垫片文件，正好写满整段"
    );
    assert_eq!(
        (live_data_units / 2) % LIVE_DATA_UNITS_PER_FILE,
        0,
        "P ÷ 2 是 K 的倍数：顺序写那一半与交错那一半各自从文件边界起"
    );
    let started = Instant::now();
    let parameters = make_filesystem_parameters_of_the_pool();
    clock.begin(ConstructionPhase::MakeFilesystemAndTheFirstFile, 1, 0);
    let mut devices = new_pool_with_the_first_file(device_bytes, storage);
    clock.step_done();
    clock.finish(&devices, None, 0, None);
    let mut publishes = 0_u64;
    let mut floor_raises = 0_u64;
    let mut data_writes_by_units: BTreeMap<u64, u64> = BTreeMap::new();
    let mut write_time_seconds = FIXED_WRITE_TIME_SECONDS;
    let mut next_write_time = || {
        write_time_seconds += 1;
        write_time_seconds
    };
    let live_data_files_of_the_pool = live_data_files(live_data_units);
    let interleaved_spacer_files_of_the_pool = interleaved_spacer_files(live_data_units);
    let spacer_files = spacer_files_of_the_construction(live_data_units);
    let lowest_live_data_inode: u64;
    let alignment_file_inode: u64;
    let first_interleaved_spacer_inode: u64;
    let alignment_spacer_units: u64;
    let registered_publishes: RegisteredPublishesOfTheDataMount;
    let spacer_slots: BTreeMap<(u64, u64), u64>;
    let inode_records_before_the_data_mount: u64;
    let inode_records_at_the_peak: u64;
    let inode_leaf_containers_at_the_peak: u64;
    {
        clock.begin(ConstructionPhase::DataMount, 1, publishes);
        let mounted = mount_writable(&parameters, &mut devices).expect("造数据挂载那一次可写挂载");
        let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
        clock.step_done();
        clock.finish(
            &devices,
            Some(version_of_trees(&session)),
            publishes,
            Some(cluster_segments_of_the_mount(&session)),
        );
        lowest_live_data_inode = version_of_trees(&session).inode_number_watermark();
        inode_records_before_the_data_mount =
            inode_records_of_the_version(version_of_trees(&session));
        let inode_records_at_most = inode_records_of_the_construction_at_most(
            live_data_units,
            inode_records_before_the_data_mount,
        );
        if let Err(refusal) = inode_leaf_containers_of_a_one_level_tree(inode_records_at_most) {
            panic!(
                "造池至多要 {inode_records_at_most} 条 inode 记录，一层 inode 树装不下（至多 {} 条）：{refusal:?}",
                inode_records_one_level_tree_holds()
            );
        }
        let inodes_to_create = inodes_created_by_the_data_mount(live_data_units);
        let mut inodes_created = 0_u64;
        clock.begin(
            ConstructionPhase::CreatingTheInodes,
            inodes_to_create.div_ceil(INODES_CREATED_BY_ONE_PUBLISH),
            publishes,
        );
        // 迭代上界：每次至少建一个，至多 ⌈(P ÷ 16 + P ÷ 32 + 1) ÷ 4096⌉ 次。
        while inodes_created < inodes_to_create {
            let count = (inodes_to_create - inodes_created).min(INODES_CREATED_BY_ONE_PUBLISH);
            let published = publish_or_the_stop(
                clock,
                &mut session,
                &mut devices,
                publishes,
                UserChange::NewInodes {
                    count,
                    write_time_seconds: next_write_time(),
                },
            )?;
            publishes += published.versions_published;
            floor_raises += published.floor_raises;
            inodes_created += count;
            clock.step_done();
        }
        clock.finish(
            &devices,
            Some(version_of_trees(&session)),
            publishes,
            Some(cluster_segments_of_the_mount(&session)),
        );
        assert_eq!(
            version_of_trees(&session).inode_number_watermark(),
            lowest_live_data_inode + inodes_to_create,
            "建的 inode 号从水位起连着发"
        );
        alignment_file_inode = lowest_live_data_inode + live_data_files_of_the_pool;
        first_interleaved_spacer_inode = alignment_file_inode + ALIGNMENT_FILES;
        clock.begin(
            ConstructionPhase::SequentialHalf,
            live_data_files_of_the_pool / 2,
            publishes,
        );
        for live_data_file in live_data_files_of_the_pool / 2..live_data_files_of_the_pool {
            let published = write_the_whole_file_or_the_stop(
                clock,
                &mut session,
                &mut devices,
                publishes,
                live_data_file_of_the_construction(lowest_live_data_inode, live_data_file),
                next_write_time(),
            )?;
            publishes += published.versions_published;
            floor_raises += published.floor_raises;
            *data_writes_by_units
                .entry(published.data_units_written.values().sum::<u64>())
                .or_insert(0) += 1;
            clock.step_done();
        }
        clock.finish(
            &devices,
            Some(version_of_trees(&session)),
            publishes,
            Some(cluster_segments_of_the_mount(&session)),
        );
        let next_data_slot = session.allocator.devices[0]
            .lowest_user_data_slot(session.allocator.cluster_segments())
            .expect("单元区装得下下一个用户数据单元");
        alignment_spacer_units = alignment_units_to_the_next_segment_boundary(
            next_data_slot.0,
            unit_area_of_device_zero(&session).start.slot().0,
        );
        clock.begin(ConstructionPhase::AlignmentSpacer, 1, publishes);
        if alignment_spacer_units > 0 {
            let published = write_the_whole_file_or_the_stop(
                clock,
                &mut session,
                &mut devices,
                publishes,
                WholeFileOfTheConstruction {
                    inode: alignment_file_inode,
                    units: alignment_spacer_units,
                    first_content_unit_index: 0,
                    write_generation: WRITE_GENERATION_OF_THE_SPACER_FILE,
                },
                next_write_time(),
            )?;
            publishes += published.versions_published;
            floor_raises += published.floor_raises;
            *data_writes_by_units
                .entry(published.data_units_written.values().sum::<u64>())
                .or_insert(0) += 1;
            clock.step_done();
        }
        clock.finish(
            &devices,
            Some(version_of_trees(&session)),
            publishes,
            Some(cluster_segments_of_the_mount(&session)),
        );
        clock.begin(
            ConstructionPhase::InterleavedHalf,
            live_data_files_of_the_pool / 2 + interleaved_spacer_files_of_the_pool,
            publishes,
        );
        // 迭代上界：P ÷ 32 个文件序号，每个两次发布。
        for interleaved_file in 0..interleaved_spacer_files_of_the_pool {
            for file in [
                live_data_file_of_the_construction(lowest_live_data_inode, interleaved_file),
                interleaved_spacer_file_of_the_construction(
                    first_interleaved_spacer_inode,
                    interleaved_file,
                    alignment_spacer_units,
                ),
            ] {
                let published = write_the_whole_file_or_the_stop(
                    clock,
                    &mut session,
                    &mut devices,
                    publishes,
                    file,
                    next_write_time(),
                )?;
                publishes += published.versions_published;
                floor_raises += published.floor_raises;
                *data_writes_by_units
                    .entry(published.data_units_written.values().sum::<u64>())
                    .or_insert(0) += 1;
                clock.step_done();
            }
        }
        clock.finish(
            &devices,
            Some(version_of_trees(&session)),
            publishes,
            Some(cluster_segments_of_the_mount(&session)),
        );
        registered_publishes =
            RegisteredPublishesOfTheDataMount::of_the_pool(live_data_units, alignment_spacer_units);
        let publishes_of_each_phase: Vec<(ConstructionPhase, u64)> = clock
            .timings
            .iter()
            .map(|timing| (timing.phase, timing.publishes))
            .collect();
        if let Some((phase, phase_publishes, registered_phase_publishes)) =
            phase_whose_publishes_differ_from_the_registration(
                &publishes_of_each_phase,
                registered_publishes,
            )
        {
            let steps_of_the_phase = clock
                .timings
                .iter()
                .find(|timing| timing.phase == phase)
                .map_or(0, |timing| timing.steps);
            return Err(
                ConstructionStopped::PublishShapeDiffersFromTheRegistration {
                    moment: MomentOfTheConstruction {
                        phase,
                        step: steps_of_the_phase,
                        publishes,
                        cluster_segments: cluster_segments_of_the_mount(&session),
                    },
                    disagreement: PublishShapeDisagreement::PublishesOfThePhase {
                        publishes: phase_publishes,
                        registered_publishes: registered_phase_publishes,
                    },
                },
            );
        }
        let version = version_of_trees(&session);
        spacer_slots = data_unit_slots_of_the_inodes(version, alignment_file_inode, spacer_files);
        inode_records_at_the_peak = inode_records_of_the_version(version);
        inode_leaf_containers_at_the_peak =
            u64::try_from(version.inode_leaf_containers.len()).expect("片数装得进 u64");
    }
    clock.begin(ConstructionPhase::FragmentMount, 1, publishes);
    let mounted = mount_writable(&parameters, &mut devices).expect("造碎片挂载那一次可写挂载");
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    clock.step_done();
    clock.finish(
        &devices,
        Some(version_of_trees(&session)),
        publishes,
        Some(cluster_segments_of_the_mount(&session)),
    );
    let written_segment_starts = fragment_segment_starts_of_the_interleaved_spacers(
        &spacer_slots,
        first_interleaved_spacer_inode,
        unit_area_of_device_zero(&session).start.slot().0,
    );
    clock.begin(
        ConstructionPhase::UnlinkingTheSpacerFiles,
        spacer_files,
        publishes,
    );
    for spacer_file in 0..spacer_files {
        let published = publish_or_the_stop(
            clock,
            &mut session,
            &mut devices,
            publishes,
            UserChange::Unlink {
                inode: InodeNumber(alignment_file_inode + spacer_file),
                write_time_seconds: next_write_time(),
            },
        )?;
        publishes += published.versions_published;
        floor_raises += published.floor_raises;
        clock.step_done();
    }
    clock.finish(
        &devices,
        Some(version_of_trees(&session)),
        publishes,
        Some(cluster_segments_of_the_mount(&session)),
    );
    let mut release_rounds = 0_u64;
    let mut is_every_spacer_slot_released = false;
    clock.begin(
        ConstructionPhase::ReleasingTheSpacerSlots,
        RELEASE_ROUNDS_AT_MOST,
        publishes,
    );
    while release_rounds < RELEASE_ROUNDS_AT_MOST {
        release_rounds += 1;
        raise_the_floor_to_the_admission_ceiling(&mut session, &mut devices);
        clock.step_done();
        let under_a_record = slots_under_an_unreleased_record_on_device_zero(
            &version_of_trees(&session).allocation_records,
        );
        is_every_spacer_slot_released = spacer_slots
            .values()
            .all(|slot| !under_a_record.contains(slot));
        if is_every_spacer_slot_released {
            break;
        }
        let published = publish_or_the_stop(
            clock,
            &mut session,
            &mut devices,
            publishes,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: next_write_time(),
            },
        )?;
        publishes += published.versions_published;
        floor_raises += published.floor_raises;
    }
    clock.finish(
        &devices,
        Some(version_of_trees(&session)),
        publishes,
        Some(cluster_segments_of_the_mount(&session)),
    );
    clock.begin(ConstructionPhase::ReadingTheGeometry, 1, publishes);
    let version = version_of_trees(&session);
    let data_slots_now: BTreeSet<u64> =
        data_unit_slots_of_the_inodes(version, lowest_live_data_inode, live_data_files_of_the_pool)
            .into_values()
            .collect();
    let unit_area = unit_area_of_device_zero(&session);
    // 量挂载是重挂，那一次挂载还没开过聚簇段、还没扫拒过哪一段。
    let greedy_segment_start_slot = segment_the_greedy_choice_picks(
        version,
        unit_area,
        &BTreeSet::new(),
        SegmentsRefusedByTheRegionScan::default(),
        session.compaction.quarantined_slots(),
        &devices,
    );
    let forced_segment = fragment_segment_to_force(
        &written_segment_starts,
        &BTreeSet::new(),
        unit_area.start.slot().0,
        &live_slots_per_cluster_segment_by_the_records(&version.allocation_records, unit_area),
        &segments_holding_a_record_other_than_a_data_unit(
            &version.allocation_records,
            unit_area,
            &data_slots_now,
        ),
    );
    let geometry = geometry_of_the_records(
        &version.allocation_records,
        unit_area,
        &written_segment_starts,
        &data_slots_now,
        greedy_segment_start_slot,
        forced_segment.and_then(|segment| segment.start_slot),
    );
    let fragment_live_slot_layout = live_slot_layout_of_the_fragment_segments(
        &version.allocation_records,
        &written_segment_starts,
    );
    clock.step_done();
    clock.finish(
        &devices,
        Some(version),
        publishes,
        Some(cluster_segments_of_the_mount(&session)),
    );
    let construction = ConstructionReport {
        construction_seconds: started.elapsed().as_secs_f64(),
        publishes,
        floor_raises,
        release_rounds,
        is_every_spacer_slot_released,
        spacer_units: alignment_spacer_units
            + interleaved_spacer_files_of_the_pool * SPACER_UNITS_PER_FILE,
        alignment_spacer_units,
        live_data_files: live_data_files_of_the_pool,
        spacer_files,
        inode_records_before_the_data_mount,
        inode_records_at_the_peak,
        inode_leaf_containers_at_the_peak,
        data_writes_by_units,
        registered_publishes_of_the_data_mount: registered_publishes,
        live_data_units_found: u64::try_from(data_slots_now.len()).expect("单元数"),
        empty_segments: session.allocator.devices[0].empty_segments(),
        mapped_units: u64::try_from(version.mapped_units.len()).expect("映射单元条数"),
        geometry,
        fragment_live_slot_layout,
        peak_resident_bytes_of_the_process: peak_resident_bytes_of_this_process(),
        phases: std::mem::take(&mut clock.timings),
    };
    drop(session);
    Ok(StartingImage {
        tier,
        devices,
        lowest_live_data_inode,
        fragment_segment_starts: written_segment_starts,
        write_generation_of_each_unit: vec![
            WRITE_GENERATION_OF_THE_DATA_MOUNT;
            usize::try_from(live_data_units)
                .expect("P 装得进 usize")
        ],
        next_write_time_seconds: next_write_time(),
        construction,
    })
}

#[cfg(test)]
mod tests;
