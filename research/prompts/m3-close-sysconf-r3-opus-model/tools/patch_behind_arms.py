#!/usr/bin/env python3
"""在仓副本的 mount.rs 里给「落后」那一判加三条臂（环境变量 SINGLEFS_PROBE_BEHIND 选）：
tail（默认，今天的判法）、record_both（改法丁照字面：两半都按这块盘上有没有那一版末条记录判落后）、
record_units（改法丁只换缺单元那一半，缺根那一半照旧按系统配置 tail 判）。用法：patch_behind_arms.py <副本的 mount.rs>"""
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old_units = """    if newest_system_configuration_journal_tail >= version_journal_position {
        return None;
    }
    let units_missing: Vec<(TransactionUnit, SlotNumber)> = units"""
new_units = """    if !probe_device_is_behind(
        reader,
        device,
        newest_self_verified,
        version_journal_position,
        ProbeHalf::Units,
    ) {
        return None;
    }
    let units_missing: Vec<(TransactionUnit, SlotNumber)> = units"""
old_root = """    let is_behind_the_version = newest_system_configuration_journal_tail < version_journal_position;"""
new_root = """    let is_behind_the_version = probe_device_is_behind(
        reader,
        device,
        newest_self_verified,
        version_journal_position,
        ProbeHalf::Root,
    );"""
helper = '''
/// 攻方原型（m3-close-sysconf-r1）：落后那一判的三条臂。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProbeHalf {
    Units,
    Root,
}

fn probe_device_holds_the_versions_last_record<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    newest_self_verified: &crate::system_configuration::SystemConfiguration,
    version_journal_position: JournalSequenceNumber,
) -> bool {
    if version_journal_position.counter == 0 {
        return true;
    }
    let record_bytes = usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("4096");
    let offset = crate::journal::record_offset(
        version_journal_position.counter,
        newest_self_verified.immutable.sizes.journal_ring_bytes,
    );
    let expected = crate::unit::unit_filesystem_identifier(
        &newest_self_verified.immutable.filesystem_identifier,
    );
    reader
        .read(device, offset, record_bytes)
        .and_then(|bytes| crate::journal::JournalRecord::parse(&bytes, expected))
        .is_some_and(|record| {
            record.instance == version_journal_position.instance_generation
                && record.counter == version_journal_position.counter
        })
}

fn probe_device_is_behind<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    newest_self_verified: &crate::system_configuration::SystemConfiguration,
    version_journal_position: JournalSequenceNumber,
    half: ProbeHalf,
) -> bool {
    let tail = JournalSequenceNumber {
        instance_generation: newest_self_verified.quantities.journal_instance,
        counter: newest_self_verified.quantities.journal_tail,
    };
    let by_tail = tail < version_journal_position;
    let by_record = || {
        !probe_device_holds_the_versions_last_record(
            reader,
            device,
            newest_self_verified,
            version_journal_position,
        )
    };
    match std::env::var("SINGLEFS_PROBE_BEHIND").as_deref() {
        Ok("record_both") => by_record(),
        Ok("record_units") => match half {
            ProbeHalf::Units => by_record(),
            ProbeHalf::Root => by_tail,
        },
        // 攻方原型（m3-close-sysconf-r3）：乙' 的放宽读法，tail 不落后或本盘有那一版末条记录，都不算落后（缺单元那一半），根那一半不动。
        Ok("record_or_tail_units") => match half {
            ProbeHalf::Units => by_tail && by_record(),
            ProbeHalf::Root => by_tail,
        },
        _ => by_tail,
    }
}
'''
anchor = "/// 落后支缺单元那一半（[`what_a_device_behind_the_version_lacks`] 先判它）"
for old, new in ((old_units, new_units), (old_root, new_root)):
    assert text.count(old) == 1, old[:60]
    text = text.replace(old, new)
assert text.count(anchor) == 1
text = text.replace(anchor, helper.lstrip("\n") + "\n" + anchor)
open(path, "w", encoding="utf-8").write(text)
print("patched", path)
