//! 第 5 次跑的成对构造、被抛弃根的上下文、变体判定与工作单位的处理。

use super::*;

/// 每块盘最新那一槽系统配置的世代号（按 `crates` 的解析判自证）。
fn newest_system_configuration_generation(
    pool: &MemoryPool,
    device: DeviceIdentity,
    slot_spacing: u64,
) -> Option<u64> {
    let bytes = newest_system_configuration_slot_bytes(pool, device, slot_spacing)?;
    singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
        .ok()
        .map(|parsed| parsed.quantities.slot_generation)
}

/// 崩溃恢复抛弃最新那条根而系统配置没见证到它：`pool` 的拷贝，每块盘世代号最大的那一槽写成「另一槽的内容、世代号改成 g − 2」，
/// 两槽都自证得过、不清零（清零之后每块盘只剩一槽自证过，可写挂载判不出拒可写）。世代号不足 2 的盘交 `Err`，调用方先核过。
pub(super) fn pool_with_the_system_configuration_rotation_unwritten(
    pool: &MemoryPool,
    slot_spacing: u64,
) -> Result<MemoryPool, String> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let mut rewound = pool.clone();
    for (device, rewound_device) in &mut rewound.devices {
        let newest = newest_system_configuration_slot_bytes(pool, *device, slot_spacing)
            .ok_or_else(|| format!("盘 {} 没有自证的系统配置槽", device.0))?;
        let newest_offset = [0u64, slot_spacing]
            .into_iter()
            .find(|offset| pool.read(device.0, *offset, slot_bytes).as_ref() == Some(&newest))
            .ok_or_else(|| format!("盘 {} 上最新那一槽不在两槽之一", device.0))?;
        let older_offset = if newest_offset == 0 { slot_spacing } else { 0 };
        let parse_the_slot_at = |offset: u64| {
            let bytes = pool
                .read(device.0, offset, slot_bytes)
                .ok_or_else(|| format!("盘 {} 偏移 {offset} 读不到", device.0))?;
            singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
                .map_err(|error| format!("盘 {} 偏移 {offset} 解不开：{error:?}", device.0))
        };
        let newest_generation = parse_the_slot_at(newest_offset)?.quantities.slot_generation;
        let mut rotation_not_written = parse_the_slot_at(older_offset)?;
        rotation_not_written.quantities.slot_generation = newest_generation
            .checked_sub(2)
            .ok_or_else(|| format!("盘 {} 最新世代号 {newest_generation} 小于 2", device.0))?;
        rewound_device.write(
            DeviceOffsetInBytes(newest_offset),
            &rotation_not_written.to_slot(),
        );
    }
    Ok(rewound)
}

/// 抛弃步造出的东西：抛弃步之后的镜像（该镜像上按新实例表判出的被抛弃根集合非空）。
pub(super) struct ConstructedPair {
    pub(super) pool_after_abandonment: MemoryPool,
    pub(super) pool_geometry: PoolGeometry,
    pub(super) plain_devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    pub(super) abandoned: BTreeSet<TimelineRoot>,
}

/// 一个 (节点, 世代) 对做抛弃步的结局（登记第一节「读法写死」表「构造对」）：成立、不适用、未造出，三类各自计数。
pub(super) enum PairConstruction {
    Constructed(ConstructedPair),
    NotApplicable(String),
    NotConstructed(String),
}

