//! 冷走读与从盘上重建上一版共用的判定：树表 key 宽与树 ID 水位、记账条目宽与条数、映射条目与 key 集合、
//! deadlist 与分配记录、inode 内部条目与叶容器、数据单元对指针与 inode 记录、指针按盘升序。

use super::*;

// ─── 冷走读与从盘上重建上一版共用的判定 ───
// 代码审阅第 24 / 33 条，用户定案（`records/2026-09-27-代码审阅38条去向.md`「可写挂载判全树头与 key 次序」那一行）：「重建时把已经读到的节点照冷走读那套判全，
// 坏了报错拒挂载，不多读盘」。下面每一道只判两边都已经读到手的字节，[`walk_to_file`] 与 [`rebuild_version`] 各在读到那一样的地方调，
// 报的成员两边相同；一道判定一处定义，不各写一份（`code-discipline.md`「重复要生成，不许手抄」）。

/// 树表单元自述的 key 宽是树表的（[`TREE_TABLE_KEY_WIDTH`]）。
pub(super) fn tree_table_key_width_is_the_tree_tables(
    tree_table: &IndexNodeHeader,
) -> Result<(), RecoveryFailure> {
    if tree_table.key_width != TREE_TABLE_KEY_WIDTH {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "树表单元自述 key 宽不是 8",
        });
    }
    Ok(())
}

/// 树表条目里的树 ID 低于根记录的树 ID 水位（I-7.8（根记录树 ID 水位不低于全池最大树 ID） 在读路径上的那一半）。
pub(super) fn tree_identifier_of_the_entry_is_below_the_watermark(
    entry: &TreeTableEntry,
    root: &RootRecord,
) -> Result<(), RecoveryFailure> {
    if entry.tree.0 >= root.tree_identifier_watermark {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-7.8",
            detail: "树 ID 不低于水位",
        });
    }
    Ok(())
}

/// 每条记账条目都与字段表一样宽（条目宽是盘上的值，不是 34（窄于、宽于都算）报 `EntryWidthDiffersFromItsFieldTable`）。排在
/// [`accounting_entry_count_is_three_plus_six_per_device`] 之前：那一判要把条目解开才认得出 inode 号水位的台阶行（统计量标签与代段都在条目里），
/// 条目宽坏了时台阶行解不开、被算进 3 + 6 × 盘数，条目宽这一坏法就报成了条目数不对，不是它自己的成员。
pub(super) fn every_accounting_entry_is_as_wide_as_its_field_table(
    accounting_entry_bytes: &[Vec<u8>],
) -> Result<(), RecoveryFailure> {
    for entry_bytes in accounting_entry_bytes {
        accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
    }
    Ok(())
}

/// 记账条目数是 3 + 6 × 盘数（D5（快照 / 空间记账机制） 已定项 8：池级三行加每盘六行），inode 号水位的台阶行不算在里面：
/// 它们是那一统计量代段早于这一版 txg（`root_txg`）的行，随根一直留着、条数随建过 inode 的发布数涨（D5 已定项 2 射程「inode 号水位的台阶行」）。
/// 解不开的条目算进 3 + 6 × 盘数；调用方先过 [`every_accounting_entry_is_as_wide_as_its_field_table`] 与逐条的
/// [`accounting_entry_parsed_and_judged_against_the_root`]，走到这里的每条都解得开、代不晚于根：代写到根之后的台阶行在那一判先红，不落到这里被算进条数。
pub(super) fn accounting_entry_count_is_three_plus_six_per_device(
    accounting_entry_bytes: &[Vec<u8>],
    root_txg: CheckpointTxg,
    device_count: usize,
) -> Result<(), RecoveryFailure> {
    let entries_other_than_the_inode_number_watermark_steps = accounting_entry_bytes
        .iter()
        .filter(|bytes| {
            !AccountingEntry::parse(bytes).is_some_and(|entry| {
                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg
            })
        })
        .count();
    if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",
        });
    }
    Ok(())
}

/// [`central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots`] 报的那一拒的不变量号与说明。
pub const CENTRAL_MAPPING_KEY_SET_INVARIANT: &str = "I-1.11";
pub const CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS: &str =
    "中央映射里的 key 集合不等于这一版进映射的单元的 key 加留存条目的 key（一条映射 key 被换了）";

