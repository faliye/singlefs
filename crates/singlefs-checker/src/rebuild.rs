//! 扫描重建（只读，不写回盘）：不读根环与树表，只从扫描方向扫出的单元头（加当前版本那几片码 3 容器的记录）重建「索引」。
//! 第一版三张派生表：中央映射、每个 inode 的数据单元集合、inode 记录集合（`.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」；
//! D21（权威态与派生态的分界） 已定项 13「救援、换树、迁移是同一条路径」）。分配记录树、记账树是派生态（D21（权威态与派生态的分界）
//! 已定项 5），第一版不重建、不比对；deadlist / livelist 等 C301（deadlist 与 livelist 的态别没写） 定了再接。遍历方向从最新根读出的
//! 三张表与这里重建的逐项比，比对在 `walk` 模块 `check_pool_image` 的末尾（I-1.13（扫描重建与遍历方向的索引相同））。
//!
//! 这里只有纯函数，分两步：[`choose_current_versions`] 从单元头择出每个对象身份的当前版本；调用方照
//! [`CurrentVersions::packed_containers_whose_records_are_read`] 把那几片码 3 的整单元字节读来，交给 [`rebuild_index_tables`]
//! 现算三张表。条款没写的那几格不猜：交回 [`ScanRebuildNotDecided`] 的一个成员（名字写明是哪条条款没定、第一版不认），调用方报不适用。
//!
//! 与遍历方向共享的只有格式解析（D13（验证路线） 已定项 12）：这里只用 `singlefs_format` 的常量与 crate 根上解单元头、
//! 解码 3 记录的函数与偏移，不调走树的函数（`walk` 模块与它的节点缓存）。本文件末尾的单测把 `use` 清单与白名单逐项比，
//! 并核非测试代码里每一处指名 crate 内一项的路径都落在白名单里（C410（checker 两个方向共享什么没有检查） 的会红形态）。
//!
//! 条款没逐字写、按条款推出来的四处读法（重建这一侧与 `walk` 里比对那一侧各两处）：
//! - 读法：没有 inode 记录也没有墓碑压着的码 1 单元照认领，进数据单元集合与中央映射（主 agent 2026-10-13 认，条款没写）。
//! - 读法：中央映射同一 key 两边的落点集合不同时，两边各只留读得出的副本再比，掉一份副本的降级镜像不判红（主 agent 2026-10-13 认，条款没写）。
//! - 读法：三张表在两边都是空表（mkfs、暖机之后）判成立，不判不适用：比对真跑了（主 agent 2026-10-13 认，条款没写）。
//! - 读法：每个 inode 的数据单元集合那张表的值比映射 key，不比落点，落点归中央映射那张表（主 agent 2026-10-13 认，条款没写）。
//!
//! 谱系记录与退役记录的两条读法（用户 2026-10-14 定，`research/prompts/m4-batch2-code-r1-forks.md` 第 1 行「甲」）：
//! - 谱系记录（C615（扫盘重建对挂着时的回退没有输入））按「不是任何现行版本的成员」认领：随回退走的那几类（码 1 数据单元、
//!   inode 叶容器）只在诞生代号落在最新那一版谱系上的版本里择当前版本，回退抛弃的那一段里诞生的版本不进任何一张表；
//!   随现行那一版走的树（墓碑树、退役树、谱系树）的容器不按谱系滤、只按已发布谓词择（用户 2026-10-08 定的 X1「退役记录·甲」）。
//! - 退役记录（C768（退役记录的扫描重建读侧没有消费者））按退役树认领：死亡 txg 在谱系上的退役记录点名的 inode 叶容器整个身份不认领。
//!
//! 两条读法落到代码时按条款推出来、条款没逐字写的三处（实现员 2026-10-15 推的，交主 agent 认）：
//! - 读法：「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义，父链的起点取 2⁶⁴ − 1 而不取最新根的 txg（重建不读根）：
//!   已发布的单元诞生代号都不过最新根的 txg（I-1.2（块头写序已发布）），已发布的谱系容器里的 T_rb 也都不过它，两个起点认出的是同一批版本。
//! - 读法：墓碑只认死亡 txg 在谱系上的（I-9.19（谱系上的墓碑不指活对象） 同一定义），回退抛弃的那一段里写的墓碑不压回退目标那一版里活着的对象。
//! - 读法：退役记录只认被退役身份的打包记录类型是 2（inode 叶容器）的，别的类型归 I-9.32（退役记录合法） 判。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, INODE_RECORD_BYTES, LINEAGE_RECORD_BYTES,
    MAPPING_KEY_BYTES, RETIREMENT_RECORD_BYTES, TOMBSTONE_RECORD_BYTES,
};

use crate::{
    packed_unit_view, read_six_byte_unsigned, read_u16, read_u32, read_u64, ContentUnitClass,
    DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET, DATA_UNIT_OBJECT_BIRTH_OFFSET,
    DATA_UNIT_OBJECT_OFFSET, INODE_RECORD_OBJECT_BIRTH_OFFSET, INODE_RECORD_SIZE_OFFSET,
    INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET, INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET,
    MAPPING_KEY_BIRTH_TREE_OFFSET, MAPPING_KEY_BIRTH_TXG_OFFSET, MAPPING_KEY_INSTANCE_OFFSET,
    MAPPING_KEY_TAIL_OFFSET, MAPPING_KEY_TAIL_PADDING, MAPPING_KEY_UNIT_CLASS_OFFSET,
    PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, PACKED_TYPE_LINEAGE, PACKED_TYPE_RETIREMENT,
    PACKED_TYPE_TOMBSTONE, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
    TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET, TOMBSTONE_RECORD_RANGE_END_OFFSET,
    TOMBSTONE_RECORD_RANGE_START_OFFSET, TOMBSTONE_WHOLE_OBJECT_RANGE_END,
    TOMBSTONE_WHOLE_OBJECT_RANGE_START, WRITE_ORDER_TRANSACTION_OFFSET,
};

/// 一份单元副本的落点：(设备, 起点槽)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UnitPlacement {
    pub(crate) device: u32,
    pub(crate) slot: u64,
}

/// 扫描方向扫出的一份码 1 / 码 3 单元副本：magic、类标签、头校验和、fsid 都过了，已发布谓词也判成已发布——
/// 这两步由调用方做（`walk` 的扫描方向，与 I-1.8（归并后版本全序） 那一遍同一份扫描结果）。
pub(crate) struct PublishedUnitCopy {
    pub(crate) placement: UnitPlacement,
    /// 明文头原样（`[0, 这一类的明文头末尾)`）。
    pub(crate) plain_header: Vec<u8>,
}

/// 中央映射 key 27 字节（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10）：码 1 = 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10；
/// 码 3 = 类标签 + 出生树 + 出生 txg + 实例代号 4 + 出生序号 4 + 补零 2。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CentralMappingKey(pub(crate) Vec<u8>);

impl CentralMappingKey {
    /// 判红时写进说明的样子：类标签、出生树、出生 txg、实例代号与尾段各自读出来。
    pub(crate) fn describe(&self) -> String {
        let key = &self.0;
        let tail = match ContentUnitClass::of(key[MAPPING_KEY_UNIT_CLASS_OFFSET]) {
            Some(ContentUnitClass::Data) => format!(
                "事务号 {}",
                read_six_byte_unsigned(key, MAPPING_KEY_TAIL_OFFSET)
            ),
            Some(ContentUnitClass::PackedRecord) | None => {
                format!("出生序号 {}", read_u32(key, MAPPING_KEY_TAIL_OFFSET))
            }
        };
        format!(
            "（码 {}、出生树 {}、出生 txg {}、实例 {}、{tail}）",
            key[MAPPING_KEY_UNIT_CLASS_OFFSET],
            read_u64(key, MAPPING_KEY_BIRTH_TREE_OFFSET),
            read_u64(key, MAPPING_KEY_BIRTH_TXG_OFFSET),
            read_u32(key, MAPPING_KEY_INSTANCE_OFFSET),
        )
    }
}

/// 类身份段里与版本无关的那一段（码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代））：同一个对象身份的各版共用它。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ObjectIdentity(pub(crate) Vec<u8>);

/// 每个 inode 的数据单元集合那张表的 key：extent key (0, inode, 偏移) 的后两段（D8（核心索引结构） 已定项 3：偏移是文件字节偏移）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DataUnitPosition {
    pub(crate) inode: u64,
    pub(crate) offset_in_bytes: u64,
}

/// 重建出的三张表。
pub(crate) struct RebuiltIndexTables {
    /// 每个已发布、当前版本、活着的码 1 单元与每片当前版本的码 3 容器（实例表的片除外：它不进映射，D19（块指针的结构与宽度预算）
    /// 已定项 8 / 已定项 12）→ 扫到它的副本的落点。
    pub(crate) central_mapping: BTreeMap<CentralMappingKey, BTreeSet<UnitPlacement>>,
    /// (inode, 偏移) → 那个数据单元当前版本的映射 key。
    pub(crate) data_units_of_each_inode: BTreeMap<DataUnitPosition, CentralMappingKey>,
    /// inode 号 → 当前版本 inode 叶容器里那条记录 140 字节原样（墓碑压着的对象的记录不在里面，在 `records_of_the_tombstoned_objects`）。
    pub(crate) inode_records: BTreeMap<u64, Vec<u8>>,
    /// 墓碑压着、记录还在的对象：inode 号 → 当前版本 inode 叶容器里它那条记录 140 字节原样（谱系上有它那一对 (号, 对象出生代) 的墓碑）。
    /// 这条记录不进上面的 inode 记录集合，它的数据单元按墓碑不认领。算不算删除中，这一侧认不了，由比对那一侧（`walk` 模块）两侧一起认
    /// （判决 `research/prompts/m4-c860-r1-main-verification.md` 第三节 C2，用户 2026-10-20 选丙）：遍历方向也有同号、同对象出生代的记录，
    /// 且两侧那条记录 nlink 都是 0（D8（核心索引结构） 已定项 16 第 1、3 条）才算，两侧都去掉它的记录、数据单元与映射（判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5）；不算的，这条记录放回 inode 记录集合，两侧照比。
    pub(crate) records_of_the_tombstoned_objects: BTreeMap<u64, Vec<u8>>,
}

/// 扫描重建停在条款没写的地方：每个成员说清是哪条条款没定、第一版不认。调用方把 [`ScanRebuildNotDecided::reason`] 报成不适用，
/// 不当成立、也不当违例。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScanRebuildNotDecided {
    /// 码 1 同一对象身份上两版写序逐字节相同：C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 择写序最大者择不出，
    /// 而码 1 的全序键要不要加诞生代号打头停在 C464（I-1.8 对码 1 的全序键写窄了）。
    DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided,
    /// 码 3 同一对象身份上两组 (诞生代号, 实例代号, 事务号) 相同：I-1.8（归并后版本全序） ② 不成立，择新键择不出。
    PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18,
    /// 同一版（归并键相同）的副本头里的载荷校验和不同：I-1.8（归并后版本全序） ① 判损坏、不择。
    CopiesOfOneVersionCarryDifferentPayloadsUnderI18,
    /// 当前版本那片码 3 容器（墓碑、inode、退役、谱系）没有一份副本解得开。
    CurrentPackedContainerDecodesFromNoCopy,
    /// 当前版本的谱系容器里的记录接不成一条父链（有一条 T_old ≥ T_rb，或同一个 T_rb 有两条）：「在谱系上」按 I-9.18（删除队列里的号有谱系上的墓碑）
    /// 的父链定义，父链断了按谱系择现行怎么认没有条款，第一版不定。
    LineageRecordsFormNoParentChain,
    /// 死亡 txg 在谱系上的退役记录点名的 inode 叶容器，择出的那一版诞生代号晚于死亡 txg：退役树说它死了、它在死之后又写过一版，
    /// 两边谁算数没有条款（I-9.34（谱系上的退役记录不指本版活叶） 在遍历方向判它红），第一版不定。
    RetiredLeafContainerHasAVersionBornAfterItsDeath,
    /// 墓碑的区间不是整对象 [0, 2⁶⁴ − 1]：第一版只写整对象（D18（块里携带什么信息） 已定项 24 第 2 条），区间墓碑怎么压 key 没有条款。
    TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,
    /// 两个不同对象身份的当前版本数据单元落在同一个 (inode, 偏移) 上（树 ID 或对象出生代不同）：第一版只有一棵 extent 树，
    /// 按哪一个认没有条款。
    TwoObjectsClaimOneDataUnitPosition,
    /// 同一个 inode 号出现在两片当前版本的 inode 叶容器里（I-9.4（容器号不超最小 key） 管的那一形）：按哪一条认没有条款。
    InodeNumberInTwoCurrentContainers,
}

