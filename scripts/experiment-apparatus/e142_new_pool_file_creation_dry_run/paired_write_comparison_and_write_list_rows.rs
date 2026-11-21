//! 写行的解析、成对写的比较、比对器的字节阳性对照、窗口段大小、写列表行与第二种写法的字段行。

use e7_index_bench::usize_from_u64;
use super::*;

/// 从一份 `E7RESULT name=<marker> device=… offset=… length=… sha256=… [kind=…] [hexadecimal=…]` 格式的文本里
/// 抠出按 (设备, 偏移, 长度) 配对要用的字段。`crates/` 侧的导出与 arm O 的历史留存产物都是这个形状。
#[derive(Clone, Debug)]
pub(super) struct ParsedWriteLine {
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) sha256: String,
    hexadecimal: Option<String>,
}

pub(super) fn parse_write_lines(text: &str, marker: &str) -> Vec<ParsedWriteLine> {
    text.lines()
        .filter(|line| line.contains(marker))
        .map(|line| {
            let fields = parse_result_line(line);
            ParsedWriteLine {
                device: fields.get("device").and_then(|value| value.parse().ok()).unwrap_or(0),
                offset: fields.get("offset").and_then(|value| value.parse().ok()).unwrap_or(0),
                length: fields.get("length").and_then(|value| value.parse().ok()).unwrap_or(0),
                sha256: fields.get("sha256").cloned().unwrap_or_default(),
                hexadecimal: fields.get("hexadecimal").cloned(),
            }
        })
        .collect()
}

/// Q142.1 的核心判定：sha256 相等 ⇒ 全等；不等就用整段十六进制定位第一处差异——抽出成独立函数是因为
/// `main()` 本身不可测（E142 第十一次跑修订第 1 条已经在 V1 上踩过一次这个教训），返回 (equal, first_diff_offset, mismatch_bytes)。
pub(super) fn compare_paired_write(model_bytes: &[u8], model_sha256: &str, impl_line: &ParsedWriteLine) -> (bool, Option<u64>, Option<u64>) {
    if impl_line.sha256 == model_sha256 {
        return (true, None, Some(0));
    }
    let impl_bytes = impl_line.hexadecimal.as_deref().map(hex_decode).unwrap_or_default();
    let (first_diff_offset, mismatch_bytes) = byte_diff_summary(&impl_bytes, model_bytes);
    (false, first_diff_offset, mismatch_bytes)
}

/// Q142.1 配对：在 crates 导出的写清单里找与 (设备, 偏移, 长度) 精确相等的那一条，返回它的下标。
/// **不按名字配对**（R3）：按名字配对是旧「量 5」的失败模式——两块设备在同一偏移各写不同内容时，
/// 按名字配对会分不清是哪一块设备，第九节 M108 钉的就是这一条。
pub(super) fn find_matching_impl_write(impl_writes: &[ParsedWriteLine], device: u32, offset: u64, length: u64) -> Option<usize> {
    impl_writes.iter().position(|line| line.device == device && line.offset == offset && line.length == length)
}

/// crates 一侧配不上模型任何一次写的下标，按原始次序——Q142.1 的「配不上的两边都报」，不能只从模型一侧遍历
/// （第九节 M109：比对器只从模型一侧遍历会漏掉「crates 多写了一次」这一类差异）。
pub(super) fn unmatched_crates_indices(impl_matched: &[bool]) -> Vec<usize> {
    impl_matched.iter().enumerate().filter(|(_, matched)| !**matched).map(|(index, _)| index).collect()
}

/// extent 上段叶序号（D8 已定项 14 第 401 行，唯一写法：写路径与读路径与单测共用）：第 k 片
/// （从 0 起）= inode ÷ 143（整除）。M138 的会红检查（143→144）落在这里。
pub(super) fn extent_upper_leaf_index(inode: u64) -> u64 {
    inode / EXTENT_TREE_UPPER_LEAF_INODES
}

