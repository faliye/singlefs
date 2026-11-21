//! 判器引到的被检测产品代码那一组：编进来的那一张表里算得出键，组成清单只有产品包的单元、有产品的恢复；这个二进制里判器判出的
//! 三种结论的版本照它拼；只按一组固定的 cfg 收，带不带 `lookup-work-counters` 的两种构建算出同一把键。

use super::*;

/// 场景：编进这个测试二进制的那一张表里判器引到的被检测产品代码那一组（构建时编进来，不读活树）。
/// 预期：算得出键；组成清单不空，每个单元都落在产品包（角色是「产品」的那五个的目录，在这里逐个写出）里——判器、池级 checker、记录核对器
/// 自己的代码一个单元都不在里面；里面有产品的恢复（核心层 `recovery::recover`）；这个二进制里判崩溃镜像那一把版本就是照这一把拼的。
/// 判别力（`crates/mutations.tsv`）：构建脚本不写这一组，这条红在「算得出键」那一句；这一组改回连入口单元一起留，这条红在「落在产品包里」那一句。
#[test]
fn the_judged_product_code_group_is_keyed_with_product_units_only_and_holds_the_recovery() {
    let product_code = judged_product_code_key()
        .unwrap_or_else(|refusal| panic!("编进来的表里这一组算得出键：{}", refusal.sentence()));
    let product_directories: Vec<String> = [
        "singlefs-format",
        "singlefs-core",
        "singlefs-replay-prefix",
        "singlefs-mount-runtime",
        "singlefs-background-gpu",
    ]
    .iter()
    .map(|package_name| format!(" @ crates/{package_name}/"))
    .collect();
    assert!(
        !product_code.composition().is_empty(),
        "前提：这一组的组成清单不空"
    );
    let units_outside: Vec<&str> = product_code
        .composition()
        .iter()
        .map(|unit| unit.unit.as_str())
        .filter(|unit| {
            !product_directories
                .iter()
                .any(|directory| unit.contains(directory.as_str()))
        })
        .collect();
    assert!(
        units_outside.is_empty(),
        "这一组里有落在产品包之外的单元（共 {} 个）：{units_outside:#?}",
        product_code.composition().len()
    );
    assert!(
        product_code.composition().iter().any(|unit| unit
            .unit
            .contains("::recovery::recover @ crates/singlefs-core/")),
        "这一组里有产品的恢复"
    );
    assert_eq!(
        ConclusionVersion::of_the_crash_image_judgement(),
        ConclusionVersion::of_the_judged_conclusion(
            JudgedConclusion::CrashImageJudgement,
            &JudgedProductCodeOfThisBinary::Keyed {
                fingerprint: *product_code.fingerprint()
            }
        ),
        "判崩溃镜像那一把版本照编进来的这一把拼"
    );
}

/// 场景：这个测试二进制是验证包的测试构建（dev 依赖替核心层并上 `lookup-work-counters`，编进去的表的 `build-feature` 行有它）。在临时拷贝上
/// 照构建脚本那一套把判器引到的被检测产品代码那一组算两遍：一遍依赖包一个特性都不当开着（不编测试、只编验证包的那一种构建看到的），
/// 一遍照这一次测试构建实际并上来的那几个。
/// 预期：前提——开着计数的那一遍比不开计数的那一遍组成清单多出单元，多出来的都落在核心层只给测试计数的 `transaction::lookup_work` 里（两组特性真收得出两种闭包；
/// 多出几个打进 `JUDGED_PRODUCT_CODE_UNITS_ONLY_UNDER_THE_TEST_BUILD` 那一行）；编进这个测试二进制的那一把就是不开计数的那一遍那一把——
/// 两种构建算出同一把键，同一批块不判两遍。
/// 判别力（`crates/mutations.tsv`「判器引到的被检测产品代码那一组照这一次编译的依赖包特性收」）：构建脚本那一组改照这一次编译里依赖包并上来的特性收，
/// 编进去的成了开着计数的那一遍那一把，这条红在「两种构建算出同一把键」那一句。
#[test]
fn the_judged_product_code_key_is_the_same_whether_the_build_turns_on_the_lookup_work_counters() {
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
    let copy = RepositoryCopy::new("judged-product-code-under-two-builds");
    copy.assert_the_watched_files_are_the_ones_the_build_read();
    let tree = TreeLoadedForTheKeys::load(&copy.root);
    let request = judged_product_code_request();
    let without_the_counters =
        tree.product_code_key_under(&request, &FeaturesEnabledByTheBuild::default());
    let with_the_counters = tree.product_code_key_under(&request, &test_build_features);
    let units_only_under_the_test_build: Vec<&String> = with_the_counters
        .composition_lines
        .iter()
        .filter(|line| !without_the_counters.composition_lines.contains(line))
        .collect();
    println!(
        "JUDGED_PRODUCT_CODE_UNITS_ONLY_UNDER_THE_TEST_BUILD count={} units_without_the_counters={} units_with_the_counters={}",
        units_only_under_the_test_build.len(),
        without_the_counters.composition_lines.len(),
        with_the_counters.composition_lines.len()
    );
    assert!(
        !units_only_under_the_test_build.is_empty()
            && units_only_under_the_test_build.iter().all(|line| line
                .split('\t')
                .nth(1)
                .is_some_and(|qualified_name| qualified_name
                    .starts_with("singlefs_core::transaction::lookup_work::"))),
        "前提：照测试构建的特性收多出来的单元都在核心层的 transaction::lookup_work 里：{units_only_under_the_test_build:#?}"
    );
    let compiled = compiled_keys()
        .get(JUDGED_PRODUCT_CODE_GROUP_NAME)
        .copied()
        .flatten()
        .expect("编进去的表里判器引到的被检测产品代码那一组算得出键");
    assert_eq!(
        Some(compiled),
        key_of_the_verdict(&without_the_counters.verdict),
        "编进测试构建的那一把就是依赖包一个特性都不开的那一把：两种构建算出同一把键"
    );
    assert_ne!(
        key_of_the_verdict(&with_the_counters.verdict),
        key_of_the_verdict(&without_the_counters.verdict),
        "对照：照测试构建的特性收的那一把是另一把"
    );
}
