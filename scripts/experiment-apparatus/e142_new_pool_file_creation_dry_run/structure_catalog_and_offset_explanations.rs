//! 结构目录、偏移分类、单元与根记录与日志记录的偏移解释、变化计数的条目、缺口表。

use super::*;

/// 量 1/3/4 的「所属结构」候选表：新池新建文件写到的 8 个单元、根记录、journal 记录（第一节第 3 条的区域清单，去掉系统配置——
/// 系统配置与改动计数无关，按第三节推导；它若意外出现在差异里，classify_offset 找不到候选、归到「未登记」而不是被这张表悄悄吃掉）。
pub(super) struct StructureCatalog<'output> {
    pub(super) units: &'output [(SlotNumber, TransactionUnit, Vec<u8>)],
    root_device: u32,
    root_offset: u64,
    journal_offset: u64,
    journal_length: u64,
}

pub(super) fn build_structure_catalog(output: &TransactionOutput, root_device: u32, root_offset: u64) -> StructureCatalog<'_> {
    StructureCatalog { units: &output.units_by_slot, root_device, root_offset, journal_offset: journal_record_offset(output.record.counter).0, journal_length: output.record_bytes.len() as u64 }
}

/// 给一个 (device, offset) 找它落在哪个候选结构里；返回 (标签, 结构内偏移)。
pub(super) fn classify_offset(catalog: &StructureCatalog, device: u32, offset: u64) -> Option<(&'static str, u64)> {
    for (slot, unit, bytes) in catalog.units {
        let base = slot.device_offset().0;
        if offset >= base && offset < base + bytes.len() as u64 {
            return Some((unit.descriptive_tag(), offset - base));
        }
    }
    if device == catalog.root_device && offset >= catalog.root_offset && offset < catalog.root_offset + PHYSICAL_BLOCK_BYTES {
        return Some(("root_record", offset - catalog.root_offset));
    }
    if offset >= catalog.journal_offset && offset < catalog.journal_offset + catalog.journal_length {
        return Some(("journal_record", offset - catalog.journal_offset));
    }
    None
}

/// NodePointer 内两条位置条目校验和的相对偏移（`NodePointer::write_to`：头 50 + 位置条目 14 × 2，校验和在条目内偏移 10）。
fn node_pointer_checksum_offsets(pointer_start: u64) -> [u64; 2] {
    [pointer_start + POINTER_HEAD_BYTES + 10, pointer_start + POINTER_HEAD_BYTES + LOC_ENTRY + 10]
}

fn overlaps(segment_offset: u64, segment_length: u64, target_offset: u64, target_length: u64) -> bool {
    segment_offset < target_offset + target_length && target_offset < segment_offset + segment_length
}

/// inode 记录字段表（D8 已定项 6，`decisions/08-核心索引结构.md:391-410`）：偏移 → 字段名。
fn inode_record_field_name(offset: u64) -> &'static str {
    match offset {
        0..=7 => "inode",
        8..=15 => "object_birth（出生代）",
        16..=23 => "locality_id",
        24..=39 => "mode/uid/gid/nlink",
        40..=63 => "size/blocks/rdev",
        64..=87 => "atime/mtime/ctime 秒",
        88..=95 => "改动计数",
        96..=107 => "atime/mtime/ctime 纳秒",
        108..=111 => "填充",
        112..=119 => "flags",
        _ => "预留",
    }
}

