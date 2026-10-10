#!/usr/bin/env python3
"""m3-c694-r2 攻方原型（第一轮攻方补丁原样在前，本轮的开关接在后面）。第一轮原文：m3-c694-r1 攻方原型：把冻结副本的一份拷贝改成「臂由环境变量选」的样子（只在拷贝上改，不碰冻结副本）。

用法：make_patched.py <拷贝根>
- crates/singlefs-core/src/mount.rs：
  * 影子账读一条被抛弃根的账与映射现算重读仍不成时（今天在那里拒），按 OPUS_C694_ARM 选：
    空或 today：照今天拒；A：一律跳过（计数照记）；B：只在那条根的树表落点在现行账里有一条覆盖它、代 > 那条根的 txg 的记录时跳过，否则照拒。
  * 可写挂载重建分配器之后，OPUS_C694_ARM 含 C：把环里读得出、txg 高于施加之后那一版、没施加的记录点名的单元与四条新根指针的落点在内存里隔离
    （现行账里仍分配着的不隔；隔离位这次挂载内不清）。
- crates/singlefs-core/src/mounted_read.rs：OPUS_C695_ARM 选 open_pool_for_read 的次序：空或 today 照今天；R：先读树表、0 条就报
  TreeTableHasNoEntries，再读映射；S：映射根是空指针就当映射 0 条、不读，往下照今天。
- 加一个工作区成员 crates/opus-c694-probe（原型二进制，源码从模型目录拷）。
每一处替换都要求旧串恰好命中一次，不中就退 3。
"""
import pathlib
import shutil
import sys

root = pathlib.Path(sys.argv[1])
here = pathlib.Path(__file__).resolve().parent


def replace_once(path, old, new):
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        print(f"{path}: 旧串命中 {count} 次（要恰好 1 次）：{old[:80]!r}", file=sys.stderr)
        sys.exit(3)
    path.write_text(text.replace(old, new))


mount = root / "crates/singlefs-core/src/mount.rs"
replace_once(
    mount,
    """                                Some(placements) => placements,
                                None => return Err(
                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(""",
    """                                Some(placements) => placements,
                                None if opus_c694_skip_the_unreadable_abandoned_root(root, current_records) => continue,
                                None => return Err(
                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(""",
)
replace_once(
    mount,
    """/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，""",
    """/// m3-c694-r1 攻方原型：臂由环境变量 OPUS_C694_ARM 选（`+` 连几个臂）。
fn opus_c694_arm_has(arm: &str) -> bool {
    std::env::var("OPUS_C694_ARM").unwrap_or_default().split('+').any(|name| name == arm)
}

/// 臂 A：一律跳过；臂 B：被抛弃根的树表落点（每条位置条目）在现行账里各有一条覆盖它、代 > 那条根的 txg 的记录才跳过。
fn opus_c694_skip_the_unreadable_abandoned_root(
    root: &RootRecord,
    current_records: &[AllocationRecord],
) -> bool {
    let skip = if opus_c694_arm_has("A") {
        true
    } else if opus_c694_arm_has("B") {
        root.tree_table.locations.iter().all(|location| {
            current_records.iter().any(|record| {
                record.device == location.device
                    && record.slot.0 <= location.slot.0
                    && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
                    && record.generation > root.checkpoint_txg
            })
        })
    } else {
        false
    };
    if std::env::var("OPUS_C694_TRACE").is_ok() {
        eprintln!(
            "OPUS_C694_UNREADABLE_ABANDONED root=({},{}) skip={skip}",
            root.instance.0, root.checkpoint_txg.0
        );
    }
    skip
}

/// 臂 C：可写挂载时，环里读得出、txg 高于施加之后那一版的记录（没施加的）点名的单元与四条新根指针的落点在内存里隔离（现行账里仍分配着的不隔）。
#[allow(clippy::ptr_arg, reason = "原型")]
fn opus_c694_isolate_units_named_by_unapplied_records<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    allocator: &mut PoolAllocator,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    effective_root: &RootRecord,
) -> u64 {
    let allocated: BTreeSet<(u32, u64)> = allocator
        .records()
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut isolated = 0;
    for record in records.values() {
        if record.checkpoint_txg <= effective_root.checkpoint_txg {
            continue;
        }
        let mut located: Vec<([LocationEntry; 2], u64)> = vec![
            (record.new_tree_table.locations, 1),
            (record.new_mapping_root.locations, 1),
            (record.new_instance_table.locations, 1),
            (record.new_allocation_record_tree_root.locations, 1),
        ];
        for named in &record.named {
            let span = match named.unit_class {
                crate::unit::UNIT_CLASS_DATA | crate::unit::UNIT_CLASS_PACKED => 2,
                _ => 1,
            };
            located.push((named.locations, span));
        }
        for (locations, span) in located {
            for location in locations {
                if location.slot.0 == 0 {
                    continue;
                }
                if placement_lies_in_the_unit_area_of_its_device(
                    devices,
                    unit_area_start,
                    location.device,
                    location.slot,
                    span,
                )
                .is_err()
                {
                    continue;
                }
                if allocated.contains(&(location.device.0, location.slot.0)) {
                    continue;
                }
                allocator.isolate_abandoned(location.device, location.slot, span);
                isolated += 1;
            }
        }
    }
    if std::env::var("OPUS_C694_TRACE").is_ok() {
        eprintln!("OPUS_C694_C_ISOLATED placements={isolated}");
    }
    isolated
}

/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，""",
)
replace_once(
    mount,
    """    } = rebuilt?;
    allocator.set_deletion_switches(deletion_switches);""",
    """    } = rebuilt?;
    if opus_c694_arm_has("C") {
        opus_c694_isolate_units_named_by_unapplied_records(
            devices,
            unit_area_start,
            &mut allocator,
            &records,
            &effective_root,
        );
    }
    allocator.set_deletion_switches(deletion_switches);""",
)

