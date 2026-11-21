//! 一次发布要做的叶工作：合并相邻、工作叶、发布工作的各步。

use super::*;

/// 合并策略找邻片时只看活叶：桩不被借、不被吸收、不算邻片（修订 2 之五；原写路径上没有桩，这一句不改它的任何一步）。
fn is_a_merge_neighbour(leaf: &WorkingLeaf) -> bool {
    !leaf.records.is_empty()
}

pub(super) struct PublishWork {
    pub(super) container_death: ContainerDeathWritePath,
    /// B-stub 写路径这次原位落了桩的身份（原写路径恒空）。
    pub(super) stubbed_identities: Vec<ContainerIdentity>,
    pub(super) first_new_unit: UnitIndex,
    pub(super) write_order: WriteOrder,
    pub(super) publish_sequence: u64,
    pub(super) new_units: Vec<Unit>,
    pub(super) leaves: Vec<WorkingLeaf>,
    pub(super) leaf_set_changed: bool,
    pub(super) inode_root_rewritten_anyway: bool,
    pub(super) retired_identities: Vec<ContainerIdentity>,
    pub(super) moved_objects: Vec<u64>,
    pub(super) tombstone_appends: Vec<(TombstoneStream, TombstoneRecord)>,
    pub(super) merge: MergeAction,
    pub(super) split_checks: Vec<bool>,
    pub(super) written: UnitsWritten,
    pub(super) shrink: Option<ShrinkEvent>,
}

pub(super) fn working_leaves_of(leaves: &[Leaf]) -> Vec<WorkingLeaf> {
    leaves
        .iter()
        .map(|leaf| WorkingLeaf {
            identity: leaf.identity,
            separator: leaf.separator,
            records: Arc::clone(&leaf.records),
            carried_unit: Some(leaf.unit),
        })
        .collect()
}

impl PublishWork {
    pub(super) fn reserve(&mut self, body: UnitBody) -> UnitIndex {
        self.new_units.push(Unit {
            write_order: self.write_order,
            written_at_publish: self.publish_sequence,
            replaced_at_publish: None,
            body,
        });
        self.first_new_unit + self.new_units.len() - 1
    }

    /// 最右那片活叶（读法 10；原写路径上没有桩，就是最后一片）。
    fn position_of_the_rightmost_live_leaf(&self) -> Option<usize> {
        self.leaves.iter().rposition(|leaf| !leaf.records.is_empty())
    }

    fn largest_key_in_the_tree(&self) -> Option<u64> {
        self.position_of_the_rightmost_live_leaf()
            .and_then(|position| self.leaves[position].records.last())
            .map(|record| record.inode)
    }

    fn merge_neighbour_to_the_right_of(&self, position: usize) -> Option<usize> {
        (position + 1..self.leaves.len()).find(|candidate| is_a_merge_neighbour(&self.leaves[*candidate]))
    }

    fn merge_neighbour_to_the_left_of(&self, position: usize) -> Option<usize> {
        (0..position).rev().find(|candidate| is_a_merge_neighbour(&self.leaves[*candidate]))
    }

    /// B-stub：消失的身份原位换成一个零条目版本（这次写、之后不再重写），右边各片不动（修订 2 之五）。
    fn leave_a_stub_in_place(&mut self, position: usize) {
        let leaf = &mut self.leaves[position];
        Arc::make_mut(&mut leaf.records).clear();
        leaf.carried_unit = None;
        self.leaf_set_changed = true;
        self.stubbed_identities.push(leaf.identity);
    }

    fn position_of_the_leaf_holding(&self, inode: u64) -> Option<usize> {
        self.leaves
            .iter()
            .position(|leaf| leaf.records.binary_search_by_key(&inode, |record| record.inode).is_ok())
    }

    pub(super) fn update_record(&mut self, inode: u64, update: impl FnOnce(&mut InodeRecord)) {
        let position = self.position_of_the_leaf_holding(inode).expect("改的记录在树里");
        let leaf = &mut self.leaves[position];
        let records = Arc::make_mut(&mut leaf.records);
        let index_in_leaf = records
            .binary_search_by_key(&inode, |record| record.inode)
            .expect("position_of_the_leaf_holding 找到的那一片装着它");
        update(&mut records[index_in_leaf]);
        leaf.carried_unit = None;
    }

