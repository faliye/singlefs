//! 正推腿 m3-batch23-code-r1 的 CPU 侧探针（harness 档，只读冻结副本、只在 rerun.sh 拷出来的副本里编，不改 crates/）。
//! 每个探针造一份镜像、跑池级 checker `singlefs_checker::walk::check_pool_image`，把判定打成 `ZS_PROBE <标签> …` 行。
//! 目的：用 CPU 判器算出 GPU 对拍用例该红哪几条（GPU 不跑），与报告里的推导逐行对。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, parameters};
use common_deletion_in_progress as c;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::records::{
    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
    TREE_KIND_TOMBSTONE,
};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_harness::memory_pool::MemoryPool;

const WATCHED: [&str; 16] = [
    "I-9.4", "I-9.5", "I-9.10", "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23",
    "I-9.24", "I-9.29", "I-3.4", "I-4.7", "I-4.6", "I-4.4",
];
/// 节点（码 2）里第一条条目的起点：头 115 + 2 × key 宽 8。
const FIRST_ENTRY_OF_A_CONTAINER_NUMBER_NODE: usize = 131;

fn kind_of(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "Holds",
        InvariantVerdict::Violated(_) => "Violated",
        InvariantVerdict::NotApplicable(_) => "NotApplicable",
        InvariantVerdict::PartiallyJudged(_) => "PartiallyJudged",
    }
}

/// 打一行红了哪几条，再逐条打被盯的 16 条各自的判定种类。
fn show(label: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    println!("ZS_PROBE {label} violated={violated:?}");
    for (name, verdict) in &verdicts {
        if WATCHED.contains(name) {
            println!("ZS_PROBE {label} {name} {}", kind_of(verdict));
        }
    }
}

/// 恢复（core 冷走读）在这份镜像上的结局，打成 `ZS_PROBE <标签> recovery=…`。
fn show_recovery(label: &str, image: &MemoryPool) {
    let report = recover(image, JournalPolicy::Consult);
    let outcome = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        RecoveryOutcome::FileRead { .. } => "FileRead".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("Failed({failure:?})"),
    };
    println!("ZS_PROBE {label} recovery={outcome}");
}

/// N6：新池新建文件写完的干净镜像（全部持久那个状态）与 mkfs 刚写完的镜像。
#[test]
fn zs_n6_new_pool_file_creation_image_and_the_image_after_mkfs() {
    let built = build_pool("zs-n6");
    show("N6_full_persisted", &built.memory_pool());
    show("N6_after_mkfs", &built.memory_pool_after_mkfs());
}

/// T13（删 → 回退 → 再删）最终镜像：九条一条不红。
#[test]
fn zs_t13_unlink_rolled_back_and_unlinked_again() {
    let mut pool = c::pool_with_new_inodes("zs-t13", 1);
    let before_the_intent = pool.output.clone();
    c::unlink_to_the_end(&mut pool, c::FIRST_FILE.0 + 1);
    c::roll_back_to(&mut pool, &before_the_intent);
    c::unlink_to_the_end(&mut pool, c::FIRST_FILE.0 + 1);
    show("T13", &pool.memory_pool());
}

/// T9：墓碑容器旋钮压到一片 1 条，两次 unlink；旧入口当没装旋钮，红 I-9.21。
#[test]
fn zs_t9_two_empty_inodes_unlinked_one_tombstone_per_container() {
    let mut pool = c::pool_with_new_inodes("zs-t9", 2);
    let capacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    for inode in [c::FIRST_FILE.0 + 1, c::FIRST_FILE.0 + 2] {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        writer.set_container_tree_capacities(capacities);
        let mut current = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            InodeNumber(inode),
            c::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            c::INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("意图");
        publish_unlink_steps_until_the_queue_is_empty(
            &mut writer,
            &mut pool.allocator,
            &mut current,
            UnlinkBatchSize::OneExtentLowerLeaf,
            c::INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("各批");
        pool.output = current;
    }
    show("T9", &pool.memory_pool());
}

/// T10：同一片 inode 叶里 1、2 号两条记录对调。
#[test]
fn zs_t10_two_records_of_the_inode_leaf_swapped() {
    let pool = c::pool_with_new_inodes("zs-t10", 1);
    let mut image = pool.memory_pool();
    let slot = c::first_container_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_INODE);
    show("T10_clean", &image);
    c::change_the_unit_and_propagate_its_checksum(&mut image, slot, 32768, |leaf| {
        assert_eq!(u16::from_le_bytes([leaf[69], leaf[70]]), 2, "两条记录");
        let first: Vec<u8> = leaf[136..276].to_vec();
        let second: Vec<u8> = leaf[276..416].to_vec();
        leaf[136..276].copy_from_slice(&second);
        leaf[276..416].copy_from_slice(&first);
    });
    show("T10_swapped", &image);
}

