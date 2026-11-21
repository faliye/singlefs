//! 这次发布释放的落点与分配记录树的收敛，连同每条 journal 记录点名的角色与事务偏移。

use super::*;

/// 这次发布换下的上一版角色与要释放的落点（释放判定路径，只查不改，D19（块指针的结构与宽度预算） 已定项 5 第 1 条）：
/// 整条实例表旧链排最前，再是经映射查到的那几个（第一个文件版本没有上一版的内存态：换下的是 mkfs 那片树表与树表 0 条那一版的分配记录树）。
///
/// # Errors
/// 释放判定路径的几种错（`ReleaseNotInMapping` 一族、`ReleaseTarget*`、`ReleaseSpanMismatch`）。
fn placements_released_by_this_publish(
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    resolved: &ResolvedPublish,
    allocator: &PoolAllocator,
) -> Result<(Vec<TransactionUnit>, Vec<Placement>), PublishError> {
    // 换下的是上一版的角色：多层码 2 树与按位置寻址的两棵树的节点按计划里「上一版被换下的节点」点名，别的角色同一个角色上一版那一份。
    let previous_roles_replaced = match previous {
        Some(previous_version) => {
            resolved.previous_roles_replaced_by_this_publish(previous_version)
        }
        None => Vec::new(),
    };
    // 释放判定分流（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：切口在这一张清单出来之后、`publish_admitted`
    // 落释放之前。`resolve` 按换下的用户可见单元定了哪几个进 deadlist；这里核它定的那一组就是按全部换下角色走出来的用户可见那一组
    // （两边换下口径不同步就会漏判、多判），再把进 deadlist 的那几个从释放里拿掉，删快照这一批放的条目接在最后。
    let kept_for_the_snapshots = &resolved.deadlist.user_visible_roles_kept_for_the_snapshots;
    if let Some(previous_version) = previous {
        let user_visible_roles_released_or_kept: BTreeSet<TransactionUnit> =
            mapped_roles_released_by_this_publish(
                previous_version,
                &previous_roles_replaced,
                &[],
                FileContentOfThePublish::of_the_plan(plan),
            )
            .into_iter()
            .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity))
            .collect();
        assert_eq!(
            user_visible_roles_released_or_kept,
            resolved
                .deadlist
                .user_visible_roles_replaced
                .iter()
                .copied()
                .collect::<BTreeSet<TransactionUnit>>(),
            "resolve 判生死的那一组用户可见单元，就是释放判定路径按这次换下的全部角色走出来的用户可见那一组"
        );
    }
    let release = match previous {
        Some(previous_version) => {
            placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
                previous_version,
                allocator,
                FileContentOfThePublish::of_the_plan(plan),
                &previous_roles_replaced,
                kept_for_the_snapshots,
            )?
        }
        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
        None => format_time_tree_table_to_release(allocator, &resolved.rewritten_roles)?,
    };
    // 整条实例表链重写（写行、回退那一次）：被换下的那条旧链逐片释放，指针取计划带着的那条链（第 1 片起的不在上一版的内存态里）。
    // 旧链排在别的角色前面，与实例表在 bump 次序里排最前同一个次序。
    let release = match &plan.instance_table {
        InstanceTablePlan::Rewrite(rewrite) => {
            if let Some(previous_version) = previous {
                assert_eq!(
                    rewrite.replaced_chain.first(),
                    Some(&previous_version.root.instance_table),
                    "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
                );
            }
            let mut chain_then_the_rest =
                instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;
            chain_then_the_rest.extend(release);
            chain_then_the_rest
        }
        InstanceTablePlan::Carry(_) => release,
    };
    // 删快照这一批放的条目：落点取它留存的映射条目（释放一律经映射，D19（块指针的结构与宽度预算） 已定项 5 第 1 条），照映射那一路逐盘核。
    let mut release = release;
    for (entry, locations) in &resolved.deadlist.entries_released_by_the_destruction {
        release.push(placement_of_a_deadlist_entry_to_release(
            entry, locations, allocator,
        )?);
    }
    Ok((previous_roles_replaced, release))
}

