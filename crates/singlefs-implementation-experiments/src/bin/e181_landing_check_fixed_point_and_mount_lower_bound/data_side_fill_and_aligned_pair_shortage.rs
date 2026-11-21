//! 数据侧填满与对齐对不够的阳性对照、测量、对照的跑法与汇总。

use super::*;

/// 填满那一步最多发几次（保险丝）：每次发布取走当时能发成的最大一批，正常两三次就到头；超过就停下，产物里记 `publish_limit_reached`。
const FILL_PUBLISHES_AT_MOST: usize = 16;
/// 为一次发布找「能发成的单元数」最多试几次（每次都在拷贝上试）；超过就当这一次没找到，产物里记试了几次。
const FILL_PROBES_AT_MOST_PER_PUBLISH: u64 = 40;
/// 不限每次发布的单元数（产品跑法；单测传一个小数，才能在小几何上造出好几次发布）。
pub(super) const FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH: u64 = u64::MAX;

/// 一次发成的填满发布。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FillPublish {
    /// 这一次一开始要的单元数：段外槽对数与每次发布的单元数上限中小的那个。
    pub(super) units_asked: u64,
    /// 发成的单元数。
    pub(super) units: u64,
    /// 为找到它试了几次（含发成的那一次）。
    probes: u64,
}

/// 填满为什么停。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FillStop {
    /// 段外再没有对齐的空槽对（登记 5.2 节「填满」的判法）。
    NoAlignedPairOutsideTheClusters,
    /// 核心层拒了：`units_asked` 是最后试的单元数（1 = 连一个单元的写也落不下），`refusal` 是那一次拒在哪一道，`probes` 是为这一次发布试了几次。
    Refused {
        units_asked: u64,
        refusal: RefusalClass,
        probes: u64,
    },
    /// 发了 [`FILL_PUBLISHES_AT_MOST`] 次还没到头。
    PublishLimitReached,
}

/// 填满的账：每次发布的单元数、停在哪、停时各盘段外还剩的对齐空槽对数、各盘 defer 队列里的槽数（换下的上一批在里面，仍占着槽）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FillReport {
    pub(super) publishes: Vec<FillPublish>,
    pub(super) stop: FillStop,
    pub(super) remaining_pairs_outside_the_clusters: Vec<u64>,
    pub(super) deferred_slots: Vec<u64>,
}

impl FillReport {
    /// 各次发布写下去的单元数之和（换下的批次也算：它们都占过槽对）。
    pub(super) fn filled_units(&self) -> u64 {
        self.publishes.iter().map(|publish| publish.units).sum()
    }

    /// 最后一次发布写成的文件的单元数（现行那一版里活着的数据单元数）。
    fn units_of_the_file_now(&self) -> u64 {
        self.publishes.last().map_or(0, |publish| publish.units)
    }

    /// 产物行里的一段字段（`aligned_pair_shortage_setup`、`aligned_pair_shortage_measure_setup`、`verdict_aligned_pair_shortage_controls` 共用）。
    pub(super) fn fields(&self) -> String {
        let units_per_publish: Vec<u64> =
            self.publishes.iter().map(|publish| publish.units).collect();
        let probes_per_publish: Vec<u64> = self
            .publishes
            .iter()
            .map(|publish| publish.probes)
            .collect();
        let stop = match &self.stop {
            FillStop::NoAlignedPairOutsideTheClusters => "no_aligned_pair_left_outside_the_clusters".to_string(),
            FillStop::Refused { units_asked, refusal, probes } => format!(
                "refused fill_refused_at_units={units_asked} fill_refusal_gate={} fill_refusal_probes={probes} fill_refusal_detail={:?}",
                refusal.name(),
                refusal.detail()
            ),
            FillStop::PublishLimitReached => "publish_limit_reached".to_string(),
        };
        format!(
            "fill_publishes={} fill_units_per_publish={units_per_publish:?} fill_probes_per_publish={probes_per_publish:?} filled_units={} units_of_the_file_now={} fill_stop={stop} remaining_aligned_pairs_outside_the_clusters={:?} deferred_slots={:?}",
            self.publishes.len(),
            self.filled_units(),
            self.units_of_the_file_now(),
            self.remaining_pairs_outside_the_clusters,
            self.deferred_slots,
        )
    }
}

/// 各盘段外对齐的空槽对数（两槽都没挡、起点偶数、不在开过的聚簇段里，登记 5.2 节「填满」数的就是它）。
pub(super) fn remaining_pairs_outside_the_clusters(pool: &Pool) -> Vec<u64> {
    (0..pool.devices.len())
        .map(|device_index| data_unit_pairs_outside_the_clusters(pool, device_index))
        .collect()
}

