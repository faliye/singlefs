//! 根环每一槽在 I-8.14、I-3.19 眼里的读法，I-8.14（新实例第一个 txg 高于更早实例的根与记录） 与
//! I-3.19（实例表行的 W 是一次恢复施加得出的最大事务号）。

use super::*;

/// 根环一槽在 I-8.14、I-3.19 眼里的样子（
/// 全零的槽可以是从没写过的，不算读不出）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootRingSlotReading {
    /// 自证过的本池的根（[`valid_roots`] 收下的那一种）。
    SelfVerified { checkpoint_txg: u64 },
    /// 读得到、整槽全零：可以是从没写过的槽。
    AllZero,
    /// 读不到（盘不在、越过盘尾），或不全零而不自证（撕了、位腐、别的池的根）：里面可能坐着一条读不出的根。
    DoesNotReadBack,
}

/// 根环每一槽（区域、区域里第几槽）在 I-8.14、I-3.19 眼里的样子，按 [`root_slot_positions`] 同一个算式读。
pub(super) fn root_ring_slot_readings(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
) -> BTreeMap<(u64, u64), RootRingSlotReading> {
    let slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    root_slot_positions(geometry)
        .into_iter()
        .map(|(region, slot, device, offset)| {
            let reading = match reader.read(device, offset, slot_bytes) {
                None => RootRingSlotReading::DoesNotReadBack,
                Some(bytes) => {
                    match crate::check_root_slot(&bytes, &geometry.filesystem_identifier) {
                        Ok(view) => RootRingSlotReading::SelfVerified {
                            checkpoint_txg: view.checkpoint_txg,
                        },
                        Err(_) if bytes.iter().all(|byte| *byte == 0) => {
                            RootRingSlotReading::AllZero
                        }
                        Err(_) => RootRingSlotReading::DoesNotReadBack,
                    }
                }
            };
            ((region, slot), reading)
        })
        .collect()
}

/// I-8.14 里一个实例的根与记录，txg 最小的那一条与最大的那一条（各带说明）。
struct CarriersOfAnInstance {
    lowest: (u64, String),
    highest: (u64, String),
}

impl CarriersOfAnInstance {
    pub(super) fn note(&mut self, checkpoint_txg: u64, what: impl Fn() -> String) {
        if checkpoint_txg < self.lowest.0 {
            self.lowest = (checkpoint_txg, what());
        }
        if checkpoint_txg > self.highest.0 {
            self.highest = (checkpoint_txg, what());
        }
    }
}