/// 量 3：给一个 (结构标签, 结构内偏移) 找「谁的校验和 / 指针罩到它」。返回 (是否归了因, 说明)。
/// 归不了因的（返回 false）按 F6 记一行 `name=gap`，不作废——重跑登记第十节 F6。
fn explain_unit_offset(unit: TransactionUnit, offset: u64) -> (bool, String) {
    let (class, key_width) = unit.class_and_key_width();
    if (10..42).contains(&offset) {
        return (true, "单元头校验和：seal_header_checksum 罩 [0,头末尾)，头内容变了就重算（D18 已定项 17）".to_string());
    }
    let payload_crc_offset: u64 = if class == UNIT_CLASS_PACKED { 89 } else { 76 + 2 * key_width as u64 };
    if (payload_crc_offset..payload_crc_offset + 4).contains(&offset) {
        return (true, format!("单元载荷 CRC：castagnoli_crc32 罩记录/条目区到单元末尾（D18 已定项 11 逐字，偏移 {payload_crc_offset}）"));
    }
    match unit {
        TransactionUnit::InodeLeaf => {
            let record_start = PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES;
            if offset >= record_start && offset < record_start + INODE_RECORD_BYTES {
                let field_offset = offset - record_start;
                return (true, format!("inode 记录内偏移 {field_offset}：{}（D8 已定项 6）", inode_record_field_name(field_offset)));
            }
        }
        TransactionUnit::InodeRoot => {
            let entries_start = index_node_header_bytes(key_width) as u64 + NONCE_MAC_RESERVED_BYTES;
            if offset >= entries_start && offset < entries_start + INODE_INTERNAL_ENTRY {
                let entry_offset = offset - entries_start;
                let pointer_start = 34u64; // 分隔 key 8 + 子身份引用 26（build_inode_internal_entry）
                if entry_offset >= pointer_start && entry_offset < pointer_start + NODE_POINTER_BYTES {
                    let checks = node_pointer_checksum_offsets(pointer_start);
                    if (checks[0]..checks[0] + 4).contains(&entry_offset) {
                        return (true, "内部条目里子指针（指向 inode 叶）第 0 条位置条目单元校验和（D19 已定项 4）".to_string());
                    }
                    if (checks[1]..checks[1] + 4).contains(&entry_offset) {
                        return (true, "内部条目里子指针（指向 inode 叶）第 1 条位置条目单元校验和（D19 已定项 4）".to_string());
                    }
                    return (true, "内部条目里子指针其余字段（出生树 / 出生代号 / 实例代号 / 出生序号，不含校验和）".to_string());
                }
                return (true, format!("内部条目内偏移 {entry_offset}（分隔 key / 子身份引用段）"));
            }
        }
        TransactionUnit::MappingTree => {
            let entries_start = index_node_header_bytes(key_width) as u64 + NONCE_MAC_RESERVED_BYTES;
            if offset >= entries_start {
                let relative = offset - entries_start;
                let entry_index = relative / MAPPING_ENTRY_BYTES;
                let entry_offset = relative % MAPPING_ENTRY_BYTES;
                if entry_offset >= MAPPING_KEY_BYTES {
                    let location_offset = entry_offset - MAPPING_KEY_BYTES;
                    let which = u64::from(location_offset >= LOC_ENTRY);
                    let local = location_offset % LOC_ENTRY;
                    if (10..14).contains(&local) {
                        return (true, format!("映射树条目 {entry_index} 的第 {which} 条位置条目单元校验和（build_mapping_entry，D19 已定项 10）"));
                    }
                }
                return (true, format!("映射树条目 {entry_index} 内偏移 {entry_offset}（key 或位置条目非校验和字段）"));
            }
        }
        TransactionUnit::TreeTable => {
            let entries_start = index_node_header_bytes(key_width) as u64 + NONCE_MAC_RESERVED_BYTES;
            if offset >= entries_start {
                let relative = offset - entries_start;
                let entry_index = relative / TREE_TABLE_ENTRY_BYTES;
                let entry_offset = relative % TREE_TABLE_ENTRY_BYTES;
                let pointer_start = 14u64; // 树 ID 8 + 条目长度 2 + 种类 2 + flags 2（TreeTableEntry::to_bytes）
                if entry_offset >= pointer_start && entry_offset < pointer_start + NODE_POINTER_BYTES {
                    let local = entry_offset - pointer_start;
                    let checks = node_pointer_checksum_offsets(0);
                    if (checks[0]..checks[0] + 4).contains(&local) {
                        return (true, format!("树表条目 {entry_index} 的根指针第 0 条位置条目单元校验和（D19 已定项 4）"));
                    }
                    if (checks[1]..checks[1] + 4).contains(&local) {
                        return (true, format!("树表条目 {entry_index} 的根指针第 1 条位置条目单元校验和（D19 已定项 4）"));
                    }
                    return (true, format!("树表条目 {entry_index} 的根指针其余字段"));
                }
                return (true, format!("树表条目 {entry_index} 内偏移 {entry_offset}（树 ID / 种类 / 出生 txg / 头 ID 一类）"));
            }
        }
        TransactionUnit::Data
        | TransactionUnit::ExtentRoot
        | TransactionUnit::Allocation(_)
        | TransactionUnit::AccountingTree => {}
    }
    (false, format!("单元内偏移 {offset}：不在头校验和 / 载荷 CRC / 已知记录条目区里，未登记"))
}

