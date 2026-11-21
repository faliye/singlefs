//! 第 2 次跑的历史：起点、覆盖写、关闭，抛弃那一步要读不出的落点与今天那条规则的取号。

use super::*;

// ---------------------------------------------------------------------------
// 10.2 历史：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写 → 关闭（C：丢掉会话；U：调 `unmount`）。
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Closing {
    ProcessExit,
    NormalUnmount,
}

impl Closing {
    pub(super) fn name(self) -> &'static str {
        match self {
            Closing::ProcessExit => "C",
            Closing::NormalUnmount => "U",
        }
    }
}

pub(super) struct SecondRunHistory {
    pub(super) pool: MemoryPool,
    pub(super) session: Option<Session>,
    /// 当前时间线（父链，跑前登记 5.0）：mkfs 的第 0 代根起，到最新一次成功发布的根。
    pub(super) timeline: Vec<TimelineRoot>,
    pub(super) content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
    pub(super) publishes_with_content: u64,
}

impl SecondRunHistory {
    pub(super) fn tip(&self) -> TimelineRoot {
        *self.timeline.last().expect("时间线至少有第 0 代根")
    }

    pub(super) fn tip_content(&self) -> Option<Vec<u8>> {
        self.content_by_root
            .get(&self.tip())
            .cloned()
            .unwrap_or(None)
    }
}

fn push_mounted_publishes(
    history: &mut SecondRunHistory,
    mounted: &Mounted,
    carried_forward_content: &Option<Vec<u8>>,
) {
    let mut roots = vec![root_pair(mounted.output.row_publish.root())];
    roots.extend(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| root_pair(publish.root())),
    );
    for raised in mounted.output.space_admission.floor_raises() {
        roots.extend(
            raised
                .publishes
                .iter()
                .map(|publish| root_pair(&publish.root)),
        );
    }
    for root in roots {
        history.timeline.push(root);
        history
            .content_by_root
            .insert(root, carried_forward_content.clone());
    }
}

pub(super) fn second_run_start(
    geometry: &Geometry,
    overwrites: u64,
) -> Result<SecondRunHistory, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let mounted = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("第一次可写挂载: {error:?}"))?;
    let mut history = SecondRunHistory {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: None,
        timeline: vec![root_pair(&genesis.root)],
        content_by_root: BTreeMap::from([(root_pair(&genesis.root), None)]),
        publishes_with_content: 0,
    };
    push_mounted_publishes(&mut history, &mounted, &None);
    let mut allocator = mounted.allocator;
    let instance = mounted.output.instance;
    let current = mounted.current;
    let content = deterministic_content(1000, FIRST_FILE_BYTES);
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            current.record_bytes(),
        )
        .map_err(|error| format!("首个文件: {error:?}"))?
    };
    history.timeline.push(root_pair(&first.root));
    history
        .content_by_root
        .insert(root_pair(&first.root), Some(content));
    history.publishes_with_content = 1;
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    history.session = Some(Session {
        allocator,
        current: PoolVersion::WithFile(std::sync::Arc::new(first)),
        instance,
    });
    for _ in 0..overwrites {
        second_run_overwrite(&mut history, &parameters)?;
    }
    Ok(history)
}

fn second_run_overwrite(
    history: &mut SecondRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<(), String> {
    let session = history.session.as_mut().ok_or("覆盖写：没有开着的会话")?;
    let PoolVersion::WithFile(previous) = &session.current else {
        return Err("覆盖写：现行版本没有文件".to_string());
    };
    let tag = 1000 + history.publishes_with_content;
    let content = deterministic_content(tag, OVERWRITE_FILE_BYTES);
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut session.allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * tag,
            },
            session.instance,
        )
        .map_err(|error| format!("覆盖写: {error:?}"))?
    };
    history.timeline.push(root_pair(&output.root));
    history
        .content_by_root
        .insert(root_pair(&output.root), Some(content));
    history.publishes_with_content += 1;
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    session.current = PoolVersion::WithFile(std::sync::Arc::new(output));
    Ok(())
}

