//! 池的几何、根环的槽位与各棵树节点数的上界（模型照条款自己算的一份）。

use super::*;

/// 池的几何：盘的身份与每块盘的字节数（与实现的 mkfs 参数由调用方各给一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPoolGeometry {
    pub devices: Vec<ModelDeviceIdentity>,
    pub device_size_in_bytes: u64,
}

/// 根环里的一个槽：第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）。
/// 公开是为了让对拍拿实现报的那个槽（`MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）与它比：
/// 两边各自声明（D13（验证路线） 已定项 5），胶水在 `model_comparison.rs` 里换过来。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelRingPosition {
    pub region: u64,
    pub slot_in_region: u64,
}

/// 一个单元在一块盘上的那一份占的槽：从 `first_slot` 起 `span_in_slots` 个槽。公开是为了让对拍拿实现报的那一份
/// （`RollbackError::IsolatedUnitCopyUnreadableOrMismatched`、`RollbackError::ResurrectedUnitCopyUnreadableOrMismatched`）
/// 与被吞的写落在哪比（故障注入，`fault_injection::left_by_the_lying_device`）；胶水在 `model_comparison.rs` 里换过来。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelUnitCopyLocation {
    pub device: ModelDeviceIdentity,
    pub span_in_slots: u32,
    pub first_slot: u64,
}

impl ModelUnitCopyLocation {
    /// 这一份在盘上占的字节区间 [起, 止)。
    ///
    /// # Panics
    /// 槽号乘槽宽越过 u64（盘上读来的槽号不会这么大，越过就是胶水换错了）。
    #[must_use]
    pub fn byte_range_on_the_device(self) -> std::ops::Range<u64> {
        let start = self
            .first_slot
            .checked_mul(SLOT_BYTES)
            .expect("槽号乘槽宽装得进 u64：槽号来自盘上的位置条目");
        let length = u64::from(self.span_in_slots)
            .checked_mul(SLOT_BYTES)
            .expect("一个单元两槽以内");
        start..start.checked_add(length).expect("单元末端装得进 u64")
    }
}

/// 模型默认建的池的 S：显式给 S 的建池入口（场景、随机历史、故障注入）取的那个值，[`IdealModel::after_make_filesystem`] 与
/// [`IdealModel::after_make_filesystem_on_a_journal_ring_of`] 照它建。按别的 S 建的池，模型经
/// [`IdealModel::after_make_filesystem_on_a_journal_ring_with_slots_per_region`] 记下那个 S，环几何跟着它算
/// （m3-close-c506 三轮「harness 模型跟着 S 走」）；一个模型建好之后 S 不变（S 是系统不可变配置）。
pub(super) const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;

/// 第 n 次发布落在区域 `n mod R` 的槽 `(n div R) mod S`，S 是这个模型建池时的那个（[`IdealModel`] 的 `slots_per_region`）。
pub(super) fn ring_position_of(
    checkpoint_txg: ModelCheckpointTxg,
    slots_per_region: u64,
) -> ModelRingPosition {
    ModelRingPosition {
        region: checkpoint_txg.0 % ROOT_RING_REGIONS,
        slot_in_region: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region,
    }
}

