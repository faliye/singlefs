//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头（扫描方向），判第一版的 23 条不变量。
//! 最新的那条根另判 I-7.2（能不能完整走完）；引用集合按全部有效根取并集——根环里更早的根仍是可回退的状态，
//! 它们引用的单元仍占着空间（I-3.1 与 I-5.1 的这一读法 2026-09-14 用户收尾弹窗定甲，见 records 2026-09-13-总审核 十一·七）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, NODE_BYTES, SLOT_BYTES, TREE_IDENTIFIER_CENTRAL_MAPPING,
};

use crate::image::{
    chosen_superblocks, judge_location_order, parse_data_pointer, parse_node_pointer,
    read_referenced_unit, root_slot_positions, valid_roots, verified_superblock_slots, ImageReader,
    InvariantVerdict, Judgements, PoolGeometry,
};
use crate::{
    check_index_node_keys, check_journal_record, checksum_field_holds, crc32_castagnoli_table,
    index_node_view, key_schema_for_tree_kind, read_u16, read_u32, read_u64, KEY_SCHEMA_MAPPING,
    KEY_SCHEMA_TREE_TABLE,
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
                self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
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
    };
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
    let accounting_seen = walk.accounting_seen;
    let accounting = walk.accounting.clone();
    for (index, (_, _, root)) in roots.iter().enumerate() {
        if index != newest_index {
            walk.walk_root(&root.record_bytes, false);
        }
    }
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
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
