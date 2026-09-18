//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头（扫描方向），判第一版的 29 条不变量。
//! 最新的那条根另判 I-7.2（能不能完整走完）；引用集合按全部有效根取并集——根环里更早的根仍是可回退的状态，
//! 它们引用的单元仍占着空间（I-3.1 与 I-5.1 的这一读法 2026-09-14 用户收尾弹窗定甲，见 records 2026-09-13-总审核 十一·七）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ALLOCATION_RECORD_BYTES, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY,
    JOURNAL_RECORD_BYTES, NODE_BYTES, SLOT_BYTES, TREE_IDENTIFIER_CENTRAL_MAPPING,
    TREE_TABLE_ENTRY_BYTES,
};

use crate::image::{
    chosen_superblocks, judge_location_order, parse_data_pointer, parse_node_pointer,
    read_referenced_unit, root_slot_positions, valid_roots, verified_superblock_slots, ImageReader,
    InvariantVerdict, Judgements, PointerView, PoolGeometry,
};
use crate::{
    check_index_node_keys, check_journal_record, checksum_field_holds, crc32_castagnoli_table,
    index_node_view, key_schema_for_tree_kind, read_six_byte_unsigned, read_u16, read_u32,
    read_u64, KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
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
    data_unit_objects: Vec<(u64, u64, String)>,
    /// 最新根下面记账树里的行：(统计量, 设备) → 值。
    accounting: BTreeMap<(u16, u32), u64>,
    accounting_seen: bool,
    /// 最新根指着的实例表里的 (实例代号, T)：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集）。
    instance_table_rows: Vec<(u32, u64)>,
}

