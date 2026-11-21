//! 一次模拟：用户改动、断言计数、发布计数与整个模拟器的状态和步进。

use super::*;

// ---------------------------------------------------------------------------------------------
// 一段历史在一个几何、一条写路径上的模拟
// ---------------------------------------------------------------------------------------------

/// 一次用户改动（S12，`mounted_session.rs:54-68`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UserChange {
    Overwrite,
    SequentialWrite { data_units: u64 },
    NewInodes { count: u64 },
    Unlink { inode: u64 },
}

/// 断言的计数：每条断言判了几次、违反几次（第七节、修订 1 第 12.6 节）。
#[derive(Clone, Debug, Default)]
pub(super) struct AssertionTally {
    pub(super) judged: BTreeMap<&'static str, u64>,
    violated: BTreeMap<&'static str, u64>,
}

impl AssertionTally {
    pub(super) fn judge(&mut self, name: &'static str, holds: bool) {
        *self.judged.entry(name).or_insert(0) += 1;
        if !holds {
            *self.violated.entry(name).or_insert(0) += 1;
        }
    }

    pub(super) fn merge(&mut self, other: &AssertionTally) {
        for (name, count) in &other.judged {
            *self.judged.entry(name).or_insert(0) += count;
        }
        for (name, count) in &other.violated {
            *self.violated.entry(name).or_insert(0) += count;
        }
    }

    pub(super) fn violations_of(&self, name: &str) -> u64 {
        self.violated.get(name).copied().unwrap_or(0)
    }
}

/// 一次 unlink 发了几次（意图加各批），A5、A10 用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UnlinkPublishCount {
    pub(super) inode: u64,
    pub(super) data_units_at_the_intent: u64,
    pub(super) publishes: u64,
}

pub(super) struct Simulation {
    pub(super) geometry: Geometry,
    pub(super) write_path: WritePath,
    pub(super) batch_data_units: u64,
    pub(super) allocator: Allocator,
    pub(super) disk: Disk,
    pub(super) version: Version,
    roots_on_disk: BTreeMap<(u64, u64), u64>,
    pub(super) truth: Vec<BTreeSet<KeyFields>>,
    pub(super) ever_seen: Vec<BTreeSet<KeyFields>>,
    pub(super) death_time: HashMap<NamespacedKey, u64>,
    pub(super) unit_lists: UnitLists,
    unit_scan_readings: Vec<UnitScanReading>,
    pub(super) trajectories: BTreeMap<(Arm, Reader, ReportedTree), Trajectory>,
    pub(super) publishes: u64,
    pub(super) scans: u64,
    pub(super) refused_steps: u64,
    pub(super) occupancy_peak: u64,
    pub(super) live_slot_peak: u64,
    /// 一次发布取落点时占着的槽：这一版的活槽加这次换下的（执行员修订 2 C17）。
    pub(super) slots_held_during_a_publish_peak: u64,
    pub(super) exceeded_the_premise_at: Option<u64>,
    pub(super) does_not_fit_at: Option<u64>,
    pub(super) placement_log: Vec<Vec<(u64, u64)>>,
    pub(super) scan_log: Vec<Vec<u64>>,
    pub(super) tree_kinds_with_units: BTreeSet<TreeKind>,
    pub(super) assertions: AssertionTally,
    pub(super) unlink_publish_counts: Vec<UnlinkPublishCount>,
    pub(super) reuse_at_exactly_the_ring_length: Vec<(u64, u64)>,
    cross_checks_every_scan: bool,
    /// 每次发布记根时回收的槽数，按 txg（A14 (ii)）。
    pub(super) slots_reclaimed_by_publish: BTreeMap<u64, u64>,
}

