#!/usr/bin/env python3
"""用法：patch_arms.py <仓副本根>。只改攻方副本：加三条只靠环境变量打开的臂、把探针 crate 加进工作区。每处旧串必须恰好命中一次，不中就退 2。
臂（环境变量不设 = 冻结副本原样）：
  OPUS_ARM_REFUSE_UNMAPPED=1  较新可读根那次发布一条记录都读不出、账读不出、映射也现算不成时，照「记录不齐」那一支办（重读、凭据、仍不成拒 C_MAPPING）。
  OPUS_ARM_REFUSE_UNMAPPED=2  同上，只在系统配置里取过号的实例代号不比那条根新时照办。
  OPUS_ARM_F2_NO_SKIP=1       F2 退到的那一版与所选根之间有一次带末条标志的记录读得出、根读不出（且不被所选根实例表抛弃）的发布时不退。
  OPUS_ARM_F2_NO_SKIP=2       同上，那次发布的根读得出也算。"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])


def replace_once(path, old, new):
    p = root / path
    s = p.read_text()
    if s.count(old) != 1:
        print(f"旧串在 {path} 里命中 {s.count(old)} 次，不是 1 次", file=sys.stderr)
        sys.exit(2)
    p.write_text(s.replace(old, new))


replace_once("Cargo.toml", '    "crates/singlefs-reference-filesystem",\n]', '    "crates/singlefs-reference-filesystem",\n    "crates/opus-probe",\n]')

replace_once(
    "crates/singlefs-core/src/mount.rs",
    """                        ) {
                            Some(computed) => computed,
                            None => continue,
                        }
                    } else if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(""",
    """                        ) {
                            Some(computed) => computed,
                            // 攻方臂（只在攻方副本）。
                            None if opus_arm_refuses_an_unmapped_newer_root(devices, root) => {
                                placements_of_an_unapplied_publish_whose_account_is_unreadable(
                                    devices,
                                    unit_area_start,
                                    root,
                                    current_records,
                                    before_the_one_reread,
                                )?
                            }
                            None => continue,
                        }
                    } else if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(""",
)
replace_once(
    "crates/singlefs-core/src/mount.rs",
    "\n/// C 并上的较新可读根的账（`C_NEWER_ROOTS`",
    '''
/// 攻方臂（只在攻方副本）：OPUS_ARM_REFUSE_UNMAPPED=1 一律照记录不齐那一支办；=2 只在系统配置里取过号的实例代号不比那条根新时照办。
fn opus_arm_refuses_an_unmapped_newer_root<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
) -> bool {
    match std::env::var("OPUS_ARM_REFUSE_UNMAPPED").as_deref() {
        Err(_) => false,
        Ok("2") => crate::recovery::choose_system_configuration(devices)
            .map_or(true, |configuration| configuration.quantities.journal_instance <= root.instance),
        Ok(_) => true,
    }
}

/// C 并上的较新可读根的账（`C_NEWER_ROOTS`''',
)

replace_once(
    "crates/singlefs-core/src/recovery.rs",
    """    readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| {
        let order = order_of_the_root_choice(root);
        let kept_by_the_instance_table_of_the_chosen_root = match &instance_table_of_the_chosen_root
        {
            Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
            None => order == witnessed_publish,
        };
        witnessed_publish <= order
            && order < chosen
            && kept_by_the_instance_table_of_the_chosen_root
    })
    .max_by_key(order_of_the_root_choice)
}""",
    """    let all_readable_roots = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let candidate = all_readable_roots
    .iter()
    .copied()
    .filter(|root| {
        let order = order_of_the_root_choice(root);
        let kept_by_the_instance_table_of_the_chosen_root = match &instance_table_of_the_chosen_root
        {
            Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
            None => order == witnessed_publish,
        };
        witnessed_publish <= order
            && order < chosen
            && kept_by_the_instance_table_of_the_chosen_root
    })
    .max_by_key(order_of_the_root_choice)?;
    // 攻方臂（只在攻方副本）：OPUS_ARM_F2_NO_SKIP=1 / 2，见 patch_arms.py 文件头。
    if let Ok(arm) = std::env::var("OPUS_ARM_F2_NO_SKIP") {
        let candidate_order = order_of_the_root_choice(&candidate);
        let skips_a_publish_whose_last_record_reads = records.values().any(|record| {
            let order = (record.checkpoint_txg, record.instance);
            let abandoned = instance_table_of_the_chosen_root.as_ref().is_some_and(|table| {
                table.rows.iter().any(|row| row.instance == record.instance && record.checkpoint_txg > row.selected_root_txg)
            });
            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
                && candidate_order < order
                && order < chosen
                && !abandoned
                && (arm == "2" || !all_readable_roots.iter().any(|root| order_of_the_root_choice(root) == order))
        });
        if skips_a_publish_whose_last_record_reads {
            return None;
        }
    }
    Some(candidate)
}""",
)
print("patched")
