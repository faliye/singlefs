//! 在临时拷贝上改一处之后，几组键变不变：池级 checker、记录核对器与观察者改一处，判器引到的被检测产品代码那一把与观察者那一把不变
//! （改的那一处真在判器或观察者收全的闭包里）；产品恢复里只在崩溃态才走到的那一支改一处，判器引到的被检测产品代码那一把变、判器判出的结论版本
//! 跟着变；仓根 `Cargo.toml` 的 `[profile.release]` 改一项，每一把都不变。连同编进去的键与盘上这棵树照同一套算法重算的逐组相同。
//! 录入键不在这里：它只由录下的写流摘要与场景标识组成，代码一个单元都不进（`crate::crash_library::recording_key_of_the_scenario_and_the_inputs`）。

use super::*;

/// 固定脚本流预置残留记录与旧尾巴那两条用例所在的测试目标的文件。
const FIXED_SCRIPT_STREAM_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs";

/// 固定脚本流那条用例挂的残留记录观察者的名字（登在 `OBSERVER_ENTRIES`）。
const RESIDUAL_RECORD_OBSERVER: &str = "residual_record_chain";

/// 构建脚本写进二进制的那份表，与这个测试在盘上这棵树上照同一套算法、同一个目标重算的逐组相同；编译之后改了树的先比摘要认出来、
/// 报树变了，不比键（[`RepositoryCopy::assert_the_watched_files_are_the_ones_the_build_read`]）。
#[test]
fn the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree() {
    let compiled = compiled_keys();
    let copy = RepositoryCopy::new("compiled-keys");
    copy.assert_the_watched_files_are_the_ones_the_build_read();
    let from_the_tree = registry_keys_under(&copy.root);
    assert_eq!(
        compiled, from_the_tree,
        "编进去的键与盘上这棵树重算的逐组相同"
    );
    assert!(
        compiled
            .get(JUDGED_PRODUCT_CODE_GROUP_NAME)
            .is_some_and(Option::is_some),
        "判器引到的被检测产品代码那一组算得出键"
    );
}

/// 编进去的那份表里依赖包的特性（`build-feature` 行，构建脚本读核心层经 `links` 交出的那一份），与这个测试二进制链进来的核心层
/// 实际开着的（[`features_the_core_layer_was_compiled_with`]，从核心层编出来的代码里认）逐个相同：只编验证包、不编它的测试时两边都空，
/// 编验证包的测试（`cargo test -p singlefs-verification`、`cargo test --all`）时两边都是核心层的
/// `lookup-work-counters`。核心层清单里的特性也要都是认得出的那几个，多了一个没法认的，这一条先红。
/// 判别力（`crates/mutations.tsv`，编测试的那一行）：构建脚本改回只读本包特性（不读核心层交出的那一份），编测试时这条红。
#[test]
fn the_dependency_features_compiled_into_the_table_are_the_ones_the_core_layer_was_built_with() {
    // 路径写成一串字面量（不经 `.parent()`）：变异判法的单元键要从字面路径认出读的是哪份文件。
    let core_manifest_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../singlefs-core/Cargo.toml");
    let core_manifest = singlefs_unit_graph::manifest::parse_crate_manifest(
        &std::fs::read_to_string(core_manifest_path).expect("核心层清单读得了"),
    )
    .expect("核心层清单解析得了");
    assert_eq!(
            core_manifest
                .features
                .keys()
                .map(String::as_str)
                .collect::<Vec<&str>>(),
            CORE_LAYER_FEATURES_RECOGNIZED_FROM_ITS_CODE,
            "核心层清单里的特性就是 features_the_core_layer_was_compiled_with 认得出的那几个（多了的要在那里加认法）"
        );
    assert_eq!(
        compiled_judging_units_keys().features_enabled_by_the_build,
        features_the_core_layer_was_compiled_with(),
        "编进去的表里依赖包的特性（构建脚本读到的）与链进来的核心层实际开着的逐个相同"
    );
}

