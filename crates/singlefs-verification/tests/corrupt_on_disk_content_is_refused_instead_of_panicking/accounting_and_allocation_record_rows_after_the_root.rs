//! 账务行与分配记录行的代号晚于根、账务条目窄于字段表、条目个数不对时，恢复与可写挂载各报哪一个错、先判哪一个。

use super::*;

// ─── 记账条目宽先于记账条目数：台阶行解不开就认不出 ───

/// 两块内存盘上第一个文件（txg 3）之后再建一个空 inode（txg 4）：记账树单个叶根 16 条，池级 3 行、每盘 6 行之外，
/// 多一级第一个文件那次留下的 inode 号水位台阶行（代段 3，早于这一版的 txg；D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。
fn pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row() -> Devices {
    let (mut devices, mut allocator, first) = pool_and_its_allocator_after_the_first_file();
    let parameters = parameters();
    let second = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut allocator,
            &first,
            1,
            WRITE_TIME_SECONDS + 60,
            first.root.instance,
        )
        .expect("建一个空 inode")
    };
    let step_rows_older_than_this_version = second
        .accounting_entries
        .iter()
        .filter(|entry| {
            entry.statistic == STATISTIC_INODE_WATERMARK
                && entry.generation < second.root.checkpoint_txg
        })
        .count();
    assert_eq!(
        (
            second.root.checkpoint_txg,
            second.accounting_entries.len(),
            step_rows_older_than_this_version,
            second.accounting_tree.node_count()
        ),
        (CheckpointTxg(4), 16, 1, 1),
        "txg 4 那一版的记账树：单个叶根 16 条，其中 1 条是代段早于 4 的台阶行"
    );
    devices
}

/// 码 2 节点自述的条目宽缩到 key 宽、每条只留 key：条目数不变，条目区紧排、空出来的那一段清零，声明长度 = 条目数 × key 宽
/// （条目数住 82 + 2k、条目宽住 84 + 2k，声明长度住偏移 8）。
fn narrow_every_entry_of_an_index_node_to_its_key(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        node[header_end - 4],
        node[header_end - 3],
    ]));
    let entry_ranges: Vec<std::ops::Range<usize>> = (0..entry_count)
        .map(|entry_index| entry_range_in_node(node, entry_index))
        .collect();
    let entries_start = header_end + 29;
    let entries_end_before = entry_ranges.last().map_or(entries_start, |range| range.end);
    // 第 i 条的 key 挪到 i × key 宽：key 宽小于原条目宽，写到的那一段止于 (i + 1) × key 宽，盖不到还没挪的那几条。
    for (entry_index, range) in entry_ranges.iter().enumerate() {
        node.copy_within(
            range.start..range.start + key_width,
            entries_start + entry_index * key_width,
        );
    }
    node[entries_start + entry_count * key_width..entries_end_before].fill(0);
    let narrowed_entry_width = u16::try_from(key_width).expect("key 宽是 1 字节的字段");
    node[header_end - 2..header_end].copy_from_slice(&narrowed_entry_width.to_le_bytes());
    let declared_length =
        u16::try_from(entry_count * key_width).expect("缩之后的条目区比缩之前短，装得进 2 字节");
    node[8..10].copy_from_slice(&declared_length.to_le_bytes());
}

/// 最新那条根（txg 4）的记账树根条目宽 34 缩到 key 宽 22、16 条每条只留 key，节点与指着它的每一道校验和重算。
fn pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys() -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| narrow_every_entry_of_an_index_node_to_its_key(node),
    );
    devices
}

/// 记账条目窄于字段表那个成员：22 对 34。
fn accounting_entry_narrower_than_its_field_table() -> RecoveryFailure {
    RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
        what: "记账条目",
        entry_bytes: usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    }
}