pub(super) fn construct_pair(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    replay_break: ReplayBreak,
) -> Result<PairConstruction, String> {
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        match newest_system_configuration_generation(&node.pool, device, slot_spacing) {
            Some(generation) if generation >= 2 => {}
            Some(_) => {
                return Ok(PairConstruction::NotApplicable(
                    "slot_generation_below_two".to_string(),
                ))
            }
            None => {
                return Ok(PairConstruction::NotApplicable(
                    "no_self_certifying_system_configuration_slot".to_string(),
                ))
            }
        }
    }
    let pool_geometry = independent_geometry(&node.pool)?;
    let history = SecondRunHistory {
        pool: node.pool.clone(),
        session: None,
        timeline: node.timeline.clone(),
        content_by_root: node.content_by_root.clone(),
        publishes_with_content: 0,
    };
    let ranges = match unreadable_ranges_of_the_abandonment(
        &history,
        &pool_geometry,
        1,
        UnreadableForm::ReadFails,
        replay_break,
    ) {
        Ok(Some(ranges)) => ranges,
        Ok(None) => {
            return Ok(PairConstruction::NotApplicable(
                "no_record_or_no_named_unit".to_string(),
            ))
        }
        Err(_) => {
            return Ok(PairConstruction::NotApplicable(
                "timeline_too_short".to_string(),
            ))
        }
    };
    let rewound = pool_with_the_system_configuration_rotation_unwritten(&node.pool, slot_spacing)?;
    let run = run_instrumented_mount_writable(&rewound, parameters, &ranges, None, None);
    if run.result.is_err() {
        return Ok(PairConstruction::NotConstructed(format!(
            "abandonment_mount_{}",
            mount_error_member_or_none(&run.result)
        )));
    }
    if !unintercepted_ranges(&ranges, &run.intercepted_reads).is_empty() {
        return Ok(PairConstruction::NotConstructed(
            "faults_not_all_intercepted".to_string(),
        ));
    }
    let abandoned = abandoned_roots_on(&run.pool_after)?;
    if abandoned.is_empty() {
        return Ok(PairConstruction::NotConstructed(
            "abandoned_set_empty".to_string(),
        ));
    }
    let pool_after_abandonment = run.pool_after;
    let geometry_after_abandonment = independent_geometry(&pool_after_abandonment)?;
    let plain_devices = devices_from_pool(&pool_after_abandonment, IMAGE_BYTES);
    Ok(PairConstruction::Constructed(ConstructedPair {
        pool_after_abandonment,
        pool_geometry: geometry_after_abandonment,
        plain_devices,
        abandoned,
    }))
}

pub(super) fn timeline_root_text(root: TimelineRoot) -> String {
    format!("{}.{}", root.0, root.1)
}

fn timeline_roots_text(roots: &[TimelineRoot]) -> String {
    dash_when_empty(
        roots
            .iter()
            .map(|root| timeline_root_text(*root))
            .collect::<Vec<String>>()
            .join(";"),
    )
}

fn parse_timeline_roots(text: &str) -> Result<Vec<TimelineRoot>, String> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(';')
        .map(|part| -> Result<TimelineRoot, String> {
            let (instance, txg) = part
                .split_once('.')
                .ok_or_else(|| format!("根认不出：{part}"))?;
            Ok((
                instance
                    .parse::<u32>()
                    .map_err(|_| format!("实例号认不出：{part}"))?,
                txg.parse::<u64>()
                    .map_err(|_| format!("txg 认不出：{part}"))?,
            ))
        })
        .collect()
}

/// 抛弃步的结局，判定单位的值（文本形态与 [`VariantResult`] 同一套写法）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PairOutcome {
    Constructed {
        abandoned: Vec<TimelineRoot>,
        /// 被抛弃根里根槽读得出的那几条：每一条做一组变体。
        readable_abandoned: Vec<TimelineRoot>,
    },
    NotApplicable(String),
    NotConstructed(String),
}

impl PairOutcome {
    fn status(&self) -> &'static str {
        match self {
            PairOutcome::Constructed { .. } => "constructed",
            PairOutcome::NotApplicable(_) => "not_applicable",
            PairOutcome::NotConstructed(_) => "not_constructed",
        }
    }

    pub(super) fn to_text(&self) -> String {
        match self {
            PairOutcome::Constructed {
                abandoned,
                readable_abandoned,
            } => format!(
                "status=constructed abandoned={} readable_abandoned={}",
                timeline_roots_text(abandoned),
                timeline_roots_text(readable_abandoned)
            ),
            PairOutcome::NotApplicable(reason) | PairOutcome::NotConstructed(reason) => {
                format!("status={} reason={reason}", self.status())
            }
        }
    }

    pub(super) fn from_text(text: &str) -> Result<Self, String> {
        match text_field_named(text, "status")? {
            "constructed" => Ok(PairOutcome::Constructed {
                abandoned: parse_timeline_roots(text_field_named(text, "abandoned")?)?,
                readable_abandoned: parse_timeline_roots(text_field_named(
                    text,
                    "readable_abandoned",
                )?)?,
            }),
            "not_applicable" => Ok(PairOutcome::NotApplicable(
                text_field_named(text, "reason")?.to_string(),
            )),
            "not_constructed" => Ok(PairOutcome::NotConstructed(
                text_field_named(text, "reason")?.to_string(),
            )),
            other => Err(format!("抛弃步结局的状态认不出：{other}")),
        }
    }
}

