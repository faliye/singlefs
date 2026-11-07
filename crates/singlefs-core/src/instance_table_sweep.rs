//! 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
//! 第二节 S1 与第六节用户定案 Q1、Q3、Q4）：一个实例的根全部轮出根环之后，抹掉池里按它那一行判未发布的单元头，再发一次专用的清扫发布
//! 把那一行删掉。
//!
//! 次序是先判、后抹（F6）：判根环每个槽都自证过、环里没有该实例的根，判池中每一个落点（[`judge_the_instance_table_sweep`]），
//! 判过了才抹（FUA 写零头所在的那一个物理块，之后每块盘一道屏障），抹头全部返回之后才发清扫发布
//! （[`erase_the_heads_and_publish_the_sweep`]）。这次清扫里有一次读失败、设备不全、根环有一槽读不出或全零或重读一次仍自证不过，
//! 这一轮一行都不删、一个头都不抹；抹之前那一遍重读报设备错或读回的不是判的那个头，同样一行都不删、一个头都不抹
//! （用户 2026-10-04 定 C，`research/prompts/m3-root-ring-code-r3-main-verification.md` 第六节 Q2）。
//! 清扫发布的落点被拒时这一轮不删行，挂载照样交回可写（Q4「不单列，被拒就这一轮不删」）。
//!
//! 谁跑、什么时候跑（Q1）：可写挂载末尾（写行、暖机、抬 F、删除续做之后，交回之前）判一次——开关关着时实例表行数超过阈值就在这次挂载里
//! 立刻判、抹、发，没超过不扫；开关开着时交给挂着时的后台清扫（[`sweep_the_instance_table_while_mounted`]），后台抹之前在同一串行段里
//! 重读要抹的头，全部仍是候选行的未发布单元才抹（F1；有一个不是，这一轮不删，C）。挂着时什么时候调度后台那一轮，这一批没做（入口在，调度器没有）。
//!
//! 池中每一个落点怎么读（Q3「两条都要」、S2-2「按整单元跳」）：每块盘从单元区起点按槽升序读头，先照认领规则认单元（D18 已定项 7：解出一个头
//! 就按它的整单元跨度把后面的槽认领掉——码 1、码 3 两槽，码 2 一槽），再跳过根环里任一可读根的账里仍分配的槽；认「是单元」只看头校验和，
//! 不要求载荷校验和过（不比任何读盘判定更严）。根环里 txg 低于现行那一版 F 的根跳过它的账（不读、不算），其余的根一条账读不出就这一轮不删
//! （用户 2026-10-04 定 A2，同一份判决第六节 Q1：F 之下的根回收门槛已许可它独占的槽被复用，它的分配记录树节点可以已被合法盖掉）。
//!
//! 第一版不做的：现行那一版树表 0 条时不清扫（[`NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`]：
//! 那一版的账住根记录那一项、不在树表里，「跳过根环里可读根的账」那一半读不全；清扫发布在那一版上的形状条款也没写）。
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use singlefs_format::{
    DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, PACKED_UNIT_HEADER_BYTES,
};

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use crate::allocator::{absolute_slot_count_of_device, PoolAllocator};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::checksum::wide_checksum_field_holds;
use crate::instance_table::InstanceRow;
use crate::journal::back_chain_of;
use crate::make_filesystem::{MakeFilesystemParameters, GENESIS_CHECKPOINT_TXG};
use crate::mount::RollbackTarget;
use crate::recovery::{
    allocation_records_under_root_in_the_unit_area_starting_at, choose_system_configuration,
    every_root_ring_slot, instance_table_chain_of_root,
    unit_area_start_of_the_chosen_system_configuration, PoolReader, RecoveryFailure,
};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{
    root_sits_in_the_ring_slot_the_formula_gives, slot_offset, target_for_publish, RootRingSlot,
    RootRingSlotsPerRegion,
};
use crate::transaction::{
    prepare_the_version_publish, publish_version, DeletionAndLineageOfThePublish,
    InstanceTablePlan, InstanceTableRewrite, PoolVersion, PoolWriter, PublishError, PublishPlan,
    ReleaseChecksumCheck, TransactionOutput,
};
use crate::unit::{
    unit_filesystem_identifier, HEADER_CHECKSUM_OFFSET, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED, UNIT_MAGIC,
};

/// 清扫在可写挂载末尾怎么走（开关，D18 已定项 11「谁跑、什么时候跑」，用户 2026-10-03 定）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceTableSweepSwitch {
    /// 开关关着：实例表行数超过阈值就在这次挂载里立刻判、抹、发清扫发布；没超过不扫。
    SweptAtTheEndOfTheMountAboveTheThreshold,
    /// 开关开着：挂载末尾不扫，交给挂着时的后台清扫（[`sweep_the_instance_table_while_mounted`]）。
    HandedToTheBackgroundSweepWhileMounted,
}

/// 实例表行数的阈值（配置项）：开关关着时行数超过它才在挂载末尾清扫。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceTableRowCountThreshold(pub u64);

impl InstanceTableRowCountThreshold {
    /// 产品路径的默认阈值 365 行：用户 2026-10-29 弹窗定（C690（清扫触发阈值的默认值没定））。它是清扫的触发阈值：挂载末尾现行那一版的
    /// 实例表行数越过它（366 行起）的那一次挂载末尾就扫。它不改每片装几行：每片仍装 369 行（370 条记录减去链指针那一条）。
    /// 每次挂载写一行、末尾清扫做成时链留在一片之内；清扫这一轮一行不删、一次挂载写几行、或开关交给挂着时的后台清扫时，链照样长出第二片，
    /// 切换预留照准入式子按片数留（`admission` 里按每片行数取上整的那一项）。
    // 出处：E183 第三段（H5）的读数，θ = 365 不越过一片的上界、θ = 369 越过（`.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md`「H5 的读数」那一条）。
    pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);
    /// 有一行就清扫（用例要在少量行上强制进入清扫那一支，`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
    pub const ANY_ROW: Self = Self(0);
}

/// 清扫发布从哪个分配器取落点：只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）——用例要强制进入「清扫发布的落点被拒」那一支，
/// 而一个刚做成可写挂载的池上要造出「写行与暖机落得下、清扫发布落不下」很难卡准。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SweepPublishPlacements {
    /// 产品路径：照这次挂载的分配器取。
    TakenFromTheAllocatorOfTheMount,
    /// 只供测试：在分配器的一份拷贝上把每块盘每个空闲槽都占掉再取落点（走真的取落点那一段，落点被拒），一个写都不发。
    EveryFreeSlotTakenAwayByTheTestOnlySwitch,
}

/// 清扫的配置：开关、阈值与只供测试的落点开关。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceTableSweepConfiguration {
    pub switch: InstanceTableSweepSwitch,
    pub row_count_threshold: InstanceTableRowCountThreshold,
    pub sweep_publish_placements: SweepPublishPlacements,
}

impl InstanceTableSweepConfiguration {
    /// 产品路径（`mount::mount_writable`）：开关关着、阈值取默认的 365 行（[`InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`]）。
    pub const PRODUCT: Self = Self {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold:
            InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE,
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    };
}

/// 一个单元头里清扫要的几样（只看头校验和认出来的）：类、写序实例、诞生代号、码 1 / 码 3 写序里的事务号、fsid 低 8 字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitHeadReadBySweep {
    pub unit_class: u8,
    pub writer_instance: InstanceGeneration,
    pub birth_txg: CheckpointTxg,
    /// 码 1、码 3 写序里的事务号；码 2 的写序只有实例代号（D18 已定项 12），这里是 0、已发布谓词不看它。
    pub transaction: u64,
    pub filesystem_identifier: u64,
}

impl UnitHeadReadBySweep {
    /// 认领规则按整单元跳的跨度（S2-2「按整单元跳」）：码 1、码 3 两槽，码 2 一槽。
    #[must_use]
    pub fn span_in_slots(&self) -> u64 {
        if self.unit_class == UNIT_CLASS_INDEX_NODE {
            NODE_SPAN_IN_SLOTS
        } else {
            DATA_OR_PACKED_UNIT_SPAN_IN_SLOTS
        }
    }

    /// 已发布谓词里有行的那一支（D18 已定项 11「已发布谓词」）：有行 (i, T, W) 时，码 1 ⇒ b ≤ T ∨ n ≤ W 已发布，码 2 / 3 ⇒ b ≤ T 已发布。
    #[must_use]
    pub fn is_unpublished_by_the_row(&self, row: &InstanceRow) -> bool {
        if self.unit_class == UNIT_CLASS_DATA {
            !(self.birth_txg <= row.selected_root_txg
                || self.transaction <= row.applied_transaction_high_water)
        } else {
            self.birth_txg > row.selected_root_txg
        }
    }
}

const NODE_SPAN_IN_SLOTS: u64 = 1;
const DATA_OR_PACKED_UNIT_SPAN_IN_SLOTS: u64 = 2;
/// 码 2 头里 key 宽那一个字节的偏移（D18 已定项 18 的偏移表）。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
/// 码 1 头里诞生代号、fsid、写序实例、事务号的偏移（D18 已定项 7 / 已定项 18 的偏移表；`unit::parse_data_unit` 读的同一处）。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
const DATA_UNIT_FILESYSTEM_IDENTIFIER_OFFSET: usize = 83;
const DATA_UNIT_WRITER_INSTANCE_OFFSET: usize = 91;
const DATA_UNIT_TRANSACTION_OFFSET: usize = 95;
/// 码 3 头里同几样的偏移（`unit::parse_packed_unit` 读的同一处）。
const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
const PACKED_UNIT_FILESYSTEM_IDENTIFIER_OFFSET: usize = 81;
const PACKED_UNIT_WRITER_INSTANCE_OFFSET: usize = 93;
const PACKED_UNIT_TRANSACTION_OFFSET: usize = 97;
/// 码 2 头里诞生代号、fsid、写序实例在 key 区间之后的偏移：52 + 2k 起依次 8、8、4 字节（`unit::parse_index_node` 读的同一处）。
const INDEX_NODE_BIRTH_TXG_OFFSET_AFTER_THE_KEY_RANGE: usize = 52;
const INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_AFTER_THE_KEY_RANGE: usize = 60;
const INDEX_NODE_WRITER_INSTANCE_OFFSET_AFTER_THE_KEY_RANGE: usize = 68;
/// 事务号在写序里占 6 字节（D18 已定项 7）。
const TRANSACTION_BYTES: usize = 6;
/// 一个槽读多少字节来认头：码 2 的明文头最宽 86 + 2 × 255 = 596 字节，读 1024 字节罩得住任何一类的头；不足一个物理块时读一个物理块。
const HEAD_BYTES_READ_PER_SLOT: u64 = 1024;

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("切出来的是 8 字节"),
    )
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("切出来的是 4 字节"),
    )
}

