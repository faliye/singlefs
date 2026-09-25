//! 池级 checker 的输入与第一层判定：读盘的口子、每条不变量的三态、系统配置与根环（I-7 类）、指针与位置条目。
//! 与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5）：偏移在这里按字段表各写一份。

use std::collections::BTreeMap;

use singlefs_format::{
    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
    ROOT_RING_SLOTS_PER_REGION_MINIMUM, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::{
    check_root_slot, check_system_configuration_slot, read_six_byte_unsigned, read_u32, read_u64,
    RootView, SystemConfigurationView,
};

/// checker 读镜像的口子。它不依赖实现的块设备抽象：harness 的崩溃镜像、宿主上的盘镜像各自接一份。
pub trait ImageReader {
    fn devices(&self) -> Vec<u32>;
    fn device_bytes(&self, device: u32) -> Option<u64>;
    /// 读不到（没有这块盘、越界）返回 None。
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>>;
    /// 扫描方向的候选：可能有单元头的 16 KiB 槽号。None = 从单元区起点扫到盘尾。
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>>;
    /// 扫描方向的候选：可能有 journal 记录的环内槽号（从环起点数，每槽一条 4096 字节的记录）。None = 整条环都扫。
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>>;
}

/// 一条不变量在一个镜像上的判定。「不适用」必须带理由，不许静默算成立。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvariantVerdict {
    Holds,
    Violated(String),
    NotApplicable(&'static str),
}

/// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
pub const IMPLEMENTED_INVARIANTS: [&str; 46] = [
    "I-1.1", "I-1.2", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-1.10", "I-2.1", "I-2.3",
    "I-2.4", "I-2.5", "I-3.1", "I-3.8", "I-3.9", "I-3.10", "I-3.11", "I-4.2", "I-4.8", "I-5.1",
    "I-5.2", "I-5.4", "I-7.1", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-7.9",
    "I-7.10", "I-7.11", "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.2", "I-9.4", "I-9.6",
    "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15",
];

/// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
#[derive(Default)]
pub struct Judgements {
    evaluated: BTreeMap<&'static str, u64>,
    violations: BTreeMap<&'static str, u64>,
    first_violation: BTreeMap<&'static str, String>,
    not_applicable: BTreeMap<&'static str, &'static str>,
}

impl Judgements {
    /// 评估一次：条件不成立就记下这一处（只留第一处）。
    pub fn judge(&mut self, invariant: &'static str, holds: bool, detail: impl FnOnce() -> String) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        *self.evaluated.entry(invariant).or_insert(0) += 1;
        if !holds {
            *self.violations.entry(invariant).or_insert(0) += 1;
            self.first_violation.entry(invariant).or_insert_with(detail);
        }
    }

    /// 到此为止判了几次违例：走读一条候选根前后各读一次，差就是这条根引用的单元里对不上的那些（I-7.4、I-4.8 按根判）。
    #[must_use]
    pub fn violation_count(&self, invariant: &'static str) -> u64 {
        self.violations.get(invariant).copied().unwrap_or(0)
    }
    /// 这个镜像上整条不适用（例如第 0 代根下面没有记账树）。已经评估过的不改。
    pub fn not_applicable(&mut self, invariant: &'static str, reason: &'static str) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.not_applicable.entry(invariant).or_insert(reason);
    }
    #[must_use]
    pub fn into_report(self) -> Vec<(&'static str, InvariantVerdict)> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                let verdict = if let Some(detail) = self.first_violation.get(invariant) {
                    InvariantVerdict::Violated(detail.clone())
                } else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {
                    InvariantVerdict::Holds
                } else {
                    InvariantVerdict::NotApplicable(
                        self.not_applicable
                            .get(invariant)
                            .copied()
                            .unwrap_or("这个镜像上没有它判的对象"),
                    )
                };
                (*invariant, verdict)
            })
            .collect()
    }
}

