//! 坏盘输入（里程碑「第二个事务」增补 3 第 5 件）：拿第 1 件生成的合法镜像按种子坏掉，喂给恢复、可写挂载与池级 checker。
//! 三者都不许 panic；恢复读回的内容要么是理想模型提交过的某一版，要么报错——**不许读回一版从没提交过的内容**。
//!
//! 坏法分两族，都由 [`DamageKind`] 逐个写明（封闭集合，`match` 不写通配臂）：
//! * **盲坏法**（里程碑那三样）：按种子翻位、清零一个扇区、截掉尾部。盘上每一层都有校验和罩着，盲坏法几乎全被校验和挡下，
//!   挡下就是「报错」这一支，合法；它们罩的是「读者在拒绝之前有没有先越界」。
//! * **指着 panic 面普查的坏法**：`records/2026-09-22-panic面普查-走得到的那些.md` 列了 21 处走得到的 panic，分三族
//!   （盘上自述的条目宽度不判下界就按固定偏移切、盘上读来的结构值不判就喂进分配器、不判就喂进写侧断言）。
//!   随机翻位打不到它们——翻一位就把校验和打穿，读者在解析之前先拒了。要打到那几处，改完字段必须把这条引用链上的每一道校验和
//!   照新内容重算（[`seal_and_write_back_the_chain`]），普查里「两道校验和照新内容重算」说的就是这件事。
//!
//! 与第 3 件（崩溃注入）、第 4 件（故障注入）同一条骨架：种子区间切片、`std::thread::scope` 并行、计数按片次序相加、
//! 报告与线程数无关。与崩溃注入的差别只在**坏在哪**：崩溃注入扣下的是「还没落盘的写」，盘上剩下的每一字节都是实现自己写出来的；
//! 这里改的是已经落盘的字节，写出它的不再是实现。
//!
//! **今天撞得到的 panic 登记在 [`KNOWN_PANIC_SITES`]**：清单里的照记不停（每条指到普查里的哪一条、欠着谁、
//! 快档里该撞几次），清单外的算新发现、判红；次数涨了也判红。增补 3 那条验收「三个读者都不许 panic」
//! **今天不成立**，这张清单就是它的登记位，清单空掉那天那条验收才算满足。清单只许缩，加一行要主 agent 点头。
//!
//! cargo-fuzz（libFuzzer）那一路这一轮没做（里程碑自己标着「两路怎么分是预想」），这里只有种子驱动的普通用例。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::check_system_configuration_slot;
use singlefs_checker::crc32_castagnoli_table;
use singlefs_checker::image::{
    chosen_system_configurations, parse_data_pointer, parse_node_pointer, root_slot_positions,
    valid_roots, ImageReader, InvariantVerdict, PoolGeometry,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, DATA_UNIT_PAYLOAD_OFFSET, EXTENT_LEAF_RECORD_BYTES,
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, NODE_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    SLOT_BYTES, TREE_TABLE_ENTRY_BYTES, UNIT_AREA_START_SLOT, WIDE_CHECKSUM_BYTES,
};

use crate::crash::{MemoryPool, SparseBlockDevice, SparseDevice, SECTOR_BYTES};
use crate::crash_injection::{seed_slices, CrashInjectionWorkerThreads};
use crate::history::{
    execute_history_with, generate_history_with_weights, with_panic_capture, CapturedPanic,
    GenerationWeights, HistoryDeviceWidth, HistoryExecution, HistorySeed, SeededRandomSource,
};
use crate::model::ObservedReadBack;
use crate::model_comparison::observed_read_back;
use crate::SharedStream;

/// 坏盘输入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
pub const BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS";

/// 工作线程数怎么来的：判定与崩溃注入共用同一份实现（只换环境变量名），所以也共用同一个类型——
/// 抄一份出来两边会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
pub type BadDiskInputWorkerThreads = CrashInjectionWorkerThreads;

/// 坏镜像的随机源与生成历史、摆崩溃点那两路岔开：同一个种子，历史怎么生成不受它影响，反过来也一样。
const BAD_DISK_SEED_SALT: u64 = 0x62_61_64_64_69_73_6b_00;

/// 一次「翻位」坏法翻几位。
const BITS_FLIPPED_BY_ONE_DAMAGE: u64 = 4;

/// 一个字节有几位（翻位坏法抽位号用）。
const BITS_IN_A_BYTE: u64 = 8;

/// 单元与索引节点共同前缀里头校验和字段的偏移（D18 已定项 7 的字段表：magic 4 + 格式版本 2 + 类标签 1 + flags 1 + 声明长度 2）。
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;

/// 共同前缀里「声明长度」那个 u16 的偏移。
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;

/// 码 2 索引节点头里 key 宽那一字节的偏移（D18 已定项 18，2026-09-14 用户定案）。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;

/// 码 1 数据单元头里载荷 CRC 那 4 字节的偏移（D18 已定项 18 的字段表）。
const DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET: usize = 101;

/// 码 2 / 码 3 指针（86 字节）里两条位置条目的偏移，以及条目里整单元校验和那 4 字节在条目内的偏移
/// （D19 已定项 4 / 已定项 11：头部 50 + 位置条目 14 × 2；位置条目 = 设备 4 + 槽号 6 + 整单元校验和 4）。
const POINTER_FIRST_LOCATION_OFFSET: usize = 50;
const POINTER_SECOND_LOCATION_OFFSET: usize = 64;
const LOCATION_DEVICE_OFFSET_IN_ENTRY: usize = 0;
const LOCATION_SLOT_OFFSET_IN_ENTRY: usize = 4;
const LOCATION_UNIT_CHECKSUM_OFFSET_IN_ENTRY: usize = 10;

/// 根记录（371 字节）里三条指针的偏移：树表紧跟 magic 4 + fsid 16 + flags 4 + 实例代号 4 + txg 8；
/// 自证校验和在 138，实例表与中央映射树根紧跟在它之后（`singlefs-core` 的 `root_record.rs` 同一张表，这里按字段表另写一份）。
const ROOT_TREE_TABLE_POINTER_OFFSET: usize = 4 + 16 + 4 + 4 + 8;
const ROOT_SELF_CHECKSUM_OFFSET: usize = ROOT_TREE_TABLE_POINTER_OFFSET + 86 + 8 + 8;
const ROOT_INSTANCE_TABLE_POINTER_OFFSET: usize = ROOT_SELF_CHECKSUM_OFFSET + 32;
/// 中央映射树的根住根记录（D19 已定项 11），紧跟实例表指针。
const ROOT_MAPPING_POINTER_OFFSET: usize = ROOT_INSTANCE_TABLE_POINTER_OFFSET + 86;

/// 树表条目（200 字节）里根指针那 86 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（D8 已定项 8）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
/// 树表条目里「树的种类」那个 u16 的偏移。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;

/// 树的种类码（D8 已定项 9 / 已定项 11；checker 的 `key_schema_for_tree_kind` 同一张表）。
const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION_RECORDS: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;

/// 分配记录条目（20 字节）里各段的偏移：设备 4 + 槽号 6 + 跨度 2 + 代 8（D3 已定项 7 / 已定项 11）。
const ALLOCATION_RECORD_DEVICE_OFFSET: usize = 0;
const ALLOCATION_RECORD_SLOT_OFFSET: usize = 4;
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;

/// 记账条目（34 字节）里统计量标签那个 u16 的偏移（D5 已定项 5）。
const ACCOUNTING_ENTRY_STATISTIC_OFFSET: usize = 0;
/// inode 号水位那一行的统计量标签（`singlefs-core` 的 `STATISTIC_INODE_WATERMARK`；这里按登记表另写一份）。
const STATISTIC_INODE_WATERMARK_LABEL: u16 = 12;
/// 把水位那一行改挂到这个标签上：登记表里没有这个号，读路径因此找不到水位那一行（普查 R11）。
const STATISTIC_LABEL_NOT_IN_THE_REGISTRY: u16 = 60_000;

/// 「分配记录的槽号落在单元区之外」那一条坏法把槽号改成这个数（远在 `UNIT_AREA_START_SLOT` 之下，普查 R6）。
const SLOT_FAR_BELOW_THE_UNIT_AREA: u64 = 1024;
/// 分配记录里跨度那个 u16 的最高位是「已释放」标志（`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`），
/// 低 15 位才是跨度。「跨度越过单元区末尾」那一条坏法把整个字段写成这个数：跨度取低 15 位的最大值、已释放位清掉
/// （还着的记录才会被重建拿去 `mark_allocated`，普查 R6 / R7）。
const SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED: u16 = 0x7FFF;
/// 跨度那个 u16 里的「已释放」位（同上，`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`）。
const SPAN_FIELD_RELEASED_BIT: u16 = 0x8000;
/// 「只在第二块盘上写成已释放」那一条坏法把释放代改成这个数（普查 R10）：回收判定是「释放代 ≤ max(F_生效, 环里最旧有效根)」，
/// 写一个高过任何根的数，这条记录在挂载重建时不会被回收掉——回收会把位图上那一格清掉，两块盘的空闲图就不一样了，
/// 落点政策会在释放判定之前先拒，坏法打不到要打的那一处。
const RELEASE_GENERATION_ABOVE_EVERY_ROOT: u64 = u64::MAX;
/// 「分配记录的设备身份不在池里」那一条坏法把设备身份改成这个号（池里只有 0 与 1，普查 R9）。
const DEVICE_IDENTITY_OUTSIDE_THE_POOL: u32 = 7;
/// 「系统配置里的区域数越过那个长 3 的数组」那一条坏法把区域数改成这个数（普查 R12）。
const REGION_COUNT_PAST_THE_THREE_REGION_ARRAY: u8 = 4;
/// 系统配置槽里区域数那一字节的偏移（checker 的 `geometry_of` 同一张字段表）。
const SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET: usize = 361;
/// 系统配置槽里自证校验和字段的偏移。
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
/// 系统配置槽宽（格式常量 4096）。
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
/// 「两条位置条目的槽号不等」那一条坏法把第二条的槽号加上这个数（普查 R5）。
const SLOT_OFFSET_THAT_BREAKS_TWO_DEVICES_ON_ONE_SLOT: u64 = 1;
/// 重写数据单元载荷时填的那个字节：与内容生成器写得出的任何一版都不同（`scenario::first_content_byte` 那一族是按长度与种子取的，
/// 这个值只在坏盘输入里出现），读回它就是「读回了一版从没提交过的内容」。
const BYTE_THAT_NO_COMMITTED_VERSION_CONTAINS: u8 = 0xA5;

/// 一个坏法。封闭集合：加一种坏法，每个 `match` 都要补一臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DamageKind {
    /// 盲坏法一：在写过的扇区里按种子翻 [`BITS_FLIPPED_BY_ONE_DAMAGE`] 位。
    FlippedBitsInWrittenSectors,
    /// 盲坏法二：按种子挑一个写过的扇区整个清零。
    ZeroedOneWrittenSector,
    /// 盲坏法三：按种子挑一个切点，两块盘上切点之后写过的扇区全部丢掉（截掉尾部）。
    DroppedEverySectorPastACut,
    /// 条目宽度那一族（普查 R1、R13）：把 extent 树根自述的条目宽改成刚好等于 key 宽（24）、条目数 1、声明长度跟着改，
    /// 链上每一道校验和重算。读者只判了「条目宽 ≥ key 宽」，之后按字段表的固定偏移切，切出界就 panic。
    NarrowedEntryWidthOfTheExtentTreeRoot,
    /// 条目宽度那一族（普查 R3、R13）：同上，改 inode 树根（key 宽 8，内部条目该有 120）。
    NarrowedEntryWidthOfTheInodeTreeRoot,
    /// 条目宽度那一族（普查 R4）：同上，改分配记录树根（key 宽 10，记录该有 20）。
    NarrowedEntryWidthOfTheAllocationRecordsTreeRoot,
    /// 条目宽度那一族（普查 R4、R13）：同上，改记账树根（key 宽 22，条目该有 34）。
    NarrowedEntryWidthOfTheAccountingTreeRoot,
    /// 条目宽度那一族（普查 R2、R13）：同上，改中央映射树根（key 宽 27，条目该有 55）。映射树的根住根记录、不经树表
    /// （D19（块指针的结构与宽度预算） 已定项 11），所以这一条重算的是三道校验和，不是五道。
    NarrowedEntryWidthOfTheCentralMappingTreeRoot,
    /// 条目宽度那一族（C504（树表条目宽在走读里无守卫，今天没坏法打得到））：改**树表单元自己**（key 宽 8，条目该有 200）。
    /// 上面五条坏的都是树表指着的某棵树的根，链上第一道就判了「树表的条目宽是 200」，一条都打不到树表自己。
    NarrowedEntryWidthOfTheTreeTableUnit,
    /// 分配器那一族（普查 R6）：把一条分配记录的槽号改到单元区起点之下。
    AllocationRecordSlotBelowTheUnitArea,
    /// 分配器那一族（普查 R6 / R7）：把一条分配记录的跨度改到越过单元区末尾。
    AllocationRecordSpanPastTheEndOfTheUnitArea,
    /// 分配器那一族（普查 R8）：把第一条分配记录的 key 换成第二条的，两条罩住同一个槽。
    TwoAllocationRecordsCoveringTheSameSlot,
    /// 分配器那一族（普查 R9）：把一条分配记录的设备身份改成池里没有的号。
    AllocationRecordOnADeviceOutsideThePool,
    /// 分配器那一族（普查 R10）：这一版实例表那个落点的分配记录，**只在第二条位置条目那块盘上**改成已释放
    /// （释放代写成高过任何根的一个数，免得挂载时就被回收掉）。释放判定路径此前只核第一条位置条目那块盘，
    /// 于是「两块盘的分配记录树不对称」原样喂进 `PoolAllocator::release` 的断言。
    /// 只动已释放位与释放代：两块盘的位图一个字节都不差（`mark_released` 不清位），
    /// 挂载之后的落点政策两盘照样答同一个槽，坏法不会在别处先被拦下。
    AllocationRecordReleasedOnTheSecondDeviceOnly,
    /// 分配器那一族（普查 R11）：把记账树里 inode 号水位那一行的统计量标签换掉，读路径找不到水位。
    RelabelledInodeWatermarkAccountingRow,
    /// 写侧断言那一族（普查 R12）：把系统配置里的区域数改到那个长 3 的数组之外。
    RegionCountPastTheThreeRegionArray,
    /// 写侧断言那一族（普查 R5）：把根记录里实例表指针的两条位置条目改成不同的槽号。
    TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot,
    /// 判别力那一条：把现行版本的数据单元整个重写成别的内容，**只**重算这个单元自己的两道校验和，
    /// 位置条目里的整单元校验和原样留着。恢复里那一道比对（`recovery.rs` 的 `read_unit_via_locations`）是唯一挡着它的东西：
    /// 去掉它，恢复就会读回这一版从没提交过的内容。
    RewrittenDataUnitResealedWithItsOwnChecksums,
}

