//! 实例表单元里的记录（D18（块里携带什么信息） 已定项 11）：`kind` 0 行与每一片末尾的链指针记录。
//! 恢复（回退行的 W）、可写挂载与回退（写行）都从这里解；checker 是独立解析器，不共用这份。
//!
//! 一张实例表是一条链：根记录指着第 0 片，第 k 片的链指针记录指着第 k + 1 片，最后一片写「无下一片」
//! （沿链读盘在 `recovery::instance_table_chain_of_root`）。**写者今天只写一片**：多于一片时各片在 bump 次序里怎么排
//! （D3（空间分配） 已定项 10 ⑤ 只写了「实例表单元最前」，排序规则管的是「其余」）、行怎么分到各片，都没有条款，
//! 可写挂载与回退在取号之前就拒（`mount::MountError::InstanceTableChainLongerThanOnePageUndecided`）。

use crate::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use crate::bytes::{ByteReader, ByteWriter};
use crate::make_filesystem::instance_table_chain_record;
use crate::pointer::NodePointer;
use crate::transaction::TREE_IDENTIFIER_NONE;
use crate::unit::{parse_packed_unit, PackedIdentity, PACKED_TYPE_INSTANCE_TABLE};
use singlefs_format::{INSTANCE_ROW_BYTES, INSTANCE_TABLE_PAGE_RECORDS, NODE_POINTER_BYTES};

/// 实例表 `kind` 0 行记录（D18（块里携带什么信息） 已定项 11）：
/// `kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（bit0 = 回退行）| 预留 66`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceRow {
    pub instance: InstanceGeneration,
    pub selected_root_txg: CheckpointTxg,
    pub applied_transaction_high_water: u64,
    pub is_rollback: bool,
}

const INSTANCE_ROW_KIND_ROW: u8 = 0;
const INSTANCE_ROW_KIND_CHAIN: u8 = 1;
const INSTANCE_ROW_FLAG_ROLLBACK: u8 = 0b0000_0001;

impl InstanceRow {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
        writer.put_u8(INSTANCE_ROW_KIND_ROW);
        writer.put_u32(self.instance.0);
        writer.put_u64(self.selected_root_txg.0);
        writer.put_u64(self.applied_transaction_high_water);
        writer.put_u8(if self.is_rollback {
            INSTANCE_ROW_FLAG_ROLLBACK
        } else {
            0
        });
        writer.skip(66);
        writer.assert_position(INSTANCE_ROW_BYTES, "实例表行记录");
        writer.into_bytes()
    }

    /// `kind` 0 才是行；链指针记录（`kind` 1）与别的 `kind` 返回 `None`。flags 只认 bit0，别的位非 0 拒收。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(INSTANCE_ROW_BYTES).expect("88")
            || bytes[0] != INSTANCE_ROW_KIND_ROW
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 1);
        let instance = InstanceGeneration(reader.get_u32());
        let selected_root_txg = CheckpointTxg(reader.get_u64());
        let applied_transaction_high_water = reader.get_u64();
        let flags = reader.get_u8();
        if flags & !INSTANCE_ROW_FLAG_ROLLBACK != 0 {
            return None;
        }
        Some(Self {
            instance,
            selected_root_txg,
            applied_transaction_high_water,
            is_rollback: flags & INSTANCE_ROW_FLAG_ROLLBACK != 0,
        })
    }
}

/// 链指针记录第 2 个字节「有无下一片」（D18（块里携带什么信息） 已定项 11）的「无」：mkfs 写出的那一片写的就是它
/// （字节表七 m1「无下一片」的链指针记录）。
const CHAIN_RECORD_NO_NEXT_PAGE: u8 = 0;
/// 「有无下一片」的「有」。⚠️ 条款只写了字段名与宽度（`有无下一片 1`），「有」取 1、0 与 1 之外的值拒收是按字段名、
/// 按同一张表里 kind 0 那个 flags 字节「其余位恒 0、非 0 拒收」同一个口径读出来的，没有逐字条款（交主 agent）。
const CHAIN_RECORD_HAS_NEXT_PAGE: u8 = 1;
/// 链指针记录里位置指针的偏移：`kind` 1 字节 + 有无下一片 1 字节。
const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
/// 链指针记录没有预留（D18（块里携带什么信息） 已定项 11「预留 0」）：kind 1 + 有无下一片 1 + 指针 86 正好是记录宽 88。
const _CHAIN_RECORD_IS_KIND_FLAG_AND_POINTER_WITHOUT_RESERVE: () = assert!(
    INSTANCE_ROW_BYTES == 2 + NODE_POINTER_BYTES,
    "链指针记录 = kind 1 + 有无下一片 1 + 位置指针 86，预留 0"
);