/// 场景：第一个文件之后再建一个空 inode（txg 4），记账树根 16 条里有一级代段 3 的 inode 号水位台阶行；根节点自述的条目宽缩到 key 宽、
/// 每条只留 key（条目数 16 不变），节点与指着它的每一道校验和重算。
/// 预期：只读恢复交回 `EntryWidthDiffersFromItsFieldTable`（记账条目，22 对 34）。台阶行要解开才认得出（统计量标签与代段都在条目里）：
/// 数记账条目那一判排在条目宽之前时，解不开的台阶行被算进 3 + 6 × 盘数，16 ≠ 15，报成「记账条目数（inode 号水位的台阶行不算）
/// 不是 3 + 6 × 盘数」。坏盘战役 `every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking` 的
/// 「条目宽：记账树根」那一种是同一个坏法，这条钉住次序。
#[test]
fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery() {
    let devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), CheckpointTxg(4))),
            failure: accounting_entry_narrower_than_its_field_table(),
        },
        "只读恢复在 txg 4 那条根上报条目宽，不报条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回 `EntryWidthDiffersFromItsFieldTable`（记账条目，22 对 34）：重建那一版读记账树时同样先核条目宽，
/// 后数条目（判法与上一条同一个理由）。
#[test]
fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount(
) {
    let mut devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == accounting_entry_narrower_than_its_field_table()),
        "可写挂载报条目宽，不报条目数：{refusal:?}"
    );
}

// ─── 记账条目逐条判代与 seq 先于记账条目数：代写到根之后的台阶行认不出 ───

/// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]）最新那条根的 txg。
const TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW: CheckpointTxg = CheckpointTxg(4);

/// 台阶行的代改成的值：晚于根的 txg 4。
const STEP_ROW_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);

/// 码 2 记账叶根里恰好一条满足 `is_the_entry_to_change` 的条目交给 `change` 改，之后全部条目按 key 重排、节点头里的最小 / 最大 key
/// （从偏移 52 起，各一个 key 宽）换成首末两条的 key：条目宽、条目数不变，key 照旧严格递增，判全那一道（节点头、key 区间与次序）照过。
fn rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
    node: &mut [u8],
    is_the_entry_to_change: impl Fn(&AccountingEntry) -> bool,
    change: impl FnOnce(&mut AccountingEntry),
) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        node[header_end - 4],
        node[header_end - 3],
    ]));
    let entry_ranges: Vec<std::ops::Range<usize>> = (0..entry_count)
        .map(|entry_index| entry_range_in_node(node, entry_index))
        .collect();
    let mut entries: Vec<AccountingEntry> = entry_ranges
        .iter()
        .map(|range| {
            AccountingEntry::parse(&node[range.clone()]).expect("记账叶根的条目宽是 34，每条解得开")
        })
        .collect();
    let indices_to_change: Vec<usize> = (0..entries.len())
        .filter(|entry_index| is_the_entry_to_change(&entries[*entry_index]))
        .collect();
    assert_eq!(indices_to_change.len(), 1, "恰好一条条目要改");
    change(&mut entries[indices_to_change[0]]);
    entries.sort_by_key(AccountingEntry::sort_key);
    for (range, entry) in entry_ranges.iter().zip(&entries) {
        node[range.clone()].copy_from_slice(&entry.to_bytes());
    }
    let smallest_key = entries.first().expect("记账叶根至少一条").key_bytes();
    let largest_key = entries.last().expect("记账叶根至少一条").key_bytes();
    node[52..52 + key_width].copy_from_slice(&smallest_key);
    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&largest_key);
}

/// 最新那条根（txg 4）记账树根里那一级台阶行（inode 号水位、代段 3）的代改成 5（晚于根），按 key 重排到这一版自己那一行（代 4）之后；
/// 节点与指着它的每一道校验和重算。攻方原型 `research/prompts/m3-batch52-54-code-r1-opus-model/probes/corrupt_on_disk_k1_append.rs` 的 ② 那一形。
fn pool_whose_accounting_step_row_carries_a_generation_after_the_root() -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
                node,
                |entry| {
                    entry.statistic == STATISTIC_INODE_WATERMARK
                        && entry.generation < TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
                },
                |step_row| step_row.generation = STEP_ROW_GENERATION_AFTER_THE_ROOT,
            );
        },
    );
    devices
}

/// 记账条目逐条判那一个成员：代晚于根或 seq 为 0。
fn accounting_entry_generation_after_the_root_or_sequence_zero() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "记账条目的代晚于根或 seq 为 0",
    }
}

