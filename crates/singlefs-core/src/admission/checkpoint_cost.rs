//! checkpoint 代价（ckpt_cost）：一个谱系容器、一个退役容器写时复制的块数与映射条目，下一版要建在其上的那一版的代价，
//! 记账树、分配记录树与中央映射树在一次空发布里至多重写的节点。

use super::*;

/// ckpt_cost 里谱系容器那一项的固定部分（D28（挂载期承诺量） 已定项 4「谱系容器单列一项，按一次谱系容器 COW 算」）：
/// 一个谱系容器（32 KiB 码 3，2 块）加谱系树根（1 块）；谱系树的内部层数另加
/// （[`checkpoint_cost_of_one_lineage_container_copy_on_write`]）。
pub const LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
pub const LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE: u64 = 1;

/// ckpt_cost 里谱系容器那一项：一次谱系容器 COW 连路径 = 容器 2 + 谱系树根 1 + 谱系树内部层数（根节点码 2 头里现读的层级减一，
/// 零根与只有根时 0）。零根时也是这么多：回退那次在零根上开第一片容器、写第一个根。
///
/// # Panics
/// 加出来装不进 u64：内部层数至多十来层。
#[must_use]
pub fn checkpoint_cost_of_one_lineage_container_copy_on_write(
    lineage_tree_internal_levels: u64,
) -> MetadataBlocks {
    MetadataBlocks(
        (LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE
            + LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE)
            .checked_add(lineage_tree_internal_levels)
            .expect("谱系那一项装得进 u64"),
    )
}

/// 一次谱系容器 COW 连祖先在中央映射树里改的条目数（每把至多删一次旧的、插一次新的）：谱系容器、谱系树根与每层内部节点各是一个进映射的单元、
/// 各一把 key（容器是一个 32 KiB 码 3 单元，一把 key，不按块数），所以 = 1 + 1 + 内部层数。ckpt_cost 中央映射树那一项的删插数把它们
/// 一并算上（D28（挂载期承诺量） 已定项 4 的删插数原文只写了「分配记录树这一项 + 记账树节点数」）。
///
/// # Panics
/// 加出来装不进 u64：内部层数至多十来层。
#[must_use]
pub fn central_mapping_entries_of_one_lineage_container_copy_on_write(
    lineage_tree_internal_levels: u64,
) -> u64 {
    const MAPPED_UNITS_OF_THE_LINEAGE_CONTAINER: u64 = 1;
    const MAPPED_UNITS_OF_THE_LINEAGE_TREE_ROOT: u64 = 1;
    (MAPPED_UNITS_OF_THE_LINEAGE_CONTAINER + MAPPED_UNITS_OF_THE_LINEAGE_TREE_ROOT)
        .checked_add(lineage_tree_internal_levels)
        .expect("谱系那几把装得进 u64")
}

/// ckpt_cost 里退役容器那一项的固定部分（「两样都做」：
/// 照谱系在 ckpt_cost 里单列一项）：一个退役容器（32 KiB 码 3，2 块）加退役树根（1 块）；退役树的内部层数另加
/// （[`checkpoint_cost_of_one_retirement_container_copy_on_write`]）。
pub const RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
pub const RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE: u64 = 1;

/// ckpt_cost 里退役容器那一项：一次退役容器 COW 连路径 = 容器 2 + 退役树根 1 + 退役树内部层数（根节点码 2 头里现读的层级减一，
/// 零根与只有根时 0），同谱系那一项（[`checkpoint_cost_of_one_lineage_container_copy_on_write`]）的算法。
///
/// # Panics
/// 加出来装不进 u64：内部层数至多十来层。
#[must_use]
pub fn checkpoint_cost_of_one_retirement_container_copy_on_write(
    retirement_tree_internal_levels: u64,
) -> MetadataBlocks {
    MetadataBlocks(
        (RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE
            + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE)
            .checked_add(retirement_tree_internal_levels)
            .expect("退役那一项装得进 u64"),
    )
}