/// 每一种坏法，报告与用例按这个次序走。
pub const EVERY_DAMAGE_KIND: [DamageKind; 18] = [
    DamageKind::FlippedBitsInWrittenSectors,
    DamageKind::ZeroedOneWrittenSector,
    DamageKind::DroppedEverySectorPastACut,
    DamageKind::NarrowedEntryWidthOfTheExtentTreeRoot,
    DamageKind::NarrowedEntryWidthOfTheInodeTreeRoot,
    DamageKind::NarrowedEntryWidthOfTheAllocationRecordsTreeRoot,
    DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot,
    DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot,
    DamageKind::NarrowedEntryWidthOfTheTreeTableUnit,
    DamageKind::AllocationRecordSlotBelowTheUnitArea,
    DamageKind::AllocationRecordSpanPastTheEndOfTheUnitArea,
    DamageKind::TwoAllocationRecordsCoveringTheSameSlot,
    DamageKind::AllocationRecordOnADeviceOutsideThePool,
    DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly,
    DamageKind::RelabelledInodeWatermarkAccountingRow,
    DamageKind::RegionCountPastTheThreeRegionArray,
    DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot,
    DamageKind::RewrittenDataUnitResealedWithItsOwnChecksums,
];

/// 这个坏法指着 panic 面普查（`records/2026-09-22-panic面普查-走得到的那些.md`）里的哪一族。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetedPanicFamily {
    /// 盲坏法，不指着哪一族。
    NotAimedAtTheSurvey,
    /// 盘上自述的宽度 / 数量没有下界判，就拿去按字段表的固定偏移切（R1–R4、R13）。
    EntryWidthWithoutALowerBound,
    /// 盘上读来的结构值不经判定就喂进分配器与记账行（R6–R11）。
    StructureValuesFedIntoTheAllocator,
    /// 盘上读来的结构值不经判定就喂进写侧断言与下标（R5、R12）。
    StructureValuesFedIntoAWriteSideAssertion,
    /// 不指着 panic：指着恢复里那一道整单元校验和比对（判别力自证）。
    TheUnitChecksumInTheLocationEntry,
}

impl DamageKind {
    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            DamageKind::FlippedBitsInWrittenSectors => "翻位：写过的扇区里翻 4 位",
            DamageKind::ZeroedOneWrittenSector => "清零：一个写过的扇区整个清零",
            DamageKind::DroppedEverySectorPastACut => "截尾：切点之后写过的扇区全丢掉",
            DamageKind::NarrowedEntryWidthOfTheExtentTreeRoot => {
                "条目宽：extent 树根自述的条目宽缩到 key 宽（链上校验和重算）"
            }
            DamageKind::NarrowedEntryWidthOfTheInodeTreeRoot => {
                "条目宽：inode 树根自述的条目宽缩到 key 宽（链上校验和重算）"
            }
            DamageKind::NarrowedEntryWidthOfTheAllocationRecordsTreeRoot => {
                "条目宽：分配记录树根自述的条目宽缩到 key 宽（链上校验和重算）"
            }
            DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot => {
                "条目宽：记账树根自述的条目宽缩到 key 宽（链上校验和重算）"
            }
            DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot => {
                "条目宽：中央映射树根自述的条目宽缩到 key 宽（根记录那条链上校验和重算）"
            }
            DamageKind::NarrowedEntryWidthOfTheTreeTableUnit => {
                "条目宽：树表单元自己自述的条目宽缩到 key 宽（树表两道 + 根槽自证一道校验和重算）"
            }
            DamageKind::AllocationRecordSlotBelowTheUnitArea => {
                "分配器：一条分配记录的槽号落在单元区之外"
            }
            DamageKind::AllocationRecordSpanPastTheEndOfTheUnitArea => {
                "分配器：一条分配记录的跨度越过单元区末尾"
            }
            DamageKind::TwoAllocationRecordsCoveringTheSameSlot => {
                "分配器：两条分配记录罩住同一个槽"
            }
            DamageKind::AllocationRecordOnADeviceOutsideThePool => {
                "分配器：一条分配记录的设备身份不在池里"
            }
            DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly => {
                "分配器：实例表那个落点的分配记录只在第二块盘上写成已释放（两盘的账不对称）"
            }
            DamageKind::RelabelledInodeWatermarkAccountingRow => {
                "分配器：记账树里 inode 号水位那一行被改挂到别的标签上"
            }
            DamageKind::RegionCountPastTheThreeRegionArray => {
                "写侧断言：系统配置里的区域数越过那个长 3 的数组"
            }
            DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot => {
                "写侧断言：一条指针的两条位置条目槽号不等"
            }
            DamageKind::RewrittenDataUnitResealedWithItsOwnChecksums => {
                "判别力：数据单元整个重写、只重算它自己的两道校验和"
            }
        }
    }

    /// 它指着普查里的哪一族。
    #[must_use]
    pub fn targeted_panic_family(self) -> TargetedPanicFamily {
        match self {
            DamageKind::FlippedBitsInWrittenSectors
            | DamageKind::ZeroedOneWrittenSector
            | DamageKind::DroppedEverySectorPastACut => TargetedPanicFamily::NotAimedAtTheSurvey,
            DamageKind::NarrowedEntryWidthOfTheExtentTreeRoot
            | DamageKind::NarrowedEntryWidthOfTheInodeTreeRoot
            | DamageKind::NarrowedEntryWidthOfTheAllocationRecordsTreeRoot
            | DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot
            | DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot
            | DamageKind::NarrowedEntryWidthOfTheTreeTableUnit => {
                TargetedPanicFamily::EntryWidthWithoutALowerBound
            }
            DamageKind::AllocationRecordSlotBelowTheUnitArea
            | DamageKind::AllocationRecordSpanPastTheEndOfTheUnitArea
            | DamageKind::TwoAllocationRecordsCoveringTheSameSlot
            | DamageKind::AllocationRecordOnADeviceOutsideThePool
            | DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly
            | DamageKind::RelabelledInodeWatermarkAccountingRow => {
                TargetedPanicFamily::StructureValuesFedIntoTheAllocator
            }
            DamageKind::RegionCountPastTheThreeRegionArray
            | DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot => {
                TargetedPanicFamily::StructureValuesFedIntoAWriteSideAssertion
            }
            DamageKind::RewrittenDataUnitResealedWithItsOwnChecksums => {
                TargetedPanicFamily::TheUnitChecksumInTheLocationEntry
            }
        }
    }
}

impl TargetedPanicFamily {
    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            TargetedPanicFamily::NotAimedAtTheSurvey => "盲坏法（不指着普查里的某一族）",
            TargetedPanicFamily::EntryWidthWithoutALowerBound => {
                "普查第一族：条目宽度没有下界判（R1–R4、R13）"
            }
            TargetedPanicFamily::StructureValuesFedIntoTheAllocator => {
                "普查第二族之一：结构值不判就喂进分配器（R6–R11）"
            }
            TargetedPanicFamily::StructureValuesFedIntoAWriteSideAssertion => {
                "普查第二族之二：结构值不判就喂进写侧断言（R5、R12）"
            }
            TargetedPanicFamily::TheUnitChecksumInTheLocationEntry => {
                "不指 panic：指着恢复里那一道整单元校验和比对"
            }
        }
    }
}

/// 今天撞得到的一处 panic：清单里的名字、它是普查里的哪一条、落在哪个文件、消息里认得出它的那一段、欠着谁。
///
/// **按「文件 + 消息片段」认，不按行号认**：`crates/singlefs-core/` 这一轮另有会话在改，行号会漂；
/// 按行号写的清单一漂就悄悄失效，撞到的 panic 会变成「清单外的新发现」，用例红在一个假的发现上。
/// 代价是消息片段罩得比一处宽——同一个文件里另起一处同形的 panic（例如 `bytes.rs` 里新加一处越界）
/// 会被这一条接走。这一件收不掉那个代价：消息里带着的长度每次都不同，比这更窄就只能写行号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownPanicSite {
    /// 清单里的名字：计数按它分格，改法落地之后照这个名字删行。
    pub registry_name: &'static str,
    /// `records/2026-09-22-panic面普查-走得到的那些.md` 里的条目号。
    pub panic_survey_item: &'static str,
    /// panic 位置里必然出现的那一段路径。
    pub file: &'static str,
    /// panic 消息里必然出现的那一段。
    pub message_fragment: &'static str,
    /// 哪一条坏法打到它、为什么它今天还在。
    pub how_it_is_reached: &'static str,
    /// 快档（[`crate::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`] 起 12 段、每段 16 步）里这一条撞到几次。
    /// **次数只许往下走，不许涨**：只按「文件 + 消息片段」认的清单会把同一个文件里同形的新 panic
    /// 静默接走（例如 `transaction.rs` 里另起一处「两盘同槽」的断言），次数涨了就红在这里。
    pub expected_hits_in_the_fast_tier: u64,
}

/// 今天撞得到的 panic。表里每一条都是被测代码的缺口，不是用例写错了；清单外的 panic 算新发现、判红。
///
/// **这张表是增补 3 那条验收「三个读者都不许 panic」（第 5 件「写死的种子数上零 panic」）的登记位：
/// 2026-09-22 第三批改法落地之后它空了 ⇒ 那条验收在这一档（写死的种子基、12 段 × 16 步）上成立，
/// 三个读者一次都没 panic。**表空着不许删掉：它现在判的是「一条都不许回来」——任何一处 panic 都落进
/// 「清单外的新发现」那一格、当场判红。
///
/// 两条规矩（2026-09-22 用户定，`tests/second_transaction_supplement_three_bad_disk_input.rs` 逐条判）：
/// 1. **每条带期望次数，次数涨了判红**（[`KnownPanicSite::expected_hits_in_the_fast_tier`]）。
///    只按「文件 + 消息片段」认的话，同一个文件里同形的新 panic 会被现成的行静默接走；带上次数，
///    多出来的那几次就红在这里。次数只许往下走。
/// 2. **清单只许缩、不许涨**：一条修好了就删行（那条用例会先红在「这一条撞不到了」上，逼着删）。
///    **往里加一行要主 agent 点头**——加行等于把一处新的 panic 登记成「已知、照记不停」，
///    那是把验收的地板往下挪，不归写实现的人自己定。表空了之后这一条更紧：往空表里加第一行，
///    等于把刚满足的那条验收撤回。
///
/// 三批删完的十条，改法是同一句：盘上读来的条目宽、槽号、跨度、设备身份、区域数、统计量标签与位置条目的槽号
/// 在边界上各验一次，返回错误成员或判红一条不变量（`code-discipline.md`「错误」：盘上读来的值不是不变量）。
/// 第三批删的三条是 `transaction.rs` 两处（普查 R11 的 inode 号水位、R5 的释放判定）与 `mount.rs` 一处
/// （R5 的 `format_time_allocator`），各自钉住「拿到的是哪一个错误成员」的用例在
/// `tests/second_transaction_supplement_three_bad_disk_input.rs`。
pub const KNOWN_PANIC_SITES: [KnownPanicSite; 0] = [];

/// 一份坏掉的镜像：坏之前是哪一步的合法镜像、坏成了什么样。
#[derive(Clone, Debug)]
pub struct DamagedImage {
    pub kind: DamageKind,
    /// 逐字写清改了哪块盘的哪个偏移、哪个字段、从什么改成什么——判红时照着它就能手工复现。
    pub what: String,
    pub image: MemoryPool,
}

/// 三个读者各自跑完还是 panic 了。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReaderOutcome {
    /// 跑完了（返回 Ok 或 Err 都算跑完），`how` 是它交回的东西。
    Finished {
        how: String,
    },
    Panicked(CapturedPanic),
}

