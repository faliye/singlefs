//! 系统配置按 D22（单元原子性怎么合成） 已定项 26 的可改性分四类之后，**盘上字节一个都不许变**。
//!
//! 两条钉法：
//! ① 逐字节：两份固定样本写出的 4096 字节各钉一个 sha256 字面量。两个字面量是**分类之前**那一版代码
//!    打出来的（草稿副本 `zz_baseline_digests`，槽宽 4096、字段顺序照 `layout/01-first-txn.md` 一的字段表），
//!    抄进来当期望值 —— 分类只改内存表示，序列化顺序不动，所以它们必须一字不差。
//! ② 逐档字节数：`to_slot` 把每一段记在它那一档上，四档各自的字节数 = 字段表给这一档的预算
//!    （389 / 4 / 36 / 52，合计 481）。
//!
//! sha256 是 FIPS 180-4 给这个算法起的名字，不是我们的缩写；值与仓外 `sha256sum` 打出来的逐字相同。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::system_configuration::{
    SlotBytesByMutability, SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_BYTES};
use singlefs_harness::sha256::sha256_hexadecimal;

/// mkfs 刚写完那一刻的形态：世代号 1、tail 0、实例代号 0，io_min 512、环取默认 768 MiB。
fn system_configuration_at_mkfs() -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: [3u8; 16],
            this_device: DeviceIdentity(1),
            device_count: 2,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            },
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: 1,
            journal_tail: 0,
            journal_instance: InstanceGeneration(0),
        },
    }
}

/// 第一个事务发布之后那一刻的形态（E142（第一个事务的干跑） 的 fsid、世代号 5、tail 3、实例代号 1）；
/// io_min 取 4096，和上一份错开，免得两个 sha256 只差在运行量那几个字节上。
fn system_configuration_after_the_first_transaction() -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: [
                0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31,
                0x2d, 0x00,
            ],
            this_device: DeviceIdentity(0),
            device_count: 2,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 4096,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: 805_306_368,
            },
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: 5,
            journal_tail: 3,
            journal_instance: InstanceGeneration(1),
        },
    }
}

/// 分类之前那一版代码打出来的两个摘要，抄进来当期望值。
const SLOT_DIGEST_AT_MKFS: &str =
    "666e95617f8902f6b69320000fa87d359e33e37bd9cfed5c5b82b234ee2c134c";
const SLOT_DIGEST_AFTER_THE_FIRST_TRANSACTION: &str =
    "5d3e773bf0a429fc4e48f58790f79b305a23beaaeeed69700d73d90835c623f9";

#[test]
fn splitting_the_system_configuration_into_four_mutability_classes_changes_no_byte_on_disk() {
    let at_mkfs = system_configuration_at_mkfs().to_slot();
    assert_eq!(at_mkfs.len(), 4096, "槽宽不变");
    assert_eq!(
        sha256_hexadecimal(&at_mkfs),
        SLOT_DIGEST_AT_MKFS,
        "mkfs 那一刻的槽，分四类前后逐字节相同"
    );

    let after = system_configuration_after_the_first_transaction().to_slot();
    assert_eq!(after.len(), 4096);
    assert_eq!(
        sha256_hexadecimal(&after),
        SLOT_DIGEST_AFTER_THE_FIRST_TRANSACTION,
        "第一个事务发布之后的槽，分四类前后逐字节相同"
    );
    assert_ne!(
        SLOT_DIGEST_AT_MKFS, SLOT_DIGEST_AFTER_THE_FIRST_TRANSACTION,
        "两份样本本来就该不同：钉的是两组字节，不是同一组抄了两遍"
    );

    for slot in [&at_mkfs, &after] {
        let padding_start = usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("481");
        assert!(
            slot[padding_start..].iter().all(|byte| *byte == 0),
            "481 之后仍然全是补齐 0：四类合计没有多写出一个字节"
        );
    }
}

/// 四类在槽里是交错的（槽世代号与整槽校验和夹在自举头中间、节点大小夹在几何段开头），
/// 所以「某一档写了多少字节」不是按偏移区间量的，是 `to_slot` 一路记出来的。
#[test]
fn each_mutability_class_writes_its_own_field_table_budget_389_4_36_52() {
    let expected = SlotBytesByMutability {
        immutable_configuration_bytes: 389,
        mutable_configuration_bytes: 4,
        runtime_configuration_bytes: 36,
        runtime_quantity_bytes: 52,
    };
    for system_configuration in [
        system_configuration_at_mkfs(),
        system_configuration_after_the_first_transaction(),
    ] {
        let (bytes, accounting) = system_configuration.to_slot_with_mutability_accounting();
        assert_eq!(accounting, expected, "四档各自的字节数与字段表不符");
        assert_eq!(bytes.len(), 4096);
    }
    assert_eq!(
        expected.immutable_configuration_bytes
            + expected.mutable_configuration_bytes
            + expected.runtime_configuration_bytes
            + expected.runtime_quantity_bytes,
        SYSTEM_CONFIGURATION_BYTES,
        "389 + 4 + 36 + 52 = 字段表 45 行的 481"
    );
}
