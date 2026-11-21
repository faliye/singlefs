//! 差分集 D1–D4（五份胶水各与 `scan_journal` 逐键逐值比）与 `--checks`（本地常量与 `crates/` 回比、锚点、`size_of`）。
use super::*;

// ============================================================================================
// 本地常量回比、几何锚点（登记第七节 A1–A6）、常驻页峰值的推算（登记修订三第七节 A7）。
// ============================================================================================

/// 本地常量与 `crates/` 的同名常量逐个回比（登记第五节）；每个进程开头调一次，单测同样调。
pub(super) fn constants_against_the_implementation() {
    assert_eq!(
        JOURNAL_RING_DEFAULT_BYTES,
        singlefs_format::JOURNAL_RING_DEFAULT_BYTES
    );
    assert_eq!(JOURNAL_RECORD_BYTES, singlefs_format::JOURNAL_RECORD_BYTES);
    assert_eq!(
        JOURNAL_RING_START_SLOT,
        singlefs_format::JOURNAL_RING_START_SLOT
    );
    assert_eq!(SLOT_BYTES, singlefs_format::SLOT_BYTES);
    assert_eq!(
        JOURNAL_NAMED_ENTRIES_PER_RECORD,
        singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD
    );
    assert_eq!(
        TEST_IMAGE_DEFAULT_BYTES,
        singlefs_format::TEST_IMAGE_DEFAULT_BYTES
    );
    // C1：默认环每盘 196 608 个记录槽，除得尽（A1）。
    assert_eq!(
        RING_SLOTS,
        JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES
    );
    assert_eq!(JOURNAL_RING_DEFAULT_BYTES % JOURNAL_RECORD_BYTES, 0);
}

// ============================================================================================
// 差分集 D（登记第九节）：小环（64 个槽），盘 4 GiB，不给提示；五份胶水（原三份加登记修订一的两条读缓存臂）各与 `scan_journal` 逐键逐值比。
// ============================================================================================

pub(super) const DIFFERENTIAL_RING_SLOTS: u64 = 64;
const DIFFERENTIAL_NAMED_ENTRIES: usize = 3;

fn differential_record(counter: u64, transaction: u64) -> JournalRecord {
    journal_record_for(
        PRODUCTION_INSTANCE,
        counter,
        transaction,
        DIFFERENTIAL_NAMED_ENTRIES,
    )
}

/// 四例的盘数与「某块盘某个槽放哪条记录」。
pub(super) fn differential_case(case: &str) -> Option<(u32, SlotSource)> {
    match case {
        // D1：2 盘，第一盘 0–31 槽有记录，第二盘 16–63 槽有记录，重叠处字节相同。
        "D1" => Some((
            2,
            slot_source_of(|device, slot| match device {
                0 if slot < 32 => Some(differential_record(slot + 1, slot + 1)),
                1 if slot >= 16 => Some(differential_record(slot + 1, slot + 1)),
                _ => None,
            }),
        )),
        // D2：2 盘，第 4 槽两盘都是（实例 1，计数器 5），事务号不同；`scan_journal` 留第一盘那一份。
        "D2" => Some((
            2,
            slot_source_of(|device, slot| {
                (slot == 4).then(|| differential_record(5, 100 + 100 * u64::from(device)))
            }),
        )),
        // D3：3 盘，第一盘全 0、第二盘半满、第三盘满。
        "D3" => Some((
            3,
            slot_source_of(|device, slot| match device {
                1 if slot < 32 => Some(differential_record(slot + 1, slot + 1)),
                2 => Some(differential_record(slot + 1, slot + 1)),
                _ => None,
            }),
        )),
        // D4：1 盘，计数器 3 的记录在第 2 槽与第 40 槽各有一份、内容不同，按偏移次序留前一份。
        "D4" => Some((
            1,
            slot_source_of(|_device, slot| match slot {
                2 => Some(differential_record(3, 300)),
                40 => Some(differential_record(3, 400)),
                _ => None,
            }),
        )),
        _ => None,
    }
}

const DIFFERENTIAL_CASES: [&str; 4] = ["D1", "D2", "D3", "D4"];
const GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL: [Arm; 5] = [
    Arm::BaselinePc,
    Arm::Split,
    Arm::SplitPc,
    Arm::ReadStageCache,
    Arm::ReadStageCacheKeepsZerosPc,
];

