//! 调查用（草稿目录副本，不进主仓）：判决 `research/prompts/m3-close-c693-r1-main-verification.md` 第二节 W2 那一格。
//! 最新一版 R（`common_unwitnessed_publish` 里录的那一次覆盖写；派发提示里叫 P）的树表里 inode 树那一条条目带一个今天的读者不认识的 flags 位
//! （偏移 12 的 u16 置 0x0002），R 之前一版（`previous_key`）没有。改法：照原单元的头重建树表单元（节点内两道校验和随之重算），写回两份，
//! 根槽与 R 那条记录里的树表指针、点名项的单元校验和跟着换（根槽自证与记录头校验和由 `to_slot` / `to_bytes` 重算）。
//! 三种盘面：系统配置见证了 R（整次发布落盘）、崩在 R 的根槽之后轮换之前（没见证、R 没确认）、R 已确认而 R 那次轮换写的系统配置槽读回全零（没见证、R 已确认）。
//! 两个对照：同一套重写只改一个读者不看的字段（inode 条目的 previous_snapshot_txg 加一）——证重写这一步本身三处都认；同一个字节改掉而校验和不重算——真损坏。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    record_one_publish_after_overwrites, root_is_in_the_ring, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::records::{TreeTableEntry, TREE_KIND_INODE};
use singlefs_core::root_record::RootRecord;
use singlefs_core::unit::{build_index_node, parse_index_node, unit_filesystem_identifier};
use singlefs_format::NODE_BYTES;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;

const OVERWRITES_BEFORE_R: u64 = 5;
/// 树表条目里 flags 那 2 字节的偏移：树 ID 8 + 长度 2 + 种类 2。
const FLAGS_OFFSET_IN_THE_ENTRY: usize = 12;
/// 树表条目里 previous_snapshot_txg 的偏移：flags 之后 2 字节，再过 86 字节根指针。
const PREVIOUS_SNAPSHOT_TXG_OFFSET_IN_THE_ENTRY: usize = 14 + 86;
/// 今天的读者不认识的 flags 位（位 1；码 14 上的位 0 是今天唯一认的位）。
const UNKNOWN_FLAGS_BIT: u16 = 0x0002;

#[derive(Clone, Copy, Debug)]
enum ChangeToTheTreeTableOfR {
    /// 对照：改一个读者不看的字段、校验和链整条重算。
    HarmlessFieldWithChecksumsRecomputed,
    /// inode 条目 flags 置一个不认识的位、校验和链整条重算。
    UnknownFlagsBitWithChecksumsRecomputed,
    /// inode 条目 flags 那一字节同样改掉，单元两份都改、校验和一处都不重算（真损坏）。
    SameByteChangedWithoutRecomputingChecksums,
}

#[derive(Clone, Copy, Debug)]
enum DiskFace {
    /// R 整次发布都落了：系统配置见证了 R。
    SystemConfigurationWitnessedR,
    /// 崩在 R 的根槽落盘之后、系统配置轮换之前：没见证 R，R 没确认。
    CrashBetweenTheRootOfRAndItsRotation,
    /// R 整次发布都落了，R 那次轮换写的系统配置槽（每块盘一槽）读回全零：没见证 R，R 已确认。
    RConfirmedButItsRotationSlotsReadBackZero,
}

fn image_and_ranges_of(
    recorded: &RecordedPublish,
    face: DiskFace,
) -> (MemoryPool, RangesReadingBackAllZero) {
    match face {
        DiskFace::SystemConfigurationWitnessedR => (
            recorded.image_with_the_whole_publish_persisted(),
            RangesReadingBackAllZero::default(),
        ),
        DiskFace::CrashBetweenTheRootOfRAndItsRotation => (
            recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
            RangesReadingBackAllZero::default(),
        ),
        DiskFace::RConfirmedButItsRotationSlotsReadBackZero => {
            let rotation_of_r: Vec<_> = recorded
                .writes
                .iter()
                .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
                .collect();
            assert_eq!(rotation_of_r.len(), 2, "R 那次轮换每块盘写一槽、两块盘");
            (
                recorded.image_with_the_whole_publish_persisted(),
                RangesReadingBackAllZero::default().with_the_writes(&rotation_of_r),
            )
        }
    }
}

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

