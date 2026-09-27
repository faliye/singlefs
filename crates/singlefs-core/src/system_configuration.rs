//! 系统配置（D22（单元原子性怎么合成） 已定项 9 / 已定项 15 / 已定项 16 / 已定项 26）：489 字节住 4096 字节的槽，
//! 每盘两槽轮换，整槽校验和罩 4096 含补齐、自身按 0 参与。字段顺序照 `layout/01-first-txn.md` 一那一节的字段表。
//!
//! 内存里按 D22（单元原子性怎么合成） 已定项 26 的可改性分四类装：系统不可变配置 389 + 系统可变配置 4 +
//! 系统运行配置 36 + 系统运行量 60 = 489。**分类只管内存表示，盘上的序列化顺序照旧按字段表来**：
//! 四类在槽里是交错的——槽世代号与整槽校验和这两条系统运行量夹在自举头中间，节点大小这条系统可变配置
//! 夹在几何段开头。[`SystemConfiguration::to_slot`] 每写一段都报自己属于哪一档，写完比对四档的字节数。

use singlefs_format::{
    DATA_UNIT_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, JOURNAL_RECORD_BYTES,
    JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LOC_ENTRY, NODE_BYTES, ROOT_RING_BASE_SLOT,
    ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS, SLOT_BYTES,
    SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, WIDE_CHECKSUM_BYTES,
};

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use crate::bytes::{ByteReader, ByteWriter};
use crate::checksum::{wide_checksum_field_holds, wide_checksum_with_field_zeroed};
use crate::journal::slot_after_the_journal_ring;
use crate::root_ring::{RootRingSlotsPerRegion, RootRingSlotsPerRegionOutOfRange};

pub const SYSTEM_CONFIGURATION_MAGIC: [u8; 4] = *b"SFSB";
pub const FORMAT_VERSION: u16 = 1;
/// incompat 位 1 = 第一条纯 SSD 布局线，系统配置带回退下界 F、根记录带卸载记号的那一版（D15（格式冻结政策） 已定项 4，
/// 用户 2026-09-26 定「换 incompat 位，挂不上」）；mkfs 起就置上。位 n 住第 n div 8 个字节的第 n mod 8 低位
/// （D22（单元原子性怎么合成） 已定项 13），位 1 是第一个字节从最低位数起的第 2 位。
pub const INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT: u8 = 0x02;
/// incompat 位 0 退役：带回退见证、系统配置不带 F 的那一版（D15（格式冻结政策） 已定项 4，用户 2026-09-26）。
/// 位一旦用过不许回收（`.claude/kb/feature-bits.md`）；它不在 [`SUPPORTED_INCOMPAT_BITS`] 里，读者见到它按不认识的位拒。
pub const INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT: u8 = 0x01;
pub const SUPPORTED_INCOMPAT_BITS: u8 =
    INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT;
const FEATURE_BITS_OFFSET: usize = 4 + 2;
const FEATURE_BITMAP_BYTES: usize = 32;
/// 写入者身份：实现标识 16 字节 ASCII 零补齐 + 版本 4（D17（实现分层与第三方管道） 债 3、I-1.5）。
pub const WRITER_IDENTITY_NAME: &[u8] = b"singlefs-rs";
pub const WRITER_IDENTITY_VERSION: u32 = 1;
/// 校验和算法标识登记表：0 无效、1 CRC-32C。
pub const CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI: u8 = 1;
/// 自举头：magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 + 校验和算法标识 1 + 本盘设备号 4 + 设备数 4 + 槽世代号 8。
pub const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8;
const FSID_OFFSET: usize = 4 + 2 + 96;
/// 几何段里「每区槽数 S」那一字节（字段表 `layout/01-first-txn.md` 一：R 在 361、S 在 362）。
/// 写侧 [`SystemConfiguration::to_slot`] 写它之前断言位置、读侧 [`SystemConfiguration::parse_slot`] 按它切，
/// 一个偏移只有这一处定义。
const ROOT_RING_SLOTS_PER_REGION_OFFSET: u64 = 362;
const REGION_DEVICES_OFFSET: u64 = 379;
/// 几何段里「单元区起始槽号」那 8 字节（字段表 `layout/01-first-txn.md` 一；D3（空间分配） 已定项 10 ④）。
/// 写侧 [`SystemConfiguration::to_slot`] 写它之前断言位置、读侧 [`unit_area_start_slot_recorded_in_the_slot`] 按它切。
const UNIT_AREA_START_SLOT_OFFSET: u64 = 417;
const TAIL_OFFSET: u64 = 469;
/// 回退下界 F 那一行的宽：checkpoint_txg 小端 8 字节（D22（单元原子性怎么合成） 已定项 9 的字段表）。
const ROLLBACK_FLOOR_FIELD_BYTES: u64 = 8;
/// 回退下界 F 住字段表最后一行、journal 实例代号之后（D22（单元原子性怎么合成） 已定项 9）：偏移 = 字段表合计减它自己的宽，
/// 从格式常量 [`SYSTEM_CONFIGURATION_BYTES`] 推出来，不另写一个数。写侧写它之前断言位置、读侧按它切。
const ROLLBACK_FLOOR_OFFSET: u64 = SYSTEM_CONFIGURATION_BYTES - ROLLBACK_FLOOR_FIELD_BYTES;
/// D2（RAID 条带策略） 已定项 18：第一版 w_max 与 g 都写 4。
const MAXIMUM_STRIPE_WIDTH: u8 = 4;
const GROUP_SIZE: u8 = 4;
/// D16（发布语义） 已定项 5 的两个可调值。
const TIME_THRESHOLD_SECONDS: u32 = 5;
const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;
/// 整理低水位 / 高水位 / 停止线三条（D26（后台整理与放置回收） 已定项 1）。
const COMPACTION_WATERMARK_COUNT: usize = 3;
/// 三条水位第一版恒 0，0 的含义是「用内置默认」（D22（单元原子性怎么合成） 已定项 15 ④）。
const COMPACTION_WATERMARK_BUILT_IN_DEFAULT: u64 = 0;

