//! 池级 checker 审计复核的两项（`records/2026-10-24-singlefs-checker-审计复核.md`）与它们留下的两条欠账（`.claude/kb/checks-owed.md` C896、C897）在完整镜像上的用例：
//! C01：别的版本先走过的单元被这一版又引用时，按这一条引用定的那几格照判——非最新候选根的树表指针记的出生 txg 与它指着的共享树表单元头不符，
//! 走读命中先走过的树表、不往下展开，I-1.2（块头写序已发布） 照样红；
//! C896：别的版本先走过的码 2 节点，这一版的父条目给的分隔 key 照判——两层中央映射树、两版共享最左那片叶，旧那一版的父节点把分隔 key 改大，
//! 命中先走过的叶、不往下展开，I-1.1（块头自述逻辑地址） 照样红；
//! C02：池级 checker 逐盘扫 journal 环按 (实例代号, 计数器) 收、同键先到先留，与 core 的 `scan_journal` 同一个选法——同一块盘上两条记录
//! 撞同一个键、或同一个计数器坐着两个实例的记录，checker 认的恢复视图与 core 扫出的逐条相同（同一个 `MemoryPool` 喂两边）；
//! C897：那几种布局里不在计数器该落的环槽上的记录判 I-8.13（记录落在它的计数器该落的环槽） 红，别的池（fsid 不同）的旧记录不判；
//! 同盘同键先到的是 core 的记录读者不收的、后到的是好记录时，恢复视图留后到的那条、与 core 相同，先到那条的提交标记字节照判 I-8.8 红。
//! 合法镜像（mkfs → 暖机 → 新池新建文件）上零违例、I-8.13 判过且成立、两边的视图相同。
//! 树 ID、类标签、「判过几个被引用单元」不翻倍与按位置寻址的树、容器树、inode 树命中时那几格的局部对照在 `crates/singlefs-checker/src/walk.rs`
//! 的库内用例里（要读私有的走读）。

mod common;
mod common_tree_split;

