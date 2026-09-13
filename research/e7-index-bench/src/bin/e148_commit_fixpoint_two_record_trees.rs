//! E148：提交固定点按两棵记录树重算——D19 已定项 8 之后每分配一个块要往分配记录树与映射树各插一条记录，
//! 两棵树的脏节点又各要分配块；空发布（U = 0）的固定点就是 c_max 的候选，U = 12 是 ckpt_cost 的候选。
//! 模型逐字沿用 E81（提交的固定点）的放置三臂，只把「一棵记录树」推广成「若干棵」。
//! 判据与失败条款在 `research/prompts/e148-preregistration.md`，装置写之前。

use e7_index_bench::Emitter;

/// E81 的分配记录树几何：叶 512 条、扇出 256、65536 叶（≈ 1 TiB 池 / 32 KiB）。
const ALLOCATION_RECORDS_PER_LEAF: u64 = 512;
const ALLOCATION_FANOUT: u64 = 256;
const ALLOCATION_LEAF_COUNT: u64 = 65536;
/// D19 已定项 6：映射叶扇出 296；内部扇出 ⌊16205 / (27 + 83)⌋ = 147。
const MAPPING_RECORDS_PER_LEAF: u64 = 296;
const MAPPING_FANOUT: u64 = 147;
/// 池里的数据单元数 = 分配记录树的 key 空间（65536 × 512）。
const UNITS_IN_POOL: u64 = ALLOCATION_LEAF_COUNT * ALLOCATION_RECORDS_PER_LEAF;
/// E81：记账树每次发布的脏节点数。
const ACCOUNTING_NODES_PER_PUBLISH: u64 = 2;
const ROUNDS_UPPER_LIMIT: usize = 100_000;
const USER_BLOCK_COUNTS: [u64; 5] = [0, 1, 12, 128, 1024];
/// 第一个事务规模两棵树各只有一个叶（512 / 296 个 key），不建分裂 ⇒ 待分配的块数要装得进半个叶，1024 那一档跑不了。
const USER_BLOCK_COUNTS_FIRST_TRANSACTION: [u64; 4] = [0, 1, 12, 128];
const SEEDS: [u64; 5] = [11, 22, 33, 44, 55];
/// 目标负载一次 fsync 的用户块数（E81 的口径）。
const TARGET_LOAD_USER_BLOCKS: u64 = 12;

/// C59（种子折叠成同一个状态）教训：乘法混淆，不许 `seed | 1`。
struct XorshiftGenerator(u64);
impl XorshiftGenerator {
    fn new(seed: u64) -> Self {
        let mut xorshift_state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if xorshift_state == 0 {
            xorshift_state = 0xDEAD_BEEF;
        }
        XorshiftGenerator(xorshift_state)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, exclusive_upper_bound: u64) -> u64 {
        self.next() % exclusive_upper_bound
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Clustered,
    Scattered,
    MetadataClustered,
}
impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::Clustered => "clustered",
            Arm::Scattered => "scattered",
            Arm::MetadataClustered => "meta_clustered",
        }
    }
    fn user_blocks_are_clustered(self) -> bool {
        match self {
            Arm::Clustered => true,
            Arm::Scattered | Arm::MetadataClustered => false,
        }
    }
    fn commit_blocks_are_clustered(self) -> bool {
        match self {
            Arm::Clustered | Arm::MetadataClustered => true,
            Arm::Scattered => false,
        }
    }
}
const ARMS: [Arm; 3] = [Arm::Clustered, Arm::Scattered, Arm::MetadataClustered];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scale {
    PoolOneTebibyte,
    FirstTransaction,
}
impl Scale {
    fn tag(self) -> &'static str {
        match self {
            Scale::PoolOneTebibyte => "pool_1tib",
            Scale::FirstTransaction => "first_transaction",
        }
    }
    fn user_block_counts(self) -> &'static [u64] {
        match self {
            Scale::PoolOneTebibyte => &USER_BLOCK_COUNTS,
            Scale::FirstTransaction => &USER_BLOCK_COUNTS_FIRST_TRANSACTION,
        }
    }
}
const SCALES: [Scale; 2] = [Scale::PoolOneTebibyte, Scale::FirstTransaction];

