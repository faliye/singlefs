//! 这次发布换下的单元的释放：经映射找要释放的落点、释放前的校验和核对与隔离、留给快照的单元。

use super::*;

/// 第一个文件版本在树表 0 条的那一版上换下的落点：mkfs 那片树表单元，加那一版写行时写下的那棵分配记录树的每个节点
/// （C512（树表 0 条的一版上被换下的单元记在哪）：它的根住根记录那一项，重开时由 `mount` 整棵读进分配器）——那棵树的树号是 0，
/// 这一版的分配记录树换成第一个文件版本发出来的号，节点一个都不照抄（D18（块里携带什么信息） 已定项 7 的树 ID 进块头）。
/// 都只在「这一角色这次重写、那一片还登记着、还没释放过」时进来。
///
/// 「还登记着、还没释放过」逐盘判（代码审阅第 34 条）：池里没有一块盘的账里它在册且未释放 ⇒ 不释放它（两块盘的账一致时与改之前
/// 只看第一块盘相同）；有一块盘的账里它在册且未释放 ⇒ 照映射那一路逐盘核（[`placement_registered_unreleased_on_every_device`]），
/// 别的盘上缺记录、已释放或跨度不对就报错、一个落点都不交回。两块盘的分配记录树对不对称是盘上读来的、不是不变量：
/// 此前只看第一块盘，第一块在册而另一块已释放时在 `PoolAllocator::release` 的断言上 panic，反过来时另一块盘上那条记录从此占着。
///
/// # Errors
/// `ReleaseTargetNotAllocated`、`ReleaseTargetAlreadyReleased`、`ReleaseSpanMismatch`，带着是哪个角色、哪块盘。
pub(super) fn format_time_tree_table_to_release(
    allocator: &PoolAllocator,
    rewritten: &[TransactionUnit],
) -> Result<Vec<Placement>, PublishError> {
    let placement_still_to_release = |identity: TransactionUnit, slot: SlotNumber| {
        let registered_unreleased_on_some_device = allocator.devices.iter().any(|device_map| {
            allocator
                .record_for(device_map.device, slot)
                .is_some_and(|record| !record.is_released)
        });
        if registered_unreleased_on_some_device {
            placement_registered_unreleased_on_every_device(identity, slot, allocator).map(Some)
        } else {
            Ok(None)
        }
    };
    let mut released = Vec::new();
    if rewritten.contains(&TransactionUnit::TreeTable) {
        if let Some(tree_table) = allocator.format_time_tree_table() {
            released.extend(placement_still_to_release(
                TransactionUnit::TreeTable,
                tree_table.slot,
            )?);
        }
    }
    if rewritten.contains(&TransactionUnit::AllocationTree) {
        if let Some(tree) = allocator.allocation_record_tree_of_the_version_without_file() {
            for (node, pointer) in &tree.version.nodes {
                let slot = slot_shared_by_both_location_entries(&pointer.locations).expect(
                    "树表 0 条那一版的分配记录树：本进程写的两盘同槽，`mount` 记它之前逐节点判过两条位置条目同槽",
                );
                released.extend(placement_still_to_release(
                    role_of_allocation_record_tree_node(*node),
                    slot,
                )?);
            }
        }
    }
    Ok(released)
}

/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：`roles` 是这次重写的角色，
/// 上一版里这些角色的落点从上一版的映射节点按 key 查出来，查不到就报「不在映射」、一个落点都不释放；
/// 这次重写文件内容时，上一版比这一版多出来的那几个数据单元一并释放（文件变短），这一版新多出来的数据单元与叶容器在上一版里没有落点、跳过。
/// 映射树与树表豁免映射，各从根记录里指着它们的那条指针取
/// （D19（块指针的结构与宽度预算） 已定项 11：映射树的根住根记录）。两盘同槽（D2（RAID 条带策略） 已定项 10）。
/// 实例表链不在这里释放：整条旧链由 [`instance_table_chain_to_release`] 按计划带着的那条链逐片释放（第 1 片起的指针只在
/// 上一片的链指针记录里，不在上一版的内存态里）。
/// 映射只给槽号；跨度取分配记录里的，并与单元种类该有的跨度互核——查出来的槽在册、没释放过、跨度对得上，三样有一样不对就报错、
/// 一个落点都不释放（第二轮攻方腿：此前查得到 key 就直接交给 `release`，落点指错时在断言上 panic）。
/// **那三样对池里每块盘各核一遍**：`PoolAllocator::release` 对每块盘都要求一条对得上的记录，而两块盘的分配记录树
/// 对不对称是盘上读来的、不是不变量（panic 面普查 R10；此前只核 `locations[0].device` 那一块）。
///
/// # Errors
/// 查不到 key、或上一版的映射节点解不开 ⇒ `ReleaseNotInMapping`；映射节点里有条目窄于字段表 ⇒
/// `MappingEntryNarrowerThanItsFieldTable`；查出来的两条位置条目槽号不等 ⇒ `ReleaseTargetLocationsOnDifferentSlots`；
/// 槽在某块盘的分配记录里没条目 ⇒ `ReleaseTargetNotAllocated`；某块盘的条目已是已释放 ⇒ `ReleaseTargetAlreadyReleased`；
/// 某块盘的记录跨度与种类不符 ⇒ `ReleaseSpanMismatch`。三样都带着是哪块盘。
pub fn placements_to_release_via_mapping(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    roles: &[TransactionUnit],
    file_content: FileContentOfThePublish,
) -> Result<Vec<Placement>, PublishError> {
    placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
        previous,
        allocator,
        file_content,
        roles,
        &[],
    )
}

