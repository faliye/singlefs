use super::*;

// ============================================================================
// 五、造历史：每一步（mkfs、一次挂载、一次发布）在镜像上各起一份录制盘，录下这一步的写流；落下的写按次序施加到镜像上，
//    根槽写同时记进账本。「崩」= 那一步的写流截在它第一个根槽写（`StepKind::RootRecordFua`）之前：记录已落、根槽没落（登记第五节）。
// ============================================================================

/// 一次挂载（或一串发布）落下的根槽写怎么记进账本：每一版的发布路径、写行那次发布写的行、清扫删的行。
struct LandingContext<'kinds> {
    kind_of: &'kinds dyn Fn(VersionKey) -> PublishKind,
    rows_written_by_the_row_publish: Vec<RowTriple>,
    rows_deleted_by_the_sweep: Vec<RowTriple>,
}

/// 一个实例挂着时手里的东西：分配器、现行那一版、账本里最后一版（下一次发布的父版本）。
pub(super) struct Session {
    pub(super) instance: InstanceGeneration,
    pub(super) allocator: PoolAllocator,
    pub(super) current: PoolVersion,
    pub(super) last_key: VersionKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MountConfiguration {
    /// 产品挂载 `mount_writable`（ss、skr、nrb、rs1）。
    Product,
    /// 每次挂载走 `mount_writable_with_instance_table_sweep_configuration`（开关关、给定阈值、落点取挂载的分配器；sw 三组与 G1）。
    SweepAtTheEndAboveTheThreshold { threshold: u64 },
}

/// 一次挂载交回的东西里这条实验要的那几样。
#[derive(Clone, Debug, Default)]
pub(super) struct MountSummary {
    pub(super) instance: u32,
    chosen: Option<VersionKey>,
    effective: Option<VersionKey>,
    pub(super) row_publish: Option<VersionKey>,
    pub(super) sweep_outcome: String,
    pub(super) sweep_version: Option<VersionKey>,
}

fn row_triple_of(row: &singlefs_core::instance_table::InstanceRow) -> RowTriple {
    (
        row.instance.0,
        row.selected_root_txg.0,
        row.applied_transaction_high_water,
    )
}

/// 清扫结局的名字（只取成员名，带的值另报）。
fn sweep_outcome_name(sweep: &InstanceTableSweepAtTheEndOfTheMount) -> String {
    match sweep {
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { rows, threshold } => {
            format!("not_swept_rows{rows}_threshold{}", threshold.0)
        }
        InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted => {
            "handed_to_the_background_sweep".to_string()
        }
        InstanceTableSweepAtTheEndOfTheMount::Swept(outcome) => match outcome {
            InstanceTableSweepOutcome::NoRowDeleted(reason) => {
                let text = format!("{reason:?}");
                let variant: String = text
                    .chars()
                    .take_while(|character| character.is_alphanumeric())
                    .collect();
                format!("no_row_deleted_{variant}")
            }
            InstanceTableSweepOutcome::RowsDeleted { rows_deleted, .. } => {
                format!("rows_deleted{}", rows_deleted.len())
            }
            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. } => {
                "sweep_publish_refused_for_space".to_string()
            }
            InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. } => {
                "stopped_by_a_block_device_error_while_erasing".to_string()
            }
            InstanceTableSweepOutcome::SweepPublishFailed { .. } => {
                "sweep_publish_failed".to_string()
            }
        },
    }
}

fn rows_deleted_by(sweep: &InstanceTableSweepAtTheEndOfTheMount) -> Option<Vec<RowTriple>> {
    match sweep {
        InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
            rows_deleted,
            ..
        }) => Some(rows_deleted.iter().map(row_triple_of).collect()),
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
        | InstanceTableSweepAtTheEndOfTheMount::Swept(
            InstanceTableSweepOutcome::NoRowDeleted(_)
            | InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. }
            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
        ) => None,
    }
}

