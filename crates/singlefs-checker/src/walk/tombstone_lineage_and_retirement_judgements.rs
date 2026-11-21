//! 墓碑、谱系与退役记录的判定：退役记录合法、不重复、不指活叶与 I-9.35（退役完备），inode 号水位的台阶，墓碑与谱系，
//! 容器容量，I-9.29（同号只许一个出生代）。

use super::*;

/// I-9.32（退役记录合法）在一版的一棵退役树上：每条记录被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg；
/// 每片退役容器的记录数 ≤ 959。不按谱系滤：每条都判。
fn judge_every_retirement_record_of_a_version_is_well_formed(
    version: &VersionToScanTheDeletionStateOf<'_>,
    retirement_tree: u64,
    containers: &[RetirementContainerOfTheScan],
    judgements: &mut Judgements,
) {
    let records_at_most = usize::try_from(RETIREMENT_CONTAINER_RECORDS).expect("959 装得进 usize");
    for container in containers {
        let record_count = container.records.len();
        judgements.judge(
            EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
            record_count <= records_at_most,
            || {
                format!(
                    "{}退役树 {retirement_tree} 里容器 {} 装了 {record_count} 条退役记录，多于一片的 {records_at_most} 条",
                    version.name, container.container
                )
            },
        );
        for record in &container.records {
            let retired = record.retired;
            let holds = retired.record_type == PACKED_TYPE_INODE
                && retired.birth_tree != 0
                && retired.container_birth < record.death_txg
                && record.death_txg <= version.checkpoint_txg;
            judgements.judge(EVERY_RETIREMENT_RECORD_IS_WELL_FORMED, holds, || {
                format!(
                    "{}退役树 {retirement_tree} 容器 {} 里的退役记录 {retired:?}（死于 txg {}）：被退役身份的类型要是 2、出生树不为 0，\
                     容器出生代 < 死亡 txg ≤ 这一版的 txg {}",
                    version.name, container.container, record.death_txg, version.checkpoint_txg
                )
            });
        }
    }
}

/// I-9.33（谱系上的退役记录按身份不重复）在一版的一棵退役树上：死亡 txg 在这一版父链上的那几条，按身份不重复。
fn judge_retirement_records_on_the_lineage_of_a_version_are_distinct(
    version_name: &str,
    retirement_tree: u64,
    containers: &[RetirementContainerOfTheScan],
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    let mut seen: BTreeMap<ContainerIdentityOfTheScan, u64> = BTreeMap::new();
    for record in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|record| lineage_chain.holds_txg(record.death_txg))
    {
        let earlier_death_txg = seen.insert(record.retired, record.death_txg);
        judgements.judge(
            RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
            earlier_death_txg.is_none(),
            || {
                format!(
                    "{version_name}退役树 {retirement_tree} 里谱系上有两条退役记录指同一个身份 {:?}（死于 txg {:?} 与 {}）",
                    record.retired, earlier_death_txg, record.death_txg
                )
            },
        );
    }
}

/// I-9.34（谱系上的退役记录不指本版活叶）在一版的一棵退役树上：死亡 txg 在这一版父链上的退役记录指认的身份不在头 ID 配得上的那棵 inode 树的叶身份集里。
fn judge_retirement_records_on_the_lineage_name_no_live_leaf(
    version_name: &str,
    retirement_tree: u64,
    containers: &[RetirementContainerOfTheScan],
    leaf_identities: &BTreeSet<ContainerIdentityOfTheScan>,
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    for record in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|record| lineage_chain.holds_txg(record.death_txg))
    {
        judgements.judge(
            RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
            !leaf_identities.contains(&record.retired),
            || {
                format!(
                    "{version_name}退役树 {retirement_tree} 里谱系上的退役记录（死于 txg {}）指的身份 {:?} 是这一版 inode 树里的一片叶",
                    record.death_txg, record.retired
                )
            },
        );
    }
}

/// I-9.35（退役完备）：每一版、它每棵 inode 树，父版（[`VersionOfTheRetirementCompleteness::parent_txg`]）在回退候选集里读得出时，
/// 父版那棵树的叶身份集 − 这一版的叶身份集 = 这一版死亡 txg 等于自己 txg 的退役记录的身份集；两个方向（漏记、多记）各报。
/// 父版那个 txg 不在候选集里的不判（崩溃跳号、最旧那一版）；父版读不全、同 txg 有不止一版的报射程里判不了。
/// 候选集滤不掉被崩溃恢复抛弃的根时整条报部分没判（照 I-9.29 的门：被抛弃线上的同一个 txg 可以是另一版）。
fn judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version(
    versions: &[VersionOfTheRetirementCompleteness],
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    judgements: &mut Judgements,
) {
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            judgements.partially_judged(
                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                format!(
                    "最新根的实例表读不出（主走读沿链没走到末片）：回退候选集里滤不掉被崩溃恢复抛弃的根，{} 版的父版一版都没比",
                    versions.len()
                ),
            );
            return;
        }
    }
    let empty_leaf_set: BTreeSet<ContainerIdentityOfTheScan> = BTreeSet::new();
    for version in versions {
        if version.leaf_identities_of_each_inode_tree.is_empty() {
            continue;
        }
        let parent_txg = match version.parent_txg {
            Ok(Some(parent_txg)) => parent_txg,
            Ok(None) => continue,
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                );
                continue;
            }
        };
        let parents: Vec<&VersionOfTheRetirementCompleteness> = versions
            .iter()
            .filter(|candidate| candidate.checkpoint_txg == parent_txg)
            .collect();
        let parent = match parents.as_slice() {
            [] => continue,
            [parent] => *parent,
            [_, _, ..] => {
                DeletionScanGap::ParentVersionNotSingle.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                );
                continue;
            }
        };
        if let Some(gap) = parent.unreadable {
            gap.note_as_a_part_of_the_range_left_unjudged(
                judgements,
                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
            );
            continue;
        }
        for (inode_tree, leaf_identities) in &version.leaf_identities_of_each_inode_tree {
            let parent_leaf_identities =
                match parent.leaf_identities_of_each_inode_tree.get(inode_tree) {
                    None => &empty_leaf_set,
                    Some(Ok(identities)) => identities,
                    Some(Err(gap)) => {
                        gap.note_as_a_part_of_the_range_left_unjudged(
                            judgements,
                            RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                        );
                        continue;
                    }
                };
            let leaf_identities = match leaf_identities {
                Ok(identities) => identities,
                Err(gap) => {
                    gap.note_as_a_part_of_the_range_left_unjudged(
                        judgements,
                        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                    );
                    continue;
                }
            };
            let retired_at_this_txg = match version
                .retired_at_this_txg_of_each_inode_tree
                .get(inode_tree)
            {
                None => &empty_leaf_set,
                Some(Ok(identities)) => identities,
                Some(Err(gap)) => {
                    gap.note_as_a_part_of_the_range_left_unjudged(
                        judgements,
                        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                    );
                    continue;
                }
            };
            let gone: BTreeSet<ContainerIdentityOfTheScan> = parent_leaf_identities
                .difference(leaf_identities)
                .copied()
                .collect();
            let missing: Vec<&ContainerIdentityOfTheScan> =
                gone.difference(retired_at_this_txg).collect();
            let extra: Vec<&ContainerIdentityOfTheScan> =
                retired_at_this_txg.difference(&gone).collect();
            judgements.judge(
                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                missing.is_empty() && extra.is_empty(),
                || {
                    format!(
                        "{}（txg {}）的 inode 树 {inode_tree} 比父版（txg {parent_txg}）少了的叶与死于这一版的退役记录对不上：漏记 {missing:?}，多记 {extra:?}",
                        version.name, version.checkpoint_txg
                    )
                },
            );
        }
    }
}

