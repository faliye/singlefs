use super::*;

/// J4：观察者挂的节点照设点文件与入口名认计划里的点：`<入口>.<周期>.<步>` 的每一段都是，名字只是同一个前缀的不是，别的文件里同名的不是。
#[test]
fn an_observer_node_holds_every_point_of_its_entry_in_its_setting_file_only() {
    let node = NodeOfAnObserver {
        setting_file: "crates/singlefs-verification/tests/common_crash_points/mod.rs".to_string(),
        entry_name: "sequential_write".to_string(),
    };
    let point_named = |code_file: &str, name: &str| PlannedCrashPoint {
        name: name.to_string(),
        ignored: false,
        segments: Vec::new(),
        ordinals: 0..0,
        reuse_key: StepAttributes {
            write_table_with_barriers: [0; DIGEST_BYTES],
            parent_output: [0; DIGEST_BYTES],
        }
        .reuse_key(),
        path_number: name.to_string(),
        identity: format!("{code_file}::{name}::00"),
        judge_inputs: [0; DIGEST_BYTES],
        plan_hash: [0; DIGEST_BYTES],
        pieces_around_the_slices: Vec::new(),
    };
    let common = "crates/singlefs-verification/tests/common_crash_points/mod.rs";
    assert!(node.holds_the_point(&point_named(common, "sequential_write.journal_record.1")));
    assert!(node.holds_the_point(&point_named(common, "sequential_write")));
    assert!(!node.holds_the_point(&point_named(
        common,
        "sequential_write_twice.journal_record.1"
    )));
    assert!(!node.holds_the_point(&point_named(
            "crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
            "sequential_write.journal_record.1"
        )));
}

/// 观察者那一把版本的文字只由标识那一截（判定项标识与观察者引到的被检测产品代码）与另读的输入组成（观察者自己的代码与手写的版本常量都不进）：
/// 观察者那一把是这两样拼成的那段文字的摘要，代码那一把是同一段文字、另读的输入取全零。文字里多一截别的，两把都对不上。
#[test]
fn the_observer_versions_hash_only_the_judging_item_identity_and_the_extra_inputs() {
    let identity_of_the_observer =
        sha256_digest(b"judging item identity of the version text unit test\n");
    let extra_inputs_of_the_observer =
        sha256_digest(b"extra inputs of the version text unit test\n");
    let observe_nothing = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           _counts: &mut CrashPointReplayObserverCounts| {};
    let observer = ObserverOnTheNodes {
        observer_name: ObserverName::new("version_text_unit_test_observer").expect("名字合规矩"),
        observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
        extra_inputs_digest: extra_inputs_of_the_observer,
        nodes: Vec::new(),
        observe: ObserverCode::CountsOnly(&observe_nothing),
    };
    let versions =
        observer_versions_of_the_identity(&observer, JudgedBy::CpuJudge, &identity_of_the_observer);
    let version_text_of = |extra_inputs: &[u8; DIGEST_BYTES]| {
        format!(
            "singlefs observer\nidentity={}\nextra_inputs={}\n",
            hexadecimal_text(&identity_of_the_observer),
            hexadecimal_text(extra_inputs)
        )
    };
    assert_eq!(
        versions.observer,
        ConclusionVersion(sha256_digest(
            version_text_of(&extra_inputs_of_the_observer).as_bytes()
        )),
        "观察者那一把是判定项标识与另读的输入拼成的文字的摘要，文字里没有别的"
    );
    assert_eq!(
        versions.observer_code,
        ConclusionVersion(sha256_digest(
            version_text_of(&[0; DIGEST_BYTES]).as_bytes()
        )),
        "代码那一把是同一段文字、另读的输入取全零"
    );
    assert_eq!(
        versions.pool_checker_version,
        ConclusionVersion::of_the_pool_checker_on_the_images_after_the_observers(),
        "池级 checker 那一把由结论种类与判器引到的被检测产品代码定，不带判器的代码"
    );
    assert_eq!(
        versions.conclusion,
        observer_versions_of_the_identity(&observer, JudgedBy::GpuJudge, &identity_of_the_observer)
            .conclusion,
        "结论那一把两条路线同一把：在哪判的不进键"
    );
}

