//! 墓碑树、谱系树与退役树只读不判地读出来：容量旋钮、容器、墓碑记录、谱系记录与退役记录。

use super::*;

/// I-9.6 墓碑那一半：所选根（最新的那条根）的树表里墓碑树（种类 10）整棵读、不判，交回已发布的（按所选根指着的实例表判，
/// [`PublishedPredicate`]）墓碑容器里全部记录的对象 ID 最大值，一条都没有时 `Ok(None)`。被崩溃恢复抛弃的时间线上写的墓碑
/// 不在所选根的墓碑树里（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.6 那一行）。墓碑树根之下：类型段 1 的条目指码 3 墓碑容器，
/// 类型段 0 的指下一层码 2 节点（层级 = 父层级 − 1，且 ≥ 1）；读不全交回 [`DeletionScanGap::TombstoneTreeUnreadable`]。
pub(super) fn largest_published_tombstone_object_identifier(
    reader: &dyn ImageReader,
    tree_table_pointer_of_the_chosen_root: &[u8],
    mapping_root_pointer_of_the_chosen_root: &[u8],
    published: &PublishedPredicate<'_>,
    index_cache: &mut IndexNodeCache,
) -> Result<Option<u64>, DeletionScanGap> {
    let tree_table_pointer = parse_node_pointer(tree_table_pointer_of_the_chosen_root);
    if tree_table_pointer.all_zero {
        return Ok(None);
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        index_cache,
    )
    .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
    if tree_table
        .entries
        .iter()
        .any(|entry| entry.len() < tree_table_entry_bytes())
    {
        return Err(DeletionScanGap::TombstoneTreeUnreadable);
    }
    // 墓碑树的节点与墓碑容器不在豁免三类里：按所选根自己的中央映射认位置（读不出时是空表，照提示）。
    let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
        reader,
        &parse_node_pointer(mapping_root_pointer_of_the_chosen_root),
        index_cache,
    );
    let central_mapping = &central_mapping;
    let mut packed_cache = PackedUnitCache::new();
    let mut largest_object_identifier: Option<u64> = None;
    let mut pending_nodes: Vec<PointerView> = tree_table
        .entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_TOMBSTONE)
        .map(|entry| parse_node_pointer(&entry[14..100]))
        .filter(|root_pointer| !root_pointer.all_zero)
        .collect();
    let tombstone_record_bytes =
        registered_record_width(PACKED_TYPE_TOMBSTONE).expect("类型 1 登记了记录宽 40");
    // 迭代上界：每个码 2 节点只从它的父条目入队一次，类型段 0 的孩子层级比父低一、层级 0 不入队，所以入队的节点数有限。
    while let Some(node_pointer) = pending_nodes.pop() {
        let node = read_index_node_without_judging(
            reader,
            &node_pointer,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            index_cache,
        )
        .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
        if node.level == 0 || node.entry_width < inode_internal_entry_bytes() {
            return Err(DeletionScanGap::TombstoneTreeUnreadable);
        }
        for entry in &node.entries {
            let child_pointer = parse_node_pointer(&entry[34..120]);
            match read_u16(entry, 16) {
                0 => {
                    let child_level = read_index_node_without_judging(
                        reader,
                        &child_pointer,
                        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
                        index_cache,
                    )
                    .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?
                    .level;
                    if node.level.checked_sub(1) != Some(child_level) || child_level == 0 {
                        return Err(DeletionScanGap::TombstoneTreeUnreadable);
                    }
                    pending_nodes.push(child_pointer);
                }
                PACKED_TYPE_TOMBSTONE => {
                    let container = read_packed_unit_without_judging(
                        reader,
                        &child_pointer,
                        central_mapping,
                        &mut packed_cache,
                    )
                    .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
                    if container.view.record_type != PACKED_TYPE_TOMBSTONE
                        || container.view.record_width < tombstone_record_bytes
                    {
                        return Err(DeletionScanGap::TombstoneTreeUnreadable);
                    }
                    let is_published = published.holds_for(
                        WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                            birth_txg: container.view.birth_txg,
                            instance: container.write_order_instance,
                        },
                    );
                    if !is_published {
                        continue;
                    }
                    for record in &container.view.records {
                        let object_identifier =
                            read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
                        largest_object_identifier = Some(
                            largest_object_identifier.map_or(object_identifier, |largest| {
                                largest.max(object_identifier)
                            }),
                        );
                    }
                }
                _type_segment_outside_the_tombstone_tree => {
                    return Err(DeletionScanGap::TombstoneTreeUnreadable);
                }
            }
        }
    }
    Ok(largest_object_identifier)
}

