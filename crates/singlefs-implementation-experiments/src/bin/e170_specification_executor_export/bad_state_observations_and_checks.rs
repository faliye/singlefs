//! 第三段：坏行过实现的判定、坏状态文件名与回比条款。

use super::*;

/// 一行坏状态上实现那一侧看到的：底态与补丁之后两个镜像摘要（同第一段的摘要算法）、实现判定（判定里 panic 的照打）。
pub(super) struct BadObservation {
    pub(super) base_digest: String,
    pub(super) patched_digest: String,
    pub(super) verdict: Result<ImplementationVerdict, String>,
}

// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn observe_bad_row(stream: &PreparedStream, row: &BadRow) -> BadObservation {
    let base_image = CrashImage {
        base: &stream.base,
        writes: &stream.writes,
        persisted: row.base.persisted.clone(),
    };
    let base_digest = sha256_text(&crash_image_digest(&base_image));
    let mut writes = stream.writes.clone();
    writes.extend(row.patches.iter().map(|patch| RetainedWrite {
        device: patch.device,
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(patch.offset),
        contents: WrittenContents::Bytes(patch.bytes.clone()),
    }));
    let mut persisted = row.base.persisted.clone();
    persisted.resize(writes.len(), true);
    let image = CrashImage {
        base: &stream.base,
        writes: &writes,
        persisted,
    };
    let patched_digest = sha256_text(&crash_image_digest(&image));
    let verdict = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        judge_crash_image(&image, stream.judged_root_index, &stream.versions)
    }))
    .map(|judgement| ImplementationVerdict::of(&judgement))
    .map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_string())
            })
            .unwrap_or_else(|| "non_text_panic".to_string())
    });
    BadObservation {
        base_digest,
        patched_digest,
        verdict,
    }
}

pub(super) const BAD_STATES_FILE: &str = "bad-states.tsv";
pub(super) const BAD_PATCHES_FILE: &str = "bad-patches.bin";
pub(super) const BAD_EXPECTED_FILE: &str = "bad-expected.tsv";
pub(super) const BAD_IMPLEMENTATION_VERDICTS_FILE: &str = "bad-impl-verdicts.tsv";
pub(super) const BAD_SEED_FILE: &str = "bad-seed.tsv";
pub(super) const BAD_FILES: [&str; 5] = [
    BAD_STATES_FILE,
    BAD_PATCHES_FILE,
    BAD_EXPECTED_FILE,
    BAD_IMPLEMENTATION_VERDICTS_FILE,
    BAD_SEED_FILE,
];

/// 重驱动的流与导出目录里那份逐字节相同：manifest 记的四个文件的 sha256 与重驱动现算的比（S1 同形，记进 `bad_s1`）。
pub(super) fn dump_agreement_checks(stream: &PreparedStream, directory: &Path) -> Vec<StopCheck> {
    let manifest: BTreeMap<String, String> = read_text_lines(directory, MANIFEST_FILE)
        .iter()
        .filter_map(|line| line.split_once('\t'))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    let (operations_text, payload) = operations_file_text_and_payload(stream);
    let label = stream.name.directory_name();
    [
        (BASE_SECTORS_FILE, base_sectors_file_bytes(stream)),
        (OPERATIONS_FILE, operations_text.into_bytes()),
        (OPERATIONS_PAYLOAD_FILE, payload),
        (CALLS_FILE, calls_file_text(stream).into_bytes()),
    ]
    .into_iter()
    .map(|(file_name, contents)| {
        StopCheck::new(
            &format!("{label}_redriven_{file_name}_matches_the_dump"),
            manifest
                .get(&format!("sha256:{file_name}"))
                .cloned()
                .unwrap_or_default(),
            sha256_hexadecimal(&contents),
        )
    })
    .collect()
}