/// 同 [`placements_to_release_via_mapping`]，`kept_for_the_snapshots` 里的角色（被活快照引用、这次进 deadlist 的那几个，判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）不释放、不核：它们的落点不进 defer 队列，映射条目留存。
///
/// # Errors
/// 同 [`placements_to_release_via_mapping`]。
pub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    file_content: FileContentOfThePublish,
    roles: &[TransactionUnit],
    kept_for_the_snapshots: &[TransactionUnit],
) -> Result<Vec<Placement>, PublishError> {
    let _inside_the_releases =
        lookup_work::entering(LookupSite::MappingEntriesOfTheReleaseJudgement);
    let mut placements = Vec::new();
    // 换下的角色逐个取上一版的映射条目：key 与落点都走建一次的索引（七·6），不逐个在 `mapped_units` 上找、逐 key 重解叶。
    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
    let mut previous_files = FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version(previous);
    for identity in roles_other_than_the_ones_kept_for_the_snapshots(
        roles_replaced_via_mapping(previous, roles, file_content),
        kept_for_the_snapshots,
        LookupSite::RolesKeptForTheSnapshotsInTheReleaseJudgement,
    ) {
        let locations = match identity {
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
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
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
                let has_a_placement = {
                    let _inside_the_file_lookups = lookup_work::entering(
                        LookupSite::FilesOfTheReplacedRolesInThePreviousVersion,
                    );
                    has_a_placement_in_the_previous_version(previous, &mut previous_files, identity)
                };
                if !has_a_placement {
                    continue;
                }
                match previous_read_by_role.locations_of_role(identity) {
                    MappingLookup::Found(locations) => locations,
                    MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
                        return Err(PublishError::ReleaseNotInMapping { unit: identity })
                    }
                    MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
                        return Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
                            unit: identity,
                            entry_bytes,
                            field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
                        })
                    }
                }
            }
            // 映射树根之下的节点同样豁免映射：父条目里的位置条目是权威（D19（块指针的结构与宽度预算） 已定项 8），
            // 上一版记着每个节点的那条指针。
            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
                previous.central_mapping_tree.pointer_of(position).locations
            }
            TransactionUnit::MappingTree => previous.root.mapping_root.locations,
            TransactionUnit::TreeTable => previous.root.tree_table.locations,
            // 整条旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                continue
            }
        };
        placements.push(placement_to_release_after_checking_every_device(
            identity, &locations, allocator,
        )?);
    }
    Ok(placements)
}

/// 整条实例表链重写时被换下的那条旧链，逐片的落点（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）：
/// `replaced_chain` 是那条旧链每一片的指针，按片序号（第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条）。
/// 实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），不经映射、不做释放之前的读盘核；每一片照样按
/// `placement_to_release_after_checking_every_device` 核三样（两条位置条目同槽、池里每块盘在册、没释放过、跨度对得上）。
/// 只查不改：带文件的一版上写行（`publish_version`）与树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）都调它；
/// 可写挂载取号之前的预演（`mount`）走的是这两处落盘之前那一段，判的是同一件事。
///
/// # Errors
/// 某一片的三样核不过 ⇒ `ReleaseTargetLocationsOnDifferentSlots` / `ReleaseTargetNotAllocated` /
/// `ReleaseTargetAlreadyReleased` / `ReleaseSpanMismatch`，带着是第几片（角色）。
pub fn instance_table_chain_to_release(
    replaced_chain: &[NodePointer],
    allocator: &PoolAllocator,
) -> Result<Vec<Placement>, PublishError> {
    replaced_chain
        .iter()
        .enumerate()
        .map(|(position, page_pointer)| {
            placement_to_release_after_checking_every_device(
                TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
                    u64::try_from(position).expect("片序号"),
                )),
                &page_pointer.locations,
                allocator,
            )
        })
        .collect()
}

