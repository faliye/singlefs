//! 只读不判地收一条根引用到的单元：分配记录的解析、按落点读副本与读缓存、各棵树的节点与数据单元。

use super::*;

/// 一条分配记录跨度段次高位的读法（盘上读到什么就是什么：与已释放标志同时置的也照读，由 I-3.17 判红）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QuarantineMarkInTheSpanField {
    Absent,
    Present,
}

/// 一条分配记录在盘上的样子。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AllocationRecordView {
    pub(super) device: u32,
    pub(super) slot: u64,
    /// 跨度段去掉已释放标志与隔离记号之后的值：这条记录罩住 [槽号, 槽号 + 跨度) 这些槽。
    pub(super) span_slots: u64,
    /// 仍分配时是分配代；已释放时是释放代（D3（空间分配） 已定项 7）。
    pub(super) generation: u64,
    pub(super) is_released: bool,
    pub(super) quarantine_mark: QuarantineMarkInTheSpanField,
}

pub(super) fn parse_allocation_record(entry: &[u8]) -> AllocationRecordView {
    let span_field = read_u16(entry, ALLOCATION_RECORD_SPAN_FIELD_OFFSET);
    AllocationRecordView {
        device: read_u32(entry, 0),
        slot: read_six_byte_unsigned(entry, 4),
        span_slots: u64::from(
            span_field & !(ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINE_MARK),
        ),
        generation: read_u64(entry, ALLOCATION_RECORD_GENERATION_OFFSET),
        is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
        quarantine_mark: if span_field & ALLOCATION_RECORD_QUARANTINE_MARK == 0 {
            QuarantineMarkInTheSpanField::Absent
        } else {
            QuarantineMarkInTheSpanField::Present
        },
    }
}

/// 树的种类码（D8（核心索引结构） 已定项 9 的登记表）在引用集合这一遍里的读法：盘上读到的码语义上允许未知，
/// 做成一个显式成员，`match` 照样穷举（`code-discipline.md`「分支：穷举写全，不写通配臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TreeKindForReferenceScan {
    Extent,
    Inode,
    Allocation,
    Accounting,
    /// `deleted_inodes`：只有一个码 2 根（层级 0），条目不引用别的单元。
    DeletedInodes,
    /// 墓碑树、谱系树：码 2 根之下是码 3 容器（条目照 inode 树内部节点的 120 字节形态）。
    ContainerTree,
    /// deadlist 树：多层码 2 树，条目只是 key、不引用别的单元，
    /// 根之下的节点各占一个槽。
    Deadlist,
    /// 只读快照的 inode 根（码 14）：根指着的是建快照那一刻活头 inode 树的根，往下照 inode 树数。
    SnapshotInodeRoot,
    /// 只读快照的 extent 根（码 15）：根指着的是建快照那一刻活头 extent 树的根，往下照 extent 树数。
    SnapshotExtentRoot,
    /// logged_ops 树：多层码 2 树，条目是意图、不引用别的单元，
    /// 根之下的节点各占一个槽。
    LoggedOperations,
    /// 条目格式还没有条款的树（livelist 6、稀疏旁表 7 与这一版不认识的码）：day-1 注册的那几棵根指针为零、
    /// 走到这里就是它们真有了根节点，那时这条根的引用集合数不全。
    WithoutWalkableEntryFormat(u16),
}

impl TreeKindForReferenceScan {
    pub(super) const fn of(code: u16) -> Self {
        match code {
            TREE_KIND_EXTENT => TreeKindForReferenceScan::Extent,
            TREE_KIND_INODE => TreeKindForReferenceScan::Inode,
            TREE_KIND_ALLOCATION => TreeKindForReferenceScan::Allocation,
            TREE_KIND_ACCOUNTING => TreeKindForReferenceScan::Accounting,
            TREE_KIND_DEADLIST => TreeKindForReferenceScan::Deadlist,
            TREE_KIND_SNAPSHOT_INODE_ROOT => TreeKindForReferenceScan::SnapshotInodeRoot,
            TREE_KIND_SNAPSHOT_EXTENT_ROOT => TreeKindForReferenceScan::SnapshotExtentRoot,
            TREE_KIND_DELETED_INODES => TreeKindForReferenceScan::DeletedInodes,
            TREE_KIND_LOGGED_OPERATIONS => TreeKindForReferenceScan::LoggedOperations,
            TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
                TreeKindForReferenceScan::ContainerTree
            }
            unregistered => TreeKindForReferenceScan::WithoutWalkableEntryFormat(unregistered),
        }
    }
}

/// 一条有效根自己引用了哪些落点、它的树表里有哪些条目：I-3.9（释放代落在停止引用它的那一格区间里）与
/// I-9.14（树表条目的诞生 txg 跨根不变）按回退候选集里每条根各走一遍取这份东西（I-7.4（近 K 代块未被复用） 已经这么走）。
/// **与主走读那一遍分开跑**：主走读用 `visited_units` 跳过走过的单元，共享子树只算进第一条走到它的根，
/// 而这两条要的正是「这条根自己引用了什么」；这一遍只读不判，别的不变量的评估次数不受它影响。
#[derive(Clone)]
pub(super) struct RootReferences {
    /// 这条根引用的落点 (设备身份, 起点槽)：与分配记录的 key 同形。只在 `depth` 是「走到叶」时数得全。
    pub(super) placements: BTreeSet<(u32, u64)>,
    /// 树表条目：树 ID → 诞生 txg。
    pub(super) tree_table_birth_txg: BTreeMap<u64, u64>,
    /// 树表单元自己落在哪 (设备身份, 起点槽)：几条根共用同一个树表单元时，它们的条目不是两处独立的证据。
    pub(super) tree_table_placement: Option<(u32, u64)>,
    /// 树表读成什么样：I-9.28 按它列出指针非零而条目没读全的根（报「部分没判」）。
    pub(super) tree_table_reading: TreeTableReading,
    /// 这条根的分配记录树叶里的记录（I-3.9 只取最新根那一份：当前的账）。
    pub(super) allocation_records: Vec<AllocationRecordView>,
    /// 这一遍按哪个深度扫的：`placements` 与 `is_complete` 只有「走到叶」那个深度上作数。
    pub(super) depth: ReferenceScanDepth,
    /// 该读的单元读不出、或走到了没有条款的形状：这条根的引用集合数不全，I-3.9 判不了。
    pub(super) is_complete: bool,
    /// 这条根自己的中央映射（只在「走到叶」那个深度上读）：豁免三类之外的指针按它认位置
    /// （[`RootReferences::note_located_by_the_central_mapping`]），往下读子树的次级读法也按它认（[`PositionOfASecondaryRead`]）。
    /// 包一层 `Rc`：往下读的那几个函数同时可变借用这份引用集合、只读借用这份映射。
    pub(super) central_mapping: std::rc::Rc<CentralMappingLocationsOfAVersion>,
}