impl ReaderOutcome {
    #[must_use]
    pub fn panic(&self) -> Option<&CapturedPanic> {
        match self {
            ReaderOutcome::Finished { .. } => None,
            ReaderOutcome::Panicked(captured) => Some(captured),
        }
    }
}

/// 恢复读回的那一版对不对：坏盘输入这一侧的判据比崩溃注入松一格——**报错是合法结局**
/// （盘上的字节已经不是实现写出来的，拒绝读是对的），不合法的只有「读回了一版从没提交过的内容」。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadBackVerdict {
    /// 恢复 panic 了，读回这一格没有判定。
    RecoveryPanicked,
    /// 恢复报了错：合法。
    ReportedAnError { what: String },
    /// 读回的内容与理想模型提交过的某一版逐字节相同：合法。
    ReadBackACommittedVersion,
    /// 读回了一版从没提交过的内容：失败。
    ReadBackAVersionNeverCommitted { what: String },
}

/// 一次「坏掉的镜像喂给三个读者」的观察。
#[derive(Clone, Debug)]
pub struct BadDiskObservation {
    pub kind: DamageKind,
    pub what: String,
    pub recovery: ReaderOutcome,
    pub mount: ReaderOutcome,
    pub checker: ReaderOutcome,
    pub read_back: ReadBackVerdict,
    /// 池级 checker 在这份坏镜像上判红的那几条（checker 判红是对的：盘坏了）。
    pub invariants_violated: Vec<&'static str>,
    /// 池级 checker 一条都没评估出来（每一条都报「不适用」）：这是它读不出这份镜像的几何时的样子
    /// （`walk::check_pool_image` 在没有一块盘交得出有效系统配置时就是这样）。
    /// **这也算「读者看见了」**：checker 拒收整份镜像是一个可观测的结局，与判红同格。
    /// 2026-09-23 加：「系统配置里的区域数越过那个长 3 的数组」那条坏法只有 checker 的几何读者判得出
    /// （核心层一处都不读盘上那个区域数，写与读都按格式常量 3 走），而 checker 读不出几何时把每一条都报成不适用、
    /// 一条都不红——这条坏法此前只在「底片本身已经坏到读者报错」的那几段历史上被偶然算成看见过。
    pub checker_judged_nothing: bool,
}

impl BadDiskObservation {
    /// 三个读者里 panic 了的那几个（读者名 + 位置 + 消息）。
    #[must_use]
    pub fn panics(&self) -> Vec<(&'static str, CapturedPanic)> {
        [
            ("恢复", &self.recovery),
            ("可写挂载", &self.mount),
            ("池级 checker", &self.checker),
        ]
        .into_iter()
        .filter_map(|(reader, outcome)| {
            outcome
                .panic()
                .map(|captured| (reader, (*captured).clone()))
        })
        .collect()
    }

    /// 这份坏镜像被读者看见了没有：三个读者里但凡有一个 panic、判红、拒收整份镜像（每一条都报不适用）、
    /// 或者恢复报了错，就算看见了。全都若无其事地跑完、checker 一条不红也没拒收，说明这个坏法在这个盘面上什么也没碰到。
    #[must_use]
    pub fn was_seen_by_a_reader(&self) -> bool {
        !self.panics().is_empty()
            || !self.invariants_violated.is_empty()
            || self.checker_judged_nothing
            || matches!(
                self.read_back,
                ReadBackVerdict::ReportedAnError { .. }
                    | ReadBackVerdict::ReadBackAVersionNeverCommitted { .. }
            )
            || matches!(&self.mount, ReaderOutcome::Finished { how } if how.starts_with("Err"))
    }
}

/// 一条新发现：清单外的 panic，或者读回了一版从没提交过的内容。
#[derive(Clone, Debug)]
pub struct BadDiskFinding {
    pub seed: HistorySeed,
    /// 坏的是哪一档的基线镜像。
    pub tier: BaseImageTier,
    pub kind: DamageKind,
    pub what: String,
    /// 一句话说清是哪一类失败。
    pub headline: String,
    pub observation_text: String,
}

impl BadDiskFinding {
    /// 认人用的签名：同一个签名只留第一个种子（报告与线程数无关）。
    /// 消息里的数字（槽号、长度、行号）抹掉再认——同一处 panic 每次带的数都不同，不抹就成了「一次一条新发现」。
    #[must_use]
    pub fn signature(&self) -> String {
        let without_numbers: String = self
            .headline
            .chars()
            .filter(|character| !character.is_ascii_digit())
            .collect();
        format!("{:?}｜{without_numbers}", self.kind)
    }

    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "种子 {} 基线档「{}」坏法「{}」：{}\n    坏在哪：{}\n    观察：{}",
            self.seed.0,
            self.tier.name(),
            self.kind.name(),
            self.headline,
            self.what,
            self.observation_text
        )
    }
}

/// 跑过的绝对数：证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BadDiskTally {
    pub histories: u64,
    /// 历史跑完之后一份可用的合法镜像都没交出来的段数（起点就失败）。
    pub histories_without_a_base_image: u64,
    /// 各基线档（[`BaseImageTier::name`]）抽到基线镜像的段数：一段历史每档至多一份。
    pub base_images_by_tier: BTreeMap<&'static str, u64>,
    /// 各基线档的基线镜像上造出的坏镜像数。
    pub damaged_images_by_tier: BTreeMap<&'static str, u64>,
    /// 各坏法各造出几份坏镜像。
    pub damaged_images_by_kind: BTreeMap<&'static str, u64>,
    /// 各坏法在这个盘面上没有可坏的对象、照实记「不适用」的次数。
    pub damages_not_applicable_by_kind: BTreeMap<&'static str, u64>,
    /// 各坏法造出的坏镜像里被读者看见了的次数（[`BadDiskObservation::was_seen_by_a_reader`]）。
    pub damages_seen_by_a_reader_by_kind: BTreeMap<&'static str, u64>,
    /// 三个读者各跑了几次。
    pub recoveries: u64,
    pub mounts: u64,
    pub checker_runs: u64,
    /// 恢复的三种结局。
    pub recoveries_reading_a_file: u64,
    pub recoveries_without_a_file: u64,
    pub recoveries_failed: u64,
    /// 可写挂载的两种结局。
    pub mounts_succeeded: u64,
    pub mounts_refused: u64,
    /// 读回判定的四格。
    pub read_back_committed_versions: u64,
    pub read_back_errors: u64,
    pub read_back_versions_never_committed: u64,
    /// 撞到 [`KNOWN_PANIC_SITES`] 里某一条的次数，按清单里的名字分。
    pub known_panics_by_registry_name: BTreeMap<&'static str, u64>,
    /// 清单外的 panic 次数。
    pub panics_outside_the_known_list: u64,
    /// 池级 checker 在坏镜像上判红的次数，按不变量分。
    pub invariant_violations: BTreeMap<&'static str, u64>,
}

impl BadDiskTally {
    /// 并进另一份（按片的次序相加，结果与线程数无关）。
    pub fn absorb(&mut self, following: &BadDiskTally) {
        self.histories += following.histories;
        self.histories_without_a_base_image += following.histories_without_a_base_image;
        absorb_counts(
            &mut self.base_images_by_tier,
            &following.base_images_by_tier,
        );
        absorb_counts(
            &mut self.damaged_images_by_tier,
            &following.damaged_images_by_tier,
        );
        absorb_counts(
            &mut self.damaged_images_by_kind,
            &following.damaged_images_by_kind,
        );
        absorb_counts(
            &mut self.damages_not_applicable_by_kind,
            &following.damages_not_applicable_by_kind,
        );
        absorb_counts(
            &mut self.damages_seen_by_a_reader_by_kind,
            &following.damages_seen_by_a_reader_by_kind,
        );
        self.recoveries += following.recoveries;
        self.mounts += following.mounts;
        self.checker_runs += following.checker_runs;
        self.recoveries_reading_a_file += following.recoveries_reading_a_file;
        self.recoveries_without_a_file += following.recoveries_without_a_file;
        self.recoveries_failed += following.recoveries_failed;
        self.mounts_succeeded += following.mounts_succeeded;
        self.mounts_refused += following.mounts_refused;
        self.read_back_committed_versions += following.read_back_committed_versions;
        self.read_back_errors += following.read_back_errors;
        self.read_back_versions_never_committed += following.read_back_versions_never_committed;
        absorb_counts(
            &mut self.known_panics_by_registry_name,
            &following.known_panics_by_registry_name,
        );
        self.panics_outside_the_known_list += following.panics_outside_the_known_list;
        absorb_counts(
            &mut self.invariant_violations,
            &following.invariant_violations,
        );
    }

    /// 抽到过基线镜像的档数：[`EVERY_BASE_IMAGE_TIER`] 里至少一段历史抽到了一份的那几档。
    #[must_use]
    pub fn base_image_tiers_sampled(&self) -> usize {
        EVERY_BASE_IMAGE_TIER
            .into_iter()
            .filter(|tier| {
                self.base_images_by_tier
                    .get(tier.name())
                    .copied()
                    .unwrap_or(0)
                    > 0
            })
            .count()
    }

    /// 报告正文。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "  历史 {} 段（起点就失败、交不出合法镜像的 {} 段）",
            self.histories, self.histories_without_a_base_image
        );
        let per_tier: Vec<String> = EVERY_BASE_IMAGE_TIER
            .into_iter()
            .map(|tier| {
                format!(
                    "「{}」{} 段、坏镜像 {} 份",
                    tier.name(),
                    self.base_images_by_tier
                        .get(tier.name())
                        .copied()
                        .unwrap_or(0),
                    self.damaged_images_by_tier
                        .get(tier.name())
                        .copied()
                        .unwrap_or(0)
                )
            })
            .collect();
        let _ = writeln!(
            text,
            "  基线档 {} / {} 档抽到过镜像：{}",
            self.base_image_tiers_sampled(),
            EVERY_BASE_IMAGE_TIER.len(),
            per_tier.join("；")
        );
        let _ = writeln!(
            text,
            "  三个读者：恢复 {} 次（读回文件 {}、没有文件 {}、报错 {}）、可写挂载 {} 次（成 {}、拒 {}）、池级 checker {} 次",
            self.recoveries,
            self.recoveries_reading_a_file,
            self.recoveries_without_a_file,
            self.recoveries_failed,
            self.mounts,
            self.mounts_succeeded,
            self.mounts_refused,
            self.checker_runs
        );
        let _ = writeln!(
            text,
            "  读回判定：模型提交过的某一版 {}、报错 {}、**从没提交过的内容 {}**",
            self.read_back_committed_versions,
            self.read_back_errors,
            self.read_back_versions_never_committed
        );
        let _ = writeln!(text, "  逐坏法（造出 / 不适用 / 被读者看见）：");
        for kind in EVERY_DAMAGE_KIND {
            let _ = writeln!(
                text,
                "    {} | {} / {} / {} | {}",
                kind.name(),
                self.damaged_images_by_kind
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0),
                self.damages_not_applicable_by_kind
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0),
                self.damages_seen_by_a_reader_by_kind
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0),
                kind.targeted_panic_family().name()
            );
        }
        let _ = writeln!(
            text,
            "  清单里的 panic（KNOWN_PANIC_SITES，今天还没修的缺口）：{:?}；清单外的 panic {} 次",
            self.known_panics_by_registry_name, self.panics_outside_the_known_list
        );
        let _ = writeln!(
            text,
            "  池级 checker 在坏镜像上判红：{:?}",
            self.invariant_violations
        );
        text
    }
}

fn absorb_counts(into: &mut BTreeMap<&'static str, u64>, from: &BTreeMap<&'static str, u64>) {
    for (key, count) in from {
        *into.entry(key).or_insert(0) += count;
    }
}

/// 一段历史坏下来的东西。
#[derive(Clone, Debug)]
pub struct HistoryBadDiskInput {
    pub seed: HistorySeed,
    pub tally: BadDiskTally,
    pub findings: Vec<BadDiskFinding>,
}

impl HistoryBadDiskInput {
    /// 这一段在按种子次序的摘要里贡献的那一行。
    #[must_use]
    fn digest_line(&self) -> String {
        format!(
            "{} {} {} {} {} {}\n",
            self.seed.0,
            self.tally.damaged_images_by_kind.values().sum::<u64>(),
            self.tally
                .known_panics_by_registry_name
                .values()
                .sum::<u64>(),
            self.tally.panics_outside_the_known_list,
            self.tally.read_back_versions_never_committed,
            self.findings.len()
        )
    }
}

/// 一次坏盘输入跑什么：种子区间、每段几步、比重、历史怎么跑、线程数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadDiskCampaign {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub worker_threads: BadDiskInputWorkerThreads,
}

/// 一批种子跑下来的坏盘输入报告。
#[derive(Clone, Debug)]
pub struct BadDiskReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: BadDiskTally,
    pub findings: Vec<BadDiskFinding>,
    /// 各段按**种子从小到大**的摘要取的 FNV-1a 64 位哈希。计数是按 `BTreeMap` 相加的，次序一乱它也不变；
    /// 这个摘要变，所以「并起来的次序与线程数无关」那一条有东西钉得住（用例 `the_report_is_the_same_with_one_worker_thread_and_four`）。
    pub in_order_digest: u64,
}

