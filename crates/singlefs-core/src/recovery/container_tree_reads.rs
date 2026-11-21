//! 读容器树：按 key 宽读进映射的树根、索引节点与叶，根对它的指针、key 范围是子树覆盖，
//! inode 叶记录跨叶升序，第一个文件与别的文件的 extent。

use super::*;

/// 重建一版时 logged_ops 根不是非空的层级 0 节点（`RecoveryFailure::UnitMalformed` 的 `what`，用例按它认）。
pub const LOGGED_OPERATIONS_ROOT_IS_NOT_A_NON_EMPTY_LEVEL_ZERO_NODE: &str =
    "logged_ops 根不是层级 0 的非空节点";
/// 重建一版时 logged_ops 根里有条目过不了字段表（宽不是 44、种类码保留、整理意图的 R 不对齐或槽数不是 64、补零非零、整理意图多于一条、
/// 不按 key 严格升序；`logged_operations::logged_operation_entries_of_the_root_entries`）：可写挂载拒（判决 G1「R 的解码拒不对齐的值」）。
pub const LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE: &str =
    "logged_ops 根里有条目过不了字段表";

/// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
pub(super) fn key_width_for_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(24),
        TREE_KIND_INODE => Some(INODE_KEY_WIDTH_IN_BYTES),
        TREE_KIND_ALLOCATION => Some(10),
        TREE_KIND_ACCOUNTING => Some(22),
        // `deleted_inodes` 的 key 是 inode 号 8 字节（D8（核心索引结构） 已定项 16 第 1 条）；墓碑树与谱系树的根条目照 inode 树内部节点的
        // 120 字节形态，key 是分隔 key = 容器号 8 字节（D18（块里携带什么信息） 已定项 24 第 1、5 条）。
        TREE_KIND_DELETED_INODES => Some(DELETED_INODES_KEY_WIDTH_IN_BYTES),
        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
            Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES)
        }
        // logged_ops 的 key = 种类 2 + 签发 txg 8（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 字段表）。
        TREE_KIND_LOGGED_OPERATIONS => Some(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES),
        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST => None,
        // 只读快照的两种条目指着的是建快照那一版活头 inode 树、extent 树的根：由快照的读路径按那两棵树解（这一批还没有），
        // 冷走读不按种类码读它们（`walk_to_file` 对它们 `continue`）。
        TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => None,
        _ => None,
    }
}

/// 读一棵进映射的树的根节点（extent、inode、分配记录；记账树多层，走 `code_two_tree::read_code_two_tree`）：位置提示读不出时经中央映射回退
/// （[`read_mapped_tree_node_via_hint_then_central_mapping`]），读到之后核它的自描述：树 ID、key 宽、出生身份、fsid，
/// 层级 0 的根再核 key 区间与条目相符（层级 ≥ 1 的根，区间是子树覆盖区间，由读孩子的一方判：inode 树根见
/// [`inode_tree_root_key_range_is_the_subtree_coverage`]）。挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
#[allow(
    clippy::too_many_arguments,
    reason = "核根要的六样加回退要的两样（查映射的口子、多跳计数），各自独立，收成结构体只会多一层没人验的名字"
)]
pub(crate) fn read_mapped_tree_root(
    reader: &dyn PoolReader,
    tree: TreeIdentifier,
    key_width: usize,
    pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
    stale_location_hint_hops: &mut usize,
) -> Result<IndexNodeHeader, RecoveryFailure> {
    let bytes = read_mapped_tree_node_via_hint_then_central_mapping(
        reader,
        pointer,
        MappedTreeNodeClass::IndexNode,
        central_mapping_locations_of_key,
        stale_location_hint_hops,
    )?;
    tree_root_checked_against_its_pointer(
        &bytes,
        tree,
        key_width,
        pointer,
        root,
        expected_filesystem_identifier,
    )
}

/// 一棵容器树（墓碑树、谱系树）里的一个单元：一片容器（叶序）、根之下的一个码 2 节点（按位置）、根。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnitOfAContainerTree {
    Container(usize),
    NodeBelowTheRoot(ContainerTreeNodePosition),
    Root,
}

impl UnitOfAContainerTree {
    /// 映射 key 里的单元类：容器是码 3 打包记录单元，节点是码 2。
    pub(super) fn unit_class(self) -> u8 {
        match self {
            UnitOfAContainerTree::Container(_) => UNIT_CLASS_PACKED,
            UnitOfAContainerTree::NodeBelowTheRoot(_) | UnitOfAContainerTree::Root => {
                UNIT_CLASS_INDEX_NODE
            }
        }
    }
}

/// 一棵容器树的三种角色（与发布路径同一组：`transaction::TransactionUnit` 里那棵树的叶容器、根、根之下的节点）。
pub(super) struct ContainerTreeRolesOfOneTree {
    container: fn(usize) -> TransactionUnit,
    root: TransactionUnit,
    node_below_the_root: fn(ContainerTreeNodePosition) -> TransactionUnit,
}

impl ContainerTreeRolesOfOneTree {
    pub(super) fn of(&self, unit: UnitOfAContainerTree) -> TransactionUnit {
        match unit {
            UnitOfAContainerTree::Container(position) => (self.container)(position),
            UnitOfAContainerTree::NodeBelowTheRoot(position) => {
                (self.node_below_the_root)(position)
            }
            UnitOfAContainerTree::Root => self.root,
        }
    }
}

fn tombstone_container_role_of_position(position: usize) -> TransactionUnit {
    TransactionUnit::TombstoneLeafContainer(TombstoneContainerIndexInTree::of_position(position))
}

fn lineage_container_role_of_position(position: usize) -> TransactionUnit {
    TransactionUnit::LineageLeafContainer(LineageContainerIndexInTree::of_position(position))
}