/// 码 2 索引节点含 key 区间与预留位的头宽：key 区间之外的 86、key 区间两个 key、预留位 29 三段相加
/// （D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16 / 已定项 18）。
///
/// 模型自己的一份式子，不调实现、也不调 checker 的那一份（D13（验证路线） 已定项 5：`singlefs-format` 只放标量；
/// 「三方各算一份 + 交叉断言」）；三份在 key 宽的全部取值上连起来比的是
/// `crates/singlefs-verification/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`。
#[must_use]
pub fn index_node_header_bytes(key_width_in_bytes: u64) -> u64 {
    let key_range_bytes = key_width_in_bytes * INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT;
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + key_range_bytes + NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// 一个索引节点装几条定宽条目：(节点 − 头) ÷ 条目宽（D8（核心索引结构） 已定项 2 / 已定项 11）。
pub(super) fn index_node_entry_capacity(key_width_in_bytes: u64, entry_width_in_bytes: u64) -> u64 {
    (NODE_BYTES - index_node_header_bytes(key_width_in_bytes)) / entry_width_in_bytes
}

/// 一个数据单元装得下多少字节的用户内容：32768 含头与预留位（D4（校验和位置） 已定项 5；D18（块里携带什么信息） 已定项 16），
/// 第一版没有扩展点声明值。
#[must_use]
pub fn data_unit_payload_capacity_in_bytes() -> u64 {
    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
}

/// 一份内容切成几个数据单元（D16（发布语义） 已定项 5 的切分纪律：一事务一单元，按净荷容量切）：长度除净荷容量向上取整，
/// 长度 0 也是一个单元（声明长度 0 的数据单元）。模型自己的一份式子，不调实现的切分（D13（验证路线） 已定项 5）。
#[must_use]
pub fn data_units_of_content(content_length_in_bytes: u64) -> u64 {
    content_length_in_bytes
        .div_ceil(data_unit_payload_capacity_in_bytes())
        .max(1)
}

/// 一个文件有 `data_units` 个数据单元时 extent 树下段有几个节点（D8（核心索引结构） 已定项 14：只有一个单元时不建下段、
/// 那个指针内联在上段叶条目里；下段按单元号按位置寻址，叶罩 144 个单元，内部节点扇出 147，往上数到只剩一个节点为止）。
#[must_use]
pub fn extent_lower_segment_nodes(data_units: u64) -> u64 {
    if data_units <= 1 {
        return 0;
    }
    let mut nodes_on_this_level = data_units.div_ceil(EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
    let mut nodes = nodes_on_this_level;
    // 迭代上界：每一层的节点数除扇出 147 向上取整，严格变少直到 1。
    while nodes_on_this_level > 1 {
        nodes_on_this_level = nodes_on_this_level.div_ceil(EXTENT_TREE_INTERNAL_FANOUT);
        nodes += nodes_on_this_level;
    }
    nodes
}

/// 一个文件有 `data_units` 个数据单元时 extent 树下段有几层（同 [`extent_lower_segment_nodes`] 的数法；没有下段是 0 层）：
/// 不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个（K-R3-1），占槽的上界按截之前的层数取。
#[must_use]
pub fn extent_lower_segment_levels(data_units: u64) -> u64 {
    if data_units <= 1 {
        return 0;
    }
    let mut nodes_on_this_level = data_units.div_ceil(EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
    let mut levels = 1;
    // 迭代上界同 `extent_lower_segment_nodes`。
    while nodes_on_this_level > 1 {
        nodes_on_this_level = nodes_on_this_level.div_ceil(EXTENT_TREE_INTERNAL_FANOUT);
        levels += 1;
    }
    levels
}

/// 中央映射树一片叶装几条映射条目：(节点 − 头) ÷ 映射条目宽（D8（核心索引结构） 已定项 11；条目 55，D19（块指针的结构与宽度预算） 已定项 6）。
fn central_mapping_tree_leaf_entry_capacity() -> u64 {
    index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_ENTRY_BYTES)
}

/// 中央映射树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
fn central_mapping_tree_internal_child_capacity() -> u64 {
    index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_KEY_BYTES + NODE_POINTER_BYTES)
}

/// 一个装不下的节点从中间切开之后，切出来的右半至少几条：装不下时是容量 + 1 条，从中间切两半各至少 ⌊(容量 + 1) ÷ 2⌋ = ⌈容量 ÷ 2⌉
/// （D8（核心索引结构） 已定项 11 ②）。
fn smallest_half_of_a_split(capacity: u64) -> u64 {
    capacity.div_ceil(2)
}

/// 中央映射树一次发布之后至多几个节点（D8（核心索引结构） 已定项 11 ②③ 与「一次发布里先删后插」）。
///
/// 节点数不是这一版条目数的函数：叶装不下时从中间切，收缩只摘空节点、不合并，删得只剩几条的叶照样留着，形状随来路变
/// （同一组条目，来路上长过的树可以比一次建起来的多几个节点）。模型不记 key、不照抄实现的分裂（D13（验证路线） 已定项 5），
/// 沿来路带一个上界：
/// - 先删：只摘空节点、根只剩一个孩子时降高，节点数不增；
/// - 后插：设叶切了 s 次、内部节点切了 t 次。切出来的右半至少 ⌊(容量 + 1) ÷ 2⌋（叶 147 条、内部 72 个孩子，
///   [`smallest_half_of_a_split`]），之后这一段只插不删，到这次结束仍至少这么多、彼此不相交：147 s ≤ 这一版的条目数 E，
///   72 t ≤ 这次之后的节点数 N − 1（根之外每个节点恰是一个内部节点的孩子）；根切开再长一个新根，至多 s + t 次。
///
/// ⇒ N ≤ N_前 + 2 (s + t) ≤ N_前 + 2 ⌊E ÷ 147⌋ + 2 ⌊(N − 1) ÷ 72⌋。右边随 N 涨得比 N 慢，满足它的 N 有个最大值：
/// 从 2 (N_前 + 2 ⌊E ÷ 147⌋)（那里已不满足）往下迭代，每一步都仍是 N 的上界，停在不动点（同分配记录树那一份的取法）。
///
/// `nodes_before`：上一版这棵树节点数的上界；这一版从空树建起时取 1（实现的规划从一片空的根兼叶起插）。
///
/// # Panics
/// `nodes_before` 为 0：一棵有条目的树至少一个节点，调用方给的是上一版的上界或 1。
#[must_use]
pub fn central_mapping_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -> u64 {
    assert!(
        nodes_before >= 1,
        "上一版的树至少一个节点（从空树建起也从一片空的根兼叶起插）"
    );
    let nodes_without_internal_splits = nodes_before
        + 2 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));
    let internal_split_bound = |nodes: u64| {
        nodes_without_internal_splits
            + 2 * ((nodes - 1)
                / smallest_half_of_a_split(central_mapping_tree_internal_child_capacity()))
    };
    let mut bound = 2 * nodes_without_internal_splits;
    // 迭代上界：bound 每一轮严格变小，不小于 1。跨轮带的是「这次之后的节点数不超过 bound」。
    loop {
        let next = internal_split_bound(bound);
        if next >= bound {
            return bound;
        }
        bound = next;
    }
}

/// 记账树一片叶装几行：(节点 − 头) ÷ 记账条目宽（D8（核心索引结构） 已定项 11；条目 34，D5（快照 / 空间记账机制） 已定项 5）。
fn accounting_tree_leaf_entry_capacity() -> u64 {
    index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)
}