/// 墓碑记录里死亡 txg 的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8；
/// 前四段的偏移住 crate 根，扫描重建也读它们），按字段表另写一份、不用实现的解析。
const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
/// 谱系记录 24 字节：T_rb 8（回退那次发布的 txg）+ T_old 8（回退目标的 txg）+ H_cur 8（cur，即回退之前的现行那一版的 inode 号水位），
/// 读作「T_rb 的父是 T_old」（D18（块里携带什么信息） 已定项 24 第 5 条；三段的逐字节次序条款没定，实现取 T_rb | T_old | H_cur，按字段表另写一份、不用实现的解析）。
const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
const LINEAGE_RECORD_PARENT_TXG_OFFSET: usize = 8;
const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;

/// 只供测试的墓碑容器容量旋钮压到的每片记录数（D18（块里携带什么信息） 已定项 24 第 3 条：旋钮不落盘，池级 checker 从镜像外拿；
///  取 K1）。取值 1..=815，构造时验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TombstoneContainerCapacityKnob {
    pub(super) records_per_container: u64,
}

impl TombstoneContainerCapacityKnob {
    /// 1..=815 之外交回 `None`（815 = ⌊(32768 − 136) ÷ 40⌋，产品容量 `TOMBSTONE_CONTAINER_RECORDS`）。
    #[must_use]
    pub fn new(records_per_container: u64) -> Option<Self> {
        (1..=TOMBSTONE_CONTAINER_RECORDS)
            .contains(&records_per_container)
            .then_some(Self {
                records_per_container,
            })
    }

    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        self.records_per_container
    }
}

/// 只供测试的谱系容器容量旋钮压到的每片记录数（同墓碑容器那一个：旋钮不落盘，池级 checker 从镜像外拿；判决
/// `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，I-9.37）。取值 1..=1359，构造时验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineageContainerCapacityKnob {
    pub(super) records_per_container: u64,
}

impl LineageContainerCapacityKnob {
    /// 1..=1359 之外交回 `None`（1359 = ⌊(32768 − 136) ÷ 24⌋，产品容量 `LINEAGE_CONTAINER_RECORDS`）。
    #[must_use]
    pub fn new(records_per_container: u64) -> Option<Self> {
        (1..=LINEAGE_CONTAINER_RECORDS)
            .contains(&records_per_container)
            .then_some(Self {
                records_per_container,
            })
    }

    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        self.records_per_container
    }
}

/// 只供测试的退役容器容量旋钮压到的每片记录数（同上，I-9.37）。取值 1..=959，构造时验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetirementContainerCapacityKnob {
    pub(super) records_per_container: u64,
}

impl RetirementContainerCapacityKnob {
    /// 1..=959 之外交回 `None`（959 = ⌊(32768 − 136) ÷ 34⌋，产品容量 `RETIREMENT_CONTAINER_RECORDS`）。
    #[must_use]
    pub fn new(records_per_container: u64) -> Option<Self> {
        (1..=RETIREMENT_CONTAINER_RECORDS)
            .contains(&records_per_container)
            .then_some(Self {
                records_per_container,
            })
    }

    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        self.records_per_container
    }
}

/// 写这张镜像时三棵容器树各压没压容量（从镜像外给）：`None` 的那一棵当没装旋钮，照产品容量判（墓碑树 I-9.21，谱系树、退役树 I-9.36）；
/// `Some` 的那一棵按旋钮值判（墓碑树 I-9.22，谱系树、退役树 I-9.37）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerCapacityKnobs {
    pub tombstone: Option<TombstoneContainerCapacityKnob>,
    pub lineage: Option<LineageContainerCapacityKnob>,
    pub retirement: Option<RetirementContainerCapacityKnob>,
}

