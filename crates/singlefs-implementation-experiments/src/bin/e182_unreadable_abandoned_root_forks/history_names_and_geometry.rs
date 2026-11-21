use super::*;

// ============================================================================
// 二、历史的名字与几何：一条历史 = (族, 几何, 第一个文件单元数, 藏根之前的覆盖写次数)。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DiskGeometry {
    Default,
    Narrow,
}

impl DiskGeometry {
    pub(super) fn name(self) -> &'static str {
        match self {
            DiskGeometry::Default => "default",
            DiskGeometry::Narrow => "narrow",
        }
    }
    pub(super) fn device_bytes(self) -> u64 {
        match self {
            DiskGeometry::Default => DEVICE_BYTES_DEFAULT_LOCAL,
            DiskGeometry::Narrow => DEVICE_BYTES_NARROW_LOCAL,
        }
    }
}

/// 被藏根那一次发布有没有整次返回（登记第一节「两档确认」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Confirmation {
    /// 录制流截在被藏根的根槽 FUA 写之后、系统配置轮换之前（发布没返回）。
    Unconfirmed,
    /// 被藏根那次发布整次返回之后才截；另把每块盘较新的那个系统配置槽也读回全零。
    Confirmed,
}

impl Confirmation {
    pub(super) fn name(self) -> &'static str {
        match self {
            Confirmation::Unconfirmed => "unconfirmed",
            Confirmation::Confirmed => "confirmed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Family {
    /// 带藏根的历史：被藏根的根槽、那次发布的全部 journal 记录、那次发布新写的全部单元在实例 2 挂载的整个进程里读回全零。
    HiddenRoot(Confirmation),
    /// 阳性对照（登记 5.1「路径真跑到」的补跑）：记录照常读得出，核心层只供测试的影子账开关关着。
    HiddenRootRecordsReadableShadowLedgerOff(Confirmation),
    /// 对照族「无藏根」：同一组起点，实例 2 正常可写挂载，不注任何故障。
    NoHiddenRoot,
}

impl Family {
    pub(super) fn name(self) -> String {
        match self {
            Family::HiddenRoot(confirmation) => format!("hidden-root-{}", confirmation.name()),
            Family::HiddenRootRecordsReadableShadowLedgerOff(confirmation) => {
                format!("records-readable-shadow-off-{}", confirmation.name())
            }
            Family::NoHiddenRoot => "no-hidden-root".to_string(),
        }
    }
    pub(super) fn confirmation(self) -> Confirmation {
        match self {
            Family::HiddenRoot(confirmation)
            | Family::HiddenRootRecordsReadableShadowLedgerOff(confirmation) => confirmation,
            Family::NoHiddenRoot => Confirmation::Confirmed,
        }
    }
    pub(super) fn hides_a_root(self) -> bool {
        match self {
            Family::HiddenRoot(_) | Family::HiddenRootRecordsReadableShadowLedgerOff(_) => true,
            Family::NoHiddenRoot => false,
        }
    }
}

/// 实例 1 里「p 次覆盖写」每一次发布的形状（登记修订 12.4 的读法与修订 12.11 的补臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OverwriteShape {
    /// 登记修订 12.4 的读法：每次覆盖写是 `publish_overwrite`，内容一个数据单元，第一次覆盖写之后文件只剩一个单元。
    OneUnit,
    /// 登记修订 12.11 的补臂：每次覆盖写是 `publish_sequential_write`，内容整份 s 个数据单元，文件在每次覆盖写之后仍是 s 个单元，
    /// 被藏根（最后一次覆盖写）引用 s 个数据单元。实例 2 挂载之后的 3 次覆盖写不变（仍是 `publish_overwrite`）。
    KeepingTheFirstFileUnits,
}

