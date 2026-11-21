//! 每一种坏法怎么改镜像：盲坏法（翻位、清零扇区、截尾）与指着 panic 面的那几样（窄化条目宽度、改分配记录、改统计行、撑大区数、让两处位置不一致）。

use super::*;

/// 按一个坏法把一份合法镜像坏掉；这个盘面上没有它要坏的对象时交回 None（那一条照实记成「不适用」，不当成跑过）。
#[must_use]
pub fn damage_image(
    base: &MemoryPool,
    kind: DamageKind,
    random: &mut SeededRandomSource,
) -> Option<DamagedImage> {
    let mut image = base.clone();
    let what = match kind {
        DamageKind::FlippedBitsInWrittenSectors => flip_bits(&mut image, random)?,
        DamageKind::ZeroedOneWrittenSector => zero_one_sector(&mut image, random)?,
        DamageKind::DroppedEverySectorPastACut => drop_every_sector_past_a_cut(&mut image, random)?,
        DamageKind::NarrowedEntryWidthOfTheExtentTreeRoot => narrow_the_entry_width_of(
            &mut image,
            TREE_KIND_EXTENT,
            EntriesKeptWhenNarrowing::OnlyTheFirst,
        )?,
        DamageKind::NarrowedEntryWidthOfTheInodeTreeRoot => narrow_the_entry_width_of(
            &mut image,
            TREE_KIND_INODE,
            EntriesKeptWhenNarrowing::OnlyTheFirst,
        )?,
        DamageKind::NarrowedEntryWidthOfTheAllocationRecordsTreeRoot => narrow_the_entry_width_of(
            &mut image,
            TREE_KIND_ALLOCATION_RECORDS,
            EntriesKeptWhenNarrowing::OnlyTheFirst,
        )?,
        DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot => narrow_the_entry_width_of(
            &mut image,
            TREE_KIND_ACCOUNTING,
            EntriesKeptWhenNarrowing::EveryOneAsItsKey,
        )?,
        DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot => {
            narrow_the_entry_width_of_the_central_mapping_tree_root(
                &mut image,
                EntriesKeptWhenNarrowing::EveryOneAsItsKey,
            )?
        }
        DamageKind::NarrowedEntryWidthOfTheTreeTableUnit => {
            narrow_the_entry_width_of_the_tree_table_unit(&mut image)?
        }
        DamageKind::AllocationRecordSlotBelowTheUnitArea => {
            rewrite_an_allocation_record(&mut image, AllocationRecordDamage::SlotBelowTheUnitArea)?
        }
        DamageKind::AllocationRecordSpanPastTheEndOfTheUnitArea => rewrite_an_allocation_record(
            &mut image,
            AllocationRecordDamage::SpanPastTheEndOfTheUnitArea,
        )?,
        DamageKind::TwoAllocationRecordsCoveringTheSameSlot => rewrite_an_allocation_record(
            &mut image,
            AllocationRecordDamage::SameSlotAsTheSecondRecord,
        )?,
        DamageKind::AllocationRecordOnADeviceOutsideThePool => {
            rewrite_an_allocation_record(&mut image, AllocationRecordDamage::DeviceOutsideThePool)?
        }
        DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly => rewrite_an_allocation_record(
            &mut image,
            AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly,
        )?,
        DamageKind::RelabelledInodeWatermarkAccountingRow => relabel_the_inode_watermark_row(
            &mut image,
            LargestKeyOfTheNodeHeader::FollowsTheRelabelledRow,
        )?,
        DamageKind::RegionCountPastTheThreeRegionArray => {
            widen_the_region_count_past_the_array(&mut image)?
        }
        DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot => {
            make_two_location_entries_disagree_on_the_slot(&mut image)?
        }
        DamageKind::RewrittenDataUnitResealedWithItsOwnChecksums => {
            rewrite_a_data_unit_resealing_only_its_own_checksums(&mut image)?
        }
    };
    Some(DamagedImage { kind, what, image })
}

/// [`DamageKind`] 里改坏记账树根、中央映射树根的那三种坏法**原来的样子**：只改了坏法名字里那个字段，节点里跟着它的条目数、
/// 节点头的 key 区间没改对。可写挂载重建照冷走读判全（代码审阅第 24 / 33 条：「可写挂载判全树头与 key 次序，
/// 坏了报错拒挂载，不多读盘」）之后，两个读者都在判全那一道（叶根「根 key 区间与条目不符」）就拒了，走不到坏法要打的那一处；
/// [`DamageKind`] 里那三种因此改成改对的样子，这三种留给用例钉「判全那一道拒成哪个成员」。
/// 不进 [`EVERY_DAMAGE_KIND`]、不进种子那一批。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageRefusedAtTheHeaderJudgement {
    /// 记账树根的条目宽缩到 key 宽、只留第一条（[`DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot`] 原来的缩法）。
    AccountingTreeRootNarrowedToItsFirstEntry,
    /// 中央映射树根的条目宽缩到 key 宽、只留第一条（[`DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot`] 原来的缩法）。
    CentralMappingTreeRootNarrowedToItsFirstEntry,
    /// inode 号水位那一行改挂到登记表外的标签上，节点头的最大 key 照旧（[`DamageKind::RelabelledInodeWatermarkAccountingRow`] 原来的改法）。
    InodeWatermarkRowRelabelledUnderTheOldLargestKey,
}

