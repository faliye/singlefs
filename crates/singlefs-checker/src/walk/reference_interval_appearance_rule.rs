//! 回退候选集里相邻两版之间 I-3.5（引用区间的精确性） 的出现规则：两版各自引用的落点、谱系里的回退记录与候选集里的空洞。

use super::*;

/// 走完的一版留给 I-3.5 的东西：它是哪一版、引用过哪些落点（还没折成单元）、这些落点在这一版里各落在哪棵用户可见树里、
/// 树表里几棵用户可见树的根、走读断没断。标签当场存下：它只看这一版自己的树，后走的版本不改它。
/// 落点到全部版本都走完才折：折叠表（`Walk::unit_of_each_placement`）在后走的版本读回第 1 条那一份时还会改
/// （[`Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），每版走完当场折的话，先走那一版存下的单元跟不上这一改，
/// I-3.5 随根环次序翻面。
/// GPU 走读内核同一个取法（每版只存落点表的下标，判时按落点表那一行此刻的单元认）。
pub(super) struct VersionPlacementsForTheAppearanceRule {
    pub(super) instance: u32,
    pub(super) checkpoint_txg: u64,
    pub(super) placements: Vec<(u32, u64)>,
    /// 这一版走读记下的落点标签（[`Walk::user_visible_tree_of_each_placement_in_this_version`] 走完这一版时的那一张）：不随后走的版本改。
    pub(super) user_visible_tree_of_each_placement: BTreeMap<(u32, u64), UserVisibleTreeOfTheWalk>,
    pub(super) user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
    pub(super) references_are_complete: bool,
    /// 这一版树表里第一条「走到没有条款的形状」的条目的种类（[`tree_table_entry_is_a_shape_without_a_clause`]）；没有是 `None`。
    pub(super) kind_of_a_tree_table_entry_without_a_clause: Option<u16>,
    /// 这一版的实例表指针头里的出生 txg（[`instance_table_pointer_birth_txg_of`]）：取号跳过的第四条件读它。
    pub(super) instance_table_pointer_birth_txg: Option<u64>,
    /// 这一版树表里谱系树的全部谱系记录（[`lineage_records_of_a_version_for_the_appearance_rule`]），读不全交回判不了的是哪一样：
    /// (b) 的谱系那一半读它。
    pub(super) lineage_records: Result<Vec<LineageRecordOfTheScan>, DeletionScanGap>,
}

/// I-3.5 比相邻两版时一个落点的三样：它在比的时候认作哪一个（第一次记下时按中央映射 key 折的按折叠表折成的单元认，别的按它自己认，
/// `Walk::placements_first_noted_through_the_central_mapping`）、取诞生代号时折成的单元（全部版本走完之后的折叠表 `Walk::unit_of_each_placement`）、这一版里它落在哪棵用户可见树里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlacementOfTheAppearanceRule {
    pub(super) identity_compared_across_versions: (u32, u64),
    pub(super) unit_whose_birth_txg_it_takes: (u32, u64),
    pub(super) user_visible_tree: Option<UserVisibleTreeOfTheWalk>,
}

/// 走完的一版在 I-3.5 眼里的样子：它是哪一版、引用了哪些落点（各认作哪一个、各取哪个单元的诞生代号、各落在哪棵用户可见树里）、
/// 树表里几棵用户可见树的根、走读断没断。
pub(super) struct VersionReferencesForTheAppearanceRule {
    pub(super) instance: u32,
    pub(super) checkpoint_txg: u64,
    /// 这一版引用的落点 → 它在 I-3.5 里的三样（C748（最终折叠表漏判引用区间的落点）：按落点比，折叠表只用来取诞生代号）。
    pub(super) placements: BTreeMap<(u32, u64), PlacementOfTheAppearanceRule>,
    pub(super) user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
    /// 这一版走读一条走读失败都没记：记了就是引用集合数不全，它参与的每一对报不适用。
    pub(super) references_are_complete: bool,
    /// 这一版树表里第一条「走到没有条款的形状」的条目的种类：有就是引用集合数不全，它参与的每一对报不适用（[`VersionPlacementsForTheAppearanceRule`] 的同名字段）。
    pub(super) kind_of_a_tree_table_entry_without_a_clause: Option<u16>,
    /// 这一版的实例表指针头里的出生 txg（[`instance_table_pointer_birth_txg_of`]）：取号跳过的第四条件读它。
    pub(super) instance_table_pointer_birth_txg: Option<u64>,
    /// 这一版谱系树里的全部谱系记录，读不全交回判不了的是哪一样（[`VersionPlacementsForTheAppearanceRule`] 的同名字段）。
    pub(super) lineage_records: Result<Vec<LineageRecordOfTheScan>, DeletionScanGap>,
}

