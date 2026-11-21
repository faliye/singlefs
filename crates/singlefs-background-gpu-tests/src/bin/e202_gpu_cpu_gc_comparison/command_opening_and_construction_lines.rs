//! 子命令共用的开头与造池的结果行：产物、开跑检查、crates 版本、卡，造池与停机的行。

use super::*;

pub(super) fn opening_of_the_command(
    product: &mut ProductWriter,
    product_path: &Path,
    command_fields: &str,
    cards: CardsOfTheCommand,
) -> Result<OpeningOfTheCommand, i32> {
    let lines_already_in_the_product =
        result_lines_already_in_the_product(product_path).map_err(|reason| {
            eprintln!("{reason}");
            EXIT_RUN_CONDITION_NOT_MET
        })?;
    let session_number = 1 + u64::try_from(
        lines_already_in_the_product
            .iter()
            .filter(|(name, _fields)| name == "session_start")
            .count(),
    )
    .expect("次数");
    product.emit(
        "session_start",
        &format!("session={session_number} {command_fields}"),
    );
    let host_hardware = match host_hardware_of_the_session(
        session_number,
        std::fs::read_to_string("/proc/cpuinfo")
            .map_err(|error| format!("读 /proc/cpuinfo：{error}")),
    ) {
        HostHardwareOfTheSession::Read {
            host_hardware,
            fields,
        } => {
            product.emit("host_hardware", &fields);
            host_hardware
        }
        HostHardwareOfTheSession::NotReadable { lines } => {
            for (kind, fields) in &lines {
                product.emit(kind, fields);
            }
            return Err(EXIT_RUN_CONDITION_NOT_MET);
        }
    };
    let registered_heavy_threads = checks_before_the_run(product)?;
    let combined = emit_the_crates_version(product).map_err(|reason| {
        product.emit(
            "crates_version",
            &format!("readable=false reason={}", field_value(&reason)),
        );
        EXIT_RUN_CONDITION_NOT_MET
    })?;
    if the_crates_changed_since_the_earlier_commands(&lines_already_in_the_product, &combined) {
        product.emit(
            "stopped",
            "clause=crates_changed_between_the_commands_of_the_segment reason=产物里已有的crates版本与这一次的不同",
        );
        return Err(EXIT_STOPPED);
    }
    match cards {
        CardsOfTheCommand::TakenBeforeOpeningTheDevices => {
            if !the_task_server_hands_over_cards_to_this_process() {
                product.emit(
                    "stopped",
                    &format!(
                        "clause=no_cards_from_the_task_server reason={} way_out={}",
                        field_value(REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER),
                        field_value("经task-server/task submit交")
                    ),
                );
                return Err(EXIT_RUN_CONDITION_NOT_MET);
            }
        }
        CardsOfTheCommand::NotNeeded => emit_the_gpus(product, false, &[]),
    }
    Ok(OpeningOfTheCommand {
        lines_already_in_the_product,
        session_number,
        host_hardware,
        registered_heavy_threads,
    })
}

/// 打 `name=tier_device` 一行：这一档登记的每块盘字节与 mkfs 认的最小一块盘；mkfs 不认交回 `false`（第十一节作废第 6 条：那一档作废，交主 agent）。
pub(super) fn emit_whether_the_make_filesystem_accepts_the_tier(
    product: &mut ProductWriter,
    tier: UnitAreaTier,
) -> bool {
    let is_accepted = the_make_filesystem_accepts_the_device_bytes(tier.device_bytes());
    product.emit(
        "tier_device",
        &format!(
            "tier={} device_bytes={} smallest_device_bytes_the_make_filesystem_accepts={} accepted_by_the_make_filesystem={is_accepted}",
            tier.name(),
            tier.device_bytes(),
            smallest_device_bytes_the_make_filesystem_accepts()
        ),
    );
    if !is_accepted {
        product.emit(
            "tier_voided",
            &format!(
                "tier={} clause=class_two_anchor_the_make_filesystem_refuses_the_registered_device_bytes reason=登记的每块盘字节小于默认环长的四倍，交主agent",
                tier.name()
            ),
        );
    }
    is_accepted
}

