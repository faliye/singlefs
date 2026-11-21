//! 坏盘输入（里程碑「覆盖写、释放、回退与复用」增补 3 第 5 件）：拿第 1 件生成的合法镜像按种子坏掉，喂给恢复、可写挂载与池级 checker。
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
//! 默认批里该撞几次），清单外的算新发现、判红；次数涨了也判红。增补 3 那条验收「三个读者都不许 panic」
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
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteReader;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::transaction::{instance_table_chain_to_release, PublishError, TransactionUnit};
use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, DATA_UNIT_PAYLOAD_OFFSET, EXTENT_LEAF_RECORD_BYTES,
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, NODE_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    SLOT_BYTES, TREE_TABLE_ENTRY_BYTES, UNIT_AREA_START_SLOT, WIDE_CHECKSUM_BYTES,
};

use crate::crash_injection::{seed_slices, CrashInjectionWorkerThreads};
use crate::history::{
    count_from_reproduction_environment, execute_history_with, generate_history_with_weights,
    number_from_reproduction_environment, render_environment_assignments, with_panic_capture,
    CapturedPanic, GenerationWeights, HistoryDeviceWidth, HistoryExecution, HistorySampling,
    HistorySeed, ReproductionEnvironmentError, ReproductionVariablePrefix, SeededRandomSource,
};
use crate::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice, SECTOR_BYTES};
use crate::model::ObservedReadBack;
use crate::model_comparison::observed_read_back;
use crate::SharedStream;

mod damages_to_the_image;
mod feeding_damaged_images_to_the_readers;
mod reaching_and_resealing_units_on_the_image;

pub use damages_to_the_image::*;
pub use feeding_damaged_images_to_the_readers::*;
pub use reaching_and_resealing_units_on_the_image::*;

/// 坏盘输入的工作线程数从这个环境变量取（十进制正整数）；没设就取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值；判法在 [`CrashInjectionWorkerThreads::from_the_environment_variable_named`]）。
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

/// 码 2 索引节点头里 key 宽那一字节的偏移（D18（块里携带什么信息） 已定项 18）。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;

/// 码 2 索引节点头里 key 区间的起点：最小 key 紧跟 key 宽那一字节，最大 key 紧跟最小 key（各 key 宽那么长；
/// `singlefs-core` 的 `build_index_node` 同一张字段表，这里按字段表另写一份）。
const INDEX_NODE_SMALLEST_KEY_OFFSET: usize = INDEX_NODE_KEY_WIDTH_OFFSET + 1;

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
/// 重写数据单元载荷时填的那个字节：与内容生成器写得出的任何一版都不同（`scenario::first_file_content` 按下标取、`history::ContentChoice::bytes` 按种子现算，
/// 整片填这一个值的只有坏盘输入），读回它就是「读回了一版从没提交过的内容」。
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
    /// 条目宽度那一族（普查 R4、R13）：改记账树根（key 宽 22，条目该有 34），条目宽缩到 key 宽，**每条条目只留它的 key、
    /// 条目数不变**，节点头的 key 区间照旧与首末两条对得上。只缩成 1 条（extent、inode、分配记录树根那三种的缩法）过不了判全那一道：可写挂载重建与冷走读
    /// 都照冷走读判全（代码审阅第 24 / 33 条），先红在「根 key 区间与条目不符」上；重建不判节点头时（变异「实审 A3d」那一形），
    /// 起重建数条目之前先核条目宽，可写挂载红在「记账条目宽不到字段表」上，不再是「记账条目数不是 3 + 6 × 盘数」；
    /// 都走不到逐条解记账条目那一步；那一种原来的样子留在 [`DamageRefusedAtTheHeaderJudgement`]。
    NarrowedEntryWidthOfTheAccountingTreeRoot,
    /// 条目宽度那一族（普查 R2、R13）：同上，改中央映射树根（key 宽 27，条目该有 55），每条只留 key、条目数不变
    /// （映射条目数同样要对得上进映射的单元数）。映射树的根住根记录、不经树表
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
    /// （释放代写成高过任何根的一个数，免得挂载时就被回收掉）。释放判定路径只核第一条位置条目那块盘的话，
    /// 于是「两块盘的分配记录树不对称」原样喂进 `PoolAllocator::release` 的断言。
    /// 只动已释放位与释放代：两块盘的位图一个字节都不差（`mark_released` 不清位），
    /// 挂载之后的落点政策两盘照样答同一个槽，坏法不会在别处先被拦下。
    /// 可写挂载重建判全（代码审阅第 24 / 33 条）之后，这份镜像在重建那一步就红在「分配记录不是每个落点每盘各一条」上
    /// （那一道之后还有「分配代 / 释放代不晚于根」，高过任何根的释放代同样过不去），释放判定那一处从盘上走不到了；那一处改由 [`release_check_of_the_instance_table_placement_released_on_the_second_device_only`]
    /// 直接调发布路径的释放判定。
    AllocationRecordReleasedOnTheSecondDeviceOnly,
    /// 分配器那一族（普查 R11）：把记账树里 inode 号水位那一行的统计量标签换掉，读路径找不到水位。
    /// 水位那一行排在最后，换上的标签仍排在最后，**节点头的最大 key 跟着改成这一行的新 key**：头里的最大 key 照旧的话，
    /// 两个读者都先红在判全那一道「根 key 区间与条目不符」上（代码审阅第 24 / 33 条），走不到取水位那一步；
    /// 那一种原来的样子留在 [`DamageRefusedAtTheHeaderJudgement`]。
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
                "条目宽：记账树根自述的条目宽缩到 key 宽、每条只留 key（链上校验和重算）"
            }
            DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot => {
                "条目宽：中央映射树根自述的条目宽缩到 key 宽、每条只留 key（根记录那条链上校验和重算）"
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
                "分配器：记账树里 inode 号水位那一行被改挂到别的标签上（节点头的最大 key 跟着改）"
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

/// 今天撞得到的一处 panic：清单里的名字、它是普查里的哪一条、落在哪个文件、消息里认得出它的那一段、默认批里撞到几次。
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
    /// 默认批（[`singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`] 起 12 段、每段 16 步）里这一条撞到几次。
    /// **次数只许往下走，不许涨**：只按「文件 + 消息片段」认的清单会把同一个文件里同形的新 panic
    /// 静默接走（例如 `transaction.rs` 里另起一处「两盘同槽」的断言），次数涨了就红在这里。
    pub expected_hits_in_the_default_batch: u64,
}