/// I-3.5 (b) 的谱系那一半（谱系记录落地之后的区间形第 9 行，
/// 写在 `crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行）：R 的谱系树里有一条记录 (T_rb, T_old)，
/// R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old。取区间、不取 T_rb = R.txg：回退那次发布的根槽丢了、
/// 后一条根成了 R 时，那条记录随现行那一版只增不删、仍在 R 的谱系树里（字面形在这一形上红合法镜像）。
fn lineage_records_name_a_rollback_between_the_two_versions_to_a_target_not_older_than_the_root(
    lineage_records: &[LineageRecordOfTheScan],
    previous_txg: u64,
    version_txg: u64,
    root_pointer_birth_txg: u64,
) -> bool {
    lineage_records.iter().any(|record| {
        previous_txg < record.rollback_txg
            && record.rollback_txg <= version_txg
            && root_pointer_birth_txg <= record.parent_txg
    })
}

/// 一版树表里谱系树的全部谱系记录，只读不判（I-3.5 (b) 的谱系那一半要它）。树表读不出、条目比字段表窄、谱系树条目不是恰好一条、
/// 谱系树读不全，交回判不了的是哪一样（读不出、形状不对由主走读与删除态扫描判，这里只如实记「判不了」）；树表 0 条的一版没有谱系树，交回空。
pub(super) fn lineage_records_of_a_version_for_the_appearance_rule(
    reader: &dyn ImageReader,
    tree_table_pointer: &[u8],
    mapping_root_pointer: &[u8],
    index_cache: &mut IndexNodeCache,
) -> Result<Vec<LineageRecordOfTheScan>, DeletionScanGap> {
    let tree_table_pointer = parse_node_pointer(tree_table_pointer);
    if tree_table_pointer.all_zero {
        return Err(DeletionScanGap::TreeTablePointerAllZero);
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        index_cache,
    )
    .ok_or(DeletionScanGap::TreeTableUnreadable)?;
    if tree_table
        .entries
        .iter()
        .any(|entry| entry.len() < tree_table_entry_bytes())
    {
        return Err(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable);
    }
    if tree_table.entries.is_empty() {
        return Ok(Vec::new());
    }
    let lineage_entries: Vec<&Vec<u8>> = tree_table
        .entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LINEAGE)
        .collect();
    let [lineage_entry] = lineage_entries.as_slice() else {
        return Err(DeletionScanGap::LineageTreeEntryNotSingle);
    };
    // 谱系树里豁免三类之外的单元按这一版自己的中央映射认位置（读不出时是空表，照提示），同删除态扫描。
    let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
        reader,
        &parse_node_pointer(mapping_root_pointer),
        index_cache,
    );
    lineage_records_without_judging(
        reader,
        &parse_node_pointer(&lineage_entry[14..100]),
        &central_mapping,
        index_cache,
        &mut PackedUnitCache::new(),
    )
}

/// 一版的实例表指针（根记录偏移 170 那一段，或由记录施加出来那一版记录新根段里的那一段）头里的出生 txg；指针整条全零时没有。
/// I-3.5 取号跳过的第四条件（C699（取号跳过改认出生代号））读它：
/// 全零的指针按「这一版不是它那个实例新建实例表的那一版」认，与 E198（取号跳过认出生代号的前提） 装置的甲同一个读法。
pub(super) fn instance_table_pointer_birth_txg_of(instance_table_pointer: &[u8]) -> Option<u64> {
    let pointer = parse_node_pointer(instance_table_pointer);
    (!pointer.all_zero).then_some(pointer.birth_txg)
}

/// 回退候选集里相邻两版之间有没有洞（I-3.5 的缺口条款；I-3.1 在短环上洞里的误红归下一批调查，那一处要接同一份判法，
/// `research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 5 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HoleInTheCandidateSet {
    /// R.txg = R⁻.txg + 1。
    NoHole,
    /// 有洞，但最新根的实例表行证明 R 所在的实例直接接在 R⁻ 那一版上（gap-row），且 R 是它那个实例新建实例表的那一版
    /// （R 的实例表指针出生 txg 等于 R.txg）：这一对照常判。
    HoleBridgedByTheInstanceTableRows,
    /// 有洞、实例表行接不上，或接得上而 R 的实例表指针不是 R 那一版新建的（R 所在实例自己更早的版本存在过）：
    /// 用户可见树子树里的单元不判（N3），别的照 (a) 判。
    HoleNotBridgedByTheInstanceTableRows,
}

