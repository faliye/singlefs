//! 改大小与部分写的发布：旧数据单元的回读、一份副本读坏时的搬迁与免检单元的修复。

use super::*;

/// 改一个 inode 的 size（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K3、K5、K11、K12）：会话里 `UserChange::Truncate`
/// 发的那一次。任意活 inode 都能缩短与变长（用户 2026-10-26 定射程，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S6；
/// 删完第一个文件之后才建的 inode 也照这一条办）。按现在的 size S 与目标 N 分三条（[`SizeChangeOfARequest`]，只看两样操作期间不变的输入）：
/// - N = S：一次只改记录（改动计数、时间）的发布（K12）；
/// - N > S：变长，一次发布、不立意图（K5）：最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写，之后到 N 是洞；
/// - N < S：截断的意图那次（K2「先落目标」）：记录写 size = N，签截断条目，截到单元 k 中间时重写单元 k（K3），size 之外的单元照挂着，
///   由之后的共用走空（[`walk_the_deletion_queue_until_it_is_empty`]）一批批放掉；放 0 个单元的缩短也走这一条（K11）。
///
/// 记录的改动计数取这次发布的 checkpoint_txg、三个时间秒取 `write_time_seconds`（照 unlink 意图的写法）；各批不碰记录。
///
/// # Errors
/// 都在任何落盘动作之前：`inode` 不是一条活记录（没有、或 nlink = 0）⇒ `TruncationRefused(InodeIsNotALiveRecord)`；⌈N ÷ 净荷⌉ × 净荷
/// 溢出 u64 ⇒ `TruncationRefused(TargetSizeRoundedUpToAWholeDataUnitOverflows)`；要重写的尾单元读盘读不回或核不过（判决
/// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3：截断尾单元那条路与通用部分写同一读法，读盘、不取内存里的字节）⇒
/// `TruncationRefused(OldDataUnitToRewriteNotReadBack)`；其余同 `publish_version`。
///
/// # Panics
/// 截断那一条而上一版的承诺类队列不空（`deleted_inodes`、销毁中的快照、截断条目）：发截断意图的入口断言三样都空（K1），
/// 调用方（会话）先把队列走空再发它。
pub fn publish_size_change<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    inode: InodeNumber,
    target_size_in_bytes: u64,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let record = previous
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode.0 && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
        .copied()
        .ok_or(PublishError::TruncationRefused(
            TruncationRefusal::InodeIsNotALiveRecord { inode: inode.0 },
        ))?;
    if !target_size_rounded_up_to_a_whole_data_unit_fits(target_size_in_bytes) {
        return Err(PublishError::TruncationRefused(
            TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
                target_size_in_bytes,
            },
        ));
    }
    let size_change = SizeChangeOfARequest::of(record.size, target_size_in_bytes);
    // 单元数取 `inode` 自己的（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3，同 unlink 的
    // [`next_unlink_step_after`]）：第一个文件取它那一棵，别的文件取 `files_other_than_the_first` 那一项，没有的（size 0、上段没有条目）取 0。
    let data_units_existing = data_units_of_the_file_in_the_version(previous, inode);
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 改动计数取成局部量，理由同 [`publish_unlink_intent`] 那一处。
    let change_count_of_the_size_change = txg.0;
    // 截断字段（判决 m4-trunc-r2 R2 与第六节用户定案）：截断意图那次按这次放不放单元判写不写（`truncate` 里的纯函数），
    // 变长与 N = S 照抄上一版。
    let truncation_field = match size_change {
        SizeChangeOfARequest::Truncation => truncation_field_after_the_truncation_intent(
            record.truncation_field,
            data_units_existing,
            target_size_in_bytes,
            txg,
        ),
        SizeChangeOfARequest::SizeUnchanged | SizeChangeOfARequest::Extension => {
            record.truncation_field
        }
    };
    let record_after_the_change = InodeRecord {
        inode: record.inode,
        object_birth: record.object_birth,
        nlink: record.nlink,
        size: target_size_in_bytes,
        change_count: change_count_of_the_size_change,
        write_time_seconds,
        truncation_field,
    };
    let (step, logged_operations) = match size_change {
        SizeChangeOfARequest::SizeUnchanged => (
            DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
                record_after_the_change,
                tail_data_unit_rewrite: None,
            },
            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
        ),
        SizeChangeOfARequest::Extension => (
            DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
                record_after_the_change,
                tail_data_unit_rewrite: tail_data_unit_rewrite_of_an_extension(
                    data_units_existing,
                    record.size,
                    target_size_in_bytes,
                ),
            },
            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
        ),
        SizeChangeOfARequest::Truncation => {
            assert!(
                previous.queue_of_committed_intents_is_empty(),
                "发截断意图之前三样队列（deleted_inodes、销毁中的快照、截断条目）都已走空（判决 m4-trunc-r1 K1）"
            );
            (
                DeletionStepOfThePublish::TruncationIntent {
                    record_after_the_intent: record_after_the_change,
                    tail_data_unit_rewrite: tail_data_unit_rewrite_of_a_truncation(
                        data_units_existing,
                        record.size,
                        target_size_in_bytes,
                    ),
                },
                LoggedOperationsStepOfThePublish::IssueTheTruncationIntent(TruncationIntent {
                    inode,
                    object_birth: record.object_birth,
                    target_size_in_bytes,
                }),
            )
        }
    };
    // 要重写的尾单元在任何分配与写之前读盘读回（判决 m4-partial-write-r1 第二节 R3；D4（校验和位置） 已定项 4）：读不回或核不过就拒，
    // 盘上逐字节不变。新净荷是旧净荷截到新的声明长度（截断），或在后面补零到它（变长）；装单元那一步取这一份，不再读一次。
    let payloads_of_the_rewritten_data_units: Vec<RewrittenDataUnitPayload> =
        match step.tail_data_unit_rewrite() {
            Some(rewrite) => {
                // 只坏一份时计数（D4（校验和位置） 已定项 4）；这个尾单元这一次就整个重写到新槽，旧落点照释放之前读盘核隔离。
                let mut payload = read_back_the_old_payload_counting_a_failing_copy(
                    &*pool.devices,
                    previous,
                    inode,
                    record,
                    rewrite.unit,
                    &pool.parameters.filesystem_identifier,
                    &pool.old_data_unit_reads_back_with_one_copy_failing,
                    &pool.mapping_keys_of_old_data_units_read_back_with_one_copy_failing,
                )
                .map_err(|read_back| {
                    PublishError::TruncationRefused(
                        TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                            inode,
                            unit: rewrite.unit,
                            read_back,
                        },
                    )
                })?;
                payload.resize(
                    usize::try_from(rewrite.declared_length_in_bytes).expect("一个单元的净荷以内"),
                    0,
                );
                vec![RewrittenDataUnitPayload {
                    unit: rewrite.unit,
                    payload,
                    residue_read: ResidueReadOfTheNewSlot::NotRead,
                }]
            }
            None => Vec::new(),
        };
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step,
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
        logged_operations,
    };
    publish_deletion_step(
        pool,
        allocator,
        previous,
        txg,
        &deletion_and_lineage,
        &payloads_of_the_rewritten_data_units,
        instance,
    )
}

