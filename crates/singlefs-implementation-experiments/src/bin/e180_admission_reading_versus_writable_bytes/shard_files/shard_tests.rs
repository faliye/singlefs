use super::*;

/// 一个几何格里一族的假结果：每个字段都取非零、随（几何、族）变的值，字符串里带制表符、换行与反斜杠（片文件要转义的三样）。
fn synthetic_outcome(geometry: Geometry, family: &'static str) -> FamilyOutcome {
    let geometry_index = ALL_GEOMETRIES
        .iter()
        .position(|known| known.name == geometry.name)
        .expect("几何在全部几何里");
    let family_index = families_of_the_geometry(geometry, None)
        .iter()
        .position(|known| *known == family)
        .expect("族在这个几何格的族里");
    let seed = u64::try_from(geometry_index * 10 + family_index).expect("种子") + 1;
    let state_name = format!("{}/{family}/{seed}", geometry.name);
    let mut tally = FamilyTally::new(2);
    tally.states = 100 + seed;
    tally.exact_states = 60 + seed;
    tally.censored_states = 30;
    tally.unreadable_states = 10;
    if family != LONG_MOUNT_FAMILY {
        for (arm_index, arm) in all_arms().into_iter().enumerate() {
            let arm_number = u64::try_from(arm_index).expect("臂号");
            tally.arms.insert(
                arm,
                ArmTally {
                    over_states: seed + arm_number,
                    not_over_states: 2 * seed + arm_number,
                    not_over_below_lower_bound_states: arm_number,
                    undetermined_states: seed % 3,
                    maximum_over_bytes: 32634 * (seed + arm_number),
                    maximum_over_at: state_name.clone(),
                    maximum_under_bytes: 16384 * seed,
                    maximum_under_at: format!("{state_name}\twith a tab"),
                    sum_under_bytes: u128::from(seed) << 70,
                    under_states: seed,
                },
            );
        }
    }
    for (device_index, residual) in tally.residuals.iter_mut().enumerate() {
        let device_number = i128::try_from(device_index).expect("盘号");
        *residual = ResidualTally {
            states: 100 + seed,
            rewritten_check_residual_nonzero_states: seed,
            original_formula_residual_nonzero_states: seed + 1,
            maximum_absolute_rewritten_check_residual: -7 - device_number,
            maximum_absolute_original_formula_residual: i128::from(seed) << 80,
            states_with_overlap: 3,
            states_with_held_ledger: 0,
            control_without_overlap_term_stayed_zero: u64::try_from(device_index).expect("盘号"),
            first_state_with_a_nonzero_rewritten_check_residual: format!(
                "{state_name}\nwith a line break"
            ),
            first_state_with_a_nonzero_original_formula_residual: String::new(),
        };
    }
    for (trajectory_index, trajectories) in [
        &mut tally.closed_minus_ring_window,
        &mut tally.closed_alone,
        &mut tally.closed_minus_all_common_commitments,
    ]
    .into_iter()
    .enumerate()
    {
        let offset = i128::try_from(trajectory_index).expect("轨迹号");
        for (device_index, trajectory) in trajectories.iter_mut().enumerate() {
            if device_index == 1 && trajectory_index == 2 {
                continue; // 留一条一个状态都没记的轨迹：峰值是 None。
            }
            for value in [52, -3 - offset, i128::from(seed), 0] {
                trajectory.record(value);
            }
        }
    }
    tally.control_failures = vec![format!("{state_name}:back\\slash")];
    tally.shortcut_pairs_checked = seed;
    tally.shortcut_differences = Vec::new();
    tally.records_disagree_with_the_disk_states = 1;
    tally.decomposition_failure_states = 2;
    tally.not_whole_slot_available_states = 3;
    tally.lag_unattributed_states = 4;
    tally.non_monotone_states = 5;
    tally.probes_run = 1000 * seed;
    tally.stop_clause_held_ledger_mismatch_states = 6;
    tally.unmount_pushes_checked = 7;
    tally.unmount_pushes_differing = vec!["none".to_string(), String::new()];
    tally.panics = Vec::new();
    FamilyOutcome {
        family,
        lines: vec![
            ProductLine::new("state")
                .field("geometry", geometry.name)
                .field("family", family)
                .field("index", 0)
                .finish(),
            ProductLine::new("state")
                .field("geometry", geometry.name)
                .field("family", family)
                .field("index", seed)
                .field("reason", "a\\b\tc")
                .finish(),
        ],
        tally,
        anchors: FirstFileAnchors {
            open_segment_start_slot: Some(50_240),
            closed_free_slots_in_the_open_segment_on_device_zero: Some(52 + seed % 2),
            first_data_unit_slot: None,
        },
        seconds: 3600 + seed,
        unit_area_slots_per_device: vec![
            geometry.unit_area_slots_of_device(0),
            geometry.unit_area_slots_of_device(1),
        ],
        trajectory_names: TRAJECTORY_NAMES,
    }
}

