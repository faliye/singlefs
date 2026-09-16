//! 里程碑「第一个事务」步 6 的验收：进程外重开两个镜像走完整的挂载路径读出那个文件；坏字节探针与 E142（第一个事务的干跑）
//! 第八次跑产物 `research/results/e142-first-txn-dry-run-2026-09-16-tree-table-200.out` 的 `name=recover_full` / `name=probe` 行逐字对。

mod common;

use common::{build_pool, file_content};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::{
    recover, JournalPolicy, JournalScanReport, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_format::{DATA_UNIT_HEADER_BYTES, FIRST_TRANSACTION_TXG, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::MemoryPool;

const ROOT_ONE_THREE: (InstanceGeneration, CheckpointTxg) =
    (InstanceGeneration(1), CheckpointTxg(3));
const ROOT_ONE_TWO: (InstanceGeneration, CheckpointTxg) = (InstanceGeneration(1), CheckpointTxg(2));

/// 产物第 44 行逐字：`name=recover_full outcome=file_read root=1:3 content_matches=true valid_records=3 above_water=0 applied=0 verification_passed=0 mapping_fallbacks=0`。
#[test]
fn cold_start_reopens_the_images_and_reads_the_file_back_choosing_instance_one_txg_three() {
    let mut pool = build_pool("cold");
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: ROOT_ONE_THREE,
            content: file_content()
        }
    );
    assert_eq!(
        report.journal,
        JournalScanReport {
            valid_records: 3,
            above_water: 0,
            prefix_applied: 0,
            verification_passed: 0,
            verification_failed: 0
        },
        "全环扫描到三条记录，没有一条高于所选根的水位"
    );
    assert_eq!(report.mapping_fallbacks, 0);
    let ignored = recover(&reopened, JournalPolicy::Ignore);
    assert_eq!(
        ignored.outcome, report.outcome,
        "根槽已持久：看不看 journal 结果一样"
    );
    // 内存镜像与文件镜像是同一份字节：同一条恢复路径给同一个答案。
    let in_memory = recover(&pool.memory_pool(), JournalPolicy::Consult);
    assert_eq!(in_memory, report);
}

struct Probe {
    name: &'static str,
    flips: Vec<(DeviceIdentity, DeviceOffsetInBytes, u64)>,
}

fn probes() -> Vec<Probe> {
    let newest_root = target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let newest_root_device =
        common::parameters().region_devices[usize::try_from(newest_root.region).expect("区域号")];
    let journal_record_three = record_offset(FIRST_TRANSACTION_TXG, JOURNAL_RING_DEFAULT_BYTES);
    let data_offset = SlotNumber(50180).to_device_offset();
    let tree_table_offset = SlotNumber(50248).to_device_offset();
    let both = |offset: DeviceOffsetInBytes, byte: u64| {
        vec![
            (DeviceIdentity(0), offset, byte),
            (DeviceIdentity(1), offset, byte),
        ]
    };
    vec![
        Probe {
            name: "newest_root_slot_one_byte",
            flips: vec![(newest_root_device, slot_offset(newest_root, 4096), 100)],
        },
        Probe {
            name: "journal_record_both_copies",
            flips: both(journal_record_three, 200),
        },
        Probe {
            name: "journal_record_one_copy",
            flips: vec![(DeviceIdentity(0), journal_record_three, 200)],
        },
        Probe {
            name: "data_payload_one_copy",
            flips: vec![(DeviceIdentity(0), data_offset, 200)],
        },
        Probe {
            name: "data_payload_both_copies",
            flips: both(data_offset, 200),
        },
        Probe {
            name: "data_header_last_byte_both_copies",
            flips: both(data_offset, DATA_UNIT_HEADER_BYTES - 1),
        },
        Probe {
            name: "tree_table_both_copies",
            flips: both(tree_table_offset, 300),
        },
        Probe {
            name: "superblock_slot_one_both_devices",
            flips: both(DeviceOffsetInBytes(4096), 50),
        },
    ]
}

fn damaged(full: &MemoryPool, probe: &Probe) -> MemoryPool {
    let mut image = full.clone();
    for (device, offset, byte_index) in &probe.flips {
        image.flip_byte(*device, *offset, *byte_index);
    }
    image
}