impl ContainerCapacityKnobs {
    /// 三棵都没装旋钮（产品镜像，[`check_pool_image`] 那一份）。
    pub const PRODUCT: Self = Self {
        tombstone: None,
        lineage: None,
        retirement: None,
    };
}

/// 删除线新九条（I-9.17–I-9.23、I-9.29；I-9.24 的在 [`judge_deletion_state_of_the_versions`] 末尾）一次都没判到时各自报的不适用理由：
/// 每条一句、说清这张镜像上为什么没有它判的对象，不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」。
/// 公开给「逐条判定、不在豁免里的一律要求成立」的用例逐字认（豁免只认这一句，别的不适用照红）。
/// I-9.18（删除队列里的号有谱系上的墓碑）：每一版的队列都是空的。
pub const NO_QUEUED_INODE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判";
/// I-9.19（谱系上的墓碑不指活对象）：没有一条谱系上的墓碑在同一版 inode 树里有同号同对象出生代的记录。
pub const NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD: &str =
    "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑";
/// I-9.20（谱系上的墓碑不重号且记录合法）：每一版墓碑树都是零根。
pub const NO_TOMBSTONE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑记录可判";
/// I-9.29（同号只许一个出生代）：没有一版收到号（inode 记录与谱系上的墓碑都没有）。
pub const NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE: &str =
    "走读到的每一版里都没有 inode 记录与谱系上的墓碑：没有号可比";
/// I-9.21（产品镜像里非最右墓碑容器装满），旧入口：没有非最右的容器。
pub const NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER: &str =
    "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器";
/// I-9.22（测试镜像里墓碑容器不超旋钮值），旧入口：当没装旋钮，这一条不判。
pub const NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY: &str =
    "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判";
/// I-9.21，新入口给了旋钮值：产品容量那一条不判。
pub const PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB: &str =
    "这张镜像用了只供测试的墓碑容量旋钮（新入口给了旋钮值）：产品容量那一条不判";
/// I-9.22，新入口给了旋钮值：每一版墓碑树都是零根。
pub const NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑容器可判";
/// I-9.36（产品镜像里非最右谱系与退役容器装满），没装旋钮的那一棵（或两棵）上：每一版都是零根或只一片容器。
pub const NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER: &str =
    "走读到的每一版里没装旋钮的谱系树与退役树都是零根或只一片容器：没有非最右的容器";
/// I-9.36，谱系树与退役树都装了旋钮（新入口给了两个旋钮值）：产品容量那一条不判。
pub const PRODUCT_LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOBS: &str =
    "这张镜像的谱系与退役容量都用了只供测试的旋钮（新入口给了两个旋钮值）：产品容量那一条不判";
/// I-9.37（测试镜像里谱系与退役容器不超旋钮值），两棵都没装旋钮（旧入口，或新入口没给这两个旋钮值）。
pub const NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB: &str =
    "池级 checker 没拿到只供测试的谱系与退役容量旋钮（旧入口，或新入口没给这两个旋钮值）：旋钮那一条不判";
/// I-9.37，装了旋钮的那一棵（或两棵）在每一版都是零根。
pub const NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS: &str =
    "走读到的每一版里装了旋钮的谱系树与退役树都是零根：没有容器可判";
/// I-9.17（谱系树条目身份与子头相符）：主走读一条谱系树内部条目都没走到。
pub const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
/// I-9.23（墓碑树条目身份与子头相符）：主走读一条墓碑树内部条目都没走到。
pub const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";

/// 退役记录那几条（I-9.32–I-9.35）在一版上的那一组：一版的树表读不出、退役树读不全时这几条在那一版判不了。
pub(super) const RETIREMENT_SCAN_INVARIANTS: [&str; 4] = [
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
];
/// I-9.32：走读到的每一版退役树都是零根。
pub const NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役记录可判";
/// I-9.33、I-9.34：没有一条退役记录的死亡 txg 落在它那一版的父链上。
pub const NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版里没有一条退役记录在谱系上：没有对象可判";
/// I-9.35：没有一版的父版在回退候选集里读得出。
pub const NO_VERSION_WHOSE_PARENT_IS_IN_THE_CANDIDATE_SET: &str =
    "走读到的每一版的父版都不在回退候选集里（或都没有 inode 树）：没有相邻两版可比";
