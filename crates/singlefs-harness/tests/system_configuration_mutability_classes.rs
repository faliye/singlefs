//! 系统配置按 D22（单元原子性怎么合成） 已定项 26 的可改性分四类之后，**盘上字节一个都不许变**。
//!
//! 两条钉法：
//! ① 逐字节：两份固定样本写出的 4096 字节各钉一个 sha256 字面量。两个字面量最初是**分类之前**那一版代码
//!    打出来的（草稿副本 `zz_baseline_digests`，槽宽 4096、字段顺序照 `layout/01-first-txn.md` 一的字段表），
//!    抄进来当期望值 —— 分类只改内存表示，序列化顺序不动，所以它们必须一字不差；2026-09-26 格式改了一次
//!    （回退下界 F 与 incompat 位 1），字面量按新代码重打，差在哪几个字节写在字面量旁边。
//! ② 逐档字节数：`to_slot` 把每一段记在它那一档上，四档各自的字节数 = 字段表给这一档的预算
//!    （389 / 4 / 36 / 60，合计 489）。
//!
//! sha256 是 FIPS 180-4 给这个算法起的名字，不是我们的缩写；值与仓外 `sha256sum` 打出来的逐字相同。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SlotBytesByMutability, SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
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
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
            root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: 1,
            journal_tail: 0,
            journal_instance: InstanceGeneration(0),
            rollback_floor: CheckpointTxg(0),
        },
    }
}

/// 新池新建文件发布之后那一刻的形态（E142（新池新建文件的干跑） 的 fsid、世代号 5、tail 3、实例代号 1）；
/// io_min 取 4096，和上一份错开，免得两个 sha256 只差在运行量那几个字节上。
fn system_configuration_after_the_new_pool_file_creation() -> SystemConfiguration {
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
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
            root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: 5,
            journal_tail: 3,
            journal_instance: InstanceGeneration(1),
            rollback_floor: CheckpointTxg(0),
        },
    }
}

/// 两份样本的摘要。分类之前那一版代码打出来的是 `666e9561…` 与 `5d3e773b…`；2026-09-26 回退改形态第一批
/// （字段表末尾加回退下界 F 8 字节、incompat 布局身份从位 0 换到位 1，D22 已定项 9、D15 已定项 4）之后按新代码重打，
/// 与旧字节逐字节比过只差 6 个字节：偏移 6（incompat 位图第一个字节 0x01 → 0x02）与 [155, 159)（整槽校验和的 CRC-32C 那 4 字节）；
/// F 在这两份样本里是 0，落在原来就是补齐 0 的 [481, 489)，一个字节都没变。
const SLOT_DIGEST_AT_MKFS: &str =
    "a065c12485a9ff72117d549852461f5cb4529d664ac2384271f6e0f22d0f167f";
const SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION: &str =
    "c67464f93c9021cf354d649f37044041e4b5eec252f23d103a7e8c1328796260";

#[test]
fn splitting_the_system_configuration_into_four_mutability_classes_changes_no_byte_on_disk() {
    let at_mkfs = system_configuration_at_mkfs().to_slot();
    assert_eq!(at_mkfs.len(), 4096, "槽宽不变");
    assert_eq!(
        sha256_hexadecimal(&at_mkfs),
        SLOT_DIGEST_AT_MKFS,
        "mkfs 那一刻的槽，分四类前后逐字节相同"
    );

    let after = system_configuration_after_the_new_pool_file_creation().to_slot();
    assert_eq!(after.len(), 4096);
    assert_eq!(
        sha256_hexadecimal(&after),
        SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION,
        "新池新建文件发布之后的槽，分四类前后逐字节相同"
    );
    assert_ne!(
        SLOT_DIGEST_AT_MKFS, SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION,
        "两份样本本来就该不同：钉的是两组字节，不是同一组抄了两遍"
    );

    for slot in [&at_mkfs, &after] {
        let padding_start = usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("489");
        assert!(
            slot[padding_start..].iter().all(|byte| *byte == 0),
            "489 之后仍然全是补齐 0：四类合计没有多写出一个字节"
        );
    }
}

/// 四类在槽里是交错的（槽世代号与整槽校验和夹在自举头中间、节点大小夹在几何段开头），
/// 所以「某一档写了多少字节」不是按偏移区间量的，是 `to_slot` 一路记出来的。
#[test]
fn each_mutability_class_writes_its_own_field_table_budget_389_4_36_60() {
    let expected = SlotBytesByMutability {
        immutable_configuration_bytes: 389,
        mutable_configuration_bytes: 4,
        runtime_configuration_bytes: 36,
        runtime_quantity_bytes: 60,
    };
    for system_configuration in [
        system_configuration_at_mkfs(),
        system_configuration_after_the_new_pool_file_creation(),
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
        "389 + 4 + 36 + 60 = 字段表 46 行的 489"
    );
}