/// 一棵记录树的几何：levels[0] = 叶层节点数。
#[derive(Clone, Debug)]
struct RecordTree {
    name: &'static str,
    records_per_leaf: u64,
    fanout: u64,
    levels: Vec<u64>,
}
impl RecordTree {
    fn new(name: &'static str, records_per_leaf: u64, fanout: u64, leaf_count: u64) -> RecordTree {
        let mut levels = vec![leaf_count];
        let mut nodes_in_level = leaf_count;
        while nodes_in_level > 1 {
            nodes_in_level = nodes_in_level.div_ceil(fanout);
            levels.push(nodes_in_level);
        }
        RecordTree { name, records_per_leaf, fanout, levels }
    }
    fn total_keys(&self) -> u64 {
        self.levels[0] * self.records_per_leaf
    }
}

fn record_trees(scale: Scale, tree_count: usize) -> Vec<RecordTree> {
    let (allocation_leaves, mapping_leaves) = match scale {
        Scale::PoolOneTebibyte => (ALLOCATION_LEAF_COUNT, UNITS_IN_POOL.div_ceil(MAPPING_RECORDS_PER_LEAF)),
        Scale::FirstTransaction => (1, 1),
    };
    let mut trees = vec![RecordTree::new("allocation", ALLOCATION_RECORDS_PER_LEAF, ALLOCATION_FANOUT, allocation_leaves)];
    if tree_count == 2 {
        trees.push(RecordTree::new("mapping", MAPPING_RECORDS_PER_LEAF, MAPPING_FANOUT, mapping_leaves));
    }
    assert_eq!(trees.len(), tree_count, "只建 1 或 2 棵记录树");
    trees
}

/// 每棵树自己的脏标记与聚簇游标（各树 key 空间独立）。
struct TreeState {
    dirty: Vec<Vec<bool>>,
    bump: u64,
    dirty_count: u64,
}

#[derive(Debug)]
struct Outcome {
    rounds: usize,
    converged: bool,
    dirty_per_tree: Vec<u64>,
    metadata_blocks: u64,
    records_inserted: u64,
    blocks_allocated: u64,
}

/// 聚簇段的游标从 key 空间中部起（E81 的口径：中部一个空段），不从 0 起。
fn initial_cursor(tree: &RecordTree) -> u64 {
    tree.total_keys() / 2
}

/// 一个块在某棵树的 key 空间里落到哪：聚簇走游标，散布走随机。
fn placement_key(tree: &RecordTree, state: &mut TreeState, clustered: bool, random_generator: &mut XorshiftGenerator) -> u64 {
    if clustered {
        assert!(state.bump < tree.total_keys(), "聚簇游标越过 {} 的 key 空间：这一档规模装不下这么多块，不建分裂", tree.name);
        state.bump += 1;
        state.bump - 1
    } else {
        random_generator.below(tree.total_keys())
    }
}

