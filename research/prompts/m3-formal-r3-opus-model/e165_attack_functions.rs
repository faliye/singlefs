// ===== m3-formal-r3 攻方腿加的量（只在副本里；E165 的判据一条不改，这里只多数几样）=====

fn attack_environment_number(name: &str) -> u32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// ATTACK_GEN：位 1 = 第 0 版是写行发布（分配记录树根非零），位 2 = 每一版的根带卸载记号。
fn attack_gen_mode() -> u32 {
    static MODE: OnceLock<u32> = OnceLock::new();
    *MODE.get_or_init(|| attack_environment_number("ATTACK_GEN"))
}

fn attack_skip_prelude() -> bool {
    attack_environment_number("ATTACK_SKIP_PRELUDE") == 1
}

thread_local! {
    /// C 域：这个状态是不是停在某一段（位 1 甲切法、位 2 乙切法）；别的域恒 0。
    static ATTACK_LEGAL_CRASH_STATE: Cell<u8> = const { Cell::new(0) };
}

/// S1 的「其余字段」：fsid、卸载记号、实例表指针、分配记录树根。
fn attack_other_fields(root: &RootRecord) -> ([u8; 16], UnmountMarker, NodePointer, NodePointer) {
    (
        root.filesystem_identifier,
        root.unmount_marker,
        root.instance_table,
        root.allocation_record_tree_root,
    )
}

#[allow(clippy::too_many_arguments, reason = "副本里的计数，参数照原判据现成的量摊开")]
fn attack_judge_a_legal_state(
    state: &LegalState<'_>,
    first: &ReplayOutcome,
    landing: Option<usize>,
    chosen_root: &RootRecord,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    truth: &LegalTruth,
    [j1_holds, j2_holds, j3_holds]: [bool; 3],
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let legal_bits = ATTACK_LEGAL_CRASH_STATE.with(Cell::get);
    let is_legal = legal_bits != 0;
    let is_legal_own = legal_bits & 1 != 0;
    let j4_holds = landing == Some(truth.farthest_version);
    let history = state.history;
    let unmount_shape = history.publishes.len() == history.chosen_instance_publish_count
        && history.publishes.len() <= 3
        && history.publishes.iter().all(|publish| publish.record_count == 1);
    if unmount_shape {
        tally.add(Counter::AtkUnmountShapeStates);
    }
    if is_legal {
        tally.add(Counter::AtkLegalStates);
        for (holds, counter) in [
            (j1_holds, Counter::AtkLegalJ1),
            (j2_holds, Counter::AtkLegalJ2),
            (j3_holds, Counter::AtkLegalJ3),
            (j4_holds, Counter::AtkLegalJ4),
        ] {
            if !holds {
                tally.add(counter);
            }
        }
        if truth.above_water_record_count > truth.in_flight_limit {
            tally.add(Counter::AtkLegalJ6PremiseFails);
        }
    }
    if is_legal_own {
        tally.add(Counter::AtkLegalOwnStates);
        for (holds, counter) in [
            (j1_holds, Counter::AtkLegalOwnJ1),
            (j2_holds, Counter::AtkLegalOwnJ2),
            (j3_holds, Counter::AtkLegalOwnJ3),
            (j4_holds, Counter::AtkLegalOwnJ4),
        ] {
            if !holds {
                tally.add(counter);
            }
        }
    }
    let mut other_fields_differ = false;
    if let ReplayOutcome::Returned(returned) = first {
        let (fsid, marker, instance_table, allocation) = attack_other_fields(&returned.root);
        let (chosen_fsid, chosen_marker, chosen_instance_table, chosen_allocation) =
            attack_other_fields(chosen_root);
        let marker_differs = marker != chosen_marker;
        let allocation_differs = allocation != chosen_allocation;
        let instance_table_differs = instance_table != chosen_instance_table;
        other_fields_differ =
            marker_differs || allocation_differs || instance_table_differs || fsid != chosen_fsid;
        if other_fields_differ {
            tally.add_violation(Counter::AtkS1OtherFieldsDiffer, describe_witness);
        }
        if marker_differs {
            tally.add_violation(Counter::AtkS1MarkerDiffers, describe_witness);
            if unmount_shape {
                tally.add_violation(Counter::AtkUnmountShapeMarkerDiffers, describe_witness);
            }
            if is_legal {
                tally.add_violation(Counter::AtkLegalS1MarkerDiffers, describe_witness);
            }
            if is_legal_own {
                tally.add_violation(Counter::AtkLegalOwnS1MarkerDiffers, describe_witness);
            }
        }
        if allocation_differs {
            tally.add_violation(Counter::AtkS1AllocDiffers, describe_witness);
        }
        if instance_table_differs {
            tally.add_violation(Counter::AtkS1InstanceTableDiffers, describe_witness);
        }
        if let Some(version) = landing.filter(|version| *version >= state.chosen_version) {
            let full_root_of_the_version = if version == state.chosen_version {
                *chosen_root
            } else {
                history.version_fields(version).to_root()
            };
            if returned.root != full_root_of_the_version {
                tally.add_violation(Counter::AtkFullRootMismatch, describe_witness);
                if is_legal {
                    tally.add(Counter::AtkLegalFullRootMismatch);
                }
            }
        }
        // txg 顶格的两种写法（只在所选根 txg ≥ u64 最大值 − 1 的 E1、E2 域判）。
        if chosen_root.checkpoint_txg.0 >= u64::MAX - 1 {
            let in_flight_premise = truth.above_water_record_count <= truth.in_flight_limit;
            let second =
                call_replay_journal(disk, &returned.root, records, state.verify_named_units);
            let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(second_returned) if second_returned.root == returned.root);
            if in_flight_premise
                && chosen_root.checkpoint_txg.0 < u64::MAX
                && returned.root.checkpoint_txg.0 < u64::MAX
            {
                tally.add(Counter::AtkJ6WritingAJudged);
                if !fixed_point_holds {
                    tally.add_violation(Counter::AtkJ6WritingAViolations, describe_witness);
                }
            }
            if in_flight_premise {
                tally.add(Counter::AtkJ6WritingBJudged);
                let equal_under_writing_b = fixed_point_holds
                    || (returned.root.checkpoint_txg.0 == u64::MAX
                        && second == ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor));
                if !equal_under_writing_b {
                    let describe_with_the_second_call =
                        || format!("{};second={}", describe_witness(), describe_outcome(&second));
                    tally.add_violation(
                        Counter::AtkJ6WritingBViolations,
                        &describe_with_the_second_call,
                    );
                }
            }
        }
    }
    let s1_holds = j1_holds && j2_holds && j3_holds && !other_fields_differ;
    if !s1_holds {
        tally.add_violation(Counter::AtkS1Violations, describe_witness);
    }
}