impl BadDiskReport {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "坏盘输入：种子 [{}, {}) × 每段 {} 步；比重「{}」；{}\n  按种子次序的摘要 {:016x}\n",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.weights.name,
            self.execution.name(),
            self.in_order_digest
        );
        text.push_str(&self.tally.render());
        if self.findings.is_empty() {
            text.push_str("  新发现：0\n");
        } else {
            let _ = writeln!(text, "  新发现 {} 条：", self.findings.len());
            for finding in &self.findings {
                let _ = writeln!(text, "    {}", finding.render());
            }
        }
        text
    }
}

/// 一条 panic 是不是 [`KNOWN_PANIC_SITES`] 里的那一条（按文件与消息片段认，不按行号认）。
#[must_use]
pub fn known_panic_site_of(captured: &CapturedPanic) -> Option<KnownPanicSite> {
    known_panic_site_among(&KNOWN_PANIC_SITES, captured)
}

/// 同上，清单由调用方给。[`KNOWN_PANIC_SITES`] 空了之后「按文件加消息片段认、不按行号认」这条规矩在真清单上
/// 没有对象可认了（空清单怎么认都交回 `None`），用例因此拿一份样本清单喂这一处，那条规矩才还有东西守着。
#[must_use]
pub fn known_panic_site_among(
    sites: &[KnownPanicSite],
    captured: &CapturedPanic,
) -> Option<KnownPanicSite> {
    sites.iter().copied().find(|site| {
        captured.location.contains(site.file) && captured.message.contains(site.message_fragment)
    })
}

/// 一份镜像上「最新那条根下面该有的那几棵树」在不在：指着普查那几族的坏法要坏的就是它们。
/// 树表 0 条的盘面（只做过 mkfs、回退到第 0 代根、零单元发布那几版）上一棵都没有，每一种指着普查的坏法都只会记「不适用」。
#[must_use]
pub fn newest_root_has_the_trees_the_targeted_damages_need(image: &MemoryPool) -> bool {
    let Some(reached) = reach_the_tree_table(image) else {
        return false;
    };
    // extent / 分配记录 / 记账三棵要的是「非空的叶」：按记录改的那几条坏法只坏叶（那几棵树内部节点的条目格式
    // 还没有条款）。inode 树只要非空：条目宽那一条对内部节点与叶都做得出来。
    let non_empty_leaf = [
        TREE_KIND_EXTENT,
        TREE_KIND_ALLOCATION_RECORDS,
        TREE_KIND_ACCOUNTING,
    ]
    .into_iter()
    .all(|tree_kind| {
        chain_from_the_tree_table(image, &reached, tree_kind).is_some_and(|chain| {
            index_node_is_a_leaf(&chain.tree_root_node)
                && IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0
        })
    });
    // 中央映射树的根住根记录、不经树表（D19 已定项 11）：条目宽那一条要它非空。
    let central_mapping_is_non_empty = open_chain_to_the_central_mapping_tree_root(image)
        .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.mapping_root_node).entry_count > 0);
    non_empty_leaf
        && central_mapping_is_non_empty
        && chain_from_the_tree_table(image, &reached, TREE_KIND_INODE)
            .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0)
}

/// 一份镜像上最新那条根指着的树表是 0 条：只做过 mkfs、第一个文件版本之前的写行与暖机那几版、回退到这样一版的根。
/// C481（坏盘输入的基线镜像取不到树表 0 条的盘面） 要的那一档基线就是它。树表走不到（根择不出、指针全零、条目宽不是 200）不算。
#[must_use]
pub fn newest_root_tree_table_has_no_entries(image: &MemoryPool) -> bool {
    reach_the_tree_table(image)
        .is_some_and(|reached| IndexNodeEntryLayout::of(&reached.tree_table_node).entry_count == 0)
}

/// 一段历史里按哪一档抽基线镜像。每一档各按种子抽一份（蓄水池抽样），抽到的每一份都把全部坏法跑一遍、判据相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaseImageTier {
    /// 最新那条根下面指着普查那几族的坏法要坏的树都在（[`newest_root_has_the_trees_the_targeted_damages_need`]）；
    /// 一段历史里一步都不合格时退回最后那一步的镜像，盲坏法照样跑。
    NewestRootHasTheTreesTheTargetedDamagesNeed,
    /// 最新那条根指着的树表 0 条（[`newest_root_tree_table_has_no_entries`]，C481）。一段历史里一步都没有就这一段不抽，不退回。
    TreeTableWithoutEntries,
}

/// 全部基线档，报告按这个次序逐档报。
pub const EVERY_BASE_IMAGE_TIER: [BaseImageTier; 2] = [
    BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed,
    BaseImageTier::TreeTableWithoutEntries,
];

/// 树表 0 条那一档的蓄水池与坏法随机源另加的盐：与三棵树都在那一档岔开，那一档每个种子抽到的镜像、坏成的样子都与加这一档之前逐字节相同。
const TREE_TABLE_WITHOUT_ENTRIES_TIER_SALT: u64 = 0x74_72_65_65_30_00_00_00;

impl BaseImageTier {
    /// 报告与计数里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => "三棵树都在",
            BaseImageTier::TreeTableWithoutEntries => "树表 0 条",
        }
    }

    /// 这一步的镜像够不够进这一档的蓄水池。
    #[must_use]
    pub fn admits(self, image: &MemoryPool) -> bool {
        match self {
            BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => {
                newest_root_has_the_trees_the_targeted_damages_need(image)
            }
            BaseImageTier::TreeTableWithoutEntries => newest_root_tree_table_has_no_entries(image),
        }
    }

    /// 这一档的随机源在种子上另异或的盐（蓄水池与坏法共用）。
    const fn salt(self) -> u64 {
        match self {
            BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => 0,
            BaseImageTier::TreeTableWithoutEntries => TREE_TABLE_WITHOUT_ENTRIES_TIER_SALT,
        }
    }
}

/// 一档基线的蓄水池：合格的镜像见过几份、手里留的是哪一份。
struct BaseImageReservoir {
    tier: BaseImageTier,
    qualifying_images_seen: u64,
    chosen: Option<MemoryPool>,
    random: SeededRandomSource,
}

/// 跑一段历史，按 [`EVERY_BASE_IMAGE_TIER`] 每一档各抽一份合法镜像，逐个坏法坏一遍，每份坏镜像喂给恢复、可写挂载与池级 checker。
///
/// 每一档的基线镜像取**按种子抽中的那一步**之后的那一份（蓄水池抽样，每档只留一份：一段历史几十步，全留下几十份稀疏镜像太占内存）：
/// - 「三棵树都在」那一档只在最新那条根下面该有的那几棵树都在的那几步里抽（[`newest_root_has_the_trees_the_targeted_damages_need`]）：
///   树表 0 条的盘面上指着普查那几族的坏法一样也做不出来，抽中它这一档就只跑了三个盲坏法。一步都不合格时退回最后那一步的镜像。
/// - 「树表 0 条」那一档只在最新那条根的树表 0 条的那几步里抽（C481（坏盘输入的基线镜像取不到树表 0 条的盘面））：
///   只做过 mkfs 的池、第一个文件版本之前的写行与暖机、回退到这样一版——这些盘面从前一档都没打过。一步都没有就这一段不抽。
///
/// 两档的判据相同（不许读回没提交过的内容、不许 panic），坏法照旧全部跑，做不出来的照实记「不适用」。
#[must_use]
pub fn feed_bad_disk_inputs_from_history(
    seed: HistorySeed,
    operations_per_history: usize,
    weights: &GenerationWeights,
    execution: HistoryExecution,
) -> HistoryBadDiskInput {
    let history = generate_history_with_weights(seed, operations_per_history, weights);
    let mut committed_contents: BTreeSet<Option<Vec<u8>>> = BTreeSet::new();
    let mut last_image: Option<MemoryPool> = None;
    let mut reservoirs: Vec<BaseImageReservoir> = EVERY_BASE_IMAGE_TIER
        .into_iter()
        .map(|tier| BaseImageReservoir {
            tier,
            qualifying_images_seen: 0,
            chosen: None,
            random: SeededRandomSource::from_seed(seed.0 ^ BAD_DISK_SEED_SALT ^ tier.salt()),
        })
        .collect();
    execute_history_with(
        &history,
        execution,
        &SharedStream::new(),
        &mut |observation| {
            for (_, file) in observation.model.committed_versions() {
                committed_contents.insert(file.map(|content| content.to_vec()));
            }
            last_image = Some(observation.image.clone());
            for reservoir in &mut reservoirs {
                if !reservoir.tier.admits(observation.image) {
                    continue;
                }
                reservoir.qualifying_images_seen += 1;
                // 蓄水池抽样：第 n 份合格镜像以 1/n 的概率换下手里那一份，跑完手里就是合格那几步里均匀抽的一份。
                if reservoir.random.below(reservoir.qualifying_images_seen) == 0 {
                    reservoir.chosen = Some(observation.image.clone());
                }
            }
        },
    );

    let mut tally = BadDiskTally {
        histories: 1,
        ..BadDiskTally::default()
    };
    if last_image.is_none() {
        tally.histories_without_a_base_image = 1;
        return HistoryBadDiskInput {
            seed,
            tally,
            findings: Vec::new(),
        };
    }
    let base_images: Vec<(BaseImageTier, MemoryPool)> = reservoirs
        .into_iter()
        .filter_map(|reservoir| {
            let chosen = match reservoir.tier {
                BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => {
                    reservoir.chosen.or_else(|| last_image.clone())
                }
                BaseImageTier::TreeTableWithoutEntries => reservoir.chosen,
            };
            chosen.map(|image| (reservoir.tier, image))
        })
        .collect();

    let parameters = execution.device_width.parameters();
    let mut findings = Vec::new();
    for (tier, base_image) in &base_images {
        *tally.base_images_by_tier.entry(tier.name()).or_insert(0) += 1;
        for (kind_index, kind) in EVERY_DAMAGE_KIND.into_iter().enumerate() {
            let mut random = SeededRandomSource::from_seed(
                seed.0
                    .wrapping_mul(u64::try_from(EVERY_DAMAGE_KIND.len()).expect("坏法数"))
                    .wrapping_add(u64::try_from(kind_index).expect("坏法序号"))
                    ^ BAD_DISK_SEED_SALT
                    ^ tier.salt(),
            );
            let Some(damaged) = damage_image(base_image, kind, &mut random) else {
                *tally
                    .damages_not_applicable_by_kind
                    .entry(kind.name())
                    .or_insert(0) += 1;
                continue;
            };
            *tally.damaged_images_by_kind.entry(kind.name()).or_insert(0) += 1;
            *tally.damaged_images_by_tier.entry(tier.name()).or_insert(0) += 1;
            let observation = feed_a_damaged_image(
                &damaged,
                &committed_contents,
                &parameters,
                execution.device_width,
            );
            absorb_observation(&mut tally, &observation);
            if observation.was_seen_by_a_reader() {
                *tally
                    .damages_seen_by_a_reader_by_kind
                    .entry(kind.name())
                    .or_insert(0) += 1;
            }
            findings.extend(findings_in(seed, *tier, &observation));
        }
    }
    HistoryBadDiskInput {
        seed,
        tally,
        findings,
    }
}

fn absorb_observation(tally: &mut BadDiskTally, observation: &BadDiskObservation) {
    tally.recoveries += 1;
    tally.mounts += 1;
    tally.checker_runs += 1;
    match &observation.recovery {
        ReaderOutcome::Panicked(_) => {}
        ReaderOutcome::Finished { how } => {
            if how.starts_with("FileRead") {
                tally.recoveries_reading_a_file += 1;
            } else if how.starts_with("NoFile") {
                tally.recoveries_without_a_file += 1;
            } else {
                tally.recoveries_failed += 1;
            }
        }
    }
    match &observation.mount {
        ReaderOutcome::Panicked(_) => {}
        ReaderOutcome::Finished { how } => {
            if how.starts_with("Ok") {
                tally.mounts_succeeded += 1;
            } else {
                tally.mounts_refused += 1;
            }
        }
    }
    match &observation.read_back {
        ReadBackVerdict::RecoveryPanicked => {}
        ReadBackVerdict::ReportedAnError { .. } => tally.read_back_errors += 1,
        ReadBackVerdict::ReadBackACommittedVersion => tally.read_back_committed_versions += 1,
        ReadBackVerdict::ReadBackAVersionNeverCommitted { .. } => {
            tally.read_back_versions_never_committed += 1;
        }
    }
    for invariant in &observation.invariants_violated {
        *tally.invariant_violations.entry(invariant).or_insert(0) += 1;
    }
    for (_, captured) in observation.panics() {
        match known_panic_site_of(&captured) {
            Some(site) => {
                *tally
                    .known_panics_by_registry_name
                    .entry(site.registry_name)
                    .or_insert(0) += 1;
            }
            None => tally.panics_outside_the_known_list += 1,
        }
    }
}

fn findings_in(
    seed: HistorySeed,
    tier: BaseImageTier,
    observation: &BadDiskObservation,
) -> Vec<BadDiskFinding> {
    let mut findings = Vec::new();
    for (reader, captured) in observation.panics() {
        if known_panic_site_of(&captured).is_some() {
            continue;
        }
        findings.push(BadDiskFinding {
            seed,
            tier,
            kind: observation.kind,
            what: observation.what.clone(),
            headline: format!(
                "{reader} panic 在 {}：{}（不在 KNOWN_PANIC_SITES 里）",
                captured.location, captured.message
            ),
            observation_text: render_observation(observation),
        });
    }
    if let ReadBackVerdict::ReadBackAVersionNeverCommitted { what } = &observation.read_back {
        findings.push(BadDiskFinding {
            seed,
            tier,
            kind: observation.kind,
            what: observation.what.clone(),
            headline: format!("恢复读回了一版从没提交过的内容：{what}"),
            observation_text: render_observation(observation),
        });
    }
    findings
}

