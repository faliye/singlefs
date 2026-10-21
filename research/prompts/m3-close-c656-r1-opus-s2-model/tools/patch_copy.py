"""在仓副本里装攻方原型要的两处改动（只在副本里，不回主工作区）：
1. 崩溃点重放逐状态展开那一个函数改成 pub（原型照它的计划逐态取镜像，展开取 `full_expansion`）；
2. mkfs 加一个按写序分派的入口 `make_filesystem_with_order`：今天的写序、乙（先作废）、甲候选（提交点之前不碰旧池读的字节）。
   `make_filesystem` 照旧是今天的写序（转调 `Today`）。
用法：python3 patch_copy.py <副本根>"""
import pathlib
import sys

tree = pathlib.Path(sys.argv[1])
crash = tree / "crates/singlefs-checker-tier/src/crash.rs"
text = crash.read_text()
old = "pub(crate) fn visit_crash_state_range_of_the_read_back_flow_with_slices("
assert text.count(old) == 1, "crash.rs 锚点变了"
crash.write_text(text.replace(old, "pub fn visit_crash_state_range_of_the_read_back_flow_with_slices("))

mk = tree / "crates/singlefs-core/src/make_filesystem.rs"
text = mk.read_text()
head = """pub fn make_filesystem<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
) -> Result<MakeFilesystemOutput, MakeFilesystemError> {
"""
assert text.count(head) == 1, "make_filesystem 签名变了"
text = text.replace(head, """pub fn make_filesystem<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
) -> Result<MakeFilesystemOutput, MakeFilesystemError> {
    make_filesystem_with_order(parameters, devices, MkfsOrderForProbe::Today)
}

/// 攻方原型用的 mkfs 写序（只在副本里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MkfsOrderForProbe {
    /// 今天的写序（未改）。
    Today,
    /// 乙：两盘各清 [0, 根环基址)、两盘各过屏障，之后清根环与 journal 环、写单元、屏障、根 FUA、系统配置、屏障。
    InvalidateFirst,
    /// 甲候选：单元写进旧池没写过的槽（`unit_slot`）→ 屏障 → 系统配置四槽（提交）→ 屏障 →
    /// 清 [0, 根环基址) 里两槽之外的字节、根环三区、journal 环 → 屏障 → 根 FUA → 屏障。
    OldPoolUntouchedUntilCommit { unit_slot: u64 },
}

pub fn make_filesystem_with_order<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    order: MkfsOrderForProbe,
) -> Result<MakeFilesystemOutput, MakeFilesystemError> {
""")
# 甲候选改单元落点：在两个落点算出来之后覆盖
anchor = "    let tree_table_genesis_placement = tree_table_genesis_placement_at(unit_area_start);\n"
assert text.count(anchor) == 1
text = text.replace(anchor, anchor + """    let (instance_table_placement, tree_table_genesis_placement) = match order {
        MkfsOrderForProbe::OldPoolUntouchedUntilCommit { unit_slot } => (
            Placement { slot: SlotNumber(unit_slot), span: INSTANCE_TABLE_SPAN_SLOTS },
            Placement { slot: SlotNumber(unit_slot + INSTANCE_TABLE_SPAN_SLOTS), span: TREE_TABLE_GENESIS_SPAN_SLOTS },
        ),
        MkfsOrderForProbe::Today | MkfsOrderForProbe::InvalidateFirst => (instance_table_placement, tree_table_genesis_placement),
    };
""")
# 写的那一段：从清零起到最后一道屏障，按写序分派。根与系统配置的字节先算好（纯计算，不写盘）。
start = text.index("    let root_ring_base = region_start(0);\n")
end_marker = "    Ok(MakeFilesystemOutput {\n        root,\n"
end = text.index(end_marker)
original = text[start:end]
root_begin = original.index("    let root = RootRecord {")
root_end = original.index("    for ring_slot in genesis_root_ring_slots() {")
root_text = original[root_begin:root_end]
sysconf_begin = original.index("        let system_configuration = SystemConfiguration {")
sysconf_end = original.index("        let slot = system_configuration.to_slot();")
sysconf_text = original[sysconf_begin:sysconf_end]
new = """    let root_ring_base = region_start(0);
""" + root_text + """    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let system_configuration_slot_bytes = |identity: &DeviceIdentity| -> Vec<u8> {
""" + sysconf_text.replace("this_device: *identity", "this_device: *identity") + """        system_configuration.to_slot()
    };
    let clear_rings = |device: &mut Device| -> Result<(), MakeFilesystemError> {
        for region_to_clear in 0..ROOT_RING_REGIONS {
            device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
        }
        device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;
        Ok(())
    };
    let write_units = |device: &mut Device| -> Result<(), MakeFilesystemError> {
        device.write_at(instance_table_placement.slot.to_device_offset(), &instance_table_unit, WriteDurability::Plain)?;
        device.write_at(tree_table_genesis_placement.slot.to_device_offset(), &tree_table_genesis_unit, WriteDurability::Plain)?;
        Ok(())
    };
    let write_roots = |devices: &mut [(DeviceIdentity, Device)]| -> Result<(), MakeFilesystemError> {
        for ring_slot in genesis_root_ring_slots() {
            let region_device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
            let (_, device) = devices.iter_mut().find(|(identity, _)| *identity == region_device).expect("check_geometry 核过区域归属");
            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;
        }
        Ok(())
    };
    let write_system_configurations = |devices: &mut [(DeviceIdentity, Device)]| -> Result<(), MakeFilesystemError> {
        for (identity, device) in devices.iter_mut() {
            let slot = system_configuration_slot_bytes(identity);
            for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
                device.write_at(DeviceOffsetInBytes(slot_index * spacing), &slot, WriteDurability::Plain)?;
            }
        }
        Ok(())
    };
    let barriers = |devices: &mut [(DeviceIdentity, Device)]| -> Result<(), MakeFilesystemError> {
        for (_, device) in devices.iter_mut() {
            device.barrier()?;
        }
        Ok(())
    };
    match order {
        MkfsOrderForProbe::Today => {
            for (_, device) in devices.iter_mut() {
                device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;
                clear_rings(device)?;
            }
            for (_, device) in devices.iter_mut() {
                write_units(device)?;
            }
            barriers(devices)?;
            write_roots(devices)?;
            write_system_configurations(devices)?;
            barriers(devices)?;
        }
        MkfsOrderForProbe::InvalidateFirst => {
            for (_, device) in devices.iter_mut() {
                device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;
            }
            barriers(devices)?;
            for (_, device) in devices.iter_mut() {
                clear_rings(device)?;
            }
            for (_, device) in devices.iter_mut() {
                write_units(device)?;
            }
            barriers(devices)?;
            write_roots(devices)?;
            write_system_configurations(devices)?;
            barriers(devices)?;
        }
        MkfsOrderForProbe::OldPoolUntouchedUntilCommit { .. } => {
            for (_, device) in devices.iter_mut() {
                write_units(device)?;
            }
            barriers(devices)?;
            write_system_configurations(devices)?;
            barriers(devices)?;
            let slot_bytes = SYSTEM_CONFIGURATION_SLOT_BYTES;
            for (_, device) in devices.iter_mut() {
                if spacing > slot_bytes {
                    device.write_zeroes_at(DeviceOffsetInBytes(slot_bytes), spacing - slot_bytes)?;
                }
                device.write_zeroes_at(DeviceOffsetInBytes(spacing + slot_bytes), root_ring_base.0 - spacing - slot_bytes)?;
                clear_rings(device)?;
            }
            barriers(devices)?;
            write_roots(devices)?;
            barriers(devices)?;
        }
    }
"""
text = text[:start] + new + text[end:]
mk.write_text(text)
print("patched", crash, mk)