/// 场景：仓根 `Cargo.toml` 的 `[profile.release]` 把 `overflow-checks` 从开改成关，在临时拷贝上于变更前与变更后各重算一次固定脚本流那条用例
/// 挂的观察者那一组与判器引到的被检测产品代码那一组。
/// 预期：变更前两组都算得出键、与编进去的逐组相同；变更后逐组不变，也还是编进去的那几把——编译档不进任何键。
/// 判别力（`crates/mutations.tsv`）：构建脚本把编译档写进键（编进去的键带上了编译档），这条红在「与编进去的逐组相同」那一句。
#[test]
fn an_edit_of_the_release_profile_of_the_root_manifest_keeps_every_key() {
    let wanted_groups = [
        observer_group_name(FIXED_SCRIPT_STREAM_TEST_FILE, RESIDUAL_RECORD_OBSERVER),
        JUDGED_PRODUCT_CODE_GROUP_NAME.to_string(),
    ];
    let is_wanted = |group_name: &str| wanted_groups.iter().any(|wanted| wanted == group_name);
    let compiled: KeysOfEachGroup = compiled_keys()
        .into_iter()
        .filter(|(group_name, _key)| is_wanted(group_name))
        .collect();
    let copy = RepositoryCopy::new("release-profile");
    copy.assert_the_watched_files_are_the_ones_the_build_read();
    let before = registry_keys_of_the_groups_under(&copy.root, &is_wanted);
    assert_eq!(
        before.values().filter(|key| key.is_some()).count(),
        wanted_groups.len(),
        "前提：两组改前都算得出键：{before:?}"
    );
    assert_eq!(before, compiled, "改前在这棵树上重算的与编进去的逐组相同");
    copy.replace_exactly_once(
        "Cargo.toml",
        "overflow-checks = true",
        "overflow-checks = false",
    );
    let after = registry_keys_of_the_groups_under(&copy.root, &is_wanted);
    assert_eq!(before, after, "改了编译档，两组的键逐组不变");
}

/// 改的那一处在收全的闭包里怎么认：落在哪份文件里的单元，或叫什么名字的单元。
enum EditedUnit<'name> {
    InTheFile(&'name str),
    Named(&'name str),
}

impl EditedUnit<'_> {
    fn is_held_by(&self, composition: &[String]) -> bool {
        match self {
            Self::InTheFile(file) => composition_holds_a_unit_in_the_file(composition, file),
            Self::Named(name) => composition_holds_a_unit_named(composition, name),
        }
    }
}

/// 几条「改一处、键不变」共用的那一段：在临时拷贝上照 `edit` 改一处，判器引到的被检测产品代码那一组前后各算一次。
/// 前提（阳性对照）：改的那一处在判器（CPU 参照判器、池级 checker、记录核对器那几个入口）收全的闭包里；预期：组成清单里没有它，键前后相同。
fn assert_an_edit_of_a_judge_keeps_the_judged_product_code_key(
    label: &str,
    edit: (&str, &str, &str),
    edited_unit: &EditedUnit<'_>,
) {
    let (before, after) = groups_before_and_after(label, &[judged_product_code_group()], edit);
    assert!(
        edited_unit.is_held_by(&before[0].whole_composition),
        "前提：改的那一处在判器那几个入口收全的闭包里"
    );
    assert!(
        !edited_unit.is_held_by(&before[0].kept_composition),
        "改的那一处不在判器引到的被检测产品代码那一组里"
    );
    assert_eq!(
        before[0].key_of(JUDGED_PRODUCT_CODE_GROUP_NAME),
        after[0].key_of(JUDGED_PRODUCT_CODE_GROUP_NAME),
        "判器引到的被检测产品代码那一把不变：改的那一处不是被检测的产品代码"
    );
}

/// 场景：池级 checker 的一个常量（载荷校验和偏移）改一个数，在临时拷贝上重算。
/// 预期：池级 checker 在判器收全的闭包里；判器引到的被检测产品代码那一组里没有它，键不变——改池级 checker 不作废已判的块（判器自己的代码不进键）。
/// 判别力（`crates/mutations.tsv`）：判器那一组留的包里加上池级 checker，这条红在「不在判器引到的被检测产品代码那一组里」那一句。
#[test]
fn an_edit_of_the_pool_checker_keeps_the_judged_product_code_key() {
    assert_an_edit_of_a_judge_keeps_the_judged_product_code_key(
        "pool-checker-outside-the-product-code-keys",
        (
            "crates/singlefs-checker/src/lib.rs",
            "const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;",
            "const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 91;",
        ),
        &EditedUnit::InTheFile("crates/singlefs-checker/src/lib.rs"),
    );
}

/// 场景：记录核对器（崩溃放量第 ② 段 CPU 参照那一份，`crash_facts::verify_states_from_facts_by_the_segment_expansions`，判器那一组的入口之一）
/// 的函数体改一处写法，在临时拷贝上重算判器引到的被检测产品代码那一组。
/// 预期：它在判器收全的闭包里；那一组里没有它（入口单元不进只键产品包的那一把），键不变。
/// 判别力（`crates/mutations.tsv`）：只键产品包那一种照留入口单元，这条红在「不在判器引到的被检测产品代码那一组里」那一句。
#[test]
fn an_edit_of_the_record_verifier_keeps_the_judged_product_code_key() {
    assert_an_edit_of_a_judge_keeps_the_judged_product_code_key(
        "record-verifier-outside-the-product-code-keys",
        (
            "crates/singlefs-verification/src/crash_facts.rs",
            "                    segment_expansions,\n                    ordinal,\n                ),",
            "                    segment_expansions,\n                    (ordinal),\n                ),",
        ),
        &EditedUnit::Named("verify_states_from_facts_by_the_segment_expansions"),
    );
}