/// 记账树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
fn accounting_tree_internal_child_capacity() -> u64 {
    index_node_entry_capacity(
        ACCOUNTING_KEY_BYTES,
        ACCOUNTING_KEY_BYTES + NODE_POINTER_BYTES,
    )
}

/// 记账树一次写记账行的发布之后至多几个节点（D8（核心索引结构） 已定项 11 ②③ 与「一次发布里先删后插」）。
///
/// 节点数不是行数的函数（收缩只摘空节点、不合并，形状随来路变）；模型不记 key、不照抄实现的分裂（D13（验证路线） 已定项 5），
/// 同中央映射树沿来路带一个上界（[`central_mapping_tree_nodes_upper_bound`]），这里多数一半：
/// - 先删：只摘空节点、根只剩一个孩子时降高，节点数不增；
/// - 后插：设叶切了 s 次、内部节点切了 t 次、根切了 r 次（切开的根上面再长一个新根）。每切一次多一个节点，根切再多一个：
///   N ≤ N_前 + s + t + r。
/// - 切开的两半各至少 ⌊(容量 + 1) ÷ 2⌋（叶 239 行、内部 75 个孩子，[`smallest_half_of_a_split`]），之后这一段只插不删，到这次结束仍至少这么多。
///   每切一次，这次结束时「切出来的」叶至少多一片（切原有的叶多两片、切切出来的叶多一片），s ≥ 1 时至少 s + 1 片、彼此不相交：
///   (s + 1) × 239 ≤ 这次之后的行数 R。内部节点同一条：(t + 1) × 75 ≤ N − 1（根之外每个节点恰是一个内部节点的孩子）。
///   没有叶切就没有内部节点切（内部节点只在孩子切开时多一个孩子）：s = 0 ⇒ t = 0。
/// - 根切：根兼叶切开那一次是叶切，之后的根都是内部节点、它切开算在 t 里：r ≤ min(s + t, 1 + t)。
///
/// ⇒ 从一个节点长到 R = 478 行（两块盘 463 级台阶，D5 已定项 2 射程）：s ≤ 1、t = 0、r ≤ 1，N ≤ 3，恰是实现长成的根加两片叶。
/// 右边随 N 涨得比 N 慢，满足它的 N 有个最大值：从 2 (N_前 + 2 s + 1)（那里已不满足）往下迭代，每一步都仍是 N 的上界，停在不动点。
///
/// `nodes_before`：上一版这棵树节点数的上界；上一版没有记账树时取 1（实现的规划从一片空的根兼叶起插）。
///
/// # Panics
/// `nodes_before` 为 0：一棵有行的树至少一个节点，调用方给的是上一版的上界或 1。
#[must_use]
pub fn accounting_tree_nodes_upper_bound(nodes_before: u64, rows: u64) -> u64 {
    assert!(
        nodes_before >= 1,
        "上一版的树至少一个节点（从空树建起也从一片空的根兼叶起插）"
    );
    let leaf_splits_bound =
        (rows / smallest_half_of_a_split(accounting_tree_leaf_entry_capacity())).saturating_sub(1);
    let nodes_after_the_splits_bound = |nodes: u64| {
        let internal_splits_bound = if leaf_splits_bound == 0 {
            0
        } else {
            ((nodes - 1) / smallest_half_of_a_split(accounting_tree_internal_child_capacity()))
                .saturating_sub(1)
        };
        let root_splits_bound =
            (leaf_splits_bound + internal_splits_bound).min(1 + internal_splits_bound);
        nodes_before + leaf_splits_bound + internal_splits_bound + root_splits_bound
    };
    let mut bound = 2 * (nodes_before + 2 * leaf_splits_bound + 1);
    // 迭代上界：bound 每一轮严格变小，不小于 nodes_before。跨轮带的是「这次之后的节点数不超过 bound」。
    loop {
        let next = nodes_after_the_splits_bound(bound);
        if next >= bound {
            return bound;
        }
        bound = next;
    }
}

/// deadlist 树一片叶装几条条目：(节点 − 头) ÷ key 宽（value 空；判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3「叶 376」）。
fn deadlist_tree_leaf_entry_capacity() -> u64 {
    index_node_entry_capacity(DEADLIST_KEY_BYTES, DEADLIST_KEY_BYTES)
}

/// deadlist 树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（判决 S3「内部 125」）。
fn deadlist_tree_internal_child_capacity() -> u64 {
    index_node_entry_capacity(DEADLIST_KEY_BYTES, DEADLIST_KEY_BYTES + NODE_POINTER_BYTES)
}

/// deadlist 树一次发布之后至多几个节点：没有条目时是零根、一个节点都没有；有条目时同中央映射树那一份的取法
/// （[`central_mapping_tree_nodes_upper_bound`]：先删只摘空节点、后插从中间切，沿来路带一个上界），容量换成 deadlist 树的。
/// `nodes_before`：上一版这棵树节点数的上界（零根时 0，从一片空的根兼叶起插按 1 算）。
#[must_use]
pub fn deadlist_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -> u64 {
    if entries == 0 {
        return 0;
    }
    let nodes_without_internal_splits = nodes_before.max(1)
        + 2 * (entries / smallest_half_of_a_split(deadlist_tree_leaf_entry_capacity()));
    let internal_split_bound = |nodes: u64| {
        nodes_without_internal_splits
            + 2 * ((nodes - 1) / smallest_half_of_a_split(deadlist_tree_internal_child_capacity()))
    };
    let mut bound = 2 * nodes_without_internal_splits;
    // 迭代上界：bound 每一轮严格变小，不小于 1。跨轮带的是「这次之后的节点数不超过 bound」。
    loop {
        let next = internal_split_bound(bound);
        if next >= bound {
            return bound;
        }
        bound = next;
    }
}

