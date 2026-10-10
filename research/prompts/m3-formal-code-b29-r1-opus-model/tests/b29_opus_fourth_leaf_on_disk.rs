//! 攻方原型（m3-formal-code-b29-r1-opus，L1）：建到第 4 片那一步，实现盘上的 inode 树根有几条条目（几片叶容器）、各片几条记录。只在攻方副本上跑。
mod common;
mod common_pointer_head_corruption;

use common_pointer_head_corruption::{node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_INODE;
use singlefs_core::unit::parse_index_node;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool};
use singlefs_core::address::DeviceIdentity;
use singlefs_harness::SharedStream;

fn leaf_record_counts(image: &MemoryPool) -> Vec<u16> {
    let root = tree_table_entry_of_kind(image, TREE_KIND_INODE).root;
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(&root), node_bytes())).unwrap();
    node.entries.iter().map(|entry| {
        let child = NodePointer::read_from(&mut ByteReader::at(entry, 34));
        let leaf = unit_bytes_at(image, child.locations[0].slot, 32768);
        u16::from_le_bytes([leaf[69], leaf[70]])
    }).collect()
}

#[test]
fn the_fourth_leaf_container_on_disk() {
    let creations = usize::try_from(3 * INODE_LEAF_RECORDS + 4).unwrap();
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(std::iter::repeat_n(HistoryOperation::CreateInode, creations));
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
    let mut snapshots: Vec<(usize, Vec<u16>)> = Vec::new();
    let stream = SharedStream::retaining_contents();
    let width = HistoryDeviceWidth::FourGibibytes;
    let run = execute_history_with(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::FourGibibytes, space_admission: SpaceAdmission::JudgedByTheFormula },
        &stream,
        &mut |observation| {
            if let StepPosition::Operation(step) = observation.position {
                if (232..=236).contains(&step) || (465..=469).contains(&step) || step >= 697 {
                    snapshots.push((step, leaf_record_counts(observation.image)));
                }
            }
        },
    );
    for (step, counts) in &snapshots {
        println!("after_step={step} leaf_containers={} records_per_leaf={counts:?}", counts.len());
    }
    println!("ending={:?}", run.ending);
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &width.fixed_geometry());
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], width.device_bytes());
    image.apply_writes(&writes);
    println!("replayed_final_image leaf_containers_records={:?}", leaf_record_counts(&image));
    let violated: Vec<&str> = singlefs_checker::walk::check_pool_image(&image).into_iter()
        .filter(|(_, v)| matches!(v, singlefs_checker::image::InvariantVerdict::Violated(_))).map(|(i, _)| i).collect();
    println!("replayed_final_image checker_violated={violated:?}");
    let recovered = singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult);
    println!("replayed_final_image effective_root={:?}", recovered.effective_root);
}
