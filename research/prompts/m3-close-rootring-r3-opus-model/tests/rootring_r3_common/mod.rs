//! 攻方模型（m3-close-rootring-r3，云端攻方腿，攻补丁前的 `crates/`）共用的几样。照抄第二轮攻方模型
//! （`research/prompts/m3-close-rootring-r2-opus-model/tests/rootring_r2_common/mod.rs`）的 F2 判法，「所选根的实例表读不出」那一支
//! 换成三取一的规则：今天（只认恰好等于见证到的那一版）、第二轮的形态（同实例的根一律当作不被抛弃）、第三轮被攻的收窄形态
//! （C751：只在见证到的那一版与所选根同实例时，才把同实例的根当作不被抛弃）。core 不改：取「今天」时每格与 core 的函数比。
#![allow(dead_code, reason = "几份测试文件各用其中一部分")]

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

/// 所选根的实例表读不出时 F2 认哪些根。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleWhenTheTableIsUnreadable {
    /// 今天：只认恰好等于见证到的那一版。
    ExactlyTheWitnessedVersion,
    /// 第二轮被打中的形态：与所选根同实例的根一律当作不被抛弃。
    SameInstanceAlways,
    /// 第三轮被攻的收窄形态：见证到的那一版与所选根同实例时，同实例的根当作不被抛弃；不同实例时照今天。
    SameInstanceOnlyWhenTheWitnessIsOfTheChosenInstance,
}

pub const RULES: [(RuleWhenTheTableIsUnreadable, &str); 3] = [
    (RuleWhenTheTableIsUnreadable::ExactlyTheWitnessedVersion, "exact"),
    (RuleWhenTheTableIsUnreadable::SameInstanceAlways, "same"),
    (RuleWhenTheTableIsUnreadable::SameInstanceOnlyWhenTheWitnessIsOfTheChosenInstance, "narrow"),
];

/// 每块盘上较新的那个系统配置槽（自证过、槽世代号大的那一槽）：盘、偏移、长度。
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

/// 见证到的那一版（计数器 = c_见证 的那条记录的 (txg, 实例)）；判不出或没有交 `None`。只为打印用。
pub fn witnessed_publish_of(reader: &dyn PoolReader) -> Option<(u64, u32)> {
    let configuration = choose_system_configuration(reader).ok()?;
    let records = scan_journal(reader, &configuration);
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let counter = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            system_configuration_slots_of_one_device_read_once(
                reader,
                device,
                spacing,
                &configuration.immutable.filesystem_identifier,
            )
            .self_verified
            .into_iter()
            .map(|slot| slot.quantities.journal_tail)
            .collect::<Vec<_>>()
        })
        .max()?;
    records
        .values()
        .filter(|record| record.counter == counter)
        .map(|record| (record.checkpoint_txg.0, record.instance.0))
        .max()
}

/// F2 退回判法照抄（`recovery.rs` 的 `newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`），
/// 只在「所选根的实例表读不出」那一支按 `rule` 取。所选那一版就是所选根（调用方都不施加记录），函数体第一句那一判省掉。
pub fn fall_back_to(
    reader: &dyn PoolReader,
    chosen: &RootRecord,
    rule: RuleWhenTheTableIsUnreadable,
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
    let witness_is_of_the_chosen_instance = witnessed_publish.1 == chosen.instance;
    let target = roots
        .iter()
        .copied()
        .filter(|root| {
            let root_order = order(root);
            let kept = match &table {
                Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
                None => {
                    root_order == witnessed_publish
                        || (root.instance == chosen.instance
                            && match rule {
                                RuleWhenTheTableIsUnreadable::ExactlyTheWitnessedVersion => false,
                                RuleWhenTheTableIsUnreadable::SameInstanceAlways => true,
                                RuleWhenTheTableIsUnreadable::SameInstanceOnlyWhenTheWitnessIsOfTheChosenInstance => {
                                    witness_is_of_the_chosen_instance
                                }
                            })
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

/// core 的函数（今天的 F2）在同一份盘面上交回哪一条：照抄的判法取「今天」时要与它相同。
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

/// 一处在 `rule` 下落到哪一版、在那一版上自己那一样读者读不读得通（触发照今天：每处看自己那一样读者）。
pub fn outcome_at(
    reader: &dyn PoolReader,
    chosen: &RootRecord,
    rule: RuleWhenTheTableIsUnreadable,
    place: Place,
) -> ((u32, u64), bool) {
    if reader_reads_through(reader, chosen, place) {
        return (key_of(chosen), true);
    }
    match fall_back_to(reader, chosen, rule) {
        Some(target) => (key_of(&target), reader_reads_through(reader, &target, place)),
        None => (key_of(chosen), false),
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

pub fn describe_three_under(reader: &dyn PoolReader, chosen: &RootRecord, rule: RuleWhenTheTableIsUnreadable) -> String {
    let outcomes = PLACES.map(|place| outcome_at(reader, chosen, rule, place));
    format!(
        "rec={} ro={} rw={}",
        describe_outcome(outcomes[0]),
        describe_outcome(outcomes[1]),
        describe_outcome(outcomes[2])
    )
}