/// 一次按偏移的写交进来的字节，至少一个（[`publish_partial_write`]、会话的 `UserChange::WriteAtOffset`）。长度 0 的写条款没写（照 POSIX
/// 什么都不做、还是发一次只改时间的发布，判决 `research/prompts/m4-partial-write-r1-main-verification.md` 没判），类型上写不出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartialWriteBytes<'content> {
    pub(super) bytes: &'content [u8],
}

impl<'content> PartialWriteBytes<'content> {
    /// `bytes` 是空的 ⇒ `None`。
    #[must_use]
    pub fn new(bytes: &'content [u8]) -> Option<Self> {
        (!bytes.is_empty()).then_some(PartialWriteBytes { bytes })
    }

    #[must_use]
    pub const fn bytes(self) -> &'content [u8] {
        self.bytes
    }

    #[must_use]
    pub fn length_in_bytes(self) -> std::num::NonZeroU64 {
        std::num::NonZeroU64::new(
            u64::try_from(self.bytes.len()).expect("一段内存里的字节数装得进 u64"),
        )
        .expect("构造时核过不是空的")
    }
}

/// 一步按读回的旧单元（或零）重写的一个数据单元交给装单元那一步的新净荷（长度就是它这次的声明长度），与装之前读不读一次这次分到的新槽的
/// 残留（判决 m4-partial-write-r1 第二节 R2）。读—改—写在任何分配与写之前读完旧单元、改好这一段全部新净荷，装单元那一步不再读旧单元。
/// 一步里的新净荷按单元号连号升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewrittenDataUnitPayload {
    pub unit: DataUnitIndexInFile,
    pub payload: Vec<u8>,
    pub residue_read: ResidueReadOfTheNewSlot,
}

/// 装一个新单元之前读不读这次分到的槽的残留（判决 m4-partial-write-r1 第二节 R2：没有旧单元、这次写到它而没盖满的读一次）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidueReadOfTheNewSlot {
    /// 取定落点之后、任何写之前读一次，字节丢掉，读不出记数、不拒写。
    ReadOnceAndDiscardTheBytes,
    NotRead,
}

/// 读—改—写读回一个旧数据单元没读成的那几样（[`read_back_the_old_payload_of_a_data_unit`]）：调用方一律让那个写失败（D4（校验和位置） 已定项 4），
/// 成员只说是哪一样没过。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OldDataUnitReadBackFailure {
    /// 两条位置提示都读不出、或读出来整单元 CRC-32C 与位置条目里的对不上，上一版的中央映射里又查不到它的落点。
    UnreadableAtTheHintsAndMissingFromTheMapping,
    /// 两条位置提示与映射里的两条落点都读不出、或整单元 CRC-32C 对不上。
    UnreadableAtTheHintsAndAtTheMappedLocations,
    /// 读到的整单元 CRC-32C 对得上，单元头却解不开、与走到它的查找路径不符、或声明长度之后的补齐非零：`detail` 说是哪一样。
    DisagreesWithTheLookupPath { detail: &'static str },
}

/// 读—改—写读回 `inode` 那个文件第 `unit` 个旧数据单元的净荷（D4（校验和位置） 已定项 3「先把旧单元整个读回来并验」，判决
/// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3「读盘」）：先按上一版数据指针里的两条位置提示逐条读整单元，整单元 CRC-32C
/// 等于那条位置条目里的才算读到；两条都不算数，再按上一版的中央映射查它的落点、照那两条再读一遍（与挂载态读者
/// `mounted_read::OpenFileForRead::read_at` 同一种读法）。读到之后单元头对查找路径：出生树是 extent 树、对象是这个 inode、对象出生代与它那条记录
/// 相同、fsid 是本池的、出生 txg 与写序与指针的相同、锚点偏移是这个单元第一个字节的文件偏移、声明长度是按写之前的 size 算的那个，补齐为零（I-2.3）。
/// 只读盘，不动分配器、不发写；`record_before` 是写之前那条记录。
///
/// # Errors
/// 见 [`OldDataUnitReadBackFailure`]。
///
/// # Panics
/// 上一版里这个文件没有第 `unit` 个单元：调用方只读写之前存在的单元。
pub fn read_back_the_old_payload_of_a_data_unit<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    previous: &TransactionOutput,
    inode: InodeNumber,
    record_before: InodeRecord,
    unit: DataUnitIndexInFile,
    filesystem_identifier: &[u8; 16],
) -> Result<Vec<u8>, OldDataUnitReadBackFailure> {
    read_back_the_old_data_unit_telling_whether_a_copy_failed(
        reader,
        previous,
        inode,
        record_before,
        unit,
        filesystem_identifier,
    )
    .map(|read_back| read_back.payload)
}