/// extent 树上段叶的 key 区间（D8 已定项 14 第 401 行，唯一写法：写路径与读路径共用）：第 k 片
/// （从 0 起，k = inode ÷ 143）罩 `[143k, 143k + 142]`，闭区间，第三分量恒 2^64 − 1（第七节 A7）。
pub(super) fn extent_upper_leaf_positional_key_range(upper_leaf_index: u64) -> (Vec<u8>, Vec<u8>) {
    let first_inode_in_span = EXTENT_TREE_UPPER_LEAF_INODES * upper_leaf_index;
    let last_inode_in_span = first_inode_in_span + EXTENT_TREE_UPPER_LEAF_INODES - 1;
    let mut smallest = [0u8; 24];
    smallest[8..16].copy_from_slice(&first_inode_in_span.to_le_bytes());
    let mut largest = [0u8; 24];
    largest[8..16].copy_from_slice(&last_inode_in_span.to_le_bytes());
    largest[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    (smallest.to_vec(), largest.to_vec())
}

/// P2（第五节 5.3）：比对器分得出一个字节——在臂 N16 自己的镜像上，四个点各翻一位，喂给同一条比对路
/// （`compare_paired_write`/`find_matching_impl_write`，与真的跟 crates 比时用的是同一对函数），
/// 必须恰好命中那一个区域、那一个偏移、且只翻中 1 字节。四个点：
/// (a) 盘 0 那一份「罩盘 0 的层级 1 节点」第 0 条条目（唯一写法：稀疏，只有这一条）子指针的第一个字节；
/// (b) 根 max_key 末字节；(c) extent 上段叶第一条条目的标签字节；(d) journal 记录第 5 个点名项首字节。
pub(super) fn run_comparator_byte_positive_control(emitter: &mut Emitter, arm_label: &str, writes: &[ModelWindowWrite]) {
    let allocation_key_start = INDEX_NODE_KEY_WIDTH_OFFSET + 1 + ALLOCATION_KEY_BYTES;
    let extent_entries_start = index_node_header_bytes(24) + usize_from_u64(NONCE_MAC_RESERVED_BYTES);
    let journal_named_item_5_offset = usize_from_u64(JOURNAL_HEADER_BYTES) + 4 * usize_from_u64(JOURNAL_NAMED_ENTRY_BYTES);
    let level1_entries_start = index_node_header_bytes(ALLOCATION_KEY_BYTES) + usize_from_u64(NONCE_MAC_RESERVED_BYTES);
    // D8 已定项 14 第 395 行（唯一写法）：内部节点只写有孩子的格，这个装置的场景每个层级 1 节点只有一个孩子
    // ⇒ 条目区里第 0 条（唯一一条）就是它，子指针紧跟在 10 字节 key 之后。
    let level1_pointer_first_byte = Some(level1_entries_start + ALLOCATION_KEY_BYTES);

    let points: [(&str, &str, u32, Option<usize>); 4] = [
        ("a_level1_pointer_first_byte", "allocation_internal_of_device_0", 0, level1_pointer_first_byte),
        ("b_root_largest_key_last_byte", "allocation_root", 0, Some(allocation_key_start + ALLOCATION_KEY_BYTES - 1)),
        ("c_extent_leaf_entry_tag_byte", "extent_root", 0, Some(extent_entries_start + 24)),
        ("d_journal_named_item_5_first_byte", "journal_record", 0, Some(journal_named_item_5_offset)),
    ];

    for (point_name, target_region, target_device, offset_in_region) in points {
        let Some(offset_in_region) = offset_in_region else {
            emit(emitter, &format!("name=positive_control_p2 arm={arm_label} point={point_name} region={target_region} device={target_device} matches_injection=false reason=no_nonempty_cell_found"));
            continue;
        };
        let synthetic: Vec<ParsedWriteLine> = writes
            .iter()
            .map(|write| {
                let mut bytes = write.bytes.clone();
                if write.region == target_region && write.device == target_device {
                    assert!(offset_in_region < bytes.len(), "P2 注入点必须落在区域内（{point_name}）");
                    bytes[offset_in_region] ^= 0xFF;
                }
                ParsedWriteLine { device: write.device, offset: write.offset, length: bytes.len() as u64, sha256: sha256_hex(&bytes), hexadecimal: Some(hex_bytes(&bytes)) }
            })
            .collect();
        let mut flagged: Vec<(String, u32, u64, u64)> = Vec::new();
        for write in writes {
            let sha = sha256_hex(&write.bytes);
            if let Some(index) = find_matching_impl_write(&synthetic, write.device, write.offset, write.bytes.len() as u64) {
                let (equal, first_diff_offset, mismatch_bytes) = compare_paired_write(&write.bytes, &sha, &synthetic[index]);
                if !equal {
                    flagged.push((write.region.to_string(), write.device, first_diff_offset.unwrap_or(u64::MAX), mismatch_bytes.unwrap_or(0)));
                }
            }
        }
        let matches_injection =
            flagged.len() == 1 && flagged[0].0 == target_region && flagged[0].1 == target_device && flagged[0].2 == offset_in_region as u64 && flagged[0].3 == 1;
        emit(emitter, &format!(
            "name=positive_control_p2 arm={arm_label} point={point_name} region={target_region} device={target_device} injected_offset={offset_in_region} regions_flagged={} matches_injection={matches_injection}",
            flagged.len()
        ));
    }
}

/// Q142.2：按 (是否屏障, 是否 FUA) 从一份写清单文本里重建段大小——切法与 `split_into_segments`
/// 的 `fua_is_boundary = true` 同一条规则：FUA 写自己关掉一段，屏障关掉它前面还没关的那一段。
pub(super) fn window_segment_sizes_from_dump(text: &str) -> Vec<usize> {
    enum Step {
        Write { is_fua: bool },
        Barrier,
    }
    let mut steps: Vec<(usize, Step)> = Vec::new();
    for line in text.lines() {
        let fields = parse_result_line(line);
        let Some(step) = fields.get("step").and_then(|value| value.parse::<usize>().ok()) else {
            continue;
        };
        if line.contains("name=device_region_bytes ") {
            let is_fua = fields.get("kind").map(String::as_str) == Some("write_fua");
            steps.push((step, Step::Write { is_fua }));
        } else if line.contains("name=window_barrier ") {
            steps.push((step, Step::Barrier));
        }
    }
    steps.sort_by_key(|(step, _)| *step);
    let mut segments = Vec::new();
    let mut current = 0usize;
    for (_, kind) in steps {
        match kind {
            Step::Write { is_fua } => {
                current += 1;
                if is_fua {
                    segments.push(current);
                    current = 0;
                }
            }
            Step::Barrier => {
                if current > 0 {
                    segments.push(current);
                    current = 0;
                }
            }
        }
    }
    if current > 0 {
        segments.push(current);
    }
    segments
}

fn catalog_unit_at_offset<'catalog>(catalog: &StructureCatalog<'catalog>, offset: u64) -> Option<&'catalog (SlotNumber, TransactionUnit, Vec<u8>)> {
    catalog.units.iter().find(|(slot, _, _)| slot.device_offset().0 == offset)
}