mounted_read = root / "crates/singlefs-core/src/mounted_read.rs"
replace_once(
    mounted_read,
    """    let mapping_tree = read_code_two_tree(
        &root.mapping_root,""",
    """    // m3-c694-r1 攻方原型：OPUS_C695_ARM=R 先判树表 0 条再读映射；=S 只在映射根是空指针时先判树表 0 条。
    let opus_c695_arm = std::env::var("OPUS_C695_ARM").unwrap_or_default();
    if opus_c695_arm == "R"
        || (opus_c695_arm == "S" && root.mapping_root == NodePointer::empty_root())
    {
        let tree_table_bytes =
            read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
        let tree_table = parse_index_node(&tree_table_bytes).map_err(|error| {
            OpenPoolForReadFailure::UnitMalformed {
                what: "树表单元",
                error,
            }
        })?;
        if tree_table.entries.is_empty() {
            return Err(OpenPoolForReadFailure::TreeTableHasNoEntries);
        }
    }
    let mapping_tree = read_code_two_tree(
        &root.mapping_root,""",
)


# ---------------------------------------------------------------------------------------------
# m3-c694-r2 攻方（本轮）加的开关，全部默认关 = 第一轮补丁的样子（OPUS_C694_ARM 选臂照旧）：
#   OPUS_C_SELECT=last     C 只认「这次发布带末条标志的那一条读得出」的发布的记录（缺省 all = 第一轮 C：水位之上全部）。
#   OPUS_C_RELEASE=ring    C 隔离的落点在这个进程写那条记录的 txg 对应的根环槽时解开（「这个进程写掉那一槽」）。
#   OPUS_C_RELEASE=d16     ring 之外，D16 已定项 1「读回全零的槽」那几样解开时机也解：进程内记根时重读那一槽读出有效根或被抛弃的根。
#                          缺省（mount）= 第一轮 C：这次挂载内不解。
#   OPUS_C694_ARM 含 W     B 的另一种凭据（第一轮正推甲 W1 的写法照抄）：树表与映射根两个指针每条位置条目都读出诞生身份不同的合法码 2 节点。
#   OPUS_RECOVERY_FALLBACK=strict|loose  恢复一侧补法（recovery.rs 的 recover）：所选那一版走读失败、系统配置没见证它时退回见证到的那一版。
#                          strict：所选那一版末条读不出时拿 c_见证 那一条比，也读不出就当见证过（不退）；loose：末条读不出就当没见证（退）。
# 另：thread_local 计数 OPUS_SKIPS（B / W / A 跳过一条读不出账的被抛弃根就加一），原型在同一线程上读。
replace_once(
    mount,
    """    let mut isolated = 0;
    for record in records.values() {
        if record.checkpoint_txg <= effective_root.checkpoint_txg {
            continue;
        }
        let mut located""",
    """    let mut isolated = 0;
    let opus_select_last = std::env::var("OPUS_C_SELECT").is_ok_and(|v| v == "last");
    let opus_slots_per_region = allocator.root_ring_occupancy().map(|occupancy| occupancy.slots_per_region());
    let opus_publishes_with_a_readable_last_record: BTreeSet<(InstanceGeneration, CheckpointTxg)> = records
        .values()
        .filter(|record| record.place_in_publish == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish)
        .map(|record| (record.instance, record.checkpoint_txg))
        .collect();
    for record in records.values() {
        if record.checkpoint_txg <= effective_root.checkpoint_txg {
            continue;
        }
        if opus_select_last && !opus_publishes_with_a_readable_last_record.contains(&(record.instance, record.checkpoint_txg)) {
            continue;
        }
        let mut opus_pinned: Vec<(DeviceIdentity, SlotNumber, u64)> = Vec::new();
        let mut located""",
)
replace_once(
    mount,
    """                allocator.isolate_abandoned(location.device, location.slot, span);
                isolated += 1;
            }
        }
    }""",
    """                allocator.isolate_abandoned(location.device, location.slot, span);
                opus_pinned.push((location.device, location.slot, span));
                isolated += 1;
            }
        }
        if let Some(slots_per_region) = opus_slots_per_region {
            allocator.opus_c_pins.push((
                crate::root_ring::target_for_publish(record.checkpoint_txg, slots_per_region),
                opus_pinned,
            ));
        }
    }""",
)
replace_once(
    mount,
    """None if opus_c694_skip_the_unreadable_abandoned_root(root, current_records) => continue,""",
    """None if opus_c694_skip_the_unreadable_abandoned_root(root, current_records, devices) => continue,""",
)
replace_once(
    mount,
    """fn opus_c694_skip_the_unreadable_abandoned_root(
    root: &RootRecord,
    current_records: &[AllocationRecord],
) -> bool {
    let skip = if opus_c694_arm_has("A") {""",
    """thread_local! {
    /// m3-c694-r2 攻方原型：这个线程上 B / W / A 跳过了几条读不出账的被抛弃根。
    pub static OPUS_SKIPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// m3-c694-r2 攻方原型：第一轮正推甲 W1 照抄（树表与映射根两个指针，每条位置条目都读出诞生身份不同的合法码 2 节点）。
fn opus_w1_bootstrap_units_overwritten<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
) -> bool {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let overwritten = |pointer: &crate::pointer::NodePointer| -> bool {
        if *pointer == crate::pointer::NodePointer::empty_root() {
            return false;
        }
        pointer.locations.iter().all(|location| {
            let Some(bytes) = PoolReader::read(
                devices.as_slice(),
                location.device,
                location.slot.to_device_offset(),
                node_bytes,
            ) else {
                return false;
            };
            if crc32_castagnoli(&bytes) == location.unit_checksum {
                return false;
            }
            match crate::unit::parse_index_node(&bytes) {
                Ok(header) => {
                    header.birth_txg != pointer.head.birth_txg
                        || header.instance != pointer.instance
                        || header.birth_sequence != pointer.birth_sequence
                }
                Err(_) => false,
            }
        })
    };
    overwritten(&root.tree_table) && overwritten(&root.mapping_root)
}

#[allow(clippy::ptr_arg, reason = "原型")]
fn opus_c694_skip_the_unreadable_abandoned_root<Device: BlockDevice>(
    root: &RootRecord,
    current_records: &[AllocationRecord],
    devices: &Vec<(DeviceIdentity, Device)>,
) -> bool {
    let skip = opus_c694_skip_inner(root, current_records, devices);
    if skip {
        OPUS_SKIPS.with(|count| count.set(count.get() + 1));
    }
    skip
}

#[allow(clippy::ptr_arg, reason = "原型")]
fn opus_c694_skip_inner<Device: BlockDevice>(
    root: &RootRecord,
    current_records: &[AllocationRecord],
    devices: &Vec<(DeviceIdentity, Device)>,
) -> bool {
    let skip = if opus_c694_arm_has("W") {
        opus_w1_bootstrap_units_overwritten(devices, root)
    } else if opus_c694_arm_has("A") {""",
)

