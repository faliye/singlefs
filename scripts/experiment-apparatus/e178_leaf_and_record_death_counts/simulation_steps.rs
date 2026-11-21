//! 模拟器的步进：建对象、写、删、合并、回退、恢复与扫描。

use super::*;

impl Simulation {
    /// 新池新建文件（txg 3，mkfs 那个进程的实例 1）、可写挂载的写行（txg 4）与暖机（txg 5）（登记 5.2，E177 装置第 3331–3336 行）。
    pub(super) fn start(cell: WritePathCell, batch_data_units: u64, scanning: bool) -> Self {
        Self::start_with(cell, batch_data_units, scanning, ContainerDeathWritePath::RetirementRecords, every_reader())
    }

    pub(super) fn start_with(
        cell: WritePathCell,
        batch_data_units: u64,
        scanning: bool,
        container_death: ContainerDeathWritePath,
        readers: Vec<Reader>,
    ) -> Self {
        assert!(
            container_death == ContainerDeathWritePath::RetirementRecords || cell.borrow_arm == BorrowArm::RaiseSeparator,
            "B-stub 只配 N1（修订 2 问法一 N2-none）"
        );
        let mut simulation = Self {
            cell,
            container_death,
            readers,
            batch_data_units,
            disk: Disk::default(),
            version: Version::genesis(),
            publish_sequence: 0,
            instance_rows: BTreeMap::new(),
            publishes: 0,
            refused_steps: 0,
            moved_objects: BTreeSet::new(),
            moved_then_shortened_objects: BTreeSet::new(),
            largest_size_before_a_shrink: BTreeMap::new(),
            moved_then_shortened: 0,
            moved_then_unlinked: 0,
            publish_records: Vec::new(),
            user_changes: Vec::new(),
            self_check: SelfCheckTally::default(),
            assertions: AssertionTally::default(),
            does_not_fit_at: None,
            scanning,
            scans: Vec::new(),
            gap_watch: None,
            gap_start: None,
            intent_txgs: BTreeMap::new(),
            positive_control_plan: PositiveControlPlan::default(),
            positive_control_outcomes: Vec::new(),
            unlinks_completed: 0,
            dead_versions_holding_a_finished_record: Vec::new(),
            segment_four: None,
        };
        simulation.publish(&Operation::FirstFile);
        simulation.switch_to_a_new_instance();
        assert_eq!(simulation.version.instance, MOUNT_INSTANCE, "可写挂载取实例 2");
        simulation.publish(&Operation::RowPublish);
        simulation.publish(&Operation::WarmUp);
        simulation
    }

    pub(super) const fn has_stopped(&self) -> bool {
        self.does_not_fit_at.is_some()
    }

    /// 换实例代号：实例表给旧实例写一行 (实例, 已施加的最大事务号)，新实例的事务号从 1 起。
    fn switch_to_a_new_instance(&mut self) {
        self.instance_rows.insert(self.version.instance, self.version.transaction);
        self.version.instance += 1;
        self.version.transaction = 0;
    }

    pub(super) fn publish(&mut self, operation: &Operation) -> bool {
        if self.has_stopped() {
            return false;
        }
        match self.prepare(operation) {
            Ok(prepared) => {
                self.adopt(prepared, matches!(operation, Operation::RollBack { .. }));
                true
            }
            Err(LeafContainersOverflowTheRoot) => {
                self.does_not_fit_at = Some(self.version.txg + 1);
                false
            }
        }
    }