fn six_byte_unsigned_at(bytes: &[u8], offset: usize) -> u64 {
    let mut widened = [0u8; 8];
    widened[..TRANSACTION_BYTES].copy_from_slice(&bytes[offset..offset + TRANSACTION_BYTES]);
    u64::from_le_bytes(widened)
}

/// 一个槽开头的字节认不认得出一个单元头：魔数、类（1 / 2 / 3，定头的宽）与头校验和过就认，别的（版本、flags、载荷校验和）不看
/// ——清扫认「是单元」不比任何读盘判定更严（判决第二节 Q2 攻方补的那一句收严）。认不出交 `None`（已抹头、没写过、别的字节）。
#[must_use]
pub fn unit_head_read_by_sweep(head_bytes: &[u8]) -> Option<UnitHeadReadBySweep> {
    if head_bytes.len() <= INDEX_NODE_KEY_WIDTH_OFFSET || head_bytes[..4] != UNIT_MAGIC {
        return None;
    }
    let unit_class = head_bytes[6];
    let header_end = match unit_class {
        UNIT_CLASS_DATA => usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105"),
        UNIT_CLASS_PACKED => usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),
        UNIT_CLASS_INDEX_NODE => {
            usize::try_from(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE).expect("86")
                + usize::try_from(INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT).expect("2")
                    * usize::from(head_bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
        }
        _ => return None,
    };
    if head_bytes.len() < header_end
        || !wide_checksum_field_holds(head_bytes, header_end, HEADER_CHECKSUM_OFFSET)
    {
        return None;
    }
    Some(match unit_class {
        UNIT_CLASS_DATA => UnitHeadReadBySweep {
            unit_class,
            birth_txg: CheckpointTxg(u64_at(head_bytes, DATA_UNIT_BIRTH_TXG_OFFSET)),
            filesystem_identifier: u64_at(head_bytes, DATA_UNIT_FILESYSTEM_IDENTIFIER_OFFSET),
            writer_instance: InstanceGeneration(u32_at(
                head_bytes,
                DATA_UNIT_WRITER_INSTANCE_OFFSET,
            )),
            transaction: six_byte_unsigned_at(head_bytes, DATA_UNIT_TRANSACTION_OFFSET),
        },
        UNIT_CLASS_PACKED => UnitHeadReadBySweep {
            unit_class,
            birth_txg: CheckpointTxg(u64_at(head_bytes, PACKED_UNIT_BIRTH_TXG_OFFSET)),
            filesystem_identifier: u64_at(head_bytes, PACKED_UNIT_FILESYSTEM_IDENTIFIER_OFFSET),
            writer_instance: InstanceGeneration(u32_at(
                head_bytes,
                PACKED_UNIT_WRITER_INSTANCE_OFFSET,
            )),
            transaction: six_byte_unsigned_at(head_bytes, PACKED_UNIT_TRANSACTION_OFFSET),
        },
        _ => {
            let key_range_bytes = usize::try_from(INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT)
                .expect("2")
                * usize::from(head_bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
            UnitHeadReadBySweep {
                unit_class,
                birth_txg: CheckpointTxg(u64_at(
                    head_bytes,
                    INDEX_NODE_BIRTH_TXG_OFFSET_AFTER_THE_KEY_RANGE + key_range_bytes,
                )),
                filesystem_identifier: u64_at(
                    head_bytes,
                    INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_AFTER_THE_KEY_RANGE + key_range_bytes,
                ),
                writer_instance: InstanceGeneration(u32_at(
                    head_bytes,
                    INDEX_NODE_WRITER_INSTANCE_OFFSET_AFTER_THE_KEY_RANGE + key_range_bytes,
                )),
                transaction: 0,
            }
        }
    })
}

/// 一个落点：哪块盘、哪个槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementOfAUnitHead {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
}

/// 要抹的一个头：落点与判的那一刻读出来的头（后台那一支抹之前重读、与它比）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitHeadToErase {
    pub placement: PlacementOfAUnitHead,
    pub head: UnitHeadReadBySweep,
}

/// 这一轮为什么一行都不删（判的那一半，一个头都没抹）。
#[derive(Debug)]
pub enum NoRowDeletedThisRound {
    /// 实例表里每一行的实例在根环里都还有根（或表里没有行）：没有候选行。
    NoCandidateRow,
    /// 交进来的盘少于池的盘数（设备维：全部设备在线才删）。
    NotEveryDeviceOfThePoolIsPresent {
        devices_present: usize,
        devices_of_the_pool: u32,
    },
    /// 根环一个槽读不出、或读成功而全零：按「可能有该实例的根」算，这一次不删。
    ARootRingSlotIsUnreadableOrAllZero { ring_slot: RootRingSlot },
    /// 根环一个槽读得出而自证不过，重读一次仍不过（自证得过、却不住在它的 txg 按落点公式该住的那一槽的错位读也算自证不过，
    /// 与 `recovery::read_root_ring_slot` 同一个核 `root_ring::root_sits_in_the_ring_slot_the_formula_gives`；
    /// 一个槽 0 读出第 0 代根、而同一遍读出的环里最大的 txg 已到发布路径写进这一槽的 txg，也算这一槽自证不过）。
    ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot: RootRingSlot },
    /// 择不到系统配置：判不了池有几块盘、根环在哪。
    SystemConfigurationNotChosen { failure: RecoveryFailure },
    /// 现行那一版的实例表读不出：判不了哪几行是候选。
    InstanceTableOfTheCurrentVersionUnreadable { failure: RecoveryFailure },
    /// 根环里一条 txg 不低于现行那一版 F 的可读根的账读不出：「跳过它的账里仍分配的槽」那一半判不了（F 之下的根不读它的账，A2）。
    AccountOfARingRootUnreadable {
        root: RollbackTarget,
        failure: RecoveryFailure,
    },
    /// 读池中一个落点的头读失败（扇区维：读不到不等于不存在）。
    AReadOfAPlacementFailed { placement: PlacementOfAUnitHead },
    /// 池里有一个本池的单元，写序实例大于挂载根的实例（已发布谓词判损坏）：这一轮任何行都不删（判决第二节 Q3 正推给的保守读法）。
    AUnitWrittenByAnInstanceAboveTheMountedOne {
        placement: PlacementOfAUnitHead,
        writer_instance: InstanceGeneration,
    },
    /// 现行那一版树表 0 条：第一版不清扫（模块文档「第一版不做的」）。在任何写之前判出，一个头都不抹。
    SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
    /// 抹之前那一遍重读，一个要抹的头报设备错或读回的不是判的那个头（用户 2026-10-04 定 C）：这一轮不删，一个头都不抹
    /// （先把要抹的头全部重读一遍、都还是判的那个头才开始抹，[`erase_the_heads_and_publish_the_sweep`]）。
    AHeadRereadBeforeErasingIsNotTheJudgedUnit {
        placement: PlacementOfAUnitHead,
        reread: HeadRereadBeforeErasing,
    },
}

/// 抹之前那一遍重读一个要抹的头读到了什么（[`NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadRereadBeforeErasing {
    /// 读失败（设备报错）。
    ReadFailed,
    /// 读得出，而认出来的头不是判的那一刻那个（已抹、被覆盖、读回全零这类认不出头的也在这里），或不再按候选行判未发布。
    NotTheJudgedUnit,
}

/// 判的那一半交回的：要删的行、要抹的头，与判的那一刻池里认出来的单元个数（交给用例看扫没扫全）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableSweepJudgement {
    pub rows_to_delete: Vec<InstanceRow>,
    pub heads_to_erase: Vec<UnitHeadToErase>,
    pub units_recognized_in_the_pool: u64,
}