const LOCAL_CONSTANTS_CHECKED: usize = 15;

/// 一次跑全部：每个几何格、每族都是 [`synthetic_outcome`]。
fn every_cell_at_once() -> Vec<GeometryCellOutcome> {
    ALL_GEOMETRIES
        .iter()
        .map(|geometry| GeometryCellOutcome {
            geometry: *geometry,
            outcomes: families_of_the_geometry(*geometry, None)
                .into_iter()
                .map(|family| synthetic_outcome(*geometry, family))
                .collect(),
        })
        .collect()
}

/// 分片跑：每一片的片文件，片头是这一片跑的那一次的总线程数与探测线程数。
fn shard_texts_of_every_cell(total_threads: usize) -> BTreeMap<String, String> {
    let mut texts = BTreeMap::new();
    for geometry in ALL_GEOMETRIES {
        for family in families_of_the_geometry(geometry, None) {
            let header = ShardHeader {
                geometry_name: geometry.name.to_string(),
                family: family.to_string(),
                total_threads,
                probe_threads: probe_threads_of_the_geometry(geometry, total_threads),
                local_constants_checked: LOCAL_CONSTANTS_CHECKED,
            };
            texts.insert(
                shard_file_name(geometry, family),
                shard_text(&header, &synthetic_outcome(geometry, family)),
            );
        }
    }
    texts
}

fn lines_named(lines: &[String], name: &str) -> usize {
    let prefix = format!("E name={name} ");
    lines
        .iter()
        .filter(|line| line.starts_with(&prefix))
        .count()
}

#[test]
fn merging_the_shards_of_every_cell_gives_the_product_of_running_every_cell_at_once_byte_for_byte()
{
    let at_once = reading_product_lines(LOCAL_CONSTANTS_CHECKED, &every_cell_at_once());
    let merged = merge_shard_texts(&shard_texts_of_every_cell(31)).expect("29 片齐了，合并该成");
    assert_eq!(
        merged.join("\n").into_bytes(),
        at_once.join("\n").into_bytes()
    );
    // 钉绝对值：两条路一起错（比如都少了一格）时这几条红。
    assert_eq!(
        at_once.last().expect("产物非空"),
        &format!("E name=done emitted={}", at_once.len())
    );
    assert_eq!(lines_named(&at_once, "geometry"), 5);
    assert_eq!(lines_named(&at_once, "verdict"), 5);
    assert_eq!(lines_named(&at_once, "family_summary"), 29);
    assert_eq!(lines_named(&at_once, "state"), 2 * 29);
    assert_eq!(lines_named(&at_once, "local_constants_checked"), 1);
    assert!(at_once.contains(
        &"E name=state geometry=unequal_devices_geometry family=abandoned_root_and_net_release index=15 reason=a\\b\tc"
            .to_string()
    ));
}