/// 一条根的树表这一遍读成什么样（[`references_of_root`] 只读不判；I-9.28 拿它判「部分没判」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TreeTableReading {
    /// 树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出，与 [`TreeTableReading::Unreadable`] 一样算没读到，
    /// 不再算「这条根没有树表」。
    PointerAllZero,
    /// 指针非零，两条位置条目都读不出、或整单元校验和对不上、或解不开成码 2 节点。
    Unreadable,
    /// 读出来了，有条目比字段表短（那几条不解字段）。
    SomeEntryShorterThanTheFieldTable,
    /// 读出来了，每条条目都按字段表解了。
    EveryEntryRead,
}

impl RootReferences {
    /// 记一条指针自己的位置条目：自举豁免的三类（树表、映射树的节点、实例表）与已经是映射位置的（映射叶条目）走这里。
    pub(super) fn note(&mut self, pointer: &PointerView) {
        if pointer.all_zero {
            return;
        }
        for location in &pointer.locations {
            self.placements.insert((location.device, location.slot));
        }
    }

    /// 记一条豁免三类之外的指针：按这条根自己的中央映射认位置（[`CentralMappingLocationsOfAVersion::locate`]），提示过期时记映射位置、不记提示
    fn note_located_by_the_central_mapping(
        &mut self,
        pointer: &PointerView,
        class: MappingKeyUnitClass,
    ) {
        let located = self
            .central_mapping
            .locate(pointer, PositionAuthority::TheCentralMapping(class));
        self.note(&located);
    }
}

/// 按位置条目读一个单元、不判任何不变量：两条位置条目里第一份校验和对得上的那一份（判定归主走读的 I-2.1）。
pub(super) fn read_unit_without_judging(
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

/// 只读不判的次级读法跟随一条指针时，被指单元落在哪里以谁为准：同主走读的 [`PositionAuthority`]（D19（块指针的结构与宽度预算）
/// 已定项 5「位置条目降为提示，中央映射是唯一入口」、已定项 8「不进映射的三类（自举豁免）」），另带上这一版的中央映射——次级读法
/// 一遍里逐版读，每一版查的是它自己的映射（同一条过期提示在一版里指着源槽、在搬迁之后的那几版里映射已换到目的地）。
#[derive(Clone, Copy, Debug)]
pub(super) enum PositionOfASecondaryRead<'mapping> {
    /// 自举豁免的三类——映射树自己的节点、树表单元、实例表单元：按指针自己的位置条目读。
    ThePointerItself,
    /// 豁免三类之外的码 1 / 码 2 / 码 3：按这一版的中央映射认位置（[`CentralMappingLocationsOfAVersion::locate`]）。这一版的映射读不出时
    /// 是空表，照提示读，与主走读同一口径。
    TheCentralMappingOfTheVersion(&'mapping CentralMappingLocationsOfAVersion),
}

impl PositionOfASecondaryRead<'_> {
    /// 一条指针按这一读法认的位置：查找路径给的类是 `class`（指针不带类标签，D19（块指针的结构与宽度预算） 已定项 8）。
    pub(super) fn locate(self, pointer: &PointerView, class: MappingKeyUnitClass) -> PointerView {
        match self {
            Self::ThePointerItself => *pointer,
            Self::TheCentralMappingOfTheVersion(central_mapping) => {
                central_mapping.locate(pointer, PositionAuthority::TheCentralMapping(class))
            }
        }
    }
}

/// 读一个认过位置的单元、不判任何不变量（`located` 是 [`PositionOfASecondaryRead::locate`] 交回的）：形态同主走读的
/// `Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from` 去掉判定——提示与映射位置不同时先按提示读，CRC 对得上且头里出生身份
/// 与指针相同才用（[`copy_at_a_location_hint`]）；读不出再按映射位置两条位置条目读、CRC 过滤；两者相同时照位置条目读。
/// 提示那一槽已释放、被 discard、被别的单元复用都不判。
pub(super) fn read_located_unit_without_judging(
    reader: &dyn ImageReader,
    located: &PointerView,
    unit_bytes: usize,
) -> Option<Vec<u8>> {
    if located.hints_differ_from_the_locations_the_walk_takes() {
        if let Some(unit) = copy_at_a_location_hint(reader, located, unit_bytes) {
            return Some(unit);
        }
    }
    read_unit_without_judging(reader, located, unit_bytes)
}

/// 次级读法那三份缓存（码 2 节点、码 3 单元、数据单元头；三处都先按 [`PositionOfASecondaryRead::locate`] 认位置，再经这里调
/// [`read_located_unit_without_judging`]）的读法：键是认过位置之后第 0 条位置条目的 (设备身份, 起点槽, 整单元校验和)——豁免三类与映射里查不到的是指针自己的，
/// 查得到的是映射位置。读出来的一律记下（内容由校验和定：同一把键读出来的是同一个单元）；「读不出」只在没有试提示那一支时记下：
/// 试了提示的那一次读不出不记（同一个映射落点，另一版的提示可能还读得出），缓存里记着「读不出」而这一次要试提示的，照读一遍。
/// 同一条过期提示在一版里读得出旧副本、在源段 discard 之后的那一版读不出，按提示当键会把后者的「读不出」带进别的版本。
pub(super) fn read_through_a_cache_keyed_by_the_located_placement<Parsed: Clone>(
    cache: &mut BTreeMap<(u32, u64, u32), Option<Parsed>>,
    located: &PointerView,
    read_and_parse: impl FnOnce() -> Option<Parsed>,
) -> Option<Parsed> {
    let key = (
        located.locations[0].device,
        located.locations[0].slot,
        located.locations[0].checksum,
    );
    let tries_the_hint = located.hints_differ_from_the_locations_the_walk_takes();
    match cache.get(&key) {
        Some(Some(parsed)) => return Some(parsed.clone()),
        Some(None) if !tries_the_hint => return None,
        Some(None) | None => {}
    }
    let parsed = read_and_parse();
    if parsed.is_some() || !tries_the_hint {
        cache.insert(key, parsed.clone());
    }
    parsed
}

/// 共同前缀里类标签的偏移（D18（块里携带什么信息） 已定项 7 / 11：三种码同一处）。
pub(super) const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;