/// T4 T6 T7 T8 T5 T2 T3：第一批那一版镜像上各改一处。
#[test]
fn zs_first_batch_image_t4_t6_t7_t8_t5_t2_t3() {
    let clean = c::image_after_the_first_batch_of_a_three_unit_file("zs-fb");
    show("FB_clean", &clean);
    let mut image = clean.clone();
    c::change_the_first_tombstone_of_the_newest_root(&mut image, |t| {
        t[0..8].copy_from_slice(&0u64.to_le_bytes());
    });
    show("T4_object_zero", &image);
    let mut image = clean.clone();
    c::change_the_first_tombstone_of_the_newest_root(&mut image, |t| {
        t[32..40].copy_from_slice(&0u64.to_le_bytes());
    });
    show("T6_death_txg_zero", &image);
    let mut image = clean.clone();
    c::append_a_copy_of_the_first_tombstone_of_the_newest_root(&mut image);
    show("T7_written_twice", &image);
    let mut image = clean.clone();
    c::change_the_first_tombstone_of_the_newest_root(&mut image, |t| {
        let birth = u64::from_le_bytes(t[8..16].try_into().expect("8")) + 1;
        t[8..16].copy_from_slice(&birth.to_le_bytes());
    });
    show("T8_object_birth_plus_one", &image);
    let mut image = clean.clone();
    c::set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
    show("T5_nlink_one", &image);
    for (label, entry_type) in [("T2_entry_type_seven", 7u16), ("T3_entry_type_zero", 0u16)] {
        let mut image = clean.clone();
        let slot = c::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_TOMBSTONE);
        c::change_the_unit_and_propagate_its_checksum(&mut image, slot, 16384, |root| {
            let entry = FIRST_ENTRY_OF_A_CONTAINER_NUMBER_NODE;
            root[entry + 16..entry + 18].copy_from_slice(&entry_type.to_le_bytes());
        });
        show(label, &image);
    }
}

/// T3b（线索）：墓碑树根第一条条目类型段写 0、身份引用三段也清零，孩子仍是码 3 容器。kb 说「0 ⇒ 子单元是码 2」，这一条该红 I-9.23。
#[test]
fn zs_t3b_type_zero_with_zero_identity_over_a_code_three_container() {
    let clean = c::image_after_the_first_batch_of_a_three_unit_file("zs-t3b");
    let mut image = clean.clone();
    let slot = c::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_TOMBSTONE);
    c::change_the_unit_and_propagate_its_checksum(&mut image, slot, 16384, |root| {
        let entry = FIRST_ENTRY_OF_A_CONTAINER_NUMBER_NODE;
        root[entry + 8..entry + 16].fill(0);
        root[entry + 16..entry + 18].copy_from_slice(&0u16.to_le_bytes());
        root[entry + 18..entry + 34].fill(0);
    });
    show("T3b_type_zero_zero_identity_child_is_a_container", &image);
}

/// T1：回退之后谱系树根第一条条目身份引用里的容器号加 1。
#[test]
fn zs_t1_lineage_root_entry_names_another_container() {
    let mut pool = c::pool_with_new_inodes("zs-t1", 1);
    let before = pool.output.clone();
    c::unlink_to_the_end(&mut pool, c::FIRST_FILE.0 + 1);
    c::roll_back_to(&mut pool, &before);
    let mut image = pool.memory_pool();
    show("T1_clean", &image);
    let slot = c::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_LINEAGE);
    c::change_the_unit_and_propagate_its_checksum(&mut image, slot, 16384, |root| {
        let field = FIRST_ENTRY_OF_A_CONTAINER_NUMBER_NODE + 18;
        let value = u64::from_le_bytes(root[field..field + 8].try_into().expect("8")) + 1;
        root[field..field + 8].copy_from_slice(&value.to_le_bytes());
    });
    show("T1_container_plus_one", &image);
}