/// 删快照的一批放的一条 deadlist 条目的落点：按它映射 key 的类标签取跨度（[`user_visible_role_standing_for_a_unit_class`]），
/// 照映射那一路逐盘核（[`placement_to_release_after_checking_every_device`]）。
///
/// # Errors
/// 类标签不认识、落点核不过 ⇒ `DeadlistEntryNotReleasable`。
fn placement_of_a_deadlist_entry_to_release(
    entry: &DeadlistEntry,
    locations: &[LocationEntry; 2],
    allocator: &PoolAllocator,
) -> Result<Placement, PublishError> {
    let role = role_standing_for_the_unit_of_a_deadlist_entry(entry)?;
    placement_to_release_after_checking_every_device(role, locations, allocator).map_err(|cause| {
        PublishError::DeadlistEntryNotReleasable {
            mapping_key: entry.mapping_key.clone(),
            refusal: DeadlistEntryReleaseRefusal::PlacementNotReleasable(Box::new(cause)),
        }
    })
}

/// 一条 deadlist 条目指的单元按类标签归到的那一类用户可见角色（[`user_visible_role_standing_for_a_unit_class`]）。
///
/// # Errors
/// 映射 key 的类标签不认识 ⇒ `DeadlistEntryNotReleasable { refusal: UnitClassOutsideTheUserVisibleOnes }`。
pub(super) fn role_standing_for_the_unit_of_a_deadlist_entry(
    entry: &DeadlistEntry,
) -> Result<TransactionUnit, PublishError> {
    let unit_class = entry.mapping_key.first().copied().unwrap_or(0);
    user_visible_role_standing_for_a_unit_class(unit_class).ok_or_else(|| {
        PublishError::DeadlistEntryNotReleasable {
            mapping_key: entry.mapping_key.clone(),
            refusal: DeadlistEntryReleaseRefusal::UnitClassOutsideTheUserVisibleOnes { unit_class },
        }
    })
}