allocator = root / "crates/singlefs-core/src/allocator.rs"
replace_once(
    allocator,
    """    deletion_switches: DeletionSwitches,
}

impl PoolAllocator {""",
    """    deletion_switches: DeletionSwitches,
    /// m3-c694-r2 攻方原型：C 隔离的落点，按那条记录的 txg 对应的根环槽分组（OPUS_C_RELEASE 选什么时候解开）。
    pub opus_c_pins: Vec<(RootRingSlot, Vec<(DeviceIdentity, SlotNumber, u64)>)>,
}

impl PoolAllocator {
    /// m3-c694-r2 攻方原型：解开根环槽 `ring_slot` 名下 C 隔离的落点（清隔离位）。
    pub fn opus_release_c_pins_at(&mut self, ring_slot: RootRingSlot, why: &str) {
        let (released, kept): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.opus_c_pins).into_iter().partition(|(slot, _)| *slot == ring_slot);
        self.opus_c_pins = kept;
        for (_, placements) in released {
            if std::env::var("OPUS_C694_TRACE").is_ok() {
                eprintln!("OPUS_C_RELEASED why={why} placements={}", placements.len());
            }
            for (device, slot, span) in placements {
                if let Some(device_map) = self.devices.iter_mut().find(|map| map.device == device) {
                    for s in slot.0..slot.0 + span {
                        device_map.clear_isolation_of_slot(SlotNumber(s));
                    }
                }
            }
        }
    }

    fn opus_release_mode() -> String {
        std::env::var("OPUS_C_RELEASE").unwrap_or_default()
    }
""",
)
replace_once(
    allocator,
    """            deletion_switches: DeletionSwitches::PRODUCT,
        }
    }

    /// 装上删除那几次发布的只供测试的开关""",
    """            deletion_switches: DeletionSwitches::PRODUCT,
            opus_c_pins: Vec::new(),
        }
    }

    /// 装上删除那几次发布的只供测试的开关""",
)
replace_once(
    allocator,
    """        let reclaimed =
            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
        self.isolate_reclaimed_slots_an_abandoned_root_still_references(""",
    """        if matches!(Self::opus_release_mode().as_str(), "ring" | "d16") {
            self.opus_release_c_pins_at(ring_slot, "this-process-wrote-the-ring-slot");
        }
        let reclaimed =
            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
        self.isolate_reclaimed_slots_an_abandoned_root_still_references(""",
)
replace_once(
    allocator,
    """        reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotBadAtMount,
    ) {
        if let Some(occupancy) = self.root_ring.as_mut() {""",
    """        reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotBadAtMount,
    ) {
        let mut opus_reread_resolved: Vec<(RootRingSlot, &'static str)> = Vec::new();
        if let Some(occupancy) = self.root_ring.as_mut() {""",
)
replace_once(
    allocator,
    """                    RereadOfARootRingSlotBadAtMount::ValidRoot {
                        checkpoint_txg: txg_read_on_the_reread,
                    } => {
                        occupancy.occupants.insert(""",
    """                    RereadOfARootRingSlotBadAtMount::ValidRoot {
                        checkpoint_txg: txg_read_on_the_reread,
                    } => {
                        opus_reread_resolved.push((ring_slot, "reread-valid-root"));
                        occupancy.occupants.insert(""",
)
replace_once(
    allocator,
    """                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {
                        occupancy.occupants.remove(&ring_slot);
                    }
                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
                    | RereadOfARootRingSlotBadAtMount::AllZero => {}
                }
            }
        }
        self.record_root_written_by_this_process(checkpoint_txg);""",
    """                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {
                        opus_reread_resolved.push((ring_slot, "reread-abandoned-root"));
                        occupancy.occupants.remove(&ring_slot);
                    }
                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
                    | RereadOfARootRingSlotBadAtMount::AllZero => {}
                }
            }
        }
        if std::env::var("OPUS_C694_TRACE").is_ok() {
            for (ring_slot, why) in &opus_reread_resolved {
                eprintln!("OPUS_D16_PIN_RESOLVED why={why} ring_slot=({},{})", ring_slot.region, ring_slot.slot);
            }
        }
        if Self::opus_release_mode() == "d16" {
            for (ring_slot, why) in opus_reread_resolved {
                self.opus_release_c_pins_at(ring_slot, why);
            }
        }
        self.record_root_written_by_this_process(checkpoint_txg);""",
)

