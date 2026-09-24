//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头与 journal 环（扫描方向），判第一版那几条不变量
//! （清单与条数在 `image::IMPLEMENTED_INVARIANTS`，别在这里另抄一个数）。
//! 最新的那条根另判 I-7.2（能不能完整走完）；引用集合按全部有效根取并集——根环里更早的根仍是可回退的状态，
//! 它们引用的单元仍占着空间（I-3.1 与 I-5.1 的这一读法 2026-09-14 用户收尾弹窗定甲，见 records 2026-09-13-总审核 十一·七）。

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
    INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES, SLOT_BYTES,
    TREE_TABLE_ENTRY_BYTES,
};

use crate::image::{
    chosen_system_configurations, judge_location_order, parse_data_pointer, parse_node_pointer,
    read_referenced_unit, root_slot_positions, valid_roots, verified_system_configuration_slots,
    ImageReader, InvariantVerdict, Judgements, PointerView, PoolGeometry,
};
use crate::{
    back_chain_of_record_header, check_index_node_keys, check_journal_record, checksum_field_holds,
    crc32_castagnoli_table, index_node_view, key_schema_for_tree_kind, read_six_byte_unsigned,
    read_u16, read_u32, read_u64, KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
};

const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;
const PACKED_TYPE_INODE: u16 = 2;
const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
/// 打包记录类型登记的记录宽：类型 2 是 140（D8（核心索引结构） 已定项 6），类型 4 是 88（实例表行）。
const fn registered_record_width(record_type: u16) -> Option<usize> {
    match record_type {
        PACKED_TYPE_INODE => Some(140),
        PACKED_TYPE_INSTANCE_TABLE => Some(88),
        _ => None,
    }
}
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
/// defer 队列待释放（D5（快照 / 空间记账机制） 已定项 4 第 5 项的行号，带设备维，单位字节）：I-3.11（已分配减 defer 等于最新根走读） 读它。
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
/// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项的行号）。
const STATISTIC_INODE_WATERMARK: u16 = 12;
/// 不带设备维的统计量把设备段写成这个保留值（D5（快照 / 空间记账机制） 已定项 10）。
const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}
fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}
fn tree_table_entry_bytes() -> usize {
    usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
}
fn allocation_record_bytes() -> usize {
    usize::try_from(ALLOCATION_RECORD_BYTES).expect("20")
}
fn extent_leaf_record_bytes() -> usize {
    usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112")
}
fn inode_internal_entry_bytes() -> usize {
    usize::try_from(INODE_INTERNAL_ENTRY).expect("120")
}
fn accounting_entry_bytes() -> usize {
    usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34")
}
/// 中央映射条目今天走读要几个字节：key 27 + 位置条目 14 × 2。**不是「该有多宽」**（那归 C307），
/// 只是「切到偏移 55 之前得有 55 个字节」。
fn mapping_entry_bytes() -> usize {
    usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
}
/// 一棵树的条目该有多宽（各棵树的条目字段表）：走读按固定偏移切条目之前先比它（I-1.10（码 2 条目宽等于字段表宽））。
/// 条目宽是索引节点头里的一个**盘上字段**，`index_node_view` 只判了它 ≥ key 宽
/// （extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」。
///
/// ⚠️ **中央映射树（种类 5）不在这张表里**：它的条目宽归 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树），
/// 那条还开着，所以 I-1.10（码 2 条目宽等于字段表宽） 明写不判它。`key_schema_for_tree_kind` 罩 5 个种类、这里只罩 4 个，
/// 两张表的差就是那一棵。
fn field_table_width_of_tree_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(extent_leaf_record_bytes()),
        TREE_KIND_INODE => Some(inode_internal_entry_bytes()),
        TREE_KIND_ALLOCATION => Some(allocation_record_bytes()),
        TREE_KIND_ACCOUNTING => Some(accounting_entry_bytes()),
        _ => None,
    }
}

/// 一次走读的状态：判定累加器、被引用单元的物理范围、走读有没有断。
struct Walk<'reader> {
    reader: &'reader dyn ImageReader,
    judgements: Judgements,
    filesystem_identifier_low: u64,
    /// (设备, 起点槽, 跨度槽) → 第一次引用它的是谁。
    references: BTreeMap<(u32, u64, u64), String>,
    visited_units: BTreeSet<(u32, u64)>,
    walk_failures: Vec<String>,
    /// 走过的全部树表条目里的树 ID（I-7.8 的一半）。
    tree_identifiers_in_tables: BTreeSet<u64>,
    /// inode 号 → 对象出生代；数据单元 → (对象, 对象出生代)。I-9.10 在最后对。
    inode_object_birth: BTreeMap<u64, u64>,
    /// 走到过 inode 树的根、并且它是层级大于 0 的内部节点（I-9.6（水位大于两处最大号） 与 I-9.12（分隔 key 落在孩子区间之外） 都要它）。
    inode_tree_walked: bool,
    /// 遍历侧算出来的「该树 inode 树内最大 key」：逐片叶容器逐条记录取最大，与记账里的水位是两条独立路径（I-9.6）。
    largest_inode_number_in_the_inode_tree: Option<u64>,
    data_unit_objects: Vec<(u64, u64, String)>,
    /// 最新根下面记账树里的行：(统计量, 设备) → 值。
    accounting: BTreeMap<(u16, u32), u64>,
    accounting_seen: bool,
    /// 最新根指着的实例表里的行：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集），
    /// I-1.8（归并后版本全序） 的已发布谓词也读它。
    instance_table_rows: Vec<InstanceTableRow>,
    /// 挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg：已发布谓词的另外两个输入
    /// （I-1.2（块头写序已发布） / I-4.2（无被引用未提交块） 在遍历方向上各判一格要它）。
    mount_root_instance: u32,
    mount_root_checkpoint_txg: u64,
    /// 遍历方向上判过出生身份的被引用单元数：一个都没有时 I-1.2 与 I-4.2 报「不适用」，不报成立。
    referenced_units_judged_against_their_pointer: u64,
}

/// 实例表里的一行（D18（块里携带什么信息） 已定项 11 的字段表：`kind` 1 + 实例代号 4 + 所选根的 `checkpoint_txg` 8 +
/// 属于该实例的最大已施加事务号 W 8 + flags 1 + 预留 66）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InstanceTableRow {
    instance: u32,
    /// 这一行记的 T：该实例被施加到哪一代为止（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的 `T_pub`）。
    published_checkpoint_txg: u64,
    /// 这一行记的 W：属于该实例的、被这次重放施加的最大事务号。
    applied_transaction_high_water_mark: u64,
    /// flags 字节的 bit0：这一行是回退写的回退行（D18（块里携带什么信息） 已定项 11「回退行在 kind 0 记录的 flags 字节里置 bit0」）。
    /// I-7.8 扫描方向按它分两种行：回退行的 T 之后那段时间线发布过根（被抛弃了），非回退行的 T 之后那一实例没有发布过根。
    is_rollback: bool,
}

/// 实例表行记录里 flags 那 1 字节的偏移：`kind` 1 + 实例代号 4 + T 8 + W 8（D18（块里携带什么信息） 已定项 11 的字段表）。
const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
/// flags 字节里标「回退行」的那一位（D18（块里携带什么信息） 已定项 11：bit0）。
const INSTANCE_TABLE_ROW_FLAG_ROLLBACK: u8 = 0b0000_0001;

/// 实例表一片末尾那条链指针记录（D18（块里携带什么信息） 已定项 11：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位）| 预留 0`）
/// 在 checker 这边的读法，按字段表另写一份，不用实现的解析。「有无下一片」的「有」按 1 读：条款只写了字段名，
/// 0 是 mkfs 那一片写的「无」（字节表七 m1），1 与拒收别的值是按字段名与同一张表里 flags 字节的口径读的。
#[derive(PartialEq)]
enum ChainRecordView {
    LastPage,
    NextPage(PointerView),
    /// 记录宽不是 88、kind 不是 1、有无下一片不是 0 或 1、无下一片而位置指针不全零、有下一片而位置指针全零。
    Malformed,
}

/// 链指针记录里位置指针的偏移：kind 1 字节 + 有无下一片 1 字节。
const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
/// 实例表记录宽（D18（块里携带什么信息） 已定项 11）：链指针记录与行记录同宽。
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;

fn chain_record_view(record: &[u8]) -> ChainRecordView {
    if record.len() != INSTANCE_TABLE_RECORD_BYTES || record[0] != 1 {
        return ChainRecordView::Malformed;
    }
    let pointer = parse_node_pointer(&record[CHAIN_RECORD_POINTER_OFFSET..]);
    match record[1] {
        0 if pointer.all_zero => ChainRecordView::LastPage,
        1 if !pointer.all_zero => ChainRecordView::NextPage(pointer),
        _flag_or_pointer_that_does_not_match => ChainRecordView::Malformed,
    }
}

/// 指针尾段装的那个量：码 1 的指针 88 尾段是写序的事务号（6 字节），码 2 / 码 3 的指针 86 尾段是出生序号（4 字节）
/// （`image::parse_data_pointer` / `image::parse_node_pointer` 各解一份；`KEY_SCHEMA_MAPPING` 那一行的注同一个口径）。
/// 码 3 的写序里还有一个事务号，指针不带它 ⇒ 不比它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TailOfTheBirthIdentity {
    TransactionNumber(u64),
    BirthSequence(u32),
}

impl TailOfTheBirthIdentity {
    fn value(self) -> u64 {
        match self {
            TailOfTheBirthIdentity::TransactionNumber(transaction) => transaction,
            TailOfTheBirthIdentity::BirthSequence(birth_sequence) => u64::from(birth_sequence),
        }
    }

    fn name(self) -> &'static str {
        match self {
            TailOfTheBirthIdentity::TransactionNumber(_) => "事务号",
            TailOfTheBirthIdentity::BirthSequence(_) => "出生序号",
        }
    }
}

/// 一个被引用单元的头里，I-1.2（块头写序已发布） 的判定输入：诞生代号 b、写序 (i, n) 与出生序号。
/// 落点按 D18（块里携带什么信息） 已定项 7 的字段表（码 1：诞生代号 75、写序 91；码 2：诞生代号 52 + 2k、
/// 写序 68 + 2k、出生序号 72 + 2k）与已定项 11 的类身份段表（码 3：诞生代号 73、写序 93、出生序号 103）各解一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BirthIdentityInTheHeader {
    write_order: WriteOrderUnderThePublishedPredicate,
    tail: TailOfTheBirthIdentity,
}

/// 从一个被引用单元的头里取出 I-1.2 的判定输入。类标签不在 1 / 2 / 3 里的没有这一格（那一格归 I-1.6（类标签登记且三处相等））。
fn birth_identity_in_the_header(unit: &[u8]) -> Option<BirthIdentityInTheHeader> {
    match unit[6] {
        1 => Some(BirthIdentityInTheHeader {
            write_order: WriteOrderUnderThePublishedPredicate::DataUnit {
                birth_txg: read_u64(unit, 75),
                instance: read_u32(unit, 91),
                transaction: read_six_byte_unsigned(unit, 95),
            },
            tail: TailOfTheBirthIdentity::TransactionNumber(read_six_byte_unsigned(unit, 95)),
        }),
        2 => {
            let key_width = usize::from(unit[51]);
            Some(BirthIdentityInTheHeader {
                write_order: WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                    birth_txg: read_u64(unit, 52 + 2 * key_width),
                    instance: read_u32(unit, 68 + 2 * key_width),
                },
                tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 72 + 2 * key_width)),
            })
        }
        3 => Some(BirthIdentityInTheHeader {
            write_order: WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                birth_txg: read_u64(unit, 73),
                instance: read_u32(unit, 93),
            },
            tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 103)),
        }),
        _outside_the_three_unit_classes => None,
    }
}