/// 一类单元的整单元长度：码 1 数据单元与码 3 打包记录单元 32 KiB，码 2 节点 16 KiB；登记表之外的码没有长度（交 None）。
pub(super) fn unit_bytes_of_the_class(class: MappingKeyUnitClass) -> Option<usize> {
    match class {
        MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
            Some(data_unit_bytes())
        }
        MappingKeyUnitClass::IndexNode => Some(node_bytes()),
        MappingKeyUnitClass::Unregistered(_) => None,
    }
}

/// 一类单元占几个 16 KiB 槽（引用集合按它记）：32 KiB 的两槽，16 KiB 的一槽。
pub(super) fn slots_spanned_by_a_unit_of_the_class(class: MappingKeyUnitClass) -> Option<u64> {
    unit_bytes_of_the_class(class)
        .map(|unit_bytes| u64::try_from(unit_bytes).expect("16384 或 32768") / SLOT_BYTES)
}

/// 树里一条内部条目的孩子是不是条目类型段说的那一类：只读孩子的单元头取类标签，不按类型段定读多长（判决
/// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行：按类型段定长度时，类型段与孩子真实的类不符，
/// 按那个长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）。
/// 两条位置条目都看：头里的类标签是登记过的码、且按那一类的长度读回的整单元校验和对得上，这一份才算自证过的孩子；
/// 有一份自证过的孩子是 `class_the_entry_type_names` 之外的一类，交回那一类（两份各自证成两个另一类时交回位置条目在前的那一份的）；
/// 自证过的都是类型段那一类、或哪一份都自证不过（类标签不在登记表里、读不出、校验和对不上：陈旧副本、坏副本），交回 None
/// （照类型段那一支走，读不出照旧归 I-2.1）。
/// 不在第一份自证过的副本处停（ H1 H-V 那一行、第三节第 2 条：
/// 两份按各自的位置条目都自证得过、却是两类时，只看第一份的话结论随那一份在哪块盘翻面）。
/// 头里的类就是类型段那一类的副本不影响结论（ N1 两故障 H1 那一行：
/// 它是陈旧副本时，第二份照样要看）。
/// 类型段不认得的条目（`class_the_entry_type_names` 是 None）没有「它说的那一类」，交回孩子自证过的那一类。
/// 只认按它自己的长度自证过的孩子：头里的类标签坏了的单元按那个长度对不上校验和，不会被当成「另一类」。只读、不判。
pub(super) fn child_unit_class_other_than_the_entry_type_names(
    reader: &dyn ImageReader,
    child: &PointerView,
    class_the_entry_type_names: Option<MappingKeyUnitClass>,
) -> Option<MappingKeyUnitClass> {
    child.locations.iter().find_map(|location| {
        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)
            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)
    })
}

/// 一份副本（一条位置条目）按它自己那一类的长度整份自证过时交回那一类：头按最短的单元（码 2，16 KiB）那么长读取类标签
/// （读者只认整块的读，32 KiB 的单元头也在这前 16 KiB 里），类标签是登记过的码、按那一类的长度读回的整单元校验和与位置条目对得上；
/// 类标签没登记、读不出、校验和对不上交 None。走读里「自证过」只有这一份判据
/// 认孩子是别类（[`child_unit_class_other_than_the_entry_type_names`]）、
/// 认另一份是类型段那一类（[`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`]，F-W）、
/// 类型段 0 认「子单元是码 2」（[`class_of_the_first_copy_verifying_at_the_length_of_its_class`]，F-ID-b）都调它。只读、不判。
fn class_of_a_copy_verifying_at_the_length_of_its_class(
    reader: &dyn ImageReader,
    location: &PointerLocation,
) -> Option<MappingKeyUnitClass> {
    let offset = location.slot * SLOT_BYTES;
    let class_in_the_header = MappingKeyUnitClass::of(
        reader.read(location.device, offset, node_bytes())?[UNIT_HEADER_CLASS_TAG_OFFSET],
    );
    let unit_bytes = unit_bytes_of_the_class(class_in_the_header)?;
    reader
        .read(location.device, offset, unit_bytes)
        .filter(|content| crc32_castagnoli_table(content) == location.checksum)
        .map(|_content| class_in_the_header)
}

/// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本是哪一类（[`class_of_a_copy_verifying_at_the_length_of_its_class`]）；
/// 哪一份都没自证过交 None，调用方当读不出。
/// 墓碑树、谱系树类型段 0 那一支认「子单元是码 2」用它（F-ID-b）。只读、不判。
pub(super) fn class_of_the_first_copy_verifying_at_the_length_of_its_class(
    reader: &dyn ImageReader,
    child: &PointerView,
) -> Option<MappingKeyUnitClass> {
    child
        .locations
        .iter()
        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))
}

/// [`child_unit_class_other_than_the_entry_type_names`] 认出另一类之后再问一句：有没有一份副本头里的类标签就是 `class_the_entry_type_names`、
/// 按那一类的长度读回的整单元校验和对得上。
/// 有的话读者用的就是这一份（core 按条目那一类的长度逐份读、对得上就用），走读照类型段那一支走它的子树；
/// 槽数也按这一类登记（C739 后一半）。只在认出另一类之后调：常见的条目不多读。只读、不判。
///
/// # Panics
/// `class_the_entry_type_names` 没有长度：调用方只拿类型段认得的那两类（码 2、码 3）来问。
pub(super) fn some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
    reader: &dyn ImageReader,
    child: &PointerView,
    class_the_entry_type_names: MappingKeyUnitClass,
) -> bool {
    assert!(
        unit_bytes_of_the_class(class_the_entry_type_names).is_some(),
        "类型段认得的那一类是码 2 或码 3，都有长度"
    );
    child.locations.iter().any(|location| {
        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)
            == Some(class_the_entry_type_names)
    })
}

/// 走读不往下读的孩子的一份副本（一条位置条目）与位置条目里的整单元校验和对不对得上：头按最短的单元（码 2，16 KiB）那么长读，
/// 头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上才算对不上
/// 只读、不判。
pub(super) fn copy_matches_its_location_checksum_at_the_length_its_header_names(
    reader: &dyn ImageReader,
    location: &PointerLocation,
) -> bool {
    let offset = location.slot * SLOT_BYTES;
    let length_of_the_class_in_the_header = reader
        .read(location.device, offset, node_bytes())
        .map(|header| MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]))
        .and_then(unit_bytes_of_the_class);
    let lengths_to_compare = match length_of_the_class_in_the_header {
        Some(unit_bytes) => vec![unit_bytes],
        None => vec![node_bytes(), data_unit_bytes()],
    };
    lengths_to_compare.into_iter().any(|unit_bytes| {
        reader
            .read(location.device, offset, unit_bytes)
            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum)
    })
}

