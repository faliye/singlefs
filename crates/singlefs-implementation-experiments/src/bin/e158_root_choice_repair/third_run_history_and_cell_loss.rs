//! 第 3 次跑的历史、被测挂载的汇总与每一格的丢失判定。

use super::*;

pub(super) fn key_text(key: Option<TimelineRoot>) -> String {
    key.map_or("none".to_string(), |(instance, txg)| {
        format!("{instance}:{txg}")
    })
}

// ---------------------------------------------------------------------------
// 11.3 历史与确认账（登记「读法写死」表「确认」一行）：每次改内容的入口调用交回 Ok 记一笔（内容号、那一版的根键、那一版引用的
//      每个单元在每块盘上的字节）；空发布、写行、暖机、抬 F 沿用上一版的内容号。内容号 k 的文件字节 = `deterministic_content(999 + k)`。
// ---------------------------------------------------------------------------

/// 一版引用的每个单元（盘, 起始槽, 槽数）在某一刻盘上的字节。
pub(super) type UnitBytesOfAVersion = BTreeMap<(u32, u64, u64), Vec<u8>>;

#[derive(Clone)]
pub(super) struct ThirdRunConfirmation {
    pub(super) content_number: u64,
    pub(super) root: TimelineRoot,
    /// 那一版引用的每个单元确认那一刻在盘上的字节。
    pub(super) unit_bytes: UnitBytesOfAVersion,
}

#[derive(Clone)]
pub(super) struct ThirdRunHistory {
    pub(super) pool: MemoryPool,
    pub(super) session: Option<Session>,
    pub(super) timeline: Vec<TimelineRoot>,
    pub(super) content_number_by_root: BTreeMap<TimelineRoot, u64>,
    pub(super) confirmations: Vec<ThirdRunConfirmation>,
    /// 7.1 第一行（F3）核过的确认次数与不符的次数。
    pub(super) tail_checks: u64,
    pub(super) tail_check_failures: Vec<String>,
}

impl ThirdRunHistory {
    pub(super) fn tip(&self) -> TimelineRoot {
        *self.timeline.last().expect("时间线至少有第 0 代根")
    }

    pub(super) fn last_confirmed_content_number(&self) -> u64 {
        self.confirmations
            .last()
            .map_or(0, |confirmation| confirmation.content_number)
    }

    pub(super) fn content_number_of(&self, root: TimelineRoot) -> Option<u64> {
        self.content_number_by_root.get(&root).copied()
    }

    pub(super) fn push_carried(&mut self, root: TimelineRoot, content_number: u64) {
        self.timeline.push(root);
        self.content_number_by_root.insert(root, content_number);
    }
}

pub(super) fn third_run_content(content_number: u64) -> Vec<u8> {
    deterministic_content(999 + content_number, FIRST_FILE_BYTES)
}

/// 读回的文件字节是第几号内容（不是任何一号就 None）。
pub(super) fn content_number_of_bytes(bytes: &[u8], highest: u64) -> Option<u64> {
    (1..=highest).find(|number| third_run_content(*number) == bytes)
}

pub(super) fn unit_bytes_referenced_by(
    pool: &MemoryPool,
    root: &RootRecord,
) -> Result<UnitBytesOfAVersion, String> {
    let placements = placements_referenced_by_root_on(pool, root)?;
    Ok(placements
        .into_iter()
        .map(|(device, slot, span)| {
            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
            let bytes = pool
                .read(device, slot * SLOT_BYTES_LOCAL, length)
                .unwrap_or_default();
            ((device, slot, span), bytes)
        })
        .collect())
}

/// 7.1 第一行（F3）：一次确认之后，每块盘世代号最新的自证系统配置槽里 tail = 那次发布末条记录的计数器、实例代号 = 那次发布的实例代号。
fn check_the_tail_after_a_confirmation(history: &mut ThirdRunHistory, root: TimelineRoot) {
    history.tail_checks += 1;
    let Ok(geometry) = independent_geometry(&history.pool) else {
        history
            .tail_check_failures
            .push(format!("{root:?}: 几何解不出"));
        return;
    };
    let decoded = device_decode(&history.pool, &geometry);
    let last_counter = decoded
        .records
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == root
                && record_ends_its_publish_by_the_device(record)
        })
        .map(|record| record.counter)
        .max();
    for device in [0u32, 1] {
        let newest = decoded
            .system_configuration_slots
            .iter()
            .filter(|(slot_device, _, _, _)| *slot_device == device)
            .max_by_key(|(_, generation, _, _)| *generation);
        let holds = newest.is_some_and(|(_, _, tail, instance)| {
            Some(*tail) == last_counter && *instance == root.0
        });
        if !holds {
            history.tail_check_failures.push(format!(
                "{root:?} 盘 {device}: 最新槽 {newest:?}，那次发布末条记录计数器 {last_counter:?}"
            ));
        }
    }
}