/// 被拒之后，下一次试几个单元（`units` 是刚被拒的那个数，至少 2）：
/// journal 记录数超限 ⇒ 减去超出的条数；式子或落得下那一判不够 ⇒ 减去缺口折成的单元数（缺口字节 ÷ 一个单元在一块盘上的两槽，向上取整，
/// 缺口读数不是正数时按一字节算）；落点那一道拒与别的拒因 ⇒ 折半。结果夹进 [1, `units` − 1]，所以单元数每次严格减小、不会降到 0。
pub(super) fn units_to_try_after_a_refusal(refusal: &RefusalClass, units: u64) -> u64 {
    assert!(units >= 2, "刚被拒的单元数至少 2，才有更小的可试：{units}");
    let units_wide = i128::from(units);
    let wanted = match refusal {
        RefusalClass::Other {
            journal_records_over_the_limit: Some(surplus),
            ..
        } => units_wide - i128::from(*surplus),
        RefusalClass::SpaceAdmission {
            shortfall_bytes, ..
        } => units_wide - units_a_shortfall_is_worth(*shortfall_bytes),
        RefusalClass::FixedPointPlacement { .. } | RefusalClass::Other { .. } => units_wide / 2,
    };
    u64::try_from(wanted.clamp(1, units_wide - 1)).expect("夹进 [1, 上一次 − 1] 之后是正数")
}

/// [`land_the_most_units_up_to`] 的结局。
enum Landing {
    Landed(FillPublish),
    NothingLands {
        units_asked: u64,
        refusal: RefusalClass,
        probes: u64,
    },
}

/// 在 `pool` 上接着换写，先试 `asked` 个单元：每次试都在 `pool` 的拷贝上做，发成了就把那份拷贝换成 `pool`（不重写一遍）；
/// 被拒了按 [`units_to_try_after_a_refusal`] 算下一次试几个，夹进 [1, 上一次 − 1]，单元数严格减小，直到发成，或一个单元也被拒，
/// 或试满 [`FILL_PROBES_AT_MOST_PER_PUBLISH`] 次（这两样交回 `NothingLands`，带最后一次的拒因）。
fn land_the_most_units_up_to(
    pool: &mut Pool,
    asked: u64,
    content: &mut ContentBuffer,
) -> Result<Landing, String> {
    let mut units = asked;
    let mut probes = 0_u64;
    loop {
        probes += 1;
        let mut probe = pool.fork();
        let outcome = write_units_and_account(
            &mut probe,
            usize::try_from(units).expect("单元数"),
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            content,
        )?;
        let refusal = match outcome {
            WriteOutcome::Published(_) => {
                *pool = probe;
                return Ok(Landing::Landed(FillPublish {
                    units_asked: asked,
                    units,
                    probes,
                }));
            }
            WriteOutcome::Refused(refusal) => refusal,
        };
        if units == 1 || probes >= FILL_PROBES_AT_MOST_PER_PUBLISH {
            return Ok(Landing::NothingLands {
                units_asked: units,
                refusal,
                probes,
            });
        }
        units = units_to_try_after_a_refusal(&refusal, units);
    }
}

/// 把数据那一侧填满（登记 12.11）：从第一个文件起多次发布，每次接着换写（`publish_sequential_write`，从偏移 0 整份换写；
/// 核心层公开入口里给数据单元的只有这一条路，没有写别的文件、续写的入口）。一次发布受 journal 记录数上限（`JournalRecordsOfThePublishExceedTheLimit`）
/// 所限，所以每次发布先要「段外槽对数」与 `units_per_publish_at_most` 中小的那个，被拒了按 [`land_the_most_units_up_to`] 往下找能发成的最大一批；
/// 换下的上一批进 defer 队列、仍占着槽，下一次发布只能落在剩下的空槽对上。停在两处之一：段外再没有对齐的空槽对，或核心层拒了一个单元的写
/// （这时拒因、剩下的槽对数都在账里，不再往下试别的）。准入式（D28 九项）扣着环窗口、checkpoint 保留池、实例切换预留，
/// 所以填满的上限可能不是槽对数。
pub(super) fn fill_the_data_side(
    pool: &mut Pool,
    content: &mut ContentBuffer,
    units_per_publish_at_most: u64,
) -> Result<FillReport, String> {
    assert!(units_per_publish_at_most >= 1, "每次发布至少 1 个单元");
    let mut publishes: Vec<FillPublish> = Vec::new();
    let stop = loop {
        let fewest_pairs = remaining_pairs_outside_the_clusters(pool)
            .into_iter()
            .min()
            .expect("至少一块盘");
        if fewest_pairs == 0 {
            break FillStop::NoAlignedPairOutsideTheClusters;
        }
        if publishes.len() >= FILL_PUBLISHES_AT_MOST {
            break FillStop::PublishLimitReached;
        }
        match land_the_most_units_up_to(pool, fewest_pairs.min(units_per_publish_at_most), content)?
        {
            Landing::Landed(publish) => publishes.push(publish),
            Landing::NothingLands {
                units_asked,
                refusal,
                probes,
            } => {
                break FillStop::Refused {
                    units_asked,
                    refusal,
                    probes,
                }
            }
        }
    };
    Ok(FillReport {
        publishes,
        stop,
        remaining_pairs_outside_the_clusters: remaining_pairs_outside_the_clusters(pool),
        deferred_slots: pool
            .allocator
            .devices
            .iter()
            .map(|device_map| device_map.deferred_slots())
            .collect(),
    })
}

