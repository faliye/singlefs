//! 装单元：出生序号的发号器，文件版本、extent 树、分配记录树、inode 树、容器树与删除树的单元，多层树的装法。

use super::*;

/// 出生序号：同一 (树, txg, 实例) 里每写出一个码 2 / 码 3 单元加 1，从 0 起（D19（块指针的结构与宽度预算） 已定项 9）。
/// 作用域是一次 checkpoint：`publish_admitted` 每次发布建一个，这次装的每个对象（`build_file_version_units`）与固定点结构都从它取号。
#[derive(Default)]
pub struct BirthSequenceAllocator {
    counters: BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>,
}

impl BirthSequenceAllocator {
    pub fn next(
        &mut self,
        tree: TreeIdentifier,
        txg: CheckpointTxg,
        instance: InstanceGeneration,
    ) -> BirthSequence {
        let counter = self.counters.entry((tree, txg, instance)).or_insert(0);
        let sequence = BirthSequence(*counter);
        *counter += 1;
        sequence
    }
}

/// 同一请求里的数据单元按取号序排，相邻两把 key 反序的次数（D16（发布语义） 已定项 5 的切分纪律；C319 的计数）。
#[must_use]
pub fn count_key_order_mismatches(extent_keys_in_issue_order: &[[u8; 24]]) -> u64 {
    let field_wise = |key: &[u8; 24]| {
        (
            u64::from_le_bytes(key[0..8].try_into().expect("8")),
            u64::from_le_bytes(key[8..16].try_into().expect("8")),
            u64::from_le_bytes(key[16..24].try_into().expect("8")),
        )
    };
    u64::try_from(
        extent_keys_in_issue_order
            .windows(2)
            .filter(|pair| field_wise(&pair[0]) >= field_wise(&pair[1]))
            .count(),
    )
    .expect("次数")
}

/// 一个文件对象这次发布的数据单元：单元字节与指针。重写时由 `build_file_version_units` 装，照抄时从上一版取
/// （`carried_file_version_units`）。extent 树的节点不在这里——它们按这些指针由 `build_extent_tree` 装；
/// 这个对象的 inode 记录也不在这里——它进 inode 树，由 `build_inode_tree_units` 按叶容器装。
pub(super) struct FileVersionUnits {
    /// 文件的数据单元，第 i 项是文件第 i 个单元（与 `data_pointers` 同序）。照抄的那几个与上一版共用同一份字节（`carried_file_version_units`）。
    pub(super) data_units: Vec<Arc<[u8]>>,
    /// 指向每个数据单元的指针，第 i 项是文件第 i 个单元；extent 树下段的叶记录（或内联的那一条）按同一个次序装（key 升序）。
    pub(super) data_pointers: Vec<DataPointer>,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数；照抄的一版恒 0。
    pub(super) key_order_mismatches: u64,
}

/// 装文件对象的单元时整个 checkpoint 共用的身份字段。
pub(super) struct FileVersionCheckpoint<'checkpoint> {
    pub(super) txg: CheckpointTxg,
    /// 提交内生块（码 3 容器）头里的写序：这次发布末条记录那个事务（共享的提交内生块只在末条点名，
    /// D23（journal 的角色与格式） 已定项 17）。数据单元各带自己那个事务的号，不取它（`build_file_version_units`）。
    pub(super) write_order: WriteOrder,
    /// 这一版那十二棵树各自的号（单元头与指针头部的出生树按它写）。
    pub(super) trees: FileVersionTreeIdentifiers,
    pub(super) filesystem_identifier: &'checkpoint [u8; 16],
    /// 位置条目按设备身份升序（I-2.5）。
    pub(super) device_identities: &'checkpoint [DeviceIdentity],
}

/// 一个文件对象带位置条目的那几个数据单元这次取到的落点，第 i 项是文件第 i 个单元。
#[derive(Clone, Debug)]
pub(super) struct FileVersionSlots {
    pub(super) data: Vec<SlotNumber>,
}