pub(super) fn record_confirmation(
    history: &mut ThirdRunHistory,
    root_record: &RootRecord,
    content_number: u64,
) -> Result<(), String> {
    let root = root_pair(root_record);
    let unit_bytes = unit_bytes_referenced_by(&history.pool, root_record)?;
    history.push_carried(root, content_number);
    history.confirmations.push(ThirdRunConfirmation {
        content_number,
        root,
        unit_bytes,
    });
    check_the_tail_after_a_confirmation(history, root);
    Ok(())
}

/// 一次可写挂载写下的根（写行、暖机、抬 F）进时间线，沿用 `content_number`。
pub(super) fn push_the_publishes_of_a_mount(
    history: &mut ThirdRunHistory,
    mounted: &Mounted,
    content_number: u64,
) {
    let mut roots = vec![root_pair(mounted.output.row_publish.root())];
    roots.extend(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| root_pair(publish.root())),
    );
    for raised in mounted.output.space_admission.floor_raises() {
        roots.extend(
            raised
                .publishes
                .iter()
                .map(|publish| root_pair(&publish.root)),
        );
    }
    for root in roots {
        history.push_carried(root, content_number);
    }
}

/// 在开着的会话上做一次改内容的调用（现行那一版有文件就覆盖写，没有就写首个文件），交回 Ok 就记一笔确认。
pub(super) fn third_run_change_content(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<(), String> {
    let content_number = history.last_confirmed_content_number() + 1;
    let content = third_run_content(content_number);
    let mut session = history.session.take().ok_or("改内容：没有开着的会话")?;
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let file = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * content_number,
        };
        match &session.current {
            PoolVersion::WithFile(previous) | PoolVersion::WithTreesAndNoFile(previous) => {
                publish_overwrite(
                    &mut writer,
                    &mut session.allocator,
                    previous,
                    file,
                    session.instance,
                )
                .map_err(|error| format!("覆盖写: {error:?}"))?
            }
            PoolVersion::WithoutFile(version) => publish_first_file(
                &mut writer,
                &mut session.allocator,
                &version.root,
                file,
                session.instance,
                session.current.record_bytes(),
            )
            .map_err(|error| format!("首个文件: {error:?}"))?,
        }
    };
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let root_record = output.root;
    session.current = PoolVersion::WithFile(std::sync::Arc::new(output));
    history.session = Some(session);
    record_confirmation(history, &root_record, content_number)
}

/// P(n1, ·) 的前半：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写，会话开着。
pub(super) fn third_run_history_start(
    geometry: &Geometry,
    overwrites: u64,
) -> Result<ThirdRunHistory, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let mounted = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("第一次可写挂载: {error:?}"))?;
    let mut history = ThirdRunHistory {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: None,
        timeline: vec![root_pair(&genesis.root)],
        content_number_by_root: BTreeMap::from([(root_pair(&genesis.root), 0)]),
        confirmations: Vec::new(),
        tail_checks: 0,
        tail_check_failures: Vec::new(),
    };
    push_the_publishes_of_a_mount(&mut history, &mounted, 0);
    history.session = Some(Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    });
    for _ in 0..=overwrites {
        third_run_change_content(&mut history, &parameters)?;
    }
    Ok(history)
}

/// 关闭 c（同第 2 次跑）：C 丢掉会话；U 调 `unmount`，并核 7.1 第六行（F14）：F 抬到现行那一版的 txg、空发布 2 或 3 次。
/// 交回 F14 那一格的判定行（C 交 None）。
pub(super) fn third_run_close(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
    closing: Closing,
) -> Result<Option<(bool, String)>, String> {
    let Some(session) = history.session.take() else {
        return Ok(None);
    };
    if closing == Closing::ProcessExit {
        return Ok(None);
    }
    let Session {
        mut allocator,
        mut current,
        ..
    } = session;
    let before_txg = current.root().checkpoint_txg.0;
    let carried = history.content_number_of(history.tip()).unwrap_or(0);
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let unmounted = singlefs_core::mount::unmount(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .map_err(|error| format!("正常卸载: {error:?}"))?;
    let singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        return Err("正常卸载：现行那一版没有文件".to_string());
    };
    for publish in &raised.publishes {
        history.push_carried(root_pair(&publish.root), carried);
    }
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let expected_publishes = if before_txg % 3 == 1 { 3 } else { 2 };
    let floors = [0u32, 1].map(|device| {
        let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
        newest_system_configuration_slot_bytes(&history.pool, DeviceIdentity(device), slot_spacing)
            .and_then(|bytes| {
                singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes).ok()
            })
            .map(|parsed| parsed.quantities.rollback_floor.0)
    });
    let holds = floors.iter().all(|floor| *floor == Some(before_txg))
        && raised.publishes.len() == expected_publishes;
    Ok(Some((
        holds,
        format!(
            "txg_before_unmount={before_txg} floors={floors:?} unmount_publishes={} expected_publishes={expected_publishes}",
            raised.publishes.len()
        ),
    )))
}

