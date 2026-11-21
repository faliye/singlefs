use super::*;

// ============================================================================
// 六、判法这一侧（只读镜像）：候选集、由记录施加出来的版本、相邻对与洞对、两条臂。
//    取法照 `walk.rs` 今天的读法另写一份（登记第一节「候选集与相邻对」、第三节；逐句对照表在执行员报告里）。
// ============================================================================

/// R 的实例表指针出生 txg：读得出的那个数，或指针全零、读不出（甲把这一对按「不认作取号跳过」处理，另计一格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PointerBirth {
    Readable(u64),
    AllZeroOrUnreadable,
}

/// 镜像上的一版：候选集里的一条根，或由记录施加出来、根槽从没落盘的一版（两样分别放在 `ImageReadingOfTheJudgements` 的两个字段里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VersionOnTheImage {
    pub(super) key: VersionKey,
    pub(super) pointer_birth: PointerBirth,
}

/// 根环里一条自证过的根：住的区域、它的版本、带的 F、实例表指针出生 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RootOnTheRing {
    pub(super) region: u64,
    pub(super) key: VersionKey,
    pub(super) rollback_floor: u64,
    pub(super) pointer_birth: PointerBirth,
}

/// 环里一条记录（扫环的读法，每个 (实例, 计数器) 一条）：判「由记录施加出来」要的几样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordOnTheRing {
    pub(super) key: VersionKey,
    is_commit: bool,
    is_the_last_record_of_the_publish: bool,
    pub(super) pointer_birth: PointerBirth,
}

/// 最新根实例表的一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RowOfTheNewestTable {
    pub(super) instance: u32,
    pub(super) published_checkpoint_txg: u64,
    pub(super) applied_transaction_high_water_mark: u64,
}

/// 相邻两组之间有没有洞、今天的判法（H1-A，`walk.rs` 的 `hole_in_the_candidate_set_between` 三条件）认不认作行接得上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HoleClass {
    NoHole,
    BridgedByTheRows,
    NotBridgedByTheRows,
}

/// H1-A（今天）：最新根的实例表里有一行 (R⁻ 的某个实例 i, T = R⁻.txg)；R 的实例 v > i；i 与 v 之间每个实例 j 都有一行、都是 (j, 0, 0)。
fn hole_class_today(
    previous_txg: u64,
    previous_instances: &BTreeSet<u32>,
    version_txg: u64,
    version_instance: u32,
    rows: &[RowOfTheNewestTable],
) -> HoleClass {
    if previous_txg.checked_add(1) == Some(version_txg) {
        return HoleClass::NoHole;
    }
    let bridged_from = |previous_instance: u32| {
        if version_instance <= previous_instance {
            return false;
        }
        let row_of_the_previous_version = rows.iter().any(|row| {
            row.instance == previous_instance && row.published_checkpoint_txg == previous_txg
        });
        let rows_between: Vec<&RowOfTheNewestTable> = rows
            .iter()
            .filter(|row| row.instance > previous_instance && row.instance < version_instance)
            .collect();
        let instances_between: BTreeSet<u32> =
            rows_between.iter().map(|row| row.instance).collect();
        let every_instance_between_has_a_row = u64::try_from(instances_between.len())
            .expect("行数装得进 u64")
            == u64::from(version_instance - previous_instance - 1);
        let every_row_between_published_nothing = rows_between.iter().all(|row| {
            row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0
        });
        row_of_the_previous_version
            && every_instance_between_has_a_row
            && every_row_between_published_nothing
    };
    if previous_instances
        .iter()
        .any(|previous_instance| bridged_from(*previous_instance))
    {
        HoleClass::BridgedByTheRows
    } else {
        HoleClass::NotBridgedByTheRows
    }
}

/// 甲（被测）：H1-A 的三条件，再加第四条件「R 的实例表指针出生 txg 等于 R.txg」；指针全零或读不出按不认作取号跳过。
/// R⁻ 那一组只用来立断言（相邻对里它们都比 R 早），第四条件只读 R 自己的指针。
pub(super) fn bridged_with_the_pointer_birth(
    hole: HoleClass,
    previous_versions: &[VersionOnTheImage],
    version: &VersionOnTheImage,
) -> bool {
    debug_assert!(previous_versions
        .iter()
        .all(|previous| previous.key.txg < version.key.txg));
    hole == HoleClass::BridgedByTheRows
        && version.pointer_birth == PointerBirth::Readable(version.key.txg)
}