/// Q142.3 区域级写清单一行：非码 2 的区域（数据单元、打包容器、journal 记录、根记录、系统配置）
/// key 相关字段写 `none`。`writer=transaction` 恒定——R2 的窗口本来就只有这一条路径的写。
pub(super) fn write_list_row(catalog: &StructureCatalog, step: usize, region: &str, device: u32, offset: u64, bytes: &[u8]) -> String {
    let sha = sha256_hex(bytes);
    let Some((slot, unit, _)) = catalog_unit_at_offset(catalog, offset) else {
        return format!(
            "name=write_list_row step={step} writer=transaction unit={region} class=other bytes={} slot=none tree=none device={device} offset={offset} level=none entries=none entry_width=none declared_length=none min_key=none max_key=none birth_sequence=none sha256={sha}",
            bytes.len()
        );
    };
    let (class, _) = unit.class_and_key_width();
    let class_name = match class {
        UNIT_CLASS_DATA => "data",
        UNIT_CLASS_PACKED => "packed",
        UNIT_CLASS_INDEX_NODE => "index_node",
        _ => "other",
    };
    if class == UNIT_CLASS_INDEX_NODE {
        if let Ok(header) = parse_index_node(bytes) {
            let entry_width = header.entries.first().map_or(0, Vec::len);
            return format!(
                "name=write_list_row step={step} writer=transaction unit={region} class={class_name} bytes={} slot={} tree={} device={device} offset={offset} level={} entries={} entry_width={entry_width} declared_length={} min_key={} max_key={} birth_sequence={} sha256={sha}",
                bytes.len(), slot.0, header.tree.0, header.level, header.entries.len(), header.entries.len() * entry_width,
                hex_bytes(&header.smallest_key), hex_bytes(&header.largest_key), header.birth_sequence.0
            );
        }
    }
    format!(
        "name=write_list_row step={step} writer=transaction unit={region} class={class_name} bytes={} slot={} tree={} device={device} offset={offset} level=none entries=none entry_width=none declared_length=none min_key=none max_key=none birth_sequence=none sha256={sha}",
        bytes.len(), slot.0, unit.tree().0
    )
}