/// 判一轮清扫（只读，不写任何一个字节）：根环每个槽都自证过（读不出、全零的这一次不删；自证不过的重读一次），实例表里实例在环里一条根都没有的行
/// 是候选行；池中每一个落点照认领规则认单元、跳过根环里任一可读根的账里仍分配的槽，按候选行判未发布的记下要抹。
///
/// # Errors
/// [`NoRowDeletedThisRound`] 的各个成员（`NoCandidateRow` 也走这里：没有要删的行就不扫池）。
pub fn judge_the_instance_table_sweep(
    reader: &dyn PoolReader,
    current_root: &RootRecord,
) -> Result<InstanceTableSweepJudgement, NoRowDeletedThisRound> {
    let system_configuration = choose_system_configuration(reader)
        .map_err(|failure| NoRowDeletedThisRound::SystemConfigurationNotChosen { failure })?;
    let immutable = &system_configuration.immutable;
    let devices_present = reader.device_identities();
    if u32::try_from(devices_present.len()).expect("盘数装得进 u32") < immutable.device_count {
        return Err(NoRowDeletedThisRound::NotEveryDeviceOfThePoolIsPresent {
            devices_present: devices_present.len(),
            devices_of_the_pool: immutable.device_count,
        });
    }
    let roots_of_the_ring = every_root_of_the_ring_self_verified(
        reader,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )?;
    let rows = instance_table_chain_of_root(reader, current_root)
        .map_err(
            |failure| NoRowDeletedThisRound::InstanceTableOfTheCurrentVersionUnreadable { failure },
        )?
        .records
        .rows;
    let instances_with_a_root_in_the_ring: BTreeSet<InstanceGeneration> =
        roots_of_the_ring.iter().map(|root| root.instance).collect();
    let rows_to_delete: Vec<InstanceRow> = rows
        .iter()
        .copied()
        .filter(|row| !instances_with_a_root_in_the_ring.contains(&row.instance))
        .collect();
    if rows_to_delete.is_empty() {
        return Err(NoRowDeletedThisRound::NoCandidateRow);
    }
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    let mut slots_allocated_in_an_account_of_the_ring: BTreeSet<PlacementOfAUnitHead> =
        BTreeSet::new();
    // A2（用户 2026-10-04 定）：txg 低于现行那一版 F 的根不读它的账——正常卸载抬过 F 之后，F 之下的旧根的分配记录树节点被合法复用，
    // 那条根的账读不出，照「一条读不出就不删」每一轮都停在这里、一行都删不掉（判决第二节 K1-a）。
    for root in roots_of_the_ring
        .iter()
        .filter(|root| root.checkpoint_txg >= current_root.rollback_floor)
    {
        let records = allocation_records_under_root_in_the_unit_area_starting_at(
            reader,
            unit_area_start,
            root,
        )
        .map_err(
            |failure| NoRowDeletedThisRound::AccountOfARingRootUnreadable {
                root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
                failure,
            },
        )?;
        for record in records.iter().filter(|record| !record.is_released) {
            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                slots_allocated_in_an_account_of_the_ring.insert(PlacementOfAUnitHead {
                    device: record.device,
                    slot: SlotNumber(slot),
                });
            }
        }
    }
    let filesystem_identifier = unit_filesystem_identifier(&immutable.filesystem_identifier);
    let head_bytes = usize::try_from(
        HEAD_BYTES_READ_PER_SLOT.max(u64::from(immutable.sizes.physical_block_size)),
    )
    .expect("读头的字节数装得进 usize");
    let rows_by_instance: BTreeMap<InstanceGeneration, InstanceRow> = rows_to_delete
        .iter()
        .map(|row| (row.instance, *row))
        .collect();
    let mut heads_to_erase = Vec::new();
    let mut units_recognized_in_the_pool = 0u64;
    for device in devices_present {
        let slots_of_the_device = reader
            .device_size_in_bytes(device)
            .map_or(0, absolute_slot_count_of_device);
        let mut slot = unit_area_start.slot().0;
        // 迭代上界是这块盘的槽数；每轮至少前进一槽（认出单元就跳过它的整单元跨度）；跨轮携带的只有游标与记下的头；
        // 提前出口是读失败与写序实例越过挂载根的单元两条。
        while slot < slots_of_the_device {
            let placement = PlacementOfAUnitHead {
                device,
                slot: SlotNumber(slot),
            };
            let Some(bytes) = reader.read(device, SlotNumber(slot).to_device_offset(), head_bytes)
            else {
                return Err(NoRowDeletedThisRound::AReadOfAPlacementFailed { placement });
            };
            let Some(head) = unit_head_read_by_sweep(&bytes) else {
                slot += 1;
                continue;
            };
            units_recognized_in_the_pool += 1;
            slot += head.span_in_slots();
            if head.filesystem_identifier != filesystem_identifier {
                continue;
            }
            if head.writer_instance > current_root.instance {
                return Err(
                    NoRowDeletedThisRound::AUnitWrittenByAnInstanceAboveTheMountedOne {
                        placement,
                        writer_instance: head.writer_instance,
                    },
                );
            }
            if slots_allocated_in_an_account_of_the_ring.contains(&placement) {
                continue;
            }
            if rows_by_instance
                .get(&head.writer_instance)
                .is_some_and(|row| head.is_unpublished_by_the_row(row))
            {
                heads_to_erase.push(UnitHeadToErase { placement, head });
            }
        }
    }
    Ok(InstanceTableSweepJudgement {
        rows_to_delete,
        heads_to_erase,
        units_recognized_in_the_pool,
    })
}

/// 读根环每个槽：读不出、读成功而全零 ⇒ 这一次不删；读得出而自证不过（含自证得过、落点公式不合的错位读）⇒ 重读一次，仍不过 ⇒ 这一次不删；
/// 都自证过之后，再按这一遍读出的环看住着第 0 代根的槽（[`genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached`]）：
/// 这一遍读出的最大 txg 已到发布路径写进这一槽的 txg，这一槽的第 0 代根算自证不过，这一槽只读过一次的重读一次、仍不过（或已重读过）⇒ 这一次不删；
/// 都过了就交回每一条根。
fn every_root_of_the_ring_self_verified(
    reader: &dyn PoolReader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &crate::system_configuration::SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Result<Vec<RootRecord>, NoRowDeletedThisRound> {
    let ring_slots = every_root_ring_slot(immutable_sizes);
    let ring_length_in_slots = ring_slots.len();
    let mut roots_read: Vec<RootOfARingSlotReadBySweep> = Vec::with_capacity(ring_length_in_slots);
    // 迭代上界是根环的槽数（3 × S），每槽至多读两次；提前出口是这一次不删的那几种。
    for ring_slot in ring_slots {
        let mut root_read = None;
        let mut readings_of_the_slot = 0u32;
        for _reading in 0..2 {
            readings_of_the_slot += 1;
            root_read = root_read_once_from_the_ring_slot(
                reader,
                region_devices,
                immutable_sizes,
                filesystem_identifier,
                ring_slot,
            )?;
            if root_read.is_some() {
                break;
            }
        }
        let Some(root) = root_read else {
            return Err(
                NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot },
            );
        };
        roots_read.push(RootOfARingSlotReadBySweep {
            ring_slot,
            root,
            self_verified_on_the_first_read: readings_of_the_slot == 1,
        });
    }
    let highest_txg_of_the_pass = roots_read
        .iter()
        .map(|read| read.root.checkpoint_txg)
        .max()
        .expect("根环至少三个区域各一槽，每槽都交回了一条根");
    let slots_per_region = immutable_sizes.root_ring_slots_per_region;
    let genesis_root_reached_by_the_publishes_of_the_pass = |root: &RootRecord, ring_slot| {
        genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached(
            root.checkpoint_txg,
            ring_slot,
            highest_txg_of_the_pass,
            slots_per_region,
            ring_length_in_slots,
        )
    };
    // 迭代上界是根环的槽数；每槽至多再读一次（判据只用上面那一遍读出的最大 txg，重读读出的不改它）；提前出口是这一次不删的那几种。
    for read in &mut roots_read {
        if !genesis_root_reached_by_the_publishes_of_the_pass(&read.root, read.ring_slot) {
            continue;
        }
        if !read.self_verified_on_the_first_read {
            return Err(
                NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread {
                    ring_slot: read.ring_slot,
                },
            );
        }
        let reread = root_read_once_from_the_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            read.ring_slot,
        )?
        .filter(|root| !genesis_root_reached_by_the_publishes_of_the_pass(root, read.ring_slot));
        let Some(root) = reread else {
            return Err(
                NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread {
                    ring_slot: read.ring_slot,
                },
            );
        };
        read.root = root;
    }
    Ok(roots_read.into_iter().map(|read| read.root).collect())
}

/// 清扫读根环一槽读出的根：住在哪一槽、读出的根、是不是第一次读就自证过（不是的已重读过一次，不再读）。
struct RootOfARingSlotReadBySweep {
    ring_slot: RootRingSlot,
    root: RootRecord,
    self_verified_on_the_first_read: bool,
}

/// 读根环一槽一次：读不出、读成功而全零 ⇒ 这一次不删（`Err`）；读得出而自证不过、或自证得过而落点公式不合（错位读）⇒ `Ok(None)`；
/// 自证过、落点对 ⇒ 那条根。
fn root_read_once_from_the_ring_slot(
    reader: &dyn PoolReader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &crate::system_configuration::SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slot: RootRingSlot,
) -> Result<Option<RootRecord>, NoRowDeletedThisRound> {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
    let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
        return Err(NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero { ring_slot });
    };
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero { ring_slot });
    }
    // 核落点公式（D16（发布语义） 已定项 6 骑手条款 1「恢复与挂载读根环时都调」，C973（实例表清扫读根环不核落点））：自证得过、却不住在它的 txg 该住的那一槽的根
    // 是错位读（第 0 代根照 mkfs 种根的槽另认），照这一槽这一次自证不过记——不核的话，错位读把一个实例在环里唯一的根换成别槽的根，
    // 那个实例就成了「环里没有根」的候选行被删掉。
    Ok(
        RootRecord::parse_slot(&bytes, filesystem_identifier).filter(|root| {
            root_sits_in_the_ring_slot_the_formula_gives(
                root.checkpoint_txg,
                ring_slot,
                immutable_sizes.root_ring_slots_per_region,
            )
        }),
    )
}

/// 住在 `ring_slot` 的是第 0 代根，而这一遍读出的环里最大的 txg 已到发布路径按 [`target_for_publish`] 写进这一槽的某个 txg
/// （C973（实例表清扫读根环不核落点）；D18（块里携带什么信息） 已定项 11「其余的槽须自证过」）。落点核按 mkfs 种根的槽认第 0 代根
/// （[`root_sits_in_the_ring_slot_the_formula_gives`]），三个区域的槽 0 里同字节的三份，错位读从别的槽 0 读回第 0 代根它核不出来；
/// 发布路径已写到这一槽之后，这一槽的合法内容不再是 mkfs 种的那一份，读出第 0 代根就当这一槽这一次自证不过。
/// 只读调用方给的这一遍读数（最大 txg），不读别的状态。
///
/// 循环：txg 从第 0 代根之后一个起往上至多走环长 3S 个（连续 3S 个 txg 每槽恰好落一个，再往上只重复）、不超过这一遍的最大 txg；
/// 不跨轮携带状态；提前出口是碰到落进这一槽的那一个。
fn genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached(
    checkpoint_txg: CheckpointTxg,
    ring_slot: RootRingSlot,
    highest_txg_of_the_pass: CheckpointTxg,
    slots_per_region: RootRingSlotsPerRegion,
    ring_length_in_slots: usize,
) -> bool {
    if checkpoint_txg != GENESIS_CHECKPOINT_TXG {
        return false;
    }
    (GENESIS_CHECKPOINT_TXG.0 + 1..=highest_txg_of_the_pass.0)
        .take(ring_length_in_slots)
        .any(|txg| target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot)
}

/// 抹头那一步与清扫发布走到哪儿。
#[derive(Debug)]
pub enum InstanceTableSweepOutcome {
    /// 判的那一半就停了，一个头都没抹。
    NoRowDeleted(NoRowDeletedThisRound),
    /// 抹完头、清扫发布落盘：删了这几行。`heads_erased` 是抹了的头，就是判的那一半要抹的全部（抹之前那一遍重读有一个不是判的那个头时
    /// 这一轮不删，走 [`NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit`]）。
    RowsDeleted {
        rows_deleted: Vec<InstanceRow>,
        heads_erased: Vec<PlacementOfAUnitHead>,
    },
    /// 抹完头，清扫发布的落点被拒（`PlacementRefused` / `SpaceAdmissionRefused`）：这一轮不删行，挂载照样交回可写（Q4）。一个发布的写都没发。
    SweepPublishRefusedForSpace {
        heads_erased: Vec<PlacementOfAUnitHead>,
        refusal: Box<PublishError>,
    },
    /// 抹头的一个写或屏障报块设备错：这一轮不删行（不发清扫发布），已抹的留着——它们按那一行判本来就未发布。
    /// 条款只写了落点被拒那一格，块设备错怎么处置没写（主 agent 交回报告里列着）；这里照落点被拒那一格不删、交回可写。
    StoppedByABlockDeviceErrorWhileErasing {
        heads_erased: Vec<PlacementOfAUnitHead>,
        error: BlockDeviceError,
    },
    /// 清扫发布落盘途中报错（或在落点之外的理由上被拒）：这一轮不删行；落盘途中报的那一次照发布路径冻结在分配器上等原样重发
    /// （`transaction::publish_version` 的契约）。条款同样没写这一格，处置同上一个成员。
    SweepPublishFailed {
        heads_erased: Vec<PlacementOfAUnitHead>,
        cause: Box<PublishError>,
    },
}