impl Walk<'_> {
    /// 遍历方向上的两格，判的是同一个被引用单元，输入也是同一份（头里的诞生代号 b、写序 (i, n)、出生序号）：
    ///
    /// ① **I-1.2（块头写序已发布）**：头里的这几段与**引用它的那条指针**记的相同。两处不等时「头记录的写序」
    ///    就不是这个单元的真实身份，②那个谓词判的东西是假的。恢复路径今天在遍历方向判的正是这一格
    ///    （`crates/singlefs-core/src/recovery.rs` 的 `RecoveryFailure::InvariantViolated { invariant: "I-1.2" }`
    ///    四处：树根出生序号与指针不符、inode 叶的出生身份与子指针不符、inode 叶写序实例与子指针不符、
    ///    数据单元头与指针的出生身份不符），池级 checker 自己按字段表解一份，不从实现拿偏移。
    ///    **扫描方向那一半（「任一单元」都按谓词判，i > 挂载根实例 ⇒ 判损坏）池级 checker 判不了**，
    ///    理由写在 [`PublishedPredicate::holds_for`] 与 `.claude/kb/invariants.md` I-1.2 的状态列里。
    ///
    /// ② **I-4.2（无被引用未提交块）**：这几段按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的
    ///    已发布谓词判成已发布——「未提交」在本工程里就是「按那个谓词判不成已发布」，被引用的块必须已提交。
    ///    谓词与 I-1.8（归并后版本全序） 扫描方向用的是同一份 [`PublishedPredicate`]。
    ///
    /// **打包记录类型 4 的实例表单元不进这两格**：I-1.2 那一行的「**例外**」逐字写着它不走这条谓词，
    /// 它的现行版本由根记录持有的物理指针唯一确定。调用点因此不在 `walk_root` 读实例表那一段上。
    fn judge_birth_identity_of_a_referenced_unit(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        what: &str,
    ) {
        let Some(identity) = birth_identity_in_the_header(unit) else {
            return;
        };
        self.referenced_units_judged_against_their_pointer += 1;
        let matches_the_pointer = identity.write_order.birth_txg() == pointer.birth_txg
            && identity.write_order.instance() == pointer.instance
            && identity.tail.value() == pointer.tail;
        self.judgements.judge("I-1.2", matches_the_pointer, || {
            format!(
                "{what}：头里的出生身份（诞生代号 {}、写序实例 {}、{} {}）与引用它的指针记的（诞生代号 {}、实例 {}、尾段 {}）不符",
                identity.write_order.birth_txg(),
                identity.write_order.instance(),
                identity.tail.name(),
                identity.tail.value(),
                pointer.birth_txg,
                pointer.instance,
                pointer.tail
            )
        });
        let mount_root_instance = self.mount_root_instance;
        let mount_root_checkpoint_txg = self.mount_root_checkpoint_txg;
        let published = PublishedPredicate {
            mount_root_instance,
            mount_root_checkpoint_txg,
            instance_table_rows: &self.instance_table_rows,
        };
        let is_published = published.holds_for(identity.write_order);
        // 借的是 `instance_table_rows` 这一个字段，`judge` 要的是 `judgements` 那一个：两处不重叠。
        // 不 clone：层 0 每个崩溃状态都要在每个被引用单元上走一遍这里。
        let rows = &self.instance_table_rows;
        self.judgements.judge("I-4.2", is_published, || {
            format!(
                "{what}：头里的诞生代号 {}、写序实例 {} 按已发布谓词判不成已发布（挂载根实例 {mount_root_instance}、txg {mount_root_checkpoint_txg}，实例表行 {rows:?}）——被引用的块必须是已提交的块",
                identity.write_order.birth_txg(),
                identity.write_order.instance()
            )
        });
    }

    fn note_reference(&mut self, device: u32, slot: u64, span: u64, what: &str) {
        self.references
            .entry((device, slot, span))
            .or_insert_with(|| what.to_string());
    }

    /// 共同前缀与类身份段：I-1.6（类标签登记且三处相等）、I-2.4（头校验和覆盖范围）、I-2.3（补齐为零且在载荷 CRC 内）、I-1.4（fsid）。
    /// 返回头能不能用（头校验和过、类标签对）。
    fn judge_unit_header(&mut self, unit: &[u8], expected_class: u8, what: &str) -> bool {
        let class = unit[6];
        let copy_matches = match class {
            1 | 3 => unit[42] == class,
            _ => true,
        };
        self.judgements.judge(
            "I-1.6",
            matches!(class, 1..=3) && class == expected_class && unit[7] == 0 && copy_matches,
            || {
                format!(
                    "{what}：类标签 {class}（要 {expected_class}）、flags {}、类身份段首字节 {}",
                    unit[7], unit[42]
                )
            },
        );
        let (header_end, payload_start, declared, payload_crc_offset, fsid_offset) =
            match expected_class {
                1 => (
                    105usize,
                    134usize,
                    usize::from(read_u16(unit, 8)),
                    101usize,
                    83usize,
                ),
                3 => (107, 136, usize::from(read_u16(unit, 8)), 89, 81),
                _ => {
                    let key_width = usize::from(unit[51]);
                    (
                        86 + 2 * key_width,
                        115 + 2 * key_width,
                        usize::from(read_u16(unit, 8)),
                        76 + 2 * key_width,
                        60 + 2 * key_width,
                    )
                }
            };
        let header_holds = unit.len() > header_end && checksum_field_holds(unit, header_end, 10);
        self.judgements.judge("I-2.4", header_holds, || {
            format!("{what}：头校验和不罩 [0, {header_end}) 或对不上")
        });
        if !header_holds || class != expected_class {
            self.walk_failures.push(format!("{what} 的头用不了"));
            return false;
        }
        let padding_start = payload_start + declared;
        let padding_zero =
            padding_start <= unit.len() && unit[padding_start..].iter().all(|byte| *byte == 0);
        let payload_crc_holds =
            read_u32(unit, payload_crc_offset) == crc32_castagnoli_table(&unit[header_end..]);
        self.judgements
            .judge("I-2.3", padding_zero && payload_crc_holds, || {
                format!(
                    "{what}：声明长度 {declared} 之后的补齐{}、载荷 CRC{}",
                    if padding_zero { "为 0" } else { "非 0" },
                    if payload_crc_holds { "对" } else { "不对" }
                )
            });
        let fsid = read_u64(unit, fsid_offset);
        self.judgements
            .judge("I-1.4", fsid == self.filesystem_identifier_low, || {
                format!("{what}：头里的 fsid {fsid:#x} 与系统配置的低 8 字节不符")
            });
        true
    }

    /// 读一个码 2 节点并判头、树 ID（I-1.3）与 key 区间（I-1.1）。
    fn read_index_node(
        &mut self,
        pointer_bytes: &[u8],
        expected_tree: u64,
        schema: crate::KeySchema,
        what: &str,
    ) -> Option<crate::IndexNodeView> {
        let pointer = parse_node_pointer(pointer_bytes);
        if pointer.all_zero {
            return None;
        }
        judge_location_order(&mut self.judgements, &pointer, what);
        for location in &pointer.locations {
            self.note_reference(location.device, location.slot, 1, what);
        }
        let Some(unit) = read_referenced_unit(
            self.reader,
            &mut self.judgements,
            &pointer.locations,
            node_bytes(),
            what,
        ) else {
            self.walk_failures
                .push(format!("{what} 两份都读不到对得上的"));
            return None;
        };
        if !self
            .visited_units
            .insert((pointer.locations[0].device, pointer.locations[0].slot))
        {
            return None;
        }
        if !self.judge_unit_header(&unit, 2, what) {
            return None;
        }
        self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, what);
        let Ok(view) = index_node_view(&unit) else {
            self.walk_failures.push(format!("{what} 的条目区解不开"));
            return None;
        };
        self.judgements
            .judge("I-1.3", view.tree_identifier == expected_tree, || {
                format!(
                    "{what}：头里的树 ID {} 不是引用它的树 {expected_tree}",
                    view.tree_identifier
                )
            });
        let keys = check_index_node_keys(&view, schema);
        self.judgements.judge("I-1.1", keys.is_ok(), || {
            format!("{what}：key 宽 / key 区间与条目不符（{keys:?}）")
        });
        Some(view)
    }

    fn walk_root(&mut self, record: &[u8], is_newest: bool) {
        // 实例表：根记录直接持有第 0 片（自证单元，I-1.3 豁免），后面各片由前一片的链指针记录指着。
        let first_page_pointer = parse_node_pointer(&record[170..256]);
        let mount_root_instance = u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
        self.walk_instance_table_chain(
            &first_page_pointer,
            is_newest.then_some(mount_root_instance),
        );
        // 树表 0 条那一版的分配记录树的根住根记录（C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案）：
        // 那一版没有树表条目可放，也还没登记过任何树 ⇒ 那一片的树 ID 是 0（由根记录独占持有，同树表单元与实例表单元）。
        // 带文件的一版这一项恒全零，`read_index_node` 对全零指针直接交 `None`。
        self.read_index_node(
            &record[342..428],
            0,
            KEY_SCHEMA_ALLOCATION,
            "树表 0 条那一版的分配记录树的根",
        );
        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
        self.walk_tree_table_and_central_mapping_root(&record[36..122], &record[256..342]);
    }

    /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`] 认出来的）：它没有根记录，
    /// 树表指针与映射根指针在那条记录的新根段里，从那里走下去，判定与根环里的候选根同一套。
    /// 根记录里另外两样这里不走，记录的新根段（188 字节）里没有它们的位置：
    /// 实例表指针——覆盖写不重写实例表，这一版的实例表就是它施加在其上那一版的，那一版在候选集里时已经走过；
    /// 「树表 0 条那一版的分配记录树的根」——带文件的一版这一项恒全零。
    /// 那一版不在候选集里、或者由记录施加出来的是树表 0 条的一版时，这两样这里就漏数了：这一格条款没写，没有定案之前照这样走。
    fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {
        let (tree_table_pointer, mapping_root_pointer) =
            version.tree_table_and_mapping_root_pointers();
        self.walk_tree_table_and_central_mapping_root(tree_table_pointer, mapping_root_pointer);
    }

    /// 一版的树表（树 ID 0）与它下面每一条树表条目，再加中央映射树的根：根环里的根从根记录里取这两条指针，
    /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）。
    fn walk_tree_table_and_central_mapping_root(
        &mut self,
        tree_table_pointer: &[u8],
        mapping_root_pointer: &[u8],
    ) {
        // 树表单元：不属于任何树（树 ID 0）。
        let Some(tree_table) =
            self.read_index_node(tree_table_pointer, 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
        else {
            return;
        };
        for entry in &tree_table.entries {
            self.walk_tree_table_entry(entry);
        }
        // 中央映射树不进树表，引用它的是那条映射根指针（根记录里的，或由记录施加出来那一版的新根段里的）：
        // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
        // 与树表条目里的树 ID 对树表指着的根是同一个读法。号不写死 15：回退到树表 0 条的一版之后再发第一个文件版本，
        // 八棵树从水位重新发号（C511（回退到无文件那一版之后诞生代怎么接））。
        if let Some(mapping) = self.read_index_node(
            mapping_root_pointer,
            parse_node_pointer(mapping_root_pointer).birth_tree,
            KEY_SCHEMA_MAPPING,
            "中央映射树的根",
        ) {
            // 条目宽是映射树根头里的一个**盘上字段**（`index_node_view` 只判了它 ≥ key 宽 27）：切到偏移 55 之前
            // 先判一次，窄的如实记一条走读断了、不按字段表的固定偏移切（普查 R2 / R13 的映射那一处）。
            // **这里判的是「今天走读要几个字节」，不是「映射条目该有多宽」**：后者归 C307（映射树两种 key 宽怎么装进
            // 一棵定宽 key 的树），那一条还开着，I-1.10（码 2 条目宽等于字段表宽） 的射程里也没有中央映射树
            // （`field_table_width_of_tree_kind` 不认种类 5）。
            if mapping.entry_width < mapping_entry_bytes() {
                self.walk_failures.push(format!(
                    "中央映射树的根自述的条目宽 {} 小于走读要的 {} 字节，映射条目这一支走不下去",
                    mapping.entry_width,
                    mapping_entry_bytes()
                ));
            } else {
                self.walk_central_mapping_entries(&mapping);
            }
        }
    }

    /// 中央映射树根的条目逐条走：每条条目里那两条位置条目按升序判，指的单元两份各读一遍。
    /// 入参的条目宽由调用方判过 ≥ 55（`walk_tree_table_and_central_mapping_root` 那一道），这里按字段表的固定偏移切。
    fn walk_central_mapping_entries(&mut self, mapping: &crate::IndexNodeView) {
        for entry in &mapping.entries {
            let locations_pointer: Vec<u8> =
                [vec![0u8; 50], entry[27..55].to_vec(), vec![0u8; 8]].concat();
            let view = parse_node_pointer(&locations_pointer);
            judge_location_order(&mut self.judgements, &view, "映射条目");
            let length = if entry[0] == 2 {
                node_bytes()
            } else {
                data_unit_bytes()
            };
            if read_referenced_unit(
                self.reader,
                &mut self.judgements,
                &view.locations,
                length,
                "映射条目指的单元",
            )
            .is_none()
            {
                self.walk_failures
                    .push("映射条目指的单元两份都读不到对得上的".to_string());
            }
        }
    }

    /// 树表的一条条目：条目宽是树表单元头里的一个**盘上字段**（`index_node_view` 只判了它 ≥ key 宽 8），而下面三步——
    /// 读树 ID（偏移 0）、读种类（偏移 10）、切根指针（`entry[14..100]`）——都按字段表的固定偏移切，窄一个字节就当场越界。
    /// 切之前先判一次，窄的如实记一条「走读断了」（I-4.8（近 K 代根校验和自洽） / I-7.4（近 K 代块未被复用） 由它说话），
    /// **不判违例**：I-1.10（码 2 条目宽等于字段表宽） 判的是「树表指着的那棵树的根节点」的条目宽、住在这两处切片之后，
    /// 罩不到树表自己（C504（树表条目宽在走读里无守卫，今天没坏法打得到），2026-09-23 用户定案「只加走读守卫，不动条款」，
    /// I-1.10（码 2 条目宽等于字段表宽） 的射程一个字不扩）。守卫与 `references_under_root` /
    /// `allocation_record_count_under_root` 那两处同款。
    fn walk_tree_table_entry(&mut self, entry: &[u8]) {
        if entry.len() < tree_table_entry_bytes() {
            self.walk_failures.push(format!(
                "树表单元自述的条目宽 {} 小于走读要的 {} 字节，树表条目这一支走不下去",
                entry.len(),
                tree_table_entry_bytes()
            ));
            return;
        }
        let tree = read_u64(entry, 0);
        let kind = read_u16(entry, 10);
        self.tree_identifiers_in_tables.insert(tree);
        let what = format!("树 {tree}（种类 {kind}）的根");
        let Some(schema) = key_schema_for_tree_kind(kind) else {
            return;
        };
        if kind == TREE_KIND_INODE {
            let pointer = parse_node_pointer(&entry[14..100]);
            if !pointer.all_zero {
                if let Some(bytes) = self.reader.read(
                    pointer.locations[0].device,
                    pointer.locations[0].slot * SLOT_BYTES,
                    node_bytes(),
                ) {
                    self.judgements.judge("I-9.1", bytes[6] == 2, || {
                        format!("树表指向的 inode 树根类标签是 {}，不是 2", bytes[6])
                    });
                }
            }
        }
        let Some(node) = self.read_index_node(&entry[14..100], tree, schema, &what) else {
            return;
        };
        // I-1.10：头里自述的条目宽等于这棵树登记的条目字段表宽度，**先于**按字段表解条目判
        // （与 I-1.7（打包容器合法与判定顺序） 判码 3 记录宽同一纪律）。不等就不往下解：下面每一支都按
        // 24 / 34 / 112 / 120 这些固定偏移切条目，窄一个字节就当场越界（普查 R13）。
        // 中央映射树（种类 5）走不到这里判：`field_table_width_of_tree_kind` 里没有它，C307 还开着。
        if let Some(entry_field_table_bytes) = field_table_width_of_tree_kind(kind) {
            let entry_width = node.entry_width;
            self.judgements
                .judge("I-1.10", entry_width == entry_field_table_bytes, || {
                    format!(
                        "{what}：头里自述的条目宽 {entry_width} 不等于这棵树的条目字段表宽度 {entry_field_table_bytes}"
                    )
                });
            if entry_width != entry_field_table_bytes {
                return;
            }
        }
        match kind {
            TREE_KIND_INODE => self.walk_inode_root(&node, tree),
            TREE_KIND_EXTENT | TREE_KIND_ALLOCATION | TREE_KIND_ACCOUNTING if node.level > 0 => {
                // 另外四棵树的内部节点条目格式没有条款（总审核 D8-D11 发现 16）：走不下去就如实说。
                self.judgements.not_applicable(
                    "I-7.2",
                    "extent / 分配 / 记账树有内部节点，而它们的内部条目格式还没有条款",
                );
            }
            TREE_KIND_EXTENT => self.walk_extent_leaf(&node, tree),
            TREE_KIND_ACCOUNTING => {
                self.accounting_seen = true;
                for row in &node.entries {
                    self.accounting
                        .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
                }
            }
            _ => {}
        }
    }

    /// I-9.1：inode 树根恒码 2（读到这里时它已经按码 2 判过头）；层级 1 的条目逐条判 I-9.2 并走到叶；
    /// 逐条记下「分隔 key 与这个孩子的 key 区间」，走完按 I-9.12（分隔 key 落在孩子区间之外） 判一遍。
    fn walk_inode_root(&mut self, root: &crate::IndexNodeView, tree: u64) {
        if root.level == 0 {
            return;
        }
        self.inode_tree_walked = true;
        // (分隔 key, 这个孩子装的最小 key, 最大 key)：读不出孩子的那几条不进来，I-9.12 只判读得出的那些。
        let mut separator_and_child_range: Vec<(u64, u64, u64)> = Vec::new();
        for entry in &root.entries {
            let record_type = read_u16(entry, 16);
            let identity = (
                read_u64(entry, 8),
                record_type,
                read_u64(entry, 18),
                read_u64(entry, 26),
            );
            let child = parse_node_pointer(&entry[34..120]);
            judge_location_order(&mut self.judgements, &child, "inode 内部条目的子指针");
            let length = if record_type == PACKED_TYPE_INODE {
                data_unit_bytes()
            } else {
                node_bytes()
            };
            for location in &child.locations {
                self.note_reference(
                    location.device,
                    location.slot,
                    if record_type == PACKED_TYPE_INODE {
                        2
                    } else {
                        1
                    },
                    "inode 树的孩子",
                );
            }
            let Some(unit) = read_referenced_unit(
                self.reader,
                &mut self.judgements,
                &child.locations,
                length,
                "inode 树的孩子",
            ) else {
                self.walk_failures
                    .push("inode 树的孩子两份都读不到对得上的".to_string());
                continue;
            };
            let header_identity = (
                read_u64(&unit, 43),
                read_u16(&unit, 51),
                read_u64(&unit, 53),
                read_u64(&unit, 61),
            );
            let entry_holds = record_type == PACKED_TYPE_INODE
                && unit[6] == 3
                && header_identity == identity
                && child.birth_tree == identity.0;
            self.judgements.judge("I-9.2", entry_holds, || {
                format!(
                    "inode 内部条目的身份 {identity:?} 与孩子的头 {header_identity:?}（类 {}）不符",
                    unit[6]
                )
            });
            if !self
                .visited_units
                .insert((child.locations[0].device, child.locations[0].slot))
            {
                continue;
            }
            self.judgements
                .judge("I-1.3", read_u64(&unit, 43) == tree, || {
                    format!("inode 树叶的出生树 {} 不是 {tree}", read_u64(&unit, 43))
                });
            self.judge_birth_identity_of_a_referenced_unit(&unit, &child, "inode 树的叶容器");
            if let Some(records) =
                self.judge_packed_container(&unit, PACKED_TYPE_INODE, "inode 树的叶容器")
            {
                let container = read_u64(&unit, 53);
                self.judgements.judge("I-9.13", !records.is_empty(), || {
                    "类型 2 容器一条记录都没有".to_string()
                });
                let mut smallest_inode_in_this_container: Option<u64> = None;
                let mut largest_inode_in_this_container: Option<u64> = None;
                for record in records {
                    let inode = read_u64(&record, 0);
                    self.judgements.judge(
                        "I-9.7",
                        record[108..140].iter().all(|byte| *byte == 0),
                        || format!("inode {inode} 的记录偏移 108 起三段不全为 0"),
                    );
                    self.judgements.judge("I-9.4", inode >= container, || {
                        format!("inode {inode} 小于它所在容器的号 {container}")
                    });
                    self.inode_object_birth.insert(inode, read_u64(&record, 8));
                    smallest_inode_in_this_container = Some(
                        smallest_inode_in_this_container.map_or(inode, |seen| seen.min(inode)),
                    );
                    largest_inode_in_this_container =
                        Some(largest_inode_in_this_container.map_or(inode, |seen| seen.max(inode)));
                    self.largest_inode_number_in_the_inode_tree = Some(
                        self.largest_inode_number_in_the_inode_tree
                            .map_or(inode, |seen| seen.max(inode)),
                    );
                }
                if let (Some(smallest), Some(largest)) = (
                    smallest_inode_in_this_container,
                    largest_inode_in_this_container,
                ) {
                    separator_and_child_range.push((read_u64(entry, 0), smallest, largest));
                }
            }
        }
        // I-9.12：条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。
        // 判的是读得出、这一遍第一次走到的那些孩子；一个都读不出来时这条在这份镜像上没有对象（末尾按「走到过 inode 树」报不适用）。
        for (position, (separator, smallest, _largest_of_this_child)) in
            separator_and_child_range.iter().enumerate()
        {
            self.judgements.judge("I-9.12", separator <= smallest, || {
                format!("第 {position} 条的分隔 key {separator} 大于这个孩子的最小 key {smallest}")
            });
            if position > 0 {
                let (previous_separator, _, previous_largest) =
                    separator_and_child_range[position - 1];
                self.judgements
                    .judge("I-9.12", *separator > previous_separator, || {
                        format!("第 {position} 条的分隔 key {separator} 不大于上一条的 {previous_separator}")
                    });
                self.judgements
                    .judge("I-9.12", *separator > previous_largest, || {
                        format!(
                            "第 {position} 条的分隔 key {separator} 不大于上一个孩子的最大 key {previous_largest}"
                        )
                    });
            }
        }
    }

    /// 沿链走一张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
    /// 每一片判 I-2.5（位置条目升序）、I-2.1（校验和）、码 3 容器那几条（`judge_packed_container`）、I-1.1（身份四元组 =
    /// (0, 4, 片序号, 0)：链指针指到了别的单元、链成了环，都在这一格红）与 I-3.8 的链指针记录那一半（最后一条是合法的链指针记录）；
    /// 挂载根那一张（`mount_root_instance_when_newest` 给了实例代号）把各片的记录交给 `judge_instance_table_rows` 判行那一半。
    /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`），每一轮是盘上一个不同的单元。
    /// 提前出口，链都不再往下走：读不出、身份不对（被引用的单元坏了或换了人，记进走读失败；头用不了由 `judge_unit_header` 记）；
    /// 容器记录不合法、链指针记录不合法（这一片自己写错了，只红 I-1.7 / I-3.8 那一格）；走到已经走过的单元（前一条根走过这条链）。
    fn walk_instance_table_chain(
        &mut self,
        first_page_pointer: &PointerView,
        mount_root_instance_when_newest: Option<u32>,
    ) {
        let mut pointer = *first_page_pointer;
        let mut page_index: u64 = 0;
        let mut pages: Vec<Vec<Vec<u8>>> = Vec::new();
        loop {
            let (what, pointer_what) = if page_index == 0 {
                ("实例表单元".to_string(), "实例表指针".to_string())
            } else {
                (
                    format!("实例表第 {page_index} 片"),
                    format!("实例表第 {} 片的链指针", page_index - 1),
                )
            };
            judge_location_order(&mut self.judgements, &pointer, &pointer_what);
            for location in &pointer.locations {
                self.note_reference(location.device, location.slot, 2, &what);
            }
            let Some(unit) = read_referenced_unit(
                self.reader,
                &mut self.judgements,
                &pointer.locations,
                data_unit_bytes(),
                &what,
            ) else {
                self.walk_failures
                    .push(format!("{what}两份都读不到对得上的"));
                break;
            };
            if !self
                .visited_units
                .insert((pointer.locations[0].device, pointer.locations[0].slot))
            {
                break;
            }
            let Some(records) =
                self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, &what)
            else {
                break;
            };
            let identity = (
                read_u64(&unit, 43),
                read_u16(&unit, 51),
                read_u64(&unit, 53),
                read_u64(&unit, 61),
            );
            let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0);
            self.judgements
                .judge("I-1.1", identity == expected_identity, || {
                    format!(
                        "{what}：身份四元组 (出生树, 类型, 容器号, 容器出生代) = {identity:?}，链上第 {page_index} 片该是 {expected_identity:?}"
                    )
                });
            if identity != expected_identity {
                self.walk_failures
                    .push(format!("{what}：指到的单元不是链上第 {page_index} 片"));
                break;
            }
            let chain = records
                .last()
                .map_or(ChainRecordView::Malformed, |last| chain_record_view(last));
            self.judgements
                .judge("I-3.8", chain != ChainRecordView::Malformed, || {
                    format!(
                        "{what}：最后一条不是合法的链指针记录（kind 1、有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零）"
                    )
                });
            pages.push(records);
            match chain {
                ChainRecordView::LastPage => break,
                ChainRecordView::NextPage(next_page) => {
                    pointer = next_page;
                    page_index += 1;
                }
                // 链指针记录自相矛盾：上面 I-3.8 那一格已经红了，链不往下走。不记走读失败——走读失败驱动 I-7.4 / I-4.8
                // 「引用的单元已被复用或抹头」，这里没有哪个被引用的单元坏了，是这一片自己的记录写错了。
                ChainRecordView::Malformed => break,
            }
        }
        if let Some(mount_root_instance) = mount_root_instance_when_newest {
            if !pages.is_empty() {
                self.judge_instance_table_rows(&pages, mount_root_instance);
            }
        }
    }

    /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
    /// I-3.8（实例表行唯一且低于挂载根）：整张表（链上各片的行接起来）里 `kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例；
    /// 链指针记录（`kind` 1）恒为每一片的最后一条（D18（块里携带什么信息） 已定项 11）。「行只在回收条件成立后删」对着一个镜像判不了。
    fn judge_instance_table_rows(&mut self, pages: &[Vec<Vec<u8>>], mount_root_instance: u32) {
        let mut instances: Vec<u32> = Vec::new();
        let mut unique = true;
        let mut below_mount_root = true;
        let mut chain_record_last = true;
        for records in pages {
            let mut chain_record_last_in_this_page = false;
            for (index, row) in records.iter().enumerate() {
                match row.first().copied() {
                    Some(0) => {
                        let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
                        if instances.contains(&instance) {
                            unique = false;
                        }
                        instances.push(instance);
                        self.instance_table_rows.push(InstanceTableRow {
                            instance,
                            published_checkpoint_txg: u64::from_le_bytes(
                                row[5..13].try_into().expect("8 字节"),
                            ),
                            applied_transaction_high_water_mark: u64::from_le_bytes(
                                row[13..21].try_into().expect("8 字节"),
                            ),
                            is_rollback: row[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
                                & INSTANCE_TABLE_ROW_FLAG_ROLLBACK
                                != 0,
                        });
                        if instance >= mount_root_instance {
                            below_mount_root = false;
                        }
                        chain_record_last_in_this_page = false;
                    }
                    Some(1) => chain_record_last_in_this_page = index + 1 == records.len(),
                    _ => chain_record_last_in_this_page = false,
                }
            }
            chain_record_last &= chain_record_last_in_this_page;
        }
        self.judgements.judge(
            "I-3.8",
            unique && below_mount_root && chain_record_last,
            || {
                format!(
                    "实例表（{} 片）：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}），每一片的链指针记录在末尾 = {chain_record_last}",
                    pages.len()
                )
            },
        );
    }

    fn judge_packed_container(
        &mut self,
        unit: &[u8],
        expected_type: u16,
        what: &str,
    ) -> Option<Vec<Vec<u8>>> {
        if !self.judge_unit_header(unit, 3, what) {
            return None;
        }
        let record_type = read_u16(unit, 51);
        let count = usize::from(read_u16(unit, 69));
        let width = usize::from(read_u16(unit, 71));
        let declared = usize::from(read_u16(unit, 8));
        let legal = record_type == expected_type
            && registered_record_width(record_type) == Some(width)
            && count * width <= declared
            && declared <= data_unit_bytes() - 136;
        self.judgements.judge("I-1.7", legal, || format!("{what}：类型 {record_type}、记录宽 {width}、记录数 {count}、声明长度 {declared} 不合法"));
        legal.then(|| {
            unit[136..136 + count * width]
                .chunks(width.max(1))
                .map(<[u8]>::to_vec)
                .collect()
        })
    }

    fn walk_extent_leaf(&mut self, leaf: &crate::IndexNodeView, tree: u64) {
        for record in &leaf.entries {
            let inode = read_u64(record, 8);
            let offset = read_u64(record, 16);
            let pointer = parse_data_pointer(&record[24..112]);
            judge_location_order(&mut self.judgements, &pointer, "extent 记录的数据指针");
            for location in &pointer.locations {
                self.note_reference(location.device, location.slot, 2, "数据单元");
            }
            let Some(unit) = read_referenced_unit(
                self.reader,
                &mut self.judgements,
                &pointer.locations,
                data_unit_bytes(),
                "数据单元",
            ) else {
                self.walk_failures
                    .push("数据单元两份都读不到对得上的".to_string());
                continue;
            };
            if !self
                .visited_units
                .insert((pointer.locations[0].device, pointer.locations[0].slot))
                || !self.judge_unit_header(&unit, 1, "数据单元")
            {
                continue;
            }
            self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");
            self.judgements
                .judge("I-1.3", read_u64(&unit, 43) == tree, || {
                    format!(
                        "数据单元头里的树 ID {} 不是 extent 树 {tree}",
                        read_u64(&unit, 43)
                    )
                });
            let five_tuple_holds = read_u64(&unit, 51) == inode
                && read_u64(&unit, 67) == offset
                && read_u64(&unit, 43) == tree;
            self.judgements.judge("I-1.1", five_tuple_holds, || {
                format!("数据单元五元组（树 {}、对象 {}、锚点 {}）与 extent key（{tree}, {inode}, {offset}）不符", read_u64(&unit, 43), read_u64(&unit, 51), read_u64(&unit, 67))
            });
            self.data_unit_objects.push((
                inode,
                read_u64(&unit, 59),
                format!("inode {inode} 偏移 {offset} 的数据单元"),
            ));
        }
    }
}