impl DamageRefusedAtTheHeaderJudgement {
    /// 坏的是 [`DamageKind`] 里哪一种的那个字段（[`DamagedImage::kind`] 记成它）。
    #[must_use]
    pub fn damage_kind_of_the_same_field(self) -> DamageKind {
        match self {
            DamageRefusedAtTheHeaderJudgement::AccountingTreeRootNarrowedToItsFirstEntry => {
                DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot
            }
            DamageRefusedAtTheHeaderJudgement::CentralMappingTreeRootNarrowedToItsFirstEntry => {
                DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot
            }
            DamageRefusedAtTheHeaderJudgement::InodeWatermarkRowRelabelledUnderTheOldLargestKey => {
                DamageKind::RelabelledInodeWatermarkAccountingRow
            }
        }
    }
}

/// 按 [`DamageRefusedAtTheHeaderJudgement`] 里的一种把一份合法镜像坏掉（坏之前那份不动）；盘面上没有它要坏的对象时交回 None。
/// 交回的 [`DamagedImage::kind`] 是坏同一个字段的那一种 [`DamageKind`]，`what` 写明是原来的样子。
#[must_use]
pub fn damage_image_the_way_the_header_judgement_refuses(
    base: &MemoryPool,
    damage: DamageRefusedAtTheHeaderJudgement,
) -> Option<DamagedImage> {
    let mut image = base.clone();
    let what = match damage {
        DamageRefusedAtTheHeaderJudgement::AccountingTreeRootNarrowedToItsFirstEntry => {
            narrow_the_entry_width_of(
                &mut image,
                TREE_KIND_ACCOUNTING,
                EntriesKeptWhenNarrowing::OnlyTheFirst,
            )?
        }
        DamageRefusedAtTheHeaderJudgement::CentralMappingTreeRootNarrowedToItsFirstEntry => {
            narrow_the_entry_width_of_the_central_mapping_tree_root(
                &mut image,
                EntriesKeptWhenNarrowing::OnlyTheFirst,
            )?
        }
        DamageRefusedAtTheHeaderJudgement::InodeWatermarkRowRelabelledUnderTheOldLargestKey => {
            relabel_the_inode_watermark_row(&mut image, LargestKeyOfTheNodeHeader::LeftAsItWas)?
        }
    };
    Some(DamagedImage {
        kind: damage.damage_kind_of_the_same_field(),
        what: format!("（原来的样子）{what}"),
        image,
    })
}

/// 普查 R10 那一处（释放判定逐盘核，`transaction.rs` 的 `placement_registered_unreleased_on_every_device`）从盘上走不到之后的入口：
/// [`DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly`] 那份镜像在可写挂载重建那一步就红在「分配记录不是每个落点每盘各一条」上
/// （代码审阅第 24 / 33 条的判全），走不到发布。这里越过盘上那一道，直接调发布路径的释放判定：
/// 拿 `image` 上最新那条根里的实例表指针，手搭一个分配器，只装这个落点在两条位置条目那两块盘上各一条记录——
/// 与坏法在盘上造的那一对相同：第一条位置条目那块盘上那条还着（分配代取这条根的 txg），第二条那块盘上那条已释放、
/// 释放代是 [`RELEASE_GENERATION_ABOVE_EVERY_ROOT`]（释放判定只看在不在册、释放没有、跨度，不看代）——
/// 再调 [`instance_table_chain_to_release`]（写行那次发布换下整条实例表链时调的就是它）。
/// 镜像上择不出根、实例表指针全零时交回 None。
#[must_use]
pub fn release_check_of_the_instance_table_placement_released_on_the_second_device_only(
    image: &MemoryPool,
) -> Option<Result<Vec<Placement>, PublishError>> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let instance_table_pointer = NodePointer::read_from(&mut ByteReader::at(
        &newest_root.slot_bytes,
        ROOT_INSTANCE_TABLE_POINTER_OFFSET,
    ));
    if instance_table_pointer == NodePointer::empty_root() {
        return None;
    }
    let span_slots = u16::try_from(TransactionUnit::InstanceTable.span_slots())
        .expect("实例表的跨度是 2 槽，装得进分配记录的 15 位跨度");
    let [first_location, second_location] = instance_table_pointer.locations;
    let records = vec![
        AllocationRecord {
            device: first_location.device,
            slot: first_location.slot,
            span_slots,
            generation: CheckpointTxg(newest_root.checkpoint_txg),
            is_released: false,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        },
        AllocationRecord {
            device: second_location.device,
            slot: first_location.slot,
            span_slots,
            generation: CheckpointTxg(RELEASE_GENERATION_ABOVE_EVERY_ROOT),
            is_released: true,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        },
    ];
    let devices = image
        .devices
        .keys()
        .map(|identity| DeviceFreeMap::new(*identity, image.device_size_in_bytes))
        .collect();
    let allocator = PoolAllocator::rebuild_from_records(devices, records);
    Some(instance_table_chain_to_release(
        &[instance_table_pointer],
        &allocator,
    ))
}

fn flip_bits(image: &mut MemoryPool, random: &mut SeededRandomSource) -> Option<String> {
    let sectors = written_sectors_of(image);
    if sectors.is_empty() {
        return None;
    }
    let mut flipped = Vec::new();
    for _ in 0..BITS_FLIPPED_BY_ONE_DAMAGE {
        let (identity, sector) =
            sectors[usize::try_from(random.below(u64::try_from(sectors.len()).expect("扇区数")))
                .expect("下标")];
        let byte_in_sector = usize::try_from(random.below(SECTOR_BYTES)).expect("扇区内偏移");
        let bit = u8::try_from(random.below(BITS_IN_A_BYTE)).expect("位号");
        let sparse = image.devices.get_mut(&identity)?;
        let mut bytes = sector_of(sparse, sector);
        bytes[byte_in_sector] ^= 1u8 << bit;
        sparse.write(DeviceOffsetInBytes(sector * SECTOR_BYTES), &bytes);
        flipped.push(format!(
            "盘 {} 偏移 {} 的第 {bit} 位",
            identity.0,
            sector * SECTOR_BYTES + u64::try_from(byte_in_sector).expect("扇区内偏移")
        ));
    }
    Some(format!("翻位：{}", flipped.join("、")))
}