/// 每块盘两槽里世代号最大的那一份自证系统配置槽的字节（按 `crates` 的解析判自证）。
pub(super) fn newest_system_configuration_slot_bytes(
    pool: &MemoryPool,
    device: DeviceIdentity,
    slot_spacing: u64,
) -> Option<Vec<u8>> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    [0u64, slot_spacing]
        .into_iter()
        .filter_map(|offset| pool.read(device.0, offset, slot_bytes))
        .filter_map(|bytes| {
            singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
                .ok()
                .map(|parsed| (parsed.quantities.slot_generation, bytes))
        })
        .max_by_key(|(generation, _)| *generation)
        .map(|(_, bytes)| bytes)
}

/// 关闭 c：C 丢掉会话；U 调 `unmount`，并核 7.1 第四行（F12）与第一行的盘上那一半（最后一个非零字节、乙族 Q2-2b 的解码）。
pub(super) fn second_run_close(
    history: &mut SecondRunHistory,
    parameters: &MakeFilesystemParameters,
    closing: Closing,
    arm: &str,
    context: &str,
) -> Result<(), String> {
    let session = history.session.take().ok_or("关闭：没有开着的会话")?;
    if closing == Closing::ProcessExit {
        return Ok(());
    }
    let Session {
        mut allocator,
        mut current,
        ..
    } = session;
    let before_txg = current.root().checkpoint_txg.0;
    let carried = history.tip_content();
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let unmounted = singlefs_core::mount::unmount(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .map_err(|error| format!("正常卸载: {error:?}"))?;
    let singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        return Err("正常卸载：现行那一版没有文件".to_string());
    };
    for publish in &raised.publishes {
        history.timeline.push(root_pair(&publish.root));
        history
            .content_by_root
            .insert(root_pair(&publish.root), carried.clone());
    }
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let expected_publishes = if before_txg % 3 == 1 { 3 } else { 2 };
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let mut floors = Vec::new();
    let mut last_nonzero = Vec::new();
    let mut published_txg_decoded = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let Some(bytes) =
            newest_system_configuration_slot_bytes(&history.pool, device, slot_spacing)
        else {
            return Err(format!("正常卸载之后盘 {} 没有自证的系统配置槽", device.0));
        };
        let parsed = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
            .map_err(|error| format!("{error:?}"))?;
        floors.push(parsed.quantities.rollback_floor.0);
        last_nonzero.push(bytes.iter().rposition(|byte| *byte != 0).unwrap_or(0));
        let start = usize::try_from(SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL).expect("489");
        let mut little_endian = [0u8; 8];
        little_endian.copy_from_slice(&bytes[start..start + 8]);
        published_txg_decoded.push(u64::from_le_bytes(little_endian));
    }
    let floor_holds = floors.iter().all(|floor| *floor == before_txg);
    let publishes_hold = raised.publishes.len() == expected_publishes;
    // 7.1 第一行的盘上那一半：今天的字段表（本地 489）下最后一个非零字节落在 F 那 8 字节 [481, 489) 里、489 起全零；
    // 乙 / 丁四臂（本地 497）下 [489, 497) 解出的就是刚落盘那条根的 txg（Q2-2b 的解码核对）。
    let field_end = local_system_configuration_bytes();
    let floor_start = SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL - 8;
    let layout_holds = if field_end == SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL {
        last_nonzero.iter().all(|index| {
            let index = u64::try_from(*index).expect("槽内下标");
            floor_start <= index && index < field_end
        })
    } else {
        published_txg_decoded
            .iter()
            .all(|decoded| *decoded == history.tip().1)
    };
    emit_result(&format!(
        "name=r2_unmount_check arm={arm} context={context:?} verdict={} layout_verdict={} txg_before_unmount={before_txg} floor_in_newest_system_configuration={floors:?} unmount_publishes={} expected_publishes={expected_publishes} last_nonzero_byte={last_nonzero:?} bytes_489_to_497_as_txg={published_txg_decoded:?} newest_root_txg={}",
        if floor_holds && publishes_hold { "pass" } else { "fail_F12" },
        if layout_holds { "pass" } else { "fail_S3" },
        raised.publishes.len(),
        history.tip().1
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// 10.3 装置自己的账：落点、一条根引用的单元、择根、被抛弃判定。
// ---------------------------------------------------------------------------

/// 一条根（(实例, txg)）的根槽在哪（checker 的独立解码：自证过的根里找到它、再按 (区域, 槽) 取 (盘, 偏移)）。
pub(super) fn root_slot_of(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    root: TimelineRoot,
) -> Option<(DeviceIdentity, u64)> {
    let positions = checker_image::root_slot_positions(geometry);
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .find(|(_, _, view)| (view.instance, view.checkpoint_txg) == root)
        .and_then(|(region, slot, _)| {
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, device, offset)| (DeviceIdentity(*device), *offset))
        })
}