/// I-7.8 扫描方向的第二道：一个码 2 节点是不是「写序实例 i 崩在发布之前写出的孤儿」——最新根指着的实例表里有 i 的一行
/// (i, Ti, ·)，它**不是回退行**、Ti > 0，而节点的诞生代号 > Ti（改法 D，`research/prompts/m2-wave3-code-r1-main-verification.md`
/// 第三节 Y1 与第四节第 1 条，被攻过零轮）。
///
/// 为什么排除：非回退行 (i, Ti, ·) 说实例 i 被施加到 Ti 为止（D23（journal 的角色与格式） 已定项 14，写行时取恢复之后的有效根），
/// 诞生代号 > Ti 的实例 i 节点就没有发布过。只数诞生代号 ≤ 根环最大 txg 那一道挡不住它：下一个实例第一次发布取的 txg 可以
/// 等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 的「≥ max(…) + 1」取等号），孤儿就被数进「出现过的」、在合法状态上判红。
/// 不排除的两种：回退行（它的 Ti 之后那段被抛弃的时间线发布过根、发过号，D8（核心索引结构） 已定项 8 ② 要水位盖住它们）；
/// Ti = 0 的行（回退写的中间实例行 (i, 0, 0) 与没发布过根的实例行，行里说不出它发布到哪一代）。
/// 节点头 fsid 不是本池的读不出写序实例（`unit_write_order_instance` 交 None），照旧算。
fn written_after_its_instance_was_last_applied(
    node: &[u8],
    birth_txg: u64,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
) -> bool {
    unit_write_order_instance(node, filesystem_identifier_low).is_some_and(|write_order_instance| {
        instance_table_rows.iter().any(|row| {
            row.instance == write_order_instance
                && !row.is_rollback
                && row.published_checkpoint_txg > 0
                && birth_txg > row.published_checkpoint_txg
        })
    })
}

/// 扫描方向：候选槽上头校验和过的码 2 节点，取它头里的树 ID。只数诞生代号不超过 `newest_published_txg` 的节点：
/// 根环里没有一条根发布过的那一代，单元还不算「出现过」（I-7.8 的这一读法 2026-09-14 用户收尾弹窗定甲；字面读法会把
/// 崩在发布之前的那个事务写出的孤儿节点也算进去，见 records 十一·七）。再排除最新根的实例表判得出没发布过的孤儿
/// （[`written_after_its_instance_was_last_applied`]）。
fn scanned_tree_identifiers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    newest_published_txg: u64,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
) -> BTreeSet<u64> {
    let mut identifiers = BTreeSet::new();
    for device in reader.devices() {
        let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
            (geometry.unit_area_start_slot..end).collect()
        });
        for slot in slots {
            let Some(bytes) = reader.read(device, slot * SLOT_BYTES, node_bytes()) else {
                continue;
            };
            if &bytes[..4] != b"SFSU" || bytes[6] != 2 {
                continue;
            }
            let header_end = 86 + 2 * usize::from(bytes[51]);
            if header_end < bytes.len() && checksum_field_holds(&bytes, header_end, 10) {
                let birth_txg = read_u64(&bytes, 52 + 2 * usize::from(bytes[51]));
                if birth_txg <= newest_published_txg
                    && !written_after_its_instance_was_last_applied(
                        &bytes,
                        birth_txg,
                        instance_table_rows,
                        filesystem_identifier_low,
                    )
                {
                    identifiers.insert(read_u64(&bytes, 42));
                }
            }
        }
    }
    identifiers
}

/// 扫描方向读单元头读多少：码 2 头最宽 86 + 2 × 255 = 596 字节，读一块 4096 就罩得住三类。
const UNIT_HEADER_SCAN_BYTES: usize = 4096;

/// 单元头写序里的实例代号：头校验和过、fsid 与本池相同才算（码 1：fsid 83、写序 91、头末 105；码 2：fsid 60 + 2k、
/// 写序 68 + 2k、头末 86 + 2k；码 3：fsid 81、写序 93、头末 107——D18（块里携带什么信息） 已定项 18 的偏移表）。
fn unit_write_order_instance(bytes: &[u8], filesystem_identifier_low: u64) -> Option<u32> {
    if bytes.len() < 52 || &bytes[..4] != b"SFSU" {
        return None;
    }
    let (header_end, fsid_offset, instance_offset) = match bytes[6] {
        1 => (105, 83, 91),
        2 => {
            let key_span = 2 * usize::from(bytes[51]);
            (86 + key_span, 60 + key_span, 68 + key_span)
        }
        3 => (107, 81, 93),
        _ => return None,
    };
    if header_end > bytes.len()
        || !checksum_field_holds(bytes, header_end, 10)
        || read_u64(bytes, fsid_offset) != filesystem_identifier_low
    {
        return None;
    }
    Some(read_u32(bytes, instance_offset))
}

/// 盘上带实例代号的东西：根环里自证过的根、journal 环里 fsid 相同的记录、单元区里头校验和过且 fsid 相同的单元写序。
/// 任何一个该读的槽读不出 ⇒ None（I-7.7（系统配置实例代号不低于根环） 读不全报不适用）。
fn instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    filesystem_identifier_low: u64,
) -> Option<Vec<(String, u32)>> {
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    for (_, _, device, offset) in root_slot_positions(geometry) {
        reader.read(device, offset, root_slot_bytes)?;
    }
    let mut carriers: Vec<(String, u32)> = roots
        .iter()
        .map(|(region, slot, root)| (format!("根环区域 {region} 槽 {slot} 的根"), root.instance))
        .collect();
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in reader.devices() {
        let journal_slots = reader
            .candidate_journal_slots(device)
            .unwrap_or_else(|| (0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).collect());
        for slot in journal_slots {
            let offset =
                geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
            let bytes = reader.read(device, offset, record_bytes)?;
            if let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) {
                carriers.push((
                    format!("盘 {device} journal 环槽 {slot} 的记录"),
                    record.instance,
                ));
            }
        }
        let unit_slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
            (geometry.unit_area_start_slot..end).collect()
        });
        for slot in unit_slots {
            let bytes = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)?;
            if let Some(instance) = unit_write_order_instance(&bytes, filesystem_identifier_low) {
                carriers.push((format!("盘 {device} 槽 {slot} 的单元写序"), instance));
            }
        }
    }
    Some(carriers)
}

/// I-7.7（系统配置实例代号不低于根环） 的 ① ②（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：
/// 每盘的实例代号取它两槽里全部自证过（fsid 与本池相同）的槽中最大的；① 本池的每个根记录、journal 记录、单元写序的实例代号
/// 不超过各盘的最大者，② 各盘不等时较大者不出现在它们里——两句合起来就是「每一个带号的东西都不超过各盘最大号里最小的那个」。
fn judge_instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let highest_by_device: Vec<Option<u32>> = verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(_, slots)| {
            slots
                .iter()
                .filter(|(view, _)| view.filesystem_identifier == geometry.filesystem_identifier)
                .map(|(view, _)| view.journal_instance)
                .max()
        })
        .collect();
    let Some(highest_by_device) = highest_by_device.into_iter().collect::<Option<Vec<u32>>>()
    else {
        judgements.not_applicable(
            "I-7.7",
            "有一块盘没有本池 fsid 的系统配置槽：那块盘归不归这个池由 I-1.4 判",
        );
        return;
    };
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("fsid 前 8 字节"),
    );
    let Some(carriers) = instance_carriers(reader, geometry, roots, filesystem_identifier_low)
    else {
        judgements.not_applicable(
            "I-7.7",
            "根环、journal 环或单元区有读不出的槽：带号的东西数不全，判不了",
        );
        return;
    };
    let pool_highest = highest_by_device.iter().copied().max().unwrap_or(0);
    let pool_lowest = highest_by_device.iter().copied().min().unwrap_or(0);
    let above_every_disk = carriers
        .iter()
        .find(|(_, instance)| *instance > pool_highest);
    judgements.judge("I-7.7", above_every_disk.is_none(), || {
        let (what, instance) = above_every_disk.expect("判红时有");
        format!("{what}带实例代号 {instance}，高于各盘系统配置的最大者 {pool_highest}（①）")
    });
    let witnessed_by_one_disk = carriers
        .iter()
        .find(|(_, instance)| *instance > pool_lowest);
    judgements.judge("I-7.7", witnessed_by_one_disk.is_none(), || {
        let (what, instance) = witnessed_by_one_disk.expect("判红时有");
        format!("各盘实例代号 {highest_by_device:?} 不等，而{what}带着较大的 {instance}（②）")
    });
}

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备身份 4, 16 KiB 槽号 6) + value (跨度段 2，最高位是已释放标志,
/// 分配代或释放代 8)。checker 按字段表自己解一份，不与实现共用代码（D13（验证路线） 已定项 5）。
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
/// 树表条目 200（D8（核心索引结构） 已定项 8）：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 +
/// `previous_snapshot_txg` 8 + **诞生 txg 8** + 头 ID 8 + 预留 76。
const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;

/// 一条分配记录在盘上的样子。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocationRecordView {
    device: u32,
    slot: u64,
    /// 跨度段去掉已释放标志之后的值：这条记录罩住 [槽号, 槽号 + 跨度) 这些槽。
    span_slots: u64,
    /// 仍分配时是分配代；已释放时是释放代（D3（空间分配） 已定项 7）。
    generation: u64,
    is_released: bool,
}

fn parse_allocation_record(entry: &[u8]) -> AllocationRecordView {
    let span_field = read_u16(entry, ALLOCATION_RECORD_SPAN_FIELD_OFFSET);
    AllocationRecordView {
        device: read_u32(entry, 0),
        slot: read_six_byte_unsigned(entry, 4),
        span_slots: u64::from(span_field & !ALLOCATION_RECORD_RELEASED_FLAG),
        generation: read_u64(entry, ALLOCATION_RECORD_GENERATION_OFFSET),
        is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
    }
}

