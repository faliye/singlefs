//! 样本：只放字段表这几段，这一格只读源码文本、不编译。
/// D13（样本） 已定项 1 挂载透明逐字段表今天的行名；不可变配置那一行折成两行（cargo fmt 对长名字就这样折）。
const MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION: &str =
    "池身份与不可变配置（本盘设备号除外）";
const MOUNT_TRANSPARENCY_TABLE_UNMOUNT_MARKER: &str = "卸载记号";
const MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS: &str = "magic 与校验和";
const MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION: &str = "系统运行配置";
pub const MOUNT_TRANSPARENCY_TABLE_ROWS_OF_TODAY: [&str; 2] = [
    MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION,
    MOUNT_TRANSPARENCY_TABLE_UNMOUNT_MARKER,
];
const ROOT_RECORD_INSTANCE: std::ops::Range<usize> = 20..24;
const READ_WHEN_THE_SLOT_IS_READ: &str = "读槽时判";

pub const ROOT_RECORD_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
    not_compared(
        "magic",
        0..4,
        READ_WHEN_THE_SLOT_IS_READ,
        MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
    ),
    must_be_the_same("fsid", 4..20, MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION),
    determined_relation(
        "实例代号（样本）",
        ROOT_RECORD_INSTANCE,
        MountTransparencyField::InstanceGeneration,
        MOUNT_TRANSPARENCY_TABLE_UNMOUNT_MARKER,
    ),
    not_compared(
        "回收门槛 t、钉槽 p",
        24..33,
        "每次现算",
        MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
    ),
];

pub const SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES: &[SystemConfigurationFieldByteRange] = &[
    system_configuration_field(
        not_compared(
            "magic",
            0..4,
            READ_WHEN_THE_SLOT_IS_READ,
            MOUNT_TRANSPARENCY_TABLE_STRUCTURAL_FIELDS,
        ),
        SystemConfigurationTier::ImmutableConfiguration,
    ),
    immutable_and_the_same("格式版本", 4..6),
    system_configuration_field(
        determined_relation(
            "整理低水位 / 高水位",
            6..22,
            MountTransparencyField::SystemRuntimeConfiguration,
            MOUNT_TRANSPARENCY_TABLE_RUNTIME_CONFIGURATION,
        ),
        SystemConfigurationTier::RuntimeConfiguration,
    ),
];

pub const TREE_TABLE_ENTRY_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
    must_be_the_same("树 ID", 0..8, MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION),
];
