//! 观察者引到的被检测产品代码：登记表里的每个观察者都在编进来的那一张表里算得出键、组成清单只有产品包的单元，没登记的拒算，
//! 登在别的包那一张表里的只在那一张里取键。观察者自己的代码不进结论的键（几把版本由判定项标识、这一组的键与另读的输入定，
//! `crate::crash_amplification::observer_identity_with_its_product_code_digest`）。

use super::*;

fn toolchain_left_empty() -> CrashPointReplayToolchainIdentity {
    CrashPointReplayToolchainIdentity {
        rustc_version_lines: String::new(),
        cargo_version: String::new(),
        target_triple: String::new(),
    }
}

/// 场景：观察者登记表（`OBSERVER_ENTRIES`）里的每个观察者，在编进这个测试二进制的那一张表里取它引到的被检测产品代码的键（构建时编进来，不读活树）。
/// 预期：一个个都算得出键（没登记、拒算的都会让它的结论照判不入库）；每一组的组成清单里每个单元都落在产品包（角色是「产品」的那五个的目录，
/// 在这里逐个写出、不照 `PRODUCT_PACKAGES_KEPT_IN_THE_KEYS` 取）里：观察者自己的代码（验证包的 `tests/` 与 `src/`）一个单元都不在里面；
/// 至少一组的组成清单不空（不然这一条在真表上没有对象）。
/// 判别力（`crates/mutations.tsv`）：构建脚本不写观察者那几组，或观察者那几组的本包特性不开 `gpu`（只在 `gpu` 下编的那个测试目标里的观察者
/// 不编进来），这条红在「算得出键」那一句；观察者那几组改回收全闭包，这条红在「落在产品包里」那一句。
#[test]
fn every_observer_of_the_observer_entries_is_keyed_in_the_compiled_table_with_product_units_only() {
    assert!(!OBSERVER_ENTRIES.is_empty(), "前提：登记表里有观察者");
    let product_directories: Vec<String> = [
        "singlefs-format",
        "singlefs-core",
        "singlefs-replay-prefix",
        "singlefs-mount-runtime",
        "singlefs-background-gpu",
    ]
    .iter()
    .map(|package_name| format!("crates/{package_name}/"))
    .collect();
    let mut not_keyed: Vec<String> = Vec::new();
    let mut units_checked = 0_usize;
    let mut units_outside: Vec<String> = Vec::new();
    for observer in OBSERVER_ENTRIES {
        match observer_product_code_key_in(
            &JudgingUnits::ObserverCode {
                observer_code_file: observer.observer_code_file.to_string(),
                observer_name: observer.observer_name.to_string(),
            },
            ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
        ) {
            Ok(product_code) => {
                for unit in product_code.composition() {
                    units_checked += 1;
                    if !product_directories
                        .iter()
                        .any(|directory| unit.unit.contains(&format!(" @ {directory}")))
                    {
                        units_outside.push(format!("{}：{}", observer.observer_name, unit.unit));
                    }
                }
            }
            Err(refusal) => not_keyed.push(refusal.sentence()),
        }
    }
    assert!(
        not_keyed.is_empty(),
        "登记表里的观察者在编进来的表里算不出键：{not_keyed:#?}"
    );
    assert!(units_checked > 0, "前提：观察者那几组的组成清单不空");
    assert!(
        units_outside.is_empty(),
        "观察者那几组里有落在产品包之外的单元（共查 {units_checked} 个）：{units_outside:#?}"
    );
}

