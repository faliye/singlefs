//! 单元的角色与名字、一段写入的分析、切成片与各段的预期值。

use super::*;

// ---------------------------------------------------------------------------------------------
// 一次发布（或一段不属于任何发布的写）的解码结果：单元按落点合成角色，角色身份照跑前登记第六节 R4.a。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RoleIdentity {
    pub(super) unit_type_tag: u8,
    pub(super) tree_identifier: u64,
    pub(super) level: u8,
    pub(super) packed_record_type: u64,
    pub(super) container_number: u64,
    pub(super) allocation_device: Option<u64>,
    pub(super) allocation_position: Option<u64>,
    pub(super) is_tree_root: bool,
}

#[derive(Clone, Debug)]
pub(super) struct UnitRole {
    pub(super) identity: RoleIdentity,
    pub(super) role_name: String,
    pub(super) slot: u64,
    /// 设备 0 那一份（两份逐字节相同由结构检查判）。
    pub(super) copy_on_first_device: DecodedRegion,
}

/// 码 2 单元的「树」：码 2 看偏移 42，码 3 看出生树（偏移 43）。
fn tree_of_region(region: &DecodedRegion) -> u64 {
    match &region.body {
        DecodedBody::IndexNode(node) => node.tree_identifier,
        DecodedBody::PackedContainer(container) => container.birth_tree,
        DecodedBody::DataUnit { .. } => EXTENT_TREE,
        DecodedBody::SystemConfiguration(_) | DecodedBody::RootRecord(_) | DecodedBody::JournalRecord(_) | DecodedBody::Unrecognized => u64::MAX,
    }
}

/// 分配记录树 key = (设备 4, 槽号 6)（layout/01 第 297 行）：key 区间起点的设备段与按位置序号（第 298 行）。
fn allocation_position_of(node: &IndexNode) -> (u64, u64) {
    let device = read_u32(&node.key_range_low, 0);
    let start_slot = read_unsigned_48_bit(&node.key_range_low, 4);
    let position = match node.level {
        0 => start_slot / ALLOCATION_LEAF_SLOTS,
        _ => start_slot / ALLOCATION_LEVEL_ONE_SLOTS,
    };
    (device, position)
}

pub(super) fn role_identity_of(region: &DecodedRegion, is_tree_root: bool) -> RoleIdentity {
    let (level, packed_record_type, container_number, allocation_device, allocation_position) = match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == ALLOCATION_RECORDS_TREE && !is_tree_root => {
            let (device, position) = allocation_position_of(node);
            (node.level, 0, 0, Some(device), Some(position))
        }
        DecodedBody::IndexNode(node) => (node.level, 0, 0, None, None),
        DecodedBody::PackedContainer(container) => (0, container.packed_record_type, container.container_number, None, None),
        DecodedBody::DataUnit { .. } | DecodedBody::SystemConfiguration(_) | DecodedBody::RootRecord(_) | DecodedBody::JournalRecord(_) | DecodedBody::Unrecognized => {
            (0, 0, 0, None, None)
        }
    };
    RoleIdentity {
        unit_type_tag: region.unit_type_tag(),
        tree_identifier: tree_of_region(region),
        level,
        packed_record_type,
        container_number,
        allocation_device,
        allocation_position,
        is_tree_root,
    }
}

