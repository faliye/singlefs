//! 岔路 1 的族 Hh：一趟历史的状态、逐观测点的行与整趟的驱动。

use super::*;

/// 一格跑完之前的状态：设备、分配器、现行那一版与各本账。只给 [`run_hh_history`] 用。
struct HhRunState {
    pub(super) parameters: MakeFilesystemParameters,
    pub(super) devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    pub(super) allocator: PoolAllocator,
    pub(super) current: TransactionOutput,
    pub(super) instance: InstanceGeneration,
    pub(super) versions: BTreeMap<u64, PublishedVersionFacts>,
    pub(super) allocation_generations: HashMap<u64, u64>,
    lineage: TreeTableLineage,
    pub(super) injected_holes: Vec<u64>,
    pub(super) workload_publishes: u64,
    tree_table_lag_seen_real: BTreeSet<(u64, u64)>,
    tree_table_lag_seen_every_publish: BTreeSet<(u64, u64)>,
    /// 第 6 次重跑登记 E7：这次挂载钉着、还没解开的槽（值是 W16 的最旧 txg）；每次挂载整张换掉，mkfs 之后为空。
    pub(super) all_zero_pins: BTreeMap<RootRingSlot, u64>,
    /// 这次挂载新实例的第一个 txg；mkfs 之后为 `None`。
    pub(super) first_txg_of_this_mount: Option<u64>,
}

impl HhRunState {
    /// 观测：读镜像与分配器、K13 与 A10 的记账、Q1e、与锚点那一行并排，打一行 `name=q1_observation`。
    fn observe(
        &mut self,
        emitter: &mut Emitter,
        request: &HhRunRequest<'_>,
        outcome: &mut HhRunOutcome,
        point: ObservationPoint,
        self_reclaimed_real: Option<u64>,
    ) {
        let reading = read_hh_state(&HhStateInputs {
            parameters: &self.parameters,
            devices: &self.devices,
            allocator: &self.allocator,
            versions: &self.versions,
            allocation_generations: &self.allocation_generations,
            injected_holes: &self.injected_holes,
            all_zero_pins: &self.all_zero_pins,
            first_txg_of_this_mount: self.first_txg_of_this_mount,
            treat_the_hole_ring_slot_as_rewritten: request.treat_the_hole_ring_slot_as_rewritten,
        });
        // W18 (a)(b)：解开的槽从这次挂载的钉住表里拿掉，各计一次。
        for ring_slot in reading
            .all_zero_pins_released_by_write
            .iter()
            .chain(&reading.all_zero_pins_released_by_reread_only)
        {
            self.all_zero_pins.remove(ring_slot);
        }
        outcome.all_zero_pins_released_by_write +=
            u64::try_from(reading.all_zero_pins_released_by_write.len()).expect("落在 u64 内");
        outcome.all_zero_pins_released_by_reread_only +=
            u64::try_from(reading.all_zero_pins_released_by_reread_only.len())
                .expect("落在 u64 内");
        let txg = self.current.root.checkpoint_txg.0;
        let free_map_zero = self
            .allocator
            .devices
            .iter()
            .find(|map| map.device == DeviceIdentity(0))
            .expect("盘 0 在池里");
        for (slot, expected_release) in std::mem::take(&mut self.lineage.pending_release_checks) {
            outcome.tree_table_release_checks += 1;
            let released_as_expected = self.allocator.records().iter().any(|record| {
                record.device == DeviceIdentity(0)
                    && record.slot.0 == slot
                    && record.is_released
                    && record.generation.0 == expected_release
            });
            if !released_as_expected {
                outcome.tree_table_release_violations += 1;
            }
        }
        for (slot, release) in self.lineage.releases.iter().copied() {
            if release > txg {
                continue;
            }
            if !self.tree_table_lag_seen_real.contains(&(slot, release))
                && free_map_zero.is_free(SlotNumber(slot))
            {
                self.tree_table_lag_seen_real.insert((slot, release));
                outcome.tree_table_lags_real.insert(txg + 1 - release);
            }
            if !self
                .tree_table_lag_seen_every_publish
                .contains(&(slot, release))
                && release <= reading.every_publish_floor
            {
                self.tree_table_lag_seen_every_publish
                    .insert((slot, release));
                outcome
                    .tree_table_lags_every_publish
                    .insert(txg + 1 - release);
            }
        }
        let checker_on_hole = if request.judge_checker_on_hole_states && reading.missing_txgs > 0 {
            let red = is_red(&verdict(
                &format!("hole-state/{}/txg-{txg}", request.family),
                &memory_pool_snapshot(&self.devices),
                "I-3.1",
            ));
            outcome.checker_judged_hole_states += 1;
            if red {
                outcome.checker_red_on_hole_states += 1;
            }
            if red {
                "red"
            } else {
                "green"
            }
        } else {
            "not_judged"
        };
        let anchor = request.anchor.observed_row(txg).copied();
        let observation = HhObservation {
            txg,
            point,
            workload_publishes: self.workload_publishes,
            holes_injected: u64::try_from(self.injected_holes.len()).expect("落在 u64 内"),
            reading,
            self_reclaimed_real,
            anchor,
        };
        emit_observation(emitter, request, &observation, checker_on_hole);
        outcome.observations.push(observation);
    }