fn explain_root_record_offset(offset: u64) -> (bool, String) {
    match offset {
        0..=3 => (true, "root magic".to_string()),
        4..=19 => (true, "fsid".to_string()),
        20..=23 => (true, "flags（位 0 卸载记号）".to_string()),
        24..=27 => (true, "instance".to_string()),
        28..=35 => (true, "checkpoint_txg".to_string()),
        36..=121 => {
            let local = offset - 36;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "tree_table 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "tree_table 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "tree_table 指针其余字段".to_string())
            }
        }
        122..=129 => (true, "tree_identifier_watermark".to_string()),
        130..=137 => (true, "rollback_floor".to_string()),
        138..=169 => (true, "根记录自证校验和（罩整个 512 槽含补齐、自身按 0 参与，D18 已定项 17）".to_string()),
        170..=255 => (true, "instance_table 指针（预期不随改动计数变；若变了记一行 gap）".to_string()),
        256..=341 => {
            let local = offset - 256;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "mapping_root 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "mapping_root 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "mapping_root 指针其余字段".to_string())
            }
        }
        342..=427 => (
            true,
            "分配记录树根指针（C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 2026-09-23 用户定案：这个装置的场景里恒 0，mkfs / 暖机 / 带文件的一版都不是「树表 0 条、写过行」那一格）".to_string(),
        ),
        428..=456 => (true, "算法类型 / nonce / MAC 留位（恒 0）".to_string()),
        _ => (false, format!("根记录内偏移 {offset}：落在 457 字节字段表之外的补齐区，未登记")),
    }
}