/// 同 [`read_back_the_old_payload_of_a_data_unit`]，读到的是只坏一份时把 `reads_with_one_copy_failing` 加一、把这个单元在上一版里的映射 key
/// 追加进 `mapping_keys_read_with_one_copy_failing`：部分写与截断、变长读回旧单元走它，计数与 key 交写入口那两份
/// （[`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]、[`PoolWriter::mapping_keys_of_old_data_units_read_back_with_one_copy_failing`]）。
///
/// # Errors
/// 见 [`OldDataUnitReadBackFailure`]。
///
/// # Panics
/// 同 [`read_back_the_old_payload_of_a_data_unit`]。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：读者、上一版、哪个 inode、写之前的记录、哪个单元、本池 fsid、计数、映射 key 的账"
)]
fn read_back_the_old_payload_counting_a_failing_copy<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    previous: &TransactionOutput,
    inode: InodeNumber,
    record_before: InodeRecord,
    unit: DataUnitIndexInFile,
    filesystem_identifier: &[u8; 16],
    reads_with_one_copy_failing: &Cell<u64>,
    mapping_keys_read_with_one_copy_failing: &RefCell<Vec<Vec<u8>>>,
) -> Result<Vec<u8>, OldDataUnitReadBackFailure> {
    let read_back = read_back_the_old_data_unit_telling_whether_a_copy_failed(
        reader,
        previous,
        inode,
        record_before,
        unit,
        filesystem_identifier,
    )?;
    match read_back.copies {
        CopiesOfAUnitRead::WithoutAFailingCopy => {}
        CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed => {
            reads_with_one_copy_failing.set(reads_with_one_copy_failing.get() + 1);
            mapping_keys_read_with_one_copy_failing
                .borrow_mut()
                .push(mapping_key_for_data(
                    read_back.pointer.head,
                    read_back.pointer.write_order,
                ));
        }
    }
    Ok(read_back.payload)
}

/// 读—改—写读回时只坏一份的旧数据单元（`mapping_keys`，[`PoolWriter::mapping_keys_of_old_data_units_read_back_with_one_copy_failing`]）里，
/// 到 `version` 这一版仍被活快照钉着的那几个，照已有的搬迁路径另发一批时的计划（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元整个搬到新槽、
/// 两份新写，旧落点照释放前核隔离」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」）：被钉着的单元换下来进 deadlist、不释放（判决 m4-partial-write-r1
/// R7），映射条目留存（`version.mapping_entries_kept_for_the_snapshots`），坏的那一份留在旧落点上，读回它的那次发布不搬它；交回的每一项就是
/// [`publish_relocation_batch`] 收的那一种（源落点取留存条目的位置项，两盘同槽；跨度按单元类）。不在留存条目里的 key（那次发布里已经释放、
/// 照释放之前读盘核隔离了）不进计划。按 `mapping_keys` 的次序，同一把 key 只进一次（被拒重判的那次发布读回同一个单元，记两把）。只读内存。
///
/// # Panics
/// 留存条目里找得到的一把 key 是空的、或类标签不是三类用户可见单元之一：写入口记的 key 都由 [`mapping_key_for_data`] 算出，走不到。
#[must_use]
pub fn relocations_of_old_data_units_kept_for_the_snapshots(
    version: &TransactionOutput,
    mapping_keys: &[Vec<u8>],
) -> Vec<PlannedRelocation> {
    let mut planned: Vec<PlannedRelocation> = Vec::new();
    // 迭代上界是 `mapping_keys` 的个数（一次改动里只坏一份的读回次数）；跨轮携带的只有已进计划的那几项。
    for mapping_key in mapping_keys {
        if planned
            .iter()
            .any(|relocation| relocation.mapping_key == *mapping_key)
        {
            continue;
        }
        let Some((_, locations)) = version
            .mapping_entries_kept_for_the_snapshots
            .iter()
            .find(|(kept_key, _)| kept_key == mapping_key)
        else {
            continue;
        };
        let role = user_visible_role_standing_for_a_unit_class(
            *mapping_key
                .first()
                .expect("映射 key 27 字节，首字节是类标签"),
        )
        .expect("这几把 key 由 mapping_key_for_data 算出、类标签是数据单元，归得到用户可见角色");
        planned.push(PlannedRelocation {
            mapping_key: mapping_key.clone(),
            source: Placement {
                slot: locations[0].slot,
                span: role.span_slots(),
            },
        });
    }
    planned
}

/// 一版里 `locations` 这几组位置条目认出来的结果（[`units_at_these_locations`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitsAtTheseLocations {
    /// 住在这几组位置条目上的、进中央映射或留存着映射条目的那几个单元的映射 key。一把 key 只出一次，按 `mapped_units` 的次序、留存的排在后面。
    pub mapping_keys: Vec<Vec<u8>>,
    /// 认不出映射 key 的那几组，按交进来的次序：豁免映射的三类（映射树自己的节点、树表、实例表片）的，与这一版里已经没有单元住着的。
    pub locations_without_a_mapping_key: Vec<[LocationEntry; 2]>,
}