/// I-9.31（退役树条目身份与子头相符）：主走读一条退役树内部条目都没走到。
pub const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役树内部条目可判";

/// 墓碑与谱系那一遍在一版上判 I-9.18、I-9.19、I-9.20、I-9.29 要的那一组：一版的树表读不出、墓碑树读不全时这几条在那一版判不了
/// （I-9.21 / I-9.22 只记装没装旋钮的那一条，[`capacity_invariant_of`]）。
pub(super) const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
];

/// 回退候选集滤没滤得掉被崩溃恢复抛弃的根：候选集按最新根指着的实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14 回退段的候选集规则），
/// 那张表读不全时滤不全，被抛弃时间线上的根可能进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
/// （`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）：被抛弃的线上同号可以是另一个对象。
/// 读没读全看的是候选集滤法用的那一遍（主走读，`Walk::walk_instance_table_chain`），不另读一遍：另一个读法（只读不判的
/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（载荷 CRC 对不上；I-3.14 的五形两边一样当解不开），
/// 用它当门会在候选集已经滤全时报判不了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AbandonedRootsFilteringOfTheCandidateSet {
    /// 主走读沿最新根的实例表链走到了末片（链指针记录说「没有下一片」）：整张表的行都收下了，被抛弃的根都按它滤掉了。
    FilteredByTheNewestRootsInstanceTable,
    /// 主走读沿最新根的实例表链没走到末片（一片读不出、解不开、身份不对、链指针记录不合法、链成了环）：被抛弃的根可能留在候选集里。
    UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
}

/// 墓碑树、谱系树、退役树沿叶序第一片容器的容器号（D18（块里携带什么信息） 已定项 24 第 1、5 条：容器号从 1 起，下一个号 = 树里最大号 + 1，
/// 零根时 1；I-9.21、I-9.22、I-9.36、I-9.37 判它）。
pub(super) const FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE: u64 = 1;

/// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。
pub(super) const fn capacity_invariant_of(
    capacity_knob: Option<TombstoneContainerCapacityKnob>,
) -> &'static str {
    match capacity_knob {
        None => NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        Some(_) => TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    }
}

/// I-9.36 / I-9.37 判的那两棵容器树：说明里怎么称呼、产品容量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LineageOrRetirementTree {
    Lineage,
    Retirement,
}

impl LineageOrRetirementTree {
    pub(super) const fn name(self) -> &'static str {
        match self {
            LineageOrRetirementTree::Lineage => "谱系树",
            LineageOrRetirementTree::Retirement => "退役树",
        }
    }

    /// 一片码 3 容器的产品容量（D18（块里携带什么信息） 已定项 24：开放容器满了才关）。
    pub(super) const fn product_records_per_container(self) -> u64 {
        match self {
            LineageOrRetirementTree::Lineage => LINEAGE_CONTAINER_RECORDS,
            LineageOrRetirementTree::Retirement => RETIREMENT_CONTAINER_RECORDS,
        }
    }
}

/// 一棵谱系树或退役树装没装旋钮（旋钮值，每片记录数）决定它判哪一条：没装判产品那一条 I-9.36，装了判测试那一条 I-9.37（同 [`capacity_invariant_of`]）。
pub(super) const fn lineage_or_retirement_capacity_invariant_of(
    knob_records_per_container: Option<u64>,
) -> &'static str {
    match knob_records_per_container {
        None => NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        Some(_) => LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    }
}

/// 一条墓碑记录的五段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TombstoneRecordOfTheScan {
    pub(super) object: u64,
    pub(super) object_birth: u64,
    pub(super) range_start: u64,
    pub(super) range_end: u64,
    pub(super) death_txg: u64,
}