/// 一次退役容器 COW 连祖先在中央映射树里改的条目数：退役容器、退役树根与每层内部节点各一把 key，= 1 + 1 + 内部层数
///
/// # Panics
/// 加出来装不进 u64：内部层数至多十来层。
#[must_use]
pub fn central_mapping_entries_of_one_retirement_container_copy_on_write(
    retirement_tree_internal_levels: u64,
) -> u64 {
    const MAPPED_UNITS_OF_THE_RETIREMENT_CONTAINER: u64 = 1;
    const MAPPED_UNITS_OF_THE_RETIREMENT_TREE_ROOT: u64 = 1;
    (MAPPED_UNITS_OF_THE_RETIREMENT_CONTAINER + MAPPED_UNITS_OF_THE_RETIREMENT_TREE_ROOT)
        .checked_add(retirement_tree_internal_levels)
        .expect("退役那几把装得进 u64")
}

/// checkpoint 保留池的 ckpt_cost（D28（挂载期承诺量） 已定项 4）：一次空发布至多写出的固定点单元数，按这次发布要接在后面的那一版的结构现算，
/// 一律按最坏情况计：
/// - 分配记录树：几何上整棵树，1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1} 这块盘单元区在层级 L 上罩到的位置数
///   （[`allocation_record_tree_positions_over_the_unit_areas`]）；
/// - 中央映射树：逐层按这次可能改的路径数与可能切出来的节点数计（[`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`]）；
/// - 记账树：这次发布写完之后那棵记账树的节点数（[`accounting_tree_nodes_after_the_publish_built_on`]：行数至多是这一版的行数加
///   这次新留的 1 级 inode 号水位台阶，按写路径同一个规划器、按 `node_capacities` 的记账树容量现算）；
/// - 树表 1（每次发布重写一个单元，D16（发布语义） 已定项 9）；实例表链不进（它的开销归已定项 3 的切换预留）；
/// - 谱系容器单列一项，按一次谱系容器 COW 连路径算（[`checkpoint_cost_of_one_lineage_container_copy_on_write`]，只算带文件的一版：
///   树表 0 条的一版没有谱系树，也走不到管理员回退）；
/// - 退役容器照谱系单列一项（[`checkpoint_cost_of_one_retirement_container_copy_on_write`]
///   第六节 R2「两样都做」，只算带文件的一版），它在中央映射树里的删插同样算上。
///
/// 「当前」= 这次发布要接在后面的那一版：
/// - 带文件的一版：分配记录树与中央映射树的高从各自根节点的码 2 头里现读（层级 + 1，不用内存里另存一份，已定项 4）——分配记录树那一项
///   按读出来的根层级数每一层的位置；中央映射树每层的节点数从这一版的形状里取。
/// - 树表 0 条的一版（`None`）：没有中央映射树与记账树（0 与 0）；分配记录树只在那一版写过行时有（分配器记着它，
///   `PoolAllocator::allocation_record_tree_of_the_version_without_file`），它按位置寻址、根的层级由池几何定（D8（核心索引结构） 已定项 14），
///   根层级取几何的——这一处分配器里只有节点与指针、没有根节点的字节可读；没写过行的（mkfs 的第 0 代）一棵都没有，取 0。
///
/// 中央映射树的节点容量按节点格式算（产品路径）；只供测试的压小容量下量同一个数用
/// [`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`]。
///
/// 分配记录树那一项不读这一版有哪些节点：按「每块盘两条叶路径」算，在开放段装不下开新段、没有全空段回落（先取这次发布重写集的叶，
/// 那些叶里没有每块盘都空的落点再取每块盘都空的槽最多的叶，落点可以在单元区的任何一片叶里）、换下很久之前落在别处的节点这三种情形下少扣，
/// 一次发布改几片叶的不动点收不了口，拿得准的上界只有几何上整棵树。代价随单元区线性涨（两块 4 GiB 盘 529 个节点）。
///
/// 同一个数也是挂载期承诺量里暖机那一半的 c_max（已定项 4 末条「按同一个现算的 c_max 取」）。
#[must_use]
pub fn checkpoint_cost_of_the_version_to_build_on(
    version_to_build_on: Option<&TransactionOutput>,
    allocator: &PoolAllocator,
) -> MetadataBlocks {
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        version_to_build_on,
        allocator,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    )
}