/// 产物第 50–57 行的八条探针逐字（outcome / root / valid_records / applied / mapping_fallbacks / reason）。
#[test]
fn probes_behave_as_milestone_step_six_expects() {
    let pool = build_pool("probes");
    let full = pool.memory_pool();
    let content = file_content();
    let file_read = |root| RecoveryOutcome::FileRead {
        root,
        content: content.clone(),
    };
    let failed = |failure| RecoveryOutcome::Failed {
        root: Some(ROOT_ONE_THREE),
        failure,
    };
    // (探针名, 期望结果, valid_records, applied, mapping_fallbacks)
    let expected: Vec<(&str, RecoveryOutcome, usize, usize, usize)> = vec![
        // 改坏最新根槽一字节：择回暖机的第 2 代根，再从 jsn 3 那条记录的新根段重建第 3 代根，文件在（D23 已定项 15）。
        (
            "newest_root_slot_one_byte",
            file_read(ROOT_ONE_TWO),
            3,
            1,
            0,
        ),
        (
            "journal_record_both_copies",
            file_read(ROOT_ONE_THREE),
            2,
            0,
            0,
        ),
        (
            "journal_record_one_copy",
            file_read(ROOT_ONE_THREE),
            3,
            0,
            0,
        ),
        ("data_payload_one_copy", file_read(ROOT_ONE_THREE), 3, 0, 0),
        (
            "data_payload_both_copies",
            failed(RecoveryFailure::MappingStillUnreadable {
                slot: SlotNumber(50180),
            }),
            3,
            0,
            1,
        ),
        (
            "data_header_last_byte_both_copies",
            failed(RecoveryFailure::MappingStillUnreadable {
                slot: SlotNumber(50180),
            }),
            3,
            0,
            1,
        ),
        (
            "tree_table_both_copies",
            failed(RecoveryFailure::UnitUnreadable {
                slot: SlotNumber(50248),
            }),
            3,
            0,
            0,
        ),
        // 陈旧的 tail（槽 1 坏了就择回槽 0：tail 2）：全环扫描不信 tail，结果与 tail 正确时相同。
        (
            "superblock_slot_one_both_devices",
            file_read(ROOT_ONE_THREE),
            3,
            0,
            0,
        ),
    ];
    let probes = probes();
    assert_eq!(probes.len(), expected.len());
    for (probe, (name, outcome, valid_records, applied, mapping_fallbacks)) in
        probes.iter().zip(expected)
    {
        assert_eq!(probe.name, name);
        let report = recover(&damaged(&full, probe), JournalPolicy::Consult);
        assert_eq!(report.outcome, outcome, "探针 {name}");
        assert_eq!(
            report.journal.valid_records, valid_records,
            "探针 {name} 的 valid_records"
        );
        assert_eq!(
            report.journal.prefix_applied, applied,
            "探针 {name} 的 applied"
        );
        assert_eq!(
            report.mapping_fallbacks, mapping_fallbacks,
            "探针 {name} 的 mapping_fallbacks"
        );
    }
}

/// 改坏最新根槽 + 改坏 t1 两份：验点名单元的恢复停在第 2 代根（记录 3 验证失败、不施加）；
/// 关掉验证的那条分支会施加记录 3、走进一个数据单元读不到的树——这就是「关掉点名单元验证并改坏单元」那条必红用例。
#[test]
fn named_unit_verification_keeps_a_damaged_transaction_out_of_the_rebuilt_root() {
    let pool = build_pool("verify");
    let mut image = pool.memory_pool();
    let newest_root = target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    image.flip_byte(DeviceIdentity(0), slot_offset(newest_root, 4096), 100);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, SlotNumber(50180).to_device_offset(), 200);
    }
    let verified = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        verified.outcome,
        RecoveryOutcome::NoFile { root: ROOT_ONE_TWO }
    );
    assert_eq!(
        (
            verified.journal.verification_failed,
            verified.journal.prefix_applied
        ),
        (1, 0)
    );
    let unverified = recover(&image, JournalPolicy::ConsultWithoutNamedVerification);
    assert_eq!(
        unverified.outcome,
        RecoveryOutcome::Failed {
            root: Some(ROOT_ONE_TWO),
            failure: RecoveryFailure::MappingStillUnreadable {
                slot: SlotNumber(50180)
            }
        },
        "不验就施加了记录 3，走读在数据单元上失败"
    );
    assert_eq!(unverified.journal.prefix_applied, 1);
}

/// 改坏 journal 记录一字节：前缀在它之前截断，恢复仍完成——改坏的是 jsn 2 两份时，jsn 3 不再连续、不施加，但根槽 1:3 在，文件照读。
#[test]
fn torn_journal_record_truncates_the_prefix_but_recovery_still_completes() {
    let pool = build_pool("torn");
    let mut image = pool.memory_pool();
    let mut newest_root_damaged = image.clone();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, record_offset(2, JOURNAL_RING_DEFAULT_BYTES), 300);
        newest_root_damaged.flip_byte(device, record_offset(2, JOURNAL_RING_DEFAULT_BYTES), 300);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: ROOT_ONE_THREE,
            content: file_content()
        }
    );
    assert_eq!(report.journal.valid_records, 2);
    // 再改坏最新根槽：所选根退到 1:2，jsn 3 在 1:2 之上、而它就是紧接着的那一条 ⇒ 仍施加、文件在。
    let newest_root = target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    newest_root_damaged.flip_byte(DeviceIdentity(0), slot_offset(newest_root, 4096), 100);
    let fallback = recover(&newest_root_damaged, JournalPolicy::Consult);
    assert_eq!(
        fallback.outcome,
        RecoveryOutcome::FileRead {
            root: ROOT_ONE_TWO,
            content: file_content()
        }
    );
    assert_eq!(
        (
            fallback.journal.valid_records,
            fallback.journal.prefix_applied
        ),
        (2, 1)
    );
}
