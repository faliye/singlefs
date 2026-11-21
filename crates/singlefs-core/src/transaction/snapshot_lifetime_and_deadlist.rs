//! 快照：下一侧、快照根项、销毁快照释放的 deadlist 项、这次发布的 deadlist 变化与快照寿命的判定。

use super::*;

/// 一个按映射 key 认的单元（deadlist 条目留存的那几个）按它的类标签归到哪一类用户可见角色上去取跨度、读盘核、报错：数据单元 → 第一个数据单元
/// （跨度 2）；码 3 容器 → 第一片 inode 叶容器（跨度 2）；码 2 节点 → extent 树根（跨度 1）。六类用户可见角色里同一个类标签的跨度相同
/// （[`TransactionUnit::span_slots`]），取哪一个都一样；别的类标签交回 `None`。
#[must_use]
pub const fn user_visible_role_standing_for_a_unit_class(
    unit_class: u8,
) -> Option<TransactionUnit> {
    match unit_class {
        UNIT_CLASS_DATA => Some(TransactionUnit::Data(DataUnitIndexInFile::FIRST)),
        UNIT_CLASS_PACKED => Some(TransactionUnit::InodeLeafContainer(
            InodeLeafContainerIndexInTree::LEFTMOST,
        )),
        UNIT_CLASS_INDEX_NODE => Some(TransactionUnit::ExtentRoot),
        _other_unit_class => None,
    }
}

/// 一个快照在派生链上的下一侧（D5（快照 / 空间记账机制） 已定项 12「下一侧怎么找」：从树表按（诞生 txg, 树号）排的码 14 链现取，零字节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextSideOfASnapshot {
    /// 下一条码 14（条目原样）。
    Snapshot(TreeTableEntry),
    /// 它是最后一个快照：下一侧是活头。
    LiveHead,
}

/// `snapshot` 那条码 14 在派生链上的下一侧：树表里的码 14 按（诞生 txg, 树号）升序排，排在它后面的第一条；没有就是活头。
/// 链上不在乎「销毁中」的记号：第一版删快照串行，带记号的只有正在删的这一条（判决第七节岔路 2）。`snapshot` 不在表里时交回 `None`。
#[must_use]
pub fn next_side_of_the_snapshot(
    tree_table_entries: &[TreeTableEntry],
    snapshot: TreeIdentifier,
) -> Option<NextSideOfASnapshot> {
    let mut chain: Vec<&TreeTableEntry> = tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .collect();
    chain.sort_by_key(|entry| (entry.birth_txg, entry.tree));
    let position = chain.iter().position(|entry| entry.tree == snapshot)?;
    Some(match chain.get(position + 1) {
        Some(next) => NextSideOfASnapshot::Snapshot(**next),
        None => NextSideOfASnapshot::LiveHead,
    })
}

/// `snapshot` 那条码 14 条目（树表里按树号找）；不在表里交回 `None`。
#[must_use]
pub fn snapshot_inode_root_entry(
    tree_table_entries: &[TreeTableEntry],
    snapshot: TreeIdentifier,
) -> Option<TreeTableEntry> {
    tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT && entry.tree == snapshot)
        .copied()
}