/// 装一个文件对象的数据单元（字节表二）：切分纪律切出的每个一单元事务装一个数据单元（写序带那个事务的号，
/// 锚点偏移与 extent key 的 offset 段都是这个单元第一个字节的文件偏移，D8（核心索引结构） 已定项 3、D9（加密） 已定项 6）。
/// 数据单元是码 1、不发出生序号；extent 树的节点按这里交出的指针另装（`build_extent_tree`）。
///
/// # Panics
/// 事务数与落点数不等：角色清单按同一张切分排，不等说明调用方给错了输入。
pub(super) fn build_file_version_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    file: &FileVersionPlan<'_>,
    transactions: &[OneUnitTransaction],
    slots: &FileVersionSlots,
) -> FileVersionUnits {
    let txg = checkpoint.txg;
    let instance = checkpoint.write_order.instance;
    let filesystem_identifier = checkpoint.filesystem_identifier;
    assert_eq!(
        transactions.len(),
        slots.data.len(),
        "每个一单元事务一个数据单元落点：角色清单与切分读的是同一份内容"
    );
    let mut data_units = Vec::with_capacity(transactions.len());
    let mut data_pointers = Vec::with_capacity(transactions.len());
    let mut extent_keys: Vec<[u8; 24]> = Vec::with_capacity(transactions.len());
    // 迭代次数的上界是这次的单元数；跨轮携带的只有往三个表里追加的那一项，没有提前出口。
    for (transaction, slot) in transactions.iter().zip(&slots.data) {
        let write_order = WriteOrder {
            instance,
            transaction: transaction.transaction_number,
        };
        // t1（字节表二）：五元组的锚点偏移是这个单元第一个字节在文件里的偏移。
        let data_identity = DataUnitIdentity {
            tree: checkpoint.trees.extent,
            object: file.inode.0,
            object_birth: file.inode_object_birth,
            anchor_offset: transaction.payload_start.0,
        };
        let data_unit = build_data_unit(
            data_identity,
            txg,
            filesystem_identifier,
            write_order,
            transaction.payload_of(file.content),
        );
        let data_pointer = DataPointer {
            head: PointerHead {
                birth_tree: checkpoint.trees.extent,
                birth_txg: txg,
            },
            locations: location_entries(checkpoint.device_identities, *slot, &data_unit),
            write_order,
        };
        let extent_record =
            build_extent_record(file.inode.0, transaction.payload_start.0, data_pointer);
        extent_keys.push(
            extent_record[..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
                .try_into()
                .expect("24"),
        );
        data_units.push(Arc::from(data_unit));
        data_pointers.push(data_pointer);
    }
    let key_order_mismatches = count_key_order_mismatches(&extent_keys);
    FileVersionUnits {
        data_units,
        data_pointers,
        key_order_mismatches,
    }
}

/// 这一版 extent 树装好的样子：每个节点这一版的指针、全部节点的单元（bump 次序：第一个文件的下段先叶后根、第一个文件之外各文件的下段
/// （按 inode 号，各自先叶后根）、上段先叶后根），与这次重写的节点各自的出生序号。
pub(super) struct BuiltExtentTree {
    pub(super) version: ExtentTreeVersion,
    /// 第一个文件之外各文件下段这一版的节点与指针，按 inode 号（与 [`FilesOtherThanTheFirstPlan::files`] 同序；内联的文件是空的）。
    pub(super) lower_nodes_of_the_files_other_than_the_first:
        Vec<(InodeNumber, Vec<(ExtentLowerNodePosition, NodePointer)>)>,
    pub(super) units: Vec<PublishedUnit>,
    pub(super) birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
}

/// 按计划装 extent 树这一版的全部节点（D8（核心索引结构） 已定项 14 的两段）：重写的节点按 bump 次序逐个装、发出生序号、取这次的落点——
/// 下段先叶后根（叶装它罩的那几个单元的 extent 叶记录，内部节点装「孩子那一段的起点 key + 孩子这一版的指针」；第一个文件的下段在前，
/// 第一个文件之外各文件的下段按 inode 号接着），再上段先叶后根（重写的一片叶装它罩的那几个 inode 号上每个有数据单元的文件那一条叶条目：
/// 一个数据单元就内联它的数据指针（标签 2），多个就放下段根这一版的指针（标签 1）；第一个文件尾部有洞的写标签 3、4，一个单元都没有而
/// 记录还在的写标签 0，size 0 也写。条目都取这一版内存里的指针）。
/// 照抄的节点字节、落点、指针全取上一版同一个位置上的。`data_pointers` 是第一个文件这一版每个数据单元的指针（第 i 项是单元 i），
/// `size_of_the_first_file_after_this_publish` 是这一版第一个文件那条记录的 size（这一版没有那条记录时为 `None`：不写它那一条），
/// `data_pointers_of_the_files_other_than_the_first` 是第一个文件之外各文件这一版的数据指针（与 `files_other_than_the_first.files` 同序）。
///
/// # Panics
/// 重写的节点没拿到落点、照抄的节点不在上一版里、两份第一个文件之外的清单次序不一：三样都是发布路径自己的不变量
/// （角色清单按同一份计划排、上一版的字节是这个进程装的或重建时核过的、两份清单由 `publish_admitted` 按同一份计划拼）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：计划、第一个文件之外各文件的计划、上一版、上一版按身份的两份索引、第一个文件的数据指针、它这一版的 size、别的文件的数据指针、树号、身份、落点、出生序号"
)]
pub(super) fn build_extent_tree(
    plan: &ExtentTreePlan,
    files_other_than_the_first: &FilesOtherThanTheFirstPlan,
    previous: Option<&TransactionOutput>,
    units_of_the_previous_version: Option<
        &FirstEntryOfEachIdentity<'_, TransactionUnit, PublishedUnit>,
    >,
    files_of_the_previous_version: Option<
        &FirstEntryOfEachIdentity<'_, InodeNumber, FileOfAnInodeOtherThanTheFirst>,
    >,
    data_pointers: &[DataPointer],
    size_of_the_first_file_after_this_publish: Option<u64>,
    data_pointers_of_the_files_other_than_the_first: &[(InodeNumber, Vec<DataPointer>)],
    tree: TreeIdentifier,
    context: &MultiLevelTreeBuildContext<'_>,
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> BuiltExtentTree {
    // 装整棵 extent 树都在照抄那一处里（七·6 后续批代码轮第一轮判决第三节第 3 条）：每照抄一个节点在这里面现建一份上一版的单元索引的，
    // 建的那几份记在这一处，不漏到没点名的那一格。按 inode 号找文件的那几次在里面另进一处。
    let _inside_the_extent_tree_build = lookup_work::entering(LookupSite::CarriedExtentTreeNodes);
    let identity = ExtentTreeNodeIdentity {
        tree,
        birth_txg: context.txg,
        filesystem_identifier: context.filesystem_identifier,
        instance: context.instance,
    };
    let data_units = u64::try_from(data_pointers.len()).expect("单元数");
    let tail_of_the_first_file = size_of_the_first_file_after_this_publish
        .map_or(TailOfAFile::NoHoleAfterTheLastDataUnit, |size_in_bytes| {
            TailOfAFile::of_data_units_and_size(data_units, size_in_bytes)
        });
    let mut units: Vec<PublishedUnit> = Vec::new();
    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
    let mut rewritten_node = |role: TransactionUnit,
                              build: &dyn Fn(BirthSequence) -> Vec<u8>,
                              units_so_far: &mut Vec<PublishedUnit>|
     -> NodePointer {
        let birth_sequence = sequences.next(tree, context.txg, context.instance);
        let bytes = build(birth_sequence);
        let slot = *slots
            .get(&role)
            .expect("这次重写的 extent 树节点都在角色清单里、取了落点");
        birth_sequences_of_rewritten_nodes.insert(role, birth_sequence);
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: tree,
                birth_txg: context.txg,
            },
            locations: location_entries(context.device_identities, slot, &bytes),
            instance: context.instance,
            birth_sequence,
        };
        units_so_far.push(PublishedUnit {
            slot,
            identity: role,
            bytes: Arc::from(bytes),
        });
        pointer
    };
    let carried_node = |role_in_the_previous_version: TransactionUnit,
                        role: TransactionUnit,
                        units_so_far: &mut Vec<PublishedUnit>| {
        let _inside_the_carried_nodes = lookup_work::entering(LookupSite::CarriedExtentTreeNodes);
        let carried = units_of_the_previous_version
            .expect("照抄的 extent 树节点只出现在接着上一版的发布里")
            .unit(role_in_the_previous_version);
        units_so_far.push(PublishedUnit {
            slot: carried.slot,
            identity: role,
            bytes: carried.bytes.clone(),
        });
    };
    let mut lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)> = Vec::new();
    // 迭代上界是下段的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
    for (position, origin) in &plan.lower_nodes {
        let role = TransactionUnit::ExtentLowerNode(*position);
        let pointer = match origin {
            ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => rewritten_node(
                role,
                &|birth_sequence| {
                    build_lower_leaf(
                        &identity,
                        FIRST_INODE_NUMBER,
                        *position,
                        data_pointers,
                        birth_sequence,
                    )
                },
                &mut units,
            ),
            ExtentTreeNodeOrigin::RewrittenThisPublish => {
                let children: Vec<(ExtentLowerNodePosition, NodePointer)> =
                    lower_children_of_a_file_without_holes(*position, data_units)
                        .into_iter()
                        .map(|child| {
                            let (_, pointer) = lower_nodes
                                .iter()
                                .find(|(built, _)| *built == child)
                                .expect("孩子在 bump 次序里排在父节点前面、先装好了");
                            (child, *pointer)
                        })
                        .collect();
                rewritten_node(
                    role,
                    &|birth_sequence| {
                        build_lower_internal_node(
                            &identity,
                            FIRST_INODE_NUMBER,
                            *position,
                            &children,
                            birth_sequence,
                        )
                    },
                    &mut units,
                )
            }
            ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                carried_node(role, role, &mut units);
                previous
                    .and_then(|previous_version| {
                        previous_version.extent_tree.lower_pointer_of(*position)
                    })
                    .expect("照抄的下段节点在上一版里")
            }
        };
        lower_nodes.push((*position, pointer));
    }
    // 第一个文件之外各文件的下段：这次写的那一个按它的数据指针整段重装（同第一个文件），别的整段照抄。迭代上界是各文件下段的节点数之和。
    let mut lower_nodes_of_the_files_other_than_the_first: Vec<(
        InodeNumber,
        Vec<(ExtentLowerNodePosition, NodePointer)>,
    )> = Vec::with_capacity(files_other_than_the_first.files.len());
    assert_eq!(
        files_other_than_the_first.files.len(),
        data_pointers_of_the_files_other_than_the_first.len(),
        "第一个文件之外各文件的计划与数据指针按同一份计划拼"
    );
    for (file_plan, (inode, file_data_pointers)) in files_other_than_the_first
        .files
        .iter()
        .zip(data_pointers_of_the_files_other_than_the_first)
    {
        assert_eq!(
            file_plan.inode, *inode,
            "第一个文件之外各文件的计划与数据指针按同一个次序（inode 号升序）"
        );
        let mut file_lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)> =
            Vec::with_capacity(file_plan.lower_nodes.len());
        for (position, origin) in &file_plan.lower_nodes {
            let role = file_plan.lower_node_role(*position);
            let pointer = match origin {
                ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => {
                    rewritten_node(
                        role,
                        &|birth_sequence| {
                            build_lower_leaf(
                                &identity,
                                inode.0,
                                *position,
                                file_data_pointers,
                                birth_sequence,
                            )
                        },
                        &mut units,
                    )
                }
                ExtentTreeNodeOrigin::RewrittenThisPublish => {
                    let children: Vec<(ExtentLowerNodePosition, NodePointer)> =
                        lower_children_of_a_file_without_holes(*position, file_plan.data_units)
                            .into_iter()
                            .map(|child| {
                                let (_, pointer) = file_lower_nodes
                                    .iter()
                                    .find(|(built, _)| *built == child)
                                    .expect("孩子在 bump 次序里排在父节点前面、先装好了");
                                (child, *pointer)
                            })
                            .collect();
                    rewritten_node(
                        role,
                        &|birth_sequence| {
                            build_lower_internal_node(
                                &identity,
                                inode.0,
                                *position,
                                &children,
                                birth_sequence,
                            )
                        },
                        &mut units,
                    )
                }
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                    carried_node(role, role, &mut units);
                    let _inside_the_carried_files =
                        lookup_work::entering(LookupSite::FilesOfTheCarriedExtentLowerNodes);
                    files_of_the_previous_version
                        .and_then(|previous_files| previous_files.first_entry_of(inode))
                        .and_then(|previous_file| {
                            previous_file
                                .extent_lower_nodes
                                .iter()
                                .find(|(position_in_the_previous_version, _)| {
                                    position_in_the_previous_version == position
                                })
                                .map(|(_, pointer)| *pointer)
                        })
                        .expect("照抄的那个文件的下段节点在上一版里")
                }
            };
            file_lower_nodes.push((*position, pointer));
        }
        lower_nodes_of_the_files_other_than_the_first.push((*inode, file_lower_nodes));
    }
    // 第一个文件之外各文件在上段叶里那一条的载荷（按 inode 号）：下段有根就指下段根这一版的指针，只有一个单元就内联它，最后一个单元之后
    // 到 size 是洞的换成标签 3、4；一个单元都没有的写标签 0（粘住的标签 0，判决 m4-multi-inode-size-r1 第二节 S2：这个文件在计划里就是
    // 上段里有它那一条，重写这片叶的发布照写、不丢）。
    let upper_leaf_entries_of_the_files_other_than_the_first: Vec<ExtentUpperLeafEntry> =
        lower_nodes_of_the_files_other_than_the_first
            .iter()
            .zip(data_pointers_of_the_files_other_than_the_first)
            .zip(&files_other_than_the_first.files)
            .map(
                |(((inode, file_lower_nodes), (_, file_data_pointers)), file_plan)| {
                    let target = match (file_lower_nodes.last(), file_data_pointers.first()) {
                        (Some((_, lower_root_pointer)), _) => {
                            ExtentUpperLeafTarget::LowerSegmentRoot(*lower_root_pointer)
                        }
                        (None, Some(only_pointer)) => {
                            ExtentUpperLeafTarget::InlineDataUnit(*only_pointer)
                        }
                        (None, None) => ExtentUpperLeafTarget::NoDataUnit,
                    }
                    .with_the_tail(file_plan.tail);
                    ExtentUpperLeafEntry {
                        inode: inode.0,
                        target,
                    }
                },
            )
            .collect();
    let upper_root_level = plan.upper_root_level();
    let previous_upper_root_level =
        previous.map(|previous_version| previous_version.extent_tree.upper_root().level);
    let mut upper_nodes: Vec<(ExtentUpperNodePosition, NodePointer)> = Vec::new();
    // 迭代上界是上段的节点数；跨轮携带的同下段。
    for (position, origin) in &plan.upper_nodes {
        let role = role_of_extent_upper_node(*position, upper_root_level);
        let role_in_the_previous_version = previous_upper_root_level
            .map(|previous_root_level| role_of_extent_upper_node(*position, previous_root_level));
        let pointer = match origin {
            ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => {
                // 第一个文件这一版的那条叶条目：下段有根就指下段根，只有一个单元就内联它；一个单元都没有时按记录在不在取
                // （`ExtentUpperLeafTarget::of_a_file_without_data_units`）：记录在就照写标签 0，size 0 与 size > 0 一样（判决 m4-trunc-r1 K5、S1，
                // m4-validate-code-r1 用户定案「丑」：重写这片叶的发布不丢这一条；m4-validate-code-r2 T1 用户定「甲」：截到 0 之后也不丢）；
                // 记录没了（unlink 最后一批）就不写这一条，只剩它一条时这片叶成了零条目根（D8（核心索引结构） 已定项 16 第 5 条）。
                // 最后一个单元之后到 size 是洞的换成标签 3、4（判决 m4-trunc-r1 K5，`ExtentUpperLeafTarget::with_the_tail`）。
                let target = match (lower_nodes.last(), data_pointers.first()) {
                    (Some((_, lower_root_pointer)), _) => {
                        Some(ExtentUpperLeafTarget::LowerSegmentRoot(*lower_root_pointer))
                    }
                    (None, Some(only_data_pointer)) => {
                        Some(ExtentUpperLeafTarget::InlineDataUnit(*only_data_pointer))
                    }
                    (None, None) => ExtentUpperLeafTarget::of_a_file_without_data_units(
                        size_of_the_first_file_after_this_publish,
                    ),
                }
                .map(|target| target.with_the_tail(tail_of_the_first_file));
                // 这片叶罩的 inode 号上第一个文件之外的文件，条目照这一版内存里的指针装（照抄的文件的指针整理之后已换成映射位置，
                // C417（父节点重写时刷新提示）的第一版政策「随重写刷新」）。
                let (first_inode_of_the_leaf, last_inode_of_the_leaf) = position.inode_range();
                let mut entries: Vec<ExtentUpperLeafEntry> =
                    upper_leaf_entries_of_the_files_other_than_the_first
                        .iter()
                        .filter(|entry| {
                            (first_inode_of_the_leaf..=last_inode_of_the_leaf)
                                .contains(&entry.inode)
                        })
                        .copied()
                        .collect();
                let the_leaf_covers_the_first_file = (first_inode_of_the_leaf
                    ..=last_inode_of_the_leaf)
                    .contains(&FIRST_INODE_NUMBER);
                if let (Some(target), true) = (target, the_leaf_covers_the_first_file) {
                    entries.push(ExtentUpperLeafEntry {
                        inode: FIRST_INODE_NUMBER,
                        target,
                    });
                }
                entries.sort_by_key(|entry| entry.inode);
                rewritten_node(
                    role,
                    &|birth_sequence| {
                        build_upper_leaf(&identity, *position, &entries, birth_sequence)
                    },
                    &mut units,
                )
            }
            ExtentTreeNodeOrigin::RewrittenThisPublish => {
                let children: Vec<(ExtentUpperNodePosition, NodePointer)> = upper_nodes
                    .iter()
                    .filter(|(child, _)| {
                        child.level + 1 == position.level && child.parent() == *position
                    })
                    .copied()
                    .collect();
                rewritten_node(
                    role,
                    &|birth_sequence| {
                        build_upper_internal_node(&identity, *position, &children, birth_sequence)
                    },
                    &mut units,
                )
            }
            ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                carried_node(
                    role_in_the_previous_version.expect("照抄的上段节点只出现在接着上一版的发布里"),
                    role,
                    &mut units,
                );
                previous
                    .and_then(|previous_version| {
                        previous_version.extent_tree.upper_pointer_of(*position)
                    })
                    .expect("照抄的上段节点在上一版里")
            }
        };
        upper_nodes.push((*position, pointer));
    }
    BuiltExtentTree {
        version: ExtentTreeVersion {
            upper_nodes,
            lower_nodes,
        },
        lower_nodes_of_the_files_other_than_the_first,
        units,
        birth_sequences_of_rewritten_nodes,
    }
}

