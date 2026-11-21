//! 强制选段后备与一次用户改动：几何核后一半走强制选段，六条臂同一点名法、同一序列。

use super::*;

// ---- 强制选段后备（登记第十二节修订第 8 条：几何核后一半照 E185（GC一批搬迁的挂钟拆分） 走强制选段，六条臂同一点名法、同一序列）----

/// 强制选段后备的一条意图：点名的那一段、同一刻 greedy 照核心层重放要选的那一段、核心层签没签出去（没签出去是它停在「一段都没资格」上，
/// 照资格五条判这一段不合格），与签出去的那一条意图的段起点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ForcedChoiceOfAnIntent {
    pub(super) forced: SegmentOfTheChoice,
    pub(super) greedy: SegmentOfTheChoice,
    issued: bool,
    issued_region_start_slot: Option<u64>,
}

/// 一次跑里强制选段后备记下的：每一条意图（按先后）、碎片段用完没有、点名用的挂钟（在 GC 线程上、计时窗口外；并发跑里拿着会话锁）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ForcedFallbackReading {
    pub(super) choices: Vec<ForcedChoiceOfAnIntent>,
    fragment_segments_ran_out: bool,
    choosing_wall: Duration,
}

impl ForcedFallbackReading {
    pub(super) fn intents_issued(&self) -> u64 {
        u64::try_from(self.choices.iter().filter(|choice| choice.issued).count()).expect("意图数")
    }

    fn segments_refused_by_the_core(&self) -> u64 {
        u64::try_from(self.choices.iter().filter(|choice| !choice.issued).count()).expect("意图数")
    }

    /// 签出去的意图不在点名的那一段上（开关没装上、或核心层没照强制的段签）的条数；应是 0。
    pub(super) fn intents_issued_off_the_forced_region(&self) -> u64 {
        u64::try_from(
            self.choices
                .iter()
                .filter(|choice| {
                    choice.issued && choice.issued_region_start_slot != choice.forced.start_slot
                })
                .count(),
        )
        .expect("意图数")
    }

    /// `name=run` 与 `name=concurrent_run` 行里的那几段。
    pub(super) fn fields(&self) -> String {
        format!(
            "choice=forced_fallback forced_intents_issued={} forced_segments_refused_by_the_core={} \
             intents_issued_off_the_forced_region={} fragment_segments_ran_out={} forced_choice_wall_nanoseconds={}",
            self.intents_issued(),
            self.segments_refused_by_the_core(),
            self.intents_issued_off_the_forced_region(),
            self.fragment_segments_ran_out,
            self.choosing_wall.as_nanos(),
        )
    }

    /// 每条意图一行 `name=forced_choice`（`prefix` 是档、臂、重复、会话那几段）。
    pub(super) fn choice_lines(&self, prefix: &str) -> Vec<String> {
        let start = |segment: SegmentOfTheChoice| {
            segment
                .start_slot
                .map_or_else(|| "none".to_string(), |slot| slot.to_string())
        };
        self.choices
            .iter()
            .enumerate()
            .map(|(intent, choice)| {
                format!(
                    "{prefix} intent={intent} choice=forced_fallback region_start_slot={} live_slots={} \
                     holds_only_data_units={} issued={} issued_region_start_slot={} greedy_region_start_slot={} \
                     greedy_live_slots={} greedy_holds_only_data_units={}",
                    start(choice.forced),
                    choice.forced.live_slots,
                    choice.forced.holds_only_data_units,
                    choice.issued,
                    choice
                        .issued_region_start_slot
                        .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
                    start(choice.greedy),
                    choice.greedy.live_slots,
                    choice.greedy.holds_only_data_units,
                )
            })
            .collect()
    }
}

/// 一次跑里强制选段后备的状态：碎片段、点过名的段、还没签出去的那一次点名（连同点名时现行那一版的 checkpoint 号）与记下的。
pub(super) struct ForcedFallbackOfTheRun {
    fragment_segment_starts: BTreeSet<u64>,
    pub(super) lowest_live_data_inode: u64,
    live_data_units: u64,
    segments_already_forced: BTreeSet<u64>,
    pending: Option<(CheckpointTxg, ForcedChoiceOfAnIntent)>,
    pub(super) reading: ForcedFallbackReading,
}

