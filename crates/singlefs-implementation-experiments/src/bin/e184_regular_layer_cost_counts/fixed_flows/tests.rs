use super::*;
use crate::helpers_shared_by_the_tests::scratch_directory;

#[test]
fn the_snapshot_creation_stream_is_recorded_with_the_pinned_numbers_and_every_check_and_control_passes(
) {
    let prepared = prepared_snapshot_creation_stream();
    let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[3]);
    assert_eq!(readings.len(), 4);
    assert!(
        readings.iter().all(|reading| reading.passed),
        "{readings:?}"
    );
    let dump = prepared.dump();
    let reading = read_the_flow(&dump, &prepared.base).expect("建快照流排得了计划");
    assert_eq!(
        (
            reading.row.writes,
            reading.row.segments,
            reading.row.longest_segment
        ),
        (64, 16, 24)
    );
    assert_eq!(
        (reading.row.two_state, reading.row.full),
        (Some(16_842_785), Some(16_842_815))
    );
    assert_eq!(reading.row.two_state_agrees_with_crates, Some(true));
    assert!(
        reading.count_disagreements.is_empty(),
        "{:?}",
        reading.count_disagreements
    );
    let full = reading.full.as_ref().expect("全量计划");
    let in_place_subsets = reading.in_place_subsets.as_ref().expect("甲二计划");
    assert_eq!(full.state_count, 16_842_815);
    assert!(in_place_subsets.state_count < full.state_count);
    assert_eq!(reading.row.blocks_full, Some(full.blocks.len()));
    let controls = controls_of_the_flow(&dump, &prepared.base, &reading).expect("对照造得出");
    assert_eq!(
        controls
            .iter()
            .map(|control| control.kind)
            .collect::<Vec<_>>(),
        vec![
            "dump_round_trip",
            "base_sector_changed",
            "name_not_in_key",
            "two_expansions"
        ]
    );
    assert!(
        controls.iter().all(|control| control.passed),
        "{controls:?}"
    );
    let direct = direct_base_control(&prepared, &dump).expect("直接的基镜像对得上");
    assert!(direct.passed, "{direct:?}");
}

#[test]
fn the_round_trip_control_compares_the_keys_only_when_both_plans_fit_under_its_limit() {
    let prepared = prepared_new_pool_stream();
    let dump = prepared.dump();
    let compared = direct_base_control_under(&prepared, &dump, 1 << 24).expect("对得上");
    assert!(
        compared.passed && compared.detail.ends_with("keys_equal=true"),
        "{compared:?}"
    );
    let base_only = direct_base_control_under(&prepared, &dump, 100).expect("对得上");
    assert!(
        base_only.passed && base_only.detail.ends_with("keys_equal=not_planned"),
        "{base_only:?}"
    );
    assert_eq!(base_only.kind, "dump_round_trip_direct_base");
}

#[test]
fn the_sixteen_unit_overwrite_chain_is_read_without_its_full_plan_and_with_its_in_place_subsets_plan(
) {
    let (flow, _overwrites) = prepared_overwrite_chain(16);
    let dump = flow.dump();
    let reading = read_the_flow(&dump, &flow.base).expect("键读得出，不因为块数而 Err");
    assert!(
        reading.row.longest_segment >= 40,
        "u = 16 的每次覆盖写有一个很长的单元段：{}",
        reading.row.longest_segment
    );
    assert_eq!(
        (reading.row.blocks_full, reading.full.is_none()),
        (None, true),
        "全量的块数超过上限（或态数装不进 u64），不排计划"
    );
    assert!(
        reading.in_place_subsets.is_some() && reading.row.blocks_in_place_subsets.is_some(),
        "甲二照排：长段不展开，块数很少"
    );
}

#[test]
fn the_new_pool_file_creation_stream_is_recorded_with_the_pinned_numbers() {
    let prepared = prepared_new_pool_stream();
    let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[0]);
    assert!(
        readings.iter().all(|reading| reading.passed),
        "{readings:?}"
    );
    let dump = prepared.dump();
    assert_eq!((dump.writes.len(), dump.segments.len()), (43, 12));
    assert_eq!(
        (
            dump.family.as_str(),
            dump.tier.as_str(),
            dump.versions_with_a_file.len()
        ),
        ("smoke", "baseline", 1)
    );
}

#[test]
fn the_multi_record_publish_stream_is_recorded_with_the_pinned_numbers() {
    let prepared = prepared_multi_record_stream();
    let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[2]);
    assert!(
        readings.iter().all(|reading| reading.passed),
        "{readings:?}"
    );
    assert_eq!(
        prepared.versions_with_a_file.len(),
        3,
        "A、B、C 三版带文件的版本"
    );
    assert_eq!(prepared.writes.len(), 117);
}

