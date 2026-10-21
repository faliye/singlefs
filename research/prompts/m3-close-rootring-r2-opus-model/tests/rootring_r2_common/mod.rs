//! 攻方模型（m3-close-rootring-r2，云端攻方腿）共用的几样：F2 退回判法的一份照抄（带 C751 这一轮的形态「所选根的实例表读不出时，
//! 把 [见证到的那一版, 所选根) 里读得出、与所选根同实例的根当作不被抛弃」那一个开关），三样读者（恢复走读、只读打开、可写建上一版）
//! 在一条根上读不读得通，与 C755 这一轮的形态「三处用同一种失败判退不退、退到哪一版，落到那一版之后成不成各照今天」的现算。
//! core 不改：照抄的判法关着开关时必须与 core 的函数逐格交回同一条根（每格打一列 `replica_matches_core`）。
#![allow(dead_code, reason = "两份测试文件各用其中一部分")]

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root,
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
    publish_is_abandoned_by_the_instance_table, readable_roots, rebuild_version,
    root_is_abandoned_by_the_instance_table, scan_journal,
    system_configuration_slots_of_one_device_read_once,
    system_configuration_witness_is_undecidable, walk_to_file, PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

pub fn key_of(root: &RootRecord) -> (u32, u64) {
    (root.instance.0, root.checkpoint_txg.0)
}

/// 每块盘上较新的那个系统配置槽（自证过、槽世代号大的那一槽）：盘、偏移、长度（照第一轮攻方模型那一份）。
pub fn newer_system_configuration_slot_of_each_device(
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    image
        .devices
        .iter()
        .map(|(device, sparse_device)| {
            let newer = [0, spacing]
                .into_iter()
                .filter_map(|offset| {
                    SystemConfiguration::parse_slot(
                        &sparse_device.read(DeviceOffsetInBytes(offset), slot_bytes),
                    )
                    .ok()
                    .map(|slot| (slot.quantities.slot_generation, offset))
                })
                .max()
                .map(|(_, offset)| offset)
                .expect("每块盘至少一槽自证过");
            (*device, newer, SYSTEM_CONFIGURATION_SLOT_BYTES)
        })
        .collect()
}

/// F2 退回判法照抄（`recovery.rs` 的 `newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`），
/// 只在「所选根的实例表读不出」那一支多一个开关：`same_instance_kept` 为真时，与所选根同实例的根也当作不被抛弃（C751 这一轮的形态）。
/// 所选那一版就是所选根（这里的调用方都不施加记录），函数体第一句那一判省掉。
pub fn fall_back_to(
    reader: &dyn PoolReader,
    chosen: &RootRecord,
    same_instance_kept: bool,
) -> Option<RootRecord> {
    let configuration = choose_system_configuration(reader).ok()?;
    let records = scan_journal(reader, &configuration);
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slots: Vec<_> = reader
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_of_one_device_read_once(
                reader,
                device,
                spacing,
                &configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    if system_configuration_witness_is_undecidable(&slots) {
        return None;
    }
    let witnessed_counter = slots
        .iter()
        .flat_map(|device| device.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    if witnessed_counter == 0 {
        return None;
    }
    let order = |root: &RootRecord| (root.checkpoint_txg, root.instance);
    let last_record_counter = records
        .values()
        .filter(|record| {
            record.instance == chosen.instance
                && record.checkpoint_txg == chosen.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max();
    let witnessed_record = records
        .values()
        .filter(|record| record.counter == witnessed_counter)
        .map(|record| (record.checkpoint_txg, record.instance))
        .max();
    let is_witnessed = match (last_record_counter, witnessed_record) {
        (Some(counter), _) => counter <= witnessed_counter,
        (None, Some(publish)) => publish >= order(chosen),
        (None, None) => true,
    };
    if is_witnessed {
        return None;
    }
    let witnessed_publish = witnessed_record?;
    let chosen_order = order(chosen);
    let table = instance_table_chain_of_root(reader, chosen)
        .ok()
        .map(|chain| chain.records);
    let roots = readable_roots(
        reader,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    );
    let target = roots
        .iter()
        .copied()
        .filter(|root| {
            let root_order = order(root);
            let kept = match &table {
                Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
                None => {
                    root_order == witnessed_publish
                        || (same_instance_kept && root.instance == chosen.instance)
                }
            };
            witnessed_publish <= root_order && root_order < chosen_order && kept
        })
        .max_by_key(order)?;
    let skips = records.values().any(|record| {
        let record_order = (record.checkpoint_txg, record.instance);
        let abandoned = table.as_ref().is_some_and(|table| {
            publish_is_abandoned_by_the_instance_table(record.instance, record.checkpoint_txg, table)
        });
        record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
            && order(&target) < record_order
            && record_order < chosen_order
            && !abandoned
            && !roots.iter().any(|root| order(root) == record_order)
    });
    if skips {
        return None;
    }
    Some(target)
}

/// core 的函数（今天的 F2）在同一份盘面上交回哪一条：照抄的判法关着开关时要与它相同。
pub fn core_fall_back_to(reader: &dyn PoolReader, chosen: &RootRecord) -> Option<RootRecord> {
    let configuration = choose_system_configuration(reader).ok()?;
    let records = scan_journal(reader, &configuration);
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
        reader,
        &configuration,
        chosen,
        chosen,
        &records,
    )
}

/// 三处：恢复（走读，`walk_to_file`）、只读挂载（打开，`open_pool_for_read`）、可写挂载（建上一版，`rebuild_version` 带那一版的末条记录）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Recovery,
    ReadOnly,
    Writable,
}

pub const PLACES: [Place; 3] = [Place::Recovery, Place::ReadOnly, Place::Writable];

pub fn reader_reads_through(reader: &dyn PoolReader, root: &RootRecord, place: Place) -> bool {
    match place {
        Place::Recovery => walk_to_file(reader, root, &mut 0usize).is_ok(),
        Place::ReadOnly => open_pool_for_read(reader, root).is_ok(),
        Place::Writable => {
            let Ok(configuration) = choose_system_configuration(reader) else {
                return false;
            };
            let own_record = scan_journal(reader, &configuration)
                .values()
                .filter(|record| {
                    record.instance == root.instance
                        && record.checkpoint_txg == root.checkpoint_txg
                        && record.place_in_publish
                            == JournalRecordPlaceInPublish::LastRecordOfThePublish
                })
                .max_by_key(|record| record.counter)
                .cloned();
            rebuild_version(reader, root, own_record).is_ok()
        }
    }
}

/// 「退不退」看哪一种失败。`OwnReader` 是今天：每处看自己那一样读者。其余四种是 C755 这一轮要统一成的「同一种失败」的候选。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    OwnReader,
    Walk,
    Open,
    Rebuild,
    Union,
}

pub const UNIFIED_TRIGGERS: [Trigger; 4] =
    [Trigger::Walk, Trigger::Open, Trigger::Rebuild, Trigger::Union];

pub fn trigger_label(trigger: Trigger) -> &'static str {
    match trigger {
        Trigger::OwnReader => "own",
        Trigger::Walk => "walk",
        Trigger::Open => "open",
        Trigger::Rebuild => "rebuild",
        Trigger::Union => "union",
    }
}