use common::{build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records};
use common_tree_split::{
    capacities, propagate_the_new_checksum, read_unit_on_device, reseal_unit_by_its_class,
    write_unit_to_the_same_slot_on_both_devices, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
};
use singlefs_checker::check_journal_record;
use singlefs_checker::image::{
    chosen_system_configurations, root_slot_positions, ImageReader, InvariantVerdict,
    JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
};
use singlefs_checker::walk::{
    check_pool_image, journal_records_as_the_recovery_reads_them, JournalRecordTheRecoveryReads,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{
    JournalRecord, NamedUnit, JOURNAL_HEADER_CHECKSUM_OFFSET, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET,
};
use singlefs_core::recovery::{choose_system_configuration, scan_journal};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{parse_index_node, unit_filesystem_identifier};
use singlefs_format::{
    JOURNAL_HEADER_BYTES, JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
    NODE_BYTES, SLOT_BYTES,
};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
use singlefs_harness::segments::StepKind;

/// 非最新候选根里那一条：暖机写的第一条根（实例 1、txg 1）。它的树表与别的候选根共享（暖机那几版没有文件，树表单元没改过）。
const OLDER_CANDIDATE_ROOT_TXG: u64 = 1;
/// 改坏之后树表指针记的出生 txg：合法历史上任何单元都不在这一代出生。
const BIRTH_TXG_NO_UNIT_CARRIES: u64 = 999;

/// 违例的那几条：（不变量，说明）。
fn violations_of(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<(&'static str, String)> {
    verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// core 扫环交回的记录，写成与池级 checker 的恢复视图同一种样子（键升序）。
fn the_core_scan_of_the_journal(image: &MemoryPool) -> Vec<JournalRecordTheRecoveryReads> {
    let system_configuration =
        choose_system_configuration(image).expect("合法历史出发的镜像，系统配置择得出");
    scan_journal(image, &system_configuration)
        .iter()
        .map(
            |((instance, counter), record)| JournalRecordTheRecoveryReads {
                instance: instance.0,
                counter: *counter,
                checkpoint_txg: record.checkpoint_txg.0,
            },
        )
        .collect()
}

/// 池级 checker 认的恢复视图。
fn the_checker_view_of_the_journal(image: &MemoryPool) -> Vec<JournalRecordTheRecoveryReads> {
    journal_records_as_the_recovery_reads_them(image).expect("合法历史出发的镜像，系统配置择得出")
}

#[test]
fn the_pool_built_through_the_first_file_has_no_violation_and_both_scans_of_the_journal_agree() {
    let built = build_pool("checker-cached-reference-legal-pool");
    let image = built.memory_pool();
    let violations = violations_of(&check_pool_image(&image));
    assert!(violations.is_empty(), "合法镜像零违例：{violations:?}");
    let checker_view = the_checker_view_of_the_journal(&image);
    assert!(!checker_view.is_empty(), "暖机与新建文件写过记录");
    assert_eq!(
        checker_view,
        the_core_scan_of_the_journal(&image),
        "每个计数器一条记录的合法环上，池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
}

/// C01（审计「完整镜像实测」那一形）：暖机写的第一条根（实例 1、txg 1）的树表指针把出生 txg 改成 999、重封根槽，
/// journal 里同代记录的新根段与反向链跟着重封（`copy_each_root_into_the_new_root_segment_of_its_journal_records`）。
/// 那张树表别的候选根也指着、先被走过：这一条根走到它时命中走过的单元、不往下展开，这一条指针的出生身份照判，I-1.2 红；别的不变量不红。
#[test]
fn an_older_candidate_root_whose_tree_table_pointer_names_another_birth_txg_reddens_only_the_birth_identity_invariant_although_the_shared_tree_table_was_walked_first(
) {
    let built = build_pool("checker-cached-reference-birth-txg");
    let original = built.memory_pool();
    assert!(
        violations_of(&check_pool_image(&original)).is_empty(),
        "改之前零违例"
    );
    let geometry = chosen_system_configurations(&original)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        .expect("系统配置择得出");
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let readable_roots: Vec<(u32, u64, RootRecord)> = root_slot_positions(&geometry)
        .into_iter()
        .filter_map(|(_, _, device, offset)| {
            let bytes = ImageReader::read(&original, device, offset, root_slot_bytes)?;
            RootRecord::parse_slot(&bytes, &geometry.filesystem_identifier)
                .map(|root| (device, offset, root))
        })
        .collect();
    let older_candidate_roots: Vec<&(u32, u64, RootRecord)> = readable_roots
        .iter()
        .filter(|(_, _, root)| {
            root.instance == InstanceGeneration(1)
                && root.checkpoint_txg == CheckpointTxg(OLDER_CANDIDATE_ROOT_TXG)
        })
        .collect();
    assert_eq!(
        older_candidate_roots.len(),
        1,
        "根环里恰有一条实例 1、txg 1 的根"
    );
    let (device, offset, older_root) = older_candidate_roots[0];
    assert!(
        readable_roots.iter().any(
            |(_, _, root)| root.checkpoint_txg != older_root.checkpoint_txg
                && root.tree_table == older_root.tree_table
        ),
        "别的根指着同一张树表（同一条树表指针）：这一条根走到它时它已被走过"
    );
    assert_ne!(
        older_root.tree_table.head.birth_txg,
        CheckpointTxg(BIRTH_TXG_NO_UNIT_CARRIES),
        "改之前树表指针的出生 txg 不是 999"
    );
    let mut root_naming_another_birth_txg = *older_root;
    root_naming_another_birth_txg.tree_table.head.birth_txg =
        CheckpointTxg(BIRTH_TXG_NO_UNIT_CARRIES);
    let mut image = original.clone();
    image.apply_writes(&[RetainedWrite {
        device: DeviceIdentity(*device),
        kind: StepKind::RootRecordFua,
        is_force_unit_access: true,
        offset: DeviceOffsetInBytes(*offset),
        contents: WrittenContents::Bytes(root_naming_another_birth_txg.to_slot(root_slot_bytes)),
    }]);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    let violations = violations_of(&check_pool_image(&image));
    let violated_invariants: Vec<&str> =
        violations.iter().map(|(invariant, _)| *invariant).collect();
    assert_eq!(
        violated_invariants,
        vec!["I-1.2"],
        "只红 I-1.2（树表指针的出生身份与共享树表单元头不符）：{violations:?}"
    );
}

/// 盘 0 journal 环里写的一条记录：照合法镜像里第一条记录改实例代号、计数器与 `checkpoint_txg`，其余照它，或按 `written_as` 改一处。
struct RecordInARingSlotOfDeviceZero {
    ring_slot: u64,
    instance: u32,
    counter: u64,
    checkpoint_txg: u64,
    written_as: HowTheRecordIsWritten,
}

/// 一条手写记录与合法镜像里第一条记录还差哪一处（实例代号、计数器与 `checkpoint_txg` 之外）。
#[derive(Clone, Copy)]
enum HowTheRecordIsWritten {
    /// 照合法镜像里第一条记录（两个读者都收）。
    AsTheFirstRecordOfThePool,
    /// 提交标记写成 0 / 1 之外的一个字节、重封头校验和：池级 checker 的记录读者收、core 的记录读者当损坏不收（D23（journal 的角色与格式） 已定项 7）。
    CommitMarkerByteTheCoreReaderRefuses(u8),
    /// 带上合法镜像环里头一条带点名项的记录的点名项，第 0 项的 flags（点名项偏移 55，D23（journal 的角色与格式） 已定项 17）写成给定的非 0 字节、
    /// 重封载荷校验和与头校验和：core 的记录读者跳过这一字节照收（`NamedUnit::read_from`），池级 checker 的记录读者判 `NonZeroFlags` 拒。
    NamedEntryFlagsByteThePoolCheckerReaderRefuses(u8),
}

/// 记录头里提交标记那 1 字节：紧挨在本次发布内序号之前（事务号 8 之后，D23（journal 的角色与格式） 已定项 4 的字段表）。
const JOURNAL_RECORD_COMMIT_MARKER_OFFSET: usize = JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET - 1;
/// 记录头里载荷校验和那 4 字节：本次发布内序号 4、反向链 4 之后（D23（journal 的角色与格式） 已定项 4 的字段表），罩点名项那一段。
const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET + 4 + 4;
/// 点名项 56 字节里 flags 那 1 字节的偏移（D23（journal 的角色与格式） 已定项 17）。
const JOURNAL_NAMED_ENTRY_FLAGS_OFFSET: usize = 55;

/// 合法镜像盘 0 环里头一条带点名项的记录的点名项（暖机的空发布不带点名项，新建文件那一次发布带）。
fn named_entries_of_the_first_record_naming_units(image: &MemoryPool) -> Vec<NamedUnit> {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
    let filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    (0..system_configuration.immutable.sizes.journal_ring_bytes / JOURNAL_RECORD_BYTES)
        .map_while(|ring_slot| {
            JournalRecord::parse(
                &ImageReader::read(
                    image,
                    0,
                    ring_start + ring_slot * JOURNAL_RECORD_BYTES,
                    record_bytes,
                )?,
                filesystem_identifier,
            )
        })
        .find(|record| !record.named.is_empty())
        .expect("新建文件那一次发布的记录带点名项")
        .named
}

/// 盘 0 journal 环里按 `records` 写几条记录、别的盘与别的槽清空（冲突布局，合法写者不写它）。每条先经池级 checker 的记录读者断言收得下；
/// core 的记录读者照提交标记那一样断言收或不收。
fn image_with_records_on_device_zero(
    tag: &str,
    records: &[RecordInARingSlotOfDeviceZero],
) -> MemoryPool {
    let built = build_pool(tag);
    let mut image = built.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
    let filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let first_record_of_the_pool = JournalRecord::parse(
        &ImageReader::read(&image, 0, ring_start, record_bytes).expect("盘 0 的环头读得出"),
        filesystem_identifier,
    )
    .expect("合法镜像环头那一槽是暖机写的第一条记录");
    let mut writes: Vec<RetainedWrite> = ImageReader::devices(&image)
        .into_iter()
        .map(|device| RetainedWrite {
            device: DeviceIdentity(device),
            kind: StepKind::ZeroFill,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(ring_start),
            contents: WrittenContents::Zeros {
                length: system_configuration.immutable.sizes.journal_ring_bytes,
            },
        })
        .collect();
    for written in records {
        let mut record = first_record_of_the_pool.clone();
        record.instance = InstanceGeneration(written.instance);
        record.counter = written.counter;
        record.checkpoint_txg = CheckpointTxg(written.checkpoint_txg);
        let mut bytes = record.to_bytes();
        let what = format!(
            "环槽 {} 实例 {}、计数器 {}、txg {}",
            written.ring_slot, written.instance, written.counter, written.checkpoint_txg
        );
        match written.written_as {
            HowTheRecordIsWritten::AsTheFirstRecordOfThePool => {
                assert!(
                    JournalRecord::parse(&bytes, filesystem_identifier).is_some(),
                    "core 的记录读者收得下（{what}）"
                );
            }
            HowTheRecordIsWritten::CommitMarkerByteTheCoreReaderRefuses(byte) => {
                bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] = byte;
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    record_bytes,
                    JOURNAL_HEADER_CHECKSUM_OFFSET,
                );
                bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
                    .copy_from_slice(&digest);
                assert!(
                    JournalRecord::parse(&bytes, filesystem_identifier).is_none(),
                    "core 的记录读者不收提交标记字节 {byte}（{what}）"
                );
            }
            HowTheRecordIsWritten::NamedEntryFlagsByteThePoolCheckerReaderRefuses(byte) => {
                record.named = named_entries_of_the_first_record_naming_units(&image);
                bytes = record.to_bytes();
                let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("496");
                let payload_end = header_bytes
                    + record.named.len() * usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
                bytes[header_bytes + JOURNAL_NAMED_ENTRY_FLAGS_OFFSET] = byte;
                let payload_checksum = crc32_castagnoli(&bytes[header_bytes..payload_end]);
                bytes[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET
                    ..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
                    .copy_from_slice(&payload_checksum.to_le_bytes());
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    record_bytes,
                    JOURNAL_HEADER_CHECKSUM_OFFSET,
                );
                bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
                    .copy_from_slice(&digest);
                assert!(
                    JournalRecord::parse(&bytes, filesystem_identifier).is_some(),
                    "core 的记录读者跳过点名项 flags 字节 {byte} 照收（{what}）"
                );
                assert_eq!(
                    check_journal_record(&bytes, filesystem_identifier),
                    Err(singlefs_checker::Verdict::NonZeroFlags),
                    "池级 checker 的记录读者只因点名项 flags 字节 {byte} 拒（{what}）"
                );
            }
        }
        match written.written_as {
            HowTheRecordIsWritten::AsTheFirstRecordOfThePool
            | HowTheRecordIsWritten::CommitMarkerByteTheCoreReaderRefuses(_) => assert!(
                check_journal_record(&bytes, filesystem_identifier).is_ok(),
                "池级 checker 的记录读者收得下（{what}）"
            ),
            HowTheRecordIsWritten::NamedEntryFlagsByteThePoolCheckerReaderRefuses(_) => {}
        }
        writes.push(RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::JournalRecord,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(ring_start + written.ring_slot * JOURNAL_RECORD_BYTES),
            contents: WrittenContents::Bytes(bytes),
        });
    }
    image.apply_writes(&writes);
    image
}

/// 同一块盘（盘 0）journal 环的头两槽里各写一条记录，计数器都写 1、实例代号与 `checkpoint_txg` 按环槽次序取给定的两对（审计 C02 的冲突布局）。
fn image_with_two_records_at_counter_one_on_device_zero(
    tag: &str,
    instance_and_txg_in_ring_slot_order: [(u32, u64); 2],
) -> MemoryPool {
    let records: Vec<RecordInARingSlotOfDeviceZero> = instance_and_txg_in_ring_slot_order
        .into_iter()
        .zip(0u64..)
        .map(
            |((instance, checkpoint_txg), ring_slot)| RecordInARingSlotOfDeviceZero {
                ring_slot,
                instance,
                counter: 1,
                checkpoint_txg,
                written_as: HowTheRecordIsWritten::AsTheFirstRecordOfThePool,
            },
        )
        .collect();
    image_with_records_on_device_zero(tag, &records)
}

/// 恢复视图里的一条。
const fn record_the_recovery_reads(
    instance: u32,
    counter: u64,
    checkpoint_txg: u64,
) -> JournalRecordTheRecoveryReads {
    JournalRecordTheRecoveryReads {
        instance,
        counter,
        checkpoint_txg,
    }
}

#[test]
fn two_records_of_one_instance_and_counter_on_one_device_leave_the_earlier_ring_slot_in_the_checker_view_as_in_the_core_scan(
) {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-same-key-txg-ten-first",
        [(1, 10), (1, 20)],
    );
    let core_scan = the_core_scan_of_the_journal(&image);
    assert_eq!(
        core_scan,
        vec![record_the_recovery_reads(1, 1, 10)],
        "core 先到先留：环槽 0 那一条（txg 10）"
    );
    assert_eq!(
        the_checker_view_of_the_journal(&image),
        core_scan,
        "池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
}

#[test]
fn two_records_of_one_instance_and_counter_on_one_device_in_reverse_order_leave_the_earlier_ring_slot_in_the_checker_view_as_in_the_core_scan(
) {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-same-key-txg-twenty-first",
        [(1, 20), (1, 10)],
    );
    let core_scan = the_core_scan_of_the_journal(&image);
    assert_eq!(
        core_scan,
        vec![record_the_recovery_reads(1, 1, 20)],
        "core 先到先留：环槽 0 那一条（txg 20）"
    );
    assert_eq!(
        the_checker_view_of_the_journal(&image),
        core_scan,
        "池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
}

#[test]
fn records_of_two_instances_at_one_counter_on_one_device_are_both_kept_in_the_checker_view_as_in_the_core_scan(
) {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-two-instances-instance-one-first",
        [(1, 10), (2, 20)],
    );
    let core_scan = the_core_scan_of_the_journal(&image);
    assert_eq!(
        core_scan,
        vec![
            record_the_recovery_reads(1, 1, 10),
            record_the_recovery_reads(2, 1, 20)
        ],
        "core 按 (实例代号, 计数器) 收：两条都留"
    );
    assert_eq!(
        the_checker_view_of_the_journal(&image),
        core_scan,
        "池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
}

#[test]
fn records_of_two_instances_at_one_counter_on_one_device_in_reverse_order_are_both_kept_in_the_checker_view_as_in_the_core_scan(
) {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-two-instances-instance-two-first",
        [(2, 20), (1, 10)],
    );
    let core_scan = the_core_scan_of_the_journal(&image);
    assert_eq!(
        core_scan,
        vec![
            record_the_recovery_reads(1, 1, 10),
            record_the_recovery_reads(2, 1, 20)
        ],
        "core 按 (实例代号, 计数器) 收：两条都留"
    );
    assert_eq!(
        the_checker_view_of_the_journal(&image),
        core_scan,
        "池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
}

// ─── C896：别的版本先走过的码 2 节点，这一版的父条目定的那几格照判 ───

/// 中央映射树叶压到 3 条、内部节点 3 个孩子：第一个文件版本的映射树长成三层；再推一次空发布，两版只共享最左那片叶。
const SMALL_CENTRAL_MAPPING: (usize, usize) = (3, 3);
/// 中央映射 key 宽（D19（块指针的结构与宽度预算） 已定项 6）：内部条目 = key 27 + 子指针 86。
const MAPPING_KEY_BYTES: usize = 27;
/// 节点指针里第 0 条位置条目的槽号：头部 50 + 设备身份 4 之后的 6 字节。
const POINTER_FIRST_LOCATION_SLOT_OFFSET: usize = 54;

/// 一版的中央映射树各节点的槽（两盘同槽，取第 0 条位置条目的）。
fn mapping_node_slots_of(version: &TransactionOutput) -> Vec<u64> {
    version
        .central_mapping_tree
        .pointers
        .iter()
        .map(|pointer| pointer.locations[0].slot.0)
        .collect()
}

/// 一条中央映射树内部条目的子指针第 0 条位置条目的槽号。
fn child_slot_of_a_mapping_internal_entry(entry: &[u8]) -> u64 {
    let start = MAPPING_KEY_BYTES + POINTER_FIRST_LOCATION_SLOT_OFFSET;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&entry[start..start + 6]);
    u64::from_le_bytes(slot_bytes)
}