/// 树的种类码（D8（核心索引结构） 已定项 9 的登记表）在引用集合这一遍里的读法：盘上读到的码语义上允许未知，
/// 做成一个显式成员，`match` 照样穷举（`code-discipline.md`「分支：穷举写全，不写通配臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeKindForReferenceScan {
    Extent,
    Inode,
    Allocation,
    Accounting,
    /// 条目格式还没有条款的树（livelist 6、稀疏旁表 7、deadlist 8 与这一版不认识的码）：day-1 注册的那三棵根指针为零、
    /// 走到这里就是它们真有了根节点，那时这条根的引用集合数不全。
    WithoutWalkableEntryFormat(u16),
}

impl TreeKindForReferenceScan {
    const fn of(code: u16) -> Self {
        match code {
            TREE_KIND_EXTENT => TreeKindForReferenceScan::Extent,
            TREE_KIND_INODE => TreeKindForReferenceScan::Inode,
            TREE_KIND_ALLOCATION => TreeKindForReferenceScan::Allocation,
            TREE_KIND_ACCOUNTING => TreeKindForReferenceScan::Accounting,
            unregistered => TreeKindForReferenceScan::WithoutWalkableEntryFormat(unregistered),
        }
    }
}

/// 一条有效根自己引用了哪些落点、它的树表里有哪些条目：I-3.9（释放代落在停止引用它的那一格区间里）与
/// I-9.14（树表条目的诞生 txg 跨根不变）按回退候选集里每条根各走一遍取这份东西（I-7.4（近 K 代块未被复用） 已经这么走）。
/// **与主走读那一遍分开跑**：主走读用 `visited_units` 跳过走过的单元，共享子树只算进第一条走到它的根，
/// 而这两条要的正是「这条根自己引用了什么」；这一遍只读不判，别的不变量的评估次数不受它影响。
#[derive(Clone)]
struct RootReferences {
    /// 这条根引用的落点 (设备身份, 起点槽)：与分配记录的 key 同形。只在 `depth` 是「走到叶」时数得全。
    placements: BTreeSet<(u32, u64)>,
    /// 树表条目：树 ID → 诞生 txg。
    tree_table_birth_txg: BTreeMap<u64, u64>,
    /// 树表单元自己落在哪 (设备身份, 起点槽)：几条根共用同一个树表单元时，它们的条目不是两处独立的证据。
    tree_table_placement: Option<(u32, u64)>,
    /// 这条根的分配记录树叶里的记录（I-3.9 只取最新根那一份：当前的账）。
    allocation_records: Vec<AllocationRecordView>,
    /// 这一遍按哪个深度扫的：`placements` 与 `is_complete` 只有「走到叶」那个深度上作数。
    depth: ReferenceScanDepth,
    /// 该读的单元读不出、或走到了没有条款的形状：这条根的引用集合数不全，I-3.9 判不了。
    is_complete: bool,
}

impl RootReferences {
    fn note(&mut self, pointer: &PointerView) {
        if pointer.all_zero {
            return;
        }
        for location in &pointer.locations {
            self.placements.insert((location.device, location.slot));
        }
    }
}

/// 按位置条目读一个单元、不判任何不变量：两条位置条目里第一份校验和对得上的那一份（判定归主走读的 I-2.1）。
fn read_unit_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    unit_bytes: usize,
) -> Option<Vec<u8>> {
    pointer.locations.iter().find_map(|location| {
        reader
            .read(location.device, location.slot * SLOT_BYTES, unit_bytes)
            .filter(|content| crc32_castagnoli_table(content) == location.checksum)
    })
}

/// 这一遍里读过的码 2 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的节点（解不开记 None）。
/// 候选集里几条根常指着同一个单元（暖机那几代都指着 mkfs 种的第 0 版树表，共享子树在每条根上都要算一遍），
/// 按位置条目记一份，省掉重复的整单元 CRC——层 0 每个崩溃状态都要跑这一遍。
type IndexNodeCache = BTreeMap<(u32, u64, u32), Option<crate::IndexNodeView>>;

fn read_index_node_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Option<crate::IndexNodeView> {
    let key = (
        pointer.locations[0].device,
        pointer.locations[0].slot,
        pointer.locations[0].checksum,
    );
    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }
    let parsed = read_unit_without_judging(reader, pointer, node_bytes())
        .and_then(|unit| index_node_view(&unit).ok());
    cache.insert(key, parsed.clone());
    parsed
}

/// 这一遍要走多深。I-9.14（树表条目的诞生 txg 跨根不变） 只要每条根的树表；I-3.9（释放代落在停止引用它的那一格区间里）
/// 要这条根引用的全部落点，而那要把每棵树都走到叶——最新根那棵账里一条已释放记录都没有时，别的根就不必走那么深
/// （层 0 上多数崩溃状态是这一类：最新的根还是暖机那几代，树表是空的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceScanDepth {
    TreeTableOnly,
    EveryReferencedPlacement,
}

/// 从一条根走一遍，取它的树表条目；`depth` 是「走到叶」时再取它引用的全部落点与分配记录。只读不判。
fn references_of_root(
    reader: &dyn ImageReader,
    record: &[u8],
    depth: ReferenceScanDepth,
    cache: &mut IndexNodeCache,
) -> RootReferences {
    let mut references = RootReferences {
        placements: BTreeSet::new(),
        tree_table_birth_txg: BTreeMap::new(),
        tree_table_placement: None,
        allocation_records: Vec::new(),
        depth,
        is_complete: true,
    };
    let instance_table = parse_node_pointer(&record[170..256]);
    references.note(&instance_table);
    // ⚠️ 实例表链上第 1 片起的落点这一遍不认（D18（块里携带什么信息） 已定项 11 的链）：今天的写者只写一片
    // （多片在 bump 次序里怎么排、行怎么分片没有条款，`MountError::InstanceTableChainLongerThanOnePageUndecided`），
    // 带已释放记录、能让 I-3.9 看到第 1 片的镜像写不出来，认链的这一段没有用例能证它会红。写者写第二片那一次一起补。
    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，不重复数——
    // 主走读的 `note_reference` 也不数它们（数了 I-3.1（已分配统计对得上） 与 I-5.1（引用不重叠） 会把同一个落点算两遍）。
    let mapping_root = parse_node_pointer(&record[256..342]);
    references.note(&mapping_root);
    // 树表 0 条那一版的分配记录树的根（C512（树表 0 条的一版上被换下的单元记在哪））：它的落点同样只有根记录指着，
    // 不数它，被抛弃根的影子账与 I-3.9 的引用集合就少一片。
    let allocation_record_tree_root = parse_node_pointer(&record[342..428]);
    references.note(&allocation_record_tree_root);
    let tree_table_pointer = parse_node_pointer(&record[36..122]);
    references.note(&tree_table_pointer);
    if tree_table_pointer.all_zero {
        references.is_complete = false;
        return references;
    }
    references.tree_table_placement = Some((
        tree_table_pointer.locations[0].device,
        tree_table_pointer.locations[0].slot,
    ));
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        references.is_complete = false;
        return references;
    };
    for entry in &tree_table.entries {
        // 条目宽是节点头里的一个字段：头校验和过、载荷 CRC 过而条目宽被改窄的镜像也要能判（那由 I-1.7 / I-1.1 说话），
        // 这一遍只如实记「数不全」，不按短条目去解字段。
        if entry.len() < tree_table_entry_bytes() {
            references.is_complete = false;
            continue;
        }
        let tree = read_u64(entry, 0);
        references
            .tree_table_birth_txg
            .insert(tree, read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET));
        let tree_root = parse_node_pointer(&entry[14..100]);
        if tree_root.all_zero {
            // day-1 注册、还没有根节点的树（livelist 6、稀疏旁表 7、deadlist 8）：不占落点。
            continue;
        }
        references.note(&tree_root);
        if depth == ReferenceScanDepth::TreeTableOnly {
            continue;
        }
        let Some(node) = read_index_node_without_judging(reader, &tree_root, cache) else {
            references.is_complete = false;
            continue;
        };
        collect_tree_references(&mut references, &node, read_u16(entry, 10));
    }
    references
}