recovery = root / "crates/singlefs-core/src/recovery.rs"
replace_once(
    recovery,
    """        Err(failure) => RecoveryOutcome::Failed {
            root: Some(root_key),
            failure,
        },
    };
    RecoveryReport {
        outcome,
        effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),
        journal,""",
    """        Err(failure) => RecoveryOutcome::Failed {
            root: Some(root_key),
            failure,
        },
    };
    let (outcome, effective_root) = opus_recovery_fallback(
        reader,
        &system_configuration,
        policy,
        &root,
        outcome,
        effective_root,
        unit_area_start,
        &mut mapping_fallbacks,
    );
    RecoveryReport {
        outcome,
        effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),
        journal,""",
)
replace_once(
    recovery,
    """/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。""",
    """thread_local! {
    /// m3-c694-r2 攻方原型：这个线程上恢复一侧补法退回过几次。
    pub static OPUS_FALLBACKS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// m3-c694-r2 攻方原型（OPUS_RECOVERY_FALLBACK=strict|loose）：所选那一版走读失败、系统配置没见证它时退回见证到的那一版。
/// c_见证 = 每块盘两槽里自证过的系统配置的 journal tail 最大值（同 mount::newer_publish_witness）。所选那一版见证过 ⟺
/// 它那次发布带末条标志的记录读得出且计数器 ≤ c_见证；末条读不出时拿计数器 = c_见证 的那条记录比 (txg, 实例)：它不比所选那一版旧就算见证过；
/// 两条都读不出：strict 当见证过（不退），loose 当没见证（退）。退到哪一版：c_见证 那条记录读得出时取根环里 (txg, 实例) 不超过它、
/// 比所选根旧的最新一条根；读不出时取比所选根旧的最新一条根。退过去不重放，直接走读；走不通就照报原来的失败。
#[allow(clippy::too_many_arguments, reason = "原型")]
fn opus_recovery_fallback(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    policy: JournalPolicy,
    chosen_root: &RootRecord,
    outcome: RecoveryOutcome,
    effective_root: RootRecord,
    unit_area_start: UnitAreaStart,
    mapping_fallbacks: &mut usize,
) -> (RecoveryOutcome, RootRecord) {
    let mode = std::env::var("OPUS_RECOVERY_FALLBACK").unwrap_or_default();
    if !(mode == "strict" || mode == "loose")
        || policy == JournalPolicy::Ignore
        || !matches!(outcome, RecoveryOutcome::Failed { .. })
    {
        return (outcome, effective_root);
    }
    let spacing = u64::from(system_configuration.immutable.sizes.fixed_structure_slot_spacing);
    let witnessed_counter = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            system_configuration_slots_of_one_device_read_once(
                reader,
                device,
                spacing,
                &system_configuration.immutable.filesystem_identifier,
            )
            .self_verified
        })
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    let records = scan_journal(reader, system_configuration);
    let version_key = (effective_root.checkpoint_txg, effective_root.instance);
    let last_record_counter = records
        .values()
        .filter(|record| {
            (record.checkpoint_txg, record.instance) == version_key
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max();
    let record_at_the_witnessed_counter = records
        .values()
        .filter(|record| record.counter == witnessed_counter)
        .map(|record| (record.checkpoint_txg, record.instance))
        .max();
    let witnessed = match (last_record_counter, record_at_the_witnessed_counter) {
        (Some(counter), _) => counter <= witnessed_counter,
        (None, Some(key)) => key >= version_key,
        (None, None) => mode == "strict",
    };
    if witnessed {
        return (outcome, effective_root);
    }
    let chosen_key = (chosen_root.checkpoint_txg, chosen_root.instance);
    let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen_key);
    let immutable = &system_configuration.immutable;
    let fallback = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| match read_root_ring_slot(
            reader,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => Some(root),
            RootRingSlotReading::Bad(_) => None,
        })
        .filter(|root| {
            let key = (root.checkpoint_txg, root.instance);
            key < chosen_key && key <= ceiling
        })
        .max_by_key(|root| (root.checkpoint_txg, root.instance));
    let Some(fallback) = fallback else {
        return (outcome, effective_root);
    };
    let fallback_key = (fallback.instance, fallback.checkpoint_txg);
    let fallback_outcome = match walk_to_file_in_the_unit_area_starting_at(
        reader,
        unit_area_start,
        &fallback,
        mapping_fallbacks,
    ) {
        Ok(Some(content)) => RecoveryOutcome::FileRead { root: fallback_key, content },
        Ok(None) => RecoveryOutcome::NoFile { root: fallback_key },
        Err(_) => return (outcome, effective_root),
    };
    OPUS_FALLBACKS.with(|count| count.set(count.get() + 1));
    (fallback_outcome, fallback)
}

/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。""",
)