impl OverwriteShape {
    /// 产物里历史名与「族 × 几何」合并标签的前缀；登记修订 12.4 的读法不带前缀，名字与第一段产物逐字相同。
    pub(super) fn label_prefix(self) -> &'static str {
        match self {
            OverwriteShape::OneUnit => "",
            OverwriteShape::KeepingTheFirstFileUnits => "units-kept.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct HistorySpecification {
    pub(super) family: Family,
    pub(super) geometry: DiskGeometry,
    pub(super) first_file_units: u64,
    pub(super) overwrites_before_the_hidden_publish: u64,
    pub(super) overwrite_shape: OverwriteShape,
}

pub(super) fn history_name(specification: &HistorySpecification) -> String {
    format!(
        "{}{}.{}.units-{}.overwrites-{}",
        specification.overwrite_shape.label_prefix(),
        specification.family.name(),
        specification.geometry.name(),
        specification.first_file_units,
        specification.overwrites_before_the_hidden_publish
    )
}

/// 登记第一节「起点」：G默认 四个起点 (s, p) ∈ {1, 64} × {5, 10}；G窄 取 (64, 10) 与 (1, 5) 两个。
/// 覆盖写保持 s 个单元的补臂（修订 12.11）只取 s = 64 的起点：s = 1 时两种形状的内容同是一个数据单元，不另建。
fn starting_points(geometry: DiskGeometry, shape: OverwriteShape) -> Vec<(u64, u64)> {
    match (shape, geometry) {
        (OverwriteShape::OneUnit, DiskGeometry::Default) => {
            vec![(1, 5), (1, 10), (64, 5), (64, 10)]
        }
        (OverwriteShape::OneUnit, DiskGeometry::Narrow) => vec![(64, 10), (1, 5)],
        (OverwriteShape::KeepingTheFirstFileUnits, DiskGeometry::Default) => {
            vec![(64, 5), (64, 10)]
        }
        (OverwriteShape::KeepingTheFirstFileUnits, DiskGeometry::Narrow) => vec![(64, 10)],
    }
}

/// 某个覆盖写形状的全部历史：每个起点三族（两档确认的带藏根 + 对照族）。
/// 登记修订 12.4 的读法共 12 + 6 = 18 条；覆盖写保持 s 个单元的补臂共 (2 + 1) × 3 = 9 条。
pub(super) fn history_specifications_of_the_shape(
    shape: OverwriteShape,
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for geometry in [DiskGeometry::Default, DiskGeometry::Narrow] {
        for (first_file_units, overwrites) in starting_points(geometry, shape) {
            for family in [
                Family::HiddenRoot(Confirmation::Unconfirmed),
                Family::HiddenRoot(Confirmation::Confirmed),
                Family::NoHiddenRoot,
            ] {
                specifications.push(HistorySpecification {
                    family,
                    geometry,
                    first_file_units,
                    overwrites_before_the_hidden_publish: overwrites,
                    overwrite_shape: shape,
                });
            }
        }
    }
    specifications
}

/// 第一段的全部历史（登记修订 12.4 的读法）：共 18 条。
pub(super) fn main_history_specifications() -> Vec<HistorySpecification> {
    history_specifications_of_the_shape(OverwriteShape::OneUnit)
}

/// 补臂（修订 12.11）的全部历史：共 9 条，与第一段的 18 条并列、名字带前缀 `units-kept.`。
fn units_kept_history_specifications() -> Vec<HistorySpecification> {
    history_specifications_of_the_shape(OverwriteShape::KeepingTheFirstFileUnits)
}

/// 两个形状的全部历史（列项与样本模式按名字找历史用）。
pub(super) fn every_history_specification() -> Vec<HistorySpecification> {
    let mut specifications = main_history_specifications();
    specifications.extend(units_kept_history_specifications());
    specifications
}

/// 登记 5.1 阳性对照「路径真跑到」的补跑：G默认 的起点、两档确认、记录读得出、影子账关着。
pub(super) fn shadow_ledger_off_history_specifications(
    shape: OverwriteShape,
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for (first_file_units, overwrites) in starting_points(DiskGeometry::Default, shape) {
        for confirmation in [Confirmation::Unconfirmed, Confirmation::Confirmed] {
            specifications.push(HistorySpecification {
                family: Family::HiddenRootRecordsReadableShadowLedgerOff(confirmation),
                geometry: DiskGeometry::Default,
                first_file_units,
                overwrites_before_the_hidden_publish: overwrites,
                overwrite_shape: shape,
            });
        }
    }
    specifications
}

pub(super) fn device_identities() -> [DeviceIdentity; 2] {
    [DeviceIdentity(0), DeviceIdentity(1)]
}

pub(super) fn pool_parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES_LOCAL,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES_LOCAL,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_LOCAL,
            journal_ring_bytes: JOURNAL_RING_BYTES_LOCAL,
            root_ring_slots_per_region: root_ring_slots_per_region(),
        },
    }
}

fn root_ring_slots_per_region() -> RootRingSlotsPerRegion {
    RootRingSlotsPerRegion::from_system_configuration_field(ROOT_RING_SLOTS_PER_REGION_LOCAL)
        .expect("S = 8 在 [4, 16] 区间内")
}

pub(super) fn fixed_geometry_of_the_pool() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_LOCAL,
        journal_ring_bytes: JOURNAL_RING_BYTES_LOCAL,
        root_ring_slots_per_region: root_ring_slots_per_region(),
    }
}

pub(super) fn empty_pool(device_bytes: u64) -> MemoryPool {
    MemoryPool::with_devices(&device_identities(), device_bytes)
}

/// 确定性内容：同一个标签同一个长度，两次逐字节相同（不同标签的内容互不相同，被盖的单元读出来认得出是谁的）。
pub(super) fn deterministic_content(tag: u64, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let value = u64::try_from(index)
                .expect("下标装得进 u64")
                .wrapping_mul(131)
                .wrapping_add(tag.wrapping_mul(977))
                .wrapping_add(7);
            u8::try_from(value % 251).expect("模 251 装得进 u8")
        })
        .collect()
}

#[cfg(test)]
mod tests;