/// 同 [`checkpoint_cost_of_the_version_to_build_on`]，中央映射树与记账树的节点容量按 `node_capacities` 取：写入口装着只供测试的压小容量时，
/// 它的空发布按压小的容量切分，量它的上界要按同一个容量算。产品路径（发布与可写挂载的准入）按节点格式调上面那一个。
#[must_use]
pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
    version_to_build_on: Option<&TransactionOutput>,
    allocator: &PoolAllocator,
    node_capacities: CodeTwoTreeNodeCapacities,
) -> MetadataBlocks {
    const TREE_TABLE_UNITS_PER_PUBLISH: u64 = 1;
    let record_trees_and_accounting_nodes = match version_to_build_on {
        Some(file_version) => {
            let root_level_read_from_the_root_node_header = u8::try_from(
                file_version
                    .position_addressed_tree_heights_read_from_the_root_node_headers()
                    .allocation_record_tree
                    .checked_sub(1)
                    .expect("树高是根节点头里的层级 + 1，至少 1"),
            )
            .expect("码 2 头的层级是 1 字节");
            let allocation_record_tree_nodes = allocation_record_tree_positions_over_the_unit_areas(
                root_level_read_from_the_root_node_header,
                allocator,
            );
            let accounting_tree_nodes_of_this_version =
                u64::try_from(file_version.accounting_tree.node_count())
                    .expect("记账树的节点数装得进 u64");
            let accounting_tree_nodes = accounting_tree_nodes_after_the_publish_built_on(
                file_version,
                node_capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
            );
            let lineage_tree_internal_levels = file_version.lineage_tree_internal_levels();
            let retirement_tree_internal_levels = file_version.retirement_tree_internal_levels();
            // 一次空发布在中央映射树里改的条目是这两棵树的节点（别的进映射的单元照抄上一版、key 不变）：重写的每个节点删一把旧 key
            // （上一版有它时）、插一把这次的 key。再加回退那次谱系容器 COW 连祖先改的那几把：谱系那一项按一次 COW 连祖先算
            // （D28（挂载期承诺量） 已定项 4），它在中央映射树里的删插一并算上。
            // 记账树的删至多是这一版的节点数、插至多是这次之后的节点数：两者在台阶行跨过一片叶的那次不同。
            let mapped_changes_with_accounting_nodes = |accounting_nodes: u64| {
                allocation_record_tree_nodes
                    .checked_add(accounting_nodes)
                    .and_then(|nodes| {
                        nodes.checked_add(
                            central_mapping_entries_of_one_lineage_container_copy_on_write(
                                lineage_tree_internal_levels,
                            ),
                        )
                    })
                    .and_then(|nodes| {
                        nodes.checked_add(
                            central_mapping_entries_of_one_retirement_container_copy_on_write(
                                retirement_tree_internal_levels,
                            ),
                        )
                    })
                    .expect("两棵树重写的节点数与谱系、退役那几把装得进 u64")
            };
            let central_mapping_tree_nodes =
                central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                    &central_mapping_tree_nodes_at_each_level(
                        &file_version.central_mapping_tree.shape,
                        file_version.height_read_from_the_root_node_header(
                            MultiLevelCodeTwoTree::CentralMapping,
                        ),
                    ),
                    // 映射条目数 = 进映射的单元 + 被活快照钉住、留存的映射条目：空发布两样都照抄。
                    u64::try_from(
                        file_version.mapped_units.len()
                            + file_version.mapping_entries_kept_for_the_snapshots.len(),
                    )
                    .expect("映射条目数装得进 u64"),
                    CentralMappingEntryChangesOfAnEmptyPublish {
                        deleted_at_most: mapped_changes_with_accounting_nodes(
                            accounting_tree_nodes_of_this_version,
                        ),
                        inserted_at_most: mapped_changes_with_accounting_nodes(
                            accounting_tree_nodes,
                        ),
                    },
                    node_capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
                );
            let lineage_container_copy_on_write =
                checkpoint_cost_of_one_lineage_container_copy_on_write(
                    lineage_tree_internal_levels,
                )
                .0;
            let retirement_container_copy_on_write =
                checkpoint_cost_of_one_retirement_container_copy_on_write(
                    retirement_tree_internal_levels,
                )
                .0;
            [
                allocation_record_tree_nodes,
                central_mapping_tree_nodes,
                accounting_tree_nodes,
                lineage_container_copy_on_write,
                retirement_container_copy_on_write,
            ]
            .into_iter()
            .try_fold(0u64, u64::checked_add)
            .expect("三棵树的节点数与谱系、退役那两项加起来装得进 u64")
        }
        None => match allocator.allocation_record_tree_of_the_version_without_file() {
            Some(_) => allocation_record_tree_positions_over_the_unit_areas(
                AllocationRecordTreeGeometry::of_allocator(allocator).root_level(),
                allocator,
            ),
            None => 0,
        },
    };
    MetadataBlocks(record_trees_and_accounting_nodes + TREE_TABLE_UNITS_PER_PUBLISH)
}