/// 删快照 `snapshot` 时能放掉的 deadlist 条目，按 key 升序（D5（快照 / 空间记账机制） 已定项 12 定案表「销毁 S」那一行，判决 S4）：
/// 头是活头 inode 树、纪元落在 [lower_epoch_bound, upper_epoch_bound) 里、birth > P——lower_epoch_bound = 它那条码 14 的诞生 txg，P = 它的前驱，upper_epoch_bound 现取：下一侧是快照时取那条码 14 的
/// 诞生 txg；下一侧是活头时条款取「写意图那次发布的 txg」，它不落盘（判决 S4「续做点不存」）——这里取 `upper_epoch_bound_when_the_next_side_is_the_live_head`
/// （调用方给这一批自己的 txg）：条目的纪元只取自上一版活头的前驱，活头前驱只在建快照与写销毁意图两处变，而意图挂着时这两样都被拒
/// （判决 m4-c145-destruction-bound-r1 B1），所以纪元 ≥ 写意图那次 txg 的条目一条都不会有，两种取法放的是同一组。余下的条目键不动，它们按纪元落进下一侧的那一段
/// （约定 B，已定项 16：下一侧的前驱在意图那次已经换成 P）。`snapshot` 不在树表里交回空表。
#[must_use]
pub fn deadlist_entries_the_destruction_of_a_snapshot_releases(
    version: &TransactionOutput,
    snapshot: TreeIdentifier,
    upper_epoch_bound_when_the_next_side_is_the_live_head: CheckpointTxg,
) -> Vec<DeadlistEntry> {
    let Some(snapshot_entry) = snapshot_inode_root_entry(&version.tree_table_entries, snapshot)
    else {
        return Vec::new();
    };
    let lower_epoch_bound = snapshot_entry.birth_txg;
    let predecessor = snapshot_entry.previous_snapshot_txg;
    let upper_epoch_bound = match next_side_of_the_snapshot(&version.tree_table_entries, snapshot) {
        Some(NextSideOfASnapshot::Snapshot(next)) => next.birth_txg,
        Some(NextSideOfASnapshot::LiveHead) | None => {
            upper_epoch_bound_when_the_next_side_is_the_live_head
        }
    };
    let head = TreeIdentifier(snapshot_entry.head_identifier);
    version
        .deadlist_entries
        .iter()
        .filter(|entry| {
            entry.head == head
                && (lower_epoch_bound..upper_epoch_bound).contains(&entry.epoch)
                && birth_txg_of_a_mapping_key(&entry.mapping_key)
                    .is_some_and(|birth| birth > predecessor)
        })
        .cloned()
        .collect()
}

/// 这次发布对 deadlist 与留存映射条目做的事（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4）：
/// 这次换下的用户可见单元里哪几个被活快照引用（进 deadlist、不释放、映射条目留存），删快照这一批放掉哪几条，这次之后的条目与留存的映射条目。
#[derive(Clone, Debug, Default)]
pub struct DeadlistChangesOfThisPublish {
    /// 这次换下、经映射走释放判定的用户可见单元（六类角色，上一版里的角色），按换下的次序：释放的与进 deadlist 的合起来。
    pub user_visible_roles_replaced: Vec<TransactionUnit>,
    /// 其中被活快照引用、进 deadlist 不释放的那几个（D5（快照 / 空间记账机制） 已定项 19：birth ≤ 活头的 `previous_snapshot_txg`，或取不到 birth）。
    pub user_visible_roles_kept_for_the_snapshots: Vec<TransactionUnit>,
    /// 删快照这一批放掉的条目，连同它们留存的映射条目的两条位置项（释放从这里取落点）。
    pub entries_released_by_the_destruction: Vec<(DeadlistEntry, [LocationEntry; 2])>,
    /// 这次之后的 deadlist 条目，按 key 的字段序升序。
    pub entries_after_this_publish: Vec<DeadlistEntry>,
    /// 这次之后留存的映射条目，按映射 key 的字段序升序。
    pub kept_mapping_entries_after_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])>,
}