/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：最新的根下面没有走全了的记账树，或记账里没有 inode 号水位那一行。
pub const NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT: &str =
    "最新的根下面没有走全了的记账树、或记账里没有 inode 号水位那一行：没有台阶行可判";

/// 一版记账里 inode 号水位那一统计量的台阶行 (g, H)（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），从那一统计量
/// （不带设备维）的行 (代段, 值) 认：代段早于这一版 txg 的行都是台阶行；代段等于这一版 txg 的那一行（每次发布都写的那一行），值与代段最大的
/// 台阶行不同（或没有更早的台阶行）才是台阶行——这一版那次发布本身水位变了。按代段升序。与写路径同一条认法、按字段表另写一份，
/// 不用实现的读法。代段晚于这一版 txg 的行不认（I-3.13 判它）。
fn inode_number_watermark_steps_of_a_version(
    version_txg: u64,
    inode_number_watermark_rows: &[(u64, u64)],
) -> Vec<(u64, u64)> {
    let mut steps: Vec<(u64, u64)> = inode_number_watermark_rows
        .iter()
        .copied()
        .filter(|(row_generation, _)| *row_generation < version_txg)
        .collect();
    steps.sort_unstable();
    if let Some(&(_, value_of_this_version)) = inode_number_watermark_rows
        .iter()
        .find(|(row_generation, _)| *row_generation == version_txg)
    {
        if steps.last().is_none_or(|(_, latest_step_watermark)| {
            *latest_step_watermark != value_of_this_version
        }) {
            steps.push((version_txg, value_of_this_version));
        }
    }
    steps
}

/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态：D5（快照 / 空间记账机制） 已定项 2 射程、D18（块里携带什么信息）
/// 已定项 24 第 5 条）在最新的根上判，台阶行 (g, H) 取 [`inode_number_watermark_steps_of_a_version`]：
/// ① 记账里的 inode 号水位（那一统计量代段最大的那一行，D8（核心索引结构） 已定项 6「读现行值」）不小于每一级台阶的 H；
/// ② 最新根 inode 树里每条记录 (号 n, 对象出生代 b)、与最新根谱系上的每条墓碑 (n, b)：没有一级台阶 H > n 且 b > g
///    （号 n 在 g 之前就发过，晚于 g 出生的对象占着它就是同一条时间线上的重发）；
/// ④ 最新根谱系树里的每条谱系记录 (T_rb, T_old, H_cur)：代段 ≤ T_rb − 1 的台阶里最大的 H 等于 H_cur，没有这样的台阶判红；
///    H_cur 为 0 也判红——不另写一判：台阶的水位至少是第一个 inode 号 1（发布路径与只供测试的压回开关都不写 0），等式在 H_cur = 0 时不成立
///    （谱系接不成一条链时整份不信、报射程里判不了，同删除态扫描「在谱系上」那一判）。
/// 草案的 ③（水位大于初值时有一级台阶 H ≥ 水位、g ≤ 最新根的 txg）不判：台阶行与每次发布写的那一行同 key 维度、只差代段，
/// 那一行按上面的认法要么本身是台阶（H = 水位），要么前一级台阶的 H 就等于水位，③ 在单张镜像上恒成立。
/// 树表、谱系树、inode 树、墓碑树读不全的那一部分报射程里判不了（同 I-9.5 那一行的共用判法），已判的照报；
/// 只读不判、不进 `walk_failures`。
// TODO(D18（块里携带什么信息） 已定项 24 第 5 条「有克隆头时这一判与台阶行都按头」)：`crates/` 今天没有克隆路径，这一判不按树段分头；
// 做克隆时台阶行按 (统计量, 树) 分、记录与墓碑按头 ID 配那棵树的台阶。
pub(super) fn judge_inode_number_watermark_steps_of_the_newest_root(
    reader: &dyn ImageReader,
    tree_table_pointer_of_the_newest_root: &[u8],
    mapping_root_pointer_of_the_newest_root: &[u8],
    newest_txg: u64,
    inode_number_watermark_rows: &[(u64, u64)],
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant =
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE;
    let Some(&(_, inode_number_watermark)) = inode_number_watermark_rows
        .iter()
        .max_by_key(|(row_generation, _)| *row_generation)
    else {
        judgements.not_applicable(
            invariant,
            NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT,
        );
        return;
    };
    let steps = inode_number_watermark_steps_of_a_version(newest_txg, inode_number_watermark_rows);
    // ①
    for (step_txg, step_watermark) in &steps {
        judgements.judge(invariant, *step_watermark <= inode_number_watermark, || {
            format!(
                "最新根（txg {newest_txg}）记账里的 inode 号水位 {inode_number_watermark} 小于 txg {step_txg} 那一级台阶的水位 {step_watermark}"
            )
        });
    }
    let tree_table_pointer = parse_node_pointer(tree_table_pointer_of_the_newest_root);
    if tree_table_pointer.all_zero {
        DeletionScanGap::TreeTablePointerAllZero
            .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
        return;
    }
    let Some(tree_table) = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        index_cache,
    ) else {
        DeletionScanGap::TreeTableUnreadable
            .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
        return;
    };
    if tree_table
        .entries
        .iter()
        .any(|entry| entry.len() < tree_table_entry_bytes())
    {
        DeletionScanGap::TreeTableEntryShorterThanTheFieldTable
            .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
        return;
    }
    let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
        tree_table
            .entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == kind)
            .collect()
    };
    // 谱系树、inode 树、墓碑树里豁免三类之外的单元按最新根自己的中央映射认位置（读不出时是空表，照提示）。
    let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
        reader,
        &parse_node_pointer(mapping_root_pointer_of_the_newest_root),
        index_cache,
    );
    let mut packed_cache = PackedUnitCache::new();
    let lineage_records = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
        [lineage_entry] => lineage_records_without_judging(
            reader,
            &parse_node_pointer(&lineage_entry[14..100]),
            &central_mapping,
            index_cache,
            &mut packed_cache,
        ),
        [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
    };
    // ④：谱系记录读得出、接得成一条链（同删除态扫描「在谱系上」那一判的前提：接不成链的谱系整份不信）才判。
    let lineage = lineage_records.and_then(|records| {
        lineage_chain_of_a_version(newest_txg, &records)
            .map(|lineage_chain| (records, lineage_chain))
    });
    match &lineage {
        Ok((records, _)) => {
            for record in records {
                let watermark_rolled_back_from =
                    record.inode_number_watermark_of_the_version_rolled_back_from;
                let largest_step_before_the_rollback = steps
                    .iter()
                    .filter(|(step_txg, _)| *step_txg < record.rollback_txg)
                    .map(|(_, step_watermark)| *step_watermark)
                    .max();
                judgements.judge(
                    invariant,
                    largest_step_before_the_rollback == Some(watermark_rolled_back_from),
                    || {
                        format!(
                            "最新根（txg {newest_txg}）谱系记录 (T_rb {}, T_old {}) 带的 H_cur {watermark_rolled_back_from} 不等于代段 ≤ T_rb − 1 的台阶里最大的水位 {largest_step_before_the_rollback:?}（H_cur 为 0 也算不等）",
                            record.rollback_txg, record.parent_txg
                        )
                    },
                );
            }
        }
        Err(gap) => gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant),
    }
    // ②
    let step_the_number_was_issued_before = |number: u64, object_birth: u64| {
        steps
            .iter()
            .find(|(step_txg, step_watermark)| *step_watermark > number && object_birth > *step_txg)
            .copied()
    };
    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
        match inode_records_without_judging(
            reader,
            &parse_node_pointer(&inode_tree_entry[14..100]),
            &central_mapping,
            index_cache,
            &mut packed_cache,
        ) {
            Ok(records) => {
                for (inode, record) in &records {
                    let step = step_the_number_was_issued_before(*inode, record.object_birth);
                    judgements.judge(invariant, step.is_none(), || {
                        format!(
                            "最新根（txg {newest_txg}）inode 树里 {inode} 号的对象出生代 {}，晚于台阶 {step:?}（txg, 水位）：那一级台阶说这个号在那之前就发过",
                            record.object_birth
                        )
                    });
                }
            }
            Err(gap) => gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant),
        }
    }
    for tombstone_tree_entry in entries_of_kind(TREE_KIND_TOMBSTONE) {
        let containers = tombstone_containers_without_judging(
            reader,
            &parse_node_pointer(&tombstone_tree_entry[14..100]),
            &central_mapping,
            index_cache,
            &mut packed_cache,
        );
        match (containers, &lineage) {
            (Ok(containers), Ok((_, lineage_chain))) => {
                for tombstone in containers
                    .iter()
                    .flat_map(|container| container.records.iter())
                    .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
                {
                    let step =
                        step_the_number_was_issued_before(tombstone.object, tombstone.object_birth);
                    judgements.judge(invariant, step.is_none(), || {
                        format!(
                            "最新根（txg {newest_txg}）谱系上的墓碑（{} 号，对象出生代 {}，死于 txg {}）晚于台阶 {step:?}（txg, 水位）出生：那一级台阶说这个号在那之前就发过",
                            tombstone.object, tombstone.object_birth, tombstone.death_txg
                        )
                    });
                }
            }
            (Err(gap), _) => gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant),
            (Ok(_), Err(gap)) => {
                gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant)
            }
        }
    }
}