/// 抹头、过屏障、发清扫发布（判之后那一半）。抹之前在同一串行段里把要抹的头全部重读一遍：每一个都仍是判的那一刻读出来的那个单元
/// （类、写序实例、诞生代号、事务号都相同、仍按候选行判未发布）才开始抹；有一个报设备错或不是那个头，这一轮不删、一个头都不抹
/// （用户 2026-10-04 定 C；F1 原来是「那一个不抹、其余照抹照删」，判完到抹之间有分配写掉了那一槽也走这一支；挂载末尾那一支两步之间没有别的写，
/// 重读只多读一次）。
/// 清扫发布接在 `current` 后面：同一实例、事务号 0、不进 W、实例表整链重写成「这一刻现行那一版的表去掉要删的行」、不判挂载准入式子
/// （一个普通分配都没有的发布不判，`transaction::prepare_the_version_publish`）。发成了 `current` 换成清扫发布那一版。
pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    judgement: &InstanceTableSweepJudgement,
    sweep_publish_placements: SweepPublishPlacements,
) -> InstanceTableSweepOutcome {
    let current_file_version = match current {
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            version.clone()
        }
        PoolVersion::WithoutFile(_) => {
            return InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
            )
        }
    };
    let physical_block_bytes = usize::try_from(parameters.geometry.physical_block_size)
        .expect("物理块的字节数装得进 usize");
    let head_bytes = usize::try_from(
        HEAD_BYTES_READ_PER_SLOT.max(u64::from(parameters.geometry.physical_block_size)),
    )
    .expect("读头的字节数装得进 usize");
    let rows_by_instance: BTreeMap<InstanceGeneration, InstanceRow> = judgement
        .rows_to_delete
        .iter()
        .map(|row| (row.instance, *row))
        .collect();
    // 先重读、后抹（C）：迭代上界是要抹的头数；不跨轮携带状态；提前出口是读失败与不是那个头两种（这一轮不删）。
    for to_erase in &judgement.heads_to_erase {
        let offset = to_erase.placement.slot.to_device_offset();
        let Some(reread_bytes) =
            PoolReader::read(&*devices, to_erase.placement.device, offset, head_bytes)
        else {
            return InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
                    placement: to_erase.placement,
                    reread: HeadRereadBeforeErasing::ReadFailed,
                },
            );
        };
        let still_the_unpublished_unit_judged =
            unit_head_read_by_sweep(&reread_bytes).is_some_and(|head| {
                head == to_erase.head
                    && rows_by_instance
                        .get(&head.writer_instance)
                        .is_some_and(|row| head.is_unpublished_by_the_row(row))
            });
        if !still_the_unpublished_unit_judged {
            return InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
                    placement: to_erase.placement,
                    reread: HeadRereadBeforeErasing::NotTheJudgedUnit,
                },
            );
        }
    }
    let mut heads_erased = Vec::new();
    // 迭代上界是要抹的头数；跨轮携带的只有抹了的那几个；提前出口是块设备错。
    for to_erase in &judgement.heads_to_erase {
        let offset = to_erase.placement.slot.to_device_offset();
        let Some((_, device)) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == to_erase.placement.device)
        else {
            continue;
        };
        if let Err(error) = device.write_at(
            offset,
            &vec![0u8; physical_block_bytes],
            WriteDurability::ForceUnitAccess,
        ) {
            return InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                heads_erased,
                error,
            };
        }
        heads_erased.push(to_erase.placement);
    }
    // 抹头全部 FUA 返回之后每块盘一道屏障，之后才发清扫发布（D18 已定项 11「删行落在一次专用的清扫发布上」）。
    for (_, device) in devices.iter_mut() {
        if let Err(error) = device.barrier() {
            return InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                heads_erased,
                error,
            };
        }
    }
    let chain = match instance_table_chain_of_root(&*devices, &current_file_version.root) {
        Ok(chain) => chain,
        Err(failure) => {
            return InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::InstanceTableOfTheCurrentVersionUnreadable { failure },
            )
        }
    };
    let instances_to_delete: BTreeSet<InstanceGeneration> =
        rows_by_instance.keys().copied().collect();
    let rows_kept: Vec<InstanceRow> = chain
        .records
        .rows
        .iter()
        .copied()
        .filter(|row| !instances_to_delete.contains(&row.instance))
        .collect();
    let plan = sweep_publish_plan(
        &current_file_version,
        InstanceTableRewrite {
            rows: rows_kept,
            replaced_chain: chain.page_pointers,
        },
    );
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let published = match sweep_publish_placements {
        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount => publish_version(
            &mut pool,
            allocator,
            plan.as_publish_plan(),
            Some(Arc::as_ref(&current_file_version)),
        ),
        SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch => {
            let mut allocator_without_a_free_slot = allocator.clone();
            take_every_free_slot_away(&mut allocator_without_a_free_slot);
            match prepare_the_version_publish(
                &pool,
                &mut allocator_without_a_free_slot,
                &plan.as_publish_plan(),
                Some(Arc::as_ref(&current_file_version)),
                current_file_version.tree_identifiers,
                ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
                UnmountMarker::NotWrittenByTheUnmountSequence,
                &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
            ) {
                Err(refusal) => Err(refusal),
                Ok(_) => panic!(
                    "只供测试的开关把每块盘每个空闲槽都占掉了，清扫发布要的实例表片取不到落点：拷贝上取到了说明这个开关没把那一支强制进去"
                ),
            }
        }
    };
    match published {
        Ok(sweep_publish) => {
            *current = PoolVersion::of_a_version_of_trees(sweep_publish);
            InstanceTableSweepOutcome::RowsDeleted {
                rows_deleted: judgement.rows_to_delete.clone(),
                heads_erased,
            }
        }
        Err(
            refusal @ (PublishError::PlacementRefused { .. }
            | PublishError::SpaceAdmissionRefused(_)),
        ) => InstanceTableSweepOutcome::SweepPublishRefusedForSpace {
            heads_erased,
            refusal: Box::new(refusal),
        },
        Err(cause) => InstanceTableSweepOutcome::SweepPublishFailed {
            heads_erased,
            cause: Box::new(cause),
        },
    }
}

/// 清扫发布的计划（它的那几样都从现行那一版来，实例表整链重写）。
struct SweepPublishPlan {
    txg: CheckpointTxg,
    counter: u64,
    highest_transaction_number_before_this_publish: u64,
    instance: InstanceGeneration,
    back_chain: u32,
    instance_table: InstanceTableRewrite,
    tree_birth_txg: CheckpointTxg,
    tree_identifier_watermark: u64,
    rollback_floor: CheckpointTxg,
}

impl SweepPublishPlan {
    fn as_publish_plan(&self) -> PublishPlan<'_> {
        PublishPlan {
            txg: self.txg,
            counter: self.counter,
            // 事务号 0、不进 W（D18 已定项 11「清扫发布的形状」）。
            transaction: 0,
            highest_transaction_number_before_this_publish: self
                .highest_transaction_number_before_this_publish,
            instance: self.instance,
            back_chain: self.back_chain,
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Rewrite(self.instance_table.clone()),
            tree_birth_txg: self.tree_birth_txg,
            tree_identifier_watermark: self.tree_identifier_watermark,
            rollback_floor: self.rollback_floor,
        }
    }
}

fn sweep_publish_plan(
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
) -> SweepPublishPlan {
    SweepPublishPlan {
        txg: CheckpointTxg(
            previous
                .root
                .checkpoint_txg
                .0
                .checked_add(1)
                .expect("现行那一版的 txg 不在 u64 顶上：它是这个进程刚发成的"),
        ),
        counter: previous.record.counter + 1,
        highest_transaction_number_before_this_publish: previous
            .highest_transaction_number_in_this_instance,
        instance: previous.root.instance,
        back_chain: back_chain_of(&previous.record_bytes),
        instance_table,
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    }
}

/// 只供测试的开关那一支：把分配器（一份拷贝）上每块盘每个空闲槽都占掉。
fn take_every_free_slot_away(allocator: &mut PoolAllocator) {
    for device_map in &mut allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        // 迭代上界是这块盘单元区的槽数；没有提前出口。
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 可写挂载末尾那一判（`mount::establish_instance` 交回之前调）的结局。
#[derive(Debug)]
pub enum InstanceTableSweepAtTheEndOfTheMount {
    /// 开关关着、行数没超过阈值：不扫。
    NotSweptAtOrBelowTheThreshold {
        rows: u64,
        threshold: InstanceTableRowCountThreshold,
    },
    /// 开关开着：交给挂着时的后台清扫。
    HandedToTheBackgroundSweepWhileMounted,
    /// 开关关着、行数超过阈值：这次挂载里判、抹、发的结局。
    Swept(InstanceTableSweepOutcome),
}

/// 可写挂载末尾那一判：开关关着、行数超过阈值就立刻判、抹、发；否则不扫。`current` 是续做之后的现行那一版，清扫发布发成了换成它。
pub fn sweep_the_instance_table_at_the_end_of_the_mount<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    configuration: InstanceTableSweepConfiguration,
) -> InstanceTableSweepAtTheEndOfTheMount {
    match configuration.switch {
        InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted => {
            return InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
        }
        InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold => {}
    }
    let rows = match instance_table_chain_of_root(&*devices, current.root()) {
        Ok(chain) => u64::try_from(chain.records.rows.len()).expect("行数装得进 u64"),
        Err(failure) => {
            return InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::NoRowDeleted(
                    NoRowDeletedThisRound::InstanceTableOfTheCurrentVersionUnreadable { failure },
                ),
            )
        }
    };
    if rows <= configuration.row_count_threshold.0 {
        return InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold {
            rows,
            threshold: configuration.row_count_threshold,
        };
    }
    InstanceTableSweepAtTheEndOfTheMount::Swept(sweep_the_instance_table_while_mounted(
        parameters,
        devices,
        allocator,
        current,
        configuration.sweep_publish_placements,
    ))
}