/// 映射条目数 = 进映射的单元数：inode 根一条，extent 树、分配记录树、记账树每个节点一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
/// （映射树自己的节点、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
pub(super) fn mapping_entry_count_is_one_per_mapped_unit(
    mapping_entries: usize,
    counts: MappedUnitCounts,
) -> Result<(), RecoveryFailure> {
    let expected = 1
        + counts.extent_tree_nodes
        + counts.allocation_record_tree_nodes
        + counts.accounting_tree_nodes
        + counts.deadlist_tree_nodes
        + counts.mapping_entries_kept_for_the_snapshots
        + counts.inode_leaf_containers
        + counts.data_units
        + counts.deletion_tree_nodes;
    if mapping_entries != expected {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树、deadlist 树的节点数 + deadlist 条目数 + inode 叶容器数 + 数据单元数",
        });
    }
    Ok(())
}

/// 建上一版那一判（I-1.11（映射 key 与单元头相符） 产品一侧；D4（校验和位置） 已定项 4 那一格「映射 key 被换的盘挂载时拒」）：中央映射里的 key 集合
/// 等于这一版进映射的单元的 key 加留存条目的 key。[`mapping_entry_count_is_one_per_mapped_unit`] 只数条数，一条 key 换了、封得对的盘数得过；
/// 不拒的话这一版照「查不到就留提示」收下，提示在单元搬走之后还指隔离的旧落点，下一次发布把这把 key 盖回那里，搬到的新槽没人引用。只比内存里读过的，不多读盘。
/// 只读路径（冷走读、`read_at`）不判：换 key 不动位置条目与单元字节，查不到报 [`RecoveryFailure::MappingMiss`]，读不出别的单元的字节。
///
/// # Errors
/// 两个集合不等 ⇒ [`RecoveryFailure::InvariantViolated`]。
pub(super) fn central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots(
    mapping_entries: &MappingEntriesOfTheVersion,
    mapped_units: &[(TransactionUnit, Vec<u8>)],
) -> Result<(), RecoveryFailure> {
    let keys_expected: BTreeSet<&[u8]> = mapped_units
        .iter()
        .map(|(_, key)| key.as_slice())
        .chain(
            mapping_entries
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .map(|(key, _)| key.as_slice()),
        )
        .collect();
    let keys_in_the_mapping: BTreeSet<&[u8]> = mapping_entries
        .mapping_position_of_key
        .keys()
        .map(Vec::as_slice)
        .collect();
    if keys_expected != keys_in_the_mapping {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: CENTRAL_MAPPING_KEY_SET_INVARIANT,
            detail: CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS,
        });
    }
    Ok(())
}

/// deadlist 树叶里的条目逐条解开（43 字节，`records::DeadlistEntry`），次序照叶里的（key 升序）。
///
/// # Errors
/// 有一条不是 43（窄于、宽于都算，欠账 C867）⇒ `EntryWidthDiffersFromItsFieldTable`。
pub(super) fn deadlist_entries_parsed(
    leaf_entries_in_key_order: &[Vec<u8>],
) -> Result<Vec<DeadlistEntry>, RecoveryFailure> {
    leaf_entries_in_key_order
        .iter()
        .map(|bytes| {
            DeadlistEntry::parse(bytes).ok_or(RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                what: "deadlist 条目",
                entry_bytes: bytes.len(),
                field_table_bytes: DEADLIST_ENTRY_KEY_BYTES,
            })
        })
        .collect()
}

/// 分配记录的跨度不是 0、代不晚于根。已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
pub(super) fn allocation_record_generations_and_spans_are_judged(
    records: &[AllocationRecord],
    root: &RootRecord,
) -> Result<(), RecoveryFailure> {
    for record in records {
        if record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
            });
        }
    }
    Ok(())
}

/// 解一条记账条目（条目宽是盘上的值，不是 34（窄于、宽于都算）报 `EntryWidthDiffersFromItsFieldTable`）。
fn accounting_entry_parsed_at_its_field_table_width(
    entry_bytes: &[u8],
) -> Result<AccountingEntry, RecoveryFailure> {
    AccountingEntry::parse(entry_bytes).ok_or(RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
        what: "记账条目",
        entry_bytes: entry_bytes.len(),
        field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    })
}

/// 解一条记账条目（条目宽是盘上的值，不是 34（窄于、宽于都算）报 `EntryWidthDiffersFromItsFieldTable`），判它的代不晚于根、seq 不是 0。
pub(super) fn accounting_entry_parsed_and_judged_against_the_root(
    entry_bytes: &[u8],
    root: &RootRecord,
) -> Result<AccountingEntry, RecoveryFailure> {
    let entry = accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
    if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目的代晚于根或 seq 为 0",
        });
    }
    Ok(entry)
}