impl Simulation {
    /// mkfs、mkfs 那个进程里的两次零单元暖机与新池新建文件（txg 3）、可写挂载的写行（txg 4）与暖机（txg 5）（12.4.1、M1–M5）。
    pub(super) fn start(
        geometry: Geometry,
        write_path: WritePath,
        batch_data_units: u64,
        tight_unit_area_slots: Option<u64>,
        cross_checks_every_scan: bool,
    ) -> Self {
        let unit_area_slots = match geometry {
            Geometry::Ring => Some(RING_GEOMETRY_UNIT_AREA_SLOTS),
            Geometry::Wide => None,
            Geometry::Tight => Some(tight_unit_area_slots.expect("G-紧 的单元区槽数由 G-宽 的峰值定")),
        };
        let mut allocator = Allocator::new(geometry, unit_area_slots);
        let mut disk = Disk::default();
        let make_filesystem_unit = |owner: UnitOwner, class: u64, slot: u64, span_slots: u64| Unit {
            owner,
            class,
            group: None,
            write_order: (0, 0, 0),
            slot,
            span_slots,
            release_txg: None,
            is_readable: true,
            mapping_key: None,
            contents: UnitContents::WithoutKeys,
        };
        disk.write(make_filesystem_unit(UnitOwner::InstanceTable, UNIT_CLASS_PACKED, INSTANCE_TABLE_GENESIS_SLOT, 2));
        disk.write(make_filesystem_unit(UnitOwner::TreeTable, UNIT_CLASS_INDEX_NODE, TREE_TABLE_GENESIS_SLOT, 1));
        allocator.record(INSTANCE_TABLE_GENESIS_SLOT, 2, 0, 0);
        allocator.record(TREE_TABLE_GENESIS_SLOT, 1, 0, 1);
        let mut roots_on_disk = BTreeMap::new();
        for region in 0..ROOT_RING_REGIONS {
            roots_on_disk.insert((region, 0), 0);
        }
        if geometry == Geometry::Ring {
            allocator.root_ring = Some(RootRingTable::after_make_filesystem());
        }
        for zero_unit_txg in 1..NEW_POOL_FILE_CREATION_TXG {
            roots_on_disk.insert(root_ring_slot_of(zero_unit_txg), zero_unit_txg);
            allocator.record_root(zero_unit_txg);
        }
        let version = Version {
            txg: NEW_POOL_FILE_CREATION_TXG - 1,
            instance: MAKE_FILESYSTEM_PROCESS_INSTANCE,
            highest_transaction: 0,
            next_inode: FIRST_FILE_INODE,
            file_units: Vec::new(),
            extent_root: None,
            extent_lower: BTreeMap::new(),
            inode_containers: Vec::new(),
            inode_root: None,
            deleted_queue: Vec::new(),
            deleted_root: None,
            tombstone_containers: Vec::new(),
            tombstone_root: None,
            allocation_nodes: BTreeMap::new(),
            accounting: CodeTwoTreeVersion::default(),
            mapping: CodeTwoTreeVersion::default(),
            tree_table: 1,
            instance_table: 0,
        };
        let readings = write_path
            .arms()
            .into_iter()
            .map(|arm| UnitScanReading::new(arm, geometry == Geometry::Ring))
            .collect();
        let mut simulation = Self {
            geometry,
            write_path,
            batch_data_units,
            allocator,
            disk,
            version,
            roots_on_disk,
            truth: vec![BTreeSet::new(); NAMESPACE_COUNT],
            ever_seen: vec![BTreeSet::new(); NAMESPACE_COUNT],
            death_time: HashMap::new(),
            unit_lists: UnitLists::default(),
            unit_scan_readings: readings,
            trajectories: BTreeMap::new(),
            publishes: 0,
            scans: 0,
            refused_steps: 0,
            occupancy_peak: 0,
            live_slot_peak: 0,
            slots_held_during_a_publish_peak: 0,
            exceeded_the_premise_at: None,
            does_not_fit_at: None,
            placement_log: Vec::new(),
            scan_log: Vec::new(),
            tree_kinds_with_units: BTreeSet::new(),
            assertions: AssertionTally::default(),
            unlink_publish_counts: Vec::new(),
            reuse_at_exactly_the_ring_length: Vec::new(),
            cross_checks_every_scan,
            slots_reclaimed_by_publish: BTreeMap::new(),
        };
        simulation.publish_or_stop(PublishOperation::FirstFile);
        let latest_txg = simulation.version.txg;
        let roots_at_mount = simulation.roots_on_disk.clone();
        simulation.allocator.reset_at_mount(&roots_at_mount, latest_txg);
        simulation.publish_or_stop(PublishOperation::RowPublish);
        simulation.publish_or_stop(PublishOperation::WarmUp);
        simulation
    }

    pub(super) fn has_stopped(&self) -> bool {
        self.does_not_fit_at.is_some()
    }

    fn publish_or_stop(&mut self, operation: PublishOperation) {
        if self.has_stopped() {
            return;
        }
        if self.publish(operation).is_err() {
            self.does_not_fit_at = Some(self.version.txg + 1);
        }
    }