/// [`FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES`]（I-8.14，C764）：环里读得出的根与记录按实例代号归组，
/// 每个实例 k 的第一个 txg f_k 取它那一组里最小的 txg；实例代号小于 k 的每一组里最大的 txg 要严格小于 f_k（D23（journal 的角色与格式）
/// 已定项 14 注 3「新实例第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」）。
/// 只比实例代号更小的。
/// 每个有更小实例可比的 k 判一次。读得出：根是 `roots`（根环里自证过的本池的根），记录是恢复眼里的环（[`records_as_the_recovery_reads_them`]：
/// 恢复的读者收得下、按 (实例代号, 计数器) 跨盘取第一份），与挂载取第一个 txg 时读的同一份（core 的 `first_txg_of_new_instance` 读扫环收下的全部记录，
/// 不看落点；落点对不上的记录归 I-8.13）。根环或 journal 环里有读不出的槽（[`RootRingSlotReading::DoesNotReadBack`]、`journal_slots_that_do_not_read_back_by_device`）
/// 时整条报部分没判：那一槽里可能坐着一条更小实例、txg 更大的根或记录。
/// 挂载那一刻读不出、今天读得出的（瞬时读故障不留在盘上）这里看不见，是射程外。
pub(super) fn judge_the_first_txg_of_each_instance_above_the_older_instances(
    roots: &[(u64, u64, crate::RootView)],
    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    journal_slots_that_do_not_read_back_by_device: &[(u32, Vec<u64>)],
    judgements: &mut Judgements,
) {
    let invariant =
        FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES;
    let mut carriers_by_instance: BTreeMap<u32, CarriersOfAnInstance> = BTreeMap::new();
    let mut note_carrier =
        |instance: u32, checkpoint_txg: u64, what: &dyn Fn() -> String| match carriers_by_instance
            .get_mut(&instance)
        {
            Some(carriers) => carriers.note(checkpoint_txg, what),
            None => {
                carriers_by_instance.insert(
                    instance,
                    CarriersOfAnInstance {
                        lowest: (checkpoint_txg, what()),
                        highest: (checkpoint_txg, what()),
                    },
                );
            }
        };
    for (region, slot, root) in roots {
        note_carrier(root.instance, root.checkpoint_txg, &|| {
            format!(
                "根环区域 {region} 槽 {slot} 上实例 {}、txg {} 的根",
                root.instance, root.checkpoint_txg
            )
        });
    }
    for ((instance, counter), record) in records_as_the_recovery_reads_them {
        note_carrier(*instance, record.checkpoint_txg, &|| {
            format!(
                "计数器 {counter}（环槽 {}）上实例 {instance}、txg {} 的记录",
                record.slot, record.checkpoint_txg
            )
        });
    }
    let mut highest_of_the_older_instances: Option<(u64, &str)> = None;
    let mut instances_compared = 0u64;
    for (instance, carriers) in &carriers_by_instance {
        if let Some((older_highest_txg, older_highest_what)) = highest_of_the_older_instances {
            instances_compared += 1;
            let (first_txg, first_what) = (&carriers.lowest.0, &carriers.lowest.1);
            judgements.judge(invariant, older_highest_txg < *first_txg, || {
                format!(
                    "实例 {instance} 的第一个 txg 是 {first_txg}（{first_what}），不高于更早实例的 {older_highest_what}（txg {older_highest_txg}）"
                )
            });
        }
        let replaces_the_highest = highest_of_the_older_instances
            .is_none_or(|(older_highest_txg, _)| carriers.highest.0 > older_highest_txg);
        if replaces_the_highest {
            highest_of_the_older_instances =
                Some((carriers.highest.0, carriers.highest.1.as_str()));
        }
    }
    for ((region, slot), reading) in root_ring_slot_readings {
        match reading {
            RootRingSlotReading::DoesNotReadBack => judgements.part_of_the_range_left_unjudged(
                invariant,
                format!("根环区域 {region} 槽 {slot} 读不出（读不到，或不全零而不自证）"),
            ),
            RootRingSlotReading::SelfVerified { .. } | RootRingSlotReading::AllZero => {}
        }
    }
    for (device, slots) in journal_slots_that_do_not_read_back_by_device {
        if let Some(first_slot) = slots.first() {
            judgements.part_of_the_range_left_unjudged(
                invariant,
                format!(
                    "盘 {device} 的 journal 环有 {} 槽读不出（读不到，或不全零而不自证；第一槽是环槽 {first_slot}）",
                    slots.len()
                ),
            );
        }
    }
    if instances_compared == 0 {
        judgements.not_applicable(invariant, NO_INSTANCE_WITH_AN_OLDER_INSTANCE_TO_COMPARE);
    }
}

/// I-3.19 一行在一个起点上能不能认出那次恢复施加出的最大事务号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HighWaterReachableFromARoot {
    /// (c, T] 里每个 txg 都读得到这个实例带末条标志的记录：值是那一段记录里最大的非 0 事务号（没有非 0 的取 0）。
    Reachable(u64),
    /// (c, T] 里有一个 txg 读不到这个实例带末条标志的记录（被环盖掉、撕了、读不出）。
    SomePublishDoesNotReadBack,
}