#[test]
fn shard_text_parses_back_to_the_same_header_and_family_outcome() {
    let geometry = geometry_named("default_geometry_with_four_slots_per_region");
    let header = ShardHeader {
        geometry_name: geometry.name.to_string(),
        family: "abandoned_root_and_net_release".to_string(),
        total_threads: 31,
        probe_threads: 5,
        local_constants_checked: LOCAL_CONSTANTS_CHECKED,
    };
    let text = shard_text(
        &header,
        &synthetic_outcome(geometry, "abandoned_root_and_net_release"),
    );
    assert_eq!(
        text.lines().filter(|line| line.contains('\t')).count(),
        text.lines().count()
    );
    assert_eq!(
        parse_shard(&text).expect("刚写的片读得回"),
        (
            header,
            synthetic_outcome(geometry, "abandoned_root_and_net_release")
        )
    );
}

#[test]
fn merge_refuses_a_missing_shard() {
    let mut texts = shard_texts_of_every_cell(31);
    texts.remove("narrow_devices_geometry-many_mounts.shard");
    let refusal = merge_shard_texts(&texts).expect_err("少一片不合并");
    assert!(
        refusal.contains("missing_shard=narrow_devices_geometry-many_mounts.shard"),
        "{refusal}"
    );
}

#[test]
fn merge_refuses_a_shard_whose_done_line_is_missing() {
    let mut texts = shard_texts_of_every_cell(31);
    let text = texts
        .get_mut("default_geometry-overwrite.shard")
        .expect("片在");
    let without_the_done_line: Vec<&str> = text
        .lines()
        .filter(|line| !line.starts_with("shard_done\t"))
        .collect();
    *text = format!("{}\n", without_the_done_line.join("\n"));
    let refusal = merge_shard_texts(&texts).expect_err("没有完成标记的片不合并");
    assert!(
        refusal.contains("shard=default_geometry-overwrite.shard last_line_is_not_the_done_marker"),
        "{refusal}"
    );
}

#[test]
fn merge_refuses_a_shard_cut_short_even_if_it_ends_with_a_done_line() {
    let mut texts = shard_texts_of_every_cell(31);
    let text = texts.get_mut("default_geometry-fill.shard").expect("片在");
    let lines: Vec<&str> = text.lines().collect();
    let mut kept: Vec<String> = lines[..lines.len() / 2]
        .iter()
        .map(|line| (*line).to_string())
        .collect();
    kept.push(lines[lines.len() - 1].to_string());
    *text = format!("{}\n", kept.join("\n"));
    let refusal = merge_shard_texts(&texts).expect_err("截短的片不合并");
    assert!(
        refusal.contains("shard=default_geometry-fill.shard last_line_is_not_the_done_marker"),
        "完成标记里的行数对不上就拒，不等读到一半才发现缺键：{refusal}"
    );
}

#[test]
fn merge_refuses_an_unexpected_shard_and_a_shard_holding_another_family() {
    let mut texts = shard_texts_of_every_cell(31);
    let extra = texts["default_geometry-fill.shard"].clone();
    texts.insert("default_geometry-hx.shard".to_string(), extra);
    let extra_shard_refusal = merge_shard_texts(&texts).expect_err("多出来的片不合并");
    assert!(
        extra_shard_refusal.contains("unexpected_shard=default_geometry-hx.shard"),
        "{extra_shard_refusal}"
    );
    let mut swapped = shard_texts_of_every_cell(31);
    let other_family = swapped["default_geometry-delete_then_write.shard"].clone();
    swapped.insert("default_geometry-overwrite.shard".to_string(), other_family);
    let swapped_shard_refusal =
        merge_shard_texts(&swapped).expect_err("片名与片里装的族对不上不合并");
    assert!(
        swapped_shard_refusal.contains(
            "shard=default_geometry-overwrite.shard holds_geometry=default_geometry holds_family=delete_then_write"
        ),
        "{swapped_shard_refusal}"
    );
}