    /// 新号只落最右叶；最右那片满了在末尾分裂：右半容器号 = 触发分裂那条记录的 inode 号、出生代 = 本次 txg，左半记录与身份不变
    /// （D8 已定项 6；`crates/singlefs-core/src/inode_tree.rs:181-283`）。第七节 K6 在每次分裂上判。
    pub(super) fn insert_new_records(&mut self, records: &[InodeRecord], leaf_capacity: u64) {
        let txg = self.write_order.birth_txg;
        for record in records {
            let lands_beyond_every_key = self.largest_key_in_the_tree().is_none_or(|largest| record.inode > largest);
            assert!(lands_beyond_every_key, "inode 号单调：新号大于树里每一个 key");
            let rightmost_live = self.position_of_the_rightmost_live_leaf();
            let rightmost_has_room =
                rightmost_live.is_some_and(|position| length_as_u64(self.leaves[position].records.len()) < leaf_capacity);
            if rightmost_has_room {
                let rightmost = &mut self.leaves[rightmost_live.expect("上一行看过有最右那片活叶")];
                Arc::make_mut(&mut rightmost.records).push(*record);
                rightmost.carried_unit = None;
                continue;
            }
            let left_before_the_split =
                rightmost_live.map(|position| (position, self.leaves[position].identity, Arc::clone(&self.leaves[position].records)));
            self.leaves.push(WorkingLeaf {
                identity: ContainerIdentity::inode_leaf(record.inode, txg),
                separator: record.inode,
                records: Arc::new(vec![*record]),
                carried_unit: None,
            });
            self.leaf_set_changed = true;
            let right_half = self.leaves.last().expect("刚推进去");
            let right_half_holds = right_half.identity.number == record.inode && right_half.identity.birth == txg;
            let left_half_holds = match left_before_the_split {
                Some((left_position, identity, left_records)) => {
                    let left_half = &self.leaves[left_position];
                    left_half.identity == identity && left_half.records == left_records
                }
                None => true,
            };
            self.split_checks.push(right_half_holds && left_half_holds);
        }
    }

    /// 左移的每一片这次全部重写（`inode_tree.rs:286-318` 的位置规则）。
    fn mark_shifted_leaves_rewritten(&mut self, first_shifted_position: usize) {
        self.leaf_set_changed = true;
        for leaf in self.leaves.iter_mut().skip(first_shifted_position) {
            leaf.carried_unit = None;
        }
    }

    /// 左吸收右：左片保留身份与分隔 key，右片的记录全部挪进左片。原写路径右片不再出现、它右边各片左移重写，交回它的身份去写
    /// 退役记录；B-stub 写路径右片原位换成桩、不左移，不交回身份。
    fn absorb_right_into_left(&mut self, left_position: usize, right_position: usize) -> Option<ContainerIdentity> {
        let moving: Vec<InodeRecord> = std::mem::take(Arc::make_mut(&mut self.leaves[right_position].records));
        self.moved_objects.extend(moving.iter().map(|record| record.inode));
        {
            let left = &mut self.leaves[left_position];
            Arc::make_mut(&mut left.records).extend(moving);
            left.carried_unit = None;
        }
        self.leaf_set_changed = true;
        match self.container_death {
            ContainerDeathWritePath::RetirementRecords => {
                let right = self.leaves.remove(right_position);
                let retired_identity = right.identity;
                self.mark_shifted_leaves_rewritten(right_position);
                Some(retired_identity)
            }
            ContainerDeathWritePath::ZeroRecordStubs => {
                self.leave_a_stub_in_place(right_position);
                None
            }
        }
    }

    /// 左从右借：右片最小的 `records_moved` 条挪进左片，两片都留着、都重写；右片身份按 N1 / N2 处理。
    fn borrow_from_the_right(&mut self, left_position: usize, right_position: usize, records_moved: u64, borrow_arm: BorrowArm) {
        let txg = self.write_order.birth_txg;
        let moving: Vec<InodeRecord> = {
            let right_records = Arc::make_mut(&mut self.leaves[right_position].records);
            right_records.drain(..u64_as_index(records_moved)).collect()
        };
        self.moved_objects.extend(moving.iter().map(|record| record.inode));
        {
            let left = &mut self.leaves[left_position];
            Arc::make_mut(&mut left.records).extend(moving);
            left.carried_unit = None;
        }
        let smallest_key_after_borrow =
            self.leaves[right_position].records.first().expect("借之后右片还有记录").inode;
        let right_leaf = &mut self.leaves[right_position];
        right_leaf.carried_unit = None;
        match borrow_arm {
            BorrowArm::RaiseSeparator => {
                right_leaf.separator = smallest_key_after_borrow;
            }
            BorrowArm::RenewIdentity => {
                let replaced_identity = right_leaf.identity;
                let renewed_identity = ContainerIdentity {
                    birth_tree: replaced_identity.birth_tree,
                    record_type: replaced_identity.record_type,
                    number: smallest_key_after_borrow,
                    birth: txg,
                };
                right_leaf.identity = renewed_identity;
                right_leaf.separator = renewed_identity.number;
                self.retired_identities.push(replaced_identity);
            }
        }
    }