impl Walk<'_> {
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
                format!("{what}：头里的 fsid {fsid:#x} 与超级块的低 8 字节不符")
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
        // 实例表单元：根记录直接持有（自证单元，I-1.3 豁免）。
        let instance_pointer = parse_node_pointer(&record[170..256]);
        judge_location_order(&mut self.judgements, &instance_pointer, "实例表指针");
        for location in &instance_pointer.locations {
            self.note_reference(location.device, location.slot, 2, "实例表单元");
        }
        if let Some(unit) = read_referenced_unit(
            self.reader,
            &mut self.judgements,
            &instance_pointer.locations,
            data_unit_bytes(),
            "实例表单元",
        ) {
            if self.visited_units.insert((
                instance_pointer.locations[0].device,
                instance_pointer.locations[0].slot,
            )) {
                let records =
                    self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
                if is_newest {
                    if let Some(records) = records {
                        let mount_root_instance =
                            u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
                        self.judge_instance_table_rows(&records, mount_root_instance);
                    }
                }
            }
        } else {
            self.walk_failures
                .push("实例表单元两份都读不到对得上的".to_string());
        }
        // 树表单元：不属于任何树（树 ID 0）。
        let Some(tree_table) =
            self.read_index_node(&record[36..122], 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
        else {
            return;
        };
        for entry in &tree_table.entries {
            self.walk_tree_table_entry(entry);
        }
        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
        if let Some(mapping) = self.read_index_node(
            &record[256..342],
            TREE_IDENTIFIER_CENTRAL_MAPPING,
            KEY_SCHEMA_MAPPING,
            "中央映射树的根",
        ) {
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
        let _ = is_newest;
    }

    fn walk_tree_table_entry(&mut self, entry: &[u8]) {
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

    /// I-9.1：inode 树根恒码 2（读到这里时它已经按码 2 判过头）；层级 1 的条目逐条判 I-9.2 并走到叶。
    fn walk_inode_root(&mut self, root: &crate::IndexNodeView, tree: u64) {
        if root.level == 0 {
            return;
        }
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
            if let Some(records) =
                self.judge_packed_container(&unit, PACKED_TYPE_INODE, "inode 树的叶容器")
            {
                let container = read_u64(&unit, 53);
                self.judgements.judge("I-9.13", !records.is_empty(), || {
                    "类型 2 容器一条记录都没有".to_string()
                });
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
                }
            }
        }
    }

    /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
    /// I-3.8（实例表行唯一且低于挂载根）：`kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例；
    /// 链指针记录（`kind` 1）恒为一片的最后一条（D18（块里携带什么信息） 已定项 11）。「行只在回收条件成立后删」对着一个镜像判不了。
    fn judge_instance_table_rows(&mut self, records: &[Vec<u8>], mount_root_instance: u32) {
        let mut instances: Vec<u32> = Vec::new();
        let mut unique = true;
        let mut below_mount_root = true;
        let mut chain_record_last = false;
        for (index, row) in records.iter().enumerate() {
            match row.first().copied() {
                Some(0) => {
                    let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
                    if instances.contains(&instance) {
                        unique = false;
                    }
                    instances.push(instance);
                    self.instance_table_rows.push((
                        instance,
                        u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),
                    ));
                    if instance >= mount_root_instance {
                        below_mount_root = false;
                    }
                    chain_record_last = false;
                }
                Some(1) => chain_record_last = index + 1 == records.len(),
                _ => chain_record_last = false,
            }
        }
        self.judgements.judge(
            "I-3.8",
            unique && below_mount_root && chain_record_last,
            || {
                format!(
                    "实例表：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}），链指针记录在末尾 = {chain_record_last}"
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

/// 扫描方向：候选槽上头校验和过的码 2 节点，取它头里的树 ID。只数诞生代号不超过 `newest_published_txg` 的节点：
/// 根环里没有一条根发布过的那一代，单元还不算「出现过」（I-7.8 的这一读法 2026-09-14 用户收尾弹窗定甲；字面读法会把
/// 崩在发布之前的那个事务写出的孤儿节点也算进去，见 records 十一·七）。
fn scanned_tree_identifiers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    newest_published_txg: u64,
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
            if header_end < bytes.len()
                && checksum_field_holds(&bytes, header_end, 10)
                && read_u64(&bytes, 52 + 2 * usize::from(bytes[51])) <= newest_published_txg
            {
                identifiers.insert(read_u64(&bytes, 42));
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
/// 任何一个该读的槽读不出 ⇒ None（I-7.7（超级块实例代号不低于根环） 读不全报不适用）。
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

/// I-7.7（超级块实例代号不低于根环） 的 ① ②（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：
/// 每盘的实例代号取它两槽里全部自证过（fsid 与本池相同）的槽中最大的；① 本池的每个根记录、journal 记录、单元写序的实例代号
/// 不超过各盘的最大者，② 各盘不等时较大者不出现在它们里——两句合起来就是「每一个带号的东西都不超过各盘最大号里最小的那个」。
fn judge_instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let highest_by_device: Vec<Option<u32>> = verified_superblock_slots(reader)
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
            "有一块盘没有本池 fsid 的超级块槽：那块盘归不归这个池由 I-1.4 判",
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
        format!("{what}带实例代号 {instance}，高于各盘超级块的最大者 {pool_highest}（①）")
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
    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，不重复数——
    // 主走读的 `note_reference` 也不数它们（数了 I-3.1（已分配统计对得上） 与 I-5.1（引用不重叠） 会把同一个落点算两遍）。
    let mapping_root = parse_node_pointer(&record[256..342]);
    references.note(&mapping_root);
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

/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    let superblocks = chosen_superblocks(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = superblocks
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    if chosen.is_empty() || chosen.len() != superblocks.len() {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "有一块盘两个超级块槽都无效：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    let geometry = chosen[0].2;
    // 各盘择到的超级块要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的超级块 fsid 与盘 {} 的不同：这块盘不属于这个池",
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
    let roots = valid_roots(reader, &geometry);
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        return root_ring_judgements.into_report();
    }
    let newest_index = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
        .map(|(index, _)| index)
        .expect("非空");
    let mut walk = Walk {
        reader,
        judgements: root_ring_judgements,
        filesystem_identifier_low: u64::from_le_bytes(
            geometry.filesystem_identifier[..8]
                .try_into()
                .expect("8 字节"),
        ),
        references: BTreeMap::new(),
        visited_units: BTreeSet::new(),
        walk_failures: Vec::new(),
        tree_identifiers_in_tables: BTreeSet::new(),
        inode_object_birth: BTreeMap::new(),
        data_unit_objects: Vec::new(),
        accounting: BTreeMap::new(),
        accounting_seen: false,
        instance_table_rows: Vec::new(),
    };
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
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
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            let abandoned = instance_table_rows.iter().any(|(row_instance, row_txg)| {
                *row_instance == root.instance && root.checkpoint_txg > *row_txg
            });
            let below_floor = root.checkpoint_txg < newest_rollback_floor;
            *index == newest_index || (!abandoned && !below_floor)
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
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
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
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    if accounting_seen {
        for device in &devices {
            let walked: u64 = per_device.get(device).map_or(0, |ranges| {
                ranges.iter().map(|(_, span, _)| span * SLOT_BYTES).sum()
            });
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            judgements.judge("I-3.1", allocated == Some(walked), || {
                format!("盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}")
            });
            let capacity = reader
                .device_bytes(*device)
                .map(|bytes| (bytes / SLOT_BYTES - geometry.unit_area_start_slot) * SLOT_BYTES);
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（全部码 2 单元头 ∪ 走过的树表条目）。
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
    let mut seen = scanned_tree_identifiers(reader, &geometry, newest_published_txg);
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    judgements.into_report()
}