/// 实例 `instance` 从 txg 为 `from_txg` 的根起重放到 `to_txg`：(from_txg, to_txg] 里记录的最大非 0 事务号，与这一段读没读全
/// （同一实例的发布 txg 逐一递增，每个 txg 要读到一条带末条标志的记录）。
fn high_water_reachable_from_a_root(
    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    instance: u32,
    from_txg: u64,
    to_txg: u64,
) -> HighWaterReachableFromARoot {
    // 带末条标志的记录按计数器次序数，txg 与上一条数过的相同的不重数（同一实例 txg 随计数器不减时就是数不同的 txg；
    // 不减不成立的坏镜像归 I-8.12，这里多数出来就算读不全）。GPU 判器同一个数法。
    let mut publishes_read_back = 0u64;
    let mut txg_of_the_last_publish_counted: Option<u64> = None;
    let mut high_water = 0u64;
    for (_, record) in
        records_as_the_recovery_reads_them.range((instance, u64::MIN)..=(instance, u64::MAX))
    {
        if record.checkpoint_txg <= from_txg || record.checkpoint_txg > to_txg {
            continue;
        }
        high_water = high_water.max(record.transaction);
        if record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
            && txg_of_the_last_publish_counted != Some(record.checkpoint_txg)
        {
            publishes_read_back += 1;
            txg_of_the_last_publish_counted = Some(record.checkpoint_txg);
        }
    }
    let publishes_in_the_span = to_txg - from_txg;
    if publishes_read_back == publishes_in_the_span {
        HighWaterReachableFromARoot::Reachable(high_water)
    } else {
        HighWaterReachableFromARoot::SomePublishDoesNotReadBack
    }
}

/// 根环里有没有一槽可能藏着一条 txg 小于 `published_txg`、已被盖掉或读不出的根（它可能就是那次挂载择到的起点）：一槽读不出，
/// 或坐着的根 txg 大于这一槽能住的最小正 txg 而那个 txg 又小于 `published_txg`（环转过之后，同一槽更早那几圈的根被盖掉了）。
/// 落点式同 [`root_ring_slot_the_txg_names`]：区域 r、槽 s 能住的 txg 是 R × (s + S × 圈数) + r。全零的槽一条根都没住过。
/// 有就交回说明（第一处），没有交回 `None`。
fn root_ring_slot_that_may_hide_an_overwritten_root_below(
    published_txg: u64,
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    geometry: &PoolGeometry,
) -> Option<String> {
    let root_ring_slot_count = geometry.regions.saturating_mul(geometry.slots_per_region);
    root_ring_slot_readings
        .iter()
        .find_map(|((region, slot), reading)| {
            let smallest_positive_txg_of_the_slot = match geometry
                .regions
                .saturating_mul(*slot)
                .saturating_add(*region)
            {
                0 => root_ring_slot_count,
                smallest => smallest,
            };
            let txg_the_slot_was_overwritten_up_to = match reading {
                RootRingSlotReading::SelfVerified { checkpoint_txg } => *checkpoint_txg,
                RootRingSlotReading::AllZero => return None,
                RootRingSlotReading::DoesNotReadBack => u64::MAX,
            };
            (smallest_positive_txg_of_the_slot < txg_the_slot_was_overwritten_up_to.min(published_txg))
                .then(|| {
                    format!(
                        "根环区域 {region} 槽 {slot}（{reading:?}）里可能有一条 txg 小于 T 的根被盖掉或读不出，那次挂载可能就从它起重放"
                    )
                })
        })
}