/// inode 树一条内部条目的孩子两份都读不出（[`child_unit_class_other_than_the_entry_type_names`] 也没认出另一类）时，I-9.2 里不看孩子就判得了的那几小句，
/// 与墓碑树、谱系树读不出孩子时同判（`Walk::walk_the_entries_of_a_container_tree_node`；判决
/// `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行「inode 树读不出孩子时 I-9.2 不判」）：
/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段 2 判「子指针头部的出生树 == 身份引用的出生树」
/// （只比父条目里的两段 H3 H-B 那一行、第三节第 1 条），
/// 子头四段与身份引用相等、子容器的打包记录类型 == 2 那几小句要比孩子的头，读不出不判。
/// 类型段不认得的条目不走到这里：调用方不往下走它的孩子、判该条目损坏（与墓碑树、谱系树类型段不认得那一支同判），
/// 每一份副本只判 I-2.1（校验和与内容匹配），做法同 `Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into`。
/// 读不出照旧归 I-2.1 与走读失败（调用方记）。
pub(super) fn judge_an_inode_entry_whose_child_does_not_read_back(
    judgements: &mut Judgements,
    entry_type: RegisteredInodeInternalEntryType,
    identity: (u64, u16, u64, u64),
    birth_tree_in_the_child_pointer_head: u64,
) {
    match entry_type {
        RegisteredInodeInternalEntryType::LeafContainer => {
            let identity_birth_tree = identity.0;
            judgements.judge(
                "I-9.2",
                birth_tree_in_the_child_pointer_head == identity_birth_tree,
                || {
                    format!(
                        "inode 内部条目类型段 2：孩子两份都读不出（比孩子头的那几小句不判），子指针头部的出生树 {birth_tree_in_the_child_pointer_head} 不等于身份引用的出生树 {identity_birth_tree}"
                    )
                },
            );
        }
        RegisteredInodeInternalEntryType::InternalNode => {
            let (birth_tree, _, container, container_birth) = identity;
            let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;
            judgements.judge("I-9.2", identity_is_zero, || {
                format!(
                    "inode 内部条目类型段 0：孩子两份都读不出（类标签那一小句不判），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
                )
            });
        }
    }
}

/// 这一遍里读过的码 2 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的节点（解不开记 None）。
/// 候选集里几条根常指着同一个单元（暖机那几代都指着 mkfs 种的第 0 版树表，共享子树在每条根上都要算一遍），
/// 按认过位置的落点记一份（[`read_through_a_cache_keyed_by_the_located_placement`]），省掉重复的整单元 CRC——崩溃点重放每个崩溃状态都要跑这一遍。
pub(super) type IndexNodeCache = BTreeMap<(u32, u64, u32), Option<crate::IndexNodeView>>;

/// 读一个码 2 节点、不判：`position` 是树表单元与映射树节点（豁免）时按指针本身读，别的码 2 按这一版的中央映射认位置
/// （[`read_located_unit_without_judging`]）。
pub(super) fn read_index_node_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    position: PositionOfASecondaryRead<'_>,
    cache: &mut IndexNodeCache,
) -> Option<crate::IndexNodeView> {
    let located = position.locate(pointer, MappingKeyUnitClass::IndexNode);
    read_through_a_cache_keyed_by_the_located_placement(cache, &located, || {
        read_located_unit_without_judging(reader, &located, node_bytes())
            .and_then(|unit| index_node_view(&unit).ok())
    })
}

/// 这一遍要走多深。I-9.14（树表条目的诞生 txg 跨根不变） 只要每条根的树表；I-3.9（释放代落在停止引用它的那一格区间里）
/// 要这条根引用的全部落点，而那要把每棵树都走到叶——最新根那棵账里一条已释放记录都没有时，别的根就不必走那么深
/// （崩溃点重放上多数崩溃状态是这一类：最新的根还是暖机那几代，树表是空的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReferenceScanDepth {
    TreeTableOnly,
    EveryReferencedPlacement,
}