/// 分配记录树这次重写哪几个节点、这一版有哪几个节点（D8（核心索引结构） 已定项 14：按位置寻址，只重写内容变了的叶与它们的祖先，
/// 没有记录的一段不写节点）。它们要在取落点之前定下来（角色清单、出生序号、映射 key、中央映射树的形状都要它），而取落点又会改记录——
/// 这次写的每个单元、连分配记录树自己的节点，每盘各记一条；换下的每一份把它那条改成已释放（D3（空间分配） 已定项 7；这就是释放链）。
/// 所以走一个固定点：从「重写集空、节点照上一版」起，按它算这次发布的全部角色，在分配器的**拷贝**上把释放与取落点照发布路径的次序
/// 走一遍，按走出来的记录算有记录的节点与内容变了的节点（[`nodes_whose_contents_changed`]）；节点对得上、变了的都在重写集里（或本来就是新节点）
/// 就定下，否则节点换成走出来的那一批、变了的并进重写集、顺着释放链走到闭包（[`merge_into_the_rewrite_set_walking_the_release_chain`]）再走一遍。
/// 多罩的节点内容没变、照样重写一份（合法的 COW）。
/// 真发时走的是同一串释放与取落点，得出同一份记录（释放之前读盘核出对不上的那几份记录留在已分配，只会让变了的节点更少）。
/// 没有上一版（第一个文件版本）时这棵树的树号换成这次发出来的（树表 0 条那一版的分配记录树是树 0），节点全部重写、不照抄。
///
/// **走不走得完，推出来的只有一半**：重写集只增不减（只经 [`merge_into_the_rewrite_set_walking_the_release_chain`] 并节点），以池里的节点数为上界；
/// 一轮没定下来，要么并进了新节点（重写集严格变大），要么重写集没变、只是这一版的节点换成走出来的那一批。后一种轮的个数推不出上界：
/// 提交内生块回落时先取这一轮重写集里的叶（[`rewrite_set_leaves_of_the_roles`]），取光了再取空槽最多的叶（D3（空间分配） 已定项 8 第 2 条），
/// 计划里的节点一变、先取的叶跟着变，这一轮的落点不是上一轮那一串的延长——装着记录的节点可以变少，这一版的节点不单调，同一个重写集下可以来回换。
/// 推的、没证：回落的落点只进两种叶，重写集里的叶与空槽最多的那片叶；后者装进这次的落点就变了内容、下一轮并进重写集、之后先取它，
/// 所以每进一片新叶都是重写集里的叶取光之后的那一次、重写集跟着变大。这一句只说回落那一支；开段与 bump 的落点按段走，与重写集无关。
/// 节点数可以比 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 大，节点数这个上界也不保证轮数够：释放链沿活跃叶连成一长串时，一轮只走一跳就要走满轮数，
/// 所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶与这一版的节点来回换那两样。E206（发布固定点轮数的代价）第一段 16 个取样点量到至多 5 轮，这几个点都有全空段、走不到回落。
/// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性也推不出来。
/// 走满 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮走不完，交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
///
/// # Errors
/// 同 [`PublishPlan::resolve`]、[`placements_released_by_this_publish`]，与拷贝上取不到落点（`PlacementRefused`）——
/// 真发时同一个落点同样取不到；轮数用完 ⇒ `AllocationRecordTreeRewriteSetDidNotSettle`。都在任何落盘动作之前交回。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配器、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、释放判定那一臂"
)]
pub(super) fn settle_the_allocation_record_tree(
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: &FileVersionTreeIdentifiers,
    device_identities: &[DeviceIdentity],
    capacities: CodeTwoTreeNodeCapacities,
    allocator: &PoolAllocator,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
    snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
) -> Result<SettledPublish, PublishError> {
    let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    // 条目换了位置、key 不变的：整理这一批搬走的，与管理员回退把条目换回目标那一版位置的（判决 m4-segment-code-r1 第三节 F1）。
    let mapping_keys_moved_by_this_publish: Vec<Vec<u8>> = relocations
        .iter()
        .map(|relocation| relocation.mapping_key.clone())
        .chain(
            mapping_keys_whose_entries_change_without_a_relocation
                .iter()
                .cloned(),
        )
        .collect();
    let (previous_nodes, previous_records): (
        BTreeSet<AllocationRecordTreeNode>,
        &[AllocationRecord],
    ) = match previous {
        Some(previous_version) => (
            previous_version.allocation_record_tree.node_set(),
            &previous_version.allocation_records,
        ),
        None => (BTreeSet::new(), &[]),
    };
    let previous_tree = previous.map(|previous_version| &previous_version.allocation_record_tree);
    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
    // 迭代上界是 ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT 轮；跨轮携带的是重写集（只增不减，每轮结束时对释放链是闭的）与这一版的节点（换成上一轮走出来的那一批）。
    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
            rewritten.intersection(&nodes_after).copied().collect();
        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
            &previous_nodes,
            &nodes_after,
            &rewritten_among_the_nodes_after,
        );
        let rewritten_by_the_plan: BTreeSet<AllocationRecordTreeNode> =
            allocation_plan.rewritten_nodes().into_iter().collect();
        let resolved = plan.resolve(
            previous,
            trees,
            device_identities,
            capacities,
            allocation_plan,
            deletion_and_lineage,
            &mapping_keys_moved_by_this_publish,
            snapshot_lifetime_judgement,
        )?;
        let (previous_roles_replaced, mut release) =
            placements_released_by_this_publish(plan, previous, &resolved, allocator)?;
        // 这次重写本来就换下的角色不另搬：它的旧落点经 COW 释放、新落点不在活 R 里（判决 G3 的一批只搬「照抄」的单元）。
        let relocations_in_effect: Vec<RelocationOfThePublish> = relocations
            .iter()
            .filter(|relocation| match relocation.holder {
                RelocatedUnitHolder::RoleOfThisVersion(role) => {
                    !previous_roles_replaced.contains(&role)
                }
                RelocatedUnitHolder::KeptForTheSnapshots => true,
            })
            .cloned()
            .collect();
        for relocation in &relocations_in_effect {
            let source = placement_to_release_after_checking_every_device(
                relocation.role_for_the_checks,
                &relocation.source_locations,
                allocator,
            )?;
            assert_eq!(
                source, relocation.source,
                "搬迁的源落点就是映射位置：publish_relocation_batch 按同一个上一版的映射核过"
            );
            release.push(source);
        }
        let mut rehearsal = allocator.clone();
        for placement in &release {
            rehearsal.release(*placement, plan.txg);
        }
        // 释放本身弄脏的叶（连祖先与根）这一轮的计划还没罩住：先并进重写集再取落点。这样取落点被拒时交回的角色
        // 是罩住它们的那一版 bump 次序里第一个取不到的（分配记录树的节点排在前面），不是只罩了一部分的那一版里的。
        // 这一版的节点也先并进释放之后装着记录的那几个（上一版没有树时释放之前就有的记录也在内：mkfs 的两个单元）；
        // 释放只改写记录、不删，这几个节点取完落点之后仍装着记录，下面「装着记录的节点等于这一版的节点」那一判不会因此来回。
        let changed_by_the_release =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
            merge_into_the_rewrite_set_walking_the_release_chain(
                &mut rewritten,
                changed_by_the_release,
                previous_tree,
                &geometry,
            );
            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
            continue;
        }
        let rewrite_set_leaves =
            rewrite_set_leaves_of_the_roles(&resolved.rewritten_roles, allocator);
        for relocation in &relocations_in_effect {
            allocate_relocation_destination(&mut rehearsal, relocation, &rewrite_set_leaves)?;
        }
        for identity in &resolved.rewritten_roles {
            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg, &rewrite_set_leaves)?;
        }
        let changed =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        let holding_records = nodes_holding_records(&geometry, rehearsal.records());
        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
            return Ok(SettledPublish {
                resolved,
                previous_roles_replaced,
                release,
                relocations_in_effect,
                rounds_walked: round_index + 1,
            });
        }
        merge_into_the_rewrite_set_walking_the_release_chain(
            &mut rewritten,
            changed,
            previous_tree,
            &geometry,
        );
        nodes_after = holding_records;
    }
    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
    })
}

