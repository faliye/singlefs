//! 攻方 m3-batch33s-code-r1（Opus）S1 原型：I-3.5 折叠表在走读中途被改写（allrbmap），CPU 按每版走完时的快照取单元、GPU 按全部走完之后的
//! 落点表取单元，两边判得不同；CPU 自己随哪一版的指针坏、哪一版先走而翻面。只跑 harness 档（池级 checker 是 singlefs-checker，不是 checker 档）。
//! 盘是内存稀疏盘（SparseBlockDevice），不建镜像文件。拷进仓副本 crates/singlefs-harness/tests/ 下跑。
#![allow(dead_code, reason = "公共模块各测试只用一部分")]

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TreeTableEntry, TREE_KIND_EXTENT};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write, publish_unlink_intent,
    publish_unlink_step, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::{
    batches_of_one_data_unit, change_the_unit_and_propagate_its_checksum,
    content_of_three_data_units, FIRST_FILE, INSTANCE_OF_THE_PUBLISHES,
};
use common_pointer_head_corruption::{node_bytes, unit_bytes_at};

/// 指针里第 0 条位置条目起点、每条宽、条目里校验和的偏移（D19 已定项 7 字段表；与 checker_known_bad_images.rs 同值）。
const POINTER_FIRST_LOCATION_OFFSET: usize = 50;
const LOCATION_CHECKSUM_OFFSET: usize = 10;

struct InMemoryPool {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

/// 同 common::build_pool（mkfs → 取号 → 暖机 → 新池新建文件），盘换成内存稀疏盘。
fn build_pool_in_memory() -> InMemoryPool {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device| {
            (
                DeviceIdentity(device),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = file_content();
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm_up = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm_up.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    InMemoryPool { devices, allocator, output }
}

impl InMemoryPool {
    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self.devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }
    fn sequential_write(&mut self, content: &[u8]) -> TransactionOutput {
        let parameters = parameters();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output = publish_sequential_write(
            &mut writer,
            &mut self.allocator,
            &previous,
            FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            InstanceGeneration(1),
        )
        .expect("顺序写");
        self.output.clone()
    }
    fn intent(&mut self) -> TransactionOutput {
        let parameters = parameters();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let write_time_seconds = previous
            .inode_record
            .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds);
        self.output = publish_unlink_intent(
            &mut writer,
            &mut self.allocator,
            &previous,
            FIRST_FILE,
            write_time_seconds,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("意图");
        self.output.clone()
    }
    fn next_step(&mut self) -> TransactionOutput {
        let parameters = parameters();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output = publish_unlink_step(
            &mut writer,
            &mut self.allocator,
            &previous,
            InodeNumber(FIRST_FILE.0),
            batches_of_one_data_unit(),
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("下一批");
        self.output.clone()
    }
}

fn tree_table_slot_of(output: &TransactionOutput) -> SlotNumber {
    assert_eq!(output.root.tree_table.locations[0].slot, output.root.tree_table.locations[1].slot, "第一版两盘同槽");
    output.root.tree_table.locations[0].slot
}

/// `tree_table_slot` 那张树表里种类 `kind` 那条条目的根指针（盘上原样 86 字节）在树表单元里的起点。
fn root_pointer_start_in_the_tree_table(image: &MemoryPool, tree_table_slot: SlotNumber, kind: u16) -> (usize, NodePointer) {
    let bytes = unit_bytes_at(image, tree_table_slot, node_bytes());
    let node = parse_index_node(&bytes).expect("树表解得开");
    let entry = node
        .entries
        .iter()
        .filter_map(|entry| TreeTableEntry::parse(entry))
        .find(|entry| entry.kind == kind)
        .expect("树表里有这种树");
    let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&entry.root);
    let starts: Vec<usize> = (0..=bytes.len() - pointer_bytes.len())
        .filter(|start| bytes[*start..*start + pointer_bytes.len()] == pointer_bytes[..])
        .collect();
    assert_eq!(starts.len(), 1, "根指针在树表里恰好一处");
    (starts[0], entry.root)
}

/// 历史：新池新建文件 → 顺序写成三个单元（W）→ unlink 意图（I：extent 树照抄 W 的）→ 第一批（B：extent 树重写，最新）。
/// 交回镜像与 W、I 两版的树表槽。
fn history() -> (MemoryPool, TransactionOutput, TransactionOutput, TransactionOutput) {
    let mut pool = build_pool_in_memory();
    let written = pool.sequential_write(&content_of_three_data_units());
    assert_eq!(written.data_pointers.len(), 3);
    let intent = pool.intent();
    let batch = pool.next_step();
    (pool.image(), written, intent, batch)
}

/// 把 `owner` 那一版树表里 extent 树根指针第 0 条位置条目的校验和改成别的（单元本身不动），重封那张树表、沿引用链补到根槽：
/// 这一版读 extent 根时盘 0 那一份对不上（I-2.1 红）、读回盘 1 那一份。
fn garble_the_first_location_checksum_of_the_extent_root_pointer_of(image: &mut MemoryPool, owner: &TransactionOutput) {
    let tree_table_slot = tree_table_slot_of(owner);
    let (start, _pointer) = root_pointer_start_in_the_tree_table(image, tree_table_slot, TREE_KIND_EXTENT);
    change_the_unit_and_propagate_its_checksum(image, tree_table_slot, node_bytes(), |table| {
        let at = start + POINTER_FIRST_LOCATION_OFFSET + LOCATION_CHECKSUM_OFFSET;
        for byte in &mut table[at..at + 4] {
            *byte ^= 0x5a;
        }
    });
}

fn violated(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(identifier, _)| identifier)
        .collect()
}

fn verdict_line(arm: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let red: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(identifier, _)| *identifier)
        .collect();
    let i35 = verdicts
        .iter()
        .find(|(identifier, _)| *identifier == "I-3.5")
        .map(|(_, verdict)| match verdict {
            InvariantVerdict::Holds => "holds".to_string(),
            InvariantVerdict::Violated(detail) => format!("violated: {detail}"),
            other => format!("{other:?}").chars().take(60).collect(),
        })
        .unwrap_or_else(|| "absent".to_string());
    println!("ATTACK_ARM arm={arm} lazy_fold={} red={red:?} i35={i35}", std::env::var_os("ATTACK_LAZY_FOLD").is_some());
}

