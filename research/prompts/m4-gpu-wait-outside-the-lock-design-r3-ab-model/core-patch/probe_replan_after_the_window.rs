
/// 只在探针的仓副本里有：`research/prompts/m4-gpu-wait-outside-the-lock-design-r3-ab-model/tools/patch_the_copy.py` 把这一段追加到副本的
/// `mounted_session.rs` 末尾。它把整理一批里不公开的几步交给探针，做第三轮判决「三、改法」第 5、7 条的 A/B；上面的产品代码一行不改。
/// 另调 `compaction_plan.rs` 的 `CompactionSwitches::forced_region`（只供测试的选段开关）与 `logged_operations.rs` 的
/// `CompactionRegion::of_one_cluster_segment`。
pub mod probe_replan_after_the_window {
    use super::*;

    /// 票：取输入那一刻的现行那一版、活意图、R 里起点在 R 的未释放记录（各块盘）、对 R 的扫描与照它算的这一批计划。
    pub struct TicketOfTheBatch {
        pub version: TransactionOutput,
        pub intent: CompactionIntent,
        pub records_in_the_region: BTreeMap<(DeviceIdentity, SlotNumber), AllocationRecord>,
        pub scan: Option<RegionScan>,
        pub planned: Vec<PlannedRelocation>,
        pub kind_name: String,
    }