/// 场景：第一个文件之后再建一个空 inode（txg 4），记账树根 16 条里那一级台阶行（代段 3）的代改成 5，晚于根的 txg 4；条目按 key 重排、
/// 节点与指着它的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「记账条目的代晚于根或 seq 为 0」。数条目按「统计量是 inode 号水位且代早于根」认台阶行，
/// 代写到根之后就认不出、被算进 3 + 6 × 盘数（16 ≠ 15）：数条目排在逐条判代之前时，这一坏法报成「记账条目数（inode 号水位的台阶行不算）
/// 不是 3 + 6 × 盘数」（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第二节 K1-1、第三节 G1）。
#[test]
fn an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_recovery(
) {
    let devices = pool_whose_accounting_step_row_carries_a_generation_after_the_root();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_generation_after_the_root_or_sequence_zero(),
        },
        "只读恢复在 txg 4 那条根上报记账条目的代晚于根，不报条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「记账条目的代晚于根或 seq 为 0」：重建那一版读记账树时同样先逐条判代与 seq、后数条目
/// （判法与上一条同一个理由）。
#[test]
fn an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_writable_mount(
) {
    let mut devices = pool_whose_accounting_step_row_carries_a_generation_after_the_root();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == accounting_entry_generation_after_the_root_or_sequence_zero()),
        "可写挂载报记账条目的代晚于根，不报条目数：{refusal:?}"
    );
}

// ─── 分配记录逐条判代与跨度先于记账条目宽与逐条判、数记账条目、数映射条目 ───

/// 分配记录的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
const ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);

/// 这一版自己那一行 inode 号水位（代段 4）的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
const WATERMARK_ROW_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);

/// 分配记录树里 `pointer` 指着的节点与它下面整棵子树：叶里满足 `is_the_record_to_change` 的记录交给 `change` 改（只改不进 key 的字段），
/// 改过的节点重封、写回两份落点；内部节点里指着改过的孩子的那条内部条目（key 10 + 子指针 86），子指针两条位置条目的整单元校验和
/// 换成孩子的新值。改过的节点连同它的新整单元校验和推进 `rewritten_nodes`，改过的记录数加进 `changed_records`；
/// 交回这个节点改过之后的整单元校验和，没改过交回 `None`。
fn rewrite_allocation_records_in_the_subtree(
    devices: &mut Devices,
    pointer: &NodePointer,
    is_the_record_to_change: &dyn Fn(&AllocationRecord) -> bool,
    change: &dyn Fn(&mut AllocationRecord),
    rewritten_nodes: &mut Vec<(NodePointer, u32)>,
    changed_records: &mut usize,
) -> Option<u32> {
    let mut node = read_bytes(
        devices,
        pointer.locations[0].device,
        pointer.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let header = parse_index_node(&node).expect("分配记录树节点解得开");
    let key_width = usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10");
    let mut is_this_node_changed = false;
    for entry_index in 0..header.entries.len() {
        let entry_range = entry_range_in_node(&node, entry_index);
        if header.level == 0 {
            let mut record = AllocationRecord::parse(&node[entry_range.clone()])
                .expect("分配记录叶的条目宽是 20");
            if is_the_record_to_change(&record) {
                change(&mut record);
                node[entry_range].copy_from_slice(&record.to_bytes());
                *changed_records += 1;
                is_this_node_changed = true;
            }
            continue;
        }
        let child =
            NodePointer::read_from(&mut ByteReader::at(&node[entry_range.clone()], key_width));
        if let Some(child_checksum) = rewrite_allocation_records_in_the_subtree(
            devices,
            &child,
            is_the_record_to_change,
            change,
            rewritten_nodes,
            changed_records,
        ) {
            repoint_the_checksums_of_a_pointer(&mut node[entry_range], key_width, child_checksum);
            is_this_node_changed = true;
        }
    }
    if !is_this_node_changed {
        return None;
    }
    reseal_index_node(&mut node);
    let node_checksum = write_unit_at_both_locations(devices, &pointer.locations, &node);
    rewritten_nodes.push((*pointer, node_checksum));
    Some(node_checksum)
}

/// 把最新那条根的分配记录树里满足 `is_the_record_to_change` 的记录交给 `change` 改，再把指着改过的节点的每一道校验和都重算：
/// 父节点内部条目里的子指针（逐层到根）、树表条目里那条根指针、树表单元、中央映射树根里每个改过的节点那一条、
/// 根记录里树表、中央映射树根两条指针（带文件的那一版根记录里分配记录树根那一条是空的）。交回改了几条记录。
fn rewrite_allocation_records_of_the_newest_root(
    devices: &mut Devices,
    is_the_record_to_change: impl Fn(&AllocationRecord) -> bool,
    change: impl Fn(&mut AllocationRecord),
) -> usize {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut tree_table = read_bytes(
        devices,
        root.tree_table.locations[0].device,
        root.tree_table.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
    let entry_index = tree_table_header
        .entries
        .iter()
        .position(|entry| {
            TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == TREE_KIND_ALLOCATION)
        })
        .expect("树表里有分配记录树");
    let allocation_tree_root_pointer =
        TreeTableEntry::parse(&tree_table_header.entries[entry_index])
            .expect("树表条目解得开")
            .root;
    assert_eq!(
        root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "带文件的那一版分配记录树根只在树表条目里，根记录里那一条是空的"
    );
    let mut rewritten_nodes: Vec<(NodePointer, u32)> = Vec::new();
    let mut changed_records = 0;
    let root_node_checksum = rewrite_allocation_records_in_the_subtree(
        devices,
        &allocation_tree_root_pointer,
        &is_the_record_to_change,
        &change,
        &mut rewritten_nodes,
        &mut changed_records,
    )
    .expect("至少改了一条记录：它所在的叶到根这一路都改过");

    let entry_range = entry_range_in_node(&tree_table, entry_index);
    // 树表条目里根指针从条目偏移 14 起（树 ID 8 + 条目长度 2 + 种类 2 + flags 2）。
    repoint_the_checksums_of_a_pointer(&mut tree_table[entry_range], 14, root_node_checksum);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);

    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    for (node_pointer, node_checksum) in &rewritten_nodes {
        repoint_the_mapping_entry_of_an_index_node(
            &mut mapping_root,
            *node_pointer,
            *node_checksum,
        );
    }
    let mapping_root_checksum =
        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);

    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
    changed_records
}