#[test]
fn merge_refuses_shards_run_with_different_total_threads() {
    let mut texts = shard_texts_of_every_cell(31);
    let other_threads = shard_texts_of_every_cell(30);
    texts.insert(
        "default_geometry_with_sixteen_slots_per_region-delete_then_write.shard".to_string(),
        other_threads["default_geometry_with_sixteen_slots_per_region-delete_then_write.shard"]
            .clone(),
    );
    let refusal =
        merge_shard_texts(&texts).expect_err("总线程数不同的片不合并（探测点随线程数变）");
    assert!(
        refusal.contains(
            "shard=default_geometry_with_sixteen_slots_per_region-delete_then_write.shard total_threads_and_local_constants=(30, 15)"
        ),
        "{refusal}"
    );
}

#[test]
fn merge_refuses_a_shard_whose_probe_threads_are_not_those_of_its_geometry_cell() {
    let mut texts = shard_texts_of_every_cell(31);
    let geometry = geometry_named("narrow_devices_geometry");
    let header = ShardHeader {
        geometry_name: geometry.name.to_string(),
        family: "fill".to_string(),
        total_threads: 31,
        probe_threads: 6,
        local_constants_checked: LOCAL_CONSTANTS_CHECKED,
    };
    texts.insert(
        "narrow_devices_geometry-fill.shard".to_string(),
        shard_text(&header, &synthetic_outcome(geometry, "fill")),
    );
    let refusal = merge_shard_texts(&texts).expect_err("探测线程数不是这一格的不合并");
    assert!(
        refusal.contains("shard=narrow_devices_geometry-fill.shard probe_threads=6 expected=5"),
        "{refusal}"
    );
}

/// 片的清单：5 个几何 × 5 个带探测的族，加除 unequal_devices_geometry 之外 4 个几何的 long_mount，共 29 片，片名不重；
/// 一片的探测线程数与一次跑全部时那一格里的那一族相同（按这个几何格的全部族数分，不按这一次跑几族分）。
#[test]
fn the_shard_list_has_twenty_nine_shards_each_with_the_probe_threads_of_its_whole_geometry_cell() {
    let lines = shard_list_lines(31);
    assert_eq!(lines.len(), 29);
    let names: BTreeSet<&str> = lines
        .iter()
        .map(|line| line.split('\t').next().expect("片名"))
        .collect();
    assert_eq!(names.len(), 29);
    assert!(lines
        .contains(&"default_geometry-overwrite.shard\tdefault_geometry\toverwrite\t5".to_string()));
    assert!(lines.contains(&"unequal_devices_geometry-abandoned_root_and_net_release.shard\tunequal_devices_geometry\tabandoned_root_and_net_release\t6".to_string()));
    assert!(lines.contains(
        &"default_geometry_with_sixteen_slots_per_region-long_mount.shard\tdefault_geometry_with_sixteen_slots_per_region\tlong_mount\t1"
            .to_string()
    ));
    assert!(!names.contains("unequal_devices_geometry-long_mount.shard"));
    for geometry in ALL_GEOMETRIES {
        assert_eq!(
            probe_threads_of_the_geometry(geometry, 31),
            31 / families_of_the_geometry(geometry, None).len(),
            "{}",
            geometry.name
        );
    }
    assert_eq!(
        probe_threads_of_the_geometry(geometry_named("default_geometry"), 3),
        1
    );
}

#[test]
fn shard_value_with_backslashes_tabs_and_line_breaks_survives_escaping() {
    for text in ["", "none", "a\\b", "\t", "\n\r", "\\t", "末尾一个反斜杠\\"] {
        assert_eq!(
            unescape_shard_value(&escape_shard_value(text)).expect("转义过的读得回"),
            text
        );
        assert!(!escape_shard_value(text).contains(['\t', '\n', '\r']));
    }
    assert!(unescape_shard_value("\\x").is_err());
    assert!(unescape_shard_value("\\").is_err());
}