/// 在飞记录数上限 = 环槽数 ÷ F，语义是一次恢复重放的前缀最多这么多条（D23（journal 的角色与格式） 已定项 18，
/// 2026-09-14 用户定案）。环槽数 = 环长 ÷ [`JOURNAL_RECORD_BYTES`]，F = [`JOURNAL_SAFETY_FACTOR`]（I-8.1（环几何够大）），两次都取整数部分。
/// core 里写系统配置那一格、mkfs 判环够不够、恢复取前缀、一次发布最多切几条记录，都调这一份。
///
/// 式子住 core 自己这里：`singlefs-format` 只放标量（D13（验证路线） 已定项 5；2026-09-27 用户定案「三方各算一份 + 交叉断言」）。
/// 实验装置 E158（择根与修复四岔路） 自写一份、不调它；checker 不算在飞上限，不另造。两份在环长上的交叉断言在
/// `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 的 `journal_in_flight_record_limit_cross_check_tests` 模块。
#[must_use]
pub fn journal_in_flight_record_limit(journal_ring_bytes: u64) -> u64 {
    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}

/// 这个环长下的在飞记录数上限（[`journal_in_flight_record_limit`]）装不装得进系统配置里它那 4 字节
/// （`layout/01-first-txn.md` 一「journal 在飞记录数上限 | 4」）。环长 ≥ 2³² × 12288 字节时装不下：mkfs 在任何写之前拒
/// （`make_filesystem` 的 `JournalInFlightRecordLimitWiderThanItsFourByteField`），读者择系统配置时拒
/// （`recovery::choose_system_configuration` 报 `JournalRingBytesOutsideTheSupportedRange`）——[`SystemConfiguration::to_slot`]
/// 写这 4 字节那一句的 expect 靠的就是这两道。
#[must_use]
pub fn journal_in_flight_record_limit_fits_its_four_byte_field(journal_ring_bytes: u64) -> bool {
    u32::try_from(journal_in_flight_record_limit(journal_ring_bytes)).is_ok()
}

/// 一槽系统配置里记着的单元区起始槽号（偏移 417 的 8 字节，小端）。只切字节、不判值：读者择系统配置时拿它与环长现算的起点比
/// （`recovery` 的 `system_configuration_values_this_reader_accepts`）。
///
/// # Panics
/// 槽短于 4096 字节：调用方先过了 [`SystemConfiguration::parse_slot`]（它判过槽宽）才调这里。
#[must_use]
pub fn unit_area_start_slot_recorded_in_the_slot(slot: &[u8]) -> SlotNumber {
    SlotNumber(
        ByteReader::at(
            slot,
            usize::try_from(UNIT_AREA_START_SLOT_OFFSET).expect("字段表里的偏移装得进 usize"),
        )
        .get_u64(),
    )
}

/// 系统配置槽里一个字段属于哪一档可改性（D22（单元原子性怎么合成） 已定项 26）。
/// 封闭集合：四类之外没有第五类，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotFieldMutability {
    /// 改它 = 重建文件系统。
    ImmutableConfiguration,
    /// 改它 = 重建索引、不丢数据（一次满量级全盘 I/O）。
    MutableConfiguration,
    /// 改它无代价：mkfs 时设默认值，每次挂载可重设、而且要回写盘上。
    RuntimeConfiguration,
    /// 不是配置：用户从来不能设，文件系统自己维护，值每次发布都在变。
    RuntimeQuantity,
}

/// 系统不可变配置里那五个 mkfs 探测或取参定下的尺寸。
///
/// 单拎成一个类型，是因为 mkfs 的参数只给得出这五个：本盘设备号由 mkfs 逐盘现填，设备数由池里的盘数现数
/// （[`crate::make_filesystem::MakeFilesystemParameters`] 拿它当 `geometry` 字段）。
/// 五个都在字段表「几何」段里，都属于系统不可变配置——改它们要重建文件系统。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemImmutableSizes {
    /// mkfs 时探测到的 physical_block_size（2026-09-14 用户定案的字段，给挂载与 checker 比对用）。
    pub physical_block_size: u32,
    /// mkfs 时探测的 io_min。
    pub minimum_input_output_bytes: u32,
    /// 固定结构槽距 = 4096 向上取整到 io_min 的整数倍（D2（RAID 条带策略） 已定项 19）。
    pub fixed_structure_slot_spacing: u32,
    /// journal 环长（mkfs 参数，默认 768 MiB，环 ≤ 容量 / 4，D23（journal 的角色与格式） 已定项 19）。
    pub journal_ring_bytes: u64,
    /// 每区槽数 S（mkfs 参数，第一版写 [`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM`] = 8，
    /// 挂载时判 4..16 区间，D22（单元原子性怎么合成） 已定项 1 的字段表）。环几何的每一处都读它，
    /// 不读编译期常量（C506（每区槽数 S 写成编译期常量，条文说它住系统配置））。
    pub root_ring_slots_per_region: RootRingSlotsPerRegion,
}

impl SystemImmutableSizes {
    /// 槽距 = 4096 向上取整到 io_min 的整数倍。
    #[must_use]
    pub fn slot_spacing_for(minimum_input_output_bytes: u32) -> u32 {
        let floor = u32::try_from(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES).expect("4096");
        if minimum_input_output_bytes == 0 {
            return floor;
        }
        floor.div_ceil(minimum_input_output_bytes) * minimum_input_output_bytes
    }
}