/// 一个 `data_units` 个单元（单元号 0 .. data_units − 1、没有洞）的文件，extent 树下段的全部位置（D8（核心索引结构） 已定项 14：
/// 只有一个单元时不建下段；第 L 层一个节点罩 144 × 147^L 个单元，从叶往上数到只剩一个节点的那一层为止）。
#[must_use]
pub fn extent_lower_node_positions(data_units: u64) -> Vec<ModelExtentLowerNodePosition> {
    let mut positions = Vec::new();
    if data_units <= 1 {
        return positions;
    }
    let mut level = 0;
    let mut span_in_data_units = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
    // 迭代上界：每一层的罩宽乘 147，节点数严格变少直到 1。
    loop {
        let nodes_in_this_level = data_units.div_ceil(span_in_data_units);
        positions.extend((0..nodes_in_this_level).map(|index_in_level| {
            ModelExtentLowerNodePosition {
                level,
                index_in_level,
            }
        }));
        if nodes_in_this_level == 1 {
            return positions;
        }
        level += 1;
        span_in_data_units *= EXTENT_TREE_INTERNAL_FANOUT;
    }
}

/// 下段一个位置罩的最后一个单元号（按满罩宽算，不按这个文件截到哪）：删除批判一个节点是不是整个落在留下那一段里用它
/// （`unlink::plan_the_extent_tree_after_a_deletion_batch` 的条款 K-R3-1）。
pub(super) fn last_data_unit_under(position: ModelExtentLowerNodePosition) -> u64 {
    let span_in_data_units = (0..position.level)
        .fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
            span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
        });
    position
        .index_in_level
        .saturating_mul(span_in_data_units)
        .saturating_add(span_in_data_units - 1)
}

/// 下段一个位置罩的第一个单元号（同 [`last_data_unit_under`] 按满罩宽算）。
pub(super) fn first_data_unit_under(position: ModelExtentLowerNodePosition) -> u64 {
    let span_in_data_units = (0..position.level)
        .fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
            span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
        });
    position.index_in_level.saturating_mul(span_in_data_units)
}

/// 一版的中央映射树里除分配记录树的节点之外装几条条目的上界（D19（块指针的结构与宽度预算） 已定项 8：进映射的是数据单元、extent 树每个节点、
/// inode 叶容器与根、分配记录树每个节点、记账树每个节点、`deleted_inodes` 根、墓碑树与谱系树的容器与索引节点；
/// 映射树自己的节点、树表、实例表豁免）。分配记录树的节点数由调用方按它的上界另加；记账树的节点按调用方给的这一版的上界
/// （`accounting_tree_nodes`，[`accounting_tree_nodes_upper_bound`]）数。
///
/// 谱系树模型不记（管理员回退往它上面追加一条，D18（块里携带什么信息） 已定项 24 第 5 条）：一条时间线上每次发布至多是一次回退，
/// 回退至多 txg 次，一片容器装 1359 条，加根；没回退过的一版没有这棵树，多算的两条只让上界松一点。
///
/// # Panics
/// 谱系容器的上界多于一片：模型罩的历史一条时间线上发布不到 1359 次，谱系树根之下不长层。
pub(super) fn central_mapping_entries_other_than_the_allocation_record_tree(
    contents: &ModelVersionContents,
    role_written_at: &BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
    accounting_tree_nodes: u64,
    checkpoint_txg: ModelCheckpointTxg,
) -> u64 {
    // 存在的单元（带洞的文件少于 size 罩的、截断途中连 size 之外还挂着的，判决 `research/prompts/m4-trunc-r1-main-verification.md`
    // 第三节 K2、K5）各一条，下段按存在的单元建。
    let data_units = contents.data_units_present();
    let entries_of_one_node_roles: u64 = role_written_at
        .keys()
        .map(|role| match role {
            // 记账树：根与根之下的每个节点各一条（台阶行多到一片叶装不下时长层），按这一版节点数的上界数。
            ModelUnitRole::AccountingTree => accounting_tree_nodes,
            // 模型罩的一版里这几个角色各一个单元（上段每个节点各是一个角色、每片 inode 叶容器各是一个角色、墓碑树一片容器加根）。
            ModelUnitRole::ExtentRoot
            | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
            | ModelUnitRole::InodeLeaf
            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::DeletedInodesRoot
            | ModelUnitRole::LoggedOperationsRoot
            | ModelUnitRole::TombstoneLeaf
            | ModelUnitRole::TombstoneRoot
            | ModelUnitRole::RetirementLeaf
            | ModelUnitRole::RetirementRoot => 1,
            // 数据单元按这一版的单元数另数，分配记录树的节点由调用方按上界另加。
            ModelUnitRole::Data | ModelUnitRole::AllocationTree => 0,
            // 豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
            ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::InstanceTable => 0,
        })
        .sum();
    let lineage_containers = checkpoint_txg.0.div_ceil(LINEAGE_CONTAINER_RECORDS);
    assert!(
        lineage_containers <= 1,
        "模型罩的历史一条时间线上发布不到 1359 次（谱系树至多一片容器）：txg {}",
        checkpoint_txg.0
    );
    let lineage_root = u64::from(lineage_containers > 0);
    data_units
        + extent_lower_segment_nodes(data_units)
        + entries_of_one_node_roles
        + lineage_containers
        + lineage_root
}