/// 这一版分配记录树装好的样子：每个节点这一版的指针（bump 次序）、这次重写的节点的单元（bump 次序），与它们各自的出生序号。
pub(super) struct BuiltAllocationRecordTree {
    pub(super) version: AllocationRecordTreeVersion,
    pub(super) rewritten_units: Vec<PublishedUnit>,
    pub(super) birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
}

/// 按计划装分配记录树这一版的全部节点（D8（核心索引结构） 已定项 14）：重写的节点按 bump 次序（按位置先叶后根）逐个装——叶装它罩的那一段里的全部记录
/// （按 key 升序），内部节点装它每个孩子「那一段的起点 key + 孩子这一版的指针」；头里的 key 区间是这个节点按位置规定罩的那一段。
/// 出生序号按同一个次序发，落点取这次分配的。照抄的节点只取指针（`previous_pointer_of`）：带文件的一版上它的字节由调用方从上一版照抄进 `units`，
/// 树表 0 条的一版上不写照抄的节点。`records` 是这次发布取完落点之后分配器里的全部记录。
///
/// # Panics
/// 重写的节点没拿到落点，或照抄的节点在上一版里没有指针：角色清单按同一份计划排，上一版的节点是这个进程记的或重建时读回来的。
#[allow(
    clippy::too_many_arguments,
    reason = "装一棵树要的：计划、几何、记录、上一版节点的指针、树号、身份字段、落点、发号器，各自独立"
)]
pub(super) fn build_allocation_record_tree(
    plan: &AllocationRecordTreePlan,
    geometry: &AllocationRecordTreeGeometry,
    records: &[AllocationRecord],
    previous_pointer_of: &dyn Fn(AllocationRecordTreeNode) -> Option<NodePointer>,
    tree: TreeIdentifier,
    context: &MultiLevelTreeBuildContext<'_>,
    slot_of_rewritten_role: &dyn Fn(TransactionUnit) -> SlotNumber,
    sequences: &mut BirthSequenceAllocator,
) -> BuiltAllocationRecordTree {
    let records_by_leaf = records_of_each_leaf(geometry, records);
    let node_set = plan.node_set();
    let mut pointers: BTreeMap<AllocationRecordTreeNode, NodePointer> = BTreeMap::new();
    let mut rewritten_units = Vec::new();
    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
    // 迭代上界是这一版的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
    for (node, origin) in &plan.nodes {
        let role = role_of_allocation_record_tree_node(*node);
        let pointer = match origin {
            AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion => {
                previous_pointer_of(*node).expect("照抄的分配记录树节点在上一版里有指针")
            }
            AllocationRecordTreeNodeOrigin::RewrittenThisPublish => {
                let birth_sequence = sequences.next(tree, context.txg, context.instance);
                let contents = match geometry.level_of(*node) {
                    0 => AllocationRecordTreeNodeContents::Leaf(
                        records_by_leaf
                            .get(node)
                            .map(Vec::as_slice)
                            .expect("这一版的叶都有记录（规划按有记录的叶排）"),
                    ),
                    _ => AllocationRecordTreeNodeContents::Internal(
                        children_of(geometry, &node_set, *node)
                            .into_iter()
                            .map(|child| {
                                (
                                    child,
                                    *pointers
                                        .get(&child)
                                        .expect("孩子在 bump 次序里排在父节点前面、先装好了"),
                                )
                            })
                            .collect(),
                    ),
                };
                let bytes = build_allocation_record_tree_node(
                    geometry,
                    *node,
                    &contents,
                    tree,
                    context.txg,
                    context.filesystem_identifier,
                    context.instance,
                    birth_sequence,
                );
                let slot = slot_of_rewritten_role(role);
                birth_sequences_of_rewritten_nodes.insert(role, birth_sequence);
                let pointer = NodePointer {
                    head: PointerHead {
                        birth_tree: tree,
                        birth_txg: context.txg,
                    },
                    locations: location_entries(context.device_identities, slot, &bytes),
                    instance: context.instance,
                    birth_sequence,
                };
                rewritten_units.push(PublishedUnit {
                    slot,
                    identity: role,
                    bytes: Arc::from(bytes),
                });
                pointer
            }
        };
        pointers.insert(*node, pointer);
    }
    BuiltAllocationRecordTree {
        version: AllocationRecordTreeVersion {
            nodes: plan
                .nodes
                .iter()
                .map(|(node, _)| (*node, pointers[node]))
                .collect(),
        },
        rewritten_units,
        birth_sequences_of_rewritten_nodes,
    }
}