/// 系统不可变配置：389 字节，改它 = 重建文件系统（D22（单元原子性怎么合成） 已定项 26 第一档）。
///
/// 字段表（`layout/01-first-txn.md` 一「系统配置字段表」）里归这一档的 37 行：
/// 自举头 8 行 147（magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 +
/// 校验和算法标识 1 + 本盘设备号 4 + 设备数 devs 4）+ 加密预留 6 行 114（系统配置自身 MAC 16 +
/// nonce 水位 12 + KDF 标识 4 + 加密类型 1 + MAC 长度声明 1 + 主密钥槽 80）+ 几何段除节点大小之外的
/// 23 行 128。这 37 行里只有下面 5 个字段（`sizes` 里 4 个值，合起来 8 个值）在内存里存着，
/// 其余由 [`SystemConfiguration::to_slot`] 照格式常量写死。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemImmutableConfiguration {
    pub filesystem_identifier: [u8; 16],
    /// 字段表里唯一的盘级字段：别的都是池级、两盘同内容（D22（单元原子性怎么合成） 已定项 9 射程）。
    pub this_device: DeviceIdentity,
    pub device_count: u32,
    pub region_devices: [DeviceIdentity; 3],
    pub sizes: SystemImmutableSizes,
}

impl SystemImmutableConfiguration {
    /// 这一档在字段表里占的字节数：147 + 114 + 128。
    pub const FIELD_TABLE_BYTES: u64 = 389;
}

/// 系统可变配置：4 字节，改它 = 重建索引、不丢数据（D22（单元原子性怎么合成） 已定项 26 第二档）。
///
/// 字段表里归这一档的只有 1 行：几何段的「节点大小 4」。
/// **没有字段**：节点大小唯一的权威定义是格式常量 `NODE_BYTES`（`singlefs-format`，门禁 27 号按 kb 的
/// format-const 标记钉住它的值），mkfs 今天不收它当参数——
/// `grep -rn 'NODE_BYTES' crates/*/src` 命中的全是读，[`crate::make_filesystem::MakeFilesystemParameters`]
/// 里没有这一项；改它要走的那条重建索引的路第一版也没有（已定项 26 第二档逐字「第一版没有这条路」）。
/// 真有了 mkfs 参数，值落在这里，格式常量退回去当默认值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemMutableConfiguration;

impl SystemMutableConfiguration {
    /// 这一档在字段表里占的字节数：节点大小 4。
    pub const FIELD_TABLE_BYTES: u64 = 4;

    /// 节点大小（D8（核心索引结构） 已定项 2）。
    #[must_use]
    pub fn node_bytes(self) -> u32 {
        u32::try_from(NODE_BYTES).expect("节点大小")
    }
}

/// 系统运行配置：36 字节，改它无代价、每次挂载可重设并回写（D22（单元原子性怎么合成） 已定项 26 第三档）。
///
/// 字段表里归这一档的 3 行：可调值段的「T_time 4」「T_dirty 8」「整理低 / 高 / 停三水位 8 × 3」。
/// **没有字段**：挂载选项通道不存在（C422（挂载选项通道不存在）），这三行今天既没有可设的入口、
/// 也没有触发路径——T_time 与 T_dirty 的发布触发没实现，整理三水位恒 0 = 内置默认
/// （D22（单元原子性怎么合成） 已定项 15 ④）。所以这一档拿内置默认值写盘，不带状态。
/// 真有了挂载选项通道，值落在这里，下面三个方法退回去当默认值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemRuntimeConfiguration;

impl SystemRuntimeConfiguration {
    /// 这一档在字段表里占的字节数：4 + 8 + 24。
    pub const FIELD_TABLE_BYTES: u64 = 36;

    /// T_time（D16（发布语义） 已定项 5）。
    #[must_use]
    pub fn time_threshold_seconds(self) -> u32 {
        TIME_THRESHOLD_SECONDS
    }

    /// T_dirty（D16（发布语义） 已定项 5）；挂载时的有效值另按环长夹取，那不改盘上这 8 字节。
    #[must_use]
    pub fn dirty_threshold_bytes(self) -> u64 {
        DIRTY_THRESHOLD_BYTES
    }

    /// 整理低水位 / 高水位 / 停止线，按字段表的次序（D26（后台整理与放置回收） 已定项 1）。
    #[must_use]
    pub fn compaction_watermarks(self) -> [u64; COMPACTION_WATERMARK_COUNT] {
        [COMPACTION_WATERMARK_BUILT_IN_DEFAULT; COMPACTION_WATERMARK_COUNT]
    }
}

/// 系统运行量：60 字节。**不是配置**——用户从来不能设，文件系统自己维护，值随发布或抬 F 在变
/// （D22（单元原子性怎么合成） 已定项 26 表下那一行）。
///
/// 字段表里归这一类的 5 行：自举头的「槽世代号 8」「整槽校验和 32」+ 运行时段的「journal tail 8」
/// 「journal 实例代号 4」「回退下界 F 8」。整槽校验和不在内存里存着：它罩着整槽 4096（自身按 0 参与），
/// 由 [`SystemConfiguration::to_slot`] 在别的字节都写完之后算出来填进去。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemRuntimeQuantities {
    pub slot_generation: u64,
    pub journal_tail: u64,
    pub journal_instance: InstanceGeneration,
    /// 回退下界 F 另记在系统配置里的那一份（D16（发布语义） 已定项 1「生效」：F 先写进每块盘的系统配置再抬；
    /// D22（单元原子性怎么合成） 已定项 9）。与根记录里的 F 同一个量、同一个名字，住的地方不同。
    /// 读出来之后怎么用（恢复时生效值取两处读得出的最大值）不在这个模块。
    pub rollback_floor: CheckpointTxg,
}