impl ScanRebuildNotDecided {
    pub(crate) fn reason(self) -> &'static str {
        match self {
            ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided => {
                "扫描重建：码 1 同一对象身份上两版写序逐字节相同，C113 定案 P3 择写序最大者择不出，码 1 的全序键停在 C464（I-1.8 对码 1 的全序键写窄了）"
            }
            ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18 => {
                "扫描重建：码 3 同一对象身份上两组 (诞生代号, 实例代号, 事务号) 相同，I-1.8 ② 不成立，择不出当前版本"
            }
            ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderI18 => {
                "扫描重建：同一版的副本头里的载荷校验和不同，I-1.8 ① 判损坏、不择"
            }
            ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy => {
                "扫描重建：当前版本那片码 3 容器（墓碑、inode、退役、谱系）没有一份副本解得开"
            }
            ScanRebuildNotDecided::LineageRecordsFormNoParentChain => {
                "扫描重建：谱系记录接不成一条父链（有一条 T_old ≥ T_rb，或同一个 T_rb 有两条），父链断了按谱系择现行怎么认没有条款，第一版不定"
            }
            ScanRebuildNotDecided::RetiredLeafContainerHasAVersionBornAfterItsDeath => {
                "扫描重建：谱系上的退役记录点名的 inode 叶容器有一版诞生代号晚于死亡 txg，退役树与那一版谁算数没有条款（I-9.34），第一版不定"
            }
            ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion => {
                "扫描重建：墓碑区间不是整对象 [0, 2⁶⁴ − 1]，第一版只写整对象（D18 已定项 24 第 2 条），区间墓碑怎么压 key 没有条款"
            }
            ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition => {
                "扫描重建：两个不同对象身份的当前版本数据单元落在同一个 (inode, 偏移) 上，按哪一个认没有条款"
            }
            ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers => {
                "扫描重建：同一个 inode 号出现在两片当前版本的 inode 叶容器里，按哪一条认没有条款"
            }
        }
    }
}

/// C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的择新键，按单元类分：码 1 取写序 (实例代号, 事务号) 最大者；
/// 码 3 取 (诞生代号, 实例代号) 最大者，同一 checkpoint 里写过几次的由事务号定——这就是 I-1.8（归并后版本全序） 码 3 的全序键三元，
/// 诞生代号打头。只在同一个对象身份的各版之间比：对象身份首字节是类标签，一个身份上的各版必然同类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CurrentVersionOrderKey {
    DataUnitWriteOrder {
        instance: u32,
        transaction: u64,
    },
    PackedRecordUnit {
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    },
}

fn current_version_order_key_of(
    class: ContentUnitClass,
    plain_header: &[u8],
) -> CurrentVersionOrderKey {
    let offsets = class.header_offsets();
    let instance = read_u32(plain_header, offsets.write_order);
    let transaction = read_six_byte_unsigned(
        plain_header,
        offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET,
    );
    match class {
        ContentUnitClass::Data => CurrentVersionOrderKey::DataUnitWriteOrder {
            instance,
            transaction,
        },
        ContentUnitClass::PackedRecord => CurrentVersionOrderKey::PackedRecordUnit {
            birth_txg: read_u64(plain_header, offsets.birth_txg),
            instance,
            transaction,
        },
    }
}

/// 一个对象身份的一版：归并键相同的副本合成一版（I-1.8（归并后版本全序） 的归并：「类身份段全部字段含写序」去掉载荷校验和）。
struct VersionOfAnObject {
    class: ContentUnitClass,
    /// 头里的载荷校验和：同一版的副本必须相同（I-1.8 ①）。
    payload_checksum: u32,
    /// 第一份副本的明文头（同一版的副本在归并键上逐字节相同，载荷校验和也相同，取哪一份都一样）。
    plain_header: Vec<u8>,
    placements: BTreeSet<UnitPlacement>,
}

/// 一个对象身份的一版，连同它的择新键。
type OrderedVersion = (CurrentVersionOrderKey, VersionOfAnObject);

/// 一个对象身份的当前版本在哪一步择：随现行那一版走的树在第一步就择定；随回退走的那几类要等第二步读出谱系记录、滤掉不在谱系上的再择。
enum VersionsOfAnObject {
    /// 墓碑、退役、谱系三棵随现行那一版走的树的容器，连同实例表的片与未登记的类型：管理员回退不取回退目标那一版的
    /// （`crates/singlefs-core/src/transaction.rs` 的 `TransactionOutput` 那三棵树的字段文档），只按已发布谓词择
    /// （C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 / P3；用户 2026-10-08 定的 X1「退役记录·甲」）。
    ChosenAmongEveryPublishedVersion(VersionOfAnObject),
    /// 码 1 数据单元与 inode 叶容器：管理员回退取回退目标那一版的 extent 树与 inode 树，回退抛弃的那一段里诞生的版本不是任何现行版本的成员。
    /// 各版按择新键升序，第二步只在诞生代号落在谱系上的版本里取最大的。
    LeftForTheLineage(Vec<OrderedVersion>),
}

/// 第一步择出的每个对象身份的各版（随现行那一版走的已择定，随回退走的留着各版等谱系）。
pub(crate) struct VersionsOfEachObject {
    of_each_object: BTreeMap<ObjectIdentity, VersionsOfAnObject>,
}

/// 一个对象身份择当前版本走哪一步：码 1 与 inode 叶容器随回退走，别的随现行那一版走。
fn current_version_is_left_for_the_lineage(class: ContentUnitClass, plain_header: &[u8]) -> bool {
    match class {
        ContentUnitClass::Data => true,
        ContentUnitClass::PackedRecord => match packed_record_type_of(plain_header) {
            PACKED_TYPE_INODE => true,
            PACKED_TYPE_TOMBSTONE
            | PACKED_TYPE_RETIREMENT
            | PACKED_TYPE_LINEAGE
            | PACKED_TYPE_INSTANCE_TABLE => false,
            _unregistered_record_type => false,
        },
    }
}

/// 一个对象身份按择新键升序排好的候选版本里取最大的：最大的两版键相同就择不出（码 1 停在 C464（I-1.8 对码 1 的全序键写窄了），
/// 码 3 是 I-1.8（归并后版本全序） ② 不成立）。一版都没有交回 `None`。各版可以是拿着的，也可以是借来的（第二步滤谱系时借第一步那一份）。
fn latest_of_the_ordered_versions<Version: std::borrow::Borrow<VersionOfAnObject>>(
    mut ordered: Vec<(CurrentVersionOrderKey, Version)>,
) -> Result<Option<Version>, ScanRebuildNotDecided> {
    let Some((current_order_key, current)) = ordered.pop() else {
        return Ok(None);
    };
    if ordered
        .last()
        .is_some_and(|(order_key, _)| *order_key == current_order_key)
    {
        return Err(match current.borrow().class {
            ContentUnitClass::Data => {
                ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided
            }
            ContentUnitClass::PackedRecord => {
                ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18
            }
        });
    }
    Ok(Some(current))
}

/// 一个单元头里的诞生代号（码 1 偏移 75、码 3 偏移 73）。
fn birth_txg_of(version: &VersionOfAnObject) -> u64 {
    read_u64(
        &version.plain_header,
        version.class.header_offsets().birth_txg,
    )
}

/// 谱系记录 24 字节里 T_rb 与 T_old 两段的偏移（`.claude/kb/layout/03-third-txn.md`「三、谱系记录」），按字段表另写一份，不用实现的解析，
/// 也不用 `walk` 里那一份。
const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
const LINEAGE_RECORD_ROLLBACK_TARGET_TXG_OFFSET: usize = 8;

/// 退役记录 34 字节（同一份字段表「五、退役记录」）：被退役身份 26 字节（出生树 8、打包记录类型 2、容器号 8、容器出生代 8），与码 3
/// 类身份段里类标签之后那 26 字节同序同宽；死亡 txg 在 26。
const RETIREMENT_RECORD_RETIRED_IDENTITY_BYTES: usize = 26;
const RETIREMENT_RECORD_RECORD_TYPE_OFFSET: usize = 8;
const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;

/// 墓碑记录 40 字节里死亡 txg 的偏移（同一份字段表「二、墓碑记录」）。
const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;

/// 码 3 对象身份（类身份段 42..69）里类标签之后那一段的起点：退役记录点名的身份从这里起逐字节相等。
const PACKED_OBJECT_IDENTITY_AFTER_THE_CLASS_TAG: usize = 1;

/// 最新那一版谱系上的 txg：父链在 txg 轴上占的几段，每段闭区间。
struct TxgsOnTheLineageOfTheNewestVersion {
    segments: Vec<(u64, u64)>,
}

impl TxgsOnTheLineageOfTheNewestVersion {
    fn holds(&self, txg: u64) -> bool {
        self.segments
            .iter()
            .any(|(lowest_txg, highest_txg)| (*lowest_txg..=*highest_txg).contains(&txg))
    }
}

/// 「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义：从最新那一版起往回走，有谱系记录 (T_rb, T_old) 的那个 txg 的父是 T_old，
/// 其余 txg 的父是 txg − 1。记录按 T_rb 从大到小过一遍：T_rb 大于此刻走到的 txg 的写在被后来的回退抛弃的分支上、跳过；否则
/// [T_rb, 此刻] 整段在链上、跳到 T_old；最后 [0, 此刻] 也在链上。起点取 2⁶⁴ − 1（文件头「读法」那一条：重建不读根，起点取最新根的 txg
/// 认出的是同一批）。
///
/// # Errors
/// 有一条 T_old ≥ T_rb、或同一个 T_rb 两条：接不成一条链，交回 [`ScanRebuildNotDecided::LineageRecordsFormNoParentChain`]。
fn txgs_on_the_lineage_of_the_newest_version(
    lineage_records: &[Vec<u8>],
) -> Result<TxgsOnTheLineageOfTheNewestVersion, ScanRebuildNotDecided> {
    let mut rollbacks_with_the_latest_first: Vec<(u64, u64)> = lineage_records
        .iter()
        .map(|record| {
            (
                read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
                read_u64(record, LINEAGE_RECORD_ROLLBACK_TARGET_TXG_OFFSET),
            )
        })
        .collect();
    rollbacks_with_the_latest_first
        .sort_unstable_by_key(|(rollback_txg, _)| std::cmp::Reverse(*rollback_txg));
    let some_target_is_not_older_than_its_rollback = rollbacks_with_the_latest_first
        .iter()
        .any(|(rollback_txg, rollback_target_txg)| rollback_target_txg >= rollback_txg);
    let some_rollback_has_two_records = rollbacks_with_the_latest_first
        .windows(2)
        .any(|adjacent| adjacent[0].0 == adjacent[1].0);
    if some_target_is_not_older_than_its_rollback || some_rollback_has_two_records {
        return Err(ScanRebuildNotDecided::LineageRecordsFormNoParentChain);
    }
    let mut segments = Vec::new();
    let mut txg_walked_to = u64::MAX;
    // 迭代上界是谱系记录条数；跨轮携带的是此刻走到的 txg，每接上一段就跳到更小的 T_old（上面判过 T_old < T_rb ≤ 此刻），只减不增。
    for (rollback_txg, rollback_target_txg) in rollbacks_with_the_latest_first {
        if rollback_txg > txg_walked_to {
            continue;
        }
        segments.push((rollback_txg, txg_walked_to));
        txg_walked_to = rollback_target_txg;
    }
    segments.push((0, txg_walked_to));
    Ok(TxgsOnTheLineageOfTheNewestVersion { segments })
}

/// 记录要读的那几类码 3 容器：墓碑判 key 死亡、inode 记录是第三张表也是数据单元认不认领的依据、谱系记录定哪些版本在谱系上、
/// 退役记录定哪些叶容器不认领。
fn packed_record_type_whose_records_are_read(record_type: u16) -> Option<usize> {
    let record_bytes = match record_type {
        PACKED_TYPE_TOMBSTONE => TOMBSTONE_RECORD_BYTES,
        PACKED_TYPE_INODE => INODE_RECORD_BYTES,
        PACKED_TYPE_RETIREMENT => RETIREMENT_RECORD_BYTES,
        PACKED_TYPE_LINEAGE => LINEAGE_RECORD_BYTES,
        _instance_table_or_unregistered => return None,
    };
    Some(usize::try_from(record_bytes).expect("记录宽装得进 usize"))
}

fn packed_record_type_of(plain_header: &[u8]) -> u16 {
    let offset = ContentUnitClass::PackedRecord
        .header_offsets()
        .packed_record_type
        .expect("码 3 的头偏移表里有打包记录类型那一段");
    read_u16(plain_header, offset)
}