fn zero_one_sector(image: &mut MemoryPool, random: &mut SeededRandomSource) -> Option<String> {
    // 只挑本来就不全 0 的扇区：写过的扇区里有一批是补齐区，整块写的就是 0，清零它等于什么也没坏。
    let sectors: Vec<(DeviceIdentity, u64)> = written_sectors_of(image)
        .into_iter()
        .filter(|(identity, sector)| {
            image
                .devices
                .get(identity)
                .is_some_and(|sparse| sector_of(sparse, *sector).iter().any(|byte| *byte != 0))
        })
        .collect();
    if sectors.is_empty() {
        return None;
    }
    let (identity, sector) =
        sectors[usize::try_from(random.below(u64::try_from(sectors.len()).expect("扇区数")))
            .expect("下标")];
    let sparse = image.devices.get_mut(&identity)?;
    sparse.write(
        DeviceOffsetInBytes(sector * SECTOR_BYTES),
        &vec![0u8; usize::try_from(SECTOR_BYTES).expect("512")],
    );
    Some(format!(
        "清零：盘 {} 偏移 {} 那一个扇区整个写 0",
        identity.0,
        sector * SECTOR_BYTES
    ))
}

fn drop_every_sector_past_a_cut(
    image: &mut MemoryPool,
    random: &mut SeededRandomSource,
) -> Option<String> {
    let sectors = written_sectors_of(image);
    if sectors.is_empty() {
        return None;
    }
    let mut offsets: Vec<u64> = sectors
        .iter()
        .map(|(_, sector)| sector * SECTOR_BYTES)
        .collect();
    offsets.sort_unstable();
    offsets.dedup();
    // 切点不取最小的那个偏移：从第一个写过的扇区就切，整份镜像什么都不剩，那不是「截掉尾部」。
    let candidates = if offsets.len() > 1 {
        &offsets[1..]
    } else {
        &offsets[..]
    };
    let cut =
        candidates[usize::try_from(random.below(u64::try_from(candidates.len()).expect("偏移数")))
            .expect("下标")];
    let device_size_in_bytes = image.device_size_in_bytes;
    let mut dropped = 0u64;
    for sparse in image.devices.values_mut() {
        let kept: Vec<(u64, Vec<u8>)> = sparse
            .written_sectors_in(DeviceOffsetInBytes(0), device_size_in_bytes)
            .into_iter()
            .filter(|sector| {
                let keep = sector * SECTOR_BYTES < cut;
                if !keep {
                    dropped += 1;
                }
                keep
            })
            .map(|sector| (sector, sector_of(sparse, sector)))
            .collect();
        let mut truncated = SparseDevice::default();
        for (sector, bytes) in kept {
            truncated.write(DeviceOffsetInBytes(sector * SECTOR_BYTES), &bytes);
        }
        *sparse = truncated;
    }
    Some(format!(
        "截尾：两块盘上偏移 ≥ {cut} 的写过的扇区全丢掉（共 {dropped} 个扇区）"
    ))
}

/// 条目宽度那一族：把一棵树根自述的条目宽改成刚好等于 key 宽、条目数改成 1、声明长度跟着改。
/// 读者只判「条目宽 ≥ key 宽」（`unit.rs` 的 `parse_index_node`、checker 的 `index_node_view`），
/// 之后按字段表的固定偏移切条目——切出界就 panic（普查 R1–R4、R13）。
/// 一棵树一条坏法、不按种子挑：挑的话每个种子只打到四棵里的一棵，哪一处 panic 撞不撞得到就成了掷骰子。
/// 缩条目宽那一族把条目宽缩到 key 宽之后，条目区留下什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntriesKeptWhenNarrowing {
    /// 只留第一条（条目数 → 1），节点头的 key 区间照旧。叶根上末条的 key 因此与头里的最大 key 对不上：
    /// 读者要是先判区间（记账树、中央映射树的叶根，代码审阅第 24 / 33 条之后的两个读者都这样），就红在区间上、走不到解条目那一步。
    OnlyTheFirst,
    /// 每条都留、只剩它的 key（条目数不变，按新条目宽从头紧排）：首末两条的 key 不变，节点头的 key 区间、条目数、key 次序都照旧对得上，
    /// 读者走到按字段表解条目那一步才红。
    EveryOneAsItsKey,
}

/// 缩完条目宽之后那几个字段各从什么改成什么（写进「坏在哪」那句话）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NarrowedEntryArea {
    entry_width_before: usize,
    entry_count_before: usize,
    narrowed_entry_width: u16,
    entry_count_after: u16,
    declared_length_after: u16,
}

impl NarrowedEntryArea {
    fn render(self) -> String {
        format!(
            "条目宽 {} → {}（= key 宽）、条目数 {} → {}、声明长度 → {}",
            self.entry_width_before,
            self.narrowed_entry_width,
            self.entry_count_before,
            self.entry_count_after,
            self.declared_length_after
        )
    }
}