pub(super) const CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE: ContainerTreeRolesOfOneTree =
    ContainerTreeRolesOfOneTree {
        container: tombstone_container_role_of_position,
        root: TransactionUnit::TombstoneRoot,
        node_below_the_root: TransactionUnit::TombstoneNodeBelowTheRoot,
    };

pub(super) const CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE: ContainerTreeRolesOfOneTree =
    ContainerTreeRolesOfOneTree {
        container: lineage_container_role_of_position,
        root: TransactionUnit::LineageRoot,
        node_below_the_root: TransactionUnit::LineageNodeBelowTheRoot,
    };

fn retirement_container_role_of_position(position: usize) -> TransactionUnit {
    TransactionUnit::RetirementLeafContainer(RetirementContainerIndexInTree::of_position(position))
}

pub(super) const CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE: ContainerTreeRolesOfOneTree =
    ContainerTreeRolesOfOneTree {
        container: retirement_container_role_of_position,
        root: TransactionUnit::RetirementRoot,
        node_below_the_root: TransactionUnit::RetirementNodeBelowTheRoot,
    };

/// 墓碑树、谱系树码 2 节点条目里的子指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句
/// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏，与 inode 树内部条目同一条）。
pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
    "墓碑树或谱系树内部条目的子指针头部 extent 偏移不是 0";
/// 墓碑树、谱系树码 2 节点条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14，与 inode 树内部条目同一条）。
pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
    &str = "墓碑树或谱系树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";

/// 一条带指针的条目里那条指针头部第一版之外的取值报哪一句：按落在哪一组字段上，在读者给的两句里挑。
pub(super) fn what_of_a_pointer_head_field_outside_the_first_version(
    field: PointerHeadFieldOutsideTheFirstVersion,
    what_of_the_encryption_or_compression_fields: &'static str,
    what_of_the_extent_offset: &'static str,
) -> &'static str {
    match field.field_group() {
        PointerHeadFieldGroup::EncryptionOrCompression => {
            what_of_the_encryption_or_compression_fields
        }
        PointerHeadFieldGroup::ExtentOffset => what_of_the_extent_offset,
    }
}

/// 读一个进映射的树节点（码 2 或码 3，提示读不出经映射回退）：重建那一路收进来的读盘办法。
pub(super) type ReadOneMappedTreeNode<'reading> =
    dyn FnMut(&NodePointer, MappedTreeNodeClass) -> Result<Vec<u8>, RecoveryFailure> + 'reading;

impl<Record> ContainerTreeRead<Record> {
    /// 进映射的单元数：每片容器、每个码 2 节点（根在内）各一把 key。
    pub(super) fn mapped_units(&self) -> usize {
        self.containers.len() + self.index_nodes.nodes.len()
    }

    /// 全部单元按 bump 次序（容器按叶序，再是码 2 节点、根最末），与发布路径装 `units` 与 `mapped_units` 的次序相同。
    pub(super) fn units_in_bump_order(&self) -> Vec<(UnitOfAContainerTree, NodePointer, Vec<u8>)> {
        let containers =
            self.containers
                .iter()
                .enumerate()
                .map(|(position, (_, pointer, bytes))| {
                    (
                        UnitOfAContainerTree::Container(position),
                        *pointer,
                        bytes.clone(),
                    )
                });
        let nodes = self
            .index_nodes
            .nodes
            .iter()
            .map(|(position, pointer, bytes)| {
                let unit = if self.index_nodes.shape.is_the_root(*position) {
                    UnitOfAContainerTree::Root
                } else {
                    UnitOfAContainerTree::NodeBelowTheRoot(*position)
                };
                (unit, *pointer, bytes.clone())
            });
        containers.chain(nodes).collect()
    }

    /// 这一版的形状与根之下每个节点的指针（根的指针住树表条目）。
    pub(super) fn index_nodes_version(&self) -> ContainerTreeIndexNodesVersion {
        ContainerTreeIndexNodesVersion {
            shape: self.index_nodes.shape.clone(),
            pointers_below_the_root: self
                .index_nodes
                .nodes
                .iter()
                .filter(|(position, _, _)| !self.index_nodes.shape.is_the_root(*position))
                .map(|(position, pointer, _)| (*position, *pointer))
                .collect(),
        }
    }
}