/// 一整轮清扫（后台那一支的入口，挂载末尾那一支也走它）：判、抹（抹之前重读）、发。`current` 是现行那一版，清扫发布发成了换成它。
/// 现行那一版树表 0 条时在判之前就交回不支持（一个字节都不写）。
pub fn sweep_the_instance_table_while_mounted<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    sweep_publish_placements: SweepPublishPlacements,
) -> InstanceTableSweepOutcome {
    match current {
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
        PoolVersion::WithoutFile(_) => {
            return InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
            )
        }
    }
    let judgement = match judge_the_instance_table_sweep(&*devices, current.root()) {
        Ok(judgement) => judgement,
        Err(no_row_deleted) => return InstanceTableSweepOutcome::NoRowDeleted(no_row_deleted),
    };
    erase_the_heads_and_publish_the_sweep(
        parameters,
        devices,
        allocator,
        current,
        &judgement,
        sweep_publish_placements,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_unit_head(instance: u32, birth_txg: u64, transaction: u64) -> UnitHeadReadBySweep {
        UnitHeadReadBySweep {
            unit_class: UNIT_CLASS_DATA,
            writer_instance: InstanceGeneration(instance),
            birth_txg: CheckpointTxg(birth_txg),
            transaction,
            filesystem_identifier: 0,
        }
    }

    fn row(
        instance: u32,
        selected_root_txg: u64,
        applied_transaction_high_water: u64,
    ) -> InstanceRow {
        InstanceRow {
            instance: InstanceGeneration(instance),
            selected_root_txg: CheckpointTxg(selected_root_txg),
            applied_transaction_high_water,
        }
    }

    /// 已发布谓词有行那一支：码 1 看诞生代号与事务号两样，码 2 / 3 只看诞生代号（D18 已定项 11）。
    #[test]
    fn the_published_predicate_of_a_row_reads_birth_and_transaction_for_data_units_and_birth_only_otherwise(
    ) {
        let the_row = row(1, 10, 7);
        assert!(
            !data_unit_head(1, 10, 9).is_unpublished_by_the_row(&the_row),
            "诞生代号 ≤ T：已发布"
        );
        assert!(
            !data_unit_head(1, 11, 7).is_unpublished_by_the_row(&the_row),
            "事务号 ≤ W：已发布"
        );
        assert!(
            data_unit_head(1, 11, 8).is_unpublished_by_the_row(&the_row),
            "两样都越过：未发布"
        );
        let node = UnitHeadReadBySweep {
            unit_class: UNIT_CLASS_INDEX_NODE,
            ..data_unit_head(1, 11, 0)
        };
        assert!(
            node.is_unpublished_by_the_row(&the_row),
            "码 2 只看诞生代号"
        );
    }

    /// 认领规则按整单元跳：码 1、码 3 两槽，码 2 一槽（S2-2）。
    #[test]
    fn a_claimed_unit_spans_two_slots_for_data_and_packed_units_and_one_slot_for_nodes() {
        assert_eq!(data_unit_head(1, 1, 1).span_in_slots(), 2);
        assert_eq!(
            UnitHeadReadBySweep {
                unit_class: UNIT_CLASS_PACKED,
                ..data_unit_head(1, 1, 1)
            }
            .span_in_slots(),
            2
        );
        assert_eq!(
            UnitHeadReadBySweep {
                unit_class: UNIT_CLASS_INDEX_NODE,
                ..data_unit_head(1, 1, 1)
            }
            .span_in_slots(),
            1
        );
    }

    /// 全零的槽、魔数不对的槽不是单元（已抹头那一形）。
    #[test]
    fn an_all_zero_head_and_a_head_without_the_magic_are_not_units() {
        assert_eq!(unit_head_read_by_sweep(&[0u8; 1024]), None);
        let mut without_magic = [0u8; 1024];
        without_magic[..4].copy_from_slice(b"SFSX");
        without_magic[6] = UNIT_CLASS_DATA;
        assert_eq!(unit_head_read_by_sweep(&without_magic), None);
    }
}

/// C973（实例表清扫读根环不核落点）的盘面：实例 1 在根环里只剩一条根，读它那一槽的读错位读回别槽里一条自证得过的实例 2 的根。
#[cfg(test)]
mod misplaced_ring_reads_tests {
    use super::*;
    use crate::address::DeviceOffsetInBytes;
    use crate::block_device::PhysicalBlockSizeInBytes;
    use crate::make_filesystem::make_filesystem;
    use crate::mount::mount_writable;
    use crate::recovery::{read_root_ring_slot, RootRingSlotReading};
    use crate::root_ring::{target_for_publish, RootRingSlotsPerRegion};
    use crate::system_configuration::SystemImmutableSizes;
    use crate::transaction::{publish_first_file, publish_overwrite, FirstFile};
    use std::cell::Cell;

    /// 每块盘的字节数：128 MiB（同验证包根环空洞那几组池），清扫扫池那一遍只走八千来个槽。
    const DEVICE_BYTES: u64 = 128 << 20;
    /// 稀疏盘一页的字节数。
    const PAGE_BYTES: u64 = 4096;
    /// 每区槽数 S = 4：环长 12，十来次发布就把实例 1 的根轮得只剩一条。
    const SLOTS_PER_REGION: u64 = 4;
    /// 第一个文件的写入时刻；之后每一版晚一秒。
    const WRITE_TIME_SECONDS_OF_THE_EARLIEST_VERSION: u64 = 1_788_000_000;
    /// 每一版文件内容的字节数（一个数据单元装得下）。
    const CONTENT_BYTES: usize = 3000;
    /// 实例 2 至多覆盖写几次去把实例 1 的根轮到只剩一条。
    const OVERWRITES_ROTATING_THE_RING_AT_MOST: u64 = 64;

    /// 只在内存里的盘：写过的页才存，没写过的读回全零；越过末尾报越界。
    struct DeviceOfPagesInMemory {
        pages: BTreeMap<u64, Vec<u8>>,
    }

    impl DeviceOfPagesInMemory {
        fn refuse_past_the_end(
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            if offset.0 + length > DEVICE_BYTES {
                return Err(BlockDeviceError::OutOfRange {
                    offset,
                    length,
                    device_size: DEVICE_BYTES,
                });
            }
            Ok(())
        }

        fn page_index_and_offset_in_page(address: u64) -> (u64, usize) {
            (
                address / PAGE_BYTES,
                usize::try_from(address % PAGE_BYTES).expect("页内偏移装得进 usize"),
            )
        }
    }