/// inode 树根恒是层级 1（I-9.1）。
pub(super) fn inode_tree_root_level_is_one(
    inode_root: &IndexNodeHeader,
) -> Result<(), RecoveryFailure> {
    if inode_root.level != 1 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.1",
            detail: "inode 树根层级不是 1",
        });
    }
    Ok(())
}

/// inode 树内部条目里的子指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句
/// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏）。
pub const INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
    "inode 树内部条目的子指针头部 extent 偏移不是 0";
/// inode 树内部条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14：判该指针所在的结构损坏）。
pub const INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
    &str = "inode 树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";

/// 解一条 inode 树内部条目（条目宽是盘上的值，不是 120（窄于、宽于都算）报 `EntryWidthDiffersFromItsFieldTable`；子指针头部 extent 偏移非 0 报
/// `UnitMalformed`，`what` 取 [`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]，加密、压缩那几段不是第一版的取值取
/// [`INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]），判类型段是 2（I-9.2（条目身份与子头相符）
/// 的第一句；第一版 inode 树根下面只有叶容器）。
pub(super) fn inode_internal_entry_parsed_and_judged(
    entry: &[u8],
) -> Result<(u64, PackedIdentity, NodePointer), RecoveryFailure> {
    let (separator_key, identity, child) =
        parse_inode_internal_entry_or_refusal(entry).map_err(|refusal| match refusal {
            PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable => {
                RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                    what: "inode 树内部条目",
                    entry_bytes: entry.len(),
                    field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
                }
            }
            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(field) => {
                RecoveryFailure::UnitMalformed {
                    what: what_of_a_pointer_head_field_outside_the_first_version(
                        field,
                        INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
                        INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
                    ),
                }
            }
        })?;
    if identity.record_type != PACKED_TYPE_INODE {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.2",
            detail: "inode 内部条目类型段不是 2",
        });
    }
    Ok((separator_key, identity, child))
}

/// 一片 inode 叶容器按指着它的那条内部条目核（I-9.2（条目身份与子头相符）：叶头的四元组身份等于条目的身份引用、记录宽 140、fsid 是本池；
/// I-1.2：出生树与出生序号等于子指针的、诞生不晚于根、写序实例等于子指针的），再逐条解它的记录、判记录号不小于分隔 key 与容器号（I-9.4）。
/// 交回解开的记录，按容器里的次序。
pub(super) fn inode_leaf_container_judged_against_its_entry(
    leaf: &PackedUnitHeader,
    entry: InodeInternalEntry,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
) -> Result<Vec<InodeRecord>, RecoveryFailure> {
    let InodeInternalEntry {
        separator_key,
        identity,
        child,
    } = entry;
    if leaf.identity != identity
        || u64::try_from(leaf.record_width).expect("记录宽") != INODE_RECORD_BYTES
        || leaf.filesystem_identifier != expected_filesystem_identifier
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.2",
            detail: "inode 叶头与条目身份引用不符",
        });
    }
    if child.head.birth_tree != identity.birth_tree || leaf.birth_sequence != child.birth_sequence {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "inode 叶的出生身份与子指针不符",
        });
    }
    if leaf.birth_txg > root.checkpoint_txg || leaf.write_order.instance != child.instance {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "inode 叶诞生于根之后或写序实例与子指针不符",
        });
    }
    let mut records = Vec::with_capacity(leaf.records.len());
    for record_bytes in &leaf.records {
        let record = InodeRecord::parse(record_bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "inode 记录",
        })?;
        inode_record_is_not_below_the_separator_key_and_the_container(
            InodeNumber(record.inode),
            SeparatorKeyOfAnInodeLeaf(separator_key),
            ContainerNumberOfAnInodeLeaf(identity.container),
        )?;
        records.push(record);
    }
    Ok(records)
}

/// 第一个文件的数据指针个数按上段叶条目的标签对 inode size（D4（校验和位置） 已定项 5 的除法；与写侧切分、挂载态读者同一条，
/// `write_request_split::data_units_read_within_the_file_size`）：标签 1、2 不少于 size 除出来的单元数（多出来的是截断中间态里
/// size 之外还挂着的单元，判决 m4-trunc-r1 K2），标签 3、4 少于它（之后到 size 是洞，K5）。交回 size 之内要读、要核的单元数。
pub(super) fn data_unit_count_matches_the_inode_size(
    data_units: usize,
    tail: TailOfAFile,
    inode_record: &InodeRecord,
) -> Result<usize, RecoveryFailure> {
    let data_units_of_the_inode = match tail {
        TailOfAFile::NoHoleAfterTheLastDataUnit => DataUnitsOfTheInode::SomeWritten,
        TailOfAFile::HoleAfterTheLastDataUnit => DataUnitsOfTheInode::HoleAfterTheLastDataUnit,
    };
    data_units_read_within_the_file_size(
        inode_record.size,
        u64::try_from(data_units).expect("单元数"),
        data_units_of_the_inode,
    )
    .map(|within| usize::try_from(within).expect("size 之内的单元数不多于单元数"))
    .map_err(|_refused| RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符",
    })
}