fn render_observation(observation: &BadDiskObservation) -> String {
    format!(
        "恢复 {:?}；可写挂载 {:?}；checker 判红 {:?}",
        observation.recovery, observation.mount, observation.invariants_violated
    )
}

/// 把一份坏掉的镜像喂给恢复、可写挂载与池级 checker，三者都用 [`with_panic_capture`] 兜着——
/// panic 是被测代码的缺口，不是这里该崩的理由；接住之后照实记下位置与消息。
///
/// 三个读者各读一份自己的镜像拷贝：可写挂载会往盘上写（取号、写行、暖机），不能让它改到别人读的那一份。
#[must_use]
pub fn feed_a_damaged_image(
    damaged: &DamagedImage,
    committed_contents: &BTreeSet<Option<Vec<u8>>>,
    parameters: &MakeFilesystemParameters,
    device_width: HistoryDeviceWidth,
) -> BadDiskObservation {
    let recovery_image = damaged.image.clone();
    let recovery = with_panic_capture(|| recover(&recovery_image, JournalPolicy::Consult));
    let (recovery_outcome, read_back) = match recovery {
        Err(captured) => (
            ReaderOutcome::Panicked(captured),
            ReadBackVerdict::RecoveryPanicked,
        ),
        Ok(report) => {
            let read_back = observed_read_back(&report.outcome);
            let verdict = read_back_verdict(&read_back, committed_contents);
            (
                ReaderOutcome::Finished {
                    how: describe_read_back(&read_back),
                },
                verdict,
            )
        }
    };

    let mount_image = damaged.image.clone();
    let mount = with_panic_capture(|| {
        let mut devices = block_devices_of(&mount_image, device_width);
        match mount_writable(parameters, &mut devices) {
            Ok(mounted) => format!(
                "Ok（实例 {}，施加前缀之后的根 txg {}）",
                mounted.output.instance.0, mounted.output.effective_root.checkpoint_txg.0
            ),
            Err(error) => format!("Err({error:?})"),
        }
    });
    let mount_outcome = match mount {
        Err(captured) => ReaderOutcome::Panicked(captured),
        Ok(how) => ReaderOutcome::Finished { how },
    };

    let checker_image = damaged.image.clone();
    let checker = with_panic_capture(|| check_pool_image(&checker_image));
    let (checker_outcome, invariants_violated, checker_judged_nothing) = match checker {
        Err(captured) => (ReaderOutcome::Panicked(captured), Vec::new(), false),
        Ok(report) => {
            let violated: Vec<&'static str> = report
                .iter()
                .filter_map(|(invariant, verdict)| match verdict {
                    InvariantVerdict::Violated(_) => Some(*invariant),
                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
                })
                .collect();
            let judged_nothing = report.iter().all(|(_, verdict)| match verdict {
                InvariantVerdict::NotApplicable(_) => true,
                InvariantVerdict::Holds | InvariantVerdict::Violated(_) => false,
            });
            (
                ReaderOutcome::Finished {
                    how: format!("判红 {} 条", violated.len()),
                },
                violated,
                judged_nothing,
            )
        }
    };

    BadDiskObservation {
        kind: damaged.kind,
        what: damaged.what.clone(),
        recovery: recovery_outcome,
        mount: mount_outcome,
        checker: checker_outcome,
        read_back,
        invariants_violated,
        checker_judged_nothing,
    }
}

fn describe_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => format!(
            "NoFile（实例 {} 第 {} 代根）",
            root.instance.0, root.checkpoint_txg.0
        ),
        ObservedReadBack::FileRead { root, content } => format!(
            "FileRead（实例 {} 第 {} 代根，{} 字节）",
            root.instance.0,
            root.checkpoint_txg.0,
            content.len()
        ),
        ObservedReadBack::Failed { what } => format!("Failed（{what}）"),
    }
}

/// 坏盘输入这一侧的读回判据：报错合法；读回的内容必须与模型提交过的某一版逐字节相同。
/// **按内容认、不按根的身份认**：坏盘能把根记录里的 txg 与实例代号一起改掉，按身份认会把「内容没错、身份被改」也判成失败，
/// 而里程碑那一句管的是内容（「不许读回一版从没提交过的内容」）。
#[must_use]
pub fn read_back_verdict(
    read_back: &ObservedReadBack,
    committed_contents: &BTreeSet<Option<Vec<u8>>>,
) -> ReadBackVerdict {
    match read_back {
        ObservedReadBack::Failed { what } => {
            ReadBackVerdict::ReportedAnError { what: what.clone() }
        }
        ObservedReadBack::NoFile { .. } => {
            if committed_contents.contains(&None) {
                ReadBackVerdict::ReadBackACommittedVersion
            } else {
                ReadBackVerdict::ReadBackAVersionNeverCommitted {
                    what: "报了没有文件，而模型提交过的每一版都带着文件".to_string(),
                }
            }
        }
        ObservedReadBack::FileRead { content, .. } => {
            if committed_contents.contains(&Some(content.clone())) {
                ReadBackVerdict::ReadBackACommittedVersion
            } else {
                ReadBackVerdict::ReadBackAVersionNeverCommitted {
                    what: format!(
                        "读回 {} 字节（首字节 {:?}、末字节 {:?}），模型提交过的 {} 版里没有这一份",
                        content.len(),
                        content.first(),
                        content.last(),
                        committed_contents.len()
                    ),
                }
            }
        }
    }
}

/// 把一份镜像接成可写挂载要的那一串块设备（每块盘一份内存里的稀疏盘）。
fn block_devices_of(
    image: &MemoryPool,
    device_width: HistoryDeviceWidth,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(device_width.device_bytes(), PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

// ---- 盘上的小端读写：与 checker、实现各写一份（D13 已定项 5，只共享 `singlefs-format` 的常量） ----

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}
fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}
fn read_six_byte_unsigned_at(bytes: &[u8], offset: usize) -> u64 {
    let mut widened = [0u8; 8];
    widened[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(widened)
}
fn write_six_byte_unsigned_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 6].copy_from_slice(&value.to_le_bytes()[..6]);
}

// ---- 校验和重算 ----

/// 把一个 32 字节的宽校验和字段按新内容重算：字段自身按 0 参与，低 4 字节写 CRC-32C、其余 28 字节恒 0
/// （`singlefs-format` 的 `WIDE_CHECKSUM_BYTES` 注、checker 的 `checksum_field_holds` 同一条口径）。
fn reseal_wide_checksum_field(bytes: &mut [u8], cover_end: usize, field_offset: usize) {
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    bytes[field_offset..field_offset + field_width].fill(0);
    let digest = crc32_castagnoli_table(&bytes[..cover_end]).to_le_bytes();
    bytes[field_offset..field_offset + digest.len()].copy_from_slice(&digest);
}

fn index_node_key_width(node: &[u8]) -> usize {
    usize::from(node[INDEX_NODE_KEY_WIDTH_OFFSET])
}

/// 码 2 节点明文头的末尾 = 86 + 2 × key 宽（也是载荷 CRC 罩的那一段的起点）。
fn index_node_plaintext_header_end(node: &[u8]) -> usize {
    usize::try_from(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE).expect("86")
        + 2 * index_node_key_width(node)
}

fn index_node_entry_count_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 4
}
fn index_node_entry_width_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 2
}
fn index_node_payload_checksum_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 10
}
fn index_node_entries_start(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node)
        + usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}

/// 一个码 2 节点改完之后把它自己的两道校验和重算：先载荷 CRC（它住在头里），再头校验和（它罩着载荷 CRC 那 4 字节）。
fn reseal_index_node(node: &mut [u8]) {
    let header_end = index_node_plaintext_header_end(node);
    let payload_checksum_offset = index_node_payload_checksum_offset(node);
    let payload_digest = crc32_castagnoli_table(&node[header_end..]).to_le_bytes();
    node[payload_checksum_offset..payload_checksum_offset + payload_digest.len()]
        .copy_from_slice(&payload_digest);
    reseal_wide_checksum_field(node, header_end, UNIT_HEADER_CHECKSUM_OFFSET);
}

/// 一个码 1 数据单元改完之后把它自己的两道校验和重算。
fn reseal_data_unit(unit: &mut [u8]) {
    let header_end = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let payload_digest = crc32_castagnoli_table(&unit[header_end..]).to_le_bytes();
    unit[DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET
        ..DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET + payload_digest.len()]
        .copy_from_slice(&payload_digest);
    reseal_wide_checksum_field(unit, header_end, UNIT_HEADER_CHECKSUM_OFFSET);
}

/// 把一条指针（86 或 88 字节）两条位置条目里的整单元校验和都改成 `checksum`。
fn write_unit_checksum_into_both_locations(pointer: &mut [u8], checksum: u32) {
    for location_offset in [
        POINTER_FIRST_LOCATION_OFFSET,
        POINTER_SECOND_LOCATION_OFFSET,
    ] {
        let field = location_offset + LOCATION_UNIT_CHECKSUM_OFFSET_IN_ENTRY;
        pointer[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
    }
}

// ---- 从最新那条根走到一棵树的根节点 ----

/// 镜像上从最新那条有效根走到一棵树的根节点的一条链：改完要照 [`seal_and_write_back_the_chain`] 逐环重算校验和写回去。
struct ChainToATreeRoot {
    tree_kind: u16,
    root_device: u32,
    root_offset: u64,
    root_slot: Vec<u8>,
    tree_table_devices: [u32; 2],
    tree_table_slot: u64,
    tree_table_node: Vec<u8>,
    /// 这棵树在树表节点里是第几条。
    tree_table_entry_index: usize,
    tree_root_devices: [u32; 2],
    tree_root_slot: u64,
    tree_root_node: Vec<u8>,
}

fn newest_geometry(image: &MemoryPool) -> Option<PoolGeometry> {
    chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen)
        .map(|(_, geometry)| geometry)
}

/// 镜像上最新那条自证过的根：住哪块盘的哪个偏移、整槽的字节。
#[derive(Clone, Debug)]
struct NewestRootSlot {
    device: u32,
    offset_in_bytes: u64,
    slot_bytes: Vec<u8>,
}

fn newest_root_slot(image: &MemoryPool, geometry: &PoolGeometry) -> Option<NewestRootSlot> {
    let slot_width = usize::try_from(geometry.physical_block_size).ok()?;
    let (region, slot, _) = valid_roots(image, geometry)
        .into_iter()
        .max_by_key(|(_, _, view)| (view.checkpoint_txg, view.instance))?;
    let (_, _, device, offset_in_bytes) = root_slot_positions(geometry).into_iter().find(
        |(candidate_region, candidate_slot, _, _)| {
            *candidate_region == region && *candidate_slot == slot
        },
    )?;
    let slot_bytes = ImageReader::read(image, device, offset_in_bytes, slot_width)?;
    Some(NewestRootSlot {
        device,
        offset_in_bytes,
        slot_bytes,
    })
}

/// 按一条码 2 / 码 3 指针读它指的单元：两条位置条目必须同槽（D2 已定项 10），不同槽就不走这条链。
fn read_unit_via_pointer(
    image: &MemoryPool,
    pointer: &[u8],
    unit_bytes: usize,
) -> Option<([u32; 2], u64, Vec<u8>)> {
    let view = parse_node_pointer(pointer);
    if view.all_zero || view.locations[0].slot != view.locations[1].slot {
        return None;
    }
    let devices = [view.locations[0].device, view.locations[1].device];
    let slot = view.locations[0].slot;
    let bytes = ImageReader::read(image, devices[0], slot * SLOT_BYTES, unit_bytes)?;
    Some((devices, slot, bytes))
}

fn write_unit_to_both_devices(image: &mut MemoryPool, devices: [u32; 2], slot: u64, bytes: &[u8]) {
    for device in devices {
        if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(device)) {
            sparse.write(DeviceOffsetInBytes(slot * SLOT_BYTES), bytes);
        }
    }
}

/// 链上走到树表为止的那一段：最新那条根与它指的树表节点。几棵树共用这一段，所以单拿出来——
/// 一步的镜像上把它读三遍，是「读 24 个根槽 + 一个 16 KiB 节点」读三遍。
#[derive(Clone, Debug)]
struct TreeTableReached {
    root_device: u32,
    root_offset: u64,
    root_slot: Vec<u8>,
    tree_table_devices: [u32; 2],
    tree_table_slot: u64,
    tree_table_node: Vec<u8>,
}

/// 从最新那条自证过的根走到树表节点，**条目宽是多少都走**；根择不出、树表指针全零都交回 None。
/// 只有坏树表单元自己那一条坏法走这一支（它要坏的正是条目宽），别的坏法走
/// [`reach_the_tree_table`]——它们要从树表条目里按 200 的固定偏移读种类与根指针。
fn reach_the_tree_table_whatever_its_entry_width_says(
    image: &MemoryPool,
) -> Option<TreeTableReached> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let root_slot = newest_root.slot_bytes;
    let (tree_table_devices, tree_table_slot, tree_table_node) = read_unit_via_pointer(
        image,
        &root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    Some(TreeTableReached {
        root_device: newest_root.device,
        root_offset: newest_root.offset_in_bytes,
        root_slot,
        tree_table_devices,
        tree_table_slot,
        tree_table_node,
    })
}