/// 阳性对照（登记 5.2 节，不可达，只当对照）：用分配器公开的隔离入口把每个空的偶数起点槽对的后一槽隔离掉，
/// G10 必须为真、今天那一臂必须拒在固定点、Q1 的判法必须先拒。
fn run_aligned_pair_shortage_positive_control(
    label: &str,
    base: &mut SessionPool,
) -> Result<bool, String> {
    let mut copy = base.fork();
    for device_map in &mut copy.session.allocator.devices {
        let first = device_map.unit_area_start().slot().0;
        let end = first + device_map.unit_area_slots();
        let mut slot = first;
        while slot + 1 < end {
            if device_map.is_free(SlotNumber(slot)) && device_map.is_free(SlotNumber(slot + 1)) {
                device_map.isolate(SlotNumber(slot + 1), 1);
            }
            slot += DATA_UNIT_SLOTS_LOCAL;
        }
    }
    let reading = RoomReading::of(&copy);
    let today = probe_one_new_inode_today(&mut copy);
    let landing_check_counting_aligned_pairs_would_refuse_first =
        reading.aligned_pairs_are_short_on_some_device();
    let control_hits = reading.holds()
        && matches!(
            today,
            RefusalOrPublished::Refused(RefusalClass::FixedPointPlacement { .. })
        )
        && landing_check_counting_aligned_pairs_would_refuse_first;
    emit(&format!(
        "name=aligned_pair_shortage_positive_control state={label} slots_enough_and_aligned_pairs_short={} slots_enough={} aligned_pairs_short={} aligned_free_pairs={:?} unblocked_slots={:?} needed_slots={} today_outcome={} landing_check_counting_aligned_pairs_would_refuse_before_the_fixed_point={landing_check_counting_aligned_pairs_would_refuse_first} control_hits={control_hits}",
        reading.holds(),
        reading.slots_are_enough_on_every_device(),
        reading.aligned_pairs_are_short_on_some_device(),
        reading.aligned_free_pairs,
        reading.unblocked_slots,
        reading.needed_slots,
        today.name(),
    ));
    Ok(control_hits)
}

/// 开跑前量 F10 的 10 步（登记 5.6 节）：G默认上填满数据那一侧、可写挂载、走 10 步，外推两个几何 × 64 种子 × 300 步要多久。
pub(super) fn run_aligned_pair_shortage_measure(content: &mut ContentBuffer) -> Result<(), String> {
    let build_started = Instant::now();
    let mut pool = build_pool_after_the_first_file(
        GEOMETRY_DEFAULT,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let built_seconds = build_started.elapsed().as_secs_f64();
    let fill_started = Instant::now();
    let fill = fill_the_data_side(&mut pool, content, FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH)?;
    let filled_seconds = fill_started.elapsed().as_secs_f64();
    let mount_started = Instant::now();
    let mut state = session_pool_of(&mut pool)?;
    drop(pool);
    let mounted_seconds = mount_started.elapsed().as_secs_f64();
    emit(&format!(
        "name=aligned_pair_shortage_measure_setup geometry={} build_seconds={built_seconds:.3} fill_seconds={filled_seconds:.3} {} mount_seconds={mounted_seconds:.3}",
        GEOMETRY_DEFAULT.name,
        fill.fields()
    ));
    let mut stream = SeededStream::of_seed(0);
    let mut tally = WalkTally::default();
    let mut step_seconds = Vec::new();
    for step in 0..10_u64 {
        let action = WALK_ACTIONS[usize::try_from(stream.below(5)).expect("下标")];
        let action_started = Instant::now();
        let result = apply_walk_action(&mut state, action)?;
        let action_seconds = action_started.elapsed().as_secs_f64();
        let observe_started = Instant::now();
        observe_state(&mut tally, &mut state, step);
        let observe_seconds = observe_started.elapsed().as_secs_f64();
        step_seconds.push(action_seconds + observe_seconds);
        emit(&format!(
            "name=aligned_pair_shortage_measure_step step={step} action={} result={result} action_seconds={action_seconds:.3} observe_seconds={observe_seconds:.3}",
            action.name()
        ));
    }
    step_seconds.sort_by(f64::total_cmp);
    let median = step_seconds[step_seconds.len() / 2];
    let total_steps = ALIGNED_PAIR_SHORTAGE_GEOMETRY_COUNT
        * ALIGNED_PAIR_SHORTAGE_SEEDS
        * ALIGNED_PAIR_SHORTAGE_STEPS;
    #[allow(
        clippy::cast_precision_loss,
        reason = "小数的步数，外推出来的小时数只报一位小数"
    )]
    let extrapolated_single_thread_hours = median * (total_steps as f64) / 3600.0;
    emit(&format!(
        "name=f10_measure_extrapolation median_step_seconds={median:.3} total_steps={total_steps} extrapolated_hours_single_thread={extrapolated_single_thread_hours:.1} budget_hours=2"
    ));
    Ok(())
}