/// 场景：固定脚本流那条用例挂的残留记录观察者（登在 `OBSERVER_ENTRIES`，写在用例文件里）的函数体改一处写法，在临时拷贝上重算
/// 判器引到的被检测产品代码那一组与这个观察者那一组。
/// 预期：它在观察者那一组收全的闭包里；观察者那一组只键产品包，组成清单里没有它，两把键都不变——观察者的代码不进它自己的版本。
/// 判别力（`crates/mutations.tsv`）：观察者那几组改回收全闭包，这条红在「不在观察者那一组里」那一句。
#[test]
fn an_edit_of_a_registered_observer_keeps_its_product_code_key_and_the_judged_product_code_key() {
    let edited_unit = EditedUnit::Named("observe_the_chain_to_the_residual_record");
    let (before, after) = groups_before_and_after(
        "observer-outside-the-product-code-keys",
        &[
            judged_product_code_group(),
            observer_group_of(FIXED_SCRIPT_STREAM_TEST_FILE, RESIDUAL_RECORD_OBSERVER),
        ],
        (
            FIXED_SCRIPT_STREAM_TEST_FILE,
            "        counts.add(\"chain_reaches_the_residual_record\", 1);",
            "        counts.add(\"chain_reaches_the_residual_record\", (1));",
        ),
    );
    assert!(
        edited_unit.is_held_by(&before[1].whole_composition),
        "前提：观察者在它自己那一组收全的闭包里"
    );
    assert!(
        !edited_unit.is_held_by(&before[1].kept_composition),
        "观察者不在观察者那一组里（只键它引到的被检测产品代码）"
    );
    for (group_before, group_after, group) in [
        (&before[0], &after[0], JUDGED_PRODUCT_CODE_GROUP_NAME),
        (&before[1], &after[1], RESIDUAL_RECORD_OBSERVER),
    ] {
        assert_eq!(
            group_before.key_of(group),
            group_after.key_of(group),
            "{group} 那一把不变：改的是观察者自己的代码"
        );
    }
}

/// 场景：产品恢复里只在崩溃态才走到的那一支（所选那一版三样读者的并集丢了字节、判退之后重扫一遍 journal 环的那一句，
/// `recovery::fall_back_on_the_three_readers`）改一处写法，在临时拷贝上重算判器引到的被检测产品代码那一组。
/// 预期：那一支的函数在这一组里（CPU 参照判器调产品的恢复）；这一组的键变，判器判出的三种结论的版本跟着变——已判的块现判。
/// 录制时新格式化的池上挂载走不到这一支，块键（写表与起步镜像）不随它变，崩溃判定的键靠这一把抓住它。
/// 判别力（`crates/mutations.tsv`）：判器那一组的入口登记里去掉 CPU 参照判器判一个崩溃镜像那一个（够不到产品的恢复），这条红在「前提」那一句；
/// 只键产品包那一种一个产品包的单元都不留，这条红在「前提」那一句。
#[test]
fn an_edit_of_a_recovery_branch_taken_only_in_a_crash_state_changes_the_judged_product_code_key() {
    let (before, after) = groups_before_and_after(
        "recovery-branch-in-the-judged-product-code-key",
        &[judged_product_code_group()],
        (
            "crates/singlefs-core/src/recovery/fall_back_on_the_three_readers.rs",
            "    let records_rescanned_after_the_trigger = scan_journal(reader, system_configuration);",
            "    let records_rescanned_after_the_trigger = scan_journal(reader, (system_configuration));",
        ),
    );
    assert!(
        composition_holds_a_unit_named(
            &before[0].kept_composition,
            "fall_back_judged_on_the_three_readers"
        ),
        "前提：判退那一支的函数在判器引到的被检测产品代码那一组里"
    );
    let key_before = before[0].key_of(JUDGED_PRODUCT_CODE_GROUP_NAME);
    let key_after = after[0].key_of(JUDGED_PRODUCT_CODE_GROUP_NAME);
    assert_ne!(
        key_before, key_after,
        "产品恢复改了，判器引到的被检测产品代码那一把变"
    );
    for conclusion in JudgedConclusion::EVERY_JUDGED_CONCLUSION {
        assert_ne!(
            ConclusionVersion::of_the_judged_conclusion(
                conclusion,
                &JudgedProductCodeOfThisBinary::Keyed {
                    fingerprint: key_before
                }
            ),
            ConclusionVersion::of_the_judged_conclusion(
                conclusion,
                &JudgedProductCodeOfThisBinary::Keyed {
                    fingerprint: key_after
                }
            ),
            "{conclusion:?} 的版本跟着变：已判的块现判"
        );
    }
}