/// 按一版认 `locations` 这几组位置条目（[`UnitsAtTheseLocations`]）：进中央映射的单元按映射里那两条位置条目认，留存的映射条目
/// （`version.mapping_entries_kept_for_the_snapshots`）按它记的那两条认。可写挂载交回之前按挂载之后那一版认建上一版读到只坏一份的那几组
/// （`recovery::PoolReaderCountingUnitReadsWithOneCopyFailing::locations_of_the_units_read_with_one_copy_failing`），会话下一次用户改动发成之后
/// 按那一版认别的读记下的那几组，认出的 key 进另发的那一批（用户 2026-11-05 定「可写挂载时搬」）。
/// 挂载这一串的发布、或这次改动已经换了落点的单元认不出：它们的旧落点只在读到只坏一份之后记着那组位置条目时走释放之前读盘核
/// （[`copies_of_the_exempt_units_read_with_one_copy_failing`] 管豁免映射的三类）。
#[must_use]
pub fn units_at_these_locations(
    version: &TransactionOutput,
    locations: &[[LocationEntry; 2]],
) -> UnitsAtTheseLocations {
    let mut mapping_keys: Vec<Vec<u8>> = Vec::new();
    if locations.is_empty() {
        return UnitsAtTheseLocations {
            mapping_keys,
            locations_without_a_mapping_key: Vec::new(),
        };
    }
    let mut locations_with_a_mapping_key: Vec<[LocationEntry; 2]> = Vec::new();
    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    // 迭代上界是这一版进映射的单元数加留存条目数；跨轮携带的只有已认出的那几把 key 与那几组位置条目。
    for (_, mapping_key) in &version.mapped_units {
        if mapping_keys.contains(mapping_key) {
            continue;
        }
        match central_mapping.locations_of_key(mapping_key) {
            MappingLookup::Found(mapped_locations) if locations.contains(&mapped_locations) => {
                mapping_keys.push(mapping_key.clone());
                locations_with_a_mapping_key.push(mapped_locations);
            }
            MappingLookup::Found(_)
            | MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {}
        }
    }
    for (kept_key, kept_locations) in &version.mapping_entries_kept_for_the_snapshots {
        if !mapping_keys.contains(kept_key) && locations.contains(kept_locations) {
            mapping_keys.push(kept_key.clone());
            locations_with_a_mapping_key.push(*kept_locations);
        }
    }
    UnitsAtTheseLocations {
        mapping_keys,
        locations_without_a_mapping_key: locations
            .iter()
            .filter(|read_locations| !locations_with_a_mapping_key.contains(read_locations))
            .copied()
            .collect(),
    }
}

/// 另发的那一批要换落点的豁免映射的单元（D4（校验和位置） 已定项 4 的三类：映射树自己的节点、树表、实例表片；[`repair_of_the_exempt_units_at_these_locations`]）：
/// 它们认不出映射 key、搬迁入口不收（[`RelocationBatchRefused::PinnedUnitInThePlan`]），换落点照它们自己的重写路子走：树表与映射树根每次发布都换，走不到这里；
/// 映射树根之下的节点经 `mapping_keys_routed_through_the_nodes` 把从根到叶那条路整条重写（[`plan_rewriting_the_paths_to_the_moved_entries`]，条目的位置不变）；
/// 实例表整条链 COW 重写、行不变（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」，没有不被改写的片）。
#[derive(Clone, Debug)]
pub struct RepairOfTheExemptUnits {
    /// 这一批换下的那几个豁免映射的角色，按映射树节点、实例表片的次序。
    pub units: Vec<TransactionUnit>,
    pub(super) mapping_keys_routed_through_the_nodes: Vec<Vec<u8>>,
    pub(super) instance_table_rewrite: Option<InstanceTableRewrite>,
    /// 认这几组时沿链读实例表读到只坏一份的那几片的位置条目（一片一组）：读到的片同一批接着换，调用方记进别的读那份账。
    pub locations_read_with_one_copy_failing_while_reading_the_chain: Vec<[LocationEntry; 2]>,
    /// 认这几组时实例表链读不出、解不开的那个失败：实例表那一半跳过（读不出也写不了行不变的重写），映射树节点与要搬的单元照发，结局报明实例表没换。
    pub instance_table_chain_unreadable: Option<RecoveryFailure>,
}

impl RepairOfTheExemptUnits {
    /// 这一批要不要为豁免映射的单元发。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
}

/// `locations_without_a_mapping_key`（[`UnitsAtTheseLocations`]）里此刻是 `version` 那一版豁免映射的单元的那几组要怎么换落点。
/// 映射树节点只比内存里这一版的指针；余下的有一组才沿链读实例表（只坏一份的读才有这几组，没有坏副本的发布不多读）。这一读经数只坏一份的那一层：
/// 读到只坏一份的片与账上记着的片一样，整条链同一批重写（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。链读不出、解不开时认不出余下那几组是不是实例表片，
/// 实例表那一半跳过，失败交在 [`RepairOfTheExemptUnits::instance_table_chain_unreadable`] 里。
/// 没有单元住着的那几组（这次改动已经换下的，树表与映射树根每次发布都换，在这里头）不进结果。
#[must_use]
pub fn repair_of_the_exempt_units_at_these_locations<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    version: &TransactionOutput,
    locations_without_a_mapping_key: &[[LocationEntry; 2]],
) -> RepairOfTheExemptUnits {
    let mut units: Vec<TransactionUnit> = Vec::new();
    let mut mapping_keys_routed_through_the_nodes: Vec<Vec<u8>> = Vec::new();
    let mut locations_left: Vec<[LocationEntry; 2]> = Vec::new();
    let mapping_tree = &version.central_mapping_tree;
    // 迭代上界是交进来的组数乘映射树的节点数；跨轮携带的只有认出的那几样。
    for locations in locations_without_a_mapping_key {
        let mapping_tree_node = mapping_tree
            .shape
            .nodes()
            .iter()
            .zip(&mapping_tree.pointers)
            .find(|(_, pointer)| pointer.locations == *locations)
            .map(|(node, _)| node);
        if let Some(node) = mapping_tree_node {
            units.push(
                MultiLevelCodeTwoTree::CentralMapping
                    .role_of_node(node.position, &mapping_tree.shape),
            );
            mapping_keys_routed_through_the_nodes
                .push(mapping_key_below_the_node(mapping_tree, node));
        } else {
            locations_left.push(*locations);
        }
    }
    if locations_left.is_empty() {
        return RepairOfTheExemptUnits {
            units,
            mapping_keys_routed_through_the_nodes,
            instance_table_rewrite: None,
            locations_read_with_one_copy_failing_while_reading_the_chain: Vec::new(),
            instance_table_chain_unreadable: None,
        };
    }
    let counting_reader = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(reader);
    let chain_read = instance_table_chain_of_root(&counting_reader, &version.root);
    let locations_read_with_one_copy_failing_while_reading_the_chain =
        counting_reader.locations_of_the_units_read_with_one_copy_failing();
    let chain = match chain_read {
        Ok(chain) => chain,
        Err(failure) => {
            return RepairOfTheExemptUnits {
                units,
                mapping_keys_routed_through_the_nodes,
                instance_table_rewrite: None,
                locations_read_with_one_copy_failing_while_reading_the_chain,
                instance_table_chain_unreadable: Some(failure),
            }
        }
    };
    let any_page_read_with_one_copy_failing = chain.page_pointers.iter().any(|page_pointer| {
        locations_left.contains(&page_pointer.locations)
            || locations_read_with_one_copy_failing_while_reading_the_chain
                .contains(&page_pointer.locations)
    });
    let instance_table_rewrite = if any_page_read_with_one_copy_failing {
        units.extend((0..chain.page_pointers.len()).map(|position| {
            TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
                u64::try_from(position).expect("片序号"),
            ))
        }));
        Some(InstanceTableRewrite {
            rows: chain.records.rows,
            replaced_chain: chain.page_pointers,
        })
    } else {
        None
    };
    RepairOfTheExemptUnits {
        units,
        mapping_keys_routed_through_the_nodes,
        instance_table_rewrite,
        locations_read_with_one_copy_failing_while_reading_the_chain,
        instance_table_chain_unreadable: None,
    }
}