/// 在 `image` 上改 R 的树表单元（两份），按 `change` 决定根槽与 R 那条记录跟不跟着换。
fn change_the_tree_table_of_r(
    image: &mut MemoryPool,
    recorded: &RecordedPublish,
    change: ChangeToTheTreeTableOfR,
) {
    let filesystem_identifier = recorded.pool.parameters.filesystem_identifier;
    let root_slot_write = recorded.root_slot_write();
    let root_slot_bytes = root_slot_write.bytes().expect("根槽写带字节").to_vec();
    let root_of_r =
        RootRecord::parse_slot(&root_slot_bytes, &filesystem_identifier).expect("R 的根自证");
    assert_eq!(
        (root_of_r.instance, root_of_r.checkpoint_txg),
        recorded.key,
        "根槽写就是 R 的根"
    );
    let location = root_of_r.tree_table.locations[0];
    let original_unit = PoolReader::read(
        &*image,
        location.device,
        location.slot.to_device_offset(),
        node_bytes(),
    )
    .expect("R 的树表单元读得到");
    assert_eq!(
        crc32_castagnoli(&original_unit),
        location.unit_checksum,
        "R 的树表单元与根上的单元校验和对得上（改之前）"
    );
    let header = parse_index_node(&original_unit).expect("R 的树表单元解得开");
    let inode_entry_index = header
        .entries
        .iter()
        .position(|entry_bytes| {
            TreeTableEntry::parse(entry_bytes).expect("写者写的条目解得开").kind == TREE_KIND_INODE
        })
        .expect("R 的树表里有 inode 树那一条");
    let mut entries = header.entries.clone();
    match change {
        ChangeToTheTreeTableOfR::HarmlessFieldWithChecksumsRecomputed => {
            let field = &mut entries[inode_entry_index]
                [PREVIOUS_SNAPSHOT_TXG_OFFSET_IN_THE_ENTRY..PREVIOUS_SNAPSHOT_TXG_OFFSET_IN_THE_ENTRY + 8];
            let value = u64::from_le_bytes(field.try_into().expect("8 字节")) + 1;
            field.copy_from_slice(&value.to_le_bytes());
            assert!(
                TreeTableEntry::parse(&entries[inode_entry_index]).is_some(),
                "对照改过的条目今天的读者照样解得开"
            );
        }
        ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed
        | ChangeToTheTreeTableOfR::SameByteChangedWithoutRecomputingChecksums => {
            assert_eq!(
                &entries[inode_entry_index][FLAGS_OFFSET_IN_THE_ENTRY..FLAGS_OFFSET_IN_THE_ENTRY + 2],
                &[0, 0],
                "R 的 inode 条目 flags 原是 0"
            );
            entries[inode_entry_index][FLAGS_OFFSET_IN_THE_ENTRY..FLAGS_OFFSET_IN_THE_ENTRY + 2]
                .copy_from_slice(&UNKNOWN_FLAGS_BIT.to_le_bytes());
            eprintln!(
                "W2PROBE_PARSE entry_with_the_unknown_flags_bit_parses={}",
                TreeTableEntry::parse(&entries[inode_entry_index]).is_some()
            );
        }
    }
    let rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽 200"),
        &header.entries,
    );
    assert_eq!(rebuilt, original_unit, "原样条目照头重建，与写者写下的逐字节相同");
    if let ChangeToTheTreeTableOfR::SameByteChangedWithoutRecomputingChecksums = change {
        let entries_start = original_unit.len()
            - original_unit
                .windows(header.entries[inode_entry_index].len())
                .rev()
                .position(|window| window == header.entries[inode_entry_index].as_slice())
                .expect("条目字节在单元里")
            - header.entries[inode_entry_index].len();
        let mut corrupted = original_unit.clone();
        corrupted[entries_start + FLAGS_OFFSET_IN_THE_ENTRY..entries_start + FLAGS_OFFSET_IN_THE_ENTRY + 2]
            .copy_from_slice(&UNKNOWN_FLAGS_BIT.to_le_bytes());
        assert!(parse_index_node(&corrupted).is_err(), "不重算校验和：节点自己的载荷校验和不过");
        for location in &root_of_r.tree_table.locations {
            image
                .devices
                .get_mut(&location.device)
                .expect("池里有这块盘")
                .write(location.slot.to_device_offset(), &corrupted);
        }
        return;
    }
    let changed_unit = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽 200"),
        &entries,
    );
    assert_ne!(changed_unit, original_unit);
    let changed_checksum = crc32_castagnoli(&changed_unit);
    let mut changed_root = root_of_r;
    for location in &mut changed_root.tree_table.locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(location.slot.to_device_offset(), &changed_unit);
        location.unit_checksum = changed_checksum;
    }
    image
        .devices
        .get_mut(&root_slot_write.device)
        .expect("池里有这块盘")
        .write(
            root_slot_write.offset,
            &changed_root.to_slot(root_slot_bytes.len()),
        );
    assert_eq!(recorded.record_count(), 1, "覆盖写那一次发布只有一条记录");
    let mut tree_table_named = 0;
    for record_write in recorded.record_writes_of_the_record(1) {
        let mut record = JournalRecord::parse(
            record_write.bytes().expect("记录写带字节"),
            unit_filesystem_identifier(&filesystem_identifier),
        )
        .expect("R 的记录解得开");
        assert_eq!(record.new_tree_table, root_of_r.tree_table, "记录新根段的树表指针就是根上那一条");
        record.new_tree_table = changed_root.tree_table;
        for named in &mut record.named {
            for named_location in &mut named.locations {
                if root_of_r
                    .tree_table
                    .locations
                    .iter()
                    .any(|location| (location.device, location.slot) == (named_location.device, named_location.slot))
                {
                    named_location.unit_checksum = changed_checksum;
                    tree_table_named += 1;
                }
            }
        }
        image
            .devices
            .get_mut(&record_write.device)
            .expect("池里有这块盘")
            .write(record_write.offset, &record.to_bytes());
    }
    eprintln!("W2PROBE_REWRITE change={change:?} tree_table_location_entries_named_in_the_records_of_r={tree_table_named}");
}

