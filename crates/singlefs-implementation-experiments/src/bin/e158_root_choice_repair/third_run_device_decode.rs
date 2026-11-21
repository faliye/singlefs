//! 第 3 次跑装置自己的解码、重放与配置见证，按装置的判法判一次挂载。

use super::*;

/// 装置解出的一份镜像：自证根的键 (实例代号, txg)、读错的根槽个数、全部自证记录（按 (实例代号, 计数器)）、
/// 每块盘两槽里自证过的系统配置 (盘, 世代号, tail, 实例代号)。
pub(super) struct DeviceDecodedImage {
    pub(super) root_keys: BTreeSet<TimelineRoot>,
    pub(super) unreadable_root_slots: u64,
    pub(super) records: BTreeMap<(u32, u64), singlefs_checker::JournalRecordView>,
    pub(super) system_configuration_slots: Vec<(u32, u64, u64, u32)>,
}

pub(super) fn device_decode(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
) -> DeviceDecodedImage {
    let root_keys = checker_image::valid_roots(reader, geometry)
        .into_iter()
        .map(|(_, _, view)| (view.instance, view.checkpoint_txg))
        .collect();
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let unreadable_root_slots = checker_image::root_slot_positions(geometry)
        .into_iter()
        .filter(|(_, _, device, offset)| reader.read(*device, *offset, root_slot_bytes).is_none())
        .count() as u64;
    let expected_filesystem_identifier = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("fsid 前 8 字节"),
    );
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let ring_start = geometry.journal_ring_start_slot * SLOT_BYTES_LOCAL;
    let mut records = BTreeMap::new();
    for device in reader.devices() {
        for index in 0..ring_slots {
            let Some(bytes) = reader.read(
                device,
                ring_start + index * JOURNAL_RECORD_BYTES,
                record_bytes,
            ) else {
                continue;
            };
            if let Ok(view) =
                singlefs_checker::check_journal_record(&bytes, expected_filesystem_identifier)
            {
                records.entry((view.instance, view.counter)).or_insert(view);
            }
        }
    }
    let system_configuration_slots = checker_image::verified_system_configuration_slots(reader)
        .into_iter()
        .flat_map(|(device, slots)| {
            slots.into_iter().map(move |(view, _)| {
                (
                    device,
                    view.slot_generation,
                    view.journal_tail,
                    view.journal_instance,
                )
            })
        })
        .collect();
    DeviceDecodedImage {
        root_keys,
        unreadable_root_slots,
        records,
        system_configuration_slots,
    }
}

/// 装置自己择根：自证根里 (txg, 实例代号) 最大的那一条。
fn device_chosen_root(decoded: &DeviceDecodedImage) -> Option<TimelineRoot> {
    decoded
        .root_keys
        .iter()
        .copied()
        .max_by_key(|(instance, txg)| (*txg, *instance))
}

pub(super) fn record_ends_its_publish_by_the_device(
    record: &singlefs_checker::JournalRecordView,
) -> bool {
    record.record_flags_byte & 1 == 1
}

fn unit_bytes_of_class_by_the_device(unit_class: u8) -> Option<usize> {
    match unit_class {
        UNIT_CLASS_DATA_LOCAL | UNIT_CLASS_PACKED_LOCAL => Some(DATA_UNIT_BYTES_LOCAL),
        UNIT_CLASS_INDEX_NODE_LOCAL => Some(NODE_BYTES_LOCAL),
        _ => None,
    }
}

/// 装置自己的在飞记录数上限：环槽数 ÷ F，两次都取整数部分（D23（journal 的角色与格式） 已定项 18）。照「装置不与实现共用代码」
/// 自写一份，不调 core 的 `system_configuration::journal_in_flight_record_limit`，只取格式常量模块的两个标量；
/// 两份在环长上的交叉断言在 `journal_in_flight_record_limit_cross_check_tests`。
fn journal_in_flight_record_limit(ring_bytes: u64) -> u64 {
    let ring_record_slots = ring_bytes / JOURNAL_RECORD_BYTES;
    ring_record_slots / singlefs_format::JOURNAL_SAFETY_FACTOR
}