pub(super) fn outcome_of_the_construction(construction: &PairConstruction) -> PairOutcome {
    match construction {
        PairConstruction::Constructed(pair) => {
            let readable_abandoned = pair
                .abandoned
                .iter()
                .copied()
                .filter(|root| {
                    root_slot_of(&pair.pool_after_abandonment, &pair.pool_geometry, *root).is_some()
                        && root_record_of(&pair.pool_after_abandonment, &pair.pool_geometry, *root)
                            .is_some()
                })
                .collect();
            PairOutcome::Constructed {
                abandoned: pair.abandoned.iter().copied().collect(),
                readable_abandoned,
            }
        }
        PairConstruction::NotApplicable(reason) => PairOutcome::NotApplicable(reason.clone()),
        PairConstruction::NotConstructed(reason) => PairOutcome::NotConstructed(reason.clone()),
    }
}

/// 一个被抛弃根上的变体种类（登记第一节「读法写死」表）：孪生一个、账读不出十二个、根槽读不出两个。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VariantSpecification {
    Twin,
    Account {
        aspect: LedgerAspect,
        severity: FaultSeverity,
        form: UnreadableForm,
    },
    RootSlot {
        form: UnreadableForm,
    },
}

impl VariantSpecification {
    pub(super) fn name(self) -> String {
        match self {
            VariantSpecification::Twin => "twin".to_string(),
            VariantSpecification::Account {
                aspect,
                severity,
                form,
            } => format!("{}.{}.{}", aspect.label(), severity.label(), form.name()),
            VariantSpecification::RootSlot { form } => format!("root_slot.{}", form.name()),
        }
    }

    pub(super) fn group(self) -> JudgementGroup {
        match self {
            VariantSpecification::Twin => JudgementGroup::Twin,
            VariantSpecification::Account { .. } => JudgementGroup::AccountUnreadable,
            VariantSpecification::RootSlot { .. } => JudgementGroup::RootSlotUnreadable,
        }
    }
}

/// 每个被抛弃根的变体表，次序固定：孪生、账读不出（单元 × 严重度 × 形态）、根槽读不出（形态）。
pub(super) fn variant_specifications() -> Vec<VariantSpecification> {
    let forms = [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros];
    let mut specifications = vec![VariantSpecification::Twin];
    for aspect in [LedgerAspect::AllocationRecordTree, LedgerAspect::TreeTable] {
        for severity in [
            FaultSeverity::Both,
            FaultSeverity::Disk0,
            FaultSeverity::Disk1,
        ] {
            for form in forms {
                specifications.push(VariantSpecification::Account {
                    aspect,
                    severity,
                    form,
                });
            }
        }
    }
    for form in forms {
        specifications.push(VariantSpecification::RootSlot { form });
    }
    specifications
}

/// 一条被抛弃根的变体都要用的东西：根记录、根槽、独占落点（U_a）。
pub(super) struct AbandonedRootContext {
    pub(super) record: RootRecord,
    pub(super) root_slot: (DeviceIdentity, u64),
    /// 被抛弃根独占的落点按槽展开（(盘, 槽, 1)）：它引用的槽减去其余每条可读根引用的槽。装置自己算，不取 `crates/` 的分配器。
    pub(super) exclusive_placements: BTreeSet<(u32, u64, u64)>,
}

pub(super) fn slots_of_placements(placements: &BTreeSet<(u32, u64, u64)>) -> BTreeSet<(u32, u64)> {
    placements
        .iter()
        .flat_map(|(device, slot, span)| (0..*span).map(move |step| (*device, slot + step)))
        .collect()
}

