use super::*;

// ============================================================================================
// E38：本地常量与 `crates/` 回比（T2）；T3：装置现算的几何与锚点脚本输出逐项比（A30、A31、A33）。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ConstantComparison {
    pub(super) name: &'static str,
    pub(super) local: u64,
    pub(super) implementation: u64,
}

impl ConstantComparison {
    pub(super) fn is_equal(&self) -> bool {
        self.local == self.implementation
    }
}

pub(super) fn constants_against_the_implementation() -> Vec<ConstantComparison> {
    let compare = |name, local, implementation| ConstantComparison {
        name,
        local,
        implementation,
    };
    vec![
        compare(
            "instance_rows_per_instance_table_page",
            INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
            singlefs_core::admission::INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
        ),
        compare(
            "instance_switches_allowed_per_mount",
            INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
            singlefs_core::admission::INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
        ),
        compare(
            "warm_up_empty_publishes_at_most_per_instance_switch",
            WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
            singlefs_core::admission::WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
        ),
        compare(
            "instance_table_page_bytes",
            INSTANCE_TABLE_PAGE_BYTES,
            TransactionUnit::InstanceTable.span_slots() * singlefs_format::SLOT_BYTES,
        ),
        compare(
            "instance_table_row_count_threshold_default_rows",
            INSTANCE_TABLE_ROW_COUNT_THRESHOLD_DEFAULT_ROWS,
            InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE.0,
        ),
        compare(
            "journal_record_bytes",
            JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        compare(
            "journal_safety_factor",
            JOURNAL_SAFETY_FACTOR,
            singlefs_format::JOURNAL_SAFETY_FACTOR,
        ),
        compare(
            "journal_ring_default_bytes",
            JOURNAL_RING_DEFAULT_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        compare(
            "allocation_record_tree_leaf_slots",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        compare(
            "allocation_record_tree_internal_fanout",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        compare(
            "deletion_batch_data_units",
            DELETION_BATCH_DATA_UNITS,
            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        ),
        compare(
            "unit_area_start_slot_at_the_default_ring",
            UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        compare(
            "slots_per_region_at_make_filesystem",
            SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
            RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        ),
        compare("slot_bytes", SLOT_BYTES, singlefs_format::SLOT_BYTES),
        compare(
            "journal_ring_start_slot",
            JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        compare(
            "root_ring_regions",
            ROOT_RING_REGIONS,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        compare(
            "root_ring_base_slot",
            ROOT_RING_BASE_SLOT,
            singlefs_format::ROOT_RING_BASE_SLOT,
        ),
        compare(
            "root_ring_chunk_bytes",
            ROOT_RING_CHUNK_BYTES,
            singlefs_format::ROOT_RING_CHUNK_BYTES,
        ),
        compare(
            "root_ring_prime_step",
            ROOT_RING_PRIME_STEP,
            singlefs_format::ROOT_RING_PRIME_STEP,
        ),
        compare(
            "system_configuration_slots_per_device",
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        ),
        compare(
            "fixed_structure_slot_spacing_bytes",
            u64::from(FIXED_STRUCTURE_SLOT_SPACING_BYTES),
            singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
        ),
    ]
}

#[derive(Clone, Copy)]
pub(super) struct RegisteredRingWindowAnchor {
    pub(super) device_bytes: u64,
    pub(super) journal_ring_bytes: u64,
    pub(super) slots_per_region: u64,
    pub(super) unit_area_first_slot: u64,
    pub(super) unit_area_slots: u64,
    pub(super) root_level: u32,
    pub(super) positions_at_each_level: &'static [u64],
    pub(super) deletion_publish_slots: u64,
    pub(super) ring_window_slots: u64,
}

impl RegisteredRingWindowAnchor {
    pub(super) fn is_reproduced_by(&self, local: &RingWindowByTheClauses) -> bool {
        local.unit_area_first_slot == self.unit_area_first_slot
            && local.unit_area_slots == self.unit_area_slots
            && local.root_level == self.root_level
            && local.positions_at_each_level == self.positions_at_each_level
            && local.deletion_publish_slots == self.deletion_publish_slots
            && local.ring_window_slots == self.ring_window_slots
    }
}

/// 登记 A33：链重写那一半（每块盘）在 rows0 = 1、366、367、370、739 上的字节数，命令一 `H4 chain_bytes` 行原样。
pub(super) const REGISTERED_CHAIN_REWRITE_BYTES: [(u64, u64); 5] = [
    (1, 131_072),
    (366, 131_072),
    (367, 262_144),
    (370, 262_144),
    (739, 393_216),
];
/// 登记第十三节命令一 `anchors_e183.py` 原样输出里的 15 行 `H4 C=… L=… S=…`（五档容量 × S = 4 / 8 / 16，L = 768 MiB）：
/// （盘字节数，环长，每区槽数，单元区起点槽，单元区槽数，根层级，各层位置数，W，环窗口槽数）。装置现算的与它逐项比（T3）。
pub(super) const REGISTERED_RING_WINDOW_ANCHORS: [RegisteredRingWindowAnchor; 15] = [
    RegisteredRingWindowAnchor {
        device_bytes: 4294967296,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 211968,
        root_level: 2,
        positions_at_each_level: &[524, 4],
        deletion_publish_slots: 374,
        ring_window_slots: 4862,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4294967296,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 211968,
        root_level: 2,
        positions_at_each_level: &[524, 4],
        deletion_publish_slots: 374,
        ring_window_slots: 9350,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4294967296,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 211968,
        root_level: 2,
        positions_at_each_level: &[524, 4],
        deletion_publish_slots: 374,
        ring_window_slots: 18326,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 68719476736,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 4144128,
        root_level: 2,
        positions_at_each_level: &[10210, 62],
        deletion_publish_slots: 432,
        ring_window_slots: 5616,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 68719476736,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 4144128,
        root_level: 2,
        positions_at_each_level: &[10210, 62],
        deletion_publish_slots: 432,
        ring_window_slots: 10800,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 68719476736,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 4144128,
        root_level: 2,
        positions_at_each_level: &[10210, 62],
        deletion_publish_slots: 432,
        ring_window_slots: 21168,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 1099511627776,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 67058688,
        root_level: 3,
        positions_at_each_level: &[165172, 980, 6],
        deletion_publish_slots: 716,
        ring_window_slots: 9308,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 1099511627776,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 67058688,
        root_level: 3,
        positions_at_each_level: &[165172, 980, 6],
        deletion_publish_slots: 716,
        ring_window_slots: 17900,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 1099511627776,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 67058688,
        root_level: 3,
        positions_at_each_level: &[165172, 980, 6],
        deletion_publish_slots: 716,
        ring_window_slots: 35084,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4398046511104,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 268385280,
        root_level: 3,
        positions_at_each_level: &[661050, 3914, 24],
        deletion_publish_slots: 734,
        ring_window_slots: 9542,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4398046511104,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 268385280,
        root_level: 3,
        positions_at_each_level: &[661050, 3914, 24],
        deletion_publish_slots: 734,
        ring_window_slots: 18350,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4398046511104,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 268385280,
        root_level: 3,
        positions_at_each_level: &[661050, 3914, 24],
        deletion_publish_slots: 734,
        ring_window_slots: 35966,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 17592184995840,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 1073691584,
        root_level: 3,
        positions_at_each_level: &[2644564, 15650, 94],
        deletion_publish_slots: 804,
        ring_window_slots: 10452,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 17592184995840,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 1073691584,
        root_level: 3,
        positions_at_each_level: &[2644564, 15650, 94],
        deletion_publish_slots: 804,
        ring_window_slots: 20100,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 17592184995840,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 1073691584,
        root_level: 3,
        positions_at_each_level: &[2644564, 15650, 94],
        deletion_publish_slots: 804,
        ring_window_slots: 39396,
    },
];

#[cfg(test)]
mod tests;