    /// 算下一版与这次要写的单元（不落盘）。一次发布一个事务号（D8 已定项 16 第 2 条）。
    fn prepare(&self, operation: &Operation) -> Result<PreparedPublish, LeafContainersOverflowTheRoot> {
        let mut next = self.version.clone();
        next.txg = self.version.txg + 1;
        next.transaction = self.version.transaction + 1;
        let txg = next.txg;
        let mut work = PublishWork {
            container_death: self.container_death,
            stubbed_identities: Vec::new(),
            first_new_unit: self.disk.units.len(),
            write_order: WriteOrder { birth_txg: txg, instance: next.instance, transaction: next.transaction },
            publish_sequence: self.publish_sequence + 1,
            new_units: Vec::new(),
            leaves: working_leaves_of(&next.leaves),
            leaf_set_changed: false,
            inode_root_rewritten_anyway: false,
            retired_identities: Vec::new(),
            moved_objects: Vec::new(),
            tombstone_appends: Vec::new(),
            merge: MergeAction::NoMerge,
            split_checks: Vec::new(),
            written: UnitsWritten::default(),
            shrink: None,
        };
        let leaf_capacity = self.cell.leaf_capacity;
        match operation {
            Operation::FirstFile => {
                let data_unit = work.reserve(UnitBody::Data { object: FIRST_FILE_INODE, object_birth: txg, unit_index_in_file: 0 });
                work.written.data_units += 1;
                next.files.insert(FIRST_FILE_INODE, FileUnits { object_birth: txg, units: vec![data_unit] });
                let record = InodeRecord {
                    inode: FIRST_FILE_INODE,
                    object_birth: txg,
                    nlink: 1,
                    size_bytes: NEW_POOL_FILE_SIZE_BYTES,
                    change_counter: txg,
                };
                work.insert_new_records(&[record], leaf_capacity);
                next.next_inode = FIRST_FILE_INODE + 1;
            }
            Operation::RowPublish | Operation::WarmUp => {}
            Operation::WriteFile { object, data_units } => {
                let record = next.record_of(*object).expect("写文件内容的对象有一条活记录");
                let data_units_before = next.data_units_of(*object);
                let units: Vec<UnitIndex> = (0..*data_units)
                    .map(|unit_index_in_file| {
                        work.reserve(UnitBody::Data { object: *object, object_birth: record.object_birth, unit_index_in_file })
                    })
                    .collect();
                work.written.data_units += *data_units;
                next.files.insert(*object, FileUnits { object_birth: record.object_birth, units });
                let size_after = *data_units * DATA_UNIT_PAYLOAD_BYTES;
                work.update_record(*object, |updated| {
                    updated.size_bytes = size_after;
                    updated.change_counter = txg;
                });
                if *data_units < data_units_before {
                    work.shrink = Some(ShrinkEvent { object: *object, size_before_bytes: record.size_bytes });
                    for stream in TOMBSTONE_STREAMS {
                        if stream.writes_ranges_when_a_user_change_shrinks() {
                            let range = TombstoneRecord {
                                object: *object,
                                object_birth: record.object_birth,
                                range_start_bytes: size_after,
                                death_txg: txg,
                            };
                            work.tombstone_appends.push((stream, range));
                        }
                    }
                }
            }
            Operation::NewInodes { count } => {
                let records: Vec<InodeRecord> = (0..*count)
                    .map(|offset| InodeRecord { inode: next.next_inode + offset, object_birth: txg, nlink: 1, size_bytes: 0, change_counter: txg })
                    .collect();
                work.insert_new_records(&records, leaf_capacity);
                next.next_inode += *count;
            }
            Operation::UnlinkIntent { object } => {
                let record = next.record_of(*object).expect("意图删的 inode 有一条记录");
                assert!(record.nlink > 0, "意图删的 inode 还活着");
                work.update_record(*object, |updated| {
                    updated.nlink = 0;
                    updated.change_counter = txg;
                });
                assert!(next.deleted_queue.is_none(), "任何一版里 deleted_inodes 至多一条 key");
                next.deleted_queue = Some(*object);
                next.objects_with_published_intent.insert(*object);
                for stream in TOMBSTONE_STREAMS {
                    if stream.writes_the_whole_object_tombstone_at_the_intent() {
                        let whole = TombstoneRecord {
                            object: *object,
                            object_birth: record.object_birth,
                            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
                            death_txg: txg,
                        };
                        work.tombstone_appends.push((stream, whole));
                    }
                }
            }
            Operation::UnlinkBatch { object, data_units_kept } => {
                let record = next.record_of(*object).expect("分批删的 inode 有一条记录");
                let file = next.files.get_mut(object).expect("分批删的对象有数据单元");
                assert!(
                    *data_units_kept >= 1 && u64_as_index(*data_units_kept) < file.units.len(),
                    "一批留下至少一个、至少截掉一个"
                );
                file.units.truncate(u64_as_index(*data_units_kept));
                let size_after = *data_units_kept * DATA_UNIT_PAYLOAD_BYTES;
                work.update_record(*object, |updated| {
                    updated.size_bytes = size_after;
                    updated.change_counter = txg;
                });
                for stream in TOMBSTONE_STREAMS {
                    if stream.writes_ranges_at_every_batch_that_is_not_the_last() {
                        let range = TombstoneRecord {
                            object: *object,
                            object_birth: record.object_birth,
                            range_start_bytes: size_after,
                            death_txg: txg,
                        };
                        work.tombstone_appends.push((stream, range));
                    }
                }
            }
            Operation::UnlinkFinal { object } => {
                let record = next.record_of(*object).expect("最后一批删的 inode 有一条记录");
                next.files.remove(object);
                work.merge = work.remove_record_and_merge(*object, self.cell);
                assert_eq!(next.deleted_queue, Some(*object), "最后一批删的是队列里那一个");
                next.deleted_queue = None;
                for stream in TOMBSTONE_STREAMS {
                    if !stream.writes_the_whole_object_tombstone_at_the_intent() {
                        let whole = TombstoneRecord {
                            object: *object,
                            object_birth: record.object_birth,
                            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
                            death_txg: txg,
                        };
                        work.tombstone_appends.push((stream, whole));
                    }
                }
            }
            Operation::RollBack { target } => {
                next.leaves = target.leaves.clone();
                next.files = target.files.clone();
                next.deleted_queue = target.deleted_queue;
                next.objects_with_published_intent = target.objects_with_published_intent.clone();
                next.next_inode = self.version.next_inode.max(target.next_inode);
                next.lineage.push((txg, target.txg));
                work.leaves = working_leaves_of(&next.leaves);
                work.inode_root_rewritten_anyway = true;
            }
        }
        let leaves = work.finish_leaves();
        // 根装不下按活叶数判；B-stub 写路径多出来的桩不按 135 停（修订 2 之五「历史与写路径格」）。
        let live_leaves = leaves.iter().filter(|leaf| !leaf.records.is_empty()).count();
        if length_as_u64(live_leaves) > ROOT_NODE_LEAF_CONTAINERS {
            return Err(LeafContainersOverflowTheRoot);
        }
        next.leaves = leaves;
        self.append_the_death_records(&mut next, &mut work, txg);
        Ok(PreparedPublish {
            next,
            new_units: work.new_units,
            written: work.written,
            merge: work.merge,
            retired_identities: work.retired_identities,
            stubbed_identities: work.stubbed_identities,
            moved_objects: work.moved_objects,
            split_checks: work.split_checks,
            shrink: work.shrink,
        })
    }