    /// 回锁里怎么发（今天锁内重算那一臂不走这里，走 [`publish_as_today`]）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ArmOfTheRelock {
        /// 判决「三、改法」第 5 条：重取隔离集，比整条记录与集合，变了的单元从票里的计划剔掉，其余在现行那一版上发；剔完一个不剩不发。
        ReplanPruning,
        /// 现行那一版的 txg 与票里那一版不等就不发；相等时照票里的计划发。
        DropWhenTheTxgMoved,
        /// 阳性对照：票里的计划原样在现行那一版上发，不比记录、不重取隔离集。
        ReplanWithoutPruning,
    }

    /// 一臂回锁那一次的结局。
    #[derive(Clone, Debug, Default)]
    pub struct OutcomeOfTheRelock {
        pub published: bool,
        /// 发成的是哪一种整理发布（`CompactionPublishKind` 的成员名）；没发成是空串。
        pub kind_name: String,
        /// 没发成的原因（txg 变了、剔完了、停在某个停因、被发布路径拒）；发成是空串。
        pub not_published_because: String,
        /// 发布路径拒的成员名（`RelocationBatchRefused`；`Publish` 带里层成员名）；没被拒是空串。
        pub refusal_member: String,
        pub units_moved: usize,
        pub units_pruned: usize,
        /// 票里与现行那一版里起点在 R 的未释放记录，按（盘、槽）对不上的条数（两边各有一条而整条不等、或只一边有）。
        pub changed_records: usize,
        pub record_sets_equal: bool,
        pub quarantine_in_the_region_after_retaking: bool,
        pub units_skipped_as_rewritten_anyway: usize,
    }

    /// 回退门此刻的读数（`mount::rollback_states_of_the_ring`）。
    #[derive(Clone, Copy, Debug)]
    pub struct RollbackStatesForTheProbe {
        pub states_now: u64,
        pub states_after_the_next_publish_and_the_margin: u64,
    }

    fn batch_units_of(allocator: &PoolAllocator) -> CompactionBatchUnits {
        match allocator.compaction().switches.batch_size {
            CompactionBatchSize::OfTheMount(units) | CompactionBatchSize::ForcedForTests(units) => {
                units
            }
        }
    }

    fn first_word(text: &str) -> String {
        text.chars()
            .take_while(|character| character.is_alphanumeric() || *character == '_')
            .collect()
    }

    /// 发布路径拒的成员名：`Publish` 那一支带上里层 `PublishError` 的成员名。
    #[must_use]
    pub fn member_name_of_the_refusal(refused: &RelocationBatchRefused) -> String {
        match refused {
            RelocationBatchRefused::Publish(cause) => {
                format!("Publish.{}", first_word(&format!("{cause:?}")))
            }
            other => first_word(&format!("{other:?}")),
        }
    }

    /// 各块盘上起点在 R 里的未释放记录，按（盘、槽）。
    #[must_use]
    pub fn unreleased_records_starting_in_the_region(
        records: &[AllocationRecord],
        region: CompactionRegion,
    ) -> BTreeMap<(DeviceIdentity, SlotNumber), AllocationRecord> {
        records
            .iter()
            .filter(|record| {
                !record.is_released
                    && record.slot >= region.start_slot()
                    && record.slot < region.end_slot()
            })
            .map(|record| ((record.device, record.slot), *record))
            .collect()
    }

    /// 在 CPU 上扫一个 R，同 `advance_compaction_by_one_publish_computing_on` 有活意图那一支（第 5628 行起）：R 里有隔离槽、扫描拒了 R、
    /// 扫出隔离类都交 `None`（计划随之是丢意图）。扫描拒了 R 时不记进不合格段表（探针只看这一批）。
    fn scan_of_the_region_on_the_cpu<Device: BlockDevice>(
        version: &TransactionOutput,
        region: CompactionRegion,
        quarantined_slots: &BTreeSet<SlotNumber>,
        devices: &[(DeviceIdentity, Device)],
    ) -> Option<RegionScan> {
        if region_holds_a_quarantined_slot(region, quarantined_slots) {
            return None;
        }
        let read_of_the_version = version_as_the_compaction_reads_it(version);
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(
            &read_of_the_version.mapping_entries,
        );
        match scan_region_over_the_identity_index(
            region,
            &version.allocation_records,
            &mapping_index,
            &read_of_the_version.pinned_slots,
            quarantined_slots,
            |device, slot, bytes| devices.read(device, slot.to_device_offset(), bytes),
        ) {
            Ok(scan)
                if !scan
                    .allocations
                    .iter()
                    .any(|allocation| allocation.class == AllocationInRegion::Quarantined) =>
            {
                Some(scan)
            }
            Ok(_) | Err(_) => None,
        }
    }

    /// 照今天的代码在 CPU 上推一次整理发布（`advance_compaction_by_one_publish_computing_on`），要它是签发：交回签出的意图。
    ///
    /// # Errors
    /// 推出来的不是签发（停、发了别的、被拒、算错），交回说明。
    pub fn issue_an_intent<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &mut [(DeviceIdentity, Device)],
    ) -> Result<CompactionIntent, String> {
        let MountedSession {
            allocator,
            current,
            instance,
            parameters_and_device_table,
            compaction,
            ..
        } = session;
        let batch_units = batch_units_of(allocator);
        let step = advance_compaction_by_one_publish_computing_on(
            parameters_and_device_table.parameters_on_disk(),
            &mut *devices,
            allocator,
            compaction,
            current,
            *instance,
            batch_units,
            CompactionRunReach::UntilAStopOrTheBudget,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
            &mut CompactionComputeOfTheBatch::OnTheCpu,
        );
        match step {
            Ok(CompactionStep::Published {
                version,
                kind: CompactionPublishKind::IntentIssued,
            }) => live_compaction_intent_of_the_entries(&version.logged_operations)
                .map(|(_, intent)| intent)
                .ok_or_else(|| "issued_version_without_a_live_intent".to_string()),
            Ok(CompactionStep::Published { kind, .. }) => {
                Err(format!("published_{kind:?}_instead_of_an_issue"))
            }
            Ok(CompactionStep::Stopped(stop)) => Err(format!("stopped:{stop:?}")),
            Err(CompactionBatchNotCompleted::Refused(refused)) => Err(format!(
                "refused:{}",
                member_name_of_the_refusal(&refused)
            )),
            Err(CompactionBatchNotCompleted::ComputeFailed(error)) => {
                Err(format!("compute_failed:{error:?}"))
            }
        }
    }

    /// 探针替签发选的 R 与它的读数（[`region_whose_batch_moves_units_of_the_first_file`]）。
    #[derive(Clone, Copy, Debug)]
    pub struct RegionForTheProbe {
        pub region: CompactionRegion,
        /// R 那块盘上未释放记录罩住的 R 里的槽数（选段打分同 `greedy_score`）。
        pub live_slots: u64,
        pub planned_units: usize,
        /// 这一批里第一个文件的数据单元个数（大于 0 才是候选）。
        pub units_of_the_first_file_planned: usize,
        /// 判过的段数（含第一个文件的数据单元的整段）。
        pub segments_judged: usize,
    }

    /// `version` 里第一个文件每个数据单元的落点槽 → 它在文件里的单元号。
    #[must_use]
    pub fn units_of_the_first_file_by_slot(version: &TransactionOutput) -> BTreeMap<SlotNumber, u64> {
        version
            .units
            .iter()
            .filter_map(|unit| match unit.identity {
                TransactionUnit::Data(DataUnitIndexInFile(index)) => Some((unit.slot, index)),
                _ => None,
            })
            .collect()
    }

    /// 探针替签发选 R：单元区那块盘上含第一个文件数据单元的整段里，照今天的计划函数算出的这一批含第一个文件的数据单元的、
    /// 按记录活槽最少的那一段（同分取起点槽号最小的）。资格照 `advance_compaction_by_one_publish_computing_on` 签发那一支：
    /// 不在这次挂载开过的聚簇段里、罩住的槽不在钉住集与常驻隔离集里、扫描不拒、扫出来没有隔离类。
    /// 用处：起点池上今天的选段（greedy）挑中的段里没有第一个文件的数据单元，前台覆盖写或删文件碰不到这一批计划要搬的单元。
    ///
    /// # Errors
    /// 现行那一版没有树、已有活意图、池里没有盘，或没有一段合格。
    pub fn region_whose_batch_moves_units_of_the_first_file<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &[(DeviceIdentity, Device)],
    ) -> Result<RegionForTheProbe, String> {
        let MountedSession {
            allocator,
            current,
            compaction,
            ..
        } = session;
        compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
        let version = current
            .version_of_trees()
            .ok_or_else(|| "version_without_trees".to_string())?;
        if live_compaction_intent_of_the_entries(&version.logged_operations).is_some() {
            return Err("a_live_intent_before_the_issue".to_string());
        }
        let device_map = allocator
            .devices
            .first()
            .ok_or_else(|| "pool_without_a_device".to_string())?;
        let device = device_map.device;
        let unit_area_start = device_map.unit_area_start().slot().0;
        let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
        let read_of_the_version = version_as_the_compaction_reads_it(version);
        let quarantined_slots = compaction.quarantined_slots().clone();
        let units_of_the_first_file = units_of_the_first_file_by_slot(version);
        let segment_starts: BTreeSet<u64> = units_of_the_first_file
            .keys()
            .filter_map(|slot| {
                let segment = slot.0.checked_sub(unit_area_start)? / CLUSTER_SEGMENT_SLOTS;
                (segment < full_segments).then_some(unit_area_start + segment * CLUSTER_SEGMENT_SLOTS)
            })
            .collect();
        let segments_judged = segment_starts.len();
        let no_record_uncertain: &[Placement] = &[];
        let mut best: Option<RegionForTheProbe> = None;
        for start in segment_starts {
            let start_slot = SlotNumber(start);
            let end_slot = SlotNumber(start + CLUSTER_SEGMENT_SLOTS);
            if allocator.cluster_segments().contains(&start_slot)
                || read_of_the_version
                    .pinned_slots
                    .range(start_slot..end_slot)
                    .next()
                    .is_some()
                || quarantined_slots.range(start_slot..end_slot).next().is_some()
            {
                continue;
            }
            let Ok(region) = CompactionRegion::of_one_cluster_segment(device, start_slot) else {
                continue;
            };
            let Some(scan) = scan_of_the_region_on_the_cpu(version, region, &quarantined_slots, devices)
            else {
                continue;
            };
            let Some((planned, _intent_step, _kind)) =
                compaction_publish_planned_with_the_records_uncertain_this_round(
                    Some((&scan, no_record_uncertain)),
                    batch_units_of(allocator),
                    &BTreeSet::new(),
                    &version.allocation_records,
                    region,
                )
            else {
                continue;
            };
            let units_of_the_first_file_planned = planned
                .iter()
                .filter(|relocation| units_of_the_first_file.contains_key(&relocation.source.slot))
                .count();
            if units_of_the_first_file_planned == 0 {
                continue;
            }
            // 迭代上界是记录条数（每条记录的跨度至多 2 槽）。
            let live_slots: u64 = version
                .allocation_records
                .iter()
                .filter(|record| record.device == device && !record.is_released)
                .map(|record| {
                    u64::try_from(
                        (record.slot.0..record.slot.0 + u64::from(record.span_slots))
                            .filter(|slot| *slot >= start_slot.0 && *slot < end_slot.0)
                            .count(),
                    )
                    .expect("一段 64 槽")
                })
                .sum();
            if best.as_ref().is_none_or(|chosen| live_slots < chosen.live_slots) {
                best = Some(RegionForTheProbe {
                    region,
                    live_slots,
                    planned_units: planned.len(),
                    units_of_the_first_file_planned,
                    segments_judged,
                });
            }
        }
        best.ok_or_else(|| {
            format!("no_segment_whose_batch_moves_a_unit_of_the_first_file:segments_judged={segments_judged}")
        })
    }

    /// 在 `region` 上签发：照今天的代码推一次（[`issue_an_intent`]），推之前把只供测试的开关 `forced_region` 设成它、推完放回原来的值。
    ///
    /// # Errors
    /// 同 [`issue_an_intent`]；签出的 R 不是 `region`。
    pub fn issue_an_intent_on_the_region<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &mut [(DeviceIdentity, Device)],
        region: CompactionRegion,
    ) -> Result<CompactionIntent, String> {
        let forced_before = session.allocator.compaction().switches.forced_region;
        session.allocator.compaction_mut().switches.forced_region = Some(region);
        let issued = issue_an_intent(session, devices);
        session.allocator.compaction_mut().switches.forced_region = forced_before;
        match issued {
            Ok(intent) if intent.region == region => Ok(intent),
            Ok(intent) => Err(format!(
                "issued_on_another_region:{}",
                intent.region.start_slot().0
            )),
            Err(failure) => Err(failure),
        }
    }

    /// 取输入那一刻（锁里）：把发布记下的隔离并进常驻隔离集，拷现行那一版，记下 R 里的记录，对 R 扫一次、照今天的计划函数算这一批。
    /// 跳过的「本来就重写」集合是空的（那一项要发布路径试算才知道，回锁里照今天的循环补）。
    ///
    /// # Errors
    /// 现行那一版没有树或没有活意图，或计划函数交 `None`（只在 GPU 不确定时，CPU 走不到）。
    pub fn ticket_of_the_batch<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &[(DeviceIdentity, Device)],
    ) -> Result<TicketOfTheBatch, String> {
        let MountedSession {
            allocator,
            current,
            compaction,
            ..
        } = session;
        compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
        let version = current
            .version_of_trees()
            .ok_or_else(|| "ticket_version_without_trees".to_string())?
            .clone();
        let Some((_, intent)) = live_compaction_intent_of_the_entries(&version.logged_operations)
        else {
            return Err("ticket_version_without_a_live_intent".to_string());
        };
        let region = intent.region;
        let quarantined_slots = compaction.quarantined_slots().clone();
        let scan = scan_of_the_region_on_the_cpu(&version, region, &quarantined_slots, devices);
        let no_record_uncertain: &[Placement] = &[];
        let (planned, _intent_step, kind) =
            compaction_publish_planned_with_the_records_uncertain_this_round(
                scan.as_ref().map(|scan| (scan, no_record_uncertain)),
                batch_units_of(allocator),
                &BTreeSet::new(),
                &version.allocation_records,
                region,
            )
            .ok_or_else(|| "ticket_plan_none".to_string())?;
        Ok(TicketOfTheBatch {
            records_in_the_region: unreleased_records_starting_in_the_region(
                &version.allocation_records,
                region,
            ),
            version,
            intent,
            scan,
            planned,
            kind_name: format!("{kind:?}"),
        })
    }

    /// 发成之后同今天那一支（第 5712 行起）记账、换现行那一版。
    fn note_the_published_compaction_step(
        allocator: &mut PoolAllocator,
        compaction: &mut CompactionOfThisMount,
        current: &mut PoolVersion,
        intent: CompactionIntent,
        published: TransactionOutput,
        kind: CompactionPublishKind,
    ) {
        match kind {
            CompactionPublishKind::BatchCompletingTheIntent
            | CompactionPublishKind::IntentCompletedWithoutABatch => {
                allocator.compaction_mut().withdraw_the_live_intent();
                compaction.observations.intents_completed += 1;
                compaction.last_completed_intent = Some(intent);
            }
            CompactionPublishKind::IntentDropped
            | CompactionPublishKind::IntentDroppedWithNothingMovable
            | CompactionPublishKind::IntentDroppedOnAStop { .. } => {
                allocator.compaction_mut().withdraw_the_live_intent();
                compaction.observations.intents_dropped += 1;
            }
            CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
        }
        *current = PoolVersion::of_a_version_of_trees(published);
    }

    /// 回锁里照 `arm` 发这一批（票是锁外那一段之前取的）。
    ///
    /// # Panics
    /// 现行那一版没有树（前台改动只发带树的一版）。
    pub fn publish_after_the_relock<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &mut [(DeviceIdentity, Device)],
        ticket: &TicketOfTheBatch,
        arm: ArmOfTheRelock,
    ) -> OutcomeOfTheRelock {
        let MountedSession {
            allocator,
            current,
            instance,
            parameters_and_device_table,
            compaction,
            ..
        } = session;
        let parameters = parameters_and_device_table.parameters_on_disk();
        let region = ticket.intent.region;
        let mut outcome = OutcomeOfTheRelock::default();
        let current_version = current
            .version_of_trees()
            .expect("前台改动之后的现行那一版有树")
            .clone();
        let records_now =
            unreleased_records_starting_in_the_region(&current_version.allocation_records, region);
        let mut changed_slots: BTreeSet<SlotNumber> = BTreeSet::new();
        for (key, record) in &ticket.records_in_the_region {
            if records_now.get(key) != Some(record) {
                changed_slots.insert(key.1);
                outcome.changed_records += 1;
            }
        }
        for key in records_now.keys() {
            if !ticket.records_in_the_region.contains_key(key) {
                changed_slots.insert(key.1);
                outcome.changed_records += 1;
            }
        }
        outcome.record_sets_equal = ticket.records_in_the_region == records_now;
        match arm {
            ArmOfTheRelock::DropWhenTheTxgMoved => {
                if current_version.root.checkpoint_txg != ticket.version.root.checkpoint_txg {
                    outcome.not_published_because = "txg_moved".to_string();
                    return outcome;
                }
            }
            ArmOfTheRelock::ReplanPruning => {
                compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
                if region_holds_a_quarantined_slot(region, compaction.quarantined_slots()) {
                    outcome.quarantine_in_the_region_after_retaking = true;
                    match publish_the_compaction_step_once(
                        parameters,
                        &mut *devices,
                        allocator,
                        &current_version,
                        &[],
                        CompactionIntentStep::Drop,
                        *instance,
                    ) {
                        Ok(published) => {
                            outcome.published = true;
                            outcome.kind_name =
                                format!("{:?}", CompactionPublishKind::IntentDropped);
                            note_the_published_compaction_step(
                                allocator,
                                compaction,
                                current,
                                ticket.intent,
                                published,
                                CompactionPublishKind::IntentDropped,
                            );
                        }
                        Err(refused) => {
                            outcome.refusal_member = member_name_of_the_refusal(&refused);
                            outcome.not_published_because =
                                "refused_by_the_publish_path".to_string();
                        }
                    }
                    return outcome;
                }
            }
            ArmOfTheRelock::ReplanWithoutPruning => {}
        }
        let no_record_uncertain: &[Placement] = &[];
        let mut units_this_publish_rewrites_anyway: BTreeSet<Vec<u8>> = BTreeSet::new();
        // 迭代上界同今天那一支：每被拒一次跳过的集合至少多一把 key，票里的扫描里能搬的 key 有限。
        loop {
            let Some((planned, intent_step, kind)) =
                compaction_publish_planned_with_the_records_uncertain_this_round(
                    ticket.scan.as_ref().map(|scan| (scan, no_record_uncertain)),
                    batch_units_of(allocator),
                    &units_this_publish_rewrites_anyway,
                    &ticket.version.allocation_records,
                    region,
                )
            else {
                outcome.not_published_because = "only_records_uncertain_this_round".to_string();
                return outcome;
            };
            let planned_before_pruning = planned.len();
            let planned: Vec<PlannedRelocation> = match arm {
                ArmOfTheRelock::ReplanPruning => planned
                    .into_iter()
                    .filter(|relocation| !changed_slots.contains(&relocation.source.slot))
                    .collect(),
                ArmOfTheRelock::DropWhenTheTxgMoved | ArmOfTheRelock::ReplanWithoutPruning => {
                    planned
                }
            };
            outcome.units_pruned = planned_before_pruning - planned.len();
            if arm == ArmOfTheRelock::ReplanPruning
                && planned_before_pruning > 0
                && planned.is_empty()
            {
                outcome.not_published_because = "pruned_to_nothing".to_string();
                return outcome;
            }
            match publish_the_planned_compaction_step_falling_back_on_the_region_check(
                parameters,
                &mut *devices,
                allocator,
                &current_version,
                &planned,
                intent_step,
                kind,
                RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
                *instance,
            ) {
                Ok((published, kind)) => {
                    outcome.published = true;
                    outcome.kind_name = format!("{kind:?}");
                    outcome.units_moved = published.units_relocated_by_this_publish.len();
                    note_the_published_compaction_step(
                        allocator,
                        compaction,
                        current,
                        ticket.intent,
                        published,
                        kind,
                    );
                    return outcome;
                }
                Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
                    mapping_keys,
                }) => {
                    let skipped_before = units_this_publish_rewrites_anyway.len();
                    units_this_publish_rewrites_anyway.extend(mapping_keys);
                    outcome.units_skipped_as_rewritten_anyway =
                        units_this_publish_rewrites_anyway.len();
                    if units_this_publish_rewrites_anyway.len() == skipped_before {
                        outcome.not_published_because =
                            "rewritten_anyway_without_a_new_key".to_string();
                        return outcome;
                    }
                }
                Err(CompactionStepRefused::Refused(refused)) => {
                    outcome.refusal_member = member_name_of_the_refusal(&refused);
                    outcome.not_published_because = match pause_by_dropping_the_live_intent_or_hand_up(
                        parameters,
                        &mut *devices,
                        allocator,
                        compaction,
                        current,
                        *instance,
                        refused,
                    ) {
                        Ok(CompactionStep::Published { .. }) => {
                            "paused_short_of_space_intent_dropped".to_string()
                        }
                        Ok(CompactionStep::Stopped(stop)) => format!("stopped:{stop:?}"),
                        Err(_refused_not_for_space) => "refused_by_the_publish_path".to_string(),
                    };
                    return outcome;
                }
            }
        }
    }

    /// 今天锁内重算那一臂：在现行那一版上照今天的代码推一次（`advance_compaction_by_one_publish_computing_on`，CPU）。
    pub fn publish_as_today<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &mut [(DeviceIdentity, Device)],
    ) -> OutcomeOfTheRelock {
        let MountedSession {
            allocator,
            current,
            instance,
            parameters_and_device_table,
            compaction,
            ..
        } = session;
        let batch_units = batch_units_of(allocator);
        let mut outcome = OutcomeOfTheRelock::default();
        match advance_compaction_by_one_publish_computing_on(
            parameters_and_device_table.parameters_on_disk(),
            &mut *devices,
            allocator,
            compaction,
            current,
            *instance,
            batch_units,
            CompactionRunReach::UntilAStopOrTheBudget,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
            &mut CompactionComputeOfTheBatch::OnTheCpu,
        ) {
            Ok(CompactionStep::Published { version, kind }) => {
                outcome.published = true;
                outcome.kind_name = format!("{kind:?}");
                outcome.units_moved = version.units_relocated_by_this_publish.len();
            }
            Ok(CompactionStep::Stopped(stop)) => {
                outcome.not_published_because = format!("stopped:{stop:?}");
            }
            Err(CompactionBatchNotCompleted::Refused(refused)) => {
                outcome.refusal_member = member_name_of_the_refusal(&refused);
                outcome.not_published_because = "refused_by_the_publish_path".to_string();
            }
            Err(CompactionBatchNotCompleted::ComputeFailed(error)) => {
                outcome.not_published_because = format!("compute_failed:{error:?}");
            }
        }
        outcome
    }

    /// 根环此刻的回退状态（`mount::rollback_states_of_the_ring`，读法同 `stop_of_the_rollback_gate`）。
    ///
    /// # Errors
    /// 根环或有效根的树表读不通。
    pub fn rollback_states<Device: BlockDevice>(
        session: &MountedSession,
        devices: &[(DeviceIdentity, Device)],
    ) -> Result<RollbackStatesForTheProbe, String> {
        let parameters = session.parameters_and_device_table.parameters_on_disk();
        session
            .allocator
            .reading_and_counting_the_other_unit_reads_with_one_copy_failing(
                &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
                |reader| rollback_states_of_the_ring(reader, parameters, session.current.root()),
            )
            .map(|states| RollbackStatesForTheProbe {
                states_now: states.states_now,
                states_after_the_next_publish_and_the_margin: states
                    .states_after_the_next_publish_and_the_margin,
            })
            .map_err(|_unreadable| "rollback_states_unreadable".to_string())
    }

    /// 今天的回退门判（`stop_of_the_rollback_gate`）：门关交回停因的名字，门开交 `None`。
    #[must_use]
    pub fn rollback_gate_stop<Device: BlockDevice>(
        session: &MountedSession,
        devices: &[(DeviceIdentity, Device)],
    ) -> Option<String> {
        stop_of_the_rollback_gate(
            session.parameters_and_device_table.parameters_on_disk(),
            devices,
            &session.allocator,
            &session.current,
        )
        .map(|stop| first_word(&format!("{stop:?}")))
    }

    /// 前台推一串抬 F 的空发布：先照准入那一条抬到上限（`mount::push_one_floor_raise_within_the_admission_budget`，这次准入已推 0 次）；
    /// 抬不动（F 已在上限或预算不够）就推一串不抬 F 的空发布（`mount::raise_rollback_floor` 给现行的 F；低于盘上现算的生效值时改给生效值）。
    /// 交回推了哪一种、几次发布。
    ///
    /// # Errors
    /// 两种推法都报错，或现行那一版没有树。
    pub fn push_one_floor_raise_or_an_empty_publish_keeping_the_floor<Device: BlockDevice>(
        session: &mut MountedSession,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<String, String> {
        let MountedSession {
            allocator,
            current,
            shadow_ledger,
            parameters_and_device_table,
            ..
        } = session;
        let parameters = parameters_and_device_table.parameters_on_disk();
        let mut version = current
            .version_of_trees()
            .ok_or_else(|| "current_without_trees".to_string())?
            .clone();
        let pushed = push_one_floor_raise_within_the_admission_budget(
            parameters,
            &mut *devices,
            allocator,
            &mut version,
            *shadow_ledger,
            0,
        );
        let text = match pushed {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => format!(
                "raised_to_the_ceiling:publishes={}:ceiling={}",
                raised.publishes.len(),
                raised.ceiling.0
            ),
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                let floor = version.root.rollback_floor;
                let kept = match crate::mount::raise_rollback_floor(
                    parameters,
                    &mut *devices,
                    allocator,
                    &mut version,
                    floor,
                    *shadow_ledger,
                ) {
                    Err(MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                        effective,
                        ..
                    }) => crate::mount::raise_rollback_floor(
                        parameters,
                        &mut *devices,
                        allocator,
                        &mut version,
                        effective,
                        *shadow_ledger,
                    ),
                    other => other,
                };
                match kept {
                    Ok(raised) => format!(
                        "kept_the_floor_after_{}:publishes={}",
                        first_word(&format!("{stop:?}")),
                        raised.publishes.len()
                    ),
                    Err(error) => {
                        *current = PoolVersion::of_a_version_of_trees(version);
                        return Err(format!(
                            "empty_publish_keeping_the_floor_failed:{}",
                            first_word(&format!("{error:?}"))
                        ));
                    }
                }
            }
            Err(error) => {
                *current = PoolVersion::of_a_version_of_trees(version);
                return Err(format!(
                    "floor_raise_failed:{}",
                    first_word(&format!("{error:?}"))
                ));
            }
        };
        *current = PoolVersion::of_a_version_of_trees(version);
        Ok(text)
    }
}
