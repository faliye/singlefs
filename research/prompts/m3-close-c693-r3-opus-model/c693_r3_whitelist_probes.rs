//! 攻方腿 m3-close-c693-r3 T2（草稿副本，不进主仓）：改法甲（F2 白名单草稿补丁）的两格。
//! 盘面与录法照 `common_unwitnessed_publish`（R = 第一个文件之后第 6 次覆盖写），三种盘面同第二轮正推的落点表。
//! 改法一：R 的树表单元里一条条目的一个字节改掉，节点自己的载荷校验和不重算，而根槽与 R 那条记录里的位置条目校验和照改后的字节重算
//!   （单元两条位置都读得到「与父指针校验和相符」的那份，节点自述解不开）。
//! 改法二：调查的那一形（inode 条目 flags 置一个不认识的位、校验和链整条重算），再跑 harness 的挂载透明判官 `read_recovery_fields`。
//! 每格打一行 R3PROBE，结论由报告按行归并。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, record_one_publish_after_overwrites, PublishRecordedAsR,
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
use singlefs_harness::mount_transparency::{read_recovery_fields, ComparedVersion};
use singlefs_harness::segments::StepKind;

const OVERWRITES_BEFORE_R: u64 = 5;
const FLAGS_OFFSET_IN_THE_ENTRY: usize = 12;
const PREVIOUS_SNAPSHOT_TXG_OFFSET_IN_THE_ENTRY: usize = 14 + 86;
const UNKNOWN_FLAGS_BIT: u16 = 0x0002;

#[derive(Clone, Copy, Debug)]
enum Change {
    /// 对照：读者不看的字段改一个字节、节点照头重建（两道校验和都对）。
    HarmlessFieldRebuilt,
    /// 改法一：同一个字节改掉，节点不重建（节点自述的载荷校验和不对），位置条目校验和照改后的字节重算。
    NodePayloadChecksumWrongButLocationChecksumRecomputed,
    /// 改法二：inode 条目 flags 置不认识的位、节点照头重建。
    UnknownFlagsBitRebuilt,
    /// 改法三：改法二，再让 R 的中央映射树根（R 那次发布新写的，两份）读回全零。
    UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero,
}

#[derive(Clone, Copy, Debug)]
enum Face {
    SystemConfigurationWitnessedR,
    CrashBetweenTheRootOfRAndItsRotation,
    RConfirmedButItsRotationSlotsReadBackZero,
}

fn image_and_ranges_of(recorded: &RecordedPublish, face: Face) -> (MemoryPool, RangesReadingBackAllZero) {
    match face {
        Face::SystemConfigurationWitnessedR => (
            recorded.image_with_the_whole_publish_persisted(),
            RangesReadingBackAllZero::default(),
        ),
        Face::CrashBetweenTheRootOfRAndItsRotation => (
            recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
            RangesReadingBackAllZero::default(),
        ),
        Face::RConfirmedButItsRotationSlotsReadBackZero => {
            let rotation_of_r: Vec<_> = recorded
                .writes
                .iter()
                .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
                .collect();
            assert_eq!(rotation_of_r.len(), 2);
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

/// 在 `image` 上把 R 的树表单元换成 `change` 的样子，根槽与 R 那条记录里点名它的位置条目校验和照新字节重算。
fn change_the_tree_table_of_r(image: &mut MemoryPool, recorded: &RecordedPublish, change: Change) {
    let filesystem_identifier = recorded.pool.parameters.filesystem_identifier;
    let root_slot_write = recorded.root_slot_write();
    let root_slot_bytes = root_slot_write.bytes().expect("根槽写带字节").to_vec();
    let root_of_r = RootRecord::parse_slot(&root_slot_bytes, &filesystem_identifier).expect("R 的根自证");
    assert_eq!((root_of_r.instance, root_of_r.checkpoint_txg), recorded.key);
    let location = root_of_r.tree_table.locations[0];
    let original_unit = PoolReader::read(&*image, location.device, location.slot.to_device_offset(), node_bytes())
        .expect("R 的树表单元读得到");
    assert_eq!(crc32_castagnoli(&original_unit), location.unit_checksum);
    let header = parse_index_node(&original_unit).expect("R 的树表单元解得开");
    let inode_entry_index = header
        .entries
        .iter()
        .position(|entry_bytes| TreeTableEntry::parse(entry_bytes).expect("条目解得开").kind == TREE_KIND_INODE)
        .expect("有 inode 条目");
    let rebuild = |entries: &Vec<Vec<u8>>| {
        build_index_node(
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
            entries,
        )
    };
    assert_eq!(rebuild(&header.entries), original_unit, "照头重建与写者写下的逐字节相同");
    let mut entries = header.entries.clone();
    let changed_unit = match change {
        Change::HarmlessFieldRebuilt => {
            entries[inode_entry_index][PREVIOUS_SNAPSHOT_TXG_OFFSET_IN_THE_ENTRY] ^= 1;
            rebuild(&entries)
        }
        Change::UnknownFlagsBitRebuilt | Change::UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero => {
            entries[inode_entry_index][FLAGS_OFFSET_IN_THE_ENTRY..FLAGS_OFFSET_IN_THE_ENTRY + 2]
                .copy_from_slice(&UNKNOWN_FLAGS_BIT.to_le_bytes());
            assert!(TreeTableEntry::parse(&entries[inode_entry_index]).is_none());
            rebuild(&entries)
        }
        Change::NodePayloadChecksumWrongButLocationChecksumRecomputed => {
            let entry = &header.entries[inode_entry_index];
            let entries_start = original_unit.len()
                - original_unit
                    .windows(entry.len())
                    .rev()
                    .position(|window| window == entry.as_slice())
                    .expect("条目字节在单元里")
                - entry.len();
            let mut unit = original_unit.clone();
            unit[entries_start + PREVIOUS_SNAPSHOT_TXG_OFFSET_IN_THE_ENTRY] ^= 1;
            assert!(parse_index_node(&unit).is_err(), "节点自述的载荷校验和不过");
            unit
        }
    };
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
        .write(root_slot_write.offset, &changed_root.to_slot(root_slot_bytes.len()));
    assert_eq!(recorded.record_count(), 1);
    for record_write in recorded.record_writes_of_the_record(1) {
        let mut record = JournalRecord::parse(
            record_write.bytes().expect("记录写带字节"),
            unit_filesystem_identifier(&filesystem_identifier),
        )
        .expect("R 的记录解得开");
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
                }
            }
        }
        image
            .devices
            .get_mut(&record_write.device)
            .expect("池里有这块盘")
            .write(record_write.offset, &record.to_bytes());
    }
}

