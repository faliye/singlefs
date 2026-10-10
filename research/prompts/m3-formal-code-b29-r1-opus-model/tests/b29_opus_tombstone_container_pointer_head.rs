//! 攻方原型（m3-formal-code-b29-r1-opus，L4 / I-2.4）：墓碑树码 2 根第 0 条条目里指向墓碑容器（码 3）的子指针，
//! 头部 extent 偏移改 1（对照：MAC 第 0 字节改 1），看池级 checker 的 I-2.4 与 core 三个入口各判什么。只在攻方副本上跑。
#![allow(clippy::unwrap_used, reason = "原型")]

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{parameters, IMAGE_BYTES};
use common_deletion_in_progress::{change_the_unit_and_propagate_its_checksum, image_after_the_intent_of_a_three_unit_file, verdict_of};
use common_pointer_head_corruption::{node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::TREE_KIND_TOMBSTONE;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const CHILD_POINTER_OFFSET_IN_THE_ENTRY: usize = 34;
const POINTER_BYTES: usize = 86;

fn corrupt(image: &mut MemoryPool, offset_in_the_pointer_head: usize, bytes: &[u8]) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_TOMBSTONE).root;
    let slot = the_slot_of(&root);
    let node = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).unwrap();
    let first = node.entries.first().unwrap().clone();
    let pointer = first[CHILD_POINTER_OFFSET_IN_THE_ENTRY..CHILD_POINTER_OFFSET_IN_THE_ENTRY + POINTER_BYTES].to_vec();
    println!("tombstone_root level={} entries={} entry_width={} child_pointer_head_bytes_28_34={:?}", node.level, node.entries.len(), node.entry_width, &pointer[28..34]);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |unit| {
        let hits: Vec<usize> = (0..=unit.len() - POINTER_BYTES).filter(|s| unit[*s..*s + POINTER_BYTES] == pointer[..]).collect();
        assert_eq!(hits.len(), 1);
        let at = hits[0] + offset_in_the_pointer_head;
        assert!(unit[at..at + bytes.len()].iter().all(|b| *b == 0));
        unit[at..at + bytes.len()].copy_from_slice(bytes);
    });
}

fn report(tag: &str, image: &MemoryPool) {
    println!("{tag} checker I-2.4={:?}", verdict_of(image, "I-2.4"));
    let violated: Vec<&str> = singlefs_checker::walk::check_pool_image(image).into_iter()
        .filter(|(_, v)| matches!(v, singlefs_checker::image::InvariantVerdict::Violated(_))).map(|(i, _)| i).collect();
    println!("{tag} checker violated={violated:?}");
    let recovered = recover(image, JournalPolicy::Consult);
    println!("{tag} recover outcome={:?} effective_root={:?}", recovered.outcome, recovered.effective_root);
    println!("{tag} mount_read_only ok={}", mount_read_only(image).is_ok());
    let mut devices: Vec<_> = image.devices.iter().map(|(identity, sparse)| {
        let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
        device.image = sparse.clone();
        (*identity, device)
    }).collect();
    match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => println!("{tag} mount_writable ok instance={:?}", mounted.output.instance),
        Err(refusal) => println!("{tag} mount_writable refused={refusal:?}"),
    }
}

#[test]
fn tombstone_container_pointer_head_fields() {
    let clean = image_after_the_intent_of_a_three_unit_file("b29-opus-tombstone-clean");
    report("clean", &clean);
    let mut extent = clean.clone();
    corrupt(&mut extent, 32, &1u16.to_le_bytes());
    report("extent_offset_1", &extent);
    let mut mac = clean.clone();
    corrupt(&mut mac, 0, &[1]);
    report("mac_byte0_1", &mac);
}