/// [`INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED`]（I-3.19，C740）：
/// 最新根实例表里每一行 (i, T, W)，
/// W 落在这一行容许的集合里。写这一行的那次挂载择到哪条根盘上认不出（一时读坏的根槽不留在盘上），
/// 所以集合把每个可能的起点都收进来：落在 T 上（T 那一槽坐着 txg ≥ T 的根，或 T 是 0）或退到 T 而不重放时含 0；
/// 从环里实例 i、txg 小于 T 的每条根 c 起重放到 T 时含 (c, T] 里的最大非 0 事务号（[`high_water_reachable_from_a_root`]）。
/// 认不出的那几格（T 那一槽读不出、(c, T] 读不全、根环里有一槽可能藏着被盖掉或读不出的更早的根，
/// [`root_ring_slot_that_may_hide_an_overwritten_root_below`]）可能就是 W：W 不在认得出的那几格里时这一行没判、报部分没判，不报违例。
/// 「被盖掉的起点」那一格是实现员加的，交主 agent 核。
/// 实例表链没走到末片：链断之后的行没读到，报部分没判。
pub(super) fn judge_the_applied_transaction_high_water_of_the_instance_table_rows(
    instance_table_rows: &[InstanceTableRow],
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    roots: &[(u64, u64, crate::RootView)],
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    geometry: &PoolGeometry,
    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    judgements: &mut Judgements,
) {
    let invariant = INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED;
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            judgements.part_of_the_range_left_unjudged(
                invariant,
                "最新根的实例表链没走到末片：链断之后那几片的行没读到".to_string(),
            );
        }
    }
    for row in instance_table_rows {
        let (instance, published_txg, high_water) = (
            row.instance,
            row.published_checkpoint_txg,
            row.applied_transaction_high_water_mark,
        );
        let mut reachable_high_waters: BTreeSet<u64> = BTreeSet::new();
        let mut unreadable_starting_points: Vec<String> = Vec::new();
        if published_txg == 0 {
            reachable_high_waters.insert(0);
        } else {
            let (region, slot) = root_ring_slot_the_txg_names(published_txg, geometry);
            match root_ring_slot_readings.get(&(region, slot)) {
                Some(RootRingSlotReading::SelfVerified { checkpoint_txg })
                    if *checkpoint_txg >= published_txg =>
                {
                    reachable_high_waters.insert(0);
                }
                Some(RootRingSlotReading::SelfVerified { .. } | RootRingSlotReading::AllZero) => {}
                Some(RootRingSlotReading::DoesNotReadBack) | None => {
                    unreadable_starting_points.push(format!(
                        "T 那一槽（区域 {region} 槽 {slot}）读不出，认不出 T 落没落过盘"
                    ));
                }
            }
        }
        for (_, _, root) in roots {
            if root.instance != instance || root.checkpoint_txg >= published_txg {
                continue;
            }
            match high_water_reachable_from_a_root(
                records_as_the_recovery_reads_them,
                instance,
                root.checkpoint_txg,
                published_txg,
            ) {
                HighWaterReachableFromARoot::Reachable(reachable) => {
                    reachable_high_waters.insert(reachable);
                }
                HighWaterReachableFromARoot::SomePublishDoesNotReadBack => {
                    unreadable_starting_points.push(format!(
                        "从 txg {} 的根起到 T 之间有发布的末条记录读不到",
                        root.checkpoint_txg
                    ));
                }
            }
        }
        if let Some(hidden) = root_ring_slot_that_may_hide_an_overwritten_root_below(
            published_txg,
            root_ring_slot_readings,
            geometry,
        ) {
            unreadable_starting_points.push(hidden);
        }
        if reachable_high_waters.contains(&high_water) {
            judgements.judge(invariant, true, String::new);
        } else if unreadable_starting_points.is_empty() {
            judgements.judge(invariant, false, || {
                format!(
                    "实例表行 (实例 {instance}, T {published_txg}, W {high_water})：W 不在这一行容许的集合 {reachable_high_waters:?} 里（落在 T 上时含 0、从环里实例 {instance} 每条更早的根起重放到 T 时含那一段的最大非 0 事务号）"
                )
            });
        } else {
            judgements.part_of_the_range_left_unjudged(
                invariant,
                format!(
                    "实例表行 (实例 {instance}, T {published_txg}, W {high_water})：W 不在认得出的集合 {reachable_high_waters:?} 里，而{}",
                    unreadable_starting_points.join("、")
                ),
            );
        }
    }
    judgements.not_applicable(invariant, NO_INSTANCE_TABLE_ROW_UNDER_THE_NEWEST_ROOT);
}

#[cfg(test)]
mod tests;