/// 数记账条目那一个成员。
fn accounting_entry_count_is_not_three_plus_six_per_device() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",
    }
}

/// 逐条判分配记录那一个成员：跨度为 0 或代晚于根。
fn allocation_record_span_zero_or_generation_after_the_root() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
    }
}

/// 映射条目数那一个成员。
fn mapping_entry_count_is_not_one_per_mapped_unit() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "映射条目数不是 1 + extent 树、分配记录树、记账树、deadlist 树的节点数 + deadlist 条目数 + inode 叶容器数 + 数据单元数",
    }
}

/// 码 2 单个叶根里删掉第 `entry_index` 条条目：之后的条目前移一条、空出来的末条清零，条目数减一、声明长度（偏移 8，= 条目数 × 条目宽）跟着减，
/// 节点头里的最小 / 最大 key（从偏移 52 起，各一个 key 宽）换成新的首末两条的 key（条目的前 key 宽字节）。条目宽不变，key 照旧严格递增，
/// 判全那一道（节点头、key 区间与次序）照过。交回删掉的那条条目的字节；重封归调用方。
fn drop_one_entry_of_an_index_leaf_root(node: &mut [u8], entry_index: usize) -> Vec<u8> {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        node[header_end - 4],
        node[header_end - 3],
    ]));
    assert!(
        entry_count >= 2 && entry_index < entry_count,
        "叶根至少两条、删的那条在里面：删完还有首末两条可取 key（{entry_count} 条，删第 {entry_index} 条）"
    );
    let dropped_range = entry_range_in_node(node, entry_index);
    let last_range = entry_range_in_node(node, entry_count - 1);
    let entry_width = dropped_range.len();
    let dropped_entry = node[dropped_range.clone()].to_vec();
    node.copy_within(dropped_range.end..last_range.end, dropped_range.start);
    node[last_range].fill(0);
    let remaining_entry_count = entry_count - 1;
    let remaining_entry_count_field =
        u16::try_from(remaining_entry_count).expect("条目数比删之前少一条，装得进 2 字节");
    node[header_end - 4..header_end - 2]
        .copy_from_slice(&remaining_entry_count_field.to_le_bytes());
    let declared_length = u16::try_from(remaining_entry_count * entry_width)
        .expect("条目区比删之前短，装得进 2 字节");
    node[8..10].copy_from_slice(&declared_length.to_le_bytes());
    let first_range = entry_range_in_node(node, 0);
    let new_last_range = entry_range_in_node(node, remaining_entry_count - 1);
    let smallest_key = node[first_range.start..first_range.start + key_width].to_vec();
    let largest_key = node[new_last_range.start..new_last_range.start + key_width].to_vec();
    node[52..52 + key_width].copy_from_slice(&smallest_key);
    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&largest_key);
    dropped_entry
}