/// 从一条根走一遍，取它的树表条目；`depth` 是「走到叶」时再取它引用的全部落点与分配记录。只读不判。
pub(super) fn references_of_root(
    reader: &dyn ImageReader,
    record: &[u8],
    depth: ReferenceScanDepth,
    cache: &mut IndexNodeCache,
) -> RootReferences {
    let mut references = RootReferences {
        placements: BTreeSet::new(),
        tree_table_birth_txg: BTreeMap::new(),
        tree_table_placement: None,
        tree_table_reading: TreeTableReading::PointerAllZero,
        allocation_records: Vec::new(),
        depth,
        is_complete: true,
        central_mapping: std::rc::Rc::new(CentralMappingLocationsOfAVersion::default()),
    };
    // 这条根自己的中央映射先读出来：下面每棵树里豁免三类之外的指针按它认位置。
    // 只在「走到叶」那个深度上读（引用集合只在那个深度上作数）；读不全时是空表，照提示数，读不全由主走读判。
    if depth == ReferenceScanDepth::EveryReferencedPlacement {
        references.central_mapping =
            std::rc::Rc::new(CentralMappingLocationsOfAVersion::read_without_judging(
                reader,
                &parse_node_pointer(&record[256..342]),
                cache,
            ));
    }
    // 往下读子树的次级读法按同一份映射认位置（豁免三类之外的码 2 节点、码 3 单元、数据单元）。
    let central_mapping = std::rc::Rc::clone(&references.central_mapping);
    let instance_table = parse_node_pointer(&record[170..256]);
    references.note(&instance_table);
    // 实例表链上第 1 片起的落点（D18（块里携带什么信息） 已定项 11 的链）也是这条根引用的：写行那次发布整条链重写、逐片释放旧链，
    // 不数它们，旧链第 1 片起那几条已释放记录在 I-3.9 这一遍里找不到引用过它的根、被当成「见证它释放的根已不在候选集里」跳过。
    if depth == ReferenceScanDepth::EveryReferencedPlacement {
        note_instance_table_pages_after_the_first(reader, &instance_table, &mut references);
    }
    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，这一遍不另数
    // （主走读把它们记进 `references`，与树里指向同一单元的那条引用同 (设备, 起点槽, 跨度)，记成一个）。
    let mapping_root = parse_node_pointer(&record[256..342]);
    references.note(&mapping_root);
    // 树表 0 条那一版的分配记录树的根（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））：它的落点同样只有根记录指着，
    // 不数它，被抛弃根的影子账与 I-3.9 的引用集合就少一片。
    let allocation_record_tree_root = parse_node_pointer(&record[342..428]);
    references.note(&allocation_record_tree_root);
    // 那棵树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：根之下的节点只有父节点指着，走到叶那个深度上逐个数进来。
    // 它的记录就是这一版的账（树表 0 条的一版上被换下的单元记在这里，C512）：I-3.9 取最新根的账时要读到它们（代码审阅 6c 第 9 条）。
    if depth == ReferenceScanDepth::EveryReferencedPlacement
        && !allocation_record_tree_root.all_zero
    {
        match allocation_record_tree_without_judging(
            reader,
            &allocation_record_tree_root,
            &central_mapping,
            cache,
        ) {
            Some((node_pointers, records)) => {
                for pointer in &node_pointers {
                    references.note_located_by_the_central_mapping(
                        pointer,
                        MappingKeyUnitClass::IndexNode,
                    );
                }
                references.allocation_records.extend(records);
            }
            None => references.is_complete = false,
        }
    }
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
    let Some(tree_table) = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    ) else {
        references.is_complete = false;
        references.tree_table_reading = TreeTableReading::Unreadable;
        return references;
    };
    references.tree_table_reading = TreeTableReading::EveryEntryRead;
    // 销毁中的快照（码 14 带销毁中记号，连同头 ID 配上它的码 15）从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外句，
    // S9）：这条根不经它们的树引用任何落点。它的待放条目还钉着的
    // 那几个单元由中央映射树里留着的映射条目记进来（下面那一段），一批放掉一个就少一个——I-3.9 对从 deadlist 放掉的块
    // 取到的区间因此落在（写意图 txg, 最后一批 txg] 里（同一判决 S5 ③）；照数它们的树，那几个单元在最后一批之前的每条根上都还算引用、
    // 合法态红。
    let snapshot_trees_not_pinning_any_block =
        snapshot_trees_of_snapshots_being_destroyed(&tree_table.entries);
    for entry in &tree_table.entries {
        // 条目宽是节点头里的一个字段：头校验和过、载荷 CRC 过而条目宽被改窄的镜像也要能判（那由 I-1.7 / I-1.1 说话），
        // 这一遍只如实记「数不全」，不按短条目去解字段。
        if entry.len() < tree_table_entry_bytes() {
            references.is_complete = false;
            references.tree_table_reading = TreeTableReading::SomeEntryShorterThanTheFieldTable;
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
        if snapshot_trees_not_pinning_any_block.contains(&tree) {
            continue;
        }
        references.note_located_by_the_central_mapping(&tree_root, MappingKeyUnitClass::IndexNode);
        if depth == ReferenceScanDepth::TreeTableOnly {
            continue;
        }
        let Some(node) = read_index_node_without_judging(
            reader,
            &tree_root,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(&central_mapping),
            cache,
        ) else {
            references.is_complete = false;
            continue;
        };
        collect_tree_references(
            reader,
            &central_mapping,
            cache,
            &mut references,
            &tree_root,
            &node,
            read_u16(entry, 10),
        );
    }
    // 中央映射树根之下的节点（多层时，D8（核心索引结构） 已定项 11）：只有它们的父节点指着，不数它们，这条根的引用集合就少几片。
    // 叶里每条映射条目指的单元也数：被快照钉住的单元（进了 deadlist 的）映射条目留在中央映射树里（判决
    // `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 的 α），活头的树已不引用它，这一版仍占着它，放掉它的那一批同一次
    // 发布删它的映射条目。没有快照的一版映射里只有本版树里引用着的单元（`crates/singlefs-core/src/transaction.rs` 的
    // `TransactionOutput::mapped_units`），多数这一遍不多出一个落点。只在「走到叶」那个深度上数（引用集合只在那个深度上作数）。
    if depth == ReferenceScanDepth::EveryReferencedPlacement && !mapping_root.all_zero {
        match read_index_node_without_judging(
            reader,
            &mapping_root,
            PositionOfASecondaryRead::ThePointerItself,
            cache,
        ) {
            Some(mapping_root_node) => {
                note_every_node_and_every_mapped_unit_below_the_mapping_root(
                    reader,
                    cache,
                    &mut references,
                    &mapping_root_node,
                )
            }
            None => references.is_complete = false,
        }
    }
    references
}

/// 中央映射树根之下：内部节点下面的每个节点记进引用集合（同 [`note_every_node_below`]），叶里每条映射条目的两条位置条目也记进去
/// （key 27 之后是位置条目 14 × 2，D19（块指针的结构与宽度预算） 已定项 10）。key 宽不是 27、条目宽窄于字段表、孩子读不出、
/// 孩子的层级不是父层级减一，都如实记成数不全、不往下走（判定归主走读的 I-1.1 与 I-2.1）。递归的上界：每下一层层级减一，层级 0 停。
fn note_every_node_and_every_mapped_unit_below_the_mapping_root(
    reader: &dyn ImageReader,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    node: &crate::IndexNodeView,
) {
    let key_width = KEY_SCHEMA_MAPPING.width();
    if node.key_width != key_width {
        references.is_complete = false;
        return;
    }
    if node.level == 0 {
        if node.entry_width < mapping_entry_bytes() {
            references.is_complete = false;
            return;
        }
        for entry in &node.entries {
            references.note(&locations_of_a_mapping_entry(entry));
        }
        return;
    }
    if node.entry_width < internal_entry_bytes(KEY_SCHEMA_MAPPING) {
        references.is_complete = false;
        return;
    }
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        references.note(&child_pointer);
        let Some(child) = read_index_node_without_judging(
            reader,
            &child_pointer,
            PositionOfASecondaryRead::ThePointerItself,
            cache,
        ) else {
            references.is_complete = false;
            continue;
        };
        // 孩子的层级是盘上读来的一字节：拿父层级减一去比（父层级这里 > 0），不拿孩子的加一（255 加一在 u8 上溢出）。
        let parent_level_minus_one = node.level.checked_sub(1);
        if parent_level_minus_one != Some(child.level) {
            references.is_complete = false;
            continue;
        }
        note_every_node_and_every_mapped_unit_below_the_mapping_root(
            reader, cache, references, &child,
        );
    }
}

/// 一条中央映射叶条目（至少 55 字节）的两条位置条目，装成一条指针的样子（头部 50 与尾段 8 填零，只有位置条目作数）：
/// 与主走读 `walk_central_mapping_entries` 切位置条目同一个读法。
pub(super) fn locations_of_a_mapping_entry(entry: &[u8]) -> PointerView {
    let key_width = KEY_SCHEMA_MAPPING.width();
    let locations_bytes = &entry[key_width..mapping_entry_bytes()];
    let pointer_bytes: Vec<u8> = [
        vec![0u8; 50],
        locations_bytes.to_vec(),
        vec![0u8; node_pointer_bytes() - 50 - locations_bytes.len()],
    ]
    .concat();
    parse_node_pointer(&pointer_bytes)
}