/// 墓碑与谱系那几条：删除态扫描那一份版本清单里
/// 每一版（树表与 txg 都相同的几版只扫一次——「在谱系上」从这一版自己的 txg 起算，txg 不同就各扫一遍）重读树表、谱系树、墓碑树、
/// inode 树与 `deleted_inodes`，只读不判、不进 `walk_failures`：判 I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22（看装没装旋钮）与 I-9.10 墓碑那一个等号，
/// 把各版的号 → 对象出生代收进一张跨版本的表，全部扫完判 I-9.29；候选集滤不掉被抛弃的根（[`AbandonedRootsFilteringOfTheCandidateSet`]）时
/// 那张表一个号都不比、I-9.29 报部分没判。读不全、配不上的报射程里判不了（同 I-9.5 那一行的共用判法）；
/// 不按已发布与否滤墓碑（第二节 T4）。I-9.10 墓碑那一个等号读不全、配不上时不记判不了，与它数据单元那一半同口径
/// （C701（别的判法静默跳过读不出的部分））。
pub(super) fn judge_tombstones_and_the_lineage_of_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionToScanTheDeletionStateOf<'_>],
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    container_capacity_knobs: ContainerCapacityKnobs,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let capacity_knob = container_capacity_knobs.tombstone;
    let lineage_and_retirement_capacity_invariants = [
        lineage_or_retirement_capacity_invariant_of(
            container_capacity_knobs
                .lineage
                .map(LineageContainerCapacityKnob::records_per_container),
        ),
        lineage_or_retirement_capacity_invariant_of(
            container_capacity_knobs
                .retirement
                .map(RetirementContainerCapacityKnob::records_per_container),
        ),
    ];
    let mut packed_cache = PackedUnitCache::new();
    let mut tree_tables_and_txgs_scanned: BTreeSet<(u32, u64, u32, u64)> = BTreeSet::new();
    let mut object_births: ObjectBirthsOfEachInodeNumber = BTreeMap::new();
    let mut versions_of_the_retirement_completeness: Vec<VersionOfTheRetirementCompleteness> =
        Vec::new();
    let capacity_invariant = capacity_invariant_of(capacity_knob);
    let unreadable_version = |version: &VersionToScanTheDeletionStateOf<'_>,
                              gap: DeletionScanGap| {
        VersionOfTheRetirementCompleteness {
            name: version.name.clone(),
            checkpoint_txg: version.checkpoint_txg,
            parent_txg: Err(gap),
            leaf_identities_of_each_inode_tree: BTreeMap::new(),
            retired_at_this_txg_of_each_inode_tree: BTreeMap::new(),
            unreadable: Some(gap),
        }
    };
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                .into_iter()
                .chain(RETIREMENT_SCAN_INVARIANTS)
                .chain([capacity_invariant])
                .chain(lineage_and_retirement_capacity_invariants)
            {
                DeletionScanGap::TreeTablePointerAllZero
                    .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
            versions_of_the_retirement_completeness.push(unreadable_version(
                version,
                DeletionScanGap::TreeTablePointerAllZero,
            ));
            continue;
        }
        let first_location = tree_table_pointer.locations[0];
        if !tree_tables_and_txgs_scanned.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
            version.checkpoint_txg,
        )) {
            continue;
        }
        // 树表读不出、条目比字段表窄：这一版的墓碑与谱系判不了；树表 0 条的一版没有这几棵树，不判。
        let tree_table = read_index_node_without_judging(
            reader,
            &tree_table_pointer,
            PositionOfASecondaryRead::ThePointerItself,
            index_cache,
        );
        let tree_table_gap = if let Some(tree_table_read_back) = &tree_table {
            tree_table_read_back
                .entries
                .iter()
                .any(|entry| entry.len() < tree_table_entry_bytes())
                .then_some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)
        } else {
            Some(DeletionScanGap::TreeTableUnreadable)
        };
        if let Some(gap) = tree_table_gap {
            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                .into_iter()
                .chain(RETIREMENT_SCAN_INVARIANTS)
                .chain([capacity_invariant])
                .chain(lineage_and_retirement_capacity_invariants)
            {
                gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
            versions_of_the_retirement_completeness.push(unreadable_version(version, gap));
            continue;
        }
        let Some(tree_table) = tree_table else {
            continue;
        };
        if tree_table.entries.is_empty() {
            // 树表 0 条的一版没有 inode 树：当父版时它每棵树的叶身份集都是空集。
            versions_of_the_retirement_completeness.push(VersionOfTheRetirementCompleteness {
                name: version.name.clone(),
                checkpoint_txg: version.checkpoint_txg,
                parent_txg: Ok(None),
                leaf_identities_of_each_inode_tree: BTreeMap::new(),
                retired_at_this_txg_of_each_inode_tree: BTreeMap::new(),
                unreadable: None,
            });
            continue;
        }
        // 这一版自己的中央映射：谱系、墓碑、inode、退役几棵树里豁免三类之外的单元按它认位置（读不出时是空表，照提示）。
        let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
            reader,
            &parse_node_pointer(version.mapping_root_pointer),
            index_cache,
        );
        versions_of_the_retirement_completeness.push(
            judge_tombstones_and_the_lineage_of_a_version(
                reader,
                version,
                &tree_table.entries,
                capacity_knob,
                container_capacity_knobs,
                &central_mapping,
                index_cache,
                &mut packed_cache,
                &mut object_births,
                judgements,
            ),
        );
    }
    judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version(
        &versions_of_the_retirement_completeness,
        abandoned_roots_filtering,
        judgements,
    );
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {
            judge_one_object_birth_per_inode_number(&object_births, judgements);
        }
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            judgements.partially_judged(
                ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                format!(
                    "最新根的实例表读不出（主走读沿链没走到末片：有一片读不出、解不开或链指针记录不合法）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 {} 个号一个都没比",
                    object_births.len()
                ),
            );
        }
    }
    // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
    judgements.not_applicable(
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
    );
    judgements.not_applicable(
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
        NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE,
    );
    judgements.not_applicable(
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
        NO_VERSION_WHOSE_PARENT_IS_IN_THE_CANDIDATE_SET,
    );
    match capacity_knob {
        None => {
            judgements.not_applicable(
                NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
                NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
            );
            judgements.not_applicable(
                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
                NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY,
            );
        }
        Some(_) => {
            judgements.not_applicable(
                NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
                PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
            );
            judgements.not_applicable(
                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
                NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
            );
        }
    }
    let (product_capacity_reason, knob_capacity_reason) = match (
        container_capacity_knobs.lineage,
        container_capacity_knobs.retirement,
    ) {
        (None, None) => (
            NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
            NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
        ),
        (Some(_), None) | (None, Some(_)) => (
            NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
            NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS,
        ),
        (Some(_), Some(_)) => (
            PRODUCT_LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOBS,
            NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS,
        ),
    };
    judgements.not_applicable(
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        product_capacity_reason,
    );
    judgements.not_applicable(
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
        knob_capacity_reason,
    );
}