    /// 墓碑（五条写流各一棵树）与退役记录（身份消失那次追加，登记第一节读法 9）。
    fn append_the_death_records(&self, next: &mut Version, work: &mut PublishWork, txg: u64) {
        for stream in TOMBSTONE_STREAMS {
            let appended: Vec<TombstoneRecord> = work
                .tombstone_appends
                .iter()
                .filter(|(appended_stream, _)| *appended_stream == stream)
                .map(|(_, record)| *record)
                .collect();
            let containers = work.append_into_packed_tree(
                &mut next.tombstone_trees[stream.position()],
                &appended,
                TOMBSTONE_CONTAINER_RECORDS,
                (TREE_IDENTIFIER_TOMBSTONE, PACKED_TYPE_TOMBSTONE),
                &|identity, records| UnitBody::Tombstones { stream, identity, records },
            );
            work.written.tombstone_containers[stream.position()] += containers;
            if containers > 0 {
                work.written.tombstone_roots[stream.position()] += 1;
            }
        }
        let retirements: Vec<RetirementRecord> = work
            .retired_identities
            .iter()
            .map(|identity| RetirementRecord { identity: *identity, death_txg: txg })
            .collect();
        let containers = work.append_into_packed_tree(
            &mut next.retirement_tree,
            &retirements,
            RETIREMENT_CONTAINER_RECORDS,
            (TREE_IDENTIFIER_RETIREMENT_IN_THIS_MODEL, PACKED_TYPE_RETIREMENT),
            &|identity, records| UnitBody::Retirements { identity, records },
        );
        work.written.retirement_containers += containers;
        work.written.retirement_records += length_as_u64(retirements.len());
        if containers > 0 {
            work.written.retirement_roots += 1;
        }
        assert!(self.cell.leaf_capacity >= 1, "叶容器容量至少 1");
    }