/// 从根往下读一棵容器树的码 2 节点（不读容器）：每个节点照树根同一组核它的自描述（`tree_root_checked_against_its_pointer`：树 ID、
/// key 宽、出生身份、fsid）；key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2）：层级 1 的节点贴紧首末两条条目的分隔 key，
/// 层级 ℓ > 1 的节点头里的最小 key 等于第一个孩子头里的最小 key、最大 key 等于最后一个孩子头里的最大 key；根的层级至少 1、每个节点非空；
/// 层级 ℓ > 1 的节点的条目指码 2 孩子——身份引用全 0（类型段 0，D8（核心索引结构） 已定项 6）、孩子的层级 = ℓ − 1、孩子的首 key = 条目的分隔 key；
/// 层级 1 的条目指容器，原样交回由读容器那一步判。
/// 递归的上界：根的层级（一字节），每往下一层层级减一。`read_index_node` 读一个码 2 节点（提示读不出经映射回退）。
///
/// # Errors
/// 读不出（经映射回退之后仍读不出）、自描述不符、条目窄于字段表、形状不合 ⇒ 对应的 `RecoveryFailure`。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：树号、种类码、两种报错名、根指针、根记录、fsid、读节点的办法"
)]
pub(super) fn read_container_tree_index_nodes(
    tree: TreeIdentifier,
    kind: u16,
    what_index_node: &'static str,
    what_entry: &'static str,
    root_pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    read_index_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
) -> Result<ContainerTreeIndexNodesRead, RecoveryFailure> {
    let key_width = key_width_for_kind(kind).expect("墓碑树与谱系树的 key 宽登记过");
    let malformed = || RecoveryFailure::UnitMalformed {
        what: what_index_node,
    };
    let root_bytes = read_index_node(root_pointer)?;
    let root_header = tree_root_checked_against_its_pointer(
        &root_bytes,
        tree,
        key_width,
        root_pointer,
        root,
        expected_filesystem_identifier,
    )?;
    container_tree_node_of_level_one_is_tight_over_its_entries(&root_header)?;
    if root_header.level == 0 || root_header.entries.is_empty() {
        return Err(malformed());
    }
    let mut level = root_header.level;
    let mut nodes_of_this_level: Vec<(NodePointer, Vec<u8>, IndexNodeHeader)> =
        vec![(*root_pointer, root_bytes, root_header)];
    // 自根往下每一层：(指针, 字节, 孩子数)。
    let mut levels_from_the_root_down: Vec<Vec<(NodePointer, Vec<u8>, usize)>> = Vec::new();
    let mut container_entries = Vec::new();
    loop {
        let mut nodes_of_the_level_below = Vec::new();
        let mut this_level = Vec::new();
        for (pointer, bytes, header) in nodes_of_this_level {
            for (entry_index, entry) in header.entries.iter().enumerate() {
                let (separator_key, identity_in_the_entry, child_pointer) =
                    parse_inode_internal_entry_or_refusal(entry).map_err(
                        |refusal| match refusal {
                            PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable => {
                                RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                                    what: what_entry,
                                    entry_bytes: entry.len(),
                                    field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY)
                                        .expect("120"),
                                }
                            }
                            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(
                                field,
                            ) => RecoveryFailure::UnitMalformed {
                                what: what_of_a_pointer_head_field_outside_the_first_version(
                                    field,
                                    CONTAINER_TREE_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
                                    CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
                                ),
                            },
                        },
                    )?;
                if level == 1 {
                    container_entries.push((separator_key, identity_in_the_entry, child_pointer));
                    continue;
                }
                if identity_in_the_entry != IDENTITY_OF_A_CODE_TWO_CHILD {
                    return Err(malformed());
                }
                let child_bytes = read_index_node(&child_pointer)?;
                let child_header = tree_root_checked_against_its_pointer(
                    &child_bytes,
                    tree,
                    key_width,
                    &child_pointer,
                    root,
                    expected_filesystem_identifier,
                )?;
                container_tree_node_of_level_one_is_tight_over_its_entries(&child_header)?;
                if child_header.level.checked_add(1) != Some(level)
                    || child_header.entries.is_empty()
                    || child_header.smallest_key[..] != separator_key.to_le_bytes()[..]
                {
                    return Err(malformed());
                }
                // 层级 ≥ 2 的节点（这一个 header）头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2）：最小的是第一个孩子头里的最小 key，
                // 最大的是最后一个孩子头里的最大 key（孩子的区间已经按同一条判过，层级 1 的贴紧首末两条容器号）。
                let is_the_first_child = entry_index == 0;
                let is_the_last_child = entry_index + 1 == header.entries.len();
                if (is_the_first_child && header.smallest_key != child_header.smallest_key)
                    || (is_the_last_child && header.largest_key != child_header.largest_key)
                {
                    return Err(RecoveryFailure::InvariantViolated {
                        invariant: "I-1.1",
                        detail: CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
                    });
                }
                nodes_of_the_level_below.push((child_pointer, child_bytes, child_header));
            }
            this_level.push((pointer, bytes, header.entries.len()));
        }
        levels_from_the_root_down.push(this_level);
        if level == 1 {
            break;
        }
        level -= 1;
        nodes_of_this_level = nodes_of_the_level_below;
    }
    levels_from_the_root_down.reverse();
    let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(
        levels_from_the_root_down
            .iter()
            .map(|nodes| nodes.iter().map(|(_, _, children)| *children).collect())
            .collect(),
    )
    .ok_or_else(malformed)?;
    let nodes = levels_from_the_root_down
        .into_iter()
        .enumerate()
        .flat_map(|(level_index, nodes)| {
            nodes
                .into_iter()
                .enumerate()
                .map(move |(index_in_level, (pointer, bytes, _))| {
                    (
                        ContainerTreeNodePosition {
                            level: u8::try_from(level_index + 1)
                                .expect("层数不超过根头里的一字节层级"),
                            index_in_level: u32::try_from(index_in_level)
                                .expect("同层节点数装得进 u32"),
                        },
                        pointer,
                        bytes,
                    )
                })
        })
        .collect();
    Ok(ContainerTreeIndexNodesRead {
        shape,
        nodes,
        container_entries,
    })
}

