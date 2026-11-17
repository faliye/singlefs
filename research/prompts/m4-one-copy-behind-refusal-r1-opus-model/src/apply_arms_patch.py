#!/usr/bin/env python3
"""把候选臂接进仓副本的 crates/singlefs-core/src/mount.rs。用法：apply_arms_patch.py <副本根>。

臂由 ONE_COPY_ARM 环境变量或模型测试直接存 ONE_COPY_MODEL_ARM 选，不设就是今天的实现：
  today          今天的实现
  c1             点名窗口，按计数器取 (tail.counter, 所选.counter]，不看实例代号
  c2             紧邻一版的点名：同实例且 tail.counter + 1 == 所选.counter 才放宽
  c1-same        点名窗口，tail 与所选实例代号不同就退回今天的比法
  c3             攻方腿自提（零轮）：所选那一版是重放施加得来的，且这块盘 tail 不落后于择到的根那一版，就不算落后
每个替换点要恰好命中一次，否则退 1、一个字节不写。
"""
import pathlib
import sys

ARMS = '''
/// 候选臂的选择（模型专用，只在这份副本里）：0 今天、1 点名窗口、2 紧邻一版、3 点名窗口同实例、4 重放施加不算落后；255 未定，读环境变量。
pub static ONE_COPY_MODEL_ARM: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(255);

fn one_copy_model_arm() -> u8 {
    use std::sync::atomic::Ordering;
    let arm = ONE_COPY_MODEL_ARM.load(Ordering::SeqCst);
    if arm != 255 {
        return arm;
    }
    let arm = match std::env::var("ONE_COPY_ARM").as_deref() {
        Ok("c1") => 1,
        Ok("c2") => 2,
        Ok("c1-same") => 3,
        Ok("c3") => 4,
        _ => 0,
    };
    ONE_COPY_MODEL_ARM.store(arm, Ordering::SeqCst);
    arm
}

thread_local! {
    /// 这次挂载择到的根那一版的 jsn；只在所选那一版是重放施加得来的（所选 != 择到的根）时有值。
    static ONE_COPY_MODEL_CHOSEN_ROOT_POSITION: Cell<Option<JournalSequenceNumber>> = const { Cell::new(None) };
}

/// (tail, 所选] 那几条记录点名的、落在 `device` 上的槽；不适用（实例不同、不紧挨）或窗口里有一个计数器在哪块盘都读不出，交回 `None`，调用方照今天比全部单元。
/// 同一计数器两块盘上各有一条不同实例的记录时取并集。
fn one_copy_model_slots_named_by_the_window<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    tail: JournalSequenceNumber,
    position: JournalSequenceNumber,
    arm: u8,
) -> Option<Vec<SlotNumber>> {
    let same_instance = tail.instance_generation == position.instance_generation;
    match arm {
        1 => {}
        2 => {
            if !same_instance || tail.counter + 1 != position.counter {
                return None;
            }
        }
        3 => {
            if !same_instance {
                return None;
            }
        }
        _ => return None,
    }
    let record_bytes =
        usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");
    let filesystem_identifier = crate::unit::unit_filesystem_identifier(
        &parameters_of_the_pool.filesystem_identifier,
    );
    let mut slots = Vec::new();
    let mut counter = tail.counter + 1;
    while counter <= position.counter {
        let offset = crate::journal::record_offset(
            counter,
            parameters_of_the_pool.geometry.journal_ring_bytes,
        );
        let mut found = false;
        for reading_device in reader.device_identities() {
            let Some(bytes) = reader.read(reading_device, offset, record_bytes) else {
                continue;
            };
            let Some(record) = crate::journal::JournalRecord::parse(&bytes, filesystem_identifier)
            else {
                continue;
            };
            if record.counter != counter {
                continue;
            }
            found = true;
            for named in &record.named {
                for location in &named.locations {
                    if location.device == device {
                        slots.push(location.slot);
                    }
                }
            }
        }
        if !found {
            return None;
        }
        counter += 1;
    }
    Some(slots)
}

'''

REPLACEMENTS = [
    (
        "/// 落后支要判的那一版（[`what_a_device_behind_the_version_lacks`]）",
        ARMS + "/// 落后支要判的那一版（[`what_a_device_behind_the_version_lacks`]）",
    ),
    (
        "    let units_missing: Vec<(TransactionUnit, SlotNumber)> = version\n",
        "    let mut units_missing: Vec<(TransactionUnit, SlotNumber)> = version\n",
    ),
    (
        "    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))\n",
        "    if let Some(slots_named_by_the_window) = one_copy_model_slots_named_by_the_window(\n"
        "        reader,\n"
        "        parameters_of_the_pool,\n"
        "        device,\n"
        "        newest_system_configuration_journal_tail,\n"
        "        version_journal_position,\n"
        "        one_copy_model_arm(),\n"
        "    ) {\n"
        "        units_missing.retain(|(_, slot)| slots_named_by_the_window.contains(slot));\n"
        "    }\n"
        "    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))\n",
    ),
    (
        "        let Some((newest_system_configuration_journal_tail, lacking)) =\n"
        "            what_a_device_behind_the_version_lacks(\n",
        "        if one_copy_model_arm() == 4 {\n"
        "            let tail = JournalSequenceNumber {\n"
        "                instance_generation: newest_self_verified.quantities.journal_instance,\n"
        "                counter: newest_self_verified.quantities.journal_tail,\n"
        "            };\n"
        "            if ONE_COPY_MODEL_CHOSEN_ROOT_POSITION\n"
        "                .with(Cell::get)\n"
        "                .is_some_and(|chosen_root_position| tail >= chosen_root_position)\n"
        "            {\n"
        "                continue;\n"
        "            }\n"
        "        }\n"
        "        let Some((newest_system_configuration_journal_tail, lacking)) =\n"
        "            what_a_device_behind_the_version_lacks(\n",
    ),
    (
        "    let unit_reads_with_one_copy_failing_while_building_on_the_version =\n",
        "    ONE_COPY_MODEL_CHOSEN_ROOT_POSITION.with(|position| {\n"
        "        position.set(\n"
        "            (chosen_root.instance != effective_root.instance\n"
        "                || chosen_root.checkpoint_txg != effective_root.checkpoint_txg)\n"
        "                .then(|| {\n"
        "                    journal_position_of_the_selected_version(\n"
        "                        &chosen_root,\n"
        "                        record_standing_for_the_version(&records, &chosen_root).as_ref(),\n"
        "                    )\n"
        "                }),\n"
        "        );\n"
        "    });\n"
        "    let unit_reads_with_one_copy_failing_while_building_on_the_version =\n",
    ),
]


def main():
    target = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/mount.rs"
    text = target.read_text()
    for old, new in REPLACEMENTS:
        hits = text.count(old)
        if hits != 1:
            print(f"替换点命中 {hits} 次，要恰好 1 次：{old[:60]!r}")
            return 1
        text = text.replace(old, new)
    target.write_text(text)
    print(f"APPLIED {len(REPLACEMENTS)} replacements to {target}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
