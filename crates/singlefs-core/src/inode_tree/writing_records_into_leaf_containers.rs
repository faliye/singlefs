//! 往叶容器里写记录：`write_records_into_leaf_containers`、写的进度与簿记的规格、它要的引理。

use super::*;

verus! {

/// 上一版就有的那几片身份与分隔 key 都没变（新开的只在它们右边）。
#[verifier::opaque]
pub open spec fn previous_leaves_keep_their_identities(before: Seq<InodeLeafContainer>, containers: Seq<InodeLeafContainer>) -> bool {
    &&& containers.len() >= before.len()
    &&& forall|position: int|
        0 <= position < before.len() ==> (#[trigger] containers[position]).identity == before[position].identity
            && containers[position].separator_key_in_the_parent_entry == before[position].separator_key_in_the_parent_entry
}

/// 上一版之外的那几片都是这次分裂出来的（[`leaf_container_is_split_off_by_this_publish`]）。
#[verifier::opaque]
pub open spec fn leaves_split_off_by_this_publish_are_named_after_their_first_record(
    before: Seq<InodeLeafContainer>,
    containers: Seq<InodeLeafContainer>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> bool {
    forall|position: int|
        before.len() <= position < containers.len() ==> leaf_container_is_split_off_by_this_publish(
            #[trigger] containers[position],
            checkpoint_txg,
            inode_tree,
        )
}

/// 重写清单按叶序严格升序、每一项都在片数以内。
#[verifier::opaque]
pub open spec fn rewritten_leaves_are_listed_in_order(rewritten: Seq<InodeLeafContainerIndexInTree>, container_count: int) -> bool {
    &&& indices_strictly_ascend(rewritten)
    &&& forall|listed: int| 0 <= listed < rewritten.len() ==> (#[trigger] rewritten[listed]).0 < container_count
}

/// 没进重写清单的那几片都是上一版那一片原样。
pub open spec fn leaves_not_rewritten_are_carried(
    before: Seq<InodeLeafContainer>,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
) -> bool {
    forall|position: int|
        #![trigger containers[position]]
        #![trigger indices_hold(rewritten, position)]
        0 <= position < containers.len() && !indices_hold(rewritten, position) ==> position < before.len()
            && leaf_containers_are_the_same(containers[position], before[position])
}

/// 写到第几条为止，叶容器与重写清单对上一版、已写的记录逐条成立的那几样（[`write_records_into_leaf_containers`] 的循环不变量与 `ensures`）：
/// 叶容器良好；记录连起来就是上一版的照已写的改写；上一版那几片身份不变、新开的照分裂纪律命名；重写清单升序；没进清单的原样。
pub open spec fn writing_progress(
    containers_before_this_publish: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
) -> bool {
    let before = containers_before_this_publish;
    &&& leaf_containers_are_well_formed(containers)
    &&& leaf_records(containers) == records_after_writing(leaf_records(before), written_so_far)
    &&& containers.len() >= before.len()
    &&& previous_leaves_keep_their_identities(before, containers)
    &&& leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers, checkpoint_txg, inode_tree)
    &&& rewritten_leaves_are_listed_in_order(rewritten, containers.len() as int)
    &&& leaves_not_rewritten_are_carried(before, containers, rewritten)
}

/// 这一片是这次分裂（或树空时新建）出来的：身份照参数、容器号与分隔 key 都是它第一条记录的 inode 号。
pub open spec fn leaf_container_is_split_off_by_this_publish(
    container: InodeLeafContainer,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> bool {
    &&& container.records@.len() >= 1
    &&& container.identity == (PackedIdentity {
        birth_tree: inode_tree,
        record_type: PACKED_TYPE_INODE,
        container: container.records@[0].inode,
        container_birth: checkpoint_txg,
    })
    &&& container.separator_key_in_the_parent_entry == container.records@[0].inode
}

/// 把一组 inode 记录写进 inode 树的叶容器，返回这次发布之后的全部容器与这次要重写的那几片。
///
/// 一条记录的落法只有两种：
/// - 它的 inode 号已经在树里 ⇒ **原地换掉那一条**（覆盖写改 size / 改动计数走这一支），记录数不变、容器身份不变，
///   只有那一片进重写清单；
/// - 它的 inode 号大于树里每一个 key ⇒ 追加到最右那片叶；最右那片已经装满 233 条就**在末尾分裂**：
///   左半（原来那片）一条记录都不动、身份不动、**不进重写清单**，右半是一片新容器，容器号 = 这条新记录的 inode 号、
///   出生代 = 这次发布的 checkpoint_txg、出生树 = `inode_tree`。
///
/// 树是空的（新池新建文件那一档）时第一条记录建第一片容器，容器号 = 它自己的 inode 号。
///
/// # Errors
/// [`InodeTreeWriteRefusal`] 的三格：中间插入、容器数超过一个根装得下的、给的记录不按 inode 号严格升序。
pub fn write_records_into_leaf_containers(
    containers_before_this_publish: &[InodeLeafContainer],
    records_to_write: &[InodeRecord],
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> (after: Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal>)
    requires
        leaf_containers_are_well_formed(containers_before_this_publish@),
        containers_before_this_publish.len() + records_to_write.len() <= u32::MAX,
    ensures
        match after {
            Ok(after) => {
                &&& inode_records_strictly_ascend(records_to_write@)
                &&& after.containers.len() <= 135
                &&& writing_progress(
                    containers_before_this_publish@,
                    records_to_write@,
                    checkpoint_txg,
                    inode_tree,
                    after.containers@,
                    after.rewritten@,
                )
                &&& after.origins.len() == after.containers.len()
                &&& forall|position: int|
                    0 <= position < after.origins.len() ==> #[trigger] after.origins@[position] == if indices_hold(
                        after.rewritten@,
                        position,
                    ) {
                        InodeLeafContainerOrigin::RewrittenThisPublish
                    } else {
                        InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree(position as u32))
                    }
                &&& after.retired.len() == 0
            },
            Err(InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder { earlier, later }) => !inode_records_strictly_ascend(
                records_to_write@,
            ) && earlier.0 >= later.0,
            Err(InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf { inode, largest_inode_number_in_the_tree }) => inode.0
                <= largest_inode_number_in_the_tree.0,
            Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers, capacity }) => capacity == 135
                && containers > capacity,
            Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode: _ }) => false,
        },
{
    let ghost before = containers_before_this_publish@;
    let mut later: usize = 1;
    while later < records_to_write.len()
        invariant
            1 <= later,
            forall|checked: int|
                1 <= checked < later && checked < records_to_write.len() ==> (#[trigger] records_to_write@[checked - 1]).inode
                    < records_to_write@[checked].inode,
        decreases records_to_write.len() - later,
    {
        if records_to_write[later - 1].inode >= records_to_write[later].inode {
            return Err(
                InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                    earlier: InodeNumber(records_to_write[later - 1].inode),
                    later: InodeNumber(records_to_write[later].inode),
                },
            );
        }
        later += 1;
    }
    proof {
        lemma_adjacent_ascending_records_ascend(records_to_write@);
    }
    let mut containers = copy_of_the_leaf_containers(containers_before_this_publish);
    // 重写清单按叶序升序、同一片只进一次：同一次发布里改同一片叶两次（改一条记录、再往它里面追加一条）只重写一遍。
    let mut rewritten: Vec<InodeLeafContainerIndexInTree> = Vec::new();
    proof {
        assert(records_to_write@.take(0) =~= Seq::<InodeRecord>::empty());
        reveal(indices_hold);
        reveal(previous_leaves_keep_their_identities);
        reveal(leaves_split_off_by_this_publish_are_named_after_their_first_record);
        reveal(rewritten_leaves_are_listed_in_order);
                assert forall|position: int| 0 <= position < containers.len() implies 1 <= (#[trigger] containers@[position]).records@.len()
            <= 233 by {
            assert(leaf_containers_are_the_same(containers@[position], before[position]));
        }
        assert forall|position: int| 0 <= position < before.len() implies (#[trigger] containers@[position]).identity
            == before[position].identity && containers@[position].separator_key_in_the_parent_entry
            == before[position].separator_key_in_the_parent_entry by {
            assert(leaf_containers_are_the_same(containers@[position], before[position]));
        }
        assert forall|position: int| 0 <= position < containers.len() && !indices_hold(rewritten@, position) implies position
            < before.len() && leaf_containers_are_the_same(#[trigger] containers@[position], before[position]) by {}
        assert(indices_strictly_ascend(rewritten@));
    }
    let mut record_position: usize = 0;
    while record_position < records_to_write.len()
        invariant
            before == containers_before_this_publish@,
            containers_before_this_publish.len() + records_to_write.len() <= u32::MAX,
            record_position <= records_to_write.len(),
            containers.len() <= before.len() + record_position,
            writing_progress(
                before,
                records_to_write@.take(record_position as int),
                checkpoint_txg,
                inode_tree,
                containers@,
                rewritten@,
            ),
        decreases records_to_write.len() - record_position,
    {
        let record = records_to_write[record_position];
        let inode = InodeNumber(record.inode);
        let ghost containers_at_the_start = containers@;
        let ghost written_so_far = records_to_write@.take(record_position as int);
        proof {
            reveal(rewritten_leaves_are_listed_in_order);
            assert forall|position: int| 0 <= position < containers.len() implies inode_records_strictly_ascend(
                #[trigger] containers@[position].records@,
            ) by {
                lemma_each_container_of_well_formed_leaves_ascends(containers@, position);
            }
            assert(records_to_write@.take(record_position as int + 1) =~= written_so_far.push(record));
        }
        let existing = position_of_the_container_holding(&containers, inode);
        match existing {
            Some(index) => {
                let holding_position = index.position();
                let position_in_container = search_records_by_inode(&containers[holding_position].records, inode.0)
                    .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
                let ghost rewritten_at_the_start = rewritten@;
                let container = &mut containers[holding_position];
                container.records[position_in_container] = record;
                note_rewritten(index, &mut rewritten);
                proof {
                    lemma_overwriting_a_record_keeps_the_writing_progress(
                        before,
                        written_so_far,
                        checkpoint_txg,
                        inode_tree,
                        containers_at_the_start,
                        rewritten_at_the_start,
                        holding_position as int,
                        position_in_container as int,
                        record,
                        containers@,
                        rewritten@,
                    );
                }
            },
            None => {
                proof {
                    lemma_an_inode_absent_from_every_container_is_absent_from_the_leaf_records(containers_at_the_start, inode.0);
                }
                let largest = largest_inode_number_in_the_tree(&containers);
                if let Some(largest_inode_number_in_the_tree) = largest {
                    if inode.0 <= largest_inode_number_in_the_tree.0 {
                        return Err(
                            InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {
                                inode,
                                largest_inode_number_in_the_tree,
                            },
                        );
                    }
                }
                let rightmost = rightmost_container_with_room(&containers);
                match rightmost {
                    Some(index) => {
                        let ghost rewritten_at_the_start = rewritten@;
                        proof {
                            lemma_the_last_leaf_record(containers_at_the_start);
                        }
                        let container = &mut containers[index.position()];
                        container.records.push(record);
                        note_rewritten(index, &mut rewritten);
                        proof {
                            lemma_appending_to_the_rightmost_leaf_keeps_the_writing_progress(
                                before,
                                written_so_far,
                                checkpoint_txg,
                                inode_tree,
                                containers_at_the_start,
                                rewritten_at_the_start,
                                record,
                                containers@,
                                rewritten@,
                            );
                        }
                    },
                    None => {
                        // 末尾分裂：左半（原来那片满的叶）一条记录、一段身份都不动，右半从这条新记录起。
                        let records = vec![record];
                        let split_off = InodeLeafContainer {
                            identity: PackedIdentity {
                                birth_tree: inode_tree,
                                record_type: PACKED_TYPE_INODE,
                                container: inode.0,
                                container_birth: checkpoint_txg,
                            },
                            separator_key_in_the_parent_entry: inode.0,
                            records,
                        };
                        let ghost rewritten_at_the_start = rewritten@;
                        let ghost split_off_view = split_off;
                        proof {
                            assert(split_off.records@ =~= seq![record]);
                            if containers_at_the_start.len() >= 1 {
                                lemma_the_last_leaf_record(containers_at_the_start);
                            }
                        }
                        containers.push(split_off);
                        note_rewritten(InodeLeafContainerIndexInTree::of_position(containers.len() - 1), &mut rewritten);
                        proof {
                            lemma_splitting_off_a_leaf_keeps_the_writing_progress(
                                before,
                                written_so_far,
                                checkpoint_txg,
                                inode_tree,
                                containers_at_the_start,
                                rewritten_at_the_start,
                                record,
                                split_off_view,
                                rewritten@,
                            );
                        }
                    },
                }
            },
        }
        record_position += 1;
    }
    proof {
        assert(records_to_write@.take(records_to_write.len() as int) =~= records_to_write@);
    }
    let capacity = leaf_containers_one_root_node_holds();
    if containers.len() > capacity {
        return Err(
            InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers: containers.len(), capacity },
        );
    }
    let mut origins: Vec<InodeLeafContainerOrigin> = Vec::new();
    let mut position: usize = 0;
    while position < containers.len()
        invariant
            position <= containers.len() <= 135,
            origins.len() == position,
            forall|done: int|
                0 <= done < position ==> #[trigger] origins@[done] == if indices_hold(rewritten@, done) {
                    InodeLeafContainerOrigin::RewrittenThisPublish
                } else {
                    InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree(done as u32))
                },
        decreases containers.len() - position,
    {
        let index = InodeLeafContainerIndexInTree::of_position(position);
        if index_is_noted(&rewritten, index) {
            origins.push(InodeLeafContainerOrigin::RewrittenThisPublish);
        } else {
            origins.push(InodeLeafContainerOrigin::CarriedFrom(index));
        }
        position += 1;
    }
    Ok(InodeLeafContainersAfterThisPublish { containers, rewritten, origins, retired: Vec::new() })
}