/// 这次发布写的文件内容是不是从第 0 个单元起整份重写（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3：
/// 由发布路径显式交，不按「这次重写的数据角色里有没有单元 0」推——截断意图那次只重写单元 0 时，那样推会把单元 1、2 一起换下）。
/// 经映射释放的三个调用点（释放判定、释放之前读盘核、deadlist 判生死）交的是同一个值：这次发布的计划带不带文件内容。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileContentOfThePublish {
    /// 覆盖写、顺序写、新池新建文件：从单元 0 起整份重写，上一版比这一版多出来的单元（文件变短）这次一起换下。
    RewrittenWholeFromTheFirstDataUnit,
    /// 别的发布：没有文件内容，或只重写一个尾单元（截断意图、变长）；换下的数据单元就是角色清单里点名的那几个。
    NotRewrittenWhole,
}

impl FileContentOfThePublish {
    /// 这次发布的计划带不带第一个文件的内容（[`PublishPlan::file`] 写的是 inode 1）。写第一个文件之外一个文件的发布不重写第一个文件，
    /// 是 `NotRewrittenWhole`：那个文件上一版的单元由 [`FilesOtherThanTheFirstPlan`] 整份点名换下，不经这里补。
    #[must_use]
    pub fn of_the_plan(plan: &PublishPlan<'_>) -> Self {
        match plan.file {
            Some(file) if file.inode.0 == FIRST_INODE_NUMBER => {
                FileContentOfThePublish::RewrittenWholeFromTheFirstDataUnit
            }
            // 写第一个文件之外的文件：第一个文件这次不重写。
            Some(_) => FileContentOfThePublish::NotRewrittenWhole,
            None => FileContentOfThePublish::NotRewrittenWhole,
        }
    }
}

/// 这次发布经映射换下的角色：这次重写的角色，加上文件内容整份重写时上一版比这一版多出来的那几个数据单元（文件变短）——
/// 它们这次没有对应的新角色，却同样被换下，不释放它们，它们就一直占着槽、再也没有树引用。释放判定路径
/// （`placements_to_release_via_mapping`）与释放之前读盘核校验和（`copies_failing_the_release_checksum_check`）读的是这同一张清单
/// （D19（块指针的结构与宽度预算） 已定项 5 第 1 条与硬规则 1：经映射释放的每一个都核）。文件内容这次没整份重写（写行、暖机、
/// 删除与截断的批、只重写尾单元的截断意图与变长）时不加：批截掉的尾部是 `ResolvedPublish::previous_roles_without_a_successor` 自己点名的。
pub(super) fn roles_replaced_via_mapping(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
    file_content: FileContentOfThePublish,
) -> Vec<TransactionUnit> {
    let previous_data_units = previous.data_pointers.len();
    let rewritten_data_units = roles
        .iter()
        .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
        .count();
    let previous_data_units_without_a_successor: Vec<TransactionUnit> = match file_content {
        FileContentOfThePublish::NotRewrittenWhole => Vec::new(),
        FileContentOfThePublish::RewrittenWholeFromTheFirstDataUnit => (rewritten_data_units
            ..previous_data_units)
            .map(|position| {
                TransactionUnit::Data(DataUnitIndexInFile(
                    u64::try_from(position).expect("单元序号"),
                ))
            })
            .collect(),
    };
    roles
        .iter()
        .copied()
        .chain(previous_data_units_without_a_successor)
        .collect()
}

