//! 副本探针：用「只改一个字段」的差分定位系统配置槽里各字段的真实偏移。
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
};

fn configuration(ring_bytes: u64, pbs: u32, min_io: u32) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: [3u8; 16],
            this_device: DeviceIdentity(1),
            device_count: 2,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: pbs,
                minimum_input_output_bytes: min_io,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: ring_bytes,
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

fn differing(a: &[u8], b: &[u8]) -> Vec<usize> {
    (0..a.len()).filter(|i| a[*i] != b[*i]).collect()
}

#[test]
fn locate_fields() {
    let base = configuration(805_306_368, 512, 512).to_slot();
    let other_ring = configuration(268_435_456, 512, 512).to_slot();
    let other_io = configuration(805_306_368, 512, 4096).to_slot();
    println!("环长改动的字节位置（校验和 32 字节另算）：{:?}", differing(&base, &other_ring));
    println!("io_min 改动的字节位置：{:?}", differing(&base, &other_io));
}