fn explain_journal_record_offset(units: &[(SlotNumber, TransactionUnit, Vec<u8>)], offset: u64) -> (bool, String) {
    // E142 第十四次跑：整张表按 D23 已定项 4 的字段表重写（第三节 3.2：旧表把偏移 8–45 标错了 2 字节）；
    // 第二十一次跑改按 487 字节头的字段表（新根段七项：实例表指针 287、分配记录树根指针 373、卸载记号 459，fsid 463、MAC 471、头末 487）。
    // 每个字段的起点由第七节 A2 的单测钉住。
    match offset {
        0..=3 => (true, "journal magic".to_string()),
        4..=5 => (true, "type".to_string()),
        6 => (true, "algorithm".to_string()),
        7 => (true, "record_flags（位 0 = 本次发布末条，已定项 4、17）".to_string()),
        8..=11 => (true, "record_length".to_string()),
        12..=15 => (true, "named_count".to_string()),
        16..=19 => (true, "instance（jsn 高位）".to_string()),
        20..=25 => (true, "counter（jsn 低位，six-byte）".to_string()),
        26..=33 => (true, "checkpoint_txg".to_string()),
        34..=45 => (true, "nonce 留位（恒 0）".to_string()),
        46..=77 => (true, "记录头校验和（罩整条记录 [0,4096) 含补齐、自身按 0 参与，D23 已定项 13）".to_string()),
        78..=85 => (true, "transaction".to_string()),
        86 => (true, "is_commit".to_string()),
        87..=90 => (true, "ordinal_within_publish（本次发布内序号，已定项 4）".to_string()),
        91..=94 => (true, "back_chain".to_string()),
        95..=98 => (true, "载荷校验和（罩点名项数组，D23 已定项 13）".to_string()),
        99..=184 => {
            let local = offset - 99;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "new_tree_table 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "new_tree_table 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "new_tree_table 指针其余字段".to_string())
            }
        }
        185..=270 => {
            let local = offset - 185;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "new_mapping_root 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "new_mapping_root 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "new_mapping_root 指针其余字段".to_string())
            }
        }
        271..=278 => (true, "new_tree_identifier_watermark".to_string()),
        279..=286 => (true, "new_rollback_floor".to_string()),
        287..=372 => {
            let local = offset - 287;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "new_instance_table 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "new_instance_table 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "new_instance_table 指针其余字段".to_string())
            }
        }
        373..=458 => {
            let local = offset - 373;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "new_allocation_record_tree_root 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "new_allocation_record_tree_root 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "new_allocation_record_tree_root 指针其余字段".to_string())
            }
        }
        459..=462 => (true, "new_unmount_marker（卸载记号，位 0）".to_string()),
        463..=470 => (true, "fsid".to_string()),
        471..=486 => (true, "MAC 留位（恒 0）".to_string()),
        _ => {
            let relative = offset - JOURNAL_HEADER_BYTES;
            let entry_index = (relative / JOURNAL_NAMED_ENTRY_BYTES) as usize;
            let entry_offset = relative % JOURNAL_NAMED_ENTRY_BYTES;
            let tag = units.get(entry_index).map_or("?", |(_, unit, _)| unit.descriptive_tag());
            if (10..14).contains(&entry_offset) {
                return (true, format!("点名项 {entry_index}（{tag}）第 0 条位置条目单元校验和"));
            }
            if (24..28).contains(&entry_offset) {
                return (true, format!("点名项 {entry_index}（{tag}）第 1 条位置条目单元校验和"));
            }
            (true, format!("点名项 {entry_index}（{tag}）内偏移 {entry_offset}（非校验和字段）"))
        }
    }
}

/// 量 3 的统一入口：按结构标签分派到对应的 explain 函数。
pub(super) fn explain_offset(catalog: &StructureCatalog, structure: &str, offset_in_structure: u64) -> (bool, String) {
    if structure == "root_record" {
        return explain_root_record_offset(offset_in_structure);
    }
    if structure == "journal_record" {
        return explain_journal_record_offset(catalog.units, offset_in_structure);
    }
    for (_, unit, _) in catalog.units {
        if unit.descriptive_tag() == structure {
            return explain_unit_offset(*unit, offset_in_structure);
        }
    }
    (false, format!("标签 {structure} 不在候选表里"))
}

/// 问题单第 1 行「够判条件」点名的六处（重跑登记第一节）：每处至少一段差异才够判。
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct ChangeCountBullets {
    pub(super) offset_88_covered: bool,
    pub(super) leaf_checksum_covered: bool,
    pub(super) pointer_to_leaf_checksum_covered: bool,
    pub(super) inode_root_checksum_covered: bool,
    pub(super) tree_table_covered: bool,
    pub(super) root_record_checksum_covered: bool,
}