pub(super) fn build_abandoned_root_context(
    pair: &ConstructedPair,
    root: TimelineRoot,
) -> Result<AbandonedRootContext, String> {
    let pool = &pair.pool_after_abandonment;
    let record = root_record_of(pool, &pair.pool_geometry, root)
        .ok_or_else(|| format!("根 {root:?} 的根记录读不出"))?;
    let root_slot = root_slot_of(pool, &pair.pool_geometry, root)
        .ok_or_else(|| format!("根 {root:?} 的根槽读不出"))?;
    let mut slots = slots_of_placements(&placements_referenced_by_root_on(pool, &record)?);
    for other in readable_roots_independent(pool, &pair.pool_geometry) {
        if other == root {
            continue;
        }
        if let Some(other_record) = root_record_of(pool, &pair.pool_geometry, other) {
            if let Ok(referenced) = placements_referenced_by_root_on(pool, &other_record) {
                for slot in slots_of_placements(&referenced) {
                    slots.remove(&slot);
                }
            }
        }
    }
    let exclusive_placements = slots
        .into_iter()
        .map(|(device, slot)| (device, slot, 1))
        .collect();
    Ok(AbandonedRootContext {
        record,
        root_slot,
        exclusive_placements,
    })
}

/// 账单元（树表、分配记录树根节点）的两条位置。
#[allow(
    clippy::ptr_arg,
    reason = "转给 locate_allocation_record_tree，它要 &dyn PoolReader、要一个有大小的读者"
)]
pub(super) fn account_locations(
    aspect: LedgerAspect,
    plain_devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    record: &RootRecord,
) -> Result<[LocationEntry; 2], String> {
    match aspect {
        LedgerAspect::TreeTable => Ok(record.tree_table.locations),
        LedgerAspect::AllocationRecordTree => locate_allocation_record_tree(plain_devices, record),
    }
}

/// 严重度选哪几份副本坏（两份、只第一块盘上那份、只第二块盘上那份）；落点 = 那份位置条目所指的槽。
fn account_ranges(
    locations: &[LocationEntry; 2],
    severity: FaultSeverity,
    form: UnreadableForm,
    label: &str,
) -> Vec<UnreadableRange> {
    locations
        .iter()
        .filter(|location| match severity {
            FaultSeverity::Both => true,
            FaultSeverity::Disk0 => location.device == DeviceIdentity(0),
            FaultSeverity::Disk1 => location.device == DeviceIdentity(1),
            FaultSeverity::Empty => false,
        })
        .map(|location| UnreadableRange {
            device: location.device,
            offset: location.slot.0 * SLOT_BYTES_LOCAL,
            length: SLOT_BYTES_LOCAL,
            form,
            label: format!(
                "{label}(device={},slot={})",
                location.device.0, location.slot.0
            ),
        })
        .collect()
}

/// through(u)：被抛弃根里根槽读得出、且账单元所指的槽与故障落点有交的根的条数。
fn through_count(
    pair: &ConstructedPair,
    aspect: LedgerAspect,
    fault_keys: &BTreeSet<(u32, u64)>,
) -> u64 {
    let count = pair
        .abandoned
        .iter()
        .copied()
        .filter(|root| {
            root_slot_of(&pair.pool_after_abandonment, &pair.pool_geometry, *root).is_some()
        })
        .filter_map(|root| root_record_of(&pair.pool_after_abandonment, &pair.pool_geometry, root))
        .filter(|other| {
            account_locations(aspect, &pair.plain_devices, other)
                .ok()
                .is_some_and(|locations| !location_keys(&locations).is_disjoint(fault_keys))
        })
        .count();
    u64::try_from(count).expect("根数装得进 u64")
}

/// 续写的终点 txg：txg_a + 3·S − 1（登记第一节「读法写死」表「续写」；txg_a + 3·S 那一次按槽序盖掉 a 的根槽，不发布）。
pub(super) fn continuation_end_txg(abandoned_root: TimelineRoot, geometry: &Geometry) -> u64 {
    abandoned_root.1 + ROOT_RING_REGIONS_LOCAL * geometry.slots_per_region - 1
}