/// 一次挂载交回的每一版根（写者交回的那一份，S2 拿它比镜像上的字节）。
fn roots_returned_by_the_mount(output: &MountOutput, current: &PoolVersion) -> Vec<RootRecord> {
    let mut roots = vec![*output.row_publish.root(), *current.root()];
    roots.extend(
        output
            .warm_up_publishes
            .iter()
            .map(|version| *version.root()),
    );
    for raised in output.space_admission.floor_raises() {
        roots.extend(raised.publishes.iter().map(|publish| publish.root));
    }
    for raised in &output.floor_raises_pushed_by_the_continuation {
        roots.extend(raised.publishes.iter().map(|publish| publish.root));
    }
    roots.extend(
        output
            .deletion_queue_continuation
            .iter()
            .map(|step| step.root),
    );
    roots
}

pub(super) struct HistoryBuilder {
    pub(super) parameters: MakeFilesystemParameters,
    fixed: FixedGeometry,
    pub(super) image: MemoryPool,
    pub(super) ledger: Ledger,
    /// 根环每个槽在哪块盘、哪个偏移（`root_ring::slot_offset` 按区域与槽现算），反查根槽写落在哪个 (区域, 槽)。
    ring_slot_of_location: BTreeMap<(u32, u64), (u64, u64)>,
    /// 写者交回的每一版根里实例表指针的出生 txg（S2 的一边）。
    returned_pointer_births: BTreeMap<VersionKey, u64>,
    publishes_so_far: usize,
}

fn is_a_root_slot_write(operation: &RetainedOperation, fixed: &FixedGeometry) -> bool {
    matches!(
        operation.operation.kind,
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
    ) && fixed.classify(&operation.operation) == StepKind::RootRecordFua
}

fn root_written_by(operation: &RetainedOperation) -> Option<RootRecord> {
    RootRecord::parse_slot(operation.contents.as_ref()?, &FILESYSTEM_IDENTIFIER)
}

/// 根槽字节里实例表指针的出生 txg（本地偏移 170 + 42，小端 8 字节；登记第七节 B7）。
pub(super) fn pointer_birth_txg_in_root_slot_bytes(slot_bytes: &[u8]) -> Option<u64> {
    let start = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + POINTER_BIRTH_TXG_OFFSET;
    let bytes: [u8; POINTER_BIRTH_TXG_BYTES] = slot_bytes
        .get(start..start + POINTER_BIRTH_TXG_BYTES)?
        .try_into()
        .ok()?;
    Some(u64::from_le_bytes(bytes))
}

/// 本地落点公式（登记第五节本地常量表第 3 行、`mount.rs` 写行那段读法）：区域 = txg mod 3，槽 = (txg ÷ 3) mod S。
fn ring_slot_by_the_local_formula(txg: u64, slots_per_region: u64) -> (u64, u64) {
    (
        txg % ROOT_RING_REGIONS,
        (txg / ROOT_RING_REGIONS) % slots_per_region,
    )
}