impl SystemRuntimeQuantities {
    /// 这一类在字段表里占的字节数：8 + 32 + 8 + 4 + 8。
    pub const FIELD_TABLE_BYTES: u64 = 60;
}

/// 一个系统配置槽的内容，按 D22（单元原子性怎么合成） 已定项 26 的可改性分四类装。字段表之后到槽末是补齐，写 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemConfiguration {
    pub immutable: SystemImmutableConfiguration,
    pub mutable: SystemMutableConfiguration,
    pub runtime: SystemRuntimeConfiguration,
    pub quantities: SystemRuntimeQuantities,
}

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 写一个系统配置槽时，四档各自写出了多少字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotBytesByMutability {
    pub immutable_configuration_bytes: u64,
    pub mutable_configuration_bytes: u64,
    pub runtime_configuration_bytes: u64,
    pub runtime_quantity_bytes: u64,
}

/// 按可改性分档记账的槽写入器：每写一段都要报它属于哪一档，写完比对四档各自的字节数与字段表。
/// 分档只是记账——字节仍按字段表的顺序一段接一段写下去，分档不改盘上的任何一个字节。
struct MutabilityTaggedSlotWriter {
    writer: ByteWriter,
    immutable_configuration_bytes: u64,
    mutable_configuration_bytes: u64,
    runtime_configuration_bytes: u64,
    runtime_quantity_bytes: u64,
}

impl MutabilityTaggedSlotWriter {
    fn new(total_bytes: usize) -> Self {
        Self {
            writer: ByteWriter::new(total_bytes),
            immutable_configuration_bytes: 0,
            mutable_configuration_bytes: 0,
            runtime_configuration_bytes: 0,
            runtime_quantity_bytes: 0,
        }
    }

    fn counter_for(&mut self, mutability: SlotFieldMutability) -> &mut u64 {
        match mutability {
            SlotFieldMutability::ImmutableConfiguration => &mut self.immutable_configuration_bytes,
            SlotFieldMutability::MutableConfiguration => &mut self.mutable_configuration_bytes,
            SlotFieldMutability::RuntimeConfiguration => &mut self.runtime_configuration_bytes,
            SlotFieldMutability::RuntimeQuantity => &mut self.runtime_quantity_bytes,
        }
    }

    fn write(
        &mut self,
        mutability: SlotFieldMutability,
        write_fields: impl FnOnce(&mut ByteWriter),
    ) {
        let before = self.writer.position();
        write_fields(&mut self.writer);
        let written =
            u64::try_from(self.writer.position() - before).expect("一段的字节数装得进 u64");
        *self.counter_for(mutability) += written;
    }

    /// 每一档写出的字节数必须等于字段表给这一档的预算，四档之和必须是 489；
    /// 对不上就是某个字段被分错了档或写漏了，当场断言（`code-discipline.md`：不变量写成断言）。
    fn finish(self) -> (Vec<u8>, SlotBytesByMutability) {
        self.writer
            .assert_position(SYSTEM_CONFIGURATION_BYTES, "系统配置");
        let accounting = SlotBytesByMutability {
            immutable_configuration_bytes: self.immutable_configuration_bytes,
            mutable_configuration_bytes: self.mutable_configuration_bytes,
            runtime_configuration_bytes: self.runtime_configuration_bytes,
            runtime_quantity_bytes: self.runtime_quantity_bytes,
        };
        assert_eq!(
            accounting.immutable_configuration_bytes,
            SystemImmutableConfiguration::FIELD_TABLE_BYTES,
            "系统不可变配置写出的字节数与字段表不符"
        );
        assert_eq!(
            accounting.mutable_configuration_bytes,
            SystemMutableConfiguration::FIELD_TABLE_BYTES,
            "系统可变配置写出的字节数与字段表不符"
        );
        assert_eq!(
            accounting.runtime_configuration_bytes,
            SystemRuntimeConfiguration::FIELD_TABLE_BYTES,
            "系统运行配置写出的字节数与字段表不符"
        );
        assert_eq!(
            accounting.runtime_quantity_bytes,
            SystemRuntimeQuantities::FIELD_TABLE_BYTES,
            "系统运行量写出的字节数与字段表不符"
        );
        (self.writer.into_bytes(), accounting)
    }
}

impl SystemConfiguration {
    /// 写出的 4096 字节：字段照字段表的顺序一段接一段，四类在槽里交错。
    #[must_use]
    pub fn to_slot(&self) -> Vec<u8> {
        self.to_slot_with_mutability_accounting().0
    }