/// 读一棵容器树：零根（树表条目的根指针置零）交回 `None`；否则读码 2 节点（[`read_container_tree_index_nodes`]），再按叶序读每片容器，
/// 核容器头的身份与父条目里的身份引用逐字相等、类型 = 这棵树的打包记录类型、出生树 = 这棵树、分隔 key = 容器号、记录宽、非空，逐条解记录。
/// `read_node` 读一个单元（码 2 或码 3，提示读不出经映射回退）。
///
/// # Errors
/// 同 [`read_container_tree_index_nodes`]；容器读不出、解不开、与条目对不上、记录解不开 ⇒ 对应的 `RecoveryFailure`。
pub(super) fn read_container_tree<Record>(
    to_read: &ContainerTreeToRead<Record>,
    root_pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    read_node: &mut ReadOneMappedTreeNode<'_>,
) -> Result<Option<ContainerTreeRead<Record>>, RecoveryFailure> {
    if *root_pointer == NodePointer::empty_root() {
        return Ok(None);
    }
    let index_nodes = read_container_tree_index_nodes(
        to_read.tree,
        to_read.kind,
        to_read.what_index_node,
        to_read.what_entry,
        root_pointer,
        root,
        expected_filesystem_identifier,
        &mut |pointer: &NodePointer| read_node(pointer, MappedTreeNodeClass::IndexNode),
    )?;
    let mut containers = Vec::with_capacity(index_nodes.container_entries.len());
    for (separator_key, identity_in_the_entry, container_pointer) in &index_nodes.container_entries
    {
        let container_bytes = read_node(container_pointer, MappedTreeNodeClass::PackedRecordUnit)?;
        let container = parse_packed_unit(&container_bytes).map_err(|_error| {
            RecoveryFailure::UnitMalformed {
                what: to_read.what_container,
            }
        })?;
        if container.identity != *identity_in_the_entry
            || container.identity.record_type != to_read.record_type
            || container.identity.birth_tree != to_read.tree
            || container.identity.container != *separator_key
            || container.record_width
                != usize::try_from(to_read.record_width_in_bytes).expect("记录宽是 40 或 16")
            || container.records.is_empty()
        {
            return Err(RecoveryFailure::UnitMalformed {
                what: to_read.what_container_against_its_entry,
            });
        }
        let records = container
            .records
            .iter()
            .map(|record_bytes| {
                (to_read.parse_record)(record_bytes).ok_or(RecoveryFailure::UnitMalformed {
                    what: to_read.what_record,
                })
            })
            .collect::<Result<Vec<Record>, RecoveryFailure>>()?;
        containers.push((
            RecordContainer {
                identity: container.identity,
                records,
            },
            *container_pointer,
            container_bytes,
        ));
    }
    Ok(Some(ContainerTreeRead {
        index_nodes,
        containers,
    }))
}

/// 一个树根节点读回来之后核的那几样：树 ID、key 宽、出生身份、fsid，层级 0 的再核 key 区间与首末条目相符（层级 ≥ 1 的区间是子树覆盖区间，
/// 由读孩子的一方判）。经映射回退读到的节点照样核这几样——搬迁只改映射一条条目，被搬的单元逐字节不变，指针里的出生身份照样对得上。
pub(super) fn tree_root_checked_against_its_pointer(
    bytes: &[u8],
    tree: TreeIdentifier,
    key_width: usize,
    pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
) -> Result<IndexNodeHeader, RecoveryFailure> {
    let node = parse_index_node(bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
        what: "树根节点",
    })?;
    if node.tree != tree {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.3",
            detail: "根头里的树 ID 与树表不符",
        });
    }
    if node.key_width != key_width {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "根自述 key 宽与树的种类不符",
        });
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "树根诞生于根之后",
        });
    }
    if node.filesystem_identifier != expected_filesystem_identifier {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.4",
            detail: "树根 fsid 不符",
        });
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "树根出生序号与指针不符",
        });
    }
    // 叶（层级 0）头里的 key 区间贴紧首末两条条目；层级 ≥ 1 的是子树覆盖区间（D18（块里携带什么信息） 已定项 2），孩子读回来才判得了，
    // 由读孩子的一方判：inode 树根读完叶容器之后判（[`inode_tree_root_key_range_is_the_subtree_coverage`]），容器树的节点在
    // [`read_container_tree_index_nodes`] 里判（层级 1 的孩子是容器、key 是容器号，区间就是首末两条条目的分隔 key）。
    if node.level == 0 && !key_range_is_tight_over_the_entries(&node) {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: "根 key 区间与条目不符",
        });
    }
    Ok(node)
}

/// 一个码 2 节点头里的 key 区间是不是首末两条条目的 key（零条目的节点不判，区间任意）。条目比 key 窄的由解条目那一步拒，这里当不贴紧。
fn key_range_is_tight_over_the_entries(node: &IndexNodeHeader) -> bool {
    let key_width = node.key_width;
    match (node.entries.first(), node.entries.last()) {
        (Some(first), Some(last)) => {
            first.get(..key_width) == Some(&node.smallest_key[..])
                && last.get(..key_width) == Some(&node.largest_key[..])
        }
        (None, _) | (_, None) => true,
    }
}

/// inode 树根头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2「孩子是码 3 容器的内部节点上，max_key 是最后那片容器里
/// 最后一条记录的 key，不是最后一条条目的分隔 key」）：[叶容器里读回来的最小记录号, 最大记录号]。读完全部叶容器之后判
/// （[`tree_root_checked_against_its_pointer`] 对层级 ≥ 1 的节点不判区间）。零条目的根（unlink 删掉树里最后一条记录之后，写者写 [0, 0]）
/// 与一条记录都没读回来的不判，与原来贴紧首末条目那一判对空节点同一读法。
///
/// # Errors
/// 头里的最小或最大 key 不是读回来的最小或最大记录号 ⇒ [`RecoveryFailure::InvariantViolated`]（I-1.1）。
pub(crate) fn inode_tree_root_key_range_is_the_subtree_coverage(
    inode_root: &IndexNodeHeader,
    inode_numbers_read_back: impl IntoIterator<Item = u64>,
) -> Result<(), RecoveryFailure> {
    if inode_root.entries.is_empty() {
        return Ok(());
    }
    let smallest_and_largest =
        inode_numbers_read_back
            .into_iter()
            .fold(None, |range: Option<(u64, u64)>, inode| {
                Some(range.map_or((inode, inode), |(smallest, largest)| {
                    (smallest.min(inode), largest.max(inode))
                }))
            });
    let Some((smallest, largest)) = smallest_and_largest else {
        return Ok(());
    };
    if inode_root.smallest_key[..] != smallest.to_le_bytes()[..]
        || inode_root.largest_key[..] != largest.to_le_bytes()[..]
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
        });
    }
    Ok(())
}