impl ForcedFallbackOfTheRun {
    pub(super) fn of_the_image(image: &StartingImage) -> ForcedFallbackOfTheRun {
        ForcedFallbackOfTheRun {
            fragment_segment_starts: image.fragment_segment_starts.clone(),
            lowest_live_data_inode: image.lowest_live_data_inode,
            live_data_units: u64::try_from(image.write_generation_of_each_unit.len()).expect("P"),
            segments_already_forced: BTreeSet::new(),
            pending: None,
            reading: ForcedFallbackReading::default(),
        }
    }

    /// 照现行那一版点名：碎片段里下一段（[`fragment_segment_to_force`]），与同一刻 greedy 要选的那一段。碎片段用完交回 `None`。
    fn choose(
        &self,
        session: &MountedSession,
        devices: &PoolDevices,
    ) -> Option<ForcedChoiceOfAnIntent> {
        let version = version_of_trees(session);
        let unit_area = unit_area_of_device_zero(session);
        let unit_area_start_slot = unit_area.start.slot().0;
        let data_slots_now: BTreeSet<u64> = data_unit_slots_of_the_inodes(
            version,
            self.lowest_live_data_inode,
            live_data_files(self.live_data_units),
        )
        .into_values()
        .collect();
        let live_slots =
            live_slots_per_cluster_segment_by_the_records(&version.allocation_records, unit_area);
        let segments_holding_other_records = segments_holding_a_record_other_than_a_data_unit(
            &version.allocation_records,
            unit_area,
            &data_slots_now,
        );
        let forced = fragment_segment_to_force(
            &self.fragment_segment_starts,
            &self.segments_already_forced,
            unit_area_start_slot,
            &live_slots,
            &segments_holding_other_records,
        )?;
        let greedy_segment_start_slot = segment_the_greedy_choice_picks(
            version,
            unit_area,
            session.allocator.cluster_segments(),
            session
                .compaction
                .segments_refused_by_the_region_scan
                .clone(),
            session.compaction.quarantined_slots(),
            devices,
        );
        Some(ForcedChoiceOfAnIntent {
            forced,
            greedy: segment_of_the_choice(
                greedy_segment_start_slot,
                unit_area_start_slot,
                &live_slots,
                &segments_holding_other_records,
            ),
            issued: false,
            issued_region_start_slot: None,
        })
    }

    /// 一次 GC 调用之前（计时窗口外）：活意图的段记成点过名；还没点名、点的段已点过名、或现行那一版换过了，就照现行那一版重新点名；
    /// 把点名的段装进开关的 `forced_region`（碎片段用完装 `None`，之后不再强制）。交回调用之前的签发计数。
    pub(super) fn before_the_call(
        &mut self,
        session: &mut MountedSession,
        devices: &PoolDevices,
    ) -> u64 {
        let started = Instant::now();
        if let Some(live) = session.allocator.compaction().live_intent() {
            self.segments_already_forced
                .insert(live.intent.region.start_slot().0);
        }
        let checkpoint_txg = version_of_trees(session).root.checkpoint_txg;
        let pending_is_current = self.pending.is_some_and(|(chosen_at, choice)| {
            chosen_at == checkpoint_txg
                && choice
                    .forced
                    .start_slot
                    .is_some_and(|start| !self.segments_already_forced.contains(&start))
        });
        if !pending_is_current && !self.reading.fragment_segments_ran_out {
            self.pending = self
                .choose(session, devices)
                .map(|choice| (checkpoint_txg, choice));
            self.reading.fragment_segments_ran_out = self.pending.is_none();
        }
        let forced_region = self
            .pending
            .and_then(|(_chosen_at, choice)| choice.forced.start_slot)
            .map(|start| {
                CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start))
                    .expect("碎片段起点在聚簇段边界上、装得进 6 字节")
            });
        session.allocator.compaction_mut().switches.forced_region = forced_region;
        self.reading.choosing_wall += started.elapsed();
        session.compaction.observations.intents_issued
    }

    /// 一次 GC 调用之后：签发计数涨了，那一次点名签出去了（签出去的段取活意图的段；签了又在同一次调用里做完的取上一条做完的意图的段）；
    /// 没涨而这次调用停在「一段都没资格」上，核心层照资格五条拒了点名的那一段。两种都记成一条意图、那一段记成点过名。
    pub(super) fn after_the_call(
        &mut self,
        session: &MountedSession,
        intents_issued_before_the_call: u64,
        call: &BackgroundCompactionCall,
    ) {
        let Some((_chosen_at, mut choice)) = self.pending else {
            return;
        };
        if session.compaction.observations.intents_issued > intents_issued_before_the_call {
            choice.issued = true;
            choice.issued_region_start_slot = session
                .allocator
                .compaction()
                .live_intent()
                .map(|live| live.intent.region)
                .or(session
                    .compaction
                    .last_completed_intent
                    .map(|intent| intent.region))
                .map(|region| region.start_slot().0);
        } else if !the_call_stopped_with_no_eligible_segment(call) {
            return;
        }
        self.segments_already_forced
            .extend(choice.forced.start_slot);
        self.reading.choices.push(choice);
        self.pending = None;
    }
}