    /// 写出槽，连带报每一档写出了多少字节：每一段写之前报自己属于哪一档，
    /// [`MutabilityTaggedSlotWriter::finish`] 收尾时比对四档的字节数与字段表的 389 / 4 / 36 / 60。
    #[must_use]
    pub fn to_slot_with_mutability_accounting(&self) -> (Vec<u8>, SlotBytesByMutability) {
        let mut slot = MutabilityTaggedSlotWriter::new(slot_bytes());
        slot.write(SlotFieldMutability::ImmutableConfiguration, |writer| {
            writer.put(&SYSTEM_CONFIGURATION_MAGIC);
            writer.put_u16(FORMAT_VERSION);
            writer.put_u8(INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT);
            writer.skip(95);
            writer.assert_position(
                u64::try_from(FSID_OFFSET).expect("fsid 偏移"),
                "系统配置 fsid",
            );
            writer.put(&self.immutable.filesystem_identifier);
            let mut writer_identity = [0u8; 16];
            writer_identity[..WRITER_IDENTITY_NAME.len()].copy_from_slice(WRITER_IDENTITY_NAME);
            writer.put(&writer_identity);
            writer.put_u32(WRITER_IDENTITY_VERSION);
            writer.put_u8(CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI);
            writer.put_u32(self.immutable.this_device.0);
            writer.put_u32(self.immutable.device_count);
        });
        slot.write(SlotFieldMutability::RuntimeQuantity, |writer| {
            writer.put_u64(self.quantities.slot_generation);
            writer.assert_position(
                u64::try_from(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET).expect("校验和偏移"),
                "系统配置整槽校验和",
            );
            writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        });
        slot.write(SlotFieldMutability::ImmutableConfiguration, |writer| {
            writer.skip(16 + 12 + 4); // 系统配置 MAC、nonce 水位、KDF 标识：第一版全 0
            writer.put_u8(0); // 加密类型：关
            writer.put_u8(16); // MAC 长度声明
            writer.skip(80); // 主密钥槽：内联进槽，加密关时全 0
        });
        slot.write(SlotFieldMutability::MutableConfiguration, |writer| {
            writer.put_u32(self.mutable.node_bytes());
        });
        slot.write(SlotFieldMutability::ImmutableConfiguration, |writer| {
            writer.put_u32(u32::try_from(DATA_UNIT_BYTES).expect("单元大小"));
            writer.put_u32(u32::try_from(SLOT_BYTES).expect("落点粒度"));
            writer.put_u32(u32::try_from(LOC_ENTRY).expect("位置条目宽度"));
            writer.put_u32(self.immutable.sizes.physical_block_size);
            writer.put_u32(0); // 扩展点声明值 N：第一版 0（D21（权威态与派生态的分界） 已定项 8）
            writer.put_u64(JOURNAL_RING_START_SLOT);
            writer.put_u64(self.immutable.sizes.journal_ring_bytes);
            writer.put_u32(u32::try_from(JOURNAL_RECORD_BYTES).expect("记录尺寸"));
            let in_flight_limit =
                journal_in_flight_record_limit(self.immutable.sizes.journal_ring_bytes);
            writer.put_u32(u32::try_from(in_flight_limit).expect(
                "在飞上限装得进 4 字节：mkfs 与读者择系统配置都按 journal_in_flight_record_limit_fits_its_four_byte_field 拒过装不下的环长",
            ));
            writer.put_u64(in_flight_limit * JOURNAL_RECORD_BYTES);
            writer.put_u32(u32::try_from(JOURNAL_SAFETY_FACTOR).expect("安全系数"));
            writer.put_u8(u8::try_from(ROOT_RING_REGIONS).expect("R"));
            writer.assert_position(ROOT_RING_SLOTS_PER_REGION_OFFSET, "每区槽数 S");
            writer.put_u8(
                self.immutable
                    .sizes
                    .root_ring_slots_per_region
                    .to_system_configuration_field(),
            );
            writer.put_u32(u32::try_from(ROOT_RING_PRIME_STEP).expect("P"));
            writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));
            writer.put_u64(ROOT_RING_BASE_SLOT);
            writer.assert_position(REGION_DEVICES_OFFSET, "根环逐区域设备身份");
            for region_device in &self.immutable.region_devices {
                writer.put_u32(region_device.0);
            }
            writer.put_u8(MAXIMUM_STRIPE_WIDTH);
            writer.put_u8(GROUP_SIZE);
            writer.skip(24); // 映射来源：第一版全 0
            writer.assert_position(UNIT_AREA_START_SLOT_OFFSET, "单元区起始槽号");
            // 第一版 = journal 环末尾的下一个槽（D3（空间分配） 已定项 10 ④），随环长走（D23（journal 的角色与格式） 已定项 19 ③）：
            // 由环长算只有 `journal::slot_after_the_journal_ring` 一处，mkfs 写实例表的落点、挂载建空闲图的起点都从它来。
            writer.put_u64(slot_after_the_journal_ring(self.immutable.sizes.journal_ring_bytes).0);
            writer.put_u32(self.immutable.sizes.minimum_input_output_bytes);
            writer.put_u32(self.immutable.sizes.fixed_structure_slot_spacing);
        });
        slot.write(SlotFieldMutability::RuntimeConfiguration, |writer| {
            writer.put_u32(self.runtime.time_threshold_seconds());
            writer.put_u64(self.runtime.dirty_threshold_bytes());
            for watermark in self.runtime.compaction_watermarks() {
                writer.put_u64(watermark);
            }
        });
        slot.write(SlotFieldMutability::RuntimeQuantity, |writer| {
            writer.assert_position(TAIL_OFFSET, "journal tail");
            writer.put_u64(self.quantities.journal_tail);
            writer.put_u32(self.quantities.journal_instance.0);
            writer.assert_position(ROLLBACK_FLOOR_OFFSET, "系统配置里的回退下界 F");
            writer.put_u64(self.quantities.rollback_floor.0);
        });
        let (mut bytes, accounting) = slot.finish();
        let digest = wide_checksum_with_field_zeroed(
            &bytes,
            slot_bytes(),
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        (bytes, accounting)
    }

    /// 读者：magic、整槽校验和、incompat 位三关都过才解。magic 或整槽校验和不过是
    /// [`SystemConfigurationSlotRefusal::NotSelfDescribing`]（这一槽不可择）；这两关都过、incompat 位图不认识是
    /// [`SystemConfigurationSlotRefusal::IncompatBitsNotRecognized`]（这一槽同样不可择，但它是一份完整的系统配置，布局不认识）；
    /// 解开之后按 D22 已定项 1 的字段表判每区槽数 S 的区间，越界是
    /// [`SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange`]（整池拒绝挂载）。
    ///
    /// # Errors
    /// 见三个成员各自的说明。
    pub fn parse_slot(bytes: &[u8]) -> Result<Self, SystemConfigurationSlotRefusal> {
        if bytes.len() < slot_bytes() || bytes[..4] != SYSTEM_CONFIGURATION_MAGIC {
            return Err(SystemConfigurationSlotRefusal::NotSelfDescribing);
        }
        if !wide_checksum_field_holds(bytes, slot_bytes(), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET) {
            return Err(SystemConfigurationSlotRefusal::NotSelfDescribing);
        }
        if !incompat_bits_are_mountable(bytes) {
            return Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized {
                incompat_bitmap: incompat_bitmap_of(bytes),
            });
        }
        let root_ring_slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(
            u64::from(bytes[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")]),
        )
        .map_err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange)?;
        let mut reader = ByteReader::at(bytes, FSID_OFFSET);
        let filesystem_identifier: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        reader.skip(16 + 4 + 1);
        let this_device = DeviceIdentity(reader.get_u32());
        let device_count = reader.get_u32();
        let slot_generation = reader.get_u64();
        reader.skip(32 + 16 + 12 + 4 + 1 + 1 + 80 + 4 + 4 + 4 + 4);
        let physical_block_size = reader.get_u32();
        reader.skip(4 + 8);
        let journal_ring_bytes = reader.get_u64();
        let mut region_reader =
            ByteReader::at(bytes, usize::try_from(REGION_DEVICES_OFFSET).expect("379"));
        let region_devices = [
            DeviceIdentity(region_reader.get_u32()),
            DeviceIdentity(region_reader.get_u32()),
            DeviceIdentity(region_reader.get_u32()),
        ];
        region_reader.skip(1 + 1 + 24 + 8);
        let minimum_input_output_bytes = region_reader.get_u32();
        let fixed_structure_slot_spacing = region_reader.get_u32();
        let mut tail_reader = ByteReader::at(bytes, usize::try_from(TAIL_OFFSET).expect("469"));
        let journal_tail = tail_reader.get_u64();
        let journal_instance = InstanceGeneration(tail_reader.get_u32());
        let mut rollback_floor_reader = ByteReader::at(
            bytes,
            usize::try_from(ROLLBACK_FLOOR_OFFSET).expect("字段表里的偏移装得进 usize"),
        );
        let rollback_floor = CheckpointTxg(rollback_floor_reader.get_u64());
        Ok(Self {
            immutable: SystemImmutableConfiguration {
                filesystem_identifier,
                this_device,
                device_count,
                region_devices,
                sizes: SystemImmutableSizes {
                    physical_block_size,
                    minimum_input_output_bytes,
                    fixed_structure_slot_spacing,
                    journal_ring_bytes,
                    root_ring_slots_per_region,
                },
            },
            // 节点大小那 4 字节不读回来：它今天只可能是格式常量 `NODE_BYTES`，读回来会多出一个
            // 「盘上的值与格式常量不符」的状态而没人判它。有了 mkfs 参数那天连着判定一起加。
            mutable: SystemMutableConfiguration,
            // 可调值那 36 字节不读回来：挂载选项通道不存在（C422（挂载选项通道不存在）），
            // 读回来的值没有任何使用者。
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation,
                journal_tail,
                journal_instance,
                rollback_floor,
            },
        })
    }
}