/// 最新那条根的中央映射树根（单个叶根）里 key 最大的那一条删掉，重封、写回两份落点，根记录里映射树根那条指针两条位置条目的校验和换成新值。
fn drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(
    devices: &mut Devices,
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let mapping_header = parse_index_node(&mapping_root).expect("映射树根解得开");
    assert_eq!(
        mapping_header.level, 0,
        "建空 inode 那一版的映射树是单个叶根"
    );
    let dropped_mapping_entry =
        drop_one_entry_of_an_index_leaf_root(&mut mapping_root, mapping_header.entries.len() - 1);
    assert!(
        parse_mapping_entry(&dropped_mapping_entry).is_some(),
        "删掉的是一条解得开的映射条目"
    );
    reseal_index_node(&mut mapping_root);
    let mapping_root_checksum =
        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 最新那条根（txg 4）的分配记录树里罩这一版树表单元那个槽的两条记录（两盘同槽各一条，代 4、还着）代改成 5，晚于根；两盘一起改，
/// 「每个落点每盘一条」那一判（比的是各盘的（槽, 跨度, 代, 已释放）集合）照过。指着改过的节点的每一道校验和重算。
fn move_the_allocation_records_of_the_tree_table_slot_after_the_root(devices: &mut Devices) {
    let (_, root) = newest_root_in_the_ring(devices);
    let tree_table_slot = root.tree_table.locations[0].slot;
    let changed_records = rewrite_allocation_records_of_the_newest_root(
        devices,
        |record| record.slot == tree_table_slot,
        |record| {
            assert_eq!(
                (record.generation, record.is_released),
                (TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW, false),
                "罩这一版树表单元的记录是这一版分配的、还着"
            );
            record.generation = ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT;
        },
    );
    assert_eq!(changed_records, 2, "罩树表单元那个槽的分配记录两盘各一条");
}

/// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]，txg 4）上两处一起坏，
/// 每一处都把指着它的校验和重算：
/// ① 记账树根里 key 最小的那一行（不是 inode 号水位的行）删掉，之后的条目前移、条目数与声明长度减一；统计量标签与代一处都不碰：
///    逐条判照过，数记账条目 16 − 1 − 1 条台阶行 = 14 ≠ 3 + 6 × 2，与读者认不认哪个标签无关。只坏这一处时只读恢复报记账条目数，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off(
) -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            let dropped_row =
                AccountingEntry::parse(&drop_one_entry_of_an_index_leaf_root(node, 0))
                    .expect("记账叶根的条目宽是 34，删掉的那条解得开");
            assert_ne!(
                dropped_row.statistic, STATISTIC_INODE_WATERMARK,
                "删的不是 inode 号水位的行：条目数少的这一条与认不认台阶行无关"
            );
        },
    );
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_count_is_not_three_plus_six_per_device(),
        },
        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
/// ① 中央映射树根（单个叶根）里 key 最大的那一条删掉（[`drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root`]）：
///    各树节点数、inode 叶容器数、数据单元数一样都没动，映射条目数比它们算出的少一条；记账树一处不碰，数记账条目照过。
///    只坏这一处时只读恢复报映射条目数，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off(
) -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(
        &mut devices,
    );
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: mapping_entry_count_is_not_one_per_mapped_unit(),
        },
        "前提：只坏映射那一处时，只读恢复在 txg 4 那条根上报映射条目数"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