fn which_content(recorded: &RecordedPublish, content: &[u8]) -> &'static str {
    if content == recorded.content.as_slice() {
        "content_of_r"
    } else if content == recorded.previous_content.as_slice() {
        "content_of_the_version_before_r"
    } else {
        "other_content"
    }
}

fn recovery_summary(recorded: &RecordedPublish, image: &MemoryPool, ranges: &RangesReadingBackAllZero) -> String {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    let label = |key| {
        if key == recorded.key {
            "r"
        } else if key == recorded.previous_key {
            "version_before_r"
        } else {
            "other_version"
        }
    };
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!(
            "file_read root={root:?}({}) {} effective_root={:?}",
            label(root),
            which_content(recorded, &content),
            report.effective_root
        ),
        RecoveryOutcome::NoFile { root } => format!("no_file root={root:?}"),
        RecoveryOutcome::Failed { root, failure } => {
            format!("failed root={root:?} failure={failure:?}")
        }
    }
}

fn probe(change: ChangeToTheTreeTableOfR, face: DiskFace) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (mut image, ranges) = image_and_ranges_of(&recorded, face);
    change_the_tree_table_of_r(&mut image, &recorded, change);
    eprintln!(
        "W2PROBE change={change:?} face={face:?} r={:?} version_before_r={:?}",
        recorded.key, recorded.previous_key
    );
    eprintln!("W2PROBE   recovery: {}", recovery_summary(&recorded, &image, &ranges));
    let read_only_devices = devices_reading_back_all_zero_on(&image, &ranges);
    match mount_read_only(&read_only_devices) {
        Ok(mounted) => eprintln!(
            "W2PROBE   read_only_mount: ok chosen_root={:?} effective_root={:?}",
            (mounted.chosen_root.instance, mounted.chosen_root.checkpoint_txg),
            (mounted.effective_root.instance, mounted.effective_root.checkpoint_txg)
        ),
        Err(failure) => eprintln!("W2PROBE   read_only_mount: err {failure:?}"),
    }
    let mut writable_devices = devices_reading_back_all_zero_on(&image, &ranges);
    match mount_writable(&recorded.pool.parameters, &mut writable_devices) {
        Ok(mounted) => {
            eprintln!(
                "W2PROBE   writable_mount: ok chosen_root={:?} effective_root={:?} new_instance={:?} rows_written={} abandoned_roots_unreadable={} isolated_slots_per_device={:?} journal={:?}",
                (mounted.output.chosen_root.instance, mounted.output.chosen_root.checkpoint_txg),
                (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg),
                mounted.output.instance,
                mounted.output.rows_written.len(),
                mounted.output.abandoned_roots_unreadable,
                mounted.output.isolated_slots_per_device,
                mounted.output.journal
            );
            let image_after_the_mount = image_of_devices_reading_back_all_zero(
                &writable_devices,
                recorded.pool.device_bytes,
            );
            eprintln!(
                "W2PROBE   recovery_after_the_writable_mount: {} root_of_r_still_self_verified_in_the_ring={}",
                recovery_summary(&recorded, &image_after_the_mount, &RangesReadingBackAllZero::default()),
                root_is_in_the_ring(&image_after_the_mount, recorded.key)
            );
        }
        Err(failure) => {
            let unchanged = image_of_devices_reading_back_all_zero(
                &writable_devices,
                recorded.pool.device_bytes,
            ) == image;
            eprintln!("W2PROBE   writable_mount: err {failure:?} disk_unchanged={unchanged}");
        }
    }
}