/// `roles` 里不在 `kept_for_the_snapshots`（这次进 deadlist、留给快照的用户可见角色）里的那几个，次序不变。
/// 与 `roles.into_iter().filter(|identity| !kept_for_the_snapshots.contains(identity))` 逐个相同，但留给快照的角色清单先建一次按角色的索引、
/// 每个角色查一次（七·6 后续批，用户 2026-10-22 定并进）：快照下整份重写一个 N 单元的文件时两份清单各约 N 项，逐个 `contains` 是 N 的平方。
/// 建与查记在 `site`，进一次。
fn roles_other_than_the_ones_kept_for_the_snapshots(
    roles: Vec<TransactionUnit>,
    kept_for_the_snapshots: &[TransactionUnit],
    site: LookupSite,
) -> Vec<TransactionUnit> {
    let _inside_the_kept_roles = lookup_work::entering(site);
    let kept_for_the_snapshots_by_role =
        FirstEntryOfEachIdentity::of_the_table(kept_for_the_snapshots, |identity| *identity);
    roles
        .into_iter()
        .filter(|identity| {
            kept_for_the_snapshots_by_role
                .first_entry_of(identity)
                .is_none()
        })
        .collect()
}

/// 这个角色在上一版里有没有落点。这次发布新建的叶容器（末尾分裂出来的右半）在上一版里没有落点 ⇒ 没有东西要释放、要核；
/// 数据单元同一条：文件变长时新多出来的那几个单元在上一版里没有落点。
/// 判据取上一版的容器数、单元数，不取「映射里查不到」：查不到要报 `ReleaseNotInMapping`，那是另一件事。
/// 第一个文件之外那一族按 inode 号找文件走 `previous_files`（它就是 `previous` 的那张表的索引，七·6）。
pub(super) fn has_a_placement_in_the_previous_version(
    previous: &TransactionOutput,
    previous_files: &mut FilesOtherThanTheFirstOfAVersionReadByInode<'_>,
    identity: TransactionUnit,
) -> bool {
    match identity {
        TransactionUnit::InodeLeafContainer(index) => {
            index.position() < previous.inode_leaf_containers.len()
        }
        TransactionUnit::Data(index) => {
            index.0 < u64::try_from(previous.data_pointers.len()).expect("单元数")
        }
        // 第一个文件之外的文件：上一版里那个文件有没有这个单元、这个下段位置（这次第一次写它、或写长了，新多出来的在上一版里没有落点）。
        TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } => previous_files
            .file_other_than_the_first(inode)
            .is_some_and(|file| unit.0 < u64::try_from(file.data_pointers.len()).expect("单元数")),
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { inode, position } => {
            previous_files
                .file_other_than_the_first(inode)
                .is_some_and(|file| {
                    file.extent_lower_nodes
                        .iter()
                        .any(|(position_in_the_previous_version, _)| {
                            *position_in_the_previous_version == position
                        })
                })
        }
        // 按位置寻址的两棵树的节点：上一版有没有那个位置（这次新长出来的节点在上一版里没有落点）。
        TransactionUnit::ExtentLowerNode(position) => {
            previous.extent_tree.lower_pointer_of(position).is_some()
        }
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
            previous.extent_tree.upper_pointer_of(position).is_some()
                && position.level != previous.extent_tree.upper_root().level
        }
        TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => previous
            .allocation_record_tree
            .pointer_of(AllocationRecordTreeNode::BelowTheRoot(position))
            .is_some(),
        // 删除的两棵树：队列空的一版没有 `deleted_inodes` 根，墓碑树零根的一版没有根与容器（意图那次新写出来的在上一版里没有落点）。
        TransactionUnit::DeletedInodesRoot => !previous.deleted_inodes.is_empty(),
        // logged_ops：没有条目的一版没有根（签发那次新写出来的在上一版里没有落点）。
        TransactionUnit::LoggedOperationsRoot => !previous.logged_operations.is_empty(),
        TransactionUnit::TombstoneLeafContainer(index) => {
            index.position() < previous.tombstone_containers.len()
        }
        // 墓碑树的根：上一版有根就有落点（长高那次新根在上一版里没有，但那次的旧根不被换下——`previous_roles_replaced_by_this_publish`
        // 不把新根列进换下清单，这里问不到那一格）。根之下的节点：上一版那个位置上有节点、而它不是上一版的根。
        TransactionUnit::TombstoneRoot => !previous.tombstone_containers.is_empty(),
        TransactionUnit::TombstoneNodeBelowTheRoot(position) => {
            previous
                .tombstone_tree_index_nodes
                .shape
                .has_a_node_at(position)
                && !previous
                    .tombstone_tree_index_nodes
                    .shape
                    .is_the_root(position)
        }
        // 谱系树：零根的一版没有根与容器（回退那次新开的那一片在上一版里没有落点）。
        TransactionUnit::LineageLeafContainer(index) => {
            index.position() < previous.lineage_containers.len()
        }
        TransactionUnit::LineageRoot => !previous.lineage_containers.is_empty(),
        TransactionUnit::LineageNodeBelowTheRoot(position) => {
            previous
                .lineage_tree_index_nodes
                .shape
                .has_a_node_at(position)
                && !previous
                    .lineage_tree_index_nodes
                    .shape
                    .is_the_root(position)
        }
        // 退役树：零根的一版没有根与容器（第一次有身份消失的那次新开的那一片在上一版里没有落点）。
        TransactionUnit::RetirementLeafContainer(index) => {
            index.position() < previous.retirement_containers.len()
        }
        TransactionUnit::RetirementRoot => !previous.retirement_containers.is_empty(),
        TransactionUnit::RetirementNodeBelowTheRoot(position) => {
            previous
                .retirement_tree_index_nodes
                .shape
                .has_a_node_at(position)
                && !previous
                    .retirement_tree_index_nodes
                    .shape
                    .is_the_root(position)
        }
        // 多层码 2 树的节点按上一版的位置点名（`previous_roles_replaced_by_this_publish` 从计划里的「被换下的节点」取），恒在上一版里。
        TransactionUnit::ExtentRoot
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        | TransactionUnit::DeadlistTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable => true,
        // 第 1 片起的实例表页在不在上一版里，上一版的内存态答不出（指针只在上一片的链指针记录里）；两个调用方都只拿进映射的角色问，
        // 旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。这一臂只为穷举。
        TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
    }
}