/// 抛弃步 A 的落点（登记 5.3「抛弃步 A」）：当前时间线上最新那 1 条根的根槽，加它那次发布按 b 取的落点
/// （b = records：那次发布全部记录的全部份；b = unit_first_copy：它第一条记录点名的第一个单元、位置条目第一条那一份）。
/// 复制自第 2 次跑的 `unreadable_ranges_of_the_abandonment`（那一份吃 `SecondRunHistory`，这里吃第 3 次跑的历史），语义不变。
pub(super) fn third_run_abandonment_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    form: UnreadableForm,
    replay_break: ReplayBreak,
    duration: FaultDuration,
) -> Result<Option<Vec<ThirdRunFault>>, String> {
    let hidden = history.tip();
    let (root_slot_device, root_slot_offset) = root_slot_of(&history.pool, geometry, hidden)
        .ok_or_else(|| format!("最新那条根 {hidden:?} 在根环里读不出"))?;
    let mut faults = vec![ThirdRunFault {
        device: root_slot_device,
        offset: root_slot_offset,
        length: u64::from(geometry.physical_block_size),
        form,
        duration,
        label: format!("root{}", key_text(Some(hidden))),
    }];
    let records = records_of_publish(&history.pool, geometry, hidden);
    if records.is_empty() {
        return Ok(None);
    }
    match replay_break {
        ReplayBreak::EveryRecord => {
            for (counter, record_offset_on_the_device, _) in &records {
                for record_device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    faults.push(ThirdRunFault {
                        device: record_device,
                        offset: *record_offset_on_the_device,
                        length: JOURNAL_RECORD_BYTES,
                        form,
                        duration,
                        label: format!("record(counter={counter},device={})", record_device.0),
                    });
                }
            }
        }
        ReplayBreak::FirstCopyOfTheFirstNamedUnit | ReplayBreak::FirstNamedUnit => {
            let Some(named) = records
                .iter()
                .find_map(|(_, _, record)| record.named.first())
            else {
                return Ok(None);
            };
            let copies = match replay_break {
                ReplayBreak::FirstNamedUnit => named.locations.len(),
                ReplayBreak::EveryRecord | ReplayBreak::FirstCopyOfTheFirstNamedUnit => 1,
            };
            for location in named.locations.iter().take(copies) {
                faults.push(ThirdRunFault {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form,
                    duration,
                    label: format!(
                        "unit(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                });
            }
        }
    }
    Ok(Some(faults))
}

// ---------------------------------------------------------------------------
// 11.4 一格的三样（登记第六节 Q1–Q3、③c 尾巴）与被测挂载的上下文（Q0、Q2、Q5 的读数、Q7）。
// ---------------------------------------------------------------------------

/// 被测那次挂载（A）的一次调用的摘要（Q7、Q5 的读数、Q2 的 E）。
pub(super) struct TestedMountSummary {
    pub(super) class: &'static str,
    pub(super) member: String,
    pub(super) writes: u64,
    pub(super) reads: u64,
    pub(super) read_bytes: u64,
    pub(super) waits: u64,
    pub(super) chosen: Option<TimelineRoot>,
    pub(super) effective: Option<TimelineRoot>,
    pub(super) abandoned_roots_unreadable: Option<u64>,
    pub(super) isolated: Option<u64>,
    pub(super) observation: Option<ArmObservation>,
}

impl TestedMountSummary {
    pub(super) fn of(
        call: &ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    ) -> Self {
        let mounted = call.outcome.as_ref().ok();
        TestedMountSummary {
            class: third_run_outcome_class(&call.outcome, call.write_calls),
            member: mount_error_member_or_none(&call.outcome),
            writes: call.write_calls,
            reads: call.read_calls,
            read_bytes: call.read_bytes,
            waits: call.waits,
            chosen: mounted.map(|mounted| root_pair(&mounted.output.chosen_root)),
            effective: mounted.map(|mounted| root_pair(&mounted.output.effective_root)),
            abandoned_roots_unreadable: mounted
                .map(|mounted| mounted.output.abandoned_roots_unreadable),
            isolated: mounted.map(|mounted| {
                mounted
                    .output
                    .isolated_slots_per_device
                    .iter()
                    .map(|(_, slots)| *slots)
                    .sum()
            }),
            observation: call.observation.clone(),
        }
    }

    pub(super) fn render(&self, prefix: &str) -> String {
        let observation = self.observation.as_ref().map_or("none".to_string(), |observation| {
            format!(
                "judged:{},newer:{},tail:{},last:{},undecidable:{},unreadable_slots:{},rounds:{},rebuilt:{},table_unreadable:{},table_rereads:{}",
                observation.witness_judged,
                observation.newer_root_on_the_first_pass,
                observation.witnessed_tail,
                observation
                    .selected_version_last_counter
                    .map_or("none".to_string(), |counter| counter.to_string()),
                observation.undecidable,
                observation.unreadable_ring_slots_on_the_first_pass,
                observation.reread_rounds,
                observation.rebuilt_from_records,
                observation.shadow_ledger_table_unreadable_on_the_first_read,
                observation.shadow_ledger_table_rereads
            )
        });
        format!(
            "{prefix}_class={} {prefix}_error={} {prefix}_writes={} {prefix}_reads={} {prefix}_read_bytes={} {prefix}_waits={} {prefix}_chosen={} {prefix}_effective={} {prefix}_abandoned_roots_unreadable={} {prefix}_isolated={} {prefix}_arm_observation={observation}",
            self.class,
            self.member,
            self.writes,
            self.reads,
            self.read_bytes,
            self.waits,
            key_text(self.chosen),
            key_text(self.effective),
            self.abandoned_roots_unreadable
                .map_or("none".to_string(), |count| count.to_string()),
            self.isolated.map_or("none".to_string(), |count| count.to_string()),
        )
    }
}

/// 一格历史末尾的三样（Q1 三栏、Q3、③c）。
pub(super) struct CellLoss {
    pub(super) in_ring_overwritten: u64,
    pub(super) out_of_ring_overwritten: u64,
    pub(super) landing_overwritten: String,
    pub(super) read_back: String,
    read_back_root: Option<TimelineRoot>,
    pub(super) tail_class: String,
}

impl CellLoss {
    pub(super) fn overwrite_cell(&self) -> bool {
        self.in_ring_overwritten + self.out_of_ring_overwritten > 0
    }

    pub(super) fn read_back_lost(&self) -> bool {
        self.read_back.starts_with("older") || self.read_back.starts_with("failed")
    }

    fn tail_lost(&self) -> bool {
        self.tail_class.starts_with("failed")
    }

    pub(super) fn lost_write_cell(&self) -> bool {
        self.overwrite_cell() || self.read_back_lost() || self.tail_lost()
    }

    pub(super) fn render(&self) -> String {
        format!(
            "q1_in_ring_overwritten={} q1_out_of_ring_overwritten={} q1_landing_overwritten={} overwrite_cell={} q3_read_back={} q3_read_back_root={} q3c_tail={} lost_write_cell={}",
            self.in_ring_overwritten,
            self.out_of_ring_overwritten,
            self.landing_overwritten,
            self.overwrite_cell(),
            self.read_back,
            key_text(self.read_back_root),
            self.tail_class,
            self.lost_write_cell()
        )
    }
}

/// 冷启动读回按内容号分四类（Q3）：last_confirmed / newer / older / failed:<成员>。
fn classify_read_back(
    outcome: &RecoveryOutcome,
    last_confirmed: u64,
    highest_content_number: u64,
) -> String {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            match content_number_of_bytes(content, highest_content_number) {
                Some(number) if number == last_confirmed => "last_confirmed".to_string(),
                Some(number) if number > last_confirmed => format!("newer:{number}"),
                Some(number) => format!("older:{number}"),
                None => "failed:content_not_in_the_ledger".to_string(),
            }
        }
        RecoveryOutcome::NoFile { .. } => {
            if last_confirmed == 0 {
                "last_confirmed".to_string()
            } else {
                "older:0".to_string()
            }
        }
        RecoveryOutcome::Failed { failure, .. } => {
            format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
        }
    }
}