/// 冷走读拼出来的内容补到 size：尾部的洞读出 0、不读盘（判决 m4-trunc-r1 K5）。
pub(super) fn content_padded_with_the_hole_up_to_the_size(
    mut content: Vec<u8>,
    inode_record: &InodeRecord,
) -> Vec<u8> {
    content.resize(
        usize::try_from(inode_record.size).expect("文件长度装得进 usize"),
        0,
    );
    content
}

/// 一个读到手的数据单元按查找路径核（I-1.1：五元组与查找路径相符；I-9.10：对象出生代等于 inode 记录的；I-1.2：诞生代号、写序、fsid
/// 与指针相符；声明长度等于 inode size 按净荷容量切出来的这一段——最后一个单元之外恒装满一个净荷；I-2.3：补齐字节为 0），交回它的净荷。
pub(super) fn data_unit_judged_against_its_pointer_and_the_inode_record<'unit>(
    bytes: &'unit [u8],
    unit: DataUnitOfAFile<'_>,
    inode_record: &InodeRecord,
    expected_filesystem_identifier: u64,
) -> Result<&'unit [u8], RecoveryFailure> {
    let payload_capacity_in_bytes = u64::try_from(data_unit_payload_capacity()).expect("32634");
    let unit_index_in_file = DataUnitIndexInFile(u64::try_from(unit.position).expect("单元序号"));
    let first_file_byte = unit_index_in_file.first_file_byte(payload_capacity_in_bytes);
    let header = parse_data_unit(bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
        what: "数据单元",
    })?;
    if header.identity.tree != unit.extent_tree
        || header.identity.object != inode_record.inode
        || header.identity.anchor_offset != first_file_byte.0
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: "数据单元五元组与查找路径不符",
        });
    }
    if header.identity.object_birth != inode_record.object_birth {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.10",
            detail: "对象出生代与 inode 记录不符",
        });
    }
    if header.birth_txg != unit.pointer.head.birth_txg
        || header.write_order != unit.pointer.write_order
        || header.filesystem_identifier != expected_filesystem_identifier
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "数据单元头与指针的出生身份不符",
        });
    }
    let expected_payload_length = inode_record
        .size
        .saturating_sub(first_file_byte.0)
        .min(payload_capacity_in_bytes);
    if u64::from(header.declared_length) != expected_payload_length {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "声明长度与 inode size 按净荷容量切出来的这一段不符",
        });
    }
    data_unit_payload(bytes, header.declared_length).map_err(|_error| {
        RecoveryFailure::InvariantViolated {
            invariant: "I-2.3",
            detail: "补齐字节非零",
        }
    })
}

/// 盘上一条指针的位置条目不按设备身份严格升序时报的成员（I-2.5（位置条目按设备身份升序））。
pub(super) fn location_entries_not_ascending() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-2.5",
        detail: "位置条目不按设备身份严格升序",
    }
}

