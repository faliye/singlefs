//! 第三段：P 行、N 行与盲测族的坏行怎么造。

use super::*;

/// 照改法造一行：补丁、期望的在 / 不在、期望落到哪里。P1 的读回与 P5 的前提另记进 `checks`（照打，不停）。
fn build_bad_row(
    facts: &BadStreamFacts<'_>,
    name: String,
    family: BadFamily,
    base: BaseState,
    shape: BadShape,
    checks: &mut Vec<StopCheck>,
) -> BadRow {
    let persisted = &base.persisted;
    let newest = facts.newest_version(persisted);
    let newest_txg = newest.root.checkpoint_txg.0;
    let newest_text = projection_text_of_root(&newest.root);
    let root_write = &facts.stream.writes[newest.root_write];
    let root_watermark_byte = root_write.offset.0
        + u64::try_from(BAD_ROOT_TREE_IDENTIFIER_WATERMARK_FIRST_BYTE).expect("122");
    let (patches, expected_verdict, expected_landing) = match shape {
        BadShape::RootRingBaseMisdirected => {
            let (patches, reads_back) = facts.root_ring_base_patches(persisted);
            checks.push(StopCheck::new(
                &format!("{name}_root_ring_base_reads_back"),
                true,
                reads_back,
            ));
            (patches, "not_in", ExpectedLanding::Nothing)
        }
        BadShape::PseudoRootAhead { txg_step } => (
            vec![facts.pseudo_root_patch(persisted, &newest, newest_txg + txg_step)],
            "not_in",
            ExpectedLanding::OutsideTheVersionTable,
        ),
        BadShape::PseudoContinuation { watermark_step } => (
            facts.pseudo_record_patches(
                persisted,
                &newest,
                &[pseudo_continuation(&newest, 1, watermark_step)],
            ),
            "not_in",
            ExpectedLanding::OutsideTheVersionTable,
        ),
        BadShape::PseudoContinuationOfTwo => (
            facts.pseudo_record_patches(persisted, &newest, &pseudo_pair(&newest, true)),
            "not_in",
            ExpectedLanding::OutsideTheVersionTable,
        ),
        BadShape::RootInstanceLowered => {
            let lowered = newest
                .root
                .instance
                .0
                .checked_sub(1)
                .expect("v_n 的实例代号 ≥ 1");
            let absent = !facts.version_identities().contains(&(lowered, newest_txg));
            checks.push(StopCheck::new(
                &format!("{name}_version_table_lacks_lowered_instance"),
                true,
                absent,
            ));
            (
                vec![facts.root_in_place_patch(&newest, |root| {
                    root.instance = InstanceGeneration(lowered)
                })],
                "not_in",
                ExpectedLanding::OutsideTheVersionTable,
            )
        }
        BadShape::NewestRootAndRecordUnreadable { expected_verdict } => {
            let area = if newest.record.named.is_empty() {
                BAD_RECORD_NEW_ROOT_SEGMENT_START
            } else {
                BAD_RECORD_NAMED_ITEM_AREA_START
            };
            let mut patches =
                vec![facts.flipped_byte_patch(persisted, root_write.device, root_watermark_byte)];
            for copy in facts.record_copies_of(&newest) {
                let write = &facts.stream.writes[copy];
                let offset = write.offset.0 + u64::try_from(area).expect("记录内偏移");
                patches.push(facts.flipped_byte_patch(persisted, write.device, offset));
            }
            (
                patches,
                expected_verdict,
                ExpectedLanding::Version(facts.previous_version_text(newest.publish_index)),
            )
        }
        BadShape::RootTxgLowered => (
            vec![facts.root_in_place_patch(&newest, |root| {
                root.checkpoint_txg = CheckpointTxg(newest_txg - BAD_LOWERED_TXG_STEP);
            })],
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::PseudoRecordAfterAGap => (
            facts.pseudo_record_patches(
                persisted,
                &newest,
                &[pseudo_continuation(&newest, 2, BAD_WATERMARK_STEP)],
            ),
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::PseudoPairWithoutLastFlag => (
            facts.pseudo_record_patches(persisted, &newest, &pseudo_pair(&newest, false)),
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::NewestRootUnverified => (
            vec![facts.flipped_byte_patch(persisted, root_write.device, root_watermark_byte)],
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::NamedUnitCopyCorrupted { publish_index } => {
            let publish = &facts.publishes[publish_index];
            let writes = &facts.stream.writes;
            let first_unit = publish.units.iter().min().copied().expect("p 有单元写");
            let copy = publish
                .units
                .iter()
                .copied()
                .filter(|unit| writes[*unit].bytes() == writes[first_unit].bytes())
                .min_by_key(|unit| (writes[*unit].device, *unit))
                .expect("第一次单元写自己就在里面");
            let offset = writes[copy].offset.0 + u64::try_from(BAD_UNIT_FLIP_OFFSET).expect("200");
            (
                vec![facts.flipped_byte_patch(persisted, writes[copy].device, offset)],
                "in",
                ExpectedLanding::Version(facts.previous_version_text(publish_index)),
            )
        }
    };
    BadRow {
        name,
        family,
        base,
        patches,
        expected_verdict,
        expected_landing,
    }
}

/// 一条流的坏状态表：P 行（F、Mk 各 6 行，Mk⁺ 上 P9）、N 行（F 上 N1–N4，Mk⁻ 上 N5）、盲测族 20 行。交回 (底态, 行)。
/// 盲测族每行依次抽：底态（F 或任一个发过根槽写的调用边界，均匀）、形态（P2 型、P3 型、P5 型，均匀）、形态的参数（P2 型 j ∈ 1..7，
/// P3 型 d ∈ [1000, 1 000 000]）；两条流共用一串 splitmix64，流一的 20 行先抽。
/// 第 1 次重跑的表（`BadTableVersion::SecondRun`）与第一次的表只差两处：P8 的期望判定，和盲测族之后多一行 P10（Mk⁺ 底态上同 P8 的改法、
/// 期望「不在」；这条流没有 Mk⁺ 就没有这一行）。P10 排在最后、不抽随机数，所以别的行的行号、补丁与盲测族抽样都不变（第十一节 S5）。
pub(super) fn bad_rows_of_stream(
    facts: &BadStreamFacts<'_>,
    generator: &mut SplitMix64,
    table_version: BadTableVersion,
    checks: &mut Vec<StopCheck>,
) -> (Vec<BaseState>, Vec<BadRow>) {
    let full = facts.all_persisted();
    let middle = facts.mk();
    let middle_plus = facts.mk_plus();
    let (middle_minus, named_publish) = facts.mk_minus();
    let mut rows = Vec::new();
    for base in [&full, &middle] {
        for (label, shape) in [
            ("P1", BadShape::RootRingBaseMisdirected),
            (
                "P2",
                BadShape::PseudoRootAhead {
                    txg_step: BAD_PSEUDO_ROOT_TXG_STEP,
                },
            ),
            (
                "P3",
                BadShape::PseudoContinuation {
                    watermark_step: BAD_WATERMARK_STEP,
                },
            ),
            ("P4", BadShape::PseudoContinuationOfTwo),
            ("P5", BadShape::RootInstanceLowered),
            (
                "P8",
                BadShape::NewestRootAndRecordUnreadable {
                    expected_verdict: table_version.previous_version_landing_expected_verdict(),
                },
            ),
        ] {
            let name = format!("{label}-{}", base.label);
            rows.push(build_bad_row(
                facts,
                name,
                BadFamily::Positive,
                base.clone(),
                shape,
                checks,
            ));
        }
    }
    if let Some(plus) = &middle_plus {
        let shape = BadShape::PseudoContinuation {
            watermark_step: BAD_WATERMARK_STEP,
        };
        rows.push(build_bad_row(
            facts,
            "P9-Mk+".to_string(),
            BadFamily::Positive,
            plus.clone(),
            shape,
            checks,
        ));
    }
    for (label, shape) in [
        ("N1", BadShape::RootTxgLowered),
        ("N2", BadShape::PseudoRecordAfterAGap),
        ("N3", BadShape::PseudoPairWithoutLastFlag),
        ("N4", BadShape::NewestRootUnverified),
    ] {
        rows.push(build_bad_row(
            facts,
            format!("{label}-F"),
            BadFamily::Negative,
            full.clone(),
            shape,
            checks,
        ));
    }
    let named_unit_shape = BadShape::NamedUnitCopyCorrupted {
        publish_index: named_publish,
    };
    rows.push(build_bad_row(
        facts,
        "N5-Mk-".to_string(),
        BadFamily::Negative,
        middle_minus.clone(),
        named_unit_shape,
        checks,
    ));
    let base_choices = u64::try_from(facts.root_issuing_calls.len() + 1).expect("调用数装得进 u64");
    for row_number in 1..=BLIND_ROWS_PER_STREAM {
        let base_choice = generator.below(base_choices);
        let base = match base_choice.checked_sub(1) {
            None => full.clone(),
            Some(call_position) => {
                let call_index =
                    facts.root_issuing_calls[usize::try_from(call_position).expect("调用下标")];
                facts.call_boundary(call_index, format!("Mk@{call_index}"))
            }
        };
        let (shape_label, shape) = match generator.below(3) {
            0 => {
                let step = 1 + generator.below(BLIND_PSEUDO_ROOT_STEPS);
                let txg_step = BAD_PSEUDO_ROOT_TXG_STEP * step;
                (
                    format!("P2:j={step}"),
                    BadShape::PseudoRootAhead { txg_step },
                )
            }
            1 => {
                let watermark_step = BLIND_LOWEST_WATERMARK_STEP
                    + generator
                        .below(BLIND_HIGHEST_WATERMARK_STEP - BLIND_LOWEST_WATERMARK_STEP + 1);
                (
                    format!("P3:d={watermark_step}"),
                    BadShape::PseudoContinuation { watermark_step },
                )
            }
            _ => ("P5".to_string(), BadShape::RootInstanceLowered),
        };
        let name = format!("B{row_number:02}:{shape_label}:{}", base.label);
        rows.push(build_bad_row(
            facts,
            name,
            BadFamily::Blind,
            base,
            shape,
            checks,
        ));
    }
    let lower_bound_base = match table_version {
        BadTableVersion::FirstRun => None,
        BadTableVersion::SecondRun => middle_plus.clone(),
    };
    if let Some(lower_bound_base) = lower_bound_base {
        rows.push(build_bad_row(
            facts,
            LOWER_BOUND_ROW_NAME.to_string(),
            BadFamily::AcknowledgementLowerBound,
            lower_bound_base,
            BadShape::NewestRootAndRecordUnreadable {
                expected_verdict: "not_in",
            },
            checks,
        ));
    }
    let mut bases = vec![full, middle, middle_minus];
    bases.extend(middle_plus);
    (bases, rows)
}

/// 底态的落地向量（只有 `2` 与 `0`）在崩溃状态序号里找同一个：只找每段序号区间的两端与 0、T − 1（调用边界落在段的边界上时，同一个持久集合
/// 就是前一段区间的末端或后一段区间的起点；登记 5.5「找不到照实报，仍用它」）。交回 (找到的序号, 找过的候选数)。
pub(super) fn crash_state_ordinal_with_the_same_landing(
    recorded_write_count: usize,
    tables: &CrashStateEnumerationTables,
    flow_facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    persisted: &[bool],
) -> (Option<u64>, usize) {
    let wanted: String = persisted
        .iter()
        .map(|is_persisted| if *is_persisted { '2' } else { '0' })
        .collect();
    let mut candidates: BTreeSet<u64> = BTreeSet::from([0, tables.state_count - 1]);
    for state_range in &tables.state_ranges_by_segment {
        if !state_range.is_empty() {
            candidates.insert(state_range.start);
            candidates.insert(state_range.end - 1);
        }
    }
    let found = candidates.iter().copied().find(|ordinal| {
        landing_vector_of(
            &flow_facts.persisted_writes_of_state(*ordinal),
            recorded_write_count,
            torn_image_index_by_write,
        ) == wanted
    });
    (found, candidates.len())
}