/// 历史末尾（`end`）上算一格的三样。`protected` = 被测那次挂载之前最后确认那一版；`base` = 被测那次挂载开始时的镜像；
/// `first_instance_at_or_after_the_tested_mount` = ③c 要读坏其根槽的最小实例代号（`None` = 这一格不做 ③c）。
pub(super) fn evaluate_cell_loss(
    history: &ThirdRunHistory,
    protected: &ThirdRunConfirmation,
    base: &MemoryPool,
    end: &MemoryPool,
    first_instance_at_or_after_the_tested_mount: Option<u32>,
) -> CellLoss {
    let geometry = independent_geometry(end);
    let end_roots = geometry
        .as_ref()
        .map(|geometry| readable_roots_independent(end, geometry))
        .unwrap_or_default();
    let highest = history.last_confirmed_content_number();
    let report = recover(&devices_from_pool(end, IMAGE_BYTES), JournalPolicy::Consult);
    let read_back_root = chosen_root_of(&report.outcome);
    let read_back = classify_read_back(&report.outcome, highest, highest);
    // 最后那次冷启动恢复落到的根（第 2 次跑的 ①）：它在被测那次挂载开始时的镜像上就是一条自证根时，取它那一版引用的单元在那一刻的字节。
    let base_geometry = independent_geometry(base);
    let landing_in_base = match (read_back_root, &base_geometry) {
        (Some(landing), Ok(base_pool_geometry)) => {
            readable_roots_independent(base, base_pool_geometry).contains(&landing)
        }
        (Some(_) | None, _) => false,
    };
    let landing_unit_bytes: Result<UnitBytesOfAVersion, String> =
        match (read_back_root, &base_geometry) {
            (Some(landing), Ok(base_pool_geometry)) if landing_in_base => {
                root_record_of(base, base_pool_geometry, landing)
                    .ok_or_else(|| "根记录读不出".to_string())
                    .and_then(|record| unit_bytes_referenced_by(base, &record))
            }
            (Some(_) | None, _) => Ok(BTreeMap::new()),
        };
    let overwritten_in = |version: &UnitBytesOfAVersion| -> u64 {
        version
            .iter()
            .filter(|((device, slot, span), bytes)| {
                let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
                end.read(*device, slot * SLOT_BYTES_LOCAL, length)
                    .unwrap_or_default()
                    != **bytes
            })
            .count() as u64
    };
    // Q1 前两栏比的是「被测那次挂载之前最后确认那一版」引用的单元（登记第六节 Q1）。
    let compared_version = &protected.unit_bytes;
    let overwritten = overwritten_in(compared_version);
    let protected_in_ring = end_roots.contains(&protected.root);
    let (in_ring_overwritten, out_of_ring_overwritten) = if protected_in_ring {
        (overwritten, 0)
    } else {
        (0, overwritten)
    };
    let landing_overwritten = match (&landing_unit_bytes, read_back_root) {
        (Ok(version), Some(_)) if landing_in_base => overwritten_in(version).to_string(),
        (Err(error), Some(_)) => format!("unreadable_in_base:{}", error.replace(' ', "_")),
        (Ok(_), Some(_)) => "written_after_the_tested_mount".to_string(),
        (Ok(_) | Err(_), None) => "none".to_string(),
    };
    let tail_class = match (first_instance_at_or_after_the_tested_mount, geometry) {
        (Some(first_instance), Ok(geometry)) => {
            let slots = checker_image::root_slot_positions(&geometry);
            let faults: Vec<ThirdRunFault> = checker_image::valid_roots(end, &geometry)
                .into_iter()
                .filter(|(_, _, view)| view.instance >= first_instance)
                .filter_map(|(region, slot, view)| {
                    slots
                        .iter()
                        .find(|(candidate_region, candidate_slot, _, _)| {
                            *candidate_region == region && *candidate_slot == slot
                        })
                        .map(|(_, _, device, offset)| ThirdRunFault {
                            device: DeviceIdentity(*device),
                            offset: *offset,
                            length: u64::from(geometry.physical_block_size),
                            form: UnreadableForm::ReadFails,
                            duration: FaultDuration::WholeCall,
                            label: format!(
                                "tail_root{}",
                                key_text(Some((view.instance, view.checkpoint_txg)))
                            ),
                        })
                })
                .collect();
            let tail = run_third_run_recover(end, &faults);
            match &tail.outcome.outcome {
                RecoveryOutcome::Failed { failure, .. } => {
                    format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
                }
                outcome @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. }) => {
                    let landing = chosen_root_of(outcome);
                    let expected = landing.and_then(|root| history.content_number_of(root));
                    let got = match outcome {
                        RecoveryOutcome::FileRead { content, .. } => {
                            content_number_of_bytes(content, highest)
                        }
                        RecoveryOutcome::NoFile { .. } => Some(0),
                        RecoveryOutcome::Failed { .. } => None,
                    };
                    if expected.is_some() && got == expected {
                        format!("own_version:{}", key_text(landing))
                    } else {
                        format!(
                            "other:{}:expected={expected:?}:got={got:?}",
                            key_text(landing)
                        )
                    }
                }
            }
        }
        (Some(_), Err(_)) => "failed:geometry".to_string(),
        (None, _) => "na".to_string(),
    };
    CellLoss {
        in_ring_overwritten,
        out_of_ring_overwritten,
        landing_overwritten,
        read_back,
        read_back_root,
        tail_class,
    }
}