/// 岔路单 K10：同一实例里 checkpoint_txg 随计数器不减（表按 (实例, 计数器) 升序）。
fn attack_k10_holds(records: &RecordTable) -> bool {
    let mut previous: Option<(InstanceGeneration, CheckpointTxg)> = None;
    for record in records.values() {
        if let Some((instance, checkpoint_txg)) = previous {
            if instance == record.instance && record.checkpoint_txg < checkpoint_txg {
                return false;
            }
        }
        previous = Some((record.instance, record.checkpoint_txg));
    }
    true
}

/// 对抗一族：K10 下 AS 三条；两种读法期望不同的态上实现等于哪一种（全部态与 K10 态各数一遍）。
fn attack_judge_adversarial(
    records: &RecordTable,
    first: &ReplayOutcome,
    expectations: &[SpecificationExpectation; 3],
    tally: &mut Tally,
) {
    let k10 = attack_k10_holds(records);
    let matches = expectations.map(|expectation| outcome_matches_the_expectation(first, expectation));
    if k10 {
        tally.add(Counter::AtkK10States);
        for (index, counter) in [
            Counter::AtkK10AsWholeTable,
            Counter::AtkK10AsGapContinues,
            Counter::AtkK10AsGapStops,
        ]
        .into_iter()
        .enumerate()
        {
            if !matches[index] {
                tally.add(counter);
            }
        }
    }
    let pairs = [
        (
            (0usize, 1usize),
            [Counter::AtkDiffWcImplW, Counter::AtkDiffWcImplC, Counter::AtkDiffWcImplNone],
            [Counter::AtkK10DiffWcImplW, Counter::AtkK10DiffWcImplC, Counter::AtkK10DiffWcImplNone],
        ),
        (
            (1, 2),
            [Counter::AtkDiffCsImplC, Counter::AtkDiffCsImplS, Counter::AtkDiffCsImplNone],
            [Counter::AtkK10DiffCsImplC, Counter::AtkK10DiffCsImplS, Counter::AtkK10DiffCsImplNone],
        ),
        (
            (0, 2),
            [Counter::AtkDiffWsImplW, Counter::AtkDiffWsImplS, Counter::AtkDiffWsImplNone],
            [Counter::AtkK10DiffWsImplW, Counter::AtkK10DiffWsImplS, Counter::AtkK10DiffWsImplNone],
        ),
    ];
    for ((first_reading, second_reading), all_counters, k10_counters) in pairs {
        if expectations[first_reading] == expectations[second_reading] {
            continue;
        }
        let which = if matches[first_reading] {
            0
        } else if matches[second_reading] {
            1
        } else {
            2
        };
        tally.add(all_counters[which]);
        if k10 {
            tally.add(k10_counters[which]);
        }
    }
}

fn attack_row(place: &str, tally: &Tally) -> String {
    let mut fields: Vec<String> = Vec::new();
    for counter in Counter::ALL {
        if counter.label().starts_with("atk_") {
            fields.push(format!("{}={}", counter.label(), tally.get(*counter)));
        }
    }
    for counter in [
        Counter::AtkS1MarkerDiffers,
        Counter::AtkUnmountShapeMarkerDiffers,
        Counter::AtkLegalOwnS1MarkerDiffers,
        Counter::AtkJ6WritingAViolations,
        Counter::AtkJ6WritingBViolations,
        Counter::AtkK10J6PremiseHolds,
        Counter::AtkS1InstanceTableDiffers,
    ] {
        if let Some(witness) = tally.witness(counter) {
            fields.push(format!("witness_{}={}", counter.label(), witness.replace(' ', "_")));
        }
    }
    format!("name=attack {place} {}", fields.join(" "))
}
