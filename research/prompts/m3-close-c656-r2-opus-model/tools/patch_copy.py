"""在仓副本里装攻方原型要的两处改动（只在副本里，不回主工作区）：
1. 崩溃点重放逐状态展开那一个函数改成 pub（原型照它的计划逐态取镜像，展开取 `full_expansion`）；
2. mkfs 加一个按写序分派的入口 `make_filesystem_with_order`：
   今天的写序、乙（两趟：两盘各清 [0, 根环基址) → 两盘各屏障 → 其余）、
   乙 + 甲七（屏障之后清单元区）、乙 + 甲七错排（单元区与 [0, 根环基址) 同段清）、
   乙但屏障只到第一块盘（第二块盘的屏障不发、它的根 FUA 当普通写）。
   `make_filesystem` 照旧是今天的写序（转调 `Today`）。
写法从 m3-close-c656-r1 攻方第二次抽样的 `tools/patch_copy.py` 改来（那一份的乙、今天两支逐字保留）。
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
    /// 乙 + 甲七：作废屏障之后，每盘清根环、journal 环，再清单元区（第 0 版两个单元之后到盘尾），之后同乙。
    InvalidateFirstThenClearUnitArea,
    /// 乙 + 甲七错排：每盘 [0, 根环基址) 与单元区在同一段里清（同一道屏障之前），之后同乙。
    ClearUnitAreaTogetherWithInvalidation,
    /// 乙，但屏障只发给第一块盘：第二块盘一道屏障都不发，它那条根 FUA 写当普通写（不认 flush 的盘一般也不认 FUA）。
    InvalidateFirstBarrierOnlyOnTheFirstDevice,
}

pub fn make_filesystem_with_order<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    order: MkfsOrderForProbe,
) -> Result<MakeFilesystemOutput, MakeFilesystemError> {
""")
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
    let first_device = devices[0].0;
    let barrier_reaches = |identity: &DeviceIdentity| -> bool {
        order != MkfsOrderForProbe::InvalidateFirstBarrierOnlyOnTheFirstDevice || *identity == first_device
    };
    let system_configuration_slot_bytes = |identity: &DeviceIdentity| -> Vec<u8> {
""" + sysconf_text + """        system_configuration.to_slot()
    };
    let clear_rings = |device: &mut Device| -> Result<(), MakeFilesystemError> {
        for region_to_clear in 0..ROOT_RING_REGIONS {
            device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
        }
        device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;
        Ok(())
    };
    // 甲七：单元区里第 0 版两个单元之后到盘尾（按槽对齐的那一截）整段清零；第 0 版两个单元自己的槽由单元写盖掉。
    let unit_area_clear_start = (tree_table_genesis_placement.slot.0 + TREE_TABLE_GENESIS_SPAN_SLOTS) * SLOT_BYTES;
    let clear_unit_area = |device: &mut Device| -> Result<(), MakeFilesystemError> {
        let end = device.size_in_bytes() / SLOT_BYTES * SLOT_BYTES;
        if end > unit_area_clear_start {
            device.write_zeroes_at(DeviceOffsetInBytes(unit_area_clear_start), end - unit_area_clear_start)?;
        }
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
            let (identity, device) = devices.iter_mut().find(|(identity, _)| *identity == region_device).expect("check_geometry 核过区域归属");
            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            let durability = if barrier_reaches(identity) { WriteDurability::ForceUnitAccess } else { WriteDurability::Plain };
            device.write_at(offset, &root_slot, durability)?;
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
        for (identity, device) in devices.iter_mut() {
            if barrier_reaches(identity) {
                device.barrier()?;
            }
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
        MkfsOrderForProbe::InvalidateFirst
        | MkfsOrderForProbe::InvalidateFirstThenClearUnitArea
        | MkfsOrderForProbe::ClearUnitAreaTogetherWithInvalidation
        | MkfsOrderForProbe::InvalidateFirstBarrierOnlyOnTheFirstDevice => {
            for (_, device) in devices.iter_mut() {
                device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;
                if order == MkfsOrderForProbe::ClearUnitAreaTogetherWithInvalidation {
                    clear_unit_area(device)?;
                }
            }
            barriers(devices)?;
            for (_, device) in devices.iter_mut() {
                clear_rings(device)?;
                if order == MkfsOrderForProbe::InvalidateFirstThenClearUnitArea {
                    clear_unit_area(device)?;
                }
            }
            for (_, device) in devices.iter_mut() {
                write_units(device)?;
            }
            barriers(devices)?;
            write_roots(devices)?;
            write_system_configurations(devices)?;
            barriers(devices)?;
        }
    }
"""
text = text[:start] + new + text[end:]
mk.write_text(text)
print("patched", crash, mk)