/// 这一版 inode 树的全部单元：每片叶容器（重写的是这次装的、没重写的照抄上一版），加上重写了的根。
pub(super) struct InodeTreeUnits {
    /// 左起按 key 序，每片一项。
    pub(super) leaf_containers: Vec<InodeLeafContainerUnit>,
    /// 这次有叶容器被重写 ⇒ 根跟着重写（COW 叶 + 全部祖先，D23（journal 的角色与格式） 已定项 1 甲）；
    /// 一片都没重写（写行、暖机）⇒ `None`，根指针照抄上一版的树表条目、连出生序号都不发。
    pub(super) rewritten_root: Option<InodeTreeRootUnit>,
}

/// 这次重写出来的 inode 树根。
pub(super) struct InodeTreeRootUnit {
    pub(super) bytes: Vec<u8>,
    pub(super) birth_sequence: BirthSequence,
}

/// 这一版 inode 树里的一片叶容器：装了什么、这一版的指针、这一版的字节，以及它从哪来（这次重写，或照抄上一版哪个叶序上的那一片）。
pub(super) struct InodeLeafContainerUnit {
    pub(super) index: InodeLeafContainerIndexInTree,
    pub(super) contents: InodeLeafContainer,
    pub(super) pointer: NodePointer,
    pub(super) bytes: Vec<u8>,
    pub(super) origin: InodeLeafContainerOrigin,
}

/// 装这次发布的 inode 树（字节表四）：先按叶序给这次重写的每一片发出生序号、装码 3 容器，再装码 2 根
/// （树内先叶后根，D3（空间分配） 已定项 10 ⑤ 的 bump 次序与 D19（块指针的结构与宽度预算） 已定项 9 的发号次序）。
/// 没重写的那几片一个字节都不动：字节、落点、指针、出生序号全取上一版它所在的那个叶序（分裂时的左半走的就是这一支；
/// 左边有片身份消失时，它在上一版里的叶序比这一版大，`InodeLeafContainersAfterThisPublish::origins` 记着）。
///
/// # Panics
/// 这次重写的某一片没拿到落点，或某一片既不在这次的重写清单里、上一版里也没有——两样都说明角色清单
/// （`PublishPlan::rewritten_roles`）与这里算的树对不上，那是发布路径自己的不变量。
pub(super) fn build_inode_tree_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    inode_tree: &InodeLeafContainersAfterThisPublish,
    previous_containers: &[InodeLeafContainerVersion],
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> InodeTreeUnits {
    let txg = checkpoint.txg;
    let instance = checkpoint.write_order.instance;
    let inode_tree_identifier = checkpoint.trees.inode;
    let mut leaf_containers = Vec::with_capacity(inode_tree.containers.len());
    for ((position, contents), origin) in inode_tree
        .containers
        .iter()
        .enumerate()
        .zip(&inode_tree.origins)
    {
        let index = InodeLeafContainerIndexInTree::of_position(position);
        let role = TransactionUnit::InodeLeafContainer(index);
        if let InodeLeafContainerOrigin::CarriedFrom(previous_index) = origin {
            let carried = previous_containers
                .get(previous_index.position())
                .expect("照抄的叶容器一定在上一版里（新建的那几片恒在重写清单里）");
            leaf_containers.push(InodeLeafContainerUnit {
                index,
                contents: contents.clone(),
                pointer: carried.pointer,
                bytes: Vec::new(),
                origin: *origin,
            });
        } else {
            let birth_sequence = sequences.next(inode_tree_identifier, txg, instance);
            let bytes = build_packed_unit(
                contents.identity,
                u16::try_from(INODE_RECORD_BYTES).expect("140"),
                &contents
                    .records
                    .iter()
                    .map(InodeRecord::to_bytes)
                    .collect::<Vec<_>>(),
                txg,
                checkpoint.filesystem_identifier,
                checkpoint.write_order,
                birth_sequence,
            );
            let pointer = NodePointer {
                head: PointerHead {
                    birth_tree: inode_tree_identifier,
                    birth_txg: txg,
                },
                locations: location_entries(
                    checkpoint.device_identities,
                    *slots.get(&role).expect("这次重写的叶容器都取了落点"),
                    &bytes,
                ),
                instance,
                birth_sequence,
            };
            leaf_containers.push(InodeLeafContainerUnit {
                index,
                contents: contents.clone(),
                pointer,
                bytes,
                origin: *origin,
            });
        }
    }
    if !inode_tree.is_the_root_rewritten() {
        return InodeTreeUnits {
            leaf_containers,
            rewritten_root: None,
        };
    }
    // t4 inode 树根：层级 1，每片叶一条 120 字节条目（按分隔 key 升序）。
    // key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2「孩子是码 3 容器的内部节点上，max_key 是最后那片容器里最后一条记录的 key，
    // 不是最后一条条目的分隔 key」）：[最左那片的最小记录号, 最右那片的最大记录号]。不取首末两条分隔 key——最右那片装到第二条记录起、
    // 最左那片删掉了打头那几条之后，子树里到得了的 key 就不是分隔 key 了。叶序 = key 升序、容器里记录按号升序，所以首末两片的首末两条就是全树的最小、最大。
    // 一片叶都不剩（unlink 删掉树里最后一条记录）⇒ 零条目根，key 区间 [0, 0]（D8（核心索引结构） 已定项 6、已定项 16 第 5 条）。
    let root_sequence = sequences.next(inode_tree_identifier, txg, instance);
    let smallest_key = leaf_containers
        .first()
        .map_or(0, |container| {
            container
                .contents
                .smallest_inode_number()
                .expect("树里的叶容器至少装一条记录（记录数为 0 的类型 2 容器不落盘，D8（核心索引结构） 已定项 6）")
                .0
        })
        .to_le_bytes();
    let largest_key = leaf_containers
        .last()
        .map_or(0, |container| {
            container
                .contents
                .largest_inode_number()
                .expect("树里的叶容器至少装一条记录（记录数为 0 的类型 2 容器不落盘，D8（核心索引结构） 已定项 6）")
                .0
        })
        .to_le_bytes();
    let root_unit = build_index_node(
        inode_tree_identifier,
        1,
        INODE_KEY_WIDTH_IN_BYTES,
        &smallest_key,
        &largest_key,
        txg,
        checkpoint.filesystem_identifier,
        instance,
        root_sequence,
        u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
        &leaf_containers
            .iter()
            .map(|container| {
                build_inode_internal_entry(
                    container.contents.separator_key(),
                    container.contents.identity,
                    container.pointer,
                )
            })
            .collect::<Vec<_>>(),
    );
    InodeTreeUnits {
        leaf_containers,
        rewritten_root: Some(InodeTreeRootUnit {
            bytes: root_unit,
            birth_sequence: root_sequence,
        }),
    }
}

/// 照抄上一版的数据单元：数据指针与每个数据单元的字节都取上一版（extent 树的节点照抄在 `build_extent_tree` 里）。
/// 这次重写的一个删除树节点：字节与出生序号。
pub(super) struct RewrittenDeletionTreeNode {
    pub(super) bytes: Vec<u8>,
    pub(super) birth_sequence: BirthSequence,
}

/// 一版里删除那两棵树的一个节点：这一版的指针，这次重写了的话带它的字节与出生序号（照抄的字节从上一版拷）。
pub(super) struct DeletionTreeNodeUnit {
    pub(super) pointer: NodePointer,
    pub(super) rewritten: Option<RewrittenDeletionTreeNode>,
}

/// 一版里容器树（墓碑树、谱系树）的一片叶容器。
pub(super) struct RecordContainerUnit<Index, Record> {
    pub(super) index: Index,
    pub(super) contents: RecordContainer<Record>,
    pub(super) node: DeletionTreeNodeUnit,
}