/// 相邻两版 (R⁻, R) 之间的洞：R⁻ 那一侧是同 txg 并成的一条（几个实例都算 R⁻ 的实例）。接得上（取号跳过）⟺ 对 R⁻ 的某个实例 i：
/// 最新根的实例表里有一行 (i, T = R⁻.txg)，R 的实例 v > i，且 i 与 v 之间的每个实例 j 都有一行、都是 (j, 0, 0)；
/// 再加第四条件：R 的实例表指针出生 txg 等于 R.txg（`version_instance_table_pointer_birth_txg`，指针全零时没有、按不等认）——
/// R 所在实例自己更早的版本不算跳过（C699（取号跳过改认出生代号），
/// 甲那一行；前提在 E198（取号跳过认出生代号的前提） 量过）。
/// 已知判不对的两形（清扫那一版也满足第四条件：sw-read、sw-crash-wrapped）照旧认作接得上，另立账。
/// 迭代上界是实例表的行数（不按实例代号逐个数，代号是盘上读来的 4 字节）。
fn hole_in_the_candidate_set_between(
    previous_txg: u64,
    previous_instances: &BTreeSet<u32>,
    version_txg: u64,
    version_instance: u32,
    version_instance_table_pointer_birth_txg: Option<u64>,
    instance_table_rows_of_the_newest_root: &[InstanceTableRow],
) -> HoleInTheCandidateSet {
    if previous_txg.checked_add(1) == Some(version_txg) {
        return HoleInTheCandidateSet::NoHole;
    }
    let bridged_from = |previous_instance: u32| {
        if version_instance <= previous_instance {
            return false;
        }
        let row_of_the_previous_version =
            instance_table_rows_of_the_newest_root.iter().any(|row| {
                row.instance == previous_instance && row.published_checkpoint_txg == previous_txg
            });
        let rows_between: Vec<&InstanceTableRow> = instance_table_rows_of_the_newest_root
            .iter()
            .filter(|row| row.instance > previous_instance && row.instance < version_instance)
            .collect();
        let instances_between: BTreeSet<u32> =
            rows_between.iter().map(|row| row.instance).collect();
        let every_instance_between_has_a_row = u64::try_from(instances_between.len())
            .expect("实例表行数装得进 u64")
            == u64::from(version_instance - previous_instance - 1);
        let every_row_between_published_nothing = rows_between.iter().all(|row| {
            row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0
        });
        row_of_the_previous_version
            && every_instance_between_has_a_row
            && every_row_between_published_nothing
    };
    let version_is_the_one_that_created_its_instance_table =
        version_instance_table_pointer_birth_txg == Some(version_txg);
    if previous_instances
        .iter()
        .any(|previous_instance| bridged_from(*previous_instance))
        && version_is_the_one_that_created_its_instance_table
    {
        return HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows;
    }
    HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
}