/// 一个系统配置槽读不成的原因，按调用方能据以行动的粒度分（`code-discipline.md`：
/// 错误成员按调用方要做的决定分）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemConfigurationSlotRefusal {
    /// magic、整槽校验和两关里有一关不过 ⇒ **这一槽不可择**：换一槽、换一盘还可以试，
    /// 系统配置每盘两槽、池里每盘一份买的就是这份冗余（D22（单元原子性怎么合成） 已定项 8 第 1 条）。
    NotSelfDescribing,
    /// magic 与整槽校验和两关都过、incompat 位图里有这个读者不认识的位或缺布局身份位（`incompat_bits_are_mountable` 不过）
    /// ⇒ **这一槽同样不可择**（incompat 档「不认识不许挂」，`.claude/rules/fs-design.md`），但它不是坏槽：是一份完整的系统配置，
    /// 布局不是这个读者认得的那一版（退役的位 0 那一版旧镜像，或更新的实现写的，D15（格式冻结政策） 已定项 4）。
    /// 与 [`Self::NotSelfDescribing`] 分开报，调用方据此把「布局不认识」与「数据坏了」分开。带着那一槽的 incompat 位图原样。
    IncompatBitsNotRecognized { incompat_bitmap: IncompatBitmap },
    /// 自述的每区槽数 S 落在格式承诺的区间之外 ⇒ **整池拒绝挂载**，不换一槽再试：S 是池级字段，
    /// 两盘四槽写的是同一个值，换一槽读到的还是它（D22（单元原子性怎么合成） 已定项 9 的射程：
    /// 字段表里只有本盘设备号是盘级的）。
    RootRingSlotsPerRegionOutOfRange(RootRingSlotsPerRegionOutOfRange),
}

/// 一个系统配置槽里的 incompat 位图（feature bits 的第一段 32 字节，D22（单元原子性怎么合成） 已定项 13）。
/// 读者不认识它时原样带出来（[`SystemConfigurationSlotRefusal::IncompatBitsNotRecognized`]），调用方拿它说是哪一版布局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IncompatBitmap(pub [u8; FEATURE_BITMAP_BYTES]);

/// 槽里的 incompat 位图原样拷出来。槽比位图的末端短是调用方的 bug：[`SystemConfiguration::parse_slot`] 先判过槽宽才走到这里。
fn incompat_bitmap_of(slot: &[u8]) -> IncompatBitmap {
    IncompatBitmap(
        slot[FEATURE_BITS_OFFSET..FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES]
            .try_into()
            .expect("切的正好是位图那 32 字节"),
    )
}

