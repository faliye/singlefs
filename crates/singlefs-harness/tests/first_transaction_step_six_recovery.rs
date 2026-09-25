//! 里程碑「第一个事务」步 6 的验收：进程外重开两个镜像走完整的挂载路径读出那个文件；坏字节探针与 E142（第一个事务的干跑）
//! 第九次跑产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 的 `name=recover_full` / `name=probe` 行逐字对。

mod common;

use common::{build_pool, file_content};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, JournalScanReport,
    PoolReader, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{
    DATA_UNIT_HEADER_BYTES, FIRST_TRANSACTION_TXG, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
};
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
            verification_failed: 0,
            maximum_applied_transaction: 0
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
    let newest_root = target_for_publish(
        CheckpointTxg(FIRST_TRANSACTION_TXG),
        common::parameters().geometry.root_ring_slots_per_region,
    );
    let newest_root_device =
        common::parameters().region_devices[usize::try_from(newest_root.region).expect("区域号")];
    let journal_record_three = record_offset(FIRST_TRANSACTION_TXG, JOURNAL_RING_DEFAULT_BYTES);
    let data_offset = SlotNumber(50180).to_device_offset();
    // 第一个事务的树表落在分配记录树五个节点、记账树、映射树之后（D8（核心索引结构） 已定项 14 之后是 50252）。
    let tree_table_offset = SlotNumber(50252).to_device_offset();
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
            name: "system_configuration_slot_one_both_devices",
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
                slot: SlotNumber(50252),
            }),
            3,
            0,
            0,
        ),
        // 陈旧的 tail（槽 1 坏了就择回槽 0：tail 2）：全环扫描不信 tail，结果与 tail 正确时相同。
        (
            "system_configuration_slot_one_both_devices",
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

/// 必红八条的「先信 tail」（C29（恢复先信 tail 会丢数据）、D23（journal 的角色与格式） 已定项 3 硬要求 1）：
/// 这条镜像上 tail 停在 2，而环里 jsn 3 那条记录既在 tail 之上、又在所选根的水位之上、还必须被施加——
/// 全环扫描找得到它、施加它、文件读得回来；「先信 tail」（只认 tail 为它作过证的那些记录）会把它整段丢掉。
/// 上面那条探针 `system_configuration_slot_one_both_devices` 是同一条条款的正例：
/// 它的所选根已经是第 3 代，tail 陈不陈旧结果都一样，**两种算法在它上面给出相同的答案**，挡不住先信 tail 的实现。
/// 这里把根也打回第 2 代，两种算法才分得开。
///
/// 怎么造出来的两处坏字节，各对一件真实的事：
/// 1. 最新根槽（第 3 代）坏一字节 —— 一次撕裂或介质错，择根退到暖机的第 2 代根（与探针
///    `newest_root_slot_one_byte` 同一处）。
/// 2. 两块盘的系统配置槽 1 各坏一字节 —— 槽 1 是第 3 代那次发布写的（tail 3），坏了就择回槽 0，
///    它带的是上一次发布留下的 tail 2。系统配置槽是发布序列的最后一步（D23 已定项 3 改动 2），
///    所以「tail 比环里的记录旧」本来就是每次崩溃都会出现的常态。
#[test]
fn a_stale_tail_does_not_hide_the_record_above_it_that_recovery_must_apply() {
    let pool = build_pool("stale_tail_with_a_record_above_it");
    let mut image = pool.memory_pool();
    let newest_root = target_for_publish(
        CheckpointTxg(FIRST_TRANSACTION_TXG),
        common::parameters().geometry.root_ring_slots_per_region,
    );
    let newest_root_device =
        common::parameters().region_devices[usize::try_from(newest_root.region).expect("区域号")];
    image.flip_byte(newest_root_device, slot_offset(newest_root, 4096), 100);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, DeviceOffsetInBytes(4096), 50);
    }

    // 前置：这条镜像上真有「tail 之后、而且必须被施加」的记录。
    // 三样都不走 `scan_journal` / `replay_journal`（它们正是被测的那段）：系统配置槽、根环、环里那一格各自直接读。
    // 没有这一段，下面的断言换一条健康镜像照样全绿——那时 applied=0、tail 也不陈旧，什么都没证明。
    let system_configuration = choose_system_configuration(&image).expect("槽 0 两块盘上都还好着");
    let stale_tail = system_configuration.quantities.journal_tail;
    assert_eq!(
        stale_tail, 2,
        "择回的系统配置槽带的是上一次发布的 tail：它比环里最新的记录旧一格"
    );
    let chosen_root = choose_root(&image, &system_configuration).expect("暖机的第 2 代根还在");
    assert_eq!(
        (chosen_root.instance, chosen_root.checkpoint_txg),
        ROOT_ONE_TWO,
        "最新根槽坏了，择根退到暖机的第 2 代根"
    );
    let record_bytes = image
        .read(
            DeviceIdentity(0),
            record_offset(FIRST_TRANSACTION_TXG, JOURNAL_RING_DEFAULT_BYTES),
            usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
        )
        .expect("环里有这一格");
    let record_above_the_tail = JournalRecord::parse(
        &record_bytes,
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier),
    )
    .expect("jsn 3 那条记录自己证得过：它是合法记录，不是残片");
    assert!(
        record_above_the_tail.counter > stale_tail,
        "这一条在 tail 之上：counter {} vs tail {stale_tail}",
        record_above_the_tail.counter
    );
    assert!(
        (
            record_above_the_tail.instance,
            record_above_the_tail.checkpoint_txg
        ) > (chosen_root.instance, chosen_root.checkpoint_txg),
        "这一条在所选根的水位之上：{:?} vs {:?}",
        (
            record_above_the_tail.instance,
            record_above_the_tail.checkpoint_txg
        ),
        (chosen_root.instance, chosen_root.checkpoint_txg)
    );
    assert!(
        record_above_the_tail.is_commit,
        "这一条是提交记录：不施加它就没有第 3 代根，文件也就回不来"
    );

    // 正题：全环扫描不信 tail ⇒ tail 之上那条记录照样被施加，文件读得回来。
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: ROOT_ONE_TWO,
            content: file_content()
        },
        "所选根是第 2 代，施加 jsn 3 之后走到第 3 代的树上读出那个文件"
    );
    assert_eq!(
        report.effective_root,
        Some(ROOT_ONE_THREE),
        "施加之后实际走的是记录 3 重建出来的第 3 代根"
    );
    assert_eq!(
        report.journal,
        JournalScanReport {
            valid_records: 3,
            above_water: 1,
            prefix_applied: 1,
            verification_passed: 1,
            verification_failed: 0,
            maximum_applied_transaction: 1
        },
        "tail 停在 2 而环里三条记录全被扫到；水位之上那一条（jsn 3）验过点名单元之后被施加"
    );
    assert_eq!(report.mapping_fallbacks, 0);
}