/// `version` 那一版的中央映射树里指着 `shared_leaf` 的那个内部节点，把那一条的分隔 key 改成那片叶头里的最大 key（大于叶的最小 key，
/// 仍小于下一条的分隔 key），重封，新的整单元校验和沿 `version` 的引用链补到根槽与它的 journal 记录。只有 `version` 引用这个内部节点。
fn image_whose_mapping_node_of_the_version_names_a_separator_past_the_smallest_key_of(
    original: &MemoryPool,
    version: &TransactionOutput,
    shared_leaf: u64,
) -> MemoryPool {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let leaf = parse_index_node(&read_unit_on_device(original, 0, shared_leaf, node_bytes))
        .expect("共享的那片叶解得开");
    assert!(
        leaf.smallest_key < leaf.largest_key,
        "共享的那片叶装了两条以上：它头里的最大 key 大于最小 key"
    );
    for parent_slot in mapping_node_slots_of(version) {
        let before = read_unit_on_device(original, 0, parent_slot, node_bytes);
        let parent = parse_index_node(&before).expect("映射树节点解得开");
        if parent.level == 0 {
            continue;
        }
        let Some(entry) = parent
            .entries
            .iter()
            .find(|entry| child_slot_of_a_mapping_internal_entry(entry) == shared_leaf)
        else {
            continue;
        };
        let entry_start = before
            .windows(entry.len())
            .position(|window| window == entry.as_slice())
            .expect("条目在节点字节里");
        let mut after = before.clone();
        after[entry_start..entry_start + MAPPING_KEY_BYTES].copy_from_slice(&leaf.largest_key);
        reseal_unit_by_its_class(&mut after);
        let mut image = original.clone();
        write_unit_to_the_same_slot_on_both_devices(&mut image, parent_slot, &after);
        let units: Vec<(u64, usize)> = version
            .units
            .iter()
            .map(|unit| (unit.slot.0, unit.bytes.len()))
            .collect();
        propagate_the_new_checksum(
            &mut image,
            &units,
            (
                parent_slot,
                crc32_castagnoli(&before),
                crc32_castagnoli(&after),
            ),
        );
        return image;
    }
    panic!("{shared_leaf} 槽那片叶在这一版的映射树里没有父节点");
}

