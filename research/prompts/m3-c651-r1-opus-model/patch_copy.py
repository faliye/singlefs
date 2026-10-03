#!/usr/bin/env python3
"""C651 攻方副本补丁：给 core 与 checker 的「全池没有可择槽 0」那一格装候选开关（环境变量 C651_CANDIDATE）。
用法：python3 patch_copy.py <副本根>   每处替换断言恰好命中一次。"""
import sys, pathlib

root = pathlib.Path(sys.argv[1])

def replace_once(path, old, new):
    p = root / path
    text = p.read_text()
    n = text.count(old)
    assert n == 1, f"{path}: 命中 {n} 次"
    p.write_text(text.replace(old, new))

CORE = "crates/singlefs-core/src/recovery.rs"
replace_once(CORE,
    "            None => slot_one_found_by_trying_every_slot_spacing_the_format_allows(reader, device)?,\n",
    "            None => c651_slot_one_when_no_slot_zero_in_the_pool(reader, device)?,\n")
replace_once(CORE,
    "/// 每盘两槽：槽 0 在偏移 0；槽 1 在固定结构槽距处",
    r'''/// C651 攻方副本：候选名。
pub fn c651_candidate() -> String {
    std::env::var("C651_CANDIDATE").unwrap_or_else(|_| "search".to_string())
}

/// C651 攻方副本：V1-探测 用的「挂载时探得的 io_min」。
pub fn c651_probed_io_min() -> u32 {
    std::env::var("C651_PROBED_IO_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(512)
}

/// C651 攻方副本：全池没有可择槽 0 时一块盘的槽 1，按候选取。
fn c651_slot_one_when_no_slot_zero_in_the_pool(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
) -> Result<SystemConfigurationSlotReading, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let candidate = c651_candidate();
    match candidate.as_str() {
        "search" => slot_one_found_by_trying_every_slot_spacing_the_format_allows(reader, device),
        "const" => mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES), slot_bytes),
        ),
        "refuse" => Ok(SystemConfigurationSlotReading::NotSelfDescribing),
        "probe" => {
            let spacing = crate::system_configuration::SystemImmutableSizes::slot_spacing_for(c651_probed_io_min());
            mountable_slot_or_refusal(device, reader.read(device, DeviceOffsetInBytes(u64::from(spacing)), slot_bytes))
        }
        "first" | "unique" => {
            let largest_spacing = largest_fixed_structure_slot_spacing_the_format_allows();
            let mut accepted: Vec<SystemConfiguration> = Vec::new();
            let mut incompat: Option<IncompatBitmap> = None;
            for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing).step_by(512) {
                match mountable_slot_or_refusal(device, reader.read(device, DeviceOffsetInBytes(candidate_spacing), slot_bytes))? {
                    SystemConfigurationSlotReading::Mountable(c) => {
                        if u64::from(c.immutable.sizes.fixed_structure_slot_spacing) == candidate_spacing {
                            accepted.push(c);
                        }
                    }
                    SystemConfigurationSlotReading::NotSelfDescribing => {}
                    SystemConfigurationSlotReading::IncompatBitsNotRecognized(b) => { incompat = incompat.or(Some(b)); }
                }
            }
            let chosen = if candidate == "first" {
                accepted.into_iter().next()
            } else if accepted.len() == 1 {
                accepted.into_iter().next()
            } else {
                None
            };
            Ok(match (chosen, incompat) {
                (Some(c), _) => SystemConfigurationSlotReading::Mountable(c),
                (None, Some(b)) => SystemConfigurationSlotReading::IncompatBitsNotRecognized(b),
                (None, None) => SystemConfigurationSlotReading::NotSelfDescribing,
            })
        }
        other => panic!("C651_CANDIDATE 不认识：{other}"),
    }
}

/// 每盘两槽：槽 0 在偏移 0；槽 1 在固定结构槽距处''')

