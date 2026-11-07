//! 崩溃镜像（`CrashImage`）交给池级 checker 的扫描候选只由崩溃状态的字节定，不随这份字节怎么拆成基线与叠写而变。
//! 崩溃判定库的块键只由崩溃状态的内容定，几条流共有的节点共用一块判定；候选几何若取基线的系统配置槽 0，基线抹了系统配置槽、
//! 流里又整槽写回的那条流与没抹的那条在共用节点上交给判器的候选不同，判定读到了块键之外的输入（里程碑三收尾批 19 验证段：
//! 同一把块键下 I-8.14 一份判到、一份不适用）。
mod common;

use common::{build_pool, geometry};
use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::PoolReader;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_verification::memory_pool::{writes_and_segments, CrashImage};

/// 池级 checker 的报告里与另一份不同的条目：（不变量, 这一份的判定, 另一份的判定）；两份条目次序相同（同一个入口按同一张表判）。
fn verdicts_that_differ(
    report: &[(&'static str, InvariantVerdict)],
    other_report: &[(&'static str, InvariantVerdict)],
) -> Vec<(&'static str, InvariantVerdict, InvariantVerdict)> {
    assert_eq!(
        report
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        other_report
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        "两份报告判的不变量与次序相同"
    );
    report
        .iter()
        .zip(other_report)
        .filter(|((_, verdict), (_, other_verdict))| verdict != other_verdict)
        .map(|((invariant, verdict), (_, other_verdict))| {
            (*invariant, verdict.clone(), other_verdict.clone())
        })
        .collect()
}

fn slot_zero_self_verifies<Reader: PoolReader>(reader: &Reader, device: DeviceIdentity) -> bool {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let slot = PoolReader::read(reader, device, DeviceOffsetInBytes(0), slot_bytes)
        .expect("池里有这块盘，槽 0 读得出");
    singlefs_checker::check_system_configuration_slot(&slot).is_ok()
}

/// 新池新建文件那条流（mkfs 之后的基线、之后每一次写都持久的那个崩溃状态，即流里最后一个节点的状态之一）拆成三份：
/// 没抹的基线加叠写；基线抹了盘 0 的两个系统配置槽与盘 1 的槽 0（与 GPU 对拍用例
/// `record_new_pool_with_only_the_slot_one_of_the_second_device_self_verified_in_the_base` 同一形）加同一份叠写，流里的系统配置槽写把这三槽整槽写回；
/// 全部字节都在基线里的落成池。三份是同一份崩溃状态字节，交给池级 checker 的候选与池级 checker 的报告逐条相同。
#[test]
fn crash_state_over_a_base_that_lost_its_system_configuration_slots_hands_the_pool_checker_what_the_same_bytes_over_an_intact_base_hand(
) {
    let pool = build_pool("scan-candidates-of-a-crash-state-split-two-ways");
    let operations = pool.retained_operations();
    let (writes, _segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let intact_base = pool.memory_pool_after_mkfs();
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let lost_slots = [
        (DeviceIdentity(0), 0),
        (DeviceIdentity(0), spacing),
        (DeviceIdentity(1), 0),
    ];
    let mut base_that_lost_its_slots = intact_base.clone();
    for (device, offset) in lost_slots {
        base_that_lost_its_slots
            .devices
            .get_mut(&device)
            .expect("基线里有这块盘")
            .write(DeviceOffsetInBytes(offset), &[0u8; 4096]);
    }
    let mut materialized = intact_base.clone();
    materialized.apply_writes(&writes);
    let over_the_intact_base = CrashImage {
        base: &intact_base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    let over_the_base_that_lost_its_slots = CrashImage {
        base: &base_that_lost_its_slots,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    let devices = [DeviceIdentity(0), DeviceIdentity(1)];
    for device in devices {
        assert!(
            slot_zero_self_verifies(&intact_base, device)
                && !slot_zero_self_verifies(&base_that_lost_its_slots, device),
            "前提：盘 {} 的槽 0 在没抹的基线里自证得过、在抹了的基线里自证不过（两种拆法的基线几何确有不同）",
            device.0
        );
    }
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (device, offset) in lost_slots {
        let over_the_intact_base_bytes = PoolReader::read(
            &over_the_intact_base,
            device,
            DeviceOffsetInBytes(offset),
            slot_bytes,
        );
        assert!(
            over_the_intact_base_bytes
                == PoolReader::read(
                    &over_the_base_that_lost_its_slots,
                    device,
                    DeviceOffsetInBytes(offset),
                    slot_bytes
                )
                && over_the_intact_base_bytes
                    == PoolReader::read(
                        &materialized,
                        device,
                        DeviceOffsetInBytes(offset),
                        slot_bytes
                    ),
            "前提：盘 {} 偏移 {offset} 那一槽被流里的系统配置槽写整槽写回，三份崩溃状态在这里逐字节相同（基线只差这三槽）",
            device.0
        );
    }
    for device in devices {
        let unit_slots_over_the_intact_base =
            ImageReader::candidate_unit_slots(&over_the_intact_base, device.0);
        assert_eq!(
            ImageReader::candidate_unit_slots(&over_the_base_that_lost_its_slots, device.0),
            unit_slots_over_the_intact_base,
            "盘 {}：基线抹了系统配置槽与没抹，同一份崩溃状态字节交的候选单元槽相同",
            device.0
        );
        assert_eq!(
            ImageReader::candidate_unit_slots(&materialized, device.0),
            unit_slots_over_the_intact_base,
            "盘 {}：全部字节都在基线里的落成池与叠写那一份交的候选单元槽相同",
            device.0
        );
        let journal_slots_over_the_intact_base =
            ImageReader::candidate_journal_slots(&over_the_intact_base, device.0);
        assert_eq!(
            ImageReader::candidate_journal_slots(&over_the_base_that_lost_its_slots, device.0),
            journal_slots_over_the_intact_base,
            "盘 {}：基线抹了系统配置槽与没抹，同一份崩溃状态字节交的候选 journal 记录槽相同（环长取崩溃状态自己的槽 0）",
            device.0
        );
        assert_eq!(
            ImageReader::candidate_journal_slots(&materialized, device.0),
            journal_slots_over_the_intact_base,
            "盘 {}：全部字节都在基线里的落成池与叠写那一份交的候选 journal 记录槽相同",
            device.0
        );
    }
    let report_over_the_intact_base = check_pool_image(&over_the_intact_base);
    let report_over_the_base_that_lost_its_slots =
        check_pool_image(&over_the_base_that_lost_its_slots);
    let report_of_the_materialized_pool = check_pool_image(&materialized);
    println!(
        "I-8.14 在基线抹了系统配置槽的那一份上：{:?}；没抹的那一份上：{:?}",
        report_over_the_base_that_lost_its_slots
            .iter()
            .find(|(invariant, _)| *invariant == "I-8.14"),
        report_over_the_intact_base
            .iter()
            .find(|(invariant, _)| *invariant == "I-8.14")
    );
    assert_eq!(
        verdicts_that_differ(
            &report_over_the_base_that_lost_its_slots,
            &report_over_the_intact_base
        ),
        Vec::new(),
        "基线抹了系统配置槽与没抹，同一份崩溃状态字节上池级 checker 的报告逐条相同"
    );
    assert_eq!(
        verdicts_that_differ(
            &report_of_the_materialized_pool,
            &report_over_the_intact_base
        ),
        Vec::new(),
        "落成池与叠写那一份上池级 checker 的报告逐条相同"
    );
}
