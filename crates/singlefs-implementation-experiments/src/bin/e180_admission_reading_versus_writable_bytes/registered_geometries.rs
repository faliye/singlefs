//! 登记的几何（盘数、容量与单元区的起点）。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 几何、池、会话的拷贝
// ---------------------------------------------------------------------------------------------------------------------

/// 登记第五节的几何：盘字节数（两块）与每区槽数 S；默认环 768 MiB、io_min 4096 不变。
#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    pub(super) name: &'static str,
    pub(super) device_bytes: [u64; 2],
    pub(super) slots_per_region: u64,
}

pub(super) const FOUR_GIBIBYTES: u64 = 4 << 30;
/// 3 GiB 加 33 槽：`crates/singlefs-verification/tests/unequal_devices.rs` 里已在用、mkfs 收的尺寸。
const THREE_GIBIBYTES_AND_33_SLOTS: u64 = (3 << 30) + 33 * SLOT_BYTES;

pub(super) const ALL_GEOMETRIES: [Geometry; 5] = [
    Geometry {
        name: "default_geometry",
        device_bytes: [FOUR_GIBIBYTES, FOUR_GIBIBYTES],
        slots_per_region: 8,
    },
    Geometry {
        name: "unequal_devices_geometry",
        device_bytes: [FOUR_GIBIBYTES, THREE_GIBIBYTES_AND_33_SLOTS],
        slots_per_region: 8,
    },
    Geometry {
        name: "narrow_devices_geometry",
        device_bytes: [THREE_GIBIBYTES_AND_33_SLOTS, THREE_GIBIBYTES_AND_33_SLOTS],
        slots_per_region: 8,
    },
    Geometry {
        name: "default_geometry_with_four_slots_per_region",
        device_bytes: [FOUR_GIBIBYTES, FOUR_GIBIBYTES],
        slots_per_region: 4,
    },
    Geometry {
        name: "default_geometry_with_sixteen_slots_per_region",
        device_bytes: [FOUR_GIBIBYTES, FOUR_GIBIBYTES],
        slots_per_region: 16,
    },
];

impl Geometry {
    pub(super) fn unit_area_slots_of_device(&self, device_index: usize) -> u64 {
        self.device_bytes[device_index] / SLOT_BYTES - DEFAULT_UNIT_AREA_START_SLOT
    }

    pub(super) fn parameters(&self) -> MakeFilesystemParameters {
        MakeFilesystemParameters {
            filesystem_identifier: *b"_SFS-E180-ADMRD\0",
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: PHYSICAL_BLOCK_BYTES,
                minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES,
                root_ring_slots_per_region:
                    RootRingSlotsPerRegion::from_system_configuration_field(self.slots_per_region)
                        .expect("每区槽数只取 4、8、16，都在挂载判的区间里"),
            },
        }
    }

    pub(super) fn blank_devices(&self) -> Vec<(DeviceIdentity, Disk<'static>)> {
        self.device_bytes
            .iter()
            .enumerate()
            .map(|(index, bytes)| {
                (
                    DeviceIdentity(u32::try_from(index).expect("盘号")),
                    Disk::blank(*bytes),
                )
            })
            .collect()
    }
}

pub(super) fn geometry_named(name: &str) -> Geometry {
    *ALL_GEOMETRIES
        .iter()
        .find(|geometry| geometry.name == name)
        .unwrap_or_else(|| stop_with("unknown_geometry", &format!("geometry={name}")))
}