/// 一棵树的根节点下面还引用了什么。第一版只有「extent 叶的数据指针」与「inode 树内部条目的子指针」两种下探；
/// 别的形状（另外四棵树的内部节点、条目格式没有条款的树）走不下去，如实记成数不全。
fn collect_tree_references(
    references: &mut RootReferences,
    node: &crate::IndexNodeView,
    tree_kind_code: u16,
) {
    // 条目宽窄于字段表的（节点头里那个字段被改过）不按短条目去解，如实记成数不全：宽度本身由 I-1.7 / I-1.1 说话。
    match TreeKindForReferenceScan::of(tree_kind_code) {
        TreeKindForReferenceScan::Extent => {
            if node.level == 0 && node.entry_width >= extent_leaf_record_bytes() {
                for record in &node.entries {
                    let data = parse_data_pointer(&record[24..112]);
                    references.note(&data);
                }
            } else {
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::Inode => {
            if node.level > 0 && node.entry_width >= inode_internal_entry_bytes() {
                for entry in &node.entries {
                    let child = parse_node_pointer(&entry[34..120]);
                    references.note(&child);
                }
            } else {
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::Allocation => {
            if node.level == 0 && node.entry_width >= allocation_record_bytes() {
                for record in &node.entries {
                    references
                        .allocation_records
                        .push(parse_allocation_record(record));
                }
            } else {
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::Accounting => {
            // 记账行不引用别的单元；有内部节点时它的条目格式没有条款（总审核 D8-D11 发现 16）。
            if node.level > 0 {
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => references.is_complete = false,
    }
}

/// 候选集里的一条根：它在 `roots` 里的下标、它的 checkpoint_txg 与这一遍取到的引用集合。
struct ScannedCandidateRoot {
    root_index: usize,
    checkpoint_txg: u64,
    references: RootReferences,
}

/// 一棵树的树表条目在某一条有效根那里的样子（I-9.14 按树 ID 归并这些）。
struct TreeTableEntrySighting {
    root_checkpoint_txg: u64,
    /// 这条根的树表单元落在哪 (设备身份, 起点槽)：两条根共用一个树表单元时，它们的条目不是两处独立的证据。
    tree_table_placement: (u32, u64),
    birth_txg: u64,
}

/// I-3.9（释放代落在停止引用它的那一格区间里）：对最新根那棵账里每条带已释放标志的分配记录，在候选集里取还引用这个落点的
/// 最新一条有效根（L）与最早不再引用它的有效根（T，L 之后的第一条），释放代必须落在 `(L, T]` 里；每条有效根都还引用它的
/// （L 之后没有根），它不许带已释放标志。候选集里一条根都没引用过它的（已经回收、还没被复用：见证它释放的根已不在候选集里）
/// 跳过不判，全是这一类时整条报不适用——三处都照 `.claude/kb/invariants.md` I-3.9 那一行。
///
/// 为什么是区间：根环里的 txg 可以有洞——一次发布的根槽从没写过、而它的 journal 记录在挂载时被施加（D23（journal 的角色与格式）：
/// 记录自带新根段），那次发布做的释放就写着一个环里没有根的 txg。`second_transaction_step_zero_layer0.rs` 的残留记录那条流
/// 正是这个形状（txg 5 的根槽从没写过、jsn 5 被施加、释放代 5、环里最早不再引用那个落点的根是 txg 6），按「等于 T」判，
/// 那条流上 12 个合法状态判红。L 与 T 之间没有洞时 `(L, T]` 只有 T 一个值，就是「等于最早不再引用它的那条根的 txg」；
/// C374（释放代与树表诞生 txg 只有验收断言盯着） 点名的判别力（发布 B 那次的释放代从 4 写成 3）照样红。
fn judge_release_generations(
    scanned: &[ScannedCandidateRoot],
    newest_position: usize,
    judgements: &mut Judgements,
) {
    let released: Vec<&AllocationRecordView> = scanned[newest_position]
        .references
        .allocation_records
        .iter()
        .filter(|record| record.is_released)
        .collect();
    if released.is_empty() {
        judgements.not_applicable("I-3.9", "最新的根下面没有带已释放标志的分配记录");
        return;
    }
    for root in scanned {
        assert_eq!(
            root.references.depth,
            ReferenceScanDepth::EveryReferencedPlacement,
            "最新根那棵账里有已释放记录时，每条候选根都按「走到叶」扫过（`judge_release_generation_and_tree_table_birth` 定的深度）"
        );
    }
    if !scanned.iter().all(|root| root.references.is_complete) {
        judgements.not_applicable(
            "I-3.9",
            "有候选根走不完（单元读不出，或走到条目格式没有条款的形状）：引用集合数不全",
        );
        return;
    }
    let mut witnessed_records = 0u64;
    for record in released {
        let placement = (record.device, record.slot);
        let Some(last_referencing_txg) = scanned
            .iter()
            .filter(|root| root.references.placements.contains(&placement))
            .map(|root| root.checkpoint_txg)
            .max()
        else {
            // 候选集里一条根都没引用过它：见证它释放的那条根已经不在候选集里（F_生效 抬过释放代之后落点被回收，
            // 盘上那条记录仍写着已释放，D16（发布语义） 已定项 1 / D3（空间分配） 已定项 7）。这个镜像上找不到那条根，这一条判不了。
            continue;
        };
        witnessed_records += 1;
        let first_root_without_it = scanned
            .iter()
            .map(|root| root.checkpoint_txg)
            .filter(|txg| *txg > last_referencing_txg)
            .min();
        let in_the_witnessed_interval = record.generation > last_referencing_txg
            && first_root_without_it.is_some_and(|txg| record.generation <= txg);
        judgements.judge("I-3.9", in_the_witnessed_interval, || {
            match first_root_without_it {
                Some(txg) => format!(
                    "盘 {} 槽 {} 的分配记录释放代 {} 不在 ({last_referencing_txg}, {txg}] 里：还引用它的最新一条有效根是 txg {last_referencing_txg}，最早不再引用它的是 txg {txg}",
                    record.device, record.slot, record.generation
                ),
                None => format!(
                    "盘 {} 槽 {} 的分配记录带已释放标志（释放代 {}），而候选集里每一条有效根都还引用它",
                    record.device, record.slot, record.generation
                ),
            }
        });
    }
    if witnessed_records == 0 {
        judgements.not_applicable(
            "I-3.9",
            "带已释放标志的记录一条都没有候选根引用过：见证它们释放的根已不在候选集里",
        );
    }
}

/// I-9.14（树表条目的诞生 txg 跨根不变）：按树 ID 归并候选集里每条根的树表条目，诞生 txg 必须相同。
/// 只在这棵树的条目出现在**两个不同的树表单元**里时才判——几条根共用一个树表单元（暖机那几代都指着 mkfs 种的第 0 版）时，
/// 拿同一份字节比自己是恒真的判定，那正是 C374（释放代与树表诞生 txg 只有验收断言盯着） 点的那一类；这时报「不适用」，不报成立。
///
/// ⚠️ **「跨根」的射程只到同一条时间线**（C511（回退到无文件那一版之后诞生代怎么接）：2026-09-23 用户定案收窄）。
/// 收窄不在这个函数里，在**入参**上：`scanned` 是回退候选集——按最新那条根指着的实例表判仍然有效的那一组根
/// （D23（journal 的角色与格式） 已定项 14：(i, T) 有效 ⟺ 实例表里没有实例 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti），
/// 筛在 `check_pool_image` 里 `abandoned` 那一条上。有行 (i, Ti, Wi) 且 T > Ti 的根在被回退切掉的那条旧线上，
/// 一进不来，所以这里比的恒是同一条线上的根。
/// 这一条靠回退行被之后每一次发布照抄下去——再发第一个文件版本那一次也照抄现行那一版的实例表指针
/// （`transaction::publish_first_file`）。探针上把那一次换成照抄 mkfs 那一片实例表，回退行就丢了、旧线的根重回候选集，
/// I-9.14 与 I-9.10（对象出生代三处一致） 一起红，形态与 C511 登记的那三份红相同。
/// **为什么必须收窄**：回退到没有文件的一版之后，这个池只剩「再发一次第一个文件版本」这一条路，那一次重新建树，
/// 而树 ID 水位随根一起退回 ⇒ 新树重号拿到同一个树 ID、诞生 txg 必然与旧线记的不同。不收窄就要在这一格判红，
/// 而那是一条正当历史。**收窄两头各有用例与变异盯着**（`checker_known_bad_images.rs`，建在回退到 A 之后那份镜像上）：
/// 被切掉的那条线上记得不同不比——`birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_…`，
/// 把这一遍改成拿根环里每一条根比就红；跨过回退行、同一条线上记得不同照样红——
/// `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_…`，
/// 把这一遍改成只拿最新根那个实例的根比就红（C374 那份镜像只有一个实例、实例表一行都没有，分不出这一格）。
/// 候选集那一条本身另有变异（去掉 `abandoned` 那一条，`rolling_back_to_the_first_root_…` 在 I-3.1（已分配统计对得上） 上红）；
/// 没有回退行的那一格由 `each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite` 盯着。
fn judge_tree_table_birth_txg(scanned: &[ScannedCandidateRoot], judgements: &mut Judgements) {
    let mut by_tree: BTreeMap<u64, Vec<TreeTableEntrySighting>> = BTreeMap::new();
    for root in scanned {
        let Some(tree_table_placement) = root.references.tree_table_placement else {
            continue;
        };
        for (tree, birth_txg) in &root.references.tree_table_birth_txg {
            by_tree
                .entry(*tree)
                .or_default()
                .push(TreeTableEntrySighting {
                    root_checkpoint_txg: root.checkpoint_txg,
                    tree_table_placement,
                    birth_txg: *birth_txg,
                });
        }
    }
    let mut compared_trees = 0u64;
    for (tree, sightings) in &by_tree {
        let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings
            .iter()
            .map(|sighting| sighting.tree_table_placement)
            .collect();
        if distinct_tree_tables.len() < 2 {
            continue;
        }
        compared_trees += 1;
        let first_birth_txg = sightings[0].birth_txg;
        let birth_txg_is_the_same_across_roots = sightings
            .iter()
            .all(|sighting| sighting.birth_txg == first_birth_txg);
        judgements.judge("I-9.14", birth_txg_is_the_same_across_roots, || {
            let seen: Vec<String> = sightings
                .iter()
                .map(|sighting| {
                    format!(
                        "根 txg {} 记 {}",
                        sighting.root_checkpoint_txg, sighting.birth_txg
                    )
                })
                .collect();
            format!("树 {tree} 的树表条目诞生 txg 跨根不一：{}", seen.join("、"))
        });
    }
    if compared_trees == 0 {
        judgements.not_applicable(
            "I-9.14",
            "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
        );
    }
}

/// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
/// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（2026-09-18 用户定案随 C374 立条时定的口径）。
fn judge_release_generation_and_tree_table_birth(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    newest_index: usize,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    if candidate_indexes.len() < 2 {
        judgements.not_applicable(
            "I-3.9",
            "回退候选集里只有一条根：没有第二条根能见证「不再引用这个落点」",
        );
        judgements.not_applicable(
            "I-9.14",
            "回退候选集里只有一条根：树表条目没有第二条根的那一份可比",
        );
        return;
    }
    // 最新根那棵账定这一遍走多深：它一条已释放记录都没有时 I-3.9 判不了，别的根只要树表（I-9.14 那一半）。
    let newest = references_of_root(
        reader,
        &roots[newest_index].2.record_bytes,
        ReferenceScanDepth::EveryReferencedPlacement,
        cache,
    );
    let depth = if newest
        .allocation_records
        .iter()
        .any(|record| record.is_released)
    {
        ReferenceScanDepth::EveryReferencedPlacement
    } else {
        ReferenceScanDepth::TreeTableOnly
    };
    let mut scanned: Vec<ScannedCandidateRoot> = Vec::with_capacity(candidate_indexes.len());
    for index in candidate_indexes.iter().copied() {
        scanned.push(ScannedCandidateRoot {
            root_index: index,
            checkpoint_txg: roots[index].2.checkpoint_txg,
            references: if index == newest_index {
                newest.clone()
            } else {
                references_of_root(reader, &roots[index].2.record_bytes, depth, cache)
            },
        });
    }
    scanned.sort_unstable_by_key(|root| root.checkpoint_txg);
    let newest_position = scanned
        .iter()
        .position(|root| root.root_index == newest_index)
        .expect("最新根在回退候选集里（`check_pool_image` 无条件把它放进去）");
    judge_release_generations(&scanned, newest_position, judgements);
    judge_tree_table_birth_txg(&scanned, judgements);
}

/// I-5.4（分配记录罩住的槽互不相交）：候选集里每条有效根的分配记录树，同一块盘上任意两条记录（不论已分配、已释放、释放代是否已低于
/// 回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交（`.claude/kb/invariants.md` I-5.4 那一行）。I-5.1（物理范围不重叠） 管的是
/// 树里的引用，判不出记录之间的重叠：代码三方第二轮 Z1-d 的镜像上引用互不重叠，而两条分配记录罩住同一个槽，下一次可写挂载重建分配器时 panic。
/// 几条根指着同一个分配记录树节点时（照抄上一版的根）那个节点只判一次；每个 (节点, 盘) 判一格，那块盘上只有一条记录也算判过。
/// 走不到记录的根不判：树表或分配记录树读不出（由 I-2.1 / I-7.2 说话）、分配记录树有内部节点（内部条目格式没有条款，总审核 D8-D11 发现 16）。
fn judge_allocation_records_disjoint(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    let mut judged_allocation_nodes = 0u64;
    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
        if tree_table_pointer.all_zero {
            continue;
        }
        let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
        else {
            continue;
        };
        for entry in &tree_table.entries {
            if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION
            {
                continue;
            }
            let allocation_root = parse_node_pointer(&entry[14..100]);
            if allocation_root.all_zero {
                continue;
            }
            let first_location = &allocation_root.locations[0];
            if !seen_allocation_nodes.insert((
                first_location.device,
                first_location.slot,
                first_location.checksum,
            )) {
                continue;
            }
            let Some(node) = read_index_node_without_judging(reader, &allocation_root, cache)
            else {
                continue;
            };
            if node.level > 0 || node.entry_width < allocation_record_bytes() {
                continue;
            }
            judge_allocation_record_ranges_of_one_node(&node, root.checkpoint_txg, judgements);
            judged_allocation_nodes += 1;
        }
    }
    if judged_allocation_nodes == 0 {
        judgements.not_applicable(
            "I-5.4",
            "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、有内部节点",
        );
    }
}

/// 一个起点槽上的单元头能不能用、能用时头里的诞生代号（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的偏移表：
/// 码 1 诞生代号 75、明文头末尾 105；码 2 诞生代号 52 + 2k、明文头末尾 86 + 2k；码 3 诞生代号 73、明文头末尾 107；
/// 头校验和三类都在 10）。「能用」= magic 对、类标签是 1 / 2 / 3、头校验和过——头读不出、类标签不认、头校验和不过的
/// 这一格没有对象（I-3.10 那一行射程 ③：读不出本身由 I-1.1 与 I-2.1 管）。
fn birth_txg_in_a_usable_unit_header(header: &[u8]) -> Option<u64> {
    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
        return None;
    }
    let (plain_header_end, birth_txg_offset) = match header[6] {
        1 => (105, 75),
        2 => {
            let key_span = 2 * usize::from(header[51]);
            (86 + key_span, 52 + key_span)
        }
        3 => (107, 73),
        _outside_the_three_unit_classes => return None,
    };
    checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET)
        .then(|| read_u64(header, birth_txg_offset))
}

/// 最新根之后、下一次挂载的恢复会先施加的那一版的树表指针（改法 E，`research/prompts/m2-wave3-code-r1-main-verification.md`
/// 第三节 Y4 与第四节第 2 条，被攻过零轮）：环里一条记录与最新根**同实例**、checkpoint_txg = 最新根的 txg + 1、带提交标记，
/// 根环里没有 (那个实例, 那个 txg) 的根——它那次发布的记录已持久、根槽没落盘，恢复要由记录重建那一版（D23（journal 的角色与格式）
/// 已定项 15）。树表指针取那条记录新根段里的（D23（journal 的角色与格式） 已定项 4 的字段表）；一版切成几条记录、两块盘各一份时
/// 它们带的是同一版的指针，取先读到的那一条。
///
/// **只接这一步**：恢复会连着施加几条记录时，第二条及以后那几版的分配记录这里读不到；不复刻恢复的整条前缀规则
/// （断号即止、点名单元验证、发布边界），这一版恢复会不会真施加这里不判——读它只多判 I-3.10 一格，
/// 它的分配记录与它罩住的单元是同一次发布写出来的，不论施加与否两个数都同源。
fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    roots: &[(u64, u64, crate::RootView)],
    newest_index: usize,
) -> Option<PointerView> {
    let newest_root = &roots[newest_index].2;
    let next_txg = newest_root.checkpoint_txg.checked_add(1)?;
    let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
        root.instance == newest_root.instance && root.checkpoint_txg == next_txg
    });
    if root_slot_is_readable {
        return None;
    }
    records_by_device
        .iter()
        .flat_map(|(_, records)| records.values())
        .find(|record| {
            record.commit_marker == CommitMarker::Present
                && record.instance == newest_root.instance
                && record.checkpoint_txg == next_txg
        })
        .map(|record| {
            parse_node_pointer(&record.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER])
        })
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）读哪几片分配记录：候选集里每条根的分配记录树
/// （树表里种类 3 那一条指着的根），树表 0 条那一版的分配记录树（根记录直接持有，C512（树表 0 条的一版上被换下的单元记在哪）），
/// 由记录施加出来的那几版的分配记录树（它们的树表在记录的新根段里），以及下一次挂载会先施加的那一版的分配记录树
/// （[`tree_table_pointer_of_the_version_the_next_mount_applies_first`]）。只读不判：判定归 I-2.1 / I-1.1 那几条的走读。
fn allocation_record_node_pointers_of_the_candidate_versions(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    versions_applied_only_by_records: &[VersionAppliedOnlyByRecords],
    tree_table_pointer_of_the_version_the_next_mount_applies_first: Option<PointerView>,
    cache: &mut IndexNodeCache,
) -> Vec<PointerView> {
    let mut tree_table_pointers: Vec<PointerView> = Vec::new();
    let mut allocation_node_pointers: Vec<PointerView> = Vec::new();
    for index in candidate_indexes.iter().copied() {
        let record = &roots[index].2.record_bytes;
        tree_table_pointers.push(parse_node_pointer(&record[36..122]));
        allocation_node_pointers.push(parse_node_pointer(&record[342..428]));
    }
    for version in versions_applied_only_by_records {
        let (tree_table_pointer, _) = version.tree_table_and_mapping_root_pointers();
        tree_table_pointers.push(parse_node_pointer(tree_table_pointer));
    }
    tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);
    for tree_table_pointer in &tree_table_pointers {
        if tree_table_pointer.all_zero {
            continue;
        }
        let Some(tree_table) = read_index_node_without_judging(reader, tree_table_pointer, cache)
        else {
            continue;
        };
        for entry in &tree_table.entries {
            if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION
            {
                continue;
            }
            allocation_node_pointers.push(parse_node_pointer(&entry[14..100]));
        }
    }
    allocation_node_pointers.retain(|pointer| !pointer.all_zero);
    allocation_node_pointers
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）：任一**未带已释放标志**的分配记录，它的分配代等于它罩住的那个单元头里的
/// 诞生代号——记录与单元是同一次发布写出来的，两个数同源（`.claude/kb/invariants.md` I-3.10 那一行）。射程照那一行：
/// ① 只管未释放的记录（已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里））；
/// ② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头——记录的 key 就是起点槽，按它读；
/// ③ 那个槽上读不出可用的单元头时这一格没有对象，不判（[`birth_txg_in_a_usable_unit_header`]）。
/// 它拦的是：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着。
/// 几条根指着同一个分配记录树节点时（照抄上一版的根）那个节点只读一次；同一条记录（盘、槽、代）出现在几片里只判一次。
/// 一格都没判到时整条报不适用并带理由，不报成立。
fn judge_allocation_generations_against_unit_births(
    reader: &dyn ImageReader,
    allocation_node_pointers: &[PointerView],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    let mut examined_records: BTreeSet<(u32, u64, u64)> = BTreeSet::new();
    let mut judged_records = 0u64;
    for pointer in allocation_node_pointers {
        let first_location = &pointer.locations[0];
        if !seen_allocation_nodes.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
        )) {
            continue;
        }
        let Some(node) = read_index_node_without_judging(reader, pointer, cache) else {
            continue;
        };
        if node.level > 0 || node.entry_width < allocation_record_bytes() {
            continue;
        }
        for entry in &node.entries {
            let record = parse_allocation_record(entry);
            if record.is_released
                || !examined_records.insert((record.device, record.slot, record.generation))
            {
                continue;
            }
            let Some(birth_txg) = reader
                .read(
                    record.device,
                    record.slot * SLOT_BYTES,
                    UNIT_HEADER_SCAN_BYTES,
                )
                .as_deref()
                .and_then(birth_txg_in_a_usable_unit_header)
            else {
                continue;
            };
            judged_records += 1;
            judgements.judge("I-3.10", record.generation == birth_txg, || {
                format!(
                    "盘 {} 槽 {} 的分配记录（未释放、跨 {} 槽）分配代 {}，它罩住的单元头里的诞生代号是 {birth_txg}",
                    record.device, record.slot, record.span_slots, record.generation
                )
            });
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            "I-3.10",
            if examined_records.is_empty() {
                "候选集里没有一片分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、有内部节点）"
            } else {
                "未释放的分配记录罩住的起点槽上一个可用的单元头都读不出：读不出本身归 I-1.1 与 I-2.1"
            },
        );
    }
}

/// 一个分配记录树叶上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
fn judge_allocation_record_ranges_of_one_node(
    node: &crate::IndexNodeView,
    root_checkpoint_txg: u64,
    judgements: &mut Judgements,
) {
    let mut ranges_per_device: BTreeMap<u32, Vec<(u64, u64)>> = BTreeMap::new();
    for entry in &node.entries {
        let record = parse_allocation_record(entry);
        ranges_per_device
            .entry(record.device)
            .or_default()
            .push((record.slot, record.span_slots));
    }
    for (device, mut ranges) in ranges_per_device {
        ranges.sort_unstable();
        match ranges
            .windows(2)
            .find(|pair| pair[0].0 + pair[0].1 > pair[1].0)
        {
            Some(pair) => {
                let ((first_slot, first_span), (second_slot, second_span)) = (pair[0], pair[1]);
                judgements.judge("I-5.4", false, || {
                    format!(
                        "txg {root_checkpoint_txg} 的根那棵分配记录树、盘 {device}：槽 {first_slot} 跨 {first_span} 的记录与槽 {second_slot} 跨 {second_span} 的记录罩住同一个槽"
                    )
                });
            }
            None => judgements.judge("I-5.4", true, String::new),
        }
    }
}

/// 一条根的分配记录树里有几条记录：按 checker 自己的字段表读这条根的树表，找种类 3（分配记录）的条目，数它根节点叶里的条目
/// （第一版分配记录树只有一个节点；一条记录记一个单元，D3（空间分配） 已定项 7）。只读不判——给理想模型的对拍数分配记录墙的真条数
/// （增补 3 第 2 件代码三方第一轮判决第三节第 1 条：从镜像上现数，不用分配器的状态）。
/// 树表里没有分配记录树的条目、或那一条的根指针全零时是 0 条：树表 0 条的一版上没有分配记录树，mkfs 写的两个单元的记录要到
/// 第一个文件版本才落盘。树表指针全零、树表或分配记录树的节点读不出或条目比字段表窄、分配记录树有内部节点（内部条目格式没有条款，
/// 总审核 D8-D11 发现 16）时数不出，交回 None。
#[must_use]
pub fn allocation_record_count_under_root(
    reader: &dyn ImageReader,
    root: &crate::RootView,
) -> Option<usize> {
    let mut cache = IndexNodeCache::new();
    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return None;
    }
    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, &mut cache)?;
    let mut records = 0;
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() {
            return None;
        }
        if read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let node = read_index_node_without_judging(reader, &allocation_root, &mut cache)?;
        if node.level > 0 || node.entry_width < allocation_record_bytes() {
            return None;
        }
        records += node.entries.len();
    }
    Some(records)
}

/// 单元头校验和字段的偏移（共同前缀，三类同一处）。
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
/// 写序那 10 字节里事务号低 48 位从第 4 字节起（D18（块里携带什么信息） 已定项 7：实例代号 4 + 事务号 6）。
const WRITE_ORDER_TRANSACTION_OFFSET: usize = 4;

/// I-1.8（归并后版本全序） 罩得到的两类单元。码 2 不进这一条（`.claude/kb/invariants.md` I-1.8 逐字「码 2 不进这条」：
/// 它的写序已收窄成只有实例代号，多版在条目级合并、不在容器级择版本）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContentUnitClass {
    /// 码 1 数据单元。
    Data,
    /// 码 3 打包记录单元。
    PackedRecord,
}

/// 一类内容单元的头里，这一条要读的那几段。码 1 按 D18（块里携带什么信息） 已定项 7 的字段表（五元组 42、诞生代号 75、
/// fsid 83、写序 91、载荷 CRC 101、明文头末尾 105）；码 3 按已定项 11 的类身份段表（标签 42、出生树 43、类型 51、容器号 53、
/// 容器出生代 61、记录数 69、记录宽 71、诞生代号 73、fsid 81、载荷校验和 89、写序 93、出生序号 103、明文头末尾 107）。
struct ContentUnitHeaderOffsets {
    plain_header_end: usize,
    payload_checksum: usize,
    filesystem_identifier: usize,
    birth_txg: usize,
    write_order: usize,
    /// 类身份段里与版本无关的那一段：码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代）。
    object_key: std::ops::Range<usize>,
    /// 归并键的两段：「类身份段全部字段含写序」**去掉载荷校验和那一段**。
    /// 载荷校验和不进归并键——它正是「同一组的成员载荷相同」要比的那个量，进了键这一条就恒真。
    merge_key_before_payload_checksum: std::ops::Range<usize>,
    merge_key_after_payload_checksum: std::ops::Range<usize>,
}

/// 已发布谓词的输入，按单元类的两种取法分成两个成员（`.claude/kb/invariants.md` I-1.2（块头写序已发布） 那一行逐字：
/// 「有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub」）：码 1 那一支带事务号 n，
/// 码 2 / 码 3 那一支不带——它们的取法不读 n，分成两个成员就不必给一个用不上的 n 编个值。
/// 码 2 与码 3 在谓词上是同一支，而它们在 I-1.8（归并后版本全序） 上不是（码 2 不进 I-1.8），
/// 所以这个枚举与 [`ContentUnitClass`] 是两件事，不合并。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteOrderUnderThePublishedPredicate {
    /// 码 1：诞生代号 b 与写序 (i, n)。
    DataUnit {
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    },
    /// 码 2 / 码 3：诞生代号 b 与写序的实例代号 i。
    IndexOrPackedRecordUnit { birth_txg: u64, instance: u32 },
}