/// 跑一次发布的固定点：每个待分配的块往每棵树各插一条记录，新脏的节点各要一个块，下一轮再插。
fn publish(trees: &[RecordTree], arm: Arm, user_block_count: u64, accounting_node_count: u64, seed: u64) -> Outcome {
    let mut random_generator = XorshiftGenerator::new(seed);
    let mut states: Vec<TreeState> = trees
        .iter()
        .map(|tree| TreeState {
            dirty: tree.levels.iter().map(|&node_count| vec![false; node_count as usize]).collect(),
            bump: initial_cursor(tree),
            dirty_count: 0,
        })
        .collect();
    let mut records_inserted = 0u64;
    let mut blocks_allocated = 0u64;
    // 待分配集里的每一项是「这个块在每棵树里的 key」。
    let mut pending: Vec<Vec<u64>> = Vec::new();
    for _ in 0..user_block_count {
        blocks_allocated += 1;
        pending.push(trees.iter().zip(states.iter_mut()).map(|(tree, state)| placement_key(tree, state, arm.user_blocks_are_clustered(), &mut random_generator)).collect());
    }
    for _ in 0..accounting_node_count {
        blocks_allocated += 1;
        pending.push(trees.iter().zip(states.iter_mut()).map(|(tree, state)| placement_key(tree, state, arm.commit_blocks_are_clustered(), &mut random_generator)).collect());
    }
    let mut rounds = 0;
    let mut converged = false;
    while rounds < ROUNDS_UPPER_LIMIT {
        if pending.is_empty() {
            converged = true;
            break;
        }
        rounds += 1;
        let mut newly_dirty_count = 0u64;
        for keys in &pending {
            for ((tree, state), &key) in trees.iter().zip(states.iter_mut()).zip(keys.iter()) {
                records_inserted += 1;
                let mut node_index = key / tree.records_per_leaf;
                for level in state.dirty.iter_mut() {
                    if !level[node_index as usize] {
                        level[node_index as usize] = true;
                        newly_dirty_count += 1;
                        state.dirty_count += 1;
                    }
                    node_index /= tree.fanout;
                }
            }
        }
        pending = (0..newly_dirty_count)
            .map(|_| {
                blocks_allocated += 1;
                trees.iter().zip(states.iter_mut()).map(|(tree, state)| placement_key(tree, state, arm.commit_blocks_are_clustered(), &mut random_generator)).collect()
            })
            .collect();
    }
    let dirty_per_tree: Vec<u64> = states.iter().map(|state| state.dirty_count).collect();
    let metadata_blocks = dirty_per_tree.iter().sum::<u64>() + accounting_node_count;
    Outcome { rounds, converged, dirty_per_tree, metadata_blocks, records_inserted, blocks_allocated }
}

