//! 从叶容器里删一条记录：`remove_record_from_leaf_containers`、删之后的并与借、它们的规格与引理。

use super::*;

verus! {

// ===== 删除与合并 =====

/// 从 inode 树的叶容器里删掉 `inode` 那一条记录（unlink 最后一批那次发布：D8（核心索引结构） 已定项 16 第 2 条「删 inode 记录」），
/// 再照 E176 M3 那一臂合并（文件头那几条）。
///
/// - 删完没并、没借、没删空 ⇒ 只有那一片进重写清单，身份不变（COW 重写身份不变，D8 已定项 6）；
/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」），身份进 `retired`；左吸收右的那一对，左片进重写清单、右片身份进 `retired`；
///   左从右借的那一对两片都进重写清单、身份都不变，右片的分隔 key 改成它这一版的最小 key；
/// - 身份消失的那几片右边的每一片位置左移，内容没变的照抄（`origins` 记着上一版的叶序），不进重写清单。
///
/// 删的是树里最后一条时交回零片容器：inode 树写零条目根（key 区间 `[0, 0]`，D8 已定项 6），那一版是「有树、文件集合为空」
/// （D8 已定项 16 第 5 条），怎么装它归发布路径。
///
/// # Errors
/// 树里没有这条记录 ⇒ [`InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree`]（在任何落盘动作之前）。
pub fn remove_record_from_leaf_containers(
    containers_before_this_publish: &[InodeLeafContainer],
    inode: InodeNumber,
) -> (after: Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal>)
    requires
        leaf_containers_are_well_formed(containers_before_this_publish@),
        containers_before_this_publish.len() <= u32::MAX,
    ensures
        match after {
            Ok(after) => {
                &&& records_hold_inode(leaf_records(containers_before_this_publish@), inode.0)
                &&& leaf_records(after.containers@) == leaf_records(containers_before_this_publish@).remove(
                    first_position_of_inode(leaf_records(containers_before_this_publish@), inode.0, 0),
                )
                &&& leaf_containers_are_well_formed(after.containers@)
                &&& removal_keeps_track_of_the_previous_version(containers_before_this_publish@, after)
            },
            Err(refusal) => refusal == (InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode })
                && !records_hold_inode(leaf_records(containers_before_this_publish@), inode.0),
        },
{
    let ghost before = containers_before_this_publish@;
    proof {
        assert forall|position: int| 0 <= position < before.len() implies inode_records_strictly_ascend(
            #[trigger] before[position].records@,
        ) by {
            lemma_each_container_of_well_formed_leaves_ascends(before, position);
        }
    }
    let Some(index) = position_of_the_container_holding(containers_before_this_publish, inode)
    else {
        proof {
            lemma_an_inode_absent_from_every_container_is_absent_from_the_leaf_records(before, inode.0);
        }
        return Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode });
    };
    let mut tree = LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
    let holding_position = index.position();
    let ghost tree_at_the_start = tree.containers@;
    proof {
        assert(leaf_containers_are_the_same(tree_at_the_start[holding_position as int].contents, before[holding_position as int]));
    }
    let holding = &mut tree.containers[holding_position];
    let position_in_container = search_records_by_inode(&holding.contents.records, inode.0)
        .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
    holding.contents.records.remove(position_in_container);
    holding.is_changed_this_publish = true;
    let ghost records_after_removal = leaf_records(before).remove(first_position_of_inode(leaf_records(before), inode.0, 0));
    proof {
        let holding_after = tree.containers@[holding_position as int].contents;
        let removed = before.update(holding_position as int, holding_after);
        assert forall|position: int| 0 <= position < removed.len() implies #[trigger] leaf_containers_are_the_same(
            contents_being_merged(tree.containers@)[position],
            removed[position],
        ) by {
            assert(leaf_containers_are_the_same(tree_at_the_start[position].contents, before[position]));
            if position != holding_position {
                assert(tree.containers@[position] == tree_at_the_start[position]);
            }
        }
        lemma_the_same_leaf_containers_have_the_same_leaf_records(contents_being_merged(tree.containers@), removed);
        lemma_removing_a_record_keeps_the_rest_of_the_leaf_records(
            before,
            holding_position as int,
            position_in_container as int,
            holding_after,
        );
        lemma_a_record_inside_the_leaf_records(before, holding_position as int, position_in_container as int);
        lemma_first_position_of_an_inode_in_ascending_records(
            leaf_records(before),
            inode.0,
            records_before_container(before, holding_position as int) + position_in_container,
            0,
        );
        lemma_removing_a_record_keeps_the_records_ascending(
            leaf_records(before),
            records_before_container(before, holding_position as int) + position_in_container,
        );
        assert forall|position: int| 0 <= position < tree.containers@.len() implies {
            let container = #[trigger] tree.containers@[position];
            &&& container.previous_index.0 < before.len()
            &&& container.contents.identity == before[container.previous_index.0 as int].identity
            &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                container.contents,
                before[container.previous_index.0 as int],
            )
        } by {
            assert(leaf_containers_are_the_same(tree_at_the_start[position].contents, before[position]));
            if position != holding_position {
                assert(tree.containers@[position] == tree_at_the_start[position]);
            }
        }
        assert(records_hold_inode(leaf_records(before), inode.0)) by {
            assert(leaf_records(before)[records_before_container(before, holding_position as int) + position_in_container].inode
                == inode.0);
        }
    }
    if tree.containers[holding_position].contents.records.is_empty() {
        let ghost tree_before_retiring = tree.containers@;
        tree.retire(holding_position);
        proof {
            assert(contents_being_merged(tree.containers@) =~= contents_being_merged(tree_before_retiring).remove(
                holding_position as int,
            ));
            lemma_removing_an_empty_leaf_keeps_the_leaf_records(contents_being_merged(tree_before_retiring), holding_position as int);
            assert forall|position: int| 0 <= position < tree.containers@.len() implies 1 <= (
            #[trigger] tree.containers@[position]).contents.records@.len() <= 233 by {
                let in_the_old = if position < holding_position { position } else { position + 1 };
                assert(tree.containers@[position] == tree_before_retiring[in_the_old]);
                assert(tree_before_retiring[in_the_old] == tree_at_the_start[in_the_old]);
                assert(leaf_containers_are_the_same(tree_at_the_start[in_the_old].contents, before[in_the_old]));
            }
        }
        let has_new_neighbours = holding_position > 0 && holding_position < tree.containers.len();
        if has_new_neighbours {
            tree.absorb_until_no_pair_fits(holding_position - 1);
        }
    } else {
        proof {
            assert forall|position: int| 0 <= position < tree.containers@.len() implies 1 <= (
            #[trigger] tree.containers@[position]).contents.records@.len() <= 233 by {
                assert(leaf_containers_are_the_same(tree_at_the_start[position].contents, before[position]));
                if position != holding_position {
                    assert(tree.containers@[position] == tree_at_the_start[position]);
                }
            }
        }
        let absorbed_at_least_once = tree.absorb_until_no_pair_fits(holding_position);
        if !absorbed_at_least_once {
            tree.borrow_from_the_right_neighbour_if_below_the_floor(holding_position);
        }
    }
    let ghost tree_at_the_end = tree.containers@;
    let after = tree.after_this_publish();
    proof {
        assert forall|position: int| 0 <= position < after.containers@.len() implies 1 <= (#[trigger] after.containers@[position]).records@.len()
            <= 233 by {
            assert(after.containers@[position] == tree_at_the_end[position].contents);
        }
    }
    Ok(after)
}