/// 把一轮弄脏的节点并进分配记录树的重写集，再顺着释放链走到闭包：重写集里的节点这一版要重写，重写就释放它在上一版的两份落点，
/// 释放改写落点那条记录，记录所在的叶（由槽号定）与叶的祖先内容就变了、也要重写；新并进来的节点接着走同一步，直到不再多。
/// 走到的节点固定点下一轮的释放一定弄脏，它们本来就在不动点里：提前并进来只省轮数，写的角色与字节不变。
/// 重写集从空集起、只经这里并节点，进来时对这一步是闭的，所以只从新并进来的节点走。上一版没有的节点没有旧落点，走到它为止；没有上一版的不走。
pub(super) fn merge_into_the_rewrite_set_walking_the_release_chain(
    rewritten: &mut BTreeSet<AllocationRecordTreeNode>,
    dirty_nodes: BTreeSet<AllocationRecordTreeNode>,
    previous_tree: Option<&AllocationRecordTreeVersion>,
    geometry: &AllocationRecordTreeGeometry,
) {
    let mut nodes_whose_old_locations_are_not_walked: Vec<AllocationRecordTreeNode> = dirty_nodes
        .into_iter()
        .filter(|node| rewritten.insert(*node))
        .collect();
    let Some(previous_tree) = previous_tree else {
        return;
    };
    // 迭代上界是树的节点数：一个节点只在头一次并进重写集时进待走的那一叠；祖先那一层循环的上界是树高。没有提前出口。
    while let Some(node) = nodes_whose_old_locations_are_not_walked.pop() {
        let Some(pointer) = previous_tree.pointer_of(node) else {
            continue;
        };
        for location in pointer.locations {
            let mut leaf_or_ancestor = Some(geometry.leaf_of_slot(location.device, location.slot));
            while let Some(node_on_the_way_to_the_root) = leaf_or_ancestor {
                if rewritten.insert(node_on_the_way_to_the_root) {
                    nodes_whose_old_locations_are_not_walked.push(node_on_the_way_to_the_root);
                }
                leaf_or_ancestor = geometry.parent_of(node_on_the_way_to_the_root);
            }
        }
    }
}