#[test]
fn attack_s1_fold_table_rewritten_mid_walk() {
    let (clean, written, intent, batch) = history();
    println!(
        "ATTACK_HISTORY written_txg={} intent_txg={} batch_txg={}",
        written.root.checkpoint_txg.0, intent.root.checkpoint_txg.0, batch.root.checkpoint_txg.0
    );
    verdict_line("clean", &clean);
    assert!(violated(&clean).is_empty(), "干净镜像零红：{:?}", violated(&clean));
    // A：意图那一版（较新、晚走）的指针坏；B：顺序写那一版（较旧）的指针坏
    let mut arm_a = clean.clone();
    garble_the_first_location_checksum_of_the_extent_root_pointer_of(&mut arm_a, &intent);
    verdict_line("A-newer-version-pointer-garbled", &arm_a);
    let mut arm_b = clean.clone();
    garble_the_first_location_checksum_of_the_extent_root_pointer_of(&mut arm_b, &written);
    verdict_line("B-older-version-pointer-garbled", &arm_b);
}

// ─── S2：空段右端那一槽按 32 KiB 读（GPU 守卫拒，CPU 稀疏读者照读「全零 16 KiB + 下一槽」） ───

/// 盘 0 单元区里一个写过的槽 w，它前一槽 w − 1 没写过：交回 w − 1（空段右端那一槽）与 w 那一槽的 16 KiB。
fn a_gap_slot_right_before_a_written_slot(image: &MemoryPool) -> (u64, Vec<u8>) {
    let device = image.devices.get(&DeviceIdentity(0)).expect("盘 0");
    let written_slots: std::collections::BTreeSet<u64> = device
        .written_sectors()
        .keys()
        .map(|sector| sector * 512 / 16384)
        .collect();
    let unit_area_start = singlefs_format::UNIT_AREA_START_SLOT;
    let w = written_slots
        .iter()
        .copied()
        .filter(|slot| *slot > unit_area_start + 1 && !written_slots.contains(&(slot - 1)))
        .last()
        .expect("有一个写过的槽前一槽没写过");
    (w - 1, device.read(singlefs_core::address::DeviceOffsetInBytes(w * 16384), 16384))
}

