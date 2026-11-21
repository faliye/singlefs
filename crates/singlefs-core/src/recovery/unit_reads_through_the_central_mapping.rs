//! 按位置条目读一个单元，按父指针里的位置提示读、读不出再经中央映射回退：进映射的树节点与码 1 数据单元，
//! 连同按需读字节的两种中央映射树。

use super::*;

pub(super) fn unit_bytes_for_class(unit_class: u8) -> Option<usize> {
    match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => {
            Some(usize::try_from(DATA_UNIT_BYTES).expect("32768"))
        }
        UNIT_CLASS_INDEX_NODE => Some(usize::try_from(NODE_BYTES).expect("16384")),
        _ => None,
    }
}

/// 一条记录点名的一个单元占几个槽（单元种类定的字节数 ÷ 16384）：可写挂载隔离水位之上没施加的记录点名的落点按它
/// （`mount::placements_named_by_unapplied_journal_records`）。种类不认识的那条点名项（坏盘上的记录：施加前逐项验点名单元时它本来就验不过）
/// 按最宽的那一档（数据单元的两槽）算，隔离只会多不会少。
pub(crate) fn span_in_slots_of_a_named_unit(named: &crate::journal::NamedUnit) -> u64 {
    let unit_bytes = unit_bytes_for_class(named.unit_class)
        .unwrap_or(usize::try_from(DATA_UNIT_BYTES).expect("32768"));
    u64::try_from(unit_bytes).expect("单元字节数装得进 u64") / SLOT_BYTES
}

/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC-32C 等于条目里的校验和才算读到。
/// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
/// 对得上的那一份报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]），读成而对不上的那一份也报
/// （[`PoolReader::note_that_the_bytes_read_fail_self_verification`]：可写挂载这次挂载共用的那一份记进读坏账、丢掉那几块）。
/// 前一条没读成、后一条对上（只坏一份）另报一声（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]，D4（校验和位置） 已定项 4 的计数）。
pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    locations: &[LocationEntry; 2],
    unit_bytes: usize,
) -> Result<Vec<u8>, RecoveryFailure> {
    let mut an_earlier_copy_failed = false;
    for location in locations {
        let Some(bytes) = reader.read(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        ) else {
            an_earlier_copy_failed = true;
            continue;
        };
        if crc32_castagnoli(&bytes) == location.unit_checksum {
            reader.note_that_the_bytes_read_self_verify(
                location.device,
                location.slot.to_device_offset(),
                &bytes,
            );
            if an_earlier_copy_failed {
                reader.note_that_a_unit_was_read_with_one_copy_failing(locations);
            }
            return Ok(bytes);
        }
        an_earlier_copy_failed = true;
        reader.note_that_the_bytes_read_fail_self_verification(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        );
    }
    Err(RecoveryFailure::UnitUnreadable {
        slot: locations[0].slot,
    })
}

/// 进中央映射的树节点分两类（D19（块指针的结构与宽度预算） 已定项 8：映射装码 1、码 2、码 3）：码 2 索引节点
/// （extent 树根、inode 树根、分配记录树与记账树的根）与码 3 打包记录单元（inode 叶容器）。类定两件事：映射 key 里的
/// 类标签与读几个字节。豁免三类（映射树根、树表、实例表）不进映射，不走这里——它们的父指针里的位置条目是权威。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MappedTreeNodeClass {
    IndexNode,
    PackedRecordUnit,
}

impl MappedTreeNodeClass {
    fn unit_class_tag(self) -> u8 {
        match self {
            MappedTreeNodeClass::IndexNode => UNIT_CLASS_INDEX_NODE,
            MappedTreeNodeClass::PackedRecordUnit => UNIT_CLASS_PACKED,
        }
    }

    pub(super) fn unit_bytes(self) -> usize {
        match self {
            MappedTreeNodeClass::IndexNode => usize::try_from(NODE_BYTES).expect("16384"),
            MappedTreeNodeClass::PackedRecordUnit => {
                usize::try_from(DATA_UNIT_BYTES).expect("32768")
            }
        }
    }
}

/// 查中央映射的口子：给一个 27 字节的映射 key，交回映射里它的两条位置条目（没有这个 key 是 `Ok(None)`）。
/// 挂载态查打开时整片读进来的条目，冷走读查映射树根——查法不同，回退那一步是同一条。
pub(crate) type CentralMappingLocationsOfKey<'lookup> =
    dyn Fn(&[u8]) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> + 'lookup;

