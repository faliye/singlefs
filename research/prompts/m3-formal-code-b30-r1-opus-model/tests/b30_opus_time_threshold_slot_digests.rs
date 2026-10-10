//! b30 攻方腿 M3 探针：第 30 批 30-3 重取的两个槽摘要，换一条路复核出处。
//! 两份样本照 `system_configuration_mutability_classes.rs` 原样构造（下面两个函数从那份用例逐字拷来），today 的 `to_slot` 写出 4096 字节；
//! 把 [433, 437) 换回 5（秒）、整槽校验和重封，摘要要等于换钉之前的两个旧值（a065c124…、c67464f9…，那两个值是 30-3 之前
//! 一直入库、逐次判绿的）；新旧两份逐字节比，差只许落在 [433, 437) 与整槽校验和 [155, 187)。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::sha256::sha256_hexadecimal;

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

const OLD_SLOT_DIGEST_AT_MKFS: &str = "a065c12485a9ff72117d549852461f5cb4529d664ac2384271f6e0f22d0f167f";
const OLD_SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION: &str =
    "c67464f93c9021cf354d649f37044041e4b5eec252f23d103a7e8c1328796260";
const NEW_SLOT_DIGEST_AT_MKFS: &str = "e5cc28abb94e3e1739b85dc7abc2191a1b120908b72a5ab3cb03588feb01c28f";
const NEW_SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION: &str =
    "144c1785dd57555c8539963909817a597c5a32d2d4b8fc3e61f7a8f6233f299e";

#[test]
fn the_two_new_slot_digests_differ_from_the_old_ones_only_by_the_time_threshold_unit() {
    for (name, configuration, old, new) in [
        ("mkfs", system_configuration_at_mkfs(), OLD_SLOT_DIGEST_AT_MKFS, NEW_SLOT_DIGEST_AT_MKFS),
        (
            "after_new_pool_file_creation",
            system_configuration_after_the_new_pool_file_creation(),
            OLD_SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION,
            NEW_SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION,
        ),
    ] {
        let today = configuration.to_slot();
        assert_eq!(&today[433..437], &5000u32.to_le_bytes(), "{name}：今天写的是 5000");
        assert_eq!(sha256_hexadecimal(&today), new, "{name}：新钉值");
        let mut in_seconds = today.clone();
        in_seconds[433..437].copy_from_slice(&5u32.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&in_seconds, 4096, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        in_seconds[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        let differing: Vec<usize> = (0..4096).filter(|i| today[*i] != in_seconds[*i]).collect();
        println!("B30PIN {name} old_equals={} differing_offsets={differing:?}", sha256_hexadecimal(&in_seconds) == old);
        assert_eq!(sha256_hexadecimal(&in_seconds), old, "{name}：换回秒之后等于旧钉值");
        assert!(
            differing.iter().all(|i| (433..437).contains(i) || (155..187).contains(i)),
            "{name}：差只落在 T_time 与整槽校验和：{differing:?}"
        );
    }
}