/// 实例表链上的第几片：身份四元组里的容器号就是它（D18（块里携带什么信息） 已定项 11「容器号 = 片序号从 0 起」）。
#[derive(Clone, Copy)]
pub struct InstanceTablePageIndex(pub u64);

impl InstanceTablePageIndex {
    /// 根记录直接指着的那一片。
    pub const FIRST: Self = Self(0);

    /// 这一片的身份四元组 (出生树 0, 类型 4, 容器号 = 片序号, 出生代 0)（D18（块里携带什么信息） 已定项 11）。
    #[must_use]
    pub const fn packed_identity(self) -> PackedIdentity {
        PackedIdentity {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            record_type: PACKED_TYPE_INSTANCE_TABLE,
            container: self.0,
            container_birth: CheckpointTxg(0),
        }
    }

    /// 链上的下一片。
    ///
    /// # Panics
    /// 片序号已经是 `u64::MAX`。走不到：沿链读的第 k 轮要求读到的单元容器号是 k，每一轮读的是盘上不同的单元
    /// （`recovery::instance_table_chain_of_root` 的迭代上界），一块盘装不下 2^64 个 32 KiB 单元。
    #[must_use]
    pub fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("片序号每一轮对应盘上一个不同的 32 KiB 单元，一块盘装不下 2^64 个"),
        )
    }
}

/// 装得下 `rows` 行要几片：每片 370 条记录，链指针记录恒为一片的最后一条、数据行 369（D18（块里携带什么信息） 已定项 11）；
/// 0 行也是一片（mkfs 写出的那一片只有链指针记录）。与 D28（挂载期承诺量） 已定项 3 的 max(1, ⌈行数 / 每片行数⌉) 同一个数。
#[must_use]
pub fn instance_table_pages_for_rows(rows: usize) -> usize {
    let rows_per_page = usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370") - 1;
    rows.div_ceil(rows_per_page).max(1)
}

/// 链指针记录（`kind` 1，恒为一片的最后一条）：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位）| 预留 0`
/// （D18（块里携带什么信息） 已定项 11；位置指针与根记录里的实例表单元指针同型，D19（块指针的结构与宽度预算） 已定项 7 / 8）。
#[derive(Debug, PartialEq)]
pub enum InstanceTableChainRecord {
    /// 这一片是链上的最后一片。
    LastPage,
    /// 下一片的指针。
    NextPage(NodePointer),
}

impl InstanceTableChainRecord {
    /// 解一条链指针记录。宽不是 88、`kind` 不是 1、「有无下一片」不是 0 或 1、无下一片而位置指针不全零（条款：清零占位）、
    /// 有下一片而位置指针全零（指不到任何地方），都是 `None`。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(INSTANCE_ROW_BYTES).expect("88")
            || bytes[0] != INSTANCE_ROW_KIND_CHAIN
        {
            return None;
        }
        let pointer_is_zero = bytes[CHAIN_RECORD_POINTER_OFFSET..]
            .iter()
            .all(|byte| *byte == 0);
        match bytes[1] {
            CHAIN_RECORD_NO_NEXT_PAGE => pointer_is_zero.then_some(Self::LastPage),
            CHAIN_RECORD_HAS_NEXT_PAGE => (!pointer_is_zero).then(|| {
                Self::NextPage(NodePointer::read_from(&mut ByteReader::at(
                    bytes,
                    CHAIN_RECORD_POINTER_OFFSET,
                )))
            }),
            _value_the_field_table_does_not_register => None,
        }
    }
}