/// 这次发布对 deadlist 与留存映射条目做的事（[`DeadlistChangesOfThisPublish`]）。`candidate_roles` 是这次换下的用户可见角色的来源
/// （这次重写的数据单元、inode 叶容器与根，没有同名后继的那几个，extent 树按计划换下的节点）：按释放判定路径同一个口径展开
/// （文件变短时尾部的数据单元，`roles_replaced_via_mapping`）、只留六类角色与上一版里有落点的，逐个按映射 key 的出生 txg 与活头的
/// `previous_snapshot_txg` 判（[`snapshot_lifetime_verdict`]；写入口上装着只供测试的强制臂时照 `snapshot_lifetime_judgement`，[`SnapshotLifetimeJudgement`]）。进 deadlist 的条目：头 = 活头 inode 树、纪元 = 活头此刻的 `previous_snapshot_txg`
/// （D5 已定项 12「纪元 = 块被杀那一刻头的 previous_snapshot_txg」）、映射 key 照上一版；它的映射条目照上一版那两条位置项留存。
/// 删快照的批放掉的条目按 [`deadlist_entries_the_destruction_of_a_snapshot_releases`] 取（最后一批取全部、别的批取前 `entries_released` 条），
/// 落点取它留存的那条映射条目。没有上一版（第一个文件版本）时什么都不做。
///
/// # Errors
/// 进 deadlist 的单元在上一版的映射里查不到、条目窄于字段表 ⇒ `ReleaseNotInMapping` / `MappingEntryNarrowerThanItsFieldTable`（同释放判定路径）；
/// 删快照这一批要放的条目在上一版留存的映射条目里没有 ⇒ `DeadlistEntryNotReleasable { refusal: KeptMappingEntryMissing }`
/// （C300（deadlist 条目按 key 查映射查不到没有条款）：第一版不处置，在任何落盘动作之前拒）。
///
/// # Panics
/// 有单元要进 deadlist 而活头的 `previous_snapshot_txg` 是 0（活头没打过快照）：照判定走不到（判定只留下映射 key 出生 txg 不大于它的单元，
/// 用户可见单元都由 txg 至少 1 的发布写下），
/// 只有只供测试的强制进 deadlist 那一臂用错了才走到（[`SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：上一版、树号、换下的候选、快照这一步、txg、判定那一臂、这次文件内容是不是整份重写"
)]
pub(super) fn deadlist_changes_of_this_publish(
    previous: Option<&TransactionOutput>,
    trees: &FileVersionTreeIdentifiers,
    candidate_roles: &[TransactionUnit],
    snapshot: SnapshotStepOfThePublish,
    txg: CheckpointTxg,
    snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
    file_content: FileContentOfThePublish,
) -> Result<DeadlistChangesOfThisPublish, PublishError> {
    let Some(previous_version) = previous else {
        return Ok(DeadlistChangesOfThisPublish::default());
    };
    let _inside_the_snapshot_judgement = lookup_work::entering(LookupSite::SnapshotJudgement);
    // 换下的用户可见角色问在上一版里有没有落点：按 inode 号找文件的那份读者每走一遍建至多一次，这一遍进一次这一处。
    let inside_the_file_lookups =
        lookup_work::entering(LookupSite::FilesOfTheUserVisibleRolesOfTheSnapshotJudgement);
    let mut previous_files =
        FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version(previous_version);
    let user_visible_roles_replaced: Vec<TransactionUnit> =
        roles_replaced_via_mapping(previous_version, candidate_roles, file_content)
            .into_iter()
            .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity))
            .filter(|identity| {
                has_a_placement_in_the_previous_version(
                    previous_version,
                    &mut previous_files,
                    *identity,
                )
            })
            .collect();
    drop(inside_the_file_lookups);
    let previous_snapshot_txg_of_the_live_head =
        previous_version.previous_snapshot_txg_of_the_live_head();
    let mut user_visible_roles_kept_for_the_snapshots = Vec::new();
    let mut entries_killed_this_publish: Vec<DeadlistEntry> = Vec::new();
    let mut mapping_entries_kept_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = Vec::new();
    // 进 deadlist 的角色取映射条目：同释放判定路径，key 与落点都走建一次的索引（七·6，用户 2026-10-21 定并进），没有进 deadlist 的不建。
    let mut previous_version_read_by_role =
        MappedUnitsOfAVersionReadByRole::of_the_version(previous_version);
    // 换下的角色各查一次映射 key，查法与照抄全部角色那一圈同一个索引（七·6）：建一次走完上一版的 `mapped_units`，
    // 之后每次查不再走整张表；同一角色取最先那一项、查不到时的 panic 文案与在 `mapped_units` 上 `iter().find` 相同。
    let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();
    for identity in &user_visible_roles_replaced {
        let mapping_key = mapped_units_of_the_previous_version
            .mapping_key_of(*identity)
            .to_vec();
        match snapshot_lifetime_judgement
            .verdict_of(&mapping_key, previous_snapshot_txg_of_the_live_head)
        {
            SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue => {}
            SnapshotLifetimeVerdict::KeptOnTheDeadlist => {
                // 纪元 0 的条目哪个快照的销毁都放不掉（快照放的是纪元落在 [它的诞生 txg, 下一侧) 里的条目，诞生 txg 至少 1），回退又照抄它
                // （判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J4）。照判定走时进 deadlist 的单元生得不晚于活头上一个
                // 快照，纪元就是那个快照的 txg、非 0；只供测试的强制进 deadlist 那一臂没打过快照就用会走到这里——它在任何写之前停下。
                assert_ne!(
                    previous_snapshot_txg_of_the_live_head,
                    CheckpointTxg(0),
                    "进 deadlist 的条目纪元非 0：活头没打过快照时没有单元该进 deadlist（只供测试的强制进 deadlist 那一臂要在活头打过快照之后用）；单元 {identity:?}"
                );
                let locations = match previous_version_read_by_role.locations_of_role(*identity) {
                    MappingLookup::Found(locations) => locations,
                    MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
                        return Err(PublishError::ReleaseNotInMapping { unit: *identity })
                    }
                    MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
                        return Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
                            unit: *identity,
                            entry_bytes,
                            field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
                        })
                    }
                };
                user_visible_roles_kept_for_the_snapshots.push(*identity);
                entries_killed_this_publish.push(DeadlistEntry {
                    head: trees.inode,
                    epoch: previous_snapshot_txg_of_the_live_head,
                    mapping_key: mapping_key.clone(),
                });
                mapping_entries_kept_this_publish.push((mapping_key, locations));
            }
        }
    }
    let entries_released: Vec<DeadlistEntry> = match snapshot {
        SnapshotStepOfThePublish::DestructionBatch {
            snapshot,
            entries_released,
        } => {
            deadlist_entries_the_destruction_of_a_snapshot_releases(previous_version, snapshot, txg)
                .into_iter()
                .take(entries_released)
                .collect()
        }
        SnapshotStepOfThePublish::DestructionFinalBatch { snapshot } => {
            deadlist_entries_the_destruction_of_a_snapshot_releases(previous_version, snapshot, txg)
        }
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::CreateSnapshot { .. }
        | SnapshotStepOfThePublish::DestructionIntent { .. } => Vec::new(),
    };
    let mut entries_released_by_the_destruction = Vec::with_capacity(entries_released.len());
    // 放掉的每条条目按映射 key 取它留存的映射条目：走建一次的索引（七·6 代码轮第三轮判决第三节第 1 条），同一把 key 取最先那一项，
    // 与在 `mapping_entries_kept_for_the_snapshots` 上 `iter().find` 同一项；一条都不放的不建。放条目的每一遍进一次这一处
    // （七·6 后续批代码轮第一轮判决第三节第 3 条：建索引次数恰好等于走了几遍）。
    let inside_the_destruction = (!entries_released.is_empty()).then(|| {
        lookup_work::entering(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction)
    });
    let mut kept_mapping_entries_by_key = None;
    for entry in entries_released {
        let locations = kept_mapping_entries_by_key
            .get_or_insert_with(|| previous_version.mapping_entries_kept_for_the_snapshots_by_key())
            .first_entry_of(&entry.mapping_key.as_slice())
            .map(|(_, locations)| *locations)
            .ok_or_else(|| PublishError::DeadlistEntryNotReleasable {
                mapping_key: entry.mapping_key.clone(),
                refusal: DeadlistEntryReleaseRefusal::KeptMappingEntryMissing,
            })?;
        entries_released_by_the_destruction.push((entry, locations));
    }
    drop(inside_the_destruction);
    let mut entries_after: BTreeMap<CodeTwoTreeKey, DeadlistEntry> = previous_version
        .deadlist_entries
        .iter()
        .map(|entry| {
            (
                CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::DEADLIST),
                entry.clone(),
            )
        })
        .collect();
    let mut kept_mapping_after: BTreeMap<CodeTwoTreeKey, (Vec<u8>, [LocationEntry; 2])> =
        previous_version
            .mapping_entries_kept_for_the_snapshots
            .iter()
            .map(|(key, locations)| {
                (
                    CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
                    (key.clone(), *locations),
                )
            })
            .collect();
    for (entry, _) in &entries_released_by_the_destruction {
        let released_entry = entries_after.remove(&CodeTwoTreeKey::new(
            &entry.key_bytes(),
            CodeTwoKeyFieldWidths::DEADLIST,
        ));
        // C145 第三条（用户 2026-10-26 定「加断言并留档」）：放块时要删的那条 deadlist 条目必须还在。放掉的条目是
        // `deadlist_entries_the_destruction_of_a_snapshot_releases` 从上一版的 deadlist 里挑的、`entries_after` 也从同一份建，构造上恒成立；
        // 删掉这条断言的变异是等价变异，等价性由 harness 用例
        // `every_deadlist_entry_a_snapshot_destruction_releases_is_in_the_deadlist_the_release_starts_from` 钉住。
        assert!(
            released_entry.is_some(),
            "删快照放掉的 deadlist 条目在上一版的 deadlist 里（放掉的是从它挑的，C145）"
        );
        kept_mapping_after.remove(&CodeTwoTreeKey::new(
            &entry.mapping_key,
            CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
        ));
    }
    for entry in entries_killed_this_publish {
        let previous_entry = entries_after.insert(
            CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::DEADLIST),
            entry,
        );
        assert!(
            previous_entry.is_none(),
            "一个单元只被杀一次：这次进 deadlist 的条目在上一版的 deadlist 里没有（它的映射 key 上一版还被这一版的树引用）"
        );
    }
    for (key, locations) in mapping_entries_kept_this_publish {
        kept_mapping_after.insert(
            CodeTwoTreeKey::new(&key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
            (key, locations),
        );
    }
    Ok(DeadlistChangesOfThisPublish {
        user_visible_roles_replaced,
        user_visible_roles_kept_for_the_snapshots,
        entries_released_by_the_destruction,
        entries_after_this_publish: entries_after.into_values().collect(),
        kept_mapping_entries_after_this_publish: kept_mapping_after.into_values().collect(),
    })
}