const ALIGNED_PAIR_SHORTAGE_GEOMETRY_COUNT: u64 = 2;
pub(super) const ALIGNED_PAIR_SHORTAGE_SEEDS: u64 = 64;
pub(super) const ALIGNED_PAIR_SHORTAGE_STEPS: u64 = 300;

pub(super) fn run_aligned_pair_shortage_controls(
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<(), String> {
    let mut unfilled =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let mut unfilled_session = session_pool_of(&mut unfilled)?;
    let unfilled_hits = run_aligned_pair_shortage_positive_control(
        &format!("{}_after_the_first_file", geometry.name),
        &mut unfilled_session,
    )?;
    drop(unfilled_session);
    let fill = fill_the_data_side(
        &mut unfilled,
        content,
        FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH,
    )?;
    let mut filled_session = session_pool_of(&mut unfilled)?;
    let filled_hits = run_aligned_pair_shortage_positive_control(
        &format!("{}_data_side_filled", geometry.name),
        &mut filled_session,
    )?;
    emit(&format!(
        "name=verdict_aligned_pair_shortage_controls geometry={} {} control_hits_after_the_first_file={unfilled_hits} control_hits_data_side_filled={filled_hits}",
        geometry.name,
        fill.fields()
    ));
    Ok(())
}

pub(super) fn emit_aligned_pair_shortage_summary(
    geometry: &str,
    seeds: u64,
    steps: u64,
    total: &WalkTally,
) {
    let refusals: Vec<String> = total
        .action_refusals
        .iter()
        .map(|(action, count)| format!("{action}={count}"))
        .collect();
    let today: Vec<String> = total
        .slots_enough_and_aligned_pairs_short_today_outcomes
        .iter()
        .map(|(outcome, count)| format!("{outcome}={count}"))
        .collect();
    let session: Vec<String> = total
        .slots_enough_and_aligned_pairs_short_session_outcomes
        .iter()
        .map(|(outcome, count)| format!("{outcome}={count}"))
        .collect();
    emit(&format!(
        "name=aligned_pair_shortage_summary geometry={geometry} seeds={seeds} steps={steps} walks={} states={} slots_enough_on_every_device_states={} aligned_pairs_short_states={} slots_enough_and_aligned_pairs_short_states={} first_slots_enough_and_aligned_pairs_short_step={} aligned_pairs_peak={} aligned_pairs_positive_states={} unblocked_slots_peak={} action_refusals=[{}] slots_enough_and_aligned_pairs_short_today_outcomes=[{}] slots_enough_and_aligned_pairs_short_session_outcomes=[{}] model_checks={} placement_refusals_with_slots_enough_and_aligned_pairs_short_false={} slots_enough_and_aligned_pairs_short_probes_that_published={}",
        total.walks,
        total.states,
        total.slots_enough_states,
        total.pairs_short_states,
        total.slots_enough_and_aligned_pairs_short_states,
        total.first_slots_enough_and_aligned_pairs_short_step.map_or_else(|| "none".to_string(), |step| step.to_string()),
        total.aligned_pairs_peak,
        total.aligned_pairs_positive_states,
        total.unblocked_slots_peak,
        refusals.join(","),
        today.join(","),
        session.join(","),
        total.model_checks,
        total.placement_refusals_with_slots_enough_and_aligned_pairs_short_false,
        total.slots_enough_and_aligned_pairs_short_probes_that_published,
    ));
    emit(&format!(
        "name=verdict_aligned_pair_shortage geometry={geometry} slots_enough_and_aligned_pairs_short_reachable={} slots_enough_and_aligned_pairs_short_states={} slots_enough_and_pairs_short_never_together={}",
        slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(total.slots_enough_and_aligned_pairs_short_states, MINIMUM_SLOTS_ENOUGH_AND_ALIGNED_PAIRS_SHORT_STATES_FOR_REACHABLE),
        total.slots_enough_and_aligned_pairs_short_states,
        total.slots_enough_and_aligned_pairs_short_states == 0
    ));
}