/// 装置按 D23（journal 的角色与格式） 已定项 14 五条口径从所选根重放：前缀不跨实例、链首锚在所选根那次发布带末条标志的那一条
/// （读不出就要求第一条 txg = 根 + 1 且序号 1）、计数器连号、发布边界按末条标志、序号连续、提交标记、点名单元两份都读得出且
/// 整单元 CRC-32C 对；末条到了才整体施加。交回 E 的键；所选根那次发布带末条标志的记录多于一条交 Err（crates 那一格也报错）。
fn device_replay(
    reader: &dyn ImageReader,
    decoded: &DeviceDecodedImage,
    root: TimelineRoot,
    ring_bytes: u64,
) -> Result<TimelineRoot, String> {
    let (instance, txg) = root;
    let anchors: Vec<u64> = decoded
        .records
        .values()
        .filter(|record| {
            record.instance == instance
                && record.checkpoint_txg == txg
                && record_ends_its_publish_by_the_device(record)
        })
        .map(|record| record.counter)
        .collect();
    if anchors.len() > 1 {
        return Err(format!(
            "所选根 {root:?} 那次发布带末条标志的记录 {} 条",
            anchors.len()
        ));
    }
    let mut above: Vec<&singlefs_checker::JournalRecordView> = decoded
        .records
        .values()
        .filter(|record| record.instance == instance && record.checkpoint_txg > txg)
        .collect();
    above.sort_by_key(|record| record.counter);
    let limit = usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    let mut expected_next = anchors.first().map(|counter| counter + 1);
    let mut open: Vec<&singlefs_checker::JournalRecordView> = Vec::new();
    let mut effective = root;
    for record in above.into_iter().take(limit) {
        match expected_next {
            Some(expected) => {
                if record.counter != expected {
                    break;
                }
            }
            None => {
                if record.checkpoint_txg != txg + 1 || record.ordinal_within_publish != 1 {
                    break;
                }
            }
        }
        if open.is_empty() && record.ordinal_within_publish != 1 {
            break;
        }
        if let Some(previous) = open.last() {
            if record.checkpoint_txg != previous.checkpoint_txg
                || record.ordinal_within_publish != previous.ordinal_within_publish + 1
                || (!previous.is_commit && record.transaction != previous.transaction)
            {
                break;
            }
        }
        expected_next = Some(record.counter + 1);
        let ends = record_ends_its_publish_by_the_device(record);
        if !record.is_commit && ends {
            break;
        }
        if ends
            && decoded
                .records
                .range((record.instance, record.counter + 1)..=(record.instance, u64::MAX))
                .map(|(_, later)| later)
                .take_while(|later| later.checkpoint_txg <= record.checkpoint_txg)
                .any(|later| later.checkpoint_txg == record.checkpoint_txg)
        {
            break;
        }
        let all_verified = record.named.iter().all(|named| {
            let Some(unit_bytes) = unit_bytes_of_class_by_the_device(named.unit_class) else {
                return false;
            };
            named.locations.iter().all(|(device, slot, checksum)| {
                reader
                    .read(*device, slot * SLOT_BYTES_LOCAL, unit_bytes)
                    .is_some_and(|bytes| {
                        singlefs_checker::crc32_castagnoli_table(&bytes) == *checksum
                    })
            })
        });
        if !all_verified {
            break;
        }
        open.push(record);
        if !ends {
            continue;
        }
        open.clear();
        effective = (record.instance, record.checkpoint_txg);
    }
    Ok(effective)
}

/// 装置按登记 5.1 第一行算 N-配置：(N, c_见证, c_E, 判不出)。
fn device_configuration_witness(
    decoded: &DeviceDecodedImage,
    effective: TimelineRoot,
) -> (bool, u64, Option<u64>, bool) {
    let witnessed_tail = decoded
        .system_configuration_slots
        .iter()
        .map(|(_, _, tail, _)| *tail)
        .max()
        .unwrap_or(0);
    let selected_version_last_counter = decoded
        .records
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == effective
                && record_ends_its_publish_by_the_device(record)
        })
        .map(|record| record.counter)
        .max();
    if witnessed_tail == 0 {
        return (false, 0, selected_version_last_counter, false);
    }
    if let Some(counter) = selected_version_last_counter {
        return (
            witnessed_tail > counter,
            witnessed_tail,
            selected_version_last_counter,
            false,
        );
    }
    match decoded
        .records
        .values()
        .find(|record| record.counter == witnessed_tail)
    {
        Some(witness) => (
            (witness.instance, witness.checkpoint_txg) > effective,
            witnessed_tail,
            None,
            false,
        ),
        None => (true, witnessed_tail, None, true),
    }
}