/// 解码器独立认出的角色映回 `TransactionUnit::tag` 的写法（layout/03 第 47 行列的角色名）；只用来对拍（S3）与 R3.c 报名字。
pub(super) fn role_name_of(
    identity: &RoleIdentity,
    region: &DecodedRegion,
    inode_container_numbers: &[u64],
    tombstone_container_numbers: &[u64],
    retirement_container_numbers: &[u64],
) -> String {
    let rank_among = |numbers: &[u64], number: u64| numbers.iter().position(|candidate| *candidate == number).unwrap_or(usize::MAX);
    match (identity.unit_type_tag, identity.tree_identifier, identity.is_tree_root) {
        (1, _, _) => "t1".to_string(),
        (2, EXTENT_TREE, true) => "t2".to_string(),
        (2, EXTENT_TREE, false) => match &region.body {
            DecodedBody::IndexNode(node) => {
                let start_offset = read_u64(&node.key_range_low, 16);
                format!("t2下@{}.{}", node.level, start_offset / (DATA_UNIT_PAYLOAD_CAPACITY * EXTENT_LOWER_LEAF_DATA_UNITS))
            }
            DecodedBody::SystemConfiguration(_)
            | DecodedBody::RootRecord(_)
            | DecodedBody::JournalRecord(_)
            | DecodedBody::PackedContainer(_)
            | DecodedBody::DataUnit { .. }
            | DecodedBody::Unrecognized => "t2下@?".to_string(),
        },
        (3, INODE_TREE, _) => match rank_among(inode_container_numbers, identity.container_number) {
            0 => "t3".to_string(),
            rank => format!("t3+{rank}"),
        },
        (2, INODE_TREE, _) => "t4".to_string(),
        (2, ALLOCATION_RECORDS_TREE, true) => "t5".to_string(),
        (2, ALLOCATION_RECORDS_TREE, false) => format!(
            "t5@{}.{}.{}",
            identity.level,
            identity.allocation_device.unwrap_or(u64::MAX),
            identity.allocation_position.unwrap_or(u64::MAX)
        ),
        (2, ACCOUNTING_TREE, _) => "t6".to_string(),
        (2, MAPPING_TREE, _) => "t7".to_string(),
        (2, 0, _) => "t8".to_string(),
        (2, DELETED_INODES_TREE, _) => "td".to_string(),
        (3, TOMBSTONE_TREE, _) => format!("tt@{}", rank_among(tombstone_container_numbers, identity.container_number)),
        (2, TOMBSTONE_TREE, _) => "tt".to_string(),
        // layout/03 第 52 行角色名：tr@叶序 退役容器、tr 退役树根；叶序照墓碑容器那一种，取退役树根条目里的容器号排序。
        (3, RETIREMENT_TREE, _) => format!("tr@{}", rank_among(retirement_container_numbers, identity.container_number)),
        (2, RETIREMENT_TREE, _) => "tr".to_string(),
        (3, 0, _) => "ti".to_string(),
        (tag, tree, _) => format!("unknown@{tag}.{tree}.{}", identity.level),
    }
}

/// 段里一步的视图（切段用）。
#[derive(Clone, Debug)]
pub(super) struct OperationView {
    pub(super) is_barrier: bool,
    pub(super) region_kind: RegionKind,
    pub(super) is_force_unit_access: bool,
}

#[derive(Clone, Debug)]
pub(super) struct SegmentAnalysis {
    pub(super) start_step: usize,
    pub(super) end_step: usize,
    pub(super) label: SegmentLabel,
    pub(super) is_deletion_publish: bool,
    pub(super) operations: Vec<OperationView>,
    pub(super) regions: Vec<DecodedRegion>,
    pub(super) units: Vec<UnitRole>,
    pub(super) root_records: Vec<(u32, u64, RootRecord)>,
    pub(super) journal_records: Vec<(u32, u64, JournalRecord)>,
    pub(super) system_configurations: Vec<(usize, u32, u64, SystemConfiguration)>,
    pub(super) structure_failures: Vec<String>,
    pub(super) unreferenced_units: Vec<String>,
}

impl SegmentAnalysis {
    pub(super) fn decoded_txg(&self) -> Option<u64> {
        self.root_records.first().map(|(_, _, record)| record.txg)
    }
    pub(super) fn role_names(&self) -> Vec<String> {
        self.units.iter().map(|unit| unit.role_name.clone()).collect()
    }
    pub(super) fn identities(&self) -> BTreeSet<RoleIdentity> {
        self.units.iter().map(|unit| unit.identity.clone()).collect()
    }
}

/// 一段历史跨段携带的「现行那一版」：每个角色最近一次写出的那一份（设备 0），没写就照抄上一版（layout/03 第 13–17 行「照抄」）。
#[derive(Clone, Debug, Default)]
pub(super) struct VersionState {
    pub(super) latest_by_identity: BTreeMap<RoleIdentity, DecodedRegion>,
    pub(super) latest_root_record: Option<RootRecord>,
}