impl VersionsOfEachObject {
    /// 要读记录的那几类码 3 容器（墓碑 1、inode 2、退役 3、谱系 5），连同扫到它的副本的落点：调用方把每一份的整单元字节读来，
    /// 按对象身份交给 [`rebuild_index_tables`]。随现行那一版走的给择定那一版的落点；inode 叶容器的当前版本要等谱系，给它每一版的落点，
    /// 第二步按择出那一版的明文头认副本。实例表的片不读（它不进映射，已发布谓词由调用方读最新根那一份）。
    pub(crate) fn packed_containers_whose_records_are_read(
        &self,
    ) -> Vec<(ObjectIdentity, BTreeSet<UnitPlacement>)> {
        let mut containers = Vec::new();
        for (identity, versions) in &self.of_each_object {
            let placements: BTreeSet<UnitPlacement> = match versions {
                VersionsOfAnObject::ChosenAmongEveryPublishedVersion(version) => {
                    if version.class != ContentUnitClass::PackedRecord
                        || packed_record_type_whose_records_are_read(packed_record_type_of(
                            &version.plain_header,
                        ))
                        .is_none()
                    {
                        continue;
                    }
                    version.placements.clone()
                }
                VersionsOfAnObject::LeftForTheLineage(ordered) => {
                    if ordered
                        .iter()
                        .all(|(_, version)| version.class != ContentUnitClass::PackedRecord)
                    {
                        continue;
                    }
                    ordered
                        .iter()
                        .flat_map(|(_, version)| version.placements.iter().copied())
                        .collect()
                }
            };
            containers.push((identity.clone(), placements));
        }
        containers
    }
}

/// 第一步：从扫出的已发布副本归并出每个对象身份的各版——同对象身份、同归并键的副本合成一版，各版按 [`CurrentVersionOrderKey`] 排好；
/// 随现行那一版走的树的容器在这一步就取最大的那一版（[`VersionsOfAnObject`]），码 1 与 inode 叶容器留到 [`rebuild_index_tables`]
/// 读出谱系记录之后再择。
///
/// 已发布谓词的输入（挂载根的实例代号与 txg、它指着的那一版实例表的行）是扫描重建允许读根的唯一一处：调用方读最新根那一份、
/// 照 I-1.8（归并后版本全序） 那一遍同一来源滤好再交进来；这里不读根环、不读树表、不读任何树节点（码 2）。
///
/// # Errors
/// 择不出或择了也不作数时交回 [`ScanRebuildNotDecided`]：同一版副本载荷校验和不同；随现行那一版走的容器两组全序键相同。
///
/// # Panics
/// 副本的类标签不是 1 / 3、或明文头短于这一类的明文头末尾：调用方（`walk` 的扫描方向）只交它按类标签与头校验和判过的副本，
/// 到这里还不对就是那一步被跳过了。
pub(crate) fn choose_current_versions(
    copies: &[PublishedUnitCopy],
) -> Result<VersionsOfEachObject, ScanRebuildNotDecided> {
    let mut versions_of_each_object: BTreeMap<
        ObjectIdentity,
        BTreeMap<Vec<u8>, VersionOfAnObject>,
    > = BTreeMap::new();
    for copy in copies {
        let class = ContentUnitClass::of(copy.plain_header[6])
            .expect("调用方只交码 1 / 码 3 的副本：扫描方向按类标签滤过");
        let offsets = class.header_offsets();
        assert!(
            copy.plain_header.len() >= offsets.plain_header_end,
            "调用方交的明文头到这一类的明文头末尾：扫描方向判头校验和时读过这一段"
        );
        let merge_key = [
            &copy.plain_header[offsets.merge_key_before_payload_checksum.clone()],
            &copy.plain_header[offsets.merge_key_after_payload_checksum.clone()],
        ]
        .concat();
        let payload_checksum = read_u32(&copy.plain_header, offsets.payload_checksum);
        let version = versions_of_each_object
            .entry(ObjectIdentity(
                copy.plain_header[offsets.object_key.clone()].to_vec(),
            ))
            .or_default()
            .entry(merge_key)
            .or_insert_with(|| VersionOfAnObject {
                class,
                payload_checksum,
                plain_header: copy.plain_header[..offsets.plain_header_end].to_vec(),
                placements: BTreeSet::new(),
            });
        if version.payload_checksum != payload_checksum {
            return Err(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderI18);
        }
        version.placements.insert(copy.placement);
    }
    let mut of_each_object = BTreeMap::new();
    for (identity, versions) in versions_of_each_object {
        let mut ordered: Vec<OrderedVersion> = versions
            .into_values()
            .map(|version| {
                (
                    current_version_order_key_of(version.class, &version.plain_header),
                    version,
                )
            })
            .collect();
        ordered.sort_by_key(|(order_key, _)| *order_key);
        let (_, any_version) = ordered
            .first()
            .expect("一个对象身份是从至少一份副本建出来的");
        // 一个对象身份上的各版同类、同打包记录类型（对象身份首字节是类标签，码 3 的身份里带打包记录类型），看哪一版都一样。
        let versions_of_this_object = if current_version_is_left_for_the_lineage(
            any_version.class,
            &any_version.plain_header,
        ) {
            VersionsOfAnObject::LeftForTheLineage(ordered)
        } else {
            VersionsOfAnObject::ChosenAmongEveryPublishedVersion(
                latest_of_the_ordered_versions(ordered)?
                    .expect("一个对象身份是从至少一份副本建出来的"),
            )
        };
        of_each_object.insert(identity, versions_of_this_object);
    }
    Ok(VersionsOfEachObject { of_each_object })
}

/// 一个当前版本的映射 key，从它的明文头现算（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的 key 规则：
/// `crates/singlefs-core/src/records.rs` 的 `mapping_key_for_data` 与 `mapping_key_for_node` 读的就是这几段，这里按字段表另写一份）。
fn central_mapping_key_of_the_header(
    class: ContentUnitClass,
    plain_header: &[u8],
) -> CentralMappingKey {
    let offsets = class.header_offsets();
    let mut key = vec![0u8; usize::try_from(MAPPING_KEY_BYTES).expect("27")];
    key[MAPPING_KEY_UNIT_CLASS_OFFSET] = plain_header[6];
    let birth_tree_bytes = MAPPING_KEY_BIRTH_TXG_OFFSET - MAPPING_KEY_BIRTH_TREE_OFFSET;
    key[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TXG_OFFSET]
        .copy_from_slice(&plain_header[offsets.birth_tree..offsets.birth_tree + birth_tree_bytes]);
    let birth_txg_bytes = MAPPING_KEY_INSTANCE_OFFSET - MAPPING_KEY_BIRTH_TXG_OFFSET;
    key[MAPPING_KEY_BIRTH_TXG_OFFSET..MAPPING_KEY_INSTANCE_OFFSET]
        .copy_from_slice(&plain_header[offsets.birth_txg..offsets.birth_txg + birth_txg_bytes]);
    let instance_bytes = MAPPING_KEY_TAIL_OFFSET - MAPPING_KEY_INSTANCE_OFFSET;
    key[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_TAIL_OFFSET]
        .copy_from_slice(&plain_header[offsets.write_order..offsets.write_order + instance_bytes]);
    match class {
        ContentUnitClass::Data => {
            let transaction_start = offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET;
            let tail_bytes = key.len() - MAPPING_KEY_TAIL_OFFSET;
            key[MAPPING_KEY_TAIL_OFFSET..]
                .copy_from_slice(&plain_header[transaction_start..transaction_start + tail_bytes]);
        }
        ContentUnitClass::PackedRecord => {
            let birth_sequence = offsets
                .birth_sequence
                .expect("码 3 的头偏移表里有出生序号那一段");
            let birth_sequence_bytes = MAPPING_KEY_TAIL_PADDING.start - MAPPING_KEY_TAIL_OFFSET;
            key[MAPPING_KEY_TAIL_OFFSET..MAPPING_KEY_TAIL_PADDING.start].copy_from_slice(
                &plain_header[birth_sequence..birth_sequence + birth_sequence_bytes],
            );
        }
    }
    CentralMappingKey(key)
}

/// 当前版本那片码 3 容器的记录：从调用方读来的副本里取第一份解得开、头与择出的那一版相同、记录类型与记录宽对得上的。
fn records_of_a_current_container(
    version: &VersionOfAnObject,
    copies: &[Vec<u8>],
    record_type: u16,
    record_bytes: usize,
) -> Result<Vec<Vec<u8>>, ScanRebuildNotDecided> {
    copies
        .iter()
        .filter(|bytes| bytes.get(..version.plain_header.len()) == Some(&version.plain_header[..]))
        .find_map(|bytes| {
            packed_unit_view(bytes)
                .ok()
                .filter(|view| view.record_type == record_type && view.record_width == record_bytes)
        })
        .map(|view| view.records)
        .ok_or(ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy)
}

/// 读法 Z 加写序的第三条（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K6，用户 2026-10-16 定）要的那张表：
/// 每个 (inode, 对象出生代) 的当前版本数据单元里，每个偏移之下（偏移严格更小）那些单元的最大出生 txg。按偏移升序扫一遍，
/// 同一个偏移上有几版（树 ID 不同）的先都记下、再并进「之下」那一份。
fn newest_birth_txg_below_each_data_unit(
    current_data_units: &[&VersionOfAnObject],
) -> BTreeMap<(u64, u64, u64), u64> {
    let mut birth_txgs_of_each_object: BTreeMap<(u64, u64), BTreeMap<u64, u64>> = BTreeMap::new();
    for version in current_data_units {
        let header = &version.plain_header;
        let newest_at_the_offset = birth_txgs_of_each_object
            .entry((
                read_u64(header, DATA_UNIT_OBJECT_OFFSET),
                read_u64(header, DATA_UNIT_OBJECT_BIRTH_OFFSET),
            ))
            .or_default()
            .entry(read_u64(header, DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET))
            .or_insert(0);
        *newest_at_the_offset = (*newest_at_the_offset).max(birth_txg_of(version));
    }
    let mut newest_below = BTreeMap::new();
    for ((inode, object_birth), birth_txg_at_each_offset) in birth_txgs_of_each_object {
        let mut newest_so_far: Option<u64> = None;
        for (offset_in_bytes, birth_txg) in birth_txg_at_each_offset {
            if let Some(newest) = newest_so_far {
                newest_below.insert((inode, object_birth, offset_in_bytes), newest);
            }
            newest_so_far = Some(newest_so_far.map_or(birth_txg, |newest| newest.max(birth_txg)));
        }
    }
    newest_below
}