impl WriteOrderUnderThePublishedPredicate {
    fn instance(self) -> u32 {
        match self {
            WriteOrderUnderThePublishedPredicate::DataUnit { instance, .. }
            | WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit { instance, .. } => {
                instance
            }
        }
    }

    fn birth_txg(self) -> u64 {
        match self {
            WriteOrderUnderThePublishedPredicate::DataUnit { birth_txg, .. }
            | WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit { birth_txg, .. } => {
                birth_txg
            }
        }
    }
}

impl ContentUnitClass {
    /// 扫描方向读到的头解成谓词的输入：码 1 带事务号，码 3 的取法不读它。
    fn write_order_under_the_published_predicate(
        self,
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    ) -> WriteOrderUnderThePublishedPredicate {
        match self {
            ContentUnitClass::Data => WriteOrderUnderThePublishedPredicate::DataUnit {
                birth_txg,
                instance,
                transaction,
            },
            ContentUnitClass::PackedRecord => {
                WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                    birth_txg,
                    instance,
                }
            }
        }
    }

    fn of(unit_class_tag: u8) -> Option<Self> {
        match unit_class_tag {
            1 => Some(ContentUnitClass::Data),
            3 => Some(ContentUnitClass::PackedRecord),
            _outside_this_invariants_reach => None,
        }
    }

    fn header_offsets(self) -> ContentUnitHeaderOffsets {
        match self {
            ContentUnitClass::Data => ContentUnitHeaderOffsets {
                plain_header_end: 105,
                payload_checksum: 101,
                filesystem_identifier: 83,
                birth_txg: 75,
                write_order: 91,
                object_key: 42..75,
                merge_key_before_payload_checksum: 42..101,
                merge_key_after_payload_checksum: 105..105,
            },
            ContentUnitClass::PackedRecord => ContentUnitHeaderOffsets {
                plain_header_end: 107,
                payload_checksum: 89,
                filesystem_identifier: 81,
                birth_txg: 73,
                write_order: 93,
                object_key: 42..69,
                merge_key_before_payload_checksum: 42..89,
                merge_key_after_payload_checksum: 93..107,
            },
        }
    }

    fn name(self) -> &'static str {
        match self {
            ContentUnitClass::Data => "码 1 数据",
            ContentUnitClass::PackedRecord => "码 3 打包记录",
        }
    }
}

/// 已发布谓词（I-1.2（块头写序已发布） 的谓词，C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 /
/// D18（块里携带什么信息） 已定项 1）的输入：挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg，
/// 加上它指着的那一版实例表里的行。I-1.8（归并后版本全序） 的射程逐字是「可读**已发布**单元」，扫描方向要拿这个谓词过滤。
struct PublishedPredicate<'rows> {
    mount_root_instance: u32,
    mount_root_checkpoint_txg: u64,
    instance_table_rows: &'rows [InstanceTableRow],
}

impl PublishedPredicate<'_> {
    /// 写序 (i, n) 与诞生代号 b 判没判成已发布。
    ///
    /// **i > 挂载根实例这一支在这里判成「不是已发布」，不判损坏**：I-1.2（块头写序已发布） 那一行逐字是
    /// 「i > 挂载根实例 ⇒ 判损坏」，而那句话站在恢复刚发布完新根的那一刻——那时不存在比挂载根更高的实例。
    /// 池级 checker 对着的是任意一份崩溃镜像：上一次挂载取了号、写出单元、一个根都没发布就崩了，盘上就有
    /// i > 最新根实例的单元，而它是合法的在飞垃圾。所以扫描方向这一支判不了（射程见 `.claude/kb/invariants.md`
    /// I-1.2 的状态列），这里只把它挡在「已发布」之外：I-1.8（归并后版本全序） 的射程要的正是这个。
    /// 被引用的单元走的是另一条路——遍历方向由 I-4.2（无被引用未提交块） 判，见
    /// [`Walk::judge_birth_identity_of_a_referenced_unit`]。
    fn holds_for(&self, write_order: WriteOrderUnderThePublishedPredicate) -> bool {
        match write_order.instance().cmp(&self.mount_root_instance) {
            Ordering::Greater => false,
            Ordering::Equal => write_order.birth_txg() <= self.mount_root_checkpoint_txg,
            Ordering::Less => match self
                .instance_table_rows
                .iter()
                .find(|row| row.instance == write_order.instance())
            {
                None => true,
                Some(row) => match write_order {
                    WriteOrderUnderThePublishedPredicate::DataUnit {
                        birth_txg,
                        transaction,
                        ..
                    } => {
                        birth_txg <= row.published_checkpoint_txg
                            || transaction <= row.applied_transaction_high_water_mark
                    }
                    WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                        birth_txg,
                        ..
                    } => birth_txg <= row.published_checkpoint_txg,
                },
            },
        }
    }
}

/// I-1.8 ② 的那个全序键。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TotalOrderKey {
    /// 码 3：(诞生代号, 实例代号, 事务号) 三元，诞生代号打头（`.claude/kb/invariants.md` I-1.8 那一行写死）。
    OfPackedRecordUnit(Vec<u64>),
    /// 码 1：**两处条款给的是两个键，没定哪一个作数，这一半停在这里**——I-1.8 那一行逐字「码 1 的键是写序」，
    /// 而 D18（块里携带什么信息） 已定项 1 的射程逐字「同 key 已发布版本的定序靠诞生代号 + 每事务每 key 一种净效果就够，
    /// 写序只在同 txg 跨事务改同 key 时顺带定序」。按前者判，同一实例的两版在盘上能撞出写序逐字节相同——
    /// I-1.8（归并后版本全序） 那一行自己的括注逐字写着「同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同」，
    /// 而码 3 用「诞生代号打头」解决了同一个问题、码 1 没有；按后者判它们由诞生代号分得开。
    /// ⚠️ 不要拿 `second_transaction_step_five_reuse.rs` 那条脚本当证据：2026-09-21 之前它上面确实有一对，
    /// 那是事务号重号（空发布把按实例的计数拉回去）造出来的，重号修掉之后那条脚本上八个写序互不相同。
    /// 哪一个作数要定案，定之前码 1 不进 ② 这一半；① 那一半（同一组的成员载荷相同）码 1 照判。
    UndecidedForDataUnit,
}

/// 扫描方向读到的一个码 1 / 码 3 已发布单元（「可读」的后半截由 `payload_checksum_holds_on_disk` 现算）。
struct ScannedContentUnit {
    device: u32,
    slot: u64,
    unit_class: ContentUnitClass,
    /// 「类身份段全部字段含写序」去掉载荷校验和那一段：同一份内容的两个副本在这上面逐字节相同。
    merge_key: Vec<u8>,
    /// 类身份段里与版本无关的那一段：I-1.8 里「同 key」的那个 key。
    object_key: Vec<u8>,
    total_order_key: TotalOrderKey,
    payload_checksum: u32,
}

impl ScannedContentUnit {
    fn describe(&self) -> String {
        format!(
            "盘 {} 槽 {} 的{}单元",
            self.device,
            self.slot,
            self.unit_class.name()
        )
    }
}

/// 一个候选槽上的头解成扫描项：magic、类标签在这一条的射程内、头校验和过、fsid 与本池相同、按已发布谓词判成已发布。
/// 任一步不成立就不是这一条判的对象（各自另有不变量说话：I-1.6 / I-2.4 / I-1.4 / I-1.2）。
fn content_unit_of_header(
    header: &[u8],
    device: u32,
    slot: u64,
    filesystem_identifier_low: u64,
    published: &PublishedPredicate<'_>,
) -> Option<ScannedContentUnit> {
    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
        return None;
    }
    let unit_class = ContentUnitClass::of(header[6])?;
    let offsets = unit_class.header_offsets();
    if !checksum_field_holds(
        header,
        offsets.plain_header_end,
        UNIT_HEADER_CHECKSUM_OFFSET,
    ) || read_u64(header, offsets.filesystem_identifier) != filesystem_identifier_low
    {
        return None;
    }
    let birth_txg = read_u64(header, offsets.birth_txg);
    let instance = read_u32(header, offsets.write_order);
    let transaction =
        read_six_byte_unsigned(header, offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET);
    if !published.holds_for(unit_class.write_order_under_the_published_predicate(
        birth_txg,
        instance,
        transaction,
    )) {
        return None;
    }
    let total_order_key = match unit_class {
        ContentUnitClass::Data => TotalOrderKey::UndecidedForDataUnit,
        ContentUnitClass::PackedRecord => {
            TotalOrderKey::OfPackedRecordUnit(vec![birth_txg, u64::from(instance), transaction])
        }
    };
    Some(ScannedContentUnit {
        device,
        slot,
        unit_class,
        merge_key: [
            &header[offsets.merge_key_before_payload_checksum],
            &header[offsets.merge_key_after_payload_checksum],
        ]
        .concat(),
        object_key: header[offsets.object_key].to_vec(),
        total_order_key,
        payload_checksum: read_u32(header, offsets.payload_checksum),
    })
}

/// 扫描方向：候选槽上的码 1 / 码 3 已发布单元。32768 的单元占两个槽，第二个槽上没有 magic，扫描自然跳过它。
fn scanned_content_units(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    filesystem_identifier_low: u64,
    published: &PublishedPredicate<'_>,
) -> Vec<ScannedContentUnit> {
    let mut units = Vec::new();
    for device in reader.devices() {
        let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
            (geometry.unit_area_start_slot..end).collect()
        });
        for slot in slots {
            let Some(header) = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)
            else {
                continue;
            };
            if let Some(unit) =
                content_unit_of_header(&header, device, slot, filesystem_identifier_low, published)
            {
                units.push(unit);
            }
        }
    }
    units
}

/// 这一份在盘上读得出来、而且它的载荷与它自己头里那个载荷校验和对得上——「可读」的后半截。
/// 头校验和过而载荷对不上的那一份不可读（撕裂、被盖），不进 I-1.8 的归并组。
fn payload_checksum_holds_on_disk(reader: &dyn ImageReader, unit: &ScannedContentUnit) -> bool {
    let offsets = unit.unit_class.header_offsets();
    let Some(bytes) = reader.read(unit.device, unit.slot * SLOT_BYTES, data_unit_bytes()) else {
        return false;
    };
    bytes.len() > offsets.plain_header_end
        && read_u32(&bytes, offsets.payload_checksum)
            == crc32_castagnoli_table(&bytes[offsets.plain_header_end..])
}

/// I-1.8（归并后版本全序）：码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，
/// ① **同一组的成员载荷相同**（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择；
/// ② **同 key 的各组两两全序键不等**——码 3 的键是 (诞生代号, 实例代号, 事务号) 三元。
///    **② 这一半今天只判码 3**：码 1 的键在两处条款里不是同一个，见 `TotalOrderKey::UndecidedForDataUnit`。
/// 射程 w = 2（每次写两盘各一份，D2（RAID 条带策略） 已定项 6）：正常镜像上一组就是同一个单元的两个副本。
/// **「可读」的后半截只在要判红的那一步才现算**：一组里载荷校验和字段全相同时 ① 已经成立、不必把载荷读回来；
/// 字段不同才逐份验载荷——整单元 CRC 是这里最贵的一步，而层 0 每个崩溃状态都要跑这一遍。
/// 交回判了几次：一次都没判到时调用方报「不适用」，不报成立（`test-discipline.md`「检查本身也可能是错的」）。
fn judge_merged_version_total_order(
    reader: &dyn ImageReader,
    units: &[ScannedContentUnit],
    judgements: &mut Judgements,
) -> u64 {
    let mut groups: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
    for unit in units {
        groups.entry(&unit.merge_key).or_default().push(unit);
    }
    let mut judged = 0u64;
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let first_payload_checksum = members[0].payload_checksum;
        let every_member_carries_the_same_payload_checksum = members
            .iter()
            .all(|member| member.payload_checksum == first_payload_checksum);
        // 校验和字段全相同时这一条已经成立，`||` 短路掉后半截：那半截要把每一份载荷整个读回来算 CRC。
        let payloads_are_the_same = every_member_carries_the_same_payload_checksum || {
            let readable_payload_checksums: BTreeSet<u32> = members
                .iter()
                .filter(|member| payload_checksum_holds_on_disk(reader, member))
                .map(|member| member.payload_checksum)
                .collect();
            readable_payload_checksums.len() <= 1
        };
        judged += 1;
        judgements.judge("I-1.8", payloads_are_the_same, || {
            let members_and_their_payloads: Vec<String> = members
                .iter()
                .map(|member| {
                    format!(
                        "{} 载荷校验和 {:#010x}",
                        member.describe(),
                        member.payload_checksum
                    )
                })
                .collect();
            format!(
                "归并成一组的成员载荷不同（{}）：同一组的成员载荷必须相同，不同即判损坏、不择",
                members_and_their_payloads.join("、")
            )
        });
    }
    // 一组里的成员共享归并键，对象 key 与全序键都是归并键的子段 ⇒ 取第一个成员当这一组的代表。
    // 码 1 的全序键没定（`TotalOrderKey::UndecidedForDataUnit`），② 这一半只判码 3；对象 key 首字节是类标签，
    // 一个 key 上的各组必然同类，不会把两类混进同一个桶。
    let mut groups_by_object_key: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
    for members in groups.values() {
        if members[0].total_order_key == TotalOrderKey::UndecidedForDataUnit {
            continue;
        }
        groups_by_object_key
            .entry(&members[0].object_key)
            .or_default()
            .push(members[0]);
    }
    for representatives in groups_by_object_key.values() {
        if representatives.len() < 2 {
            continue;
        }
        let distinct_total_order_keys: BTreeSet<&TotalOrderKey> = representatives
            .iter()
            .map(|representative| &representative.total_order_key)
            .collect();
        judged += 1;
        judgements.judge(
            "I-1.8",
            distinct_total_order_keys.len() == representatives.len(),
            || {
                let groups_on_this_key: Vec<String> = representatives
                    .iter()
                    .map(|representative| {
                        format!(
                            "{} 全序键 {:?}",
                            representative.describe(),
                            representative.total_order_key
                        )
                    })
                    .collect();
                format!(
                    "同一个 key 上归并出 {} 组，两两全序键不等这一条不成立（{}）",
                    representatives.len(),
                    groups_on_this_key.join("、")
                )
            },
        );
    }
    judged
}

/// mkfs 把第 0 代种进根环（D22（单元原子性怎么合成） 已定项 8）：I-7.3（环健康性） 的例外那一支按这个代号判。
const GENESIS_CHECKPOINT_TXG: u64 = 0;

/// I-7.3（环健康性）：S（根槽里自证校验和通过的记录集合）中除代号最大者外，至少还存在一条更早代号的记录，
/// 不存在即判红——某次提交把上一代直接覆盖了，轮换逻辑已失效。**例外只有一个**：S 全部是第 0 代时判绿
/// （`.claude/kb/invariants.md` I-7.3 那一行的例外原样照搬；「崩溃后回退到这个态的镜像同样判绿」落在同一支上——
/// 判据只读 S 里的代号，不读这个态是怎么来的）。
/// 调用点在 I-7.1（根槽有效集非空） 之后：S 空时这一条报「不适用」，没有「代号最大者」可谈。
fn judge_root_ring_health(roots: &[(u64, u64, crate::RootView)], judgements: &mut Judgements) {
    let newest_checkpoint_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .expect("S 非空：`check_pool_image` 判过 I-7.1 之后 S 空就返回了，走不到这里");
    let earlier_generation_exists = roots
        .iter()
        .any(|(_, _, root)| root.checkpoint_txg < newest_checkpoint_txg);
    let every_root_is_the_genesis_generation = roots
        .iter()
        .all(|(_, _, root)| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG);
    judgements.judge(
        "I-7.3",
        earlier_generation_exists || every_root_is_the_genesis_generation,
        || {
            let generations: Vec<u64> = roots
                .iter()
                .map(|(_, _, root)| root.checkpoint_txg)
                .collect();
            format!(
                "根环里自证过的根代号 {generations:?}：最大的是 {newest_checkpoint_txg}，除它之外一条更早代号的记录都没有，而它们又不全是第 0 代 ⇒ 上一代被直接覆盖了，下一次撕裂无路可退"
            )
        },
    );
}

/// journal 记录的 jsn 计数器从 1 起、全池接着走（D23（journal 的角色与格式） 已定项 9 / 已定项 19 注 3：
/// 新实例从前缀末 + 1 接着写、不归零）。
const FIRST_JOURNAL_COUNTER: u64 = 1;

/// 记录头里提交标记那 1 字节（D23（journal 的角色与格式） 已定项 7，事务号之后那 1 字节）解出来的样子：
/// 0 不带、1 带，别的值是盘上读到的未知编码——做成显式成员，不并进「不带」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommitMarker {
    Absent,
    Present,
    Unrecognized(u8),
}

impl CommitMarker {
    const fn of(byte: u8) -> Self {
        match byte {
            0 => Self::Absent,
            1 => Self::Present,
            unrecognized => Self::Unrecognized(unrecognized),
        }
    }
}