/// 每臂每族的汇总（第六节各量的分母与分子都报整数）。
#[derive(Default)]
pub(super) struct ThirdRunFamilySummary {
    pub(super) cells: u64,
    pub(super) void_cells: u64,
    pub(super) fault_not_reached_cells: u64,
    pub(super) overwrite_cells: u64,
    pub(super) in_ring_overwrite_cells: u64,
    pub(super) out_of_ring_overwrite_cells: u64,
    pub(super) landing_overwrite_cells: u64,
    pub(super) rollback_cells: u64,
    pub(super) tested_ok_cells: u64,
    pub(super) read_back: BTreeMap<String, u64>,
    pub(super) read_back_last_confirmed_cells: u64,
    pub(super) read_back_lost_cells: u64,
    pub(super) tail: BTreeMap<String, u64>,
    pub(super) tail_lost_cells: u64,
    pub(super) lost_write_cells: u64,
    pub(super) tested_classes: BTreeMap<String, u64>,
    pub(super) remount_classes: BTreeMap<String, u64>,
    pub(super) abandoned_hidden_root_cells: u64,
    /// 停机 S4：装置与 `crates/` 对不上的格（7.3 第一行）。
    pub(super) stop_condition_four_mismatches: Vec<String>,
    /// 失败条款 F4：点名单元只有第一份读不出时施加了那次发布。
    pub(super) failure_clause_four_findings: Vec<String>,
    /// 失败条款 F13：新实例第一次发布的 txg 与今天的规则不等。
    pub(super) failure_clause_thirteen_findings: Vec<String>,
    pub(super) failure_clause_thirteen_checks: u64,
    pub(super) formula_mismatches: Vec<String>,
}