/// 第二步：从第一步归并出的各版与那几片容器的整单元字节现算三张表。
///
/// 认领的规矩，逐条有出处：数据单元按对象身份择当前版本（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）；
/// 有同一对象出生代的 inode 记录时按读法 Z 加写序（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K6，用户 2026-10-16 定，
/// D18（块里携带什么信息） 已定项 10）判死活，三条都满足才认领：对象出生代与记录相同；偏移在 ⌈size ÷ 净荷⌉.max(1) 个单元之内
/// （size 之外的不认领：截断中间态里还挂着的、覆盖写变短与截断放掉而还在盘上的）；同一 (inode, 对象出生代) 的当前版本里没有偏移更小、
/// 出生 txg 更大的单元（变长留洞之后、洞里放掉而还在盘上的旧单元由它判死）。前提是活单元的出生 txg 随偏移不降：今天的写入口
/// 都从偏移 0 整份写、截断只重写最后留下的单元、变长只重写最后一个存在的单元；出现「写到偏移 o 而不重写 o 之后的单元」的入口
/// （pwrite、punch）时这一条要重开。三条之外再合取第四条（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R3、R4）：
/// 记录偏移 120 的截断字段 T ≠ 0 时，单元号不小于 ⌈N_w ÷ 净荷⌉.max(1)（N_w 在偏移 128）且单元头出生 txg 早于 T 的不认领。
/// 没有同一对象出生代的记录的单元照旧认领（见文件头「没有 inode 记录也没有墓碑压着的码 1 单元照认领」；判决 m4-trunc-r1 第六节 S3）；
/// 码 1 与 inode 叶容器只在诞生代号落在谱系上的版本里择（谱系记录按「不是任何现行版本的成员」认领，见文件头）；
/// 墓碑判 key 死亡（D11（索引节点要不要留消息缓冲区） 已定项 4）——第一版墓碑只写整对象，压住 (对象, 对象出生代) 的全部单元，
/// 只认死亡 txg 在谱系上的；死亡 txg 在谱系上的退役记录点名的 inode 叶容器不认领（退役记录按退役树认领，见文件头）；
/// 码 3 容器除实例表的片之外都进映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12 只豁免映射树自己、树表与实例表）。
/// 条款没写的几格交回 [`ScanRebuildNotDecided`]，见各成员。
///
/// # Errors
/// 见 [`ScanRebuildNotDecided`] 各成员；同一版副本载荷校验和不同、随现行那一版走的容器两组全序键相同这两格在第一步就交回了。
///
/// # Panics
/// 第一步择定的里有 inode 叶容器或码 1：[`current_version_is_left_for_the_lineage`] 把它们一律留给这一步，走到就是第一步分错了。
pub(crate) fn rebuild_index_tables(
    versions: &VersionsOfEachObject,
    container_copies: &BTreeMap<ObjectIdentity, Vec<Vec<u8>>>,
) -> Result<RebuiltIndexTables, ScanRebuildNotDecided> {
    let copies_of = |identity: &ObjectIdentity| {
        container_copies
            .get(identity)
            .map_or(&[][..], Vec::as_slice)
    };
    let mut central_mapping: BTreeMap<CentralMappingKey, BTreeSet<UnitPlacement>> = BTreeMap::new();
    let mut lineage_records: Vec<Vec<u8>> = Vec::new();
    let mut retirement_records: Vec<Vec<u8>> = Vec::new();
    let mut tombstone_records: Vec<Vec<u8>> = Vec::new();
    for (identity, versions_of_this_object) in &versions.of_each_object {
        let VersionsOfAnObject::ChosenAmongEveryPublishedVersion(version) = versions_of_this_object
        else {
            continue;
        };
        let record_type = packed_record_type_of(&version.plain_header);
        if record_type != PACKED_TYPE_INSTANCE_TABLE {
            central_mapping.insert(
                central_mapping_key_of_the_header(version.class, &version.plain_header),
                version.placements.clone(),
            );
        }
        let Some(record_bytes) = packed_record_type_whose_records_are_read(record_type) else {
            continue;
        };
        let records = records_of_a_current_container(
            version,
            copies_of(identity),
            record_type,
            record_bytes,
        )?;
        let records_of_this_tree = match record_type {
            PACKED_TYPE_LINEAGE => &mut lineage_records,
            PACKED_TYPE_RETIREMENT => &mut retirement_records,
            PACKED_TYPE_TOMBSTONE => &mut tombstone_records,
            _inode_container_left_for_the_lineage => unreachable!(
                "第一步择定的只有随现行那一版走的树的容器：inode 叶容器留给谱系那一步（current_version_is_left_for_the_lineage）"
            ),
        };
        records_of_this_tree.extend(records);
    }
    let txgs_on_the_lineage = txgs_on_the_lineage_of_the_newest_version(&lineage_records)?;
    // 被退役身份（类标签之后那 26 字节）→ 谱系上最早的那个死亡 txg。
    let mut retired_leaf_containers: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
    for record in &retirement_records {
        let death_txg = read_u64(record, RETIREMENT_RECORD_DEATH_TXG_OFFSET);
        if read_u16(record, RETIREMENT_RECORD_RECORD_TYPE_OFFSET) != PACKED_TYPE_INODE
            || !txgs_on_the_lineage.holds(death_txg)
        {
            continue;
        }
        retired_leaf_containers
            .entry(record[..RETIREMENT_RECORD_RETIRED_IDENTITY_BYTES].to_vec())
            .and_modify(|earliest_death_txg| {
                *earliest_death_txg = (*earliest_death_txg).min(death_txg);
            })
            .or_insert(death_txg);
    }
    let mut tombstoned_objects: BTreeSet<(u64, u64)> = BTreeSet::new();
    for record in &tombstone_records {
        if read_u64(record, TOMBSTONE_RECORD_RANGE_START_OFFSET)
            != TOMBSTONE_WHOLE_OBJECT_RANGE_START
            || read_u64(record, TOMBSTONE_RECORD_RANGE_END_OFFSET)
                != TOMBSTONE_WHOLE_OBJECT_RANGE_END
        {
            return Err(
                ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,
            );
        }
        if txgs_on_the_lineage.holds(read_u64(record, TOMBSTONE_RECORD_DEATH_TXG_OFFSET)) {
            tombstoned_objects.insert((
                read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
                read_u64(record, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
            ));
        }
    }
    let inode_record_bytes = packed_record_type_whose_records_are_read(PACKED_TYPE_INODE)
        .expect("inode 叶容器的记录要读");
    let mut inode_records: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut current_data_units: Vec<&VersionOfAnObject> = Vec::new();
    for (identity, versions_of_this_object) in &versions.of_each_object {
        let VersionsOfAnObject::LeftForTheLineage(ordered) = versions_of_this_object else {
            continue;
        };
        let born_on_the_lineage: Vec<(CurrentVersionOrderKey, &VersionOfAnObject)> = ordered
            .iter()
            .filter(|(_, version)| txgs_on_the_lineage.holds(birth_txg_of(version)))
            .map(|(order_key, version)| (*order_key, version))
            .collect();
        let Some(version) = latest_of_the_ordered_versions(born_on_the_lineage)? else {
            continue;
        };
        match version.class {
            ContentUnitClass::Data => current_data_units.push(version),
            ContentUnitClass::PackedRecord => {
                if let Some(death_txg) = retired_leaf_containers
                    .get(&identity.0[PACKED_OBJECT_IDENTITY_AFTER_THE_CLASS_TAG..])
                {
                    if birth_txg_of(version) > *death_txg {
                        return Err(
                            ScanRebuildNotDecided::RetiredLeafContainerHasAVersionBornAfterItsDeath,
                        );
                    }
                    continue;
                }
                central_mapping.insert(
                    central_mapping_key_of_the_header(version.class, &version.plain_header),
                    version.placements.clone(),
                );
                let records = records_of_a_current_container(
                    version,
                    copies_of(identity),
                    PACKED_TYPE_INODE,
                    inode_record_bytes,
                )?;
                for record in records {
                    if inode_records.insert(read_u64(&record, 0), record).is_some() {
                        return Err(ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers);
                    }
                }
            }
        }
    }
    // 墓碑压着、记录还在的对象（谱系上有墓碑）：记录先放一边、不进这一侧的 inode 记录集合，算不算删除中由比对那一侧两侧一起认
    // （判决 m4-c860-r1 第三节 C2，见 `RebuiltIndexTables::records_of_the_tombstoned_objects`）；它的数据单元下面按墓碑不认领。
    let tombstoned_inodes_that_still_have_a_record: Vec<u64> = inode_records
        .iter()
        .filter(|(inode, record)| {
            tombstoned_objects
                .contains(&(**inode, read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET)))
        })
        .map(|(inode, _)| *inode)
        .collect();
    let mut records_of_the_tombstoned_objects: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    for inode in tombstoned_inodes_that_still_have_a_record {
        let record = inode_records
            .remove(&inode)
            .expect("这个号是上面刚从 inode_records 里挑出来的");
        records_of_the_tombstoned_objects.insert(inode, record);
    }
    let payload_capacity_in_bytes = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET;
    let newest_birth_txg_below = newest_birth_txg_below_each_data_unit(&current_data_units);
    let mut data_units_of_each_inode: BTreeMap<DataUnitPosition, CentralMappingKey> =
        BTreeMap::new();
    for version in current_data_units {
        let header = &version.plain_header;
        let inode = read_u64(header, DATA_UNIT_OBJECT_OFFSET);
        let object_birth = read_u64(header, DATA_UNIT_OBJECT_BIRTH_OFFSET);
        let offset_in_bytes = read_u64(header, DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET);
        let record_of_this_object = inode_records
            .get(&inode)
            .filter(|record| read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET) == object_birth);
        if tombstoned_objects.contains(&(inode, object_birth)) {
            continue;
        }
        if let Some(record) = record_of_this_object {
            // 一条记录罩几个数据单元：size 为 0 时 1 个、否则 ⌈size ÷ 净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs`
            // 的 `data_unit_count_of_a_sequential_write` 里 `div_ceil(...).max(1)`，I-4.6（中间态其余不变量成立） 那一行引的同一个式子）。
            let data_units_the_size_covers = read_u64(record, INODE_RECORD_SIZE_OFFSET)
                .div_ceil(payload_capacity_in_bytes)
                .max(1);
            if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {
                continue;
            }
            if newest_birth_txg_below
                .get(&(inode, object_birth, offset_in_bytes))
                .is_some_and(|newest_below| *newest_below > birth_txg_of(version))
            {
                continue;
            }
            // 读法 Z 第四条（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R3、R4，与写序那一条是合取）：记录的截断字段
            // T ≠ 0 时，单元号不小于 ⌈N_w ÷ 净荷容量⌉.max(1)、单元头出生 txg 早于 T 的是那次截断放掉而还在盘上的旧单元，不认领。
            // 截在单元边界上再变长时这几个旧单元与留下的单元同一个出生 txg，写序那一条判不死它们。
            let truncation_field_txg = read_u64(record, INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET);
            let data_units_the_truncation_field_keeps =
                read_u64(record, INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET)
                    .div_ceil(payload_capacity_in_bytes)
                    .max(1);
            if truncation_field_txg != 0
                && offset_in_bytes / payload_capacity_in_bytes
                    >= data_units_the_truncation_field_keeps
                && birth_txg_of(version) < truncation_field_txg
            {
                continue;
            }
        }
        let key = central_mapping_key_of_the_header(version.class, header);
        let position = DataUnitPosition {
            inode,
            offset_in_bytes,
        };
        if data_units_of_each_inode
            .insert(position, key.clone())
            .is_some()
        {
            return Err(ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition);
        }
        central_mapping.insert(key, version.placements.clone());
    }
    Ok(RebuiltIndexTables {
        central_mapping,
        data_units_of_each_inode,
        inode_records,
        records_of_the_tombstoned_objects,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        choose_current_versions, rebuild_index_tables, DataUnitPosition, ObjectIdentity,
        PublishedUnitCopy, RebuiltIndexTables, ScanRebuildNotDecided, UnitPlacement,
        VersionsOfEachObject,
    };
    use crate::{
        crc32_castagnoli_bitwise, PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE, PACKED_TYPE_RETIREMENT,
        PACKED_TYPE_TOMBSTONE,
    };

    /// 这个模块自己的源码：白名单单测读它的 `use` 清单与测试模块之前那一段里的路径。
    const SOURCE_OF_THE_REBUILD_MODULE: &str = include_str!("rebuild.rs");

    /// 扫描重建许用的每一项（C410（checker 两个方向共享什么没有检查） 的会红形态：两个方向共享的只有格式解析，D13（验证路线） 已定项 12）：
    /// 标准库的两个容器、`singlefs_format` 的常量、crate 根上解单元头与码 3 记录的函数与偏移。`walk` 模块与它的节点缓存一项都不许有。
    const ITEMS_THE_REBUILD_MAY_USE: &[&str] = &[
        "std::collections::BTreeMap",
        "std::collections::BTreeSet",
        "singlefs_format::DATA_UNIT_BYTES",
        "singlefs_format::DATA_UNIT_PAYLOAD_OFFSET",
        "singlefs_format::INODE_RECORD_BYTES",
        "singlefs_format::LINEAGE_RECORD_BYTES",
        "singlefs_format::MAPPING_KEY_BYTES",
        "singlefs_format::RETIREMENT_RECORD_BYTES",
        "singlefs_format::TOMBSTONE_RECORD_BYTES",
        "crate::packed_unit_view",
        "crate::read_six_byte_unsigned",
        "crate::read_u16",
        "crate::read_u32",
        "crate::read_u64",
        "crate::ContentUnitClass",
        "crate::DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET",
        "crate::DATA_UNIT_OBJECT_BIRTH_OFFSET",
        "crate::DATA_UNIT_OBJECT_OFFSET",
        "crate::INODE_RECORD_OBJECT_BIRTH_OFFSET",
        "crate::INODE_RECORD_SIZE_OFFSET",
        "crate::INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET",
        "crate::INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET",
        "crate::MAPPING_KEY_BIRTH_TREE_OFFSET",
        "crate::MAPPING_KEY_BIRTH_TXG_OFFSET",
        "crate::MAPPING_KEY_INSTANCE_OFFSET",
        "crate::MAPPING_KEY_TAIL_OFFSET",
        "crate::MAPPING_KEY_TAIL_PADDING",
        "crate::MAPPING_KEY_UNIT_CLASS_OFFSET",
        "crate::PACKED_TYPE_INODE",
        "crate::PACKED_TYPE_INSTANCE_TABLE",
        "crate::PACKED_TYPE_LINEAGE",
        "crate::PACKED_TYPE_RETIREMENT",
        "crate::PACKED_TYPE_TOMBSTONE",
        "crate::TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET",
        "crate::TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET",
        "crate::TOMBSTONE_RECORD_RANGE_END_OFFSET",
        "crate::TOMBSTONE_RECORD_RANGE_START_OFFSET",
        "crate::TOMBSTONE_WHOLE_OBJECT_RANGE_END",
        "crate::TOMBSTONE_WHOLE_OBJECT_RANGE_START",
        "crate::WRITE_ORDER_TRANSACTION_OFFSET",
    ];

    /// 测试模块那一行属性：一整行恰是它（行首起、行尾止，前后不带别的字）才算切点。
    const TEST_MODULE_ATTRIBUTE_LINE: &str = "#[cfg(test)]";

    /// 一份源码里测试模块那一行属性之前的部分：切在恰是 [`TEST_MODULE_ATTRIBUTE_LINE`] 的那一行（注释、字符串里带这几个字的行不算）。
    /// 恰是它的行要恰好一行：第二行这样的属性会把它之后的非测试代码也切进「测试模块」里，那样判红，不悄悄少扫一段。
    fn source_before_the_test_module_of(source: &str) -> &str {
        let mut line_start = 0;
        let mut cut = None;
        let mut attribute_lines = 0;
        for line in source.split_inclusive('\n') {
            if line.strip_suffix('\n').unwrap_or(line) == TEST_MODULE_ATTRIBUTE_LINE {
                attribute_lines += 1;
                cut.get_or_insert(line_start);
            }
            line_start += line.len();
        }
        assert_eq!(
            attribute_lines, 1,
            "源码里恰是 `{TEST_MODULE_ATTRIBUTE_LINE}` 的行要恰好一行（测试模块那一行），这里有 {attribute_lines} 行"
        );
        &source[..cut.expect("恰好一行，上面断言过")]
    }

    /// 这个模块测试模块之前的源码：重建模块本身的代码（测试里引了谁不算重建模块调了谁）。
    fn source_before_the_test_module() -> &'static str {
        source_before_the_test_module_of(SOURCE_OF_THE_REBUILD_MODULE)
    }

    /// 一段 Rust 源码剥掉 `//` 注释（连同文档注释）、`/* */` 注释与字符串、字符字面量之后的样子：剥掉的字换成空格，换行留着。
    /// 字符串认普通的（带 `\` 转义）与原始的（`r"…"`、`r#"…"#`，前面可带 `b`）；`'` 后面跟一个字再跟 `'`、或跟 `\` 的是字符字面量，
    /// 别的是生命周期，照留。
    fn source_without_comments_and_string_literals(source: &str) -> String {
        let characters: Vec<char> = source.chars().collect();
        let mut kept = String::with_capacity(source.len());
        let blank = |character: char| if character == '\n' { '\n' } else { ' ' };
        let mut position = 0;
        while position < characters.len() {
            let character = characters[position];
            let next = characters.get(position + 1).copied();
            // `r` / `br` 前面紧挨着标识符的字（`for`、`bar"`）不是原始字符串的前缀
            let is_not_preceded_by_an_identifier_character = position == 0
                || !(characters[position - 1].is_alphanumeric() || characters[position - 1] == '_');
            if character == '/' && next == Some('/') {
                while position < characters.len() && characters[position] != '\n' {
                    kept.push(' ');
                    position += 1;
                }
            } else if character == '/' && next == Some('*') {
                let mut depth = 0usize;
                loop {
                    if position >= characters.len() {
                        break;
                    }
                    if characters[position] == '/' && characters.get(position + 1) == Some(&'*') {
                        depth += 1;
                        kept.push_str("  ");
                        position += 2;
                    } else if characters[position] == '*'
                        && characters.get(position + 1) == Some(&'/')
                    {
                        depth -= 1;
                        kept.push_str("  ");
                        position += 2;
                        if depth == 0 {
                            break;
                        }
                    } else {
                        kept.push(blank(characters[position]));
                        position += 1;
                    }
                }
            } else if is_not_preceded_by_an_identifier_character
                && (character == 'r' || (character == 'b' && next == Some('r')))
                && {
                    let after_prefix = position + if character == 'b' { 2 } else { 1 };
                    let hashes = characters[after_prefix..]
                        .iter()
                        .take_while(|candidate| **candidate == '#')
                        .count();
                    characters.get(after_prefix + hashes) == Some(&'"')
                }
            {
                let after_prefix = position + if character == 'b' { 2 } else { 1 };
                let hashes = characters[after_prefix..]
                    .iter()
                    .take_while(|candidate| **candidate == '#')
                    .count();
                let body_start = after_prefix + hashes + 1;
                let mut end = body_start;
                while end < characters.len() {
                    if characters[end] == '"'
                        && characters[end + 1..]
                            .iter()
                            .take(hashes)
                            .filter(|candidate| **candidate == '#')
                            .count()
                            == hashes
                    {
                        break;
                    }
                    end += 1;
                }
                let stop = (end + 1 + hashes).min(characters.len());
                for skipped in &characters[position..stop] {
                    kept.push(blank(*skipped));
                }
                position = stop;
            } else if character == '"' {
                kept.push(' ');
                position += 1;
                while position < characters.len() && characters[position] != '"' {
                    let step = if characters[position] == '\\' { 2 } else { 1 };
                    for skipped in characters.iter().skip(position).take(step) {
                        kept.push(blank(*skipped));
                    }
                    position += step;
                }
                if position < characters.len() {
                    kept.push(' ');
                    position += 1;
                }
            } else if character == '\'' && next == Some('\\') {
                kept.push_str("  ");
                position += 2;
                while position < characters.len() && characters[position] != '\'' {
                    kept.push(' ');
                    position += 1;
                }
                if position < characters.len() {
                    kept.push(' ');
                    position += 1;
                }
            } else if character == '\'' && characters.get(position + 2) == Some(&'\'') {
                kept.push_str("   ");
                position += 3;
            } else {
                kept.push(character);
                position += 1;
            }
        }
        kept
    }

    /// 一段非测试源码里的每条 `use` 声明展开成一项一个（先剥注释与字面量；只认一层花括号，嵌套判红，免得白名单被一种认不出的写法绕过去）。
    fn items_used_by(source_before_the_test_module: &str) -> Vec<String> {
        let mut items = Vec::new();
        for statement in
            source_without_comments_and_string_literals(source_before_the_test_module).split(';')
        {
            let code = statement.trim_start();
            let Some(path) = ["use ", "pub use ", "pub(crate) use "]
                .iter()
                .find_map(|keyword| code.strip_prefix(keyword))
            else {
                continue;
            };
            let path: String = path.split_whitespace().collect();
            match path.split_once("::{") {
                Some((prefix, group)) => {
                    let group = group.strip_suffix('}').expect("花括号收尾");
                    assert!(
                        !group.contains('{'),
                        "use 声明里的花括号嵌套了一层以上（{path}）：白名单单测只认一层，改写成一层"
                    );
                    items.extend(
                        group
                            .split(',')
                            .filter(|item| !item.is_empty())
                            .map(|item| format!("{prefix}::{item}")),
                    );
                }
                None => items.push(path),
            }
        }
        items
    }

    /// 一段非测试源码的 `use` 清单与白名单逐项比：(白名单之外的, 白名单里没用上的)。
    fn use_list_differences_from_the_whitelist(
        source_before_the_test_module: &str,
    ) -> (Vec<String>, Vec<String>) {
        let used: BTreeSet<String> = items_used_by(source_before_the_test_module)
            .into_iter()
            .collect();
        let allowed: BTreeSet<String> = ITEMS_THE_REBUILD_MAY_USE
            .iter()
            .map(|item| (*item).to_string())
            .collect();
        (
            used.difference(&allowed).cloned().collect(),
            allowed.difference(&used).cloned().collect(),
        )
    }

    /// 一段非测试源码（剥掉注释与字面量之后）里按路径指名的、白名单不放行的东西：`super::` 路径、白名单之外的 `crate::` 一项、
    /// `walk::` 路径与 `check_pool_image` 这个名字（不论经哪条路径）。
    fn names_outside_the_whitelist_in(source_before_the_test_module: &str) -> Vec<String> {
        let code = source_without_comments_and_string_literals(source_before_the_test_module);
        let mut named_outside_the_whitelist = Vec::new();
        if code.contains("super::") {
            named_outside_the_whitelist.push("super::".to_string());
        }
        if code.contains("walk::") {
            named_outside_the_whitelist.push("walk::".to_string());
        }
        if code.contains("check_pool_image") {
            named_outside_the_whitelist.push("check_pool_image".to_string());
        }
        let allowed_crate_items: BTreeSet<&str> = ITEMS_THE_REBUILD_MAY_USE
            .iter()
            .filter_map(|item| item.strip_prefix("crate::"))
            .collect();
        for (index, _) in code.match_indices("crate::") {
            let item: String = code[index + "crate::".len()..]
                .chars()
                .take_while(|character| character.is_alphanumeric() || *character == '_')
                .collect();
            // 后面跟花括号的是 use 清单那一组（use 清单那一条逐项比）。
            if !item.is_empty() && !allowed_crate_items.contains(item.as_str()) {
                named_outside_the_whitelist.push(format!("crate::{item}"));
            }
        }
        named_outside_the_whitelist
    }

    /// 攻方 W1 的绕过（m4-batch2-code-r1 判决 S8）：`use` 清单之后一行注释里带测试模块那一行属性的字样，下一行再引走树的函数。
    /// 拼成一行字符串（`\n` 换行），本文件里没有一行恰是那一行属性。
    const SOURCE_WITH_A_WALK_IMPORT_BEHIND_A_COMMENT_NAMING_THE_TEST_ATTRIBUTE: &str = "use std::collections::BTreeMap;\n// 测试模块从 #[cfg(test)] 那一行起\nuse crate::walk::check_pool_image;\n\nfn rebuild() {}\n\n#[cfg(test)]\nmod tests {}\n";

    #[test]
    fn the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist() {
        let (outside_the_whitelist, whitelisted_but_unused) =
            use_list_differences_from_the_whitelist(source_before_the_test_module());
        assert!(
            outside_the_whitelist.is_empty() && whitelisted_but_unused.is_empty(),
            "扫描重建的 use 清单与白名单逐项不同：白名单之外的 {outside_the_whitelist:?}、白名单里没用上的 {whitelisted_but_unused:?}（两个方向共享的只有格式解析，D13 已定项 12）"
        );
    }

    #[test]
    fn the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path() {
        let named_outside_the_whitelist =
            names_outside_the_whitelist_in(source_before_the_test_module());
        assert!(
            named_outside_the_whitelist.is_empty(),
            "扫描重建的非测试代码按路径指名了白名单不放行的 {named_outside_the_whitelist:?}：不许调走树的函数（walk 模块与它的节点缓存），只许经 use 清单引白名单里的项"
        );
    }

    /// W1：注释里带测试模块那一行属性的字样不是切点，切在恰是那一行的地方；它后面那条 `use` 算进非测试代码、判在白名单之外。
    #[test]
    fn a_walk_import_behind_a_comment_naming_the_test_attribute_is_outside_the_use_list_whitelist()
    {
        let source_before_the_test_module = source_before_the_test_module_of(
            SOURCE_WITH_A_WALK_IMPORT_BEHIND_A_COMMENT_NAMING_THE_TEST_ATTRIBUTE,
        );
        assert!(
            source_before_the_test_module.ends_with("fn rebuild() {}\n\n"),
            "切点是恰是那一行属性的那一行，不是注释里的字样：{source_before_the_test_module:?}"
        );
        let (outside_the_whitelist, _) =
            use_list_differences_from_the_whitelist(source_before_the_test_module);
        assert_eq!(
            outside_the_whitelist,
            vec!["crate::walk::check_pool_image".to_string()],
            "注释之后那条 use 判在白名单之外"
        );
    }

    /// W1：同一份源码按路径找，`crate::walk`、`walk::` 与 `check_pool_image` 都判出来。
    #[test]
    fn a_walk_import_behind_a_comment_naming_the_test_attribute_is_named_outside_the_whitelist() {
        let named_outside_the_whitelist =
            names_outside_the_whitelist_in(source_before_the_test_module_of(
                SOURCE_WITH_A_WALK_IMPORT_BEHIND_A_COMMENT_NAMING_THE_TEST_ATTRIBUTE,
            ));
        assert_eq!(
            named_outside_the_whitelist,
            vec![
                "walk::".to_string(),
                "check_pool_image".to_string(),
                "crate::walk".to_string()
            ]
        );
    }

    /// 注释（行注释、文档注释、块注释）与字符串、字符字面量里提到走树的函数不算指名；字符串里的 `//` 不吞掉它后面的代码。
    #[test]
    fn walk_named_only_in_comments_and_string_literals_is_not_taken_for_a_path() {
        let mentions_only = "//! 比对在 `walk::check_pool_image` 的末尾\n/// 不调 crate::walk::check_pool_image\nfn rebuild() {\n    /* walk::check_pool_image */\n    let _ = \"crate::walk::check_pool_image\";\n    let _ = r#\"walk::\"check_pool_image\"#;\n    let _ = '\"';\n}\n";
        assert_eq!(
            names_outside_the_whitelist_in(mentions_only),
            Vec::<String>::new(),
            "只在注释与字面量里提到的不判"
        );
        let code_after_a_slash_pair_in_a_string =
            "fn rebuild() {\n    let _ = \"//\"; let _ = crate::walk::check_pool_image;\n}\n";
        assert_eq!(
            names_outside_the_whitelist_in(code_after_a_slash_pair_in_a_string),
            vec![
                "walk::".to_string(),
                "check_pool_image".to_string(),
                "crate::walk".to_string()
            ],
            "字符串里的 // 之后的代码照判"
        );
    }

    /// 恰是测试模块那一行属性的行多于一行：第二行会把它之后的非测试代码也切进测试模块，判红。
    #[test]
    #[should_panic(expected = "要恰好一行")]
    fn a_second_line_that_is_exactly_the_test_attribute_is_refused() {
        source_before_the_test_module_of(
            "#[cfg(test)]\nuse crate::walk::check_pool_image;\nfn rebuild() {}\n#[cfg(test)]\nmod tests {}\n",
        );
    }

    const EXTENT_TREE: u64 = 65536;
    const INODE_TREE: u64 = 65537;
    const TOMBSTONE_TREE: u64 = 65545;
    const LINEAGE_TREE: u64 = 65546;
    const RETIREMENT_TREE: u64 = 65548;
    const FIRST_INODE: u64 = 1;
    const OBJECT_BIRTH_OF_THE_FIRST_INODE: u64 = 3;
    /// 数据单元净荷容量（32768 − 134）：第二个单元的锚点偏移。
    const SECOND_DATA_UNIT_OFFSET_IN_BYTES: u64 = 32768 - 134;
    const INODE_RECORD_WIDTH: usize = 140;
    const TOMBSTONE_RECORD_WIDTH: usize = 40;
    const LINEAGE_RECORD_WIDTH: usize = 24;
    const RETIREMENT_RECORD_WIDTH: usize = 34;

    fn placement_on_device_zero(slot: u64) -> UnitPlacement {
        UnitPlacement { device: 0, slot }
    }

    /// 码 1 数据单元一个版本的头里这一组单测要的几段。
    struct DataUnitVersion {
        tree: u64,
        offset_in_bytes: u64,
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    }

    impl DataUnitVersion {
        fn of_the_first_inode_at(offset_in_bytes: u64) -> Self {
            Self {
                tree: EXTENT_TREE,
                offset_in_bytes,
                birth_txg: 3,
                instance: 1,
                transaction: 1,
            }
        }
    }

    /// 一份码 1 副本的明文头：五元组 (1, 树, 第一个 inode, 它的对象出生代, 锚点偏移)、诞生代号与写序，其余全零
    /// （重建不判头校验和，那一步归扫描方向）。
    fn data_unit_copy(placement: UnitPlacement, version: &DataUnitVersion) -> PublishedUnitCopy {
        let mut plain_header = vec![0u8; 105];
        plain_header[..4].copy_from_slice(b"SFSU");
        plain_header[6] = 1;
        plain_header[42] = 1;
        plain_header[43..51].copy_from_slice(&version.tree.to_le_bytes());
        plain_header[51..59].copy_from_slice(&FIRST_INODE.to_le_bytes());
        plain_header[59..67].copy_from_slice(&OBJECT_BIRTH_OF_THE_FIRST_INODE.to_le_bytes());
        plain_header[67..75].copy_from_slice(&version.offset_in_bytes.to_le_bytes());
        plain_header[75..83].copy_from_slice(&version.birth_txg.to_le_bytes());
        plain_header[91..95].copy_from_slice(&version.instance.to_le_bytes());
        plain_header[95..101].copy_from_slice(&version.transaction.to_le_bytes()[..6]);
        PublishedUnitCopy {
            placement,
            plain_header,
        }
    }

    /// 一片码 3 容器的类身份段与记录（这一组单测要的几段）。
    struct PackedContainer<'records> {
        birth_tree: u64,
        record_type: u16,
        container: u64,
        record_width: usize,
        records: &'records [Vec<u8>],
        birth_txg: u64,
    }

    /// 一片码 3 容器的整单元 32768 字节，两道校验和都封上（`check_unit` 要它们对得上）：类身份段 (3, 出生树, 类型, 容器号, 容器出生代 3)、
    /// 诞生代号、写序实例 1，记录从 136 起。
    fn sealed_packed_container(container: &PackedContainer<'_>) -> Vec<u8> {
        let mut unit = vec![0u8; 32768];
        unit[..4].copy_from_slice(b"SFSU");
        unit[4..6].copy_from_slice(&1u16.to_le_bytes());
        unit[6] = 3;
        let declared_length =
            u16::try_from(container.records.len() * container.record_width).expect("装得进一片");
        unit[8..10].copy_from_slice(&declared_length.to_le_bytes());
        unit[42] = 3;
        unit[43..51].copy_from_slice(&container.birth_tree.to_le_bytes());
        unit[51..53].copy_from_slice(&container.record_type.to_le_bytes());
        unit[53..61].copy_from_slice(&container.container.to_le_bytes());
        unit[61..69].copy_from_slice(&3u64.to_le_bytes());
        unit[69..71].copy_from_slice(
            &u16::try_from(container.records.len())
                .expect("条数")
                .to_le_bytes(),
        );
        unit[71..73].copy_from_slice(
            &u16::try_from(container.record_width)
                .expect("记录宽")
                .to_le_bytes(),
        );
        unit[73..81].copy_from_slice(&container.birth_txg.to_le_bytes());
        unit[93..97].copy_from_slice(&1u32.to_le_bytes());
        for (index, record) in container.records.iter().enumerate() {
            assert_eq!(record.len(), container.record_width, "记录宽");
            let start = 136 + index * container.record_width;
            unit[start..start + container.record_width].copy_from_slice(record);
        }
        let payload_crc = crc32_castagnoli_bitwise(&unit[107..]);
        unit[89..93].copy_from_slice(&payload_crc.to_le_bytes());
        let header_checksum = crc32_castagnoli_bitwise(&unit[..107]);
        unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
        unit
    }

    fn copy_of_the_container(placement: UnitPlacement, unit: &[u8]) -> PublishedUnitCopy {
        PublishedUnitCopy {
            placement,
            plain_header: unit[..107].to_vec(),
        }
    }

    fn inode_record(inode: u64, object_birth: u64, size_in_bytes: u64) -> Vec<u8> {
        let mut record = vec![0u8; INODE_RECORD_WIDTH];
        record[..8].copy_from_slice(&inode.to_le_bytes());
        record[8..16].copy_from_slice(&object_birth.to_le_bytes());
        record[36..40].copy_from_slice(&1u32.to_le_bytes());
        record[40..48].copy_from_slice(&size_in_bytes.to_le_bytes());
        record
    }

    fn tombstone_record(inode: u64, object_birth: u64, range_end_in_bytes: u64) -> Vec<u8> {
        let mut record = vec![0u8; TOMBSTONE_RECORD_WIDTH];
        record[..8].copy_from_slice(&inode.to_le_bytes());
        record[8..16].copy_from_slice(&object_birth.to_le_bytes());
        record[24..32].copy_from_slice(&range_end_in_bytes.to_le_bytes());
        record[32..40].copy_from_slice(&5u64.to_le_bytes());
        record
    }

    fn inode_container_holding(container: u64, records: &[Vec<u8>]) -> Vec<u8> {
        sealed_packed_container(&PackedContainer {
            birth_tree: INODE_TREE,
            record_type: PACKED_TYPE_INODE,
            container,
            record_width: INODE_RECORD_WIDTH,
            records,
            birth_txg: 3,
        })
    }

    fn tombstone_container_holding(records: &[Vec<u8>]) -> Vec<u8> {
        sealed_packed_container(&PackedContainer {
            birth_tree: TOMBSTONE_TREE,
            record_type: PACKED_TYPE_TOMBSTONE,
            container: 1,
            record_width: TOMBSTONE_RECORD_WIDTH,
            records,
            birth_txg: 5,
        })
    }

    /// 扫到的副本连同那几片容器的整单元字节（调用方读回来交给第二步的那一份）：择当前版本、读容器、现算三张表。
    fn rebuilt_from(
        copies: &[PublishedUnitCopy],
        containers: &[(UnitPlacement, Vec<u8>)],
    ) -> Result<RebuiltIndexTables, ScanRebuildNotDecided> {
        let versions: VersionsOfEachObject = choose_current_versions(copies)?;
        let mut container_copies: BTreeMap<ObjectIdentity, Vec<Vec<u8>>> = BTreeMap::new();
        for (identity, placements) in versions.packed_containers_whose_records_are_read() {
            let bytes: Vec<Vec<u8>> = containers
                .iter()
                .filter(|(placement, _)| placements.contains(placement))
                .map(|(_, unit)| unit.clone())
                .collect();
            container_copies.insert(identity, bytes);
        }
        rebuild_index_tables(&versions, &container_copies)
    }

    #[test]
    fn the_current_version_of_a_data_unit_is_the_one_with_the_largest_write_order() {
        let later_placement = placement_on_device_zero(50254);
        let copies = [
            data_unit_copy(
                later_placement,
                &DataUnitVersion {
                    birth_txg: 4,
                    instance: 2,
                    transaction: 1,
                    ..DataUnitVersion::of_the_first_inode_at(0)
                },
            ),
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion {
                    birth_txg: 5,
                    instance: 1,
                    transaction: 9,
                    ..DataUnitVersion::of_the_first_inode_at(0)
                },
            ),
        ];
        let tables =
            rebuilt_from(&copies, &[]).expect("写序互不相同，择得出；没有墓碑也没有记录，照认领");
        let placements: Vec<&BTreeSet<UnitPlacement>> = tables.central_mapping.values().collect();
        assert_eq!(
            placements,
            vec![&BTreeSet::from([later_placement])],
            "写序 (实例 2, 事务 1) 大于 (实例 1, 事务 9)：当前版本是实例 2 那一版（C113 定案 P3 码 1 取写序最大者），与诞生代号无关"
        );
    }

    #[test]
    fn two_data_unit_versions_with_the_same_write_order_are_not_decided_until_c464() {
        let copies = [
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion {
                    birth_txg: 4,
                    transaction: 7,
                    ..DataUnitVersion::of_the_first_inode_at(0)
                },
            ),
            data_unit_copy(
                placement_on_device_zero(50254),
                &DataUnitVersion {
                    birth_txg: 5,
                    transaction: 7,
                    ..DataUnitVersion::of_the_first_inode_at(0)
                },
            ),
        ];
        assert_eq!(
            rebuilt_from(&copies, &[]).err(),
            Some(ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided),
            "两版写序逐字节相同、诞生代号不同：P3 择不出，码 1 的键停在 C464，交回没定、不猜"
        );
    }

    #[test]
    fn copies_of_one_version_with_different_payload_checksums_are_not_decided() {
        let version = DataUnitVersion::of_the_first_inode_at(0);
        let mut second_copy = data_unit_copy(placement_on_device_zero(50181), &version);
        second_copy.plain_header[101] ^= 0xFF;
        let copies = [
            data_unit_copy(placement_on_device_zero(50180), &version),
            second_copy,
        ];
        assert_eq!(
            choose_current_versions(&copies).err(),
            Some(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderI18),
            "归并键相同而头里的载荷校验和不同：I-1.8 ① 判损坏、不择"
        );
    }

    #[test]
    fn two_packed_versions_with_the_same_total_order_key_are_not_decided() {
        let first = inode_container_holding(1, &[inode_record(FIRST_INODE, 3, 10)]);
        let second = inode_container_holding(
            1,
            &[
                inode_record(FIRST_INODE, 3, 10),
                inode_record(FIRST_INODE + 1, 3, 10),
            ],
        );
        let first_placement = placement_on_device_zero(50242);
        let second_placement = placement_on_device_zero(50254);
        let copies = [
            copy_of_the_container(first_placement, &first),
            copy_of_the_container(second_placement, &second),
        ];
        assert_eq!(
            rebuilt_from(
                &copies,
                &[(first_placement, first), (second_placement, second)]
            )
            .err(),
            Some(ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18),
            "同一片容器两版 (诞生代号, 实例代号, 事务号) 相同、记录数不同（归并键不同，是两组）：I-1.8 ② 不成立，择不出"
        );
    }

    #[test]
    fn the_inode_records_table_is_read_from_the_current_inode_container() {
        let record = inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, 3000);
        let container = inode_container_holding(1, std::slice::from_ref(&record));
        let container_placement = placement_on_device_zero(50242);
        let copies = [
            copy_of_the_container(container_placement, &container),
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion::of_the_first_inode_at(0),
            ),
        ];
        let tables = rebuilt_from(&copies, &[(container_placement, container)])
            .expect("一个文件、一片 inode 叶容器：每一格都有条款");
        assert_eq!(
            tables.inode_records,
            BTreeMap::from([(FIRST_INODE, record)]),
            "inode 记录集合是当前版本那片叶容器里的记录原样"
        );
        assert_eq!(
            tables
                .data_units_of_each_inode
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            vec![DataUnitPosition {
                inode: FIRST_INODE,
                offset_in_bytes: 0
            }],
            "size 3000 罩一个数据单元：偏移 0 那一个认领"
        );
        assert_eq!(
            tables.central_mapping.len(),
            2,
            "中央映射：数据单元一条、inode 叶容器一条"
        );
    }

    #[test]
    fn a_whole_object_tombstone_kills_the_data_units_of_an_object_whose_record_is_gone() {
        let tombstone_container = tombstone_container_holding(&[tombstone_record(
            FIRST_INODE,
            OBJECT_BIRTH_OF_THE_FIRST_INODE,
            u64::MAX,
        )]);
        let tombstone_placement = placement_on_device_zero(50300);
        let copies = [
            copy_of_the_container(tombstone_placement, &tombstone_container),
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion::of_the_first_inode_at(0),
            ),
        ];
        let tables = rebuilt_from(&copies, &[(tombstone_placement, tombstone_container)])
            .expect("记录已删、墓碑压着：数据单元判死、不认领");
        assert!(
            tables.data_units_of_each_inode.is_empty(),
            "墓碑判 key 死亡（D11 已定项 4）：被删对象的数据单元不进数据单元集合，实际 {:?}",
            tables.data_units_of_each_inode
        );
        assert_eq!(tables.central_mapping.len(), 1, "中央映射只剩那片墓碑容器");
    }

    /// 墓碑压着、记录还在的对象：它的记录与数据单元都不进重建出的表，记录原样放进 `records_of_the_tombstoned_objects`，扫描重建照常交回
    /// （此前整张交回「条款没写」）。算不算删除中这一侧不认，由比对那一侧两侧一起认（判决 m4-c860-r1 第三节 C2）。
    #[test]
    fn a_tombstoned_object_that_still_has_its_record_is_set_aside_for_the_comparison_to_judge_with_both_sides(
    ) {
        let tombstone_container = tombstone_container_holding(&[tombstone_record(
            FIRST_INODE,
            OBJECT_BIRTH_OF_THE_FIRST_INODE,
            u64::MAX,
        )]);
        let inode_container = inode_container_holding(
            1,
            &[inode_record(
                FIRST_INODE,
                OBJECT_BIRTH_OF_THE_FIRST_INODE,
                3000,
            )],
        );
        let tombstone_placement = placement_on_device_zero(50300);
        let inode_placement = placement_on_device_zero(50242);
        let copies = [
            copy_of_the_container(tombstone_placement, &tombstone_container),
            copy_of_the_container(inode_placement, &inode_container),
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion::of_the_first_inode_at(0),
            ),
        ];
        let tables = rebuilt_from(
            &copies,
            &[
                (tombstone_placement, tombstone_container),
                (inode_placement, inode_container),
            ],
        )
        .expect("墓碑压着、记录还在的对象先放一边，扫描重建照常交回");
        assert_eq!(
            tables.records_of_the_tombstoned_objects,
            BTreeMap::from([(
                FIRST_INODE,
                inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, 3000)
            )]),
            "意图写了墓碑、记录还在：那条记录原样放进墓碑压着的对象那一张"
        );
        assert!(
            tables.inode_records.is_empty() && tables.data_units_of_each_inode.is_empty(),
            "墓碑压着的对象的记录与数据单元都不进重建出的表，实际记录 {:?}、单元 {:?}",
            tables.inode_records.keys().collect::<Vec<_>>(),
            tables.data_units_of_each_inode
        );
    }

    /// 读法 Z 加写序（判决 m4-trunc-r1 K6）的那几条单测共用的镜像：第一个 inode 的记录（size `size_in_bytes`），与它在偏移 0、一个净荷处
    /// 各一个数据单元，出生 txg 分别是 `birth_txgs`（写序跟着出生 txg 走：实例 1、事务号 = 2 × 出生 txg 加单元号，同一个出生 txg 的两个单元是同一次发布里
    /// 先后两个事务，映射 key 不同）。交回重建出的数据单元集合里有哪几个偏移。
    fn offsets_claimed_with_two_data_units(size_in_bytes: u64, birth_txgs: [u64; 2]) -> Vec<u64> {
        offsets_claimed_with_two_data_units_under_a_truncation_field(
            size_in_bytes,
            TruncationFieldOfTheRecord {
                txg: 0,
                size_in_bytes: 0,
            },
            birth_txgs,
        )
    }

    /// inode 记录偏移 120 起的截断字段（判决 m4-trunc-r2 第三节 R1）：T 与 N_w。
    struct TruncationFieldOfTheRecord {
        txg: u64,
        size_in_bytes: u64,
    }

    /// 同 [`offsets_claimed_with_two_data_units`]，第一个 inode 的记录另带截断字段 `truncation_field`（读法 Z 第四条，判决 m4-trunc-r2 R3）。
    fn offsets_claimed_with_two_data_units_under_a_truncation_field(
        size_in_bytes: u64,
        truncation_field: TruncationFieldOfTheRecord,
        birth_txgs: [u64; 2],
    ) -> Vec<u64> {
        let mut record = inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, size_in_bytes);
        record[120..128].copy_from_slice(&truncation_field.txg.to_le_bytes());
        record[128..136].copy_from_slice(&truncation_field.size_in_bytes.to_le_bytes());
        let inode_container = inode_container_holding(1, &[record]);
        let inode_placement = placement_on_device_zero(50242);
        let data_unit_at = |offset_in_bytes: u64, birth_txg: u64| DataUnitVersion {
            tree: EXTENT_TREE,
            offset_in_bytes,
            birth_txg,
            instance: 1,
            transaction: 2 * birth_txg + offset_in_bytes / SECOND_DATA_UNIT_OFFSET_IN_BYTES,
        };
        let copies = [
            copy_of_the_container(inode_placement, &inode_container),
            data_unit_copy(
                placement_on_device_zero(50180),
                &data_unit_at(0, birth_txgs[0]),
            ),
            data_unit_copy(
                placement_on_device_zero(50182),
                &data_unit_at(SECOND_DATA_UNIT_OFFSET_IN_BYTES, birth_txgs[1]),
            ),
        ];
        let tables = rebuilt_from(&copies, &[(inode_placement, inode_container)])
            .expect("读法 Z 加写序下这几张镜像都定得下来");
        assert_eq!(
            tables.data_units_of_each_inode.len(),
            tables
                .central_mapping
                .keys()
                .filter(|key| key.0[0] == 1)
                .count(),
            "认领的数据单元同时进数据单元集合与中央映射"
        );
        tables
            .data_units_of_each_inode
            .keys()
            .map(|position| position.offset_in_bytes)
            .collect()
    }

    #[test]
    fn a_data_unit_beyond_the_size_of_its_record_is_not_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units(3000, [3, 3]),
            [0],
            "size 3000 只罩一个单元：第二个单元在 size 之外，读法 Z 不认领（判决 m4-trunc-r1 K6），不再交回没定"
        );
    }

    #[test]
    fn a_data_unit_born_before_a_data_unit_at_a_smaller_offset_is_not_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units(2 * SECOND_DATA_UNIT_OFFSET_IN_BYTES, [5, 3]),
            [0],
            "size 罩两个单元，偏移 0 那个出生 txg 5、第二个出生 txg 3：偏移更小的单元更新，第二个是变长留洞之前放掉的旧单元，写序判死、不认领"
        );
    }

    #[test]
    fn a_data_unit_born_after_the_data_units_at_smaller_offsets_is_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units(SECOND_DATA_UNIT_OFFSET_IN_BYTES + 100, [3, 5]),
            [0, SECOND_DATA_UNIT_OFFSET_IN_BYTES],
            "截断重写了最后留下的那个单元（出生 txg 5），它之下的单元更旧：出生 txg 随偏移不降，两个都认领"
        );
    }

    /// 题面那一形（判决 m4-trunc-r2 第三节 R3）：两个单元同在 txg 3 写下，txg 4 截在单元边界上（N_w = 一个净荷，不重写单元 0）、
    /// 放掉单元 1，再变长到两个净荷（洞）。单元 1 还在盘上、出生 txg 与单元 0 相同，写序那一条判不死它；第四条判死它，只认领单元 0。
    #[test]
    fn a_data_unit_born_before_the_truncation_field_beyond_its_size_is_not_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units_under_a_truncation_field(
                2 * SECOND_DATA_UNIT_OFFSET_IN_BYTES,
                TruncationFieldOfTheRecord {
                    txg: 4,
                    size_in_bytes: SECOND_DATA_UNIT_OFFSET_IN_BYTES,
                },
                [3, 3],
            ),
            [0],
            "截断字段 (4, 一个净荷)：单元 1 的单元号 1 ≥ ⌈N_w ÷ 净荷⌉ = 1、出生 txg 3 < 4，是截断放掉的旧单元，不认领"
        );
    }

    /// 截到 0（N_w = 0）再变长：⌈0 ÷ 净荷⌉.max(1) = 1，单元 0 不在第四条的射程里，出生 txg 早于 T 也照认领（去掉 .max(1) 会判死它）。
    #[test]
    fn a_truncation_field_to_size_zero_still_keeps_the_first_data_unit_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units_under_a_truncation_field(
                3000,
                TruncationFieldOfTheRecord {
                    txg: 4,
                    size_in_bytes: 0,
                },
                [3, 3],
            ),
            [0],
            "截断字段 (4, 0)：单元 0 的单元号 0 < ⌈0 ÷ 净荷⌉.max(1) = 1，照认领；单元 1 在 size 3000 之外，第二条不认领"
        );
    }

    /// 出生 txg 恰为 T：不早于 T，第四条不判死（`<` 写成 `<=` 会判死截断那一次及之后写下的单元）。
    #[test]
    fn a_data_unit_born_at_the_truncation_field_txg_is_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units_under_a_truncation_field(
                2 * SECOND_DATA_UNIT_OFFSET_IN_BYTES,
                TruncationFieldOfTheRecord {
                    txg: 3,
                    size_in_bytes: SECOND_DATA_UNIT_OFFSET_IN_BYTES,
                },
                [3, 3],
            ),
            [0, SECOND_DATA_UNIT_OFFSET_IN_BYTES],
            "截断字段 (3, 一个净荷)：单元 1 出生 txg 3 不早于 T = 3，两个都认领"
        );
    }

    /// 单元号恰比边界小一：N_w 比一个净荷多 1 字节，⌈N_w ÷ 净荷⌉ = 2，单元 1 是截断留下的单元，出生 txg 早于 T 也照认领
    /// （向下取整会判死它）。
    #[test]
    fn a_data_unit_below_the_rounded_up_truncation_field_size_is_claimed() {
        assert_eq!(
            offsets_claimed_with_two_data_units_under_a_truncation_field(
                2 * SECOND_DATA_UNIT_OFFSET_IN_BYTES,
                TruncationFieldOfTheRecord {
                    txg: 4,
                    size_in_bytes: SECOND_DATA_UNIT_OFFSET_IN_BYTES + 1,
                },
                [3, 3],
            ),
            [0, SECOND_DATA_UNIT_OFFSET_IN_BYTES],
            "截断字段 (4, 一个净荷 + 1)：⌈N_w ÷ 净荷⌉ = 2，单元 1 的单元号 1 < 2，照认领"
        );
    }

    /// 谱系记录 24 字节：T_rb、T_old、H_cur（回退之前那一版的 inode 号水位，重建不读它）。
    fn lineage_record(rollback_txg: u64, rollback_target_txg: u64) -> Vec<u8> {
        let mut record = vec![0u8; LINEAGE_RECORD_WIDTH];
        record[..8].copy_from_slice(&rollback_txg.to_le_bytes());
        record[8..16].copy_from_slice(&rollback_target_txg.to_le_bytes());
        record[16..24].copy_from_slice(&2u64.to_le_bytes());
        record
    }

    fn lineage_container_holding(records: &[Vec<u8>], birth_txg: u64) -> Vec<u8> {
        sealed_packed_container(&PackedContainer {
            birth_tree: LINEAGE_TREE,
            record_type: PACKED_TYPE_LINEAGE,
            container: 1,
            record_width: LINEAGE_RECORD_WIDTH,
            records,
            birth_txg,
        })
    }

    /// 退役记录 34 字节：被退役身份（inode 树、类型 2、容器号、容器出生代 3——`sealed_packed_container` 写的容器出生代）与死亡 txg。
    fn retirement_record_of_the_inode_container(container: u64, death_txg: u64) -> Vec<u8> {
        let mut record = vec![0u8; RETIREMENT_RECORD_WIDTH];
        record[..8].copy_from_slice(&INODE_TREE.to_le_bytes());
        record[8..10].copy_from_slice(&PACKED_TYPE_INODE.to_le_bytes());
        record[10..18].copy_from_slice(&container.to_le_bytes());
        record[18..26].copy_from_slice(&3u64.to_le_bytes());
        record[26..34].copy_from_slice(&death_txg.to_le_bytes());
        record
    }

    fn retirement_container_holding(records: &[Vec<u8>], birth_txg: u64) -> Vec<u8> {
        sealed_packed_container(&PackedContainer {
            birth_tree: RETIREMENT_TREE,
            record_type: PACKED_TYPE_RETIREMENT,
            container: 1,
            record_width: RETIREMENT_RECORD_WIDTH,
            records,
            birth_txg,
        })
    }

    /// 同一片 inode 叶容器（容器号 `container`、容器出生代 3）诞生代号为 `birth_txg` 的那一版。
    fn inode_container_version_born_at(
        container: u64,
        records: &[Vec<u8>],
        birth_txg: u64,
    ) -> Vec<u8> {
        sealed_packed_container(&PackedContainer {
            birth_tree: INODE_TREE,
            record_type: PACKED_TYPE_INODE,
            container,
            record_width: INODE_RECORD_WIDTH,
            records,
            birth_txg,
        })
    }

    fn tombstone_record_dying_at(inode: u64, object_birth: u64, death_txg: u64) -> Vec<u8> {
        let mut record = tombstone_record(inode, object_birth, u64::MAX);
        record[32..40].copy_from_slice(&death_txg.to_le_bytes());
        record
    }

    /// 回退那一形：第一个文件那一版 txg 3（数据单元与 inode 叶容器诞生代号 3），覆盖写一版 txg 4（诞生代号 4，写序更大），
    /// 挂着回退到 txg 3、回退那次发布 txg 5 写一条谱系记录 (5, 3)。txg 4 不在最新那一版的谱系上：那一版的两个单元不是任何现行版本的成员。
    #[test]
    fn a_lineage_record_leaves_out_the_versions_born_on_the_branch_the_rollback_abandoned() {
        let kept_data_unit = placement_on_device_zero(50180);
        let abandoned_data_unit = placement_on_device_zero(50254);
        let kept_record = inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, 3000);
        let kept_container =
            inode_container_version_born_at(1, std::slice::from_ref(&kept_record), 3);
        let abandoned_container = inode_container_version_born_at(
            1,
            &[inode_record(
                FIRST_INODE,
                OBJECT_BIRTH_OF_THE_FIRST_INODE,
                5000,
            )],
            4,
        );
        let lineage_container = lineage_container_holding(&[lineage_record(5, 3)], 5);
        let kept_container_placement = placement_on_device_zero(50242);
        let abandoned_container_placement = placement_on_device_zero(50260);
        let lineage_placement = placement_on_device_zero(50310);
        let copies = [
            data_unit_copy(kept_data_unit, &DataUnitVersion::of_the_first_inode_at(0)),
            data_unit_copy(
                abandoned_data_unit,
                &DataUnitVersion {
                    birth_txg: 4,
                    transaction: 2,
                    ..DataUnitVersion::of_the_first_inode_at(0)
                },
            ),
            copy_of_the_container(kept_container_placement, &kept_container),
            copy_of_the_container(abandoned_container_placement, &abandoned_container),
            copy_of_the_container(lineage_placement, &lineage_container),
        ];
        let tables = rebuilt_from(
            &copies,
            &[
                (kept_container_placement, kept_container),
                (abandoned_container_placement, abandoned_container),
                (lineage_placement, lineage_container),
            ],
        )
        .expect("谱系记录接成一条父链：[5, ∞) 与 [0, 3] 在谱系上，每一格都有条款");
        assert_eq!(
            tables.inode_records,
            BTreeMap::from([(FIRST_INODE, kept_record)]),
            "inode 叶容器择诞生代号 3 那一版（size 3000），不择回退抛弃的诞生代号 4 那一版"
        );
        let placements: BTreeSet<UnitPlacement> = tables
            .central_mapping
            .values()
            .flat_map(|placements| placements.iter().copied())
            .collect();
        assert_eq!(
            placements,
            BTreeSet::from([kept_data_unit, kept_container_placement, lineage_placement]),
            "中央映射只有诞生代号 3 的数据单元与叶容器、连同谱系容器自己；诞生代号 4 的两个单元不进映射"
        );
    }

    /// 回退抛弃的那一段（txg 4）里删了第一个文件、写了墓碑（死亡 txg 4），回退到 txg 3 之后文件又活着：墓碑随现行那一版走、照样择得到，
    /// 但它的死亡 txg 不在谱系上，不压回退目标那一版里的对象。
    #[test]
    fn a_tombstone_written_on_the_abandoned_branch_does_not_kill_the_object_the_rollback_brings_back(
    ) {
        let record = inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, 3000);
        let inode_container = inode_container_version_born_at(1, std::slice::from_ref(&record), 3);
        let tombstone_container = sealed_packed_container(&PackedContainer {
            birth_tree: TOMBSTONE_TREE,
            record_type: PACKED_TYPE_TOMBSTONE,
            container: 1,
            record_width: TOMBSTONE_RECORD_WIDTH,
            records: &[tombstone_record_dying_at(
                FIRST_INODE,
                OBJECT_BIRTH_OF_THE_FIRST_INODE,
                4,
            )],
            birth_txg: 4,
        });
        let lineage_container = lineage_container_holding(&[lineage_record(5, 3)], 5);
        let inode_placement = placement_on_device_zero(50242);
        let tombstone_placement = placement_on_device_zero(50300);
        let lineage_placement = placement_on_device_zero(50310);
        let copies = [
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion::of_the_first_inode_at(0),
            ),
            copy_of_the_container(inode_placement, &inode_container),
            copy_of_the_container(tombstone_placement, &tombstone_container),
            copy_of_the_container(lineage_placement, &lineage_container),
        ];
        let tables = rebuilt_from(
            &copies,
            &[
                (inode_placement, inode_container),
                (tombstone_placement, tombstone_container),
                (lineage_placement, lineage_container),
            ],
        )
        .expect("墓碑不在谱系上，记录还在也不是删除中");
        assert_eq!(
            tables
                .data_units_of_each_inode
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            vec![DataUnitPosition {
                inode: FIRST_INODE,
                offset_in_bytes: 0
            }],
            "回退带回来的文件的数据单元照认领"
        );
    }

    /// 叶容器 2 在 txg 6 退役（死亡 txg 6 在谱系上）：它最后那一版还在盘上扫得到，按退役树整个身份不认领——记录不进 inode 记录集合，
    /// 容器不进中央映射。
    #[test]
    fn a_retirement_record_on_the_lineage_leaves_out_the_leaf_container_it_names() {
        let live_record = inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, 0);
        let live_container =
            inode_container_version_born_at(1, std::slice::from_ref(&live_record), 3);
        let retired_container =
            inode_container_version_born_at(2, &[inode_record(FIRST_INODE + 1, 3, 0)], 3);
        let retirement_container =
            retirement_container_holding(&[retirement_record_of_the_inode_container(2, 6)], 6);
        let live_placement = placement_on_device_zero(50242);
        let retired_placement = placement_on_device_zero(50254);
        let retirement_placement = placement_on_device_zero(50320);
        let copies = [
            copy_of_the_container(live_placement, &live_container),
            copy_of_the_container(retired_placement, &retired_container),
            copy_of_the_container(retirement_placement, &retirement_container),
        ];
        let tables = rebuilt_from(
            &copies,
            &[
                (live_placement, live_container),
                (retired_placement, retired_container),
                (retirement_placement, retirement_container),
            ],
        )
        .expect("退役记录在谱系上，退役那片的版本诞生代号早于死亡 txg");
        assert_eq!(
            tables.inode_records,
            BTreeMap::from([(FIRST_INODE, live_record)]),
            "退役的那片叶容器里的记录不进 inode 记录集合"
        );
        assert!(
            tables
                .central_mapping
                .values()
                .all(|placements| !placements.contains(&retired_placement)),
            "退役的那片叶容器不进中央映射：{:?}",
            tables.central_mapping.values().collect::<Vec<_>>()
        );
    }

    /// 叶容器 2 在 txg 5 退役，之后回退到 txg 4（谱系记录 (6, 4)）：死亡 txg 5 落在回退抛弃的那一段里，退役树随现行那一版走、照样择得到，
    /// 但那条退役记录不在谱系上，叶容器 2 在回退目标那一版里还活着、照认领。
    #[test]
    fn a_retirement_record_off_the_lineage_leaves_the_leaf_container_it_names_claimed() {
        let revived_record = inode_record(FIRST_INODE + 1, 3, 0);
        let revived_container =
            inode_container_version_born_at(2, std::slice::from_ref(&revived_record), 3);
        let retirement_container =
            retirement_container_holding(&[retirement_record_of_the_inode_container(2, 5)], 5);
        let lineage_container = lineage_container_holding(&[lineage_record(6, 4)], 6);
        let revived_placement = placement_on_device_zero(50254);
        let retirement_placement = placement_on_device_zero(50320);
        let lineage_placement = placement_on_device_zero(50310);
        let copies = [
            copy_of_the_container(revived_placement, &revived_container),
            copy_of_the_container(retirement_placement, &retirement_container),
            copy_of_the_container(lineage_placement, &lineage_container),
        ];
        let tables = rebuilt_from(
            &copies,
            &[
                (revived_placement, revived_container),
                (retirement_placement, retirement_container),
                (lineage_placement, lineage_container),
            ],
        )
        .expect("退役记录不在谱系上");
        assert_eq!(
            tables.inode_records,
            BTreeMap::from([(FIRST_INODE + 1, revived_record)]),
            "死亡 txg 不在谱系上：叶容器 2 照认领"
        );
    }

    #[test]
    fn lineage_records_that_form_no_parent_chain_are_not_decided() {
        let lineage_container =
            lineage_container_holding(&[lineage_record(5, 3), lineage_record(5, 2)], 5);
        let placement = placement_on_device_zero(50310);
        let copies = [copy_of_the_container(placement, &lineage_container)];
        assert_eq!(
            rebuilt_from(&copies, &[(placement, lineage_container)]).err(),
            Some(ScanRebuildNotDecided::LineageRecordsFormNoParentChain),
            "同一个 T_rb 两条：接不成一条父链，父链断了怎么认没有条款，交回没定"
        );
    }

    #[test]
    fn a_lineage_record_whose_target_is_not_older_than_its_rollback_is_not_decided() {
        let lineage_container = lineage_container_holding(&[lineage_record(5, 5)], 5);
        let placement = placement_on_device_zero(50310);
        let copies = [copy_of_the_container(placement, &lineage_container)];
        assert_eq!(
            rebuilt_from(&copies, &[(placement, lineage_container)]).err(),
            Some(ScanRebuildNotDecided::LineageRecordsFormNoParentChain),
            "T_old 不小于 T_rb：接不成一条父链，交回没定"
        );
    }

    #[test]
    fn a_retired_leaf_container_with_a_version_born_after_its_death_is_not_decided() {
        let rewritten_container =
            inode_container_version_born_at(2, &[inode_record(FIRST_INODE + 1, 3, 0)], 7);
        let retirement_container =
            retirement_container_holding(&[retirement_record_of_the_inode_container(2, 6)], 6);
        let rewritten_placement = placement_on_device_zero(50254);
        let retirement_placement = placement_on_device_zero(50320);
        let copies = [
            copy_of_the_container(rewritten_placement, &rewritten_container),
            copy_of_the_container(retirement_placement, &retirement_container),
        ];
        assert_eq!(
            rebuilt_from(
                &copies,
                &[
                    (rewritten_placement, rewritten_container),
                    (retirement_placement, retirement_container)
                ]
            )
            .err(),
            Some(ScanRebuildNotDecided::RetiredLeafContainerHasAVersionBornAfterItsDeath),
            "退役在 txg 6、那一片又有诞生代号 7 的一版：退役树与那一版谁算数没有条款，交回没定"
        );
    }

    #[test]
    fn a_tombstone_whose_range_is_not_the_whole_object_is_not_decided() {
        let tombstone_container = tombstone_container_holding(&[tombstone_record(
            FIRST_INODE,
            OBJECT_BIRTH_OF_THE_FIRST_INODE,
            4096,
        )]);
        let placement = placement_on_device_zero(50300);
        let copies = [copy_of_the_container(placement, &tombstone_container)];
        assert_eq!(
            rebuilt_from(&copies, &[(placement, tombstone_container)]).err(),
            Some(ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion),
            "墓碑区间止 4096、不是 2⁶⁴ − 1：第一版只写整对象，区间墓碑怎么压 key 没有条款"
        );
    }

    #[test]
    fn a_current_container_without_a_decodable_copy_is_not_decided() {
        let inode_container = inode_container_holding(1, &[inode_record(FIRST_INODE, 3, 3000)]);
        let mut torn_copy = inode_container.clone();
        torn_copy[200] ^= 0xFF;
        let placement = placement_on_device_zero(50242);
        let copies = [copy_of_the_container(placement, &inode_container)];
        assert_eq!(
            rebuilt_from(&copies, &[(placement, torn_copy)]).err(),
            Some(ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy),
            "当前版本那片 inode 叶容器唯一一份载荷 CRC 对不上：解不开，交回没定"
        );
    }

    #[test]
    fn two_objects_at_one_data_unit_position_are_not_decided() {
        let copies = [
            data_unit_copy(
                placement_on_device_zero(50180),
                &DataUnitVersion::of_the_first_inode_at(0),
            ),
            data_unit_copy(
                placement_on_device_zero(50254),
                &DataUnitVersion {
                    tree: EXTENT_TREE + 13,
                    ..DataUnitVersion::of_the_first_inode_at(0)
                },
            ),
        ];
        assert_eq!(
            rebuilt_from(&copies, &[]).err(),
            Some(ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition),
            "两棵树的数据单元落在同一个 (inode, 偏移)：第一版只有一棵 extent 树，按哪一个认没有条款"
        );
    }

    #[test]
    fn one_inode_number_in_two_current_containers_is_not_decided() {
        let record = inode_record(FIRST_INODE, 3, 3000);
        let first = inode_container_holding(1, std::slice::from_ref(&record));
        let second = inode_container_holding(2, &[record]);
        let first_placement = placement_on_device_zero(50242);
        let second_placement = placement_on_device_zero(50254);
        let copies = [
            copy_of_the_container(first_placement, &first),
            copy_of_the_container(second_placement, &second),
        ];
        assert_eq!(
            rebuilt_from(
                &copies,
                &[(first_placement, first), (second_placement, second)]
            )
            .err(),
            Some(ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers),
            "容器 1 与容器 2 里都有 inode 1：按哪一条认没有条款"
        );
    }
}