/// 别的包构建脚本照同一套写的一张键表（`singlefs_unit_graph::entry_groups::entry_group_table_text`）：每个（观察者名，闭包键）一组，
/// 观察者代码文件都是单测那一份。交回的原文活到进程结束（登记表收的是 `&'static str`）。
fn table_text_registering_the_unit_test_observers(
    observers: &[(&str, [u8; DIGEST_BYTES])],
) -> &'static str {
    String::leak(singlefs_unit_graph::entry_groups::entry_group_table_text(
        &[],
        &observers
            .iter()
            .map(
                |(observer_name, closure_key)| singlefs_unit_graph::entry_groups::EntryGroupKey {
                    group_name: crate::judging_units_entries::observer_group_name(
                        OBSERVER_UNIT_TEST_SETTING_FILE,
                        observer_name,
                    ),
                    verdict: singlefs_unit_graph::entry_groups::EntryGroupVerdict::Keyed {
                        key: singlefs_unit_graph::sha256::Sha256Digest(*closure_key),
                    },
                    entries_not_compiled: Vec::new(),
                    composition_lines: Vec::new(),
                    watched_files: std::collections::BTreeSet::new(),
                },
            )
            .collect::<Vec<singlefs_unit_graph::entry_groups::EntryGroupKey>>(),
    ))
}

/// 单测那一份观察者代码文件里叫 `observer_name` 的观察者，只记计数、另读的输入取 `extra_inputs_digest`。
fn unit_test_observer_named<'observer>(
    observer_name: &str,
    extra_inputs_digest: [u8; DIGEST_BYTES],
    observe: &'observer ObserveACrashState<'observer>,
) -> ObserverOnTheNodes<'observer> {
    ObserverOnTheNodes {
        observer_name: ObserverName::new(observer_name).expect("名字合规矩"),
        observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
        extra_inputs_digest,
        nodes: Vec::new(),
        observe: ObserverCode::CountsOnly(observe),
    }
}

/// 场景：同一份观察者代码文件里两个观察者（名字不同、另读的输入相同），都登在一张键表里，挂在同一块上（各自一把版本）。
/// 预期：两个都登记了；两把观察者版本不同、两把结论版本也不同（同一块上的两份结论不撞成一把）；各自那一截判定项标识就是
/// 「代码文件与观察者名拼的组名」取的摘要（[`observer_identity_digest`]），版本照它与表里那一组的键拼（[`observer_identity_with_its_product_code_digest`]）。
/// 判别力（`crates/mutations.tsv`）：判定项标识那一截只取种类名（不带代码文件与观察者名），这条红。
#[test]
fn two_observers_in_one_file_take_two_versions_by_their_judging_item_identities() {
    let observe_nothing = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           _counts: &mut CrashPointReplayObserverCounts| {};
    let extra_inputs = sha256_digest(b"extra inputs shared by the two unit test observers\n");
    let counting_the_states =
        unit_test_observer_named("counting_the_states", extra_inputs, &observe_nothing);
    let counting_the_roots =
        unit_test_observer_named("counting_the_roots", extra_inputs, &observe_nothing);
    let observer_keys = ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage {
        table_text: table_text_registering_the_unit_test_observers(&[
            ("counting_the_states", [0x11; DIGEST_BYTES]),
            ("counting_the_roots", [0x11; DIGEST_BYTES]),
        ]),
    };
    let versions_of = |observer: &ObserverOnTheNodes<'_>| {
        let (versions, refusals, versions_kept_out) =
            observer_versions_keeping_the_refused_out_with_the_observer_keys_in(
                observer,
                JudgedBy::CpuJudge,
                observer_keys,
            );
        assert!(
            refusals.is_empty() && versions_kept_out.is_empty(),
            "前提：两个观察者都登记了"
        );
        versions
    };
    let (states_versions, roots_versions) = (
        versions_of(&counting_the_states),
        versions_of(&counting_the_roots),
    );
    assert_ne!(
        states_versions.observer, roots_versions.observer,
        "两个观察者两把观察者版本"
    );
    assert_ne!(
        states_versions.conclusion, roots_versions.conclusion,
        "两个观察者两把结论版本"
    );
    assert_eq!(
        observer_identity_digest(&observer_code_units_of(&counting_the_states)),
        sha256_digest(
            format!(
                "singlefs observer identity\nobserver_code:{OBSERVER_UNIT_TEST_SETTING_FILE}:counting_the_states\n"
            )
            .as_bytes()
        ),
        "判定项标识那一截是代码文件与观察者名拼的组名取的摘要"
    );
    assert_eq!(
        states_versions,
        observer_versions_of_the_identity(
            &counting_the_states,
            JudgedBy::CpuJudge,
            &observer_identity_with_its_product_code_digest(
                &observer_code_units_of(&counting_the_states),
                &JudgingUnitsKey::of_the_parts([0x11; DIGEST_BYTES], Vec::new()),
            ),
        ),
        "登记了的观察者照它的判定项标识与表里那一组的键拼版本"
    );
}