impl VersionState {
    pub(super) fn latest_node(&self, tree: u64, want_root: bool) -> Option<&IndexNode> {
        self.latest_by_identity
            .iter()
            .filter(|(identity, _)| identity.unit_type_tag == 2 && identity.tree_identifier == tree && identity.is_tree_root == want_root)
            .find_map(|(_, region)| match &region.body {
                DecodedBody::IndexNode(node) => Some(node),
                DecodedBody::SystemConfiguration(_)
                | DecodedBody::RootRecord(_)
                | DecodedBody::JournalRecord(_)
                | DecodedBody::PackedContainer(_)
                | DecodedBody::DataUnit { .. }
                | DecodedBody::Unrecognized => None,
            })
    }
    pub(super) fn latest_containers(&self, tree: u64, packed_record_type: u64) -> Vec<&PackedContainer> {
        self.latest_by_identity
            .iter()
            .filter(|(identity, _)| identity.unit_type_tag == 3 && identity.tree_identifier == tree && identity.packed_record_type == packed_record_type)
            .filter_map(|(_, region)| match &region.body {
                DecodedBody::PackedContainer(container) => Some(container),
                DecodedBody::SystemConfiguration(_)
                | DecodedBody::RootRecord(_)
                | DecodedBody::JournalRecord(_)
                | DecodedBody::IndexNode(_)
                | DecodedBody::DataUnit { .. }
                | DecodedBody::Unrecognized => None,
            })
            .collect()
    }
}

/// 身份引用里的容器号（inode 树与墓碑树内部条目：分隔 key 8 + 出生树 8 + 打包记录类型 2 + 容器号 8，layout/01 第 261 行）。
pub(super) fn container_numbers_of_root(node: Option<&IndexNode>) -> Vec<u64> {
    node.map(|node| node.entries.iter().map(|entry| read_u64(entry, 18)).collect()).unwrap_or_default()
}

/// 树表条目：树 ID 0、条目长度 8、种类 10、flags 12、根指针 14、`previous_snapshot_txg` 100、诞生 txg 108、头 ID 116、预留 124（layout/01 第 387 行按行累加）。
#[derive(Clone, Debug)]
pub(super) struct TreeTableEntry {
    pub(super) tree: u64,
    pub(super) kind_code: u64,
    pub(super) root_pointer: BlockPointer,
    pub(super) head_identifier: u64,
}

pub(super) fn tree_table_entries_of(node: &IndexNode) -> Vec<TreeTableEntry> {
    node.entries
        .iter()
        .map(|entry| TreeTableEntry {
            tree: read_u64(entry, 0),
            kind_code: read_u16(entry, 10),
            root_pointer: block_pointer_of(&entry[14..14 + NODE_POINTER_BYTES]),
            head_identifier: read_u64(entry, 116),
        })
        .collect()
}