/// 续写：用挂载交回的会话继续 `publish_overwrite`，直到刚发布的 txg 等于 `end_txg`；每发布一次记一次累计复用数
/// （`placements_overwritten_between(baseline, 当下镜像, placements)`）。交回 (最后一次发布的 txg, 轨迹, 续写末态镜像)；
/// 挂载交回时 txg 已不小于终点则一次不发布，末态就是挂载交回的镜像。
pub(super) fn continue_with_overwrites(
    pool: &MemoryPool,
    session: Session,
    parameters: &MakeFilesystemParameters,
    baseline: &MemoryPool,
    placements: &BTreeSet<(u32, u64, u64)>,
    end_txg: u64,
) -> Result<(u64, Vec<u64>, MemoryPool), String> {
    let mut devices = devices_from_pool(pool, IMAGE_BYTES);
    let mut end_state = pool.clone();
    let mut allocator = session.allocator;
    let mut current = session.current;
    let mut txg = current.root().checkpoint_txg.0;
    let mut trajectory = Vec::new();
    let mut step_number = 0u64;
    while txg < end_txg {
        step_number += 1;
        let PoolVersion::WithFile(previous) = &current else {
            return Err("续写：现行版本没有文件".to_string());
        };
        let tag = 9000 + step_number;
        let content = deterministic_content(tag, OVERWRITE_FILE_BYTES);
        let output = {
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut allocator,
                previous,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * tag,
                },
                session.instance,
            )
            .map_err(|error| format!("续写 publish_overwrite: {error:?}"))?
        };
        txg = output.root.checkpoint_txg.0;
        current = PoolVersion::WithFile(std::sync::Arc::new(output));
        end_state = memory_pool_of(&devices, IMAGE_BYTES);
        trajectory.push(placements_overwritten_between(
            baseline, &end_state, placements,
        ));
    }
    Ok((txg, trajectory, end_state))
}

/// 落点按盘分列的槽数。
fn slot_counts_by_device(placements: &BTreeSet<(u32, u64, u64)>) -> Vec<(u32, u64)> {
    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    for (device, _slot) in slots_of_placements(placements) {
        *counts.entry(device).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}

/// 一个变体要用的全部输入。
pub(super) struct VariantContext<'pair> {
    pub(super) pair: &'pair ConstructedPair,
    pub(super) root: TimelineRoot,
    pub(super) root_context: &'pair AbandonedRootContext,
    pub(super) parameters: &'pair MakeFilesystemParameters,
    pub(super) geometry: &'pair Geometry,
}

/// 做一个变体：在抛弃步之后的镜像上做一次新的 `mount_writable`（故障只在这次调用里有效），记结局；
/// 孪生、账两份坏的变体与根槽变体挂载做成后续写到终点。
pub(super) fn judge_variant(
    context: &VariantContext,
    specification: VariantSpecification,
) -> Result<VariantResult, String> {
    let pair = context.pair;
    let root_context = context.root_context;
    let (ranges, expected_count, shared, continues) = match specification {
        VariantSpecification::Twin => (Vec::new(), 0, false, true),
        VariantSpecification::Account {
            aspect,
            severity,
            form,
        } => {
            let locations = account_locations(aspect, &pair.plain_devices, &root_context.record)?;
            let ranges = account_ranges(&locations, severity, form, aspect.label());
            if ranges.is_empty() {
                return Ok(VariantResult::not_applicable());
            }
            let unit_keys = location_keys(&locations);
            let shared = is_unit_shared(
                &pair.plain_devices,
                &pair.pool_after_abandonment,
                &pair.pool_geometry,
                context.root,
                &unit_keys,
            );
            let both_copies = severity == FaultSeverity::Both;
            let expected_count = if both_copies {
                let fault_keys: BTreeSet<(u32, u64)> = ranges
                    .iter()
                    .map(|range| (range.device.0, range.offset / SLOT_BYTES_LOCAL))
                    .collect();
                through_count(pair, aspect, &fault_keys)
            } else {
                0
            };
            (ranges, expected_count, shared, both_copies)
        }
        VariantSpecification::RootSlot { form } => (
            vec![UnreadableRange {
                device: root_context.root_slot.0,
                offset: root_context.root_slot.1,
                length: u64::from(pair.pool_geometry.physical_block_size),
                form,
                label: format!("root_slot{:?}", context.root),
            }],
            0,
            false,
            true,
        ),
    };
    let run = run_instrumented_mount_writable(
        &pair.pool_after_abandonment,
        context.parameters,
        &ranges,
        None,
        None,
    );
    let InstrumentedMount {
        result,
        write_calls,
        intercepted_reads,
        pool_after,
        ..
    } = run;
    let mut variant = VariantResult {
        applicable: true,
        class: mount_outcome_class(&result, write_calls).to_string(),
        error: mount_error_member_or_none(&result),
        count: None,
        expected_count,
        isolated_by_device: Vec::new(),
        intercepted_all: unintercepted_ranges(&ranges, &intercepted_reads).is_empty(),
        writes: write_calls,
        shared,
        exclusive_slots: u64::try_from(root_context.exclusive_placements.len())
            .expect("槽数装得进 u64"),
        exclusive_by_device: slot_counts_by_device(&root_context.exclusive_placements),
        continued_to_txg: None,
        reuse_trajectory: Vec::new(),
    };
    if let Ok(mounted) = result {
        variant.count = Some(mounted.output.abandoned_roots_unreadable);
        variant.isolated_by_device = mounted
            .output
            .isolated_slots_per_device
            .iter()
            .map(|(device, slots)| (device.0, *slots))
            .collect();
        if continues {
            let session = Session {
                allocator: mounted.allocator,
                current: mounted.current,
                instance: mounted.output.instance,
            };
            let (reached, trajectory, _end_state) = continue_with_overwrites(
                &pool_after,
                session,
                context.parameters,
                &pair.pool_after_abandonment,
                &root_context.exclusive_placements,
                continuation_end_txg(context.root, context.geometry),
            )?;
            variant.continued_to_txg = Some(reached);
            variant.reuse_trajectory = trajectory;
        }
    }
    Ok(variant)
}