/// 系统配置里 checker 要用的几何（偏移按系统配置字段表各写一份：物理块 317、journal 环起点 325 与长度 333、R 361、S 362、P 363、
/// chunk 367、根环基址 371、逐区域设备 379、单元区起点 417、io_min 425、槽距 429）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolGeometry {
    pub filesystem_identifier: [u8; 16],
    pub physical_block_size: u32,
    pub journal_ring_start_slot: u64,
    pub journal_ring_bytes: u64,
    pub regions: u64,
    pub slots_per_region: u64,
    pub prime_step: u64,
    pub chunk_bytes: u64,
    pub base_slot: u64,
    pub region_devices: [u32; 3],
    pub unit_area_start_slot: u64,
    pub slot_spacing: u64,
}

/// 系统配置字段表给逐区域设备身份留了几个字段：偏移 379 / 383 / 387，一共三个。
/// R（偏移 361 那一字节）是**盘上读来的值**，可以是 0..255 里的任何一个；它大于这个数时，
/// 第 3 个及以后的区域在字段表里根本没有设备身份可读——这份系统配置这个格式版本读不了。
pub const REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION: usize = 3;

/// 从一个自证过的系统配置槽里取几何。
///
/// # Errors
/// R 大于 [`REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION`] ⇒
/// [`crate::Verdict::RegionCountPastTheRegionDeviceFields`]：不夹成 3 往下走（那是静默跳过几个区域，
/// 根环少读几个槽而没人说），也不 panic（panic 面普查 R12：`root_slot_positions` 拿它下标一个长 3 的数组）。
///
/// S（偏移 362 那一字节）不在格式承诺的区间里 ⇒
/// [`crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval`]，同一条理由：
/// 夹一下往下走就是按一个不存在的几何走读。两条判据都在盘上那一字节进 [`PoolGeometry`] 的唯一入口上。
pub fn geometry_of(
    slot: &[u8],
    view: &SystemConfigurationView,
) -> Result<PoolGeometry, crate::Verdict> {
    let regions = u64::from(slot[361]);
    if regions
        > u64::try_from(REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION).expect("三个区域字段")
    {
        return Err(crate::Verdict::RegionCountPastTheRegionDeviceFields);
    }
    let slots_per_region = u64::from(slot[362]);
    if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)
        .contains(&slots_per_region)
    {
        return Err(crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval);
    }
    Ok(PoolGeometry {
        filesystem_identifier: view.filesystem_identifier,
        physical_block_size: view.declared_physical_block_size,
        journal_ring_start_slot: read_u64(slot, 325),
        journal_ring_bytes: read_u64(slot, 333),
        regions,
        slots_per_region,
        prime_step: u64::from(read_u32(slot, 363)),
        chunk_bytes: u64::from(read_u32(slot, 367)),
        base_slot: read_u64(slot, 371),
        region_devices: [
            read_u32(slot, 379),
            read_u32(slot, 383),
            read_u32(slot, 387),
        ],
        unit_area_start_slot: read_u64(slot, 417),
        slot_spacing: u64::from(read_u32(slot, 429)),
    })
}

fn parse_system_configuration_slot(
    bytes: &[u8],
) -> Option<(SystemConfigurationView, PoolGeometry)> {
    let view = check_system_configuration_slot(bytes).ok()?;
    let geometry = geometry_of(bytes, &view).ok()?;
    Some((view, geometry))
}

/// 每盘两槽里自证过的系统配置：槽 0 在偏移 0；槽 1 的偏移按槽 0 记的槽距，槽 0 无效时按最小槽距 4096（2026-09-14 用户收尾弹窗定甲）。
/// I-7.7（系统配置实例代号不低于根环） 按这个读法取每盘的实例代号（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）。
#[must_use]
pub fn verified_system_configuration_slots(
    reader: &dyn ImageReader,
) -> Vec<(u32, Vec<(SystemConfigurationView, PoolGeometry)>)> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    reader
        .devices()
        .into_iter()
        .map(|device| {
            let slot_zero = reader
                .read(device, 0, slot_bytes)
                .as_deref()
                .and_then(parse_system_configuration_slot);
            let spacing = slot_zero.as_ref().map_or(
                FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
                |(_, geometry)| geometry.slot_spacing,
            );
            let slot_one = reader
                .read(device, spacing, slot_bytes)
                .as_deref()
                .and_then(parse_system_configuration_slot);
            (device, slot_zero.into_iter().chain(slot_one).collect())
        })
        .collect()
}