/// 经映射释放的角色（数据、extent 根、inode 叶容器与根、分配记录树、记账树）：这次重写的角色里走得到映射条目的那几个，按角色次序。
/// 映射树、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），硬规则 1 的读盘核挂在「经映射核到那条映射条目之后」，
/// 它们不在其内；这次新多出来、上一版没有落点的角色也不在其内。
pub(super) fn mapped_roles_released_by_this_publish(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
    kept_for_the_snapshots: &[TransactionUnit],
    file_content: FileContentOfThePublish,
) -> Vec<TransactionUnit> {
    let roles_not_kept_for_the_snapshots = roles_other_than_the_ones_kept_for_the_snapshots(
        roles_replaced_via_mapping(previous, roles, file_content),
        kept_for_the_snapshots,
        LookupSite::RolesKeptForTheSnapshotsAmongTheMappedRolesReleased,
    );
    // 按 inode 号找文件的那份读者一调建至多一次：每调一次进一次这一处，建的次数与进的次数相同才是一调一份。
    let _inside_the_file_lookups =
        lookup_work::entering(LookupSite::FilesOfTheMappedRolesReleasedByThisPublish);
    let mut previous_files = FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version(previous);
    roles_not_kept_for_the_snapshots
        .into_iter()
        .filter(|identity| match identity {
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
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
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => true,
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
        })
        .filter(|identity| {
            has_a_placement_in_the_previous_version(previous, &mut previous_files, *identity)
        })
        .collect()
}

/// 一个经映射释放的角色在上一版映射里的那两条位置项。
///
/// # Panics
/// 在上一版的映射里查不到：调用方先走 `placements_to_release_via_mapping`、它按同一个上一版、同一串角色查过，
/// 查不到时它已经报了 `ReleaseNotInMapping` 或 `MappingEntryNarrowerThanItsFieldTable`，走不到这里。
fn mapping_locations_of_a_released_unit(
    previous_read_by_role: &mut MappedUnitsOfAVersionReadByRole<'_>,
    identity: TransactionUnit,
) -> [LocationEntry; 2] {
    match previous_read_by_role.locations_of_role(identity) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => panic!(
            "{identity:?} 在上一版的映射里查不到：placements_to_release_via_mapping 刚按同一个上一版查过，查不到时它已经报错返回"
        ),
    }
}