#[test]
fn harmless_change_on_the_witnessed_face() {
    probe(ChangeToTheTreeTableOfR::HarmlessFieldWithChecksumsRecomputed, DiskFace::SystemConfigurationWitnessedR);
}

#[test]
fn harmless_change_on_the_crash_face() {
    probe(ChangeToTheTreeTableOfR::HarmlessFieldWithChecksumsRecomputed, DiskFace::CrashBetweenTheRootOfRAndItsRotation);
}

#[test]
fn harmless_change_on_the_confirmed_face_whose_rotation_slots_read_back_zero() {
    probe(ChangeToTheTreeTableOfR::HarmlessFieldWithChecksumsRecomputed, DiskFace::RConfirmedButItsRotationSlotsReadBackZero);
}

#[test]
fn unknown_flags_bit_on_the_witnessed_face() {
    probe(ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed, DiskFace::SystemConfigurationWitnessedR);
}

#[test]
fn unknown_flags_bit_on_the_crash_face() {
    probe(ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed, DiskFace::CrashBetweenTheRootOfRAndItsRotation);
}

#[test]
fn unknown_flags_bit_on_the_confirmed_face_whose_rotation_slots_read_back_zero() {
    probe(ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed, DiskFace::RConfirmedButItsRotationSlotsReadBackZero);
}

#[test]
fn corrupted_unit_on_the_witnessed_face() {
    probe(ChangeToTheTreeTableOfR::SameByteChangedWithoutRecomputingChecksums, DiskFace::SystemConfigurationWitnessedR);
}

#[test]
fn corrupted_unit_on_the_crash_face() {
    probe(ChangeToTheTreeTableOfR::SameByteChangedWithoutRecomputingChecksums, DiskFace::CrashBetweenTheRootOfRAndItsRotation);
}

#[test]
fn corrupted_unit_on_the_confirmed_face_whose_rotation_slots_read_back_zero() {
    probe(ChangeToTheTreeTableOfR::SameByteChangedWithoutRecomputingChecksums, DiskFace::RConfirmedButItsRotationSlotsReadBackZero);
}

/// 最小复现（钉结局）：R 已确认、R 那次轮换写的系统配置槽读回全零（没见证 R），R 的树表 inode 条目带不认识的 flags 位：
/// 可写挂载做成、落在 R 之前一版，之后恢复读出的是 R 之前一版的内容，R 的内容没了。
#[test]
fn minimal_unwitnessed_confirmed_r_with_an_unknown_flags_bit_is_mounted_writable_on_the_version_before_r() {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    assert_ne!(recorded.content, recorded.previous_content, "R 与它之前一版内容不同");
    let (mut image, ranges) =
        image_and_ranges_of(&recorded, DiskFace::RConfirmedButItsRotationSlotsReadBackZero);
    change_the_tree_table_of_r(
        &mut image,
        &recorded,
        ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed,
    );
    let mut devices = devices_reading_back_all_zero_on(&image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("没见证 R：可写挂载做成（退到 R 之前一版）");
    assert_eq!(
        (mounted.output.chosen_root.instance, mounted.output.chosen_root.checkpoint_txg),
        recorded.previous_key,
        "可写挂载落在 R 之前一版"
    );
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let report = recover(&image_after_the_mount, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead { content, .. } = report.outcome else {
        panic!("挂载之后恢复读得出文件：{:?}", report.outcome);
    };
    assert_eq!(content, recorded.previous_content, "挂载之后读出的是 R 之前一版的内容");
    assert_ne!(content, recorded.content, "R 的内容没了");
}

/// 最小复现的对照（钉结局）：同一改法、系统配置见证了 R：可写挂载在任何写之前拒，报树表条目解不开，盘上逐字节不变。
#[test]
fn minimal_witnessed_r_with_an_unknown_flags_bit_is_refused_by_the_writable_mount() {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (mut image, ranges) =
        image_and_ranges_of(&recorded, DiskFace::SystemConfigurationWitnessedR);
    change_the_tree_table_of_r(
        &mut image,
        &recorded,
        ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed,
    );
    let mut devices = devices_reading_back_all_zero_on(&image, &ranges);
    let failure = mount_writable(&recorded.pool.parameters, &mut devices)
        .err()
        .expect("见证了 R：可写挂载拒");
    assert_eq!(
        format!("{failure:?}"),
        "Recovery(UnitMalformed { what: \"树表条目\" })"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == image,
        "拒在任何写之前"
    );
}