    impl BlockDevice for DeviceOfPagesInMemory {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let length = u64::try_from(buffer.len()).expect("长度装得进 u64");
            Self::refuse_past_the_end(offset, length)?;
            for (index_in_buffer, byte) in buffer.iter_mut().enumerate() {
                let (page_index, index_in_page) = Self::page_index_and_offset_in_page(
                    offset.0 + u64::try_from(index_in_buffer).expect("下标装得进 u64"),
                );
                *byte = self
                    .pages
                    .get(&page_index)
                    .map_or(0, |page| page[index_in_page]);
            }
            Ok(())
        }
        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let length = u64::try_from(bytes.len()).expect("长度装得进 u64");
            Self::refuse_past_the_end(offset, length)?;
            for (index_in_bytes, byte) in bytes.iter().enumerate() {
                let (page_index, index_in_page) = Self::page_index_and_offset_in_page(
                    offset.0 + u64::try_from(index_in_bytes).expect("下标装得进 u64"),
                );
                self.pages
                    .entry(page_index)
                    .or_insert_with(|| vec![0; usize::try_from(PAGE_BYTES).expect("4096")])
                    [index_in_page] = *byte;
            }
            Ok(())
        }
        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            Self::refuse_past_the_end(offset, length)?;
            let end = offset.0 + length;
            let pages_touched: Vec<u64> = self
                .pages
                .range(offset.0 / PAGE_BYTES..end.div_ceil(PAGE_BYTES))
                .map(|(page_index, _)| *page_index)
                .collect();
            for page_index in pages_touched {
                let page_start = page_index * PAGE_BYTES;
                let zeroed_from = offset.0.max(page_start) - page_start;
                let zeroed_to = end.min(page_start + PAGE_BYTES) - page_start;
                let page = self.pages.get_mut(&page_index).expect("刚从表里列出的页");
                page[usize::try_from(zeroed_from).expect("页内偏移")
                    ..usize::try_from(zeroed_to).expect("页内偏移")]
                    .fill(0);
            }
            Ok(())
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            PhysicalBlockSizeInBytes(512)
        }
        fn size_in_bytes(&self) -> u64 {
            DEVICE_BYTES
        }
    }

    type DevicesInMemory = Vec<(DeviceIdentity, DeviceOfPagesInMemory)>;

    fn parameters() -> MakeFilesystemParameters {
        MakeFilesystemParameters {
            filesystem_identifier: [7; 16],
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: 1 << 20,
                root_ring_slots_per_region:
                    RootRingSlotsPerRegion::from_system_configuration_field(SLOTS_PER_REGION)
                        .expect("S = 4 在区间里"),
            },
        }
    }

    /// 第 `txg` 版的文件内容（各版不同）。
    fn content_of_the_version(txg: u64) -> Vec<u8> {
        (0..CONTENT_BYTES)
            .map(|index| {
                let index = u64::try_from(index).expect("长度装得进 u64");
                u8::try_from((index * 131 + txg * 7 + 1) % 251).expect("小于 256")
            })
            .collect()
    }

    /// 根环每一槽照恢复的读法（核自证与落点公式）读一遍：自证过、落点对的根连同它住的槽。
    fn roots_of_the_ring_by_slot(
        devices: &DevicesInMemory,
    ) -> Vec<(RootRingSlot, Option<RootRecord>)> {
        let parameters = parameters();
        every_root_ring_slot(&parameters.geometry)
            .into_iter()
            .map(|ring_slot| {
                let reading = read_root_ring_slot(
                    devices,
                    &parameters.region_devices,
                    &parameters.geometry,
                    &parameters.filesystem_identifier,
                    ring_slot,
                );
                let root = match reading {
                    RootRingSlotReading::SelfVerified(root) => Some(root),
                    RootRingSlotReading::Bad(_) => None,
                };
                (ring_slot, root)
            })
            .collect()
    }

    /// 清扫判之前的盘面：两块盘、现行那一版的根、实例 1 在环里唯一那条根住的槽、一个住着实例 2 自证过的根的别的槽。
    struct PoolWithAnInstanceDownToOneRingRoot {
        devices: DevicesInMemory,
        current_root: RootRecord,
        instance_rotated_out: InstanceGeneration,
        ring_slot_of_the_only_root_of_the_instance: RootRingSlot,
        ring_slot_of_a_root_of_the_rotating_instance: RootRingSlot,
    }

    /// mkfs → 实例 1 可写挂载、第一个文件、两次覆盖写（不卸载）→ 实例 2 可写挂载、覆盖写到根环每一槽都住着自证过的根、
    /// 实例 1 的根只剩一条为止。实例表里实例 1 那一行还在（两次挂载末尾都没清扫：行数没越过默认阈值）。
    fn pool_with_an_instance_down_to_one_ring_root() -> PoolWithAnInstanceDownToOneRingRoot {
        let parameters = parameters();
        let mut devices: DevicesInMemory = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    DeviceOfPagesInMemory {
                        pages: BTreeMap::new(),
                    },
                )
            })
            .collect();
        make_filesystem(&parameters, &mut devices).expect("128 MiB 两盘、S = 4 建得成");
        let instance_rotated_out = {
            let mounted = mount_writable(&parameters, &mut devices).expect("实例 1 的可写挂载");
            let instance_rotated_out = mounted.output.instance;
            let mut allocator = mounted.allocator;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let mut current = publish_first_file(
                &mut writer,
                &mut allocator,
                mounted.current.root(),
                FirstFile {
                    content: &content_of_the_version(0),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_EARLIEST_VERSION,
                },
                instance_rotated_out,
                mounted.current.record_bytes(),
            )
            .expect("实例 1 的第一个文件");
            // 迭代上界：两次覆盖写。
            for _ in 0..2 {
                let txg = current.root.checkpoint_txg.0 + 1;
                current = publish_overwrite(
                    &mut writer,
                    &mut allocator,
                    &current,
                    FirstFile {
                        content: &content_of_the_version(txg),
                        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_EARLIEST_VERSION + txg,
                    },
                    instance_rotated_out,
                )
                .expect("实例 1 的覆盖写");
            }
            instance_rotated_out
        };
        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
        let instance_rotating_the_ring = mounted.output.instance;
        assert!(
            instance_rotating_the_ring > instance_rotated_out,
            "前提：实例 2 取的号比实例 1 大"
        );
        let mut allocator = mounted.allocator;
        let mut current = mounted
            .current
            .into_file_version()
            .expect("前提：实例 2 挂上的那一版带文件");
        let ring_slots_of_the_rotated_out_instance =
            |devices_read_now: &DevicesInMemory| -> Option<Vec<RootRingSlot>> {
                let roots_by_slot = roots_of_the_ring_by_slot(devices_read_now);
                if roots_by_slot.iter().any(|(_, root)| root.is_none()) {
                    return None;
                }
                Some(
                    roots_by_slot
                        .iter()
                        .filter(|(_, root)| {
                            root.is_some_and(|resident| resident.instance == instance_rotated_out)
                        })
                        .map(|(ring_slot, _)| *ring_slot)
                        .collect(),
                )
            };
        // 迭代上界：实例 2 至多覆盖写 OVERWRITES_ROTATING_THE_RING_AT_MOST 次；跨轮携带的只有现行那一版；
        // 提前出口是根环每一槽都住着自证过的根、实例 1 的根只剩一条。
        for _ in 0..OVERWRITES_ROTATING_THE_RING_AT_MOST {
            if ring_slots_of_the_rotated_out_instance(&devices)
                .is_some_and(|ring_slots| ring_slots.len() == 1)
            {
                break;
            }
            let txg = current.root.checkpoint_txg.0 + 1;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(txg),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_EARLIEST_VERSION + txg,
                },
                instance_rotating_the_ring,
            )
            .expect("实例 2 的覆盖写");
        }
        let ring_slots = ring_slots_of_the_rotated_out_instance(&devices)
            .expect("前提：根环每一槽都住着自证过、落点对的根");
        assert_eq!(
            ring_slots.len(),
            1,
            "前提：实例 1 在根环里只剩一条根（住在 {ring_slots:?}）"
        );
        let rows = instance_table_chain_of_root(&devices, &current.root)
            .expect("前提：现行那一版的实例表读得出")
            .records
            .rows;
        assert!(
            rows.iter().any(|row| row.instance == instance_rotated_out),
            "前提：现行那一版的实例表里实例 1 那一行还在"
        );
        let ring_slot_of_a_root_of_the_rotating_instance = target_for_publish(
            current.root.checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        assert_ne!(
            ring_slot_of_a_root_of_the_rotating_instance, ring_slots[0],
            "前提：现行那一版的根与实例 1 那条根不住同一槽"
        );
        PoolWithAnInstanceDownToOneRingRoot {
            devices,
            current_root: current.root,
            instance_rotated_out,
            ring_slot_of_the_only_root_of_the_instance: ring_slots[0],
            ring_slot_of_a_root_of_the_rotating_instance,
        }
    }

    /// 读者：读 `misplaced` 那一槽的前 `misplaced_reads_left` 次读回 `read_back_instead` 那一槽的字节（错位读），别的读原样交给盘。
    struct PoolReadingOneRingSlotMisplaced<'pool> {
        devices: &'pool DevicesInMemory,
        misplaced: (DeviceIdentity, DeviceOffsetInBytes),
        read_back_instead: (DeviceIdentity, DeviceOffsetInBytes),
        misplaced_reads_left: Cell<u32>,
    }

    impl<'pool> PoolReadingOneRingSlotMisplaced<'pool> {
        fn over(pool: &'pool PoolWithAnInstanceDownToOneRingRoot, misplaced_reads: u32) -> Self {
            let parameters = parameters();
            let device_and_offset_of = |ring_slot: RootRingSlot| {
                (
                    parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                    slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing),
                )
            };
            Self {
                devices: &pool.devices,
                misplaced: device_and_offset_of(pool.ring_slot_of_the_only_root_of_the_instance),
                read_back_instead: device_and_offset_of(
                    pool.ring_slot_of_a_root_of_the_rotating_instance,
                ),
                misplaced_reads_left: Cell::new(misplaced_reads),
            }
        }
    }

    impl PoolReader for PoolReadingOneRingSlotMisplaced<'_> {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            PoolReader::device_identities(self.devices)
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            PoolReader::device_size_in_bytes(self.devices, device)
        }
        fn read(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Option<Vec<u8>> {
            if (device, offset) == self.misplaced && self.misplaced_reads_left.get() > 0 {
                self.misplaced_reads_left
                    .set(self.misplaced_reads_left.get() - 1);
                let (device_read_instead, offset_read_instead) = self.read_back_instead;
                return PoolReader::read(
                    self.devices,
                    device_read_instead,
                    offset_read_instead,
                    length,
                );
            }
            PoolReader::read(self.devices, device, offset, length)
        }
        fn journal_record_offsets_hint(
            &self,
            device: DeviceIdentity,
            ring_start: DeviceOffsetInBytes,
            ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            PoolReader::journal_record_offsets_hint(self.devices, device, ring_start, ring_bytes)
        }
    }

    /// 场景（C973 验收）：实例 1 在根环里只剩一条根，读它那一槽的两次读（第一次与重读）都错位读回实例 2 住在别槽的一条自证得过的根。
    /// 预期：清扫那一遍不删实例 1 那一行——错位读照落点公式核出来当这一槽自证不过，重读一次仍不过，这一轮一行都不删
    /// （D16（发布语义） 已定项 6 骑手条款 1「恢复与挂载读根环时都调」）。
    /// 判别力：`crates/mutations.tsv`「C973 清扫读根环不核落点公式」那一行，这一条红在 `Ok` 那一支（实例 1 那一行判成要删）。
    #[test]
    fn a_misplaced_read_replacing_the_only_ring_root_of_an_instance_on_both_reads_deletes_no_row() {
        let pool = pool_with_an_instance_down_to_one_ring_root();
        let reader = PoolReadingOneRingSlotMisplaced::over(&pool, 2);
        match judge_the_instance_table_sweep(&reader, &pool.current_root) {
            Err(NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread {
                ring_slot,
            }) => {
                assert_eq!(
                    ring_slot, pool.ring_slot_of_the_only_root_of_the_instance,
                    "停在错位读的那一槽"
                );
            }
            Ok(judgement) => panic!(
                "错位读把实例 1 唯一的根换成了别槽的根，清扫判出要删的行 {:?}",
                judgement.rows_to_delete
            ),
            Err(other) => panic!("这一轮该停在错位读的那一槽，停在了 {other:?}"),
        }
        assert_eq!(
            reader.misplaced_reads_left.get(),
            0,
            "那一槽读了两次（第一次与重读），两次都错位"
        );
    }

    /// 场景：同一张盘面，只有第一次读错位，重读读回那一槽真住着的实例 1 的根。
    /// 预期：实例 1 在环里有根，那一行不是候选行（阳性对照：落点核只让错位那一读作废，重读读对了照认那条根）。
    /// 判别力：`crates/mutations.tsv`「C973 清扫读根环自证不过不重读」那一行，这一条红（第一次错位就停，不重读）；
    /// 「C973 清扫读根环不核落点公式」那一行同样红（错位那一读当自证过，实例 1 那一行判成要删）。
    #[test]
    fn a_misplaced_read_of_the_only_ring_root_of_an_instance_reread_right_keeps_its_row() {
        let pool = pool_with_an_instance_down_to_one_ring_root();
        let reader = PoolReadingOneRingSlotMisplaced::over(&pool, 1);
        let instance_rotated_out = pool.instance_rotated_out;
        match judge_the_instance_table_sweep(&reader, &pool.current_root) {
            Err(NoRowDeletedThisRound::NoCandidateRow) => {}
            Ok(judgement) => assert!(
                judgement
                    .rows_to_delete
                    .iter()
                    .all(|row| row.instance != instance_rotated_out),
                "重读读回了实例 1 的根，那一行不该判成要删：{:?}",
                judgement.rows_to_delete
            ),
            Err(other) => panic!("重读读对了，这一轮不该停在 {other:?}"),
        }
        assert_eq!(reader.misplaced_reads_left.get(), 0, "第一次读错位过了");
    }

    /// 点名的那一次写落盘之后就断电：它与之前的写都已落盘，之后两块盘都不再落任何写（同一个「断电了没有」两块盘共用）。
    struct DeviceLosingEveryWriteAfterAPowerCut {
        memory: DeviceOfPagesInMemory,
        identity: DeviceIdentity,
        power_cut_after_the_write_to: (DeviceIdentity, DeviceOffsetInBytes),
        power_cut: std::rc::Rc<Cell<bool>>,
    }

    impl BlockDevice for DeviceLosingEveryWriteAfterAPowerCut {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            self.memory.read_at(offset, buffer)
        }
        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            if self.power_cut.get() {
                return Ok(());
            }
            self.memory.write_at(offset, bytes, durability)?;
            if (self.identity, offset) == self.power_cut_after_the_write_to {
                self.power_cut.set(true);
            }
            Ok(())
        }
        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            if self.power_cut.get() {
                return Ok(());
            }
            self.memory.write_zeroes_at(offset, length)
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            self.memory.probe_physical_block_size()
        }
        fn size_in_bytes(&self) -> u64 {
            self.memory.size_in_bytes()
        }
    }

    /// mkfs → 实例 1 可写挂载，写行 txg 1 那次发布写区域 1 槽 0 的根槽落盘之后就断电（系统配置轮换与暖机 txg 2 都丢；同
    /// E205（多处读坏倒挂三条改法的代价） 起点镜像「根槽 FUA 持久、系统配置轮换没持久」那一形）→ 实例 2 可写挂载、新池新建文件、覆盖写到根环每一槽都住着自证过的根为止
    /// （S = 4：区域 0 槽 0 要到 txg 12 才轮到）。
    /// 盘面：区域 0 槽 0 仍是 mkfs 种的第 0 代根；实例 1 在环里只剩写行那条根（txg 1，区域 1 槽 0，也是 mkfs 种过第 0 代根的槽）；
    /// 实例表里实例 1 那一行还在。`ring_slot_of_a_root_of_the_rotating_instance` 这里放区域 0 槽 0（错位读回第 0 代根）。
    fn pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem(
    ) -> PoolWithAnInstanceDownToOneRingRoot {
        let parameters = parameters();
        let mut devices_made_by_make_filesystem: DevicesInMemory =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        DeviceOfPagesInMemory {
                            pages: BTreeMap::new(),
                        },
                    )
                })
                .collect();
        make_filesystem(&parameters, &mut devices_made_by_make_filesystem)
            .expect("128 MiB 两盘、S = 4 建得成");
        let region_one_slot_zero_written_last = RootRingSlot { region: 1, slot: 0 };
        let power_cut = std::rc::Rc::new(Cell::new(false));
        let power_cut_after_the_write_to = (
            parameters.region_devices[1],
            slot_offset(
                region_one_slot_zero_written_last,
                parameters.geometry.fixed_structure_slot_spacing,
            ),
        );
        let mut devices_losing_writes: Vec<(DeviceIdentity, DeviceLosingEveryWriteAfterAPowerCut)> =
            devices_made_by_make_filesystem
                .into_iter()
                .map(|(identity, memory)| {
                    (
                        identity,
                        DeviceLosingEveryWriteAfterAPowerCut {
                            memory,
                            identity,
                            power_cut_after_the_write_to,
                            power_cut: power_cut.clone(),
                        },
                    )
                })
                .collect();
        drop(mount_writable(&parameters, &mut devices_losing_writes));
        assert!(
            power_cut.get(),
            "前提：实例 1 的挂载写到了区域 1 槽 0 那一次根槽写（写行 txg 1）"
        );
        let mut devices: DevicesInMemory = devices_losing_writes
            .into_iter()
            .map(|(identity, device)| (identity, device.memory))
            .collect();
        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
        let instance_of_the_mount = mounted.output.instance;
        let mut allocator = mounted.allocator;
        let mut current = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_first_file(
                &mut writer,
                &mut allocator,
                mounted.current.root(),
                FirstFile {
                    content: &content_of_the_version(0),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_EARLIEST_VERSION,
                },
                instance_of_the_mount,
                mounted.current.record_bytes(),
            )
            .expect("实例 2 的新池新建文件")
        };
        // 迭代上界：至多 OVERWRITES_ROTATING_THE_RING_AT_MOST 次；跨轮携带的只有现行那一版；提前出口是根环每一槽都住着自证过的根。
        for _ in 0..OVERWRITES_ROTATING_THE_RING_AT_MOST {
            if roots_of_the_ring_by_slot(&devices)
                .iter()
                .all(|(_, root)| root.is_some())
            {
                break;
            }
            let txg = current.root.checkpoint_txg.0 + 1;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(txg),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_EARLIEST_VERSION + txg,
                },
                instance_of_the_mount,
            )
            .expect("实例 2 的覆盖写");
        }
        let roots_by_slot = roots_of_the_ring_by_slot(&devices);
        assert!(
            roots_by_slot.iter().all(|(_, root)| root.is_some()),
            "前提：根环每一槽都住着自证过、落点对的根：{roots_by_slot:?}"
        );
        let region_zero_slot_zero = RootRingSlot { region: 0, slot: 0 };
        let region_one_slot_zero = RootRingSlot { region: 1, slot: 0 };
        let root_at = |ring_slot: RootRingSlot| {
            roots_by_slot
                .iter()
                .find(|(resident_slot, _)| *resident_slot == ring_slot)
                .and_then(|(_, root)| *root)
                .expect("每一槽都住着根")
        };
        assert_eq!(
            root_at(region_zero_slot_zero).checkpoint_txg,
            crate::make_filesystem::GENESIS_CHECKPOINT_TXG,
            "前提：区域 0 槽 0 仍是 mkfs 种的第 0 代根"
        );
        let instance_cut_by_the_power = root_at(region_one_slot_zero).instance;
        assert!(
            instance_cut_by_the_power < instance_of_the_mount
                && instance_cut_by_the_power > InstanceGeneration(0),
            "前提：区域 1 槽 0 住着实例 1 写行那条根（{:?}）",
            root_at(region_one_slot_zero)
        );
        let ring_slots_of_the_instance: Vec<RootRingSlot> = roots_by_slot
            .iter()
            .filter(|(_, root)| {
                root.is_some_and(|resident| resident.instance == instance_cut_by_the_power)
            })
            .map(|(ring_slot, _)| *ring_slot)
            .collect();
        assert_eq!(
            ring_slots_of_the_instance,
            vec![region_one_slot_zero],
            "前提：实例 1 在根环里只剩区域 1 槽 0 那一条根"
        );
        let rows = instance_table_chain_of_root(&devices, &current.root)
            .expect("前提：现行那一版的实例表读得出")
            .records
            .rows;
        assert!(
            rows.iter()
                .any(|row| row.instance == instance_cut_by_the_power),
            "前提：现行那一版的实例表里实例 1 那一行还在：{rows:?}"
        );
        PoolWithAnInstanceDownToOneRingRoot {
            devices,
            current_root: current.root,
            instance_rotated_out: instance_cut_by_the_power,
            ring_slot_of_the_only_root_of_the_instance: region_one_slot_zero,
            ring_slot_of_a_root_of_the_rotating_instance: region_zero_slot_zero,
        }
    }

    /// 场景（C973 验收）：上面那张盘面；读实例 1 唯一那条根所在的区域 1 槽 0 时，两次读（第一次与重读）都错位读回区域 0 槽 0 那份第 0 代根。
    /// 落点核认第 0 代根住三个区域的槽 0 任一槽，核不出这一错位；这一遍读出的环里最大的 txg 已越过发布路径写进区域 1 槽 0 的 txg 1。
    /// 预期：区域 1 槽 0 读出的第 0 代根算自证不过，重读仍是第 0 代根，这一轮不删，停在区域 1 槽 0（实例 1 那一行不删）。
    /// 判别力：`crates/mutations.tsv`「C973 清扫槽 0 读到第 0 代根不看环里更大的 txg」「C973 清扫重读第 0 代根那一槽不再核第 0 代根判据」两行，
    /// 这一条红在 `Ok` 那一支（实例 1 那一行判成要删）。
    #[test]
    fn misplaced_read_returning_the_genesis_root_over_the_only_ring_root_of_an_instance_on_both_reads_deletes_no_row(
    ) {
        let pool =
            pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem();
        let reader = PoolReadingOneRingSlotMisplaced::over(&pool, 2);
        match judge_the_instance_table_sweep(&reader, &pool.current_root) {
            Err(NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread {
                ring_slot,
            }) => {
                assert_eq!(
                    ring_slot, pool.ring_slot_of_the_only_root_of_the_instance,
                    "停在错位读回第 0 代根的区域 1 槽 0"
                );
            }
            Ok(judgement) => panic!(
                "错位读回第 0 代根把实例 1 唯一的根换掉，清扫判出要删的行 {:?}",
                judgement.rows_to_delete
            ),
            Err(other) => panic!("这一轮该停在区域 1 槽 0，停在了 {other:?}"),
        }
    }

    /// 场景：同一张盘面，只有第一次读区域 1 槽 0 错位读回第 0 代根，重读读回那一槽真住着的实例 1 的根。
    /// 预期：实例 1 在环里有根，没有候选行（阳性对照：第 0 代根那一条判据只让那一读作废，重读读对了照认那条根）。
    /// 判别力：`crates/mutations.tsv`「C973 清扫第 0 代根那一槽不重读、直接拒」那一行，这一条红（停在区域 1 槽 0）；
    /// 「C973 清扫槽 0 读到第 0 代根不看环里更大的 txg」那一行下这一条也红（第一次那一读当自证过，实例 1 那一行判成要删）。
    #[test]
    fn misplaced_read_returning_the_genesis_root_over_the_only_ring_root_of_an_instance_reread_right_keeps_its_row(
    ) {
        let pool =
            pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem();
        let reader = PoolReadingOneRingSlotMisplaced::over(&pool, 1);
        let judged = judge_the_instance_table_sweep(&reader, &pool.current_root);
        assert!(
            matches!(judged, Err(NoRowDeletedThisRound::NoCandidateRow)),
            "重读读回了实例 1 的根，没有候选行：{judged:?}"
        );
        assert_eq!(reader.misplaced_reads_left.get(), 0, "第一次读错位过了");
    }

    /// 场景：同一张盘面，不错位；区域 0 槽 0 仍住着 mkfs 种的第 0 代根，环里最大的 txg 是 11，发布路径还没写到这一槽（S = 4 时要到 txg 12）。
    /// 预期：区域 0 槽 0 的第 0 代根照自证过算，清扫不因第 0 代根那一条判据拒；每个实例在环里都有根，没有候选行。
    /// 判别力：`crates/mutations.tsv`「C973 清扫槽 0 读到的第 0 代根一律算自证不过」「C973 清扫判第 0 代根时把第 0 代根自己的 txg 也算成发布路径写进这一槽」
    /// 两行，这一条红在停在区域 0 槽 0 那一支。
    #[test]
    fn the_genesis_root_left_in_a_ring_slot_no_publish_has_reached_yet_is_self_verified_and_no_row_is_a_candidate(
    ) {
        let pool =
            pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem();
        let judged = judge_the_instance_table_sweep(&pool.devices, &pool.current_root);
        assert!(
            matches!(judged, Err(NoRowDeletedThisRound::NoCandidateRow)),
            "不错位时实例 1 有根、区域 0 槽 0 的第 0 代根照认：{judged:?}"
        );
    }

    /// 场景：同一张盘面，错位读回的换成区域 0 槽 1 那条实例 2 的根（不是第 0 代根）。
    /// 预期：落点核照拦——这一槽两读都自证不过，这一轮不删，停在区域 1 槽 0（阳性对照：第 0 代根那一条判据之外，落点核照旧起作用）。
    /// 判别力：`crates/mutations.tsv`「C973 清扫读根环落点核判反」那一行，这一条红（停在区域 0 槽 0）。
    #[test]
    fn misplaced_read_returning_another_root_than_the_genesis_root_on_the_same_pool_deletes_no_row()
    {
        let mut pool =
            pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem();
        pool.ring_slot_of_a_root_of_the_rotating_instance = RootRingSlot { region: 0, slot: 1 };
        let reader = PoolReadingOneRingSlotMisplaced::over(&pool, 2);
        let judged = judge_the_instance_table_sweep(&reader, &pool.current_root);
        assert!(
            matches!(
                judged,
                Err(NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot })
                    if ring_slot == RootRingSlot { region: 1, slot: 0 }
            ),
            "错位读回非第 0 代根：这一轮不删、停在区域 1 槽 0：{judged:?}"
        );
    }

    /// 场景：刚 mkfs 的池（S = 4），环里只有三个区域槽 0 各一份第 0 代根，别的槽读回全零；拿读得出的那三槽与这一遍的最大 txg（0）判。
    /// 预期：三槽都不算发布路径已写到（第 0 代根住在 mkfs 种根的槽，这一遍没有比第 0 代根大的 txg），第 0 代根那一条判据一槽都不拒。
    /// 判别力：「C973 清扫槽 0 读到的第 0 代根一律算自证不过」「C973 清扫判第 0 代根时把第 0 代根自己的 txg 也算成发布路径写进这一槽」两行，这一条红。
    #[test]
    fn right_after_make_filesystem_no_ring_slot_holding_the_genesis_root_counts_as_reached_by_the_publishes(
    ) {
        let parameters = parameters();
        let mut devices: DevicesInMemory = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    DeviceOfPagesInMemory {
                        pages: BTreeMap::new(),
                    },
                )
            })
            .collect();
        make_filesystem(&parameters, &mut devices).expect("128 MiB 两盘、S = 4 建得成");
        let roots_read: Vec<(RootRingSlot, RootRecord)> = roots_of_the_ring_by_slot(&devices)
            .into_iter()
            .filter_map(|(ring_slot, root)| root.map(|resident| (ring_slot, resident)))
            .collect();
        assert_eq!(
            roots_read
                .iter()
                .map(|(ring_slot, root)| (*ring_slot, root.checkpoint_txg))
                .collect::<Vec<_>>(),
            crate::make_filesystem::genesis_root_ring_slots()
                .map(|ring_slot| (ring_slot, GENESIS_CHECKPOINT_TXG))
                .collect::<Vec<_>>(),
            "前提：刚 mkfs 的环里读得出的只有三个区域槽 0 的第 0 代根"
        );
        let ring_length_in_slots = every_root_ring_slot(&parameters.geometry).len();
        let ring_slots_reached: Vec<RootRingSlot> = roots_read
            .iter()
            .filter(|(ring_slot, root)| {
                genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached(
                    root.checkpoint_txg,
                    *ring_slot,
                    GENESIS_CHECKPOINT_TXG,
                    parameters.geometry.root_ring_slots_per_region,
                    ring_length_in_slots,
                )
            })
            .map(|(ring_slot, _)| *ring_slot)
            .collect();
        assert_eq!(
            ring_slots_reached,
            Vec::new(),
            "刚 mkfs：一槽都不算发布路径已写到"
        );
    }

    /// 场景：S = 4（环长 12），住着第 0 代根的三个区域槽 0，这一遍的最大 txg 取发布路径第一次写进那一槽的 txg 与它前一个
    /// （区域 1 槽 0 是 txg 1、区域 2 槽 0 是 txg 2、区域 0 槽 0 是 txg 12）；同一槽住的不是第 0 代根时取最大 txg 12。
    /// 预期：最大 txg 到了那一个 txg 才算发布路径已写到，前一个不算；不是第 0 代根的一律不算。
    /// 判别力：`crates/mutations.tsv`「C973 清扫判第 0 代根时往上只走环长少一个 txg」那一行，这一条红在区域 0 槽 0、最大 txg 12 那一格。
    #[test]
    fn genesis_root_counts_as_reached_from_the_first_txg_the_publish_path_writes_into_its_ring_slot(
    ) {
        let parameters = parameters();
        let slots_per_region = parameters.geometry.root_ring_slots_per_region;
        let ring_length_in_slots = every_root_ring_slot(&parameters.geometry).len();
        let reached =
            |checkpoint_txg: u64, ring_slot: RootRingSlot, highest_txg_of_the_pass: u64| {
                genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached(
                    CheckpointTxg(checkpoint_txg),
                    ring_slot,
                    CheckpointTxg(highest_txg_of_the_pass),
                    slots_per_region,
                    ring_length_in_slots,
                )
            };
        let region_zero_slot_zero = RootRingSlot { region: 0, slot: 0 };
        let region_one_slot_zero = RootRingSlot { region: 1, slot: 0 };
        let region_two_slot_zero = RootRingSlot { region: 2, slot: 0 };
        assert_eq!(
            [
                reached(0, region_one_slot_zero, 0),
                reached(0, region_one_slot_zero, 1),
                reached(0, region_two_slot_zero, 1),
                reached(0, region_two_slot_zero, 2),
                reached(0, region_zero_slot_zero, 11),
                reached(0, region_zero_slot_zero, 12),
                reached(12, region_zero_slot_zero, 12),
            ],
            [false, true, false, true, false, true, false],
            "区域 1 槽 0 从 txg 1 起、区域 2 槽 0 从 txg 2 起、区域 0 槽 0 从 txg 12 起算发布路径已写到；txg 12 的根不是第 0 代根"
        );
    }

    /// 读者：读 `misplaced` 那一槽的前几次依次读回 `read_back_instead_in_order` 里那几槽的字节（每次错位读回的不同），之后读原样交给盘；
    /// 别的读原样交给盘。
    struct PoolReadingOneRingSlotMisplacedInOrder<'pool> {
        devices: &'pool DevicesInMemory,
        misplaced: (DeviceIdentity, DeviceOffsetInBytes),
        read_back_instead_in_order:
            std::cell::RefCell<std::collections::VecDeque<(DeviceIdentity, DeviceOffsetInBytes)>>,
    }

    impl PoolReader for PoolReadingOneRingSlotMisplacedInOrder<'_> {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            PoolReader::device_identities(self.devices)
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            PoolReader::device_size_in_bytes(self.devices, device)
        }
        fn read(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Option<Vec<u8>> {
            let read_back_instead = if (device, offset) == self.misplaced {
                self.read_back_instead_in_order.borrow_mut().pop_front()
            } else {
                None
            };
            match read_back_instead {
                Some((device_read_instead, offset_read_instead)) => PoolReader::read(
                    self.devices,
                    device_read_instead,
                    offset_read_instead,
                    length,
                ),
                None => PoolReader::read(self.devices, device, offset, length),
            }
        }
        fn journal_record_offsets_hint(
            &self,
            device: DeviceIdentity,
            ring_start: DeviceOffsetInBytes,
            ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            PoolReader::journal_record_offsets_hint(self.devices, device, ring_start, ring_bytes)
        }
    }

    /// 场景：第 0 代根那张盘面；读区域 1 槽 0 第一次错位读回区域 0 槽 1 那条实例 2 的根（落点核拦下），重读错位读回区域 0 槽 0 的第 0 代根
    /// （落点核照收），再往后的读读回那一槽真住着的实例 1 的根。
    /// 预期：这一槽已重读过一次，第 0 代根那一条判据判它自证不过时不再读第三次，这一轮不删，停在区域 1 槽 0。
    /// 判别力：`crates/mutations.tsv`「C973 清扫第 0 代根那一槽已重读过还再读一次」那一行，这一条红（第三次读回实例 1 的根，没有候选行）。
    #[test]
    fn ring_slot_reread_after_one_misplaced_read_and_read_back_as_the_genesis_root_is_not_read_third_time_and_deletes_no_row(
    ) {
        let pool =
            pool_with_the_only_root_of_an_instance_in_a_ring_slot_seeded_by_make_filesystem();
        let parameters = parameters();
        let device_and_offset_of = |ring_slot: RootRingSlot| {
            (
                parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing),
            )
        };
        let reader = PoolReadingOneRingSlotMisplacedInOrder {
            devices: &pool.devices,
            misplaced: device_and_offset_of(pool.ring_slot_of_the_only_root_of_the_instance),
            read_back_instead_in_order: std::cell::RefCell::new(
                [
                    device_and_offset_of(RootRingSlot { region: 0, slot: 1 }),
                    device_and_offset_of(RootRingSlot { region: 0, slot: 0 }),
                ]
                .into_iter()
                .collect(),
            ),
        };
        let judged = judge_the_instance_table_sweep(&reader, &pool.current_root);
        assert!(
            matches!(
                judged,
                Err(NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot })
                    if ring_slot == pool.ring_slot_of_the_only_root_of_the_instance
            ),
            "已重读过的那一槽读出第 0 代根：这一轮不删、停在区域 1 槽 0：{judged:?}"
        );
        assert!(
            reader.read_back_instead_in_order.borrow().is_empty(),
            "前提：两次错位读都用掉了"
        );
    }
}