/// Q142.17 码 2 节点的字段级行：头部按固定偏移（D8（核心索引结构） 已定项 11、
/// D18（块里携带什么信息） 已定项 18）逐字段列出；条目区（唯一写法：稀疏，D8 已定项 14 第 395 行）
/// 每条都列，不再有空格；条目区之后到单元末尾的补齐区另出一行——字段偏移连续覆盖整个 16384 字节，不留缝。
pub(super) fn code2_field_rows(unit_tag: &str, device: u32, bytes: &[u8], header: &IndexNodeHeader) -> Vec<String> {
    let key_width = header.key_width;
    let header_end = 86 + 2 * key_width;
    let entries_start = header_end + usize_from_u64(NONCE_MAC_RESERVED_BYTES);
    let header_fields: [(&str, usize, usize); 19] = [
        ("magic", 0, 4),
        ("format_version", 4, 6),
        ("class", 6, 7),
        ("flags", 7, 8),
        ("declared_length", 8, 10),
        ("seal_header_checksum", 10, 42),
        ("tree_id", 42, 50),
        ("level", 50, 51),
        ("key_width", 51, 52),
        ("smallest_key", 52, 52 + key_width),
        ("largest_key", 52 + key_width, 52 + 2 * key_width),
        ("birth_txg", 52 + 2 * key_width, 60 + 2 * key_width),
        ("fsid", 60 + 2 * key_width, 68 + 2 * key_width),
        ("instance", 68 + 2 * key_width, 72 + 2 * key_width),
        ("birth_sequence", 72 + 2 * key_width, 76 + 2 * key_width),
        ("payload_crc", 76 + 2 * key_width, 80 + 2 * key_width),
        ("reserved", 80 + 2 * key_width, 82 + 2 * key_width),
        ("entry_count", 82 + 2 * key_width, 84 + 2 * key_width),
        ("entry_width", 84 + 2 * key_width, 86 + 2 * key_width),
    ];
    let mut out: Vec<String> = header_fields
        .iter()
        .map(|(field, start, end)| {
            format!(
                "name=field_row unit={unit_tag} device_of_copy={device} field={field} offset_in_unit={start} width={} value={} source=clause",
                end - start,
                hex_bytes(&bytes[*start..*end])
            )
        })
        .collect();
    out.push(format!(
        "name=field_row unit={unit_tag} device_of_copy={device} field=unit_reserved offset_in_unit={header_end} width={} value={} source=clause",
        entries_start - header_end,
        hex_bytes(&bytes[header_end..entries_start])
    ));

    let entry_width = header.entries.first().map_or(0, Vec::len);
    for (index, entry) in header.entries.iter().enumerate() {
        let offset_in_unit = entries_start + index * entry_width;
        let key_bytes = &entry[..key_width.min(entry.len())];
        out.push(format!(
            "name=field_row unit={unit_tag} device_of_copy={device} field=entry_{index}_key offset_in_unit={offset_in_unit} width={} value={} source=clause",
            key_bytes.len(),
            hex_bytes(key_bytes)
        ));
        if entry.len() > key_width {
            out.push(format!(
                "name=field_row unit={unit_tag} device_of_copy={device} field=entry_{index}_payload offset_in_unit={} width={} value={} source=clause",
                offset_in_unit + key_width,
                entry.len() - key_width,
                hex_bytes(&entry[key_width..])
            ));
        }
    }
    // 条目区之后到单元末尾的补齐区（D18 已定项 18：恒 0 且参与载荷 CRC）——Q142.17 要求字段偏移连续覆盖
    // 整个 16384 字节，这一行补上条目区结束到单元末尾这一段。
    let entries_end = entries_start + header.entries.len() * entry_width;
    if entries_end < usize_from_u64(NODE_BYTES) {
        out.push(format!(
            "name=field_row unit={unit_tag} device_of_copy={device} field=entries_padding offset_in_unit={entries_end} width={} value={} source=clause",
            usize_from_u64(NODE_BYTES) - entries_end,
            hex_bytes(&bytes[entries_end..usize_from_u64(NODE_BYTES)])
        ));
    }
    out
}

#[cfg(test)]
#[path = "paired_write_comparison_and_write_list_rows_tests.rs"]
mod tests;