# 原型二进制：工作区加一个成员，源码从模型目录拷。
probe = root / "crates/opus-c694r2-probe"
(probe / "src").mkdir(parents=True, exist_ok=True)
shutil.copy(here / "probe" / "Cargo.toml", probe / "Cargo.toml")
shutil.copy(here / "probe" / "main.rs", probe / "src" / "main.rs")
replace_once(
    root / "Cargo.toml",
    """    "crates/singlefs-reference-filesystem",
]""",
    """    "crates/singlefs-reference-filesystem",
    "crates/opus-c694r2-probe",
]""",
)
print("patched", root)

# 跟踪（只在 OPUS_C694_TRACE 设了时打）：挂载时钉着的读回全零的槽，与进程内每一次重读的结局。
replace_once(
    allocator,
    """            for (ring_slot, reread_of_the_slot) in rereads {
                match reread_of_the_slot {""",
    """            for (ring_slot, reread_of_the_slot) in rereads {
                if std::env::var("OPUS_C694_TRACE").is_ok() {
                    eprintln!("OPUS_D16_REREAD ring_slot=({},{}) txg_recorded={} outcome={reread_of_the_slot:?}", ring_slot.region, ring_slot.slot, checkpoint_txg.0);
                }
                match reread_of_the_slot {""",
)
replace_once(
    mount,
    """    if std::env::var("OPUS_C694_TRACE").is_ok() {
        eprintln!("OPUS_C694_C_ISOLATED placements={isolated}");""",
    """    if std::env::var("OPUS_C694_TRACE").is_ok() {
        if let Some(occupancy) = allocator.root_ring_occupancy() {
            eprintln!(
                "OPUS_MOUNT_PINS all_zero={:?} bad={:?}",
                occupancy.ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held(),
                occupancy.ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
            );
        }
        eprintln!("OPUS_C694_C_ISOLATED placements={isolated}");""",
)
