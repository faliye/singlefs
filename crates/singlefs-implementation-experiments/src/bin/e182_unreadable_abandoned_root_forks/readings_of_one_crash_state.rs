use super::*;

// ============================================================================
// 十一、一个崩溃状态上的全部读数：两块盘都在时的恢复、每块盘单独留下的三条臂、B4 的分类。
// ============================================================================

/// 核心层 `choose_root` 在这块盘上择到的根（不看 journal、不退回见证到的那一版）。
fn crates_choice(reader: &dyn PoolReader) -> Option<RootKey> {
    let configuration = choose_system_configuration(reader).ok()?;
    choose_root(reader, &configuration).map(|root| root_key_of(&root))
}

/// 装置自己读根环、判自证的那一份（checker 的独立解码）：自证根里 (txg, 实例代号) 最大的。
fn independent_choice(view: &KeptDiskView<'_, '_>, geometry: &PoolGeometry) -> Option<RootKey> {
    checker_image::valid_roots(view, geometry)
        .into_iter()
        .map(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg))
        .max_by_key(|(instance, txg)| (*txg, *instance))
}

/// 实例代号之差 `配置 − 所选根` 写进计数名：负数前面写 minus。
fn signed_difference_label(configuration_instance: u32, root_instance: u32) -> String {
    if configuration_instance >= root_instance {
        format!("{}", configuration_instance - root_instance)
    } else {
        format!("minus_{}", root_instance - configuration_instance)
    }
}

fn pick_of(
    history: &RecordedHistory,
    image: &CrashImage<'_>,
    key: Option<RootKey>,
    only_on_device: Option<u32>,
) -> Result<Option<RootPick>, String> {
    let Some(key) = key else {
        return Ok(None);
    };
    let log = history
        .roots
        .get(&key)
        .ok_or_else(|| format!("停机：恢复择到的根 {key:?} 不在全知日志里"))?;
    let covered = some_unit_of_the_root_is_covered(image, log, only_on_device);
    Ok(Some(classify_root_pick(key, history.hidden_root, covered)))
}

/// B4 的类（登记 5.2）：类 a = 被藏根已确认且它引用的单元两块盘上都没被盖；类 b = 被藏根引用的单元至少一个被盖了；其余记类 c。
fn class_of_the_hidden_root(
    confirmation: Confirmation,
    some_unit_is_covered: bool,
) -> &'static str {
    match (some_unit_is_covered, confirmation) {
        (true, _) => "class_some_unit_covered",
        (false, Confirmation::Confirmed) => "class_confirmed_none_covered",
        (false, Confirmation::Unconfirmed) => "class_unconfirmed_none_covered",
    }
}

/// 一个状态里被藏根属于 B4 的哪一类：两块盘上它引用的单元都看（不只看某一块盘）。
pub(super) fn class_of_the_hidden_root_in_the_state(
    image: &CrashImage<'_>,
    hidden_log: &RootLog,
    confirmation: Confirmation,
) -> &'static str {
    let some_unit_is_covered = some_unit_of_the_root_is_covered(image, hidden_log, None);
    class_of_the_hidden_root(confirmation, some_unit_is_covered)
}

/// 根环里区域 / 槽的落点公式（登记第五节「根环区域 = txg mod 3」）：第 txg 次发布写区域 txg mod 3 的槽 (txg div 3) mod S。
pub(super) fn ring_slot_of_txg(txg: u64) -> (u64, u64) {
    (
        txg % ROOT_RING_REGIONS_LOCAL,
        (txg / ROOT_RING_REGIONS_LOCAL) % ROOT_RING_SLOTS_PER_REGION_LOCAL,
    )
}

fn count_the_pick(counters: &mut Counters, prefix: &str, pick: Option<RootPick>) {
    match pick {
        None => counters.add(&format!("{prefix}pick.none"), 1),
        Some(root_pick) => counters.add(&format!("{prefix}pick.{}", root_pick.name()), 1),
    }
}

fn count_the_arm_against_today(
    counters: &mut Counters,
    prefix: &str,
    today_pick: Option<RootPick>,
    arm_pick: Option<RootPick>,
    arm: &ArmResult,
) {
    let avoided = arm.avoided_roots > 0;
    if avoided {
        counters.add(&format!("{prefix}avoided"), 1);
    }
    if arm.could_not_mask {
        counters.add(&format!("{prefix}could_not_mask"), 1);
    }
    let after = match arm_pick {
        None => "none",
        Some(RootPick::Legal) => "legal",
        Some(RootPick::HiddenRootIntact) => "hidden_root_intact",
        Some(RootPick::HiddenRootCovered) => "hidden_root_covered",
        Some(RootPick::OtherRootCovered) => "other_root_covered",
    };
    match today_pick {
        Some(RootPick::Legal) if avoided => {
            counters.add(&format!("{prefix}avoided_a_legal_root"), 1);
            counters.add(&format!("{prefix}avoided_a_legal_root.then_{after}"), 1);
        }
        Some(RootPick::HiddenRootIntact) if avoided => {
            counters.add(&format!("{prefix}avoided_an_intact_hidden_root"), 1);
            counters.add(
                &format!("{prefix}avoided_an_intact_hidden_root.then_{after}"),
                1,
            );
        }
        Some(RootPick::HiddenRootCovered) => {
            counters.add(&format!("{prefix}on_a_covered_hidden_root"), 1);
            counters.add(&format!("{prefix}on_a_covered_hidden_root.then_{after}"), 1);
        }
        Some(RootPick::OtherRootCovered) if avoided => {
            counters.add(&format!("{prefix}avoided_another_covered_root"), 1);
            counters.add(
                &format!("{prefix}avoided_another_covered_root.then_{after}"),
                1,
            );
        }
        Some(RootPick::Legal | RootPick::HiddenRootIntact | RootPick::OtherRootCovered) | None => {}
    }
}