impl ChangeCountBullets {
    pub(super) fn all_covered(&self) -> bool {
        self.offset_88_covered && self.leaf_checksum_covered && self.pointer_to_leaf_checksum_covered && self.inode_root_checksum_covered && self.tree_table_covered && self.root_record_checksum_covered
    }
    pub(super) fn record(&mut self, structure: &str, offset_in_structure: u64, length: u64) {
        match structure {
            "inode_leaf" => {
                let record_start = PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES;
                if overlaps(offset_in_structure, length, record_start + 88, 8) {
                    self.offset_88_covered = true;
                }
                if overlaps(offset_in_structure, length, 10, 32) || overlaps(offset_in_structure, length, 89, 4) {
                    self.leaf_checksum_covered = true;
                }
            }
            "inode_root" => {
                // 结构内偏移 225 / 239（重跑登记第七节乙类，entries_start 131 + pointer_start 34 + {60,74}）。
                if overlaps(offset_in_structure, length, 225, 4) || overlaps(offset_in_structure, length, 239, 4) {
                    self.pointer_to_leaf_checksum_covered = true;
                }
                if overlaps(offset_in_structure, length, 10, 32) || overlaps(offset_in_structure, length, 92, 4) {
                    self.inode_root_checksum_covered = true;
                }
            }
            "tree_table" => self.tree_table_covered = true,
            "root_record" if overlaps(offset_in_structure, length, 138, 32) => {
                self.root_record_checksum_covered = true;
            }
            _ => {}
        }
    }
}

// ───────────────────────── 装置自己补的取法：空白清单 ─────────────────────────