/// 一个自证过的系统配置槽里的回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：哪块盘、槽世代号、解出来的条目或解不开的那一样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RollbackWitnessOfASlot {
    pub device: u32,
    pub slot_generation: u64,
    pub witness: Result<Vec<crate::RollbackWitnessEntryView>, &'static str>,
}

/// 每盘两槽里自证过的系统配置槽各自的回退见证表（读法与 `verified_system_configuration_slots` 相同：槽 1 按槽 0 记的槽距找）。
/// 条数上限按那一槽自述的 R、S 算（R × S − 1）。
#[must_use]
pub fn rollback_witness_of_every_verified_slot(
    reader: &dyn ImageReader,
) -> Vec<RollbackWitnessOfASlot> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut witnesses = Vec::new();
    for device in reader.devices() {
        let slot_zero = reader.read(device, 0, slot_bytes);
        let spacing = slot_zero
            .as_deref()
            .and_then(parse_system_configuration_slot)
            .map_or(
                FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
                |(_, geometry)| geometry.slot_spacing,
            );
        let slot_one = reader.read(device, spacing, slot_bytes);
        for bytes in [slot_zero, slot_one].into_iter().flatten() {
            let Some((view, geometry)) = parse_system_configuration_slot(&bytes) else {
                continue;
            };
            witnesses.push(RollbackWitnessOfASlot {
                device,
                slot_generation: view.slot_generation,
                witness: crate::rollback_witness_of_system_configuration_slot(
                    &bytes,
                    (geometry.regions * geometry.slots_per_region).saturating_sub(1),
                ),
            });
        }
    }
    witnesses
}

/// 一个池此刻的回退见证：每块盘上见证表解得开的槽里世代号最大的那一槽的条目，各盘取并集（与实现同一个读法：解不开见证表的槽
/// 在实现那边就是读不出的槽，不参与择槽）。
#[must_use]
pub fn rollback_witness_of_the_pool(
    slots: &[RollbackWitnessOfASlot],
) -> Vec<crate::RollbackWitnessEntryView> {
    let mut chosen_per_device: BTreeMap<u32, (u64, &Vec<crate::RollbackWitnessEntryView>)> =
        BTreeMap::new();
    for slot in slots {
        let Ok(entries) = &slot.witness else {
            continue;
        };
        let is_newer = chosen_per_device
            .get(&slot.device)
            .is_none_or(|(generation, _)| slot.slot_generation > *generation);
        if is_newer {
            chosen_per_device.insert(slot.device, (slot.slot_generation, entries));
        }
    }
    let mut union: Vec<crate::RollbackWitnessEntryView> = chosen_per_device
        .values()
        .flat_map(|(_, entries)| entries.iter().copied())
        .collect();
    union.sort_unstable();
    union.dedup();
    union
}

/// 每盘择一个系统配置：两槽里世代号大的那一份（相等取槽 0）。
#[must_use]
pub fn chosen_system_configurations(
    reader: &dyn ImageReader,
) -> Vec<(u32, Option<(SystemConfigurationView, PoolGeometry)>)> {
    verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(device, slots)| {
            let chosen = slots.into_iter().fold(
                None,
                |best: Option<(SystemConfigurationView, PoolGeometry)>, candidate| match best {
                    Some(current) if candidate.0.slot_generation <= current.0.slot_generation => {
                        Some(current)
                    }
                    Some(_) | None => Some(candidate),
                },
            );
            (device, chosen)
        })
        .collect()
}

/// 根环全部槽：(区域, 槽, 盘, 偏移)，择根与 I-7.7（系统配置实例代号不低于根环） 的「读不全」判定共用这一个算式。
#[must_use]
pub fn root_slot_positions(geometry: &PoolGeometry) -> Vec<(u64, u64, u32, u64)> {
    let mut positions = Vec::new();
    for region in 0..geometry.regions {
        // 下标在范围内不是这里判的：`geometry_of` 是盘上那一字节进 `PoolGeometry` 的唯一入口，
        // 它已经把 R > 3 的槽拒成 `Verdict::RegionCountPastTheRegionDeviceFields`、不交出 `PoolGeometry`。
        let device = geometry.region_devices[usize::try_from(region).expect(
            "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
        )];
        for slot in 0..geometry.slots_per_region {
            let offset = geometry.base_slot * SLOT_BYTES
                + region * geometry.prime_step * geometry.chunk_bytes
                + slot * geometry.slot_spacing;
            positions.push((region, slot, device, offset));
        }
    }
    positions
}