/// 按父指针里的位置提示读一个进映射的树节点；两条提示都读不出（读不回字节，或整单元校验和对不上）时，
/// 按这个节点的映射 key 查中央映射、照映射里的两条位置条目再读一次（D19（块指针的结构与宽度预算） 已定项 8：
/// 豁免三类之外的单元位置提示读不出时一律经映射回退）。挂载态的读（`crate::mounted_read`）与冷启动走读走这同一条。
/// 每回退一次 `stale_location_hint_hops` 加一（已定项 5 硬规则 3：提示过期的多跳要有运行时观测点）。
///
/// # Errors
/// 提示读不出、映射里没有这个 key ⇒ `MappingMiss`（带提示的槽）；映射落点也读不出 ⇒ `MappingStillUnreadable`
/// （带映射落点的槽）——与冷走读里数据单元那一条回退同两个成员。查映射本身判红的（映射树根读不出、条目窄于字段表），原样交回。
pub(crate) fn read_mapped_tree_node_via_hint_then_central_mapping(
    reader: &dyn PoolReader,
    pointer: &NodePointer,
    class: MappedTreeNodeClass,
    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
    stale_location_hint_hops: &mut usize,
) -> Result<Vec<u8>, RecoveryFailure> {
    let unit_bytes = class.unit_bytes();
    let mapping_key = mapping_key_for_node(class.unit_class_tag(), *pointer);
    // 只坏一份按这个单元此刻的权威落点判：按提示读到只坏一份而提示不是映射里那两条（提示过期）不报
    // （[`PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`]，判决 m4-close-code-r3 第三节第 4 条）。
    let reader: &dyn PoolReader = &PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
        reader,
        &mapping_key,
        central_mapping_locations_of_key,
    );
    if let Ok(bytes) = read_unit_via_locations(reader, &pointer.locations, unit_bytes) {
        return Ok(bytes);
    }
    *stale_location_hint_hops += 1;
    let Some(mapped_locations) = central_mapping_locations_of_key(&mapping_key)? else {
        return Err(RecoveryFailure::MappingMiss {
            slot: pointer.locations[0].slot,
        });
    };
    read_unit_via_locations(reader, &mapped_locations, unit_bytes).map_err(|_still_unreadable| {
        RecoveryFailure::MappingStillUnreadable {
            slot: mapped_locations[0].slot,
        }
    })
}

/// 在映射树根兼叶的条目里找一个 key 的两条位置条目：逐条解（条目宽是盘上的值，窄于 55 的报
/// `EntryWidthDiffersFromItsFieldTable`，不按固定偏移切），同 key 多条取最后一条。冷走读查映射只走这一个函数。
pub(super) fn central_mapping_locations_among_entries(
    mapping_entries: &[Vec<u8>],
    mapping_key: &[u8],
) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
    let mut mapped = None;
    for entry in mapping_entries {
        let (candidate_key, locations) = parse_mapping_entry(entry).ok_or(
            RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                what: "映射条目",
                entry_bytes: entry.len(),
                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
            },
        )?;
        if candidate_key == mapping_key {
            mapped = Some(locations);
        }
    }
    Ok(mapped)
}

/// 按数据指针里的位置提示读一个码 1 数据单元；两条提示都读不出（读不回字节，或整单元校验和对不上）时，按它的码 1 映射 key
/// 查中央映射、照映射里的两条位置条目再读一次（D19（块指针的结构与宽度预算） 已定项 8：豁免三类之外的单元位置提示读不出时一律经映射回退）。
/// 冷启动走读与从盘上重建上一版走这同一条；每回退一次 `data_unit_stale_location_hint_hops` 加一（已定项 5 硬规则 3 的观测点）。
/// 数据单元读不出不是这里的错：交回 [`DataUnitReadThroughTheCentralMapping`] 的两种读不出，由调用方按各自的条款处置。
///
/// # Errors
/// 查映射本身判红的（映射树根读不出、条目窄于字段表），原样交回。
pub(super) fn read_data_unit_via_hint_then_central_mapping(
    reader: &dyn PoolReader,
    data_pointer: &DataPointer,
    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
    data_unit_stale_location_hint_hops: &mut usize,
) -> Result<DataUnitReadThroughTheCentralMapping, RecoveryFailure> {
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mapping_key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
    // 只坏一份按这个单元此刻的权威落点判（同 [`read_mapped_tree_node_via_hint_then_central_mapping`]）。
    let reader: &dyn PoolReader = &PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
        reader,
        &mapping_key,
        central_mapping_locations_of_key,
    );
    if let Ok(bytes) = read_unit_via_locations(reader, &data_pointer.locations, data_unit_bytes) {
        return Ok(DataUnitReadThroughTheCentralMapping::Content(bytes));
    }
    *data_unit_stale_location_hint_hops += 1;
    let Some(mapped_data_unit_locations) = central_mapping_locations_of_key(&mapping_key)? else {
        return Ok(
            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping {
                hint_slot: data_pointer.locations[0].slot,
            },
        );
    };
    Ok(
        match read_unit_via_locations(reader, &mapped_data_unit_locations, data_unit_bytes) {
            Ok(bytes) => DataUnitReadThroughTheCentralMapping::Content(bytes),
            Err(_still_unreadable) => {
                DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation {
                    mapped_slot: mapped_data_unit_locations[0].slot,
                }
            }
        },
    )
}