/// [`remove_record_from_leaf_containers`] 交回的样子与上一版对得上的那几样：重写清单升序、与 `origins` 一一对应；照抄的那几片与上一版那一份
/// 逐字节相同；退役清单按上一版叶序不减、每一项是上一版那一片的身份。
pub open spec fn removal_keeps_track_of_the_previous_version(
    before: Seq<InodeLeafContainer>,
    after: InodeLeafContainersAfterThisPublish,
) -> bool {
    &&& after.origins.len() == after.containers.len()
    &&& indices_strictly_ascend(after.rewritten@)
    &&& forall|listed: int| 0 <= listed < after.rewritten.len() ==> (#[trigger] after.rewritten@[listed]).0 < after.containers.len()
    &&& forall|position: int|
        0 <= position < after.origins.len() ==> ((#[trigger] after.origins@[position]) is RewrittenThisPublish <==> indices_hold(
            after.rewritten@,
            position,
        ))
    &&& forall|position: int|
        0 <= position < after.origins.len() ==> match #[trigger] after.origins@[position] {
            InodeLeafContainerOrigin::CarriedFrom(previous_index) => previous_index.0 < before.len()
                && leaf_containers_are_the_same(after.containers@[position], before[previous_index.0 as int]),
            InodeLeafContainerOrigin::RewrittenThisPublish => true,
        }
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < after.retired.len() ==> (#[trigger] after.retired@[earlier]).previous_index.0
            <= (#[trigger] after.retired@[later]).previous_index.0
    &&& forall|listed: int|
        0 <= listed < after.retired.len() ==> (#[trigger] after.retired@[listed]).previous_index.0 < before.len()
            && after.retired@[listed].identity == before[after.retired@[listed].previous_index.0 as int].identity
}

/// 一次删除发布里合并到一半的叶容器：每片带着它在上一版里的叶序与这次有没有改过；身份消失的那几片挪进 `retired`。
struct LeafContainersBeingMerged {
    containers: Vec<LeafContainerBeingMerged>,
    retired: Vec<RetiredInodeLeafContainer>,
    /// 上一版的叶容器（只给 Verus 看：每片记得的上一版叶序指的就是它）。
    #[allow(dead_code, reason = "只给 Verus 看的幽灵字段，产品构建里不读它")]
    previous_version: Ghost<Seq<InodeLeafContainer>>,
}

struct LeafContainerBeingMerged {
    contents: InodeLeafContainer,
    previous_index: InodeLeafContainerIndexInTree,
    is_changed_this_publish: bool,
}

/// 合并到一半的叶容器的内容（规格里用）。
spec fn contents_being_merged(containers: Seq<LeafContainerBeingMerged>) -> Seq<InodeLeafContainer> {
    containers.map_values(|container: LeafContainerBeingMerged| container.contents)
}

/// 合并到一半时每片都记得它在上一版里是哪一片：叶序在上一版以内、身份就是那一片的，没改过的与那一片逐字节相同；退役清单按上一版叶序不减、
/// 每一项是上一版那一片的身份。
spec fn merging_keeps_track_of_the_previous_version(
    containers: Seq<LeafContainerBeingMerged>,
    retired: Seq<RetiredInodeLeafContainer>,
    before: Seq<InodeLeafContainer>,
) -> bool {
    &&& containers.len() <= before.len()
    &&& forall|position: int|
        0 <= position < containers.len() ==> {
            let container = #[trigger] containers[position];
            &&& container.previous_index.0 < before.len()
            &&& container.contents.identity == before[container.previous_index.0 as int].identity
            &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                container.contents,
                before[container.previous_index.0 as int],
            )
        }
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < retired.len() ==> (#[trigger] retired[earlier]).previous_index.0
            <= (#[trigger] retired[later]).previous_index.0
    &&& forall|listed: int|
        0 <= listed < retired.len() ==> (#[trigger] retired[listed]).previous_index.0 < before.len()
            && retired[listed].identity == before[retired[listed].previous_index.0 as int].identity
}

/// 合并到一半的叶容器的记录连起来（规格里用）。
spec fn records_being_merged(containers: Seq<LeafContainerBeingMerged>) -> Seq<InodeRecord> {
    leaf_records(contents_being_merged(containers))
}

/// 合并到一半的每片都装着 1 到 233 条。
spec fn every_leaf_being_merged_holds_one_to_a_full_container(containers: Seq<LeafContainerBeingMerged>) -> bool {
    forall|position: int|
        0 <= position < containers.len() ==> 1 <= (#[trigger] containers[position]).contents.records@.len() <= 233
}

/// 合并到一半时的不变量：每片 1 到 233 条，记录连起来严格升序且就是 `records`，每片记得上一版。
spec fn merging_is_on_track(
    containers: Seq<LeafContainerBeingMerged>,
    retired: Seq<RetiredInodeLeafContainer>,
    before: Seq<InodeLeafContainer>,
    records: Seq<InodeRecord>,
) -> bool {
    &&& every_leaf_being_merged_holds_one_to_a_full_container(containers)
    &&& leaf_records(contents_being_merged(containers)) == records
    &&& inode_records_strictly_ascend(records)
    &&& merging_keeps_track_of_the_previous_version(containers, retired, before)
}

impl LeafContainersBeingMerged {
    fn from_the_previous_version(containers: &[InodeLeafContainer]) -> (tree: Self)
        requires
            containers.len() <= u32::MAX,
        ensures
            tree.previous_version@ == containers@,
            tree.containers.len() == containers.len(),
            forall|position: int|
                0 <= position < tree.containers.len() ==> #[trigger] leaf_containers_are_the_same(
                    tree.containers@[position].contents,
                    containers@[position],
                ),
            tree.retired@.len() == 0,
            forall|position: int|
                0 <= position < tree.containers.len() ==> {
                    let container = #[trigger] tree.containers@[position];
                    &&& container.previous_index.0 == position
                    &&& !container.is_changed_this_publish
                },
    {
        let mut being_merged: Vec<LeafContainerBeingMerged> = Vec::new();
        let mut position: usize = 0;
        while position < containers.len()
            invariant
                position <= containers.len() <= u32::MAX,
                being_merged.len() == position,
                forall|done: int|
                    0 <= done < position ==> {
                        let container = #[trigger] being_merged@[done];
                        &&& leaf_containers_are_the_same(container.contents, containers@[done])
                        &&& container.previous_index.0 == done
                        &&& !container.is_changed_this_publish
                    },
            decreases containers.len() - position,
        {
            being_merged.push(
                LeafContainerBeingMerged {
                    contents: copy_of_one_leaf_container(&containers[position]),
                    previous_index: InodeLeafContainerIndexInTree::of_position(position),
                    is_changed_this_publish: false,
                },
            );
            position += 1;
        }
        Self { containers: being_merged, retired: Vec::new(), previous_version: Ghost(containers@) }
    }

    fn records_at(&self, position: usize) -> (records: usize)
        requires
            position < self.containers.len(),
        ensures
            records == self.containers@[position as int].contents.records@.len(),
    {
        self.containers[position].contents.records.len()
    }

    /// 这一对（`left_position` 与它右邻）合起来装得下一片（≤ 233 条）。
    fn pair_fits_in_one_container(&self, left_position: usize) -> (fits: bool)
        requires
            left_position + 1 < self.containers.len(),
            every_leaf_being_merged_holds_one_to_a_full_container(self.containers@),
        ensures
            fits == (self.containers@[left_position as int].contents.records@.len() + self.containers@[left_position + 1].contents.records@.len()
                <= 233),
    {
        proof {
            assert(self.containers@[left_position as int].contents.records@.len() <= 233);
            assert(self.containers@[left_position + 1].contents.records@.len() <= 233);
        }
        u64::try_from(self.records_at(left_position) + self.records_at(left_position + 1))
            .expect("两片之和至多 466 条")
            <= INODE_LEAF_RECORDS
    }

    /// 那一片的身份消失（删空摘掉或被吸收）：从这一版拿掉，身份连同上一版的叶序插进 `retired`（按上一版叶序，同序号的排在已有的之后）。
    fn retire(&mut self, position: usize) -> (retired: InodeLeafContainer)
        requires
            position < old(self).containers.len(),
            merging_keeps_track_of_the_previous_version(old(self).containers@, old(self).retired@, old(self).previous_version@),
        ensures
            final(self).previous_version == old(self).previous_version,
            final(self).containers@ == old(self).containers@.remove(position as int),
            retired == old(self).containers@[position as int].contents,
            final(self).retired@.len() == old(self).retired@.len() + 1,
            merging_keeps_track_of_the_previous_version(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
            ),
    {
        let ghost before = self.previous_version@;
        let removed = self.containers.remove(position);
        let entry = RetiredInodeLeafContainer { previous_index: removed.previous_index, identity: removed.contents.identity };
        let ghost retired_before = self.retired@;
        let mut insertion_point: usize = 0;
        while insertion_point < self.retired.len() && self.retired[insertion_point].previous_index.0 <= entry.previous_index.0
            invariant
                self.retired@ == retired_before,
                insertion_point <= self.retired.len(),
                forall|earlier: int| 0 <= earlier < insertion_point ==> (#[trigger] self.retired@[earlier]).previous_index.0
                    <= entry.previous_index.0,
            decreases self.retired.len() - insertion_point,
        {
            insertion_point += 1;
        }
        self.retired.insert(insertion_point, entry);
        proof {
            let retired_after = self.retired@;
            assert forall|earlier: int, later: int| 0 <= earlier < later < retired_after.len() implies (#[trigger] retired_after[earlier]).previous_index.0
                <= (#[trigger] retired_after[later]).previous_index.0 by {
                if later < insertion_point {
                    assert(retired_after[earlier] == retired_before[earlier]);
                    assert(retired_after[later] == retired_before[later]);
                } else if later == insertion_point {
                    assert(retired_after[earlier] == retired_before[earlier]);
                } else if earlier < insertion_point {
                    assert(retired_after[earlier] == retired_before[earlier]);
                    assert(retired_after[later] == retired_before[later - 1]);
                    assert(retired_before[earlier].previous_index.0 <= entry.previous_index.0);
                    assert(retired_before[insertion_point as int].previous_index.0 > entry.previous_index.0);
                    if later - 1 > insertion_point {
                        assert(retired_before[insertion_point as int].previous_index.0 <= retired_before[later - 1].previous_index.0);
                    }
                } else if earlier == insertion_point {
                    assert(retired_after[later] == retired_before[later - 1]);
                    assert(retired_before[insertion_point as int].previous_index.0 > entry.previous_index.0);
                    if later - 1 > insertion_point {
                        assert(retired_before[insertion_point as int].previous_index.0 <= retired_before[later - 1].previous_index.0);
                    }
                } else {
                    assert(retired_after[earlier] == retired_before[earlier - 1]);
                    assert(retired_after[later] == retired_before[later - 1]);
                }
            }
            assert forall|listed: int| 0 <= listed < retired_after.len() implies (#[trigger] retired_after[listed]).previous_index.0
                < before.len() && retired_after[listed].identity == before[retired_after[listed].previous_index.0 as int].identity by {
                if listed < insertion_point {
                    assert(retired_after[listed] == retired_before[listed]);
                } else if listed > insertion_point {
                    assert(retired_after[listed] == retired_before[listed - 1]);
                }
            }
            assert forall|any_position: int| 0 <= any_position < self.containers@.len() implies {
                let container = #[trigger] self.containers@[any_position];
                &&& container.previous_index.0 < before.len()
                &&& container.contents.identity == before[container.previous_index.0 as int].identity
                &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                    container.contents,
                    before[container.previous_index.0 as int],
                )
            } by {
                let in_the_old = if any_position < position { any_position } else { any_position + 1 };
                assert(self.containers@[any_position] == old(self).containers@[in_the_old]);
            }
        }
        removed.contents
    }


    /// 左片吸收它右邻：左片保留身份与分隔 key，右片的记录接在后面，右片身份消失。
    fn absorb_the_right_neighbour(&mut self, left_position: usize)
        requires
            left_position + 1 < old(self).containers.len(),
            merging_is_on_track(
                old(self).containers@,
                old(self).retired@,
                old(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
            old(self).containers@[left_position as int].contents.records@.len() + old(self).containers@[left_position
                + 1].contents.records@.len() <= 233,
        ensures
            final(self).previous_version == old(self).previous_version,
            final(self).containers.len() == old(self).containers.len() - 1,
            merging_is_on_track(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
    {
        let ghost before = self.previous_version@;
        let ghost containers_before = self.containers@;
        let mut absorbed = self.retire(left_position + 1);
        let survivor = &mut self.containers[left_position];
        survivor.contents.records.append(&mut absorbed.records);
        survivor.is_changed_this_publish = true;
        proof {
            let containers_after = self.containers@;
            let merged = containers_after[left_position as int].contents;
            assert(contents_being_merged(containers_after) =~= contents_being_merged(containers_before).remove(
                left_position + 1,
            ).update(left_position as int, merged));
            lemma_merging_two_neighbours_keeps_the_leaf_records(
                contents_being_merged(containers_before),
                left_position as int,
                merged,
            );
            assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (
            #[trigger] containers_after[position]).contents.records@.len() <= 233 by {
                if position < left_position {
                    assert(containers_after[position] == containers_before[position]);
                } else if position > left_position {
                    assert(containers_after[position] == containers_before[position + 1]);
                }
            }
            assert forall|position: int| 0 <= position < containers_after.len() implies {
                let container = #[trigger] containers_after[position];
                &&& container.previous_index.0 < before.len()
                &&& container.contents.identity == before[container.previous_index.0 as int].identity
                &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                    container.contents,
                    before[container.previous_index.0 as int],
                )
            } by {
                if position < left_position {
                    assert(containers_after[position] == containers_before[position]);
                } else if position > left_position {
                    assert(containers_after[position] == containers_before[position + 1]);
                } else {
                    assert(containers_after[position].contents.identity == containers_before[position].contents.identity);
                    assert(containers_after[position].previous_index == containers_before[position].previous_index);
                }
            }
        }
    }

    /// 从 `start_position` 那片起先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右；并完从幸存那片接着查，直到没有一对装得下
    /// （E176 M3 主取样点「先查右边」）。交回这一趟有没有并过；一次都没并过时什么都没动，且它与右邻装不进一片。
    ///
    /// 迭代上界是片数：每一轮要么并掉一片、要么退出；跨轮携带的是「当前那片」的位置，它只往左挪（左邻吸收它时换成左邻），
    /// 被摘掉的总在它右边或就是它被左邻吸收，所以位置一直指着同一片幸存的叶。
    fn absorb_until_no_pair_fits(&mut self, start_position: usize) -> (absorbed_at_least_once: bool)
        requires
            start_position < old(self).containers.len(),
            merging_is_on_track(
                old(self).containers@,
                old(self).retired@,
                old(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
        ensures
            final(self).previous_version == old(self).previous_version,
            merging_is_on_track(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
            !absorbed_at_least_once ==> final(self).containers@ == old(self).containers@ && final(self).retired@ == old(
                self,
            ).retired@ && !(start_position + 1 < old(self).containers.len() && old(self).containers@[start_position
                as int].contents.records@.len() + old(self).containers@[start_position + 1].contents.records@.len() <= 233),
    {
        let ghost containers_at_the_start = self.containers@;
        let ghost retired_at_the_start = self.retired@;
        let ghost records = records_being_merged(self.containers@);
        let mut current_position = start_position;
        let mut absorbed_at_least_once = false;
        loop
            invariant
                containers_at_the_start == old(self).containers@,
                retired_at_the_start == old(self).retired@,
                current_position < self.containers.len(),
                self.previous_version == old(self).previous_version,
                records == records_being_merged(containers_at_the_start),
                merging_is_on_track(self.containers@, self.retired@, self.previous_version@, records),
                !absorbed_at_least_once ==> current_position == start_position && self.containers@ == containers_at_the_start
                    && self.retired@ == retired_at_the_start,
            decreases self.containers.len(),
        {
            let right_pair_fits = current_position + 1 < self.containers.len() && self.pair_fits_in_one_container(
                current_position,
            );
            let left_pair_fits = current_position > 0 && self.pair_fits_in_one_container(current_position - 1);
            if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else {
                return absorbed_at_least_once;
            }
            absorbed_at_least_once = true;
        }
    }

    /// 左从右借（D8（核心索引结构） 已定项 6；E176 M3）：`borrower_position` 那片记录数小于下限且有右邻时，把右邻最小的
    /// ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾（均分：借完两片条数相差不超过 1），两片都保留身份，右邻的分隔 key 改成它这一版的最小 key（N1）；
    /// 借完从右邻起照 [`Self::absorb_until_no_pair_fits`] 并。它是最右那片时不借（右从左借会破「记录号 ≥ 容器号」）。
    ///
    /// # Panics
    /// 要借的条数不在 [1, 右邻记录数) 里（`requires` 排除的那一种）：调用方只在「从这片起一次都没并过」之后调它，那时它与右邻之和 > 233，
    /// 它又少于 117 条 ⇒ 右邻多于 116 条、借的条数 ⌊(右 − 左) ÷ 2⌋ ≥ 117 − 左 ≥ 1，且小于右邻的条数。
    fn borrow_from_the_right_neighbour_if_below_the_floor(&mut self, borrower_position: usize)
        requires
            borrower_position < old(self).containers.len(),
            merging_is_on_track(
                old(self).containers@,
                old(self).retired@,
                old(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
            borrower_position + 1 < old(self).containers.len() ==> old(self).containers@[borrower_position as int].contents.records@.len()
                + old(self).containers@[borrower_position + 1].contents.records@.len() > 233,
        ensures
            final(self).previous_version == old(self).previous_version,
            merging_is_on_track(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
    {
        let ghost before = self.previous_version@;
        let lender_position = borrower_position + 1;
        if lender_position >= self.containers.len() {
            return;
        }
        let records_in_the_borrower = self.records_at(borrower_position);
        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour() {
            return;
        }
        let records_in_the_lender = self.records_at(lender_position);
        proof {
            assert(records_in_the_lender <= 233);
            assert(records_in_the_borrower >= 1);
        }
        let moved_count = match records_in_the_lender.checked_sub(records_in_the_borrower) {
            Some(difference) => difference / 2,
            None => 0,
        };
        assert!(
            moved_count >= 1 && moved_count < records_in_the_lender,
            "借的条数要在 [1, 右邻记录数) 里：借方 {records_in_the_borrower} 条、右邻 {records_in_the_lender} 条、借 {moved_count} 条（这一对没并成说明两片之和 > 233，借方又少于 117 条）"
        );
        let ghost containers_before = self.containers@;
        let lender = &mut self.containers[lender_position];
        let mut moved = lender.contents.records.split_off(moved_count);
        std::mem::swap(&mut lender.contents.records, &mut moved);
        lender.contents.separator_key_in_the_parent_entry = lender
            .contents
            .records
            .first()
            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条")
            .inode;
        lender.is_changed_this_publish = true;
        let borrower = &mut self.containers[borrower_position];
        borrower.contents.records.append(&mut moved);
        borrower.is_changed_this_publish = true;
        proof {
            let containers_after = self.containers@;
            assert(contents_being_merged(containers_after) =~= contents_being_merged(containers_before).update(
                borrower_position as int,
                containers_after[borrower_position as int].contents,
            ).update(lender_position as int, containers_after[lender_position as int].contents));
            assert(containers_after[borrower_position as int].contents.records@ + containers_after[lender_position as int].contents.records@
                =~= containers_before[borrower_position as int].contents.records@ + containers_before[lender_position as int].contents.records@);
            lemma_moving_records_between_neighbours_keeps_the_leaf_records(
                contents_being_merged(containers_before),
                borrower_position as int,
                containers_after[borrower_position as int].contents,
                containers_after[lender_position as int].contents,
            );
            assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (
            #[trigger] containers_after[position]).contents.records@.len() <= 233 by {
                if position != borrower_position && position != lender_position {
                    assert(containers_after[position] == containers_before[position]);
                }
            }
            assert forall|position: int| 0 <= position < containers_after.len() implies {
                let container = #[trigger] containers_after[position];
                &&& container.previous_index.0 < before.len()
                &&& container.contents.identity == before[container.previous_index.0 as int].identity
                &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                    container.contents,
                    before[container.previous_index.0 as int],
                )
            } by {
                if position != borrower_position && position != lender_position {
                    assert(containers_after[position] == containers_before[position]);
                } else {
                    assert(containers_after[position].contents.identity == containers_before[position].contents.identity);
                    assert(containers_after[position].previous_index == containers_before[position].previous_index);
                }
            }
        }
        self.absorb_until_no_pair_fits(lender_position);
    }

    fn after_this_publish(self) -> (after: InodeLeafContainersAfterThisPublish)
        requires
            merging_keeps_track_of_the_previous_version(self.containers@, self.retired@, self.previous_version@),
            self.previous_version@.len() <= u32::MAX,
        ensures
            after.containers@ == contents_being_merged(self.containers@),
            removal_keeps_track_of_the_previous_version(self.previous_version@, after),
    {
        let ghost before = self.previous_version@;
        let ghost containers_at_the_start = self.containers@;
        let mut tree = self;
        let mut rewritten: Vec<InodeLeafContainerIndexInTree> = Vec::new();
        let mut origins: Vec<InodeLeafContainerOrigin> = Vec::new();
        let mut containers: Vec<InodeLeafContainer> = Vec::new();
        let mut position: usize = 0;
        while position < tree.containers.len()
            invariant
                tree.containers.len() == containers_at_the_start.len(),
                containers_at_the_start.len() <= before.len() <= u32::MAX,
                merging_keeps_track_of_the_previous_version(containers_at_the_start, tree.retired@, before),
                forall|later: int|
                    position <= later < tree.containers.len() ==> #[trigger] tree.containers@[later] == containers_at_the_start[later],
                position <= tree.containers.len(),
                containers.len() == position,
                origins.len() == position,
                forall|done: int| 0 <= done < position ==> #[trigger] containers@[done] == containers_at_the_start[done].contents,
                forall|done: int|
                    0 <= done < position ==> #[trigger] origins@[done] == if containers_at_the_start[done].is_changed_this_publish {
                        InodeLeafContainerOrigin::RewrittenThisPublish
                    } else {
                        InodeLeafContainerOrigin::CarriedFrom(containers_at_the_start[done].previous_index)
                    },
                indices_strictly_ascend(rewritten@),
                forall|listed: int| 0 <= listed < rewritten.len() ==> (#[trigger] rewritten@[listed]).0 < position,
                forall|done: int|
                    0 <= done < position ==> (indices_hold(rewritten@, done) <==> #[trigger] containers_at_the_start[done].is_changed_this_publish),
            decreases tree.containers.len() - position,
        {
            let is_changed = tree.containers[position].is_changed_this_publish;
            let previous_index = tree.containers[position].previous_index;
            let mut contents = InodeLeafContainer {
                identity: tree.containers[position].contents.identity,
                separator_key_in_the_parent_entry: 0,
                records: Vec::new(),
            };
            let ghost rewritten_before = rewritten@;
            let ghost containers_before_the_swap = tree.containers@;
            std::mem::swap(&mut tree.containers[position].contents, &mut contents);
            proof {
                assert(contents == containers_before_the_swap[position as int].contents);
                assert forall|later: int| position < later < tree.containers.len() implies #[trigger] tree.containers@[later]
                    == containers_at_the_start[later] by {
                    assert(tree.containers@[later] == containers_before_the_swap[later]);
                }
            }
            if is_changed {
                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));
                origins.push(InodeLeafContainerOrigin::RewrittenThisPublish);
            } else {
                origins.push(InodeLeafContainerOrigin::CarriedFrom(previous_index));
            }
            containers.push(contents);
            position += 1;
            proof {
                reveal(indices_hold);
                assert forall|done: int| 0 <= done < position implies (indices_hold(rewritten@, done)
                    <==> #[trigger] containers_at_the_start[done].is_changed_this_publish) by {
                    if done < position - 1 {
                        if indices_hold(rewritten@, done) {
                            let listed = choose|listed: int| 0 <= listed < rewritten@.len() && (#[trigger] rewritten@[listed]).0 == done;
                            if listed < rewritten_before.len() {
                                assert(rewritten_before[listed] == rewritten@[listed]);
                            }
                        }
                        if containers_at_the_start[done].is_changed_this_publish {
                            let listed = choose|listed: int|
                                0 <= listed < rewritten_before.len() && (#[trigger] rewritten_before[listed]).0 == done;
                            assert(rewritten@[listed] == rewritten_before[listed]);
                        }
                    } else if is_changed {
                        assert(rewritten@[rewritten@.len() - 1].0 == done);
                    } else {
                        assert(rewritten@ == rewritten_before);
                        if indices_hold(rewritten@, done) {
                            let listed = choose|listed: int| 0 <= listed < rewritten@.len() && (#[trigger] rewritten@[listed]).0 == done;
                        }
                    }
                }
            }
        }
        proof {
            assert(containers@ =~= contents_being_merged(containers_at_the_start));
            assert forall|done: int| 0 <= done < origins@.len() implies match #[trigger] origins@[done] {
                InodeLeafContainerOrigin::CarriedFrom(previous_index) => previous_index.0 < before.len()
                    && leaf_containers_are_the_same(containers@[done], before[previous_index.0 as int]),
                InodeLeafContainerOrigin::RewrittenThisPublish => true,
            } by {
                assert(containers_at_the_start[done] == containers_at_the_start[done]);
            }
        }
        InodeLeafContainersAfterThisPublish { containers, rewritten, origins, retired: tree.retired }
    }
}

/// 两片邻居并成一片（左片的记录接上右片的），记录连起来不变。
proof fn lemma_merging_two_neighbours_keeps_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    left_position: int,
    merged: InodeLeafContainer,
)
    requires
        0 <= left_position,
        left_position + 1 < containers.len(),
        merged.records@ == containers[left_position].records@ + containers[left_position + 1].records@,
    ensures
        leaf_records(containers.remove(left_position + 1).update(left_position, merged)) == leaf_records(containers),
{
    let before_the_pair = containers.take(left_position);
    let after_the_pair = containers.subrange(left_position + 2, containers.len() as int);
    let pair = seq![containers[left_position], containers[left_position + 1]];
    assert(containers =~= before_the_pair + pair + after_the_pair);
    assert(containers.remove(left_position + 1).update(left_position, merged) =~= before_the_pair + seq![merged] + after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, pair, after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, seq![merged], after_the_pair);
    lemma_leaf_records_of_two_containers(containers[left_position], containers[left_position + 1]);
    lemma_leaf_records_of_one_container(merged);
}

/// 两片邻居之间挪记录（两片的记录接起来不变），记录连起来不变。
proof fn lemma_moving_records_between_neighbours_keeps_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    left_position: int,
    left_after: InodeLeafContainer,
    right_after: InodeLeafContainer,
)
    requires
        0 <= left_position,
        left_position + 1 < containers.len(),
        left_after.records@ + right_after.records@ == containers[left_position].records@ + containers[left_position + 1].records@,
    ensures
        leaf_records(containers.update(left_position, left_after).update(left_position + 1, right_after)) == leaf_records(containers),
{
    let before_the_pair = containers.take(left_position);
    let after_the_pair = containers.subrange(left_position + 2, containers.len() as int);
    let pair = seq![containers[left_position], containers[left_position + 1]];
    let pair_after = seq![left_after, right_after];
    assert(containers =~= before_the_pair + pair + after_the_pair);
    assert(containers.update(left_position, left_after).update(left_position + 1, right_after) =~= before_the_pair + pair_after
        + after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, pair, after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, pair_after, after_the_pair);
    lemma_leaf_records_of_two_containers(containers[left_position], containers[left_position + 1]);
    lemma_leaf_records_of_two_containers(left_after, right_after);
}

/// 第 `holding_position` 片去掉第 `position_in_container` 条之后，记录连起来就是原来的去掉那一条（它之前的条数加 `position_in_container`）。
proof fn lemma_removing_a_record_keeps_the_rest_of_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    holding_position: int,
    position_in_container: int,
    holding_after: InodeLeafContainer,
)
    requires
        0 <= holding_position < containers.len(),
        0 <= position_in_container < containers[holding_position].records@.len(),
        holding_after.records@ == containers[holding_position].records@.remove(position_in_container),
    ensures
        leaf_records(containers.update(holding_position, holding_after)) == leaf_records(containers).remove(
            records_before_container(containers, holding_position) + position_in_container,
        ),
{
    lemma_leaf_records_around_one_container(containers, holding_position, holding_after);
    assert(leaf_records(containers.take(holding_position)) + holding_after.records@ + leaf_records(
        containers.subrange(holding_position + 1, containers.len() as int),
    ) =~= (leaf_records(containers.take(holding_position)) + containers[holding_position].records@ + leaf_records(
        containers.subrange(holding_position + 1, containers.len() as int),
    )).remove(records_before_container(containers, holding_position) + position_in_container));
}

/// 删空的那一片拿掉，记录连起来不变。
proof fn lemma_removing_an_empty_leaf_keeps_the_leaf_records(containers: Seq<InodeLeafContainer>, position: int)
    requires
        0 <= position < containers.len(),
        containers[position].records@.len() == 0,
    ensures
        leaf_records(containers.remove(position)) == leaf_records(containers),
{
    let earlier = containers.take(position);
    let later = containers.subrange(position + 1, containers.len() as int);
    assert(containers =~= earlier + seq![containers[position]] + later);
    assert(containers.remove(position) =~= earlier + later);
    lemma_leaf_records_of_a_concatenation(earlier + seq![containers[position]], later);
    lemma_leaf_records_of_a_concatenation(earlier, seq![containers[position]]);
    lemma_leaf_records_of_a_concatenation(earlier, later);
    lemma_leaf_records_of_one_container(containers[position]);
    assert(leaf_records(earlier) + containers[position].records@ =~= leaf_records(earlier));
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inode_tree::tests::{live_inode_record, INODE_TREE, TREE_BIRTH_TXG};

    /// 三片叶容器：第一片 1..=233、第二片 234..=466、第三片只有 467（分裂纪律写出来的形状）。
    fn three_containers() -> Vec<InodeLeafContainer> {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=467u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("467 条");
        assert_eq!(after.containers.len(), 3, "233 + 233 + 1");
        after.containers
    }

    /// `full_containers` 片装满 233 条的叶容器（inode 1 起连号；容器号 = 每片第一条的号、分隔 key 同容器号）。
    fn full_containers(full_containers: u64) -> Vec<InodeLeafContainer> {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=233 * full_containers)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("满的几片");
        assert_eq!(
            after.containers.len(),
            usize::try_from(full_containers).expect("片数")
        );
        after.containers
    }

    /// 把第 `position` 片的记录留成按号升序的前 `kept` 条（直接在内容上改：这里判的是删一条之后的落法，不是怎么删到这个样子）。
    fn keep_the_first_records(containers: &mut [InodeLeafContainer], position: usize, kept: usize) {
        containers[position].records.truncate(kept);
    }

    fn container_numbers(containers: &[InodeLeafContainer]) -> Vec<u64> {
        containers
            .iter()
            .map(|container| container.identity.container)
            .collect()
    }

    fn record_counts(containers: &[InodeLeafContainer]) -> Vec<usize> {
        containers
            .iter()
            .map(|container| container.records.len())
            .collect()
    }

    /// 删一条、那一片还有记录、与左右邻都并不了、不少于 117 条：只重写那一片，身份不变，别的片照抄，没有身份消失。
    #[test]
    fn removing_a_record_that_triggers_no_merge_rewrites_only_that_container() {
        let before = full_containers(3);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
        assert_eq!(record_counts(&after.containers), vec![233, 232, 233]);
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "COW 重写身份不变"
        );
        assert!(
            after.containers[1]
                .records
                .iter()
                .all(|kept| kept.inode != 300),
            "那一条不在了"
        );
        assert_eq!(after.containers[0], before[0]);
        assert_eq!(after.containers[2], before[2]);
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)]
        );
        assert_eq!(
            after.origins,
            vec![
                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
                    0
                )),
                InodeLeafContainerOrigin::RewrittenThisPublish,
                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
                    2
                )),
            ]
        );
        assert!(after.retired.is_empty());
        assert!(after.is_the_root_rewritten());
    }

    /// 删空最右那一片：整片不落盘、身份消失，重写清单是空的，根照样重写（少了一条条目）。它左边没有右邻可并。
    #[test]
    fn removing_the_only_record_of_the_rightmost_container_drops_it_and_still_rewrites_the_root() {
        let before = three_containers();
        let after = remove_record_from_leaf_containers(&before, InodeNumber(467)).expect("删 467");
        assert_eq!(after.containers, before[..2].to_vec(), "前两片原样");
        assert!(after.rewritten.is_empty(), "没有一片要重写");
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(2),
                identity: before[2].identity,
            }]
        );
        assert!(after.is_the_root_rewritten(), "根少一条条目，要重写");
    }

    /// 删空最左一片（M1 写出来的树才有这种形状：左边一片只剩一条）：整片不落盘、身份消失；它没有左邻，不并；
    /// 右边每一片左移一格、内容没变，照抄、不进重写清单（`origins` 记着上一版的叶序）——20 片与 40 片两种几何重写清单都是空的，
    /// 不随右边片数涨。
    #[test]
    fn dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number(
    ) {
        for containers_in_the_tree in [20_u64, 40] {
            let mut before = full_containers(containers_in_the_tree);
            keep_the_first_records(&mut before, 0, 1);
            let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
            assert_eq!(
                after.containers,
                before[1..].to_vec(),
                "{containers_in_the_tree} 片：右边各片内容、身份、分隔 key 原样"
            );
            assert!(
                after.rewritten.is_empty(),
                "{containers_in_the_tree} 片：挪了位置的一片都不重写"
            );
            assert_eq!(
                after.origins,
                (1..before.len())
                    .map(|previous_position| InodeLeafContainerOrigin::CarriedFrom(
                        InodeLeafContainerIndexInTree::of_position(previous_position)
                    ))
                    .collect::<Vec<_>>(),
                "{containers_in_the_tree} 片：这一版第 i 片照抄上一版第 i + 1 片"
            );
            assert_eq!(after.retired_identities(), vec![before[0].identity]);
        }
    }

    /// 删空中间一片（M1 写出来的形状）：身份消失；左邻与新右邻之和 > 233 不并；新右邻左移一格照抄，左边那片不动。
    #[test]
    fn dropping_an_emptied_middle_container_carries_the_ones_to_its_right_without_rewriting_them() {
        let mut before = three_containers();
        keep_the_first_records(&mut before, 1, 1);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(234)).expect("删 234");
        assert_eq!(after.containers, vec![before[0].clone(), before[2].clone()]);
        assert!(
            after.rewritten.is_empty(),
            "左邻 233 + 新右邻 1 = 234 > 233，不并"
        );
        assert_eq!(
            after.origins[1],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
        );
        assert_eq!(after.retired_identities(), vec![before[1].identity]);
    }

    /// 左吸收右（M3）：第一片删完一条剩 99 条、第二片 100 条，之和 199 ≤ 233 ⇒ 第一片吸收第二片：保留左半身份与分隔 key，记录接在后面；
    /// 第二片身份消失；第三片左移一格照抄。前后叶序：容器号 [1, 234, 467] → [1, 467]。
    #[test]
    fn the_left_container_absorbs_its_right_neighbour_when_the_two_fit_in_one_and_keeps_its_identity(
    ) {
        let mut before = full_containers(3);
        keep_the_first_records(&mut before, 0, 100);
        keep_the_first_records(&mut before, 1, 100);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
        assert_eq!(container_numbers(&after.containers), vec![1, 467]);
        assert_eq!(record_counts(&after.containers), vec![199, 233]);
        assert_eq!(
            after.containers[0].identity, before[0].identity,
            "保留左半身份"
        );
        assert_eq!(after.containers[0].separator_key(), 1, "左半分隔 key 不变");
        assert_eq!(
            after.containers[0]
                .records
                .last()
                .map(|record| record.inode),
            Some(333),
            "右半的记录接在后面"
        );
        assert_eq!(after.containers[1], before[2], "第三片原样");
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
        assert_eq!(
            after.origins[1],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
        );
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(1),
                identity: before[1].identity,
            }],
            "被吸收的右半身份消失"
        );
    }

    /// 左从右借（M3 均分借）：第一片删完一条剩 99 条、第二片 233 条，之和 > 233 不并；99 < 117 ⇒ 从第二片借最小的 ⌊(233 − 99) ÷ 2⌋ = 67 条，
    /// 借完两片 166 / 166（相差不超过 1）；两片都保留身份，第二片的分隔 key 改成它这一版的最小 key 301（N1）；没有身份消失，第三片照抄。
    #[test]
    fn container_below_the_floor_borrows_half_the_difference_from_its_right_neighbour() {
        let mut before = full_containers(3);
        keep_the_first_records(&mut before, 0, 100);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
        assert_eq!(container_numbers(&after.containers), vec![1, 234, 467]);
        assert_eq!(record_counts(&after.containers), vec![166, 166, 233]);
        assert_eq!(after.containers[0].identity, before[0].identity);
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "右片保留身份"
        );
        assert_eq!(
            after
                .containers
                .iter()
                .map(InodeLeafContainer::separator_key)
                .collect::<Vec<_>>(),
            vec![1, 301, 467],
            "右片的分隔 key = 它这一版的最小 key（N1），别的不变"
        );
        assert_eq!(
            after.containers[0]
                .records
                .last()
                .map(|record| record.inode),
            Some(300),
            "借来的是右片最小的 67 条：234..=300"
        );
        assert_eq!(after.containers[2], before[2]);
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1)
            ]
        );
        assert!(after.retired.is_empty(), "借不退役");
    }

    /// 借了之后再删到 116、右片又借、……：同一对一路删 233 条，最后一条让两片之和恰好 233，左吸收右。
    /// 均分借每次都让两片条数相差不超过 1；中间每一版相邻两片之和都 ≥ 234（M3 那一臂的丙类断言）。
    #[test]
    fn deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs() {
        let mut containers = full_containers(3);
        let mut borrows = 0;
        let mut deletions = 0;
        let mut absorbed = false;
        // 迭代上界：这一对一共 466 条，每一轮删一条；吸收那一轮退出。
        while !absorbed {
            let inode = containers[0].records[1].inode;
            let after = remove_record_from_leaf_containers(&containers, InodeNumber(inode))
                .expect("删左片第二条");
            deletions += 1;
            absorbed = !after.retired.is_empty();
            if after.rewritten.len() == 2 {
                borrows += 1;
                let counts = record_counts(&after.containers);
                assert!(counts[0].abs_diff(counts[1]) <= 1, "均分借：{counts:?}");
            }
            if !absorbed {
                for pair in after.containers.windows(2) {
                    assert!(
                        pair[0].records.len() + pair[1].records.len() >= 234,
                        "相邻两片之和 ≥ 234"
                    );
                }
            }
            containers = after.containers;
        }
        assert_eq!(deletions, 233, "两片之和 466 删到 233 才装得下一片");
        assert_eq!(
            borrows, 7,
            "116 → 174 / 145 / 131 / 123 / 120 / 118 / 117 七次借"
        );
        assert_eq!(container_numbers(&containers), vec![1, 467]);
        assert_eq!(record_counts(&containers), vec![233, 233]);
    }

    /// 三片各 117 条、删中间那片的一条：
    /// 中间那片剩 116 条，与左邻、右邻之和都恰好 233，两对都装得下。M3「先查右邻」⇒ 中间那片吸收右邻，退役的是右邻（容器号 467），
    /// 左邻原样照抄；先查左邻的写法让左邻吸收中间那片、退役的是中间那片（容器号 234）。两种写法的镜像池级 checker 都不红，方向只有这里钉。
    #[test]
    fn removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first() {
        let mut before = full_containers(3);
        for position in 0..3 {
            keep_the_first_records(&mut before, position, 117);
        }
        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
        assert_eq!(
            container_numbers(&after.containers),
            vec![1, 234],
            "中间那片吸收右邻：留下左邻与中间那片"
        );
        assert_eq!(record_counts(&after.containers), vec![117, 233]);
        assert_eq!(after.containers[0], before[0], "左邻一个字节都不动");
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "吸收方保留中间那片的身份"
        );
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(2),
                identity: before[2].identity,
            }],
            "先查右邻：退役的是右邻，不是中间那片"
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)]
        );
        assert_eq!(
            after.origins[0],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::LEFTMOST)
        );
    }

    /// 删的是树里唯一一条（unlink 唯一的文件）：一片都不剩、没有要重写的片，那片身份消失，根照样重写成零条目根。
    #[test]
    fn removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root() {
        let before = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 4096, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("一条")
        .containers;
        let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
        assert!(after.containers.is_empty(), "一片都不剩");
        assert!(after.rewritten.is_empty());
        assert_eq!(after.retired_identities(), vec![before[0].identity]);
        assert!(after.is_the_root_rewritten(), "根重写成零条目根");
    }

    /// 要删的号不在树里：在任何落盘动作之前拒。
    #[test]
    fn removing_a_number_that_is_not_in_the_tree_is_refused() {
        let before = three_containers();
        assert_eq!(
            remove_record_from_leaf_containers(&before, InodeNumber(468)),
            Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree {
                inode: InodeNumber(468)
            })
        );
    }

    /// 等价性留档要比的两种改写（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第二行：在所有输入上与原式同值的变异，
    /// 把等价性写成测试留档）。：两条腿都推等价，
    /// 前提是「相邻两片之和 ≥ 234」——E176 M2、M3 的断言；追加只在最右那片满了才新开一片，删一条之后能并的一对之和恰好 233、
    /// 并完那片满 233 条，删空的那一片删之前只有 1 条、它的两个邻居都满着。
    #[derive(Clone, Copy, Debug)]
    enum MergeRuleRewriting {
        /// `k4-no-chain`：从某片起的每一趟并掉一对就停，不从幸存那片接着查（借完那一趟也是）。
        EveryPassStopsAfterAbsorbingOnePair,
        /// m4：删空的那一片摘掉之后不从它的左邻起并。
        NothingIsAbsorbedAfterDroppingTheEmptiedContainer,
    }

    /// 改写之下从 `start_position` 那片起的一趟并：交回并没并过。
    fn absorb_pass_under_the_rewriting(
        tree: &mut LeafContainersBeingMerged,
        start_position: usize,
        rewriting: MergeRuleRewriting,
    ) -> bool {
        match rewriting {
            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                if start_position + 1 < tree.containers.len()
                    && tree.pair_fits_in_one_container(start_position)
                {
                    tree.absorb_the_right_neighbour(start_position);
                    true
                } else if start_position > 0 && tree.pair_fits_in_one_container(start_position - 1)
                {
                    tree.absorb_the_right_neighbour(start_position - 1);
                    true
                } else {
                    false
                }
            }
            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
                tree.absorb_until_no_pair_fits(start_position)
            }
        }
    }

    /// 改写之下的左从右借：借的条数、挪哪几条、右片的分隔 key 与实现相同，借完那一趟并照改写走。
    fn borrow_under_the_rewriting(
        tree: &mut LeafContainersBeingMerged,
        borrower_position: usize,
        rewriting: MergeRuleRewriting,
    ) {
        match rewriting {
            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                let lender_position = borrower_position + 1;
                if lender_position >= tree.containers.len()
                    || tree.records_at(borrower_position)
                        >= records_below_which_a_leaf_borrows_from_its_right_neighbour()
                {
                    return;
                }
                let moved_count =
                    (tree.records_at(lender_position) - tree.records_at(borrower_position)) / 2;
                let moved_records: Vec<InodeRecord> = tree.containers[lender_position]
                    .contents
                    .records
                    .drain(..moved_count)
                    .collect();
                let smallest_kept_by_the_lender =
                    tree.containers[lender_position].contents.records[0].inode;
                tree.containers[lender_position]
                    .contents
                    .separator_key_in_the_parent_entry = smallest_kept_by_the_lender;
                tree.containers[lender_position].is_changed_this_publish = true;
                tree.containers[borrower_position]
                    .contents
                    .records
                    .extend(moved_records);
                tree.containers[borrower_position].is_changed_this_publish = true;
                absorb_pass_under_the_rewriting(tree, lender_position, rewriting);
            }
            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
                tree.borrow_from_the_right_neighbour_if_below_the_floor(borrower_position);
            }
        }
    }

    /// 改写之下删一条：删法、删空摘片、先并后借的次序与 [`remove_record_from_leaf_containers`] 相同，只有改写点到的那一处换掉。
    fn remove_record_under_the_rewriting(
        containers_before_this_publish: &[InodeLeafContainer],
        inode: InodeNumber,
        rewriting: MergeRuleRewriting,
    ) -> InodeLeafContainersAfterThisPublish {
        let holding_position =
            position_of_the_container_holding(containers_before_this_publish, inode)
                .expect("族里每一次删的号都取自那棵树")
                .position();
        let mut tree =
            LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
        tree.containers[holding_position]
            .contents
            .records
            .retain(|kept| kept.inode != inode.0);
        tree.containers[holding_position].is_changed_this_publish = true;
        if tree.records_at(holding_position) == 0 {
            tree.retire(holding_position);
            let has_new_neighbours_on_both_sides =
                holding_position > 0 && holding_position < tree.containers.len();
            match rewriting {
                MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                    if has_new_neighbours_on_both_sides {
                        absorb_pass_under_the_rewriting(&mut tree, holding_position - 1, rewriting);
                    }
                }
                MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {}
            }
        } else if !absorb_pass_under_the_rewriting(&mut tree, holding_position, rewriting) {
            borrow_under_the_rewriting(&mut tree, holding_position, rewriting);
        }
        tree.after_this_publish()
    }

    /// 等价性那一族每片记录数的取值：1（删空）、2（借后吸收里的第三片）、116 / 117 / 118（删一条可并、借、借 1 条）、233（满片）。
    const RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY: [usize; 6] = [1, 2, 116, 117, 118, 233];

    /// 那一族的树至多这么多片。
    const CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST: usize = 4;

    /// 1 到 4 片、每片记录数取自 [`RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY`]、相邻两片之和都 ≥ 234 的每一组记录数。
    fn record_counts_where_every_adjacent_pair_overflows_one_container() -> Vec<Vec<usize>> {
        let capacity = usize::try_from(INODE_LEAF_RECORDS).expect("233");
        let mut of_the_current_length: Vec<Vec<usize>> = vec![Vec::new()];
        let mut family = Vec::new();
        // 迭代上界：CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST 轮，每轮多一片。
        for _ in 0..CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST {
            of_the_current_length = of_the_current_length
                .iter()
                .flat_map(|prefix| {
                    RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY
                        .iter()
                        .filter(|count| prefix.last().is_none_or(|last| last + **count > capacity))
                        .map(|count| {
                            let mut longer = prefix.clone();
                            longer.push(*count);
                            longer
                        })
                })
                .collect();
            family.extend(of_the_current_length.iter().cloned());
        }
        family
    }

    /// 记录数照 `record_counts` 的一棵树：inode 号从 1 起连号，每片容器号与分隔 key = 它的第一条（分裂纪律写出来的形状）。
    fn tree_of_record_counts(record_counts: &[usize]) -> Vec<InodeLeafContainer> {
        let mut first_inode_of_the_next_container = 1_u64;
        record_counts
            .iter()
            .map(|count| {
                let first_inode = first_inode_of_the_next_container;
                first_inode_of_the_next_container += u64::try_from(*count).expect("至多 233");
                InodeLeafContainer {
                    identity: PackedIdentity {
                        birth_tree: INODE_TREE,
                        record_type: PACKED_TYPE_INODE,
                        container: first_inode,
                        container_birth: TREE_BIRTH_TXG,
                    },
                    separator_key_in_the_parent_entry: first_inode,
                    records: (first_inode..first_inode_of_the_next_container)
                        .map(|inode| live_inode_record(inode, 0, 3))
                        .collect(),
                }
            })
            .collect()
    }

    /// 每片删第一条与最后一条（只有一条的那片删一次）：删哪一片决定落法，删片里哪一条只改挪走哪几个号。
    fn inodes_removed_from_each_container(containers: &[InodeLeafContainer]) -> Vec<InodeNumber> {
        let mut removed = Vec::new();
        for container in containers {
            let smallest = container.smallest_inode_number().expect("族里每片至少一条");
            let largest = container.largest_inode_number().expect("族里每片至少一条");
            removed.push(smallest);
            if largest != smallest {
                removed.push(largest);
            }
        }
        removed
    }

    /// 这次删除里有片的分隔 key 变了（只有借会改：右片改成它这一版的最小 key）。
    fn some_separator_key_moved(
        before: &[InodeLeafContainer],
        after: &InodeLeafContainersAfterThisPublish,
    ) -> bool {
        after.containers.iter().any(|kept| {
            before.iter().any(|earlier| {
                earlier.identity == kept.identity
                    && earlier.separator_key_in_the_parent_entry
                        != kept.separator_key_in_the_parent_entry
            })
        })
    }

    /// `k4-no-chain`（连锁只并一轮）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，
    /// 实现与「每一趟并掉一对就停」逐片相同（容器、重写清单、来处、退役，整份比）——能并的一对之和恰好 233、并完那片满着，
    /// 再与哪个邻居相加都 > 233；借之后借方与借出片之和不变、仍 > 233，借出片吸收右邻之后同理。
    /// 族里删一条并、借后吸收、删空摘片后从左邻起那一趟各至少走到一次；删完之后相邻两片之和仍 ≥ 234（前提对删除封闭）。
    #[test]
    fn stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container(
    ) {
        let rewriting = MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair;
        let mut removals_that_absorbed_without_borrowing = 0;
        let mut removals_that_borrowed_and_then_absorbed = 0;
        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
            let before = tree_of_record_counts(&record_counts);
            for inode in inodes_removed_from_each_container(&before) {
                let holding_position = position_of_the_container_holding(&before, inode)
                    .expect("取自这棵树")
                    .position();
                let implementation = remove_record_from_leaf_containers(&before, inode)
                    .expect("族里每一次删的号都在树里");
                assert_eq!(
                    implementation,
                    remove_record_under_the_rewriting(&before, inode, rewriting),
                    "各片 {record_counts:?} 删 {inode:?}：实现与「每一趟并掉一对就停」不同"
                );
                for pair in implementation.containers.windows(2) {
                    assert!(
                        pair[0].records.len() + pair[1].records.len() >= 234,
                        "各片 {record_counts:?} 删 {inode:?}：删完之后相邻两片之和 < 234（前提不再成立）"
                    );
                }
                let emptied = record_counts[holding_position] == 1;
                let borrowed = some_separator_key_moved(&before, &implementation);
                if emptied && holding_position > 0 && holding_position + 1 < record_counts.len() {
                    removals_that_dropped_a_container_between_two_neighbours += 1;
                }
                if !emptied && !implementation.retired.is_empty() {
                    if borrowed {
                        removals_that_borrowed_and_then_absorbed += 1;
                    } else {
                        removals_that_absorbed_without_borrowing += 1;
                    }
                }
            }
        }
        assert!(
            removals_that_absorbed_without_borrowing >= 1,
            "族里有删一条就并的"
        );
        assert!(
            removals_that_borrowed_and_then_absorbed >= 1,
            "族里有借完借出的那一片接着并的"
        );
        assert!(
            removals_that_dropped_a_container_between_two_neighbours >= 1,
            "族里有删空摘片之后两边还有邻居、从左邻起看一趟的"
        );
    }

    /// m4（删空摘片后不再并）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，实现与「删空的那一片摘掉之后
    /// 不从它的左邻起并」逐片相同（整份比）——删空的那一片删之前只有 1 条，它的两个邻居都满 233 条，摘掉之后那一对之和 466，
    /// 左邻再与它的左邻之和也 > 233。族里删空摘片之后两边还有邻居的至少走到一次。
    #[test]
    fn not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container(
    ) {
        let rewriting = MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer;
        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
            let before = tree_of_record_counts(&record_counts);
            for inode in inodes_removed_from_each_container(&before) {
                let holding_position = position_of_the_container_holding(&before, inode)
                    .expect("取自这棵树")
                    .position();
                assert_eq!(
                    remove_record_from_leaf_containers(&before, inode)
                        .expect("族里每一次删的号都在树里"),
                    remove_record_under_the_rewriting(&before, inode, rewriting),
                    "各片 {record_counts:?} 删 {inode:?}：实现与「删空摘片之后不再并」不同"
                );
                if record_counts[holding_position] == 1
                    && holding_position > 0
                    && holding_position + 1 < record_counts.len()
                {
                    removals_that_dropped_a_container_between_two_neighbours += 1;
                }
            }
        }
        assert!(
            removals_that_dropped_a_container_between_two_neighbours >= 1,
            "族里有删空摘片之后两边还有邻居的"
        );
    }
}