/// 实例表链上的一片：这一片的行，与它末尾那条链指针记录（D18（块里携带什么信息） 已定项 11）。
#[derive(Debug, PartialEq)]
pub struct InstanceTablePage {
    pub rows: Vec<InstanceRow>,
    pub chain: InstanceTableChainRecord,
}

impl InstanceTablePage {
    /// 把一个单元当链上第 `page_index` 片解出来。单元解不开、身份四元组不是 (0, 4, 片序号, 0)（指针指到了别的单元，
    /// 或链的次序乱了、成了环）、最后一条不是合法的链指针记录、它前面有行之外的记录，都是 `None`。
    #[must_use]
    pub fn parse(unit_bytes: &[u8], page_index: InstanceTablePageIndex) -> Option<Self> {
        let unit = parse_packed_unit(unit_bytes).ok()?;
        if unit.identity != page_index.packed_identity() {
            return None;
        }
        let (last, rows_bytes) = unit.records.split_last()?;
        let chain = InstanceTableChainRecord::parse(last)?;
        let mut rows = Vec::with_capacity(rows_bytes.len());
        for bytes in rows_bytes {
            rows.push(InstanceRow::parse(bytes)?);
        }
        Some(Self { rows, chain })
    }
}

/// 一张实例表的全部行，按链上的次序接起来（第 0 片的行在前）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableRecords {
    pub rows: Vec<InstanceRow>,
}

impl InstanceTableRecords {
    /// 一片就是整张表（第 0 片、链指针记录是「无下一片」）时，从那一片单元解出整张表。
    /// 链指针说还有下一片 ⇒ `None`：下一片要按指针读盘，只拿一片的字节解不出整张表，交回前半张会让调用方按半张表判；
    /// 沿链读盘在 `recovery::instance_table_chain_of_root`。别的解不开同 [`InstanceTablePage::parse`]。
    #[must_use]
    pub fn parse(instance_table_unit: &[u8]) -> Option<Self> {
        let first_page =
            InstanceTablePage::parse(instance_table_unit, InstanceTablePageIndex::FIRST)?;
        match first_page.chain {
            InstanceTableChainRecord::LastPage => Some(Self {
                rows: first_page.rows,
            }),
            InstanceTableChainRecord::NextPage(_) => None,
        }
    }

    /// 写行那次发布重写出去的那一片的记录：行在前、「无下一片」的链指针记录最末（D18（块里携带什么信息） 已定项 11）。
    ///
    /// # Panics
    /// 行数 + 1（链指针记录）多于一片的 370 条。产品路径走不到：写行只有 `mount` 的可写挂载与回退两条路，
    /// 都经 `establish_instance` 在取号之前算这一版的行数加这次要写的行数，多于一片就拒
    /// （`MountError::InstanceTableChainLongerThanOnePageUndecided`：第二片在 bump 次序里排第几、行怎么分片没有条款）。
    #[must_use]
    pub fn to_records(&self) -> Vec<Vec<u8>> {
        let records_per_page = usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370");
        assert!(
            self.rows.len() < records_per_page,
            "{} 行加链指针记录装不进一片 {records_per_page} 条：可写挂载与回退在取号之前就该按片数拒掉",
            self.rows.len()
        );
        let mut records: Vec<Vec<u8>> = self.rows.iter().map(InstanceRow::to_bytes).collect();
        records.push(instance_table_chain_record());
        records
    }
}

#[cfg(test)]
mod chain_record_tests {
    use super::{
        instance_table_pages_for_rows, InstanceTableChainRecord, InstanceTablePage,
        InstanceTablePageIndex, InstanceTableRecords,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
    use crate::bytes::ByteWriter;
    use crate::make_filesystem::instance_table_chain_record;
    use crate::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
    use crate::transaction::TREE_IDENTIFIER_NONE;
    use crate::unit::{build_packed_unit, WriteOrder};
    use singlefs_format::INSTANCE_ROW_BYTES;

    fn next_page_pointer() -> NodePointer {
        NodePointer {
            head: PointerHead {
                birth_tree: crate::address::TreeIdentifier(TREE_IDENTIFIER_NONE),
                birth_txg: CheckpointTxg(9),
            },
            locations: [
                LocationEntry {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(60_000),
                    unit_checksum: 0x1234_5678,
                },
                LocationEntry {
                    device: DeviceIdentity(1),
                    slot: SlotNumber(60_000),
                    unit_checksum: 0x1234_5678,
                },
            ],
            instance: InstanceGeneration(4),
            birth_sequence: BirthSequence(1),
        }
    }

    fn chain_record(has_next_page: u8, pointer: &NodePointer) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
        writer.put_u8(1);
        writer.put_u8(has_next_page);
        pointer.write_to(&mut writer);
        writer.into_bytes()
    }