/// 容器树（墓碑树、谱系树、退役树）层级 1 的码 2 节点：孩子是码 3 容器、树的 key 是容器号，子树覆盖区间就是首末两条条目的分隔 key
/// （读容器那一步另核分隔 key = 容器号），读回来马上判，与 [`tree_root_checked_against_its_pointer`] 对叶的那一判同一处、同一个报错。
/// 层级 ≥ 2 的节点由 [`read_container_tree_index_nodes`] 读完孩子判。
fn container_tree_node_of_level_one_is_tight_over_its_entries(
    node: &IndexNodeHeader,
) -> Result<(), RecoveryFailure> {
    if node.level == 1 && !key_range_is_tight_over_the_entries(node) {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: "根 key 区间与条目不符",
        });
    }
    Ok(())
}

/// inode 树根头里的 key 区间不是子树覆盖区间时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
/// （[`inode_tree_root_key_range_is_the_subtree_coverage`]）。
pub const INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE: &str =
    "inode 树根 key 区间不是子树覆盖区间（最小、最大记录号）";

/// 一片 inode 叶容器里相邻两条记录的号不严格升序时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
/// （[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]，I-9.4 第四句）。
pub const INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND: &str =
    "一片 inode 叶容器里相邻两条记录的号不严格升序";

/// 一片 inode 叶容器的第一条记录号不大于叶序在它前面那几片的最大记录号（两片叶交叠）时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
/// （[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]，I-9.12）。
pub const INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM: &str =
    "inode 叶容器的第一条记录号不大于叶序在它前面那几片的最大记录号（两片叶交叠）";

/// 指着一片 inode 叶容器的那条内部条目的分隔 key 不大于叶序在它前面那几片的最大记录号（上一个孩子的最大 key）时
/// [`RecoveryFailure::InvariantViolated`] 的 `detail`（[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]，I-9.12 第三句）。
pub const INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT: &str =
    "inode 树内部条目的分隔 key 不大于叶序在它前面那几片的最大记录号（上一个孩子的最大 key）";

/// 一片 inode 叶容器里有一条记录号小于指着它的那条内部条目的分隔 key、或小于这片叶的容器号时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
/// （[`inode_record_is_not_below_the_separator_key_and_the_container`]，I-9.4 第一句与 I-9.12 第二句「分隔 key ≤ 孩子的 key 区间下界」）。
pub const INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER: &str =
    "inode 记录号小于分隔 key 或容器号";

/// 指着一片 inode 叶的那条内部条目的分隔 key（I-9.12）：与记录号、容器号底下都是 u64，各是一个量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SeparatorKeyOfAnInodeLeaf(pub(crate) u64);

/// 一片 inode 叶容器头里的容器号（I-9.4）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContainerNumberOfAnInodeLeaf(pub(crate) u64);

/// 一片 inode 叶容器里的一条记录号不小于指着这片叶的那条内部条目的分隔 key、也不小于这片叶的容器号（I-9.4 第一句；分隔 key 那一半是
/// I-9.12 第二句「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界」，逐条记录判就判到了下界）。读 inode 树的四个读者——可写挂载重建、冷走读、
/// 快照第一个文件的冷读与只读挂载打开池——逐条记录调这一个，同一张镜像一个结局（判决 `research/prompts/m4-close-code-r3-main-verification.md`
/// 第三节第 2 条：只读挂载此前丢了分隔 key）。
///
/// # Errors
/// 记录号小于分隔 key 或容器号 ⇒ [`RecoveryFailure::InvariantViolated`]（I-9.4，[`INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER`]）。
pub(crate) fn inode_record_is_not_below_the_separator_key_and_the_container(
    record_inode: InodeNumber,
    separator_key: SeparatorKeyOfAnInodeLeaf,
    container: ContainerNumberOfAnInodeLeaf,
) -> Result<(), RecoveryFailure> {
    if record_inode.0 < separator_key.0 || record_inode.0 < container.0 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.4",
            detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
        });
    }
    Ok(())
}

/// 读回来的一片 inode 叶容器（按根的条目次序逐片喂）照写者的前提判：全部叶的记录按叶序连起来按 inode 号严格升序
/// （`inode_tree::leaf_containers_are_well_formed` 的前一半，[`crate::inode_tree::write_records_into_leaf_containers`] 的 `requires`；
/// 按叶序折半找 `TransactionOutput::inode_record_found_by_its_leaf` 也靠它）。盘上不满足这一条的一版（两片叶交叠违反 I-9.12，
/// 一片里不升序违反 I-9.4 第四句）读者不收：收下了写者的前提就不成立，按叶序折半找漏找、维护的 n_live 与 n_记录漂走
/// （判决 `research/prompts/m4-close-code-r2-main-verification.md` 第二节 T4，第三节取改法甲）。`largest_of_the_leaves_before` 是叶序在它前面那几片的
/// 最大记录号（头一片交 `None`），交回读完这一片之后的最大记录号；没有记录的叶由调用方另拒，这里交回原来那个。
/// 再判 I-9.12 第三句「第 i 条的分隔 key > 第 i−1 个孩子的 key 区间上界」：`separator_key_of_the_leaf` 是指着这片叶的那条内部条目的分隔 key，
/// 要大于前面那几片的最大记录号（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 1 条，与池级 checker 同一结局）。
/// 第二句「分隔 key ≤ 这个孩子的最小 key」由调用方逐条记录先判（[`inode_record_is_not_below_the_separator_key_and_the_container`]）；
/// 「条目按分隔 key 严格递增」由这两句合出来（叶都不空：前一条的分隔 key ≤ 前一片的最小号 ≤ 前一片的最大号 < 这一条的分隔 key），不另判。
/// 两片交叠的一形（第一条不大于前面的最大号）先报交叠：分隔 key 不大于第一条，交叠时它也不大于前面的最大号，两句都红、报先判的那一句。
///
/// # Errors
/// 片里相邻两条不严格升序 ⇒ [`RecoveryFailure::InvariantViolated`]（I-9.4，[`INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND`]）；
/// 第一条不大于前面那几片的最大记录号 ⇒ 同一成员（I-9.12，[`INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM`]）；
/// 分隔 key 不大于前面那几片的最大记录号 ⇒ 同一成员（I-9.12，[`INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT`]）。
pub(crate) fn inode_records_of_the_leaf_ascend_after_the_leaves_before(
    separator_key_of_the_leaf: u64,
    largest_of_the_leaves_before: Option<u64>,
    records_of_the_leaf: &[InodeRecord],
) -> Result<Option<u64>, RecoveryFailure> {
    let Some(first) = records_of_the_leaf.first() else {
        return Ok(largest_of_the_leaves_before);
    };
    if largest_of_the_leaves_before.is_some_and(|largest_before| largest_before >= first.inode) {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.12",
            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
        });
    }
    if largest_of_the_leaves_before
        .is_some_and(|largest_before| largest_before >= separator_key_of_the_leaf)
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.12",
            detail: INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
        });
    }
    if records_of_the_leaf
        .windows(2)
        .any(|adjacent| adjacent[0].inode >= adjacent[1].inode)
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.4",
            detail: INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
        });
    }
    Ok(records_of_the_leaf.last().map(|last| last.inode))
}