impl ThirdRunFamilySummary {
    pub(super) fn count(&mut self, loss: &CellLoss, rollback: bool, tested_ok: bool) {
        self.cells += 1;
        if loss.overwrite_cell() {
            self.overwrite_cells += 1;
        }
        if loss.in_ring_overwritten > 0 {
            self.in_ring_overwrite_cells += 1;
        }
        if loss.out_of_ring_overwritten > 0 {
            self.out_of_ring_overwrite_cells += 1;
        }
        if loss
            .landing_overwritten
            .parse::<u64>()
            .is_ok_and(|count| count > 0)
        {
            self.landing_overwrite_cells += 1;
        }
        if rollback {
            self.rollback_cells += 1;
        }
        if tested_ok {
            self.tested_ok_cells += 1;
        }
        *self
            .read_back
            .entry(loss.read_back.split(':').next().unwrap_or("").to_string())
            .or_insert(0) += 1;
        if loss.read_back == "last_confirmed" {
            self.read_back_last_confirmed_cells += 1;
        }
        if loss.read_back_lost() {
            self.read_back_lost_cells += 1;
        }
        *self
            .tail
            .entry(loss.tail_class.split(':').next().unwrap_or("").to_string())
            .or_insert(0) += 1;
        if loss.tail_lost() {
            self.tail_lost_cells += 1;
        }
        if loss.lost_write_cell() || rollback {
            self.lost_write_cells += 1;
        }
    }

    pub(super) fn emit(&self, family: &str, arm: &str) {
        emit_result(&format!(
            "name=r3_family_summary family={family} arm={arm} cells={} void_cells_v2={} fault_not_reached_cells={} q1_overwrite_cells={} q1_in_ring_overwrite_cells={} q1_out_of_ring_overwrite_cells={} q1_landing_overwrite_cells={} q2_rollback_cells={} tested_ok_cells={} q3_read_back={:?} q3_last_confirmed_cells={} q3_lost_cells={} q3c_tail={:?} q3c_lost_cells={} lost_write_cells_q1_q2_q3={} tested_classes={:?} remount_classes={:?} q0_hidden_root_abandoned_cells={} s4_mismatches={} f4_failures={} f13_checked={} f13_failures={} formula_mismatches={}",
            self.cells,
            self.void_cells,
            self.fault_not_reached_cells,
            self.overwrite_cells,
            self.in_ring_overwrite_cells,
            self.out_of_ring_overwrite_cells,
            self.landing_overwrite_cells,
            self.rollback_cells,
            self.tested_ok_cells,
            self.read_back,
            self.read_back_last_confirmed_cells,
            self.read_back_lost_cells,
            self.tail,
            self.tail_lost_cells,
            self.lost_write_cells,
            self.tested_classes,
            self.remount_classes,
            self.abandoned_hidden_root_cells,
            self.stop_condition_four_mismatches.len(),
            self.failure_clause_four_findings.len(),
            self.failure_clause_thirteen_checks,
            self.failure_clause_thirteen_findings.len(),
            self.formula_mismatches.len()
        ));
        for (kind, list) in [
            ("s4", &self.stop_condition_four_mismatches),
            ("f4", &self.failure_clause_four_findings),
            ("f13", &self.failure_clause_thirteen_findings),
            ("formula", &self.formula_mismatches),
        ] {
            for detail in list {
                emit_result(&format!(
                    "name=r3_family_finding family={family} arm={arm} kind={kind} detail={:?}",
                    detail
                ));
            }
        }
    }
}

/// 7.1 第四行（F13）今天那一臂的期望值：新实例第一次发布的 txg = max(这次调用里读得出的根的 txg, 读得出的自证记录的 checkpoint_txg) + 1。
/// 根环在这次调用里被读两遍才走到这一步（择根一遍、`highest_root_txg` 一遍），记录只在扫描那一遍读：
/// 根槽在第二次读还坏的不算，记录在第一次读就坏的不算（今天那一臂从不等，T 与 W 同）。
pub(super) fn first_txg_expected_by_the_rule_of_today(
    twin: &MemoryPool,
    geometry: &PoolGeometry,
    faults: &[ThirdRunFault],
) -> u64 {
    let second_read_faults: Vec<ThirdRunFault> = faults
        .iter()
        .filter(|fault| fault.duration.is_active(2, 0))
        .cloned()
        .map(|mut fault| {
            fault.duration = FaultDuration::WholeCall;
            fault
        })
        .collect();
    let roots = device_decode(
        &FirstPassReader {
            pool: twin,
            faults: &second_read_faults,
        },
        geometry,
    );
    let records = device_decode(&FirstPassReader { pool: twin, faults }, geometry);
    let highest_root = roots
        .root_keys
        .iter()
        .map(|(_, txg)| *txg)
        .max()
        .unwrap_or(0);
    let highest_record = records
        .records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(0);
    1 + highest_root.max(highest_record)
}