/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。释放链每次并进重写集都走到闭包，产品路径上轮数只剩取落点碰到新叶
/// 与回落先取重写集的叶让这一版的节点来回换那两样，E206（发布固定点轮数的代价）全部取样点量到至多 5 轮；这两样与只供测试的复用窗口置 0 开关下的单调性
/// 都推不出上界，这个数给它们一个边。
pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;

/// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
/// 2026-09-23 定）：这次写了 N 个数据单元（N ≥ 2）⇒ N 个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），
/// 第 k 个事务（k < N − 1）一条记录、只点名文件第 k 个数据单元；这次发布共享的提交内生块（数据单元之外的每个重写角色）
/// **只在最后一个事务的记录里点名**，与第 N − 1 个数据单元一起，前面几条不重复点名它们。写了一个数据单元或一个都没写 ⇒
/// 一个事务点名全部重写角色（新池新建文件今天的形态）。
///
/// 最后一个事务要点名的项多于一条记录装得下的 64 项（[`JOURNAL_NAMED_ENTRIES_PER_RECORD`]）时**末条再跨记录**
/// （已定项 17，用户 2026-09-24 定）：按 bump 次序装满一条再开下一条，这些记录都属于最后一个事务
/// （[`transaction_offset_of_each_record_of_the_publish`]）；只有真正的最后一条带「本次发布末条」标志。
///
/// 每一项按 `rewritten` 里的次序（bump 次序）列角色；返回至少一项，每项至多 64 个角色。落盘写出的记录读的就是它。
#[must_use]
pub fn roles_named_by_each_record_of_the_publish(
    rewritten: &[TransactionUnit],
    named_entry_capacity: JournalRecordNamedEntryCapacity,
) -> Vec<Vec<TransactionUnit>> {
    let data_roles: Vec<TransactionUnit> = rewritten
        .iter()
        .copied()
        .filter(|identity| identity.is_a_data_unit())
        .collect();
    let data_roles_named_before_the_last_record = &data_roles[..data_roles.len().saturating_sub(1)];
    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =
        data_roles_named_before_the_last_record
            .iter()
            .map(|data_role| vec![*data_role])
            .collect();
    roles_named_by_each_record.push(
        roles_other_than_the_data_roles_named_before_the_last_record(
            rewritten,
            data_roles_named_before_the_last_record,
        ),
    );
    // 末条再跨记录（已定项 17）：最后一个事务那一项装不下一条记录时切成几条，装满一条再开下一条。
    let roles_named_by_the_last_transaction = roles_named_by_each_record
        .pop()
        .expect("上一句刚推进去最后一个事务那一项");
    if roles_named_by_the_last_transaction.is_empty() {
        // 一个角色都不点名的发布照样写一条记录（它一条就是一次发布）。
        roles_named_by_each_record.push(roles_named_by_the_last_transaction);
    } else {
        let named_unit_capacity = named_entry_capacity.named_entries_per_record();
        roles_named_by_each_record.extend(
            roles_named_by_the_last_transaction
                .chunks(named_unit_capacity)
                .map(<[TransactionUnit]>::to_vec),
        );
    }
    roles_named_by_each_record
}

/// `rewritten` 里不在 `data_roles_named_before_the_last_record`（最后一个事务之前那几条记录各点名的那一个数据角色）里的那几个，次序不变：
/// 与 `rewritten.iter().copied().filter(|identity| !data_roles_named_before_the_last_record.contains(identity))` 逐个相同，但那几个数据角色先建一次
/// 按角色的索引、每个角色查一次（七·6 后续批，用户 2026-10-22 定并进）：整份重写一个 N 单元的文件时两边各约 N 项，逐个 `contains` 是 N 的平方。
/// 建与查记在 [`LookupSite::DataRolesNamedBeforeTheLastRecord`]，进一次。
fn roles_other_than_the_data_roles_named_before_the_last_record(
    rewritten: &[TransactionUnit],
    data_roles_named_before_the_last_record: &[TransactionUnit],
) -> Vec<TransactionUnit> {
    let _inside_the_named_data_roles =
        lookup_work::entering(LookupSite::DataRolesNamedBeforeTheLastRecord);
    let data_roles_named_before_the_last_record_by_role = FirstEntryOfEachIdentity::of_the_table(
        data_roles_named_before_the_last_record,
        |identity| *identity,
    );
    rewritten
        .iter()
        .copied()
        .filter(|identity| {
            data_roles_named_before_the_last_record_by_role
                .first_entry_of(identity)
                .is_none()
        })
        .collect()
}