/// 容器树（墓碑树、谱系树、退役树）一个码 2 节点头里的 key 区间不是子树覆盖区间时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
/// （[`read_container_tree_index_nodes`]）。
pub const CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE: &str =
    "容器树码 2 节点 key 区间不是子树覆盖区间（最左那片到最右那片的容器号）";

/// 从盘上读回来的 extent 树里第一个文件之外一个在上段里有条目的文件：它的数据指针（第 i 个是单元 i；标签 0 时为空）、下段的每个节点
/// （位置、指针、字节，bump 次序；内联或标签 0 时为空）与标签声明的尾部（标签 3、4 最后一个单元之后到 size 是洞）。
pub(crate) struct ExtentsOfAFileOtherThanTheFirst {
    pub inode: InodeNumber,
    pub data_pointers: Vec<DataPointer>,
    pub lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer, Vec<u8>)>,
    pub tail: TailOfAFile,
}

/// 从盘上读回来的 extent 树里第一个文件的样子：它的数据指针（第 i 个是单元 i）、这一版 extent 树的节点与指针、
/// 第一个文件之外各有数据单元的文件，与每个节点的角色、指针与字节（bump 次序：第一个文件的下段先叶后根、第一个文件之外各文件的下段
/// （按 inode 号）、上段先叶后根）。
pub(crate) struct ExtentsOfTheFirstFile {
    pub data_pointers: Vec<DataPointer>,
    /// 上段叶条目标签声明的尾部（判决 m4-trunc-r1 K5）：标签 3、4 最后一个单元之后到 size 是洞。
    pub tail: TailOfAFile,
    pub version: ExtentTreeVersion,
    /// 第一个文件之外各有数据单元的文件，按 inode 号升序。
    pub files_other_than_the_first: Vec<ExtentsOfAFileOtherThanTheFirst>,
    pub nodes_in_bump_order: Vec<(TransactionUnit, NodePointer, Vec<u8>)>,
}

impl ExtentsOfTheFirstFile {
    /// 第一个文件与第一个文件之外各文件的数据单元一共几个（数映射条目要它）。
    pub(super) fn data_units_of_every_file(&self) -> usize {
        self.data_pointers.len()
            + self
                .files_other_than_the_first
                .iter()
                .map(|file| file.data_pointers.len())
                .sum::<usize>()
    }
}

/// 从整棵读回来的 extent 树里取第一个文件与第一个文件之外各在上段里有条目的文件（`transaction` 按它们建这一版在内存里的样子）：
/// 每个文件的单元从 0 起连号、中间没有洞，尾部的洞由标签声明（标签 3、4），标签 0（一个单元都没有）的收下、交回空的数据指针与
/// 「之后到 size 是洞」的尾部：整个文件是洞（判决 m4-trunc-r1 K5；任意活 inode，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
/// 第二节 S2）。上段里可以没有第一个文件那一条（它删完了、或还没有单元）。单元数对不对 size 由调用方按标签声明的尾部判
/// （`data_unit_count_matches_the_inode_size`）。
///
/// # Errors
/// 哪个文件的单元中间有洞 ⇒ `UnitMalformed`：今天的写路径写不出来（每个文件版本从偏移 0 顺序写，截断只从尾部放），
/// 盘上读来的却可以是任何样子——第一版不支持，在任何落盘动作之前交回。
pub(crate) fn extents_of_the_first_file(
    tree: ExtentTreeReadFromDisk,
) -> Result<ExtentsOfTheFirstFile, RecoveryFailure> {
    let ExtentTreeReadFromDisk { upper, files } = tree;
    let mut extents_of_the_first_file_on_disk: Option<ExtentsOfAFileReadFromDisk> = None;
    let mut files_other_than_the_first: Vec<ExtentsOfAFileOtherThanTheFirst> = Vec::new();
    // 迭代上界是上段叶条目数；按 inode 号升序（上段按位置读回来就是这个次序）。
    for (inode, extents) in files {
        if inode == FIRST_INODE_NUMBER {
            extents_of_the_first_file_on_disk = Some(extents);
            continue;
        }
        let data_pointers_of_the_file = extents.data_pointers();
        let units_are_numbered_from_zero_without_a_hole = data_pointers_of_the_file
            .iter()
            .enumerate()
            .all(|(position, (unit, _))| u64::try_from(position).expect("单元序号") == *unit);
        if !units_are_numbered_from_zero_without_a_hole {
            return Err(RecoveryFailure::UnitMalformed {
                what: "extent 树里第一个文件之外的文件单元中间有洞（第一版不支持）",
            });
        }
        let tail = extents.tail();
        let lower_nodes = match extents {
            ExtentsOfAFileReadFromDisk::LowerSegment(segment, _) => segment.nodes,
            ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(..) => {
                Vec::new()
            }
        };
        files_other_than_the_first.push(ExtentsOfAFileOtherThanTheFirst {
            inode: InodeNumber(inode),
            data_pointers: data_pointers_of_the_file
                .into_iter()
                .map(|(_, pointer)| pointer)
                .collect(),
            lower_nodes,
            tail,
        });
    }
    // 上段里没有第一个文件那一条：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），或删完之后只有别的文件有条目，
    // 第一个文件没有数据单元、没有下段。
    let Some(extents) = extents_of_the_first_file_on_disk else {
        return Ok(extents_assembled_with_the_upper_segment(
            upper,
            Vec::new(),
            TailOfAFile::NoHoleAfterTheLastDataUnit,
            Vec::new(),
            files_other_than_the_first,
        ));
    };
    // 标签 0（一个数据单元都没有）收下：整个文件是洞（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5「标签 0 配 size > 0
    // 是整个文件都是洞」），读出 0、不读盘；单元数对不对 size 由调用方按标签声明的尾部判（`data_unit_count_matches_the_inode_size`）。
    // 与挂载态读者同一读法（`mounted_read` 把标签 0 当整个洞）。今天的写路径写不出这一形。
    let data_pointers_with_units = extents.data_pointers();
    if data_pointers_with_units
        .iter()
        .enumerate()
        .any(|(position, (unit, _))| u64::try_from(position).expect("单元序号") != *unit)
    {
        return Err(RecoveryFailure::UnitMalformed {
            what: "extent 树里第一个文件的单元有洞（第一版不支持）",
        });
    }
    let tail = extents.tail();
    let lower_nodes = match extents {
        ExtentsOfAFileReadFromDisk::LowerSegment(segment, _) => segment.nodes,
        ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(..) => {
            Vec::new()
        }
    };
    Ok(extents_assembled_with_the_upper_segment(
        upper,
        data_pointers_with_units
            .into_iter()
            .map(|(_, pointer)| pointer)
            .collect(),
        tail,
        lower_nodes,
        files_other_than_the_first,
    ))
}