pub(super) fn analyze_segment(segment: &ExportSegment, operations: &[WindowOperation], state: &VersionState) -> SegmentAnalysis {
    let mut views = Vec::new();
    let mut regions = Vec::new();
    for operation in operations.iter().filter(|operation| operation.step() >= segment.start_step && operation.step() < segment.end_step) {
        match operation {
            WindowOperation::Barrier { .. } => views.push(OperationView {
                is_barrier: true,
                region_kind: RegionKind::Unrecognized,
                is_force_unit_access: false,
            }),
            WindowOperation::Write(write) => {
                let region = decode_region(write);
                views.push(OperationView {
                    is_barrier: false,
                    region_kind: region.region_kind,
                    is_force_unit_access: write.is_force_unit_access,
                });
                regions.push(region);
            }
        }
    }
    let mut structure_failures: Vec<String> = regions
        .iter()
        .flat_map(|region| region.structure_failures.iter().map(move |failure| format!("step {} 设备 {}：{failure}", region.step, region.device)))
        .collect();
    // 树根：这一段写的根记录与树表单元指着的落点（树表没写就取现行那一版）。
    let mut root_locations: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut root_records = Vec::new();
    let mut journal_records = Vec::new();
    let mut system_configurations = Vec::new();
    for region in &regions {
        match &region.body {
            DecodedBody::RootRecord(record) => {
                for block_pointer in [&record.tree_table_pointer, &record.instance_table_pointer, &record.mapping_root_pointer, &record.allocation_root_pointer] {
                    root_locations.extend(block_pointer.locations.iter().map(|location| (location.device, location.slot)));
                }
                root_records.push((region.device, region.offset, record.clone()));
            }
            DecodedBody::JournalRecord(record) => journal_records.push((region.device, region.offset, record.clone())),
            DecodedBody::SystemConfiguration(configuration) => system_configurations.push((region.step, region.device, region.offset, configuration.clone())),
            DecodedBody::IndexNode(_) | DecodedBody::PackedContainer(_) | DecodedBody::DataUnit { .. } | DecodedBody::Unrecognized => {}
        }
    }
    let written_tree_table = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == 0 && region.region_kind == RegionKind::Unit => Some(node.clone()),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::IndexNode(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    });
    let tree_table = written_tree_table.or_else(|| state.latest_node(0, true).cloned());
    if let Some(node) = &tree_table {
        for entry in tree_table_entries_of(node) {
            root_locations.extend(entry.root_pointer.locations.iter().map(|location| (location.device, location.slot)));
        }
    }
    // 单元按落点成组：每个落点两块盘各一份、逐字节相同（layout/01 第 393 行）。
    let mut by_slot: BTreeMap<u64, Vec<&DecodedRegion>> = BTreeMap::new();
    let mut slot_order = Vec::new();
    for region in regions.iter().filter(|region| region.region_kind == RegionKind::Unit) {
        if !by_slot.contains_key(&region.slot()) {
            slot_order.push(region.slot());
        }
        by_slot.entry(region.slot()).or_default().push(region);
    }
    let inode_root_in_segment = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == INODE_TREE => Some(node),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::IndexNode(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    });
    let tombstone_root_in_segment = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == TOMBSTONE_TREE => Some(node),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::IndexNode(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    });
    let retirement_root_in_segment = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == RETIREMENT_TREE => Some(node),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::IndexNode(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    });
    let inode_container_numbers = container_numbers_of_root(inode_root_in_segment.or_else(|| state.latest_node(INODE_TREE, true)));
    let tombstone_container_numbers = container_numbers_of_root(tombstone_root_in_segment.or_else(|| state.latest_node(TOMBSTONE_TREE, true)));
    let retirement_container_numbers = container_numbers_of_root(retirement_root_in_segment.or_else(|| state.latest_node(RETIREMENT_TREE, true)));
    let segment_locations: Vec<LocationEntry> = regions.iter().flat_map(|region| region.locations_inside.iter().copied()).collect();
    let mut units = Vec::new();
    let mut unreferenced_units = Vec::new();
    for slot in slot_order {
        let copies = &by_slot[&slot];
        let devices: BTreeSet<u32> = copies.iter().map(|copy| copy.device).collect();
        if copies.len() != 2 || devices != BTreeSet::from([0, 1]) || copies[0].bytes != copies[1].bytes {
            structure_failures.push(format!("槽 {slot} 的单元不是两块盘各一份逐字节相同（{} 份）", copies.len()));
        }
        for copy in copies {
            let referencing: Vec<&LocationEntry> = segment_locations.iter().filter(|location| location.device == copy.device && location.slot == slot).collect();
            if referencing.is_empty() {
                unreferenced_units.push(format!("step {} 设备 {} 槽 {slot}", copy.step, copy.device));
            }
            let expected_checksum = u64::from(castagnoli_crc32(&copy.bytes));
            for location in referencing {
                if location.checksum != expected_checksum {
                    structure_failures.push(format!("step {} 设备 {} 槽 {slot}：父指针位置条目里的整单元校验和 {} ≠ {expected_checksum}", copy.step, copy.device, location.checksum));
                }
            }
        }
        let Some(first_copy) = copies.iter().find(|copy| copy.device == 0).or(copies.first()) else { continue };
        let is_tree_root = root_locations.contains(&(first_copy.device, slot));
        let identity = role_identity_of(first_copy, is_tree_root);
        let role_name = role_name_of(&identity, first_copy, &inode_container_numbers, &tombstone_container_numbers, &retirement_container_numbers);
        units.push(UnitRole { identity, role_name, slot, copy_on_first_device: (*first_copy).clone() });
    }
    SegmentAnalysis {
        start_step: segment.start_step,
        end_step: segment.end_step,
        label: segment.label.clone(),
        is_deletion_publish: segment.is_deletion_publish(),
        operations: views,
        regions,
        units,
        root_records,
        journal_records,
        system_configurations,
        structure_failures,
        unreferenced_units,
    }
}

