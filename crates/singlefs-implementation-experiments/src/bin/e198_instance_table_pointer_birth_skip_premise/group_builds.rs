use super::*;

// ============================================================================
// 九、每一组怎么造（登记第五节取样域表），每一版之后按组取镜像、当场判（不留整份镜像，一段历史只留读数）。
// ============================================================================

/// 清扫那一形（sw 三组与 G1）的读数：清扫那个实例的挂载怎么扫的、清扫那一版、按读码推的 b + 2 + w。
#[derive(Clone, Debug, Default)]
pub(super) struct SweepReading {
    pub(super) sweep_instance: u32,
    pub(super) outcome: String,
    pub(super) sweep_version: Option<VersionKey>,
    pub(super) last_txg_before_the_sweep_instance: u64,
    pub(super) predicted_sweep_txg: u64,
    pub(super) instances_mounted_before_the_sweep: u32,
}

#[derive(Clone, Debug)]
pub(super) struct HistoryResult {
    pub(super) specification: HistorySpecification,
    pub(super) name: String,
    pub(super) images: Vec<ImageResult>,
    /// 写者在哪一步返回 `Err`（合法结局、之后没有镜像）；没停是 `None`。
    pub(super) stopped: Option<String>,
    pub(super) sweep: Option<SweepReading>,
    pub(super) premise: PremiseCounts,
    pub(super) ledger: Ledger,
}

/// 暖机次数的读码推法（`anchors.py` 的 `w_of`，登记第四节第 4 条）：写行那一版落在哪块盘，补到两块盘都有本实例的根，至多 3 次。
fn predicted_warm_up_publishes(row_publish_txg: u64) -> u64 {
    let device_of = |txg: u64| {
        REGION_DEVICES[usize::try_from(txg % ROOT_RING_REGIONS).expect("区域号装得进 usize")]
    };
    let mut covered = BTreeSet::from([device_of(row_publish_txg)]);
    let mut warm_ups = 0;
    let mut txg = row_publish_txg;
    while covered.len() < 2 && warm_ups < 3 {
        txg += 1;
        covered.insert(device_of(txg));
        warm_ups += 1;
    }
    warm_ups
}

/// mkfs → 实例 1 挂载 → 第一个文件 → 用户写到 txg `last`。
pub(super) fn build_the_first_instance(
    builder: &mut HistoryBuilder,
    configuration: MountConfiguration,
    last: u64,
    user_write: UserWrite,
) -> Result<Session, String> {
    builder.make_filesystem()?;
    let (mut session, _) = builder.mount(configuration)?;
    builder.publish(&mut session, Publish::FirstFile)?;
    while session.last_key.txg < last {
        builder.publish(&mut session, Publish::UserWrite(user_write))?;
    }
    Ok(session)
}

fn write_until(
    builder: &mut HistoryBuilder,
    session: &mut Session,
    last: u64,
    user_write: UserWrite,
) -> Result<(), String> {
    while session.last_key.txg < last {
        builder.publish(session, Publish::UserWrite(user_write))?;
    }
    Ok(())
}

struct ImageSink<'images> {
    pub(super) images: &'images mut Vec<ImageResult>,
    pub(super) run_the_pool_checker: bool,
}

impl ImageSink<'_> {
    fn judge_now(&mut self, builder: &HistoryBuilder, label: String, step: u64) {
        self.images.push(judge_the_image(
            &builder.image,
            &builder.ledger,
            label,
            step,
            self.run_the_pool_checker,
            Vec::new(),
        ));
    }

    fn judge_with_corrupted_root_slots(
        &mut self,
        builder: &HistoryBuilder,
        versions: &[VersionKey],
        label: String,
        step: u64,
    ) {
        let (image, ledger, skipped) = builder.copy_with_corrupted_root_slots(versions);
        self.images.push(judge_the_image(
            &image,
            &ledger,
            label,
            step,
            self.run_the_pool_checker,
            skipped,
        ));
    }
}

/// ss：实例 1 覆盖写到 last → last + 1 覆盖写崩 → 实例 2 产品挂载 → 顺序写 k 次，k = 0 … 3S 每个 k 一张。
fn build_the_short_ring(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    last: u64,
    chunks: u64,
) -> Result<(), String> {
    let first_instance = build_the_first_instance(
        builder,
        MountConfiguration::Product,
        last,
        UserWrite::Overwrite,
    )?;
    builder.publish_crashing(first_instance, Publish::UserWrite(UserWrite::Overwrite))?;
    let (mut session, _) = builder.mount(MountConfiguration::Product)?;
    sink.judge_now(builder, "publishes_after_the_mount=0".to_string(), 0);
    let content_bytes =
        usize::try_from(chunks).expect("块数装得进 usize") * SEQUENTIAL_WRITE_CHUNK_BYTES;
    for step in 1..=3 * builder.slots_per_region() {
        builder.publish(
            &mut session,
            Publish::UserWrite(UserWrite::SequentialWrite { content_bytes }),
        )?;
        sink.judge_now(builder, format!("publishes_after_the_mount={step}"), step);
    }
    Ok(())
}

