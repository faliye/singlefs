# 变异清单：名字 -> (文件, 原文, 替换文)；原文要恰好命中一次。
MUTATIONS = {
 "m3-left-first": ("crates/singlefs-core/src/inode_tree.rs",
"""            if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else {""",
"""            if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else {"""),
 "m3-floor-116": ("crates/singlefs-core/src/inode_tree.rs",
"""usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")""",
"""usize::try_from(INODE_LEAF_RECORDS / 2).expect("117")"""),
 "m3-floor-118": ("crates/singlefs-core/src/inode_tree.rs",
"""usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")""",
"""usize::try_from(INODE_LEAF_RECORDS.div_ceil(2) + 1).expect("117")"""),
 "r3-carried-from-new-position": ("crates/singlefs-core/src/inode_tree.rs",
"""                origins.push(InodeLeafContainerOrigin::CarriedFrom(
                    container.previous_index,
                ));""",
"""                origins.push(InodeLeafContainerOrigin::CarriedFrom(
                    InodeLeafContainerIndexInTree::of_position(position),
                ));"""),
 "checker-i934-no-lineage-filter": ("crates/singlefs-checker/src/walk.rs",
"""        .filter(|record| lineage_chain.holds_txg(record.death_txg))
    {
        judgements.judge(
            RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,""",
"""        .filter(|record| lineage_chain.holds_txg(record.death_txg) || record.death_txg > 0)
    {
        judgements.judge(
            RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,"""),
 "checker-i932-no-birth-before-death": ("crates/singlefs-checker/src/walk.rs",
"""                && retired.container_birth < record.death_txg
                && record.death_txg <= version.checkpoint_txg;""",
"""                && record.death_txg <= version.checkpoint_txg;"""),
 "checker-i932-no-death-at-most-version": ("crates/singlefs-checker/src/walk.rs",
"""                && retired.container_birth < record.death_txg
                && record.death_txg <= version.checkpoint_txg;""",
"""                && retired.container_birth < record.death_txg;"""),
}

# 「去掉退役那几项」的副本（K5-2 的对照）：标签 6 的 2 + H_退役 + 1 与 35 × n_记录、W 的 6 + H_退役、ckpt_cost 的退役项与它在映射树里那几把全置 0。
NORET_EDITS = [
 ("crates/singlefs-core/src/admission.rs",
  "pub const RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA: u64 = DATA_UNIT_BYTES / SLOT_BYTES;",
  "pub const RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA: u64 = 0;"),
 ("crates/singlefs-core/src/admission.rs",
  "pub const RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA: u64 = 1;",
  "pub const RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA: u64 = 0;"),
 ("crates/singlefs-core/src/admission.rs",
  "pub const RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA: u64 =\n    DATA_UNIT_BYTES.div_ceil(RETIREMENT_CONTAINER_RECORDS);",
  "pub const RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA: u64 = 0;"),
 ("crates/singlefs-core/src/admission.rs",
  "pub const DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST: u64 = 6;",
  "pub const DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST: u64 = 0;"),
 ("crates/singlefs-core/src/admission.rs",
  """    MetadataBlocks(
        (RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COW + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COW)
            .checked_add(retirement_tree_internal_levels)
            .expect("退役那一项装得进 u64"),
    )""",
  """    let _ = retirement_tree_internal_levels;
    MetadataBlocks(0)"""),
 ("crates/singlefs-core/src/admission.rs",
  """    (MAPPED_UNITS_OF_THE_RETIREMENT_CONTAINER + MAPPED_UNITS_OF_THE_RETIREMENT_TREE_ROOT)
        .checked_add(retirement_tree_internal_levels)
        .expect("退役那几把装得进 u64")""",
  """    let _ = (retirement_tree_internal_levels, MAPPED_UNITS_OF_THE_RETIREMENT_CONTAINER, MAPPED_UNITS_OF_THE_RETIREMENT_TREE_ROOT);
    0"""),
 ("crates/singlefs-core/src/admission.rs",
  "        .and_then(|slots| slots.checked_add(retirement_tree_internal_levels))",
  "        .and_then(|slots| slots.checked_add(retirement_tree_internal_levels * 0))"),
]