/// 一次发布至多切成几条 journal 记录（代码审阅第 26 条）：在飞记录数上限（环槽数 ÷ F，D23（journal 的角色与格式） 已定项 18，
/// 语义是「一次恢复重放的前缀最多这么多条」）与本次发布内序号的 32 位（已定项 4）取小。多于在飞上限的那次发布，恢复施加前缀时
/// 取不全它的记录、永远施加不了它；多于环槽数的还会在落盘途中盖掉自己前面的记录与所选根的锚点记录。
#[must_use]
pub fn journal_record_limit_of_one_publish(journal_ring_bytes: u64) -> u64 {
    journal_in_flight_record_limit(journal_ring_bytes).min(u64::from(u32::MAX))
}

/// 这次发布切出来的记录条数 `records_of_the_publish`（[`roles_named_by_each_record_of_the_publish`] 的项数）多于
/// [`journal_record_limit_of_one_publish`] 就拒。各条发布路径在动分配器与发任何一个写之前调它。
///
/// # Errors
/// `JournalRecordsOfThePublishExceedTheLimit`，带着条数与上限。
pub(super) fn refuse_a_publish_of_more_journal_records_than_the_limit(
    records_of_the_publish: usize,
    journal_ring_bytes: u64,
) -> Result<(), PublishError> {
    let records_of_the_publish =
        u64::try_from(records_of_the_publish).expect("一次发布的记录条数装得进 u64");
    let record_limit = journal_record_limit_of_one_publish(journal_ring_bytes);
    if records_of_the_publish > record_limit {
        return Err(PublishError::JournalRecordsOfThePublishExceedTheLimit {
            records_of_the_publish,
            record_limit,
        });
    }
    Ok(())
}

/// 一次发布里从 0 数第 k 条记录属于这次发布从 0 数的第几个事务（与 [`roles_named_by_each_record_of_the_publish`] 同一种切法）：
/// 前 N − 1 条（N = 这次写的数据单元数，N ≥ 2）各是自己那个数据单元的事务，第 k 条就是第 k 个事务；
/// 其余的——最后一个事务那条记录与它装不下再跨出去的几条（D23（journal 的角色与格式） 已定项 17）——都属于最后一个事务，
/// 共享它的事务号（已定项 7「同一事务的全部记录共享它」）。
#[must_use]
pub fn transaction_offset_of_each_record_of_the_publish(
    rewritten: &[TransactionUnit],
    named_entry_capacity: JournalRecordNamedEntryCapacity,
) -> Vec<u64> {
    let transactions_named_one_record_each = rewritten
        .iter()
        .filter(|identity| identity.is_a_data_unit())
        .count()
        .saturating_sub(1);
    (0..roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity).len())
        .map(|record_offset_in_this_publish| {
            u64::try_from(record_offset_in_this_publish.min(transactions_named_one_record_each))
                .expect("一次发布的事务数装得进 u64")
        })
        .collect()
}

/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。按角色查走上一版 `units` 的索引：一次发布照抄约有上一版那么多个单元，
/// 索引在装这一版的 `units` 之前建一次（七·6）。
pub(super) fn carried_unit(
    units_of_the_previous_version: &FirstEntryOfEachIdentity<'_, TransactionUnit, PublishedUnit>,
    identity: TransactionUnit,
) -> PublishedUnit {
    units_of_the_previous_version.unit(identity).clone()
}

#[cfg(test)]
mod tests;