/// 沿实例表链（根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片，D18（块里携带什么信息） 已定项 11）把第 1 片起每一片的
/// 指针记进这条根的引用集合。只读不判：一片读不出、解不开、身份不是 (0, 4, 片序号, 0)、I-3.14 不成立（链指针记录之前有不是行的记录、
/// 行的 flags 不是 0、最后一条不是合法的链指针记录、它说有下一片而位置指针头部有一段不是第一版的取值），都只如实记成数不全
/// （判定归主走读：I-2.1、I-1.1、I-3.14）。第五形那条指针不记进引用集合，与主走读不跟随它同一个读法（[`chain_record_view`]）。
/// 迭代上界：第 k 片要求容器号是 k，每一轮读的是盘上一个不同的单元；跨轮带的只有下一片的指针；提前出口是走到最后一片与上面那几样读不下去。
fn note_instance_table_pages_after_the_first(
    reader: &dyn ImageReader,
    first_page_pointer: &PointerView,
    references: &mut RootReferences,
) {
    let pages = instance_table_pages_after_the_first_without_judging(reader, first_page_pointer);
    for page_pointer in &pages.page_pointers {
        references.note(page_pointer);
    }
    if !pages.is_complete {
        references.is_complete = false;
    }
}

/// 实例表链上第 1 片起每一片的指针（[`instance_table_pages_after_the_first_without_judging`] 交回的）：链没读到末片时 `is_complete` 是假。
pub(super) struct InstanceTablePagesAfterTheFirst {
    pub(super) page_pointers: Vec<PointerView>,
    pub(super) is_complete: bool,
}

/// 沿实例表链取第 1 片起每一片的指针，只读不判，读法与 [`note_instance_table_pages_after_the_first`] 那一段说的一样（它就是经这里取的）；
/// I-3.15（未释放记录罩住的单元由映射指在这一槽） 认「被父指针直接指着」的实例表各片也读它。
pub(super) fn instance_table_pages_after_the_first_without_judging(
    reader: &dyn ImageReader,
    first_page_pointer: &PointerView,
) -> InstanceTablePagesAfterTheFirst {
    let mut pages = InstanceTablePagesAfterTheFirst {
        page_pointers: Vec::new(),
        is_complete: true,
    };
    let mut pointer = *first_page_pointer;
    let mut page_index: u64 = 0;
    loop {
        let Some(page) = read_unit_without_judging(reader, &pointer, data_unit_bytes())
            .and_then(|unit| packed_unit_view(&unit).ok())
        else {
            pages.is_complete = false;
            return pages;
        };
        let identity = (
            page.birth_tree,
            page.record_type,
            page.container,
            page.container_birth,
        );
        if identity != (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0) {
            pages.is_complete = false;
            return pages;
        }
        // I-3.14 不成立的一片解不开（D18（块里携带什么信息） 已定项 11）：链指针记录之前有不是行的记录
        // （kind 不是 0 或 1、片中间有链指针记录）或行的 flags 不是 0，与主走读、只读不判的读法同一个读法，数不全。
        let records_before_the_last = page
            .records
            .split_last()
            .map_or(&[][..], |(_, before)| before);
        if records_before_the_last.iter().any(|record| {
            record.len() != INSTANCE_TABLE_RECORD_BYTES
                || InstanceTableRecordKind::of_record(record) != InstanceTableRecordKind::Row
                || record[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION
        }) {
            pages.is_complete = false;
            return pages;
        }
        match page
            .records
            .last()
            .map_or(ChainRecordView::Malformed, |last| chain_record_view(last))
        {
            ChainRecordView::LastPage => return pages,
            ChainRecordView::NextPage(next_page) => {
                pages.page_pointers.push(next_page);
                pointer = next_page;
                page_index += 1;
            }
            ChainRecordView::Malformed => {
                pages.is_complete = false;
                return pages;
            }
        }
    }
}

/// 多层码 2 树（记账树、中央映射树）一个节点下面的全部节点都记进引用集合（它们各占一个槽）：按内部条目里的子指针逐层往下，
/// 条目宽窄于 key 宽 + 86、孩子读不出、孩子的层级不是父层级减一，都如实记成数不全、不往下走（那几样由主走读的 I-1.1 / I-1.10 说话）。
/// 递归的上界：每下一层层级减一。
fn note_every_node_below(
    reader: &dyn ImageReader,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    node: &crate::IndexNodeView,
    schema: crate::KeySchema,
) {
    if node.level == 0 {
        return;
    }
    let key_width = schema.width();
    if node.key_width != key_width || node.entry_width < internal_entry_bytes(schema) {
        references.is_complete = false;
        return;
    }
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        references
            .note_located_by_the_central_mapping(&child_pointer, MappingKeyUnitClass::IndexNode);
        let Some(child) = read_index_node_without_judging(
            reader,
            &child_pointer,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            cache,
        ) else {
            references.is_complete = false;
            continue;
        };
        // 孩子的层级是盘上读来的一字节：拿父层级减一去比（父层级这里 > 0），不拿孩子的加一——255 加一在 u8 上溢出（代码审阅 6c 第 6 条）。
        if node.level.checked_sub(1) != Some(child.level) {
            references.is_complete = false;
            continue;
        }
        note_every_node_below(reader, central_mapping, cache, references, &child, schema);
    }
}

/// inode 树一个内部节点下面的孩子都记进引用集合：类型段 2 的叶容器记它的落点；类型段 0 的内部节点（I-9.2（条目身份与子头相符））
/// 记它的落点再往下数它的孩子。读不出、层级不是父层级减一、层级 0、条目窄于字段表、类型段不认得，都如实记成数不全（判定归主走读）。
/// 递归的上界：每下一层层级减一，层级 0 不往下走。
fn note_every_inode_child_below(
    reader: &dyn ImageReader,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    node: &crate::IndexNodeView,
) {
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {
                references.note_located_by_the_central_mapping(
                    &child_pointer,
                    MappingKeyUnitClass::PackedRecordUnit,
                );
            }
            InodeInternalEntryType::InternalNode => {
                references.note_located_by_the_central_mapping(
                    &child_pointer,
                    MappingKeyUnitClass::IndexNode,
                );
                let Some(child) = read_index_node_without_judging(
                    reader,
                    &child_pointer,
                    PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
                    cache,
                ) else {
                    references.is_complete = false;
                    continue;
                };
                if node.level.checked_sub(1) != Some(child.level)
                    || child.level == 0
                    || child.entry_width < inode_internal_entry_bytes()
                {
                    references.is_complete = false;
                    continue;
                }
                note_every_inode_child_below(reader, central_mapping, cache, references, &child);
            }
            InodeInternalEntryType::Unregistered(_) => {
                references.note(&child_pointer);
                references.is_complete = false;
            }
        }
    }
}