    /// 「有无下一片」只认 0 与 1；0 时位置指针要清零占位、1 时不许全零（D18（块里携带什么信息） 已定项 11）。
    #[test]
    fn chain_record_parse_accepts_only_a_zeroed_last_page_or_a_real_next_page_pointer() {
        let pointer = next_page_pointer();
        assert_eq!(
            InstanceTableChainRecord::parse(&instance_table_chain_record()),
            Some(InstanceTableChainRecord::LastPage),
            "mkfs 写的那一条：无下一片、指针清零"
        );
        assert_eq!(
            InstanceTableChainRecord::parse(&chain_record(1, &pointer)),
            Some(InstanceTableChainRecord::NextPage(pointer))
        );
        assert_eq!(
            InstanceTableChainRecord::parse(&chain_record(0, &pointer)),
            None,
            "无下一片而指针没清零"
        );
        assert_eq!(
            InstanceTableChainRecord::parse(&chain_record(1, &NodePointer::empty_root())),
            None,
            "有下一片而指针全零"
        );
        assert_eq!(
            InstanceTableChainRecord::parse(&chain_record(2, &pointer)),
            None,
            "有无下一片写 2"
        );
        let mut row_kind = chain_record(1, &pointer);
        row_kind[0] = 0;
        assert_eq!(
            InstanceTableChainRecord::parse(&row_kind),
            None,
            "kind 0 是行，不是链指针记录"
        );
    }

    fn page_unit(page_index: InstanceTablePageIndex, chain: Vec<u8>) -> Vec<u8> {
        build_packed_unit(
            page_index.packed_identity(),
            u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
            &[chain],
            CheckpointTxg(9),
            &[7u8; 16],
            WriteOrder {
                instance: InstanceGeneration(4),
                transaction: 0,
            },
            BirthSequence(0),
        )
    }

    /// 一片按链上第几片解：身份四元组里的容器号要等于片序号（链指到别的片、成环都在这一判上断）；
    /// 第 0 片的链指针说还有下一片时，只拿这一片解不出整张表。
    #[test]
    fn each_page_parses_only_as_the_page_its_container_number_names_and_a_chained_first_page_is_not_a_whole_table(
    ) {
        let second_page = page_unit(InstanceTablePageIndex(1), instance_table_chain_record());
        assert!(InstanceTablePage::parse(&second_page, InstanceTablePageIndex(1)).is_some());
        assert_eq!(
            InstanceTablePage::parse(&second_page, InstanceTablePageIndex::FIRST),
            None,
            "容器号 1 的单元不是第 0 片"
        );
        let chained_first_page = page_unit(
            InstanceTablePageIndex::FIRST,
            chain_record(1, &next_page_pointer()),
        );
        assert!(
            InstanceTablePage::parse(&chained_first_page, InstanceTablePageIndex::FIRST).is_some()
        );
        assert_eq!(InstanceTableRecords::parse(&chained_first_page), None);
    }

    /// 片数 = max(1, ⌈行数 / 369⌉)（每片 370 条，链指针记录占最后一条；D28（挂载期承诺量） 已定项 3 同一个数）。
    #[test]
    fn pages_for_rows_is_at_least_one_and_opens_a_page_every_three_hundred_sixty_nine_rows() {
        assert_eq!(
            instance_table_pages_for_rows(0),
            1,
            "mkfs 那一片只有链指针记录"
        );
        assert_eq!(instance_table_pages_for_rows(369), 1);
        assert_eq!(instance_table_pages_for_rows(370), 2);
        assert_eq!(instance_table_pages_for_rows(738), 2);
        assert_eq!(instance_table_pages_for_rows(739), 3);
    }
}