/// 扫描方向读到的一条 journal 记录：它的实例代号、事务号、落在哪个环槽、反向链字段，以及「下一条记录的反向链该等于的值」。
struct ScannedJournalRecord {
    instance: u32,
    /// 记录头里的 checkpoint_txg：这条记录施加出来的是哪一版（「由记录施加出来、根槽从没落盘的那一版」按它认）。
    checkpoint_txg: u64,
    /// 记录头里的新根段 188 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8，D23（journal 的角色与格式） 已定项 4）：
    /// 这条记录施加出来的那一版从哪个树表、哪个映射根走下去。
    new_root_segment: Vec<u8>,
    /// 记录头里的事务号（D23（journal 的角色与格式） 已定项 7）：I-8.7（实例内事务号不重号） 判的就是它。
    transaction: u64,
    /// 记录头里的提交标记：I-8.8（前缀里的事务不被切开） 判的就是它。
    commit_marker: CommitMarker,
    slot: u64,
    back_chain: u32,
    /// CRC32C(这一条的 311 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
    chain_value_the_next_record_must_carry: u32,
}

/// 一块盘的 journal 环里自证过的记录，按 jsn 计数器索引。计数器与环槽一一对应（记录 n 落 `(计数器 − 1) mod 槽数 × 4096`，
/// D23（journal 的角色与格式） 已定项 18）⇒ 同一块盘上两条自证过的记录不会撞同一个计数器。
fn scanned_journal_records_of_device(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    device: u32,
    filesystem_identifier_low: u64,
) -> BTreeMap<u64, ScannedJournalRecord> {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let slots = reader
        .candidate_journal_slots(device)
        .unwrap_or_else(|| (0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).collect());
    let mut records = BTreeMap::new();
    for slot in slots {
        let offset = geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            continue;
        };
        let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) else {
            continue;
        };
        records.insert(
            record.counter,
            ScannedJournalRecord {
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                new_root_segment: record.new_root_segment,
                transaction: record.transaction,
                commit_marker: CommitMarker::of(record.commit_marker_byte),
                slot,
                back_chain: record.back_chain,
                chain_value_the_next_record_must_carry: back_chain_of_record_header(&bytes),
            },
        );
    }
    records
}

/// 逐盘扫一遍 journal 环，把自证过的记录按盘收齐：判 journal 的三条不变量（I-8.6（反向链算法）、
/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开））读的是同一批记录，扫一次三条一起判，不各扫一遍。
fn scanned_journal_records_by_device(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    filesystem_identifier_low: u64,
) -> Vec<(u32, BTreeMap<u64, ScannedJournalRecord>)> {
    reader
        .devices()
        .into_iter()
        .map(|device| {
            (
                device,
                scanned_journal_records_of_device(
                    reader,
                    geometry,
                    device,
                    filesystem_identifier_low,
                ),
            )
        })
        .collect()
}

/// I-8.6（反向链算法）：任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 311 字节头，`header_csum` 那 32 字节
/// 按零参与)；**本实例写出的第一条记录反向链恒 0**。逐盘判：环两盘互为镜像，一块盘上的链坏了另一块不该跟着坏。
/// 「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条，三种情形穷举：
/// ① 计数器 == 1 ⇒ 全池第一条，必是本实例第一条 ⇒ 反向链恒 0；
/// ② 计数器 − 1 那一条在同一块盘上自证过、实例代号相同 ⇒ 反向链 = 它的头算出来的链值；
/// ③ 计数器 − 1 那一条自证过而实例代号不同 ⇒ 这一条是本实例写出的第一条（一个实例的计数器连续、接着上一个实例走，
///    D23（journal 的角色与格式） 已定项 19 注 3）⇒ 反向链恒 0。**跨实例边界的两条记录之间不比链值**——I-8.6 明写不判那一格。
/// 计数器 − 1 那一条读不出、自证不过、或环转过一圈之后那一格坐着上一圈的记录（计数器对不上）⇒ 本实例内逻辑前一条不在盘上，
/// 这一条判不了，跳过。
fn judge_journal_back_chain(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut judged_records = 0u64;
    for (device, records) in records_by_device {
        for (counter, record) in records {
            let expected_back_chain = if *counter == FIRST_JOURNAL_COUNTER {
                Some(0)
            } else {
                match records.get(&(counter - 1)) {
                    Some(previous) if previous.instance == record.instance => {
                        Some(previous.chain_value_the_next_record_must_carry)
                    }
                    Some(_record_of_another_instance) => Some(0),
                    None => None,
                }
            };
            let Some(expected_back_chain) = expected_back_chain else {
                continue;
            };
            judged_records += 1;
            judgements.judge("I-8.6", record.back_chain == expected_back_chain, || {
                format!(
                    "盘 {device} journal 环槽 {} 的记录（实例 {}、计数器 {counter}）反向链 {:#010x}，本实例内逻辑前一条的头算出来的是 {expected_back_chain:#010x}",
                    record.slot, record.instance, record.back_chain
                )
            });
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            "I-8.6",
            "环里没有一条自证过的记录找得到本实例内逻辑前一条：链判不了",
        );
    }
}

/// 不承载事务的空发布记录在记录头上写的事务号（D23（journal 的角色与格式） 已定项 19 ①）。
const TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH: u64 = 0;

/// I-8.7（实例内事务号不重号）：**同一实例**在环里写出的记录按 jsn 排下来，一个事务号只占一段。两条判据，
/// 红了在违例说明里标出是哪一条（「判据①」「判据②」）：
/// ① 按计数器相邻的两条非 0 记录，事务号**不减**（相等或严格递增：D23（journal 的角色与格式） 已定项 7
///    「一个事务可以跨多条记录」，同一事务的记录共享事务号）；
/// ② 一个事务号的记录在计数器上**连续**：被别的记录（事务号 0 的空发布也算）隔开之后不再出现。
/// 这是已定项 7「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」的投影：
/// 同一实例里号不许重（一个号不许给两个隔开的事务用）、也不许往回走。
/// 逐盘判：环两盘互为镜像，一块盘上的记录坏了另一块不该跟着坏（与 I-8.6（反向链算法） 同一个口径）。
///
/// **射程**（照判据原样，多判一格少判一格都是错）：
/// ① **跨实例不判**：事务号各实例各算各的，两个实例各有一个 1 不是违例；这一条只比同一个实例代号的记录。
/// ② **事务号 0 不进 ① 的序列**：它是不承载事务的空发布记录（已定项 19 ①），夹在同一实例两个**不同**的非 0 号中间
///    （抬回退下界那两次空发布就是这个形态）照样合法；夹在两条**同号**记录中间就是把那个号隔开了，② 红
///    （一次发布的记录不会被另一次空发布插进来）。
/// ③ 两条同号记录之间只有读不出的槽、或别的实例的记录 ⇒ 看不到本实例在那几个计数器上写过什么，② 不判红。
/// ④ 同一块盘上同一实例非 0 的记录不足两条 ⇒ 没有一对号可比，整条报「不适用」并带理由。
///
/// 判的一格是「同一块盘、同一实例、按计数器排下来相邻的两条非 0 记录」：号相等时看它们之间有没有本实例别的记录（②），
/// 号不等时要求严格变大（①，号不等时「不减」就是严格变大）；这个号以前出现过（被别的非 0 号隔开之后又出现，②）
/// 一定伴着一处 ① 红，不另判，只在正好是同一对时写进 ① 那句说明。
/// `records` 按计数器索引，计数器与 jsn 在同一实例里同序（jsn = 实例代号 32 位 + 计数器 48 位，
/// D23（journal 的角色与格式） 已定项 9）⇒ 按计数器遍历就是按 jsn 排序。
///
/// ⚠️ 它钉的是「事务号计数只活在内存里」那个前提（`crates/singlefs-core/src/transaction.rs` 的
/// `publish_overwrite` 取这个实例见过的最大事务号加一、不取上一条记录上的事务号加一）：那条链式传递成立与否
/// 压在「每条恢复路径之后都取新实例代号」上，而在这一条之前没有任何东西判得出它失效
/// （代码轮第一轮三方判决 `research/prompts/m2-checker-supp-code-r1-main-verification.md` 判定四）。
/// 两个各自提交了的事务撞号而又紧挨着时，号不减、同号相连，这一条与「一事务两条」分不开——分得开的是提交标记，
/// 归 I-8.8（前缀里的事务不被切开）。
fn judge_transaction_numbers_per_instance(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut judged_pairs = 0u64;
    for (device, records) in records_by_device {
        let mut newest_non_empty_publish_by_instance: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        // (实例代号, 事务号) → 这个号在这块盘上第一次出现的计数器。
        let mut first_counter_by_transaction: BTreeMap<(u32, u64), u64> = BTreeMap::new();
        for (counter, record) in records {
            if record.transaction == TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {
                continue;
            }
            let first_counter_of_this_number = *first_counter_by_transaction
                .entry((record.instance, record.transaction))
                .or_insert(*counter);
            let previous = newest_non_empty_publish_by_instance
                .insert(record.instance, (*counter, record.transaction));
            let Some((previous_counter, previous_transaction)) = previous else {
                continue;
            };
            judged_pairs += 1;
            if record.transaction == previous_transaction {
                let separating_record = records
                    .range(previous_counter + 1..*counter)
                    .find(|(_, between)| between.instance == record.instance);
                judgements.judge("I-8.7", separating_record.is_none(), || {
                    let (separating_counter, separating) =
                        separating_record.expect("判红的这一支里，隔开它们的那条记录找到了");
                    format!(
                        "判据②：盘 {device} 实例 {} 事务号 {} 的两条记录（计数器 {previous_counter}、{counter}）中间坐着计数器 {separating_counter}（环槽 {}）上事务号 {} 的记录——这个号被隔开之后又出现",
                        record.instance, record.transaction, separating.slot, separating.transaction
                    )
                });
                continue;
            }
            // 号不等时，「被别的非 0 号隔开之后又出现」（②）一定伴着一处 ①：号先变走再变回来，中间必有一对是减的。
            // 所以 ② 的这一半不单独判，只在 ① 红的这一对上正好也是它时写进同一句说明。
            let reappears_after_another_number = first_counter_of_this_number != *counter;
            judgements.judge("I-8.7", record.transaction > previous_transaction, || {
                let decrease = format!(
                    "判据①：盘 {device} 实例 {} 的两条记录：计数器 {previous_counter} 上事务号 {previous_transaction}，计数器 {counter}（环槽 {}）上事务号 {}——同一实例按 jsn 排下来事务号减了",
                    record.instance, record.slot, record.transaction
                );
                if reappears_after_another_number {
                    format!(
                        "{decrease}；判据②：事务号 {} 在计数器 {first_counter_of_this_number} 出现过，被别的号隔开之后又在计数器 {counter} 出现",
                        record.transaction
                    )
                } else {
                    decrease
                }
            });
        }
    }
    if judged_pairs == 0 {
        judgements.not_applicable(
            "I-8.7",
            "环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来",
        );
    }
}

/// I-8.8（前缀里的事务不被切开）：同一实例在环里写出的记录，按事务看提交标记。判据各自单独判，
/// 红了在违例说明里逐条列出是哪几条（「判据③」「提交标记字节」「判据④」；① ② 是 I-8.7（实例内事务号不重号） 的）：
/// ③ 一组里带提交标记的至多一条，且它是这一组计数器最大的那一条；
/// 提交标记字节：那 1 字节只许 0 或 1，别的值判红（不许把它静默读成「不带」）；
/// ④ 一组里一条提交标记都没有时，它后面这个实例不许再有任何记录（事务号 0 的空发布也算）：
///    D23（journal 的角色与格式） 已定项 7「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」——
///    失败之后这个实例不再写任何记录（2026-09-23 主 agent 定案：空发布也在内）。
/// 逐盘判（与 I-8.6（反向链算法）、I-8.7 同一个口径）。一组 = 同一块盘上实例代号与事务号都相同的非 0 记录：
/// 事务号按实例计数（已定项 7），两个实例各有一个 1 是两个事务。组里的次序不在这里判——同号记录连不连续归 I-8.7 的 ②。
///
/// **射程**：
/// 一、**事务号 0 不成组**：它是不承载事务的空发布记录（D23（journal 的角色与格式） 已定项 19 ①
///    「不进『丢掉提交标记没出现的尾巴』判定」）；它自己带提交标记 1，拿它成组 ③ 当场就红。它照样受「提交标记字节」管。
/// 二、④ 看的是同一实例在这一组之后的每一条记录，不分号大号小、不分是不是事务号 0：没提交的组后面跟着一条空发布也红；
///    别的实例的记录不算（新实例接着写是失败之后该有的样子）。
/// 三、**一组里一条提交标记都没有，本身合法**：那是提交标记还没落盘的尾巴，恢复要丢掉的正是它（④ 管它是不是尾巴）。
///
/// **不适用**：同一块盘上没有哪一组两条以上（③ 没有对象）、也没有哪一组一条提交标记都没有（④ 没有对象）
/// ⇒ 整条报「不适用」，不报成立。「提交标记字节」这时照样判、判出别的值照样红，只是它单独成立不算这一条成立——
/// 第一版一事务一条、每条都带提交标记，可达镜像上都是这个形态，这一条的判别力在坏镜像上
/// （`crates/singlefs-harness/tests/checker_known_bad_images.rs`）。
fn judge_commit_markers_per_transaction(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut first_misplaced_commit_marker: Option<String> = None;
    let mut first_unrecognized_commit_marker: Option<String> = None;
    let mut first_record_after_an_uncommitted_transaction: Option<String> = None;
    let mut judged_transactions = 0u64;
    for (device, records) in records_by_device {
        for (counter, record) in records {
            match record.commit_marker {
                CommitMarker::Absent | CommitMarker::Present => {}
                CommitMarker::Unrecognized(byte) => {
                    first_unrecognized_commit_marker.get_or_insert_with(|| {
                        format!(
                            "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的提交标记字节是 {byte}——只许 0 或 1",
                            record.instance, record.slot
                        )
                    });
                }
            }
        }
        let mut records_by_transaction: BTreeMap<(u32, u64), Vec<(u64, &ScannedJournalRecord)>> =
            BTreeMap::new();
        for (counter, record) in records
            .iter()
            .filter(|(_, scanned)| scanned.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)
        {
            records_by_transaction
                .entry((record.instance, record.transaction))
                .or_default()
                .push((*counter, record));
        }
        for ((instance, transaction), members) in &records_by_transaction {
            let (last_counter, _) = members
                .last()
                .expect("每一组至少一条：组是拿记录一条条追加出来的");
            let member_counters: Vec<u64> = members.iter().map(|(counter, _)| *counter).collect();
            let commit_marked_counters: Vec<u64> = members
                .iter()
                .filter(|(_, member)| member.commit_marker == CommitMarker::Present)
                .map(|(counter, _)| *counter)
                .collect();
            if members.len() >= 2 {
                judged_transactions += 1;
            }
            let commit_marker_misplaced = match commit_marked_counters.as_slice() {
                [] => false,
                [only_commit_marked] => only_commit_marked != last_counter,
                [_, _, ..] => true,
            };
            if commit_marker_misplaced {
                first_misplaced_commit_marker.get_or_insert_with(|| {
                    format!(
                        "盘 {device} 实例 {instance} 事务号 {transaction} 的记录（计数器 {member_counters:?}）里带提交标记的是计数器 {commit_marked_counters:?}——至多一条、且只许是这一组计数器最大的 {last_counter}"
                    )
                });
            }
            if !commit_marked_counters.is_empty() {
                continue;
            }
            judged_transactions += 1;
            if let Some((later_counter, later)) = records
                .range(last_counter + 1..)
                .find(|(_, later)| later.instance == *instance)
            {
                first_record_after_an_uncommitted_transaction.get_or_insert_with(|| {
                    format!(
                        "盘 {device} 实例 {instance} 事务号 {transaction} 的记录（计数器 {member_counters:?}）一条提交标记都没有，后面计数器 {later_counter}（环槽 {}）上却还有同实例事务号 {} 的记录——没提交的事务之后这个实例不许再写任何记录",
                        later.slot, later.transaction
                    )
                });
            }
        }
    }
    let violated_criteria: Vec<String> = [
        ("判据③", first_misplaced_commit_marker),
        ("提交标记字节", first_unrecognized_commit_marker),
        ("判据④", first_record_after_an_uncommitted_transaction),
    ]
    .into_iter()
    .filter_map(|(criterion, first_violation)| {
        first_violation.map(|detail| format!("{criterion}：{detail}"))
    })
    .collect();
    if !violated_criteria.is_empty() {
        judgements.judge("I-8.8", false, || violated_criteria.join("；"));
    } else if judged_transactions > 0 {
        judgements.judge("I-8.8", true, String::new);
    } else {
        judgements.not_applicable(
            "I-8.8",
            "环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立",
        );
    }
}

/// 记录新根段里两条指针的落点（D23（journal 的角色与格式） 已定项 4 的字段表：树表指针 86、映射根指针 86，再往后是树 ID 水位 8 与 F 8）。
const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;
const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 86..172;

/// 由记录施加出来、根槽从没落盘的那一版：一条 journal 记录自带新根段（树表指针与映射根指针），恢复把它施加在所选根之上，
/// 这一版就成了现行版；之后写行那次发布把它换下的单元放进 defer——那些单元仍占着空间、仍算在「已分配」里，
/// 而根环里没有任何一条根引用它们（它自己的根槽一次都没写过）。
struct VersionAppliedOnlyByRecords {
    instance: u32,
    checkpoint_txg: u64,
    new_root_segment: Vec<u8>,
}

impl VersionAppliedOnlyByRecords {
    fn tree_table_and_mapping_root_pointers(&self) -> (&[u8], &[u8]) {
        (
            &self.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
            &self.new_root_segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
        )
    }
}