    /// 落盘、换版：上一版引用而这一版不再引用的单元记在这次换下；这一版引用的单元都算被引用（回退重新引用 R_old 的）。
    fn adopt(&mut self, prepared: PreparedPublish, is_a_rollback: bool) {
        let publish_sequence = self.publish_sequence + 1;
        for unit in prepared.new_units {
            self.assertions.judge("units_carry_the_publish_that_wrote_them", unit.written_at_publish == publish_sequence);
            self.disk.write(unit);
        }
        self.publish_sequence = publish_sequence;
        let before = self.version.referenced_units();
        let after = prepared.next.referenced_units();
        let still_referenced: HashSet<UnitIndex> = after.iter().copied().collect();
        for unit_index in before {
            if !still_referenced.contains(&unit_index) {
                self.disk.units[unit_index].replaced_at_publish = Some(publish_sequence);
            }
        }
        for unit_index in after {
            self.disk.units[unit_index].replaced_at_publish = None;
        }
        let moved_object_shrunk = prepared.shrink.is_some_and(|shrink| self.moved_objects.contains(&shrink.object));
        if let Some(event) = self.gap_watch {
            if self.gap_start.is_none() && event.happened_in(prepared.merge, moved_object_shrunk) {
                self.gap_start = Some(GapStart { version_before: self.version.clone(), event_publish_sequence: publish_sequence });
            }
        }
        if let Some(shrink) = prepared.shrink {
            let largest = self.largest_size_before_a_shrink.entry(shrink.object).or_insert(0);
            *largest = (*largest).max(shrink.size_before_bytes);
            if moved_object_shrunk {
                self.moved_then_shortened += 1;
                self.moved_then_shortened_objects.insert(shrink.object);
            }
        }
        self.moved_objects.extend(prepared.moved_objects.iter().copied());
        for holds in &prepared.split_checks {
            self.assertions.judge("K6_split_right_half_numbered_after_the_record_left_half_unchanged", *holds);
        }
        let queue_before = self.version.deleted_queue;
        let finished_unlink = match (queue_before, prepared.next.deleted_queue, is_a_rollback) {
            (Some(object), None, false) => Some(object),
            (Some(_) | None, _, _) => None,
        };
        if let Some(object) = finished_unlink {
            self.unlinks_completed += 1;
            self.note_dead_versions_holding_the_finished_record(object, &prepared.retired_identities, &prepared.stubbed_identities);
        }
        self.version = prepared.next;
        self.publishes += 1;
        self.self_check.note(&self.version.shape());
        self.publish_records.push(PublishRecord {
            publish_sequence,
            txg: self.version.txg,
            merge: prepared.merge,
            retired_identities: prepared.retired_identities,
            written: prepared.written,
            shrink: prepared.shrink,
            moved_object_shrunk,
            leaf_digest: self.version.leaf_digest(),
            is_a_rollback,
            extent_digest: self.version.extent_digest(),
            stubs_written: length_as_u64(prepared.stubbed_identities.len()),
            leaves_in_the_leaf_order: length_as_u64(self.version.leaves.len()),
            stubs_in_the_leaf_order: self.version.stub_count(),
            holds_an_empty_inode_leaf: self.version.leaves.iter().map(|leaf| leaf.records.len()).any(|record_count| record_count == 0),
            unlinks_completed_so_far: self.unlinks_completed,
        });
        if let (None, Some(object), false) = (queue_before, self.version.deleted_queue, is_a_rollback) {
            self.intent_txgs.insert(object, self.version.txg);
        }
        if self.scanning {
            let record = self.scan_the_current_version(publish_sequence);
            self.scans.push(record);
            self.run_the_planned_positive_controls(publish_sequence);
            if self.segment_four.is_some() {
                self.run_the_segment_four_probes(publish_sequence);
            }
        }
    }

    /// 剩下 > b ⇒ 留 剩下 − b；剩下 ≤ b ⇒ 最后一批（D8 已定项 16 第 2 条，`crates/singlefs-core/src/unlink.rs:118-130`）。
    pub(super) const fn unlink_step_for(&self, object: u64, data_units_remaining: u64) -> Operation {
        if data_units_remaining <= self.batch_data_units {
            Operation::UnlinkFinal { object }
        } else {
            Operation::UnlinkBatch { object, data_units_kept: data_units_remaining - self.batch_data_units }
        }
    }

    pub(super) fn next_unlink_step(&self, object: u64) -> Operation {
        self.unlink_step_for(object, self.version.data_units_of(object))
    }

    /// 把删除队列走空（会话里每次改动先做这一步，D8 已定项 16 第 6 条）。
    pub(super) fn walk_the_deletion_queue_empty(&mut self) {
        while let Some(object) = self.version.deleted_queue {
            let step = self.next_unlink_step(object);
            if !self.publish(&step) {
                return;
            }
        }
    }