/// 装置对一次被测挂载（带故障的第一遍读）与它的孪生镜像（不装故障）算的东西。
pub(super) struct DeviceJudgement {
    pub(super) chosen: Option<TimelineRoot>,
    pub(super) effective: Option<TimelineRoot>,
    pub(super) configuration_newer: bool,
    pub(super) witnessed_tail: u64,
    pub(super) selected_version_last_counter: Option<u64>,
    pub(super) undecidable: bool,
    pub(super) unreadable_root_slots: u64,
    /// N_真：孪生镜像上有自证根的键 (实例代号, txg) 大于 E_故障 的。
    pub(super) truth_newer: bool,
    /// 同一句按择根的次序 (txg, 实例代号) 比，与上面那个不同的格另记。
    pub(super) truth_orders_disagree: bool,
    /// 孪生镜像上重放能走得比 E_故障 远、而没有键更大的根槽（标签「只由记录走得更远」）。
    pub(super) truth_by_records_only: bool,
    pub(super) error: Option<String>,
}

pub(super) fn device_judgement(
    twin: &MemoryPool,
    geometry: &PoolGeometry,
    faults: &[ThirdRunFault],
) -> DeviceJudgement {
    let reader = FirstPassReader { pool: twin, faults };
    let decoded = device_decode(&reader, geometry);
    let chosen = device_chosen_root(&decoded);
    let mut error = None;
    let effective = chosen.and_then(|root| {
        device_replay(&reader, &decoded, root, geometry.journal_ring_bytes)
            .map_err(|message| error = Some(message))
            .ok()
    });
    let (configuration_newer, witnessed_tail, selected_version_last_counter, undecidable) =
        match effective {
            Some(effective) => device_configuration_witness(&decoded, effective),
            None => (false, 0, None, false),
        };
    let twin_decoded = device_decode(twin, geometry);
    let (truth_newer, truth_orders_disagree, truth_by_records_only) = match effective {
        Some(effective) => {
            let by_key = twin_decoded.root_keys.iter().any(|key| *key > effective);
            let by_choice_order = twin_decoded
                .root_keys
                .iter()
                .any(|(instance, txg)| (*txg, *instance) > (effective.1, effective.0));
            let twin_effective = device_chosen_root(&twin_decoded).and_then(|root| {
                device_replay(twin, &twin_decoded, root, geometry.journal_ring_bytes).ok()
            });
            let by_records_only = !by_key
                && twin_effective
                    .is_some_and(|twin_key| (twin_key.1, twin_key.0) > (effective.1, effective.0));
            (by_key, by_key != by_choice_order, by_records_only)
        }
        None => (false, false, false),
    };
    DeviceJudgement {
        chosen,
        effective,
        configuration_newer,
        witnessed_tail,
        selected_version_last_counter,
        undecidable,
        unreadable_root_slots: decoded.unreadable_root_slots,
        truth_newer,
        truth_orders_disagree,
        truth_by_records_only,
        error,
    }
}

/// core 与装置两份在飞记录数上限在环长上的交叉断言。
///
/// 取值域：环长是系统配置里 8 字节的「环长度（字节）」字段，恢复照读盘上的值，所以两份要在整个 u64 上相等；mkfs 收的环长
/// 在 [3 × 4096, 设备容量 ÷ 4]（D23（journal 的角色与格式） 已定项 19 ③），容量被 6 字节槽号 × 16 KiB 槽
/// （D19（块指针的结构与宽度预算） 已定项 4）顶在 2^62 字节，所以按条款 mkfs 收得下的环长不过 2^60。
/// 2^64 个值扫不遍，只取枚举点：① 0 到 2^20 逐个取（两份只除以 4096 与 3，这一段把模 12288 的每个余数走了 85 遍）；
/// ② 每个 2 的幂与它 ±1，以及它以下最近的 12288 的倍数与那个倍数附近的 4096 边界（每个数量级上的截断、溢出、取整方向）；
/// ③ u64 顶端；④ 在飞上限刚越过 u32 的那一处（写系统配置时它要装进 4 字节）；⑤ 默认环 768 MiB。
/// 两份一起歪（改共享的两个标量）时互比看不出，靠 `the_in_flight_record_limit_is_pinned_at_values_worked_out_from_the_clauses` 钉的绝对值。
#[cfg(test)]
mod journal_in_flight_record_limit_cross_check_tests;

#[cfg(test)]
mod third_run_tests;