fn label(recorded: &RecordedPublish, key: (singlefs_core::address::InstanceGeneration, singlefs_core::address::CheckpointTxg)) -> &'static str {
    if key == recorded.key {
        "r"
    } else if key == recorded.previous_key {
        "version_before_r"
    } else {
        "other_version"
    }
}

fn three_places(change: Change, face: Face) {
    let recorded = record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (mut image, mut ranges) = image_and_ranges_of(&recorded, face);
    change_the_tree_table_of_r(&mut image, &recorded, change);
    if let Change::UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero = change {
        let root_of_r = RootRecord::parse_slot(
            recorded.root_slot_write().bytes().expect("根槽写带字节"),
            &recorded.pool.parameters.filesystem_identifier,
        )
        .expect("R 的根自证");
        for location in &root_of_r.mapping_root.locations {
            ranges.0.push((location.device, location.slot.to_device_offset().0, NODE_BYTES));
        }
    }
    let devices = devices_reading_back_all_zero_on(&image, &ranges);
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("file_read:{}", label(&recorded, root)),
        RecoveryOutcome::NoFile { root } => format!("no_file:{}", label(&recorded, root)),
        RecoveryOutcome::Failed { failure, .. } => format!("refused:{failure:?}"),
    };
    let read_only = match mount_read_only(&devices_reading_back_all_zero_on(&image, &ranges)) {
        Ok(mounted) => format!(
            "ok:{}",
            label(&recorded, (mounted.chosen_root.instance, mounted.chosen_root.checkpoint_txg))
        ),
        Err(failure) => format!("refused:{failure:?}"),
    };
    let mut writable_devices = devices_reading_back_all_zero_on(&image, &ranges);
    let writable = match mount_writable(&recorded.pool.parameters, &mut writable_devices) {
        Ok(mounted) => format!(
            "ok:{}",
            label(&recorded, (mounted.output.chosen_root.instance, mounted.output.chosen_root.checkpoint_txg))
        ),
        Err(failure) => format!("refused:{failure:?}"),
    };
    let judge = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let fields = read_recovery_fields(&devices_reading_back_all_zero_on(&image, &ranges), ComparedVersion::TheRecoveredVersion);
        format!("{:?}", fields.report.effective_root.map(|key| label(&recorded, key)))
    }))
    .unwrap_or_else(|payload| {
        let text = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
            .unwrap_or_default();
        format!("panicked:{}", text.lines().next().unwrap_or("").chars().take(60).collect::<String>())
    });
    eprintln!(
        "R3PROBE change={change:?} face={face:?} recovery={recovery} read_only={read_only} writable={writable} mount_transparency_judge={judge}"
    );
}

macro_rules! cell {
    ($name:ident, $change:expr, $face:expr) => {
        #[test]
        fn $name() {
            three_places($change, $face);
        }
    };
}

cell!(harmless_witnessed, Change::HarmlessFieldRebuilt, Face::SystemConfigurationWitnessedR);
cell!(harmless_crash, Change::HarmlessFieldRebuilt, Face::CrashBetweenTheRootOfRAndItsRotation);
cell!(harmless_confirmed_zero, Change::HarmlessFieldRebuilt, Face::RConfirmedButItsRotationSlotsReadBackZero);
cell!(payload_checksum_witnessed, Change::NodePayloadChecksumWrongButLocationChecksumRecomputed, Face::SystemConfigurationWitnessedR);
cell!(payload_checksum_crash, Change::NodePayloadChecksumWrongButLocationChecksumRecomputed, Face::CrashBetweenTheRootOfRAndItsRotation);
cell!(payload_checksum_confirmed_zero, Change::NodePayloadChecksumWrongButLocationChecksumRecomputed, Face::RConfirmedButItsRotationSlotsReadBackZero);
cell!(unknown_flags_witnessed, Change::UnknownFlagsBitRebuilt, Face::SystemConfigurationWitnessedR);
cell!(unknown_flags_crash, Change::UnknownFlagsBitRebuilt, Face::CrashBetweenTheRootOfRAndItsRotation);
cell!(unknown_flags_confirmed_zero, Change::UnknownFlagsBitRebuilt, Face::RConfirmedButItsRotationSlotsReadBackZero);
cell!(unknown_flags_and_mapping_root_zero_crash, Change::UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero, Face::CrashBetweenTheRootOfRAndItsRotation);
cell!(unknown_flags_and_mapping_root_zero_confirmed_zero, Change::UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero, Face::RConfirmedButItsRotationSlotsReadBackZero);
cell!(unknown_flags_and_mapping_root_zero_witnessed, Change::UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero, Face::SystemConfigurationWitnessedR);