/// 第一个文件的数据指针、尾部与下段，第一个文件之外各文件，连同上段，拼成 [`ExtentsOfTheFirstFile`]（节点按 bump 次序：第一个文件的下段、
/// 第一个文件之外各文件的下段、上段）。
fn extents_assembled_with_the_upper_segment(
    upper: ExtentUpperSegmentReadFromDisk,
    data_pointers: Vec<DataPointer>,
    tail: TailOfAFile,
    lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer, Vec<u8>)>,
    files_other_than_the_first: Vec<ExtentsOfAFileOtherThanTheFirst>,
) -> ExtentsOfTheFirstFile {
    let upper_root_level = upper
        .nodes
        .last()
        .map(|(position, _, _)| position.level)
        .expect("读回来的上段至少有根");
    let version = ExtentTreeVersion {
        upper_nodes: upper
            .nodes
            .iter()
            .map(|(position, pointer, _)| (*position, *pointer))
            .collect(),
        lower_nodes: lower_nodes
            .iter()
            .map(|(position, pointer, _)| (*position, *pointer))
            .collect(),
    };
    let lower_nodes_of_the_files_other_than_the_first: Vec<(
        TransactionUnit,
        NodePointer,
        Vec<u8>,
    )> = files_other_than_the_first
        .iter()
        .flat_map(|file| {
            file.lower_nodes.iter().map(|(position, pointer, bytes)| {
                (
                    TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
                        inode: file.inode,
                        position: *position,
                    },
                    *pointer,
                    bytes.clone(),
                )
            })
        })
        .collect();
    let nodes_in_bump_order = lower_nodes
        .into_iter()
        .map(|(position, pointer, bytes)| {
            (TransactionUnit::ExtentLowerNode(position), pointer, bytes)
        })
        .chain(lower_nodes_of_the_files_other_than_the_first)
        .chain(upper.nodes.into_iter().map(|(position, pointer, bytes)| {
            (
                role_of_extent_upper_node(position, upper_root_level),
                pointer,
                bytes,
            )
        }))
        .collect();
    ExtentsOfTheFirstFile {
        data_pointers,
        tail,
        version,
        files_other_than_the_first,
        nodes_in_bump_order,
    }
}

/// 读侧认得删除新写出的三棵树的 key 宽（里程碑三·2 第 7 条「读侧补新种类的 key 宽」）：根指针不全零时冷走读按它读根、核根的自述 key 宽；
/// 不认得的种类在那一步拒成「树的种类没登记」。logged_ops 的 key 是种类 2 + 签发 txg 8（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）。
#[cfg(test)]
mod tree_kind_key_width_tests {
    use super::key_width_for_kind;
    use crate::inode_tree::INODE_KEY_WIDTH_IN_BYTES;
    use crate::records::{
        TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS,
        TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
    };

    #[test]
    fn deleted_inodes_tombstone_lineage_and_retirement_trees_have_eight_byte_keys_and_logged_operations_ten(
    ) {
        assert_eq!(
            key_width_for_kind(TREE_KIND_DELETED_INODES),
            Some(8),
            "`deleted_inodes` 的 key = inode 号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_TOMBSTONE),
            Some(8),
            "墓碑树根条目的分隔 key = 容器号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_LINEAGE),
            Some(8),
            "谱系树根条目的分隔 key = 容器号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_RETIREMENT),
            Some(8),
            "退役树根条目的分隔 key = 容器号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_LOGGED_OPERATIONS),
            Some(10),
            "logged_ops 的 key = 种类 2 + 签发 txg 8"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_INODE),
            Some(INODE_KEY_WIDTH_IN_BYTES)
        );
        assert_eq!(
            INODE_KEY_WIDTH_IN_BYTES, 8,
            "inode 树的 key = inode 号 8 字节（D8 已定项 6）"
        );
    }
}