#[test]
fn the_fixed_script_stream_is_recorded_with_the_pinned_numbers() {
    let prepared = prepared_fixed_script_stream();
    let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[1]);
    assert!(
        readings.iter().all(|reading| reading.passed),
        "{readings:?}"
    );
    assert_eq!(prepared.segments.len(), 80);
    assert_eq!(
        prepared.versions_with_a_file.len(),
        17,
        "脚本里 versions 的条数：A、B（2），写行与暖机三个（5），C（6），回退 D（7），五次覆盖写（12），抬 F 两个（14），E（15），卸载两个（17）"
    );
}

#[test]
fn an_overwrite_chain_runs_until_a_thousand_writes_and_keeps_one_version_per_overwrite() {
    let (flow, overwrites) = prepared_overwrite_chain(16);
    assert!(
        flow.writes.len() >= WRITES_OF_A_THOUSAND_WRITE_STREAM,
        "写数 {} 不到 1000",
        flow.writes.len()
    );
    assert!(overwrites >= 1);
    assert_eq!(
        flow.versions_with_a_file.len(),
        overwrites + 1,
        "第一个文件一版，加每次覆盖写一版"
    );
    assert_eq!(flow.label, "overwrite_chain-16_units_per_overwrite");
    let capacity_of_one_data_unit = singlefs_core::unit::data_unit_payload_capacity();
    assert!(
        flow.versions_with_a_file[1].content.len() > capacity_of_one_data_unit,
        "u = 16 的每一版都占多个数据单元，一个单元装不下"
    );
    assert_eq!(
        flow.versions_with_a_file
            .last()
            .map(|version| version.content.clone()),
        Some(multi_record_steps::content_needing(16, overwrites)),
        "最后一版的内容就是第 K 次的 content_needing(16, K)"
    );
    assert_eq!(flow.ending, format!("NotAHistory:overwrites={overwrites}"));
    assert_eq!(
        Some(flow.judged_root_index),
        flow.writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
    );
    let dump = flow.dump();
    assert_eq!(FlowDump::from_bytes(&dump.to_bytes()), Ok(dump.clone()));
    assert_eq!(
        dump.base_pool(),
        flow.base,
        "转储里的基镜像叠回去得到建好的池"
    );
}

#[test]
fn the_fixed_flows_are_named_and_ordered_chains_first_then_the_four_smoke_flows() {
    let labels: Vec<String> = FixedFlowKind::every_fixed_flow_kind()
        .iter()
        .map(|kind| kind.label())
        .collect();
    assert_eq!(
        labels,
        vec![
            "overwrite_chain-01_units_per_overwrite",
            "overwrite_chain-04_units_per_overwrite",
            "overwrite_chain-16_units_per_overwrite",
            "smoke-new_pool_file_creation_stream",
            "smoke-overwrite_fixed_script_stream",
            "smoke-multi_record_publish_stream",
            "smoke-snapshot_creation_stream",
        ]
    );
    assert_eq!(OVERWRITE_CHAIN_UNITS_PER_OVERWRITE, [1, 4, 16]);
}

#[test]
fn the_record_fixed_chains_command_writes_seven_dumps_and_skips_them_the_second_time() {
    let out = scratch_directory("chains");
    let arguments = vec!["--out".to_string(), out.to_str().expect("路径").to_string()];
    assert_eq!(run_record_fixed_chains_command(&arguments), EXIT_CODE_OK);
    let dumps = files_with_suffix(&[out.to_path_buf()], DUMP_FILE_SUFFIX, None).expect("列得出");
    assert_eq!(dumps.len(), 7);
    for path in &dumps {
        let dump = FlowDump::read_from(path).expect("每份转储读得回");
        assert!(
            dump.judged_root_index.is_some() && !dump.writes.is_empty(),
            "{}",
            dump.label
        );
    }
    let chain = FlowDump::read_from(&out.join("overwrite_chain-04_units_per_overwrite.dump"))
        .expect("覆盖写链 units=4");
    assert!(
        chain.writes.len() >= WRITES_OF_A_THOUSAND_WRITE_STREAM
            && chain.ending.starts_with("NotAHistory:overwrites=")
    );
    assert_eq!(
        run_record_fixed_chains_command(&arguments),
        EXIT_CODE_OK,
        "已有写完的转储整份跳过"
    );
}