/// 改坏最新根槽 + 改坏 t1 两份：验点名单元的恢复停在第 2 代根（记录 3 验证失败、不施加）；
/// 关掉验证的那条分支会施加记录 3、走进一个数据单元读不到的树——这就是「关掉点名单元验证并改坏单元」那条必红用例。
#[test]
fn named_unit_verification_keeps_a_damaged_transaction_out_of_the_rebuilt_root() {
    let pool = build_pool("verify");
    let mut image = pool.memory_pool();
    let newest_root = target_for_publish(
        CheckpointTxg(FIRST_TRANSACTION_TXG),
        common::parameters().geometry.root_ring_slots_per_region,
    );
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

/// 只供测试的开关要能从运行时读数看出走了哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）：
/// 两臂**施加同样长的前缀、走到同一个结局**，唯一分得开它们的读数就是 `verification_passed`——
/// 不验那一臂一个单元都不读，它只能是 0。
///
/// ⚠️ 镜像要选「根槽坏、点名单元完好」这一档：发布完的健康镜像上根槽已是最新代，
/// journal 一条可施加的前缀都没有，那个循环体一次都不进，两臂都报 0、什么也证明不了
/// （2026-09-22 主 agent 第一版就写成了健康镜像，被修 panic 那条线拿打点读数指出来）。
#[test]
fn the_named_unit_verification_switch_shows_up_in_the_runtime_counter() {
    let pool = build_pool("verification-switch-visible");
    let mut image = pool.memory_pool();
    // 只改坏最新那条根，点名单元一个不动：恢复落回 (1, 2)，再按 journal 把 txg 3 那条施加上去，
    // 验的那一臂因此真的要去读 50180 并比校验和。
    let newest_root = target_for_publish(
        CheckpointTxg(FIRST_TRANSACTION_TXG),
        common::parameters().geometry.root_ring_slots_per_region,
    );
    image.flip_byte(DeviceIdentity(0), slot_offset(newest_root, 4096), 100);

    let verified = recover(&image, JournalPolicy::Consult);
    let unverified = recover(&image, JournalPolicy::ConsultWithoutNamedVerification);
    assert_eq!(
        verified.outcome, unverified.outcome,
        "两臂走到同一个结局：光看结局分不出走了哪一条"
    );
    assert_eq!(
        verified.journal.prefix_applied, unverified.journal.prefix_applied,
        "两臂施加的前缀一样长：光看它也分不出"
    );
    assert!(
        verified.journal.prefix_applied > 0,
        "这个镜像上要真有一条记录被施加，否则那个循环体一次都不进、下面两条断言什么都证明不了：实测 {}",
        verified.journal.prefix_applied
    );
    assert_eq!(
        verified.journal.verification_passed, verified.journal.prefix_applied,
        "验的那一臂：每条被施加的记录都逐个读过点名单元、比过校验和"
    );
    assert_eq!(
        unverified.journal.verification_passed, 0,
        "不验的那一臂一个单元都没读，这个数只能是 0；它要是跟着涨，计数就在宣称验过了"
    );
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
    let newest_root = target_for_publish(
        CheckpointTxg(FIRST_TRANSACTION_TXG),
        common::parameters().geometry.root_ring_slots_per_region,
    );
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