impl TombstoneRecordOfTheScan {
    pub(super) fn of(record: &[u8]) -> Self {
        Self {
            object: read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
            object_birth: read_u64(record, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
            range_start: read_u64(record, TOMBSTONE_RECORD_RANGE_START_OFFSET),
            range_end: read_u64(record, TOMBSTONE_RECORD_RANGE_END_OFFSET),
            death_txg: read_u64(record, TOMBSTONE_RECORD_DEATH_TXG_OFFSET),
        }
    }

    pub(super) fn covers_the_whole_object(self) -> bool {
        self.range_start < self.range_end
            && self.range_start == TOMBSTONE_WHOLE_OBJECT_RANGE_START
            && self.range_end == TOMBSTONE_WHOLE_OBJECT_RANGE_END
    }
}

/// 一片墓碑容器：头里的容器号（I-9.21 / I-9.22 沿叶序比它，不沿根的条目读：根长高之后根的条目是类型段 0、容器号恒 0）与它的记录。
pub(super) struct TombstoneContainerOfTheScan {
    pub(super) container: u64,
    pub(super) records: Vec<TombstoneRecordOfTheScan>,
}

/// 一版树表里的一棵墓碑树：树 ID、头 ID（I-9.27：等于这张表里那棵 inode 树的树 ID）与按叶序读出的容器（读不全交回判不了的理由）。
pub(super) struct TombstoneTreeOfTheScan {
    pub(super) tree: u64,
    pub(super) head_identifier: u64,
    pub(super) containers: Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap>,
}

/// 一条谱系记录：T_rb 的父是 T_old；H_cur 是回退之前的现行那一版的 inode 号水位。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LineageRecordOfTheScan {
    pub(super) rollback_txg: u64,
    pub(super) parent_txg: u64,
    pub(super) inode_number_watermark_of_the_version_rolled_back_from: u64,
}

impl LineageRecordOfTheScan {
    pub(super) fn of(record: &[u8]) -> Self {
        Self {
            rollback_txg: read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
            parent_txg: read_u64(record, LINEAGE_RECORD_PARENT_TXG_OFFSET),
            inode_number_watermark_of_the_version_rolled_back_from: read_u64(
                record,
                LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET,
            ),
        }
    }
}

/// 一版的父链在 txg 轴上占的那几段（每段闭区间）：「在谱系上」= 死亡 txg 落在其中一段里。
pub(super) struct LineageChainOfAVersion {
    segments_on_the_chain: Vec<(u64, u64)>,
}

impl LineageChainOfAVersion {
    pub(super) fn holds_txg(&self, txg: u64) -> bool {
        self.segments_on_the_chain
            .iter()
            .any(|(lowest_txg, highest_txg)| (*lowest_txg..=*highest_txg).contains(&txg))
    }
}

/// 一版里一棵 inode 树的全部记录（号 → 记录），读不全交回判不了的理由。
pub(super) type InodeRecordsOfAnInodeTreeOfTheScan =
    Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap>;

/// 跨版本那张表（I-9.29）：(inode 树的树 ID, inode 号) → 对象出生代 → 第一次在哪一版的什么地方见到它。
pub(super) type ObjectBirthsOfEachInodeNumber = BTreeMap<(u64, u64), BTreeMap<u64, String>>;