/// ckpt_cost 里记账树那一项（D28（挂载期承诺量） 已定项 4「记账树每发布的节点数」）：接在 `version_to_build_on` 后面的那次发布写完之后
/// 那棵记账树的节点数，记账树节点容量取 `capacity`。
///
/// 那次发布的记账行与它做什么无关（空发布、覆盖写、建 inode、管理员回退都一样，`transaction` 装记账行那一段）：池级三行加每块盘六行，
/// 代段换成那次的 txg，加照抄的 inode 号水位台阶行（[`TransactionOutput::inode_number_watermark_steps`]）。行数因此至多是这一版的行数加 1：
/// 这一版自己那一行是一级台阶时，下一次照抄它、再写自己那一行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。
/// 形状按写路径同一个规划器从这一版的形状现算（`code_two_tree::plan_the_tree_after_this_publish`）：台阶行正好跨过一片叶的容量那次，
/// 那次的树比这一版多节点（产品容量两块盘时第 463 级台阶起 +2，同一条射程），只拿这一版的节点数会少算。
/// 那次的代段只要大于这一版的 txg 就排在同一处，取这一版的 txg + 1（D16（发布语义） 已定项 6）。
///
/// 规划器拒（树要长到 257 层，或这一版的形状按分隔 key 走不到它自己叶里的 key）时，接在这一版后面的发布拿同一个形状、同一组 key 规划，
/// 同样在任何落盘之前拒、一个记账树节点都不写：这一项取这一版的节点数。
///
/// # Panics
/// 这一版的 txg 加 1 装不进 u64；或算出来的行数多于这一版的行数加 1（照抄的台阶行代段都小于这一版的 txg，与这一版那一组行不重 key）。
#[must_use]
pub fn accounting_tree_nodes_after_the_publish_built_on(
    version_to_build_on: &TransactionOutput,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    let version_txg = version_to_build_on.root.checkpoint_txg;
    let txg_of_the_publish_built_on_it = CheckpointTxg(
        version_txg
            .0
            .checked_add(1)
            .expect("checkpoint 号加 1 装得进 u64"),
    );
    let rows_written_by_that_publish = version_to_build_on
        .accounting_entries
        .iter()
        .filter(|entry| entry.generation == version_txg)
        .map(|entry| AccountingEntry {
            statistic: entry.statistic,
            tree: entry.tree,
            device: entry.device,
            generation: txg_of_the_publish_built_on_it,
            value: entry.value,
            sequence: entry.sequence,
        });
    let keys_after_that_publish: BTreeSet<CodeTwoTreeKey> = rows_written_by_that_publish
        .chain(version_to_build_on.inode_number_watermark_steps())
        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
        .collect();
    assert!(
        keys_after_that_publish.len() <= version_to_build_on.accounting_entries.len() + 1,
        "接在这一版后面的发布至多比这一版多 1 行记账（新留的那一级台阶）：算出 {} 行，这一版 {} 行",
        keys_after_that_publish.len(),
        version_to_build_on.accounting_entries.len()
    );
    let nodes = match plan_the_tree_after_this_publish(
        &version_to_build_on.accounting_tree.shape,
        &keys_after_that_publish,
        capacity,
    ) {
        Ok(plan) => plan.shape.nodes().len(),
        Err(_refused_before_any_write) => version_to_build_on.accounting_tree.node_count(),
    };
    u64::try_from(nodes).expect("记账树的节点数装得进 u64")
}

