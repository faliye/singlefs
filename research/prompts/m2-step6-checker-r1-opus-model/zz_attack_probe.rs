//! 攻方腿的探针（副本专用，不入库）：把干净镜像上每条根的指针摊开。
mod common;

use common::build_pool;
use singlefs_checker::image::{chosen_superblocks, parse_node_pointer, valid_roots, ImageReader};
use singlefs_harness::crash::MemoryPool;

fn dump(pool: &MemoryPool, tag: &str) {
    let chosen = chosen_superblocks(pool);
    let geometry = chosen[0].1.clone().expect("超级块").1;
    let roots = valid_roots(pool, &geometry);
    println!("=== {tag}：{} 条自证过的根", roots.len());
    for (region, slot, root) in &roots {
        let tree_table = parse_node_pointer(&root.record_bytes[36..122]);
        let mapping = parse_node_pointer(&root.record_bytes[256..342]);
        let instance_table = parse_node_pointer(&root.record_bytes[170..256]);
        println!(
            "区域 {region} 槽 {slot}：txg {} 实例 {} F {} 水位 {} | 树表全零 {} | 映射槽 {:?} 全零 {} | 实例表槽 {:?}",
            root.checkpoint_txg,
            root.instance,
            root.rollback_floor,
            root.tree_identifier_watermark,
            tree_table.all_zero,
            mapping.locations[0].slot,
            mapping.all_zero,
            instance_table.locations[0].slot,
        );
        println!(
            "    树表 loc0 = (盘 {}, 槽 {}, 校验和 {:#x})",
            tree_table.locations[0].device,
            tree_table.locations[0].slot,
            tree_table.locations[0].checksum
        );
    }
}

#[test]
#[ignore = "探针"]
fn probe_clean_pool_roots() {
    let built = build_pool("zz-probe");
    let pool = built.memory_pool();
    dump(&pool, "第一个事务之后");
    for (invariant, verdict) in singlefs_checker::walk::check_pool_image(&pool) {
        println!("{invariant}: {verdict:?}");
    }
    let _ = ImageReader::devices(&pool);
}

#[test]
#[ignore = "探针"]
fn probe_genesis_tree_table_entries() {
    let built = build_pool("zz-probe2");
    let pool = built.memory_pool();
    let unit = singlefs_checker::image::ImageReader::read(&pool, 0, 50178 * 16384, 16384)
        .expect("读 50178");
    let view = singlefs_checker::index_node_view(&unit).expect("解开 50178");
    println!(
        "50178：树 ID {} 层级 {} 条目 {}",
        view.tree_identifier,
        view.level,
        view.entries.len()
    );
    for entry in &view.entries {
        let tree = u64::from_le_bytes(entry[0..8].try_into().expect("8"));
        let kind = u16::from_le_bytes(entry[10..12].try_into().expect("2"));
        let pointer = singlefs_checker::image::parse_node_pointer(&entry[14..100]);
        println!(
            "  树 {tree} 种类 {kind}：全零 {} loc0 = (盘 {}, 槽 {})",
            pointer.all_zero, pointer.locations[0].device, pointer.locations[0].slot
        );
    }
}


#[test]
#[ignore = "探针"]
fn probe_single_copy_damage_and_genesis_tree_table_damage() {
    let built = build_pool("zz-probe4");
    // 甲：只改盘 0 的那一份数据单元（与仓里 I-2.1 那份坏镜像同形）。
    let mut one_copy = built.memory_pool();
    {
        let mut bytes = singlefs_checker::image::ImageReader::read(&one_copy, 0, 50180 * 16384, 32768)
            .expect("读数据单元");
        bytes[200] ^= 1;
        one_copy
            .devices
            .get_mut(&singlefs_core::address::DeviceIdentity(0))
            .expect("盘")
            .write(singlefs_core::address::DeviceOffsetInBytes(50180 * 16384), &bytes);
    }
    for (invariant, verdict) in singlefs_checker::walk::check_pool_image(&one_copy) {
        if matches!(invariant, "I-2.1" | "I-4.8" | "I-7.2" | "I-7.4") {
            println!("甲 {invariant}: {verdict:?}");
        }
    }
    // 乙：两盘的 mkfs 树表都改内容再重封（仓里 I-7.4 那份坏镜像）。
    let mut genesis = built.memory_pool();
    {
        let mut bytes = singlefs_checker::image::ImageReader::read(&genesis, 0, 50178 * 16384, 16384)
            .expect("读树表");
        bytes[42..50].copy_from_slice(&77u64.to_le_bytes());
        let payload_crc = singlefs_core::checksum::crc32_castagnoli(&bytes[86 + 2 * usize::from(bytes[51])..]);
        let crc_at = 76 + 2 * usize::from(bytes[51]);
        bytes[crc_at..crc_at + 4].copy_from_slice(&payload_crc.to_le_bytes());
        let header_end = 86 + 2 * usize::from(bytes[51]);
        singlefs_core::unit::seal_header_checksum(&mut bytes, header_end);
        for device in [0u32, 1] {
            genesis
                .devices
                .get_mut(&singlefs_core::address::DeviceIdentity(device))
                .expect("盘")
                .write(singlefs_core::address::DeviceOffsetInBytes(50178 * 16384), &bytes);
        }
    }
    for (invariant, verdict) in singlefs_checker::walk::check_pool_image(&genesis) {
        if matches!(invariant, "I-2.1" | "I-4.8" | "I-7.2" | "I-7.4") {
            println!("乙 {invariant}: {verdict:?}");
        }
    }
}