/// 一棵分配记录树整棵读、不判（按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：交回每个节点的指针（根在内）与全部记录。
/// 节点读不出、层级不是它的位置规定的那一层、内部条目指不到一个合法的孩子、条目窄于字段表：交回 `None`（数不全；判定归主走读的
/// I-1.1 / I-1.10 / I-2.1）。迭代上界：每个节点至多进一次（按第一条位置条目记过的不再读），每下一层层级减一。
pub(super) fn allocation_record_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Option<(Vec<PointerView>, Vec<AllocationRecordView>)> {
    let pool_devices = reader.devices();
    let device_slots: Vec<u64> = pool_devices
        .iter()
        .map(|device| reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)
        .collect();
    let root_level = allocation_record_tree_root_level(&device_slots)?;
    let internal_entry_bytes =
        usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96");
    let key_width = KEY_SCHEMA_ALLOCATION.width();
    let mut node_pointers = Vec::new();
    let mut records = Vec::new();
    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut pending = vec![(*root_pointer, AllocationRecordTreeCell::Root, root_level)];
    while let Some((pointer, cell, level)) = pending.pop() {
        if pointer.all_zero
            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
        {
            return None;
        }
        node_pointers.push(pointer);
        let node = read_index_node_without_judging(
            reader,
            &pointer,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            cache,
        )?;
        if node.level != level || node.key_width != key_width {
            return None;
        }
        if level == 0 {
            if node.entry_width < allocation_record_bytes() {
                return None;
            }
            records.extend(
                node.entries
                    .iter()
                    .map(|entry| parse_allocation_record(entry)),
            );
            continue;
        }
        if node.entry_width < internal_entry_bytes {
            return None;
        }
        for entry in &node.entries {
            let child = allocation_record_tree_child_of_entry_key(
                cell,
                level,
                &entry[..key_width],
                &pool_devices,
            )?;
            pending.push((
                parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                child,
                level - 1,
            ));
        }
    }
    Some((node_pointers, records))
}

/// 一棵 extent 树整棵读、不判（上段与每个文件的下段，D8（核心索引结构） 已定项 14）：交回每个节点的指针（根在内）与全部数据指针
/// （下段叶的 extent 叶记录与上段叶条目里内联的那一个）。读不下去的同 [`allocation_record_tree_without_judging`] 交回 `None`。
pub(super) fn extent_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Option<(Vec<PointerView>, Vec<PointerView>)> {
    extent_tree_with_the_inode_of_each_data_pointer_without_judging(
        reader,
        root_pointer,
        central_mapping,
        cache,
    )
    .map(|extent_tree| {
        (
            extent_tree.node_pointers,
            extent_tree
                .data_pointers_of_inodes
                .into_iter()
                .map(|data_pointer_of_an_inode| data_pointer_of_an_inode.pointer)
                .collect(),
        )
    })
}

/// [`extent_tree_with_the_inode_of_each_data_pointer_without_judging`] 读出的一棵 extent 树：每个节点的指针（根在内），
/// 每个数据指针连同它属于哪个 inode 号、落在文件哪个字节偏移，与上段叶条目对每个 inode 号声明了什么（标签 0 到 4）。
pub(super) struct ExtentTreeReadWithoutJudging {
    node_pointers: Vec<PointerView>,
    pub(super) data_pointers_of_inodes: Vec<DataPointerOfAnInode>,
    pub(super) declarations_of_the_upper_leaf_entries:
        BTreeMap<u64, DataUnitsDeclaredByTheUpperLeafEntry>,
}

/// 上段叶条目对一个 inode 号声明的数据单元：
/// 标签 0 是一个单元都没有；标签 1 到 4 带单元，标签 3、4 声明最后一个存在的单元之后到 size 是洞。同一个号两条上段叶条目时
/// 后读到的那条盖掉先读到的（key 严格递增由主走读的 I-1.1 判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DataUnitsDeclaredByTheUpperLeafEntry {
    NoDataUnit,
    WithDataUnits(TailAfterTheLastDataUnit),
}

/// extent 树里的一个数据指针，连同它所在 extent key (0, inode, 偏移) 的后两段：上段叶条目内联的那一个偏移是 0
/// （走读 `walk_extent_data_pointer` 同一个读法），下段叶的 extent 叶记录取记录 key 偏移 16 那 8 字节。
pub(super) struct DataPointerOfAnInode {
    pub(super) inode: u64,
    pub(super) offset_in_bytes: u64,
    pub(super) pointer: PointerView,
}