/// 经映射释放的每个单元，映射条目的两条位置项各指池里的一块盘、两条指的不是同一块（D19（块指针的结构与宽度预算） 已定项 5
/// 「硬规则 1 的读盘核读不出、核出对不上时怎么办」：位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏，
/// 在任何写之前拒绝，盘上不变；用户 2026-09-25 定）。只读内存里的上一版、不读盘：发布路径在读盘核之前判，
/// 可写挂载在取号之前的预演里按写行那一次换下的角色判（`mount` 的 `dry_run_of_the_publishes_after_acquisition`），
/// 两处判的是同一件事。
///
/// # Errors
/// 位置项指的盘不在 `pool_devices` 里 ⇒ `MappingEntryLocationOnADeviceOutsideThePool`；
/// 两条位置项指同一块盘 ⇒ `MappingEntryLocationsOnTheSameDevice`。按角色次序报第一个。
///
/// # Panics
/// 同 [`mapping_locations_of_a_released_unit`]。
pub fn refuse_mapping_entries_that_do_not_name_two_pool_devices(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
    kept_for_the_snapshots: &[TransactionUnit],
    pool_devices: &[DeviceIdentity],
    file_content: FileContentOfThePublish,
) -> Result<(), PublishError> {
    let _inside_the_releases =
        lookup_work::entering(LookupSite::MappingEntriesCheckedForTwoPoolDevices);
    // 同释放判定路径：key 与落点都走建一次的索引（七·6）。
    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
    for identity in
        mapped_roles_released_by_this_publish(previous, roles, kept_for_the_snapshots, file_content)
    {
        let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);
        refuse_locations_that_do_not_name_two_pool_devices(identity, &locations, pool_devices)?;
    }
    Ok(())
}

/// 一个经映射释放的单元，映射条目的两条位置项各指池里的一块盘、两条指的不是同一块（[`refuse_mapping_entries_that_do_not_name_two_pool_devices`]
/// 逐个单元判的那一件事；管理员回退释放现行那一版的用户可见单元时按它自己列的单元判同一件事，`mount::roll_back_by_a_forward_publish`）。
///
/// # Errors
/// 同 [`refuse_mapping_entries_that_do_not_name_two_pool_devices`]。
pub(crate) fn refuse_locations_that_do_not_name_two_pool_devices(
    identity: TransactionUnit,
    locations: &[LocationEntry; 2],
    pool_devices: &[DeviceIdentity],
) -> Result<(), PublishError> {
    if let Some(outside) = locations
        .iter()
        .find(|location| !pool_devices.contains(&location.device))
    {
        return Err(PublishError::MappingEntryLocationOnADeviceOutsideThePool {
            unit: identity,
            device: outside.device,
            slot: outside.slot,
        });
    }
    if locations[0].device == locations[1].device {
        return Err(PublishError::MappingEntryLocationsOnTheSameDevice {
            unit: identity,
            device: locations[0].device,
            slot: locations[0].slot,
        });
    }
    Ok(())
}

/// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，用户 2026-09-23 定案）：
/// 这次重写的角色里经映射释放的那几个（[`mapped_roles_released_by_this_publish`]），映射条目的每条位置项指的那一份
/// 整单元读出来算 CRC-32C、与位置项里的比（写的时候位置项的校验和就是整单元 CRC-32C，`make_filesystem::location_entries`）。
/// **读盘本身失败、或读出来核出对不上，都先重读一次，两次都没对上才按对不上处置**（读不出那一半用户 2026-09-24 定案，C394 三问的第一问；
/// 对不上那一半主 agent 2026-09-25 定，D19（块指针的结构与宽度预算） 已定项 5）：只重读一次，不退避、不多次重试。
/// **任一份核出对不上（读不出也算），这个单元在每块盘上的分配记录都留在「已分配」**、不改成已释放——另一块盘上那一份对得上也一起留，
/// 各盘的账保持对称（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定）：交回的是这些单元在池里每块盘上的那一份，
/// 按角色次序、位置项的次序（设备身份升序），每一份带着它自己读出来的样子（[`QuarantinedCopyReading`]）；发布照常释放（映射条目去掉），
/// 只是这些份的记录留在已分配（`PoolAllocator::release_leaving_the_record_allocated_on`），发布照成。
///
/// 只读，在动分配器、发任何一个写之前调；调用方先走 `placements_to_release_via_mapping`、它核过了才走到这里。
///
/// # Errors
/// 映射条目的位置项指池外的盘、或两条指同一块盘（[`refuse_mapping_entries_that_do_not_name_two_pool_devices`]）：
/// 在读任何一份之前返回。
///
/// # Panics
/// 同 [`mapping_locations_of_a_released_unit`]。
pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
    previous: &TransactionOutput,
    roles: &[TransactionUnit],
    kept_for_the_snapshots: &[TransactionUnit],
    reader: &Reader,
    file_content: FileContentOfThePublish,
) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
    let pool_devices = reader.device_identities();
    refuse_mapping_entries_that_do_not_name_two_pool_devices(
        previous,
        roles,
        kept_for_the_snapshots,
        &pool_devices,
        file_content,
    )?;
    let _inside_the_releases =
        lookup_work::entering(LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums);
    let mut quarantined = Vec::new();
    // 同释放判定路径：key 与落点都走建一次的索引（七·6）。
    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
    for identity in
        mapped_roles_released_by_this_publish(previous, roles, kept_for_the_snapshots, file_content)
    {
        let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);
        quarantined.extend(
            copies_of_a_released_unit_failing_the_release_checksum_check(
                identity, &locations, reader,
            ),
        );
    }
    Ok(quarantined)
}