/// C896（审计 C01 一节「其他 schema、层级等约束存在相似早退边界」，在完整镜像上复现）：第一个文件版本与它之后的一次空发布共享中央映射树
/// 最左那片叶；旧那一版（第一个文件版本）的层级 1 节点把指着它的那一条分隔 key 改成叶头里的最大 key。最新根先走、先走过那片叶，旧那一版
/// 走到它时命中、不往下展开，这一版的分隔 key 照判：I-1.1 红（只红它）。同样的改法落在最新那一版的节点上时首次走读就判红，说明同一句。
#[test]
fn an_older_version_whose_mapping_node_names_a_separator_past_the_smallest_key_of_a_leaf_it_shares_with_the_newest_version_reddens_the_logical_address_invariant(
) {
    let tree_capacities = capacities(ACCOUNTING_OF_THE_NODE_FORMAT, SMALL_CENTRAL_MAPPING);
    let mut pool = TreeSplitPool::with_the_first_file_version_under(tree_capacities);
    pool.empty_publish(tree_capacities);
    let original = pool.memory_pool();
    assert!(
        violations_of(&check_pool_image(&original)).is_empty(),
        "改之前零违例"
    );
    let older = pool.version_before_the_current_one().clone();
    let newest_slots = mapping_node_slots_of(&pool.output);
    let shared: Vec<u64> = mapping_node_slots_of(&older)
        .into_iter()
        .filter(|slot| newest_slots.contains(slot))
        .collect();
    assert_eq!(
        shared.len(),
        1,
        "两版的中央映射树只共享一个节点：{shared:?}"
    );
    let image = image_whose_mapping_node_of_the_version_names_a_separator_past_the_smallest_key_of(
        &original, &older, shared[0],
    );
    let violations = violations_of(&check_pool_image(&image));
    let violated_invariants: Vec<&str> =
        violations.iter().map(|(invariant, _)| *invariant).collect();
    assert_eq!(
        violated_invariants,
        vec!["I-1.1"],
        "只红 I-1.1（旧那一版的分隔 key 大于共享那片叶头里的最小 key）：{violations:?}"
    );
    assert!(
        violations[0]
            .1
            .contains("的分隔 key 大于这个孩子头里的最小 key"),
        "红的是分隔 key 那一句：{violations:?}"
    );
}