    /// W9：每条环里的根，装置自己那一份引用与盘上 `allocation_records_under_root` 读出的仍分配记录逐条比（两块盘各比一次）。
    fn cross_check_references_against_the_disk(&self, outcome: &mut HhRunOutcome) {
        let ring = readable_roots(
            &self.devices,
            &self.parameters.region_devices,
            &self.parameters.geometry,
            &self.parameters.filesystem_identifier,
        );
        for root in &ring {
            outcome.disk_cross_checked_roots += 1;
            let Some(facts) = self.versions.get(&root.checkpoint_txg.0) else {
                outcome.disk_cross_check_mismatches += 1;
                outcome
                    .disk_cross_check_failures
                    .push(format!("txg{}:no_version_facts", root.checkpoint_txg.0));
                continue;
            };
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                match referenced_slots_read_from_disk(&self.devices, root, device) {
                    Ok(slots) if slots == facts.referenced_slots => {}
                    Ok(slots) => {
                        outcome.disk_cross_check_mismatches += 1;
                        outcome.disk_cross_check_failures.push(format!(
                            "txg{}:device{}:disk_only={}:device_only={}",
                            root.checkpoint_txg.0,
                            device.0,
                            slots.difference(&facts.referenced_slots).count(),
                            facts.referenced_slots.difference(&slots).count()
                        ));
                    }
                    Err(failure) => {
                        outcome.disk_cross_check_mismatches += 1;
                        outcome.disk_cross_check_failures.push(format!(
                            "txg{}:device{}:unreadable:{}",
                            root.checkpoint_txg.0,
                            device.0,
                            failure.replace(' ', "_")
                        ));
                    }
                }
            }
        }
    }
}