/// `node` 下面的一把映射 key（沿头一个孩子走到叶、取叶里头一把）：把它当「条目换了位置」交给发布，从根到它那片叶的路整条重写，`node` 在路上。
///
/// # Panics
/// 节点的孩子不在同一份形状里、或叶或内部节点是空的：形状由规划或读树造出来，节点都不空、孩子都在（`code_two_tree` 读树那一步判过）。
fn mapping_key_below_the_node(
    mapping_tree: &CodeTwoTreeVersion,
    node: &CodeTwoTreeShapeNode,
) -> Vec<u8> {
    let mut contents = &node.contents;
    // 迭代上界：每一轮下降一层，至多树高那么多轮；走到叶就停。
    loop {
        match contents {
            CodeTwoTreeNodeContents::Leaf { keys } => {
                return keys
                    .first()
                    .expect("映射树的叶不空：规划与读树都不留空叶")
                    .bytes()
                    .to_vec();
            }
            CodeTwoTreeNodeContents::Internal { children } => {
                let child = children
                    .first()
                    .expect("映射树的内部节点至少一个孩子：规划与读树都不留空节点")
                    .child;
                contents = &mapping_tree
                    .shape
                    .node(child)
                    .expect("孩子的位置取自同一份形状")
                    .contents;
            }
        }
    }
}

/// 这次发布换下的豁免映射的单元里，读到只坏一份之后记着位置条目的那几个（`locations_read_with_one_copy_failing`，
/// [`PoolAllocator::locations_read_with_one_copy_failing_waiting_for_the_next_user_change`]）照释放之前读盘核那一路读：一份没对上就两份都隔离
/// （D4（校验和位置） 已定项 4「旧落点照释放前核隔离」，读法同 [`copies_of_a_released_unit_failing_the_release_checksum_check`]）。
/// 没有记着的不读：豁免映射的单元不在硬规则 1 的射程里（D19（块指针的结构与宽度预算） 已定项 5），这一读只跟着读到只坏一份的那几组。
pub(super) fn copies_of_the_exempt_units_read_with_one_copy_failing<Reader: PoolReader + ?Sized>(
    previous: &TransactionOutput,
    previous_roles_replaced: &[TransactionUnit],
    instance_table: &InstanceTablePlan,
    locations_read_with_one_copy_failing: &[[LocationEntry; 2]],
    reader: &Reader,
) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
    let mut quarantined = Vec::new();
    if locations_read_with_one_copy_failing.is_empty() {
        return quarantined;
    }
    let mut check_when_it_was_read_with_one_copy_failing =
        |identity: TransactionUnit, locations: &[LocationEntry; 2]| {
            if locations_read_with_one_copy_failing.contains(locations) {
                quarantined.extend(
                    copies_of_a_released_unit_failing_the_release_checksum_check(
                        identity, locations, reader,
                    ),
                );
            }
        };
    // 迭代上界是这次换下的角色数；不是豁免映射的那几个走 `copies_failing_the_release_checksum_check`。
    for identity in previous_roles_replaced {
        match identity {
            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
                check_when_it_was_read_with_one_copy_failing(
                    *identity,
                    &previous
                        .central_mapping_tree
                        .pointer_of(*position)
                        .locations,
                );
            }
            TransactionUnit::MappingTree => check_when_it_was_read_with_one_copy_failing(
                *identity,
                &previous.root.mapping_root.locations,
            ),
            TransactionUnit::TreeTable => check_when_it_was_read_with_one_copy_failing(
                *identity,
                &previous.root.tree_table.locations,
            ),
            TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_)
            | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {}
        }
    }
    // 整条实例表旧链由计划带着（第 1 片起的指针不在上一版的内存态里）。
    match instance_table {
        InstanceTablePlan::Rewrite(rewrite) => {
            for (position, page_pointer) in rewrite.replaced_chain.iter().enumerate() {
                check_when_it_was_read_with_one_copy_failing(
                    TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
                        u64::try_from(position).expect("片序号"),
                    )),
                    &page_pointer.locations,
                );
            }
        }
        InstanceTablePlan::Carry(_) => {}
    }
    quarantined
}