/// incompat 位图里不认识的位 ⇒ 挂不上（`.claude/rules/fs-design.md`）；且必须带布局身份位。
/// 退役的位 0（系统配置不带 F 的旧镜像）不在 [`SUPPORTED_INCOMPAT_BITS`] 里，按不认识的位拒，
/// 与新的布局身份位同时置上也拒（D15（格式冻结政策） 已定项 4）。
#[must_use]
pub fn incompat_bits_are_mountable(slot: &[u8]) -> bool {
    let incompat = &slot[FEATURE_BITS_OFFSET..FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];
    let unknown_in_first_byte = incompat[0] & !SUPPORTED_INCOMPAT_BITS;
    let unknown_in_rest = incompat[1..].iter().any(|byte| *byte != 0);
    let has_layout_identity =
        incompat[0] & INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT != 0;
    unknown_in_first_byte == 0 && !unknown_in_rest && has_layout_identity
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

    fn sample() -> SystemConfiguration {
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

    /// 把槽里 incompat 位图第一个字节改成 `first_incompat_byte`，整槽校验和重封：magic 与校验和两关照样过，拒它的只能是 incompat 判。
    fn slot_with_first_incompat_byte(first_incompat_byte: u8) -> Vec<u8> {
        let mut slot = sample().to_slot();
        slot[FEATURE_BITS_OFFSET] = first_incompat_byte;
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            slot_bytes(),
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        slot
    }

    #[test]
    fn system_configuration_is_489_bytes_in_a_4096_slot_and_round_trips() {
        let slot = sample().to_slot();
        assert_eq!(slot.len(), 4096);
        assert!(
            slot[489..].iter().all(|byte| *byte == 0),
            "489 之后全是补齐 0：字段表之后不装别的东西"
        );
        assert_eq!(
            slot[6], 0x02,
            "incompat 只有位 1（新的布局身份），退役的位 0 不置"
        );
        assert_eq!(
            &slot[FSID_OFFSET + 16..FSID_OFFSET + 16 + 11],
            b"singlefs-rs"
        );
        assert_eq!(SystemConfiguration::parse_slot(&slot), Ok(sample()));
        assert_eq!(
            slot[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")],
            8,
            "每区槽数 S 写在偏移 362 那一字节，mkfs 第一版写 8"
        );
        assert!(wide_checksum_field_holds(
            &slot,
            4096,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET
        ));
    }

    /// 坏槽与不认识的 incompat 位都挂不上，但报的成员不同：前者是这一槽自证不过（数据坏了），后者自证得过、只是布局不认识。
    #[test]
    fn corrupted_slot_and_unknown_incompat_bit_are_both_unmountable() {
        let mut slot = sample().to_slot();
        slot[4000] ^= 1;
        assert_eq!(
            SystemConfiguration::parse_slot(&slot),
            Err(SystemConfigurationSlotRefusal::NotSelfDescribing),
            "补齐区改一字节也失配：整槽校验和罩 4096"
        );
        let mut unknown = sample().to_slot();
        unknown[6] |= 0x04;
        assert!(!incompat_bits_are_mountable(&unknown));
        let resealed_unknown = slot_with_first_incompat_byte(
            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 0x04,
        );
        let mut expected_bitmap = [0u8; FEATURE_BITMAP_BYTES];
        expected_bitmap[0] =
            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 0x04;
        assert_eq!(
            SystemConfiguration::parse_slot(&resealed_unknown),
            Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized {
                incompat_bitmap: IncompatBitmap(expected_bitmap),
            }),
            "位 2 不认识、整槽重封过：报布局不认识、带着位图，不报自证不过"
        );
    }

    /// 位 0 退役（D15（格式冻结政策） 已定项 4，用户 2026-09-26）：只带位 0 的旧镜像、位 0 与位 1 都带的、一个布局身份位都不带的，
    /// 三样都按 incompat 判拒成「这一槽不可择」、报的是布局不认识（带着那一槽的位图），不是自证不过；只带位 1 的收。
    /// 四份槽都重封过整槽校验和，拒它们的只能是 incompat 判。
    #[test]
    fn slot_carrying_the_retired_layout_bit_zero_is_refused_like_an_unknown_incompat_bit() {
        for (first_incompat_byte, what) in [
            (
                INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
                "只带退役的位 0：带回退见证、系统配置不带 F 的旧镜像",
            ),
            (
                INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT
                    | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT,
                "位 0 与位 1 都带",
            ),
            (0, "一个布局身份位都不带"),
        ] {
            let slot = slot_with_first_incompat_byte(first_incompat_byte);
            assert!(
                wide_checksum_field_holds(&slot, 4096, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET),
                "{what}：这一槽自证得过"
            );
            assert!(!incompat_bits_are_mountable(&slot), "{what}：挂不上");
            let mut expected_bitmap = [0u8; FEATURE_BITMAP_BYTES];
            expected_bitmap[0] = first_incompat_byte;
            assert_eq!(
                SystemConfiguration::parse_slot(&slot),
                Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized {
                    incompat_bitmap: IncompatBitmap(expected_bitmap),
                }),
                "{what}：读者拒成这一槽不可择，报布局不认识"
            );
        }
        let new_layout_only = slot_with_first_incompat_byte(
            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT,
        );
        assert_eq!(
            SystemConfiguration::parse_slot(&new_layout_only),
            Ok(sample()),
            "只带位 1 的收"
        );
    }

    /// 系统配置里的回退下界 F（D22（单元原子性怎么合成） 已定项 9）：写在 [481, 489)、小端，读者从同一处读回来；
    /// 取一个每个字节都不同的值，写错位置、读错位置、读成 0 都对不上。
    #[test]
    fn the_rollback_floor_is_written_little_endian_at_481_and_read_back_from_the_slot() {
        let mut with_floor = sample();
        with_floor.quantities.rollback_floor = CheckpointTxg(0x0807_0605_0403_0201);
        let slot = with_floor.to_slot();
        assert_eq!(
            slot[481..489],
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            "F 住字段表末尾 [481, 489)、小端"
        );
        assert_eq!(
            u32::from_le_bytes(slot[477..481].try_into().expect("4 字节")),
            0,
            "F 之前那 4 字节还是 journal 实例代号，没被 F 盖掉"
        );
        let parsed = SystemConfiguration::parse_slot(&slot).expect("自证得过");
        assert_eq!(
            parsed.quantities.rollback_floor,
            CheckpointTxg(0x0807_0605_0403_0201),
            "读者交回的 F 就是槽里那 8 字节"
        );
        assert_eq!(parsed, with_floor, "整份来回一趟不变");
    }

    /// 把槽里那一字节的 S 改成区间外的值（校验和重算，三关照样过），读者必须报得出拒的是什么：
    /// 不是「这一槽不可择」，而是点名的越界成员，带着盘上读到的值与两条边。
    #[test]
    fn self_describing_slot_declaring_an_out_of_range_slots_per_region_is_refused_by_name() {
        for (declared, expected) in [(3u64, 3u64), (17, 17), (0, 0), (255, 255)] {
            let mut slot = sample().to_slot();
            slot[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")] =
                u8::try_from(declared).expect("采样点都装得进一字节");
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes(),
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            assert!(
                wide_checksum_field_holds(&slot, 4096, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET),
                "这一槽自证得过，拒它的只能是区间判"
            );
            assert_eq!(
                SystemConfiguration::parse_slot(&slot),
                Err(
                    SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(
                        RootRingSlotsPerRegionOutOfRange {
                            declared_slots_per_region: expected,
                            minimum: 4,
                            maximum: 16,
                        }
                    )
                ),
                "S = {declared} 越界"
            );
        }
        for declared in [4u8, 8, 16] {
            let mut slot = sample().to_slot();
            slot[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")] = declared;
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes(),
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            let parsed = SystemConfiguration::parse_slot(&slot).expect("区间里的值收");
            assert_eq!(
                parsed.immutable.sizes.root_ring_slots_per_region.count(),
                u64::from(declared),
                "读回来的 S 就是盘上那一字节，不是编译期常量"
            );
        }
    }

    #[test]
    fn slot_spacing_rounds_4096_up_to_a_multiple_of_minimum_input_output_size() {
        assert_eq!(SystemImmutableSizes::slot_spacing_for(512), 4096);
        assert_eq!(SystemImmutableSizes::slot_spacing_for(4096), 4096);
        assert_eq!(
            SystemImmutableSizes::slot_spacing_for(3072),
            6144,
            "io_min = 3072 时槽距 6144，两槽不共享映射单元"
        );
        assert_eq!(SystemImmutableSizes::slot_spacing_for(65536), 65536);
    }

    /// D22（单元原子性怎么合成） 已定项 26 的四类字节预算：389 + 4 + 36 + 60 = 489（已定项 9 的分段表），
    /// 与字段表的合计（格式常量 `SYSTEM_CONFIGURATION_BYTES`）逐项对上。
    /// 每一档的数另一头由 [`MutabilityTaggedSlotWriter::finish`] 按 `to_slot` 真写出的字节数钉着。
    #[test]
    fn the_four_mutability_classes_budget_389_4_36_60_and_add_up_to_the_field_table_total() {
        assert_eq!(
            SystemImmutableConfiguration::FIELD_TABLE_BYTES,
            389,
            "系统不可变配置：自举头 147 + 加密预留 114 + 几何除节点大小 128"
        );
        assert_eq!(
            SystemMutableConfiguration::FIELD_TABLE_BYTES,
            4,
            "系统可变配置：几何段的节点大小一行"
        );
        assert_eq!(
            SystemRuntimeConfiguration::FIELD_TABLE_BYTES,
            36,
            "系统运行配置：T_time 4 + T_dirty 8 + 整理三水位 24"
        );
        assert_eq!(
            SystemRuntimeQuantities::FIELD_TABLE_BYTES,
            60,
            "系统运行量：槽世代号 8 + 整槽校验和 32 + journal tail 8 + 实例代号 4 + 回退下界 F 8"
        );
        assert_eq!(
            SystemImmutableConfiguration::FIELD_TABLE_BYTES
                + SystemMutableConfiguration::FIELD_TABLE_BYTES
                + SystemRuntimeConfiguration::FIELD_TABLE_BYTES
                + SystemRuntimeQuantities::FIELD_TABLE_BYTES,
            SYSTEM_CONFIGURATION_BYTES,
            "四类合计 = 字段表 46 行的 489 字节"
        );
    }

    /// 四类在槽里是交错的：`to_slot` 一路把每一段记在它那一档上。这一条核的是**真写出去的字节**
    /// 落在哪一档——上一条只核四个常量之间的算术，分错档它一个字也不说。
    #[test]
    fn to_slot_writes_exactly_the_budgeted_bytes_into_each_mutability_class() {
        let (bytes, accounting) = sample().to_slot_with_mutability_accounting();
        assert_eq!(bytes.len(), 4096);
        assert_eq!(
            accounting,
            SlotBytesByMutability {
                immutable_configuration_bytes: 389,
                mutable_configuration_bytes: 4,
                runtime_configuration_bytes: 36,
                runtime_quantity_bytes: 60,
            },
            "字段分档改了、或者某一段写漏写多，这一条先红"
        );
    }
}