/// 原地改了第 `changed_position` 片（身份、分隔 key、第一条的号都不变）、把它记进重写清单之后，四样账目照样成立。
proof fn lemma_changing_one_leaf_in_place_keeps_the_bookkeeping(
    before: Seq<InodeLeafContainer>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    changed_position: int,
    containers_after: Seq<InodeLeafContainer>,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        previous_leaves_keep_their_identities(before, containers),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers, checkpoint_txg, inode_tree),
        rewritten_leaves_are_listed_in_order(rewritten, containers.len() as int),
        leaves_not_rewritten_are_carried(before, containers, rewritten),
        0 <= changed_position < containers.len(),
        containers_after.len() == containers.len(),
        forall|position: int|
            0 <= position < containers.len() && position != changed_position ==> #[trigger] containers_after[position]
                == containers[position],
        containers_after[changed_position].identity == containers[changed_position].identity,
        containers_after[changed_position].separator_key_in_the_parent_entry
            == containers[changed_position].separator_key_in_the_parent_entry,
        containers[changed_position].records@.len() >= 1,
        containers_after[changed_position].records@.len() >= 1,
        containers_after[changed_position].records@[0].inode == containers[changed_position].records@[0].inode,
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == changed_position),
    ensures
        previous_leaves_keep_their_identities(before, containers_after),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers_after, checkpoint_txg, inode_tree),
        rewritten_leaves_are_listed_in_order(rewritten_after, containers_after.len() as int),
        leaves_not_rewritten_are_carried(before, containers_after, rewritten_after),
{
    reveal(previous_leaves_keep_their_identities);
    reveal(leaves_split_off_by_this_publish_are_named_after_their_first_record);
    reveal(rewritten_leaves_are_listed_in_order);
        assert forall|position: int| 0 <= position < before.len() implies (#[trigger] containers_after[position]).identity
        == before[position].identity && containers_after[position].separator_key_in_the_parent_entry
        == before[position].separator_key_in_the_parent_entry by {
        assert(containers[position].identity == before[position].identity);
    }
    assert forall|position: int| before.len() <= position < containers_after.len() implies leaf_container_is_split_off_by_this_publish(
        #[trigger] containers_after[position],
        checkpoint_txg,
        inode_tree,
    ) by {
        assert(leaf_container_is_split_off_by_this_publish(containers[position], checkpoint_txg, inode_tree));
    }
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies (#[trigger] rewritten_after[listed]).0
        < containers_after.len() by {
        reveal(indices_hold);
        let position = rewritten_after[listed].0 as int;
        assert(indices_hold(rewritten_after, position));
        if indices_hold(rewritten, position) {
            let earlier = choose|earlier: int| 0 <= earlier < rewritten.len() && (#[trigger] rewritten[earlier]).0 == position;
        }
    }
    assert forall|position: int|
        #![trigger containers_after[position]]
        #![trigger indices_hold(rewritten_after, position)]
        0 <= position < containers_after.len() && !indices_hold(rewritten_after, position) implies position < before.len()
            && leaf_containers_are_the_same(containers_after[position], before[position]) by {
        assert(!indices_hold(rewritten, position));
        assert(position != changed_position);
        assert(containers_after[position] == containers[position]);
        assert(0 <= position < containers.len() && !indices_hold(rewritten, position));
        assert(leaf_containers_are_the_same(containers[position], before[position]));
        assert(position < before.len());
    }
}

/// 在末尾添一片照分裂纪律命名的新容器、把它记进重写清单之后，四样账目照样成立。
proof fn lemma_adding_a_split_off_leaf_keeps_the_bookkeeping(
    before: Seq<InodeLeafContainer>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    split_off: InodeLeafContainer,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        previous_leaves_keep_their_identities(before, containers),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers, checkpoint_txg, inode_tree),
        rewritten_leaves_are_listed_in_order(rewritten, containers.len() as int),
        leaves_not_rewritten_are_carried(before, containers, rewritten),
        leaf_container_is_split_off_by_this_publish(split_off, checkpoint_txg, inode_tree),
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == containers.len()),
    ensures
        previous_leaves_keep_their_identities(before, containers.push(split_off)),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(
            before,
            containers.push(split_off),
            checkpoint_txg,
            inode_tree,
        ),
        rewritten_leaves_are_listed_in_order(rewritten_after, containers.len() as int + 1),
        leaves_not_rewritten_are_carried(before, containers.push(split_off), rewritten_after),
{
    reveal(previous_leaves_keep_their_identities);
    reveal(leaves_split_off_by_this_publish_are_named_after_their_first_record);
    reveal(rewritten_leaves_are_listed_in_order);
        let containers_after = containers.push(split_off);
    assert forall|position: int| 0 <= position < before.len() implies (#[trigger] containers_after[position]).identity
        == before[position].identity && containers_after[position].separator_key_in_the_parent_entry
        == before[position].separator_key_in_the_parent_entry by {
        assert(containers_after[position] == containers[position]);
    }
    assert forall|position: int| before.len() <= position < containers_after.len() implies leaf_container_is_split_off_by_this_publish(
        #[trigger] containers_after[position],
        checkpoint_txg,
        inode_tree,
    ) by {
        if position < containers.len() {
            assert(containers_after[position] == containers[position]);
        }
    }
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies (#[trigger] rewritten_after[listed]).0
        < containers_after.len() by {
        reveal(indices_hold);
        let position = rewritten_after[listed].0 as int;
        assert(indices_hold(rewritten_after, position));
        if indices_hold(rewritten, position) {
            let earlier = choose|earlier: int| 0 <= earlier < rewritten.len() && (#[trigger] rewritten[earlier]).0 == position;
        }
    }
    assert forall|position: int|
        #![trigger containers_after[position]]
        #![trigger indices_hold(rewritten_after, position)]
        0 <= position < containers_after.len() && !indices_hold(rewritten_after, position) implies position < before.len()
            && leaf_containers_are_the_same(containers_after[position], before[position]) by {
        assert(!indices_hold(rewritten, position));
        assert(position != containers.len());
        assert(containers_after[position] == containers[position]);
        assert(0 <= position < containers.len() && !indices_hold(rewritten, position));
        assert(leaf_containers_are_the_same(containers[position], before[position]));
        assert(position < before.len());
    }
}

/// 原地换掉第 `holding_position` 片的第 `position_in_container` 条（同号）、把这一片记进重写清单之后，[`writing_progress`] 照样成立。
proof fn lemma_overwriting_a_record_keeps_the_writing_progress(
    before: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    holding_position: int,
    position_in_container: int,
    record: InodeRecord,
    containers_after: Seq<InodeLeafContainer>,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        writing_progress(before, written_so_far, checkpoint_txg, inode_tree, containers, rewritten),
        0 <= holding_position < containers.len(),
        0 <= position_in_container < containers[holding_position].records@.len(),
        containers[holding_position].records@[position_in_container].inode == record.inode,
        containers_after.len() == containers.len(),
        forall|position: int|
            0 <= position < containers.len() && position != holding_position ==> #[trigger] containers_after[position]
                == containers[position],
        containers_after[holding_position].identity == containers[holding_position].identity,
        containers_after[holding_position].separator_key_in_the_parent_entry
            == containers[holding_position].separator_key_in_the_parent_entry,
        containers_after[holding_position].records@ == containers[holding_position].records@.update(position_in_container, record),
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == holding_position),
    ensures
        writing_progress(before, written_so_far.push(record), checkpoint_txg, inode_tree, containers_after, rewritten_after),
{
    let flattened = leaf_records(containers);
    let start = records_before_container(containers, holding_position);
    let global = start + position_in_container;
    assert(containers_after =~= containers.update(holding_position, containers_after[holding_position]));
    lemma_leaf_records_around_one_container(containers, holding_position, containers_after[holding_position]);
    lemma_a_record_inside_the_leaf_records(containers, holding_position, position_in_container);
    assert(leaf_records(containers_after) =~= flattened.update(global, record));
    lemma_first_position_of_an_inode_in_ascending_records(flattened, record.inode, global, 0);
    assert(written_so_far.push(record).drop_last() =~= written_so_far);
    lemma_updating_a_record_with_the_same_inode_keeps_the_records_ascending(flattened, global, record);
    assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (#[trigger] containers_after[position]).records@.len()
        <= 233 by {
        assert(1 <= containers[position].records@.len() <= 233);
    }
    assert(containers[holding_position].records@.len() >= 1);
    assert(containers_after[holding_position].records@[0].inode == containers[holding_position].records@[0].inode);
    lemma_changing_one_leaf_in_place_keeps_the_bookkeeping(
        before,
        checkpoint_txg,
        inode_tree,
        containers,
        rewritten,
        holding_position,
        containers_after,
        rewritten_after,
    );
}

/// 往最右那片（还装得下）末尾追加一条比树里都大的记录、把这一片记进重写清单之后，[`writing_progress`] 照样成立。
proof fn lemma_appending_to_the_rightmost_leaf_keeps_the_writing_progress(
    before: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    record: InodeRecord,
    containers_after: Seq<InodeLeafContainer>,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        writing_progress(before, written_so_far, checkpoint_txg, inode_tree, containers, rewritten),
        containers.len() >= 1,
        containers.last().records@.len() < 233,
        !records_hold_inode(leaf_records(containers), record.inode),
        containers.last().records@.last().inode < record.inode,
        containers_after.len() == containers.len(),
        forall|position: int| 0 <= position < containers.len() - 1 ==> #[trigger] containers_after[position] == containers[position],
        containers_after.last().identity == containers.last().identity,
        containers_after.last().separator_key_in_the_parent_entry == containers.last().separator_key_in_the_parent_entry,
        containers_after.last().records@ == containers.last().records@.push(record),
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == containers.len()
                - 1),
    ensures
        writing_progress(before, written_so_far.push(record), checkpoint_txg, inode_tree, containers_after, rewritten_after),
{
    let last = containers.len() - 1;
    let flattened = leaf_records(containers);
    assert(containers_after =~= containers.update(last, containers_after[last]));
    lemma_leaf_records_around_one_container(containers, last, containers_after[last]);
    assert(containers.subrange(last + 1, containers.len() as int) =~= Seq::<InodeLeafContainer>::empty());
    assert(leaf_records(containers_after) =~= flattened.push(record));
    lemma_first_position_of_an_absent_inode(flattened, record.inode, 0);
    assert(written_so_far.push(record).drop_last() =~= written_so_far);
    lemma_the_last_leaf_record(containers);
    lemma_pushing_a_larger_record_keeps_the_records_ascending(flattened, record);
    assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (#[trigger] containers_after[position]).records@.len()
        <= 233 by {
        assert(1 <= containers[position].records@.len() <= 233);
    }
    assert(containers[last].records@.len() >= 1);
    assert(containers_after[last].records@[0] == containers[last].records@[0]);
    lemma_changing_one_leaf_in_place_keeps_the_bookkeeping(
        before,
        checkpoint_txg,
        inode_tree,
        containers,
        rewritten,
        last,
        containers_after,
        rewritten_after,
    );
}

/// 最右那片满了（或树是空的）、在末尾分裂出一片只装这一条的新容器、把它记进重写清单之后，[`writing_progress`] 照样成立。
proof fn lemma_splitting_off_a_leaf_keeps_the_writing_progress(
    before: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    record: InodeRecord,
    split_off: InodeLeafContainer,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        writing_progress(before, written_so_far, checkpoint_txg, inode_tree, containers, rewritten),
        !records_hold_inode(leaf_records(containers), record.inode),
        containers.len() >= 1 ==> containers.last().records@.last().inode < record.inode,
        split_off.records@ == seq![record],
        split_off.identity == (PackedIdentity {
            birth_tree: inode_tree,
            record_type: PACKED_TYPE_INODE,
            container: record.inode,
            container_birth: checkpoint_txg,
        }),
        split_off.separator_key_in_the_parent_entry == record.inode,
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == containers.len()),
    ensures
        writing_progress(before, written_so_far.push(record), checkpoint_txg, inode_tree, containers.push(split_off), rewritten_after),
{
    let containers_after = containers.push(split_off);
    let flattened = leaf_records(containers);
    assert(containers_after.drop_last() =~= containers);
    assert(leaf_records(containers_after) =~= flattened.push(record));
    lemma_first_position_of_an_absent_inode(flattened, record.inode, 0);
    assert(written_so_far.push(record).drop_last() =~= written_so_far);
    if containers.len() >= 1 {
        lemma_the_last_leaf_record(containers);
    }
    lemma_pushing_a_larger_record_keeps_the_records_ascending(flattened, record);
    assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (#[trigger] containers_after[position]).records@.len()
        <= 233 by {
        if position < containers.len() {
            assert(1 <= containers[position].records@.len() <= 233);
        }
    }
    assert(leaf_container_is_split_off_by_this_publish(split_off, checkpoint_txg, inode_tree));
    lemma_adding_a_split_off_leaf_keeps_the_bookkeeping(
        before,
        checkpoint_txg,
        inode_tree,
        containers,
        rewritten,
        split_off,
        rewritten_after,
    );
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inode_tree::tests::{live_inode_record, INODE_TREE, TREE_BIRTH_TXG};

    /// 新池新建文件那一档：树是空的，第一条记录建第一片容器，容器号 = 它自己的 inode 号、出生代 = 这次发布的 txg。
    #[test]
    fn the_first_record_creates_the_first_container_numbered_after_itself() {
        let after = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 3000, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("建第一片容器");
        assert_eq!(after.containers.len(), 1);
        assert_eq!(
            after.containers[0].identity,
            PackedIdentity {
                birth_tree: INODE_TREE,
                record_type: PACKED_TYPE_INODE,
                container: 1,
                container_birth: TREE_BIRTH_TXG,
            },
            "容器号 = 第一条记录的 inode 号（D8 已定项 6）"
        );
        assert_eq!(
            after.containers[0].records,
            vec![live_inode_record(1, 3000, 3)]
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
    }

    /// 覆盖写那一档：号已经在树里 ⇒ 原地换掉那一条，记录数不变、身份不变，只有那一片进重写清单。
    #[test]
    fn writing_a_number_already_in_the_tree_replaces_that_record_and_keeps_the_container_identity()
    {
        let before = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 3000, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("建第一片容器")
        .containers;
        let after = write_records_into_leaf_containers(
            &before,
            &[live_inode_record(1, 4096, 4)],
            CheckpointTxg(4),
            INODE_TREE,
        )
        .expect("覆盖写");
        assert_eq!(after.containers.len(), 1);
        assert_eq!(
            after.containers[0].identity, before[0].identity,
            "COW 重写身份不变（D8 已定项 6）：出生代仍是建树那次的 3，不是这次发布的 4"
        );
        assert_eq!(
            after.containers[0].records,
            vec![live_inode_record(1, 4096, 4)]
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
    }

    /// 一容器 233 条：第 233 条还在第一片里，树只有一片叶。
    #[test]
    fn two_hundred_thirty_three_records_still_fit_one_container() {
        let records: Vec<InodeRecord> = (1..=233u64)
            .map(|inode| live_inode_record(inode, 0, 3))
            .collect();
        let after = write_records_into_leaf_containers(&[], &records, TREE_BIRTH_TXG, INODE_TREE)
            .expect("233 条");
        assert_eq!(INODE_LEAF_RECORDS, 233);
        assert_eq!(after.containers.len(), 1, "233 条装得下一片");
        assert_eq!(after.containers[0].records.len(), 233);
        assert_eq!(after.containers[0].free_record_slots(), 0);
    }

    /// 第 234 条触发末尾分裂：左半保留全部原有记录与身份、**一个字节都不重写**；
    /// 右半是新容器，容器号 = 触发分裂那条新记录的 inode 号、出生代 = 本次发布的 checkpoint_txg、出生树 = 执行分裂的那棵树。
    /// 这是 C116（叶容器的分裂 / 合并纪律没有会失败的检查） 那三条纪律的会红的量。
    #[test]
    fn the_two_hundred_thirty_fourth_record_splits_at_the_end_and_the_left_half_keeps_everything() {
        let before = write_records_into_leaf_containers(
            &[],
            &(1..=233u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("233 条")
        .containers;
        let split_txg = CheckpointTxg(9);
        let after = write_records_into_leaf_containers(
            &before,
            &[live_inode_record(234, 0, 9)],
            split_txg,
            INODE_TREE,
        )
        .expect("第 234 条");

        assert_eq!(after.containers.len(), 2, "分裂出第二片容器");
        assert_eq!(
            after.containers[0], before[0],
            "左半保留全部原有记录与身份：记录、容器号、出生代、出生树逐项不变"
        );
        assert_eq!(
            after.containers[0].records.len(),
            233,
            "分裂点取末尾：左半一条记录都不搬走"
        );
        assert_eq!(
            after.containers[1].identity,
            PackedIdentity {
                birth_tree: INODE_TREE,
                record_type: PACKED_TYPE_INODE,
                container: 234,
                container_birth: split_txg,
            },
            "右半容器号 = 触发分裂那条新记录的 inode 号，出生代 = 本次发布的 checkpoint_txg"
        );
        assert_eq!(
            after.containers[1].records,
            vec![live_inode_record(234, 0, 9)]
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)],
            "只重写右半：左半的字节没变，连 COW 都不做"
        );
    }

    /// 分裂之后两片容器的分隔 key 与 key 区间：I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 在树这一侧的形态。
    #[test]
    fn after_the_split_the_separator_keys_are_the_container_numbers_and_the_ranges_do_not_overlap()
    {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=234u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("234 条");
        let separators: Vec<u64> = after
            .containers
            .iter()
            .map(InodeLeafContainer::separator_key)
            .collect();
        assert_eq!(separators, vec![1, 234]);
        assert_eq!(
            after.containers[0].smallest_inode_number(),
            Some(InodeNumber(1))
        );
        assert_eq!(
            after.containers[0].largest_inode_number(),
            Some(InodeNumber(233))
        );
        assert_eq!(
            after.containers[1].smallest_inode_number(),
            Some(InodeNumber(234))
        );
        assert!(
            after.containers[0].largest_inode_number()
                < Some(after.containers[1].container_number()),
            "左容器的最大 key < 右容器号（I-9.4 那一句逼出的合并方向）"
        );
        for (container, separator) in after.containers.iter().zip(&separators) {
            assert!(
                container.smallest_inode_number() >= Some(InodeNumber(*separator)),
                "容器号不超它装的最小 key"
            );
        }
    }

    /// 一次写多条：新号接着最右叶追加，跨过 233 时中间那一片满着不动。
    #[test]
    fn appending_across_the_two_hundred_thirty_three_threshold_fills_left_to_right() {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=500u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("500 条");
        assert_eq!(after.containers.len(), 3, "500 = 233 + 233 + 34");
        assert_eq!(
            after
                .containers
                .iter()
                .map(|container| container.records.len())
                .collect::<Vec<_>>(),
            vec![233, 233, 34]
        );
        assert_eq!(
            after
                .containers
                .iter()
                .map(|container| container.identity.container)
                .collect::<Vec<_>>(),
            vec![1, 234, 467],
            "每片容器号 = 它诞生时装的第一条记录的 inode 号"
        );
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1),
                InodeLeafContainerIndexInTree::of_position(2),
            ],
            "这一次三片都动过：第一片从空装到满，后两片是新建的"
        );
    }

    /// 中间插入（号不在树里、又不比树里每一个 key 大）在任何人动盘之前被拒：非末尾分裂的纪律没写。
    #[test]
    fn inserting_a_number_that_would_not_land_in_the_rightmost_leaf_is_refused() {
        let before = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 0, 3), live_inode_record(5, 0, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("两条")
        .containers;
        let refusal = write_records_into_leaf_containers(
            &before,
            &[live_inode_record(3, 0, 4)],
            CheckpointTxg(4),
            INODE_TREE,
        )
        .expect_err("中间插入今天一律被拒");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {
                inode: InodeNumber(3),
                largest_inode_number_in_the_tree: InodeNumber(5),
            }
        );
    }

    /// 容器数超过一个码 2 根装得下的 135 片：树要长高，怎么长没条款 ⇒ 拒。
    #[test]
    fn more_containers_than_one_root_node_holds_is_refused() {
        assert_eq!(leaf_containers_one_root_node_holds(), 135, "扇出 135");
        let records: Vec<InodeRecord> = (1..=(233 * 135 + 1))
            .map(|inode| live_inode_record(inode, 0, 3))
            .collect();
        let refusal = write_records_into_leaf_containers(&[], &records, TREE_BIRTH_TXG, INODE_TREE)
            .expect_err("第 136 片容器今天装不下");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
                containers: 136,
                capacity: 135,
            }
        );
    }

    /// 同一次写里的记录不按 inode 号严格升序 ⇒ 拒（升序取号是 D16 已定项 5 末段的纪律，这里不替调用方排）。
    #[test]
    fn records_not_in_ascending_inode_order_are_refused() {
        let refusal = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(3, 0, 3), live_inode_record(2, 0, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect_err("降序被拒");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                earlier: InodeNumber(3),
                later: InodeNumber(2),
            }
        );
        assert_eq!(
            write_records_into_leaf_containers(
                &[],
                &[live_inode_record(3, 0, 3), live_inode_record(3, 1, 3)],
                TREE_BIRTH_TXG,
                INODE_TREE,
            )
            .expect_err("同号两条也被拒"),
            InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                earlier: InodeNumber(3),
                later: InodeNumber(3),
            }
        );
    }

    /// 一次写里既改旧记录又追加新记录：改的那一片与新建的那一片都进重写清单，各进一次。
    #[test]
    fn one_write_that_both_replaces_and_appends_lists_each_touched_container_once() {
        let before = write_records_into_leaf_containers(
            &[],
            &(1..=233u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("233 条")
        .containers;
        let after = write_records_into_leaf_containers(
            &before,
            &[
                live_inode_record(1, 4096, 9),
                live_inode_record(234, 0, 9),
                live_inode_record(235, 0, 9),
            ],
            CheckpointTxg(9),
            INODE_TREE,
        )
        .expect("改一条、追加两条");
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1),
            ]
        );
        assert_eq!(after.containers[1].records.len(), 2, "新容器装两条");
        assert_eq!(
            after.containers[1].identity.container, 234,
            "容器号是触发分裂那条（234），不是这次写的最后一条（235）"
        );
        assert_eq!(
            after.containers[0].records[0],
            live_inode_record(1, 4096, 9)
        );
    }
}