/// 打起步镜像的 `name=construction` 一行与每阶段一行 `name=construction_phase`；几何没造成交回 `false`（登记第十二节修订第 14 条追加的停机条款第 7 条：几何核没过就停）。
pub(super) fn emit_the_construction(product: &mut ProductWriter, image: &StartingImage) -> bool {
    let construction = &image.construction;
    let geometry = &construction.geometry;
    let histogram: Vec<String> = geometry
        .live_slot_histogram
        .iter()
        .map(|(live, segments)| format!("{live}:{segments}"))
        .collect();
    product.emit(
        "construction",
        &format!(
            "tier={} device_bytes={} live_data_units={} live_data_units_found={} construction_seconds={:.3} publishes={} \
             floor_raises={} release_rounds={} spacer_units={} alignment_spacer_units={} every_spacer_slot_released={} \
             units_per_file={} live_data_files={} spacer_files={} inode_records_before_the_data_mount={} \
             inode_records_at_the_peak={} inode_leaf_containers_at_the_peak={} inode_records_one_level_tree_holds={} \
             unit_area_segments={} empty_segments={} mapped_units={} registered_fragment_segments={} written_segments={} \
             written_segments_with_half_live={} choice=forced_fallback forced_segment_start_slot={} forced_live_slots={} \
             forced_holds_only_data_units={} greedy_segment_start_slot={} greedy_live_slots={} greedy_holds_only_data_units={} \
             live_slot_histogram={} geometry_formed={} {}",
            image.tier.name(),
            image.tier.device_bytes(),
            image.tier.live_data_units(),
            construction.live_data_units_found,
            construction.construction_seconds,
            construction.publishes,
            construction.floor_raises,
            construction.release_rounds,
            construction.spacer_units,
            construction.alignment_spacer_units,
            construction.is_every_spacer_slot_released,
            LIVE_DATA_UNITS_PER_FILE,
            construction.live_data_files,
            construction.spacer_files,
            construction.inode_records_before_the_data_mount,
            construction.inode_records_at_the_peak,
            construction.inode_leaf_containers_at_the_peak,
            inode_records_one_level_tree_holds(),
            image.tier.unit_area_segments(),
            construction.empty_segments,
            construction.mapped_units,
            image.tier.fragment_segments(),
            geometry.written_segments,
            geometry.written_segments_with_half_live,
            geometry
                .forced
                .start_slot
                .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
            geometry.forced.live_slots,
            geometry.forced.holds_only_data_units,
            geometry
                .greedy
                .start_slot
                .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
            geometry.greedy.live_slots,
            geometry.greedy.holds_only_data_units,
            histogram.join(","),
            geometry.is_formed(),
            whole_file_construction_fields(construction)
        ),
    );
    for timing in &construction.phases {
        product.emit(
            "construction_phase",
            &format!("tier={} {}", image.tier.name(), timing.fields()),
        );
    }
    geometry.is_formed()
}

/// 几何核没过时打的 `name=stopped` 那一行的字段（登记第十二节修订第 14 条追加的停机条款第 7 条：造法只有一种，不改负载重造，几何没过就停）。
pub(super) const GEOMETRY_NOT_FORMED_STOP_FIELDS: &str =
    "clause=geometry_not_formed reason=几何核没过，停机条款第7条，交主agent";

/// 两种造池结果行（`name=construction`、`name=construction_timing`）都带的读数（登记第十二节修订第 14 条量 B）：造完那一刻本进程常驻内存的峰、
/// 每次数据写类调用写进的单元数（单元数:次数）、造数据挂载登记的发布数合计、碎片段活槽的极大连续片数（片数:段数）与活槽恰好是段前半的碎片段数。
pub(super) fn whole_file_construction_fields(construction: &ConstructionReport) -> String {
    let pairs = |counts: &BTreeMap<u64, u64>| -> String {
        counts
            .iter()
            .map(|(key, count)| format!("{key}:{count}"))
            .collect::<Vec<String>>()
            .join(",")
    };
    format!(
        "peak_resident_bytes_of_the_process={} data_writes_by_units={} registered_publishes_of_the_data_mount={} \
         fragment_segment_live_slot_runs={} fragment_segments_live_in_the_front_half={}",
        construction
            .peak_resident_bytes_of_the_process
            .map_or_else(|| "unread".to_string(), |bytes| bytes.to_string()),
        pairs(&construction.data_writes_by_units),
        construction.registered_publishes_of_the_data_mount.total(),
        pairs(&construction.fragment_live_slot_layout.live_slot_runs_histogram),
        construction
            .fragment_live_slot_layout
            .segments_live_in_exactly_the_front_half,
    )
}