/// 根环全部槽里自证过的根（I-7.1 的集合 S），带它住的区域与槽号。
#[must_use]
pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {
    let slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    root_slot_positions(geometry)
        .into_iter()
        .filter_map(|(region, slot, device, offset)| {
            reader
                .read(device, offset, slot_bytes)
                .and_then(|bytes| check_root_slot(&bytes, &geometry.filesystem_identifier).ok())
                .map(|view| (region, slot, view))
        })
        .collect()
}

/// 位置条目：设备身份 4 + 槽号 6 + 整单元校验和 4。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerLocation {
    pub device: u32,
    pub slot: u64,
    pub checksum: u32,
}

fn location_at(bytes: &[u8], offset: usize) -> PointerLocation {
    PointerLocation {
        device: read_u32(bytes, offset),
        slot: read_six_byte_unsigned(bytes, offset + 4),
        checksum: read_u32(bytes, offset + 10),
    }
}

/// 指针的公共部分：头部 50（出生树在 34、出生 txg 在 42）+ 位置条目 14 × 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerView {
    pub birth_tree: u64,
    pub birth_txg: u64,
    pub locations: [PointerLocation; 2],
    /// 码 2 / 码 3 指针：实例代号 + 出生序号；码 1 指针：写序的实例代号 + 事务号。
    pub instance: u32,
    pub tail: u64,
    pub all_zero: bool,
}

/// 指向码 2 / 码 3 的指针 86：尾段 = 实例代号 4 + 出生序号 4。
#[must_use]
pub fn parse_node_pointer(bytes: &[u8]) -> PointerView {
    PointerView {
        birth_tree: read_u64(bytes, 34),
        birth_txg: read_u64(bytes, 42),
        locations: [location_at(bytes, 50), location_at(bytes, 64)],
        instance: read_u32(bytes, 78),
        tail: u64::from(read_u32(bytes, 82)),
        all_zero: bytes[..86].iter().all(|byte| *byte == 0),
    }
}

/// 指向码 1 的指针 88：尾段 = 写序（实例代号 4 + 事务号 6）。
#[must_use]
pub fn parse_data_pointer(bytes: &[u8]) -> PointerView {
    PointerView {
        birth_tree: read_u64(bytes, 34),
        birth_txg: read_u64(bytes, 42),
        locations: [location_at(bytes, 50), location_at(bytes, 64)],
        instance: read_u32(bytes, 78),
        tail: read_six_byte_unsigned(bytes, 82),
        all_zero: bytes[..88].iter().all(|byte| *byte == 0),
    }
}

/// I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免。
pub fn judge_location_order(judgements: &mut Judgements, pointer: &PointerView, what: &str) {
    if pointer.all_zero {
        return;
    }
    judgements.judge(
        "I-2.5",
        pointer.locations[0].device < pointer.locations[1].device,
        || {
            format!(
                "{what} 的位置条目设备身份 {} / {} 不是严格升序",
                pointer.locations[0].device, pointer.locations[1].device
            )
        },
    );
}

/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。
pub fn read_referenced_unit(
    reader: &dyn ImageReader,
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    unit_bytes: usize,
    what: &str,
) -> Option<Vec<u8>> {
    let mut first_good = None;
    for location in locations {
        let bytes = reader.read(location.device, location.slot * SLOT_BYTES, unit_bytes);
        let matches = bytes
            .as_ref()
            .is_some_and(|content| crate::crc32_castagnoli_table(content) == location.checksum);
        judgements.judge("I-2.1", matches, || {
            format!(
                "{what} 在盘 {} 槽 {} 的那一份与位置条目里的校验和对不上",
                location.device, location.slot
            )
        });
        if matches && first_good.is_none() {
            first_good = bytes;
        }
    }
    first_good
}