/// 从最新那条自证过的根走到树表节点；根择不出、树表指针全零、条目宽不是 200，都交回 None。
fn reach_the_tree_table(image: &MemoryPool) -> Option<TreeTableReached> {
    let reached = reach_the_tree_table_whatever_its_entry_width_says(image)?;
    if IndexNodeEntryLayout::of(&reached.tree_table_node).entry_width
        != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
    {
        return None;
    }
    Some(reached)
}

/// 从树表接着走到 `tree_kind` 那棵树的根节点；树表里没有这棵树、根指针全零、或者节点读不出来，都交回 None。
fn chain_from_the_tree_table(
    image: &MemoryPool,
    reached: &TreeTableReached,
    tree_kind: u16,
) -> Option<ChainToATreeRoot> {
    let layout = IndexNodeEntryLayout::of(&reached.tree_table_node);
    let tree_table_entry_index = (0..layout.entry_count).find(|index| {
        read_u16_at(
            &reached.tree_table_node,
            layout.offset_of_entry(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
        ) == tree_kind
    })?;
    let root_pointer_start =
        layout.offset_of_entry(tree_table_entry_index) + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let (tree_root_devices, tree_root_slot, tree_root_node) = read_unit_via_pointer(
        image,
        &reached.tree_table_node[root_pointer_start..root_pointer_start + 86],
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    Some(ChainToATreeRoot {
        tree_kind,
        root_device: reached.root_device,
        root_offset: reached.root_offset,
        root_slot: reached.root_slot.clone(),
        tree_table_devices: reached.tree_table_devices,
        tree_table_slot: reached.tree_table_slot,
        tree_table_node: reached.tree_table_node.clone(),
        tree_table_entry_index,
        tree_root_devices,
        tree_root_slot,
        tree_root_node,
    })
}

/// 开一条走到 `tree_kind` 那棵树根节点的链；链上哪一环走不通都交回 None。
fn open_chain_to_the_root_of(image: &MemoryPool, tree_kind: u16) -> Option<ChainToATreeRoot> {
    chain_from_the_tree_table(image, &reach_the_tree_table(image)?, tree_kind)
}

/// 走到中央映射树根节点的一条链。映射树的根**住根记录**、不经树表（D19 已定项 11），所以这条链只有两环、
/// 改完要重算的是三道校验和（节点的载荷 CRC 与头校验和、根槽的自证校验和），不是 [`ChainToATreeRoot`] 那五道。
struct ChainToTheCentralMappingTreeRoot {
    root_device: u32,
    root_offset: u64,
    root_slot: Vec<u8>,
    mapping_root_devices: [u32; 2],
    mapping_root_slot: u64,
    mapping_root_node: Vec<u8>,
}

/// 从最新那条自证过的根走到中央映射树根节点；根择不出、映射根指针全零、两条位置条目不同槽，都交回 None。
fn open_chain_to_the_central_mapping_tree_root(
    image: &MemoryPool,
) -> Option<ChainToTheCentralMappingTreeRoot> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let root_slot = newest_root.slot_bytes;
    let (mapping_root_devices, mapping_root_slot, mapping_root_node) = read_unit_via_pointer(
        image,
        &root_slot[ROOT_MAPPING_POINTER_OFFSET..ROOT_MAPPING_POINTER_OFFSET + 86],
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    Some(ChainToTheCentralMappingTreeRoot {
        root_device: newest_root.device,
        root_offset: newest_root.offset_in_bytes,
        root_slot,
        mapping_root_devices,
        mapping_root_slot,
        mapping_root_node,
    })
}

/// 把改过的映射链逐环写回镜像：映射树根 → 根记录里那条指针的整单元校验和 → 根槽的自证校验和。
fn seal_and_write_back_the_central_mapping_chain(
    mut chain: ChainToTheCentralMappingTreeRoot,
    image: &mut MemoryPool,
) {
    reseal_index_node(&mut chain.mapping_root_node);
    write_unit_to_both_devices(
        image,
        chain.mapping_root_devices,
        chain.mapping_root_slot,
        &chain.mapping_root_node,
    );
    let mapping_root_checksum = crc32_castagnoli_table(&chain.mapping_root_node);
    write_unit_checksum_into_both_locations(
        &mut chain.root_slot[ROOT_MAPPING_POINTER_OFFSET..ROOT_MAPPING_POINTER_OFFSET + 86],
        mapping_root_checksum,
    );
    let cover_end = chain.root_slot.len();
    reseal_wide_checksum_field(&mut chain.root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(chain.root_device)) {
        sparse.write(DeviceOffsetInBytes(chain.root_offset), &chain.root_slot);
    }
}

/// 把改过的树表单元逐环写回镜像：树表节点两道（载荷 CRC 与头校验和）→ 根记录里那条指针的整单元校验和 →
/// 根槽的自证校验和。坏的是树表**自己**，所以链只到树表为止，不往下走到某一棵树的根
/// （[`seal_and_write_back_the_chain`] 那五道里最里面两道没有对象）。
fn seal_and_write_back_the_tree_table(mut reached: TreeTableReached, image: &mut MemoryPool) {
    reseal_index_node(&mut reached.tree_table_node);
    write_unit_to_both_devices(
        image,
        reached.tree_table_devices,
        reached.tree_table_slot,
        &reached.tree_table_node,
    );
    let tree_table_checksum = crc32_castagnoli_table(&reached.tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut reached.root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = reached.root_slot.len();
    reseal_wide_checksum_field(&mut reached.root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(reached.root_device)) {
        sparse.write(DeviceOffsetInBytes(reached.root_offset), &reached.root_slot);
    }
}

/// 把改过的链逐环写回镜像：树根 → 树表条目里的整单元校验和 → 树表节点 → 根记录里的整单元校验和 → 根槽的自证校验和。
/// 少重算一道，读者在解析之前就先拒了，指着普查那几族的坏法一次也打不到。
fn seal_and_write_back_the_chain(mut chain: ChainToATreeRoot, image: &mut MemoryPool) {
    reseal_index_node(&mut chain.tree_root_node);
    write_unit_to_both_devices(
        image,
        chain.tree_root_devices,
        chain.tree_root_slot,
        &chain.tree_root_node,
    );

    let layout = IndexNodeEntryLayout::of(&chain.tree_table_node);
    let root_pointer_start =
        layout.offset_of_entry(chain.tree_table_entry_index) + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let tree_root_checksum = crc32_castagnoli_table(&chain.tree_root_node);
    write_unit_checksum_into_both_locations(
        &mut chain.tree_table_node[root_pointer_start..root_pointer_start + 86],
        tree_root_checksum,
    );
    reseal_index_node(&mut chain.tree_table_node);
    write_unit_to_both_devices(
        image,
        chain.tree_table_devices,
        chain.tree_table_slot,
        &chain.tree_table_node,
    );

    let tree_table_checksum = crc32_castagnoli_table(&chain.tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut chain.root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = chain.root_slot.len();
    reseal_wide_checksum_field(&mut chain.root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(chain.root_device)) {
        sparse.write(DeviceOffsetInBytes(chain.root_offset), &chain.root_slot);
    }
}

/// 码 2 索引节点头里层级那一字节的偏移（0 = 叶）。
const INDEX_NODE_LEVEL_OFFSET: usize = 50;

fn index_node_is_a_leaf(node: &[u8]) -> bool {
    node[INDEX_NODE_LEVEL_OFFSET] == 0
}

/// 一个码 2 节点条目区的样子：从哪起、几条、每条多宽（都按节点自述的字段读，没判过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IndexNodeEntryLayout {
    start_in_the_node: usize,
    entry_count: usize,
    entry_width: usize,
}

impl IndexNodeEntryLayout {
    fn of(node: &[u8]) -> Self {
        Self {
            start_in_the_node: index_node_entries_start(node),
            entry_count: usize::from(read_u16_at(node, index_node_entry_count_offset(node))),
            entry_width: usize::from(read_u16_at(node, index_node_entry_width_offset(node))),
        }
    }
    fn offset_of_entry(self, index: usize) -> usize {
        self.start_in_the_node + index * self.entry_width
    }
}

// ---- 逐个坏法 ----

/// 镜像上写过的每一个扇区（哪块盘、第几个扇区）。
fn written_sectors_of(image: &MemoryPool) -> Vec<(DeviceIdentity, u64)> {
    let mut sectors = Vec::new();
    for (identity, sparse) in &image.devices {
        for sector in sparse.written_sectors_in(DeviceOffsetInBytes(0), image.device_size_in_bytes)
        {
            sectors.push((*identity, sector));
        }
    }
    sectors
}

fn sector_of(sparse: &SparseDevice, sector: u64) -> Vec<u8> {
    sparse.read(
        DeviceOffsetInBytes(sector * SECTOR_BYTES),
        usize::try_from(SECTOR_BYTES).expect("512"),
    )
}

/// 按一个坏法把一份合法镜像坏掉；这个盘面上没有它要坏的对象时交回 None（那一条照实记成「不适用」，不当成跑过）。
#[must_use]
pub fn damage_image(
    base: &MemoryPool,
    kind: DamageKind,
    random: &mut SeededRandomSource,
) -> Option<DamagedImage> {
    let mut image = base.clone();
    let what = match kind {
        DamageKind::FlippedBitsInWrittenSectors => flip_bits(&mut image, random)?,
        DamageKind::ZeroedOneWrittenSector => zero_one_sector(&mut image, random)?,
        DamageKind::DroppedEverySectorPastACut => drop_every_sector_past_a_cut(&mut image, random)?,
        DamageKind::NarrowedEntryWidthOfTheExtentTreeRoot => {
            narrow_the_entry_width_of(&mut image, TREE_KIND_EXTENT)?
        }
        DamageKind::NarrowedEntryWidthOfTheInodeTreeRoot => {
            narrow_the_entry_width_of(&mut image, TREE_KIND_INODE)?
        }
        DamageKind::NarrowedEntryWidthOfTheAllocationRecordsTreeRoot => {
            narrow_the_entry_width_of(&mut image, TREE_KIND_ALLOCATION_RECORDS)?
        }
        DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot => {
            narrow_the_entry_width_of(&mut image, TREE_KIND_ACCOUNTING)?
        }
        DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot => {
            narrow_the_entry_width_of_the_central_mapping_tree_root(&mut image)?
        }
        DamageKind::NarrowedEntryWidthOfTheTreeTableUnit => {
            narrow_the_entry_width_of_the_tree_table_unit(&mut image)?
        }
        DamageKind::AllocationRecordSlotBelowTheUnitArea => {
            rewrite_an_allocation_record(&mut image, AllocationRecordDamage::SlotBelowTheUnitArea)?
        }
        DamageKind::AllocationRecordSpanPastTheEndOfTheUnitArea => rewrite_an_allocation_record(
            &mut image,
            AllocationRecordDamage::SpanPastTheEndOfTheUnitArea,
        )?,
        DamageKind::TwoAllocationRecordsCoveringTheSameSlot => rewrite_an_allocation_record(
            &mut image,
            AllocationRecordDamage::SameSlotAsTheSecondRecord,
        )?,
        DamageKind::AllocationRecordOnADeviceOutsideThePool => {
            rewrite_an_allocation_record(&mut image, AllocationRecordDamage::DeviceOutsideThePool)?
        }
        DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly => rewrite_an_allocation_record(
            &mut image,
            AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly,
        )?,
        DamageKind::RelabelledInodeWatermarkAccountingRow => {
            relabel_the_inode_watermark_row(&mut image)?
        }
        DamageKind::RegionCountPastTheThreeRegionArray => {
            widen_the_region_count_past_the_array(&mut image)?
        }
        DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot => {
            make_two_location_entries_disagree_on_the_slot(&mut image)?
        }
        DamageKind::RewrittenDataUnitResealedWithItsOwnChecksums => {
            rewrite_a_data_unit_resealing_only_its_own_checksums(&mut image)?
        }
    };
    Some(DamagedImage { kind, what, image })
}

fn flip_bits(image: &mut MemoryPool, random: &mut SeededRandomSource) -> Option<String> {
    let sectors = written_sectors_of(image);
    if sectors.is_empty() {
        return None;
    }
    let mut flipped = Vec::new();
    for _ in 0..BITS_FLIPPED_BY_ONE_DAMAGE {
        let (identity, sector) =
            sectors[usize::try_from(random.below(u64::try_from(sectors.len()).expect("扇区数")))
                .expect("下标")];
        let byte_in_sector = usize::try_from(random.below(SECTOR_BYTES)).expect("扇区内偏移");
        let bit = u8::try_from(random.below(BITS_IN_A_BYTE)).expect("位号");
        let sparse = image.devices.get_mut(&identity)?;
        let mut bytes = sector_of(sparse, sector);
        bytes[byte_in_sector] ^= 1u8 << bit;
        sparse.write(DeviceOffsetInBytes(sector * SECTOR_BYTES), &bytes);
        flipped.push(format!(
            "盘 {} 偏移 {} 的第 {bit} 位",
            identity.0,
            sector * SECTOR_BYTES + u64::try_from(byte_in_sector).expect("扇区内偏移")
        ));
    }
    Some(format!("翻位：{}", flipped.join("、")))
}

fn zero_one_sector(image: &mut MemoryPool, random: &mut SeededRandomSource) -> Option<String> {
    // 只挑本来就不全 0 的扇区：写过的扇区里有一批是补齐区，整块写的就是 0，清零它等于什么也没坏。
    let sectors: Vec<(DeviceIdentity, u64)> = written_sectors_of(image)
        .into_iter()
        .filter(|(identity, sector)| {
            image
                .devices
                .get(identity)
                .is_some_and(|sparse| sector_of(sparse, *sector).iter().any(|byte| *byte != 0))
        })
        .collect();
    if sectors.is_empty() {
        return None;
    }
    let (identity, sector) =
        sectors[usize::try_from(random.below(u64::try_from(sectors.len()).expect("扇区数")))
            .expect("下标")];
    let sparse = image.devices.get_mut(&identity)?;
    sparse.write(
        DeviceOffsetInBytes(sector * SECTOR_BYTES),
        &vec![0u8; usize::try_from(SECTOR_BYTES).expect("512")],
    );
    Some(format!(
        "清零：盘 {} 偏移 {} 那一个扇区整个写 0",
        identity.0,
        sector * SECTOR_BYTES
    ))
}

fn drop_every_sector_past_a_cut(
    image: &mut MemoryPool,
    random: &mut SeededRandomSource,
) -> Option<String> {
    let sectors = written_sectors_of(image);
    if sectors.is_empty() {
        return None;
    }
    let mut offsets: Vec<u64> = sectors
        .iter()
        .map(|(_, sector)| sector * SECTOR_BYTES)
        .collect();
    offsets.sort_unstable();
    offsets.dedup();
    // 切点不取最小的那个偏移：从第一个写过的扇区就切，整份镜像什么都不剩，那不是「截掉尾部」。
    let candidates = if offsets.len() > 1 {
        &offsets[1..]
    } else {
        &offsets[..]
    };
    let cut =
        candidates[usize::try_from(random.below(u64::try_from(candidates.len()).expect("偏移数")))
            .expect("下标")];
    let device_size_in_bytes = image.device_size_in_bytes;
    let mut dropped = 0u64;
    for sparse in image.devices.values_mut() {
        let kept: Vec<(u64, Vec<u8>)> = sparse
            .written_sectors_in(DeviceOffsetInBytes(0), device_size_in_bytes)
            .into_iter()
            .filter(|sector| {
                let keep = sector * SECTOR_BYTES < cut;
                if !keep {
                    dropped += 1;
                }
                keep
            })
            .map(|sector| (sector, sector_of(sparse, sector)))
            .collect();
        let mut truncated = SparseDevice::default();
        for (sector, bytes) in kept {
            truncated.write(DeviceOffsetInBytes(sector * SECTOR_BYTES), &bytes);
        }
        *sparse = truncated;
    }
    Some(format!(
        "截尾：两块盘上偏移 ≥ {cut} 的写过的扇区全丢掉（共 {dropped} 个扇区）"
    ))
}

/// 条目宽度那一族：把一棵树根自述的条目宽改成刚好等于 key 宽、条目数改成 1、声明长度跟着改。
/// 读者只判「条目宽 ≥ key 宽」（`unit.rs` 的 `parse_index_node`、checker 的 `index_node_view`），
/// 之后按字段表的固定偏移切条目——切出界就 panic（普查 R1–R4、R13）。
/// 一棵树一条坏法、不按种子挑：挑的话每个种子只打到四棵里的一棵，哪一处 panic 撞不撞得到就成了掷骰子。
/// 一个码 2 节点自述的条目宽缩到刚好等于 key 宽、条目数改成 1、声明长度跟着改；交回改之前的 (条目宽, 条目数, 新条目宽)。
/// 条目区本来就是 0 条、或者条目宽本来就等于 key 宽（没得缩）时交回 None。
/// **五棵树共用这一份**：偏移与缩法只有这一处，两条链（经树表的四棵、住根记录的映射树）各自重算自己那几道校验和。
fn narrow_the_entry_width_of_the_node(node: &mut [u8]) -> Option<(usize, usize, u16)> {
    let key_width = index_node_key_width(node);
    let layout = IndexNodeEntryLayout::of(node);
    if layout.entry_count == 0 || layout.entry_width <= key_width {
        return None;
    }
    let narrowed = u16::try_from(key_width).expect("key 宽装得进 u16");
    let entry_count_offset = index_node_entry_count_offset(node);
    let entry_width_offset = index_node_entry_width_offset(node);
    node[entry_count_offset..entry_count_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    node[entry_width_offset..entry_width_offset + 2].copy_from_slice(&narrowed.to_le_bytes());
    node[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
        .copy_from_slice(&narrowed.to_le_bytes());
    Some((layout.entry_width, layout.entry_count, narrowed))
}

fn narrow_the_entry_width_of(image: &mut MemoryPool, tree_kind: u16) -> Option<String> {
    let mut chain = open_chain_to_the_root_of(image, tree_kind)?;
    let (entry_width, entry_count, narrowed) =
        narrow_the_entry_width_of_the_node(&mut chain.tree_root_node)?;
    let what = format!(
        "条目宽：树种类 {} 的根（盘 {:?} 槽 {}，层级 {}）条目宽 {entry_width} → {narrowed}（= key 宽）、条目数 {entry_count} → 1、声明长度 → {narrowed}；链上三道校验和重算",
        chain.tree_kind,
        chain.tree_root_devices,
        chain.tree_root_slot,
        chain.tree_root_node[INDEX_NODE_LEVEL_OFFSET],
    );
    seal_and_write_back_the_chain(chain, image);
    Some(what)
}

/// 条目宽度那一族的第六处：**树表单元自己**（C504（树表条目宽在走读里无守卫，今天没坏法打得到））。
/// 前五条坏的都是树表**指着的**某棵树的根，走的是 [`reach_the_tree_table`]——它先判「条目宽是 200」，
/// 于是没有一条打得到树表自己；这一条因此走那支不判条目宽的 [`reach_the_tree_table_whatever_its_entry_width_says`]。
/// 条目宽 200 → 8（= 树表的 key 宽）、条目数 → 1、声明长度 → 8，重算树表节点两道与根槽自证一道。
fn narrow_the_entry_width_of_the_tree_table_unit(image: &mut MemoryPool) -> Option<String> {
    let mut reached = reach_the_tree_table_whatever_its_entry_width_says(image)?;
    let (entry_width, entry_count, narrowed) =
        narrow_the_entry_width_of_the_node(&mut reached.tree_table_node)?;
    let what = format!(
        "条目宽：树表单元自己（盘 {:?} 槽 {}）条目宽 {entry_width} → {narrowed}（= key 宽 8）、条目数 {entry_count} → 1、声明长度 → {narrowed}；节点两道 + 根槽自证一道校验和重算",
        reached.tree_table_devices, reached.tree_table_slot,
    );
    seal_and_write_back_the_tree_table(reached, image);
    Some(what)
}

/// 条目宽度那一族的第五棵树（普查 R2）：中央映射树的根。它住根记录，所以走的是那条只有两环的链。
fn narrow_the_entry_width_of_the_central_mapping_tree_root(
    image: &mut MemoryPool,
) -> Option<String> {
    let mut chain = open_chain_to_the_central_mapping_tree_root(image)?;
    let (entry_width, entry_count, narrowed) =
        narrow_the_entry_width_of_the_node(&mut chain.mapping_root_node)?;
    let what = format!(
        "条目宽：中央映射树根（住根记录，盘 {:?} 槽 {}，层级 {}）条目宽 {entry_width} → {narrowed}（= key 宽 27）、条目数 {entry_count} → 1、声明长度 → {narrowed}；节点两道 + 根槽自证一道校验和重算",
        chain.mapping_root_devices,
        chain.mapping_root_slot,
        chain.mapping_root_node[INDEX_NODE_LEVEL_OFFSET],
    );
    seal_and_write_back_the_central_mapping_chain(chain, image);
    Some(what)
}

/// 分配记录那一条改成什么样（普查 R6 / R7 / R8 / R9）。单列一个枚举，是为了让
/// [`rewrite_an_allocation_record`] 的 `match` 只面对它真做得出的那四样，用不着兜底臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocationRecordDamage {
    SlotBelowTheUnitArea,
    SpanPastTheEndOfTheUnitArea,
    SameSlotAsTheSecondRecord,
    DeviceOutsideThePool,
    InstanceTablePlacementReleasedOnTheSecondDeviceOnly,
}

/// 把「根槽 → 树表 → 分配记录树根」这条链上第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
/// （越过单元区末尾，普查 R6 / R7），并把链上的五道校验和逐道重算：分配记录树根两道 →
/// 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
///
/// **槽号也要挪，只改跨度不够**：几何判那四样按「设备身份 → 槽号下界 → 跨度上界 → 同盘两条罩同一个槽」的次序判，
/// 一条从单元区低处起跨 32767 槽的记录会先被**最后**那一样接走（它罩过了后面每一条记录），
/// 于是「跨度越过单元区末尾」那一判被遮蔽、去掉它也不红。挪到 `unit_area_end_slot - 1` 之后这条记录谁都不罩，
/// 跨度那一判是**唯一**拦着它的东西。
///
/// **三份字节由调用方从盘上读来、改完自己写回去**：坏盘输入那一路读的是 [`MemoryPool`]，
/// 步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
/// （`code-discipline.md`「重复要生成，不许手抄」）。跨度字段整个写成
/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 15 位的最大值、已释放位跟着清掉
/// （只有还着的记录才会被影子账拿去隔离）。
///
/// 交回「坏在哪」那句话；树表里没有分配记录树、它的根不是叶、或者一条记录都没有时交回 `None`。
#[must_use]
pub fn move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
    root_slot: &mut [u8],
    tree_table_node: &mut [u8],
    allocation_tree_node: &mut [u8],
    unit_area_end_slot: u64,
) -> Option<String> {
    let layout = IndexNodeEntryLayout::of(allocation_tree_node);
    if layout.entry_count == 0 || !index_node_is_a_leaf(allocation_tree_node) {
        return None;
    }
    let tree_table_layout = IndexNodeEntryLayout::of(tree_table_node);
    let allocation_entry_index = (0..tree_table_layout.entry_count).find(|index| {
        read_u16_at(
            tree_table_node,
            tree_table_layout.offset_of_entry(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
        ) == TREE_KIND_ALLOCATION_RECORDS
    })?;

    let first_entry = layout.offset_of_entry(0);
    let slot_offset = first_entry + ALLOCATION_RECORD_SLOT_OFFSET;
    let old_slot = read_six_byte_unsigned_at(allocation_tree_node, slot_offset);
    let last_slot_of_the_unit_area = unit_area_end_slot.checked_sub(1)?;
    write_six_byte_unsigned_at(
        allocation_tree_node,
        slot_offset,
        last_slot_of_the_unit_area,
    );
    let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
    let old_span_field = read_u16_at(allocation_tree_node, span_offset);
    allocation_tree_node[span_offset..span_offset + 2]
        .copy_from_slice(&SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes());
    reseal_index_node(allocation_tree_node);

    let root_pointer_start = tree_table_layout.offset_of_entry(allocation_entry_index)
        + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let allocation_tree_checksum = crc32_castagnoli_table(allocation_tree_node);
    write_unit_checksum_into_both_locations(
        &mut tree_table_node[root_pointer_start..root_pointer_start + 86],
        allocation_tree_checksum,
    );
    reseal_index_node(tree_table_node);

    let tree_table_checksum = crc32_castagnoli_table(tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = root_slot.len();
    reseal_wide_checksum_field(root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);

    Some(format!(
        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上五道校验和重算",
        old_span_field & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
    ))
}

/// 分配器那一族：把分配记录树根里的一条记录改坏（普查 R6 / R7 / R8 / R9）。
fn rewrite_an_allocation_record(
    image: &mut MemoryPool,
    damage: AllocationRecordDamage,
) -> Option<String> {
    let mut chain = open_chain_to_the_root_of(image, TREE_KIND_ALLOCATION_RECORDS)?;
    let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
    // 内部节点的条目不是分配记录（那几棵树的内部条目格式还没有条款），只坏叶。
    if layout.entry_count == 0 || !index_node_is_a_leaf(&chain.tree_root_node) {
        return None;
    }
    let first_entry = layout.offset_of_entry(0);
    let what = match damage {
        AllocationRecordDamage::SlotBelowTheUnitArea => {
            let old = read_six_byte_unsigned_at(
                &chain.tree_root_node,
                first_entry + ALLOCATION_RECORD_SLOT_OFFSET,
            );
            write_six_byte_unsigned_at(
                &mut chain.tree_root_node,
                first_entry + ALLOCATION_RECORD_SLOT_OFFSET,
                SLOT_FAR_BELOW_THE_UNIT_AREA,
            );
            format!(
                "分配器：第一条分配记录的槽号 {old} → {SLOT_FAR_BELOW_THE_UNIT_AREA}（单元区起点是 {UNIT_AREA_START_SLOT}）"
            )
        }
        AllocationRecordDamage::SpanPastTheEndOfTheUnitArea => {
            let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
            let old = read_u16_at(&chain.tree_root_node, span_offset);
            chain.tree_root_node[span_offset..span_offset + 2].copy_from_slice(
                &SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes(),
            );
            format!(
                "分配器：第一条分配记录的跨度字段 {old:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）",
                old & 0x7FFF
            )
        }
        AllocationRecordDamage::SameSlotAsTheSecondRecord => {
            if layout.entry_count < 2 {
                return None;
            }
            let second_entry = layout.offset_of_entry(1);
            let key_width = index_node_key_width(&chain.tree_root_node);
            let second_key = chain.tree_root_node[second_entry..second_entry + key_width].to_vec();
            chain.tree_root_node[first_entry..first_entry + key_width].copy_from_slice(&second_key);
            format!(
                "分配器：第一条分配记录的 key 换成第二条的（盘 {}、槽 {}），两条罩住同一个槽",
                read_u32_at(&second_key, ALLOCATION_RECORD_DEVICE_OFFSET),
                read_six_byte_unsigned_at(&second_key, ALLOCATION_RECORD_SLOT_OFFSET)
            )
        }
        AllocationRecordDamage::DeviceOutsideThePool => {
            let device_offset = first_entry + ALLOCATION_RECORD_DEVICE_OFFSET;
            let old = read_u32_at(&chain.tree_root_node, device_offset);
            chain.tree_root_node[device_offset..device_offset + 4]
                .copy_from_slice(&DEVICE_IDENTITY_OUTSIDE_THE_POOL.to_le_bytes());
            format!(
                "分配器：第一条分配记录的设备身份 {old} → {DEVICE_IDENTITY_OUTSIDE_THE_POOL}（池里只有 0 与 1）"
            )
        }
        AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly => {
            // 这一版实例表的落点：每次发布都重写实例表，所以这个落点一定走到释放判定路径
            // （`TransactionUnit::InstanceTable` 的落点从根记录里那条指针取，不经映射）。
            let instance_table_pointer = &chain.root_slot
                [ROOT_INSTANCE_TABLE_POINTER_OFFSET..ROOT_INSTANCE_TABLE_POINTER_OFFSET + 86];
            let slot = read_six_byte_unsigned_at(
                instance_table_pointer,
                POINTER_FIRST_LOCATION_OFFSET + LOCATION_SLOT_OFFSET_IN_ENTRY,
            );
            // 第二条位置条目那块盘：释放判定路径此前只核第一条那块。
            let second_device = read_u32_at(
                instance_table_pointer,
                POINTER_SECOND_LOCATION_OFFSET + LOCATION_DEVICE_OFFSET_IN_ENTRY,
            );
            let entry_index = (0..layout.entry_count).find(|index| {
                let entry = layout.offset_of_entry(*index);
                read_u32_at(
                    &chain.tree_root_node,
                    entry + ALLOCATION_RECORD_DEVICE_OFFSET,
                ) == second_device
                    && read_six_byte_unsigned_at(
                        &chain.tree_root_node,
                        entry + ALLOCATION_RECORD_SLOT_OFFSET,
                    ) == slot
            })?;
            let entry = layout.offset_of_entry(entry_index);
            let span_offset = entry + ALLOCATION_RECORD_SPAN_OFFSET;
            let old_span_field = read_u16_at(&chain.tree_root_node, span_offset);
            if old_span_field & SPAN_FIELD_RELEASED_BIT != 0 {
                return None;
            }
            chain.tree_root_node[span_offset..span_offset + 2]
                .copy_from_slice(&(old_span_field | SPAN_FIELD_RELEASED_BIT).to_le_bytes());
            let generation_offset = entry + ALLOCATION_RECORD_GENERATION_OFFSET;
            let old_generation = u64::from_le_bytes(
                chain.tree_root_node[generation_offset..generation_offset + 8]
                    .try_into()
                    .expect("8 字节"),
            );
            chain.tree_root_node[generation_offset..generation_offset + 8]
                .copy_from_slice(&RELEASE_GENERATION_ABOVE_EVERY_ROOT.to_le_bytes());
            format!(
                "分配器：实例表落点（槽 {slot}）在盘 {second_device} 上那条分配记录（第 {entry_index} 条）改成已释放、释放代 {old_generation} → {RELEASE_GENERATION_ABOVE_EVERY_ROOT}（高过任何根，挂载时不会被回收掉）；盘 {} 上那条原样",
                read_u32_at(
                    instance_table_pointer,
                    POINTER_FIRST_LOCATION_OFFSET + LOCATION_DEVICE_OFFSET_IN_ENTRY
                )
            )
        }
    };
    seal_and_write_back_the_chain(chain, image);
    Some(what)
}

/// 分配器那一族（普查 R11）：把记账树里 inode 号水位那一行改挂到登记表外的标签上，读路径取水位时找不到它。
fn relabel_the_inode_watermark_row(image: &mut MemoryPool) -> Option<String> {
    let mut chain = open_chain_to_the_root_of(image, TREE_KIND_ACCOUNTING)?;
    if !index_node_is_a_leaf(&chain.tree_root_node) {
        return None;
    }
    let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
    let watermark_row = (0..layout.entry_count).find(|index| {
        read_u16_at(
            &chain.tree_root_node,
            layout.offset_of_entry(*index) + ACCOUNTING_ENTRY_STATISTIC_OFFSET,
        ) == STATISTIC_INODE_WATERMARK_LABEL
    })?;
    let statistic_offset =
        layout.offset_of_entry(watermark_row) + ACCOUNTING_ENTRY_STATISTIC_OFFSET;
    chain.tree_root_node[statistic_offset..statistic_offset + 2]
        .copy_from_slice(&STATISTIC_LABEL_NOT_IN_THE_REGISTRY.to_le_bytes());
    let what = format!(
        "分配器：记账树第 {watermark_row} 条的统计量标签 {STATISTIC_INODE_WATERMARK_LABEL}（inode 号水位）→ {STATISTIC_LABEL_NOT_IN_THE_REGISTRY}"
    );
    seal_and_write_back_the_chain(chain, image);
    Some(what)
}

/// 写侧断言那一族（普查 R12）：把每块盘每一个自证过的系统配置槽里的区域数改到那个长 3 的数组之外，整槽校验和重算。
fn widen_the_region_count_past_the_array(image: &mut MemoryPool) -> Option<String> {
    let geometry = newest_geometry(image)?;
    let mut changed = Vec::new();
    let devices: Vec<u32> = ImageReader::devices(image);
    for device in devices {
        for slot_offset in [0, geometry.slot_spacing] {
            let Some(mut slot) =
                ImageReader::read(image, device, slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
            else {
                continue;
            };
            if check_system_configuration_slot(&slot).is_err() {
                continue;
            }
            let old = slot[SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET];
            slot[SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET] =
                REGION_COUNT_PAST_THE_THREE_REGION_ARRAY;
            reseal_wide_checksum_field(
                &mut slot,
                SYSTEM_CONFIGURATION_SLOT_BYTES,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(device)) {
                sparse.write(DeviceOffsetInBytes(slot_offset), &slot);
            }
            changed.push(format!("盘 {device} 偏移 {slot_offset}（{old} → {REGION_COUNT_PAST_THE_THREE_REGION_ARRAY}）"));
        }
    }
    (!changed.is_empty()).then(|| format!("写侧断言：区域数改到数组之外——{}", changed.join("、")))
}

/// 写侧断言那一族（普查 R5）：把最新那条根记录里实例表指针的两条位置条目改成不同的槽号，整槽校验和重算。
fn make_two_location_entries_disagree_on_the_slot(image: &mut MemoryPool) -> Option<String> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let device = newest_root.device;
    let offset = newest_root.offset_in_bytes;
    let mut slot = newest_root.slot_bytes;
    let pointer_start = ROOT_INSTANCE_TABLE_POINTER_OFFSET;
    let view = parse_node_pointer(&slot[pointer_start..pointer_start + 86]);
    if view.all_zero {
        return None;
    }
    let second_slot_offset =
        pointer_start + POINTER_SECOND_LOCATION_OFFSET + LOCATION_SLOT_OFFSET_IN_ENTRY;
    let old = read_six_byte_unsigned_at(&slot, second_slot_offset);
    let changed = old + SLOT_OFFSET_THAT_BREAKS_TWO_DEVICES_ON_ONE_SLOT;
    write_six_byte_unsigned_at(&mut slot, second_slot_offset, changed);
    let cover_end = slot.len();
    reseal_wide_checksum_field(&mut slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    image
        .devices
        .get_mut(&DeviceIdentity(device))?
        .write(DeviceOffsetInBytes(offset), &slot);
    Some(format!(
        "写侧断言：盘 {device} 偏移 {offset} 的根记录里实例表指针，第二条位置条目的槽号 {old} → {changed}（第一条还是 {}）",
        view.locations[0].slot
    ))
}

/// 判别力那一条：把最新那条根下 extent 树第一条记录指的数据单元整个重写，**只**重算这个单元自己的两道校验和，
/// extent 记录里位置条目上的整单元校验和原样留着。
/// 挡着它的只有恢复里 `read_unit_via_locations` 那一道整单元 CRC 比对：去掉它，恢复就读回这一版从没提交过的内容。
fn rewrite_a_data_unit_resealing_only_its_own_checksums(image: &mut MemoryPool) -> Option<String> {
    let chain = open_chain_to_the_root_of(image, TREE_KIND_EXTENT)?;
    let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
    if layout.entry_count == 0
        || !index_node_is_a_leaf(&chain.tree_root_node)
        || layout.entry_width < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112")
    {
        return None;
    }
    let key_width = index_node_key_width(&chain.tree_root_node);
    let data_pointer_start = layout.offset_of_entry(0) + key_width;
    let view =
        parse_data_pointer(&chain.tree_root_node[data_pointer_start..data_pointer_start + 88]);
    if view.all_zero || view.locations[0].slot != view.locations[1].slot {
        return None;
    }
    let devices = [view.locations[0].device, view.locations[1].device];
    let slot = view.locations[0].slot;
    let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut unit = ImageReader::read(image, devices[0], slot * SLOT_BYTES, unit_bytes)?;
    let declared_length = usize::from(read_u16_at(&unit, UNIT_DECLARED_LENGTH_OFFSET));
    if declared_length == 0 {
        return None;
    }
    let payload_start = usize::try_from(DATA_UNIT_PAYLOAD_OFFSET).expect("134");
    unit[payload_start..payload_start + declared_length]
        .fill(BYTE_THAT_NO_COMMITTED_VERSION_CONTAINS);
    reseal_data_unit(&mut unit);
    write_unit_to_both_devices(image, devices, slot, &unit);
    Some(format!(
        "判别力：盘 {devices:?} 槽 {slot} 的数据单元载荷（{declared_length} 字节）整个写成 {BYTE_THAT_NO_COMMITTED_VERSION_CONTAINS:#04x}，只重算它自己的两道校验和；extent 记录里的整单元校验和原样留着"
    ))
}

// ---- 一批种子 ----

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片
/// （与崩溃注入共用 [`seed_slices`]），线程按片号从小到大领片，调用线程收到一片就打一行 `BAD_DISK_INPUT_PROGRESS`，
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以 [`BadDiskReport::render`] 与线程数无关。
///
/// # Panics
/// 有一片领了却没交回（工作线程自己 panic 了——坏镜像上的 panic 在 [`with_panic_capture`] 里接住，走到这里的是别的）。
#[must_use]
pub fn run_bad_disk_campaign(campaign: &BadDiskCampaign) -> BadDiskReport {
    let BadDiskCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        weights,
        execution,
        worker_threads,
    } = *campaign;
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    println!(
        "BAD_DISK_INPUT_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        first_seed + seed_count,
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryBadDiskInput>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let inputs: Vec<HistoryBadDiskInput> = slice
                    .clone()
                    .map(|offset| {
                        feed_bad_disk_inputs_from_history(
                            HistorySeed(first_seed.wrapping_add(offset)),
                            operations_per_history,
                            &weights,
                            execution,
                        )
                    })
                    .collect();
                if sender.send((slice_index, inputs)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryBadDiskInput>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryBadDiskInput> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, inputs)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "BAD_DISK_INPUT_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                first_seed.wrapping_add(slice.start),
                first_seed.wrapping_add(slice.end),
                finished_slice_count + 1,
                slices.len(),
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(slice_index, inputs);
            while let Some(ready) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                in_order.extend(ready);
                next_slice_to_merge += 1;
            }
        }
        (in_order, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "BAD_DISK_INPUT_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed + seed_count,
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = BadDiskTally::default();
    let mut findings: Vec<BadDiskFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<String> = BTreeSet::new();
    let mut digest_text = String::new();
    for input in &finished_slices {
        tally.absorb(&input.tally);
        digest_text.push_str(&input.digest_line());
        for finding in &input.findings {
            if signatures_seen.insert(finding.signature()) {
                findings.push(finding.clone());
            }
        }
    }
    BadDiskReport {
        first_seed,
        seed_count,
        operations_per_history,
        weights,
        execution,
        tally,
        findings,
        in_order_digest: crate::fnv1a_64(digest_text.as_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_damage_kind_is_listed_once_and_names_are_distinct() {
        let names: BTreeSet<&'static str> =
            EVERY_DAMAGE_KIND.iter().map(|kind| kind.name()).collect();
        assert_eq!(
            names.len(),
            EVERY_DAMAGE_KIND.len(),
            "每种坏法一个名字，报告按名字分格"
        );
        let kinds: BTreeSet<DamageKind> = EVERY_DAMAGE_KIND.into_iter().collect();
        assert_eq!(kinds.len(), EVERY_DAMAGE_KIND.len(), "清单里没有重复");
    }
}