/// ① 记账树根里这一版自己那一行 inode 号水位（代段 4，等于根的 txg；台阶行是代段 3 的那一行，不碰）的代改成 5，晚于根，条目按 key 重排：
///    条目宽照旧 34；台阶行照旧认得出，数记账条目照旧 3 + 6 × 2，只有逐条判代与 seq 那一判报它。只坏这一处时只读恢复报记账条目的代，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root(
) -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
                node,
                |entry| {
                    entry.statistic == STATISTIC_INODE_WATERMARK
                        && entry.generation == TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
                },
                |watermark_row_of_this_version| {
                    watermark_row_of_this_version.generation =
                        WATERMARK_ROW_GENERATION_AFTER_THE_ROOT;
                },
            );
        },
    );
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_generation_after_the_root_or_sequence_zero(),
        },
        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目的代晚于根"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
/// ① 记账树根的条目宽 34 缩到 key 宽 22、16 条每条只留 key（[`pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys`]）：
///    条目宽那一判报它，条目解不开、逐条判与数条目都轮不到。只坏这一处时只读恢复报记账条目窄于字段表，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys(
) -> Devices {
    let mut devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_narrower_than_its_field_table(),
        },
        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目窄于字段表"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里 key 最小的那一行删掉（标签与代都不碰，记账条目数因此对不上），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在数记账条目之前
/// （先判得了自己那一条的，再数；判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4）。
/// 那一判挪到数记账条目之后，这份镜像报成记账条目数。排在数映射条目、记账条目逐条判、记账条目宽之前那三个次序由下面三张镜像各钉一个。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版在读记账树之前就逐条判分配记录的代与跨度
/// （与冷走读同一个次序），那一判挪到数记账条目之后，这份镜像报成记账条目数。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报记账条目数：{refusal:?}"
    );
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；中央映射树根里 key 最大的那一条删掉（映射条目数因此对不上，记账树不碰），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度也排在数映射条目之前
/// （判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4「排在两个数之前」的另一个数；
/// 攻方原型 `research/prompts/m3-batch57-code-r1-opus-model/probes/corrupt_on_disk_k3_append.rs` 的 ③ 那一形）。
/// 数映射条目挪到那一判之前，这份镜像报成映射条目数，而可写挂载照报分配记录：两条路对同一张盘报不同的成员。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报映射条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
/// 数映射条目在读完记账树之后，与冷走读报同一个成员。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报映射条目数：{refusal:?}"
    );
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里这一版自己那一行 inode 号水位的代改成 5（晚于根；台阶行不碰，记账条目数照旧对得上），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
/// 记账条目逐条判代与 seq 之前（冷走读那一段的次序注释与重建相同：分配记录、条目宽、逐条判、数）。
/// 那一判挪到记账条目逐条判之后（仍在两个数之前），这份镜像报成「记账条目的代晚于根或 seq 为 0」。这张盘每条记账条目宽都是 34，
/// 条目宽那一判不坏，「分配记录排在条目宽之前」这半句它钉不到，由最后两条（记账树根缩到 key 宽的那张盘）钉。可写挂载那一条在下面。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目的代"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
/// 排在记账条目逐条判代与 seq 之前，与冷走读报同一个成员。重建把那一判挪到记账条目逐条判之后（仍在数记账条目之前），
/// 这份镜像报成「记账条目的代晚于根或 seq 为 0」，而上面两张盘的可写挂载照报分配记录：那两张盘坏的是两个数，挪到两个数之前它们分不出
/// （攻方原型 `research/prompts/m3-batch58-code-r1-opus-output.md`「K3-3」一节的 `r-rebuild-alloc-after-each-entry`）。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报记账条目的代：{refusal:?}"
    );
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根的条目宽缩到 key 宽、每条只留 key（条目数 16 不变），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
/// 记账条目宽那一判之前（冷走读那一段的次序：分配记录、条目宽、逐条判、数）。那一判挪到条目宽之后（逐条判与两个数之前），
/// 这份镜像报成记账条目窄于字段表（22 对 34）；上面那张逐条判坏的盘条目宽都对，分不出这一形。可写挂载那一条在下面。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目窄于字段表"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
/// 排在记账条目宽那一判之前，与冷走读报同一个成员。重建把那一判挪到条目宽之后（逐条判与数记账条目之前），这份镜像报成记账条目窄于字段表。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报记账条目窄于字段表：{refusal:?}"
    );
}