#[cfg(test)]
mod inode_leaf_records_ascending_across_the_leaves_tests {
    use super::{
        inode_record_is_not_below_the_separator_key_and_the_container,
        inode_records_of_the_leaf_ascend_after_the_leaves_before, ContainerNumberOfAnInodeLeaf,
        RecoveryFailure, SeparatorKeyOfAnInodeLeaf, INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
        INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
        INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
        INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
    };
    use crate::address::{CheckpointTxg, InodeNumber};
    use crate::records::{
        InodeRecord, InodeRecordTruncationField, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
    };

    fn records_numbered(inode_numbers: &[u64]) -> Vec<InodeRecord> {
        inode_numbers
            .iter()
            .map(|&inode| InodeRecord {
                inode,
                object_birth: CheckpointTxg(3),
                nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
                size: 0,
                change_count: 3,
                write_time_seconds: 0,
                truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
            })
            .collect()
    }

    /// 逐片喂，每片的分隔 key 取它的第一条记录号（诞生时分隔 key 是容器号、也是最小号），交回最后的最大记录号或头一处拒。
    fn judged_leaf_by_leaf(leaves: &[&[u64]]) -> Result<Option<u64>, RecoveryFailure> {
        let leaves_with_their_separators: Vec<(u64, &[u64])> = leaves
            .iter()
            .map(|leaf| (*leaf.first().expect("每片叶至少一条记录"), *leaf))
            .collect();
        judged_leaf_by_leaf_with_their_separators(&leaves_with_their_separators)
    }

    /// 逐片喂（分隔 key、这片的记录号），交回最后的最大记录号或头一处拒。
    fn judged_leaf_by_leaf_with_their_separators(
        leaves: &[(u64, &[u64])],
    ) -> Result<Option<u64>, RecoveryFailure> {
        leaves
            .iter()
            .try_fold(None, |largest_before, (separator_key, leaf)| {
                inode_records_of_the_leaf_ascend_after_the_leaves_before(
                    *separator_key,
                    largest_before,
                    &records_numbered(leaf),
                )
            })
    }

    /// 攻方 `separator_200` 那一形缩小：两片叶 [1, 2, 3] 与 [5, 7]，右片的分隔 key 改成 3（不大于左片最大号 3，仍不大于右片最小号 5）：
    /// 记录照叶序严格升序，拒在 I-9.12 第三句。分隔 key 4 落在 (3, 5] 里照收，交回最大号 7。
    #[test]
    fn separator_key_not_above_the_largest_record_of_the_leaves_before_is_refused() {
        assert_eq!(
            judged_leaf_by_leaf_with_their_separators(&[(1, &[1, 2, 3]), (3, &[5, 7])]),
            Err(RecoveryFailure::InvariantViolated {
                invariant: "I-9.12",
                detail: INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
            })
        );
        assert_eq!(
            judged_leaf_by_leaf_with_their_separators(&[(1, &[1, 2, 3]), (4, &[5, 7])]),
            Ok(Some(7))
        );
    }

    /// 记录号小于分隔 key（攻方 `separator_235` 那一形：分隔 key 235、头一条 234）或小于容器号：拒在 I-9.4；等于两者照收。
    #[test]
    fn record_below_the_separator_key_or_the_container_is_refused_on_the_container_number_invariant(
    ) {
        let below = Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.4",
            detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
        });
        assert_eq!(
            inode_record_is_not_below_the_separator_key_and_the_container(
                InodeNumber(234),
                SeparatorKeyOfAnInodeLeaf(235),
                ContainerNumberOfAnInodeLeaf(234)
            ),
            below
        );
        assert_eq!(
            inode_record_is_not_below_the_separator_key_and_the_container(
                InodeNumber(234),
                SeparatorKeyOfAnInodeLeaf(234),
                ContainerNumberOfAnInodeLeaf(235)
            ),
            below
        );
        assert_eq!(
            inode_record_is_not_below_the_separator_key_and_the_container(
                InodeNumber(234),
                SeparatorKeyOfAnInodeLeaf(234),
                ContainerNumberOfAnInodeLeaf(234)
            ),
            Ok(())
        );
    }

    /// 两片叶 [1, 2, 3] 与 [4, 7]：连起来严格升序，交回最大号 7。
    #[test]
    fn two_leaves_ascending_across_the_boundary_are_accepted_with_the_largest_record_number() {
        assert_eq!(judged_leaf_by_leaf(&[&[1, 2, 3], &[4, 7]]), Ok(Some(7)));
    }

    /// 攻方第二次抽样那一形缩小：左片末条改成比右片头一条大（[1, 2, 9] 与 [4, 7]），每片自己升序，两片交叠：拒在 I-9.12。
    /// 右片头一条恰等于左片末条（[1, 2, 4] 与 [4, 7]）同样拒：严格升序，等号不放过。
    #[test]
    fn left_leaf_reaching_past_the_first_record_of_the_right_leaf_is_refused_as_overlapping() {
        let overlapping = Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.12",
            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
        });
        assert_eq!(judged_leaf_by_leaf(&[&[1, 2, 9], &[4, 7]]), overlapping);
        assert_eq!(judged_leaf_by_leaf(&[&[1, 2, 4], &[4, 7]]), overlapping);
    }

    /// 一片里两条倒序（[1, 3, 2]）或同号两条（[1, 1]）：拒在 I-9.4 第四句。
    #[test]
    fn records_out_of_order_within_one_leaf_are_refused_on_the_container_number_invariant() {
        let out_of_order = Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.4",
            detail: INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
        });
        assert_eq!(judged_leaf_by_leaf(&[&[1, 3, 2]]), out_of_order);
        assert_eq!(judged_leaf_by_leaf(&[&[1, 1]]), out_of_order);
    }
}