/// skr：实例 1 到 last → 实例 2 产品挂载崩在写行那次根槽写之前 → 实例 3 产品挂载 → 覆盖写 k 次，k = 0 … 14。
fn build_the_skipped_row(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    last: u64,
) -> Result<(), String> {
    build_the_first_instance(
        builder,
        MountConfiguration::Product,
        last,
        UserWrite::Overwrite,
    )?;
    builder.mount_crashing_at_the_first_root_slot_write(MountConfiguration::Product)?;
    let (mut session, _) = builder.mount(MountConfiguration::Product)?;
    sink.judge_now(builder, "publishes_after_the_mount=0".to_string(), 0);
    for step in 1..=14 {
        builder.publish(&mut session, Publish::UserWrite(UserWrite::Overwrite))?;
        sink.judge_now(builder, format!("publishes_after_the_mount={step}"), step);
    }
    Ok(())
}

/// nrb：实例 1 到 last → 实例 2 产品挂载（写行 r）→ 用户写 k 次 → 读坏 r … r + bad − 1 的根槽；实例 1 的覆盖写与挂载之后的写同一种写法
/// （线索模型 `rerun-all.sh` 的 nrb 段：顺序写那一组 `PROTO_SEQ=1` 管到实例 1 的每次覆盖写）。
fn build_the_newest_row_bad(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    last: u64,
    steps_after_the_mount: u64,
    bad_slots: u64,
    user_write: UserWrite,
) -> Result<(), String> {
    build_the_first_instance(builder, MountConfiguration::Product, last, user_write)?;
    let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
    let row_publish = summary
        .row_publish
        .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
    for _ in 0..steps_after_the_mount {
        builder.publish(&mut session, Publish::UserWrite(user_write))?;
    }
    let bad: Vec<VersionKey> = (row_publish.txg..row_publish.txg + bad_slots)
        .map(|txg| VersionKey {
            instance: row_publish.instance,
            txg,
        })
        .collect();
    let label = format!(
        "bad={}",
        bad.iter()
            .map(|key| key.text())
            .collect::<Vec<_>>()
            .join(",")
    );
    sink.judge_with_corrupted_root_slots(builder, &bad, label, 0);
    Ok(())
}

/// rs1：实例 1 到 10 → 实例 2 产品挂载（写行 r）→ 回退到 (实例 1, target)（那一版 v）→ 动作串 → R ∈ {v, …, v + 动作数} 每个一张，
/// 读坏实例 2 在 [r, R) 里的每一版的根槽（R 成了洞右边第一条读得出的根）。
fn build_the_rollback_slot_bad(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    target: u64,
    actions: &[RollbackAction],
) -> Result<(), String> {
    build_the_first_instance(
        builder,
        MountConfiguration::Product,
        ROLLBACK_FIRST_INSTANCE_LAST,
        UserWrite::Overwrite,
    )?;
    let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
    let row_publish = summary
        .row_publish
        .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
    let rollback = builder.roll_back(&mut session, target)?;
    for action in actions {
        builder.publish(
            &mut session,
            Publish::RollbackAction {
                action: *action,
                target,
            },
        )?;
    }
    let last_txg = session.last_key.txg;
    for (step, first_readable_txg) in (rollback.txg..=last_txg).enumerate() {
        let bad: Vec<VersionKey> = builder
            .ledger
            .versions
            .keys()
            .filter(|key| {
                key.instance == row_publish.instance
                    && key.txg >= row_publish.txg
                    && key.txg < first_readable_txg
            })
            .copied()
            .collect();
        sink.judge_with_corrupted_root_slots(
            builder,
            &bad,
            format!("R={first_readable_txg}"),
            u64::try_from(step).expect("步号装得进 u64"),
        );
    }
    Ok(())
}