// ─── C897：I-8.13（记录落在它的计数器该落的环槽） 与恢复读者的收录口径 ───

/// 一条不变量在这份镜像上的判定。
fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("清单里有")
        .1
}

/// 判 I-8.13 红、说明里点名的就是 `misplaced`（「盘 0 journal 环槽 … 的记录计数器是 …，它该落在环槽 …」那一句）。
fn assert_the_ring_slot_invariant_reddens_naming(image: &MemoryPool, misplaced: &str) {
    match verdict_of(
        image,
        JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
    ) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(misplaced),
            "I-8.13 红的是「{misplaced}」那一条：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("坐错环槽的记录要判 I-8.13 红，判成了 {other:?}")
        }
    }
}

/// 合法镜像（每条记录都落在计数器该落的环槽）：I-8.13 判过且成立，不报不适用。
#[test]
fn the_pool_built_through_the_first_file_holds_the_ring_slot_invariant() {
    let built = build_pool("checker-journal-ring-slot-legal-pool");
    assert_eq!(
        verdict_of(
            &built.memory_pool(),
            JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES
        ),
        InvariantVerdict::Holds
    );
}

/// 审计 C02 那四种排列都把计数器 1 的第二条记录放在环槽 1（计数器 1 该落环槽 0）：I-8.13 红、点名环槽 1 那一条。
const RECORD_AT_COUNTER_ONE_IN_RING_SLOT_ONE: &str =
    "盘 0 journal 环槽 1 的记录计数器是 1，它该落在环槽 0";