impl VersionState {
    pub(super) fn absorb(&mut self, analysis: &SegmentAnalysis) {
        for unit in &analysis.units {
            self.latest_by_identity.insert(unit.identity.clone(), unit.copy_on_first_device.clone());
        }
        if let Some((_, _, record)) = analysis.root_records.first() {
            self.latest_root_record = Some(record.clone());
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 切段（D13 已定项 4：屏障与 FUA 写切段；屏障按设备记、每块盘一步）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SegmentPiece {
    kinds: Vec<(String, usize)>,
}

impl SegmentPiece {
    fn write_count(&self) -> usize {
        self.kinds.iter().filter(|(kind, _)| kind != "barrier").map(|(_, count)| count).sum()
    }
    fn add(&mut self, kind: &str) {
        match self.kinds.iter_mut().find(|(existing, _)| existing == kind) {
            Some((_, count)) => *count += 1,
            None => self.kinds.push((kind.to_string(), 1)),
        }
    }
    fn kind_text(&self) -> String {
        let parts: Vec<String> = self
            .kinds
            .iter()
            .map(|(kind, count)| if *count == 1 { kind.clone() } else { format!("{kind}×{count}") })
            .collect();
        format!("[{}]", parts.join(","))
    }
}

pub(super) fn cut_into_pieces(operations: &[OperationView]) -> Vec<SegmentPiece> {
    let mut pieces = Vec::new();
    let mut current = SegmentPiece { kinds: Vec::new() };
    let mut current_is_closed_by_a_barrier = false;
    for operation in operations {
        if operation.is_barrier {
            current.add("barrier");
            current_is_closed_by_a_barrier = true;
        } else if operation.is_force_unit_access {
            if !current.kinds.is_empty() {
                pieces.push(std::mem::replace(&mut current, SegmentPiece { kinds: Vec::new() }));
            }
            let mut force_unit_access_piece = SegmentPiece { kinds: Vec::new() };
            force_unit_access_piece.add(operation.region_kind.step_kind_name());
            pieces.push(force_unit_access_piece);
            current_is_closed_by_a_barrier = false;
        } else {
            if current_is_closed_by_a_barrier {
                pieces.push(std::mem::replace(&mut current, SegmentPiece { kinds: Vec::new() }));
                current_is_closed_by_a_barrier = false;
            }
            current.add(operation.region_kind.step_kind_name());
        }
    }
    if !current.kinds.is_empty() {
        pieces.push(current);
    }
    pieces
}

pub(super) fn segment_count_text(pieces: &[SegmentPiece]) -> String {
    pieces.iter().map(|piece| piece.write_count().to_string()).collect::<Vec<_>>().join("+")
}

pub(super) fn segment_kind_text(pieces: &[SegmentPiece]) -> String {
    pieces.iter().map(SegmentPiece::kind_text).collect::<Vec<_>>().join("|")
}

/// 跑前登记第一节 1.2：普通发布的种类串把单元写数换成 2 × 单元数（layout/01 第 411 行）。
pub(super) fn expected_kind_text(units_per_device: usize) -> String {
    format!(
        "[unit_write×{},barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        2 * units_per_device
    )
}

pub(super) fn expected_count_text(units_per_device: usize) -> String {
    format!("{}+2+1+2", 2 * units_per_device)
}

/// A-根槽：第 n 次发布写区域 `n mod 3` 的槽 `(n div 3) mod 8`（layout/01 第 163 行）。
pub(super) fn expected_root_slot(txg: u64) -> (u32, u64) {
    let region = txg % 3;
    let slot = (txg / 3) % ROOT_RING_SLOTS_PER_REGION;
    (
        ROOT_RING_REGION_DEVICES[usize_of(region)],
        ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE_BYTES + slot * ROOT_RING_SLOT_SPACING_BYTES,
    )
}

/// A-journal：记录 n 落 `(计数器 − 1) × 4096`（layout/01 第 319 行），跑前登记取「计数器 = txg」。
pub(super) fn expected_journal_offset_assuming_counter_equals_txg(txg: u64) -> u64 {
    JOURNAL_RING_START_BYTES + (txg - 1) * u64_of(JOURNAL_RECORD_BYTES)
}

/// A-系统配置：槽世代号 mkfs 1、取号两轮 2 / 3、暖机 4 / 5、新池新建文件 6 ⇒ txg t 的那次发布写世代号 t + 3，槽 = 世代号 mod 2（layout/01 第 97、119、399 行）。
pub(super) fn expected_system_configuration_for_txg_in_the_first_instance(txg: u64) -> (u64, u64) {
    let generation = txg + 3;
    (generation, (generation % 2) * u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES))
}
