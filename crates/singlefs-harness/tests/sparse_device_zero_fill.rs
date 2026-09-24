//! 稀疏设备的整段清零不许把扇区实体化。
//!
//! 2026-09-22 实测的代价：mkfs 一次清 768 MiB 的 journal 环（`make_filesystem.rs` 的
//! `write_zeroes_at`）。按「插进 length / 512 个全 0 扇区」写，每块盘就是 1 572 864 条
//! 512 字节的项，连 `Vec` 头与 `BTreeMap` 节点每条 600 字节以上 ⇒ 每盘约 940 MB、
//! 每份 `MemoryPool` 约 1.9 GB；`MemoryPool` 派生了 `Clone`、每个崩溃点观测各拷一份，
//! 再乘上 libtest 默认按核数并行，三分钟把 27 GiB 可用内存吃光、触发 OOM。
//!
//! 稀疏设备里「没记着的扇区」读出来就是全 0，所以删掉与插满读回的字节逐位相同——
//! 这条用例钉的是**两种写法在内存与「盘上哪里有东西」这两件事上不同**。

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_harness::crash::{SparseDevice, SECTOR_BYTES};

/// 与 `JOURNAL_RING_DEFAULT_BYTES` 同一个数：mkfs 真正要清的那一段。
const RING_BYTES: u64 = 768 * 1024 * 1024;

#[test]
fn zero_filling_a_range_removes_its_sectors_instead_of_materialising_them() {
    let mut device = SparseDevice::default();
    let sector = usize::try_from(SECTOR_BYTES).expect("512");

    // 先在环里写两个扇区（一头一尾），再写一个环外的：清零只许动环里那两个。
    device.write(DeviceOffsetInBytes(0), &vec![7u8; sector]);
    device.write(
        DeviceOffsetInBytes(RING_BYTES - SECTOR_BYTES),
        &vec![9u8; sector],
    );
    device.write(DeviceOffsetInBytes(RING_BYTES), &vec![5u8; sector]);
    assert_eq!(
        device
            .written_sectors_in(DeviceOffsetInBytes(0), RING_BYTES)
            .len(),
        2,
        "清零之前环里记着两个扇区"
    );

    device.zero_fill(DeviceOffsetInBytes(0), RING_BYTES);

    assert_eq!(
        device.written_sectors_in(DeviceOffsetInBytes(0), RING_BYTES),
        Vec::<u64>::new(),
        "清了 768 MiB 之后这一段一个扇区都不许记着：插满就是每块盘 150 万条项、约 940 MB，\
         乘上 MemoryPool 的 clone 与 libtest 的并行会把机器吃到 OOM（2026-09-22 实测）"
    );
    assert_eq!(
        device
            .written_sectors_in(DeviceOffsetInBytes(RING_BYTES), SECTOR_BYTES)
            .len(),
        1,
        "环外那个扇区不许被清掉：清零的射程就是给它的那一段"
    );
    assert_eq!(
        device.read(DeviceOffsetInBytes(0), sector),
        vec![0u8; sector],
        "读回来仍是全 0：删掉与插满这两种写法读回的字节必须逐位相同"
    );
    assert_eq!(
        device.read(DeviceOffsetInBytes(RING_BYTES), sector),
        vec![5u8; sector],
        "环外那个扇区的内容原样还在"
    );
}