/// 一个要释放的单元按它映射条目的两条位置项逐份读盘核（[`copies_failing_the_release_checksum_check`] 逐个单元做的那一件事，
/// 读法、重读一次、两份一起留的口径都在那里）：两份都对得上交回空；任一份没对上交回两份，各带它自己读出来的样子。
/// 管理员回退释放现行那一版的用户可见单元时按它自己列的单元调这一处（`mount::roll_back_by_a_forward_publish`），与发布路径同一段读法。
/// 调用方先判过两条位置项各指池里一块不同的盘（[`refuse_locations_that_do_not_name_two_pool_devices`]）。
pub(crate) fn copies_of_a_released_unit_failing_the_release_checksum_check<
    Reader: PoolReader + ?Sized,
>(
    identity: TransactionUnit,
    locations: &[LocationEntry; 2],
    reader: &Reader,
) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
    let unit_bytes = usize::try_from(identity.span_slots() * SLOT_BYTES)
        .expect("一个单元两槽以内，字节数装得进 usize");
    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
    let reading_of_each_copy: Vec<(LocationEntry, QuarantinedCopyReading)> = locations
        .iter()
        .map(|location| {
            // 第一次读不出、或读出来核出对不上，都先重读一次（只一次，D19（块指针的结构与宽度预算） 已定项 5
            // 「硬规则 1 的读盘核读不出、核出对不上时怎么办」）：一次瞬时坏读不许让单元永久隔离。
            // 重读那一次对得上就算这一份对得上；两次都没对上，按重读那一次的样子报。
            let reading = match read_one_copy_and_check_it_against_its_location_entry(
                reader, location, unit_bytes,
            ) {
                CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {
                    match read_one_copy_and_check_it_against_its_location_entry(
                        &rereading_reader,
                        location,
                        unit_bytes,
                    ) {
                        CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                        CopyCheck::Unreadable => QuarantinedCopyReading::UnreadableAfterOneReread,
                        CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,
                    }
                }
            };
            (*location, reading)
        })
        .collect();
    let some_copy_failed = reading_of_each_copy
        .iter()
        .any(|(_, reading)| match reading {
            QuarantinedCopyReading::ChecksumMismatch
            | QuarantinedCopyReading::UnreadableAfterOneReread => true,
            QuarantinedCopyReading::IntactButAnotherCopyFailed => false,
        });
    if !some_copy_failed {
        return Vec::new();
    }
    // 一份没对上、另一份对得上（重读也没对上的不算读到）：这一读读到只坏一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）。
    if reading_of_each_copy
        .iter()
        .any(|(_, reading)| match reading {
            QuarantinedCopyReading::IntactButAnotherCopyFailed => true,
            QuarantinedCopyReading::ChecksumMismatch
            | QuarantinedCopyReading::UnreadableAfterOneReread => false,
        })
    {
        reader.note_that_a_unit_was_read_with_one_copy_failing(locations);
    }
    // 两条位置项各指池里一块不同的盘（调用方判过；第一版池里恰好两块盘，mkfs 断言过），两份都留：按位置项的次序（设备身份升序）各交一份。
    reading_of_each_copy
        .into_iter()
        .map(
            |(location, reading)| CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: identity,
                device: location.device,
                placement: Placement {
                    slot: location.slot,
                    span: identity.span_slots(),
                },
                reading,
            },
        )
        .collect()
}

/// 按一条位置项读那一份整单元、算 CRC-32C 与位置项里的比（[`copies_of_a_released_unit_failing_the_release_checksum_check`] 读一次、重读一次各走它）：
/// 对得上的那几字节报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]：重读那一遍的读者拿它换掉这次挂载记下的那一份），
/// 读成而对不上的也报（[`PoolReader::note_that_the_bytes_read_fail_self_verification`]：这次挂载共用的那一份记进读坏账、丢掉那几块）。只读盘。
fn read_one_copy_and_check_it_against_its_location_entry<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    location: &LocationEntry,
    unit_bytes: usize,
) -> CopyCheck {
    let offset = location.slot.to_device_offset();
    match reader.read(location.device, offset, unit_bytes) {
        None => CopyCheck::Unreadable,
        Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {
            reader.note_that_the_bytes_read_fail_self_verification(
                location.device,
                offset,
                unit_bytes,
            );
            CopyCheck::ChecksumMismatch
        }
        Some(copy) => {
            reader.note_that_the_bytes_read_self_verify(location.device, offset, &copy);
            CopyCheck::Intact
        }
    }
}