/// 一版里容器树根之下的一个码 2 节点：位置、这一版的角色、照抄时它在上一版里的角色（长高那次旧根上一版是根）、指针与重写的字节。
pub(super) struct ContainerTreeNodeBelowTheRootUnit {
    pub(super) position: ContainerTreeNodePosition,
    pub(super) identity: TransactionUnit,
    pub(super) identity_in_the_previous_version: TransactionUnit,
    pub(super) node: DeletionTreeNodeUnit,
}

/// 一版里一棵容器树的全部单元：每片容器（叶序）、根之下的码 2 节点（bump 次序）、根（零根时没有）。
pub(super) struct ContainerTreeUnits<Index, Record> {
    pub(super) containers: Vec<RecordContainerUnit<Index, Record>>,
    pub(super) nodes_below_the_root: Vec<ContainerTreeNodeBelowTheRootUnit>,
    pub(super) root: Option<DeletionTreeNodeUnit>,
}

/// 这一版删除那两棵树与谱系树的全部单元（D8（核心索引结构） 已定项 16、D18（块里携带什么信息） 已定项 24）：`deleted_inodes` 根（队列空时没有），
/// 墓碑树、谱系树各自的每片容器、根之下的节点与根（零根时都没有）。
pub(super) struct DeletionTreeUnits {
    pub(super) deleted_inodes_root: Option<DeletionTreeNodeUnit>,
    pub(super) tombstone: ContainerTreeUnits<TombstoneContainerIndexInTree, TombstoneRecord>,
    pub(super) lineage: ContainerTreeUnits<LineageContainerIndexInTree, LineageRecord>,
    /// logged_ops 根（没有条目时没有，判决 G1）。
    pub(super) logged_operations_root: Option<DeletionTreeNodeUnit>,
    pub(super) retirement: ContainerTreeUnits<RetirementContainerIndexInTree, RetirementRecord>,
}

/// 装一棵容器树（墓碑树、谱系树）这一版的单元要的几样：哪棵树、这次之后的样子、记录宽与记录的字节、三种角色、
/// 上一版的容器指针、根之下节点的指针与根指针从哪取。
struct ContainerTreeToBuild<'plans, Index, Record> {
    tree: TreeIdentifier,
    after_this_publish: &'plans ContainerTreeAfterThisPublish<Record>,
    record_width_in_bytes: u64,
    record_bytes: fn(&Record) -> Vec<u8>,
    index_of_position: fn(usize) -> Index,
    roles: ContainerTreeRoles,
    carried_container_pointer: &'plans dyn Fn(usize) -> NodePointer,
    carried_node_below_the_root_pointer: &'plans dyn Fn(ContainerTreeNodePosition) -> NodePointer,
    carried_root_pointer: &'plans dyn Fn() -> NodePointer,
}