/// 「在谱系上」的父链：从这一版自己的 txg 起、
/// 按这一版树表里谱系树的记录往回走——有谱系记录 (T_rb, T_old) 的那个 txg 的父是 T_old，其余 txg 的父是 txg − 1（I-9.18 那一行）。
/// 记录按 T_rb 从大到小过一遍：T_rb 大于此刻走到的 txg 的不在这条链上（写在被后来的回退抛弃的分支上，或晚于这一版），跳过；
/// 否则 [T_rb, 此刻] 整段在链上、跳到 T_old。最后 [0, 此刻] 也在链上。按段走是 O(L log L)，不按 txg 一步步走。
/// T_old ≥ T_rb、同一个 T_rb 两条的谱系接不成一条链，交回 [`DeletionScanGap::LineageRecordsInconsistent`]。
pub(super) fn lineage_chain_of_a_version(
    version_txg: u64,
    lineage_records: &[LineageRecordOfTheScan],
) -> Result<LineageChainOfAVersion, DeletionScanGap> {
    let mut records_with_the_latest_rollback_first = lineage_records.to_vec();
    records_with_the_latest_rollback_first
        .sort_unstable_by_key(|record| std::cmp::Reverse(record.rollback_txg));
    let some_record_is_not_younger_than_its_parent = records_with_the_latest_rollback_first
        .iter()
        .any(|record| record.parent_txg >= record.rollback_txg);
    let some_rollback_has_two_records = records_with_the_latest_rollback_first
        .windows(2)
        .any(|adjacent| adjacent[0].rollback_txg == adjacent[1].rollback_txg);
    if some_record_is_not_younger_than_its_parent || some_rollback_has_two_records {
        return Err(DeletionScanGap::LineageRecordsInconsistent);
    }
    let mut segments_on_the_chain = Vec::new();
    let mut txg_walked_to = version_txg;
    // 迭代上界是谱系记录条数；跨轮携带的是此刻走到的 txg，每接上一段就跳到更小的 T_old（上面判过 T_old < T_rb ≤ 此刻），只减不增。
    for record in &records_with_the_latest_rollback_first {
        if record.rollback_txg > txg_walked_to {
            continue;
        }
        segments_on_the_chain.push((record.rollback_txg, txg_walked_to));
        txg_walked_to = record.parent_txg;
    }
    segments_on_the_chain.push((0, txg_walked_to));
    Ok(LineageChainOfAVersion {
        segments_on_the_chain,
    })
}

/// 墓碑树或谱系树（根恒码 2、条目 120 字节，类型段 0 指下一层码 2 节点、类型段 = `record_type` 指码 3 容器）整棵读、不判，
/// 按叶序交回每片容器；节点层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的那一个都交回 `gap`
/// （读不出、形状不对由主走读判：I-2.1、I-1.7、I-9.17、I-9.23 等，这一遍只如实记判不了——I-1.7 的坏镜像因此只红 I-1.7）。
/// 根指针全零（零根）交回空。
fn containers_of_a_container_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    record_type: u16,
    gap: DeletionScanGap,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<crate::PackedUnitView>, DeletionScanGap> {
    let mut containers = Vec::new();
    if root_pointer.all_zero {
        return Ok(containers);
    }
    let root = read_index_node_without_judging(
        reader,
        root_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        index_cache,
    )
    .ok_or(gap)?;
    collect_containers_of_a_container_tree_below(
        reader,
        &root,
        record_type,
        gap,
        central_mapping,
        index_cache,
        packed_cache,
        &mut containers,
    )?;
    Ok(containers)
}

/// [`containers_of_a_container_tree_without_judging`] 的一个码 2 节点：条目逐条，类型段 0 的往下一层（层级 = 父层级 − 1 且 ≥ 1），
/// 容器按条目次序追加进 `containers`。递归的上界：每下一层层级减一。
#[allow(
    clippy::too_many_arguments,
    reason = "节点、打包记录类型、读不全时的缺口、这一版的中央映射、两份缓存与收集表各有名字与类型，不为凑短签名收成结构体"
)]
fn collect_containers_of_a_container_tree_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    record_type: u16,
    gap: DeletionScanGap,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    containers: &mut Vec<crate::PackedUnitView>,
) -> Result<(), DeletionScanGap> {
    if node.level == 0 || node.entry_width < inode_internal_entry_bytes() {
        return Err(gap);
    }
    let registered_width = registered_record_width(record_type);
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        match ContainerTreeEntryType::of(read_u16(entry, 16), record_type) {
            ContainerTreeEntryType::InternalNode => {
                if child_pointer.all_zero {
                    return Err(gap);
                }
                let child = read_index_node_without_judging(
                    reader,
                    &child_pointer,
                    PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
                    index_cache,
                )
                .ok_or(gap)?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return Err(gap);
                }
                collect_containers_of_a_container_tree_below(
                    reader,
                    &child,
                    record_type,
                    gap,
                    central_mapping,
                    index_cache,
                    packed_cache,
                    containers,
                )?;
            }
            ContainerTreeEntryType::Container => {
                let container = read_packed_unit_without_judging(
                    reader,
                    &child_pointer,
                    central_mapping,
                    packed_cache,
                )
                .ok_or(gap)?;
                if container.view.record_type != record_type
                    || Some(container.view.record_width) != registered_width
                {
                    return Err(gap);
                }
                containers.push(container.view);
            }
            ContainerTreeEntryType::Unregistered(_) => return Err(gap),
        }
    }
    Ok(())
}