#[test]
fn two_records_of_one_instance_and_counter_on_one_device_redden_the_ring_slot_invariant() {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-ring-slot-same-key-txg-ten-first",
        [(1, 10), (1, 20)],
    );
    assert_the_ring_slot_invariant_reddens_naming(&image, RECORD_AT_COUNTER_ONE_IN_RING_SLOT_ONE);
}

#[test]
fn two_records_of_one_instance_and_counter_on_one_device_in_reverse_order_redden_the_ring_slot_invariant(
) {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-ring-slot-same-key-txg-twenty-first",
        [(1, 20), (1, 10)],
    );
    assert_the_ring_slot_invariant_reddens_naming(&image, RECORD_AT_COUNTER_ONE_IN_RING_SLOT_ONE);
}

#[test]
fn records_of_two_instances_at_one_counter_on_one_device_redden_the_ring_slot_invariant() {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-ring-slot-two-instances-instance-one-first",
        [(1, 10), (2, 20)],
    );
    assert_the_ring_slot_invariant_reddens_naming(&image, RECORD_AT_COUNTER_ONE_IN_RING_SLOT_ONE);
}

#[test]
fn records_of_two_instances_at_one_counter_on_one_device_in_reverse_order_redden_the_ring_slot_invariant(
) {
    let image = image_with_two_records_at_counter_one_on_device_zero(
        "checker-journal-ring-slot-two-instances-instance-two-first",
        [(2, 20), (1, 10)],
    );
    assert_the_ring_slot_invariant_reddens_naming(&image, RECORD_AT_COUNTER_ONE_IN_RING_SLOT_ONE);
}