/// 六类用户可见子树的角色（判决 S3：只有它们问 D5（快照 / 空间记账机制） 已定项 19 的判定，其余 14 类一律直接进 defer）。
#[must_use]
pub const fn asks_the_snapshot_lifetime_judgement(identity: TransactionUnit) -> bool {
    match identity {
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot => true,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
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
        | TransactionUnit::RetirementNodeBelowTheRoot(_)
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
    }
}

/// D5（快照 / 空间记账机制） 已定项 19 的判定（第一版不带「n > 1」那一臂：稀疏旁表恒 0 条，判决 S9）：birth > p ⇒ 立即释放（进 defer）；
/// 否则（含 birth 取不到，已定项 14「判据的输入取不到时默认进 deadlist」）⇒ 进 deadlist。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotLifetimeVerdict {
    ReleasedIntoTheDeferQueue,
    KeptOnTheDeadlist,
}

/// 被杀的用户可见单元走释放判定（[`snapshot_lifetime_verdict`]）的哪一臂（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入；
/// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 末句、`research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F4）。
/// 封闭集合。产品路径照判定（写入口新开时的值），另两种只供测试、装在写入口上（[`PoolWriter::set_snapshot_lifetime_judgement`]）：
/// 强制的那一臂对这次发布换下的每一个问判定的单元都成立，判定本身不算。只管问判定的六类角色（[`asks_the_snapshot_lifetime_judgement`]），
/// 别的 14 类照旧直接进 defer；删快照的批放哪几条不归它管。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotLifetimeJudgement {
    /// 照 D5（快照 / 空间记账机制） 已定项 19 的判定（[`snapshot_lifetime_verdict`]）。
    FollowTheJudgement,
    /// 只供测试：一律立即释放（进 defer），被活快照引用的块也放。
    ForcedToReleaseIntoTheDeferQueueForTests,
    /// 只供测试：一律进 deadlist、不释放、映射条目留存，比活头上一个快照新的块也进。活头要打过快照：条目的纪元取活头此刻的
    /// `previous_snapshot_txg`，没打过快照时是 0，纪元 0 的条目哪个快照的销毁都放不掉，发布在任何写之前断言停下
    /// （判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J4）。
    ForcedOntoTheDeadlistForTests,
}