/// 一次发布（按 (实例, txg) 认）在 journal 环里的记录：逐槽解两块盘上的那一份，交回 (计数器, 偏移, 记录)，按计数器排。
pub(super) fn records_of_publish(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    root: TimelineRoot,
) -> Vec<(u64, u64, JournalRecord)> {
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let mut found: BTreeMap<u64, (u64, JournalRecord)> = BTreeMap::new();
    for slot_index in 0..ring_slots {
        let offset = record_offset(slot_index + 1, geometry.journal_ring_bytes);
        for device_number in [0u32, 1u32] {
            let Some(bytes) = pool.read(device_number, offset.0, record_bytes) else {
                continue;
            };
            let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) else {
                continue;
            };
            if (record.instance.0, record.checkpoint_txg.0) == root {
                found.entry(record.counter).or_insert((offset.0, record));
            }
        }
    }
    found
        .into_iter()
        .map(|(counter, (offset, record))| (counter, offset, record))
        .collect()
}

/// 断链发布的重放怎么接不上（跑前登记 5.1 抛弃步 A(k, b) 的 b）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayBreak {
    /// b = rec：那次发布的全部记录的全部份读不出。
    EveryRecord,
    /// b = unit：那次发布第一条记录点名的第一个单元两份读不出。
    FirstNamedUnit,
    /// b = unit_first_copy（第 2 次跑修订加的，只补不改）：只让那个单元排在位置条目第一条的那一份读不出。
    /// 重放验点名单元要全部份都验过（`recovery.rs` `replay_journal` 里 `named.locations.iter().all`），第一份读错就不再读第二份，
    /// b = unit 的第二份于是一次都拦不到（V2 作废）；这一支量「一份就够」。
    FirstCopyOfTheFirstNamedUnit,
}

impl ReplayBreak {
    pub(super) fn name(self) -> &'static str {
        match self {
            ReplayBreak::EveryRecord => "records",
            ReplayBreak::FirstNamedUnit => "unit",
            ReplayBreak::FirstCopyOfTheFirstNamedUnit => "unit_first_copy",
        }
    }
}