/// 断链发布的重放怎么接不上，三种都做（登记第一节「读法写死」表「抛弃步」），次序固定。
pub(super) const FIFTH_RUN_REPLAY_BREAKS: [ReplayBreak; 3] = [
    ReplayBreak::EveryRecord,
    ReplayBreak::FirstNamedUnit,
    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
];

#[derive(Clone, Debug)]
pub(super) struct VariantRecord {
    pub(super) root: TimelineRoot,
    pub(super) specification: VariantSpecification,
    pub(super) result: VariantResult,
}

/// 一个 (节点, 断链方式) 对的全部判定：抛弃步的结局与每个被抛弃根的变体。
#[derive(Clone, Debug)]
pub(super) struct PairRecord {
    pub(super) starting_overwrites: u64,
    pub(super) steps: Vec<HistoryStep>,
    pub(super) replay_break: ReplayBreak,
    pub(super) outcome: PairOutcome,
    pub(super) variants: Vec<VariantRecord>,
    /// 这一对里现判了几个判定单位、从库里复用了几个。
    pub(super) fresh: u64,
    pub(super) reused: u64,
}

impl PairRecord {
    pub(super) fn variant(
        &self,
        root: TimelineRoot,
        specification: VariantSpecification,
    ) -> Option<&VariantResult> {
        self.variants
            .iter()
            .find(|record| record.root == root && record.specification == specification)
            .map(|record| &record.result)
    }
}

pub(super) fn constructed_or_error(
    cell: &std::cell::OnceCell<Result<PairConstruction, String>>,
    make: impl FnOnce() -> Result<PairConstruction, String>,
) -> Result<&PairConstruction, String> {
    cell.get_or_init(make)
        .as_ref()
        .map_err(|error| error.clone())
}

/// 判一个被抛弃根的变体要用的全部东西（节点、抛弃步的格子、这条根的格子）。
pub(super) struct VariantJob<'job> {
    pub(super) node: &'job SimNode,
    pub(super) parameters: &'job MakeFilesystemParameters,
    pub(super) geometry: &'job Geometry,
    pub(super) replay_break: ReplayBreak,
    pub(super) root: TimelineRoot,
    pub(super) construction_cell: &'job std::cell::OnceCell<Result<PairConstruction, String>>,
    root_cell: &'job std::cell::OnceCell<Result<AbandonedRootContext, String>>,
}

/// 三组变体判法共用的做法：抛弃步（没做过才做）、这条根的上下文、一个变体；交判定值文本。
pub(super) fn variant_judgement_text(
    job: &VariantJob,
    specification: VariantSpecification,
) -> Result<String, String> {
    let PairConstruction::Constructed(pair) = constructed_or_error(job.construction_cell, || {
        construct_pair(job.node, job.parameters, job.replay_break)
    })?
    else {
        return Err("变体要用的抛弃步这一趟没有造成立".to_string());
    };
    let root_context = job
        .root_cell
        .get_or_init(|| build_abandoned_root_context(pair, job.root))
        .as_ref()
        .map_err(|error| error.clone())?;
    let context = VariantContext {
        pair,
        root: job.root,
        root_context,
        parameters: job.parameters,
        geometry: job.geometry,
    };
    Ok(judge_variant(&context, specification)?.to_text())
}