/// 一个数据单元按位置提示、再经中央映射读下来的结局（[`read_data_unit_via_hint_then_central_mapping`]）。
/// 读不出的两种各自带着冷走读报错要点名的那个槽；冷走读把它们报成错，从盘上重建上一版照抄位置项、不读内容。
pub(super) enum DataUnitReadThroughTheCentralMapping {
    Content(Vec<u8>),
    /// 提示读不出，映射里没有它的 key。
    MissingFromTheMapping {
        hint_slot: SlotNumber,
    },
    /// 提示读不出，映射落点也读不出。
    UnreadableAtTheMappedLocation {
        mapped_slot: SlotNumber,
    },
}

impl DataUnitReadThroughTheCentralMapping {
    /// 冷走读的读法：读不出就是这一步走不下去。
    ///
    /// # Errors
    /// 映射里没有 ⇒ `MappingMiss`（带提示的槽）；映射落点也读不出 ⇒ `MappingStillUnreadable`（带映射落点的槽）。
    pub(super) fn into_content(self) -> Result<Vec<u8>, RecoveryFailure> {
        match self {
            DataUnitReadThroughTheCentralMapping::Content(bytes) => Ok(bytes),
            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { hint_slot } => {
                Err(RecoveryFailure::MappingMiss { slot: hint_slot })
            }
            DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { mapped_slot } => {
                Err(RecoveryFailure::MappingStillUnreadable { slot: mapped_slot })
            }
        }
    }
}

/// 从盘上重建上一版、读一条根的分配记录时用的中央映射树（自举豁免，只按父指针里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8）：
/// 到第一次有提示读不出、要经映射回退时，或重建走到映射树那一步时才读，读过一次就留着——每个节点的盘上字节连同形状，重建要把字节照抄进上一版。
/// 多层时整棵读回来（D8（核心索引结构） 已定项 11），核到哪一步看 `judgement`：从盘上重建上一版照冷走读判全
/// （`EveryHeaderAgainstItsReference`：树 ID、key 宽、出生身份、fsid、层级、区间与 key 次序，代码审阅第 24 / 33 条），
/// 读别的根的分配记录（影子账、抬 F 的上限）只按「拼得成一棵树」核（`OnlyWhatTheShapeNeeds`）。
/// 不提前读：提示都读得出的镜像上，读序与报错的次序照旧。
pub(super) struct CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
    reader: &'reader dyn PoolReader,
    root: &'reader RootRecord,
    judgement: CodeTwoTreeHeaderJudgement,
    tree: OnceCell<CodeTwoTreeReadFromDisk>,
}

impl<'reader> CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
    pub(super) fn new(
        reader: &'reader dyn PoolReader,
        root: &'reader RootRecord,
        judgement: CodeTwoTreeHeaderJudgement,
    ) -> Self {
        Self {
            reader,
            root,
            judgement,
            tree: OnceCell::new(),
        }
    }

    pub(super) fn tree(&self) -> Result<&CodeTwoTreeReadFromDisk, RecoveryFailure> {
        if let Some(read) = self.tree.get() {
            return Ok(read);
        }
        let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
        let tree = read_code_two_tree(
            &self.root.mapping_root,
            &MultiLevelCodeTwoTree::CentralMapping
                .read_expectation(self.root.mapping_root.head.birth_tree),
            self.judgement,
            self.root,
            unit_filesystem_identifier(&self.root.filesystem_identifier),
            &mut |pointer: &NodePointer| {
                read_unit_via_locations(self.reader, &pointer.locations, node_bytes)
            },
        )?;
        Ok(self.tree.get_or_init(|| tree))
    }

    pub(super) fn locations_of_key(
        &self,
        mapping_key: &[u8],
    ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
        central_mapping_locations_among_entries(
            &self.tree()?.leaf_entries_in_key_order,
            mapping_key,
        )
    }

    pub(super) fn into_tree(self) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
        self.tree()?;
        Ok(self
            .tree
            .into_inner()
            .expect("上一行刚把映射树读进来，读不出已经返回了"))
    }
}

impl CentralMappingTreeReadOnFirstUse<'_> {
    pub(super) fn tree(&self) -> Result<&CodeTwoTreeReadFromDisk, RecoveryFailure> {
        if let Some(tree) = self.tree.get() {
            return Ok(tree);
        }
        let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
        let tree = read_code_two_tree(
            &self.root.mapping_root,
            &MultiLevelCodeTwoTree::CentralMapping
                .read_expectation(self.root.mapping_root.head.birth_tree),
            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
            self.root,
            self.expected_filesystem_identifier,
            &mut |pointer: &NodePointer| {
                read_unit_via_locations(self.reader, &pointer.locations, node_bytes)
            },
        )?;
        Ok(self.tree.get_or_init(|| tree))
    }

    pub(super) fn locations_of_key(
        &self,
        mapping_key: &[u8],
    ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
        central_mapping_locations_among_entries(
            &self.tree()?.leaf_entries_in_key_order,
            mapping_key,
        )
    }

    pub(super) fn into_tree(self) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
        self.tree()?;
        Ok(self
            .tree
            .into_inner()
            .expect("上一行刚把映射树读进来，读不出已经返回了"))
    }
}