/// 抛弃步的落点：当前时间线上最新 `newest_roots_hidden` 条根的根槽，加断链发布（当前时间线上第 `newest_roots_hidden` 新的那条根的
/// 那次发布）按 `replay_break` 取的落点。断链发布没有记录、或 b = unit 而它没有点名单元时交 None（这一支「不适用」）。
pub(super) fn unreadable_ranges_of_the_abandonment(
    history: &SecondRunHistory,
    geometry: &PoolGeometry,
    newest_roots_hidden: usize,
    form: UnreadableForm,
    replay_break: ReplayBreak,
) -> Result<Option<Vec<UnreadableRange>>, String> {
    let count = history.timeline.len();
    if newest_roots_hidden == 0 || newest_roots_hidden >= count {
        return Err(format!(
            "藏 {newest_roots_hidden} 条根：时间线只有 {count} 条"
        ));
    }
    let mut ranges = Vec::new();
    for root in &history.timeline[count - newest_roots_hidden..] {
        let (device, offset) = root_slot_of(&history.pool, geometry, *root)
            .ok_or_else(|| format!("时间线上的根 {root:?} 在根环里读不出（不绕环时不应发生）"))?;
        ranges.push(UnreadableRange {
            device,
            offset,
            length: u64::from(geometry.physical_block_size),
            form,
            label: format!("root{root:?}"),
        });
    }
    let broken = history.timeline[count - newest_roots_hidden];
    let records = records_of_publish(&history.pool, geometry, broken);
    if records.is_empty() {
        return Ok(None);
    }
    match replay_break {
        ReplayBreak::EveryRecord => {
            for (counter, offset, _) in &records {
                for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    ranges.push(UnreadableRange {
                        device,
                        offset: *offset,
                        length: JOURNAL_RECORD_BYTES,
                        form,
                        label: format!("record(counter={counter},device={})", device.0),
                    });
                }
            }
        }
        ReplayBreak::FirstNamedUnit | ReplayBreak::FirstCopyOfTheFirstNamedUnit => {
            let Some(named) = records
                .iter()
                .find_map(|(_, _, record)| record.named.first())
            else {
                return Ok(None);
            };
            let copies = if replay_break == ReplayBreak::FirstNamedUnit {
                named.locations.len()
            } else {
                1
            };
            for location in named.locations.iter().take(copies) {
                ranges.push(UnreadableRange {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form,
                    label: format!(
                        "unit(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                });
            }
        }
    }
    Ok(Some(ranges))
}

/// 一条根引用的单元（装在 `pool` 上读）：它的账里没释放的分配记录（`recovery::allocation_records_under_root`），
/// 加根记录直接指着的树表、映射树根、分配记录树根与实例表链认得出的各片。交回 (盘, 起始槽, 槽数)。
pub(super) fn placements_referenced_by_root_on(
    pool: &MemoryPool,
    root: &RootRecord,
) -> Result<BTreeSet<(u32, u64, u64)>, String> {
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let mut placements = BTreeSet::new();
    let records = allocation_records_under_root(&devices, root)
        .map_err(|error| format!("账读不出: {error:?}"))?;
    for record in records.iter().filter(|record| !record.is_released) {
        placements.insert((record.device.0, record.slot.0, u64::from(record.span_slots)));
    }
    let direct_pointers = [
        &root.tree_table,
        &root.mapping_root,
        &root.allocation_record_tree_root,
    ];
    let instance_table_pages =
        singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(&devices, root);
    for pointer in direct_pointers
        .into_iter()
        .chain(instance_table_pages.iter())
    {
        for location in &pointer.locations {
            if location.slot.0 != 0 {
                placements.insert((location.device.0, location.slot.0, 1));
            }
        }
    }
    Ok(placements)
}

/// `placements` 里有几处在 `before` 与 `after` 两份镜像上字节不同（被覆盖）。
pub(super) fn placements_overwritten_between(
    before: &MemoryPool,
    after: &MemoryPool,
    placements: &BTreeSet<(u32, u64, u64)>,
) -> u64 {
    placements
        .iter()
        .filter(|(device, slot, span)| {
            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
            before.read(*device, slot * SLOT_BYTES_LOCAL, length)
                != after.read(*device, slot * SLOT_BYTES_LOCAL, length)
        })
        .count() as u64
}

/// 这份镜像上恢复会落到哪条根：`crates` 的 `choose_root` 与装置自己的择根（checker 解出的自证根里 (txg, 实例) 最大）逐条比，
/// 对不上走停机 S5。交回 (那条根, 它的根记录)。
pub(super) fn landing_root_on(pool: &MemoryPool) -> Result<(TimelineRoot, RootRecord), String> {
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let system_configuration = singlefs_core::recovery::choose_system_configuration(&devices)
        .map_err(|error| format!("择不出系统配置: {error:?}"))?;
    let crates_choice = singlefs_core::recovery::choose_root(&devices, &system_configuration)
        .ok_or("crates 择不出根")?;
    let geometry = independent_geometry(pool)?;
    let device_choice = readable_roots_independent(pool, &geometry)
        .into_iter()
        .max_by_key(|(instance, txg)| (*txg, *instance))
        .ok_or("装置解不出任何自证根")?;
    if root_pair(&crates_choice) != device_choice {
        return Err(format!(
            "S5：crates 择根 {:?} 与装置自己择的 {device_choice:?} 对不上",
            root_pair(&crates_choice)
        ));
    }
    Ok((device_choice, crates_choice))
}

/// 这份镜像上被抛弃的根（读法写死表「被抛弃的根」）：根环里 (txg, 实例) 最大那条根指着的实例表里，有行 (i, T) 且根的实例 = i、
/// txg > T。实例表按 `crates` 的解析读出行，判定是装置自己的式子；逐条与 `root_is_abandoned_by_the_instance_table` 比，对不上走 S5。
pub(super) fn abandoned_roots_on(pool: &MemoryPool) -> Result<BTreeSet<TimelineRoot>, String> {
    let (_, newest) = landing_root_on(pool)?;
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let table = instance_table_chain_of_root(&devices, &newest)
        .map_err(|error| format!("最新根的实例表读不出: {error:?}"))?
        .records;
    let system_configuration = singlefs_core::recovery::choose_system_configuration(&devices)
        .map_err(|error| format!("{error:?}"))?;
    let mut abandoned = BTreeSet::new();
    for root in singlefs_core::recovery::readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    ) {
        let by_the_device = table.rows.iter().any(|row| {
            row.instance.0 == root.instance.0 && root.checkpoint_txg.0 > row.selected_root_txg.0
        });
        let by_crates =
            singlefs_core::recovery::root_is_abandoned_by_the_instance_table(&root, &table);
        if by_the_device != by_crates {
            return Err(format!(
                "S5：根 {:?} 装置判被抛弃={by_the_device}、crates 判={by_crates}",
                root_pair(&root)
            ));
        }
        if by_the_device {
            abandoned.insert(root_pair(&root));
        }
    }
    Ok(abandoned)
}

/// 冷启动读回（`recovery::recover`，看 journal）的结局，比「最后一次确认的那一版」的内容：
/// last_confirmed / other_content / no_file / failed:<成员>。另交回读回落到的根。
pub(super) fn read_back_against(
    pool: &MemoryPool,
    last_confirmed: &Option<Vec<u8>>,
) -> (String, Option<TimelineRoot>) {
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let report = recover(&devices, JournalPolicy::Consult);
    let root = chosen_root_of(&report.outcome);
    let label = match &report.outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            if Some(content.clone()) == *last_confirmed {
                "last_confirmed".to_string()
            } else {
                "other_content".to_string()
            }
        }
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::Failed { failure, .. } => {
            format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
        }
    };
    (label, root)
}