/// 一个码 2 节点自述的条目宽缩到刚好等于 key 宽，条目区按 `kept` 留条目，条目数与声明长度跟着改。
/// 条目区本来就是 0 条、或者条目宽本来就等于 key 宽（没得缩）时交回 None。
/// **六处共用这一份**（五棵树的根与树表单元自己）：偏移与缩法只有这一处，各条链各自重算自己那几道校验和。
fn narrow_the_entry_width_of_the_node(
    node: &mut [u8],
    kept: EntriesKeptWhenNarrowing,
) -> Option<NarrowedEntryArea> {
    let key_width = index_node_key_width(node);
    let layout = IndexNodeEntryLayout::of(node);
    if layout.entry_count == 0 || layout.entry_width <= key_width {
        return None;
    }
    let narrowed = u16::try_from(key_width).expect("key 宽装得进 u16");
    let entry_count_after = match kept {
        EntriesKeptWhenNarrowing::OnlyTheFirst => 1,
        EntriesKeptWhenNarrowing::EveryOneAsItsKey => {
            // 迭代上界是条目数。第 i 条的 key 从 i × 原条目宽挪到 i × key 宽：原条目宽 > key 宽，往前挪写到的那一段
            // 止于 (i + 1) × key 宽 < (i + 1) × 原条目宽，盖不到还没挪的那几条。
            for entry_index in 0..layout.entry_count {
                let from = layout.offset_of_entry(entry_index);
                let to = layout.start_in_the_node + entry_index * key_width;
                node.copy_within(from..from + key_width, to);
            }
            // 紧排之后空出来的那一段清零（写者写出来的节点，条目区之后恒 0）。
            node[layout.start_in_the_node + layout.entry_count * key_width
                ..layout.offset_of_entry(layout.entry_count)]
                .fill(0);
            u16::try_from(layout.entry_count).expect("条目数是从 2 字节的字段读来的")
        }
    };
    let declared_length_after = entry_count_after
        .checked_mul(narrowed)
        .expect("缩之后的条目区比缩之前短，缩之前的声明长度装得进 2 字节");
    let entry_count_offset = index_node_entry_count_offset(node);
    let entry_width_offset = index_node_entry_width_offset(node);
    node[entry_count_offset..entry_count_offset + 2]
        .copy_from_slice(&entry_count_after.to_le_bytes());
    node[entry_width_offset..entry_width_offset + 2].copy_from_slice(&narrowed.to_le_bytes());
    node[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
        .copy_from_slice(&declared_length_after.to_le_bytes());
    Some(NarrowedEntryArea {
        entry_width_before: layout.entry_width,
        entry_count_before: layout.entry_count,
        narrowed_entry_width: narrowed,
        entry_count_after,
        declared_length_after,
    })
}

fn narrow_the_entry_width_of(
    image: &mut MemoryPool,
    tree_kind: u16,
    kept: EntriesKeptWhenNarrowing,
) -> Option<String> {
    let mut chain = open_chain_to_the_root_of(image, tree_kind)?;
    let narrowed = narrow_the_entry_width_of_the_node(&mut chain.tree_root_node, kept)?;
    let what = format!(
        "条目宽：树种类 {} 的根（盘 {:?} 槽 {}，层级 {}）{}；链上三道校验和重算",
        chain.tree_kind,
        chain.tree_root_devices,
        chain.tree_root_slot,
        chain.tree_root_node[INDEX_NODE_LEVEL_OFFSET],
        narrowed.render(),
    );
    seal_and_write_back_the_chain(chain, image);
    Some(what)
}

/// 条目宽度那一族的第六处：**树表单元自己**（C504（树表条目宽在走读里无守卫，今天没坏法打得到））。
/// 前五条坏的都是树表**指着的**某棵树的根，走的是 [`reach_the_tree_table`]——它先判「条目宽是 200」，
/// 于是没有一条打得到树表自己；这一条因此走那支不判条目宽的 [`reach_the_tree_table_whatever_its_entry_width_says`]。
/// 条目宽 200 → 8（= 树表的 key 宽）、条目数 → 1、声明长度 → 8，重算树表节点两道与根槽自证一道。
fn narrow_the_entry_width_of_the_tree_table_unit(image: &mut MemoryPool) -> Option<String> {
    let mut reached = reach_the_tree_table_whatever_its_entry_width_says(image)?;
    let narrowed = narrow_the_entry_width_of_the_node(
        &mut reached.tree_table_node,
        EntriesKeptWhenNarrowing::OnlyTheFirst,
    )?;
    let what = format!(
        "条目宽：树表单元自己（盘 {:?} 槽 {}）{}（key 宽 8）；节点两道 + 根槽自证一道校验和重算",
        reached.tree_table_devices,
        reached.tree_table_slot,
        narrowed.render(),
    );
    seal_and_write_back_the_tree_table(reached, image);
    Some(what)
}

/// 条目宽度那一族的第五棵树（普查 R2）：中央映射树的根。它住根记录，所以走的是那条只有两环的链。
fn narrow_the_entry_width_of_the_central_mapping_tree_root(
    image: &mut MemoryPool,
    kept: EntriesKeptWhenNarrowing,
) -> Option<String> {
    let mut chain = open_chain_to_the_central_mapping_tree_root(image)?;
    let narrowed = narrow_the_entry_width_of_the_node(&mut chain.mapping_root_node, kept)?;
    let what = format!(
        "条目宽：中央映射树根（住根记录，盘 {:?} 槽 {}，层级 {}）{}（key 宽 27）；节点两道 + 根槽自证一道校验和重算",
        chain.mapping_root_devices,
        chain.mapping_root_slot,
        chain.mapping_root_node[INDEX_NODE_LEVEL_OFFSET],
        narrowed.render(),
    );
    seal_and_write_back_the_central_mapping_chain(chain, image);
    Some(what)
}

/// 分配记录那一条改成什么样（普查 R6 / R7 / R8 / R9）。单列一个枚举，是为了让
/// [`rewrite_an_allocation_record`] 的 `match` 只面对它真做得出的那四样，用不着兜底臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocationRecordDamage {
    SlotBelowTheUnitArea,
    SpanPastTheEndOfTheUnitArea,
    SameSlotAsTheSecondRecord,
    DeviceOutsideThePool,
    InstanceTablePlacementReleasedOnTheSecondDeviceOnly,
}