/// mkfs 覆盖用过的池那一类：别的池（fsid 不同）留下的一条旧记录坐在环槽 5、计数器 1（按本池的落点式该落环槽 0）。它对本池不自证，
/// I-8.13 不判它，合法镜像照旧零违例。
#[test]
fn a_record_another_pool_left_in_a_ring_slot_its_counter_does_not_name_is_not_judged_by_the_ring_slot_invariant(
) {
    let built = build_pool("checker-journal-ring-slot-record-of-another-pool");
    let mut image = built.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
    let filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut record_of_another_pool = JournalRecord::parse(
        &ImageReader::read(&image, 0, ring_start, record_bytes).expect("盘 0 的环头读得出"),
        filesystem_identifier,
    )
    .expect("合法镜像环头那一槽是暖机写的第一条记录");
    record_of_another_pool.filesystem_identifier = !filesystem_identifier;
    let bytes = record_of_another_pool.to_bytes();
    assert!(
        check_journal_record(&bytes, !filesystem_identifier).is_ok(),
        "那条旧记录对它自己的池是自证过的"
    );
    let ring_slot_its_counter_does_not_name = 5;
    image.apply_writes(&[RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::JournalRecord,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(
            ring_start + ring_slot_its_counter_does_not_name * JOURNAL_RECORD_BYTES,
        ),
        contents: WrittenContents::Bytes(bytes),
    }]);
    assert!(
        violations_of(&check_pool_image(&image)).is_empty(),
        "别的池的旧记录不判：零违例"
    );
}

/// 同盘同键先到的是 core 的记录读者不收的（提交标记字节 2）、后到的是好记录：池级 checker 认的恢复视图留后到的那条、与 core 扫出的逐条相同；
/// 先到那条的提交标记字节照判 I-8.8（前缀里的事务不被切开） 红。
fn assert_the_recovery_view_skips_the_refused_record_and_its_commit_marker_still_reddens(
    image: &MemoryPool,
    record_the_core_scan_keeps: JournalRecordTheRecoveryReads,
) {
    let core_scan = the_core_scan_of_the_journal(image);
    assert_eq!(
        core_scan,
        vec![record_the_core_scan_keeps],
        "core 扫环收不进提交标记字节 2 的那条，留后到的好记录"
    );
    assert_eq!(
        the_checker_view_of_the_journal(image),
        core_scan,
        "池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
    match verdict_of(image, "I-8.8") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("提交标记字节是 2"),
            "I-8.8 红的是先到那条的提交标记字节：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("先到那条提交标记字节是 2，I-8.8 要照判红，判成了 {other:?}")
        }
    }
}

/// 计数器 1：被拒的那条坐在它该落的环槽 0（先到），好记录坐在环槽 1（后到）。
#[test]
fn a_record_the_core_reader_refuses_in_its_own_ring_slot_ahead_of_a_good_record_of_the_same_key_is_skipped_by_the_recovery_view_as_in_the_core_scan(
) {
    let image = image_with_records_on_device_zero(
        "checker-journal-refused-record-in-its-own-ring-slot",
        &[
            RecordInARingSlotOfDeviceZero {
                ring_slot: 0,
                instance: 1,
                counter: 1,
                checkpoint_txg: 10,
                written_as: HowTheRecordIsWritten::CommitMarkerByteTheCoreReaderRefuses(2),
            },
            RecordInARingSlotOfDeviceZero {
                ring_slot: 1,
                instance: 1,
                counter: 1,
                checkpoint_txg: 20,
                written_as: HowTheRecordIsWritten::AsTheFirstRecordOfThePool,
            },
        ],
    );
    assert_the_recovery_view_skips_the_refused_record_and_its_commit_marker_still_reddens(
        &image,
        record_the_recovery_reads(1, 1, 20),
    );
}

/// 计数器 2：被拒的那条坐在环槽 0（先到、不是它该落的槽），好记录坐在它该落的环槽 1（后到）。
#[test]
fn a_record_the_core_reader_refuses_off_its_ring_slot_ahead_of_a_good_record_of_the_same_key_is_skipped_by_the_recovery_view_as_in_the_core_scan(
) {
    let image = image_with_records_on_device_zero(
        "checker-journal-refused-record-off-its-ring-slot",
        &[
            RecordInARingSlotOfDeviceZero {
                ring_slot: 0,
                instance: 1,
                counter: 2,
                checkpoint_txg: 10,
                written_as: HowTheRecordIsWritten::CommitMarkerByteTheCoreReaderRefuses(2),
            },
            RecordInARingSlotOfDeviceZero {
                ring_slot: 1,
                instance: 1,
                counter: 2,
                checkpoint_txg: 20,
                written_as: HowTheRecordIsWritten::AsTheFirstRecordOfThePool,
            },
        ],
    );
    assert_the_recovery_view_skips_the_refused_record_and_its_commit_marker_still_reddens(
        &image,
        record_the_recovery_reads(1, 2, 20),
    );
}

