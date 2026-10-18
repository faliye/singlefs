//! 往系统配置槽里换进一个不是格式常量 `NODE_BYTES` 的节点大小（C840 的坏镜像造法）：只改节点大小那 4 字节、整槽校验和重封，
//! magic、整槽校验和、incompat 位三关照样过，拒它的只能是节点大小那一判。读者与择系统配置那一侧
//! （`system_configuration_slot_with_a_foreign_node_size_is_not_chosen.rs`）与池级 checker 那一侧
//! （`pool_checker_reddens_only_the_tail_slot_invariant_when_every_system_configuration_slot_has_a_foreign_node_size.rs`）各 `mod` 一次。
//! 这里只造镜像、不调池级 checker。
#![allow(dead_code, reason = "两份测试文件各自 mod 进来，只用到其中一部分")]

use crate::common::parameters;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::recovery::PoolReader;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_format::{NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 系统配置槽里节点大小那 4 字节：自举头 155、整槽校验和 32、加密预留 114 之后，几何段的第一行（`layout/01-first-txn.md` 一）。
const NODE_BYTES_FIELD: std::ops::Range<usize> = 301..305;
/// 换进去的节点大小：不是格式常量 16384 的一个合法的 2 的幂。
pub const FOREIGN_NODE_BYTES: u32 = 8192;

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 一块盘上两个系统配置槽的盘内偏移：槽 0 在 0，槽 1 在槽距处（D22（单元原子性怎么合成） 已定项 16）。
pub fn system_configuration_slot_offsets() -> [u64; 2] {
    [
        0,
        u64::from(parameters().geometry.fixed_structure_slot_spacing),
    ]
}

/// `device` 上偏移 `offset` 那一个系统配置槽的整槽字节。
pub fn read_slot(image: &MemoryPool, device: u32, offset: u64) -> Vec<u8> {
    PoolReader::read(
        image,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        slot_bytes(),
    )
    .expect("系统配置槽读得到")
}

/// 把 `device` 上偏移 `offset` 那一个系统配置槽的节点大小换成 [`FOREIGN_NODE_BYTES`]、整槽校验和重封，写回去。
pub fn plant_a_foreign_node_size(image: &mut MemoryPool, device: u32, offset: u64) {
    let mut slot = read_slot(image, device, offset);
    assert_eq!(
        u64::from(u32::from_le_bytes(
            slot[NODE_BYTES_FIELD].try_into().expect("4 字节")
        )),
        NODE_BYTES,
        "盘 {device} 偏移 {offset}：mkfs 与后来的系统配置写都在偏移 301 写格式常量"
    );
    slot[NODE_BYTES_FIELD].copy_from_slice(&FOREIGN_NODE_BYTES.to_le_bytes());
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes(), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    image
        .devices
        .get_mut(&DeviceIdentity(device))
        .expect("池里有这块盘")
        .write(DeviceOffsetInBytes(offset), &slot);
}

/// 两块盘四个系统配置槽的节点大小都换成 [`FOREIGN_NODE_BYTES`]。
pub fn plant_a_foreign_node_size_into_every_slot(image: &mut MemoryPool) {
    for device in [0, 1] {
        for offset in system_configuration_slot_offsets() {
            plant_a_foreign_node_size(image, device, offset);
        }
    }
}