pub(super) fn evaluate_one_state(
    history: &RecordedHistory,
    persisted: &[bool],
    counters: &mut Counters,
) -> Result<(), String> {
    let image = CrashImage {
        base: &history.base,
        writes: &history.writes,
        persisted: persisted.to_vec(),
    };
    counters.add("states", 1);
    // 两块盘都在时的恢复（对照：C554 行写的「两块盘都在时 0 失败」）。
    let both_report = recover(&image, JournalPolicy::Consult);
    counters.add(
        &format!("both.outcome.{}", outcome_class(&both_report.outcome)),
        1,
    );
    let both_pick = pick_of(history, &image, outcome_root(&both_report.outcome), None)?;
    count_the_pick(counters, "both.", both_pick);
    for device_number in [0u32, 1u32] {
        let device = DeviceIdentity(device_number);
        let kept = kept_disk(&image, device);
        let by_crates = crates_choice(&kept);
        let by_the_device = independent_choice(&kept, &history.pool_geometry);
        if by_crates != by_the_device {
            return Err(format!(
                "停机：只留盘 {device_number} 时核心层择根 {by_crates:?} 与装置独立解码的 {by_the_device:?} 不同"
            ));
        }
        let today_report = recover(&kept, JournalPolicy::Consult);
        let today = arm_result_on_the_kept_disk(
            &image,
            device,
            &history.pool_geometry,
            Arm::Today,
            &today_report,
        );
        let today_pick = pick_of(history, &image, today.key, Some(device_number))?;
        let configuration_instance_on_the_disk =
            newest_configuration_instance_on_the_kept_disk(&kept, device);
        for arm in Arm::EVERY_ARM {
            let result = match arm {
                Arm::Today => today.clone(),
                Arm::NewerInstanceRecord | Arm::ConfigurationInstance => {
                    arm_result_on_the_kept_disk(
                        &image,
                        device,
                        &history.pool_geometry,
                        arm,
                        &today_report,
                    )
                }
            };
            let arm_pick = pick_of(history, &image, result.key, Some(device_number))?;
            let prefix = format!("{}.kept{device_number}.", arm.name());
            counters.add(&format!("{prefix}states"), 1);
            counters.add(&format!("{prefix}outcome.{}", result.outcome), 1);
            count_the_pick(counters, &prefix, arm_pick);
            if arm != Arm::Today {
                count_the_arm_against_today(counters, &prefix, today_pick, arm_pick, &result);
            }
        }
        if let (Some(configuration_instance), Some((root_instance, _))) =
            (configuration_instance_on_the_disk, today.key)
        {
            counters.add(
                &format!(
                    "configuration_instance.kept{device_number}.difference.{}",
                    signed_difference_label(configuration_instance, root_instance)
                ),
                1,
            );
        }
    }
    count_the_hidden_root_classification(history, &image, persisted, counters);
    Ok(())
}

/// B4：实例 2 写行那次发布持久之后的状态，按被藏根的单元有没有被盖分类；另记今天第一次发布的 txg 落在被藏根那一槽（txg 撞上）的状态。
fn count_the_hidden_root_classification(
    history: &RecordedHistory,
    image: &CrashImage<'_>,
    persisted: &[bool],
    counters: &mut Counters,
) {
    let (Some(hidden_root), Some(row_write_index)) =
        (history.hidden_root, history.row_publish_root_write)
    else {
        return;
    };
    if !persisted[row_write_index] {
        return;
    }
    let Some(hidden_log) = history.roots.get(&hidden_root) else {
        return;
    };
    let class = class_of_the_hidden_root_in_the_state(
        image,
        hidden_log,
        history.specification.family.confirmation(),
    );
    counters.add("after_the_row_publish.states", 1);
    counters.add(&format!("after_the_row_publish.{class}"), 1);
    let row_root = history.mount.row_publish_root;
    let collides = row_root
        .is_some_and(|(_, row_txg)| ring_slot_of_txg(row_txg) == ring_slot_of_txg(hidden_root.1));
    if collides {
        counters.add(
            "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot",
            1,
        );
        counters.add(
            &format!(
                "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.{class}"
            ),
            1,
        );
    }
    let hidden_root_still_in_the_ring = checker_image::valid_roots(image, &history.pool_geometry)
        .into_iter()
        .any(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg) == hidden_root);
    if !hidden_root_still_in_the_ring {
        counters.add("after_the_row_publish.hidden_root_slot_overwritten", 1);
        counters.add(
            &format!("after_the_row_publish.hidden_root_slot_overwritten.{class}"),
            1,
        );
    }
}

#[cfg(test)]
mod tests;