/// 一例一臂：交回的表与 `scan_journal` 在同一个池上交回的是否逐键逐值相同。
fn differential_result(case: &str, arm: Arm) -> (bool, u64) {
    let (devices, source) = differential_case(case).expect("差分例名是 D1–D4");
    let system_configuration =
        system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
    let pool = GeneratedPool::new(
        devices,
        TEST_IMAGE_DEFAULT_BYTES,
        DIFFERENTIAL_RING_SLOTS,
        source,
    );
    let reference = scan_journal(&pool, &system_configuration);
    let (candidate, _) = run_the_arm_over_the_pool(arm, &pool, &system_configuration);
    (
        tables_are_equal(&reference, &candidate),
        u64::try_from(reference.len()).expect("表长装得进 u64"),
    )
}

/// 差分集 D 的结果行；返回是否全等。
pub(super) fn run_the_differential_set() -> (Vec<String>, bool) {
    let mut lines = Vec::new();
    let mut all_equal = true;
    for case in DIFFERENTIAL_CASES {
        for arm in GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL {
            let (equal, reference_records) = differential_result(case, arm);
            all_equal &= equal;
            lines.push(format!(
                "E7RESULT name=e197_diff case={case} arm={} equal={equal} reference_records={reference_records}",
                arm.name()
            ));
        }
    }
    (lines, all_equal)
}

/// 生产格上的一臂与 `scan_journal` 在同一个池上逐键逐值比（登记第十一节 S2「任一格」）：先跑臂、留着它交回的表，再跑 `scan_journal`。
/// 不量峰值，所以放在量窗之外的专用进程里（`--compare`）；交 (是否全等, 基准表的键数)。
pub(super) fn compare_arm_with_scan_journal(
    specification: CellSpecification,
    ring_slots: u64,
) -> (bool, u64) {
    let filled_slots = specification.fill.filled_slots_per_device(ring_slots);
    let system_configuration =
        system_configuration_for(ring_slots * JOURNAL_RECORD_BYTES, specification.devices);
    let pool = GeneratedPool::new(
        specification.devices,
        TEST_IMAGE_DEFAULT_BYTES,
        ring_slots,
        production_slot_source(filled_slots, specification.named),
    );
    let (candidate, _) = run_the_arm_over_the_pool(specification.arm, &pool, &system_configuration);
    let reference = scan_journal(&pool, &system_configuration);
    (
        tables_are_equal(&reference, &candidate),
        u64::try_from(reference.len()).expect("表长装得进 u64"),
    )
}

/// `--checks`：本地常量回比、A1、A4、A5、A6 的读数（打出来，单测另钉）。
pub(super) fn run_the_checks() -> Vec<String> {
    constants_against_the_implementation();
    let sizes = structure_sizes();
    let default_ring_bytes = JOURNAL_RING_DEFAULT_BYTES;
    vec![
        format!(
            "E7RESULT name=e197_checks constants_match_the_implementation=true ring_slots={RING_SLOTS} ring_slots_remainder={} device_holds_the_default_ring={} capacity_after_196608_pushes={} capacity_after_786432_pushes={}",
            default_ring_bytes % JOURNAL_RECORD_BYTES,
            device_holds_the_ring(TEST_IMAGE_DEFAULT_BYTES, default_ring_bytes),
            vec_capacity_after_pushes(RING_SLOTS),
            vec_capacity_after_pushes(4 * RING_SLOTS),
        ),
        format!(
            "E7RESULT name=e197_sizes size_location_entry={} size_named_unit={} size_node_pointer={} size_journal_record={} size_cache_key={}",
            sizes.location_entry,
            sizes.named_unit,
            sizes.node_pointer,
            sizes.journal_record,
            cache_key_bytes()
        ),
    ]
}

/// 登记修订一第七节 A11：读缓存的键 `(DeviceIdentity, DeviceOffsetInBytes, usize)` 的 `size_of`（推 24：字段重排后 8 + 8 + 4 按 8 对齐）。
fn cache_key_bytes() -> u64 {
    u64::try_from(std::mem::size_of::<(
        DeviceIdentity,
        DeviceOffsetInBytes,
        usize,
    )>())
    .expect("键的大小装得进 u64")
}

#[cfg(test)]
mod tests;