/// 读到只坏一份的那几个单元（`mapping_keys`：可写挂载读盘建上一版读到的，与可写挂载期间别的读读到的，都是 [`units_at_these_locations`] 认出来的）里，到 `version`
/// 这一版仍在中央映射里或留存条目里的那几个，照已有的搬迁路径另发一批时的计划（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」；与
/// [`relocations_of_old_data_units_kept_for_the_snapshots`] 同一批发，`planned_before` 是那一份、已在里面的 key 不重复）：源落点取映射里那两条
/// 位置条目（或留存条目记的那两条）的槽，跨度按 key 的类标签归到的用户可见角色取（[`user_visible_role_standing_for_a_unit_class`]）。
/// 到 `version` 这一版已经不在的（这次改动或挂载之后的发布把它整个重写了、旧落点照释放之前读盘核那一路走）不进计划。
#[must_use]
pub fn relocations_of_the_units_read_with_one_copy_failing(
    version: &TransactionOutput,
    mapping_keys: &[Vec<u8>],
    planned_before: &[PlannedRelocation],
) -> Vec<PlannedRelocation> {
    let mut planned: Vec<PlannedRelocation> = Vec::new();
    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    // 迭代上界是 `mapping_keys` 的个数；跨轮携带的只有已进计划的那几项。
    for mapping_key in mapping_keys {
        if planned_before
            .iter()
            .chain(&planned)
            .any(|relocation| relocation.mapping_key == *mapping_key)
        {
            continue;
        }
        let Some(role) = mapping_key
            .first()
            .and_then(|unit_class| user_visible_role_standing_for_a_unit_class(*unit_class))
        else {
            continue;
        };
        let source_locations = match central_mapping.locations_of_key(mapping_key) {
            MappingLookup::Found(mapped_locations) => Some(mapped_locations),
            MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => version
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .find(|(kept_key, _)| kept_key == mapping_key)
                .map(|(_, kept_locations)| *kept_locations),
        };
        let Some(source_locations) = source_locations else {
            continue;
        };
        planned.push(PlannedRelocation {
            mapping_key: mapping_key.clone(),
            source: Placement {
                slot: source_locations[0].slot,
                span: role.span_slots(),
            },
        });
    }
    planned
}

/// 按一组两条位置条目读一个单元时读到的是哪一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopiesOfAUnitRead {
    /// 头一条就对上，或两条位置提示都没对上、映射那一组头一条对上（提示过期与两份都坏分不开，不算只坏一份），
    /// 或按提示读到只坏一份而映射里查到这个单元此刻的落点不是提示（提示过期：旧槽上坏的那一份不归这个单元计）。
    WithoutAFailingCopy,
    /// 同一组里前一条读不回字节或整单元 CRC-32C 对不上，后一条对上：只坏一份。
    FromTheOtherCopyAfterOneFailed,
}

/// 读回的旧单元：净荷与读到的是哪一份（[`read_back_the_old_data_unit_telling_whether_a_copy_failed`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OldDataUnitReadBack {
    pub payload: Vec<u8>,
    pub copies: CopiesOfAUnitRead,
    /// 上一版里走到这个单元的那条数据指针（只坏一份时按它的头与写序算映射 key，[`mapping_key_for_data`]）。
    pub pointer: DataPointer,
}

/// 同 [`read_back_the_old_payload_of_a_data_unit`]，另交回读到的是哪一份（[`CopiesOfAUnitRead`]）：部分写与截断、变长经
/// [`PoolWriter`] 读回时按它给写入口记账（[`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。
///
/// # Errors
/// 见 [`OldDataUnitReadBackFailure`]。
///
/// # Panics
/// 同 [`read_back_the_old_payload_of_a_data_unit`]。
pub fn read_back_the_old_data_unit_telling_whether_a_copy_failed<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    previous: &TransactionOutput,
    inode: InodeNumber,
    record_before: InodeRecord,
    unit: DataUnitIndexInFile,
    filesystem_identifier: &[u8; 16],
) -> Result<OldDataUnitReadBack, OldDataUnitReadBackFailure> {
    let position = usize::try_from(unit.0).expect("单元序号");
    let pointer = if inode.0 == FIRST_INODE_NUMBER {
        previous.data_pointers.get(position)
    } else {
        previous
            .file_other_than_the_first(inode)
            .and_then(|file| file.data_pointers.get(position))
    }
    .copied()
    .expect("读回的是写之前这个文件存在的单元");
    let role = role_of_a_data_unit_of_the_inode(inode, unit);
    let unit_bytes = usize::try_from(role.span_slots() * SLOT_BYTES)
        .expect("一个单元两槽以内，字节数装得进 usize");
    // 读到的是哪一份：同一组里前一条没读成、后一条对上时改记只坏一份（D4（校验和位置） 已定项 4 的计数）。
    let copies_read = Cell::new(CopiesOfAUnitRead::WithoutAFailingCopy);
    let read_by_the_locations = |locations: &[LocationEntry; 2]| {
        let mut an_earlier_copy_failed = false;
        locations.iter().find_map(|location| {
            let read = reader
                .read(
                    location.device,
                    location.slot.to_device_offset(),
                    unit_bytes,
                )
                .filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum);
            match (&read, an_earlier_copy_failed) {
                (Some(_), true) => {
                    copies_read.set(CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed);
                }
                (Some(_), false) => {}
                (None, _) => an_earlier_copy_failed = true,
            }
            read
        })
    };
    let bytes = match read_by_the_locations(&pointer.locations) {
        Some(bytes) => {
            // 按提示读到只坏一份：这个单元此刻的权威落点是上一版中央映射里那两条位置条目（D19（块指针的结构与宽度预算） 已定项 8），
            // 提示就是它们才算只坏一份；单元被整理搬走之后提示还指旧槽，旧槽上坏的那一份不归这个单元计
            // （判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 4 条，与冷走读、可写挂载建上一版、挂载态读同一判）。
            if copies_read.get() == CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed {
                let hints_are_the_mapping_position =
                    match MappedUnitsOfAVersionReadByRole::of_the_version(previous)
                        .locations_of_role(role)
                    {
                        MappingLookup::Found(mapped_locations) => {
                            mapped_locations == pointer.locations
                        }
                        // 查不出落点就判不出提示过期：这一份是真读到的坏副本，照计（查不到的判红留给照映射释放那一步）。
                        MappingLookup::NodeMalformedOrNoEntryWithThisKey
                        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => true,
                    };
                if !hints_are_the_mapping_position {
                    copies_read.set(CopiesOfAUnitRead::WithoutAFailingCopy);
                }
            }
            bytes
        }
        None => {
            let mapped_locations =
                match MappedUnitsOfAVersionReadByRole::of_the_version(previous)
                    .locations_of_role(role)
                {
                    MappingLookup::Found(locations) => locations,
                    MappingLookup::NodeMalformedOrNoEntryWithThisKey
                    | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => return Err(
                        OldDataUnitReadBackFailure::UnreadableAtTheHintsAndMissingFromTheMapping,
                    ),
                };
            read_by_the_locations(&mapped_locations)
                .ok_or(OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations)?
        }
    };
    let disagrees =
        |detail: &'static str| OldDataUnitReadBackFailure::DisagreesWithTheLookupPath { detail };
    let header = parse_data_unit(&bytes).map_err(|_unit_error| disagrees("单元头解不开"))?;
    if header.identity.tree != previous.tree_identifiers.extent {
        return Err(disagrees("出生树不是 extent 树"));
    }
    if header.identity.object != inode.0 {
        return Err(disagrees("对象不是这个 inode"));
    }
    if header.identity.object_birth != record_before.object_birth {
        return Err(disagrees("对象出生代与 inode 记录不符"));
    }
    if header.filesystem_identifier != unit_filesystem_identifier(filesystem_identifier) {
        return Err(disagrees("fsid 不是本池的"));
    }
    if header.birth_txg != pointer.head.birth_txg || header.write_order != pointer.write_order {
        return Err(disagrees("出生身份与指针不符"));
    }
    let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("32634");
    if header.identity.anchor_offset != unit.first_file_byte(payload_capacity).0 {
        return Err(disagrees("锚点偏移不是这个单元第一个字节的文件偏移"));
    }
    if u64::from(header.declared_length) != declared_length_within_a_size(unit, record_before.size)
    {
        return Err(disagrees("声明长度与写之前的 size 按净荷算出来的不符"));
    }
    let payload = data_unit_payload(&bytes, header.declared_length)
        .map_err(|_unit_error| disagrees("声明长度之后的补齐字节非零"))?;
    Ok(OldDataUnitReadBack {
        payload: payload.to_vec(),
        copies: copies_read.get(),
        pointer,
    })
}