CHECKER = "crates/singlefs-checker/src/image.rs"
replace_once(CHECKER,
    """            let spacing = spacing_recorded_by_slot_zero(&slot_zero)
                .or(spacing_recorded_by_the_first_valid_slot_zero)
                .unwrap_or(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES);
            let slot_one = read_system_configuration_slot(reader, device, spacing);
""",
    """            let slot_one = match spacing_recorded_by_slot_zero(&slot_zero)
                .or(spacing_recorded_by_the_first_valid_slot_zero)
            {
                Some(spacing) => read_system_configuration_slot(reader, device, spacing),
                None => c651_checker_slot_one_when_no_slot_zero_in_the_pool(reader, device),
            };
""")
replace_once(CHECKER,
    "/// 每盘两槽里可择的系统配置（[`SystemConfigurationSlotReading::Usable`]），槽的读法见",
    r'''/// C651 攻方副本：checker 一侧同一组候选（与 core 的 `c651_slot_one_when_no_slot_zero_in_the_pool` 同义）。
fn c651_checker_slot_one_when_no_slot_zero_in_the_pool(
    reader: &dyn ImageReader,
    device: u32,
) -> SystemConfigurationSlotReading {
    let candidate = std::env::var("C651_CANDIDATE").unwrap_or_else(|_| "const".to_string());
    match candidate.as_str() {
        "const" => read_system_configuration_slot(reader, device, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES),
        "refuse" => SystemConfigurationSlotReading::NotSelfVerified,
        "probe" => {
            let io_min: u64 = std::env::var("C651_PROBED_IO_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(512);
            let spacing = 4096u64.div_ceil(io_min) * io_min;
            read_system_configuration_slot(reader, device, spacing)
        }
        "search" | "first" | "unique" => {
            let largest = (1u64 << 20) - SYSTEM_CONFIGURATION_SLOT_BYTES;
            let mut accepted: Vec<(SystemConfigurationView, PoolGeometry)> = Vec::new();
            let mut not_choosable = false;
            for s in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest).step_by(512) {
                match read_system_configuration_slot(reader, device, s) {
                    refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => return refused,
                    SystemConfigurationSlotReading::Usable(view, geometry) => {
                        if geometry.slot_spacing == s {
                            accepted.push((view, geometry));
                        }
                    }
                    SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => not_choosable = true,
                    SystemConfigurationSlotReading::NotSelfVerified => {}
                }
            }
            let chosen = match candidate.as_str() {
                "search" => {
                    let mut best: Option<(SystemConfigurationView, PoolGeometry)> = None;
                    for (view, geometry) in accepted {
                        if best.as_ref().is_none_or(|(b, _)| view.slot_generation > b.slot_generation) {
                            best = Some((view, geometry));
                        }
                    }
                    best
                }
                "first" => accepted.into_iter().next(),
                _ => if accepted.len() == 1 { accepted.into_iter().next() } else { None },
            };
            match chosen {
                Some((view, geometry)) => SystemConfigurationSlotReading::Usable(view, geometry),
                None if not_choosable => SystemConfigurationSlotReading::SelfVerifiedButNotChoosable,
                None => SystemConfigurationSlotReading::NotSelfVerified,
            }
        }
        other => panic!("C651_CANDIDATE 不认识：{other}"),
    }
}

/// 每盘两槽里可择的系统配置（[`SystemConfigurationSlotReading::Usable`]），槽的读法见''')
MKFS = "crates/singlefs-core/src/make_filesystem.rs"
replace_once(MKFS,
    """    for (_, device) in devices.iter_mut() {
        for region_to_clear in 0..ROOT_RING_REGIONS {
""",
    """    for (_, device) in devices.iter_mut() {
        // C651 攻方副本：零轮收严「mkfs 把 [0, 根环基址) 整段清零」（C651_MKFS_CLEARS 设了才做）。
        if std::env::var("C651_MKFS_CLEARS").is_ok() {
            device.write_zeroes_at(DeviceOffsetInBytes(0), region_start(0).0)?;
        }
        for region_to_clear in 0..ROOT_RING_REGIONS {
""")
print("patched")
