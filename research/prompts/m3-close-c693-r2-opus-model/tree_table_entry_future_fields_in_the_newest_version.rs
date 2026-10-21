//! m3-close-c693-r2 攻方腿草稿（仓副本，不进主仓）：调查用测试 `tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version.rs`
//! （调查员草稿目录，sha256 89c6b0b7…）的搭建原样沿用，只把「改 R 的树表 inode 条目」换成几种「未来写者合法地写下、今天的读者不认」的字段：
//! 预留 76 字节里写一个非零值（D8 已定项 8 预留认购表的第一行：livelist 每头计数器住头条目）、条目长度字段改成别的值（D8 已定项 8 ④ 的条目自带长度），
//! 与不认识的 flags 位（调查已量过的那一形，作对照）。三种盘面照调查。只看三处落到哪一版、R 的内容还在不在。
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

/// 树表条目里条目长度那 2 字节的偏移：树 ID 8 之后。
const LENGTH_OFFSET_IN_THE_ENTRY: usize = 8;
/// 树表条目里预留 76 字节的起点：200 − 76。
const RESERVED_OFFSET_IN_THE_ENTRY: usize = 200 - 76;
#[derive(Clone, Copy, Debug)]
enum ChangeToTheTreeTableOfR {
    /// 对照：改一个读者不看的字段（previous_snapshot_txg 加一）、校验和链整条重算。
    HarmlessFieldWithChecksumsRecomputed,
    /// inode 条目 flags 置一个不认识的位（调查量过的那一形）。
    UnknownFlagsBitWithChecksumsRecomputed,
    /// inode 条目预留 76 字节的第一个字节写 1（未来写者照认购表写 livelist 每头计数器），校验和链整条重算。
    ReservedBytesSubscribedWithChecksumsRecomputed,
    /// inode 条目的条目长度字段（偏移 8 的 u16）写成 208（未来写者照 ④ 加宽条目时这一字段的取值），校验和链整条重算。
    EntryLengthFieldWiderWithChecksumsRecomputed,
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
        }
        ChangeToTheTreeTableOfR::UnknownFlagsBitWithChecksumsRecomputed => {
            entries[inode_entry_index][FLAGS_OFFSET_IN_THE_ENTRY..FLAGS_OFFSET_IN_THE_ENTRY + 2]
                .copy_from_slice(&UNKNOWN_FLAGS_BIT.to_le_bytes());
        }
        ChangeToTheTreeTableOfR::ReservedBytesSubscribedWithChecksumsRecomputed => {
            assert_eq!(entries[inode_entry_index][RESERVED_OFFSET_IN_THE_ENTRY], 0, "预留原是 0");
            entries[inode_entry_index][RESERVED_OFFSET_IN_THE_ENTRY] = 1;
        }
        ChangeToTheTreeTableOfR::EntryLengthFieldWiderWithChecksumsRecomputed => {
            entries[inode_entry_index][LENGTH_OFFSET_IN_THE_ENTRY..LENGTH_OFFSET_IN_THE_ENTRY + 2]
                .copy_from_slice(&208u16.to_le_bytes());
        }
    }
    eprintln!(
        "R2PROBE_PARSE change={change:?} entry_parses_today={}",
        TreeTableEntry::parse(&entries[inode_entry_index]).is_some()
    );
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
    eprintln!("R2PROBE_REWRITE change={change:?} tree_table_location_entries_named_in_the_records_of_r={tree_table_named}");
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
        "R2PROBE change={change:?} face={face:?} r={:?} version_before_r={:?}",
        recorded.key, recorded.previous_key
    );
    eprintln!("R2PROBE   recovery: {}", recovery_summary(&recorded, &image, &ranges));
    let read_only_devices = devices_reading_back_all_zero_on(&image, &ranges);
    match mount_read_only(&read_only_devices) {
        Ok(mounted) => eprintln!(
            "R2PROBE   read_only_mount: ok chosen_root={:?} effective_root={:?}",
            (mounted.chosen_root.instance, mounted.chosen_root.checkpoint_txg),
            (mounted.effective_root.instance, mounted.effective_root.checkpoint_txg)
        ),
        Err(failure) => eprintln!("R2PROBE   read_only_mount: err {failure:?}"),
    }
    let mut writable_devices = devices_reading_back_all_zero_on(&image, &ranges);
    match mount_writable(&recorded.pool.parameters, &mut writable_devices) {
        Ok(mounted) => {
            eprintln!(
                "R2PROBE   writable_mount: ok chosen_root={:?} effective_root={:?} new_instance={:?} rows_written={} abandoned_roots_unreadable={} isolated_slots_per_device={:?} journal={:?}",
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
                "R2PROBE   recovery_after_the_writable_mount: {} root_of_r_still_self_verified_in_the_ring={}",
                recovery_summary(&recorded, &image_after_the_mount, &RangesReadingBackAllZero::default()),
                root_is_in_the_ring(&image_after_the_mount, recorded.key)
            );
        }
        Err(failure) => {
            let unchanged = image_of_devices_reading_back_all_zero(
                &writable_devices,
                recorded.pool.device_bytes,
            ) == image;
            eprintln!("R2PROBE   writable_mount: err {failure:?} disk_unchanged={unchanged}");
        }
    }
}

#[test]
fn harmless_field_on_the_witnessed_face() {
    probe(ChangeToTheTreeTableOfR::HarmlessFieldWithChecksumsRecomputed, DiskFace::SystemConfigurationWitnessedR);
}

#[test]
fn harmless_field_on_the_crash_face() {
    probe(ChangeToTheTreeTableOfR::HarmlessFieldWithChecksumsRecomputed, DiskFace::CrashBetweenTheRootOfRAndItsRotation);
}

#[test]
fn harmless_field_on_the_confirmed_face_whose_rotation_slots_read_back_zero() {
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
fn subscribed_reserved_bytes_on_the_witnessed_face() {
    probe(ChangeToTheTreeTableOfR::ReservedBytesSubscribedWithChecksumsRecomputed, DiskFace::SystemConfigurationWitnessedR);
}

#[test]
fn subscribed_reserved_bytes_on_the_crash_face() {
    probe(ChangeToTheTreeTableOfR::ReservedBytesSubscribedWithChecksumsRecomputed, DiskFace::CrashBetweenTheRootOfRAndItsRotation);
}

#[test]
fn subscribed_reserved_bytes_on_the_confirmed_face_whose_rotation_slots_read_back_zero() {
    probe(ChangeToTheTreeTableOfR::ReservedBytesSubscribedWithChecksumsRecomputed, DiskFace::RConfirmedButItsRotationSlotsReadBackZero);
}

#[test]
fn wider_entry_length_field_on_the_witnessed_face() {
    probe(ChangeToTheTreeTableOfR::EntryLengthFieldWiderWithChecksumsRecomputed, DiskFace::SystemConfigurationWitnessedR);
}

#[test]
fn wider_entry_length_field_on_the_crash_face() {
    probe(ChangeToTheTreeTableOfR::EntryLengthFieldWiderWithChecksumsRecomputed, DiskFace::CrashBetweenTheRootOfRAndItsRotation);
}

#[test]
fn wider_entry_length_field_on_the_confirmed_face_whose_rotation_slots_read_back_zero() {
    probe(ChangeToTheTreeTableOfR::EntryLengthFieldWiderWithChecksumsRecomputed, DiskFace::RConfirmedButItsRotationSlotsReadBackZero);
}