/// 这次调用起跑了、停在「一段都没资格」上（强制选的那一段不合格也停在这里）。
fn the_call_stopped_with_no_eligible_segment(call: &BackgroundCompactionCall) -> bool {
    matches!(
        call,
        BackgroundCompactionCall::Ran { run, .. }
            if matches!(run.outcome, CompactionRunOutcome::Stopped(CompactionRunStop::NoEligibleSegment))
    )
}

/// 前台第 `change_number` 次用户改动（种子与序号定单元与内容）：按那个单元在它那个活数据文件里的偏移覆盖写它
/// （[`unit_index_of_the_user_change`] 只取文件的最后一个单元，只重写这一个），记下这个单元最后一次写是第几次。
pub(super) fn publish_the_user_change(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    lowest_live_data_inode: u64,
    change_number: u64,
    write_generation_of_each_unit: &mut [u64],
    write_time_seconds: u64,
) {
    let live_data_units = u64::try_from(write_generation_of_each_unit.len()).expect("P");
    let unit_index = unit_index_of_the_user_change(change_number, live_data_units);
    let (file, index_in_file) = file_and_index_of_a_live_data_unit(unit_index);
    assert_eq!(
        index_in_file + 1,
        LIVE_DATA_UNITS_PER_FILE,
        "前台用户改动只覆盖写文件的最后一个单元（按偏移写重写到文件末尾，R4）"
    );
    let write_generation = WRITE_GENERATION_OF_THE_FIRST_USER_CHANGE + change_number;
    let mut unit_bytes = vec![0u8; data_unit_payload_capacity()];
    fill_the_content_of_a_data_unit(unit_index, write_generation, &mut unit_bytes);
    let capacity = u64::try_from(unit_bytes.len()).expect("净荷容量装得进 u64");
    session
        .publish_user_change(
            devices,
            UserChange::WriteAtOffset {
                inode: InodeNumber(lowest_live_data_inode + file),
                offset: FileOffsetInBytes(index_in_file * capacity),
                bytes: PartialWriteBytes::new(&unit_bytes).expect("一个单元的净荷不是空的"),
                write_time_seconds,
            },
        )
        .unwrap_or_else(|refusal| panic!("前台第 {change_number} 次用户改动发得成：{refusal:?}"));
    write_generation_of_each_unit[usize::try_from(unit_index).expect("单元下标")] =
        write_generation;
}

/// 每块盘的单元区（CPU 多判臂对每块盘的分配记录各调一次活槽段）。
pub(super) fn unit_areas_of_every_device(session: &MountedSession) -> Vec<UnitAreaOfOneDevice> {
    session
        .allocator
        .devices
        .iter()
        .zip([DeviceIdentity(0), DeviceIdentity(1)])
        .map(|(free_map, device)| UnitAreaOfOneDevice {
            device,
            start: free_map.unit_area_start(),
            slots: free_map.unit_area_slots(),
        })
        .collect()
}

/// CPU 多判臂在窗口之内做的那一段：这次调用发出的每一批把三段的 CPU 写法各再跑一遍，里层计时在这条线程的 CPU 钟上量。
pub(super) fn rejudge_the_batches_of_the_call_on_the_cpu(
    call: &BackgroundCompactionCall,
    devices: &PoolDevices,
    unit_areas: &[UnitAreaOfOneDevice],
) -> (Option<Duration>, RejudgeTally) {
    let BackgroundCompactionCall::Ran { run, .. } = call else {
        return (Some(Duration::ZERO), RejudgeTally::default());
    };
    let (tally, spent) = thread_cpu_spent_in(|| {
        let mut tally = RejudgeTally::default();
        for version in run
            .versions_published
            .iter()
            .filter(|version| !version.units_relocated_by_this_publish.is_empty())
        {
            tally.add(rejudged_batch_on_the_cpu(version, devices, unit_areas));
        }
        tally
    });
    (spent, tally)
}