fn emit_observation(
    emitter: &mut Emitter,
    request: &HhRunRequest<'_>,
    observation: &HhObservation,
    checker_on_hole: &str,
) {
    let reading = &observation.reading;
    let device_zero = &reading.device_zero;
    let anchor_fields = observation.anchor.map_or_else(
        || "anchor_row=missing".to_string(),
        |row| {
            format!(
                "anchor_floor={} anchor_h_que={} anchor_h_dong={} tree_table_lower_bound={}",
                row.floor, row.missing_txgs, row.live_holes, row.tree_table_lower_bound
            )
        },
    );
    let positive_control_fields = if request.emit_positive_control_no_reclaim {
        format!(
            " pc_no_reclaim_q1a_real={} pc_no_reclaim_q1a_every={} pc_no_reclaim_q1b_real={} pc_no_reclaim_q1b_every={}",
            device_zero.no_reclaim_floor_rule_real,
            device_zero.no_reclaim_floor_rule_every_publish,
            device_zero.no_reclaim_interval_rule_real,
            device_zero.no_reclaim_interval_rule_every_publish
        )
    } else {
        String::new()
    };
    emitter.emit(&format!(
        "name=q1_observation family={} {} txg={} point={} workload_publishes={} holes_injected={} h_que={} h_dong={} floor={} f_effective={} q1a_real={} q1a_every={} q1b_real={} q1b_every={} delta_real={} delta_every={} q1a_plus_real={} q1a_plus_every={} q1b_plus={} self_reclaimed_real={} allocated={} deferred={} isolated={} not_free_beyond_allocated={} devices_equal={} root_ring_installed={} k10_violations={} abandoned_roots={} roots_without_version_facts={} released_without_allocation_generation={} i31_on_hole={checker_on_hole} {anchor_fields}{positive_control_fields}",
        request.family,
        request.shape.fields(),
        observation.txg,
        observation.point.label(),
        observation.workload_publishes,
        observation.holes_injected,
        reading.missing_txgs,
        reading.live_holes,
        reading.ring_threshold,
        reading.effective_floor,
        device_zero.floor_rule_real,
        device_zero.floor_rule_every_publish,
        device_zero.interval_rule_real,
        device_zero.interval_rule_every_publish,
        observation.delta(ReclaimTiming::Real),
        observation.delta(ReclaimTiming::EveryPublish),
        device_zero.floor_rule_real_referenced_but_free,
        device_zero.floor_rule_every_publish_referenced_but_free,
        device_zero.interval_rule_referenced_but_free,
        observation
            .self_reclaimed_real
            .map_or_else(|| "not_a_publish".to_string(), |count| count.to_string()),
        reading.allocated_slots,
        reading.deferred_slots,
        reading.isolated_slots,
        reading.not_free_beyond_allocated,
        reading.devices_equal,
        reading.root_ring_installed,
        device_zero.reclaim_rule_violations,
        reading.abandoned_roots,
        reading.roots_without_version_facts,
        device_zero.released_records_without_allocation_generation,
    ));
}

