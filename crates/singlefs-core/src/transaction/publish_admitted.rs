//! 准入过了之后的那一次发布：`publish_admitted`。

use super::*;

/// 准入之后的那一段：释放、分配、装单元，交回装好、还没落盘的字节（按持久顺序排好的 [`PublishWrites`]）与这一版
/// （写账是空的，落盘由调用方做：`persist_the_publish_or_freeze_it`）。失败时分配器由调用方退回，这里不管。
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "一次发布就是一件能单独验证的事：九个角色的装法与一条持久顺序，拆开只会把顺序藏进几个函数；\
              参数各是一样东西：写入口、分配器、计划、上一版、算好的树、要释放的、要隔离的、树号、卸载记号、谱系这一步、读回旧单元改好的新净荷"
)]
pub(super) fn publish_admitted<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    resolved: &ResolvedPublish,
    release: &[Placement],
    quarantine: &[CopyQuarantinedAfterReleaseChecksumMismatch],
    trees: FileVersionTreeIdentifiers,
    unmount_marker: UnmountMarker,
    lineage: LineageStepOfThePublish,
    snapshot: SnapshotStepOfThePublish,
    relocations: &[RelocationOfThePublish],
    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
) -> Result<(PublishWrites, TransactionOutput), PublishError> {
    let rewritten = &resolved.rewritten_roles;
    let txg = plan.txg;
    let instance = plan.instance;
    // 这次发布切成几条记录、每条点名哪几个角色、属于第几个事务（D23（journal 的角色与格式） 已定项 17：最后一个事务装不下一条
    // 记录时末条再跨记录）。
    let named_entry_capacity = pool.journal_record_named_entry_capacity();
    let roles_named_by_each_record =
        roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity);
    let transaction_offset_of_each_record =
        transaction_offset_of_each_record_of_the_publish(rewritten, named_entry_capacity);
    // 一个数据单元一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）：第 k 个事务是第 `plan.transaction + k` 号，
    // 共享的提交内生块在最后一个事务的记录里点名 ⇒ 它们头里的写序取最后那个事务的号（数据单元各带自己那个事务的号）。
    let last_transaction_of_this_publish = plan.transaction
        + transaction_offset_of_each_record
            .last()
            .copied()
            .expect("一次发布至少一条记录");
    let write_order = WriteOrder {
        instance,
        transaction: last_transaction_of_this_publish,
    };
    let file_content_transactions = plan.file_content_transactions();
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let mut sequences = BirthSequenceAllocator::default();
    // 第一个文件这次之后那条记录（截断意图与变长写的那一版、或照抄的）：尾单元的对象出生代与 extent 树上段叶条目的标签都读它的。
    let record_of_the_first_file_after_this_publish = resolved
        .inode_tree
        .containers
        .iter()
        .flat_map(|container| container.records.iter())
        .find(|record| record.inode == FIRST_INODE_NUMBER)
        .copied();

    // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
    // 释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录
    // 留在已分配、不改成已释放、分配代不变，跨度段置上隔离记号 0x4000、随这次发布落盘（D19（块指针的结构与宽度预算） 已定项 5，
    // 用户 2026-09-24 定案、记号 2026-10-22 定落点甲），核得上的那一份照常释放。
    for placement in release {
        let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
            .iter()
            .filter(|copy| copy.placement == *placement)
            .map(|copy| copy.device)
            .collect();
        allocator.release_leaving_the_record_allocated_on(
            *placement,
            txg,
            &devices_whose_copy_failed_the_checksum,
        );
    }
    // 第一个文件版本把树表 0 条那一版的分配记录树整棵换下了（上面释放的就是它的节点，`format_time_tree_table_to_release`）：
    // 分配器不再记着它，之后的发布按这一版自己的分配记录树走。
    if previous.is_none() {
        allocator.forget_the_allocation_record_tree_of_the_version_without_file();
    }
    // 隔离的那几份记进这次挂载的常驻隔离集（整理签发之后 R 里出现隔离槽就丢意图，攻方 K-c，`compaction_plan::region_holds_a_quarantined_slot`）。
    for copy in quarantine {
        allocator
            .compaction_mut()
            .note_quarantined_placement(copy.placement.slot, copy.placement.span);
    }
    // 整理的一批：源落点上面已经随释放进了 defer；目的落点在这次重写的角色之前取（固定点里走的是同一串，`settle_the_allocation_record_tree`），
    // 分配代写单元的诞生代。目的地定下之后，上一版拷一份、把搬走的单元的映射位置换到目的地（`take_the_mapping_positions_over_the_location_hints`）：
    // 照抄这些单元的角色、装映射条目、重写它们的父节点与树表时，读的都是这一份。
    let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(rewritten, allocator);
    let mut relocated_copies: Vec<(PublishedUnit, NamedUnit, UnitRelocatedByThisPublish)> =
        Vec::with_capacity(relocations.len());
    for relocation in relocations {
        let destination =
            allocate_relocation_destination(allocator, relocation, &rewrite_set_leaves)?;
        let destination_locations = pool.location_entries(destination.slot, &relocation.bytes);
        let mut key_tail = [0u8; 10];
        key_tail.copy_from_slice(&relocation.mapping_key[17..27]);
        let birth_tree = TreeIdentifier(u64::from_le_bytes(
            relocation.mapping_key[1..9]
                .try_into()
                .expect("映射 key 27 字节：类标签 1 + 出生树 8"),
        ));
        relocated_copies.push((
            PublishedUnit {
                slot: destination.slot,
                identity: relocation.role_for_the_checks,
                bytes: Arc::from(relocation.bytes.as_slice()),
            },
            NamedUnit {
                locations: destination_locations,
                unit_class: relocation.mapping_key[0],
                birth_tree,
                birth_txg: relocation.birth_txg,
                key_tail,
            },
            UnitRelocatedByThisPublish {
                mapping_key: relocation.mapping_key.clone(),
                holder: relocation.holder,
                source: relocation.source_locations,
                destination: destination_locations,
            },
        ));
    }
    // 映射条目换到目的地的那几把 key（只供测试的必红开关让映射留在源落点上时是空的）。
    let destination_of_key: BTreeMap<Vec<u8>, [LocationEntry; 2]> =
        match allocator.compaction().switches.relocation_publish_order {
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping => {
                relocated_copies
                    .iter()
                    .map(|(_, _, relocated)| (relocated.mapping_key.clone(), relocated.destination))
                    .collect()
            }
            RelocationPublishOrder::SourceReleasedWhileTheMappingStillNamesItForTests => {
                BTreeMap::new()
            }
        };
    // 留存的映射条目（被快照钉住的单元）照这次之后的那一份，搬走的换到目的地：装中央映射树与交回的这一版都读它。
    let kept_mapping_entries_after_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = resolved
        .deadlist
        .kept_mapping_entries_after_this_publish
        .iter()
        .map(|(key, locations)| {
            (
                key.clone(),
                destination_of_key.get(key).copied().unwrap_or(*locations),
            )
        })
        .collect();
    let previous_with_the_relocated_units_at_their_destinations: Option<TransactionOutput> =
        match (previous, destination_of_key.is_empty()) {
            (Some(previous_version), false) => {
                let mut relocated_version = previous_version.clone();
                take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {
                    destination_of_key.get(key).copied()
                });
                for (key, locations) in
                    &mut relocated_version.mapping_entries_kept_for_the_snapshots
                {
                    if let Some(destination) = destination_of_key.get(key) {
                        *locations = *destination;
                    }
                }
                Some(relocated_version)
            }
            (Some(_), true) | (None, _) => None,
        };
    let previous: Option<&TransactionOutput> =
        match &previous_with_the_relocated_units_at_their_destinations {
            Some(relocated_version) => Some(relocated_version),
            None => previous,
        };

    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
    for identity in rewritten {
        let placement =
            allocate_placement_for_role(allocator, *identity, txg, &rewrite_set_leaves)?;
        slots.insert(*identity, placement.slot);
    }
    // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，
    // 环里最旧有效根往前挪了就按谓词回收，记账行按回收之后的数写（与这条根一起说同一件事）。失败时整个分配器由调用方换回去。
    // 记之前照 E2 重读挂载时读不出、记了这条根之后会是窗口里最小的那一槽（D16（发布语义） 已定项 1，用户 2026-10-03 定）。
    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
        &*pool.devices,
        pool.parameters,
        allocator,
        txg,
    );
    let slot_of = |identity: TransactionUnit| slots[&identity];
    // 记完这条根、回收完之后的门槛与最小钉值就是根记录 t 与 p 的采样点（回收之后、装根之前，C773（读坏与全零根槽的回收门槛钉只在内存里））。
    let reclaim_floor_when_the_root_was_recorded = allocator
        .reclaim_floor_when_the_latest_root_was_recorded()
        .expect(
            "上面刚记过这条根：record_root_written_by_this_process 记完就记下回收门槛与最小钉值",
        );
    let device_identities: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let checkpoint = FileVersionCheckpoint {
        txg,
        write_order,
        trees,
        filesystem_identifier,
        device_identities: &device_identities,
    };

    // 部分写的新单元：落点取定之后、任何写之前（写都在这一段交回之后落盘）读一次这次分到的槽的残留（读到的不用、读不出只记数）。
    read_the_residue_of_the_new_slots(
        pool,
        resolved.data_units_rewritten_from_their_payloads,
        payloads_of_the_rewritten_data_units,
        &slots,
    );
    // 文件内容角色：第一个文件有新版本就按切分装每个数据单元，没有就照抄上一版的指针与字节（写第一个文件之外的文件的发布也照抄第一个文件）。
    let first_file_version = plan.first_file_version();
    let carried_file = match first_file_version {
        Some(_) => None,
        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
    };
    let FileVersionUnits {
        mut data_units,
        mut data_pointers,
        key_order_mismatches,
    } = match (first_file_version, carried_file) {
        (Some(file), _) => build_file_version_units(
            &checkpoint,
            file,
            &file_content_transactions,
            &FileVersionSlots {
                data: file_content_transactions
                    .iter()
                    .map(|transaction| {
                        slot_of(TransactionUnit::Data(transaction.unit_index_in_file))
                    })
                    .collect(),
            },
        ),
        // 照抄上一版有的那几个（部分写加单元时这一版比上一版多，多出来的由下面按新净荷装、接在后面）。
        (None, Some(carried)) => carried_file_version_units(
            carried,
            resolved.carried_data_units.min(carried.data_pointers.len()),
        ),
        (None, None) => {
            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
        }
    };
    // 按读回的旧单元（或零）重写的那一段（截断意图与变长的尾单元，判决 m4-trunc-r1 K3、K5；部分写从偏移所在的单元起到最后一个，判决
    // m4-partial-write-r1 第二节 R4）：别的单元照抄，这一段每个按入口在任何写之前读好、改好的新净荷装（不在这里再读一次旧单元），
    // 一个单元一个事务、写序带切给它的那个事务号（只有一个时就是这次发布的那一个，与提交内生块同一条记录）；单元号到了上一版单元数的
    // 是这次新加的，连号接在后面。
    let transactions_of_the_rewritten_data_units = plan
        .file_content_transactions_and_the_rewritten_data_units(
            resolved.data_units_rewritten_from_their_payloads,
        );
    let rewrites_data_units_of_the_first_file = resolved
        .data_units_rewritten_from_their_payloads
        .is_some_and(|rewritten_by_the_step| rewritten_by_the_step.inode.0 == FIRST_INODE_NUMBER);
    if first_file_version.is_none()
        && carried_file.is_some()
        && rewrites_data_units_of_the_first_file
    {
        let record = record_of_the_first_file_after_this_publish.expect(
            "按读回的旧单元重写第一个文件的那一步改的是它那条记录：截断意图、变长与部分写写的那一版记录在这一版里",
        );
        // 迭代次数的上界是这一步重写的单元数；跨轮携带的是往两张表里换进或接上的那一项，没有提前出口。
        for transaction in &transactions_of_the_rewritten_data_units {
            let data_unit_write_order = WriteOrder {
                instance,
                transaction: transaction.transaction_number,
            };
            let anchor_offset_of_the_rewritten_unit = transaction.payload_start.0;
            let data_unit = build_data_unit(
                DataUnitIdentity {
                    tree: trees.extent,
                    object: FIRST_INODE_NUMBER,
                    object_birth: record.object_birth,
                    anchor_offset: anchor_offset_of_the_rewritten_unit,
                },
                txg,
                filesystem_identifier,
                data_unit_write_order,
                payload_of_a_rewritten_data_unit(
                    payloads_of_the_rewritten_data_units,
                    *transaction,
                ),
            );
            let pointer = DataPointer {
                head: PointerHead {
                    birth_tree: trees.extent,
                    birth_txg: txg,
                },
                locations: location_entries(
                    &device_identities,
                    slot_of(TransactionUnit::Data(transaction.unit_index_in_file)),
                    &data_unit,
                ),
                write_order: data_unit_write_order,
            };
            let position = usize::try_from(transaction.unit_index_in_file.0).expect("单元序号");
            if position < data_pointers.len() {
                data_pointers[position] = pointer;
                data_units[position] = Arc::from(data_unit);
            } else {
                assert_eq!(
                    position,
                    data_pointers.len(),
                    "部分写新加的单元从上一版的单元数起连号接在后面"
                );
                data_pointers.push(pointer);
                data_units.push(Arc::from(data_unit));
            }
        }
    }
    // 第一个文件之外这次写的那个文件（按 inode 号写数据的入口）：按切分装它的每个数据单元——五元组与 extent key 的对象是它，
    // 写序各带自己那个事务的号，落点取它那一族角色的。
    let file_other_than_the_first_written: Option<(InodeNumber, FileVersionUnits)> = plan
        .file
        .as_ref()
        .filter(|file| file.inode.0 != FIRST_INODE_NUMBER)
        .map(|file| {
            (
                file.inode,
                build_file_version_units(
                    &checkpoint,
                    file,
                    &file_content_transactions,
                    &FileVersionSlots {
                        data: file_content_transactions
                            .iter()
                            .map(|transaction| {
                                slot_of(TransactionUnit::DataOfAFileOtherThanTheFirst {
                                    inode: file.inode,
                                    unit: transaction.unit_index_in_file,
                                })
                            })
                            .collect(),
                    },
                ),
            )
        });
    // 第一个文件之外那个文件这一步按读回的旧单元（或零）重写的那一段（截断意图、变长的尾单元，判决 m4-multi-inode-size-r1 第二节 S3；
    // 部分写的那一段，判决 m4-partial-write-r1 第二节 R6）：对象、角色、对象出生代全按它（不取第一个文件那条记录），新净荷取入口交进来的；
    // 一个单元一个事务，写序同第一个文件那一段。
    let data_units_of_a_file_other_than_the_first_rewritten: Vec<
        RewrittenDataUnitOfAFileOtherThanTheFirst,
    > = match (
        resolved.data_units_rewritten_from_their_payloads,
        carried_file,
    ) {
        (Some(rewritten_by_the_step), Some(_))
            if rewritten_by_the_step.inode.0 != FIRST_INODE_NUMBER =>
        {
            let inode = rewritten_by_the_step.inode;
            let record = resolved
                    .inode_tree
                    .containers
                    .iter()
                    .flat_map(|container| container.records.iter())
                    .find(|record| record.inode == inode.0)
                    .copied()
                    .expect("按读回的旧单元重写的那一步改的是这个文件那条记录：截断意图、变长与部分写写的那一版记录在这一版里");
            transactions_of_the_rewritten_data_units
                .iter()
                .map(|transaction| {
                    let data_unit_write_order = WriteOrder {
                        instance,
                        transaction: transaction.transaction_number,
                    };
                    let anchor_offset_of_the_rewritten_unit = transaction.payload_start.0;
                    let bytes = build_data_unit(
                        DataUnitIdentity {
                            tree: trees.extent,
                            object: inode.0,
                            object_birth: record.object_birth,
                            anchor_offset: anchor_offset_of_the_rewritten_unit,
                        },
                        txg,
                        filesystem_identifier,
                        data_unit_write_order,
                        payload_of_a_rewritten_data_unit(
                            payloads_of_the_rewritten_data_units,
                            *transaction,
                        ),
                    );
                    let pointer = DataPointer {
                        head: PointerHead {
                            birth_tree: trees.extent,
                            birth_txg: txg,
                        },
                        locations: location_entries(
                            &device_identities,
                            slot_of(TransactionUnit::DataOfAFileOtherThanTheFirst {
                                inode,
                                unit: transaction.unit_index_in_file,
                            }),
                            &bytes,
                        ),
                        write_order: data_unit_write_order,
                    };
                    RewrittenDataUnitOfAFileOtherThanTheFirst {
                        inode,
                        unit: transaction.unit_index_in_file,
                        bytes,
                        pointer,
                    }
                })
                .collect()
        }
        (Some(_) | None, _) => Vec::new(),
    };
    let key_order_mismatches_of_this_publish = key_order_mismatches
        + file_other_than_the_first_written
            .as_ref()
            .map_or(0, |(_, written_units)| written_units.key_order_mismatches);
    // 第一个文件之外各文件这一版的数据指针（与 `resolved.files_other_than_the_first.files` 同序）：这次写的那一个是这次装的，别的照抄上一版
    // （整理的一批之后上一版的拷贝里它们已换成映射位置）。照抄的文件约有上一版那么多个，按 inode 号查走上一版的索引（七·6）。
    let files_of_the_previous_version =
        previous.map(TransactionOutput::files_other_than_the_first_by_inode);
    // 上一版 `units` 按角色的索引一次发布建一次（七·6）：照抄 extent 树与多层码 2 树的节点、照抄全部单元那一圈都按角色查它。
    let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);
    let data_pointers_of_the_files_other_than_the_first: Vec<(InodeNumber, Vec<DataPointer>)> =
        resolved
            .files_other_than_the_first
            .files
            .iter()
            .map(|file_plan| match file_plan.origin {
                FileOtherThanTheFirstOrigin::RewrittenThisPublish => {
                    let (_, written_units) = file_other_than_the_first_written
                        .as_ref()
                        .expect("计划里整份重写的那个文件就是这次写的那一个（同一份计划拼的）");
                    (file_plan.inode, written_units.data_pointers.clone())
                }
                // 改 size 的那一个照抄留下的前几个，重写的尾单元换成这次装的那一个（判决 m4-multi-inode-size-r1 第二节 S3）；部分写的那一个
                // 照抄上一版的、重写的那一段换成这次装的，新加的连号接在后面（判决 m4-partial-write-r1 第二节 R6）。
                FileOtherThanTheFirstOrigin::SizeChangedThisPublish { .. } => {
                    let mut file_data_pointers: Vec<DataPointer> = files_of_the_previous_version
                        .as_ref()
                        .and_then(|files| files.first_entry_of(&file_plan.inode))
                        .map(|previous_file| {
                            previous_file
                                .data_pointers
                                .iter()
                                .take(usize::try_from(file_plan.data_units).expect("单元数"))
                                .copied()
                                .collect()
                        })
                        .unwrap_or_default();
                    for rewritten_data_unit in data_units_of_a_file_other_than_the_first_rewritten
                        .iter()
                        .filter(|rewritten_data_unit| rewritten_data_unit.inode == file_plan.inode)
                    {
                        let position =
                            usize::try_from(rewritten_data_unit.unit.0).expect("单元序号");
                        if position < file_data_pointers.len() {
                            file_data_pointers[position] = rewritten_data_unit.pointer;
                        } else {
                            assert_eq!(
                                position,
                                file_data_pointers.len(),
                                "部分写新加的单元从上一版的单元数起连号接在后面"
                            );
                            file_data_pointers.push(rewritten_data_unit.pointer);
                        }
                    }
                    assert_eq!(
                        u64::try_from(file_data_pointers.len()).expect("单元数"),
                        file_plan.data_units,
                        "改 size 与部分写之后这个文件的数据指针数就是计划里的单元数"
                    );
                    (file_plan.inode, file_data_pointers)
                }
                // 删除的一批截的那一个照抄留下的前几个（判决 m3-multi-file-unlink-r1 第三节 M1）。
                FileOtherThanTheFirstOrigin::CarriedFromThePreviousVersion
                | FileOtherThanTheFirstOrigin::CutByADeletionBatch => (
                    file_plan.inode,
                    files_of_the_previous_version
                        .as_ref()
                        .and_then(|files| files.first_entry_of(&file_plan.inode))
                        .expect("照抄的文件在上一版里")
                        .data_pointers
                        .iter()
                        .take(usize::try_from(file_plan.data_units).expect("单元数"))
                        .copied()
                        .collect(),
                ),
            })
            .collect();
    let multi_level_tree_context = MultiLevelTreeBuildContext {
        txg,
        filesystem_identifier,
        instance,
        device_identities: &device_identities,
    };
    // t2 extent 树（D8（核心索引结构） 已定项 14 的两段）：写文件内容的发布重写那个文件的下段整段与上段那一串，别的照抄。
    let extent_built = build_extent_tree(
        &resolved.extent_tree,
        &resolved.files_other_than_the_first,
        previous,
        units_of_the_previous_version.as_ref(),
        files_of_the_previous_version.as_ref(),
        &data_pointers,
        record_of_the_first_file_after_this_publish.map(|record| record.size),
        &data_pointers_of_the_files_other_than_the_first,
        trees.extent,
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );

    // t3 inode 树（字节表四）：这次重写的叶容器与根都在这里装；没重写的叶容器取上一版的指针，字节下面照抄。
    let previous_inode_leaf_containers: &[InodeLeafContainerVersion] = match previous {
        Some(previous_version) => &previous_version.inode_leaf_containers,
        None => &[],
    };
    let inode_tree_units = build_inode_tree_units(
        &checkpoint,
        &resolved.inode_tree,
        previous_inode_leaf_containers,
        &slots,
        &mut sequences,
    );
    // 第一个文件那条 inode 记录：树里 inode 号 1 的那一条。unlink 最后一批删掉它之后的每一版没有它（D8（核心索引结构） 已定项 16
    // 第 5 条；删完之后不再建带数据的文件，C618），那时数据单元一个不剩（`carried_data_units` 是 0）。
    let inode_record = inode_tree_units
        .leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == FIRST_INODE_NUMBER)
        .copied();
    assert!(
        inode_record.is_some() || data_pointers.is_empty(),
        "没有第一个文件那条记录的一版没有数据单元：最后一批同一次发布释放剩下的全部单元"
    );

    // 实例表链（码 3 打包记录类型 4）：重写时每一片的容器身份是 (0, 4, 片序号, 0)（第 0 片照 mkfs），归树 0、在树表之前发号
    // （mkfs 也是先实例表后树表）；多于一片时尾片先装、先发号（`build_instance_table_chain`）。
    let instance_table_rewrite: Vec<InstanceTablePageUnit> = match &plan.instance_table {
        InstanceTablePlan::Rewrite(rewrite) => build_instance_table_chain(
            &rewrite.rows,
            &slot_of,
            txg,
            write_order,
            filesystem_identifier,
            &device_identities,
            &mut sequences,
        ),
        InstanceTablePlan::Carry(_) => Vec::new(),
    };
    let instance_table_pointer = match &plan.instance_table {
        InstanceTablePlan::Carry(pointer) => *pointer,
        InstanceTablePlan::Rewrite(_) => {
            instance_table_rewrite
                .first()
                .expect("一条链至少一片：0 行也写出空的那一片")
                .pointer
        }
    };

    // t5 分配记录树（字节表五；D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，
    // 每盘各一条，按 (设备, 槽号) 升序分装进各自那片叶。重写哪几个节点在准入之前走到固定点定下来（`settle_the_allocation_record_tree`）：
    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配、带上隔离记号：那条记录照样变了，
    // 变的是拷贝上照常释放的同一片叶）。
    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let rewritten_allocation_nodes: BTreeSet<AllocationRecordTreeNode> = resolved
        .allocation_record_tree
        .rewritten_nodes()
        .into_iter()
        .collect();
    let previous_allocation_records: &[AllocationRecord] = match previous {
        Some(previous_version) => &previous_version.allocation_records,
        None => &[],
    };
    assert!(
        nodes_whose_contents_changed(
            &allocation_geometry,
            previous_allocation_records,
            &allocation_records
        )
        .is_subset(&rewritten_allocation_nodes)
            && nodes_holding_records(&allocation_geometry, &allocation_records)
                == resolved.allocation_record_tree.node_set(),
        "准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树：两边走的是同一串释放与取落点"
    );
    let allocation_built = build_allocation_record_tree(
        &resolved.allocation_record_tree,
        &allocation_geometry,
        &allocation_records,
        &|node| {
            previous.and_then(|previous_version| {
                previous_version.allocation_record_tree.pointer_of(node)
            })
        },
        trees.allocation_records,
        &multi_level_tree_context,
        &slot_of,
        &mut sequences,
    );

    // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
    // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
    // inode 号水位 = 下一个可用号（D5（快照 / 空间记账机制） 已定项 4 第 12 项）：每次发布重写这一行，
    // 这次建了几个 inode 就在上一版的水位上加几。树里最大的 key 恒小于它（I-9.6（水位大于两处最大号））——
    // 这里按「已经发出去的号」算，不按「树里现在有什么」算：号一旦发出去就不再复用（D8（核心索引结构） 已定项 6）。
    let new_inodes_of_this_publish =
        u64::try_from(plan.new_inode_records.len()).expect("这次新建的 inode 数");
    let inode_number_watermark_counted_from_the_previous_version = match previous {
        // 只供测试的开关装着时从压回去的那个号起算（C479），与 `publish_new_inodes` 起号读的是同一个开关、同一份上一版。
        Some(previous_version) => pool
            .inode_number_watermark_source()
            .watermark_this_publish_starts_from(previous_version.inode_number_watermark()),
        // 新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）；开关装着走到这里当场 panic（开关给错了）。
        None => pool
            .inode_number_watermark_source()
            .watermark_of_a_publish_without_a_previous_version(),
    } + new_inodes_of_this_publish;
    let inode_number_watermark = match lineage {
        // 管理员回退那次：水位是回退那一判算好的（D23（journal 的角色与格式） 已定项 14「水位」一格的 max），这次不建 inode；
        // 上一版（`mount::version_to_publish_the_rollback_after`）的记账行原样是 cur 的，台阶行照抄 cur 的。
        LineageStepOfThePublish::AdministratorRollback {
            inode_number_watermark_after_the_rollback,
            ..
        } => {
            assert_eq!(
                new_inodes_of_this_publish, 0,
                "管理员回退那次发布不建 inode（`mount::roll_back_by_a_forward_publish` 的计划里新 inode 记录是空的）"
            );
            inode_number_watermark_after_the_rollback
        }
        LineageStepOfThePublish::NoLineageRecord => {
            inode_number_watermark_counted_from_the_previous_version
        }
    };
    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    // inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程）：照抄上一版的，与准入之前算记账树形状读的是同一份。
    let inode_number_watermark_steps_carried =
        previous.map_or_else(Vec::new, TransactionOutput::inode_number_watermark_steps);
    let accounting_entries = accounting_entries_of_this_publish(
        txg,
        trees.inode,
        &device_identities_of_the_accounting_rows,
        &inode_number_watermark_steps_carried,
        &|row| match row {
            PoolWideAccountingRow::InodeNumberWatermark => inode_number_watermark,
            // 待删占用与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。
            PoolWideAccountingRow::PendingDeleteBytes => {
                crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0
            }
            // 标签 6 按这一版现算（D3（空间分配） 已定项 2）：发布路径判式子时读的是同一个函数算的同一个数。
            PoolWideAccountingRow::CommittedReservationBytes => {
                committed_reservation_for_tombstones_and_retirement_records_after_this_publish(
                    resolved,
                    previous,
                    ReplicaCount::of_every_device_in_the_pool(allocator),
                )
                .0
            }
        },
        &|row, device| {
            let device_map = allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == device)
                .expect("行按分配器里的那几块盘展开");
            match row {
                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,
                // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
                PerDeviceAccountingRow::FreeBytes => device_map.free_slots() * SLOT_BYTES,
                PerDeviceAccountingRow::UnreclaimableBytes => {
                    crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0
                }
                // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
                PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,
                PerDeviceAccountingRow::FragmentationRuns => device_map.free_runs(),
                PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),
            }
        },
    );
    // 记账树按计划装（D8（核心索引结构） 已定项 11）：行装不下一个节点时从中间切，节点按先叶后根发出生序号、取这次的落点。
    let accounting_built = build_multi_level_tree(
        MultiLevelCodeTwoTree::Accounting,
        trees.accounting,
        &resolved.accounting_tree,
        previous,
        units_of_the_previous_version.as_ref(),
        &accounting_entries
            .iter()
            .map(|entry| {
                (
                    CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING),
                    entry.to_bytes(),
                )
            })
            .collect(),
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );

    // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：叶条目就是 43 字节的 key（value 空），按计划装；
    // 这次没有条目进出时整棵照抄，一条都不剩时零根（一个节点都不装）。
    let deadlist_built = build_multi_level_tree(
        MultiLevelCodeTwoTree::Deadlist,
        trees.deadlist,
        &resolved.deadlist_tree,
        previous,
        units_of_the_previous_version.as_ref(),
        &resolved
            .deadlist
            .entries_after_this_publish
            .iter()
            .map(|entry| {
                let key_bytes = entry.key_bytes();
                (
                    CodeTwoTreeKey::new(&key_bytes, CodeTwoKeyFieldWidths::DEADLIST),
                    key_bytes,
                )
            })
            .collect(),
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );

    // 删除那两棵树（D8（核心索引结构） 已定项 16、D18（块里携带什么信息） 已定项 24）：不是删除的发布全部照抄。
    let deletion_tree_units =
        build_deletion_tree_units(&checkpoint, resolved, previous, &slots, &mut sequences);

    let node_pointer = |tree: TreeIdentifier,
                        identity: TransactionUnit,
                        unit: &[u8],
                        sequence: BirthSequence| NodePointer {
        head: PointerHead {
            birth_tree: tree,
            birth_txg: txg,
        },
        locations: pool.location_entries(slot_of(identity), unit),
        instance,
        birth_sequence: sequence,
    };
    // extent 树根（上段的根）的指针：装树时已经按这次的落点算好（照抄的取上一版）。
    let extent_pointer = *extent_built
        .version
        .upper_nodes
        .last()
        .map(|(_, pointer)| pointer)
        .expect("上段恒有第一个文件那片叶到根那一串");
    // inode 树根的指针：这次重写了就按这次的落点算，一片叶都没重写就取上一版树表里那一条。
    let inode_root_pointer = match (&inode_tree_units.rewritten_root, previous) {
        (Some(root), _) => node_pointer(
            trees.inode,
            TransactionUnit::InodeRoot,
            &root.bytes,
            root.birth_sequence,
        ),
        (None, Some(carried)) => carried.tree_root_pointer(trees.inode.0),
        (None, None) => {
            unreachable!("没有上一版的发布（第一个文件版本）恒要写 inode 记录 ⇒ 恒重写 inode 树")
        }
    };
    let allocation_pointer = allocation_built.version.root_pointer();
    let accounting_pointer = accounting_built.version.root_pointer();
    let deadlist_pointer = deadlist_built
        .version
        .pointers
        .last()
        .copied()
        .unwrap_or_else(NodePointer::empty_root);

    // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 的 extent 树每个节点、每片 inode 叶容器、inode 根、
    // 分配记录树每个节点、记账树的每个节点各一条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // 照抄的角色 key 照旧、位置照旧；重写的按这次的指针算。数据单元的码 1 key 带写序（事务号），
    // 一事务一单元 ⇒ 同一个文件的各个单元 key 不撞（D19（块指针的结构与宽度预算） 已定项 6 压在切分纪律上）。
    let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> =
        data_pointers
            .iter()
            .enumerate()
            .map(|(position, pointer)| {
                (
                    TransactionUnit::Data(DataUnitIndexInFile(
                        u64::try_from(position).expect("单元序号"),
                    )),
                    mapping_key_for_data(pointer.head, pointer.write_order),
                    pointer.locations,
                )
            })
            .collect();
    // 第一个文件之外各文件的数据单元接在第一个文件的后面（按 inode 号、单元序号），与 `resolve` 排映射 key 的次序相同。
    for (inode, file_data_pointers) in &data_pointers_of_the_files_other_than_the_first {
        for (position, pointer) in file_data_pointers.iter().enumerate() {
            mapped_units_with_locations.push((
                TransactionUnit::DataOfAFileOtherThanTheFirst {
                    inode: *inode,
                    unit: DataUnitIndexInFile(u64::try_from(position).expect("单元序号")),
                },
                mapping_key_for_data(pointer.head, pointer.write_order),
                pointer.locations,
            ));
        }
    }
    // extent 树：第一个文件的下段、第一个文件之外各文件的下段、上段（与装树发出生序号的次序相同）。
    let upper_root_level_of_this_version = extent_built.version.upper_root().level;
    let extent_tree_nodes_in_bump_order: Vec<(TransactionUnit, NodePointer)> = extent_built
        .version
        .lower_nodes
        .iter()
        .map(|(position, pointer)| (TransactionUnit::ExtentLowerNode(*position), *pointer))
        .chain(
            extent_built
                .lower_nodes_of_the_files_other_than_the_first
                .iter()
                .flat_map(|(inode, file_lower_nodes)| {
                    file_lower_nodes.iter().map(|(position, pointer)| {
                        (
                            TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
                                inode: *inode,
                                position: *position,
                            },
                            *pointer,
                        )
                    })
                }),
        )
        .chain(
            extent_built
                .version
                .upper_nodes
                .iter()
                .map(|(position, pointer)| {
                    (
                        role_of_extent_upper_node(*position, upper_root_level_of_this_version),
                        *pointer,
                    )
                }),
        )
        .collect();
    for (role, pointer) in &extent_tree_nodes_in_bump_order {
        mapped_units_with_locations.push((
            *role,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    for container in &inode_tree_units.leaf_containers {
        mapped_units_with_locations.push((
            TransactionUnit::InodeLeafContainer(container.index),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
            container.pointer.locations,
        ));
    }
    mapped_units_with_locations.push((
        TransactionUnit::InodeRoot,
        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
        inode_root_pointer.locations,
    ));
    for (node, pointer) in &allocation_built.version.nodes {
        mapped_units_with_locations.push((
            role_of_allocation_record_tree_node(*node),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    for (node, pointer) in accounting_built
        .version
        .shape
        .nodes()
        .iter()
        .zip(&accounting_built.version.pointers)
    {
        mapped_units_with_locations.push((
            MultiLevelCodeTwoTree::Accounting
                .role_of_node(node.position, &accounting_built.version.shape),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    for (node, pointer) in deadlist_built
        .version
        .shape
        .nodes()
        .iter()
        .zip(&deadlist_built.version.pointers)
    {
        mapped_units_with_locations.push((
            MultiLevelCodeTwoTree::Deadlist
                .role_of_node(node.position, &deadlist_built.version.shape),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
            pointer.locations,
        ));
    }
    if let Some(root) = &deletion_tree_units.deleted_inodes_root {
        mapped_units_with_locations.push((
            TransactionUnit::DeletedInodesRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
            root.pointer.locations,
        ));
    }
    for container in &deletion_tree_units.tombstone.containers {
        mapped_units_with_locations.push((
            TransactionUnit::TombstoneLeafContainer(container.index),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.node.pointer),
            container.node.pointer.locations,
        ));
    }
    for node in &deletion_tree_units.tombstone.nodes_below_the_root {
        mapped_units_with_locations.push((
            node.identity,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node.node.pointer),
            node.node.pointer.locations,
        ));
    }
    if let Some(root) = &deletion_tree_units.tombstone.root {
        mapped_units_with_locations.push((
            TransactionUnit::TombstoneRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
            root.pointer.locations,
        ));
    }
    for container in &deletion_tree_units.lineage.containers {
        mapped_units_with_locations.push((
            TransactionUnit::LineageLeafContainer(container.index),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.node.pointer),
            container.node.pointer.locations,
        ));
    }
    for node in &deletion_tree_units.lineage.nodes_below_the_root {
        mapped_units_with_locations.push((
            node.identity,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node.node.pointer),
            node.node.pointer.locations,
        ));
    }
    if let Some(root) = &deletion_tree_units.lineage.root {
        mapped_units_with_locations.push((
            TransactionUnit::LineageRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
            root.pointer.locations,
        ));
    }
    if let Some(root) = &deletion_tree_units.logged_operations_root {
        mapped_units_with_locations.push((
            TransactionUnit::LoggedOperationsRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
            root.pointer.locations,
        ));
    }
    for container in &deletion_tree_units.retirement.containers {
        mapped_units_with_locations.push((
            TransactionUnit::RetirementLeafContainer(container.index),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.node.pointer),
            container.node.pointer.locations,
        ));
    }
    for node in &deletion_tree_units.retirement.nodes_below_the_root {
        mapped_units_with_locations.push((
            node.identity,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node.node.pointer),
            node.node.pointer.locations,
        ));
    }
    if let Some(root) = &deletion_tree_units.retirement.root {
        mapped_units_with_locations.push((
            TransactionUnit::RetirementRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
            root.pointer.locations,
        ));
    }
    let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations
        .iter()
        .map(|(identity, key, _)| (*identity, key.clone()))
        .collect();
    assert_eq!(
        mapped_units, resolved.mapped_units,
        "中央映射树的形状按 resolve 现算的那一份映射 key 算过：这里装出来的 key 要逐项相等（出生序号按 bump 次序发，D19 已定项 9）"
    );
    // 被活快照钉住的单元的映射条目留在树里（判决 S3 的 α）：key 与两条位置项照上一版，这一版的树不引用它们。
    let mapping_entries_by_key: BTreeMap<CodeTwoTreeKey, Vec<u8>> = mapped_units_with_locations
        .iter()
        .map(|(_, key, locations)| (key, locations))
        .chain(
            kept_mapping_entries_after_this_publish
                .iter()
                .map(|(key, locations)| (key, locations)),
        )
        .map(|(key, locations)| {
            (
                CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
                build_mapping_entry(key, *locations),
            )
        })
        .collect();
    let mapping_keys: Vec<Vec<u8>> = mapping_entries_by_key
        .keys()
        .map(|key| key.bytes().to_vec())
        .collect();
    let mapping_built = build_multi_level_tree(
        MultiLevelCodeTwoTree::CentralMapping,
        trees.central_mapping,
        &resolved.central_mapping_tree,
        previous,
        units_of_the_previous_version.as_ref(),
        &mapping_entries_by_key,
        &multi_level_tree_context,
        &slots,
        &mut sequences,
    );
    let mapping_pointer = mapping_built.version.root_pointer();

    // t8 树表单元：十二条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
    // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己，extent 树、`deleted_inodes` 与墓碑树写它服务的头（inode 树的树 ID），其余 0。
    // `previous_snapshot_txg`（D5（快照 / 空间记账机制） 已定项 9）只写在活头 inode 树那一条与码 14 上：活头那一条照上一版的值续上
    // （没有上一版、或还没打过快照是 0），建快照那次换成这次的 checkpoint_txg（S.txg，已定项 14）；其余十一条写 0。
    let previous_snapshot_txg_of_the_live_inode_tree_before_this_publish = previous
        .and_then(|previous_version| {
            previous_version
                .tree_table_entries
                .iter()
                .find(|entry| entry.kind == TREE_KIND_INODE)
        })
        .map_or(CheckpointTxg(0), |entry| entry.previous_snapshot_txg);
    // 删快照的意图那次：被删的那个快照的前驱 P 传给那一刻的下一侧（D5（快照 / 空间记账机制） 已定项 12、已定项 13「prev 要跟着传递」，
    // 不许拆到以后的发布）——下一侧是活头就换活头 inode 条目的前驱，是下一条码 14 就换那一条的。
    let predecessor_passed_on_by_the_destruction_intent: Option<(
        NextSideOfASnapshot,
        CheckpointTxg,
    )> = match snapshot {
        SnapshotStepOfThePublish::DestructionIntent {
            snapshot: snapshot_being_destroyed,
        } => {
            let previous_version = previous.expect(
                "删快照的意图接在有那个快照的一版之后：publish_snapshot_destruction_intent 判过",
            );
            let destroyed_entry = snapshot_inode_root_entry(
                &previous_version.tree_table_entries,
                snapshot_being_destroyed,
            )
            .expect("被删的快照在上一版树表里：publish_snapshot_destruction_intent 判过");
            let next_side = next_side_of_the_snapshot(
                &previous_version.tree_table_entries,
                snapshot_being_destroyed,
            )
            .expect("被删的快照在上一版树表里：同上");
            Some((next_side, destroyed_entry.previous_snapshot_txg))
        }
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::CreateSnapshot { .. }
        | SnapshotStepOfThePublish::DestructionBatch { .. }
        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => None,
    };
    let previous_snapshot_txg_of_the_live_inode_tree = match snapshot {
        SnapshotStepOfThePublish::CreateSnapshot { .. } => txg,
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::DestructionIntent { .. }
        | SnapshotStepOfThePublish::DestructionBatch { .. }
        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => {
            match predecessor_passed_on_by_the_destruction_intent {
                Some((NextSideOfASnapshot::LiveHead, predecessor)) => predecessor,
                Some((NextSideOfASnapshot::Snapshot(_), _)) | None => {
                    previous_snapshot_txg_of_the_live_inode_tree_before_this_publish
                }
            }
        }
    };
    let table_entry =
        |kind: u16, tree: TreeIdentifier, root: NodePointer, head_identifier: u64| TreeTableEntry {
            kind,
            tree,
            root,
            previous_snapshot_txg: if kind == TREE_KIND_INODE {
                previous_snapshot_txg_of_the_live_inode_tree
            } else {
                CheckpointTxg(0)
            },
            birth_txg: plan.tree_birth_txg,
            head_identifier,
        };
    let mut tree_table_entries = vec![
        table_entry(
            TREE_KIND_EXTENT,
            trees.extent,
            extent_pointer,
            trees.inode.0,
        ),
        table_entry(
            TREE_KIND_INODE,
            trees.inode,
            inode_root_pointer,
            trees.inode.0,
        ),
        table_entry(
            TREE_KIND_ALLOCATION,
            trees.allocation_records,
            allocation_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_ACCOUNTING,
            trees.accounting,
            accounting_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_LIVELIST,
            trees.livelist,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_SPARSE_SIDE_TABLE,
            trees.sparse_side_table,
            NodePointer::empty_root(),
            0,
        ),
        // deadlist 树一条条目都没有时零根（同 day-1 注册的那一形，D5（快照 / 空间记账机制） 已定项 11）。
        table_entry(TREE_KIND_DEADLIST, trees.deadlist, deadlist_pointer, 0),
        // 第一个文件版本那次零根注册的四棵（D8（核心索引结构） 已定项 16 第 3 条、D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条）：
        // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：`deleted_inodes` 与墓碑树写 inode 树的树 ID，谱系树写 0（全池一份），
        // logged_ops 写 0（全池一份）。
        // 队列空 / 墓碑树零根时根指针置零（D8（核心索引结构） 已定项 16 第 2、5 条）。
        table_entry(
            TREE_KIND_DELETED_INODES,
            trees.deleted_inodes,
            deletion_tree_units
                .deleted_inodes_root
                .as_ref()
                .map_or_else(NodePointer::empty_root, |root| root.pointer),
            trees.inode.0,
        ),
        table_entry(
            TREE_KIND_TOMBSTONE,
            trees.tombstone,
            deletion_tree_units
                .tombstone
                .root
                .as_ref()
                .map_or_else(NodePointer::empty_root, |root| root.pointer),
            trees.inode.0,
        ),
        // 谱系树零根时根指针置零；回退那次追加之后指着它的根，之后每一版照抄（随现行那一版）。
        table_entry(
            TREE_KIND_LINEAGE,
            trees.lineage,
            deletion_tree_units
                .lineage
                .root
                .as_ref()
                .map_or_else(NodePointer::empty_root, |root| root.pointer),
            0,
        ),
        // logged_ops（判决 G1）：没有条目时根指针置零；签发那次指着新写的根，之后照抄，删光那次置零（随现行那一版）。头 ID 写 0（全池一份）。
        table_entry(
            TREE_KIND_LOGGED_OPERATIONS,
            trees.logged_operations,
            deletion_tree_units
                .logged_operations_root
                .as_ref()
                .map_or_else(NodePointer::empty_root, |root| root.pointer),
            0,
        ),
        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）：day-1 零根注册、排在最后，
        // 头 ID 写 inode 树的树 ID（同墓碑树）；零根时根指针置零，有身份消失之后指着它的根，之后每一版照抄（随现行那一版）。
        table_entry(
            TREE_KIND_RETIREMENT,
            trees.retirement,
            deletion_tree_units
                .retirement
                .root
                .as_ref()
                .map_or_else(NodePointer::empty_root, |root| root.pointer),
            trees.inode.0,
        ),
    ];
    assert!(
        tree_table_entries
            .windows(2)
            .all(|pair| pair[0].tree < pair[1].tree),
        "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：十三个号连号发、中央映射树不进树表，剩下十二条仍按发号次序升序"
    );
    // 只读快照的条目（码 14、15）随每一次发布续上（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2）：
    // 上一版树表里的照盘上原样、照原次序接在十二条之后（它们的树号是建快照那次从水位发的，比十二棵树的号都大）；建快照这一次再追加一对。
    // 删快照的最后一批同一次发布删掉被删那一对（判决 S4）；意图那次下一侧是下一条码 14 时换它的前驱。
    let snapshot_whose_entries_the_final_batch_deletes = match snapshot {
        SnapshotStepOfThePublish::DestructionFinalBatch {
            snapshot: snapshot_being_destroyed,
        } => Some(snapshot_being_destroyed),
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::CreateSnapshot { .. }
        | SnapshotStepOfThePublish::DestructionIntent { .. }
        | SnapshotStepOfThePublish::DestructionBatch { .. } => None,
    };
    let snapshot_entries_carried_from_the_previous_version: Vec<TreeTableEntry> = previous
        .map(|previous_version| {
            previous_version
                .tree_table_entries
                .iter()
                .filter(|entry| {
                    entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
                        || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT
                })
                .filter(|entry| {
                    snapshot_whose_entries_the_final_batch_deletes.is_none_or(|deleted| {
                        let is_the_deleted_inode_root =
                            entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT && entry.tree == deleted;
                        let is_the_deleted_extent_root = entry.kind
                            == TREE_KIND_SNAPSHOT_EXTENT_ROOT
                            && entry.head_identifier == deleted.0;
                        !is_the_deleted_inode_root && !is_the_deleted_extent_root
                    })
                })
                .map(
                    |entry| match predecessor_passed_on_by_the_destruction_intent {
                        Some((NextSideOfASnapshot::Snapshot(next), predecessor))
                            if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
                                && entry.tree == next.tree =>
                        {
                            TreeTableEntry {
                                kind: entry.kind,
                                tree: entry.tree,
                                root: entry.root,
                                previous_snapshot_txg: predecessor,
                                birth_txg: entry.birth_txg,
                                head_identifier: entry.head_identifier,
                            }
                        }
                        Some(_) | None => *entry,
                    },
                )
                .collect()
        })
        .unwrap_or_default();
    tree_table_entries.extend(snapshot_entries_carried_from_the_previous_version);
    // 这一版带「销毁中」记号的码 14：上一版的，加意图那次的，减最后一批的（判决 S4：意图写在条目自己身上，删掉那一对时记号跟着没了）。
    let mut snapshots_being_destroyed: Vec<TreeIdentifier> = previous
        .map(|previous_version| previous_version.snapshots_being_destroyed.clone())
        .unwrap_or_default();
    match snapshot {
        SnapshotStepOfThePublish::DestructionIntent {
            snapshot: snapshot_being_destroyed,
        } => {
            snapshots_being_destroyed.push(snapshot_being_destroyed);
            snapshots_being_destroyed.sort_unstable();
        }
        SnapshotStepOfThePublish::DestructionFinalBatch {
            snapshot: snapshot_being_destroyed,
        } => snapshots_being_destroyed.retain(|tree| *tree != snapshot_being_destroyed),
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::CreateSnapshot { .. }
        | SnapshotStepOfThePublish::DestructionBatch { .. } => {}
    }
    match snapshot {
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::DestructionIntent { .. }
        | SnapshotStepOfThePublish::DestructionBatch { .. }
        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => {}
        SnapshotStepOfThePublish::CreateSnapshot {
            snapshot_inode_root_tree,
            snapshot_extent_root_tree,
        } => {
            // 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1：根指针逐字抄这一版活头的两个根；诞生 txg = S.txg = 这次的
            // checkpoint_txg；码 14 的 `previous_snapshot_txg` = 捕获那一刻活头的值（换成 S.txg 之前那一个）、头 ID = 活头 inode 树的树号；
            // 码 15 的头 ID = 配对的码 14 的树号、`previous_snapshot_txg` 0；flags 0（位 0「销毁中」这一批不写）。
            tree_table_entries.push(TreeTableEntry {
                kind: TREE_KIND_SNAPSHOT_INODE_ROOT,
                tree: snapshot_inode_root_tree,
                root: inode_root_pointer,
                previous_snapshot_txg:
                    previous_snapshot_txg_of_the_live_inode_tree_before_this_publish,
                birth_txg: txg,
                head_identifier: trees.inode.0,
            });
            tree_table_entries.push(TreeTableEntry {
                kind: TREE_KIND_SNAPSHOT_EXTENT_ROOT,
                tree: snapshot_extent_root_tree,
                root: extent_pointer,
                previous_snapshot_txg: CheckpointTxg(0),
                birth_txg: txg,
                head_identifier: snapshot_inode_root_tree.0,
            });
        }
    }
    assert!(
        tree_table_entries
            .windows(2)
            .all(|pair| pair[0].tree < pair[1].tree),
        "快照条目的树号在建它那次从水位发、只增不减（D8（核心索引结构） 已定项 8 ②）：续上与追加之后树表仍按树 ID 严格升序"
    );
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &trees.extent.0.to_le_bytes(),
        &tree_table_entries
            .last()
            .expect("十二条之后才续快照条目：树表不空")
            .tree
            .0
            .to_le_bytes(),
        txg,
        filesystem_identifier,
        instance,
        tree_table_sequence,
        u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
        &tree_table_entries
            .iter()
            .map(|entry| {
                let mark = if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
                    && snapshots_being_destroyed.contains(&entry.tree)
                {
                    SnapshotDestructionMark::BeingDestroyed
                } else {
                    SnapshotDestructionMark::NotBeingDestroyed
                };
                entry.to_bytes_with_destruction_mark(mark)
            })
            .collect::<Vec<_>>(),
    );
    let tree_table_pointer = node_pointer(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        TransactionUnit::TreeTable,
        &tree_table_unit,
        tree_table_sequence,
    );

    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（按 bump 次序：每个数据单元、extent 根、每片 inode 叶容器、
    // inode 根、分配记录树、记账树与中央映射树的每个节点（树内先叶后根）、树表，实例表链的各片在末尾）。
    let rewritten_unit = |identity: TransactionUnit, bytes: Arc<[u8]>| PublishedUnit {
        slot: slot_of(identity),
        identity,
        bytes,
    };
    let data_role_at = |position: usize| {
        TransactionUnit::Data(DataUnitIndexInFile(
            u64::try_from(position).expect("单元序号"),
        ))
    };
    // 照抄第一个文件的发布：第一个文件的每个数据单元问它在不在这次重写的角色里（截断与变长重写的尾单元在里面）。这次重写的角色清单
    // 建一次按角色的索引、每个数据单元查一次（七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）：逐个在清单上找是第一个文件的
    // 数据单元数乘这次重写的角色数，第一个文件占池的大头又整份重写一个大文件时两样都到池大小的量级。先问完再退出那一处：
    // 下面照抄单元时按角色取上一版单元的那几次查不记在这里。第一个文件有新版本的发布一个都不问。
    let is_each_data_unit_of_the_carried_first_file_rewritten: Vec<bool> = match carried_file {
        None => Vec::new(),
        Some(_) => {
            let _inside_the_rewritten_roles = lookup_work::entering(
                LookupSite::RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile,
            );
            let rewritten_by_role =
                FirstEntryOfEachIdentity::of_the_table(rewritten, |identity| *identity);
            (0..data_units.len())
                .map(|position| {
                    rewritten_by_role
                        .first_entry_of(&data_role_at(position))
                        .is_some()
                })
                .collect()
        }
    };
    let mut units: Vec<PublishedUnit> = Vec::new();
    for (position, data_unit) in data_units.iter().enumerate() {
        let identity = data_role_at(position);
        units.push(match carried_file {
            Some(_) if !is_each_data_unit_of_the_carried_first_file_rewritten[position] => {
                carried_unit(
                    units_of_the_previous_version
                        .as_ref()
                        .expect("照抄第一个文件的发布接在上一版之后：carried_file 就是上一版"),
                    identity,
                )
            }
            None | Some(_) => rewritten_unit(identity, Arc::clone(data_unit)),
        });
    }
    // 第一个文件之外各文件的数据单元（按 inode 号、单元序号）：这次写的那一个是这次装的，别的从上一版照抄。
    for file_plan in &resolved.files_other_than_the_first.files {
        for position in 0..file_plan.data_units {
            let identity = file_plan.data_role_at(position);
            units.push(match file_plan.origin {
                FileOtherThanTheFirstOrigin::RewrittenThisPublish => {
                    let (_, written_units) = file_other_than_the_first_written
                        .as_ref()
                        .expect("计划里整份重写的那个文件就是这次写的那一个");
                    rewritten_unit(
                        identity,
                        Arc::clone(
                            written_units
                                .data_units
                                .get(usize::try_from(position).expect("单元序号"))
                                .expect("这次写的那个文件的单元数就是计划里的单元数"),
                        ),
                    )
                }
                FileOtherThanTheFirstOrigin::SizeChangedThisPublish { .. }
                    if file_plan
                        .origin
                        .rewrites_the_data_unit(DataUnitIndexInFile(position)) =>
                {
                    rewritten_unit(
                        identity,
                        Arc::from(
                            rewritten_data_unit_of_a_file_other_than_the_first(
                                &data_units_of_a_file_other_than_the_first_rewritten,
                                file_plan.inode,
                                DataUnitIndexInFile(position),
                            )
                            .expect("计划里重写的单元就是这次装的那一段里的（同一份计划拼的）")
                            .bytes
                            .as_slice(),
                        ),
                    )
                }
                FileOtherThanTheFirstOrigin::CarriedFromThePreviousVersion
                | FileOtherThanTheFirstOrigin::CutByADeletionBatch
                | FileOtherThanTheFirstOrigin::SizeChangedThisPublish { .. } => carried_unit(
                    units_of_the_previous_version
                        .as_ref()
                        .expect("照抄的文件只出现在接着上一版的发布里"),
                    identity,
                ),
            });
        }
    }
    units.extend(extent_built.units.iter().cloned());
    for container in &inode_tree_units.leaf_containers {
        let identity = TransactionUnit::InodeLeafContainer(container.index);
        units.push(match container.origin {
            InodeLeafContainerOrigin::RewrittenThisPublish => {
                rewritten_unit(identity, Arc::from(container.bytes.as_slice()))
            }
            // 照抄上一版那个叶序上的那一份（字节、落点不动），装进这一版时换成这一版的角色。
            InodeLeafContainerOrigin::CarriedFrom(previous_index) => {
                let carried = carried_unit(
                    units_of_the_previous_version
                        .as_ref()
                        .expect("没重写的叶容器只会出现在接着上一版的发布里"),
                    TransactionUnit::InodeLeafContainer(previous_index),
                );
                PublishedUnit {
                    slot: carried.slot,
                    identity,
                    bytes: carried.bytes,
                }
            }
        });
    }
    units.push(match &inode_tree_units.rewritten_root {
        Some(root) => rewritten_unit(TransactionUnit::InodeRoot, Arc::from(root.bytes.as_slice())),
        None => carried_unit(
            units_of_the_previous_version
                .as_ref()
                .expect("没重写 inode 根的发布恒接在上一版之后"),
            TransactionUnit::InodeRoot,
        ),
    });
    // 分配记录树按 bump 次序：重写的是这次装的，照抄的从上一版同一个位置那个角色拷。
    for (node, _) in &allocation_built.version.nodes {
        let identity = role_of_allocation_record_tree_node(*node);
        units.push(
            match allocation_built
                .rewritten_units
                .iter()
                .find(|unit| unit.identity == identity)
            {
                Some(allocation_record_tree_node_unit) => allocation_record_tree_node_unit.clone(),
                None => carried_unit(
                    units_of_the_previous_version
                        .as_ref()
                        .expect("照抄的分配记录树节点只出现在接着上一版的发布里"),
                    identity,
                ),
            },
        );
    }
    units.extend(accounting_built.units.iter().cloned());
    units.extend(deadlist_built.units.iter().cloned());
    let deletion_tree_unit =
        |identity: TransactionUnit, node: &DeletionTreeNodeUnit| match &node.rewritten {
            Some(rewritten_node) => {
                rewritten_unit(identity, Arc::from(rewritten_node.bytes.as_slice()))
            }
            None => carried_unit(
                units_of_the_previous_version
                    .as_ref()
                    .expect("照抄的删除树节点只出现在接着上一版的发布里"),
                identity,
            ),
        };
    if let Some(root) = &deletion_tree_units.deleted_inodes_root {
        units.push(deletion_tree_unit(TransactionUnit::DeletedInodesRoot, root));
    }
    // 容器树根之下的节点照抄时按它在上一版里的角色拷（长高那次的旧根上一版是根），装进这一版时换成这一版的角色。
    let container_tree_node_below_the_root_unit =
        |node: &ContainerTreeNodeBelowTheRootUnit| match &node.node.rewritten {
            Some(rewritten_node) => {
                rewritten_unit(node.identity, Arc::from(rewritten_node.bytes.as_slice()))
            }
            None => {
                let carried = carried_unit(
                    units_of_the_previous_version
                        .as_ref()
                        .expect("照抄的容器树节点只出现在接着上一版的发布里"),
                    node.identity_in_the_previous_version,
                );
                PublishedUnit {
                    slot: carried.slot,
                    identity: node.identity,
                    bytes: carried.bytes,
                }
            }
        };
    for container in &deletion_tree_units.tombstone.containers {
        units.push(deletion_tree_unit(
            TransactionUnit::TombstoneLeafContainer(container.index),
            &container.node,
        ));
    }
    for node in &deletion_tree_units.tombstone.nodes_below_the_root {
        units.push(container_tree_node_below_the_root_unit(node));
    }
    if let Some(root) = &deletion_tree_units.tombstone.root {
        units.push(deletion_tree_unit(TransactionUnit::TombstoneRoot, root));
    }
    for container in &deletion_tree_units.lineage.containers {
        units.push(deletion_tree_unit(
            TransactionUnit::LineageLeafContainer(container.index),
            &container.node,
        ));
    }
    for node in &deletion_tree_units.lineage.nodes_below_the_root {
        units.push(container_tree_node_below_the_root_unit(node));
    }
    if let Some(root) = &deletion_tree_units.lineage.root {
        units.push(deletion_tree_unit(TransactionUnit::LineageRoot, root));
    }
    if let Some(root) = &deletion_tree_units.logged_operations_root {
        units.push(deletion_tree_unit(
            TransactionUnit::LoggedOperationsRoot,
            root,
        ));
    }
    for container in &deletion_tree_units.retirement.containers {
        units.push(deletion_tree_unit(
            TransactionUnit::RetirementLeafContainer(container.index),
            &container.node,
        ));
    }
    for node in &deletion_tree_units.retirement.nodes_below_the_root {
        units.push(container_tree_node_below_the_root_unit(node));
    }
    if let Some(root) = &deletion_tree_units.retirement.root {
        units.push(deletion_tree_unit(TransactionUnit::RetirementRoot, root));
    }
    units.extend(mapping_built.units.iter().cloned());
    units.push(rewritten_unit(
        TransactionUnit::TreeTable,
        Arc::from(tree_table_unit.as_slice()),
    ));
    // 实例表链上的各片按链上的次序（第 0 片在前）：重写的是这次装的，照抄的是上一版带着的那几片。
    match (&plan.instance_table, previous) {
        (InstanceTablePlan::Rewrite(_), _) => {
            for page in &instance_table_rewrite {
                units.push(rewritten_unit(page.role, Arc::from(page.bytes.as_slice())));
            }
        }
        (InstanceTablePlan::Carry(_), Some(previous_version)) => {
            units.extend(
                previous_version
                    .units
                    .iter()
                    .filter(|unit| unit.identity.is_a_page_of_the_instance_table())
                    .cloned(),
            );
        }
        (InstanceTablePlan::Carry(_), None) => {}
    }

    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
        match identity {
                TransactionUnit::Data(index) => data_key_tail(
                    data_pointers
                        .get(usize::try_from(index.0).expect("单元序号"))
                        .expect("点名的数据单元是这次装的那几个之一")
                        .write_order,
                ),
                TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } => data_key_tail(
                    file_other_than_the_first_written
                        .as_ref()
                        .filter(|(written_inode, _)| *written_inode == inode)
                        .and_then(|(_, written_units)| {
                            written_units
                                .data_pointers
                                .get(usize::try_from(unit.0).expect("单元序号"))
                        })
                        .or_else(|| {
                            rewritten_data_unit_of_a_file_other_than_the_first(
                                &data_units_of_a_file_other_than_the_first_rewritten,
                                inode,
                                unit,
                            )
                            .map(|rewritten_data_unit| &rewritten_data_unit.pointer)
                        })
                        .expect("点名的第一个文件之外的数据单元是这次写的那个文件的、或这次重写的它的那一段里的")
                        .write_order,
                ),
                TransactionUnit::ExtentLowerNode(_)
                | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
                | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
                | TransactionUnit::ExtentRoot => node_key_tail(
                    instance,
                    *extent_built
                        .birth_sequences_of_rewritten_nodes
                        .get(&identity)
                        .expect("点名的 extent 树节点是这次重写的"),
                ),
                TransactionUnit::InodeLeafContainer(index) => node_key_tail(
                    instance,
                    inode_tree_units
                        .leaf_containers
                        .get(index.position())
                        .expect("点名的叶容器是这次重写的那几片之一")
                        .pointer
                        .birth_sequence,
                ),
                TransactionUnit::InodeRoot => node_key_tail(
                    instance,
                    inode_tree_units
                        .rewritten_root
                        .as_ref()
                        .expect("点名 inode 根的发布一定重写了它")
                        .birth_sequence,
                ),
                TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                | TransactionUnit::AllocationTree => node_key_tail(
                    instance,
                    *allocation_built
                        .birth_sequences_of_rewritten_nodes
                        .get(&identity)
                        .expect("点名的分配记录树节点是这次重写的"),
                ),
                TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
                | TransactionUnit::AccountingTree => node_key_tail(
                    instance,
                    accounting_built.birth_sequence_of_a_rewritten_node(identity),
                ),
                TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
                | TransactionUnit::DeadlistTree => node_key_tail(
                    instance,
                    deadlist_built.birth_sequence_of_a_rewritten_node(identity),
                ),
                TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                    node_key_tail(
                        instance,
                        mapping_built.birth_sequence_of_a_rewritten_node(identity),
                    )
                }
                TransactionUnit::DeletedInodesRoot => node_key_tail(
                    instance,
                    deletion_tree_units
                        .deleted_inodes_root
                        .as_ref()
                        .and_then(|root| root.rewritten.as_ref())
                        .expect("点名 deleted_inodes 根的发布一定重写了它")
                        .birth_sequence,
                ),
                TransactionUnit::LoggedOperationsRoot => node_key_tail(
                    instance,
                    deletion_tree_units
                        .logged_operations_root
                        .as_ref()
                        .and_then(|root| root.rewritten.as_ref())
                        .expect("点名 logged_ops 根的发布一定重写了它")
                        .birth_sequence,
                ),
                TransactionUnit::TombstoneLeafContainer(index) => node_key_tail(
                    instance,
                    deletion_tree_units
                        .tombstone
                        .containers
                        .get(index.position())
                        .and_then(|container| container.node.rewritten.as_ref())
                        .expect("点名的墓碑容器是这次重写的")
                        .birth_sequence,
                ),
                TransactionUnit::TombstoneRoot => node_key_tail(
                    instance,
                    deletion_tree_units
                        .tombstone
                        .root
                        .as_ref()
                        .and_then(|root| root.rewritten.as_ref())
                        .expect("点名墓碑树根的发布一定重写了它")
                        .birth_sequence,
                ),
                TransactionUnit::TombstoneNodeBelowTheRoot(_) => node_key_tail(
                    instance,
                    deletion_tree_units
                        .tombstone
                        .nodes_below_the_root
                        .iter()
                        .find(|node| node.identity == identity)
                        .and_then(|node| node.node.rewritten.as_ref())
                        .expect("点名的墓碑树根之下节点是这次重写的")
                        .birth_sequence,
                ),
                TransactionUnit::LineageLeafContainer(index) => node_key_tail(
                    instance,
                    deletion_tree_units
                        .lineage
                        .containers
                        .get(index.position())
                        .and_then(|container| container.node.rewritten.as_ref())
                        .expect("点名的谱系容器是这次重写的")
                        .birth_sequence,
                ),
                TransactionUnit::LineageRoot => node_key_tail(
                    instance,
                    deletion_tree_units
                        .lineage
                        .root
                        .as_ref()
                        .and_then(|root| root.rewritten.as_ref())
                        .expect("点名谱系树根的发布一定重写了它")
                        .birth_sequence,
                ),
                TransactionUnit::LineageNodeBelowTheRoot(_) => node_key_tail(
                    instance,
                    deletion_tree_units
                        .lineage
                        .nodes_below_the_root
                        .iter()
                        .find(|node| node.identity == identity)
                        .and_then(|node| node.node.rewritten.as_ref())
                        .expect("点名的谱系树根之下节点是这次重写的")
                        .birth_sequence,
                ),
                TransactionUnit::RetirementLeafContainer(index) => node_key_tail(
                    instance,
                    deletion_tree_units
                        .retirement
                        .containers
                        .get(index.position())
                        .and_then(|container| container.node.rewritten.as_ref())
                        .expect("点名的退役容器是这次重写的")
                        .birth_sequence,
                ),
                TransactionUnit::RetirementRoot => node_key_tail(
                    instance,
                    deletion_tree_units
                        .retirement
                        .root
                        .as_ref()
                        .and_then(|root| root.rewritten.as_ref())
                        .expect("点名退役树根的发布一定重写了它")
                        .birth_sequence,
                ),
                TransactionUnit::RetirementNodeBelowTheRoot(_) => node_key_tail(
                    instance,
                    deletion_tree_units
                        .retirement
                        .nodes_below_the_root
                        .iter()
                        .find(|node| node.identity == identity)
                        .and_then(|node| node.node.rewritten.as_ref())
                        .expect("点名的退役树根之下节点是这次重写的")
                        .birth_sequence,
                ),
                TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
                TransactionUnit::InstanceTable
                | TransactionUnit::InstanceTablePageAfterTheFirst(_) => node_key_tail(
                    instance,
                    instance_table_rewrite
                        .iter()
                        .find(|page| page.role == identity)
                        .expect("点名实例表那一片的发布一定重写了整条链")
                        .pointer
                        .birth_sequence,
                ),
            }
    };
    // 这一版 `units` 按角色的索引一次装建一次（七·6 后续批，用户 2026-10-22 定并进）：要写的单元与记录点名的单元都对这次重写的角色
    // 各取一次，逐个在 `units` 上找是重写的角色数乘单元数。同一角色取最先那一项，与在 `units` 上 `iter().find` 同一项。
    let inside_the_written_units =
        lookup_work::entering(LookupSite::UnitsOfTheRolesWrittenByThisPublish);
    let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);
    let written_units: Vec<&PublishedUnit> = relocated_copies
        .iter()
        .map(|(copy, _, _)| copy)
        .chain(rewritten.iter().map(|identity| {
            units_by_role
                .first_entry_of(identity)
                .expect("重写的角色都装了单元")
        }))
        .collect();
    drop(inside_the_written_units);
    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
        let unit = {
            let _inside_the_named_units =
                lookup_work::entering(LookupSite::UnitsOfTheRolesNamedByTheRecords);
            units_by_role
                .first_entry_of(&identity)
                .expect("点名的角色都是这次重写、装过单元的")
        };
        NamedUnit {
            locations: pool.location_entries(unit.slot, &unit.bytes),
            unit_class: identity.unit_class(),
            birth_tree: identity.tree(&trees),
            birth_txg: txg,
            key_tail: key_tail_of(identity),
        }
    };
    // 这次发布的记录（D23（journal 的角色与格式） 已定项 7 / 已定项 17）：第 k 条的 jsn 计数器是 `plan.counter + k`、
    // 事务号是 `plan.transaction` 加它属于的那个事务的序号（前 N − 1 条一条一个事务；最后一个事务装不下一条记录时再跨几条，
    // 那几条共享它的事务号）。提交标记只在一个事务的最后一条记录上（已定项 7；I-8.8（前缀里的事务不被切开） ③），
    // 「本次发布末条」标志只在这次发布真正的最后一条上（已定项 17）——恢复按它认发布边界（已定项 14 第六条）。
    // 每条都带整次发布的新根段（已定项 15）。反向链：第一条接 `plan.back_chain`（上一条记录的头），之后每条接这次发布里前一条的头（已定项 8）。
    let root = RootRecord {
        filesystem_identifier: *filesystem_identifier,
        unmount_marker,
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: mapping_pointer,
        // 带文件的一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8）：根记录这一项恒全零，
        // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
        allocation_record_tree_root: NodePointer::empty_root(),
        // 带单元的发布建了这一版的账：t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)，p = 记完之后还钉着的槽里最小的钉值
        // （C773（读坏与全零根槽的回收门槛钉只在内存里），用户 2026-10-20 定；池级 checker 的 I-7.18 求等、I-7.19 判钉槽）。
        reclaim_threshold: reclaim_floor_when_the_root_was_recorded
            .reclaim_threshold_of_a_root_carrying(plan.rollback_floor),
        pinned_ring_slot: reclaim_floor_when_the_root_was_recorded.pinned_ring_slot(),
    };
    // 每条记录点名哪几项：整理的一批（不重写数据单元，一个事务）点名搬迁副本在前、这次重写的角色在后，按一条记录装几个点名项切
    // （判决 G3「journal 点名项 = 搬迁副本 + 新写的节点，一个事务号」）；别的发布照角色切法。
    let named_units_of_each_record: Vec<Vec<NamedUnit>> = if relocated_copies.is_empty() {
        roles_named_by_each_record
            .iter()
            .map(|named_roles| named_roles.iter().copied().map(named_unit_of).collect())
            .collect()
    } else {
        assert!(
            transaction_offset_of_each_record
                .iter()
                .all(|transaction_offset| *transaction_offset == 0),
            "整理的一批不重写数据单元：整次一个事务"
        );
        let named_units: Vec<NamedUnit> = relocated_copies
            .iter()
            .map(|(_, named, _)| *named)
            .chain(rewritten.iter().copied().map(named_unit_of))
            .collect();
        named_units
            .chunks(named_entry_capacity.named_entries_per_record())
            .map(<[NamedUnit]>::to_vec)
            .collect()
    };
    let transaction_offset_of_each_record: Vec<u64> = if relocated_copies.is_empty() {
        transaction_offset_of_each_record
    } else {
        vec![0; named_units_of_each_record.len()]
    };
    let records_in_this_publish =
        u64::try_from(named_units_of_each_record.len()).expect("记录条数");
    let mut written_records: Vec<WrittenJournalRecord> =
        Vec::with_capacity(named_units_of_each_record.len());
    // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
    for (record_offset_in_this_publish, named_units) in
        named_units_of_each_record.iter().enumerate()
    {
        let offset = u64::try_from(record_offset_in_this_publish).expect("记录序号");
        let back_chain = match written_records.last() {
            None => plan.back_chain,
            Some(previous_record_of_this_publish) => {
                back_chain_of(&previous_record_of_this_publish.bytes)
            }
        };
        let transaction_offset = transaction_offset_of_each_record[record_offset_in_this_publish];
        let transaction_of_the_next_record = transaction_offset_of_each_record
            .get(record_offset_in_this_publish + 1)
            .copied();
        let is_the_last_record_of_its_transaction =
            transaction_of_the_next_record != Some(transaction_offset);
        let place_in_publish = if transaction_of_the_next_record.is_none() {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        };
        let record = JournalRecord {
            instance,
            counter: plan.counter + offset,
            checkpoint_txg: txg,
            transaction: plan.transaction + transaction_offset,
            is_commit: is_the_last_record_of_its_transaction,
            // 本次发布内序号依次是 1..N（D23（journal 的角色与格式） 已定项 4）：所选根那条记录读不出时，
            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(
                record_offset_in_this_publish,
            ),
            place_in_publish,
            back_chain,
            filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
            // 新根段七项取这次发布写的根上的（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）。
            new_tree_table: root.tree_table,
            new_mapping_root: root.mapping_root,
            new_tree_identifier_watermark: root.tree_identifier_watermark,
            new_rollback_floor: root.rollback_floor,
            new_reclaim_threshold: root.reclaim_threshold,
            new_pinned_ring_slot: root.pinned_ring_slot,
            new_instance_table: root.instance_table,
            new_allocation_record_tree_root: root.allocation_record_tree_root,
            new_unmount_marker: root.unmount_marker,
            named: named_units.clone(),
        };
        let bytes = record.to_bytes();
        written_records.push(WrittenJournalRecord { record, bytes });
    }
    let last_counter_of_this_publish = plan.counter + (records_in_this_publish - 1);
    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
    // 这里只装、不落盘：调用方拿它落盘，中途失败时这次已记的写进失败账（增补 2 第 20b 行）、这次发布冻结等原样重发。
    let writes = PublishWrites {
        units: written_units.into_iter().cloned().collect(),
        records: written_records
            .iter()
            .map(|written_record| JournalRecordWrite {
                counter: written_record.record.counter,
                bytes: written_record.bytes.clone(),
            })
            .collect(),
        checkpoint_txg: txg,
        root_slot: root.to_slot(pool.root_slot_bytes()),
        // 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
        journal_tail: last_counter_of_this_publish,
        journal_instance: instance,
    };

    let WrittenJournalRecord {
        record,
        bytes: record_bytes,
    } = written_records
        .pop()
        .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
    // 装好这一版的 units 之后记下两棵容器树根单元的下标：下一次准入读 H 按它直接取（运行时决策路径，不在 units 上找）。
    let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);
    let output = TransactionOutput {
        root,
        record,
        record_bytes,
        earlier_records_of_this_publish: written_records,
        units,
        rewritten: rewritten.to_vec(),
        data_pointers,
        files_other_than_the_first: extent_built
            .lower_nodes_of_the_files_other_than_the_first
            .into_iter()
            .zip(data_pointers_of_the_files_other_than_the_first)
            .zip(&resolved.files_other_than_the_first.files)
            .map(
                |(((inode, extent_lower_nodes), (_, file_data_pointers)), file_plan)| {
                    FileOfAnInodeOtherThanTheFirst {
                        inode,
                        data_pointers: file_data_pointers,
                        extent_lower_nodes,
                        tail: file_plan.tail,
                    }
                },
            )
            .collect(),
        mapping_keys,
        allocation_records,
        allocation_record_tree: allocation_built.version,
        extent_tree: extent_built.version,
        accounting_entries,
        accounting_tree: accounting_built.version,
        central_mapping_tree: mapping_built.version,
        deadlist_entries: resolved.deadlist.entries_after_this_publish.clone(),
        deadlist_tree: deadlist_built.version,
        mapping_entries_kept_for_the_snapshots: kept_mapping_entries_after_this_publish,
        snapshots_being_destroyed,
        tree_table_entries,
        tree_identifiers: trees,
        inode_record,
        deleted_inodes: resolved.deleted_inodes.clone(),
        logged_operations: resolved.logged_operations.clone(),
        tombstone_containers: deletion_tree_units
            .tombstone
            .containers
            .into_iter()
            .map(|container| TombstoneContainerVersion {
                contents: container.contents,
                pointer: container.node.pointer,
            })
            .collect(),
        tombstone_tree_index_nodes: ContainerTreeIndexNodesVersion {
            shape: resolved.tombstone_tree.shape.clone(),
            pointers_below_the_root: deletion_tree_units
                .tombstone
                .nodes_below_the_root
                .iter()
                .map(|node| (node.position, node.node.pointer))
                .collect(),
        },
        lineage_containers: deletion_tree_units
            .lineage
            .containers
            .into_iter()
            .map(|container| LineageContainerVersion {
                contents: container.contents,
                pointer: container.node.pointer,
            })
            .collect(),
        lineage_tree_index_nodes: ContainerTreeIndexNodesVersion {
            shape: resolved.lineage_tree.shape.clone(),
            pointers_below_the_root: deletion_tree_units
                .lineage
                .nodes_below_the_root
                .iter()
                .map(|node| (node.position, node.node.pointer))
                .collect(),
        },
        retirement_containers: deletion_tree_units
            .retirement
            .containers
            .into_iter()
            .map(|container| RetirementContainerVersion {
                contents: container.contents,
                pointer: container.node.pointer,
            })
            .collect(),
        retirement_tree_index_nodes: ContainerTreeIndexNodesVersion {
            shape: resolved.retirement_tree.shape.clone(),
            pointers_below_the_root: deletion_tree_units
                .retirement
                .nodes_below_the_root
                .iter()
                .map(|node| (node.position, node.node.pointer))
                .collect(),
        },
        inode_leaf_containers: inode_tree_units
            .leaf_containers
            .into_iter()
            .map(|container| InodeLeafContainerVersion {
                contents: container.contents,
                pointer: container.pointer,
            })
            .collect(),
        mapped_units,
        released: release.to_vec(),
        quarantined_after_release_checksum_mismatch: quarantine.to_vec(),
        units_relocated_by_this_publish: relocated_copies
            .into_iter()
            .map(|(_, _, relocated)| relocated)
            .collect(),
        key_order_mismatches: key_order_mismatches_of_this_publish,
        highest_transaction_number_in_this_instance: plan
            .highest_transaction_number_before_this_publish
            .max(last_transaction_of_this_publish),
        // 落盘之后由 `persist_the_publish_or_freeze_it` 换成这次真写出去的账。
        writes: WritesByStructureKind::NOTHING_WRITTEN,
        container_tree_root_unit_positions,
        inode_record_counts: resolved.inode_record_counts,
    };
    Ok((writes, output))
}