/// 根记录里的四条指针按 I-2.5 判（全零的豁免）：可写挂载要照抄写回的那几条（零单元发布、暖机照抄实例表与树表指针）从这里进来。
pub(super) fn root_record_pointers_ascend_by_device(
    root: &RootRecord,
) -> Result<(), RecoveryFailure> {
    for pointer in [
        &root.tree_table,
        &root.instance_table,
        &root.mapping_root,
        &root.allocation_record_tree_root,
    ] {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(())
}

/// 顶着这一版的那条 journal 记录里盘上读来的位置条目按 I-2.5 判，坏的报 [`location_entries_not_ascending`]：
/// 新根段里的四条指针（树表、映射树根、实例表、分配记录树根；全零的豁免），判法与根记录那四条（[`root_record_pointers_ascend_by_device`]）、
/// 重建出来的这一版（[`pointers_of_the_rebuilt_version_ascend_by_device`]）同一个（逐条 `NodePointer::location_entries_ascend_by_device`）；
/// 每个点名项的两条位置条目（I-2.5 射程里的「journal 点名项里的位置条目数组」，点名项不是指针、没有全零豁免，
/// 与数据指针同一个判法 `pointer::location_entries_ascend_by_device`）。
/// 重建一拿到这条记录就重新编码它（`JournalRecord::to_bytes`），写者 `NodePointer::write_to` 在那里断言新根段四条升序：
/// 那一步之前不判，坏镜像就在可写挂载的那条断言上 panic（I-2.5 罩「任一块指针」，新根段这四条也是盘上读来的块指针）。
/// 点名项的写者 `NamedUnit::write_to` 不断言升序，重新编码不 panic；恢复这一侧别处没有按 I-2.5 判点名项的位置条目
/// （代码轮 `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L4 那一句），并在这一判里。
pub(super) fn location_entries_of_the_record_ascend_by_device(
    record: &JournalRecord,
) -> Result<(), RecoveryFailure> {
    for pointer in [
        &record.new_tree_table,
        &record.new_mapping_root,
        &record.new_instance_table,
        &record.new_allocation_record_tree_root,
    ] {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    for named_unit in &record.named {
        location_entries_ascend_by_device(&named_unit.locations)
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(())
}

/// 一棵容器树（墓碑树、谱系树、退役树）这一版里盘上读来的指针：每片容器的指针与根之下每个码 2 节点的指针（根的指针住树表条目，
/// 随树表条目判）。下一次重写这棵树的某个节点时，没换的条目连同它们的指针照抄写回。
fn container_tree_pointers_read_from_disk<'version>(
    container_pointers: impl Iterator<Item = &'version NodePointer> + 'version,
    index_nodes: &'version ContainerTreeIndexNodesVersion,
) -> impl Iterator<Item = &'version NodePointer> + 'version {
    container_pointers.chain(
        index_nodes
            .pointers_below_the_root
            .iter()
            .map(|(_, pointer)| pointer),
    )
}

/// 重建出来的这一版里每一条盘上读来的指针按 I-2.5 判（C476 普查「缺口」那张表 `pointer.rs` 那一行，实测坐实走得到）：
/// 下一次发布照抄没重写的角色时把它们原样写回（树表条目、多层码 2 树与分配记录树的内部条目、inode 树根的条目、extent 树、
/// 三棵容器树的内部条目），写者 `NodePointer::write_to` / `DataPointer::write_to` 在那里断言升序——不在读进来的地方判，
/// 坏镜像就在写者的断言上 panic（三棵容器树那一份：代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K4）。
pub(super) fn pointers_of_the_rebuilt_version_ascend_by_device(
    output: &TransactionOutput,
) -> Result<(), RecoveryFailure> {
    root_record_pointers_ascend_by_device(&output.root)?;
    let tombstone_tree_pointers = container_tree_pointers_read_from_disk(
        output
            .tombstone_containers
            .iter()
            .map(|container| &container.pointer),
        &output.tombstone_tree_index_nodes,
    );
    let lineage_tree_pointers = container_tree_pointers_read_from_disk(
        output
            .lineage_containers
            .iter()
            .map(|container| &container.pointer),
        &output.lineage_tree_index_nodes,
    );
    let retirement_tree_pointers = container_tree_pointers_read_from_disk(
        output
            .retirement_containers
            .iter()
            .map(|container| &container.pointer),
        &output.retirement_tree_index_nodes,
    );
    let node_pointers = output
        .tree_table_entries
        .iter()
        .map(|entry| &entry.root)
        .chain(
            output
                .extent_tree
                .upper_nodes
                .iter()
                .map(|(_, pointer)| pointer),
        )
        .chain(
            output
                .extent_tree
                .lower_nodes
                .iter()
                .map(|(_, pointer)| pointer),
        )
        .chain(
            output
                .inode_leaf_containers
                .iter()
                .map(|container| &container.pointer),
        )
        .chain(
            output
                .allocation_record_tree
                .nodes
                .iter()
                .map(|(_, pointer)| pointer),
        )
        .chain(output.accounting_tree.pointers.iter())
        .chain(output.deadlist_tree.pointers.iter())
        .chain(output.central_mapping_tree.pointers.iter())
        .chain(tombstone_tree_pointers)
        .chain(lineage_tree_pointers)
        .chain(retirement_tree_pointers);
    for pointer in node_pointers {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    for data_pointer in &output.data_pointers {
        location_entries_ascend_by_device(&data_pointer.locations)
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    // 第一个文件之外各文件的数据指针与下段节点指针同一判。
    for file in &output.files_other_than_the_first {
        for data_pointer in &file.data_pointers {
            location_entries_ascend_by_device(&data_pointer.locations)
                .map_err(|_not_ascending| location_entries_not_ascending())?;
        }
        for (_, pointer) in &file.extent_lower_nodes {
            pointer
                .location_entries_ascend_by_device()
                .map_err(|_not_ascending| location_entries_not_ascending())?;
        }
    }
    Ok(())
}