/// 造池停下时要打的结果行（种类，字段），按打的次序（登记第十二节修订第 14 条追加的停机条款第 7、8 条；停机时产物照打、退 3）：
/// 被拒的先打 `name=construction_refused`（字段 tier、phase、step、publishes、cluster_segments、unit_area_segments、`judged_short_by`、
/// `most_a_floor_raise_could_make_available`，另带 P 与拒绝的成员名；不是推满也放不下那一形的，后两样写 `none`），两种都再打一行 `name=stopped`。
pub(super) fn construction_stop_lines(
    tier: UnitAreaTier,
    live_data_units: u64,
    stopped: &ConstructionStopped,
) -> Vec<(&'static str, String)> {
    let moment_fields = |moment: &MomentOfTheConstruction| {
        format!(
            "tier={} live_data_units={live_data_units} phase={} step={} publishes={} cluster_segments={} unit_area_segments={}",
            tier.name(),
            moment.phase.name(),
            moment.step,
            moment.publishes,
            moment.cluster_segments.opened_by_this_mount,
            moment.cluster_segments.unit_area_segments,
        )
    };
    match stopped {
        ConstructionStopped::PublishRefused { moment, refusal } => {
            let joined_or_none = |values: Vec<String>| {
                if values.is_empty() {
                    "none".to_string()
                } else {
                    values.join(",")
                }
            };
            vec![
                (
                    "construction_refused",
                    format!(
                        "{} refusal={} judged_short_by={} most_a_floor_raise_could_make_available={}",
                        moment_fields(moment),
                        refusal.name_of_the_refusal,
                        joined_or_none(refusal.judged_short_by.clone()),
                        joined_or_none(
                            refusal
                                .most_a_floor_raise_could_make_available
                                .iter()
                                .map(i128::to_string)
                                .collect()
                        ),
                    ),
                ),
                (
                    "stopped",
                    format!(
                        "clause=construction_refused {} reason=造池里一次发布被核心层拒，停机条款第7条，交主agent",
                        moment_fields(moment)
                    ),
                ),
            ]
        }
        ConstructionStopped::PublishShapeDiffersFromTheRegistration {
            moment,
            disagreement,
        } => {
            let disagreement_fields = match disagreement {
                PublishShapeDisagreement::VersionsOfADataWrite {
                    inode,
                    versions_published,
                } => format!(
                    "disagreement=versions_of_a_data_write inode={inode} versions_published={versions_published}"
                ),
                PublishShapeDisagreement::UnitsOfADataWrite {
                    inode,
                    units_written,
                    units_of_the_file,
                } => format!(
                    "disagreement=units_of_a_data_write inode={inode} units_written={units_written} units_of_the_file={units_of_the_file}"
                ),
                PublishShapeDisagreement::PublishesOfThePhase {
                    publishes,
                    registered_publishes,
                } => format!(
                    "disagreement=publishes_of_the_phase phase_publishes={publishes} registered_publishes={registered_publishes}"
                ),
            };
            vec![(
                "stopped",
                format!(
                    "clause=construction_publish_shape_differs_from_the_registration {} {disagreement_fields} \
                     reason=造池发布的形状或阶段发布数与登记的算式不符，停机条款第8条，装置与crates两边都查",
                    moment_fields(moment)
                ),
            )]
        }
    }
}

/// 打造池停下时的结果行（[`construction_stop_lines`]）。
pub(super) fn emit_the_construction_stop(
    product: &mut ProductWriter,
    tier: UnitAreaTier,
    live_data_units: u64,
    stopped: &ConstructionStopped,
) {
    for (name, fields) in construction_stop_lines(tier, live_data_units, stopped) {
        product.emit(name, &fields);
    }
}

/// 造这一档的起步镜像；造池停下（停机条款第 7 条被拒、第 8 条形状对不上）时打停机的行、交回退出码 3。
/// 几何没造成照样交回镜像，由调用方照 [`emit_the_construction`] 的结局停。
pub(super) fn construct_the_starting_image_or_emit_the_stop(
    product: &mut ProductWriter,
    tier: UnitAreaTier,
) -> Result<StartingImage, i32> {
    construct_the_starting_image(tier).map_err(|stopped| {
        emit_the_construction_stop(product, tier, tier.live_data_units(), &stopped);
        EXIT_STOPPED
    })
}

#[cfg(test)]
mod tests;