/// 场景：一个没登记入口 fn 的观察者，在编进来的那一张表里认。
/// 预期：拒算（照判、不入库），形态是 `not_computed_at_the_build`，红句点名它那一组与登在哪（`OBSERVER_ENTRIES`）。
/// 判别力（`crates/mutations.tsv`）：认登记时一律当登记了，这条红。
#[test]
fn an_observer_without_a_registered_entry_is_refused_as_not_computed_at_the_build() {
    let refused = observer_product_code_key_in(
        &JudgingUnits::ObserverCode {
            observer_code_file: "crates/singlefs-verification/tests/nowhere.rs".to_string(),
            observer_name: "unregistered".to_string(),
        },
        ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
    )
    .expect_err("没登记就拒算");
    assert_eq!(refused.form_name(), "not_computed_at_the_build");
    let sentence = refused.sentence();
    assert!(
        sentence
            .contains("observer_code:crates/singlefs-verification/tests/nowhere.rs:unregistered")
            && sentence.contains("OBSERVER_ENTRIES"),
        "{sentence}"
    );
}

/// 别的包构建脚本照同一套写的一张键表（`singlefs_unit_graph::entry_groups::entry_group_table_text`）：只有一个观察者那一组，键是 `key`。
/// 交回的原文活到进程结束（枚举入口收的是 `&'static str`，与编进二进制的那一张同形）。
fn table_text_of_another_package_keying_one_observer(
    observer_code_file: &str,
    observer_name: &str,
    key: [u8; DIGEST_BYTES],
) -> &'static str {
    String::leak(singlefs_unit_graph::entry_groups::entry_group_table_text(
        &[],
        &[singlefs_unit_graph::entry_groups::EntryGroupKey {
            group_name: observer_group_name(observer_code_file, observer_name),
            verdict: EntryGroupVerdict::Keyed {
                key: singlefs_unit_graph::sha256::Sha256Digest(key),
            },
            entries_not_compiled: Vec::new(),
            composition_lines: Vec::new(),
            watched_files: BTreeSet::new(),
        }],
    ))
}

/// 场景：一个观察者只登在别的包（实验包）构建脚本写的那一张键表里；它分别在点名的那一张、在验证包那一张里取键，那一张里没有的另一个观察者
/// 在点名的那一张里取；实验包的测试照 [`judging_units_key_with_the_observers_keyed_in`] 取它在那一张里的键。
/// 预期：在点名的那一张里取得出、就是那一张里写的那一把；在验证包那一张里拒算；那一张里没有的拒算、红句点名实验包的观察者登记表；
/// 经实验包那个入口取得出的也是那一把。
/// 判别力（`crates/mutations.tsv`）：点名别的包那一张时改回在验证包那一张里取，这条红。
#[test]
fn an_observer_keyed_in_the_table_of_another_package_takes_its_key_there_and_not_from_the_verification_table(
) {
    let observer_code_file = "crates/singlefs-implementation-experiments/src/bin/probe.rs";
    let key_in_the_other_table = [0x5a; DIGEST_BYTES];
    let observer_keys = ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage {
        table_text: table_text_of_another_package_keying_one_observer(
            observer_code_file,
            "keyed_in_the_experiments",
            key_in_the_other_table,
        ),
    };
    let observer_units = JudgingUnits::ObserverCode {
        observer_code_file: observer_code_file.to_string(),
        observer_name: "keyed_in_the_experiments".to_string(),
    };
    assert_eq!(
        observer_product_code_key_in(&observer_units, observer_keys)
            .expect("登在点名的那一张里就取得出")
            .fingerprint(),
        &key_in_the_other_table,
        "键是点名的那一张里写的那一把"
    );
    assert_eq!(
        observer_product_code_key_in(
            &observer_units,
            ObserverJudgingUnitsKeyTable::OfTheVerificationPackage
        )
        .expect_err("验证包那一张里没登记它")
        .form_name(),
        "not_computed_at_the_build"
    );
    let missing_from_the_other_table = observer_product_code_key_in(
        &JudgingUnits::ObserverCode {
            observer_code_file: observer_code_file.to_string(),
            observer_name: "not_registered_anywhere".to_string(),
        },
        observer_keys,
    )
    .expect_err("点名的那一张里没有它");
    assert!(
        missing_from_the_other_table
            .sentence()
            .contains("实验包的 judging_units_entries::OBSERVER_ENTRIES"),
        "{}",
        missing_from_the_other_table.sentence()
    );
    assert_eq!(
        judging_units_key_with_the_observers_keyed_in(
            &observer_units,
            observer_keys,
            &toolchain_left_empty()
        )
        .expect("登在点名的那一张里就取得出")
        .fingerprint(),
        &key_in_the_other_table,
        "键是点名的那一张里写的那一把"
    );
}