/// 本地常量与 core 的回比（`bad_s1`）：点名项区起点、新根段起点、根记录树 ID 水位第一个字节、根 n 所在槽与区域公式给的槽。
pub(super) fn bad_local_constant_checks(
    facts: &BadStreamFacts<'_>,
    full: &BaseState,
) -> Vec<StopCheck> {
    let label = facts.stream.name.directory_name();
    let newest = facts.newest_version(&full.persisted);
    let mut raised = newest.root;
    raised.tree_identifier_watermark += 1;
    let width =
        usize::try_from(facts.stream.writes[newest.root_write].length_in_bytes()).expect("根槽宽");
    let original_slot = newest.root.to_slot(width);
    let raised_slot = raised.to_slot(width);
    let first_differing_byte = original_slot
        .iter()
        .zip(&raised_slot)
        .position(|(original, raised_byte)| original != raised_byte);
    let configuration = &facts.chosen_system_configurations(&full.persisted)[0].parsed;
    let target = target_for_publish(
        newest.root.checkpoint_txg,
        configuration.immutable.sizes.root_ring_slots_per_region,
    );
    let root_write = &facts.stream.writes[newest.root_write];
    vec![
        StopCheck::new(
            &format!("{label}_record_named_item_area_start"),
            BAD_RECORD_NAMED_ITEM_AREA_START,
            usize::try_from(singlefs_format::JOURNAL_HEADER_BYTES)
                .expect("记录头 496 字节装得进 usize"),
        ),
        StopCheck::new(
            &format!("{label}_record_new_root_segment_start"),
            BAD_RECORD_NEW_ROOT_SEGMENT_START,
            JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET + 4 + 4 + 4,
        ),
        StopCheck::new(
            &format!("{label}_root_tree_identifier_watermark_first_byte"),
            Some(BAD_ROOT_TREE_IDENTIFIER_WATERMARK_FIRST_BYTE),
            first_differing_byte,
        ),
        StopCheck::new(
            &format!("{label}_root_n_reencodes_to_the_written_bytes"),
            true,
            root_write.bytes() == Some(original_slot.as_slice()),
        ),
        StopCheck::new(
            &format!("{label}_root_n_sits_where_the_region_formula_puts_it"),
            (root_write.device, root_write.offset.0),
            (
                configuration.immutable.region_devices
                    [usize::try_from(target.region).expect("区域号")],
                slot_offset(
                    target,
                    configuration.immutable.sizes.fixed_structure_slot_spacing,
                )
                .0,
            ),
        ),
    ]
}

/// 底态的落地向量文本：持久的写记 `2`、没持久的记 `0`（与 `states.tsv` 同一种写法）。
fn landing_vector_text(persisted: &[bool]) -> String {
    persisted
        .iter()
        .map(|is_persisted| if *is_persisted { '2' } else { '0' })
        .collect()
}

/// S5b（`research/prompts/e170-r1-prereg.md` 第十一节）：P10 那一行的底态落地向量与底态镜像摘要，与同一条流 P9-Mk+ 那一行的相同；
/// P10 行有没有，与 P9-Mk+ 行有没有一致（两行都由 Mk⁺ 给得出才造）。底态取错（例如取了 Mk）时落地向量与摘要都会不同。
pub(super) fn lower_bound_row_base_checks(
    stream_label: &str,
    rows: &[BadRow],
    observations: &[BadObservation],
) -> Vec<StopCheck> {
    let position_of = |row_name: &str| rows.iter().position(|row| row.name == row_name);
    let lower_bound = position_of(LOWER_BOUND_ROW_NAME);
    let first_run_mk_plus_row_position = position_of(FIRST_RUN_MK_PLUS_ROW_NAME);
    let mut checks = vec![StopCheck::new(
        &format!("{stream_label}_p10_row_exists_exactly_when_p9_mk_plus_row_exists"),
        first_run_mk_plus_row_position.is_some(),
        lower_bound.is_some(),
    )];
    if let (Some(lower_bound), Some(first_run_mk_plus_row_position)) =
        (lower_bound, first_run_mk_plus_row_position)
    {
        checks.push(StopCheck::new(
            &format!("{stream_label}_p10_base_landing_vector_equals_p9_mk_plus"),
            landing_vector_text(&rows[first_run_mk_plus_row_position].base.persisted),
            landing_vector_text(&rows[lower_bound].base.persisted),
        ));
        checks.push(StopCheck::new(
            &format!("{stream_label}_p10_base_digest_equals_p9_mk_plus"),
            &observations[first_run_mk_plus_row_position].base_digest,
            &observations[lower_bound].base_digest,
        ));
    }
    checks
}