/// `owner` 那一版 extent 树里装着第一个文件第 0 个数据指针的那个节点：把那条数据指针第 0 条位置条目改指盘 0 的 `slot`、校验和改成 `checksum`，
/// 重封那个节点、沿引用链补到根槽。
fn redirect_the_first_location_of_the_first_data_pointer(image: &mut MemoryPool, owner: &TransactionOutput, slot: u64, checksum: u32) {
    let pointer_bytes = common_pointer_head_corruption::data_pointer_bytes(&owner.data_pointers[0]);
    let node_slots: Vec<SlotNumber> = owner
        .extent_tree
        .upper_nodes
        .iter()
        .map(|(_, pointer)| pointer.locations[0].slot)
        .chain(owner.extent_tree.lower_nodes.iter().map(|(_, pointer)| pointer.locations[0].slot))
        .collect();
    let (node_slot, start) = node_slots
        .iter()
        .find_map(|node_slot| {
            let bytes = unit_bytes_at(image, *node_slot, node_bytes());
            (0..=bytes.len() - pointer_bytes.len())
                .find(|start| bytes[*start..*start + pointer_bytes.len()] == pointer_bytes[..])
                .map(|start| (*node_slot, start))
        })
        .expect("那条数据指针在这一版 extent 树的某个节点里");
    change_the_unit_and_propagate_its_checksum(image, node_slot, node_bytes(), |node| {
        let location = start + POINTER_FIRST_LOCATION_OFFSET;
        node[location + 4..location + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
        node[location + LOCATION_CHECKSUM_OFFSET..location + LOCATION_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&checksum.to_le_bytes());
    });
}

#[test]
fn attack_s2_a_data_pointer_location_on_the_last_slot_of_a_gap() {
    let (clean, _written, _intent, batch) = history();
    let (gap_slot, next_slot_bytes) = a_gap_slot_right_before_a_written_slot(&clean);
    let mut zeros_then_next = vec![0u8; 16384];
    zeros_then_next.extend_from_slice(&next_slot_bytes);
    let crc_of_zeros_then_next = singlefs_core::checksum::crc32_castagnoli(&zeros_then_next);
    let crc_of_32k_zeros = singlefs_core::checksum::crc32_castagnoli(&vec![0u8; 32768]);
    println!("ATTACK_S2 gap_slot={gap_slot} next_slot_all_zero={}", next_slot_bytes.iter().all(|b| *b == 0));
    let mut split = clean.clone();
    redirect_the_first_location_of_the_first_data_pointer(&mut split, &batch, gap_slot, crc_of_zeros_then_next);
    verdict_line("S2-crc-of-zeros-then-next-slot", &split);
    let mut agree = clean.clone();
    redirect_the_first_location_of_the_first_data_pointer(&mut agree, &batch, gap_slot, crc_of_32k_zeros);
    verdict_line("S2-crc-of-32k-zeros", &agree);
}

// ─── S1 扫描：用户决定的那几步放开（三种历史），每一对相邻的版本、每一棵两版树表里根指针逐字相同的树，各改坏较新那一版的第 0 条校验和 ───

impl InMemoryPool {
    fn new_inodes(&mut self, count: u64) -> TransactionOutput {
        let parameters = parameters();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output = singlefs_core::transaction::publish_new_inodes(
            &mut writer,
            &mut self.allocator,
            &previous,
            count,
            common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("建空 inode");
        self.output.clone()
    }
}

/// 三种历史（新池新建文件那一版也算一版）：交回镜像与按发布次序的各版。
fn histories() -> Vec<(&'static str, MemoryPool, Vec<TransactionOutput>)> {
    let mut out = Vec::new();
    {
        let mut pool = build_pool_in_memory();
        let mut versions = vec![pool.output.clone()];
        versions.push(pool.sequential_write(&content_of_three_data_units()));
        versions.push(pool.intent());
        versions.push(pool.next_step());
        out.push(("write3-intent-batch", pool.image(), versions));
    }
    {
        let mut pool = build_pool_in_memory();
        let mut versions = vec![pool.output.clone()];
        versions.push(pool.sequential_write(&content_of_three_data_units()));
        versions.push(pool.intent());
        while !pool.output.deleted_inodes.is_empty() {
            versions.push(pool.next_step());
        }
        out.push(("write3-unlink-to-the-end", pool.image(), versions));
    }
    {
        let mut pool = build_pool_in_memory();
        let mut versions = vec![pool.output.clone()];
        versions.push(pool.sequential_write(&content_of_three_data_units()));
        for _ in 0..3 {
            versions.push(pool.new_inodes(1));
        }
        out.push(("write3-new-inodes-x3", pool.image(), versions));
    }
    out
}

fn tree_table_root_pointers(image: &MemoryPool, owner: &TransactionOutput) -> Vec<(u16, Vec<u8>)> {
    let bytes = unit_bytes_at(image, tree_table_slot_of(owner), node_bytes());
    parse_index_node(&bytes)
        .expect("树表解得开")
        .entries
        .iter()
        .filter_map(|entry| TreeTableEntry::parse(entry))
        .map(|entry| (entry.kind, common_pointer_head_corruption::node_pointer_bytes(&entry.root)))
        .collect()
}

fn garble_the_first_location_checksum_of_the_root_pointer_of(image: &mut MemoryPool, owner: &TransactionOutput, kind: u16) {
    let tree_table_slot = tree_table_slot_of(owner);
    let (start, _pointer) = root_pointer_start_in_the_tree_table(image, tree_table_slot, kind);
    change_the_unit_and_propagate_its_checksum(image, tree_table_slot, node_bytes(), |table| {
        let at = start + POINTER_FIRST_LOCATION_OFFSET + LOCATION_CHECKSUM_OFFSET;
        for byte in &mut table[at..at + 4] {
            *byte ^= 0x5a;
        }
    });
}

fn i35_of(image: &MemoryPool) -> &'static str {
    match check_pool_image(image).into_iter().find(|(identifier, _)| *identifier == "I-3.5").map(|(_, verdict)| verdict) {
        Some(InvariantVerdict::Holds) => "holds",
        Some(InvariantVerdict::Violated(_)) => "violated",
        Some(_) => "other",
        None => "absent",
    }
}

#[test]
fn attack_s1_scan_the_user_steps() {
    // 只在快照读法（不设 ATTACK_LAZY_FOLD）下跑一遍：每格同一个进程里先按快照、再按 GPU 的读法各判一次
    if std::env::var_os("ATTACK_LAZY_FOLD").is_some() {
        return;
    }
    for (history, clean, versions) in histories() {
        for pair in versions.windows(2) {
            let (older, newer) = (&pair[0], &pair[1]);
            if tree_table_slot_of(older) == tree_table_slot_of(newer) {
                continue;
            }
            let older_pointers = tree_table_root_pointers(&clean, older);
            for (kind, pointer) in tree_table_root_pointers(&clean, newer) {
                if pointer.iter().all(|byte| *byte == 0) || !older_pointers.contains(&(kind, pointer.clone())) {
                    continue;
                }
                let mut image = clean.clone();
                garble_the_first_location_checksum_of_the_root_pointer_of(&mut image, newer, kind);
                let snapshot = i35_of(&image);
                std::env::set_var("ATTACK_LAZY_FOLD", "1");
                let lazy = i35_of(&image);
                std::env::remove_var("ATTACK_LAZY_FOLD");
                println!(
                    "ATTACK_SCAN history={history} older_txg={} newer_txg={} kind={kind} snapshot_i35={snapshot} lazy_i35={lazy}",
                    older.root.checkpoint_txg.0, newer.root.checkpoint_txg.0
                );
            }
        }
        // 走读次序（快照读法下谁先走，决定哪一对会翻面）
        println!("ATTACK_ORDER_OF history={history}");
        std::env::set_var("ATTACK_ORDER_ONLY", "1");
        let _ = check_pool_image(&clean);
        std::env::remove_var("ATTACK_ORDER_ONLY");
    }
}
