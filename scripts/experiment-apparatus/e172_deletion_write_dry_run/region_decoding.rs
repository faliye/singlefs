//! 字节的覆盖图、区域的各种记录结构，以及按区域种类逐单元与整区域的解码。

use super::*;

// ---------------------------------------------------------------------------------------------
// 覆盖图：每个字节归到哪一类（跑前登记第一节 1.2「第 2 行的登记按位置读」）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ByteClass {
    Unclaimed,
    LayoutThreeCell,
    LayoutOneCell,
    ConstantZeroPadding,
}

#[derive(Clone, Debug)]
pub(super) struct CoverageMap {
    pub(super) classes: Vec<ByteClass>,
    pub(super) double_claimed_bytes: u64,
}

impl CoverageMap {
    pub(super) fn new(length: usize) -> Self {
        CoverageMap { classes: vec![ByteClass::Unclaimed; length], double_claimed_bytes: 0 }
    }
    pub(super) fn claim(&mut self, start: usize, end: usize, class: ByteClass) {
        let clamped_end = end.min(self.classes.len());
        for byte_class in &mut self.classes[start.min(clamped_end)..clamped_end] {
            if *byte_class != ByteClass::Unclaimed {
                self.double_claimed_bytes += 1;
            }
            *byte_class = class;
        }
    }
    fn reclassify(&mut self, start: usize, end: usize, class: ByteClass) {
        let clamped_end = end.min(self.classes.len());
        for byte_class in &mut self.classes[start.min(clamped_end)..clamped_end] {
            *byte_class = class;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 指针与位置条目（layout/01 第 200–216 行）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct LocationEntry {
    pub(super) device: u32,
    pub(super) slot: u64,
    pub(super) checksum: u64,
}

pub(super) fn location_at(bytes: &[u8], offset: usize) -> LocationEntry {
    LocationEntry {
        device: u32::try_from(read_u32(bytes, offset)).expect("4 字节装得进 u32"),
        slot: read_unsigned_48_bit(bytes, offset + 4),
        checksum: read_u32(bytes, offset + 10),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BlockPointer {
    pub(super) is_all_zero: bool,
    pub(super) birth_tree: u64,
    birth_txg: u64,
    pub(super) locations: Vec<LocationEntry>,
    /// 指向码 2 / 码 3：实例代号 4 + 出生序号 4；指向码 1：写序 10（第 212 行）。
    key_tail: Vec<u8>,
}

/// `pointer_bytes` 是 86（节点指针）或 88（数据指针）。全零的指针（根指针置零）不解位置条目。
pub(super) fn block_pointer_of(pointer_bytes: &[u8]) -> BlockPointer {
    let is_all_zero = is_all_zero(pointer_bytes);
    let locations = if is_all_zero {
        Vec::new()
    } else {
        (0..2).map(|copy| location_at(pointer_bytes, POINTER_FIRST_LOCATION_OFFSET + copy * LOCATION_ENTRY_BYTES)).collect()
    };
    BlockPointer {
        is_all_zero,
        birth_tree: read_u64(pointer_bytes, 34),
        birth_txg: read_u64(pointer_bytes, 42),
        locations,
        key_tail: pointer_bytes[78..].to_vec(),
    }
}

// ---------------------------------------------------------------------------------------------
// 码 2 索引节点与码 3 打包记录单元。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct IndexNode {
    pub(super) tree_identifier: u64,
    pub(super) level: u8,
    pub(super) key_width: usize,
    pub(super) key_range_low: Vec<u8>,
    pub(super) key_range_high: Vec<u8>,
    pub(super) birth_txg: u64,
    pub(super) entry_count: usize,
    pub(super) entry_width: usize,
    pub(super) entries: Vec<Vec<u8>>,
}

pub(super) fn index_node_header_bytes(key_width: usize) -> usize {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + 2 * key_width
}

#[derive(Clone, Debug)]
pub(super) struct PackedContainer {
    pub(super) birth_tree: u64,
    pub(super) packed_record_type: u64,
    pub(super) container_number: u64,
    pub(super) container_birth: u64,
    pub(super) record_count: usize,
    pub(super) record_width: usize,
    pub(super) birth_txg: u64,
    pub(super) records: Vec<Vec<u8>>,
}

/// 共同前缀的头校验和：CRC32C 4 + 28 零，覆盖 `[0, plaintext_header_end)`，自身按零参与（layout/01 第 180 行；D18 已定项 18）。
fn header_checksum_holds(bytes: &[u8], plaintext_header_end: usize) -> bool {
    if bytes.len() < plaintext_header_end {
        return false;
    }
    let mut covered = bytes[..plaintext_header_end].to_vec();
    for byte in &mut covered[UNIT_HEADER_CHECKSUM_OFFSET..UNIT_COMMON_PREFIX_BYTES] {
        *byte = 0;
    }
    castagnoli_crc32(&covered) == u32::try_from(read_u32(bytes, UNIT_HEADER_CHECKSUM_OFFSET)).expect("4 字节")
        && is_all_zero(&bytes[UNIT_HEADER_CHECKSUM_OFFSET + 4..UNIT_COMMON_PREFIX_BYTES])
}

pub(super) fn crc_matches(bytes: &[u8], stored: u64) -> bool {
    u64::from(castagnoli_crc32(bytes)) == stored
}

// ---------------------------------------------------------------------------------------------
// 系统配置槽、根记录、journal 记录。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct SystemConfiguration {
    pub(super) generation: u64,
    pub(super) journal_tail: u64,
    pub(super) instance: u64,
    pub(super) time_threshold_milliseconds: u64,
}

#[derive(Clone, Debug)]
pub(super) struct RootRecord {
    pub(super) txg: u64,
    pub(super) watermark: u64,
    pub(super) floor: u64,
    pub(super) tree_table_pointer: BlockPointer,
    pub(super) instance_table_pointer: BlockPointer,
    pub(super) mapping_root_pointer: BlockPointer,
    pub(super) allocation_root_pointer: BlockPointer,
}

#[derive(Clone, Debug)]
pub(super) struct JournalRecord {
    pub(super) instance: u64,
    pub(super) counter: u64,
    pub(super) txg: u64,
    pub(super) watermark: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RegionKind {
    SystemConfigurationSlot,
    RootRecordSlot,
    JournalRecord,
    Unit,
    Unrecognized,
}

impl RegionKind {
    pub(super) fn step_kind_name(self) -> &'static str {
        match self {
            RegionKind::SystemConfigurationSlot => "system_configuration_slot",
            RegionKind::RootRecordSlot => "root_record_fua",
            RegionKind::JournalRecord => "journal_record",
            RegionKind::Unit => "unit_write",
            RegionKind::Unrecognized => "unrecognized",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum DecodedBody {
    SystemConfiguration(SystemConfiguration),
    RootRecord(RootRecord),
    JournalRecord(JournalRecord),
    IndexNode(IndexNode),
    PackedContainer(PackedContainer),
    DataUnit { birth_txg: u64 },
    Unrecognized,
}

#[derive(Clone, Debug)]
pub(super) struct DecodedRegion {
    pub(super) step: usize,
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) bytes: Vec<u8>,
    pub(super) region_kind: RegionKind,
    pub(super) body: DecodedBody,
    pub(super) structure_failures: Vec<String>,
    pub(super) coverage: CoverageMap,
    pub(super) locations_inside: Vec<LocationEntry>,
}

impl DecodedRegion {
    pub(super) fn slot(&self) -> u64 {
        self.offset / SLOT_BYTES
    }
    pub(super) fn unit_type_tag(&self) -> u8 {
        match &self.body {
            DecodedBody::IndexNode(_) => 2,
            DecodedBody::PackedContainer(_) => 3,
            DecodedBody::DataUnit { .. } => 1,
            DecodedBody::SystemConfiguration(_)
            | DecodedBody::RootRecord(_)
            | DecodedBody::JournalRecord(_)
            | DecodedBody::Unrecognized => 0,
        }
    }
    /// 单元头里的诞生代号、journal 记录与根记录的 checkpoint_txg（跑前登记第六节 R8.b 的归属键）。
    pub(super) fn birth_or_checkpoint_txg(&self) -> Option<u64> {
        match &self.body {
            DecodedBody::IndexNode(node) => Some(node.birth_txg),
            DecodedBody::PackedContainer(container) => Some(container.birth_txg),
            DecodedBody::DataUnit { birth_txg } => Some(*birth_txg),
            DecodedBody::JournalRecord(record) => Some(record.txg),
            DecodedBody::RootRecord(record) => Some(record.txg),
            DecodedBody::SystemConfiguration(_) | DecodedBody::Unrecognized => None,
        }
    }
}

fn root_ring_position(offset: u64) -> Option<(u64, u64)> {
    for region in 0..3u64 {
        let region_start = ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE_BYTES;
        if offset >= region_start && offset < region_start + ROOT_RING_SLOTS_PER_REGION * ROOT_RING_SLOT_SPACING_BYTES {
            let inside = offset - region_start;
            if inside.is_multiple_of(ROOT_RING_SLOT_SPACING_BYTES) {
                return Some((region, inside / ROOT_RING_SLOT_SPACING_BYTES));
            }
        }
    }
    None
}

/// 只按偏移、长度与 magic 判种类，不读导出的 `kind=`（FUA 另由 `is_force_unit_access` 带）。
pub(super) fn region_kind_of(write: &RegionWrite) -> RegionKind {
    let length = write.bytes.len();
    let magic = write.bytes.get(0..4);
    if (write.offset == 0 || write.offset == u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES))
        && length == SYSTEM_CONFIGURATION_SLOT_BYTES
        && magic == Some(&SYSTEM_CONFIGURATION_MAGIC[..])
    {
        return RegionKind::SystemConfigurationSlot;
    }
    if root_ring_position(write.offset).is_some() && length == ROOT_RECORD_SLOT_BYTES && magic == Some(&ROOT_RECORD_MAGIC[..]) {
        return RegionKind::RootRecordSlot;
    }
    if write.offset >= JOURNAL_RING_START_BYTES
        && write.offset < JOURNAL_RING_START_BYTES + JOURNAL_RING_BYTES
        && (write.offset - JOURNAL_RING_START_BYTES).is_multiple_of(u64_of(JOURNAL_RECORD_BYTES))
        && length == JOURNAL_RECORD_BYTES
        && magic == Some(&JOURNAL_MAGIC[..])
    {
        return RegionKind::JournalRecord;
    }
    if write.offset >= UNIT_AREA_START_SLOT * SLOT_BYTES && write.offset.is_multiple_of(SLOT_BYTES) && magic == Some(&UNIT_MAGIC[..]) {
        return RegionKind::Unit;
    }
    RegionKind::Unrecognized
}

/// 整块自证校验和：CRC32C 4 + 28 零，覆盖整块、自身按零参与（系统配置 layout/01 第 99 行、根记录第 377 行、journal 第 347 行）。
pub(super) fn whole_slot_checksum_holds(bytes: &[u8], checksum_offset: usize) -> bool {
    let mut covered = bytes.to_vec();
    for byte in &mut covered[checksum_offset..checksum_offset + 32] {
        *byte = 0;
    }
    crc_matches(&covered, read_u32(bytes, checksum_offset)) && is_all_zero(&bytes[checksum_offset + 4..checksum_offset + 32])
}

pub(super) fn decode_system_configuration(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>) -> SystemConfiguration {
    // layout/01 第 158 行：槽世代号 147、整槽校验和 155、本盘设备号 139、T_time 433、journal tail 469、实例代号 477、回退下界 F 481。
    if !whole_slot_checksum_holds(bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET) {
        failures.push("系统配置整槽校验和不对".to_string());
    }
    coverage.claim(0, SYSTEM_CONFIGURATION_FIELD_BYTES, ByteClass::LayoutOneCell);
    coverage.claim(SYSTEM_CONFIGURATION_FIELD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, ByteClass::ConstantZeroPadding);
    SystemConfiguration {
        generation: read_u64(bytes, 147),
        journal_tail: read_u64(bytes, 469),
        instance: read_u32(bytes, 477),
        time_threshold_milliseconds: read_u32(bytes, SYSTEM_CONFIGURATION_TIME_THRESHOLD_OFFSET),
    }
}

fn decode_root_record(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>, locations: &mut Vec<LocationEntry>) -> RootRecord {
    // layout/01 第 372–388 行按行累加：flags 20、实例代号 24、checkpoint_txg 28、树表单元指针 36、树 ID 水位 122、回退下界 F 130、
    // 自证校验和 138、实例表单元指针 170、中央映射树根指针 256、分配记录树根指针 342（layout/02 第 15 行「偏移 342」同值）、
    // 回收门槛 t 428、钉槽 p 436、算法类型 437、nonce 438、MAC 450，到 466 为止。
    if !whole_slot_checksum_holds(bytes, 138) {
        failures.push("根记录自证校验和不对".to_string());
    }
    coverage.claim(0, ROOT_RECORD_FIELD_BYTES, ByteClass::LayoutOneCell);
    coverage.claim(ROOT_RECORD_FIELD_BYTES, ROOT_RECORD_SLOT_BYTES, ByteClass::ConstantZeroPadding);
    let pointer = |offset: usize| block_pointer_of(&bytes[offset..offset + NODE_POINTER_BYTES]);
    let record = RootRecord {
        txg: read_u64(bytes, 28),
        watermark: read_u64(bytes, 122),
        floor: read_u64(bytes, 130),
        tree_table_pointer: pointer(36),
        instance_table_pointer: pointer(170),
        mapping_root_pointer: pointer(256),
        allocation_root_pointer: pointer(342),
    };
    for block_pointer in [&record.tree_table_pointer, &record.instance_table_pointer, &record.mapping_root_pointer, &record.allocation_root_pointer] {
        locations.extend(block_pointer.locations.iter().copied());
    }
    record
}

fn decode_journal_record(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>, locations: &mut Vec<LocationEntry>) -> JournalRecord {
    // layout/01 第 362 行：类型 4、记录长度 8、点名项数 12、jsn 16（实例代号 16、计数器 20）、checkpoint_txg 26、头校验和 46、
    // 载荷校验和 95、新根段 99（树表 99、映射根 185、水位 271、F 279、实例表 287、分配记录树根 373、卸载记号 459、回收门槛 t 463、钉槽 p 471）、
    // fsid 472、MAC 480，到 496 为止；点名项第 364 行。
    if !whole_slot_checksum_holds(bytes, 46) {
        failures.push("journal 记录头校验和不对".to_string());
    }
    let item_count = usize_of(read_u32(bytes, 12));
    let items_end = JOURNAL_HEADER_BYTES + item_count * JOURNAL_NAMED_ITEM_BYTES;
    if read_u32(bytes, 8) != u64_of(JOURNAL_RECORD_BYTES) || items_end > JOURNAL_RECORD_BYTES {
        failures.push(format!("journal 记录长度 {} 或点名项数 {item_count} 越界", read_u32(bytes, 8)));
        return JournalRecord { instance: 0, counter: 0, txg: 0, watermark: 0 };
    }
    if !crc_matches(&bytes[JOURNAL_HEADER_BYTES..items_end], read_u32(bytes, 95)) {
        failures.push("journal 载荷校验和不对".to_string());
    }
    coverage.claim(0, JOURNAL_HEADER_BYTES, ByteClass::LayoutOneCell);
    coverage.claim(JOURNAL_HEADER_BYTES, items_end, ByteClass::LayoutOneCell);
    coverage.claim(items_end, JOURNAL_RECORD_BYTES, ByteClass::ConstantZeroPadding);
    for offset in [99usize, 185, 287, 373] {
        locations.extend(block_pointer_of(&bytes[offset..offset + NODE_POINTER_BYTES]).locations);
    }
    for item_index in 0..item_count {
        let item_start = JOURNAL_HEADER_BYTES + item_index * JOURNAL_NAMED_ITEM_BYTES;
        for copy in 0..2 {
            locations.push(location_at(bytes, item_start + copy * LOCATION_ENTRY_BYTES));
        }
    }
    JournalRecord {
        instance: read_u32(bytes, 16),
        counter: read_unsigned_48_bit(bytes, 20),
        txg: read_u64(bytes, 26),
        watermark: read_u64(bytes, 271),
    }
}

/// 码 2 条目按树分：认得的条目整条归格（layout/01 或 layout/03），认不得的留 `Unclaimed`（第 2 行的没登记）。
pub(super) fn claim_index_node_entries(node: &IndexNode, entries_offset: usize, coverage: &mut CoverageMap, locations: &mut Vec<LocationEntry>) {
    for (entry_index, entry) in node.entries.iter().enumerate() {
        let start = entries_offset + entry_index * node.entry_width;
        let end = start + node.entry_width;
        let mut pointer_at = |offset: usize, width: usize| locations.extend(block_pointer_of(&entry[offset..offset + width]).locations);
        match (node.tree_identifier, node.entry_width) {
            (EXTENT_TREE, EXTENT_UPPER_LEAF_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                match entry[24] {
                    1 => pointer_at(25, NODE_POINTER_BYTES),
                    2 => pointer_at(25, DATA_POINTER_BYTES),
                    _ => {}
                }
            }
            (EXTENT_TREE, EXTENT_LOWER_LEAF_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                pointer_at(24, DATA_POINTER_BYTES);
            }
            (INODE_TREE, INODE_INTERNAL_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                pointer_at(34, NODE_POINTER_BYTES);
            }
            (ALLOCATION_RECORDS_TREE, ALLOCATION_RECORD_BYTES) if node.level == 0 => coverage.claim(start, end, ByteClass::LayoutOneCell),
            (ALLOCATION_RECORDS_TREE, ALLOCATION_INTERNAL_ENTRY_BYTES) if node.level >= 1 => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                pointer_at(10, NODE_POINTER_BYTES);
            }
            (ACCOUNTING_TREE, ACCOUNTING_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                if read_u16(entry, 0) == 6 {
                    coverage.reclassify(start + 22, start + 30, ByteClass::LayoutThreeCell);
                }
            }
            (MAPPING_TREE, MAPPING_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                locations.push(location_at(entry, 27));
                locations.push(location_at(entry, 41));
            }
            (0, TREE_TABLE_ENTRY_BYTES) => {
                let tree = read_u64(entry, 0);
                // layout/03 第 22–23 行登记的五条：`deleted_inodes`、墓碑树、谱系树、logged_ops 与退役树（65544..=65548）。
                let class = if (DELETED_INODES_TREE..=RETIREMENT_TREE).contains(&tree) { ByteClass::LayoutThreeCell } else { ByteClass::LayoutOneCell };
                coverage.claim(start, end, class);
                pointer_at(14, NODE_POINTER_BYTES);
            }
            (DELETED_INODES_TREE, DELETED_INODES_ENTRY_BYTES) => coverage.claim(start, end, ByteClass::LayoutThreeCell),
            (TOMBSTONE_TREE, INODE_INTERNAL_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutThreeCell);
                pointer_at(34, NODE_POINTER_BYTES);
            }
            // layout/03 第 21 行「树根与墓碑树同形：码 2 根、条目宽 120」：条目照墓碑树根那一种认领，子指针在偏移 34。
            (RETIREMENT_TREE, INODE_INTERNAL_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutThreeCell);
                pointer_at(34, NODE_POINTER_BYTES);
            }
            _ => {}
        }
    }
}

pub(super) fn claim_container_records(container: &PackedContainer, coverage: &mut CoverageMap, locations: &mut Vec<LocationEntry>) {
    for (record_index, record) in container.records.iter().enumerate() {
        let start = PACKED_UNIT_RECORDS_OFFSET + record_index * container.record_width;
        let end = start + container.record_width;
        match (container.packed_record_type, container.record_width) {
            (2, INODE_RECORD_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                if read_u64(record, 0) == DELETED_INODE {
                    // layout/03 第 12 行：nlink 在偏移 36。
                    coverage.reclassify(start + 36, start + 40, ByteClass::LayoutThreeCell);
                }
            }
            (1, TOMBSTONE_RECORD_BYTES) => coverage.claim(start, end, ByteClass::LayoutThreeCell),
            (RETIREMENT_PACKED_RECORD_TYPE, RETIREMENT_RECORD_BYTES) => coverage.claim(start, end, ByteClass::LayoutThreeCell),
            (4, INSTANCE_TABLE_RECORD_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                if record[0] == 1 && record[1] == 1 {
                    locations.extend(block_pointer_of(&record[2..2 + NODE_POINTER_BYTES]).locations);
                }
            }
            _ => {}
        }
    }
}

fn decode_unit(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>, locations: &mut Vec<LocationEntry>) -> DecodedBody {
    let declared_length = usize_of(read_u16(bytes, UNIT_DECLARED_LENGTH_OFFSET));
    match read_u8(bytes, 6) {
        2 => {
            if bytes.len() != NODE_BYTES {
                failures.push(format!("码 2 单元长 {}", bytes.len()));
                return DecodedBody::Unrecognized;
            }
            let key_width = usize::from(read_u8(bytes, 51));
            let header_end = index_node_header_bytes(key_width);
            let entries_offset = header_end + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
            let entry_count = usize_of(read_u16(bytes, 82 + 2 * key_width));
            let entry_width = usize_of(read_u16(bytes, 84 + 2 * key_width));
            if !header_checksum_holds(bytes, header_end) {
                failures.push("码 2 头校验和不对".to_string());
            }
            if !crc_matches(&bytes[header_end..], read_u32(bytes, 76 + 2 * key_width)) {
                failures.push("码 2 载荷 CRC 不对".to_string());
            }
            if entry_count * entry_width != declared_length {
                failures.push(format!("码 2 条目数 {entry_count} × 条目宽 {entry_width} ≠ 声明长度 {declared_length}"));
            }
            let entries_end = (entries_offset + entry_count * entry_width).min(NODE_BYTES);
            let node = IndexNode {
                tree_identifier: read_u64(bytes, 42),
                level: read_u8(bytes, 50),
                key_width,
                key_range_low: bytes[52..52 + key_width].to_vec(),
                key_range_high: bytes[52 + key_width..52 + 2 * key_width].to_vec(),
                birth_txg: read_u64(bytes, 52 + 2 * key_width),
                entry_count,
                entry_width,
                entries: (0..entry_count)
                    .filter(|entry_index| entries_offset + (entry_index + 1) * entry_width <= NODE_BYTES)
                    .map(|entry_index| bytes[entries_offset + entry_index * entry_width..entries_offset + (entry_index + 1) * entry_width].to_vec())
                    .collect(),
            };
            coverage.claim(0, header_end, ByteClass::LayoutOneCell);
            coverage.claim(header_end, entries_offset, ByteClass::ConstantZeroPadding);
            claim_index_node_entries(&node, entries_offset, coverage, locations);
            coverage.claim(entries_end, NODE_BYTES, ByteClass::ConstantZeroPadding);
            DecodedBody::IndexNode(node)
        }
        3 => {
            if bytes.len() != DATA_UNIT_BYTES {
                failures.push(format!("码 3 单元长 {}", bytes.len()));
                return DecodedBody::Unrecognized;
            }
            if !header_checksum_holds(bytes, PACKED_UNIT_PLAINTEXT_HEADER_BYTES) {
                failures.push("码 3 头校验和不对".to_string());
            }
            if !crc_matches(&bytes[PACKED_UNIT_PLAINTEXT_HEADER_BYTES..], read_u32(bytes, 89)) {
                failures.push("码 3 载荷校验和不对".to_string());
            }
            if read_u8(bytes, 42) != 3 {
                failures.push("码 3 偏移 42 的标签副本不是 3".to_string());
            }
            let record_count = usize_of(read_u16(bytes, 69));
            let record_width = usize_of(read_u16(bytes, 71));
            if record_count * record_width > declared_length {
                failures.push(format!("码 3 记录数 {record_count} × 记录宽 {record_width} > 声明长度 {declared_length}"));
            }
            let records_end = (PACKED_UNIT_RECORDS_OFFSET + record_count * record_width).min(DATA_UNIT_BYTES);
            let container = PackedContainer {
                birth_tree: read_u64(bytes, 43),
                packed_record_type: read_u16(bytes, 51),
                container_number: read_u64(bytes, 53),
                container_birth: read_u64(bytes, 61),
                record_count,
                record_width,
                birth_txg: read_u64(bytes, 73),
                records: (0..record_count)
                    .filter(|record_index| PACKED_UNIT_RECORDS_OFFSET + (record_index + 1) * record_width <= DATA_UNIT_BYTES)
                    .map(|record_index| bytes[PACKED_UNIT_RECORDS_OFFSET + record_index * record_width..PACKED_UNIT_RECORDS_OFFSET + (record_index + 1) * record_width].to_vec())
                    .collect(),
            };
            coverage.claim(0, PACKED_UNIT_PLAINTEXT_HEADER_BYTES, ByteClass::LayoutOneCell);
            coverage.claim(PACKED_UNIT_PLAINTEXT_HEADER_BYTES, PACKED_UNIT_RECORDS_OFFSET, ByteClass::ConstantZeroPadding);
            claim_container_records(&container, coverage, locations);
            coverage.claim(records_end, DATA_UNIT_BYTES, ByteClass::ConstantZeroPadding);
            DecodedBody::PackedContainer(container)
        }
        1 => {
            if bytes.len() != DATA_UNIT_BYTES {
                failures.push(format!("码 1 单元长 {}", bytes.len()));
                return DecodedBody::Unrecognized;
            }
            if !header_checksum_holds(bytes, DATA_UNIT_PLAINTEXT_HEADER_BYTES) {
                failures.push("码 1 头校验和不对".to_string());
            }
            if !crc_matches(&bytes[DATA_UNIT_PLAINTEXT_HEADER_BYTES..], read_u32(bytes, 101)) {
                failures.push("码 1 载荷 CRC 不对".to_string());
            }
            coverage.claim(0, DATA_UNIT_PLAINTEXT_HEADER_BYTES, ByteClass::LayoutOneCell);
            coverage.claim(DATA_UNIT_PLAINTEXT_HEADER_BYTES, DATA_UNIT_PAYLOAD_OFFSET, ByteClass::ConstantZeroPadding);
            let payload_end = (DATA_UNIT_PAYLOAD_OFFSET + declared_length).min(DATA_UNIT_BYTES);
            coverage.claim(DATA_UNIT_PAYLOAD_OFFSET, payload_end, ByteClass::LayoutOneCell);
            coverage.claim(payload_end, DATA_UNIT_BYTES, ByteClass::ConstantZeroPadding);
            DecodedBody::DataUnit { birth_txg: read_u64(bytes, 75) }
        }
        other => {
            failures.push(format!("单元类型标签 {other} 不认得"));
            DecodedBody::Unrecognized
        }
    }
}

pub(super) fn decode_region(write: &RegionWrite) -> DecodedRegion {
    let region_kind = region_kind_of(write);
    let mut coverage = CoverageMap::new(write.bytes.len());
    let mut failures = Vec::new();
    let mut locations = Vec::new();
    let body = match region_kind {
        RegionKind::SystemConfigurationSlot => DecodedBody::SystemConfiguration(decode_system_configuration(&write.bytes, &mut coverage, &mut failures)),
        RegionKind::RootRecordSlot => DecodedBody::RootRecord(decode_root_record(&write.bytes, &mut coverage, &mut failures, &mut locations)),
        RegionKind::JournalRecord => DecodedBody::JournalRecord(decode_journal_record(&write.bytes, &mut coverage, &mut failures, &mut locations)),
        RegionKind::Unit => decode_unit(&write.bytes, &mut coverage, &mut failures, &mut locations),
        RegionKind::Unrecognized => {
            failures.push(format!("偏移 {} 长 {} 的写认不出种类（magic 或落点不对）", write.offset, write.bytes.len()));
            DecodedBody::Unrecognized
        }
    };
    if region_kind == RegionKind::RootRecordSlot && !write.is_force_unit_access {
        failures.push("根槽写不是 FUA".to_string());
    }
    DecodedRegion {
        step: write.step,
        device: write.device,
        offset: write.offset,
        bytes: write.bytes.clone(),
        region_kind,
        body,
        structure_failures: failures,
        coverage,
        locations_inside: locations,
    }
}