/// 分配记录树一次空发布至多重写的节点数，取几何上整棵树（K1；D8（核心索引结构） 已定项 14：按位置寻址、根里按盘分路、
/// 根之下每个节点只属于一块盘）：
/// 1（根，罩整个 key 空间）+ Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），
/// S_L = 812 × 169^L（[`span_in_slots_at_level`]，两个数是 `singlefs_format` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS` 与
/// `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`）。分配记录只落在单元区里，一次发布重写的节点都是这一版之后那棵树的节点，
/// 而那棵树每块盘在根之下至多这么多个位置——所以恒是上界，不读这一版有哪些节点、不依赖 ckpt_cost，O(盘数 × 根层级) 算完。
/// 两块 4 GiB 盘（单元区 [50176, 262144)、根层级 2）：每块盘叶 262 个、层级 1 两个，1 + 2 × 264 = 529；
/// 单元区 240 / 256 / 384 槽的窄盘每块盘只罩叶 61、62 两片，1 + 2 × 2 = 5（与每块盘两条叶路径那一版同值）。
///
/// # Panics
/// 根层级是 0（分配记录树的根层级至少 1，`AllocationRecordTreeGeometry::of_devices`）；有一块盘的单元区一槽都没有
/// （mkfs 把实例表与树表写在单元区里，拒单元区在盘外的几何）；或加起来装不进 u64。
fn allocation_record_tree_positions_over_the_unit_areas(
    root_level: u8,
    allocator: &PoolAllocator,
) -> u64 {
    assert!(
        root_level >= 1,
        "分配记录树的根层级至少 1：根装每块盘上根层级 − 1 那一层的格"
    );
    const THE_ROOT_COVERING_THE_WHOLE_KEY_SPACE: u64 = 1;
    allocation_record_tree_positions_at_each_level_below_the_root(root_level, allocator)
        .into_iter()
        .try_fold(THE_ROOT_COVERING_THE_WHOLE_KEY_SPACE, u64::checked_add)
        .expect("整棵树的位置数装得进 u64：槽号至多 2^48")
}

/// 分配记录树根之下每一层（层级 0 到根层级 − 1，按层级升序）全部盘的单元区合计罩到的位置数——D28（挂载期承诺量） 已定项 1
/// W 式子里的 N_ℓ（「分配记录树第 ℓ 层两块盘合计罩到的位置数」）。每块盘在层级 ℓ 上罩到 ⌊末槽 ÷ S_ℓ⌋ − ⌊首槽 ÷ S_ℓ⌋ + 1 格。
pub(super) fn allocation_record_tree_positions_at_each_level_below_the_root(
    root_level: u8,
    allocator: &PoolAllocator,
) -> Vec<u64> {
    let mut positions_at_each_level = vec![0_u64; usize::from(root_level)];
    // 迭代上界：盘数 × 根层级（根层级至多 6，`AllocationRecordTreeGeometry::of_devices`）；跨轮只累加 `positions_at_each_level`。
    for device_map in &allocator.devices {
        let first_slot_of_the_unit_area = device_map.unit_area_start().slot().0;
        let last_slot_of_the_unit_area = first_slot_of_the_unit_area
            + device_map
                .unit_area_slots()
                .checked_sub(1)
                .expect("单元区至少一槽：mkfs 把实例表与树表写在单元区里");
        for level in 0..root_level {
            let span_at_this_level = span_in_slots_at_level(level);
            let positions_at_this_level = last_slot_of_the_unit_area / span_at_this_level
                - first_slot_of_the_unit_area / span_at_this_level
                + 1;
            let positions_of_this_level = &mut positions_at_each_level[usize::from(level)];
            *positions_of_this_level = positions_of_this_level
                .checked_add(positions_at_this_level)
                .expect("一层的位置数装得进 u64：槽号至多 2^48");
        }
    }
    positions_at_each_level
}