/// 一版的墓碑树按叶序读出的每片容器与记录，只读不判。
pub(super) fn tombstone_containers_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_TOMBSTONE,
        DeletionScanGap::TombstoneTreeOfTheVersionUnreadable,
        central_mapping,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .map(|container| TombstoneContainerOfTheScan {
            container: container.container,
            records: container
                .records
                .iter()
                .map(|record| TombstoneRecordOfTheScan::of(record))
                .collect(),
        })
        .collect())
}

/// 一片谱系容器：头里的容器号（I-9.36 / I-9.37 沿叶序比它）与它的记录。
pub(super) struct LineageContainerOfTheScan {
    pub(super) container: u64,
    pub(super) records: Vec<LineageRecordOfTheScan>,
}

/// 一版的谱系树按叶序读出的每片容器与记录，只读不判。
pub(super) fn lineage_containers_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<LineageContainerOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_LINEAGE,
        DeletionScanGap::LineageTreeUnreadable,
        central_mapping,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .map(|container| LineageContainerOfTheScan {
            container: container.container,
            records: container
                .records
                .iter()
                .map(|record| LineageRecordOfTheScan::of(record))
                .collect(),
        })
        .collect())
}

/// 一版的谱系树里的全部谱系记录，只读不判。
pub(super) fn lineage_records_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<LineageRecordOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_LINEAGE,
        DeletionScanGap::LineageTreeUnreadable,
        central_mapping,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .flat_map(|container| container.records.iter())
        .map(|record| LineageRecordOfTheScan::of(record))
        .collect())
}

/// 退役记录各段的偏移：被退役身份的出生树 0、
/// 打包记录类型 8（2 字节）、容器号 10、容器出生代 18，死亡 txg 26。checker 按条款另写一份，不用实现的读法。
const RETIREMENT_RECORD_BIRTH_TREE_OFFSET: usize = 0;
const RETIREMENT_RECORD_RECORD_TYPE_OFFSET: usize = 8;
const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;
const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;

/// 一片打包记录容器的身份四段（出生树、打包记录类型、容器号、容器出生代）：inode 树内部条目里的身份引用、退役记录里被退役的那一个都是这一形。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ContainerIdentityOfTheScan {
    pub(super) birth_tree: u64,
    pub(super) record_type: u16,
    pub(super) container: u64,
    pub(super) container_birth: u64,
}