/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
/// （越过单元区末尾，普查 R6 / R7），并把链上的校验和逐道重算：叶两道 → 上一层第 0 条条目子指针里的整单元校验和、那一层两道 →
/// …… → 分配记录树根两道 → 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
///
/// **分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14）**，这条记录先撞上的是叶那一判：起点不在这片叶按位置罩的那一段里
/// （读者报「分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽」）。「跨度越过单元区末尾」那一判只剩罩着盘末尾的那片叶上
/// 的记录够得着（一条落在叶里、末槽越过盘末尾的记录），这条坏法打不到它。
///
/// **字节由调用方从盘上读来、改完自己写回去**：`allocation_tree_nodes_from_the_root` 是从根起、每层沿第 0 条条目往下读到的那一路
/// （根在前、叶在末）。坏盘输入那一路读的是 [`MemoryPool`]，步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
/// （`code-discipline.md`「重复要生成，不许手抄」）。跨度字段整个写成
/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 15 位的最大值、已释放位跟着清掉
/// （只有还着的记录才会被影子账拿去隔离）。
///
/// 交回「坏在哪」那句话；树表里没有分配记录树、这一路的末一个不是叶、中间有一层没有条目、或者叶里一条记录都没有时交回 `None`。
#[must_use]
pub fn move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
    root_slot: &mut [u8],
    tree_table_node: &mut [u8],
    allocation_tree_nodes_from_the_root: &mut [Vec<u8>],
    unit_area_end_slot: u64,
) -> Option<String> {
    let leaf = allocation_tree_nodes_from_the_root.last_mut()?;
    let layout = IndexNodeEntryLayout::of(leaf);
    if layout.entry_count == 0 || !index_node_is_a_leaf(leaf) {
        return None;
    }
    let tree_table_layout = IndexNodeEntryLayout::of(tree_table_node);
    let allocation_entry_index = (0..tree_table_layout.entry_count).find(|index| {
        read_u16_at(
            tree_table_node,
            tree_table_layout.offset_of_entry(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
        ) == TREE_KIND_ALLOCATION_RECORDS
    })?;

    let first_entry = layout.offset_of_entry(0);
    let slot_offset = first_entry + ALLOCATION_RECORD_SLOT_OFFSET;
    let old_slot = read_six_byte_unsigned_at(leaf, slot_offset);
    let last_slot_of_the_unit_area = unit_area_end_slot.checked_sub(1)?;
    write_six_byte_unsigned_at(leaf, slot_offset, last_slot_of_the_unit_area);
    let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
    let old_span_field = read_u16_at(leaf, span_offset);
    leaf[span_offset..span_offset + 2]
        .copy_from_slice(&SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes());
    reseal_index_node(leaf);
    // 迭代上界是这一路的节点数；每一轮把下一层的整单元校验和写进这一层第 0 条条目的子指针、再封这一层。
    for level_from_the_root in (0..allocation_tree_nodes_from_the_root.len() - 1).rev() {
        let (above, below) =
            allocation_tree_nodes_from_the_root.split_at_mut(level_from_the_root + 1);
        let node = above.last_mut()?;
        let child_checksum = crc32_castagnoli_table(&below[0]);
        let node_layout = IndexNodeEntryLayout::of(node);
        if node_layout.entry_count == 0 || index_node_is_a_leaf(node) {
            return None;
        }
        let pointer_start = node_layout.offset_of_entry(0) + index_node_key_width(node);
        write_unit_checksum_into_both_locations(
            &mut node[pointer_start..pointer_start + 86],
            child_checksum,
        );
        reseal_index_node(node);
    }
    let allocation_tree_root = allocation_tree_nodes_from_the_root.first()?;

    let root_pointer_start = tree_table_layout.offset_of_entry(allocation_entry_index)
        + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let allocation_tree_checksum = crc32_castagnoli_table(allocation_tree_root);
    write_unit_checksum_into_both_locations(
        &mut tree_table_node[root_pointer_start..root_pointer_start + 86],
        allocation_tree_checksum,
    );
    reseal_index_node(tree_table_node);

    let tree_table_checksum = crc32_castagnoli_table(tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = root_slot.len();
    reseal_wide_checksum_field(root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);

    Some(format!(
        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上每一道校验和重算",
        old_span_field & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
    ))
}

/// 分配记录树（按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）里一把 key 的两段：(盘, 槽号)，按数值比（盘上小端，逐字节比不对）。
fn allocation_record_key_fields(key: &[u8]) -> (u32, u64) {
    (
        read_u32_at(key, ALLOCATION_RECORD_DEVICE_OFFSET),
        read_six_byte_unsigned_at(key, ALLOCATION_RECORD_SLOT_OFFSET),
    )
}

/// 分配器那一族：把分配记录树某一片叶里的一条记录改坏（普查 R6 / R7 / R8 / R9）。分配记录树按位置寻址、根恒在第 1 层以上
/// （D8（核心索引结构） 已定项 14）：记录住在叶里，改哪一片叶按坏法定（实例表那一条改它所在的那片，别的改最左那片），
/// 改完沿路往上把整单元校验和补到根、再补树表与根槽。
fn rewrite_an_allocation_record(
    image: &mut MemoryPool,
    damage: AllocationRecordDamage,
) -> Option<String> {
    let mut chain = open_chain_to_the_root_of(image, TREE_KIND_ALLOCATION_RECORDS)?;
    let target_key = match damage {
        AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly => {
            let instance_table_pointer = &chain.root_slot
                [ROOT_INSTANCE_TABLE_POINTER_OFFSET..ROOT_INSTANCE_TABLE_POINTER_OFFSET + 86];
            (
                read_u32_at(
                    instance_table_pointer,
                    POINTER_SECOND_LOCATION_OFFSET + LOCATION_DEVICE_OFFSET_IN_ENTRY,
                ),
                read_six_byte_unsigned_at(
                    instance_table_pointer,
                    POINTER_FIRST_LOCATION_OFFSET + LOCATION_SLOT_OFFSET_IN_ENTRY,
                ),
            )
        }
        AllocationRecordDamage::SlotBelowTheUnitArea
        | AllocationRecordDamage::SpanPastTheEndOfTheUnitArea
        | AllocationRecordDamage::SameSlotAsTheSecondRecord
        | AllocationRecordDamage::DeviceOutsideThePool => (0, 0),
    };
    let mut path = path_down_to_the_leaf_holding(image, &chain.tree_root_node, &|entry_key| {
        allocation_record_key_fields(entry_key) <= target_key
    })?;
    let leaf = &mut path.last_mut()?.bytes;
    let what = damage_an_allocation_record_in_the_leaf(leaf, &chain.root_slot, damage)?;
    reseal_the_path_below_the_root(image, &mut chain.tree_root_node, &mut path);
    seal_and_write_back_the_chain(chain, image);
    Some(what)
}

/// 把 `leaf`（分配记录树的一片叶）里的一条记录按坏法改掉；交回「坏在哪」那句话，这片叶上没有要坏的对象时交回 `None`。
fn damage_an_allocation_record_in_the_leaf(
    leaf: &mut [u8],
    root_slot: &[u8],
    damage: AllocationRecordDamage,
) -> Option<String> {
    let layout = IndexNodeEntryLayout::of(leaf);
    if layout.entry_count == 0 || !index_node_is_a_leaf(leaf) {
        return None;
    }
    let first_entry = layout.offset_of_entry(0);
    let what = match damage {
        AllocationRecordDamage::SlotBelowTheUnitArea => {
            let old = read_six_byte_unsigned_at(leaf, first_entry + ALLOCATION_RECORD_SLOT_OFFSET);
            write_six_byte_unsigned_at(
                leaf,
                first_entry + ALLOCATION_RECORD_SLOT_OFFSET,
                SLOT_FAR_BELOW_THE_UNIT_AREA,
            );
            format!(
                "分配器：第一条分配记录的槽号 {old} → {SLOT_FAR_BELOW_THE_UNIT_AREA}（单元区起点是 {UNIT_AREA_START_SLOT}）"
            )
        }
        AllocationRecordDamage::SpanPastTheEndOfTheUnitArea => {
            let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
            let old = read_u16_at(leaf, span_offset);
            leaf[span_offset..span_offset + 2].copy_from_slice(
                &SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes(),
            );
            format!(
                "分配器：第一条分配记录的跨度字段 {old:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）",
                old & 0x7FFF
            )
        }
        AllocationRecordDamage::SameSlotAsTheSecondRecord => {
            if layout.entry_count < 2 {
                return None;
            }
            let second_entry = layout.offset_of_entry(1);
            let key_width = index_node_key_width(leaf);
            let second_key = leaf[second_entry..second_entry + key_width].to_vec();
            leaf[first_entry..first_entry + key_width].copy_from_slice(&second_key);
            format!(
                "分配器：第一条分配记录的 key 换成第二条的（盘 {}、槽 {}），两条罩住同一个槽",
                read_u32_at(&second_key, ALLOCATION_RECORD_DEVICE_OFFSET),
                read_six_byte_unsigned_at(&second_key, ALLOCATION_RECORD_SLOT_OFFSET)
            )
        }
        AllocationRecordDamage::DeviceOutsideThePool => {
            let device_offset = first_entry + ALLOCATION_RECORD_DEVICE_OFFSET;
            let old = read_u32_at(leaf, device_offset);
            leaf[device_offset..device_offset + 4]
                .copy_from_slice(&DEVICE_IDENTITY_OUTSIDE_THE_POOL.to_le_bytes());
            format!(
                "分配器：第一条分配记录的设备身份 {old} → {DEVICE_IDENTITY_OUTSIDE_THE_POOL}（池里只有 0 与 1）"
            )
        }
        AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly => {
            // 这一版实例表的落点：每次发布都重写实例表，所以这个落点一定走到释放判定路径
            // （`TransactionUnit::InstanceTable` 的落点从根记录里那条指针取，不经映射）。
            let instance_table_pointer = &root_slot
                [ROOT_INSTANCE_TABLE_POINTER_OFFSET..ROOT_INSTANCE_TABLE_POINTER_OFFSET + 86];
            let slot = read_six_byte_unsigned_at(
                instance_table_pointer,
                POINTER_FIRST_LOCATION_OFFSET + LOCATION_SLOT_OFFSET_IN_ENTRY,
            );
            // 第二条位置条目那块盘：释放判定路径要核到它，不只核第一条那块。
            let second_device = read_u32_at(
                instance_table_pointer,
                POINTER_SECOND_LOCATION_OFFSET + LOCATION_DEVICE_OFFSET_IN_ENTRY,
            );
            let entry_index = (0..layout.entry_count).find(|index| {
                let entry = layout.offset_of_entry(*index);
                read_u32_at(leaf, entry + ALLOCATION_RECORD_DEVICE_OFFSET) == second_device
                    && read_six_byte_unsigned_at(leaf, entry + ALLOCATION_RECORD_SLOT_OFFSET)
                        == slot
            })?;
            let entry = layout.offset_of_entry(entry_index);
            let span_offset = entry + ALLOCATION_RECORD_SPAN_OFFSET;
            let old_span_field = read_u16_at(leaf, span_offset);
            if old_span_field & SPAN_FIELD_RELEASED_BIT != 0 {
                return None;
            }
            leaf[span_offset..span_offset + 2]
                .copy_from_slice(&(old_span_field | SPAN_FIELD_RELEASED_BIT).to_le_bytes());
            let generation_offset = entry + ALLOCATION_RECORD_GENERATION_OFFSET;
            let old_generation = u64::from_le_bytes(
                leaf[generation_offset..generation_offset + 8]
                    .try_into()
                    .expect("8 字节"),
            );
            leaf[generation_offset..generation_offset + 8]
                .copy_from_slice(&RELEASE_GENERATION_ABOVE_EVERY_ROOT.to_le_bytes());
            format!(
                "分配器：实例表落点（槽 {slot}）在盘 {second_device} 上那条分配记录（第 {entry_index} 条）改成已释放、释放代 {old_generation} → {RELEASE_GENERATION_ABOVE_EVERY_ROOT}（高过任何根，挂载时不会被回收掉）；盘 {} 上那条原样",
                read_u32_at(
                    instance_table_pointer,
                    POINTER_FIRST_LOCATION_OFFSET + LOCATION_DEVICE_OFFSET_IN_ENTRY
                )
            )
        }
    };
    Some(what)
}

/// 改完水位那一行的标签之后，记账树根（叶）节点头里的最大 key 怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LargestKeyOfTheNodeHeader {
    /// 跟着改成水位那一行的新 key（那一行排在最后，换上的标签仍排在最后）：节点头的 key 区间照旧与首末两条对得上，读者走到取水位那一步。
    FollowsTheRelabelledRow,
    /// 照旧：头里的最大 key 还是水位那一行原来的 key，读者先判区间（代码审阅第 24 / 33 条之后的两个读者都这样），红在区间上。
    LeftAsItWas,
}

/// 分配器那一族（普查 R11）：把记账树里 inode 号水位那一行改挂到登记表外的标签上，读路径取水位时找不到它。
/// 节点头的最大 key 按 `largest_key` 办；跟着改时要求水位那一行是最后一条（换上的标签比登记表里哪个都大，只有它原本在最后，
/// 改完才仍按 key 严格递增），不是就交回 None。
fn relabel_the_inode_watermark_row(
    image: &mut MemoryPool,
    largest_key: LargestKeyOfTheNodeHeader,
) -> Option<String> {
    let mut chain = open_chain_to_the_root_of(image, TREE_KIND_ACCOUNTING)?;
    if !index_node_is_a_leaf(&chain.tree_root_node) {
        return None;
    }
    let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
    // 台阶行（D5（快照 / 空间记账机制） 已定项 2 射程）与每次发布写的那一行同标签、代段更早、排在前面：改的是最后那一条（现行值那一行）。
    let watermark_row = (0..layout.entry_count).rev().find(|index| {
        read_u16_at(
            &chain.tree_root_node,
            layout.offset_of_entry(*index) + ACCOUNTING_ENTRY_STATISTIC_OFFSET,
        ) == STATISTIC_INODE_WATERMARK_LABEL
    })?;
    let statistic_offset =
        layout.offset_of_entry(watermark_row) + ACCOUNTING_ENTRY_STATISTIC_OFFSET;
    chain.tree_root_node[statistic_offset..statistic_offset + 2]
        .copy_from_slice(&STATISTIC_LABEL_NOT_IN_THE_REGISTRY.to_le_bytes());
    let header = match largest_key {
        LargestKeyOfTheNodeHeader::FollowsTheRelabelledRow => {
            if watermark_row + 1 != layout.entry_count {
                return None;
            }
            let key_width = index_node_key_width(&chain.tree_root_node);
            let row_start = layout.offset_of_entry(watermark_row);
            let largest_key_offset = INDEX_NODE_SMALLEST_KEY_OFFSET + key_width;
            chain
                .tree_root_node
                .copy_within(row_start..row_start + key_width, largest_key_offset);
            "，节点头的最大 key 跟着改成这一行的新 key"
        }
        LargestKeyOfTheNodeHeader::LeftAsItWas => "，节点头的最大 key 照旧",
    };
    let what = format!(
        "分配器：记账树第 {watermark_row} 条的统计量标签 {STATISTIC_INODE_WATERMARK_LABEL}（inode 号水位）→ {STATISTIC_LABEL_NOT_IN_THE_REGISTRY}{header}"
    );
    seal_and_write_back_the_chain(chain, image);
    Some(what)
}

/// 写侧断言那一族（普查 R12）：把每块盘每一个自证过的系统配置槽里的区域数改到那个长 3 的数组之外，整槽校验和重算。
fn widen_the_region_count_past_the_array(image: &mut MemoryPool) -> Option<String> {
    let geometry = newest_geometry(image)?;
    let mut changed = Vec::new();
    let devices: Vec<u32> = ImageReader::devices(image);
    for device in devices {
        for slot_offset in [0, geometry.slot_spacing] {
            let Some(mut slot) =
                ImageReader::read(image, device, slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
            else {
                continue;
            };
            if check_system_configuration_slot(&slot).is_err() {
                continue;
            }
            let old = slot[SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET];
            slot[SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET] =
                REGION_COUNT_PAST_THE_THREE_REGION_ARRAY;
            reseal_wide_checksum_field(
                &mut slot,
                SYSTEM_CONFIGURATION_SLOT_BYTES,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(device)) {
                sparse.write(DeviceOffsetInBytes(slot_offset), &slot);
            }
            changed.push(format!("盘 {device} 偏移 {slot_offset}（{old} → {REGION_COUNT_PAST_THE_THREE_REGION_ARRAY}）"));
        }
    }
    (!changed.is_empty()).then(|| format!("写侧断言：区域数改到数组之外——{}", changed.join("、")))
}

/// 写侧断言那一族（普查 R5）：把最新那条根记录里实例表指针的两条位置条目改成不同的槽号，整槽校验和重算。
fn make_two_location_entries_disagree_on_the_slot(image: &mut MemoryPool) -> Option<String> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let device = newest_root.device;
    let offset = newest_root.offset_in_bytes;
    let mut slot = newest_root.slot_bytes;
    let pointer_start = ROOT_INSTANCE_TABLE_POINTER_OFFSET;
    let view = parse_node_pointer(&slot[pointer_start..pointer_start + 86]);
    if view.all_zero {
        return None;
    }
    let second_slot_offset =
        pointer_start + POINTER_SECOND_LOCATION_OFFSET + LOCATION_SLOT_OFFSET_IN_ENTRY;
    let old = read_six_byte_unsigned_at(&slot, second_slot_offset);
    let changed = old + SLOT_OFFSET_THAT_BREAKS_TWO_DEVICES_ON_ONE_SLOT;
    write_six_byte_unsigned_at(&mut slot, second_slot_offset, changed);
    let cover_end = slot.len();
    reseal_wide_checksum_field(&mut slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    image
        .devices
        .get_mut(&DeviceIdentity(device))?
        .write(DeviceOffsetInBytes(offset), &slot);
    Some(format!(
        "写侧断言：盘 {device} 偏移 {offset} 的根记录里实例表指针，第二条位置条目的槽号 {old} → {changed}（第一条还是 {}）",
        view.locations[0].slot
    ))
}

/// extent 树上段叶条目的标签：载荷是下段根的节点指针 / 这个文件唯一那个数据单元的数据指针（D8（核心索引结构） 已定项 14）。
const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;

/// 最新那条根下 extent 树第一个数据单元的数据指针那 88 字节：extent 树按位置寻址（D8（核心索引结构） 已定项 14），
/// 上段沿第 0 条条目走到最左那片叶，取第一条上段叶条目（key 24 + 标签 1 + 载荷 88）；标签 2 时载荷就是那个数据指针，
/// 标签 1 时载荷的前 86 字节是下段根指针，下段再沿第 0 条条目走到最左那片叶，取第一条 extent 叶记录（key 24 + 数据指针 88）的指针。
/// 哪一步走不通（条目宽窄于字段表、标签是 0 或不认识、节点读不出）交回 `None`。
fn first_data_pointer_of_the_extent_tree(image: &MemoryPool) -> Option<Vec<u8>> {
    let chain = open_chain_to_the_root_of(image, TREE_KIND_EXTENT)?;
    let upper_path = path_down_to_the_leaf_holding(image, &chain.tree_root_node, &|_| false)?;
    let upper_leaf: &[u8] = upper_path
        .last()
        .map_or(chain.tree_root_node.as_slice(), |node| {
            node.bytes.as_slice()
        });
    let upper_layout = IndexNodeEntryLayout::of(upper_leaf);
    let key_width = index_node_key_width(upper_leaf);
    if upper_layout.entry_count == 0 || upper_layout.entry_width < key_width + 1 + 88 {
        return None;
    }
    let tag_offset = upper_layout.offset_of_entry(0) + key_width;
    let payload = &upper_leaf[tag_offset + 1..tag_offset + 1 + 88];
    match upper_leaf[tag_offset] {
        EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT => Some(payload.to_vec()),
        EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT => {
            let (_, _, lower_root) = read_unit_via_pointer(
                image,
                &payload[..86],
                usize::try_from(NODE_BYTES).expect("16384"),
            )?;
            let lower_path = path_down_to_the_leaf_holding(image, &lower_root, &|_| false)?;
            let lower_leaf: &[u8] = lower_path
                .last()
                .map_or(lower_root.as_slice(), |node| node.bytes.as_slice());
            let lower_layout = IndexNodeEntryLayout::of(lower_leaf);
            if lower_layout.entry_count == 0
                || lower_layout.entry_width
                    < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112")
            {
                return None;
            }
            let data_pointer_start =
                lower_layout.offset_of_entry(0) + index_node_key_width(lower_leaf);
            Some(lower_leaf[data_pointer_start..data_pointer_start + 88].to_vec())
        }
        _ => None,
    }
}

/// 判别力那一条：把最新那条根下 extent 树第一个数据单元整个重写，**只**重算这个单元自己的两道校验和，
/// 指着它的那条数据指针里位置条目上的整单元校验和原样留着。
/// 挡着它的只有恢复里 `read_unit_via_locations` 那一道整单元 CRC 比对：去掉它，恢复就读回这一版从没提交过的内容。
fn rewrite_a_data_unit_resealing_only_its_own_checksums(image: &mut MemoryPool) -> Option<String> {
    let data_pointer = first_data_pointer_of_the_extent_tree(image)?;
    let view = parse_data_pointer(&data_pointer);
    if view.all_zero || view.locations[0].slot != view.locations[1].slot {
        return None;
    }
    let devices = [view.locations[0].device, view.locations[1].device];
    let slot = view.locations[0].slot;
    let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut unit = ImageReader::read(image, devices[0], slot * SLOT_BYTES, unit_bytes)?;
    let declared_length = usize::from(read_u16_at(&unit, UNIT_DECLARED_LENGTH_OFFSET));
    if declared_length == 0 {
        return None;
    }
    let payload_start = usize::try_from(DATA_UNIT_PAYLOAD_OFFSET).expect("134");
    unit[payload_start..payload_start + declared_length]
        .fill(BYTE_THAT_NO_COMMITTED_VERSION_CONTAINS);
    reseal_data_unit(&mut unit);
    write_unit_to_both_devices(image, devices, slot, &unit);
    Some(format!(
        "判别力：盘 {devices:?} 槽 {slot} 的数据单元载荷（{declared_length} 字节）整个写成 {BYTE_THAT_NO_COMMITTED_VERSION_CONTAINS:#04x}，只重算它自己的两道校验和；extent 记录里的整单元校验和原样留着"
    ))
}