impl SnapshotLifetimeJudgement {
    /// 这一臂对一个被杀的单元给出的结论：照判定，或强制的那一种。
    #[must_use]
    pub fn verdict_of(
        self,
        mapping_key: &[u8],
        previous_snapshot_txg_of_the_live_head: CheckpointTxg,
    ) -> SnapshotLifetimeVerdict {
        match self {
            SnapshotLifetimeJudgement::FollowTheJudgement => {
                snapshot_lifetime_verdict(mapping_key, previous_snapshot_txg_of_the_live_head)
            }
            SnapshotLifetimeJudgement::ForcedToReleaseIntoTheDeferQueueForTests => {
                SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue
            }
            SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests => {
                SnapshotLifetimeVerdict::KeptOnTheDeadlist
            }
        }
    }
}

/// 一个被杀的单元按它映射 key 里的出生 txg 与活头的 `previous_snapshot_txg` 判（[`SnapshotLifetimeVerdict`]）。
#[must_use]
pub fn snapshot_lifetime_verdict(
    mapping_key: &[u8],
    previous_snapshot_txg_of_the_live_head: CheckpointTxg,
) -> SnapshotLifetimeVerdict {
    match birth_txg_of_a_mapping_key(mapping_key) {
        Some(birth) if birth > previous_snapshot_txg_of_the_live_head => {
            SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue
        }
        Some(_) | None => SnapshotLifetimeVerdict::KeptOnTheDeadlist,
    }
}

#[cfg(test)]
mod tests;