/// 场景：这个测试二进制是验证包的测试构建（dev 依赖替核心层并上 `lookup-work-counters`，编进去的表的 `build-feature` 行有它）。在临时拷贝上
/// 照构建脚本那一套把登记表里每个观察者那一组算两遍：一遍依赖包一个特性都不当开着（不编测试、只编验证包的那一种构建看到的），
/// 一遍照这一次测试构建实际并上来的那几个。
/// 预期：每个观察者那一组两遍算出同一把键，编进这个测试二进制的那一把也就是这一把：带不带计数、带不带 `gpu` 的几种构建算出同一把观察者的版本，
/// 同一批块不判两遍。比过的组数打进 `OBSERVER_KEYS_UNDER_TWO_BUILDS` 那一行。
/// 判别力（`crates/mutations.tsv`「收测试目标的闭包不顺 dev 依赖解特性」）：测试目标的闭包改成只顺普通依赖解特性，核心层的计数特性只剩
/// 依赖包那一侧交进来的才开，两遍算出两把键，这条红在「两组依赖包特性下同一把键」那一句。
#[test]
fn the_observer_product_code_keys_are_the_same_under_the_dependency_features_of_either_build() {
    let test_build_features = compiled_judging_units_keys()
        .features_enabled_by_the_build
        .clone();
    assert!(
        test_build_features
            .features_of_each_package
            .get(CORE_LAYER_PACKAGE_NAME)
            .is_some_and(|features| features.contains(LOOKUP_WORK_COUNTERS_FEATURE)),
        "前提：这个测试二进制是测试构建，编进去的表里核心层开着 {LOOKUP_WORK_COUNTERS_FEATURE}：{test_build_features:?}"
    );
    let copy = RepositoryCopy::new("observer-keys-under-two-builds");
    copy.assert_the_watched_files_are_the_ones_the_build_read();
    let tree = TreeLoadedForTheKeys::load(&copy.root);
    let compiled = compiled_keys();
    let mut differing_between_the_builds: Vec<String> = Vec::new();
    let mut differing_from_the_compiled: Vec<String> = Vec::new();
    let requests = observer_requests();
    for request in &requests {
        let without_the_build_features = key_of_the_verdict(
            &tree
                .product_code_key_under(request, &FeaturesEnabledByTheBuild::default())
                .verdict,
        );
        let with_the_build_features = key_of_the_verdict(
            &tree
                .product_code_key_under(request, &test_build_features)
                .verdict,
        );
        if without_the_build_features.is_none()
            || with_the_build_features != without_the_build_features
        {
            differing_between_the_builds.push(request.group_name.clone());
        }
        if compiled.get(&request.group_name).copied().flatten() != without_the_build_features {
            differing_from_the_compiled.push(request.group_name.clone());
        }
    }
    println!(
        "OBSERVER_KEYS_UNDER_TWO_BUILDS groups={} differing_between_the_builds={} differing_from_the_compiled={}",
        requests.len(),
        differing_between_the_builds.len(),
        differing_from_the_compiled.len()
    );
    assert!(!requests.is_empty(), "前提：登记表里有观察者");
    assert!(
        differing_between_the_builds.is_empty(),
        "两组依赖包特性下同一把键（算不出键的也算对不上）：{differing_between_the_builds:#?}"
    );
    assert!(
        differing_from_the_compiled.is_empty(),
        "编进测试构建的那一把就是依赖包一个特性都不开的那一把：{differing_from_the_compiled:#?}"
    );
}