/// 按偏移写一个活 inode（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节；任意活 inode，R6）：把 `bytes` 写到文件偏移
/// `offset` 起，一次发布（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」：与截断、变长走同一条发布路径，只是重写的那一段不同）。
/// 怎么切由 [`PartialWriteOfAFile`] 定：从偏移所在的单元起到写之后最后一个单元一起重写（R4，用户 2026-10-26 定「保留写序，写到 o 时把 o 之后的单元
/// 一起重写」），每个单元照 [`crate::write_request_split::DataUnitOfAPartialWrite`] 那一格装——有旧单元而没盖满的读盘读回旧单元（R3），没有旧单元而写到的读新槽残留一次、
/// 字节丢掉、旧内容按零（R2，用户 2026-10-26 定「首次写也读回」「残留读出错记数、不拒写」），空隙补整单元零、不读（R5）。写跨末尾时 size 换成写的
/// 终点、上段叶标签按写之后的单元数与 size 换（R5），截断字段照抄（T、N_w 部分写不碰）；对象出生代照记录、不改，改动计数取这次的 txg。
/// 快照下只把这次真换下的单元进 deadlist（计划交 `NotRewrittenWhole`，R7、截断判决 K4）。
///
/// # Errors
/// 都在任何分配与写之前，盘上逐字节不变：`inode` 不在上一版的 inode 树里 ⇒ `WriteToAnInodeThatIsNotInTheTree`；第一个文件已经删完而 `inode`
/// 的对象出生代晚于它墓碑的死亡 txg ⇒ `DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`（同整份写，K-U5 的 β 读法）；
/// nlink = 0 ⇒ `WriteToAnInodeBeingDeleted`；写的终点越过 u64 ⇒ `TruncationRefused(TargetSizeRoundedUpToAWholeDataUnitOverflows)`（带 `u64::MAX`），
/// ⌈写之后的 size ÷ 净荷⌉ × 净荷越过 u64 ⇒ 同一个成员带写之后的 size；这次重写的单元数多于一次发布的 journal 记录上限（一个单元一个事务一条记录）⇒
/// `JournalRecordsOfThePublishExceedTheLimit`；要读回的旧单元读盘读不回或核不过 ⇒ `TruncationRefused(OldDataUnitToRewriteNotReadBack)`
/// （外层为什么是改 size 的拒绝，见那个成员的文档）；其余同 `publish_version`。
///
/// # Panics
/// 上一版的承诺类意图队列不空（`deleted_inodes`、销毁中的快照、截断条目）：会话里每一次用户改动先走空再做它自己（D8（核心索引结构） 已定项 16
/// 第 6 条），直接调这个函数的调用方同样先走空（同 [`publish_sequential_write_of_an_inode`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、上一版、写哪个 inode、从哪个偏移写、写什么、写入时间、实例"
)]
pub fn publish_partial_write<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    inode: InodeNumber,
    offset: FileOffsetInBytes,
    bytes: PartialWriteBytes<'_>,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    // 三判与整份写同一套（[`plan_of_a_whole_write_of_a_file_other_than_the_first`]），写法另起：`crates/mutations.tsv` 按字面锚着那边的几串。
    let Some(record) = previous.inode_record_of(inode) else {
        let refusal = PublishError::WriteToAnInodeThatIsNotInTheTree { inode: inode.0 };
        return Err(refusal);
    };
    let born_after_the_first_file_was_deleted = previous
        .death_txg_of_the_deleted_first_file()
        .filter(|death_txg_of_the_first_file| *death_txg_of_the_first_file < record.object_birth);
    if let Some(death_txg_of_the_first_file) = born_after_the_first_file_was_deleted {
        let refusal =
            PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion {
                inode: inode.0,
                object_birth: record.object_birth,
                death_txg_of_the_first_file,
            };
        return Err(refusal);
    }
    if NLINK_OF_AN_INODE_BEING_DELETED == record.nlink {
        return Err(PublishError::WriteToAnInodeBeingDeleted { inode: inode.0 });
    }
    assert!(
        previous.queue_of_committed_intents_is_empty(),
        "部分写之前承诺类意图队列已走空（D8（核心索引结构） 已定项 16 第 6 条：会话与挂载续做都先走空）：inode {}",
        inode.0
    );
    let Some(write) = PartialWriteOfAFile::new(
        data_units_of_the_file_in_the_version(previous, inode),
        record.size,
        offset,
        bytes.length_in_bytes(),
    ) else {
        return Err(PublishError::TruncationRefused(
            TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
                target_size_in_bytes: u64::MAX,
            },
        ));
    };
    let size_after_the_write = write.size_after_in_bytes();
    if !target_size_rounded_up_to_a_whole_data_unit_fits(size_after_the_write) {
        return Err(PublishError::TruncationRefused(
            TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
                target_size_in_bytes: size_after_the_write,
            },
        ));
    }
    // 一个数据单元一个事务一条记录（C310（事务切分纪律与记录数口径打架） 用户定案）：重写的单元数多于一次发布的记录上限就在读旧单元、
    // 装新净荷之前拒，不为一次注定被拒的发布在内存里装那么多个单元（空隙补零的代价线性于空隙单元数，判决第二节 R5）。
    let rewritten_units = write.rewritten_units();
    refuse_a_publish_of_more_journal_records_than_the_limit(
        usize::try_from(rewritten_units.end - rewritten_units.start).unwrap_or(usize::MAX),
        pool.parameters.geometry.journal_ring_bytes,
    )?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 读回旧单元、改好新净荷都在任何分配与写之前（判决第二节 R3）：读不回或核不过就拒，盘上逐字节不变。
    let mut payloads_of_the_rewritten_data_units = Vec::new();
    // 迭代次数的上界是这次重写的单元数（上面按记录上限界住）；跨轮携带的只有往新净荷表里追加的那一项，提前出口只有读回失败那一个。
    for unit_number in rewritten_units {
        let unit = DataUnitIndexInFile(unit_number);
        let data_unit = write.data_unit(unit);
        let mut payload = if data_unit.reads_the_old_unit_back() {
            // 只坏一份时计数（D4（校验和位置） 已定项 4）；读回的单元这一次就整个重写到新槽，旧落点照释放之前读盘核隔离。
            read_back_the_old_payload_counting_a_failing_copy(
                &*pool.devices,
                previous,
                inode,
                record,
                unit,
                &pool.parameters.filesystem_identifier,
                &pool.old_data_unit_reads_back_with_one_copy_failing,
                &pool.mapping_keys_of_old_data_units_read_back_with_one_copy_failing,
            )
            .map_err(|read_back| {
                PublishError::TruncationRefused(
                    TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                        inode,
                        unit,
                        read_back,
                    },
                )
            })?
        } else {
            Vec::new()
        };
        // 旧内容之外的字节（空隙、洞里、新单元、末尾单元补满的那一段）一律按零：残留读到的字节一个都不进来（R2）。
        payload.resize(
            usize::try_from(write.declared_length_after_the_write(unit))
                .expect("一个单元的净荷以内"),
            0,
        );
        if let Some(written) = write.bytes_written_into(unit) {
            let into_the_payload =
                usize::try_from(written.offset_in_the_payload).expect("一个单元的净荷以内");
            let from_the_bytes_written = usize::try_from(written.offset_in_the_bytes_written)
                .expect("一段内存里的偏移装得进 usize");
            let length = usize::try_from(written.length_in_bytes).expect("一个单元的净荷以内");
            payload[into_the_payload..into_the_payload + length].copy_from_slice(
                &bytes.bytes()[from_the_bytes_written..from_the_bytes_written + length],
            );
        }
        payloads_of_the_rewritten_data_units.push(RewrittenDataUnitPayload {
            unit,
            payload,
            residue_read: if data_unit.reads_the_residue_of_its_slot() {
                ResidueReadOfTheNewSlot::ReadOnceAndDiscardTheBytes
            } else {
                ResidueReadOfTheNewSlot::NotRead
            },
        });
    }
    // 改动计数取成局部量，理由同 [`publish_unlink_intent`] 那一处（`crates/mutations.tsv` 按字面锚着别处的那几串）。
    let change_count_of_the_partial_write = txg.0;
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step: DeletionStepOfThePublish::PartialWrite {
            record_after_the_write: InodeRecord {
                inode: record.inode,
                object_birth: record.object_birth,
                nlink: record.nlink,
                size: size_after_the_write,
                change_count: change_count_of_the_partial_write,
                write_time_seconds,
                // 部分写不碰截断字段（判决第二节 R5：T、N_w 照抄，同变长）。
                truncation_field: record.truncation_field,
            },
            rewritten_data_units: DataUnitsRewrittenInOnePublish {
                first: write.first_rewritten_unit().0,
                end_exclusive: write.data_units_after(),
            },
        },
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
    };
    publish_deletion_step(
        pool,
        allocator,
        previous,
        txg,
        &deletion_and_lineage,
        &payloads_of_the_rewritten_data_units,
        instance,
    )
}