/// [`extent_tree_without_judging`] 那一遍，每个数据指针带上它属于哪个 inode 号（上段叶条目的 inode 号；下段叶记录属于引它那条上段条目的号），
/// 每个 inode 号带上上段叶条目的标签声明了什么：删除态扫描按号数数据单元、对标签（I-4.7（删除中 inode 有账）、I-4.6（中间态其余不变量成立）、
/// I-4.10（截断中间态有账）、I-9.38（数据单元声明长度对 size））。
pub(super) fn extent_tree_with_the_inode_of_each_data_pointer_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Option<ExtentTreeReadWithoutJudging> {
    // 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。
    enum Pending {
        Upper(PointerView, Option<(u8, u64)>),
        Lower(PointerView, u64, Option<(u8, u64)>),
    }
    let internal_entry_bytes = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");
    let upper_leaf_entry_bytes = usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113");
    let key_width = KEY_SCHEMA_EXTENT.width();
    let mut node_pointers = Vec::new();
    let mut data_pointers = Vec::new();
    let mut declarations_of_the_upper_leaf_entries = BTreeMap::new();
    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut pending = vec![Pending::Upper(*root_pointer, None)];
    while let Some(next) = pending.pop() {
        let (pointer, lower_of_inode, position) = match next {
            Pending::Upper(pointer, position) => (pointer, None, position),
            Pending::Lower(pointer, inode, position) => (pointer, Some(inode), position),
        };
        if pointer.all_zero
            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
        {
            return None;
        }
        node_pointers.push(pointer);
        let node = read_index_node_without_judging(
            reader,
            &pointer,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            cache,
        )?;
        let (level, index) = position.unwrap_or((node.level, 0));
        if node.level != level || node.key_width != key_width {
            return None;
        }
        match (lower_of_inode, level) {
            (None, 0) => {
                if node.entry_width < upper_leaf_entry_bytes {
                    return None;
                }
                for entry in &node.entries {
                    match extent_upper_leaf_entry_view(entry) {
                        ExtentUpperLeafEntryView::NoDataUnit { inode } => {
                            declarations_of_the_upper_leaf_entries
                                .insert(inode, DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit);
                        }
                        ExtentUpperLeafEntryView::LowerSegmentRoot {
                            inode,
                            pointer: lower_root_pointer,
                            tail,
                        } => {
                            declarations_of_the_upper_leaf_entries.insert(
                                inode,
                                DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(tail),
                            );
                            pending.push(Pending::Lower(
                                parse_node_pointer(&lower_root_pointer),
                                inode,
                                None,
                            ));
                        }
                        ExtentUpperLeafEntryView::InlineDataUnit {
                            inode,
                            pointer: inline_data_pointer,
                            tail,
                        } => {
                            declarations_of_the_upper_leaf_entries.insert(
                                inode,
                                DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(tail),
                            );
                            data_pointers.push(DataPointerOfAnInode {
                                inode,
                                offset_in_bytes: 0,
                                pointer: parse_data_pointer(&inline_data_pointer),
                            });
                        }
                        ExtentUpperLeafEntryView::Malformed => return None,
                    }
                }
            }
            (Some(inode), 0) => {
                if node.entry_width < extent_leaf_record_bytes() {
                    return None;
                }
                data_pointers.extend(node.entries.iter().map(|record| DataPointerOfAnInode {
                    inode,
                    offset_in_bytes: read_u64(record, 16),
                    pointer: parse_data_pointer(&record[24..112]),
                }));
            }
            (None, _) => {
                if node.entry_width < internal_entry_bytes {
                    return None;
                }
                for entry in &node.entries {
                    let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])?;
                    pending.push(Pending::Upper(
                        parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                        Some(child),
                    ));
                }
            }
            (Some(inode), _) => {
                if node.entry_width < internal_entry_bytes {
                    return None;
                }
                for entry in &node.entries {
                    let child =
                        extent_lower_child_of_entry_key(inode, level, index, &entry[..key_width])?;
                    pending.push(Pending::Lower(
                        parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                        inode,
                        Some(child),
                    ));
                }
            }
        }
    }
    Some(ExtentTreeReadWithoutJudging {
        node_pointers,
        data_pointers_of_inodes: data_pointers,
        declarations_of_the_upper_leaf_entries,
    })
}

/// 一棵树的根节点下面还引用了什么：extent 树与分配记录树整棵按位置读（每个节点、数据指针、分配记录）、
/// 「inode 树内部条目的子指针」与记账树根之下的全部节点；条目格式没有条款的树走不下去，如实记成数不全。
fn collect_tree_references(
    reader: &dyn ImageReader,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    tree_root: &PointerView,
    node: &crate::IndexNodeView,
    tree_kind_code: u16,
) {
    // 条目宽窄于字段表的（节点头里那个字段被改过）不按短条目去解，如实记成数不全：宽度本身由 I-1.7 / I-1.1 说话。
    match TreeKindForReferenceScan::of(tree_kind_code) {
        // 快照的 extent 根与活头的 extent 树同一个数法（销毁中的快照的两条由调用方跳过，[`snapshot_trees_of_snapshots_being_destroyed`]）。
        TreeKindForReferenceScan::Extent | TreeKindForReferenceScan::SnapshotExtentRoot => {
            match extent_tree_without_judging(reader, tree_root, central_mapping, cache) {
                Some((node_pointers, data_pointers)) => {
                    for pointer in &node_pointers {
                        references.note_located_by_the_central_mapping(
                            pointer,
                            MappingKeyUnitClass::IndexNode,
                        );
                    }
                    for pointer in &data_pointers {
                        references.note_located_by_the_central_mapping(
                            pointer,
                            MappingKeyUnitClass::DataUnit,
                        );
                    }
                }
                None => references.is_complete = false,
            }
        }
        // 快照的 inode 根与活头的 inode 树同一个数法。
        TreeKindForReferenceScan::Inode | TreeKindForReferenceScan::SnapshotInodeRoot => {
            if node.level > 0 && node.entry_width >= inode_internal_entry_bytes() {
                note_every_inode_child_below(reader, central_mapping, cache, references, node);
            } else {
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::Allocation => {
            match allocation_record_tree_without_judging(reader, tree_root, central_mapping, cache)
            {
                Some((node_pointers, records)) => {
                    for pointer in &node_pointers {
                        references.note_located_by_the_central_mapping(
                            pointer,
                            MappingKeyUnitClass::IndexNode,
                        );
                    }
                    references.allocation_records.extend(records);
                }
                None => references.is_complete = false,
            }
        }
        TreeKindForReferenceScan::Accounting => {
            // 记账行不引用别的单元；多层时根之下的节点各占一个槽（D8（核心索引结构） 已定项 11：内部条目 = key 22 + 子指针 86）。
            note_every_node_below(
                reader,
                central_mapping,
                cache,
                references,
                node,
                KEY_SCHEMA_ACCOUNTING,
            );
        }
        // `deleted_inodes` 根的条目只是号，不引用别的单元（根自己由调用方记）。
        TreeKindForReferenceScan::DeletedInodes => {}
        // deadlist 的条目只是 key（被钉单元的中央映射 key），不引用别的单元；多层时根之下的节点各占一个槽。被钉的单元由中央映射树里
        // 留着的那条映射条目记进引用（[`note_every_node_and_every_mapped_unit_below_the_mapping_root`]）。
        TreeKindForReferenceScan::Deadlist => {
            note_every_node_below(
                reader,
                central_mapping,
                cache,
                references,
                node,
                KEY_SCHEMA_DEADLIST,
            );
        }
        // logged_ops 的条目是意图，不引用别的单元；多层时根之下的节点各占一个槽。
        TreeKindForReferenceScan::LoggedOperations => {
            note_every_node_below(
                reader,
                central_mapping,
                cache,
                references,
                node,
                KEY_SCHEMA_LOGGED_OPERATIONS,
            );
        }
        TreeKindForReferenceScan::ContainerTree => {
            if node.level == 1 && node.entry_width >= inode_internal_entry_bytes() {
                for entry in &node.entries {
                    references.note_located_by_the_central_mapping(
                        &parse_node_pointer(&entry[34..120]),
                        MappingKeyUnitClass::PackedRecordUnit,
                    );
                }
            } else {
                // 根长高之后（层级 > 1）的引用集合这一遍还不往下数，如实记成数不全。
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => references.is_complete = false,
    }
}

#[cfg(test)]
mod tests;