/// 装一棵容器树这一版的单元：先按叶序给重写的每片容器发出生序号、装码 3 容器，再按 bump 次序（层级升序、根最末）装码 2 节点
/// （条目照 inode 树内部节点的 120 字节形态：分隔 key 8 + 身份引用 26 + 子指针 86。层级 1 的孩子是容器，身份引用取容器的；层级更高的
/// 孩子是码 2 节点，身份引用全 0、类型段 0，D8（核心索引结构） 已定项 6；分隔 key 是孩子那一枝最左那片容器的号，key 区间是子树覆盖区间 [最左那片的号, 最右那片的号]，D18（块里携带什么信息） 已定项 2）。
/// 没重写的照抄上一版同一个位置的指针（长高那次的旧根取上一版的根指针）。节点有没有重写跟着「最右那一条路径上有容器重写」走（COW 叶 + 祖先）。
fn build_container_tree_units<Index: Copy, Record>(
    to_build: &ContainerTreeToBuild<'_, Index, Record>,
    checkpoint: &FileVersionCheckpoint<'_>,
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> ContainerTreeUnits<Index, Record>
where
    Record: Clone,
{
    let txg = checkpoint.txg;
    let instance = checkpoint.write_order.instance;
    let after = to_build.after_this_publish;
    let pointer_of = |role: TransactionUnit, bytes: &[u8], birth_sequence| NodePointer {
        head: PointerHead {
            birth_tree: to_build.tree,
            birth_txg: txg,
        },
        locations: location_entries(
            checkpoint.device_identities,
            *slots
                .get(&role)
                .expect("这次重写的容器树节点都在角色清单里、取了落点"),
            bytes,
        ),
        instance,
        birth_sequence,
    };
    let mut containers = Vec::with_capacity(after.containers.len());
    // 重写的是最右端连着的那几片：序号不小于头一片就重写了，不在重写清单上逐个找。
    let first_rewritten_container = after.first_rewritten_container();
    for (position, contents) in after.containers.iter().enumerate() {
        let index = (to_build.index_of_position)(position);
        let role = (to_build.roles.container)(position);
        let node = if first_rewritten_container.is_some_and(|first| position >= first) {
            let birth_sequence = sequences.next(to_build.tree, txg, instance);
            let bytes = build_packed_unit(
                contents.identity,
                u16::try_from(to_build.record_width_in_bytes).expect("记录宽是 40、24 或 34"),
                &contents
                    .records
                    .iter()
                    .map(to_build.record_bytes)
                    .collect::<Vec<_>>(),
                txg,
                checkpoint.filesystem_identifier,
                checkpoint.write_order,
                birth_sequence,
            );
            DeletionTreeNodeUnit {
                pointer: pointer_of(role, &bytes, birth_sequence),
                rewritten: Some(RewrittenDeletionTreeNode {
                    bytes,
                    birth_sequence,
                }),
            }
        } else {
            DeletionTreeNodeUnit {
                pointer: (to_build.carried_container_pointer)(position),
                rewritten: None,
            }
        };
        containers.push(RecordContainerUnit {
            index,
            contents: contents.clone(),
            node,
        });
    }
    // 每个码 2 节点的指针与它那一枝最左、最右那片容器的号，按 bump 次序排：上一层的条目与 key 区间要读下一层已装好的这三样。
    let mut built_nodes: BTreeMap<ContainerTreeNodePosition, (NodePointer, u64, u64)> =
        BTreeMap::new();
    let mut nodes_below_the_root = Vec::new();
    let mut root = None;
    let root_position = after.shape.root_position();
    let root_before_this_publish = after.node_counts_before_this_publish.root_position();
    // 每个节点的孩子区间每层顺着累加一遍（一个一个问、每个都从这一层最左加起是这一层节点数的平方）。
    let children_of_every_node = after.shape.children_of_every_node();
    for (position, origin) in after.node_origins() {
        let identity = to_build.roles.of_node(position, root_position);
        let children = children_of_every_node[usize::from(position.level) - 1]
            [usize::try_from(position.index_in_level).expect("同层序号装得进 usize")]
        .clone();
        // 这个节点的条目：(分隔 key, 身份引用, 子指针)，按孩子的次序；连同每个孩子那一枝最右那片容器的号。
        let entries_and_largest_container_numbers: Vec<((u64, PackedIdentity, NodePointer), u64)> =
            children
                .map(|child| {
                    if position.level == 1 {
                        let container = &containers[child];
                        (
                            (
                                container.contents.separator_key(),
                                container.contents.identity,
                                container.node.pointer,
                            ),
                            container.contents.separator_key(),
                        )
                    } else {
                        let child_position = ContainerTreeNodePosition {
                            level: position.level - 1,
                            index_in_level: u32::try_from(child).expect("同层节点数装得进 u32"),
                        };
                        let (child_pointer, smallest_container_number, largest_container_number) =
                            built_nodes
                                .get(&child_position)
                                .copied()
                                .expect("bump 次序里下一层的节点都在上一层之前装好");
                        (
                            (
                                smallest_container_number,
                                IDENTITY_OF_A_CODE_TWO_CHILD,
                                child_pointer,
                            ),
                            largest_container_number,
                        )
                    }
                })
                .collect();
        let entries: Vec<(u64, PackedIdentity, NodePointer)> =
            entries_and_largest_container_numbers
                .iter()
                .map(|(entry, _)| *entry)
                .collect();
        // 节点头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2）：这棵树的 key 是容器号，最小的是第一个孩子那一枝最左那片的号
        // （= 第一条条目的分隔 key），最大的是最后一个孩子那一枝最右那片的号。层级 1 的节点上它就是末条条目的分隔 key；层级 ≥ 2 的不是——
        // 末条条目的分隔 key 是最后一个孩子那一枝**最左**那片的号。
        let smallest_container_number = entries.first().expect("形状里每个节点至少一个孩子").0;
        let largest_container_number = entries_and_largest_container_numbers
            .last()
            .expect("形状里每个节点至少一个孩子")
            .1;
        let node = match origin {
            ContainerTreeNodeOrigin::RewrittenThisPublish => {
                let birth_sequence = sequences.next(to_build.tree, txg, instance);
                let bytes = build_index_node(
                    to_build.tree,
                    position.level,
                    CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES,
                    &smallest_container_number.to_le_bytes(),
                    &largest_container_number.to_le_bytes(),
                    txg,
                    checkpoint.filesystem_identifier,
                    instance,
                    birth_sequence,
                    u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
                    &entries
                        .iter()
                        .map(|(separator_key, child_identity, child_pointer)| {
                            build_inode_internal_entry(
                                *separator_key,
                                *child_identity,
                                *child_pointer,
                            )
                        })
                        .collect::<Vec<_>>(),
                );
                DeletionTreeNodeUnit {
                    pointer: pointer_of(identity, &bytes, birth_sequence),
                    rewritten: Some(RewrittenDeletionTreeNode {
                        bytes,
                        birth_sequence,
                    }),
                }
            }
            ContainerTreeNodeOrigin::CarriedFromThePreviousVersion => DeletionTreeNodeUnit {
                pointer: if root_before_this_publish == Some(position) {
                    (to_build.carried_root_pointer)()
                } else {
                    (to_build.carried_node_below_the_root_pointer)(position)
                },
                rewritten: None,
            },
        };
        built_nodes.insert(
            position,
            (
                node.pointer,
                smallest_container_number,
                largest_container_number,
            ),
        );
        if root_position == Some(position) {
            root = Some(node);
        } else {
            nodes_below_the_root.push(ContainerTreeNodeBelowTheRootUnit {
                position,
                identity,
                identity_in_the_previous_version: to_build
                    .roles
                    .of_node(position, root_before_this_publish),
                node,
            });
        }
    }
    ContainerTreeUnits {
        containers,
        nodes_below_the_root,
        root,
    }
}

/// 装这次发布的删除那两棵树：`deleted_inodes` 根是码 2 层级 0（条目 = key 8 字节、无 value，key 区间是首末两个号），
/// 墓碑树先按叶序给重写的每片容器发出生序号、装码 3 类型 1 容器（记录 40 字节），再装码 2 层级 1 的根（条目照 inode 树内部节点的 120 字节形态：
/// 分隔 key 容器号 8 + 身份引用 26 + 子指针 86；key 区间是子树覆盖区间，层级 1 的根上就是首末两条条目的分隔 key）。没重写的照抄上一版的指针。
///
/// # Panics
/// 重写的节点没拿到落点、照抄的不在上一版里、`deleted_inodes` 的号多于一个根装得下的：前两样是角色清单与这里对不上，
/// 后一样在 `ordinary_allocation_part` 里已经拒成 `DeletedInodesQueueRefused`。
pub(super) fn build_deletion_tree_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    resolved: &ResolvedPublish,
    previous: Option<&TransactionOutput>,
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> DeletionTreeUnits {
    let txg = checkpoint.txg;
    let instance = checkpoint.write_order.instance;
    let pointer_of =
        |tree: TreeIdentifier, role: TransactionUnit, bytes: &[u8], birth_sequence| NodePointer {
            head: PointerHead {
                birth_tree: tree,
                birth_txg: txg,
            },
            locations: location_entries(
                checkpoint.device_identities,
                *slots
                    .get(&role)
                    .expect("这次重写的删除树节点都在角色清单里、取了落点"),
                bytes,
            ),
            instance,
            birth_sequence,
        };
    let carried_previous = || previous.expect("照抄的删除树节点只出现在接着上一版的发布里");
    let deleted_inodes_tree = checkpoint.trees.deleted_inodes;
    let deleted_inodes_root = match resolved.deleted_inodes_root {
        DeletedInodesRootOrigin::NoRoot => None,
        DeletedInodesRootOrigin::CarriedFromThePreviousVersion => Some(DeletionTreeNodeUnit {
            pointer: carried_previous().tree_root_pointer(deleted_inodes_tree.0),
            rewritten: None,
        }),
        DeletedInodesRootOrigin::RewrittenThisPublish => {
            let birth_sequence = sequences.next(deleted_inodes_tree, txg, instance);
            let entries = deleted_inodes_root_entries(&resolved.deleted_inodes)
                .expect("号数在 ordinary_allocation_part 里判过装得下一个根");
            let smallest_key = entries.first().expect("队列非空才有根").clone();
            let largest_key = entries.last().expect("队列非空才有根").clone();
            let bytes = build_index_node(
                deleted_inodes_tree,
                0,
                DELETED_INODES_KEY_WIDTH_IN_BYTES,
                &smallest_key,
                &largest_key,
                txg,
                checkpoint.filesystem_identifier,
                instance,
                birth_sequence,
                u16::try_from(DELETED_INODES_ENTRY_BYTES).expect("8"),
                &entries,
            );
            Some(DeletionTreeNodeUnit {
                pointer: pointer_of(
                    deleted_inodes_tree,
                    TransactionUnit::DeletedInodesRoot,
                    &bytes,
                    birth_sequence,
                ),
                rewritten: Some(RewrittenDeletionTreeNode {
                    bytes,
                    birth_sequence,
                }),
            })
        }
    };
    // logged_ops 根（判决 G1）：码 2 层级 0，条目 44 字节 = key 10 + value 34，key 区间是首末两条的 key。
    let logged_operations_tree = checkpoint.trees.logged_operations;
    let logged_operations_root = match resolved.logged_operations_root {
        LoggedOperationsRootOrigin::NoRoot => None,
        LoggedOperationsRootOrigin::CarriedFromThePreviousVersion => Some(DeletionTreeNodeUnit {
            pointer: carried_previous().tree_root_pointer(logged_operations_tree.0),
            rewritten: None,
        }),
        LoggedOperationsRootOrigin::RewrittenThisPublish => {
            let birth_sequence = sequences.next(logged_operations_tree, txg, instance);
            let entries = logged_operations_root_entries(&resolved.logged_operations);
            let key_of = |entry: &Vec<u8>| entry[..LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES].to_vec();
            let smallest_key = key_of(entries.first().expect("有条目才有根"));
            let largest_key = key_of(entries.last().expect("有条目才有根"));
            let bytes = build_index_node(
                logged_operations_tree,
                0,
                LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
                &smallest_key,
                &largest_key,
                txg,
                checkpoint.filesystem_identifier,
                instance,
                birth_sequence,
                u16::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44"),
                &entries,
            );
            Some(DeletionTreeNodeUnit {
                pointer: pointer_of(
                    logged_operations_tree,
                    TransactionUnit::LoggedOperationsRoot,
                    &bytes,
                    birth_sequence,
                ),
                rewritten: Some(RewrittenDeletionTreeNode {
                    bytes,
                    birth_sequence,
                }),
            })
        }
    };
    let carried_tombstone_container_pointer = |position: usize| {
        carried_previous()
            .tombstone_containers
            .get(position)
            .expect("没重写的墓碑容器一定在上一版里（新开的那一片恒在重写清单里）")
            .pointer
    };
    let carried_tombstone_node_pointer = |position: ContainerTreeNodePosition| {
        carried_previous()
            .tombstone_tree_index_nodes
            .pointer_below_the_root(position)
            .expect("照抄的墓碑树根之下节点一定在上一版里（新开的节点恒在最右那一条路径上、重写）")
    };
    let carried_tombstone_root_pointer =
        || carried_previous().tree_root_pointer(checkpoint.trees.tombstone.0);
    let tombstone = build_container_tree_units(
        &ContainerTreeToBuild {
            tree: checkpoint.trees.tombstone,
            after_this_publish: &resolved.tombstone_tree,
            record_width_in_bytes: TOMBSTONE_RECORD_BYTES,
            record_bytes: TombstoneRecord::to_bytes,
            index_of_position: TombstoneContainerIndexInTree::of_position,
            roles: ContainerTreeRoles::OF_THE_TOMBSTONE_TREE,
            carried_container_pointer: &carried_tombstone_container_pointer,
            carried_node_below_the_root_pointer: &carried_tombstone_node_pointer,
            carried_root_pointer: &carried_tombstone_root_pointer,
        },
        checkpoint,
        slots,
        sequences,
    );
    let carried_lineage_container_pointer = |position: usize| {
        carried_previous()
            .lineage_containers
            .get(position)
            .expect("没重写的谱系容器一定在上一版里（新开的那一片恒在重写清单里）")
            .pointer
    };
    let carried_lineage_node_pointer = |position: ContainerTreeNodePosition| {
        carried_previous()
            .lineage_tree_index_nodes
            .pointer_below_the_root(position)
            .expect("照抄的谱系树根之下节点一定在上一版里（新开的节点恒在最右那一条路径上、重写）")
    };
    let carried_lineage_root_pointer =
        || carried_previous().tree_root_pointer(checkpoint.trees.lineage.0);
    let lineage = build_container_tree_units(
        &ContainerTreeToBuild {
            tree: checkpoint.trees.lineage,
            after_this_publish: &resolved.lineage_tree,
            record_width_in_bytes: LINEAGE_RECORD_BYTES,
            record_bytes: LineageRecord::to_bytes,
            index_of_position: LineageContainerIndexInTree::of_position,
            roles: ContainerTreeRoles::OF_THE_LINEAGE_TREE,
            carried_container_pointer: &carried_lineage_container_pointer,
            carried_node_below_the_root_pointer: &carried_lineage_node_pointer,
            carried_root_pointer: &carried_lineage_root_pointer,
        },
        checkpoint,
        slots,
        sequences,
    );
    let carried_retirement_container_pointer = |position: usize| {
        carried_previous()
            .retirement_containers
            .get(position)
            .expect("没重写的退役容器一定在上一版里（新开的那几片恒在重写清单里）")
            .pointer
    };
    let carried_retirement_node_pointer = |position: ContainerTreeNodePosition| {
        carried_previous()
            .retirement_tree_index_nodes
            .pointer_below_the_root(position)
            .expect(
                "照抄的退役树根之下节点一定在上一版里（新开的节点恒是这次重写的容器的祖先、重写）",
            )
    };
    let carried_retirement_root_pointer =
        || carried_previous().tree_root_pointer(checkpoint.trees.retirement.0);
    let retirement = build_container_tree_units(
        &ContainerTreeToBuild {
            tree: checkpoint.trees.retirement,
            after_this_publish: &resolved.retirement_tree,
            record_width_in_bytes: RETIREMENT_RECORD_BYTES,
            record_bytes: RetirementRecord::to_bytes,
            index_of_position: RetirementContainerIndexInTree::of_position,
            roles: ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
            carried_container_pointer: &carried_retirement_container_pointer,
            carried_node_below_the_root_pointer: &carried_retirement_node_pointer,
            carried_root_pointer: &carried_retirement_root_pointer,
        },
        checkpoint,
        slots,
        sequences,
    );
    DeletionTreeUnits {
        deleted_inodes_root,
        tombstone,
        lineage,
        logged_operations_root,
        retirement,
    }
}

/// 第一个文件之外那个文件这一步按读回的旧单元（或零）重写的一个数据单元（截断意图与变长的尾单元，判决 m4-multi-inode-size-r1 第二节 S3；
/// 部分写那一段里的一个，判决 m4-partial-write-r1 第二节 R6）：这次装出来的字节与它这一版的数据指针。
pub(super) struct RewrittenDataUnitOfAFileOtherThanTheFirst {
    pub(super) inode: InodeNumber,
    pub(super) unit: DataUnitIndexInFile,
    pub(super) bytes: Vec<u8>,
    pub(super) pointer: DataPointer,
}

/// `rewritten`（一步重写的那一段，单元号连号升序）里 `inode` 那个文件第 `unit` 个单元那一个；不在里面交回 `None`。按偏移取、不逐个找。
pub(super) fn rewritten_data_unit_of_a_file_other_than_the_first(
    rewritten: &[RewrittenDataUnitOfAFileOtherThanTheFirst],
    inode: InodeNumber,
    unit: DataUnitIndexInFile,
) -> Option<&RewrittenDataUnitOfAFileOtherThanTheFirst> {
    let first = rewritten.first()?;
    if first.inode != inode {
        return None;
    }
    let found = rewritten.get(usize::try_from(unit.0.checked_sub(first.unit.0)?).ok()?)?;
    assert_eq!(found.unit, unit, "一步重写的那一段单元号连号升序");
    Some(found)
}

/// 一步重写的那一段一单元事务（[`PublishPlan::file_content_transactions_and_the_rewritten_data_units`]，单元号连号升序）里第 `unit` 个单元那一个；
/// 不在里面交回 `None`。按偏移取、不逐个找（部分写从单元 0 起重写一个大文件时那一段与文件一样长）。
pub(super) fn transaction_of_a_rewritten_data_unit(
    transactions: &[OneUnitTransaction],
    unit: DataUnitIndexInFile,
) -> Option<OneUnitTransaction> {
    let first = transactions.first()?.unit_index_in_file.0;
    let transaction = *transactions.get(usize::try_from(unit.0.checked_sub(first)?).ok()?)?;
    assert_eq!(
        transaction.unit_index_in_file, unit,
        "一步重写的那一段单元号连号升序"
    );
    Some(transaction)
}

/// 一步重写的那一段里 `transaction` 那个单元的新净荷（入口读好、改好交进来的，[`RewrittenDataUnitPayload`]）。
///
/// # Panics
/// 交进来的新净荷里没有这个单元、或长度不是它这次的声明长度：入口与计划按同一步切出同一段，不等说明调用方给错了输入。
pub(super) fn payload_of_a_rewritten_data_unit(
    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
    transaction: OneUnitTransaction,
) -> &[u8] {
    let first = payloads_of_the_rewritten_data_units
        .first()
        .expect("这一步重写的单元在入口里都读好、改好了新净荷")
        .unit
        .0;
    let rewritten = payloads_of_the_rewritten_data_units
        .get(
            usize::try_from(
                transaction
                    .unit_index_in_file
                    .0
                    .checked_sub(first)
                    .expect("新净荷按单元号连号升序、从这一段的第一个起"),
            )
            .expect("单元序号"),
        )
        .expect("这一步重写的单元在入口里都读好、改好了新净荷");
    assert_eq!(
        rewritten.unit, transaction.unit_index_in_file,
        "新净荷按单元号连号升序"
    );
    assert_eq!(
        u64::try_from(rewritten.payload.len()).expect("净荷长度"),
        transaction.payload_length_in_bytes(),
        "新净荷的长度就是这个单元这次的声明长度"
    );
    &rewritten.payload
}

/// 读新槽残留（判决 m4-partial-write-r1 第二节 R2，用户 2026-10-26 定「首次写也读回」）：这一步要读残留的那几个新单元，在取定落点之后、
/// 任何写之前，在这次分到的槽、第一条位置项那块盘上各读一次整单元。读到的字节丢掉（新净荷在入口里已按零装好，残留一个字节都不进新单元：
/// 残留常是同一文件放掉的旧单元），读不出记进写入口的账（[`PoolWriter::residue_reads`]）、不拒写。
pub(super) fn read_the_residue_of_the_new_slots<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    data_units_rewritten_from_their_payloads: Option<DataUnitsRewrittenByTheStep>,
    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
) {
    let Some(rewritten) = data_units_rewritten_from_their_payloads else {
        return;
    };
    let reader: &[(DeviceIdentity, Device)] = &*pool.devices;
    let Some((device_of_the_first_location_entry, _)) = reader.first() else {
        return;
    };
    // 迭代次数的上界是这一步重写的单元数；每一轮只读一次、记一笔，没有提前出口。
    for payload in payloads_of_the_rewritten_data_units {
        match payload.residue_read {
            ResidueReadOfTheNewSlot::ReadOnceAndDiscardTheBytes => {
                let role = role_of_a_data_unit_of_the_inode(rewritten.inode, payload.unit);
                let slot = *slots
                    .get(&role)
                    .expect("读残留的单元是这次重写的角色，上面取过落点");
                let unit_bytes = usize::try_from(role.span_slots() * SLOT_BYTES)
                    .expect("一个单元两槽以内，字节数装得进 usize");
                let read_back = reader
                    .read(
                        *device_of_the_first_location_entry,
                        slot.to_device_offset(),
                        unit_bytes,
                    )
                    .is_some();
                pool.note_a_residue_read(read_back);
            }
            ResidueReadOfTheNewSlot::NotRead => {}
        }
    }
}