/// 岔路 1 一格：mkfs（产品路径分配器）→ 取号 → 暖机 → 新池新建文件（txg 3）→ 工作负载，在锚点模型给的洞 txg 上造 c2 并可写挂载，
/// 跑到锚点模型给的终点。任一次发布或挂载报错即截断（W8）。
#[allow(
    clippy::too_many_lines,
    reason = "一格的主历史（mkfs、新池新建文件、工作负载、c2 与挂载、逐观测点）连着写才对得上登记 5.3 的骨架逐行"
)]
pub(super) fn run_hh_history(emitter: &mut Emitter, request: &HhRunRequest<'_>) -> HhRunOutcome {
    let shape = request.shape;
    let parameters = parameters_with_slots_per_region(shape.slots_per_region);
    let mut outcome = HhRunOutcome {
        family: request.family,
        shape,
        allocator_source: request.allocator_source,
        cut_point: request.cut_point,
        registered_end: request.anchor.end,
        actual_end: 0,
        truncation: None,
        observations: Vec::new(),
        recoveries: Vec::new(),
        genesis_root_in_ring: false,
        slots_per_region_read_back: 0,
        new_pool_file_creation_matches_the_registered_anchor: false,
        ring_wrap_overwrites_checked: 0,
        ring_wrap_overwrite_mismatches: 0,
        tree_table_release_checks: 0,
        tree_table_release_violations: 0,
        mount_schedule_mismatches: 0,
        disk_cross_checked_roots: 0,
        disk_cross_check_mismatches: 0,
        disk_cross_check_failures: Vec::new(),
        admission_refusals: 0,
        tree_table_lags_real: BTreeSet::new(),
        tree_table_lags_every_publish: BTreeSet::new(),
        checker_red_on_hole_states: 0,
        checker_judged_hole_states: 0,
        all_zero_pin_mounts: Vec::new(),
        all_zero_pins_released_by_write: 0,
        all_zero_pins_released_by_reread_only: 0,
    };
    let ring_length = E156_ROOT_RING_REGIONS * shape.slots_per_region;
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("Hh mkfs");
    outcome.genesis_root_in_ring = readable_roots(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .iter()
    .any(|root| root.checkpoint_txg.0 == 0);
    outcome.slots_per_region_read_back = choose_system_configuration(&devices)
        .map(|configuration| {
            configuration
                .immutable
                .sizes
                .root_ring_slots_per_region
                .count()
        })
        .unwrap_or(0);
    let mut allocator = allocator_for_hh(request.allocator_source, &parameters, &devices, &genesis);
    let genesis_facts = facts_of_zero_unit_version(&genesis.root);
    let mut versions: BTreeMap<u64, PublishedVersionFacts> = BTreeMap::new();
    for zero_unit_txg in 0..=2u64 {
        versions.insert(zero_unit_txg, genesis_facts.clone());
    }
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("Hh 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("Hh 暖机")
    };
    let mut allocation_generations: HashMap<u64, u64> = HashMap::new();
    let before_first = snapshot_allocated_generations(&allocator);
    let first = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
    };
    let first = match first {
        Ok(first) => first,
        Err(error) => {
            outcome.truncation = Some((3, format!("{error:?}")));
            emit_cell_summary(emitter, &outcome);
            return outcome;
        }
    };
    record_release_generations(&mut allocation_generations, &before_first, &allocator);
    outcome.new_pool_file_creation_matches_the_registered_anchor =
        new_pool_file_creation_matches_the_registered_anchor(&allocator, &first);
    let mut state = HhRunState {
        parameters,
        devices,
        allocator,
        current: first.clone(),
        instance,
        versions,
        allocation_generations,
        lineage: TreeTableLineage {
            previous_tree_table_slot: genesis_facts.tree_table_slot,
            pending_release_checks: Vec::new(),
            releases: Vec::new(),
        },
        injected_holes: Vec::new(),
        workload_publishes: 0,
        tree_table_lag_seen_real: BTreeSet::new(),
        tree_table_lag_seen_every_publish: BTreeSet::new(),
        all_zero_pins: BTreeMap::new(),
        first_txg_of_this_mount: None,
    };
    state.lineage.note_version(&mut state.versions, &first);
    state.observe(
        emitter,
        request,
        &mut outcome,
        ObservationPoint::NewPoolFileCreation,
        None,
    );

    let publishes_per_overwrite = shape.density.publishes_per_overwrite();
    let mut workload_index_in_segment = 0u64;
    let mut content_salt = 0u64;
    let mut overwrites_before_the_ring_wraps = 0u64;
    while state.current.root.checkpoint_txg.0 < request.anchor.end {
        let next_txg = state.current.root.checkpoint_txg.0 + 1;
        let kind = if workload_index_in_segment.is_multiple_of(publishes_per_overwrite) {
            WorkloadPublishKind::Overwrite
        } else {
            WorkloadPublishKind::EmptyPublish
        };
        content_salt += 1;
        if request.anchor.holes.contains(&next_txg) {
            let before_crash = snapshot_allocated_generations(&state.allocator);
            let crashed = match crash_publish_before_its_root_persists(
                &state.parameters,
                &state.devices,
                &state.allocator,
                &state.current,
                state.instance,
                kind,
                content_salt,
                request.cut_point,
            ) {
                Ok(crashed) => crashed,
                Err(error) => {
                    if matches!(error, PublishError::SpaceAdmissionRefused(_)) {
                        outcome.admission_refusals += 1;
                    }
                    outcome.truncation = Some((next_txg, format!("{error:?}")));
                    break;
                }
            };
            record_release_generations(
                &mut state.allocation_generations,
                &before_crash,
                &crashed.allocator_after_the_crashed_publish,
            );
            state
                .lineage
                .note_version(&mut state.versions, &crashed.crashed_output);
            state.injected_holes.push(next_txg);
            let mut recovered_devices = devices_from_memory_pool(&crashed.cut_image);
            let (slots_reading_back_all_zero_at_mount, bad_slots_at_mount) =
                root_ring_slots_reading_back_all_zero_at_mount(
                    &recovered_devices,
                    &state.parameters,
                    ring_length,
                );
            let mounted = match mount_writable(&state.parameters, &mut recovered_devices) {
                Ok(mounted) => mounted,
                Err(error) => {
                    outcome.truncation = Some((next_txg, format!("{error:?}")));
                    break;
                }
            };
            state.devices = recovered_devices;
            let mut mount_versions: Vec<TransactionOutput> = Vec::new();
            if let Some(row) = mounted.output.row_publish.file_version() {
                mount_versions.push(row.clone());
            }
            for warm in &mounted.output.warm_up_publishes {
                if let Some(version) = warm.file_version() {
                    mount_versions.push(version.clone());
                }
            }
            let floor_raise_sequences = mounted.output.space_admission.floor_raises();
            for raised in floor_raise_sequences {
                mount_versions.extend(raised.publishes.iter().cloned());
            }
            let mut before_mount =
                snapshot_allocated_generations(&crashed.allocator_after_the_crashed_publish);
            for version in &mount_versions {
                for record in version
                    .allocation_records
                    .iter()
                    .filter(|record| !record.is_released && record.device == DeviceIdentity(0))
                {
                    before_mount.insert(record.slot.0, record.generation.0);
                }
            }
            record_release_generations(
                &mut state.allocation_generations,
                &before_mount,
                &mounted.allocator,
            );
            for version in &mount_versions {
                state.lineage.note_version(&mut state.versions, version);
            }
            let first_txg_of_the_new_instance = mounted.output.row_publish.root().checkpoint_txg.0;
            // 第 6 次重跑登记 E8：钉住表整张换成这次挂载读回全零的槽（W16、W17）。
            let all_zero_pins: BTreeMap<RootRingSlot, u64> = slots_reading_back_all_zero_at_mount
                .iter()
                .filter_map(|ring_slot| {
                    oldest_txg_an_all_zero_slot_can_have_held(
                        *ring_slot,
                        first_txg_of_the_new_instance,
                        ring_length,
                        &state.parameters,
                    )
                    .map(|oldest_txg| (*ring_slot, oldest_txg))
                })
                .collect();
            state.all_zero_pins.clone_from(&all_zero_pins);
            state.first_txg_of_this_mount = Some(first_txg_of_the_new_instance);
            let all_zero_pins_at_mount = AllZeroPinsAtMount {
                hole_txg: next_txg,
                first_txg_of_the_new_instance,
                pinned: all_zero_pins,
                bad_slots_at_mount,
            };
            let publishes_in_the_mount = 1
                + u64::try_from(mounted.output.warm_up_publishes.len()).expect("落在 u64 内")
                + floor_raise_sequences
                    .iter()
                    .map(|raised| u64::try_from(raised.publishes.len()).expect("落在 u64 内"))
                    .sum::<u64>();
            let mount_admission = mount_admission_label(&mounted.output.space_admission);
            let mount_floor_raise_sequences =
                u64::try_from(floor_raise_sequences.len()).expect("落在 u64 内");
            let prefix_applied = mounted.output.journal.prefix_applied;
            state.allocator = mounted.allocator;
            state.instance = mounted.output.instance;
            state.current = mounted
                .current
                .into_file_version()
                .expect("Hh c2 恢复之后现行版本带文件");
            let last_txg_of_the_mount = state.current.root.checkpoint_txg.0;
            let hole_index = state.injected_holes.len() - 1;
            let planned_end = request.anchor.recovery_ends.get(hole_index).copied();
            if first_txg_of_the_new_instance != next_txg + 1
                || planned_end != Some(last_txg_of_the_mount)
                || planned_end.map(|end| end - next_txg) != Some(publishes_in_the_mount)
            {
                outcome.mount_schedule_mismatches += 1;
            }
            let ring_has_a_root_at_the_hole_txg = readable_roots(
                &state.devices,
                &state.parameters.region_devices,
                &state.parameters.geometry,
                &state.parameters.filesystem_identifier,
            )
            .iter()
            .any(|root| root.checkpoint_txg.0 == next_txg);
            let recovery = HoleRecovery {
                hole_txg: next_txg,
                crashed_publish_kind: kind,
                prefix_applied,
                first_txg_of_the_new_instance,
                last_txg_of_the_mount,
                publishes_in_the_mount,
                ring_has_a_root_at_the_hole_txg,
                hole_slot_content: describe_the_root_ring_slot_of_txg(
                    &state.devices,
                    &state.parameters,
                    next_txg,
                ),
                cut_matches_the_restored_image: crashed.cut_matches_the_restored_image,
                sectors_differing_from_the_root_slot_only_image: crashed
                    .sectors_differing_from_the_root_slot_only_image
                    .clone(),
                steps_after_the_root_slot_write: crashed.steps_after_the_root_slot_write.clone(),
                root_slot_writes_in_the_crashed_publish: crashed.root_slot_writes_in_the_publish,
                mount_admission,
                mount_floor_raise_sequences,
            };
            emitter.emit(&format!(
                "name=hole_recovery family={} {} hole_txg={} crashed_publish={} prefix_applied={} first_txg_of_new_instance={} last_txg_of_mount={} publishes_in_mount={} planned_last_txg_of_mount={} ring_has_root_at_hole_txg={} hole_slot_content={} cut_point={:?} cut_matches_root_and_system_configuration_restored_image={} sectors_differing_from_root_slot_only_restored_image={} steps_after_root_slot_write={} root_slot_writes_in_crashed_publish={} mount_admission={} mount_floor_raise_sequences={}",
                request.family,
                shape.fields(),
                recovery.hole_txg,
                recovery.crashed_publish_kind.label(),
                recovery.prefix_applied,
                recovery.first_txg_of_the_new_instance,
                recovery.last_txg_of_the_mount,
                recovery.publishes_in_the_mount,
                planned_end.map_or_else(|| "none".to_string(), |end| end.to_string()),
                recovery.ring_has_a_root_at_the_hole_txg,
                recovery.hole_slot_content,
                request.cut_point,
                recovery.cut_matches_the_restored_image,
                if recovery.sectors_differing_from_the_root_slot_only_image.is_empty() {
                    "none".to_string()
                } else {
                    recovery
                        .sectors_differing_from_the_root_slot_only_image
                        .iter()
                        .map(|(device, offset)| format!("d{device}@{offset}"))
                        .collect::<Vec<_>>()
                        .join(",")
                },
                if recovery.steps_after_the_root_slot_write.is_empty() {
                    "none".to_string()
                } else {
                    recovery.steps_after_the_root_slot_write.join(",")
                },
                recovery.root_slot_writes_in_the_crashed_publish,
                recovery.mount_admission,
                recovery.mount_floor_raise_sequences,
            ));
            emitter.emit(&format!(
                "name=all_zero_pins_at_mount family={} {} hole_txg={} first_txg={} pinned=[{}] bad_slots_at_mount={}",
                request.family,
                shape.fields(),
                all_zero_pins_at_mount.hole_txg,
                all_zero_pins_at_mount.first_txg_of_the_new_instance,
                all_zero_pins_at_mount
                    .pinned
                    .iter()
                    .map(|(ring_slot, txg)| format!("({},{},{txg})", ring_slot.region, ring_slot.slot))
                    .collect::<Vec<_>>()
                    .join(","),
                all_zero_pins_at_mount.bad_slots_at_mount,
            ));
            outcome.all_zero_pin_mounts.push(all_zero_pins_at_mount);
            outcome.recoveries.push(recovery);
            workload_index_in_segment = 0;
            state.cross_check_references_against_the_disk(&mut outcome);
            state.observe(
                emitter,
                request,
                &mut outcome,
                ObservationPoint::MountReturn,
                None,
            );
        } else {
            let before_publish = snapshot_allocated_generations(&state.allocator);
            let released_not_reclaimed_before =
                released_not_reclaimed_slots_on_device_zero(&state.allocator);
            let records_before = state.allocator.records().len();
            let published = publish_one_workload(
                &state.parameters,
                state.devices.as_mut_slice(),
                &mut state.allocator,
                &state.current,
                state.instance,
                kind,
                content_salt,
            );
            let published = match published {
                Ok(published) => published,
                Err(error) => {
                    if matches!(error, PublishError::SpaceAdmissionRefused(_)) {
                        outcome.admission_refusals += 1;
                    }
                    outcome.truncation = Some((next_txg, format!("{error:?}")));
                    break;
                }
            };
            state.current = published;
            record_release_generations(
                &mut state.allocation_generations,
                &before_publish,
                &state.allocator,
            );
            let current = state.current.clone();
            state.lineage.note_version(&mut state.versions, &current);
            workload_index_in_segment += 1;
            state.workload_publishes += 1;
            if request.check_overwrites_before_the_ring_wraps
                && kind == WorkloadPublishKind::Overwrite
                && overwrites_before_the_ring_wraps < E156_OVERWRITES_CHECKED_BEFORE_THE_RING_WRAPS
            {
                overwrites_before_the_ring_wraps += 1;
                outcome.ring_wrap_overwrites_checked += 1;
                let released = self_release_slots_of_this_publish(
                    &state.allocator,
                    DeviceIdentity(0),
                    state.current.root.checkpoint_txg,
                );
                let record_delta = u64::try_from(state.allocator.records().len() - records_before)
                    .expect("这一步的记录增量落在 u64 内");
                if released != E156_OVERWRITE_RELEASED_SLOTS_BEFORE_THE_RING_WRAPS
                    || record_delta != E156_OVERWRITE_RECORD_DELTA_BEFORE_THE_RING_WRAPS
                {
                    outcome.ring_wrap_overwrite_mismatches += 1;
                }
                emitter.emit(&format!(
                    "name=ring_wrap_overwrite_check family={} {} txg={} released_d0={released} record_delta={record_delta} registered_released_d0={E156_OVERWRITE_RELEASED_SLOTS_BEFORE_THE_RING_WRAPS} registered_record_delta={E156_OVERWRITE_RECORD_DELTA_BEFORE_THE_RING_WRAPS}",
                    request.family,
                    shape.fields(),
                    state.current.root.checkpoint_txg.0
                ));
            }
            let free_map_zero = state
                .allocator
                .devices
                .iter()
                .find(|map| map.device == DeviceIdentity(0))
                .expect("盘 0 在池里");
            let self_reclaimed = released_not_reclaimed_before
                .iter()
                .filter(|slot| free_map_zero.is_free(SlotNumber(**slot)))
                .count();
            state.observe(
                emitter,
                request,
                &mut outcome,
                ObservationPoint::Workload(kind),
                Some(u64::try_from(self_reclaimed).expect("落在 u64 内")),
            );
        }
    }
    outcome.actual_end = state.current.root.checkpoint_txg.0;
    state.cross_check_references_against_the_disk(&mut outcome);
    emit_cell_summary(emitter, &outcome);
    outcome
}

#[cfg(test)]
mod tests;