/// 新实例第一次发布的 txg 按今天的规则该是几（跑前登记 7.1 第六行）：装置自己解出的、这次调用里读得出的根（藏起来的根槽不算）
/// 与自证记录（两份都藏起来的不算）的 txg 取 max 再 + 1。
pub(super) fn first_txg_by_the_rule_of_today(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    unreadable: &[UnreadableRange],
) -> u64 {
    let slots = checker_image::root_slot_positions(geometry);
    let highest_root = checker_image::valid_roots(pool, geometry)
        .into_iter()
        .filter(|(region, slot, _)| {
            slots
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    candidate_region == region && candidate_slot == slot
                })
                .is_some_and(|(_, _, device, offset)| {
                    !unreadable.iter().any(|range| {
                        range.overlaps(
                            DeviceIdentity(*device),
                            *offset,
                            u64::from(geometry.physical_block_size),
                        )
                    })
                })
        })
        .map(|(_, _, view)| view.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let mut highest_record = 0u64;
    for slot_index in 0..ring_slots {
        let offset = record_offset(slot_index + 1, geometry.journal_ring_bytes);
        for device_number in [0u32, 1u32] {
            if unreadable.iter().any(|range| {
                range.overlaps(
                    DeviceIdentity(device_number),
                    offset.0,
                    JOURNAL_RECORD_BYTES,
                )
            }) {
                continue;
            }
            if let Some(record) = pool
                .read(device_number, offset.0, record_bytes)
                .and_then(|bytes| JournalRecord::parse(&bytes, expected_filesystem_identifier))
            {
                highest_record = highest_record.max(record.checkpoint_txg.0);
            }
        }
    }
    highest_root.max(highest_record) + 1
}

/// 系统配置里读得出的 F 的最大值（乙F-留环的水位那一半，与 `effective_rollback_floor` 的系统配置那一半同一取法）。
pub(super) fn highest_floor_in_the_system_configurations(
    pool: &MemoryPool,
    slot_spacing: u64,
) -> u64 {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    [0u32, 1]
        .into_iter()
        .flat_map(|device| [(device, 0u64), (device, slot_spacing)])
        .filter_map(|(device, offset)| pool.read(device, offset, slot_bytes))
        .filter_map(|bytes| {
            singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes).ok()
        })
        .map(|parsed| parsed.quantities.rollback_floor.0)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