    /// 一次用户改动经会话：`crates/` 会拒的照拒、记一次、不发布（E177 装置 `apply`）。
    pub(super) fn apply(&mut self, change: UserChange) {
        if self.has_stopped() {
            return;
        }
        self.walk_the_deletion_queue_empty();
        self.user_changes.push(change);
        match change {
            UserChange::WriteFile { object, data_units } => {
                if self.version.is_live(object) {
                    self.publish(&Operation::WriteFile { object, data_units });
                } else {
                    self.refused_steps += 1;
                }
            }
            UserChange::NewInodes { count } => {
                self.publish(&Operation::NewInodes { count });
            }
            UserChange::Unlink { object } => {
                if self.version.is_with_trees_and_no_file() || !self.version.is_live(object) {
                    self.refused_steps += 1;
                    return;
                }
                if self.moved_objects.contains(&object) {
                    self.moved_then_unlinked += 1;
                }
                if self.publish(&Operation::UnlinkIntent { object }) {
                    self.walk_the_deletion_queue_empty();
                }
            }
        }
    }

    pub(super) fn roll_back_to(&mut self, target: Version) -> bool {
        self.publish(&Operation::RollBack { target: Box::new(target) })
    }

    /// 意图到最后一批之间崩溃（登记第一节读法 5）：下一步（一批或最后一批）的单元全部写上了盘、它的根没写。
    /// 这次写在崩溃那一刻算换下；现行那一版不动。交回崩溃那一刻的发布序号。
    pub(super) fn crash_with_the_next_unlink_step_written(&mut self) -> Option<u64> {
        let object = self.version.deleted_queue?;
        let step = self.next_unlink_step(object);
        let prepared = self.prepare(&step).ok()?;
        let crash_publish = self.publish_sequence + 1;
        for unit in prepared.new_units {
            let mut abandoned = unit;
            abandoned.replaced_at_publish = Some(crash_publish);
            self.disk.write(abandoned);
        }
        self.publish_sequence = crash_publish;
        Some(crash_publish)
    }

    /// 恢复：回到崩溃时现行那一版，换一个实例代号（旧实例的已施加事务号 = 那一版的事务号），照挂载先发写行、暖机，txg 重发。
    pub(super) fn recover_after_the_crash(&mut self) {
        self.switch_to_a_new_instance();
        self.publish(&Operation::RowPublish);
        self.publish(&Operation::WarmUp);
    }

    /// F10 的覆盖写、顺序写取哪个对象：「记录被借走或被吸收过、单元多于 1」里号最小的活对象，没有就取单元多于 1 的号最小的活对象。
    pub(super) fn rewrite_target(&self) -> Option<u64> {
        let candidates: Vec<u64> = self
            .version
            .live_inodes()
            .into_iter()
            .filter(|inode| self.version.data_units_of(*inode) > 1)
            .collect();
        candidates
            .iter()
            .copied()
            .find(|inode| self.moved_objects.contains(inode))
            .or_else(|| candidates.first().copied())
    }

    pub(super) fn count_merges(&self) -> (u64, u64, u64) {
        let mut borrows = 0;
        let mut absorbs = 0;
        let mut drops = 0;
        for record in &self.publish_records {
            match record.merge {
                MergeAction::BorrowedFromTheRightNeighbour { .. } => borrows += 1,
                MergeAction::AbsorbedTheRightNeighbour | MergeAction::AbsorbedIntoTheLeftNeighbour => absorbs += 1,
                MergeAction::EmptiedAndDropped => drops += 1,
                MergeAction::NoMerge => {}
            }
        }
        (borrows, absorbs, drops)
    }

    pub(super) fn total_written(&self) -> UnitsWritten {
        let mut total = UnitsWritten::default();
        for record in &self.publish_records {
            let written = record.written;
            total.data_units += written.data_units;
            total.inode_leaf_containers += written.inode_leaf_containers;
            total.inode_roots += written.inode_roots;
            total.retirement_containers += written.retirement_containers;
            total.retirement_roots += written.retirement_roots;
            total.retirement_records += written.retirement_records;
            for stream in TOMBSTONE_STREAMS {
                total.tombstone_containers[stream.position()] += written.tombstone_containers[stream.position()];
                total.tombstone_roots[stream.position()] += written.tombstone_roots[stream.position()];
            }
        }
        total
    }
}