fn emit(emitter: &mut Emitter, line: &str) {
    println!("{}", emitter.emit_raw(line));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!(
        "name=config allocation_leaf_cap={ALLOCATION_RECORDS_PER_LEAF} allocation_fanout={ALLOCATION_FANOUT} allocation_leaves={ALLOCATION_LEAF_COUNT} mapping_leaf_cap={MAPPING_RECORDS_PER_LEAF} mapping_fanout={MAPPING_FANOUT} mapping_leaves={} acc_nodes={ACCOUNTING_NODES_PER_PUBLISH} model=counting",
        UNITS_IN_POOL.div_ceil(MAPPING_RECORDS_PER_LEAF)
    ));
    for scale in SCALES {
        for tree_count in [1usize, 2] {
            let trees = record_trees(scale, tree_count);
            emit(&mut emitter, &format!(
                "name=geometry scale={} trees={tree_count} levels={}",
                scale.tag(),
                trees.iter().map(|tree| format!("{}:{}", tree.name, tree.levels.len())).collect::<Vec<_>>().join(",")
            ));
            for arm in ARMS {
                for &user_block_count in scale.user_block_counts() {
                    for seed in SEEDS {
                        let outcome = publish(&trees, arm, user_block_count, ACCOUNTING_NODES_PER_PUBLISH, seed);
                        assert!(outcome.converged, "必须收敛");
                        assert_eq!(outcome.records_inserted, outcome.blocks_allocated * tree_count as u64, "守恒破了");
                        let dirty: Vec<String> = trees.iter().zip(&outcome.dirty_per_tree).map(|(tree, dirty)| format!("{}={dirty}", tree.name)).collect();
                        emit(&mut emitter, &format!(
                            "name=fixpoint scale={} trees={tree_count} arm={} u={user_block_count} seed={seed} rounds={} dirty={} meta_blocks={} journal_items={}",
                            scale.tag(), arm.tag(), outcome.rounds, dirty.join(","), outcome.metadata_blocks, user_block_count + outcome.metadata_blocks
                        ));
                        if arm != Arm::Scattered {
                            break; // 确定性臂只报一个种子
                        }
                    }
                }
            }
        }
        // c_max 与 ckpt_cost 的候选：两棵树、空发布与目标负载。
        let trees = record_trees(scale, 2);
        for arm in ARMS {
            let empty = publish(&trees, arm, 0, ACCOUNTING_NODES_PER_PUBLISH, SEEDS[0]);
            let target = publish(&trees, arm, TARGET_LOAD_USER_BLOCKS, ACCOUNTING_NODES_PER_PUBLISH, SEEDS[0]);
            emit(&mut emitter, &format!(
                "name=c_max scale={} arm={} empty_publish_meta_blocks={} target_load_meta_blocks={}",
                scale.tag(), arm.tag(), empty.metadata_blocks, target.metadata_blocks
            ));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 手算锚点：两棵相同的小树（叶 4 条、扇出 4、8 叶 ⇒ 3 层，32 个 key，游标从 16 起），U = 1，无记账，clustered。
    /// 第 1 轮 key 16 ⇒ 每棵树叶 4 + 内部 1 + 根 = 3 新 ⇒ 6 块，key 17..22；第 2 轮 17–19 落叶 4（已脏）、20–22 落叶 5（新）⇒ 每树 1 ⇒ 2 块，key 23、24；
    /// 第 3 轮 23 落叶 5（已脏）、24 落叶 6（新）⇒ 每树 1 ⇒ 2 块，key 25、26；第 4 轮都落叶 6 ⇒ 0 新 ⇒ 收敛。
    #[test]
    fn hand_case_two_small_trees() {
        let trees = vec![RecordTree::new("a", 4, 4, 8), RecordTree::new("b", 4, 4, 8)];
        let outcome = publish(&trees, Arm::Clustered, 1, 0, 1);
        assert!(outcome.converged);
        assert_eq!(outcome.rounds, 4);
        assert_eq!(outcome.dirty_per_tree, vec![5, 5]);
        assert_eq!(outcome.metadata_blocks, 10);
        assert_eq!(outcome.blocks_allocated, 11);
        assert_eq!(outcome.records_inserted, 22);
    }

    /// 同一几何只有一棵树：第 1 轮 3 新 ⇒ 3 块，key 17–19 落叶 4 ⇒ 0 新 ⇒ 2 轮、脏 3、块 4。
    #[test]
    fn hand_case_one_small_tree() {
        let trees = vec![RecordTree::new("a", 4, 4, 8)];
        let outcome = publish(&trees, Arm::Clustered, 1, 0, 1);
        assert_eq!((outcome.rounds, outcome.dirty_per_tree.clone(), outcome.blocks_allocated), (2, vec![3], 4));
    }

    /// 阳性对照（条款 c）：一棵树在 E81 几何上 U = 12、clustered 必须复现 E81 的 2 轮、5 块。
    #[test]
    fn single_tree_reproduces_e81_target_load() {
        let trees = record_trees(Scale::PoolOneTebibyte, 1);
        assert_eq!(trees[0].levels.len(), 3);
        let outcome = publish(&trees, Arm::Clustered, 12, ACCOUNTING_NODES_PER_PUBLISH, 11);
        assert_eq!((outcome.rounds, outcome.metadata_blocks), (2, 5));
        let scattered = publish(&trees, Arm::Scattered, 12, ACCOUNTING_NODES_PER_PUBLISH, 11);
        assert!(scattered.metadata_blocks >= 5 * outcome.metadata_blocks, "机制不在场：{}", scattered.metadata_blocks);
    }

    /// 跑前写死的解析预测：池规模两棵树 clustered，U = 12 与 U = 0 都是 2 轮、9 块；映射树 4 层。
    #[test]
    fn two_trees_at_pool_scale_are_pinned() {
        let trees = record_trees(Scale::PoolOneTebibyte, 2);
        assert_eq!(trees[1].levels.len(), 4, "113 360 → 772 → 6 → 1");
        assert_eq!(trees[1].levels[0], 113_360);
        let target = publish(&trees, Arm::Clustered, 12, ACCOUNTING_NODES_PER_PUBLISH, 11);
        assert_eq!((target.rounds, target.dirty_per_tree.clone(), target.metadata_blocks), (2, vec![3, 4], 9));
        let empty = publish(&trees, Arm::Clustered, 0, ACCOUNTING_NODES_PER_PUBLISH, 11);
        assert_eq!((empty.rounds, empty.metadata_blocks), (2, 9));
    }

    /// 第一个事务规模：两棵树各一个叶，空发布与目标负载都是 1 + 1 + 2 = 4 块。
    #[test]
    fn first_transaction_scale_is_pinned() {
        let trees = record_trees(Scale::FirstTransaction, 2);
        assert!(trees.iter().all(|tree| tree.levels.len() == 1));
        for user_block_count in [0u64, 12] {
            let outcome = publish(&trees, Arm::Clustered, user_block_count, ACCOUNTING_NODES_PER_PUBLISH, 11);
            assert_eq!(outcome.metadata_blocks, 4, "U={user_block_count}");
        }
    }

    /// 游标从 key 空间中部起：8 叶 × 4 条的小树是 16，E81 几何是 33 554 432 / 2。
    #[test]
    fn clustered_cursor_starts_at_the_middle_of_the_key_space() {
        assert_eq!(initial_cursor(&RecordTree::new("a", 4, 4, 8)), 16);
        assert_eq!(initial_cursor(&record_trees(Scale::PoolOneTebibyte, 1)[0]), 16_777_216);
    }

    /// 三臂的次序在 U = 128 上分得开：只聚簇提交块的臂比全聚簇多脏叶（用户块随机散在 65536 个叶上），又比全散布少。
    #[test]
    fn meta_clustered_sits_strictly_between_clustered_and_scattered() {
        let trees = record_trees(Scale::PoolOneTebibyte, 2);
        let clustered = publish(&trees, Arm::Clustered, 128, ACCOUNTING_NODES_PER_PUBLISH, 33);
        let meta_clustered = publish(&trees, Arm::MetadataClustered, 128, ACCOUNTING_NODES_PER_PUBLISH, 33);
        let scattered = publish(&trees, Arm::Scattered, 128, ACCOUNTING_NODES_PER_PUBLISH, 33);
        assert!(clustered.metadata_blocks < meta_clustered.metadata_blocks, "{} vs {}", clustered.metadata_blocks, meta_clustered.metadata_blocks);
        assert!(meta_clustered.metadata_blocks < scattered.metadata_blocks, "{} vs {}", meta_clustered.metadata_blocks, scattered.metadata_blocks);
    }

    /// 守恒（条款 b）：插记录数 = 分配块数 × 树数，三臂都验。
    #[test]
    fn conservation_holds_on_every_arm() {
        let trees = record_trees(Scale::PoolOneTebibyte, 2);
        for arm in ARMS {
            let outcome = publish(&trees, arm, 128, ACCOUNTING_NODES_PER_PUBLISH, 22);
            assert!(outcome.converged, "{:?}", arm);
            assert_eq!(outcome.records_inserted, outcome.blocks_allocated * 2, "{:?}", arm);
        }
    }

    /// 映射树的内部扇出与叶扇出按 D19 已定项 6 钉住。
    #[test]
    fn mapping_tree_geometry_matches_the_knowledge_base() {
        assert_eq!(MAPPING_RECORDS_PER_LEAF, 296);
        assert_eq!(MAPPING_FANOUT, 16205 / (27 + 83));
        assert_eq!(UNITS_IN_POOL, 33_554_432);
    }
}