impl HistoryBuilder {
    pub(super) fn new(specification: &HistorySpecification) -> HistoryBuilder {
        let parameters = pool_parameters(
            specification.slots_per_region,
            specification.journal_ring_bytes,
        );
        let fixed = fixed_geometry_of(&parameters);
        let mut ring_slot_of_location = BTreeMap::new();
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..specification.slots_per_region {
                let device = REGION_DEVICES[usize::try_from(region).expect("区域号装得进 usize")];
                let offset = slot_offset(
                    RootRingSlot { region, slot },
                    parameters.geometry.fixed_structure_slot_spacing,
                );
                ring_slot_of_location.insert((device, offset.0), (region, slot));
            }
        }
        HistoryBuilder {
            parameters,
            fixed,
            image: MemoryPool::with_devices(&device_identities(), DEVICE_BYTES),
            ledger: Ledger::default(),
            ring_slot_of_location,
            returned_pointer_births: BTreeMap::new(),
            publishes_so_far: 0,
        }
    }

    pub(super) fn slots_per_region(&self) -> u64 {
        self.parameters.geometry.root_ring_slots_per_region.count()
    }

    /// 在此刻的镜像上起一份录制盘，做 `body`，交回它的结果与这一步录下的写流。
    fn recorded<Value>(
        &self,
        body: impl FnOnce(&mut RecordedDevices) -> Value,
    ) -> (Value, Vec<RetainedOperation>) {
        let stream = SharedStream::retaining_contents();
        let mut devices = recording_devices_of(&self.image, &stream);
        let value = body(&mut devices);
        drop(devices);
        (value, stream.retained_operations())
    }

    fn note_returned_roots(&mut self, roots: &[RootRecord]) {
        for root in roots {
            self.returned_pointer_births.insert(
                VersionKey::of_root(root),
                root.instance_table.head.birth_txg.0,
            );
        }
    }

    /// 账本里加一版（已有的只补「根槽落了」）。行集按发布路径推：写行那次发布 = 父版本的行并上这次写的行；清扫 = 父版本的行减去删的行；别的照抄父版本。
    fn register_version(
        &mut self,
        root: &RootRecord,
        parent: Option<VersionKey>,
        context: &LandingContext<'_>,
        root_landed: bool,
    ) -> VersionKey {
        let key = VersionKey::of_root(root);
        if let Some(existing) = self.ledger.versions.get_mut(&key) {
            existing.root_landed |= root_landed;
            return key;
        }
        let kind = (context.kind_of)(key);
        let parent_rows = self.ledger.rows_of(parent);
        let rows = match kind {
            PublishKind::RowPublish => {
                let mut rows = (*parent_rows).clone();
                rows.extend(context.rows_written_by_the_row_publish.iter().copied());
                Arc::new(rows)
            }
            PublishKind::Sweep => {
                let mut rows = (*parent_rows).clone();
                for deleted in &context.rows_deleted_by_the_sweep {
                    rows.remove(deleted);
                }
                Arc::new(rows)
            }
            PublishKind::MakeFilesystem
            | PublishKind::WarmUp
            | PublishKind::OtherMountPublish
            | PublishKind::FirstFile
            | PublishKind::Overwrite
            | PublishKind::SequentialWrite
            | PublishKind::NewInode
            | PublishKind::EmptyPublish
            | PublishKind::Rollback => parent_rows,
        };
        self.ledger.versions.insert(
            key,
            LedgerVersion {
                parent,
                kind,
                pointer_birth_txg: root.instance_table.head.birth_txg.0,
                root_landed,
                applied_by_a_mount: false,
                publish_order: self.publishes_so_far,
                rows,
            },
        );
        self.publishes_so_far += 1;
        key
    }

    /// 根槽写落下那一刻：核落点公式（本地公式、`target_for_publish`、写的位置三方相同）与 S2（[212, 220) 字节对写者交回的出生 txg），记占位。
    fn note_a_landed_root_slot_write(&mut self, operation: &RetainedOperation, root: &RootRecord) {
        let key = VersionKey::of_root(root);
        let location = (operation.operation.device.0, operation.operation.offset.0);
        let Some(&ring_slot) = self.ring_slot_of_location.get(&location) else {
            self.ledger.consistency_problems.push(format!(
                "版本 {} 的根槽写落在盘 {} 偏移 {}，不是根环里的槽",
                key.text(),
                location.0,
                location.1
            ));
            return;
        };
        if key.txg != GENESIS_TXG {
            let local = ring_slot_by_the_local_formula(key.txg, self.slots_per_region());
            let crates_target = target_for_publish(
                CheckpointTxg(key.txg),
                self.parameters.geometry.root_ring_slots_per_region,
            );
            if local != ring_slot || (crates_target.region, crates_target.slot) != ring_slot {
                self.ledger.ring_formula_mismatches.push(format!(
                    "版本 {}：写在 {ring_slot:?}，本地公式 {local:?}，target_for_publish ({}, {})",
                    key.text(),
                    crates_target.region,
                    crates_target.slot
                ));
            }
        } else if ring_slot.1 != 0 {
            self.ledger
                .ring_formula_mismatches
                .push(format!("第 0 代根写在 {ring_slot:?}，不是某个区域的槽 0"));
        }
        let in_bytes = operation
            .contents
            .as_deref()
            .and_then(pointer_birth_txg_in_root_slot_bytes);
        match self.returned_pointer_births.get(&key) {
            Some(returned) => {
                self.ledger.root_slot_writes_compared += 1;
                if in_bytes != Some(*returned) {
                    self.ledger.root_slot_write_mismatches.push(format!(
                        "版本 {}：根槽 [212, 220) 读出 {in_bytes:?}，写者交回 {returned}",
                        key.text()
                    ));
                }
            }
            None => self.ledger.root_slot_writes_without_a_returned_root += 1,
        }
        self.ledger.ring_occupants.insert(ring_slot, key);
        self.ledger.corrupted_ring_slots.remove(&ring_slot);
    }

    /// 把一步的写流按次序施加到镜像上；根槽写逐条记进账本，父版本接 `chain_from` 往下串。交回这一串里最后一版。
    fn land_operations(
        &mut self,
        operations: &[RetainedOperation],
        context: &LandingContext<'_>,
        chain_from: Option<VersionKey>,
    ) -> Option<VersionKey> {
        let mut last = chain_from;
        for operation in operations {
            if is_a_root_slot_write(operation, &self.fixed) {
                match root_written_by(operation) {
                    Some(root) => {
                        // 已在账本里的版本（mkfs 把第 0 代根种进三个区域）只补「根槽落了」，父版本不改（`register_version`）。
                        let key = self.register_version(&root, last, context, true);
                        self.note_a_landed_root_slot_write(operation, &root);
                        last = Some(key);
                    }
                    None => self
                        .ledger
                        .consistency_problems
                        .push("一次根槽写的内容解不出根记录".to_string()),
                }
            }
            self.image.apply(std::slice::from_ref(operation));
        }
        last
    }

    /// 「崩」：施加第一个根槽写之前的那一截，那一版记成「发了、根槽没落」。交回那一版。
    fn land_up_to_the_first_root_slot_write(
        &mut self,
        operations: &[RetainedOperation],
        context: &LandingContext<'_>,
        parent: Option<VersionKey>,
    ) -> Result<VersionKey, String> {
        let cut = operations
            .iter()
            .position(|operation| is_a_root_slot_write(operation, &self.fixed))
            .ok_or_else(|| "要崩的那一步写流里没有根槽写".to_string())?;
        let root = root_written_by(&operations[cut])
            .ok_or_else(|| "要崩的那一步的根槽写解不出根记录".to_string())?;
        self.image.apply(&operations[..cut]);
        Ok(self.register_version(&root, parent, context, false))
    }

    /// 一次挂载的恢复施加出来的那几版（有效根沿父版本往回到所选根为止，不含所选根）记成「被某次挂载施加过」。
    fn mark_the_versions_the_mount_applied(&mut self, chosen: VersionKey, effective: VersionKey) {
        let mut cursor = Some(effective);
        while let Some(key) = cursor {
            if key == chosen {
                return;
            }
            let Some(version) = self.ledger.versions.get_mut(&key) else {
                self.ledger.consistency_problems.push(format!(
                    "挂载的有效根 {} 沿父版本往回走到 {}，账本里没有这一版",
                    effective.text(),
                    key.text()
                ));
                return;
            };
            version.applied_by_a_mount = true;
            cursor = version.parent;
        }
        self.ledger.consistency_problems.push(format!(
            "挂载的有效根 {} 沿父版本往回走不到所选根 {}",
            effective.text(),
            chosen.text()
        ));
    }

    /// mkfs：录下 mkfs 的写流、施加，第 0 代根记进账本（父版本无）。
    pub(super) fn make_filesystem(&mut self) -> Result<(), String> {
        let parameters = self.parameters.clone();
        let (result, operations) =
            self.recorded(|devices| make_filesystem(&parameters, devices.as_mut_slice()));
        let _made = result.map_err(|error| format!("mkfs：{error:?}"))?;
        let kind_of = |_key: VersionKey| PublishKind::MakeFilesystem;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        self.land_operations(&operations, &context, None);
        Ok(())
    }

    fn mount_on(
        parameters: &MakeFilesystemParameters,
        devices: &mut RecordedDevices,
        configuration: MountConfiguration,
    ) -> Result<Mounted, singlefs_core::mount::MountError> {
        match configuration {
            MountConfiguration::Product => mount_writable(parameters, devices),
            MountConfiguration::SweepAtTheEndAboveTheThreshold { threshold } => {
                mount_writable_with_instance_table_sweep_configuration(
                    parameters,
                    devices,
                    SpaceAdmission::JudgedByTheFormula,
                    ShadowLedger::On,
                    InstanceTableSweepConfiguration {
                        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                        row_count_threshold: InstanceTableRowCountThreshold(threshold),
                        sweep_publish_placements:
                            SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
                    },
                )
            }
        }
    }

    /// 一次可写挂载的发布路径：写行那次、暖机、清扫（发成了的那一版就是 `Mounted::current`），别的记「挂载里别的发布」。
    fn summary_of_the_mount(mounted: &Mounted) -> MountSummary {
        let output = &mounted.output;
        let rows_deleted = rows_deleted_by(&output.instance_table_sweep);
        MountSummary {
            instance: output.instance.0,
            chosen: Some(VersionKey::of_root(&output.chosen_root)),
            effective: Some(VersionKey::of_root(&output.effective_root)),
            row_publish: Some(VersionKey::of_root(output.row_publish.root())),
            sweep_outcome: sweep_outcome_name(&output.instance_table_sweep),
            sweep_version: rows_deleted.map(|_| VersionKey::of_root(mounted.current.root())),
        }
    }

    /// 可写挂载做成：写流施加到镜像、根槽写记进账本（写行那次发布的父版本是有效根）、恢复施加出来的那几版记成被施加过。
    /// 挂载返回 `Err` 是合法结局：镜像不动，交回错误，这段历史从这里停。
    pub(super) fn mount(
        &mut self,
        configuration: MountConfiguration,
    ) -> Result<(Session, MountSummary), String> {
        let parameters = self.parameters.clone();
        let (result, operations) =
            self.recorded(|devices| HistoryBuilder::mount_on(&parameters, devices, configuration));
        let mounted = result.map_err(|error| format!("可写挂载：{error:?}"))?;
        let summary = HistoryBuilder::summary_of_the_mount(&mounted);
        self.note_returned_roots(&roots_returned_by_the_mount(
            &mounted.output,
            &mounted.current,
        ));
        let warm_up_keys: BTreeSet<VersionKey> = mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|version| VersionKey::of_root(version.root()))
            .collect();
        let row_publish = summary.row_publish;
        let sweep_version = summary.sweep_version;
        let kind_of = |key: VersionKey| {
            if Some(key) == row_publish {
                PublishKind::RowPublish
            } else if warm_up_keys.contains(&key) {
                PublishKind::WarmUp
            } else if Some(key) == sweep_version {
                PublishKind::Sweep
            } else {
                PublishKind::OtherMountPublish
            }
        };
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: mounted
                .output
                .rows_written
                .iter()
                .map(row_triple_of)
                .collect(),
            rows_deleted_by_the_sweep: rows_deleted_by(&mounted.output.instance_table_sweep)
                .unwrap_or_default(),
        };
        let effective = summary.effective.expect("挂载交回有效根");
        let chosen = summary.chosen.expect("挂载交回所选根");
        if !self.ledger.versions.contains_key(&effective) {
            self.ledger.consistency_problems.push(format!(
                "实例 {} 挂载的有效根 {} 不在账本里",
                summary.instance,
                effective.text()
            ));
        }
        self.mark_the_versions_the_mount_applied(chosen, effective);
        let last = self
            .land_operations(&operations, &context, Some(effective))
            .unwrap_or(effective);
        let Mounted {
            output,
            allocator,
            current,
        } = mounted;
        let session = Session {
            instance: output.instance,
            allocator,
            current,
            last_key: last,
        };
        Ok((session, summary))
    }

    /// skr 里实例 2 那一次挂载：挂载在内存里做完，写流截在它第一个根槽写（写行那次发布）之前；那一版记成「发了、根槽没落」。
    pub(super) fn mount_crashing_at_the_first_root_slot_write(
        &mut self,
        configuration: MountConfiguration,
    ) -> Result<(MountSummary, VersionKey), String> {
        let parameters = self.parameters.clone();
        let (result, operations) =
            self.recorded(|devices| HistoryBuilder::mount_on(&parameters, devices, configuration));
        let mounted = result.map_err(|error| format!("要崩的那次可写挂载：{error:?}"))?;
        let summary = HistoryBuilder::summary_of_the_mount(&mounted);
        self.note_returned_roots(&roots_returned_by_the_mount(
            &mounted.output,
            &mounted.current,
        ));
        let row_publish = summary.row_publish;
        let kind_of = |key: VersionKey| {
            if Some(key) == row_publish {
                PublishKind::RowPublish
            } else {
                PublishKind::OtherMountPublish
            }
        };
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: mounted
                .output
                .rows_written
                .iter()
                .map(row_triple_of)
                .collect(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let crashed =
            self.land_up_to_the_first_root_slot_write(&operations, &context, summary.effective)?;
        Ok((summary, crashed))
    }

    fn run_the_publish(
        parameters: &MakeFilesystemParameters,
        devices: &mut RecordedDevices,
        allocator: &mut PoolAllocator,
        current: &PoolVersion,
        publish: Publish,
        instance: InstanceGeneration,
    ) -> Result<TransactionOutput, String> {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let salt = current.root().checkpoint_txg.0 + 1;
        let write_time_seconds = WRITE_TIME_SECONDS + salt;
        if publish == Publish::FirstFile {
            let content = deterministic_content(0, OVERWRITE_CONTENT_BYTES);
            return publish_first_file(
                &mut writer,
                allocator,
                current.root(),
                FirstFile {
                    content: &content,
                    write_time_seconds,
                },
                instance,
                current.record_bytes(),
            )
            .map_err(|error| format!("第一个文件：{error:?}"));
        }
        let previous = current
            .file_version()
            .ok_or_else(|| "现行那一版没有文件，接不上用户改动".to_string())?;
        let result = match publish {
            Publish::FirstFile => unreachable!("第一个文件在进 match 之前已交回"),
            Publish::UserWrite(UserWrite::Overwrite)
            | Publish::RollbackAction {
                action: RollbackAction::Overwrite,
                ..
            } => {
                let content = deterministic_content(salt, OVERWRITE_CONTENT_BYTES);
                publish_overwrite(
                    &mut writer,
                    allocator,
                    previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                )
            }
            Publish::UserWrite(UserWrite::SequentialWrite { content_bytes }) => {
                let content = deterministic_content(salt, content_bytes);
                publish_sequential_write(
                    &mut writer,
                    allocator,
                    previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                )
            }
            Publish::RollbackAction {
                action: RollbackAction::LongerSequentialWrite,
                target,
            } => {
                let mut content = deterministic_content(target, OVERWRITE_CONTENT_BYTES);
                content.extend(deterministic_content(salt, OVERWRITE_CONTENT_BYTES));
                publish_sequential_write(
                    &mut writer,
                    allocator,
                    previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                )
            }
            Publish::RollbackAction {
                action: RollbackAction::NewInode,
                ..
            } => publish_new_inodes(
                &mut writer,
                allocator,
                previous,
                1,
                write_time_seconds,
                instance,
            ),
            Publish::RollbackAction {
                action: RollbackAction::EmptyPublish,
                ..
            } => publish_version(
                &mut writer,
                allocator,
                PublishPlan {
                    txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
                    counter: previous.record.counter + 1,
                    transaction: 0,
                    highest_transaction_number_before_this_publish: previous
                        .highest_transaction_number_in_this_instance,
                    instance,
                    back_chain: back_chain_of(&previous.record_bytes),
                    file: None,
                    new_inode_records: &[],
                    instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
                    tree_birth_txg: previous.tree_birth_txg(),
                    tree_identifier_watermark: previous.root.tree_identifier_watermark,
                    rollback_floor: previous.root.rollback_floor,
                },
                Some(previous),
            ),
        };
        result.map_err(|error| format!("{}：{error:?}", publish.kind().name()))
    }

    /// 一次发布（第一个文件、用户写、rs1 回退之后的动作），落下的根槽写记进账本，父版本是会话里最后一版。
    pub(super) fn publish(
        &mut self,
        session: &mut Session,
        publish: Publish,
    ) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let instance = session.instance;
        let current = session.current.clone();
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            HistoryBuilder::run_the_publish(
                &parameters,
                devices,
                allocator,
                &current,
                publish,
                instance,
            )
        });
        let output = result?;
        self.note_returned_roots(&[output.root]);
        let kind = publish.kind();
        let kind_of = move |_key: VersionKey| kind;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let last = self
            .land_operations(&operations, &context, Some(session.last_key))
            .ok_or_else(|| format!("{} 那一次发布的写流里没有根槽写", kind.name()))?;
        session.current = PoolVersion::of_a_version_of_trees(output);
        session.last_key = last;
        Ok(last)
    }

    /// 「崩」的那一次发布：写流截在它的根槽写之前，那一版记成「发了、根槽没落」，会话到此为止。
    pub(super) fn publish_crashing(
        &mut self,
        mut session: Session,
        publish: Publish,
    ) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let instance = session.instance;
        let current = session.current.clone();
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            HistoryBuilder::run_the_publish(
                &parameters,
                devices,
                allocator,
                &current,
                publish,
                instance,
            )
        });
        let output = result?;
        self.note_returned_roots(&[output.root]);
        let kind = publish.kind();
        let kind_of = move |_key: VersionKey| kind;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        self.land_up_to_the_first_root_slot_write(&operations, &context, Some(session.last_key))
    }

    /// rs1：管理员回退（一次向前发布）到 (实例 1, target)。
    pub(super) fn roll_back(
        &mut self,
        session: &mut Session,
        target: u64,
    ) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let mut current = session
            .current
            .file_version()
            .cloned()
            .ok_or_else(|| "回退之前现行那一版没有文件".to_string())?;
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                &mut current,
                RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(target),
                },
            )
        });
        let _rolled_back =
            result.map_err(|error| format!("回退到 (实例 1, txg {target})：{error:?}"))?;
        self.note_returned_roots(&[current.root]);
        let kind_of = |_key: VersionKey| PublishKind::Rollback;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let last = self
            .land_operations(&operations, &context, Some(session.last_key))
            .ok_or_else(|| "回退那一次发布的写流里没有根槽写".to_string())?;
        session.current = PoolVersion::of_a_version_of_trees(current);
        session.last_key = last;
        Ok(last)
    }

    /// 此刻镜像与账本的一份拷贝，把点名的几版的根槽各读坏（槽内第 40 字节取反）。槽里已经住着别的版本的不翻、记进交回的第三样。
    pub(super) fn copy_with_corrupted_root_slots(
        &self,
        versions: &[VersionKey],
    ) -> (MemoryPool, Ledger, Vec<String>) {
        let mut image = self.image.clone();
        let mut ledger = self.ledger.clone();
        let mut skipped = Vec::new();
        for version in versions {
            let ring_slot = ring_slot_by_the_local_formula(version.txg, self.slots_per_region());
            if version.txg == GENESIS_TXG || ledger.ring_occupants.get(&ring_slot) != Some(version)
            {
                skipped.push(format!("{} 的根槽此刻不住着它", version.text()));
                continue;
            }
            let device = REGION_DEVICES[usize::try_from(ring_slot.0).expect("区域号装得进 usize")];
            let offset = slot_offset(
                RootRingSlot {
                    region: ring_slot.0,
                    slot: ring_slot.1,
                },
                self.parameters.geometry.fixed_structure_slot_spacing,
            );
            image.flip_byte(
                DeviceIdentity(device),
                offset,
                CORRUPTED_BYTE_IN_THE_ROOT_SLOT,
            );
            ledger.corrupted_ring_slots.insert(ring_slot);
        }
        (image, ledger, skipped)
    }
}

/// 一次发布是哪一种（`HistoryBuilder::publish` 的参数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Publish {
    FirstFile,
    UserWrite(UserWrite),
    RollbackAction { action: RollbackAction, target: u64 },
}

impl Publish {
    pub(super) fn kind(self) -> PublishKind {
        match self {
            Publish::FirstFile => PublishKind::FirstFile,
            Publish::UserWrite(UserWrite::Overwrite)
            | Publish::RollbackAction {
                action: RollbackAction::Overwrite,
                ..
            } => PublishKind::Overwrite,
            Publish::UserWrite(UserWrite::SequentialWrite { .. })
            | Publish::RollbackAction {
                action: RollbackAction::LongerSequentialWrite,
                ..
            } => PublishKind::SequentialWrite,
            Publish::RollbackAction {
                action: RollbackAction::NewInode,
                ..
            } => PublishKind::NewInode,
            Publish::RollbackAction {
                action: RollbackAction::EmptyPublish,
                ..
            } => PublishKind::EmptyPublish,
        }
    }
}