/// extent 树下段一个节点的位置（D8（核心索引结构） 已定项 14：按单元号按位置寻址，层级 0 是叶，同层从单元 0 起数第几个）：
/// 一个没有洞的文件有几个单元就定了下段有哪几个位置。模型自己一份，不引实现的类型（D13（验证路线） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelExtentLowerNodePosition {
    pub level: u32,
    pub index_in_level: u64,
}

impl IdealModel {
    /// 分配记录树一次发布至多重写几个节点（每个节点每盘占一个槽）。
    ///
    /// 树的形状照 D8（核心索引结构） 已定项 14 另算一份：按绝对槽号按位置寻址，叶罩 812 个槽，内部节点罩 169 个孩子那么宽的一段；
    /// 根按盘分路，根的层号取「每块盘按下一层的宽度切出来的段数之和装得进一个根」的最低一层（实现员的取法，交回里写明）。
    ///
    /// 重写的是根，与这次之后还装着记录、内容变了的节点。模型不记哪几片变了，取「这次之后可能装着记录的节点」的全数：
    /// 记录只住在落点上；用户数据与开新段按 D3（空间分配） 已定项 8 第 1 条、已定项 10 从最低处取（用户数据取最低的空槽对，开新段取最低的全空段），
    /// 每个落点至多多开一个 64 槽的段；提交内生块回落（已定项 8 第 2 条：先取重写集的叶、再取每块盘都空的槽最多的叶）不取最低处，可它只在
    /// 每块盘上一个全空段都没有时才走，那时单元区里每个整段（活 `R` 那一段除外，见下面推不出的几角）至少有一槽被挡 ⇒ 挡住各段的槽都是沿来路写过的槽时，
    /// 单元区不过 `64 × P` 槽，回落落在哪一片叶都仍在单元区起点往后 `64 × P` 个槽里 ⇒ 沿来路写过 P 个槽之后，落点都在这一截里（单元区墙那一格同一个读法）。
    /// 推不出的几角，每一角都让「单元区不过 `64 × P` 槽」这一步推不出来，回落可以落到那一截之外，这个上界在那里罩不罩得住没有推出来：
    /// - 有的段只靠影子账隔离或抬 F 扣住的槽撑成非空（那几槽不是这条来路写的，不在 P 里），没有全空段时单元区可以长过 `64 × P`；
    /// - 活 `R` 那一段可以整段空而回落照走：分配器找全空段时排除活 `R` 那一段（`DeviceFreeMap::lowest_empty_segment_outside_the_region`），
    ///   那一段一槽都不必被挡，单元区可以多出这 64 槽；
    /// - 单元区尾巴不满 64 槽的那一截不算段：同一处找全空段只走整段（单元区槽数整除 64 的那几段），没有全空段时那一截可以一槽都不被挡。
    ///
    /// 这几角欠在 D3（空间分配） 已定项 8 的「欠」（理想模型重写节点上界在隔离撑起的非空段一角）。
    /// 这次重写的节点自己也占槽，P 里含着要求的那个数：设重写数为 A，A ≤ g(A)（g 是「写过 P₀ + A 个槽之后可能装着记录的节点数」，
    /// 对 A 单调不减），从 g(∞)（单元区里的全部节点）起往下迭代，每一步都仍是 A 的上界，停在不动点。
    pub(super) fn allocation_record_tree_nodes_rewritten_upper_bound(
        &self,
        occupied_slots_before_the_tree_nodes: u64,
    ) -> u64 {
        let device_slots = self.geometry.device_size_in_bytes / SLOT_BYTES;
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        let span_at_level = |level: u32| {
            ALLOCATION_RECORD_TREE_LEAF_SLOTS
                .saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.saturating_pow(level))
        };
        let root_level = (1_u32..)
            .find(|level| {
                device_count * device_slots.div_ceil(span_at_level(level - 1))
                    <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
            })
            .expect("层号往上走，每块盘切出来的段数终归是 1，盘数不到 169 就装得进一个根");
        let nodes_that_may_hold_records = |occupied_slots: u64| {
            let reach = self
                .unit_area_start_slot
                .saturating_add(CLUSTER_SEGMENT_SLOTS.saturating_mul(occupied_slots))
                .min(device_slots);
            let per_device: u64 = if reach <= self.unit_area_start_slot {
                0
            } else {
                (0..root_level)
                    .map(|level| {
                        let span = span_at_level(level);
                        (reach - 1) / span - self.unit_area_start_slot / span + 1
                    })
                    .sum()
            };
            1 + device_count * per_device
        };
        let mut bound = nodes_that_may_hold_records(u64::MAX);
        loop {
            let next = nodes_that_may_hold_records(
                occupied_slots_before_the_tree_nodes.saturating_add(bound),
            );
            if next >= bound {
                return bound;
            }
            bound = next;
        }
    }

    /// 这次发布之后中央映射树节点数的上界，与分配记录树这次至多重写几个节点
    /// （[`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`]）。
    ///
    /// 两个量互相牵连：映射树装着分配记录树每个节点的一条条目（D19（块指针的结构与宽度预算） 已定项 8），分配记录树可能装着记录的节点数
    /// 又随映射树多写的节点占的槽涨。两个都对另一个单调不减：从「分配记录树罩满单元区」那一头起交替往下迭代，
    /// 每一步都仍是两者的上界，停在映射树节点数不再变小的那一步。
    ///
    /// `occupied_slots_before_the_two_trees`：沿来路写过的槽，加这次除映射树根之下的节点与分配记录树之外写的槽（映射树的根已算在里面）。
    pub(super) fn central_mapping_and_allocation_record_tree_nodes_upper_bounds(
        &self,
        central_mapping_tree_nodes_before: u64,
        central_mapping_entries_other_than_the_allocation_record_tree: u64,
        occupied_slots_before_the_two_trees: u64,
    ) -> (u64, u64) {
        let mapping_nodes_given = |allocation_record_tree_nodes: u64| {
            central_mapping_tree_nodes_upper_bound(
                central_mapping_tree_nodes_before,
                central_mapping_entries_other_than_the_allocation_record_tree
                    + allocation_record_tree_nodes,
            )
        };
        let allocation_record_tree_nodes_given = |mapping_nodes: u64| {
            self.allocation_record_tree_nodes_rewritten_upper_bound(
                occupied_slots_before_the_two_trees
                    + (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots(),
            )
        };
        let mut mapping_nodes =
            mapping_nodes_given(self.allocation_record_tree_nodes_rewritten_upper_bound(u64::MAX));
        // 迭代上界：映射树节点数每一轮严格变小、不小于 1。跨轮带的是「两者的真值都不超过这一轮的 (映射树, 分配记录树)」。
        loop {
            let allocation_record_tree_nodes = allocation_record_tree_nodes_given(mapping_nodes);
            let next = mapping_nodes_given(allocation_record_tree_nodes);
            if next >= mapping_nodes {
                return (mapping_nodes, allocation_record_tree_nodes);
            }
            mapping_nodes = next;
        }
    }

    /// 接在 `previous` 后面、写记账行的一次发布写几行：池级三行加每块盘六行（D5（快照 / 空间记账机制） 已定项 8），加照抄的
    /// inode 号水位台阶行（D5 已定项 2 射程「inode 号水位的台阶行」）——这次自己那一行是不是新的一级台阶不改行数（它就是池级那一行）。
    pub(super) fn accounting_rows_of_a_publish_on(&self, previous: &ModelRoot) -> u64 {
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        POOL_WIDE_ACCOUNTING_ROWS
            + ACCOUNTING_ROWS_PER_DEVICE * device_count
            + previous.inode_number_watermark_steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 模型跟着建池时的 S 落槽（m3-close-c506 三轮「harness 模型跟着 S 走」）：txg 12 在 S = 4 的模型里落区域 0 槽 0、
    /// 盖掉第 0 代根；在 S = 8 的模型里落区域 0 槽 4，第 0 代根还在环里。
    #[test]
    fn the_model_places_roots_by_the_slots_per_region_it_was_built_with() {
        let geometry = ModelPoolGeometry {
            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
            device_size_in_bytes: 4 << 30,
        };
        let txgs_in_the_ring_after_writing_txg_twelve = |slots_per_region: u64| {
            let mut model =
                IdealModel::after_make_filesystem_on_a_journal_ring_with_slots_per_region(
                    geometry.clone(),
                    JOURNAL_RING_DEFAULT_BYTES,
                    slots_per_region,
                );
            let mut root_of_txg_twelve = model.newest_root().clone();
            root_of_txg_twelve.key.checkpoint_txg = ModelCheckpointTxg(12);
            model.write_root(root_of_txg_twelve);
            model
                .committed_versions()
                .iter()
                .map(|(key, _)| key.checkpoint_txg.0)
                .collect::<Vec<u64>>()
        };
        assert_eq!(
            txgs_in_the_ring_after_writing_txg_twelve(4),
            vec![12],
            "S = 4：环长 12，txg 12 绕回区域 0 槽 0，盖掉第 0 代根"
        );
        assert_eq!(
            txgs_in_the_ring_after_writing_txg_twelve(8),
            vec![0, 12],
            "S = 8：环长 24，txg 12 落区域 0 槽 4，第 0 代根还在"
        );
    }

    /// 分配记录树一次发布至多重写几个节点（D8（核心索引结构） 已定项 14：叶罩 812 槽、内部扇出 169）：4 GiB 两块盘，根在第 2 层；
    /// 第一个文件版本之前每盘写过 3 槽（mkfs 的实例表 2、树表 1），这次树之外写 9 槽（数据 2、extent 根 1、inode 叶 2、inode 根 1、
    /// 记账树、映射树、树表各 1）。设这次重写 A 个节点：落点都在单元区起点 50176 往后 64 × (12 + A) 槽里。
    /// A = 9 时那一段到 51520，罩叶 61..=63 三片、第 1 层 1 个 ⇒ 1 + 2 × 4 = 9，是不动点；从单元区全部 529 个节点往下迭代停在这里。
    #[test]
    fn allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine()
    {
        let model = two_device_model();
        assert_eq!(
            model.allocation_record_tree_nodes_rewritten_upper_bound(u64::MAX / 128),
            1 + 2 * (262 + 2),
            "单元区里的全部节点：每盘叶 61..=322、第 1 层 0..=1，加根"
        );
        assert_eq!(
            model.allocation_record_tree_nodes_rewritten_upper_bound(3 + 9),
            9
        );
        let mut model = model;
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
        assert_eq!(slots_before, 3, "暖机是零单元发布，一个槽都不占");
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        assert_eq!(
            first.expected_roots[0].occupied_slots_upper_bound_per_device,
            3 + 9 + 9
        );
    }

    /// extent 树下段的节点数与层数（D8（核心索引结构） 已定项 14：一个单元内联、不建下段；叶罩 144 个单元、内部扇出 147）：
    /// 模型按它给顺序写与删除批的占槽上界。
    #[test]
    fn the_lower_segment_of_a_file_has_a_leaf_per_144_data_units_and_an_internal_node_per_147_children(
    ) {
        assert_eq!(
            [1, 2, 144, 145, 288, 289].map(|data_units| (
                extent_lower_segment_nodes(data_units),
                extent_lower_segment_levels(data_units)
            )),
            [(0, 0), (1, 1), (1, 1), (2 + 1, 2), (2 + 1, 2), (3 + 1, 2)]
        );
        assert_eq!(
            (
                extent_lower_segment_nodes(144 * 147 + 1),
                extent_lower_segment_levels(144 * 147 + 1)
            ),
            (148 + 2 + 1, 3),
            "148 片叶要两个第一层节点，再一个根"
        );
    }

    /// 中央映射树节点数的上界（D8（核心索引结构） 已定项 11：叶装 294 条、内部节点装 143 个孩子，装不下从中间切，切出来的右半至少 147 条 / 72 个孩子；
    /// 收缩只摘空节点）：条目不到 147 条时一次都切不了，上界照上一版的；每多 147 条至多多切一片叶、多长一个新根；上一版长过的节点不往回收。
    #[test]
    fn central_mapping_tree_nodes_grow_by_two_per_147_entries_and_keep_the_previous_bound() {
        assert_eq!(
            (
                central_mapping_tree_leaf_entry_capacity(),
                central_mapping_tree_internal_child_capacity(),
                smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()),
                smallest_half_of_a_split(central_mapping_tree_internal_child_capacity()),
            ),
            (294, 143, 147, 72)
        );
        assert_eq!(
            [(1, 146), (1, 147), (1, 294), (5, 10), (1, 147 * 72)]
                .map(|(nodes_before, entries)| central_mapping_tree_nodes_upper_bound(
                    nodes_before,
                    entries
                )),
            [1, 1 + 2, 1 + 2 * 2, 5, 1 + 2 * 72 + 2 * 2],
            "147 × 72 条：叶至多切 72 次，内部节点再切的次数 t 满足 72 t ≤ N − 1，停在 N = 149（t = 2）"
        );
    }

    /// 顺序写一个跨三片下段叶的文件（292 个单元）：映射条目 302 条加分配记录树的节点，多于一片叶装的 294 条，映射树从中间切开。
    /// 4 GiB 两块盘、第一个文件之后（每盘写过 21 槽，映射树一个节点）：这次映射树之外写 595 槽（数据 2 × 292、下段 3 片叶加 1 个内部节点、
    /// extent 根 1、inode 叶 2、inode 根 1、记账树、映射树的根、树表各 1）；分配记录树至多 123 个节点，条目至多 302 + 123 = 425，
    /// 叶至多切 2 次 ⇒ 映射树至多 1 + 2 × 2 = 5 个节点，根之下 4 个各占一槽：这次占槽上界 595 + 4 + 123 = 722。
    /// 条目里 302 = 数据 292 + 下段 4 + extent 根、inode 叶、inode 根、记账树各 1 + 谱系树按 txg 4 至多一片容器加根 2。
    #[test]
    fn sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes_below_the_root_in_the_occupied_slots(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
        assert_eq!(
            (
                slots_before,
                model.newest_root().central_mapping_tree_nodes_upper_bound
            ),
            (3 + 9 + 9, 1),
            "第一个文件之后：映射条目十来条，映射树一个节点"
        );
        let data_units = 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4;
        let capacity = data_unit_payload_capacity_in_bytes();
        let content =
            vec![5_u8; usize::try_from((data_units - 1) * capacity + 1).expect("九兆多字节")];
        assert_eq!(
            data_units_of_content(u64::try_from(content.len()).expect("内容长度")),
            292
        );
        let sequential_write = model
            .answer_publish_sequential_write(&content)
            .expect("会话开着、现行版本带文件");
        assert!(
            sequential_write.required_refusals.is_empty(),
            "{:?}",
            sequential_write.required_refusals
        );
        let written = &sequential_write.expected_roots[0];
        let slots_outside_the_two_trees = 2 * 292 + (3 + 1) + 1 + 2 + 1 + 1 + 1 + 1;
        assert_eq!(
            (
                written.central_mapping_tree_nodes_upper_bound,
                model.allocation_record_tree_nodes_rewritten_upper_bound(
                    slots_before + slots_outside_the_two_trees + 4
                ),
                written.occupied_slots_upper_bound_per_device - slots_before,
            ),
            (5, 123, slots_outside_the_two_trees + 4 + 123),
            "映射树至多 5 个节点（根之下 4 个），分配记录树至多 123 个节点：这次占槽上界 722"
        );
    }

    /// 记账树节点数的上界（D8（核心索引结构） 已定项 11：叶装 477 行、内部节点装 150 个孩子，装不下从中间切，切开的两半各至少 239 行 / 75 个孩子；
    /// 收缩只摘空节点）：不到 2 × 239 = 478 行一次都切不了，上界照上一版的；从一个节点长到 478 行至多切一片叶、根切一次，3 个节点；
    /// 717 = 3 × 239 行至多切两片叶，4 个；76 × 239 行 76 片叶挂一个根，77 个；151 × 239 行 151 片叶一个根装不下，根切一次、长成两个内部节点加新根，154 个。
    /// 上一版长过的节点不往回收（3 个节点、477 行仍是 3），长过之后每多切一片叶多两个（叶一个、根一个的上界）。
    #[test]
    fn accounting_tree_nodes_grow_to_three_at_478_rows_and_keep_the_previous_bound() {
        assert_eq!(
            (
                accounting_tree_leaf_entry_capacity(),
                accounting_tree_internal_child_capacity(),
                smallest_half_of_a_split(accounting_tree_leaf_entry_capacity()),
                smallest_half_of_a_split(accounting_tree_internal_child_capacity()),
            ),
            (477, 150, 239, 75)
        );
        assert_eq!(
            [
                (1, 15),
                (1, 477),
                (1, 478),
                (1, 3 * 239),
                (1, 76 * 239),
                (1, 151 * 239),
                (3, 477),
                (3, 479),
            ]
            .map(|(nodes_before, rows)| accounting_tree_nodes_upper_bound(nodes_before, rows)),
            [1, 1, 3, 4, 77, 154, 3, 5],
            "478 行：叶切 1 次、根切 1 次；151 × 239 行：叶切 150 次、内部节点切 1 次（2 × 75 ≤ 153）、根切 2 次之内取 1 + t = 2"
        );
    }

    /// 两块 4 GiB 盘：第一个文件、unlink、最后一批之后一次建一个 inode，第 462 个那一版写 477 行、记账树一个节点；第 463 个那次照抄 463 级台阶、
    /// 写 478 行，记账树长成根加两片叶（上界 3）。这次占槽上界：分配记录树与映射树之外写 8 槽（新记录落的第 2 片 inode 叶容器 2、inode 根 1、
    /// 记账树 3 个节点各 1、映射树的根 1、树表 1），加映射树根之下的节点与分配记录树按上界算的节点。
    #[test]
    fn the_inode_creation_whose_watermark_step_crosses_one_accounting_leaf_counts_the_nodes_below_the_root_in_the_occupied_slots(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let intent = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &intent);
        let final_batch = model
            .answer_deletion_step()
            .expect("会话开着、现行版本删除中");
        succeed(&mut model, &final_batch);
        for _ in 0..462 {
            let created = model
                .answer_create_inode()
                .expect("会话开着、第一个文件已删完");
            succeed(&mut model, &created);
        }
        let before = model.newest_root().clone();
        let created = model
            .answer_create_inode()
            .expect("会话开着、第一个文件已删完");
        assert!(
            created.required_refusals.is_empty(),
            "{:?}",
            created.required_refusals
        );
        let written = &created.expected_roots[0];
        let slots_outside_the_two_trees = 2 + 1 + 3 + 1 + 1;
        let mapping_nodes_below_the_root = written.central_mapping_tree_nodes_upper_bound - 1;
        assert_eq!(
            (
                (before.accounting_rows, before.accounting_tree_nodes_upper_bound),
                (written.accounting_rows, written.accounting_tree_nodes_upper_bound),
                written.occupied_slots_upper_bound_per_device
                    - before.occupied_slots_upper_bound_per_device,
            ),
            (
                (477, 1),
                (478, 3),
                slots_outside_the_two_trees
                    + mapping_nodes_below_the_root
                    + model.allocation_record_tree_nodes_rewritten_upper_bound(
                        before.occupied_slots_upper_bound_per_device
                            + slots_outside_the_two_trees
                            + mapping_nodes_below_the_root
                    ),
            ),
            "第 462 个那一版 477 行一个节点；第 463 个那次 478 行、根加两片叶，三个节点各占一槽进这次的占槽上界"
        );
    }

    /// extent 树下段的位置（D8（核心索引结构） 已定项 14）：一个单元不建下段；145 个单元两片叶加一个根；三片叶的那一档三片加根。
    #[test]
    fn extent_lower_node_positions_follow_the_data_unit_count() {
        assert!(extent_lower_node_positions(1).is_empty());
        let leaf = |index_in_level| ModelExtentLowerNodePosition {
            level: 0,
            index_in_level,
        };
        let root = ModelExtentLowerNodePosition {
            level: 1,
            index_in_level: 0,
        };
        assert_eq!(extent_lower_node_positions(2), vec![leaf(0)]);
        assert_eq!(
            extent_lower_node_positions(EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1),
            vec![leaf(0), leaf(1), root]
        );
        assert_eq!(
            extent_lower_node_positions(2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4),
            vec![leaf(0), leaf(1), leaf(2), root]
        );
        assert_eq!(
            u64::try_from(
                extent_lower_node_positions(2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4).len()
            )
            .expect("个数"),
            extent_lower_segment_nodes(2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4),
            "与按层数个数的那一份式子数出来的相同"
        );
    }
}