/// 按最新根的实例表判抛弃：有行 (i, Ti) 且 t > Ti。
fn is_abandoned_by_the_rows(key: VersionKey, rows: &[RowOfTheNewestTable]) -> bool {
    rows.iter()
        .any(|row| row.instance == key.instance && key.txg > row.published_checkpoint_txg)
}

/// 择最新根：按 (txg, 实例) 取最大，相同时取读根环次序里最前的那条（只在严格更大时才换）。
fn newest_root_index(roots: &[RootOnTheRing]) -> Option<usize> {
    let mut newest: Option<(usize, (u64, u32))> = None;
    for (index, root) in roots.iter().enumerate() {
        let choice_key = (root.key.txg, root.key.instance);
        match newest {
            Some((_, newest_key)) if choice_key <= newest_key => {}
            Some(_) | None => newest = Some((index, choice_key)),
        }
    }
    newest.map(|(index, _)| index)
}

/// F 生效值：max(每块盘上没被抛弃、(txg, 实例) 最大那条根带的 F, 被抛弃的根带的 F, 自证过、fsid 是本池的系统配置槽带的 F)。
fn effective_rollback_floor(
    roots: &[RootOnTheRing],
    rows: &[RowOfTheNewestTable],
    configuration_floors: &[u64],
) -> u64 {
    let mut newest_valid_on_each_device: BTreeMap<u32, &RootOnTheRing> = BTreeMap::new();
    let mut highest_floor_on_the_abandoned_roots = 0;
    for root in roots {
        if is_abandoned_by_the_rows(root.key, rows) {
            highest_floor_on_the_abandoned_roots =
                highest_floor_on_the_abandoned_roots.max(root.rollback_floor);
            continue;
        }
        let device = REGION_DEVICES[usize::try_from(root.region).expect("区域号装得进 usize")];
        let newest_on_this_device = newest_valid_on_each_device.entry(device).or_insert(root);
        if (root.key.txg, root.key.instance)
            > (
                newest_on_this_device.key.txg,
                newest_on_this_device.key.instance,
            )
        {
            *newest_on_this_device = root;
        }
    }
    let on_the_roots = newest_valid_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .max()
        .unwrap_or(0)
        .max(highest_floor_on_the_abandoned_roots);
    configuration_floors
        .iter()
        .copied()
        .fold(on_the_roots, u64::max)
}

/// 回退候选集里的根：最新根恒收；别的根不被最新根的实例表判抛弃、txg ≥ F 生效值。按读根环的次序。
fn candidate_root_indexes(
    roots: &[RootOnTheRing],
    rows: &[RowOfTheNewestTable],
    floor: u64,
    newest: usize,
) -> Vec<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(index, root)| {
            *index == newest || (!is_abandoned_by_the_rows(root.key, rows) && root.key.txg >= floor)
        })
        .map(|(index, _)| index)
        .collect()
}