/// T11 T12：删到最后一批之后零条目的 extent 上段根（条目宽 113 → 24）与 inode 树根（120 → 8）。
#[test]
fn zs_t11_t12_narrow_empty_roots_after_the_last_batch() {
    let mut pool = c::pool_with_a_three_unit_file("zs-t11");
    c::publish_the_intent(&mut pool);
    for _batch in 0..3 {
        c::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let clean = pool.memory_pool();
    show("LB_clean", &clean);
    show_recovery("LB_clean", &clean);
    for (label, kind, width) in [
        ("T11_extent_root_width_24", TREE_KIND_EXTENT, 24u16),
        ("T12_inode_root_width_8", TREE_KIND_INODE, 8u16),
    ] {
        let mut image = clean.clone();
        let slot = c::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, kind);
        c::change_the_unit_and_propagate_its_checksum(&mut image, slot, 16384, |root| {
            let key_width = usize::from(root[51]);
            let count_offset = 82 + 2 * key_width;
            assert_eq!(
                u16::from_le_bytes([root[count_offset], root[count_offset + 1]]),
                0,
                "零条目"
            );
            let width_offset = 86 + 2 * key_width - 2;
            root[width_offset..width_offset + 2].copy_from_slice(&width.to_le_bytes());
        });
        let violated = c::violated_invariants(&image);
        println!("ZS_PROBE {label} violated={violated:?}");
        show_recovery(label, &image);
    }
}

/// 队列 {1} → {1, 7}（GPU 对拍里被多断言 I-9.24 的那一条）：条目数 +1、声明长度 +8、头里最大 key 改成 7。
#[test]
fn zs_queue_one_and_seven_on_the_first_batch_image() {
    let clean = c::image_after_the_first_batch_of_a_three_unit_file("zs-q17");
    let mut image = clean.clone();
    let slot = c::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
    c::change_the_unit_and_propagate_its_checksum(&mut image, slot, 16384, |root| {
        let count_offset = 82 + 2 * 8;
        let entries_start = 115 + 2 * 8;
        let count = usize::from(u16::from_le_bytes([root[count_offset], root[count_offset + 1]]));
        assert_eq!(count, 1, "队列 {{1}}");
        root[entries_start + 8..entries_start + 16].copy_from_slice(&7u64.to_le_bytes());
        root[count_offset..count_offset + 2].copy_from_slice(&2u16.to_le_bytes());
        let declared = u16::from_le_bytes([root[8], root[9]]) + 8;
        root[8..10].copy_from_slice(&declared.to_le_bytes());
        root[52 + 8..52 + 16].copy_from_slice(&7u64.to_le_bytes());
    });
    show("Q17", &image);
}

const SLOT_BYTES: u64 = 16384;

fn read_bytes(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(pool, DeviceIdentity(device), DeviceOffsetInBytes(offset), length).expect("读")
}

/// 根环全部槽：区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096，每区 8 槽（`common::parameters`）。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| {
                (
                    [0u32, 1, 0][usize::try_from(region).expect("区域")],
                    64 * SLOT_BYTES + region * 3 * (1 << 20) + slot * 4096,
                )
            })
        })
        .collect()
}

/// N5 探针：干净镜像上最新根指着的实例表单元两份各翻一个载荷字节（不重封）。池里没有被抛弃的根，
/// kb I-9.29 字面是「读不出 且 被抛弃的根滤不掉」时记判不了；这里看实现在没有被抛弃的根时报什么。
#[test]
fn zs_n5_clean_image_whose_newest_instance_table_does_not_read_back() {
    let built = build_pool("zs-n5");
    let clean = built.memory_pool();
    show("N5_clean", &clean);
    let newest_root = root_slots()
        .into_iter()
        .map(|(device, offset)| read_bytes(&clean, device, offset, 512))
        .filter(|root| &root[..4] == b"SFSR")
        .max_by_key(|root| {
            (
                u64::from_le_bytes(root[28..36].try_into().expect("8")),
                u32::from_le_bytes(root[24..28].try_into().expect("4")),
            )
        })
        .expect("根环里有根");
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&newest_root[170 + 50 + 4..170 + 50 + 4 + 6]);
    let slot = u64::from_le_bytes(slot_bytes);
    let mut image = clean.clone();
    for device in [0u32, 1] {
        let mut bytes = read_bytes(&image, device, slot * SLOT_BYTES, 32768);
        bytes[32768 - 100] ^= 0xff;
        image
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("盘")
            .write(DeviceOffsetInBytes(slot * SLOT_BYTES), &bytes);
    }
    show("N5_instance_table_broken", &image);
    let verdicts = check_pool_image(&image);
    for (name, verdict) in &verdicts {
        if *name == "I-9.29" {
            if let InvariantVerdict::PartiallyJudged(unjudged) = verdict {
                println!("ZS_PROBE N5_instance_table_broken I-9.29 text={unjudged}");
            }
        }
    }
}