/// `inode` 那个文件第 `unit` 个数据单元的角色：第一个文件是 [`TransactionUnit::Data`]，别的是 [`TransactionUnit::DataOfAFileOtherThanTheFirst`]。
pub(super) fn role_of_a_data_unit_of_the_inode(
    inode: InodeNumber,
    unit: DataUnitIndexInFile,
) -> TransactionUnit {
    if inode.0 == FIRST_INODE_NUMBER {
        TransactionUnit::Data(unit)
    } else {
        TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit }
    }
}

pub(super) fn carried_file_version_units(
    carried: &TransactionOutput,
    carried_data_units: usize,
) -> FileVersionUnits {
    // 第一个文件的每个数据单元按角色取一次：先建上一版 `units` 的索引（七·6）。
    let units_of_the_carried_version = carried.units_by_role();
    FileVersionUnits {
        data_units: (0..carried_data_units)
            .map(|position| {
                units_of_the_carried_version
                    .unit(TransactionUnit::Data(DataUnitIndexInFile(
                        u64::try_from(position).expect("单元序号"),
                    )))
                    .bytes
                    .clone()
            })
            .collect(),
        data_pointers: carried.data_pointers[..carried_data_units].to_vec(),
        key_order_mismatches: 0,
    }
}

/// 照抄 `tree` 这棵树的节点那几次查记在哪一处：三棵树各记各的（七·6 后续批代码轮第一轮判决第三节第 3 条）。
fn lookup_site_of_the_carried_nodes_of(tree: MultiLevelCodeTwoTree) -> LookupSite {
    match tree {
        MultiLevelCodeTwoTree::Accounting => LookupSite::CarriedNodesOfTheAccountingTree,
        MultiLevelCodeTwoTree::CentralMapping => LookupSite::CarriedNodesOfTheCentralMappingTree,
        MultiLevelCodeTwoTree::Deadlist => LookupSite::CarriedNodesOfTheDeadlistTree,
    }
}