impl ContainerIdentityOfTheScan {
    /// 从一段按「出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8」排的字节里读（`offset` 是出生树那一段的起点）。
    fn read_at(bytes: &[u8], offset: usize) -> Self {
        Self {
            birth_tree: read_u64(bytes, offset),
            record_type: read_u16(
                bytes,
                offset + RETIREMENT_RECORD_RECORD_TYPE_OFFSET - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
            container: read_u64(
                bytes,
                offset + RETIREMENT_RECORD_CONTAINER_OFFSET - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
            container_birth: read_u64(
                bytes,
                offset + RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET
                    - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
        }
    }
}

/// 一条退役记录：被退役的身份与死亡 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RetirementRecordOfTheScan {
    pub(super) retired: ContainerIdentityOfTheScan,
    pub(super) death_txg: u64,
}

impl RetirementRecordOfTheScan {
    pub(super) fn of(record: &[u8]) -> Self {
        Self {
            retired: ContainerIdentityOfTheScan::read_at(
                record,
                RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
            death_txg: read_u64(record, RETIREMENT_RECORD_DEATH_TXG_OFFSET),
        }
    }
}

/// 一片退役容器：头里的容器号与它的记录（I-9.32 数每片不多于 959 条）。
pub(super) struct RetirementContainerOfTheScan {
    pub(super) container: u64,
    pub(super) records: Vec<RetirementRecordOfTheScan>,
}

/// 一版树表里的一棵退役树：树 ID、头 ID（I-9.27：等于这张表里那棵 inode 树的树 ID）与按叶序读出的容器（读不全交回判不了的理由）。
pub(super) struct RetirementTreeOfTheScan {
    pub(super) tree: u64,
    pub(super) head_identifier: u64,
    pub(super) containers: Result<Vec<RetirementContainerOfTheScan>, DeletionScanGap>,
}

/// 一版的退役树按叶序读出的每片容器与记录，只读不判。
pub(super) fn retirement_containers_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<RetirementContainerOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_RETIREMENT,
        DeletionScanGap::RetirementTreeUnreadable,
        central_mapping,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .map(|container| RetirementContainerOfTheScan {
            container: container.container,
            records: container
                .records
                .iter()
                .map(|record| RetirementRecordOfTheScan::of(record))
                .collect(),
        })
        .collect())
}

/// 一版 inode 树里每片叶容器的身份（内部条目里类型段 2 那几条的身份引用，不开叶： I-9.34「读内部条目的身份引用即可」），
/// 只读不判。根指针全零、零条目根交回空。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]（主走读判 I-9.2 那些）。
pub(super) fn inode_leaf_identities_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
) -> Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap> {
    let mut identities = BTreeSet::new();
    if root_pointer.all_zero {
        return Ok(identities);
    }
    let root = read_index_node_without_judging(
        reader,
        root_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        index_cache,
    )
    .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
    if root.level == 0 {
        return if root.entries.is_empty() {
            Ok(identities)
        } else {
            Err(DeletionScanGap::InodeTreeUnreadable)
        };
    }
    collect_inode_leaf_identities_below(
        reader,
        &root,
        central_mapping,
        index_cache,
        &mut identities,
    )?;
    Ok(identities)
}

/// [`inode_leaf_identities_without_judging`] 的一个内部节点：类型段 2 的条目收它的身份引用，类型段 0 的往下一层（层级 = 父层级 − 1 且 ≥ 1）。
/// 递归的上界：每下一层层级减一。
fn collect_inode_leaf_identities_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    identities: &mut BTreeSet<ContainerIdentityOfTheScan>,
) -> Result<(), DeletionScanGap> {
    if node.entry_width < inode_internal_entry_bytes() {
        return Err(DeletionScanGap::InodeTreeUnreadable);
    }
    for entry in &node.entries {
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {
                identities.insert(ContainerIdentityOfTheScan::read_at(entry, 8));
            }
            InodeInternalEntryType::InternalNode => {
                let child_pointer = parse_node_pointer(&entry[34..120]);
                if child_pointer.all_zero {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                let child = read_index_node_without_judging(
                    reader,
                    &child_pointer,
                    PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
                    index_cache,
                )
                .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                collect_inode_leaf_identities_below(
                    reader,
                    &child,
                    central_mapping,
                    index_cache,
                    identities,
                )?;
            }
            InodeInternalEntryType::Unregistered(_) => {
                return Err(DeletionScanGap::InodeTreeUnreadable);
            }
        }
    }
    Ok(())
}

/// I-9.35 跨相邻两版比要的、一版里的那几样：txg、父版的 txg（这一版谱系树里有 T_rb 等于这一版 txg 的记录时取它的 T_old，否则 txg − 1；
/// txg 0 没有父版；谱系树读不全交回判不了的理由）、每棵 inode 树（按树 ID）的叶身份集、经头 ID 配到每棵 inode 树的退役树里死亡 txg 等于这一版
/// txg 的退役记录的身份集。树表 0 条的一版没有 inode 树（当父版时它的每棵树都是空集）。
pub(super) struct VersionOfTheRetirementCompleteness {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    pub(super) parent_txg: Result<Option<u64>, DeletionScanGap>,
    pub(super) leaf_identities_of_each_inode_tree:
        BTreeMap<u64, Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>>,
    pub(super) retired_at_this_txg_of_each_inode_tree:
        BTreeMap<u64, Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>>,
    /// 这一版树表读不出、条目窄：它当父版时比不了。
    pub(super) unreadable: Option<DeletionScanGap>,
}