/// sw 三组与 G1（登记第五节取样域表、第八节 G1）。
fn build_the_sweep(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    first_instance_last: u64,
    extra_overwrites: u64,
    threshold: u64,
    accumulate_rows_by_mounts: bool,
    variant: SweepVariant,
) -> Result<SweepReading, String> {
    let configuration = MountConfiguration::SweepAtTheEndAboveTheThreshold { threshold };
    let ring_slots = 3 * builder.slots_per_region();
    build_the_first_instance(
        builder,
        configuration,
        first_instance_last,
        UserWrite::Overwrite,
    )?;
    let (
        mut sweep_session,
        sweep_summary,
        last_txg_before_the_sweep_instance,
        instances_mounted_before_the_sweep,
    ) = if accumulate_rows_by_mounts {
        let mut mounted_without_a_sweep = 0;
        loop {
            let before = builder
                .ledger
                .versions
                .keys()
                .max_by_key(|key| (key.txg, key.instance))
                .copied();
            let (session, summary) = builder.mount(configuration)?;
            if summary.sweep_version.is_some() {
                let last_version_before_the_sweep_mount =
                    before.ok_or_else(|| "清扫那次挂载之前账本是空的".to_string())?;
                break (
                    session,
                    summary,
                    last_version_before_the_sweep_mount.txg,
                    mounted_without_a_sweep,
                );
            }
            mounted_without_a_sweep += 1;
            if summary.instance >= SWEEP_ROW_ACCUMULATION_INSTANCES_AT_MOST {
                return Err(format!(
                    "挂到实例 {} 还没扫（阈值 {threshold}）：清扫那一形没造出来",
                    summary.instance
                ));
            }
        }
    } else {
        let (mut second, _) = builder.mount(configuration)?;
        let last_txg_before_the_sweep_instance =
            first_instance_last + ring_slots + extra_overwrites;
        write_until(
            builder,
            &mut second,
            last_txg_before_the_sweep_instance,
            UserWrite::Overwrite,
        )?;
        drop(second);
        let (session, summary) = builder.mount(configuration)?;
        (session, summary, last_txg_before_the_sweep_instance, 1)
    };
    let reading = SweepReading {
        sweep_instance: sweep_summary.instance,
        outcome: sweep_summary.sweep_outcome.clone(),
        sweep_version: sweep_summary.sweep_version,
        last_txg_before_the_sweep_instance,
        predicted_sweep_txg: last_txg_before_the_sweep_instance
            + 2
            + predicted_warm_up_publishes(last_txg_before_the_sweep_instance + 1),
        instances_mounted_before_the_sweep,
    };
    let last_before_the_crash = last_txg_before_the_sweep_instance + ring_slots - 1;
    match variant {
        SweepVariant::ReadFaultOnTheSweepInstance => {
            let first_version_on_the_image = sweep_session.last_key.txg;
            let bad: Vec<VersionKey> = builder
                .ledger
                .versions
                .keys()
                .filter(|key| {
                    key.instance == sweep_summary.instance && key.txg < first_version_on_the_image
                })
                .copied()
                .collect();
            sink.judge_with_corrupted_root_slots(
                builder,
                &bad,
                format!("version_txg={first_version_on_the_image}"),
                0,
            );
            let mut step = 0;
            while sweep_session.last_key.txg < last_before_the_crash {
                builder.publish(&mut sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
                step += 1;
                sink.judge_with_corrupted_root_slots(
                    builder,
                    &bad,
                    format!("version_txg={}", sweep_session.last_key.txg),
                    step,
                );
            }
        }
        SweepVariant::CrashBeforeTheNextInstance | SweepVariant::NoCrash => {
            write_until(
                builder,
                &mut sweep_session,
                last_before_the_crash,
                UserWrite::Overwrite,
            )?;
            if variant == SweepVariant::CrashBeforeTheNextInstance {
                builder
                    .publish_crashing(sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
            } else {
                builder.publish(&mut sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
                drop(sweep_session);
            }
            let (mut next_session, _) = builder.mount(configuration)?;
            sink.judge_now(builder, "publishes_after_the_mount=0".to_string(), 0);
            for step in 1..=ring_slots {
                builder.publish(&mut next_session, Publish::UserWrite(UserWrite::Overwrite))?;
                sink.judge_now(builder, format!("publishes_after_the_mount={step}"), step);
            }
        }
    }
    Ok(reading)
}

/// 建一段历史、当场判每一张镜像；写者返回 `Err` 的那一步之后没有镜像，`stopped` 记原因（合法结局，登记第一节待认项 4）。
pub(super) fn run_the_history(specification: &HistorySpecification) -> HistoryResult {
    let mut builder = HistoryBuilder::new(specification);
    let mut images = Vec::new();
    let run_the_pool_checker = matches!(
        specification.group,
        Group::NewestRowBad | Group::RollbackSlotBad
    );
    let mut sweep = None;
    let outcome = {
        let mut sink = ImageSink {
            images: &mut images,
            run_the_pool_checker,
        };
        match &specification.shape {
            HistoryShape::ShortRing { last, chunks } => {
                build_the_short_ring(&mut builder, &mut sink, *last, *chunks)
            }
            HistoryShape::SkippedRow { last } => {
                build_the_skipped_row(&mut builder, &mut sink, *last)
            }
            HistoryShape::NewestRowBad {
                last,
                steps_after_the_mount,
                bad_slots,
                user_write,
            } => build_the_newest_row_bad(
                &mut builder,
                &mut sink,
                *last,
                *steps_after_the_mount,
                *bad_slots,
                *user_write,
            ),
            HistoryShape::RollbackSlotBad { target, actions } => {
                build_the_rollback_slot_bad(&mut builder, &mut sink, *target, actions)
            }
            HistoryShape::Sweep {
                first_instance_last,
                extra_overwrites,
                threshold,
                accumulate_rows_by_mounts,
                variant,
            } => build_the_sweep(
                &mut builder,
                &mut sink,
                *first_instance_last,
                *extra_overwrites,
                *threshold,
                *accumulate_rows_by_mounts,
                *variant,
            )
            .map(|reading| {
                sweep = Some(reading);
            }),
        }
    };
    HistoryResult {
        specification: specification.clone(),
        name: history_name(specification),
        images,
        stopped: outcome.err(),
        sweep,
        premise: premise_counts(&builder.ledger),
        ledger: builder.ledger,
    }
}

#[cfg(test)]
mod tests;