/// 由记录施加出来、根槽从没落盘的那几版（`crates/singlefs-checker/src/walk.rs` 第 12419 行 `versions_applied_only_by_records`
/// 的文档注释 ①–④ 四条同时成立）：① 带提交标记、最新根的实例表里有这个实例的行
/// 且 txg ≤ Ti；② 根环里自证过的根没有一条是这一版；③ txg ≥ F 生效值；④ 根环里有一条没被抛弃的根 txg 比它小。
/// 同一版几条记录时取带末条标志的那一条的新根段，读不到末条就取先到的那一条（`records` 按 (实例, 计数器) 升序）。
fn versions_applied_only_by_records(
    records: &[RecordOnTheRing],
    roots: &[RootOnTheRing],
    rows: &[RowOfTheNewestTable],
    floor: u64,
) -> Vec<VersionOnTheImage> {
    let Some(oldest_valid_root_txg) = roots
        .iter()
        .filter(|root| !is_abandoned_by_the_rows(root.key, rows))
        .map(|root| root.key.txg)
        .min()
    else {
        return Vec::new();
    };
    let mut taken: BTreeMap<VersionKey, (bool, PointerBirth)> = BTreeMap::new();
    for record in records {
        let applied_by_a_recovery = record.is_commit
            && rows.iter().any(|row| {
                row.instance == record.key.instance
                    && record.key.txg <= row.published_checkpoint_txg
            });
        let root_slot_is_readable = roots.iter().any(|root| root.key == record.key);
        let at_or_above_the_floor = record.key.txg >= floor;
        let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.key.txg;
        if !(applied_by_a_recovery
            && !root_slot_is_readable
            && at_or_above_the_floor
            && its_units_are_not_reclaimable_yet)
        {
            continue;
        }
        let takes_this_record = match taken.get(&record.key) {
            None => true,
            Some((from_the_last_record, _)) => {
                !from_the_last_record && record.is_the_last_record_of_the_publish
            }
        };
        if takes_this_record {
            taken.insert(
                record.key,
                (
                    record.is_the_last_record_of_the_publish,
                    record.pointer_birth,
                ),
            );
        }
    }
    taken
        .into_iter()
        .map(|(key, (_, pointer_birth))| VersionOnTheImage { key, pointer_birth })
        .collect()
}

/// 一个洞对 (R⁻ 那一组, R) 与两条臂的分类。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HolePair {
    pub(super) previous_txg: u64,
    pub(super) previous_group: Vec<VersionKey>,
    pub(super) version: VersionOnTheImage,
    pub(super) bridged_today: bool,
    pub(super) bridged_with_the_pointer_birth: bool,
}

/// 候选版本按 txg 升序、同 txg 并成一组，相邻两组 (R⁻, R) 为一对，R 侧逐条各算一对；只交回洞对（R.txg > R⁻.txg + 1）。
fn hole_pairs_of(versions: &[VersionOnTheImage], rows: &[RowOfTheNewestTable]) -> Vec<HolePair> {
    let mut groups: BTreeMap<u64, Vec<VersionOnTheImage>> = BTreeMap::new();
    for version in versions {
        groups.entry(version.key.txg).or_default().push(*version);
    }
    let groups: Vec<(u64, Vec<VersionOnTheImage>)> = groups.into_iter().collect();
    let mut pairs = Vec::new();
    for adjacent in groups.windows(2) {
        let (previous_txg, previous_versions) = (adjacent[0].0, &adjacent[0].1);
        let previous_instances: BTreeSet<u32> = previous_versions
            .iter()
            .map(|version| version.key.instance)
            .collect();
        let previous_group: Vec<VersionKey> = previous_versions
            .iter()
            .map(|version| version.key)
            .collect();
        for version in &adjacent[1].1 {
            let hole = hole_class_today(
                previous_txg,
                &previous_instances,
                version.key.txg,
                version.key.instance,
                rows,
            );
            if hole == HoleClass::NoHole {
                continue;
            }
            pairs.push(HolePair {
                previous_txg,
                previous_group: previous_group.clone(),
                version: *version,
                bridged_today: hole == HoleClass::BridgedByTheRows,
                bridged_with_the_pointer_birth: bridged_with_the_pointer_birth(
                    hole,
                    previous_versions,
                    version,
                ),
            });
        }
    }
    pairs
}

/// 一张镜像上判法读到的东西。
#[derive(Clone, Debug)]
pub(super) struct ImageReadingOfTheJudgements {
    pub(super) newest: VersionKey,
    pub(super) rows: Vec<RowOfTheNewestTable>,
    pub(super) effective_floor: u64,
    pub(super) candidates: Vec<VersionOnTheImage>,
    pub(super) applied_only_by_records: Vec<VersionOnTheImage>,
    pub(super) hole_pairs: Vec<HolePair>,
}

fn pointer_birth_of(pointer: &NodePointer, birth_from_the_bytes: Option<u64>) -> PointerBirth {
    if *pointer == NodePointer::empty_root() {
        return PointerBirth::AllZeroOrUnreadable;
    }
    birth_from_the_bytes.map_or(PointerBirth::AllZeroOrUnreadable, PointerBirth::Readable)
}