/// 装一棵多层码 2 树时整次发布共用的身份字段。
pub(super) struct MultiLevelTreeBuildContext<'build> {
    pub(super) txg: CheckpointTxg,
    pub(super) filesystem_identifier: &'build [u8; 16],
    pub(super) instance: InstanceGeneration,
    /// 位置条目按设备身份升序（I-2.5）。
    pub(super) device_identities: &'build [DeviceIdentity],
}

/// 一棵多层码 2 树这一版装好的样子。
pub(super) struct BuiltMultiLevelTree {
    pub(super) version: CodeTwoTreeVersion,
    /// 这一版的全部节点单元，按 bump 次序（先叶后根、同层按 key 升序），根在最末。
    pub(super) units: Vec<PublishedUnit>,
    /// 这次重写的节点各自的出生序号（点名项的 key 尾段要它）。
    pub(super) birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
}

impl BuiltMultiLevelTree {
    /// # Panics
    /// 这个角色这次没重写：点名项只点名这次重写的角色。
    pub(super) fn birth_sequence_of_a_rewritten_node(
        &self,
        identity: TransactionUnit,
    ) -> BirthSequence {
        *self
            .birth_sequences_of_rewritten_nodes
            .get(&identity)
            .expect("点名的是这次重写的节点")
    }
}

/// 按计划装一棵多层码 2 树这一版的全部节点（D8（核心索引结构） 已定项 11）：重写的节点按 bump 次序逐个装——先叶后根、
/// 同层按 key 升序（D3（空间分配） 已定项 10 ⑤）、出生序号按同一个次序发（D19（块指针的结构与宽度预算） 已定项 9）、
/// 落点取这次分配的；叶装这一版那几把 key 的完整条目，内部节点装「分隔 key + 孩子这一版的指针」（孩子先装好，指针才写得出），
/// 头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），层级取计划里的位置。照抄的节点字节、落点、指针全取
/// 上一版那个位置上的，角色换成这一版的位置。
///
/// # Panics
/// 计划里叶的 key 与 `leaf_entry_bytes_by_key` 的 key 不是同一个集合（计划按 `resolve` 现算的 key 算，条目按这次装出来的，
/// 两边出生序号的发号次序不同步就对不上；上一版叶里有重复 key 时条目那一边并掉一把，也对不上）；照抄的节点不在上一版里；
/// 重写的节点没拿到落点——三样都是发布路径自己的不变量。重复 key 那一格从盘上走不到，靠的是读者那一道：可写挂载接着发布的上一版由
/// `recovery::rebuild_version` 读记账树与中央映射树，按 `CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference` 判过
/// 每个节点的 key 严格递增、区间落在父条目里（实审 A3a 第 33 条；坏镜像用例在 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs`）；
/// 同一个进程里接着的上一版是发布路径自己装的，key 取自 `BTreeMap`。
#[allow(
    clippy::too_many_arguments,
    reason = "装一棵树要的九样：哪一棵、它的号、计划、上一版、上一版按角色的单元索引、条目、身份字段、落点、发号器，各自独立"
)]
pub(super) fn build_multi_level_tree(
    tree: MultiLevelCodeTwoTree,
    tree_identifier: TreeIdentifier,
    plan: &CodeTwoTreePlan,
    previous: Option<&TransactionOutput>,
    units_of_the_previous_version: Option<
        &FirstEntryOfEachIdentity<'_, TransactionUnit, PublishedUnit>,
    >,
    leaf_entry_bytes_by_key: &BTreeMap<CodeTwoTreeKey, Vec<u8>>,
    context: &MultiLevelTreeBuildContext<'_>,
    slots: &BTreeMap<TransactionUnit, SlotNumber>,
    sequences: &mut BirthSequenceAllocator,
) -> BuiltMultiLevelTree {
    // 装整棵树都在这棵树照抄节点那一处里（同 `build_extent_tree`）：每照抄一个节点现建一份索引的，建的那几份记在这一处。
    let _inside_the_tree_build = lookup_work::entering(lookup_site_of_the_carried_nodes_of(tree));
    let planned_keys: Vec<&CodeTwoTreeKey> = plan.shape.keys_in_order();
    assert!(
        planned_keys.len() == leaf_entry_bytes_by_key.len()
            && planned_keys
                .iter()
                .zip(leaf_entry_bytes_by_key.keys())
                .all(|(planned, built)| *planned == built),
        "{tree:?}：计划里叶的 key 与这次装出来的条目的 key 不是同一个集合（上一版叶里有重复 key 时才会分叉：盘上读来的上一版由 \
         recovery::rebuild_version 按 CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference 判过 key 严格递增）"
    );
    let key_width = tree.key_field_widths().key_width_in_bytes();
    let key_ranges = crate::code_two_tree::key_ranges_in_bump_order(&plan.shape);
    let mut pointers_by_position: BTreeMap<CodeTwoTreeNodePosition, NodePointer> = BTreeMap::new();
    let mut pointers: Vec<NodePointer> = Vec::with_capacity(plan.shape.nodes().len());
    let mut units: Vec<PublishedUnit> = Vec::with_capacity(plan.shape.nodes().len());
    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
    // 迭代上界是这一版的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
    for ((node, origin), (smallest_key, largest_key)) in
        plan.shape.nodes().iter().zip(&plan.origins).zip(key_ranges)
    {
        let identity = tree.role_of_node(node.position, &plan.shape);
        let (pointer, unit) = match origin {
            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
                let previous_version = previous.expect("照抄的节点只出现在接着上一版的发布里");
                let previous_tree = previous_version.multi_level_tree(tree);
                let _inside_the_carried_nodes =
                    lookup_work::entering(lookup_site_of_the_carried_nodes_of(tree));
                let carried = units_of_the_previous_version
                    .expect(
                        "照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来",
                    )
                    .unit(tree.role_of_node(*previous_position, &previous_tree.shape));
                (
                    previous_tree.pointer_of(*previous_position),
                    PublishedUnit {
                        slot: carried.slot,
                        identity,
                        bytes: carried.bytes.clone(),
                    },
                )
            }
            CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
                let (entry_width, entries): (usize, Vec<Vec<u8>>) = match &node.contents {
                    CodeTwoTreeNodeContents::Leaf { keys } => (
                        tree.leaf_entry_width_in_bytes(),
                        keys.iter()
                            .map(|key| leaf_entry_bytes_by_key[key].clone())
                            .collect(),
                    ),
                    CodeTwoTreeNodeContents::Internal { children } => (
                        internal_entry_width_in_bytes(key_width),
                        children
                            .iter()
                            .map(|child| {
                                build_internal_entry(
                                    child.separator_key.bytes(),
                                    pointers_by_position[&child.child],
                                )
                            })
                            .collect(),
                    ),
                };
                let birth_sequence = sequences.next(tree_identifier, context.txg, context.instance);
                let bytes = build_index_node(
                    tree_identifier,
                    node.position.level,
                    key_width,
                    &smallest_key,
                    &largest_key,
                    context.txg,
                    context.filesystem_identifier,
                    context.instance,
                    birth_sequence,
                    u16::try_from(entry_width).expect("条目宽 2 字节"),
                    &entries,
                );
                let slot = *slots
                    .get(&identity)
                    .expect("这次重写的节点都在角色清单里、取了落点");
                birth_sequences_of_rewritten_nodes.insert(identity, birth_sequence);
                (
                    NodePointer {
                        head: PointerHead {
                            birth_tree: tree_identifier,
                            birth_txg: context.txg,
                        },
                        locations: location_entries(context.device_identities, slot, &bytes),
                        instance: context.instance,
                        birth_sequence,
                    },
                    PublishedUnit {
                        slot,
                        identity,
                        bytes: Arc::from(bytes),
                    },
                )
            }
        };
        pointers_by_position.insert(node.position, pointer);
        pointers.push(pointer);
        units.push(unit);
    }
    BuiltMultiLevelTree {
        version: CodeTwoTreeVersion {
            shape: plan.shape.clone(),
            pointers,
        },
        units,
        birth_sequences_of_rewritten_nodes,
    }
}

#[cfg(test)]
mod tests;