/// 变体按所属的组走各自的入口。
fn variant_judgement_of_group(
    job: &VariantJob,
    specification: VariantSpecification,
) -> Result<String, String> {
    match specification.group() {
        JudgementGroup::Twin => twin_judgement_entry(job),
        JudgementGroup::AccountUnreadable => account_unreadable_judgement_entry(job, specification),
        JudgementGroup::RootSlotUnreadable => {
            root_slot_unreadable_judgement_entry(job, specification)
        }
        JudgementGroup::Abandonment => Err("抛弃步不是变体".to_string()),
    }
}

/// 判一个节点上的一个对要用的参数。
pub(super) struct PairJudging<'judging> {
    pub(super) geometry: &'judging Geometry,
    pub(super) parameters: &'judging MakeFilesystemParameters,
    pub(super) arm: &'judging str,
    pub(super) starting_overwrites: u64,
    pub(super) judgements: &'judging ReusableJudgements<'judging>,
}

impl PairJudging<'_> {
    /// 判定单位的键文本：臂、几何、起点、步序列、断链方式、被抛弃根、变体名（登记 5.7 第 4 条的判定键）。
    fn key_prefix(&self, steps: &[HistoryStep], replay_break: ReplayBreak) -> String {
        format!(
            "r5/{}/S={}/ring={}/n1={}/steps={}/b={}",
            self.arm,
            self.geometry.slots_per_region,
            self.geometry.journal_ring_bytes,
            self.starting_overwrites,
            step_sequence_text(steps),
            replay_break.name()
        )
    }

    /// 判一个对：先问库里有没有抛弃步的结局与每个变体；缺的才做抛弃步（只做一次）并现判缺的那几个，现判的记进 `awaiting_write`。
    pub(super) fn judge(
        &self,
        node: &SimNode,
        replay_break: ReplayBreak,
        awaiting_write: &mut JudgementsAwaitingWrite,
    ) -> Result<PairRecord, String> {
        let steps = steps_after_the_starting_overwrites(node, self.starting_overwrites);
        let key_prefix = self.key_prefix(&steps, replay_break);
        let construction_cell: std::cell::OnceCell<Result<PairConstruction, String>> =
            std::cell::OnceCell::new();
        let mut fresh = 0u64;
        let mut reused = 0u64;
        let (outcome_text, pair_reused) = self.judgements.judge_or_reuse(
            awaiting_write,
            JudgementGroup::Abandonment,
            &format!("{key_prefix}/pair"),
            || abandonment_judgement_entry(node, self.parameters, replay_break, &construction_cell),
        )?;
        if pair_reused {
            reused += 1;
        } else {
            fresh += 1;
        }
        let outcome = PairOutcome::from_text(&outcome_text)?;
        let mut variants = Vec::new();
        if let PairOutcome::Constructed {
            readable_abandoned, ..
        } = &outcome
        {
            for &root in readable_abandoned {
                let root_cell: std::cell::OnceCell<Result<AbandonedRootContext, String>> =
                    std::cell::OnceCell::new();
                let job = VariantJob {
                    node,
                    parameters: self.parameters,
                    geometry: self.geometry,
                    replay_break,
                    root,
                    construction_cell: &construction_cell,
                    root_cell: &root_cell,
                };
                for specification in variant_specifications() {
                    let key_text = format!(
                        "{key_prefix}/a={}/{}",
                        timeline_root_text(root),
                        specification.name()
                    );
                    let (text, variant_reused) = self.judgements.judge_or_reuse(
                        awaiting_write,
                        specification.group(),
                        &key_text,
                        || variant_judgement_of_group(&job, specification),
                    )?;
                    if variant_reused {
                        reused += 1;
                    } else {
                        fresh += 1;
                    }
                    variants.push(VariantRecord {
                        root,
                        specification,
                        result: VariantResult::from_text(&text)?,
                    });
                }
            }
        }
        Ok(PairRecord {
            starting_overwrites: self.starting_overwrites,
            steps,
            replay_break,
            outcome,
            variants,
            fresh,
            reused,
        })
    }
}