/// 一处落到哪一版、在那一版上自己那一样读者读不读得通。判退：照 `trigger` 在所选根上失败就交 F2 判法；F2 交回 `None` 就留在所选根。
pub fn outcome_at(
    reader: &dyn PoolReader,
    chosen: &RootRecord,
    trigger: Trigger,
    same_instance_kept: bool,
    place: Place,
) -> ((u32, u64), bool) {
    let fails = match trigger {
        Trigger::OwnReader => !reader_reads_through(reader, chosen, place),
        Trigger::Walk => !reader_reads_through(reader, chosen, Place::Recovery),
        Trigger::Open => !reader_reads_through(reader, chosen, Place::ReadOnly),
        Trigger::Rebuild => !reader_reads_through(reader, chosen, Place::Writable),
        Trigger::Union => PLACES
            .iter()
            .any(|each| !reader_reads_through(reader, chosen, *each)),
    };
    if !fails {
        return (key_of(chosen), reader_reads_through(reader, chosen, place));
    }
    match fall_back_to(reader, chosen, same_instance_kept) {
        Some(target) => (key_of(&target), reader_reads_through(reader, &target, place)),
        None => (key_of(chosen), reader_reads_through(reader, chosen, place)),
    }
}

pub fn describe_outcome(outcome: ((u32, u64), bool)) -> String {
    let ((instance, txg), ok) = outcome;
    if ok {
        format!("Ok({instance},{txg})")
    } else {
        format!("Fail@({instance},{txg})")
    }
}

/// 三处在 `trigger` 与开关下的结局，空格隔开：`rec=… ro=… rw=…`。
pub fn three_places(
    reader: &dyn PoolReader,
    chosen: &RootRecord,
    trigger: Trigger,
    same_instance_kept: bool,
) -> [((u32, u64), bool); 3] {
    PLACES.map(|place| outcome_at(reader, chosen, trigger, same_instance_kept, place))
}

pub fn describe_three(outcomes: &[((u32, u64), bool); 3]) -> String {
    format!(
        "rec={} ro={} rw={}",
        describe_outcome(outcomes[0]),
        describe_outcome(outcomes[1]),
        describe_outcome(outcomes[2])
    )
}