/// 回退候选集要补上的那几版：**由记录施加出来、根槽从没落盘的那一版**（增补 2 收口表第 54 行，
/// 2026-09-23 用户定候选 b，`research/prompts/m2-placement-falsepositive-r1-main-verification.md` Z3）。
/// 只看盘上这一份镜像能读出的东西，四条同时成立才算一版：
///
/// ① **它被施加过**：最新那条根指着的实例表里有这个实例的行 (i, Ti, Wi)，而这条记录的 checkpoint_txg ≤ Ti。
///    行里的 Ti 就是那次恢复停在哪一版（`crates/singlefs-core/src/mount.rs` 写行时取恢复之后的有效根的 txg），
///    施加出来的每一版的 txg 都不超过它；没有这个实例的行 ⇒ 还没有哪一次恢复把它施加出来，环里的记录只是残留
///    （层 0 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格：施加要等下一次挂载，这一刻还不算）。
///    记录要带提交标记（D23（journal 的角色与格式） 已定项 7：不带的是恢复要丢的尾巴）。
/// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。
/// ③ **不低于回退下界**：txg ≥ 最新根带的 F，与根环里的候选根同一条（D16（发布语义） 已定项 1）。
/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小。
///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 环里最旧有效根)」
///    （D3（空间分配） 已定项 7）。环里最旧的有效根比这一版老 ⇒ 最旧有效根 < 这一版 < 释放代，③ 又让 F ≤ 这一版 ⇒ 回收不了，
///    它们还算在已分配里；反过来环里最旧的有效根已经比这一版新，这一版里只有它引用的单元释放代都不超过那条根，已可回收复用，
///    再把它并进来就是拿已经不归它的槽去比。
///
/// 一版由那条记录的 (实例代号, checkpoint_txg) 认；两块盘各一份镜像记录，取先读到的那一份。
fn versions_applied_only_by_records(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    roots: &[(u64, u64, crate::RootView)],
    instance_table_rows: &[InstanceTableRow],
    rollback_floor: u64,
) -> Vec<VersionAppliedOnlyByRecords> {
    let is_abandoned = |instance: u32, checkpoint_txg: u64| {
        instance_table_rows
            .iter()
            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)
    };
    let Some(oldest_valid_root_txg) = roots
        .iter()
        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
    else {
        return Vec::new();
    };
    let mut versions: BTreeMap<(u32, u64), Vec<u8>> = BTreeMap::new();
    for (_, records) in records_by_device {
        for record in records.values() {
            let applied_by_a_recovery = record.commit_marker == CommitMarker::Present
                && instance_table_rows.iter().any(|row| {
                    row.instance == record.instance
                        && record.checkpoint_txg <= row.published_checkpoint_txg
                });
            let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
                root.instance == record.instance && root.checkpoint_txg == record.checkpoint_txg
            });
            let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;
            let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;
            if applied_by_a_recovery
                && !root_slot_is_readable
                && at_or_above_the_floor
                && its_units_are_not_reclaimable_yet
            {
                versions
                    .entry((record.instance, record.checkpoint_txg))
                    .or_insert_with(|| record.new_root_segment.clone());
            }
        }
    }
    versions
        .into_iter()
        .map(
            |((instance, checkpoint_txg), new_root_segment)| VersionAppliedOnlyByRecords {
                instance,
                checkpoint_txg,
                new_root_segment,
            },
        )
        .collect()
}

/// 走读记下的被引用落点（(设备, 起点槽, 跨度) → 第一次引用它的是谁）逐盘加起来是多少槽。
/// I-3.1（已分配统计对得上） 拿走完全部候选版本之后那一份算，I-3.11（已分配减 defer 等于最新根走读） 拿只走过最新根那一刻的那一份算：
/// 两条是同一个走法、同一个加法，差别只在取哪一刻。
fn slots_referenced_per_device(
    references: &BTreeMap<(u32, u64, u64), String>,
) -> BTreeMap<u32, u64> {
    let mut slots_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    for (device, _start_slot, span_slots) in references.keys() {
        *slots_per_device.entry(*device).or_insert(0) += span_slots;
    }
    slots_per_device
}

/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    let system_configurations = chosen_system_configurations(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = system_configurations
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
    if chosen.is_empty() {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    let geometry = chosen[0].2;
    // 各盘择到的系统配置要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的系统配置 fsid 与盘 {} 的不同：这块盘不属于这个池",
                    chosen[0].0
                )
            },
        );
    }
    let devices = reader.devices();
    let regions = usize::try_from(geometry.regions).expect("R");
    let region_devices = &geometry.region_devices[..regions.min(3)];
    let distinct: BTreeSet<u32> = region_devices.iter().copied().collect();
    let heaviest = devices
        .iter()
        .map(|device| {
            region_devices
                .iter()
                .filter(|candidate| *candidate == device)
                .count()
        })
        .max()
        .unwrap_or(0);
    let pigeonhole = regions.div_ceil(devices.len().max(1));
    root_ring_judgements.judge(
        "I-7.6",
        distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
        || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
    );
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("8 字节"),
    );
    let roots = valid_roots(reader, &geometry);
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    // I-8.6、I-8.7 与 I-8.8 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
    let journal_records_by_device =
        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
    judge_journal_back_chain(&journal_records_by_device, &mut root_ring_judgements);
    judge_transaction_numbers_per_instance(&journal_records_by_device, &mut root_ring_judgements);
    judge_commit_markers_per_transaction(&journal_records_by_device, &mut root_ring_judgements);
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        root_ring_judgements.not_applicable(
            "I-7.3",
            "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
        );
        return root_ring_judgements.into_report();
    }
    judge_root_ring_health(&roots, &mut root_ring_judgements);
    let newest_index = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
        .map(|(index, _)| index)
        .expect("非空");
    let mut walk = Walk {
        reader,
        judgements: root_ring_judgements,
        filesystem_identifier_low,
        references: BTreeMap::new(),
        visited_units: BTreeSet::new(),
        walk_failures: Vec::new(),
        tree_identifiers_in_tables: BTreeSet::new(),
        inode_object_birth: BTreeMap::new(),
        inode_tree_walked: false,
        largest_inode_number_in_the_inode_tree: None,
        data_unit_objects: Vec::new(),
        accounting: BTreeMap::new(),
        accounting_seen: false,
        instance_table_rows: Vec::new(),
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        referenced_units_judged_against_their_pointer: 0,
    };
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
    // I-3.11（已分配减 defer 等于最新根走读）要的「从最新有效根走读到的、这块盘上被引用的槽数」：走法与 I-3.1 同一个
    // （`note_reference` 记下的 (设备, 起点槽, 跨度)），只取最新根——这一刻 `references` 里还只有最新根这一遍记下的。
    let slots_referenced_by_the_newest_root = slots_referenced_per_device(&walk.references);
    // I-4.8（近 K 代根校验和自洽）与 I-7.4（近 K 代块未被复用）：候选集里任一根（最新根也在候选集里）出发遍历，所有块的校验和
    // 都与父指针一致、走读不断——最新根那一次各算一格；候选集只剩最新根时两条都还判得到（本地攻方腿：全称量词在单元素集合上照样成立）。
    let newest_txg = roots[newest_index].2.checkpoint_txg;
    let newest_walked_into_reused_or_erased_unit =
        !newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;
    walk.judgements
        .judge("I-4.8", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）出发的遍历有单元对不上或读不出")
        });
    walk.judgements
        .judge("I-7.4", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）")
        });
    let accounting_seen = walk.accounting_seen;
    let accounting = walk.accounting.clone();
    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
    // 再加一条：txg ≥ 最新根带的回退下界 F（D16（发布语义） 已定项 1 的回退候选集）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let newest_rollback_floor = u64::from_le_bytes(
        roots[newest_index].2.record_bytes[130..138]
            .try_into()
            .expect("8 字节"),
    );
    // 掉出遍历的根槽按理由各记一次（两样都占的两边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
    let mut root_slots_dropped_as_abandoned = 0u64;
    let mut root_slots_dropped_below_floor = 0u64;
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            let abandoned = instance_table_rows.iter().any(|row| {
                row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
            });
            let below_floor = root.checkpoint_txg < newest_rollback_floor;
            let walked = *index == newest_index || (!abandoned && !below_floor);
            if !walked {
                if abandoned {
                    root_slots_dropped_as_abandoned += 1;
                }
                if below_floor {
                    root_slots_dropped_below_floor += 1;
                }
            }
            walked
        })
        .map(|(index, _)| index)
        .collect();
    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        if index != newest_index {
            let mismatches_before = walk.judgements.violation_count("I-2.1");
            let failures_before = walk.walk_failures.len();
            walk.walk_root(&root.record_bytes, false);
            // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
            // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
            let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
                > mismatches_before
                || walk.walk_failures.len() > failures_before;
            let root_txg = root.checkpoint_txg;
            walk.judgements
                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                    format!(
                        "候选根 txg {root_txg} 引用的单元已被复用或抹头（校验和对不上或头用不了）"
                    )
                });
            walk.judgements
                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                    format!("候选根 txg {root_txg} 出发的遍历有单元对不上或读不出")
                });
        }
    }
    // 回退候选集补上「由记录施加出来、根槽从没落盘的那一版」（2026-09-23 用户定候选 b）：它们没有根槽，
    // 上面按根环走的那一遍走不到，而它们换下的单元还在 defer 里、仍算在已分配里（增补 2 收口表第 54 行那 12 个状态差的 65 536 字节）。
    // 走法与判定照候选根：引用进同一个并集（I-3.1 / I-5.1），单元照判 I-2.1 等，I-7.4 / I-4.8 每一版各判一格。
    let versions_applied_only_by_records = versions_applied_only_by_records(
        &journal_records_by_device,
        &roots,
        &instance_table_rows,
        newest_rollback_floor,
    );
    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");
        let failures_before = walk.walk_failures.len();
        walk.walk_version_applied_only_by_records(version);
        let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
            > mismatches_before
            || walk.walk_failures.len() > failures_before;
        let (instance, version_txg) = (version.instance, version.checkpoint_txg);
        walk.judgements
            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）"
                )
            });
        walk.judgements
            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）出发的遍历有单元对不上或读不出"
                )
            });
    }
    let referenced_units_judged_against_their_pointer =
        walk.referenced_units_judged_against_their_pointer;
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    // I-1.2 与 I-4.2 在遍历方向上判的是同一批单元：走读一个被引用单元都没走到时两条都没有对象
    // （根槽读得出而树表指针全零、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
    if referenced_units_judged_against_their_pointer == 0 {
        for invariant in ["I-1.2", "I-4.2"] {
            judgements.not_applicable(
                invariant,
                "走读一个被引用的单元都没读到（实例表单元按 I-1.2 那一行的例外不进这两条）：出生身份与已发布谓词都没有对象",
            );
        }
    }
    // I-1.8：扫描方向按已发布谓词过滤之后归并成组。谓词要挂载根的 (实例代号, txg) 与它指着的那一版实例表，
    // 所以这一遍排在走读之后（`instance_table_rows` 是走最新根时取的）。
    let published = PublishedPredicate {
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        instance_table_rows: &instance_table_rows,
    };
    let content_units =
        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
    if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {
        judgements.not_applicable(
            "I-1.8",
            "扫描方向没有两份归并到一起、或两组归并到同一个 key 的码 1 / 码 3 已发布单元：归并与定序都比不出来",
        );
    }
    // 这两遍各自按候选集里每条根读树表与树节点：共用一份按位置条目记的缓存，候选根常指着同一个单元，层 0 每个崩溃状态都跑。
    let mut index_node_cache = IndexNodeCache::new();
    judge_release_generation_and_tree_table_birth(
        reader,
        &roots,
        &candidate_indexes,
        newest_index,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_allocation_records_disjoint(
        reader,
        &roots,
        &candidate_indexes,
        &mut index_node_cache,
        &mut judgements,
    );
    let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(
        reader,
        &roots,
        &candidate_indexes,
        &versions_applied_only_by_records,
        tree_table_pointer_of_the_version_the_next_mount_applies_first(
            &journal_records_by_device,
            &roots,
            newest_index,
        ),
        &mut index_node_cache,
    );
    judge_allocation_generations_against_unit_births(
        reader,
        &allocation_node_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
    // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
    // 另一半（> 全部已发布的墓碑记录的对象 ID）今天没有对象：墓碑是打包记录类型 1，这一版一片都不写
    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
    let inode_number_watermark = accounting
        .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
        .copied();
    match (
        inode_number_watermark,
        walk.largest_inode_number_in_the_inode_tree,
    ) {
        (Some(watermark), Some(largest_inode)) => {
            judgements.judge("I-9.6", watermark > largest_inode, || {
                format!(
                    "记账里的 inode 号水位 {watermark} 不大于 inode 树内最大 key {largest_inode}"
                )
            });
        }
        (None, _) => judgements.not_applicable(
            "I-9.6",
            "最新的根下面没有记账树、或记账里没有 inode 号水位那一行",
        ),
        (Some(_), None) => {
            judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")
        }
    }
    if !walk.inode_tree_walked {
        judgements.not_applicable(
            "I-9.12",
            "最新的根下面走不到 inode 树的内部节点（树表 0 条或根读不出）",
        );
    }
    for (inode, unit_object_birth, what) in &walk.data_unit_objects {
        if let Some(record_birth) = walk.inode_object_birth.get(inode) {
            judgements.judge("I-9.10", record_birth == unit_object_birth, || {
                format!(
                    "{what} 的对象出生代 {unit_object_birth} 与 inode 记录的 {record_birth} 不符"
                )
            });
        }
    }
    // I-5.1：同一块盘上，不同的引用不许占重叠的槽（两条位置条目落在不同盘上是显式的多副本）。
    let mut per_device: BTreeMap<u32, Vec<(u64, u64, &String)>> = BTreeMap::new();
    for ((device, slot, span), what) in &walk.references {
        per_device
            .entry(*device)
            .or_default()
            .push((*slot, *span, what));
    }
    for (device, mut ranges) in per_device.clone() {
        ranges.sort_unstable_by_key(|(slot, span, _)| (*slot, *span));
        for pair in ranges.windows(2) {
            let (first_slot, first_span, first_what) = pair[0];
            let (second_slot, _, second_what) = pair[1];
            judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {
                format!("盘 {device}：{first_what}（槽 {first_slot} 跨 {first_span}）与 {second_what}（槽 {second_slot}）重叠")
            });
        }
    }
    // I-3.1 判红时，说明文字里带一段机理标识：遍历覆盖了哪些根、剩下的按什么理由没覆盖。
    // 判读的一方（`singlefs-harness` 的 `history::allocation_statistic_mechanism`）按它分辨「记账多算」是哪一种机理造成的：
    // 环转过一圈把 F 之上的根挤出了环，还是回退下界把环里读得到的根挡在了外面——两者的签名（只有 I-3.1 红、记账多于遍历）
    // 一模一样，不带这一段就只能按签名认，机理不同的新问题会被「已知红」清单接走（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半）。
    let root_ring_slot_count = geometry.regions * geometry.slots_per_region;
    let oldest_readable_root_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
        .unwrap_or(0);
    let readable_root_slot_count = u64::try_from(roots.len()).expect("根槽数");
    let walked_root_slot_count = u64::try_from(candidate_indexes.len()).expect("候选根槽数");
    let walked_versions_applied_only_by_records =
        u64::try_from(versions_applied_only_by_records.len()).expect("版本数");
    // 「并进遍历的由记录施加出来的版本」那一段是 2026-09-23 候选 b 加的；判读的一方按字段名取数（`leading_number_after`），
    // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
    let mechanism = move || {
        format!(
            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    };
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    // I-3.11：同一块盘上「已分配」减「defer 待释放」对只走最新根那一遍得到的和。
    let slots_referenced_by_every_walked_version = slots_referenced_per_device(&walk.references);
    if accounting_seen {
        for device in &devices {
            let walked = slots_referenced_by_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            judgements.judge("I-3.1", allocated == Some(walked), || {
                format!(
                    "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}{}",
                    mechanism()
                )
            });
            let capacity = reader
                .device_bytes(*device)
                .map(|bytes| (bytes / SLOT_BYTES - geometry.unit_area_start_slot) * SLOT_BYTES);
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
            // I-3.11（已分配减 defer 等于最新根走读）：写者把活单元记成已释放、放进 defer 时，已分配与空闲两个和照样对得上
            // （I-3.1 / I-5.2 都绿），只有这一条的差对不上。判据写成加法（defer + 最新根走读 == 已分配），盘上读来的值大到溢出也判红、不回绕。
            let deferred = accounting
                .get(&(STATISTIC_DEFER_QUEUE_BYTES, *device))
                .copied();
            let referenced_by_the_newest_root = slots_referenced_by_the_newest_root
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            judgements.judge(
                "I-3.11",
                matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),
                || {
                    format!("盘 {device}：记账的已分配 {allocated:?} 减 defer 待释放 {deferred:?}，不等于从最新根（txg {newest_txg}）走读到的 {referenced_by_the_newest_root}")
                },
            );
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-3.11", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（码 2 单元头 ∪ 走过的树表条目；
    // 单元头那一半不数没发布过的孤儿，读法见 `scanned_tree_identifiers`）。
    let watermark = roots
        .iter()
        .map(|(_, _, root)| root.tree_identifier_watermark)
        .max()
        .unwrap_or(0);
    let newest_published_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let mut seen = scanned_tree_identifiers(
        reader,
        &geometry,
        newest_published_txg,
        &instance_table_rows,
        filesystem_identifier_low,
    );
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    judgements.into_report()
}