/// 一次空发布在中央映射树里至多删几把 key、插几把 key。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CentralMappingEntryChangesOfAnEmptyPublish {
    deleted_at_most: u64,
    inserted_at_most: u64,
}

/// 中央映射树这一版每层有几个节点，自叶（层级 0）到根，共 `height` 层（根节点头里现读的高）。形状里的节点按 (层级, 层内序号) 升序排，
/// 每层的边界二分找，代价是树高 × log(节点数)，不逐个数。
///
/// # Panics
/// 形状的高与根节点头里读出来的高不等，或有一层一个节点都没有：两样出自同一次发布（或同一次从盘上重建），对不上说明造它的一方写错了。
fn central_mapping_tree_nodes_at_each_level(shape: &CodeTwoTreeShape, height: u64) -> Vec<u64> {
    assert_eq!(
        shape.height(),
        height,
        "中央映射树形状的高与根节点头里的层级 + 1 相等：出自同一次发布或同一次重建"
    );
    let nodes = shape.nodes();
    (0..height)
        .map(|level| {
            let level = u8::try_from(level).expect("码 2 头的层级是 1 字节");
            let first = nodes.partition_point(|node| node.position.level < level);
            let past_the_last = nodes.partition_point(|node| node.position.level <= level);
            let nodes_at_this_level =
                u64::try_from(past_the_last - first).expect("一层的节点数装得进 u64");
            assert!(
                nodes_at_this_level > 0,
                "中央映射树第 {level} 层至少一个节点：高是 {height}"
            );
            nodes_at_this_level
        })
        .collect()
}

/// 一层上至多切几次（D8（核心索引结构） 已定项 11 ②：装不下时从中间切，左半 ⌈n ÷ 2⌉、右半其余）：
/// `additions` 是这一次往这一层加的条目数（叶层是插的 key，内部层是下一层切出来的孩子），`pre_existing_nodes` 是这一层原有的节点数。
/// 原有的节点第一次切至少要加 1 条（它可能正好装满）；切出来的两半都至少空着「⌊(容量 + 1) ÷ 2⌋ − 1」格，
/// 再切一次至少要加 `additions_per_split_of_a_fresh_node` = ⌊(容量 + 1) ÷ 2⌋ 条。先删后插（`code_two_tree::plan_the_tree_after_this_publish`），
/// 删只让节点更空。于是切的次数 ≤ min(原有节点数, 加的条数) + 余下的条数 ÷ 那个数。
fn splits_at_one_level_at_most(
    pre_existing_nodes: u64,
    additions: u64,
    additions_per_split_of_a_fresh_node: u64,
) -> u64 {
    let first_splits_of_pre_existing_nodes = pre_existing_nodes.min(additions);
    first_splits_of_pre_existing_nodes
        + (additions - first_splits_of_pre_existing_nodes) / additions_per_split_of_a_fresh_node
}