/// 只读镜像：择系统配置、读根环（每条根连同槽，槽里 [212, 220) 字节取出生 txg）、读最新根的实例表、算 F 生效值、候选集、扫环认由记录施加出来的版本、
/// 取洞对。系统配置择不出、根环一条根都没有、最新根的实例表读不出时交 `Err`（这一张不进 E1 / E2，单列；登记第十一节 Z6）。
pub(super) fn read_the_image(image: &MemoryPool) -> Result<ImageReadingOfTheJudgements, String> {
    let system_configuration = choose_system_configuration(image)
        .map_err(|failure| format!("系统配置择不出：{failure:?}"))?;
    let immutable = &system_configuration.immutable;
    let ring = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let mut roots = Vec::new();
    for (ring_slot, root) in &ring {
        let ring_slot = *ring_slot;
        let device = immutable.region_devices
            [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
        let offset = slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing);
        let slot_bytes = singlefs_core::recovery::PoolReader::read(
            image,
            device,
            offset,
            usize::try_from(PHYSICAL_BLOCK_BYTES).expect("512 装得进 usize"),
        );
        let birth_in_bytes = slot_bytes
            .as_deref()
            .and_then(pointer_birth_txg_in_root_slot_bytes);
        roots.push(RootOnTheRing {
            region: ring_slot.region,
            key: VersionKey::of_root(root),
            rollback_floor: root.rollback_floor.0,
            pointer_birth: pointer_birth_of(&root.instance_table, birth_in_bytes),
        });
    }
    let newest =
        newest_root_index(&roots).ok_or_else(|| "根环里一条自证过的根都没有".to_string())?;
    let newest_key = roots[newest].key;
    let newest_root = ring[newest].1;
    let rows: Vec<RowOfTheNewestTable> = instance_table_of_root(image, &newest_root)
        .ok_or_else(|| format!("最新根 {} 的实例表读不出", newest_key.text()))?
        .rows
        .iter()
        .map(|row| RowOfTheNewestTable {
            instance: row.instance.0,
            published_checkpoint_txg: row.selected_root_txg.0,
            applied_transaction_high_water_mark: row.applied_transaction_high_water,
        })
        .collect();
    let configuration_floors: Vec<u64> = verified_system_configuration_slots(image)
        .into_iter()
        .flat_map(|(_, slots)| slots.into_iter().map(|(view, _)| view))
        .filter(|view| view.filesystem_identifier == immutable.filesystem_identifier)
        .map(|view| view.rollback_floor)
        .collect();
    let effective_floor = effective_rollback_floor(&roots, &rows, &configuration_floors);
    let candidates: Vec<VersionOnTheImage> =
        candidate_root_indexes(&roots, &rows, effective_floor, newest)
            .into_iter()
            .map(|index| VersionOnTheImage {
                key: roots[index].key,
                pointer_birth: roots[index].pointer_birth,
            })
            .collect();
    let records: Vec<RecordOnTheRing> = scan_journal(image, &system_configuration)
        .into_iter()
        .map(|((instance, _counter), record)| RecordOnTheRing {
            key: VersionKey {
                instance: instance.0,
                txg: record.checkpoint_txg.0,
            },
            is_commit: record.is_commit,
            is_the_last_record_of_the_publish: record.place_in_publish
                == JournalRecordPlaceInPublish::LastRecordOfThePublish,
            pointer_birth: pointer_birth_of(
                &record.new_instance_table,
                Some(record.new_instance_table.head.birth_txg.0),
            ),
        })
        .collect();
    let applied_only_by_records =
        versions_applied_only_by_records(&records, &roots, &rows, effective_floor);
    let mut versions = candidates.clone();
    versions.extend(applied_only_by_records.iter().copied());
    let hole_pairs = hole_pairs_of(&versions, &rows);
    Ok(ImageReadingOfTheJudgements {
        newest: newest_key,
        rows,
        effective_floor,
        candidates,
        applied_only_by_records,
        hole_pairs,
    })
}

#[cfg(test)]
mod tests;