/// 每一条是仓里没有条款、装置不得不自己取一个值才写得出字节的地方。文字里不用空格，好让结果行按空格切段。
pub(super) const GAPS: &[(&str, &str)] = &[
    ("G1", "已收口（2026-09-13 D8已定项11 + D18已定项16/18）：码2头=86+2×key宽，含29字节预留位115+2×key宽（inode与树表131、分配135、记账159、extent163、映射169）；跑出空白那天三处预想数（84、68+44、72/81/90三档）互不相等"),
    ("G2", "已收口（2026-09-14用户定案，C325还清）：key宽字段从81+2k挪到**偏移51**，它自己的偏移不依赖k⇒扫描期按[0,51)这段定长前缀读出k、一次定出头末端，I-2.4对码2从此直接判得了；装置的parse_index_node不再收调用方给的key宽"),
    ("G3", "已收口（2026-09-14用户定案改写D19已定项10）：映射key**一律27**——码2/码3的25字节末尾补零到27、条目一律55；2026-09-13那一版的「按类27/25不补齐、条目55/53」作废"),
    ("G4", "根记录字段序：D22已定项7的表与字节表七的表行序不同（水位、F、校验和、实例表指针四行的先后），两处都没写偏移；2026-09-14加的算法类型1+nonce12+MAC16同样只说「放在中央映射树根指针之后」、没写偏移；装置按D22的表序"),
    ("G5", "已收口（2026-09-13 D18已定项17）：32字节校验和字段里放CRC32C4字节+28字节零，自证结构罩整个槽含补齐、字段按零参与；跑出空白那天装置取的是SHA-256"),
    ("G6", "已收口（2026-09-13 D23已定项19②，2026-09-14改写口径，2026-10-02头宽改487）：previous_hash=CRC32C(**本实例内逻辑前一条**记录的487字节头，header_csum那32字节按零参与)，本实例写出的第一条恒0、不读盘；已定项17列的改动字节没提这一处，见G26"),
    ("G7", "已收口（2026-09-13 D23已定项15）：记录头加新根段364（树表指针86+映射树根指针86+树ID水位8+回退下界F8+实例表指针86+分配记录树根指针86+卸载记号4），施加一条记录=把所选根这七个字段换成记录里的；实例代号与txg照记录头、实例表指针、卸载记号、分配记录树根指针从新根段来⇒journal从此承重（journal_effect差异态3个）"),
    ("G8", "码1映射key靠D16的事务切分纪律（一个事务最多写一个单元的用户数据）才唯一：同一事务写两个数据单元key相同；按D23已定项7一条记录一个事务，每个数据单元一条4KiB记录"),
    ("G9", "journal两份镜像何时算「记录在」没有条款（一份合法即在、还是两份都要）；装置取任一份合法即在"),
    ("G10", "里程碑步4验收「把位置提示改坏、经映射仍读到」在字节层做不到：提示住父节点、父节点被树表指针里的整单元CRC罩着，改坏提示先红父节点；要验的是搬走单元那条路"),
    ("G11", "里程碑步1验收「同参数两次mkfs逐字节相同」与字节表「fsid=mkfs随机」矛盾：fsid必须是mkfs参数"),
    ("G12", "已收口（2026-09-26 D15已定项4）：incompat位1=第一条纯SSD布局线，位0空出、没登记的位读者一律拒；装置从mkfs起置位1，位图其余全0"),
    ("G13", "已收口（2026-09-13 D8已定项11）：条目数2与条目宽2进头（偏移82+2k、84+2k），不再住载荷内部布局"),
    ("G14", "实例表链指针行宽2026-09-13从64改成88（C304，D18已定项11）；2026-09-14指针从83变86之后那3字节预留被吃掉，记录宽仍88⇒**这条记录没有余量了**，指针再宽一个字节INSTANCE_ROW_BYTES就要跟着涨（打包记录定长，kind0与kind1必须同宽），而条款里没有这一句"),
    ("G15", "已收口（2026-09-13 D18已定项18）：码2的声明长度=条目数×条目宽，条目区之后的补齐区恒0且参与载荷CRC，I-2.3射程扩到码2"),
    ("G16", "已收口（2026-09-13 D19已定项9）：出生序号从0起、同一棵树内码2与码3共用一个计数、换checkpoint清零、同一checkpoint里重写换新号"),
    ("G17", "已收口（2026-09-13 C313用户定案）：FUA写算段边界；装置主臂按它枚举，另一读法只报数不判"),
    ("G18", "D23已定项12按「12项事务恰占1条记录」算余量，而D16的事务切分纪律让一次带8个数据单元的fsync至少是8个事务、8条记录；两条已定条款对同一负载算出的记录数不同"),
    ("G19", "mkfs种根的25次操作（21写+两道池屏障按设备记成4步，段序列14+1+1+1+4、16402个崩溃状态，D22已定项8第3条含两块盘各清[0,根环基址)、根环三区域与journal环共10步）不在层0枚举里：装置从mkfs之后的池起枚举（取号、暖机与事务），mkfs的崩溃状态没有任何东西判；段序列另发一行钉住，与layout/01-first-txn.md八「mkfs种根」那一行登记的真值一致"),
    ("G20", "已收口（2026-09-14用户定案，C321还清）：映射条目回到一宽55⇒码2头那个u16的条目宽字段够用，声明长度=条目数×条目宽原样成立；装置删掉了「条目宽0当变长哨兵」那套自创取法，parse_index_node对条目宽0一律判结构错"),
    ("G21", "取号那一步（D23已定项16：第一次可写挂载写每一份系统配置之后才动单元）不是根槽写路径，layout/01-first-txn八那张表罩不到它；屏障怎么放没有条款，装置按最少屏障取「不另加屏障，靠暖机第一次空发布开头那道屏障收段」⇒段序列独占一行[system_configuration_slot×2]"),
    // E142 第十七次跑（问题单第 8 行，R19）：这条原来把空闲字节、全空聚簇段数、runs 三个数写成字面
    // （3472670720、3310、4），跟 `name=accounting` 现算出来的数各走各的、能悄悄漂开。占位符
    // `{FREE_BYTES}`/`{EMPTY_CLUSTER_SEGMENTS}`/`{FRAGMENTATION_RUNS}` 在打印这一条时（main() 里的
    // GAPS 循环，特判 id == "G22"）替换成 `output.accounting_entries` 现算的同一份值，与
    // `name=accounting` 那一行同一个来源；文本数字自检（Q142.28/29）在同一行另加三个绑定字段核对。
    ("G22", "**仍欠着**（C323，2026-09-14加注）：镜像大小（单元区的末端）全仓仍没有条款，D23已定项19③只定了「环≤设备容量÷4」⇒它给出容量的**下界**而不是值；装置按mkfs参数取4GiB（1GiB装不下默认768MiB的环）并在name=config里报出来，空闲字节{FREE_BYTES}、全空聚簇段数{EMPTY_CLUSTER_SEGMENTS}、runs{FRAGMENTATION_RUNS}三个数都随它变"),
    ("G23", "已收口（2026-09-14用户定案，C324还清）：码3打包容器按「数据单元」那一档取落点（起点32768对齐、两槽都空）；连同D3已定项10⑤的bump次序（树ID升序、树内先叶后根、映射树倒数第二、树表最末）⇒t2 extent根拿50240、游标停在50241而t3要对齐⇒t3拿50242–50243，**槽50241空着**、runs因此从3变4"),
    ("G24", "树表条目的「头ID」（D5已定项9，2026-09-14用户定案）只说了inode树写自己、extent树写65537、其余写0，没说**这个数从哪来**：第一版只有一个可写头，装置按「inode树的树ID就是头ID」写；多头之后头ID与树ID还是不是一回事，条款没答"),
    ("G25", "journal记录头2026-09-14加的fsid8与MAC16，条款只说fsid「与单元头同口径」（系统配置fsid的低8字节）、MAC第一版全0；**读者拿fsid做什么没写**——装置按I-1.4把fsid不符的记录整条丢掉（不进重放前缀），而「丢掉」与「判损坏断链」在条款里分不出来"),
    ("G26", "E142第十四次跑F1a：D23已定项17写标志改动「改新池新建文件的字节（w1、w4、t9三条记录的这一字节与头部校验和）」，没列w4、t9的反向链；而已定项10「罩前一条的整个记录头」使反向链必然随头宽与标志字节的改动跟着变（w1是本实例第一条、反向链恒0，不受影响）——不作废，如实记这一处条款没提到"),
    ("G27", "5.2 ι：D8已定项14第395行只说内部条目「key10+子指针86、按key排」，没写这个key取「孩子按位置罩的段起点」还是「孩子子树里最小那条记录的key」；装置取ι甲（段起点），ι乙（记录最小key）登记成第二段的读法对照函数，主读法不建变体分支"),
    ("G28", "5.2 γ：D18已定项2射程与D8已定项14第385行的[k×W,(k+1)×W)都没写分配记录树叶/层级1节点区间的末端是闭区间还是半开——装置取γ甲（闭区间，D18定案句「[min_key,max_key]」与「最大key」更直接），γ乙（半开右端）登记成第二段的读法对照函数"),
    ("G29", "5.2 ε：D8已定项14第398/400行「标签0全零、只有读者认、写者不写」与「上段叶罩143个inode」两句字面上互相拉扯——装置取ε甲（稀疏，只写有文件的条目），ε乙（143格全写、空格标签0）登记成第二段的读法对照函数"),
    ("G30", "5.2 ζ：条款只说「上段按inode号的位置寻址」，没写「根的层级怎么算」；装置取「最低的、罩得住最大inode号的层级」（这个装置的几何inode1⇒层级0，根即叶），没有找到第二种写得出字节的读法，不登记变体"),
    ("G31", "5.2 θ：D23已定项17只说「一项一个单元」，没写点名项的次序；装置取「照单元的先后（数据单元在前，其后照bump次序）」，映射条目按D8已定项11的映射key排；这个装置的几何两种候选次序同值，不登记变体"),
    ("G32", "5.2 κ：条款没说层级1节点的段伸出盘外时区间末端截不截到盘末；这个装置的几何（两块4GiB盘）每盘只写层级1第0个节点、整段在盘内，走不到这一格，不登记读法"),
    ("F2", "D3已定项10⑤的落点表（t5『分配记录树根兼叶』落50245、t8树表落50248）与字节表零那一节写清单表的t5那一行都还是按位置寻址之前的布局，与D8已定项14『4GiB×2时根在第2层、树高3』字面对不上：条款自己两处说法不一致，不作废、不停机，这一次的写清单照D8已定项14与D3已定项10⑤的规则句（bump次序）写，两处旧表由书记员另行改"),
];

#[cfg(test)]
#[path = "structure_catalog_and_offset_explanations_tests.rs"]
mod tests;