/// 中央映射树一次空发布至多重写的节点数：
/// - 每一层原有的节点里，这次重写的都在某一把删掉或插进的 key 的路径上（叶到根整条 COW，D8（核心索引结构） 已定项 11 ①），
///   一把 key 一层一个 ⇒ 至多 min(这一层的节点数, 删的 + 插的)；
/// - 切出来的节点都是新写的：叶层至多 [`splits_at_one_level_at_most`]（加的是插的 key），内部层同一条（加的是下一层切出来的孩子）；
///   这一层装得下下一层全部节点时（叶层：全树的条目加插的装得进一片叶）一次都切不了；
/// - 根切开就长高一层：新根一个，它起步两个孩子、之后下一层每多切一次多一个孩子，装不下再切、再长高。
///
/// # Panics
/// `nodes_at_each_level` 是空的（带文件的一版中央映射树至少一个节点），或加起来装不进 u64。
fn central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
    nodes_at_each_level: &[u64],
    entries_before_the_publish: u64,
    changes: CentralMappingEntryChangesOfAnEmptyPublish,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    assert!(
        !nodes_at_each_level.is_empty(),
        "带文件的一版中央映射树至少一个节点"
    );
    let leaf_entries = u64::try_from(capacity.leaf_entries).expect("叶容量装得进 u64");
    let internal_entries =
        u64::try_from(capacity.internal_entries).expect("内部节点容量装得进 u64");
    // ⌊(容量 + 1) ÷ 2⌋ = ⌈容量 ÷ 2⌉：切开之后较空的那一半再切一次至少要加的条数。
    let leaf_additions_per_split_of_a_fresh_node = leaf_entries.div_ceil(2);
    let internal_additions_per_split_of_a_fresh_node = internal_entries.div_ceil(2);
    assert!(
        leaf_additions_per_split_of_a_fresh_node >= 1 && internal_entries >= 2,
        "叶至少装 1 条、内部节点至少装 2 个孩子（`code_two_tree::plan_the_tree_after_this_publish` 同一条断言）"
    );
    let changed_paths = changes
        .deleted_at_most
        .checked_add(changes.inserted_at_most)
        .expect("删的加插的装得进 u64");
    let mut nodes_rewritten: u64 = 0;
    let mut splits_at_the_level_below: u64 = 0;
    // 迭代上界是这一版的高：一层一轮。
    for (level, nodes_at_this_level) in nodes_at_each_level.iter().copied().enumerate() {
        let splits_at_this_level = match level {
            0 => {
                let entries_at_most = entries_before_the_publish
                    .checked_add(changes.inserted_at_most)
                    .expect("条目数加插的装得进 u64");
                if entries_at_most <= leaf_entries {
                    0
                } else {
                    splits_at_one_level_at_most(
                        nodes_at_this_level,
                        changes.inserted_at_most,
                        leaf_additions_per_split_of_a_fresh_node,
                    )
                }
            }
            _ => {
                let nodes_below_at_most =
                    nodes_at_each_level[level - 1] + splits_at_the_level_below;
                if nodes_below_at_most <= internal_entries {
                    0
                } else {
                    splits_at_one_level_at_most(
                        nodes_at_this_level,
                        splits_at_the_level_below,
                        internal_additions_per_split_of_a_fresh_node,
                    )
                }
            }
        };
        nodes_rewritten = nodes_rewritten
            .checked_add(nodes_at_this_level.min(changed_paths) + splits_at_this_level)
            .expect("重写的节点数装得进 u64");
        splits_at_the_level_below = splits_at_this_level;
    }
    // 迭代上界：每一轮新层上切的次数严格少于下一层（新根起步两个孩子，⌊(下一层切的次数 − 1) ÷ 那个数⌋ < 下一层切的次数），到 0 停。
    while splits_at_the_level_below > 0 {
        let children_of_the_new_root = splits_at_the_level_below + 1;
        let splits_of_the_new_level = if children_of_the_new_root <= internal_entries {
            0
        } else {
            (splits_at_the_level_below - 1) / internal_additions_per_split_of_a_fresh_node
        };
        nodes_rewritten = nodes_rewritten
            .checked_add(1 + splits_of_the_new_level)
            .expect("重写的节点数装得进 u64");
        splits_at_the_level_below = splits_of_the_new_level;
    }
    nodes_rewritten
}

#[cfg(test)]
mod tests;