    /// 一次发布：先释放这次换下的全部落点，再按 bump 次序取落点，写单元，再记根（M6）；之后扫一遍。
    fn publish(&mut self, operation: PublishOperation) -> Result<(), PlacementRefused> {
        let txg = self.version.txg + 1;
        let plan = plan_logical(&self.version, operation, txg, self.write_path, NEW_POOL_FILE_CREATION_TXG);
        let assembled = settle_publish(&plan, &self.version, &self.disk, &self.allocator)?;
        let allocator_before = self.allocator.clone();
        for unit_identifier in &assembled.releases {
            let (slot, span_slots) = (self.disk.units[*unit_identifier].slot, self.disk.units[*unit_identifier].span_slots);
            self.allocator.release(slot, span_slots, txg);
            self.disk.units[*unit_identifier].release_txg = Some(txg);
        }
        let first_new_unit = self.disk.units.len();
        let mut placements = Vec::new();
        let mut unit_of_role: HashMap<Role, UnitIdentifier> = HashMap::new();
        for (offset, role) in assembled.roles.iter().enumerate() {
            let unit_identifier = first_new_unit + offset;
            let slot = self
                .allocator
                .allocate(role.placement_rule(), txg, unit_identifier)
                .expect("固定点在拷贝上取得到的落点，真发时同一串分配照样取得到");
            placements.push((slot, role.placement_rule().span_slots()));
            unit_of_role.insert(*role, unit_identifier);
        }
        self.judge_cluster_segments_of_this_publish(&allocator_before, &assembled.roles, &placements);
        let new_units = self.build_units(&plan, &assembled, &placements);
        let mut made_unreadable = Vec::new();
        for unit in new_units {
            for slot in unit.slot..unit.slot + unit.span_slots {
                self.judge_reuse(slot, txg);
            }
            if let Some(kind) = unit.owner.tree_kind() {
                self.tree_kinds_with_units.insert(kind);
            }
            made_unreadable.extend(self.disk.write(unit));
        }
        let reclaimed_placements = self.allocator.record_root(txg);
        self.slots_reclaimed_by_publish
            .insert(txg, reclaimed_placements.iter().map(|(_, span)| *span).sum());
        self.roots_on_disk.insert(root_ring_slot_of(txg), txg);
        self.assertions.judge("reclaim_reports_whole_placements", reclaimed_placements.iter().all(|(_, span)| *span >= 1));
        self.version = self.build_version(&plan, &assembled, &unit_of_role);
        self.note_unit_lists(first_new_unit);
        if let Some(record) = plan.tombstones_written.first() {
            let tombstone_holds_the_intent_txg = record.death_txg == txg;
            self.assertions.judge("K3_tombstone_death_txg_is_the_intent_txg", tombstone_holds_the_intent_txg);
        }
        self.placement_log.push(placements);
        self.publishes += 1;
        let written: Vec<UnitIdentifier> = (first_new_unit..self.disk.units.len()).collect();
        self.scan(txg, &assembled.releases, &written, &made_unreadable, &plan.tombstones_written);
        Ok(())
    }

    fn note_unit_lists(&mut self, first_new_unit: UnitIdentifier) {
        for unit_identifier in first_new_unit..self.disk.units.len() {
            let unit = &self.disk.units[unit_identifier];
            let is_an_accounting_leaf = matches!(unit.contents, UnitContents::AccountingKeys(_));
            match unit.owner {
                UnitOwner::FileData => self.unit_lists.data_units.push(unit_identifier),
                UnitOwner::InodeContainer => self.unit_lists.inode_containers.push(unit_identifier),
                UnitOwner::TombstoneContainer => self.unit_lists.tombstone_containers.push(unit_identifier),
                UnitOwner::AccountingNode => {
                    if is_an_accounting_leaf {
                        self.unit_lists.accounting_leaves.push(unit_identifier);
                    }
                }
                UnitOwner::ExtentLowerNode
                | UnitOwner::ExtentRoot
                | UnitOwner::InodeRoot
                | UnitOwner::AllocationNode
                | UnitOwner::MappingNode
                | UnitOwner::DeletedInodesRoot
                | UnitOwner::TombstoneRoot
                | UnitOwner::TreeTable
                | UnitOwner::InstanceTable => {}
            }
        }
    }