    /// 删一条记录，再按 M(θ) 合并（登记 5.1）：删空照今天不落盘；|C| < θ 时有右邻就吸收或借，C 是最右一片就让左邻吸收。
    pub(super) fn remove_record_and_merge(&mut self, inode: u64, cell: WritePathCell) -> MergeAction {
        let position = self.position_of_the_leaf_holding(inode).expect("删的记录在树里");
        {
            let leaf = &mut self.leaves[position];
            let records = Arc::make_mut(&mut leaf.records);
            let index_in_leaf = records
                .binary_search_by_key(&inode, |record| record.inode)
                .expect("position_of_the_leaf_holding 找到的那一片装着它");
            records.remove(index_in_leaf);
            leaf.carried_unit = None;
        }
        if self.leaves[position].records.is_empty() {
            match self.container_death {
                ContainerDeathWritePath::RetirementRecords => {
                    let emptied_leaf = self.leaves.remove(position);
                    self.retired_identities.push(emptied_leaf.identity);
                    self.mark_shifted_leaves_rewritten(position);
                }
                ContainerDeathWritePath::ZeroRecordStubs => self.leave_a_stub_in_place(position),
            }
            return MergeAction::EmptiedAndDropped;
        }
        let size_of_this_leaf = length_as_u64(self.leaves[position].records.len());
        if cell.merge_threshold == 0 || size_of_this_leaf >= cell.merge_threshold {
            return MergeAction::NoMerge;
        }
        if let Some(right_position) = self.merge_neighbour_to_the_right_of(position) {
            let size_of_the_right_leaf = length_as_u64(self.leaves[right_position].records.len());
            if size_of_this_leaf + size_of_the_right_leaf <= cell.leaf_capacity {
                if let Some(retired_identity) = self.absorb_right_into_left(position, right_position) {
                    self.retired_identities.push(retired_identity);
                }
                return MergeAction::AbsorbedTheRightNeighbour;
            }
            let records_moved = (size_of_the_right_leaf - size_of_this_leaf).div_ceil(2);
            self.borrow_from_the_right(position, right_position, records_moved, cell.borrow_arm);
            return MergeAction::BorrowedFromTheRightNeighbour { records_moved };
        }
        if let Some(left_position) = self.merge_neighbour_to_the_left_of(position) {
            let size_of_the_left_leaf = length_as_u64(self.leaves[left_position].records.len());
            if size_of_the_left_leaf + size_of_this_leaf <= cell.leaf_capacity {
                if let Some(retired_identity) = self.absorb_right_into_left(left_position, position) {
                    self.retired_identities.push(retired_identity);
                }
                return MergeAction::AbsorbedIntoTheLeftNeighbour;
            }
        }
        MergeAction::NoMerge
    }

    /// 叶容器落单元：重写的那几片各写一个新单元；有片重写、片数变了或回退时，inode 树根重写一次。
    pub(super) fn finish_leaves(&mut self) -> Vec<Leaf> {
        let leaves = std::mem::take(&mut self.leaves);
        let mut finished = Vec::with_capacity(leaves.len());
        let mut any_leaf_rewritten = false;
        for leaf in leaves {
            let unit = if let Some(carried) = leaf.carried_unit {
                carried
            } else {
                any_leaf_rewritten = true;
                self.written.inode_leaf_containers += 1;
                self.reserve(UnitBody::InodeLeaf { identity: leaf.identity, records: Arc::clone(&leaf.records) })
            };
            finished.push(Leaf { identity: leaf.identity, separator: leaf.separator, records: leaf.records, unit });
        }
        if any_leaf_rewritten || self.leaf_set_changed || self.inode_root_rewritten_anyway {
            self.written.inode_roots += 1;
        }
        finished
    }

    /// 往墓碑树或退役树追加：开放容器有新记录那次重写，满了开新容器（号 = 下一个号、出生代 = 本次 txg）。交回这次写了几个容器。
    pub(super) fn append_into_packed_tree<Record: Clone>(
        &mut self,
        tree: &mut PackedTree<Record>,
        appended: &[Record],
        container_capacity: u64,
        identity_template: (u64, u64),
        body_of: &dyn Fn(ContainerIdentity, Arc<Vec<Record>>) -> UnitBody,
    ) -> u64 {
        let (birth_tree, record_type) = identity_template;
        let mut touched_positions: BTreeSet<usize> = BTreeSet::new();
        for record in appended {
            let needs_a_new_container = tree
                .containers
                .last()
                .is_none_or(|last| length_as_u64(last.records.len()) >= container_capacity);
            if needs_a_new_container {
                let identity = ContainerIdentity {
                    birth_tree,
                    record_type,
                    number: tree.next_container_number,
                    birth: self.write_order.birth_txg,
                };
                tree.next_container_number += 1;
                tree.containers.push(PackedContainer { identity, records: Arc::new(Vec::new()), unit: UnitIndex::MAX });
            }
            let position = tree.containers.len() - 1;
            Arc::make_mut(&mut tree.containers[position].records).push(record.clone());
            touched_positions.insert(position);
        }
        for position in &touched_positions {
            let identity = tree.containers[*position].identity;
            let records = Arc::clone(&tree.containers[*position].records);
            tree.containers[*position].unit = self.reserve(body_of(identity, records));
        }
        length_as_u64(touched_positions.len())
    }
}