/// I-3.5（引用区间的精确性） 的出现规则（TF + β + gap + gap-row + N3；
/// 措辞是 `crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行）：回退候选集（走过的每一版，与 I-7.4 同一份）按 txg 升序，
/// 同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 逐对判。R 引用而 R⁻ 不引用的每个落点 b
/// （第一次出现与再次出现都判；「不引用」按落点认，中央映射 key 折的落点按 key 的单元认，[`PlacementOfTheAppearanceRule`]）满足
/// (a) birth(b) > R⁻.txg；或 (b) b 落在 R 树表里一棵用户可见树的子树里，那棵树的根指针出生 txg 严格小于 R⁻.txg，且 R 里那棵子树每个读得出的
/// 单元头的诞生代号都不大于那个出生 txg，且 R 的谱系树里有一条记录担保这棵树是回退带回来的
/// （[`lineage_records_name_a_rollback_between_the_two_versions_to_a_target_not_older_than_the_root`]）。
/// 不满足 (a)、满足 (b) 前两半而 R 的谱系树读不全的落点判不了，记部分没判（条款「谱系树读不全时该对不适用」；
/// 不满足 (a) 也不满足 (b) 前两半的不读谱系、照判，满足 (a) 的照判成立）。
/// birth(b) 取 b 按全部版本走完之后的折叠表折成的单元头里的诞生代号，头读不出的不判、记部分没判。
/// 有洞（[`hole_in_the_candidate_set_between`]）而实例表行接不上时，用户可见树子树里的落点不判（N3）。R 或 R⁻ 走读记过失败的那一对不判；
/// 一个落点都没判到时整条报不适用。
pub(super) fn judge_the_appearance_rule_of_adjacent_candidate_versions(
    versions: &[VersionReferencesForTheAppearanceRule],
    birth_txg_of_walked_units: &BTreeMap<(u32, u64), u64>,
    birth_txg_of_walked_instance_table_pages: &BTreeMap<(u32, u64), u64>,
    instance_table_rows_of_the_newest_root: &[InstanceTableRow],
    judgements: &mut Judgements,
) {
    let birth_txg_of = |unit: &(u32, u64)| {
        birth_txg_of_walked_units
            .get(unit)
            .or_else(|| birth_txg_of_walked_instance_table_pages.get(unit))
            .copied()
    };
    let mut versions_of_each_txg: BTreeMap<u64, Vec<&VersionReferencesForTheAppearanceRule>> =
        BTreeMap::new();
    for version in versions {
        versions_of_each_txg
            .entry(version.checkpoint_txg)
            .or_default()
            .push(version);
    }
    let groups: Vec<(u64, Vec<&VersionReferencesForTheAppearanceRule>)> =
        versions_of_each_txg.into_iter().collect();
    let mut units_judged: u64 = 0;
    // 读不出而没判的（C701（别的判法静默跳过读不出的部分））：哪一对相邻两版、哪个单元，逐条列出。
    let mut unjudged_because_something_does_not_read_back: Vec<String> = Vec::new();
    for adjacent in groups.windows(2) {
        let (previous_txg, previous_versions) = (adjacent[0].0, &adjacent[0].1);
        let versions_at_this_txg = &adjacent[1].1;
        let previous_references_are_complete = previous_versions
            .iter()
            .all(|previous_version| previous_version.references_are_complete);
        // R⁻ 那一组引用的落点各认作哪一个（C748：按落点比，不按折叠表折成单元）。
        let units_referenced_by_the_previous_version: BTreeSet<(u32, u64)> = previous_versions
            .iter()
            .flat_map(|previous_version| {
                previous_version
                    .placements
                    .values()
                    .map(|appearance| appearance.identity_compared_across_versions)
            })
            .collect();
        let previous_instances: BTreeSet<u32> = previous_versions
            .iter()
            .map(|previous_version| previous_version.instance)
            .collect();
        for version in versions_at_this_txg {
            if !previous_references_are_complete || !version.references_are_complete {
                unjudged_because_something_does_not_read_back.push(format!(
                    "相邻两版 (实例 {previous_instances:?}、txg {previous_txg}) → (实例 {}、txg {})：有一版走读记过失败、引用没收全，这一对不判",
                    version.instance, version.checkpoint_txg
                ));
                continue;
            }
            if let Some(kind) = previous_versions
                .iter()
                .chain(std::iter::once(version))
                .find_map(|walked_version| {
                    walked_version.kind_of_a_tree_table_entry_without_a_clause
                })
            {
                unjudged_because_something_does_not_read_back.push(format!(
                    "相邻两版 (实例 {previous_instances:?}、txg {previous_txg}) → (实例 {}、txg {})：有一版树表里有种类 {kind} 的条目，走读走到没有条款的形状、引用没收全，这一对不判",
                    version.instance, version.checkpoint_txg
                ));
                continue;
            }
            let hole = hole_in_the_candidate_set_between(
                previous_txg,
                &previous_instances,
                version.checkpoint_txg,
                version.instance,
                version.instance_table_pointer_birth_txg,
                instance_table_rows_of_the_newest_root,
            );
            let trees_inherited_whole: BTreeSet<UserVisibleTreeOfTheWalk> = version
                .user_visible_tree_roots
                .iter()
                .filter(|root| {
                    root.root_pointer_birth_txg < previous_txg
                        && version
                            .placements
                            .values()
                            .filter(|appearance| appearance.user_visible_tree == Some(root.tree))
                            .filter_map(|appearance| {
                                birth_txg_of(&appearance.unit_whose_birth_txg_it_takes)
                            })
                            .all(|birth_txg| birth_txg <= root.root_pointer_birth_txg)
                })
                .map(|root| root.tree)
                .collect();
            // (b) 的谱系那一半，按整棵继承下来的每棵树各判一次（谱系树读不全的交回判不了的是哪一样）。
            let rollback_on_the_lineage_of_each_tree_inherited_whole: BTreeMap<
                UserVisibleTreeOfTheWalk,
                Result<bool, DeletionScanGap>,
            > = version
                .user_visible_tree_roots
                .iter()
                .filter(|root| trees_inherited_whole.contains(&root.tree))
                .map(|root| {
                    let brought_back = version.lineage_records.as_ref().map_err(|gap| *gap).map(
                        |lineage_records| {
                            lineage_records_name_a_rollback_between_the_two_versions_to_a_target_not_older_than_the_root(
                                lineage_records,
                                previous_txg,
                                version.checkpoint_txg,
                                root.root_pointer_birth_txg,
                            )
                        },
                    );
                    (root.tree, brought_back)
                })
                .collect();
            for (placement, appearance) in &version.placements {
                let unit = &appearance.identity_compared_across_versions;
                let user_visible_tree = &appearance.user_visible_tree;
                if units_referenced_by_the_previous_version.contains(unit) {
                    continue;
                }
                if hole == HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
                    && user_visible_tree.is_some()
                {
                    continue;
                }
                let birth_unit = appearance.unit_whose_birth_txg_it_takes;
                let Some(birth_txg) = birth_txg_of(&birth_unit) else {
                    unjudged_because_something_does_not_read_back.push(format!(
                        "相邻两版 txg {previous_txg} → (实例 {}、txg {})：后一版新引用的落点盘 {} 槽 {}（折成单元盘 {} 槽 {}）头读不出、诞生代号不知道",
                        version.instance, version.checkpoint_txg, placement.0, placement.1, birth_unit.0, birth_unit.1
                    ));
                    continue;
                };
                let born_after_the_previous_version = birth_txg > previous_txg;
                let in_a_tree_inherited_whole =
                    user_visible_tree.is_some_and(|tree| trees_inherited_whole.contains(&tree));
                // (b) 的谱系那一半只在 (a) 不成立、前两半成立时用：别的落点不管谱系读不读得全都判得出。
                let rollback_on_the_lineage = user_visible_tree.and_then(|tree| {
                    rollback_on_the_lineage_of_each_tree_inherited_whole
                        .get(&tree)
                        .copied()
                });
                let inherited_tree_brought_back_by_a_rollback_on_the_lineage = match (
                    born_after_the_previous_version,
                    rollback_on_the_lineage,
                ) {
                    (false, Some(Ok(brought_back))) => brought_back,
                    (false, Some(Err(gap))) => {
                        unjudged_because_something_does_not_read_back.push(format!(
                            "相邻两版 txg {previous_txg} → (实例 {}、txg {})：后一版新引用的落点盘 {} 槽 {}（折成单元盘 {} 槽 {}，用户可见树 {user_visible_tree:?}）诞生代号 {birth_txg} 不大于前一版的 txg、落在整棵继承下来的树里，后一版的谱系树读不全（{gap:?}），谱系那一半判不了",
                            version.instance, version.checkpoint_txg, placement.0, placement.1, birth_unit.0, birth_unit.1
                        ));
                        continue;
                    }
                    (false | true, None) | (true, Some(Ok(_) | Err(_))) => false,
                };
                units_judged += 1;
                judgements.judge(
                    REFERENCE_INTERVALS_ARE_EXACT,
                    born_after_the_previous_version
                        || (in_a_tree_inherited_whole
                            && inherited_tree_brought_back_by_a_rollback_on_the_lineage),
                    || {
                        format!(
                            "相邻两版 (实例 {previous_instances:?}、txg {previous_txg}) → (实例 {}、txg {})（{hole:?}）：后一版引用、前一版不引用的落点盘 {} 槽 {}（折成单元盘 {} 槽 {}，用户可见树 {user_visible_tree:?}）诞生代号 {birth_txg} 不大于前一版的 txg，也不在一棵整棵继承下来、谱系担保是回退带回来的用户可见树里（根指针出生 txg 小于前一版的 txg、子树里读得出的诞生代号都不大于它，且后一版的谱系树里有一条记录 (T_rb, T_old) 满足前一版的 txg < T_rb ≤ 后一版的 txg、根指针出生 txg ≤ T_old；整棵继承：{in_a_tree_inherited_whole}）",
                            version.instance, version.checkpoint_txg, placement.0, placement.1, birth_unit.0, birth_unit.1
                        )
                    },
                );
            }
        }
    }
    if !unjudged_because_something_does_not_read_back.is_empty() {
        judgements.partially_judged(
            REFERENCE_INTERVALS_ARE_EXACT,
            unjudged_because_something_does_not_read_back.join("；"),
        );
    }
    if units_judged == 0 {
        judgements.not_applicable(
            REFERENCE_INTERVALS_ARE_EXACT,
            "回退候选集里相邻两版之间一个新出现、头读得出的落点都没判到：候选集只剩一版，或缺口那一对新出现的只有用户可见树里的落点",
        );
    }
}

#[cfg(test)]
mod tests;