/// [`judge_tombstones_and_the_lineage_of_the_versions`] 的一版：读谱系（父链）、每棵 inode 树的记录与每棵墓碑树，再逐条判。
#[allow(
    clippy::too_many_arguments,
    reason = "版本、树表条目、墓碑旋钮与三棵容器树的旋钮、两份缓存、跨版本那张表与判定累加器各有名字与类型，不为凑短签名收成结构体"
)]
fn judge_tombstones_and_the_lineage_of_a_version(
    reader: &dyn ImageReader,
    version: &VersionToScanTheDeletionStateOf<'_>,
    tree_table_entries: &[Vec<u8>],
    capacity_knob: Option<TombstoneContainerCapacityKnob>,
    container_capacity_knobs: ContainerCapacityKnobs,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    object_births: &mut ObjectBirthsOfEachInodeNumber,
    judgements: &mut Judgements,
) -> VersionOfTheRetirementCompleteness {
    let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
        tree_table_entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == kind)
            .collect()
    };
    // 一张树表恰好一条谱系树条目（I-9.16 ④）：这一版的父链从它的记录与这一版自己的 txg 起算。
    let lineage_tree = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
        [lineage_entry] => lineage_containers_without_judging(
            reader,
            &parse_node_pointer(&lineage_entry[14..100]),
            central_mapping,
            index_cache,
            packed_cache,
        )
        .map(|containers| (read_u64(lineage_entry, 0), containers)),
        [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
    };
    // I-9.36 / I-9.37 在谱系树上：读不全的报射程里判不了。
    let lineage_capacity_knob = container_capacity_knobs
        .lineage
        .map(LineageContainerCapacityKnob::records_per_container);
    match &lineage_tree {
        Ok((lineage_tree_identifier, containers)) => {
            judge_lineage_or_retirement_container_capacity_of_a_version(
                &version.name,
                LineageOrRetirementTree::Lineage,
                *lineage_tree_identifier,
                &containers
                    .iter()
                    .map(|container| (container.container, container.records.len()))
                    .collect::<Vec<(u64, usize)>>(),
                lineage_capacity_knob,
                judgements,
            );
        }
        Err(gap) => gap.note_as_a_part_of_the_range_left_unjudged(
            judgements,
            lineage_or_retirement_capacity_invariant_of(lineage_capacity_knob),
        ),
    }
    let lineage_records = lineage_tree.map(|(_, containers)| {
        containers
            .into_iter()
            .flat_map(|container| container.records)
            .collect::<Vec<LineageRecordOfTheScan>>()
    });
    let lineage_chain = lineage_records
        .clone()
        .and_then(|records| lineage_chain_of_a_version(version.checkpoint_txg, &records));
    // I-9.35 的父版：这一版谱系树里有 T_rb 等于这一版 txg 的记录时取它的 T_old，否则 txg − 1。
    let parent_txg = lineage_records.as_ref().map_err(|gap| *gap).map(|records| {
        records
            .iter()
            .find(|record| record.rollback_txg == version.checkpoint_txg)
            .map_or(version.checkpoint_txg.checked_sub(1), |record| {
                Some(record.parent_txg)
            })
    });
    let mut inode_trees: Vec<(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = Vec::new();
    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
        inode_trees.push((
            read_u64(inode_tree_entry, 0),
            inode_records_without_judging(
                reader,
                &parse_node_pointer(&inode_tree_entry[14..100]),
                central_mapping,
                index_cache,
                packed_cache,
            ),
        ));
    }
    let mut tombstone_trees: Vec<TombstoneTreeOfTheScan> = Vec::new();
    for tombstone_tree_entry in entries_of_kind(TREE_KIND_TOMBSTONE) {
        tombstone_trees.push(TombstoneTreeOfTheScan {
            tree: read_u64(tombstone_tree_entry, 0),
            head_identifier: read_u64(
                tombstone_tree_entry,
                TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
            ),
            containers: tombstone_containers_without_judging(
                reader,
                &parse_node_pointer(&tombstone_tree_entry[14..100]),
                central_mapping,
                index_cache,
                packed_cache,
            ),
        });
    }
    for (inode_tree, records) in &inode_trees {
        match records {
            Ok(records) => {
                for (inode, record) in records {
                    object_births
                        .entry((*inode_tree, *inode))
                        .or_default()
                        .entry(record.object_birth)
                        .or_insert_with(|| format!("{}的 inode 记录", version.name));
                }
            }
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                );
            }
        }
    }
    for tombstone_tree in &tombstone_trees {
        let containers = match &tombstone_tree.containers {
            Ok(containers) => containers,
            Err(gap) => {
                for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                    .into_iter()
                    .chain([capacity_invariant_of(capacity_knob)])
                {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
        };
        judge_tombstone_container_capacity_of_a_version(
            &version.name,
            tombstone_tree.tree,
            containers,
            capacity_knob,
            judgements,
        );
        judge_every_tombstone_of_a_version_is_well_formed(
            version,
            tombstone_tree.tree,
            containers,
            judgements,
        );
        match &lineage_chain {
            Ok(lineage_chain) => judge_tombstones_on_the_lineage_of_a_version_are_distinct(
                &version.name,
                tombstone_tree.tree,
                containers,
                lineage_chain,
                judgements,
            ),
            Err(gap) => gap.note_as_a_part_of_the_range_left_unjudged(
                judgements,
                TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
            ),
        }
        // 墓碑树经头 ID 配同一张表里树 ID 等于它的那一棵 inode 树（同 extent 树，I-9.27）：配不上、或那棵树的记录读不全的，墓碑对记录那几判判不了。
        let paired_inode_trees: Vec<&(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = inode_trees
            .iter()
            .filter(|(inode_tree, _)| *inode_tree == tombstone_tree.head_identifier)
            .collect();
        let records = match paired_inode_trees.as_slice() {
            [(_, Ok(records))] => records,
            [(_, Err(gap))] => {
                for invariant in [
                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                ] {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
            [] | [_, _, ..] => {
                for invariant in [
                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                ] {
                    DeletionScanGap::TombstoneTreeHeadIdentifierNamesNoSingleInodeTree
                        .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
        };
        judge_tombstones_against_the_object_births_of_the_records_by_number(
            &version.name,
            containers,
            records,
            judgements,
        );
        match &lineage_chain {
            Ok(lineage_chain) => {
                judge_tombstones_on_the_lineage_name_no_live_object(
                    &version.name,
                    containers,
                    records,
                    lineage_chain,
                    judgements,
                );
                for tombstone in containers
                    .iter()
                    .flat_map(|container| container.records.iter())
                    .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
                {
                    object_births
                        .entry((tombstone_tree.head_identifier, tombstone.object))
                        .or_default()
                        .entry(tombstone.object_birth)
                        .or_insert_with(|| {
                            format!(
                                "{}墓碑树里谱系上的墓碑（死于 txg {}）",
                                version.name, tombstone.death_txg
                            )
                        });
                }
            }
            Err(gap) => {
                for invariant in [
                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                ] {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
            }
        }
    }
    for (inode_tree, records) in &inode_trees {
        judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage(
            reader,
            version,
            tree_table_entries,
            *inode_tree,
            records,
            &tombstone_trees,
            &lineage_chain,
            central_mapping,
            index_cache,
            judgements,
        );
    }
    // 退役树那几条：I-9.32 每条记录都判；I-9.33、I-9.34 只判死亡 txg
    // 在这一版父链上的；I-9.35 要的那几样收下来，扫完各版再跨相邻两版比。
    let mut leaf_identities_of_each_inode_tree = BTreeMap::new();
    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
        leaf_identities_of_each_inode_tree.insert(
            read_u64(inode_tree_entry, 0),
            inode_leaf_identities_without_judging(
                reader,
                &parse_node_pointer(&inode_tree_entry[14..100]),
                central_mapping,
                index_cache,
            ),
        );
    }
    let mut retired_at_this_txg_of_each_inode_tree = BTreeMap::new();
    let retirement_capacity_knob = container_capacity_knobs
        .retirement
        .map(RetirementContainerCapacityKnob::records_per_container);
    for retirement_tree_entry in entries_of_kind(TREE_KIND_RETIREMENT) {
        let retirement_tree = RetirementTreeOfTheScan {
            tree: read_u64(retirement_tree_entry, 0),
            head_identifier: read_u64(
                retirement_tree_entry,
                TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
            ),
            containers: retirement_containers_without_judging(
                reader,
                &parse_node_pointer(&retirement_tree_entry[14..100]),
                central_mapping,
                index_cache,
                packed_cache,
            ),
        };
        let containers = match &retirement_tree.containers {
            Ok(containers) => containers,
            Err(gap) => {
                for invariant in RETIREMENT_SCAN_INVARIANTS.into_iter().chain([
                    lineage_or_retirement_capacity_invariant_of(retirement_capacity_knob),
                ]) {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                retired_at_this_txg_of_each_inode_tree
                    .insert(retirement_tree.head_identifier, Err(*gap));
                continue;
            }
        };
        judge_lineage_or_retirement_container_capacity_of_a_version(
            &version.name,
            LineageOrRetirementTree::Retirement,
            retirement_tree.tree,
            &containers
                .iter()
                .map(|container| (container.container, container.records.len()))
                .collect::<Vec<(u64, usize)>>(),
            retirement_capacity_knob,
            judgements,
        );
        judge_every_retirement_record_of_a_version_is_well_formed(
            version,
            retirement_tree.tree,
            containers,
            judgements,
        );
        retired_at_this_txg_of_each_inode_tree.insert(
            retirement_tree.head_identifier,
            Ok(containers
                .iter()
                .flat_map(|container| container.records.iter())
                .filter(|record| record.death_txg == version.checkpoint_txg)
                .map(|record| record.retired)
                .collect::<BTreeSet<ContainerIdentityOfTheScan>>()),
        );
        let lineage_chain = match &lineage_chain {
            Ok(lineage_chain) => lineage_chain,
            Err(gap) => {
                for invariant in [
                    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
                    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
                ] {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
        };
        judge_retirement_records_on_the_lineage_of_a_version_are_distinct(
            &version.name,
            retirement_tree.tree,
            containers,
            lineage_chain,
            judgements,
        );
        // 退役树经头 ID 配同一张表里树 ID 等于它的那一棵 inode 树（同墓碑树，I-9.27）：配不上、或那棵树读不全的，I-9.34 判不了。
        let paired_leaf_identities: Vec<
            &Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>,
        > = leaf_identities_of_each_inode_tree
            .iter()
            .filter(|(inode_tree, _)| **inode_tree == retirement_tree.head_identifier)
            .map(|(_, identities)| identities)
            .collect();
        match paired_leaf_identities.as_slice() {
            [Ok(leaf_identities)] => judge_retirement_records_on_the_lineage_name_no_live_leaf(
                &version.name,
                retirement_tree.tree,
                containers,
                leaf_identities,
                lineage_chain,
                judgements,
            ),
            [Err(gap)] => gap.note_as_a_part_of_the_range_left_unjudged(
                judgements,
                RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
            ),
            [] | [_, _, ..] => DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree
                .note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
                ),
        }
    }
    // 有一棵退役树的头 ID 配不上这张表里任何一棵 inode 树（I-9.27 判它）：它的记录算不到哪棵 inode 树头上，这一版每棵 inode 树的
    // 退役集合都判不了——I-9.35 在这一版报射程里判不了，不按「没有退役记录」判漏记。
    let some_retirement_tree_names_no_inode_tree = retired_at_this_txg_of_each_inode_tree
        .keys()
        .any(|head_identifier| !leaf_identities_of_each_inode_tree.contains_key(head_identifier));
    if some_retirement_tree_names_no_inode_tree {
        for inode_tree in leaf_identities_of_each_inode_tree.keys() {
            retired_at_this_txg_of_each_inode_tree.insert(
                *inode_tree,
                Err(DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree),
            );
        }
    }
    VersionOfTheRetirementCompleteness {
        name: version.name.clone(),
        checkpoint_txg: version.checkpoint_txg,
        parent_txg,
        leaf_identities_of_each_inode_tree,
        retired_at_this_txg_of_each_inode_tree,
        unreadable: None,
    }
}

/// I-9.18（删除队列里的号有谱系上的墓碑）在一版的一棵 inode 树上：经头 ID 配这棵树的 `deleted_inodes`（恰好一条）与墓碑树（恰好一条），
/// 队列里每个号 N 要在墓碑树里有 (N, N 的记录的对象出生代, 整对象) 且死亡 txg 在这一版的父链上；N 没有记录的那一形判不了、不判红
/// （`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.18 那一行：这一形只让 I-9.5 红）。队列空的一版没有对象。
#[allow(
    clippy::too_many_arguments,
    reason = "版本、树表条目、这棵 inode 树与它的记录、墓碑树、父链、缓存与判定累加器各有名字与类型，不为凑短签名收成结构体"
)]
fn judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage(
    reader: &dyn ImageReader,
    version: &VersionToScanTheDeletionStateOf<'_>,
    tree_table_entries: &[Vec<u8>],
    inode_tree: u64,
    records: &InodeRecordsOfAnInodeTreeOfTheScan,
    tombstone_trees: &[TombstoneTreeOfTheScan],
    lineage_chain: &Result<LineageChainOfAVersion, DeletionScanGap>,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant = QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE;
    let deleted_inodes_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| {
            read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DELETED_INODES
                && read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET) == inode_tree
        })
        .collect();
    let queue = match deleted_inodes_entries.as_slice() {
        [deleted_inodes_entry] => deleted_inodes_queue_without_judging(
            reader,
            &parse_node_pointer(&deleted_inodes_entry[14..100]),
            central_mapping,
            index_cache,
        ),
        [] | [_, _, ..] => Err(DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry),
    };
    let queue = match queue {
        Ok(queue) => queue,
        Err(gap) => {
            gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            return;
        }
    };
    if queue.is_empty() {
        return;
    }
    let records = match records {
        Ok(records) => records,
        Err(gap) => {
            gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            return;
        }
    };
    let paired_tombstone_trees: Vec<&TombstoneTreeOfTheScan> = tombstone_trees
        .iter()
        .filter(|tombstone_tree| tombstone_tree.head_identifier == inode_tree)
        .collect();
    let (tombstone_tree, containers) = match paired_tombstone_trees.as_slice() {
        [tombstone_tree] => match &tombstone_tree.containers {
            Ok(containers) => (tombstone_tree.tree, containers),
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                return;
            }
        },
        [] | [_, _, ..] => {
            DeletionScanGap::InodeTreeNamedByNoSingleTombstoneTreeEntry
                .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            return;
        }
    };
    let lineage_chain = match lineage_chain {
        Ok(lineage_chain) => lineage_chain,
        Err(gap) => {
            gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            return;
        }
    };
    for inode in &queue {
        let Some(record) = records.get(inode) else {
            DeletionScanGap::QueuedInodeWithoutARecord
                .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            continue;
        };
        let tombstones_of_the_inode: Vec<TombstoneRecordOfTheScan> = containers
            .iter()
            .flat_map(|container| container.records.iter())
            .filter(|tombstone| tombstone.object == *inode)
            .copied()
            .collect();
        let has_a_whole_object_tombstone_on_the_lineage =
            tombstones_of_the_inode.iter().any(|tombstone| {
                tombstone.object_birth == record.object_birth
                    && tombstone.covers_the_whole_object()
                    && lineage_chain.holds_txg(tombstone.death_txg)
            });
        judgements.judge(invariant, has_a_whole_object_tombstone_on_the_lineage, || {
            format!(
                "{}：deleted_inodes 里的号 {inode}（记录的对象出生代 {}）在墓碑树（树 {tombstone_tree}）里没有在谱系上的整对象墓碑；同号的墓碑 {tombstones_of_the_inode:?}",
                version.name, record.object_birth
            )
        });
    }
}

/// I-9.10（对象出生代三处一致） 墓碑那一个等号：这一版墓碑树里每一条墓碑按号认，这一版 inode 树里有这个号的记录时，
/// 墓碑的对象出生代 == 记录的。不看谱系（回退到删之前那一形的墓碑在谱系外，也比），删完之后记录不在、这一判没有对象。
fn judge_tombstones_against_the_object_births_of_the_records_by_number(
    version_name: &str,
    containers: &[TombstoneContainerOfTheScan],
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    judgements: &mut Judgements,
) {
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
    {
        let Some(record) = records.get(&tombstone.object) else {
            continue;
        };
        judgements.judge("I-9.10", record.object_birth == tombstone.object_birth, || {
            format!(
                "{version_name}：墓碑（号 {}、对象出生代 {}、死于 txg {}）与这一版 inode 记录的对象出生代 {} 不符",
                tombstone.object, tombstone.object_birth, tombstone.death_txg, record.object_birth
            )
        });
    }
}

/// I-9.19（谱系上的墓碑不指活对象）：这一版在谱系上的每条墓碑，这一版 inode 树里有号与对象出生代都相同的记录时，那条记录 nlink = 0。
fn judge_tombstones_on_the_lineage_name_no_live_object(
    version_name: &str,
    containers: &[TombstoneContainerOfTheScan],
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
    {
        let Some(record) = records
            .get(&tombstone.object)
            .filter(|record| record.object_birth == tombstone.object_birth)
        else {
            continue;
        };
        judgements.judge(
            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
            record.nlink == 0,
            || {
                format!(
                    "{version_name}：谱系上的墓碑（号 {}、对象出生代 {}、死于 txg {}）指着 nlink {} 的记录",
                    tombstone.object, tombstone.object_birth, tombstone.death_txg, record.nlink
                )
            },
        );
    }
}

/// I-9.20（谱系上的墓碑不重号且记录合法） 管全部墓碑的那几句：每条整对象 [0, 2⁶⁴ − 1]
/// （区间起 < 区间止是它的推论，照条款原文合取）、对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每片容器记录数 ≤ 815（与 I-1.7 重合，留作合取）。
fn judge_every_tombstone_of_a_version_is_well_formed(
    version: &VersionToScanTheDeletionStateOf<'_>,
    tombstone_tree: u64,
    containers: &[TombstoneContainerOfTheScan],
    judgements: &mut Judgements,
) {
    let invariant = TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED;
    let version_name = &version.name;
    let version_txg = version.checkpoint_txg;
    for container in containers {
        let record_count =
            u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
        judgements.judge(invariant, record_count <= TOMBSTONE_CONTAINER_RECORDS, || {
            format!(
                "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 装了 {record_count} 条，多于 {TOMBSTONE_CONTAINER_RECORDS}",
                container.container
            )
        });
        for tombstone in &container.records {
            judgements.judge(invariant, tombstone.covers_the_whole_object(), || {
                format!(
                    "{version_name}：墓碑（号 {}、对象出生代 {}）的区间 [{}, {}] 不是整对象 [0, 2⁶⁴ − 1]",
                    tombstone.object, tombstone.object_birth, tombstone.range_start, tombstone.range_end
                )
            });
            let death_txg_in_range =
                tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;
            judgements.judge(invariant, death_txg_in_range, || {
                format!(
                    "{version_name}：墓碑（号 {}、对象出生代 {}）的死亡 txg {} 不在 [对象出生代, 这一版的 txg {version_txg}] 里",
                    tombstone.object, tombstone.object_birth, tombstone.death_txg
                )
            });
        }
    }
}

/// I-9.20 不重号那一句（只判在谱系上的）：这一版在谱系上的墓碑按 (号, 对象出生代) 不重复。
fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(
    version_name: &str,
    tombstone_tree: u64,
    containers: &[TombstoneContainerOfTheScan],
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    let mut objects_on_the_lineage: BTreeSet<(u64, u64)> = BTreeSet::new();
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
    {
        let is_the_first_on_the_lineage =
            objects_on_the_lineage.insert((tombstone.object, tombstone.object_birth));
        judgements.judge(
            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
            is_the_first_on_the_lineage,
            || {
                format!(
                    "{version_name}：墓碑树（树 {tombstone_tree}）里在谱系上的墓碑 (号 {}、对象出生代 {}) 不止一条（又一条死于 txg {}）",
                    tombstone.object, tombstone.object_birth, tombstone.death_txg
                )
            },
        );
    }
}

/// I-9.36 / I-9.37，判法与 I-9.21 / I-9.22 同形：这一版谱系树或
/// 退役树按叶序的容器（容器号, 记录数），读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，装了至少一片）判容器号与记录数的三句
/// 沿叶序第一片容器号是 1、相邻两片加一、
/// 每片（含最右那片）至少一条——写侧只经两个追加函数开新片，新号 = 最右号 + 1、零根时 1，有新记录才开、没有删片的路
/// （D18（块里携带什么信息） 已定项 24 第 1、5 条）；再按旋钮：没装判 I-9.36——最右那片之外每片记录数 == 产品容量
/// （谱系 1359、退役 959；开放容器满了才关），零根或只一片容器没有对象；装了旋钮判 I-9.37——每片记录数 ≤ 旋钮值、最右那片之外每片记录数
/// == 旋钮值（满了才关在旋钮下的那一半，D18（块里携带什么信息） 已定项 24 第 3 条「满了才关；关闭路径加只供测试的容量旋钮」），
/// 零根没有对象。
fn judge_lineage_or_retirement_container_capacity_of_a_version(
    version_name: &str,
    tree_kind: LineageOrRetirementTree,
    tree: u64,
    container_numbers_and_record_counts: &[(u64, usize)],
    knob_records_per_container: Option<u64>,
    judgements: &mut Judgements,
) {
    let invariant = lineage_or_retirement_capacity_invariant_of(knob_records_per_container);
    let tree_name = tree_kind.name();
    let judged_when_the_tree_has_at_least = match knob_records_per_container {
        None => 2,
        Some(_) => 1,
    };
    if container_numbers_and_record_counts.len() < judged_when_the_tree_has_at_least {
        return;
    }
    if let Some((first_container_in_leaf_order, _)) = container_numbers_and_record_counts.first() {
        let first_container_in_leaf_order = *first_container_in_leaf_order;
        judgements.judge(
            invariant,
            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,
            || {
                format!(
                    "{version_name}：{tree_name}（树 {tree}）沿叶序第一片容器的容器号是 {first_container_in_leaf_order}，不是 {FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE}（容器号从 1 起）"
                )
            },
        );
    }
    for adjacent in container_numbers_and_record_counts.windows(2) {
        let (left_container, right_container) = (adjacent[0].0, adjacent[1].0);
        let right_container_follows_the_left_by_one =
            left_container.checked_add(1) == Some(right_container);
        judgements.judge(invariant, right_container_follows_the_left_by_one, || {
            format!(
                "{version_name}：{tree_name}（树 {tree}）沿叶序容器号不是逐片加一：容器 {left_container} 之后是容器 {right_container}（下一个号 = 最右号 + 1）"
            )
        });
    }
    for (position_in_leaf_order, (container, record_count)) in
        container_numbers_and_record_counts.iter().enumerate()
    {
        let record_count = u64::try_from(*record_count).expect("一片容器的记录数装得进 u64");
        let lineage_or_retirement_container_holds_a_record = record_count > 0;
        judgements.judge(invariant, lineage_or_retirement_container_holds_a_record, || {
            format!(
                "{version_name}：{tree_name}（树 {tree}）的容器 {container} 一条记录都没有（有新记录才开新片）"
            )
        });
        match knob_records_per_container {
            None => {
                let is_the_rightmost =
                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();
                if is_the_rightmost {
                    continue;
                }
                let product_records = tree_kind.product_records_per_container();
                judgements.judge(invariant, record_count == product_records, || {
                    format!(
                        "{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是 {product_records}（满了才关）"
                    )
                });
            }
            Some(records_the_knob_allows) => {
                judgements.judge(invariant, record_count <= records_the_knob_allows, || {
                    format!(
                        "{version_name}：{tree_name}（树 {tree}）的容器 {container} 装了 {record_count} 条，多于旋钮值 {records_the_knob_allows}"
                    )
                });
                let is_the_rightmost_under_the_knob =
                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();
                if is_the_rightmost_under_the_knob {
                    continue;
                }
                let non_rightmost_container_holds_the_knob_value =
                    record_count == records_the_knob_allows;
                judgements.judge(
                    invariant,
                    non_rightmost_container_holds_the_knob_value,
                    || {
                        format!(
                            "{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是旋钮值 {records_the_knob_allows}（满了才关）"
                        )
                    },
                );
            }
        }
    }
}

/// I-9.21 / I-9.22：这一版墓碑树按叶序的容器，读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，
/// 装了至少一片）判容器号与记录数的三句（同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]）：沿叶序第一片容器号是 1、
/// 相邻两片加一、每片（含最右那片）至少一条；再按旋钮：没装判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；
/// 装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值、最右那片之外每片记录数 == 旋钮值（满了才关在旋钮下的那一半，
/// 同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]），零根没有对象。
fn judge_tombstone_container_capacity_of_a_version(
    version_name: &str,
    tombstone_tree: u64,
    containers: &[TombstoneContainerOfTheScan],
    capacity_knob: Option<TombstoneContainerCapacityKnob>,
    judgements: &mut Judgements,
) {
    let invariant = capacity_invariant_of(capacity_knob);
    let judged_when_the_tree_has_at_least = match capacity_knob {
        None => 2,
        Some(_) => 1,
    };
    if containers.len() < judged_when_the_tree_has_at_least {
        return;
    }
    if let Some(first_in_leaf_order) = containers.first() {
        judgements.judge(
            invariant,
            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,
            || {
                format!(
                    "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序第一片容器的容器号是 {}，不是 {FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE}（容器号从 1 起）",
                    first_in_leaf_order.container
                )
            },
        );
    }
    for adjacent in containers.windows(2) {
        let (left, right) = (&adjacent[0], &adjacent[1]);
        let right_tombstone_container_follows_the_left_by_one =
            left.container.checked_add(1) == Some(right.container);
        judgements.judge(
            invariant,
            right_tombstone_container_follows_the_left_by_one,
            || {
                format!(
                    "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序容器号不是逐片加一：容器 {} 之后是容器 {}（下一个号 = 最右号 + 1）",
                    left.container, right.container
                )
            },
        );
    }
    for (position_in_leaf_order, container) in containers.iter().enumerate() {
        let record_count =
            u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
        let tombstone_container_holds_a_record = record_count > 0;
        judgements.judge(invariant, tombstone_container_holds_a_record, || {
            format!(
                "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 一条记录都没有（有新记录才开新片）",
                container.container
            )
        });
        match capacity_knob {
            None => {
                let is_the_rightmost = position_in_leaf_order + 1 == containers.len();
                if is_the_rightmost {
                    continue;
                }
                judgements.judge(invariant, record_count == TOMBSTONE_CONTAINER_RECORDS, || {
                    format!(
                        "{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是 {TOMBSTONE_CONTAINER_RECORDS}（满了才关）",
                        container.container
                    )
                });
            }
            Some(knob) => {
                let knob_records = knob.records_per_container();
                judgements.judge(invariant, record_count <= knob_records, || {
                    format!(
                        "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 装了 {record_count} 条，多于旋钮值 {knob_records}",
                        container.container
                    )
                });
                let is_the_rightmost_tombstone_container_under_the_knob =
                    position_in_leaf_order + 1 == containers.len();
                if is_the_rightmost_tombstone_container_under_the_knob {
                    continue;
                }
                let non_rightmost_tombstone_container_holds_the_knob_value =
                    record_count == knob_records;
                judgements.judge(
                    invariant,
                    non_rightmost_tombstone_container_holds_the_knob_value,
                    || {
                        format!(
                            "{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是旋钮值 {knob_records}（满了才关）",
                            container.container
                        )
                    },
                );
            }
        }
    }
}

/// I-9.29（同号只许一个出生代）：各版的 inode 记录与各版自己谱系上的墓碑合起来，同一棵 inode 树里每个号只对应一个对象出生代
/// 每个号判一次。
fn judge_one_object_birth_per_inode_number(
    object_births: &ObjectBirthsOfEachInodeNumber,
    judgements: &mut Judgements,
) {
    for ((inode_tree, inode), births) in object_births {
        judgements.judge(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, births.len() == 1, || {
            let where_each_birth_was_seen: Vec<String> = births
                .iter()
                .map(|(object_birth, first_seen)| format!("对象出生代 {object_birth}（{first_seen}）"))
                .collect();
            format!(
                "inode 树（树 {inode_tree}）里 inode {inode} 在回退候选集里对应不止一个对象出生代：{}",
                where_each_birth_was_seen.join("、")
            )
        });
    }
}

#[cfg(test)]
mod tests;