/// 被测那次挂载的「之前与之后」对拍（7.3 第一行，今天那一臂）：装置择的根、重放出的 E、系统配置槽解码、被抛弃判定与 `crates/` 的比。
pub(super) fn today_cross_checks(
    base: &MemoryPool,
    judgement: &DeviceJudgement,
    tested: &ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    label: &str,
    summary: &mut ThirdRunFamilySummary,
) {
    if let Some(error) = &judgement.error {
        summary
            .stop_condition_four_mismatches
            .push(format!("{label}: 装置重放报错 {error}"));
    }
    if let Ok(mounted) = &tested.outcome {
        let crates_chosen = root_pair(&mounted.output.chosen_root);
        let crates_effective = root_pair(&mounted.output.effective_root);
        if judgement.chosen != Some(crates_chosen) || judgement.effective != Some(crates_effective)
        {
            summary.stop_condition_four_mismatches.push(format!(
                "{label}: 择根 / E 装置 {} / {}，crates {} / {}",
                key_text(judgement.chosen),
                key_text(judgement.effective),
                key_text(Some(crates_chosen)),
                key_text(Some(crates_effective))
            ));
        }
    }
    let devices = devices_from_pool(base, IMAGE_BYTES);
    let Ok(geometry) = independent_geometry(base) else {
        summary
            .stop_condition_four_mismatches
            .push(format!("{label}: 几何解不出"));
        return;
    };
    let decoded = device_decode(base, &geometry);
    for device in [0u32, 1] {
        let mut by_crates: Vec<(u64, u64, u32)> =
            singlefs_core::recovery::verified_system_configuration_slots(
                &devices,
                DeviceIdentity(device),
                geometry.slot_spacing,
                &FILESYSTEM_IDENTIFIER,
            )
            .into_iter()
            .map(|slot| {
                (
                    slot.quantities.slot_generation,
                    slot.quantities.journal_tail,
                    slot.quantities.journal_instance.0,
                )
            })
            .collect();
        let mut by_the_device: Vec<(u64, u64, u32)> = decoded
            .system_configuration_slots
            .iter()
            .filter(|(slot_device, _, _, _)| *slot_device == device)
            .map(|(_, generation, tail, instance)| (*generation, *tail, *instance))
            .collect();
        by_crates.sort_unstable();
        by_the_device.sort_unstable();
        if by_crates != by_the_device {
            summary.stop_condition_four_mismatches.push(format!(
                "{label}: 盘 {device} 系统配置槽 装置 {by_the_device:?}，crates {by_crates:?}"
            ));
        }
    }
    if tested.outcome.is_ok() {
        if let Err(error) = abandoned_roots_on(&tested.pool_after) {
            if error.starts_with("S5") {
                summary
                    .stop_condition_four_mismatches
                    .push(format!("{label}: 调用之后 {error}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 11.5 H1d（登记 5.3 第一行）：P(n1, C) → A → 断在「这次挂载写行根落盘之前」的某一点 → 撤故障 → `mount_writable`（不注入）→
//      冷启动 `recover` 读回；断点在这一臂自己那次调用的写序列上取。
// ---------------------------------------------------------------------------

/// 这一臂在 H1d / PC 上跑哪几种时长（登记 5.3：各臂 W 与 O(1 只)，乙 三臂另加 T(δ)、T(2δ)；今天、甲、丙 另跑 T(δ)
/// 核 7.3 第三行「T(τ) 对它们与 W 逐格相同」，只跑不断那一格）。交回 (时长, 只跑不断那一格)。
pub(super) fn held_once_durations_of(arm: &str) -> Vec<(FaultDuration, bool)> {
    let mut durations = vec![
        (FaultDuration::WholeCall, false),
        (FaultDuration::OnlyTheNthRead(1), false),
    ];
    match ThirdRunCandidate::of(arm) {
        Some(ThirdRunCandidate::Yi) => {
            durations.push((
                FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
                false,
            ));
            durations.push((
                FaultDuration::UntilVirtualTime(2 * THIRD_RUN_VIRTUAL_INTERVAL),
                false,
            ));
        }
        Some(ThirdRunCandidate::Today | ThirdRunCandidate::Jia | ThirdRunCandidate::Bing)
        | None => {
            durations.push((
                FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
                true,
            ));
        }
    }
    durations
}

/// 一次被测挂载（A）在一份起始镜像上跑完、交回它与装置的判定；今天那一臂顺带做 7.3 第一行、F13。
pub(super) struct TestedMount {
    pub(super) call: ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    pub(super) summary: TestedMountSummary,
    pub(super) judgement: DeviceJudgement,
    pub(super) unintercepted: Vec<String>,
}

pub(super) fn run_tested_mount(
    arm: &str,
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    label: &str,
    family_summary: &mut ThirdRunFamilySummary,
) -> Result<TestedMount, String> {
    let geometry = independent_geometry(base)?;
    let call = run_third_run_mount_writable(base, parameters, faults, None, None);
    let judgement = device_judgement(base, &geometry, faults);
    let summary = TestedMountSummary::of(&call);
    let unintercepted = call.unintercepted(faults);
    if arm == "today" {
        today_cross_checks(base, &judgement, &call, label, family_summary);
        if let Ok(mounted) = &call.outcome {
            family_summary.failure_clause_thirteen_checks += 1;
            let expected = first_txg_expected_by_the_rule_of_today(base, &geometry, faults);
            let got = mounted.output.row_publish.root().checkpoint_txg.0;
            if got != expected {
                family_summary
                    .failure_clause_thirteen_findings
                    .push(format!(
                        "{label}: 写行那次发布 txg {got}，按今天的规则 {expected}"
                    ));
            }
        }
    }
    Ok(TestedMount {
        call,
        summary,
        judgement,
        unintercepted,
    })
}

/// Q0：被测那次挂载 `Ok` 之后撤故障，被藏那条根还在不在根环里、装置按新实例表判它被不被抛弃（没 `Ok` 的两样都记 none）。
pub(super) fn hidden_root_after_the_tested_mount(
    call: &ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    hidden: TimelineRoot,
) -> (Option<bool>, Option<bool>) {
    if call.outcome.is_err() {
        return (None, None);
    }
    let in_ring = independent_geometry(&call.pool_after)
        .ok()
        .map(|geometry| readable_roots_independent(&call.pool_after, &geometry).contains(&hidden));
    let abandoned = abandoned_roots_on(&call.pool_after)
        .ok()
        .map(|abandoned| abandoned.contains(&hidden));
    (in_ring, abandoned)
}

pub(super) fn optional_flag_text(flag: Option<bool>) -> String {
    flag.map_or("none".to_string(), |value| value.to_string())
}

pub(super) fn render_device_judgement(judgement: &DeviceJudgement) -> String {
    format!(
        "device_chosen={} device_effective={} device_n_cfg={} device_c_witness={} device_c_e={} device_undecidable={} device_n_slot={} device_unreadable_root_slots={} truth_newer={} truth_orders_disagree={} truth_by_records_only={}",
        key_text(judgement.chosen),
        key_text(judgement.effective),
        judgement.configuration_newer,
        judgement.witnessed_tail,
        judgement
            .selected_version_last_counter
            .map_or("none".to_string(), |counter| counter.to_string()),
        judgement.undecidable,
        judgement.unreadable_root_slots > 0,
        judgement.unreadable_root_slots,
        judgement.truth_newer,
        judgement.truth_orders_disagree,
        judgement.truth_by_records_only
    )
}

/// 一格：A 之后（断点那一刻）的镜像 `post` → 撤故障 → 再可写挂载（这一臂的代码，不注入）→ 历史末尾算三样。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的全部坐标与上下文原样打进结果行，拆成结构体只多一层搬运"
)]
pub(super) fn evaluate_held_once_cell_of_the_third_run(
    arm: &str,
    coordinates: &str,
    history: &ThirdRunHistory,
    protected: &ThirdRunConfirmation,
    base: &MemoryPool,
    post: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    tested_line: &str,
    rollback: bool,
    tested_ok: bool,
    do_the_tail: bool,
    summary: &mut ThirdRunFamilySummary,
) {
    let remount = run_third_run_mount_writable(post, parameters, &[], None, None);
    let remount_class = third_run_outcome_class(&remount.outcome, remount.write_calls);
    let remount_member = mount_error_member_or_none(&remount.outcome);
    let first_new_instance = history.tip().0 + 1;
    let loss = evaluate_cell_loss(
        history,
        protected,
        base,
        &remount.pool_after,
        do_the_tail.then_some(first_new_instance),
    );
    summary.count(&loss, rollback, tested_ok);
    *summary
        .remount_classes
        .entry(remount_class.to_string())
        .or_insert(0) += 1;
    emit_result(&format!(
        "name=r3_h1d_cell arm={arm} {coordinates} {tested_line} q2_rollback={rollback} remount_class={remount_class} remount_error={remount_member} remount_chosen={} {}",
        key_text(
            remount
                .outcome
                .as_ref()
                .ok()
                .map(|mounted| root_pair(&mounted.output.chosen_root))
        ),
        loss.render()
    ));
}