// ─── C897 收录口径反方向的那一处：点名项 flags 字节非 0 的记录 core 收、池级 checker 的记录读者拒 ───

/// 同盘同键（实例 1、计数器 1）两条坐环槽 0 与环槽 1，一条点名项 flags 字节非 0、一条是好记录：池级 checker 认的恢复视图与 core 扫出的
/// 逐条相同（留先到的那条），坐在环槽 1 的那条判 I-8.13 红；违例集合是 `violated_invariants_expected`（改前判红的那几条一条不少）。
fn assert_the_recovery_view_keeps_what_the_core_scan_keeps_and_the_ring_slot_invariant_reddens(
    image: &MemoryPool,
    record_the_core_scan_keeps: JournalRecordTheRecoveryReads,
    violated_invariants_expected: &[&str],
) {
    let core_scan = the_core_scan_of_the_journal(image);
    assert_eq!(
        core_scan,
        vec![record_the_core_scan_keeps],
        "core 扫环跳过点名项 flags 字节照收，同键先到先留"
    );
    assert_eq!(
        the_checker_view_of_the_journal(image),
        core_scan,
        "池级 checker 认的恢复视图与 core 扫出的逐条相同"
    );
    assert_the_ring_slot_invariant_reddens_naming(image, RECORD_AT_COUNTER_ONE_IN_RING_SLOT_ONE);
    let violations = violations_of(&check_pool_image(image));
    let violated_invariants: Vec<&str> =
        violations.iter().map(|(invariant, _)| *invariant).collect();
    assert_eq!(
        violated_invariants, violated_invariants_expected,
        "违例集合：{violations:?}"
    );
}

/// 点名项 flags 字节非 0 的那条坐它该落的环槽 0（先到），好记录坐环槽 1（后到）：core 留先到的那条（txg 10）。
#[test]
fn a_record_whose_named_entry_flags_the_pool_checker_reader_refuses_ahead_of_a_good_record_of_the_same_key_is_kept_by_the_recovery_view_as_in_the_core_scan(
) {
    let image = image_with_records_on_device_zero(
        "checker-journal-named-entry-flags-record-first",
        &[
            RecordInARingSlotOfDeviceZero {
                ring_slot: 0,
                instance: 1,
                counter: 1,
                checkpoint_txg: 10,
                written_as: HowTheRecordIsWritten::NamedEntryFlagsByteThePoolCheckerReaderRefuses(
                    1,
                ),
            },
            RecordInARingSlotOfDeviceZero {
                ring_slot: 1,
                instance: 1,
                counter: 1,
                checkpoint_txg: 20,
                written_as: HowTheRecordIsWritten::AsTheFirstRecordOfThePool,
            },
        ],
    );
    assert_the_recovery_view_keeps_what_the_core_scan_keeps_and_the_ring_slot_invariant_reddens(
        &image,
        record_the_recovery_reads(1, 1, 10),
        &["I-8.13"],
    );
}

/// 好记录坐环槽 0（先到），点名项 flags 字节非 0 的那条坐环槽 1（后到、不是它该落的槽）：core 留先到的好记录（txg 10）；
/// 坐错槽的是被池级 checker 的记录读者拒的那条，它两道校验和都过、fsid 是本池的，I-8.13 照判红。
#[test]
fn a_record_whose_named_entry_flags_the_pool_checker_reader_refuses_behind_a_good_record_of_the_same_key_is_skipped_by_the_recovery_view_as_in_the_core_scan_and_reddens_the_ring_slot_invariant(
) {
    let image = image_with_records_on_device_zero(
        "checker-journal-named-entry-flags-record-second",
        &[
            RecordInARingSlotOfDeviceZero {
                ring_slot: 0,
                instance: 1,
                counter: 1,
                checkpoint_txg: 10,
                written_as: HowTheRecordIsWritten::AsTheFirstRecordOfThePool,
            },
            RecordInARingSlotOfDeviceZero {
                ring_slot: 1,
                instance: 1,
                counter: 1,
                checkpoint_txg: 20,
                written_as: HowTheRecordIsWritten::NamedEntryFlagsByteThePoolCheckerReaderRefuses(
                    1,
                ),
            },
        ],
    );
    assert_the_recovery_view_keeps_what_the_core_scan_keeps_and_the_ring_slot_invariant_reddens(
        &image,
        record_the_recovery_reads(1, 1, 10),
        &["I-8.13"],
    );
}