/// 今天撞得到的 panic。表里每一条都是被测代码的缺口，不是用例写错了；清单外的 panic 算新发现、判红。
///
/// **这张表是增补 3 那条验收「三个读者都不许 panic」（第 5 件「写死的种子数上零 panic」）的登记位：
/// 它空着 ⇒ 那条验收在默认批（写死的种子基、12 段 × 16 步）上成立，
/// 三个读者一次都没 panic。**表空着不许删掉：它现在判的是「一条都不许回来」——任何一处 panic 都落进
/// 「清单外的新发现」那一格、当场判红。
///
/// 两条规矩（`tests/bad_disk_input_campaign.rs` 逐条判）：
/// 1. **每条带期望次数，次数涨了判红**（[`KnownPanicSite::expected_hits_in_the_default_batch`]）。
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
/// `tests/bad_disk_input_campaign.rs`。
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
    /// 「系统配置里的区域数越过那个长 3 的数组」那条坏法只有 checker 的几何读者判得出
    /// （核心层一处都不读盘上那个区域数，写与读都按格式常量 3 走），而 checker 读不出几何时把每一条都报成不适用、
    /// 一条都不红——这条坏法只会在「底片本身已经坏到读者报错」的那几段历史上被偶然算成看见过。
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

    /// 这一批的取样参数（每段步数、比重、跑法）。
    #[must_use]
    pub fn history_sampling(&self) -> HistorySampling {
        HistorySampling {
            operations_per_history: self.operations_per_history,
            weights: self.weights,
            execution: self.execution,
        }
    }

    /// 重放这一整批的那一行命令（[`bad_disk_input_replay_line`]）。
    #[must_use]
    pub fn replay_line(&self) -> String {
        bad_disk_input_replay_line(self.first_seed, self.seed_count, &self.history_sampling())
    }
}

/// 坏盘输入那个二进制的环境变量前缀；取样参数（每段步数、比重、checker、盘宽、空间准入）的后缀与随机历史共用
/// （`singlefs_verification::history::HistorySamplingVariable`，格式只有那一份定义；代码三方 test-script-align-code-r1 判决 C607）。
pub const BAD_DISK_INPUT_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_BAD_DISK_");

crate::enum_listing_every_member! {
/// 重放行与坏盘输入入口里坏盘输入自己的两个变量（接在 [`BAD_DISK_INPUT_VARIABLE_PREFIX`] 后面；取样参数接在它们后面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadDiskInputVariable {
    /// 种子区间的起点。
    FirstSeed,
    /// 种子数。
    Seeds,
}
}

impl BadDiskInputVariable {
    #[must_use]
    pub fn variable_name(self) -> String {
        BAD_DISK_INPUT_VARIABLE_PREFIX.variable_named(match self {
            BadDiskInputVariable::FirstSeed => "FIRST_SEED",
            BadDiskInputVariable::Seeds => "SEEDS",
        })
    }
}

