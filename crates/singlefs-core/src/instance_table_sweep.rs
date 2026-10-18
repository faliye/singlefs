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

use singlefs_format::{
    DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, INSTANCE_TABLE_PAGE_RECORDS, PACKED_UNIT_HEADER_BYTES,
};

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use crate::allocator::{absolute_slot_count_of_device, PoolAllocator};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::checksum::wide_checksum_field_holds;
use crate::instance_table::InstanceRow;
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mount::RollbackTarget;
use crate::recovery::{
    allocation_records_under_root_in_the_unit_area_starting_at, choose_system_configuration,
    every_root_ring_slot, instance_table_chain_of_root,
    unit_area_start_of_the_chosen_system_configuration, PoolReader, RecoveryFailure,
};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{slot_offset, RootRingSlot};
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
    /// 默认值没定（C690（清扫触发阈值的默认值没定））：这里的占位取一片实例表装得下的数据行数 369（370 条记录减去链指针那一条），
    /// 行数越过它链就长出第二片。不是定案，定了照定案换。
    pub const PLACEHOLDER_UNTIL_C690: Self = Self(INSTANCE_TABLE_PAGE_RECORDS - 1);
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
    /// 产品路径（`mount::mount_writable`）：开关关着、阈值取占位（[`InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690`]）。
    pub const PRODUCT: Self = Self {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,
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
    /// 根环一个槽读得出而自证不过，重读一次仍不过。
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

/// 读根环每个槽：读不出、读成功而全零 ⇒ 这一次不删；读得出而自证不过 ⇒ 重读一次，仍不过 ⇒ 这一次不删；都自证过就交回每一条根。
fn every_root_of_the_ring_self_verified(
    reader: &dyn PoolReader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &crate::system_configuration::SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Result<Vec<RootRecord>, NoRowDeletedThisRound> {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    let mut roots = Vec::new();
    // 迭代上界是根环的槽数（3 × S），每槽至多读两次；提前出口是这一次不删的那几种。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
        let mut root_read = None;
        for _reading in 0..2 {
            let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
                return Err(NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero {
                    ring_slot,
                });
            };
            if bytes.iter().all(|byte| *byte == 0) {
                return Err(NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero {
                    ring_slot,
                });
            }
            root_read = RootRecord::parse_slot(&bytes, filesystem_identifier);
            if root_read.is_some() {
                break;
            }
        }
        let Some(root) = root_read else {
            return Err(
                NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot },
            );
        };
        roots.push(root);
    }
    Ok(roots)
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
            Some(&current_file_version),
        ),
        SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch => {
            let mut allocator_without_a_free_slot = allocator.clone();
            take_every_free_slot_away(&mut allocator_without_a_free_slot);
            match prepare_the_version_publish(
                &pool,
                &mut allocator_without_a_free_slot,
                &plan.as_publish_plan(),
                Some(&current_file_version),
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