    /// A15：G-环 每一次再分配（第 t 次发布写进的槽，上一回住在那里的单元释放代 s）都满足 t ≥ s + 24；等号那几格记下（A14、A15）。
    fn judge_reuse(&mut self, slot: u64, txg: u64) {
        let Some(previous_occupant) = self.disk.occupant_of_slot.get(&slot).copied() else {
            return;
        };
        let released_at = self.disk.units[previous_occupant].release_txg;
        match self.geometry {
            Geometry::Ring => {
                let holds = released_at.is_some_and(|released| txg >= released + RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION);
                self.assertions.judge("A15_reuse_waits_a_ring_length", holds);
                if released_at.is_some_and(|released| txg == released + RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION) {
                    self.reuse_at_exactly_the_ring_length.push((txg, slot));
                }
            }
            Geometry::Tight => {
                let holds = released_at.is_some_and(|released| released < txg);
                self.assertions.judge("tight_reuse_only_of_released_before_this_publish", holds);
            }
            Geometry::Wide => self.assertions.judge("wide_never_reuses", false),
        }
    }

    /// A18：用户数据落点不落在这次挂载开过的任何聚簇段里；每次开新段，那一段在这次发布开始时全空。另写一份按落点逐个核，不调分配函数。
    fn judge_cluster_segments_of_this_publish(&mut self, before: &Allocator, roles: &[Role], placements: &[(u64, u64)]) {
        if self.geometry == Geometry::Tight {
            return;
        }
        for (role, (slot, _)) in roles.iter().zip(placements) {
            if let Role::Data(_) = role {
                let segment = UNIT_AREA_START_SLOT
                    + ((slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS) * CLUSTER_SEGMENT_SLOTS;
                let in_a_cluster_segment = self.allocator.cluster_segments.contains(&segment);
                self.assertions.judge("A18_user_data_outside_cluster_segments", !in_a_cluster_segment);
            }
        }
        let opened: Vec<u64> = self
            .allocator
            .cluster_segments
            .difference(&before.cluster_segments)
            .copied()
            .collect();
        for segment in opened {
            let was_empty = (segment..segment + CLUSTER_SEGMENT_SLOTS).all(|slot| before.is_free(slot));
            self.assertions.judge("A18_opened_segment_was_empty", was_empty);
        }
    }

    fn data_unit_birth(&self, plan: &LogicalPlan, position: usize) -> u64 {
        match plan.data_units[position] {
            PlannedUnit::Rewritten => plan.txg,
            PlannedUnit::Carried(unit) => self.disk.units[unit].birth_txg(),
        }
    }

    /// 这次写的每个单元（落点先于内容：分配记录树的叶装的是取完落点之后的记录，M6）。
    fn build_units(&self, plan: &LogicalPlan, assembled: &AssembledPublish, placements: &[(u64, u64)]) -> Vec<Unit> {
        let data_units_after = length_as_u64(plan.data_units.len());
        let mut units = Vec::new();
        for (role, (slot, span_slots)) in assembled.roles.iter().zip(placements) {
            let mapping_key = assembled.mapping_keys_of_rewritten.get(role).copied();
            let mut write_order = (plan.txg, plan.instance, plan.transaction_base);
            let mut group = None;
            let (owner, class, contents) = match role {
                Role::Data(position) => {
                    let anchor_unit = length_as_u64(*position);
                    write_order.2 = plan.transaction_base + anchor_unit;
                    group = Some(SelectionGroup::DataUnit {
                        birth_tree: TREE_IDENTIFIER_EXTENT,
                        object: FIRST_FILE_INODE,
                        object_birth: plan.file_object_birth,
                        anchor_unit,
                    });
                    (
                        UnitOwner::FileData,
                        UNIT_CLASS_DATA,
                        UnitContents::DataUnit {
                            object: FIRST_FILE_INODE,
                            object_birth: plan.file_object_birth,
                            anchor_unit,
                        },
                    )
                }
                Role::InstanceTable => (UnitOwner::InstanceTable, UNIT_CLASS_PACKED, UnitContents::WithoutKeys),
                Role::ExtentLower((level, index)) => {
                    let contents = if *level == 0 {
                        let first = index * EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
                        let last_exclusive = ((index + 1) * EXTENT_TREE_LOWER_LEAF_DATA_UNITS).min(data_units_after);
                        UnitContents::ExtentRecords(
                            (first..last_exclusive)
                                .map(|data_unit| ExtentRecordSummary {
                                    inode: FIRST_FILE_INODE,
                                    data_unit,
                                    pointer_birth: self.data_unit_birth(plan, u64_as_index(data_unit)),
                                })
                                .collect(),
                        )
                    } else {
                        UnitContents::WithoutKeys
                    };
                    (UnitOwner::ExtentLowerNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::ExtentRoot => {
                    let inline = if data_units_after == 1 {
                        vec![ExtentRecordSummary {
                            inode: FIRST_FILE_INODE,
                            data_unit: 0,
                            pointer_birth: self.data_unit_birth(plan, 0),
                        }]
                    } else {
                        Vec::new()
                    };
                    (UnitOwner::ExtentRoot, UNIT_CLASS_INDEX_NODE, UnitContents::ExtentRecords(inline))
                }
                Role::InodeContainer(position) => {
                    let container = &plan.inode_containers[*position];
                    group = Some(SelectionGroup::PackedContainer {
                        birth_tree: TREE_IDENTIFIER_INODE,
                        record_type: PACKED_TYPE_INODE,
                        container_number: container.container_number,
                        container_birth: container.container_birth,
                    });
                    (
                        UnitOwner::InodeContainer,
                        UNIT_CLASS_PACKED,
                        UnitContents::InodeRecords(container.records.clone()),
                    )
                }
                Role::InodeRoot => (UnitOwner::InodeRoot, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
                Role::AllocationNode(node) => {
                    let contents = match node {
                        AllocationNode::BelowTheRoot {
                            level: 0,
                            index_in_device,
                            ..
                        } => {
                            let first_slot = index_in_device * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                            UnitContents::AllocationRecords(
                                self.allocator
                                    .records
                                    .range(first_slot..first_slot + ALLOCATION_RECORD_TREE_LEAF_SLOTS)
                                    .map(|(record_slot, record)| (*record_slot, record.is_released))
                                    .collect(),
                            )
                        }
                        AllocationNode::BelowTheRoot { .. } | AllocationNode::Root => UnitContents::WithoutKeys,
                    };
                    (UnitOwner::AllocationNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::AccountingNode(position) => {
                    let contents = match assembled.accounting_plan.shape.contents_at(*position) {
                        CodeTwoNodeContents::Leaf { keys } => UnitContents::AccountingKeys(keys.clone()),
                        CodeTwoNodeContents::Internal { .. } => UnitContents::WithoutKeys,
                    };
                    (UnitOwner::AccountingNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::DeletedInodesRoot => (
                    UnitOwner::DeletedInodesRoot,
                    UNIT_CLASS_INDEX_NODE,
                    UnitContents::DeletedInodes(plan.deleted_queue.clone()),
                ),
                Role::TombstoneContainer(position) => {
                    let container = &plan.tombstone_containers[*position];
                    group = Some(SelectionGroup::PackedContainer {
                        birth_tree: TREE_IDENTIFIER_TOMBSTONE,
                        record_type: PACKED_TYPE_TOMBSTONE,
                        container_number: container.container_number,
                        container_birth: container.container_birth,
                    });
                    (
                        UnitOwner::TombstoneContainer,
                        UNIT_CLASS_PACKED,
                        UnitContents::TombstoneRecords(container.records.clone()),
                    )
                }
                Role::TombstoneRoot => (UnitOwner::TombstoneRoot, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
                Role::MappingNode(position) => {
                    let contents = match assembled.mapping_plan.shape.contents_at(*position) {
                        CodeTwoNodeContents::Leaf { keys } => UnitContents::MappingKeys(keys.clone()),
                        CodeTwoNodeContents::Internal { .. } => UnitContents::WithoutKeys,
                    };
                    (UnitOwner::MappingNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::TreeTable => (UnitOwner::TreeTable, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
            };
            units.push(Unit {
                owner,
                class,
                group,
                write_order,
                slot: *slot,
                span_slots: *span_slots,
                release_txg: None,
                is_readable: true,
                mapping_key,
                contents,
            });
        }
        units
    }

    fn unit_of_planned(planned: PlannedUnit, role: Role, unit_of_role: &HashMap<Role, UnitIdentifier>) -> UnitIdentifier {
        match planned {
            PlannedUnit::Carried(unit) => unit,
            PlannedUnit::Rewritten => *unit_of_role.get(&role).expect("重写的角色这次取了落点"),
        }
    }

    fn code_two_version_after(
        plan: &CodeTwoPlan,
        previous: &CodeTwoTreeVersion,
        role_of: fn(CodeTwoNodePosition) -> Role,
        unit_of_role: &HashMap<Role, UnitIdentifier>,
    ) -> CodeTwoTreeVersion {
        let units = plan
            .shape
            .nodes
            .iter()
            .zip(&plan.origins)
            .map(|((position, _), origin)| match origin {
                CodeTwoOrigin::CarriedFrom(previous_position) => previous.unit_at(*previous_position),
                CodeTwoOrigin::Rewritten => *unit_of_role.get(&role_of(*position)).expect("重写的节点这次取了落点"),
            })
            .collect();
        CodeTwoTreeVersion {
            shape: plan.shape.clone(),
            units,
        }
    }

    fn build_version(&self, plan: &LogicalPlan, assembled: &AssembledPublish, unit_of_role: &HashMap<Role, UnitIdentifier>) -> Version {
        let previous = &self.version;
        Version {
            txg: plan.txg,
            instance: plan.instance,
            highest_transaction: plan.highest_transaction_after,
            next_inode: plan.next_inode_after,
            file_units: plan
                .data_units
                .iter()
                .enumerate()
                .map(|(position, planned)| Self::unit_of_planned(*planned, Role::Data(position), unit_of_role))
                .collect(),
            extent_root: Some(Self::unit_of_planned(plan.extent_root, Role::ExtentRoot, unit_of_role)),
            extent_lower: plan
                .extent_lower
                .iter()
                .map(|(position, planned)| {
                    (*position, Self::unit_of_planned(*planned, Role::ExtentLower(*position), unit_of_role))
                })
                .collect(),
            inode_containers: plan
                .inode_containers
                .iter()
                .enumerate()
                .map(|(position, container)| InodeContainerVersion {
                    container_number: container.container_number,
                    container_birth: container.container_birth,
                    records: container.records.clone(),
                    unit: Self::unit_of_planned(container.origin, Role::InodeContainer(position), unit_of_role),
                })
                .collect(),
            inode_root: Some(Self::unit_of_planned(plan.inode_root, Role::InodeRoot, unit_of_role)),
            deleted_queue: plan.deleted_queue.clone(),
            deleted_root: plan
                .deleted_root
                .map(|planned| Self::unit_of_planned(planned, Role::DeletedInodesRoot, unit_of_role)),
            tombstone_containers: plan
                .tombstone_containers
                .iter()
                .enumerate()
                .map(|(position, container)| TombstoneContainerVersion {
                    container_number: container.container_number,
                    container_birth: container.container_birth,
                    records: container.records.clone(),
                    unit: Self::unit_of_planned(container.origin, Role::TombstoneContainer(position), unit_of_role),
                })
                .collect(),
            tombstone_root: plan
                .tombstone_root
                .map(|planned| Self::unit_of_planned(planned, Role::TombstoneRoot, unit_of_role)),
            allocation_nodes: assembled
                .allocation_nodes
                .iter()
                .map(|(node, planned)| (*node, Self::unit_of_planned(*planned, Role::AllocationNode(*node), unit_of_role)))
                .collect(),
            accounting: Self::code_two_version_after(&assembled.accounting_plan, &previous.accounting, Role::AccountingNode, unit_of_role),
            mapping: Self::code_two_version_after(&assembled.mapping_plan, &previous.mapping, Role::MappingNode, unit_of_role),
            tree_table: *unit_of_role.get(&Role::TreeTable).expect("树表每次都重写"),
            instance_table: if plan.instance_table_rewritten {
                *unit_of_role.get(&Role::InstanceTable).expect("写行那次重写实例表")
            } else {
                previous.instance_table
            },
        }
    }

    /// 第 t 次发布落盘之后扫一次（第一节「扫描时刻」）：先更新真值与死亡记录，再更新 R-U 的增量计数，再按三种读法量每一格。
    fn scan(
        &mut self,
        txg: u64,
        released: &[UnitIdentifier],
        written: &[UnitIdentifier],
        made_unreadable: &[UnitIdentifier],
        tombstones_written: &[TombstoneRecordSummary],
    ) {
        let new_truth = truth_of_the_version(&self.version, &self.disk, &self.allocator);
        let mut born = Vec::new();
        let mut died = Vec::new();
        for namespace in NAMESPACES {
            let position = namespace.position();
            for fields in new_truth[position].difference(&self.truth[position]) {
                born.push(namespaced(namespace, *fields));
            }
            for fields in self.truth[position].difference(&new_truth[position]) {
                died.push(namespaced(namespace, *fields));
            }
            self.ever_seen[position].extend(new_truth[position].iter().copied());
        }
        for key in &died {
            self.death_time.insert(*key, txg);
        }
        self.truth = new_truth;
        for reading in &mut self.unit_scan_readings {
            for unit_identifier in made_unreadable {
                reading.on_unit_unreadable(*unit_identifier, &self.disk.units[*unit_identifier], &self.death_time);
            }
            for unit_identifier in released {
                reading.on_unit_released(*unit_identifier, &self.disk.units[*unit_identifier], txg, &self.death_time);
            }
            for unit_identifier in written {
                reading.on_unit_readable(*unit_identifier, &self.disk.units[*unit_identifier], &self.death_time);
            }
            for key in &born {
                reading.set_truth(*key, true);
            }
            for key in &died {
                reading.set_truth(*key, false);
                if reading.arm == Arm::DeathRecordForEveryKey {
                    reading.on_key_death(*key);
                }
            }
            if reading.arm == Arm::TodaysTombstones {
                for tombstone in tombstones_written {
                    reading.on_tombstone(*tombstone, &self.disk);
                }
            }
            reading.expire_the_window(txg, &self.disk, &self.death_time);
        }
        self.scans += 1;
        self.occupancy_peak = self.occupancy_peak.max(self.allocator.allocated_slot_count);
        self.live_slot_peak = self.live_slot_peak.max(self.allocator.live_slot_count);
        let released_slots: u64 = released.iter().map(|unit_identifier| self.disk.units[*unit_identifier].span_slots).sum();
        self.slots_held_during_a_publish_peak = self
            .slots_held_during_a_publish_peak
            .max(self.allocator.live_slot_count + released_slots);
        if self.geometry == Geometry::Ring
            && self.allocator.allocated_slot_count > OCCUPANCY_CAP_SLOTS
            && self.exceeded_the_premise_at.is_none()
        {
            self.exceeded_the_premise_at = Some(txg);
        }
        let readable_by_scan = self.disk.readable_units_by_scan();
        self.assertions.judge(
            "A19_readable_units_equal_written_minus_overwritten",
            readable_by_scan + self.disk.units_made_unreadable == self.disk.units_written,
        );
        self.assertions.judge(
            "A2_accounting_rows_per_version",
            length_as_u64(self.truth[Namespace::Accounting.position()].len())
                == ACCOUNTING_POOL_WIDE_ROWS + ACCOUNTING_PER_DEVICE_ROWS * u64::from(DEVICE_COUNT),
        );
        self.measure_every_cell(txg);
        if self.cross_checks_every_scan {
            self.cross_check_the_unit_scan();
        }
    }

    fn measure_every_cell(&mut self, txg: u64) {
        let is_in_unlink = !self.version.deleted_queue.is_empty();
        let inputs = ScanInputs {
            disk: &self.disk,
            unit_lists: &self.unit_lists,
            death_time: &self.death_time,
            planted: None,
            erased: None,
        };
        let mut metrics_of_this_scan = Vec::new();
        for arm in self.write_path.arms() {
            let unit_scan = self
                .unit_scan_readings
                .iter()
                .find(|reading| reading.arm == arm)
                .expect("这条写路径的每条臂一份 R-U 计数");
            let select_current = select_current_reading(Reader::SelectCurrent, arm, &inputs);
            let gated = select_current_reading(Reader::SelectCurrentGatedByObject, arm, &inputs);
            for reader in READERS {
                for tree in REPORTED_TREES {
                    let (measurement, deaths) = match tree.namespace(reader) {
                        None => (CellMeasurement::default(), 0),
                        Some(namespace) => {
                            let position = namespace.position();
                            let deaths = length_as_u64(self.ever_seen[position].len() - self.truth[position].len());
                            let measurement = match reader {
                                Reader::UnitScan => CellMeasurement {
                                    resurrected: unit_scan.resurrected[position],
                                    missed: unit_scan.missed[position],
                                    resurrected_in_window: unit_scan.resurrected_in_window[position],
                                },
                                Reader::SelectCurrent => {
                                    measure_against_truth(&select_current.keys[position], &self.truth[position], txg)
                                }
                                Reader::SelectCurrentGatedByObject => {
                                    measure_against_truth(&gated.keys[position], &self.truth[position], txg)
                                }
                            };
                            (measurement, deaths)
                        }
                    };
                    self.trajectories
                        .entry((arm, reader, tree))
                        .or_default()
                        .observe(txg, measurement, deaths, is_in_unlink);
                    if arm == Arm::WithoutTombstones && reader == Reader::UnitScan && self.geometry == Geometry::Wide {
                        self.assertions.judge("A3_unit_scan_on_wide_reads_every_death", measurement.resurrected == deaths);
                    }
                    if arm == Arm::DeathRecordForEveryKey {
                        let name = if self.geometry == Geometry::Ring {
                            "A13_death_records_zero_on_ring"
                        } else {
                            "A4_death_records_zero"
                        };
                        self.assertions.judge(name, measurement.resurrected == 0 && measurement.missed == 0);
                    }
                    metrics_of_this_scan.extend([measurement.resurrected, measurement.missed, deaths]);
                }
            }
        }
        self.scan_log.push(metrics_of_this_scan);
    }

    /// 最后一次扫描（单测里每次扫描）把 R-U 的增量计数与整扫参照路径逐个名字空间比（两条路不共用代码）。
    pub(super) fn cross_check_the_unit_scan(&mut self) {
        let txg = self.version.txg;
        let inputs = ScanInputs {
            disk: &self.disk,
            unit_lists: &self.unit_lists,
            death_time: &self.death_time,
            planted: None,
            erased: None,
        };
        let mut every_namespace_matches = true;
        for reading in &self.unit_scan_readings {
            let brute = unit_scan_reading_by_brute_force(reading.arm, &inputs);
            for namespace in NAMESPACES {
                let position = namespace.position();
                let measured = measure_against_truth(&brute.keys[position], &self.truth[position], txg);
                let window_matches = !reading.tracks_the_ring_window
                    || measured.resurrected_in_window == reading.resurrected_in_window[position];
                if measured.resurrected != reading.resurrected[position]
                    || measured.missed != reading.missed[position]
                    || !window_matches
                {
                    every_namespace_matches = false;
                }
            }
        }
        self.assertions.judge("unit_scan_incremental_equals_brute_force", every_namespace_matches);
    }

    /// 一次用户改动经会话（12.4.1 第 3 条、C15）：`crates/` 会拒的照拒、记一行、不发布。
    pub(super) fn apply(&mut self, change: UserChange) {
        if self.has_stopped() {
            return;
        }
        match change {
            UserChange::Overwrite => {
                if self.version.has_live_inode(FIRST_FILE_INODE) {
                    self.publish_or_stop(PublishOperation::Overwrite);
                } else {
                    self.refused_steps += 1;
                }
            }
            UserChange::SequentialWrite { data_units } => {
                if self.version.has_live_inode(FIRST_FILE_INODE) {
                    self.publish_or_stop(PublishOperation::SequentialWrite { data_units });
                } else {
                    self.refused_steps += 1;
                }
            }
            UserChange::NewInodes { count } => self.publish_or_stop(PublishOperation::NewInodes { count }),
            UserChange::Unlink { inode } => {
                if self.version.is_with_trees_and_no_file() || !self.version.has_live_inode(inode) {
                    self.refused_steps += 1;
                    return;
                }
                let data_units_at_the_intent = if inode == FIRST_FILE_INODE {
                    length_as_u64(self.version.file_units.len())
                } else {
                    0
                };
                let publishes_before = self.publishes;
                self.publish_or_stop(PublishOperation::UnlinkIntent { inode });
                while !self.has_stopped() && !self.version.deleted_queue.is_empty() {
                    let step = if inode == FIRST_FILE_INODE {
                        let remaining = length_as_u64(self.version.file_units.len());
                        if remaining <= self.batch_data_units {
                            PublishOperation::UnlinkFinal { inode }
                        } else {
                            PublishOperation::UnlinkBatch {
                                data_units_kept: remaining - self.batch_data_units,
                            }
                        }
                    } else {
                        PublishOperation::UnlinkFinal { inode }
                    };
                    self.publish_or_stop(step);
                }
                self.unlink_publish_counts.push(UnlinkPublishCount {
                    inode,
                    data_units_at_the_intent,
                    publishes: self.publishes - publishes_before,
                });
            }
        }
    }

    /// 盘上见过的每个 inode 的对象出生代（号不复用，一个号一个出生代）。
    pub(super) fn inode_births(&self) -> BTreeMap<u64, u64> {
        let mut births = BTreeMap::new();
        for unit_identifier in &self.unit_lists.inode_containers {
            if let UnitContents::InodeRecords(records) = &self.disk.units[*unit_identifier].contents {
                for record in records {
                    births.insert(record.inode, record.object_birth);
                }
            }
        }
        births
    }

    /// 现行那一版墓碑树里的全部墓碑（墓碑只增、现行容器恒读得出，盘上的墓碑就是这些）。
    pub(super) fn tombstones_on_disk(&self) -> Vec<TombstoneRecordSummary> {
        self.version
            .tombstone_containers
            .iter()
            .flat_map(|container| container.records.iter().copied())
            .collect()
    }
}

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