/// 释放之前读盘核一份的一次结果（[`copies_failing_the_release_checksum_check`]）：读不出、读出来整单元 CRC-32C 与位置项里的对不上、对得上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CopyCheck {
    Unreadable,
    ChecksumMismatch,
    Intact,
}

/// 一个被换下的单元的落点：两条位置条目同槽，池里每块盘各有一条在册、未释放、跨度对得上的记录；有一样不对就报错、不交回落点。
///
/// **池里每块盘都核一遍，不只核 `locations[0].device` 那一块**：`PoolAllocator::release` 对每块盘都要求
/// 一条在册、未释放、跨度对得上的记录，而两块盘的分配记录树对不对称是盘上读来的、不是不变量
/// （panic 面普查 R10：盘 0 有记录、盘 1 没有 / 已释放 / 跨度不同时，此前这三样在 `release` 的
/// `expect` 与两条断言上 panic）。这道判与那三样逐条对齐：少判一样，那一样就还在断言上。
/// 跨度取核过的那个值，每块盘都要与种类该有的跨度相同，所以逐盘核完取哪一块都一样。
///
/// 三处调它，判的是同一件事、不各写一份：带文件的一版按映射（或根记录）查出位置条目之后
/// （`placements_to_release_via_mapping`）；树表 0 条的一版上写行时按根记录里那条实例表指针
/// （`publish_instance_table_on_version_without_file`——那一版没有上一版的内存态，也没有映射树）；管理员回退释放现行那一版
/// 不再被引用的用户可见单元时（`mount::roll_back_by_a_forward_publish`）。逐盘那一半另有第四处：第一个文件版本换下 mkfs 那片树表
/// 与树表 0 条那一版的分配记录树节点（`format_time_tree_table_to_release`，槽号取分配器记着的，不经位置条目）。
///
/// # Errors
/// `ReleaseTargetLocationsOnDifferentSlots`、`ReleaseTargetNotAllocated`、`ReleaseTargetAlreadyReleased`、`ReleaseSpanMismatch`。
pub(crate) fn placement_to_release_after_checking_every_device(
    identity: TransactionUnit,
    locations: &[LocationEntry; 2],
    allocator: &PoolAllocator,
) -> Result<Placement, PublishError> {
    let slot = slot_shared_by_both_location_entries(locations).map_err(|disagreement| {
        PublishError::ReleaseTargetLocationsOnDifferentSlots {
            unit: identity,
            disagreement,
        }
    })?;
    placement_registered_unreleased_on_every_device(identity, slot, allocator)
}

/// [`placement_to_release_after_checking_every_device`] 逐盘的那一半：池里每块盘各有一条起点是 `slot`、在册、未释放、
/// 跨度与这个角色的种类相符的分配记录，交回那个落点；有一块盘不对就报错、不交回。
///
/// # Errors
/// `ReleaseTargetNotAllocated`、`ReleaseTargetAlreadyReleased`、`ReleaseSpanMismatch`，带着是哪块盘。
fn placement_registered_unreleased_on_every_device(
    identity: TransactionUnit,
    slot: SlotNumber,
    allocator: &PoolAllocator,
) -> Result<Placement, PublishError> {
    for device in allocator.devices.iter().map(|device_map| device_map.device) {
        let record =
            allocator
                .record_for(device, slot)
                .ok_or(PublishError::ReleaseTargetNotAllocated {
                    unit: identity,
                    device,
                    slot,
                })?;
        if record.is_released {
            return Err(PublishError::ReleaseTargetAlreadyReleased {
                unit: identity,
                device,
                slot,
            });
        }
        let recorded_span = u64::from(record.span_slots);
        if recorded_span != identity.span_slots() {
            return Err(PublishError::ReleaseSpanMismatch {
                unit: identity,
                device,
                slot,
                recorded_span,
                expected_span: identity.span_slots(),
            });
        }
    }
    Ok(Placement {
        slot,
        span: identity.span_slots(),
    })
}