/// 场景：同一个观察者登在两张键表里，两张表里它那一组的键不同（验证包那一张的观察者组只留产品包的单元：键变了是它调到的产品代码改了），
/// 各取一遍版本；再造一张同一组键相同的表取第三遍。
/// 预期：键不同的两遍几把版本逐把不同（观察者调到的产品挂载、恢复代码改了，它的块现录、现判）；键相同的两遍逐把相同。
/// 观察者自己的代码不进那一组的键由构建脚本按包过滤管（`crate::crash_identity` 键表测试里改观察者一处那一条）。
/// 判别力（`crates/mutations.tsv`）：标识那一截不拼那一组的键（观察者引到的被检测产品代码不进版本），这条红在「逐把不同」那一句。
#[test]
fn a_change_of_the_product_code_the_observer_reaches_changes_its_versions() {
    let observe_nothing = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           _counts: &mut CrashPointReplayObserverCounts| {};
    let observer = unit_test_observer_named(
        "counting_the_states",
        sha256_digest(b"extra inputs of the unit test observer whose product code changes\n"),
        &observe_nothing,
    );
    let versions_under_the_product_code_key = |product_code_key: [u8; DIGEST_BYTES]| {
        let (versions, refusals, _versions_kept_out) =
            observer_versions_keeping_the_refused_out_with_the_observer_keys_in(
                &observer,
                JudgedBy::GpuJudge,
                ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage {
                    table_text: table_text_registering_the_unit_test_observers(&[(
                        "counting_the_states",
                        product_code_key,
                    )]),
                },
            );
        assert!(refusals.is_empty(), "前提：观察者登记了、那一组算得出键");
        versions
    };
    let before = versions_under_the_product_code_key([0x21; DIGEST_BYTES]);
    let after = versions_under_the_product_code_key([0x22; DIGEST_BYTES]);
    assert_ne!(before.observer, after.observer, "观察者那一把逐把不同");
    assert_ne!(
        before.observer_code, after.observer_code,
        "代码那一把逐把不同"
    );
    assert_ne!(before.conclusion, after.conclusion, "结论那一把逐把不同");
    assert_eq!(
        before,
        versions_under_the_product_code_key([0x21; DIGEST_BYTES]),
        "那一组的键相同，几把版本逐把相同"
    );
}

/// 录制设备记下的读按扇区对齐、按盘与起扇区排、相邻或相交的并成一段，长度 0 的不记。
#[test]
fn recorded_reads_are_sector_aligned_sorted_and_merged_into_ranges() {
    use crate::RecordedReadRange;
    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
    let ranges = ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(&[
        RecordedReadRange {
            device: DeviceIdentity(2),
            offset: DeviceOffsetInBytes(4096),
            length: 1024,
        },
        RecordedReadRange {
            device: DeviceIdentity(2),
            offset: DeviceOffsetInBytes(5120),
            length: 512,
        },
        RecordedReadRange {
            device: DeviceIdentity(2),
            offset: DeviceOffsetInBytes(0),
            length: 100,
        },
        RecordedReadRange {
            device: DeviceIdentity(1),
            offset: DeviceOffsetInBytes(512),
            length: 512,
        },
        RecordedReadRange {
            device: DeviceIdentity(2),
            offset: DeviceOffsetInBytes(4096),
            length: 0,
        },
    ]);
    assert_eq!(
        ranges,
        vec![
            ReadRangeOnTheCrashImage {
                device: 1,
                first_sector: 1,
                end_sector: 2
            },
            ReadRangeOnTheCrashImage {
                device: 2,
                first_sector: 0,
                end_sector: 1
            },
            ReadRangeOnTheCrashImage {
                device: 2,
                first_sector: 8,
                end_sector: 11
            },
        ]
    );
}