/// 坏盘输入判红时打的重放命令：种子区间与取样参数（每段步数、比重、checker、盘宽、空间准入），
/// 交坏盘输入那条入口（`bad_disk_input_campaign` 的 `bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites`）照抄就是原来那一批。
/// 线程数不在里面：线程数不改结论（`the_report_is_the_same_with_one_worker_thread_and_four`）。
#[must_use]
pub fn bad_disk_input_replay_line(
    first_seed: u64,
    seed_count: u64,
    sampling: &HistorySampling,
) -> String {
    let mut assignments: Vec<(String, String)> = BadDiskInputVariable::ALL
        .into_iter()
        .map(|variable| {
            let value = match variable {
                BadDiskInputVariable::FirstSeed => first_seed.to_string(),
                BadDiskInputVariable::Seeds => seed_count.to_string(),
            };
            (variable.variable_name(), value)
        })
        .collect();
    assignments.extend(sampling.environment_assignments(BAD_DISK_INPUT_VARIABLE_PREFIX));
    format!(
        "{} cargo test --release -p singlefs-verification --test bad_disk_input_campaign -- --exact bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites",
        render_environment_assignments(&assignments)
    )
}

/// 坏盘输入那条入口读环境：读的变量与 [`bad_disk_input_replay_line`] 写出的逐个相同，没设的取 `defaults` 的；
/// 线程数不在重放行里，照 `defaults` 的。种子数是「跑几段」的计数，经 `history` 的计数读法读（写 0 与写错一样认不得：
/// 0 段的一批照「零 panic、不读回没提交过的版本」判成绿，上游 `show-me-test.md`「扫到 0 项也不是通过」）。
///
/// # Errors
/// 某个变量的取值认不得（种子数写 0 也算）。
pub fn bad_disk_campaign_from_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    defaults: &BadDiskCampaign,
) -> Result<BadDiskCampaign, ReproductionEnvironmentError> {
    let first_seed = number_from_reproduction_environment(
        lookup,
        &BadDiskInputVariable::FirstSeed.variable_name(),
        defaults.first_seed,
    )?;
    let seed_count = count_from_reproduction_environment(
        lookup,
        &BadDiskInputVariable::Seeds.variable_name(),
        defaults.seed_count,
    )?;
    let sampling = HistorySampling::from_environment(
        BAD_DISK_INPUT_VARIABLE_PREFIX,
        lookup,
        &HistorySampling {
            operations_per_history: defaults.operations_per_history,
            weights: defaults.weights,
            execution: defaults.execution,
        },
    )?;
    Ok(BadDiskCampaign {
        first_seed,
        seed_count,
        operations_per_history: sampling.operations_per_history,
        weights: sampling.weights,
        execution: sampling.execution,
        worker_threads: defaults.worker_threads,
    })
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

/// 镜像上最新那条自证过的根：住哪块盘的哪个偏移、整槽的字节、它的 txg。
#[derive(Clone, Debug)]
struct NewestRootSlot {
    device: u32,
    offset_in_bytes: u64,
    slot_bytes: Vec<u8>,
    checkpoint_txg: u64,
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

/// 从一棵树的根往下走、沿路读到的一个节点：它落在的两块盘与槽、它的字节，与上一层里指着它的是第几条条目。
struct NodeBelowTheRoot {
    devices: [u32; 2],
    slot: u64,
    bytes: Vec<u8>,
    entry_index_in_the_parent: usize,
}

/// 码 2 索引节点头里层级那一字节的偏移（0 = 叶）。
const INDEX_NODE_LEVEL_OFFSET: usize = 50;

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

    /// 池级 checker 判出来的只有不适用与部分没判：算「一条都没判」（读者拒收整份镜像那一格），部分没判不算判过；
    /// 有一条成立或违例就不算。
    #[test]
    fn checker_report_of_only_not_applicable_and_partially_judged_verdicts_counts_as_judging_nothing(
    ) {
        let not_applicable_and_partially_judged = [
            ("I-1.1", InvariantVerdict::NotApplicable("读不出几何")),
            (
                "I-9.28",
                InvariantVerdict::PartiallyJudged(
                    "盘 0 上的根（实例 1、txg 4）：树表读不出".to_string(),
                ),
            ),
        ];
        assert!(
            checker_judged_no_invariant(&not_applicable_and_partially_judged),
            "只有不适用与部分没判：一条都没判"
        );
        for judged in [
            InvariantVerdict::Holds,
            InvariantVerdict::Violated("对不上".to_string()),
        ] {
            let mut report = not_applicable_and_partially_judged.to_vec();
            report.push(("I-3.1", judged.clone()));
            assert!(
                !checker_judged_no_invariant(&report),
                "有一条 {judged:?}：判过"
            );
        }
    }
}
