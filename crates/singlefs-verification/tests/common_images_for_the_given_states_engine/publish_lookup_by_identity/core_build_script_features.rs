//! `singlefs-core` 构建脚本交出的特性只认 cargo 这一次真开的：特性真开着时报开着，`RUSTFLAGS` 带进来的与它自己的配置不符时停下。

use super::*;

/// cargo 真开着 `lookup-work-counters` 编 `singlefs-core` 时（这个测试包经 dev 依赖打开它，计数的几处才编得进来），它的构建脚本交给依赖方的
/// 特性里有它（「开着要报开着」那一半：外带变量那一条用例把构建脚本编进这个测试包，那里的 cfg 恒为假，
/// 只钉得住「没开别报开着」）。读的是构建脚本经 `cargo::rustc-env` 编进 `singlefs-core` 的那一份，与它打给依赖方的 `cargo::metadata` 同一个值。
/// 判别力：`crates/mutations.tsv` 里点名这一条的那一行（T4b：表里那一格 cfg 写死成假）。
/// 这个测试包的 dev 依赖另打开发布触发阈值的只供测试来源（`test-only-publish-thresholds`，判决 `research/prompts/m4-publish-trigger-r1-main-verification.md`
/// 第二节「只供测试的开关」那一格），交出的是两个，按清单里的名字排好。
pub fn run_the_scenario_where_the_core_build_script_reports_the_lookup_work_counters_feature_that_cargo_enabled_for_this_test_package(
) {
    assert_eq!(
        ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT,
        "LOOKUP_WORK_COUNTERS,TEST_ONLY_PUBLISH_THRESHOLDS",
        "cargo 开着 lookup-work-counters 与 test-only-publish-thresholds 编的 singlefs-core：构建脚本交出的特性里有这两个"
    );
}

/// 构建脚本比 `CARGO_ENCODED_RUSTFLAGS` 里的 `--cfg feature=…` 与自己的 cfg（T4 ①）：
/// 这个测试包里构建脚本那一格 cfg 为假（同外带变量那一条），参数里经 `--cfg feature="lookup-work-counters"`（分两项、写成一项、等号两边带空白）
/// 点名打开它的判成对不上、点名那个特性；没点名它的（空、别的特性、别的参数、`--check-cfg` 里提到它、不跟在 `--cfg` 后面）不判；
/// 表说开着时同样的参数不判。构建时对不上就停（`build.rs` 的 `main`）。
/// 判别力：`crates/mutations.tsv` 里点名这一条的那一行（认不出 `feature` 那个键）。
pub fn run_the_scenario_where_the_core_build_script_finds_the_features_its_rustflags_open_while_its_own_compile_time_configuration_says_off(
) {
    let table = singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS;
    assert_eq!(
        table,
        [
            ("lookup-work-counters", false),
            ("test-only-publish-thresholds", false)
        ],
        "这个测试包里构建脚本那两格 cfg 都为假（编进来的构建脚本读的是这个测试包自己的特性）"
    );
    for encoded_rustflags in [
        "--cfg\u{1f}feature=\"lookup-work-counters\"",
        "--cfg=feature=\"lookup-work-counters\"",
        "-C\u{1f}opt-level=1\u{1f}--cfg\u{1f}feature = \"lookup-work-counters\"",
    ] {
        assert_eq!(
            singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(
                encoded_rustflags,
                &table,
            ),
            ["lookup-work-counters"],
            "{encoded_rustflags:?} 点名打开 lookup-work-counters、构建脚本的 cfg 说没开：对不上"
        );
    }
    for encoded_rustflags in [
        "",
        "--cfg\u{1f}feature=\"gpu\"",
        "-C\u{1f}opt-level=1",
        "--check-cfg\u{1f}cfg(feature, values(\"lookup-work-counters\"))",
        "feature=\"lookup-work-counters\"",
    ] {
        assert!(
            singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(
                encoded_rustflags,
                &table,
            )
            .is_empty(),
            "{encoded_rustflags:?} 没经 --cfg 点名打开表里的特性：不判"
        );
    }
    assert!(
        singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(
            "--cfg\u{1f}feature=\"lookup-work-counters\"",
            &[("lookup-work-counters", true)],
        )
        .is_empty(),
        "cfg 说开着、参数也点名打开：对得上"
    );
}

/// 子进程标记：设着时这条用例是父进程起的那一趟（进程环境里带着外面设的那个特性变量）。
const CHILD_RUN_MARKER_OF_THE_BUILD_SCRIPT_CHECK: &str =
    "SINGLEFS_HARNESS_CORE_BUILD_SCRIPT_CHECK_CHILD_RUN";

/// cargo 给构建脚本设的、表示 `lookup-work-counters` 开着的那个环境变量名：这里从外面带进来、冒充开着。
const VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE: &str = "CARGO_FEATURE_LOOKUP_WORK_COUNTERS";

/// 下面那条用例的名字（子进程按它 `--exact` 只跑这一条）。
const NAME_OF_THE_BUILD_SCRIPT_CHECK: &str =
    "the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside";

/// `singlefs-core` 清单 `[features]` 里的特性名（`name = …` 那几行等号左边，不含注释行），按名字排好。
fn feature_names_of_the_core_manifest() -> Vec<String> {
    let manifest_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../singlefs-core/Cargo.toml"
    ))
    .expect("读得了核心层清单");
    let mut names: Vec<String> = manifest_text
        .lines()
        .skip_while(|line| line.trim() != "[features]")
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| {
            line.split_once('=')
                .map(|(name, _)| name.trim().to_string())
        })
        .collect();
    names.sort();
    names
}

/// `singlefs-core` 构建脚本交出的特性只认 cargo 这一次真开的：
/// - 进程环境里外带 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1`、特性不开（这个测试包没有这个特性，编进来的构建脚本里那一行 cfg 为假）：交出空串。
///   变量要真在进程环境里才判得出「不读环境」：在子进程里跑本测试二进制的这一条（带那个变量与一个标记），父进程判子进程绿、恰好跑了一条；
/// - cfg 说开着时交出 `LOOKUP_WORK_COUNTERS`（依赖方 `singlefs_unit_graph::build_features` 按这个写法读回清单里的 `lookup-work-counters`）；
/// - 构建脚本里的特性表与它清单 `[features]` 逐个相同（漏写一个，依赖方就当它没开）。
///
/// 判别力：`crates/mutations.tsv` 里「七·6 后续批 构建脚本：」起头的那一行（交出前把进程环境里设着的 `CARGO_FEATURE_*` 也算开着）。
pub fn run_the_scenario_where_the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside(
) {
    let reported_with_the_feature_off = singlefs_core_build_script::enabled_features_metadata_value(
        &singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS,
    );
    if std::env::var_os(CHILD_RUN_MARKER_OF_THE_BUILD_SCRIPT_CHECK).is_some() {
        assert!(
            std::env::var_os(VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE).is_some(),
            "子进程里外带的 {VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE} 设着"
        );
        assert_eq!(
            reported_with_the_feature_off, "",
            "环境里外带 {VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE}=1、特性没开：构建脚本不报开着"
        );
        return;
    }
    let child = std::process::Command::new(std::env::current_exe().expect("测试二进制自己的路径"))
        .args([
            "--exact",
            NAME_OF_THE_BUILD_SCRIPT_CHECK,
            "--test-threads",
            "1",
            "--nocapture",
        ])
        .env(VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE, "1")
        .env(CHILD_RUN_MARKER_OF_THE_BUILD_SCRIPT_CHECK, "1")
        .output()
        .expect("起得了子进程");
    let child_output = format!(
        "{}{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(
        child.status.success() && child_output.contains("test result: ok. 1 passed"),
        "外带 {VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE}=1 的子进程里这一条绿、恰好跑了一条（退出 {:?}）：{child_output}",
        child.status.code()
    );
    assert_eq!(
        reported_with_the_feature_off, "",
        "不外带变量、特性没开：构建脚本不报开着"
    );
    assert_eq!(
        singlefs_core_build_script::enabled_features_metadata_value(&[(
            "lookup-work-counters",
            true
        )]),
        "LOOKUP_WORK_COUNTERS",
        "cfg 说开着：交出 cargo 环境变量名的写法"
    );
    let mut names_in_the_table: Vec<String> =
        singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS
            .iter()
            .map(|(feature, _)| (*feature).to_string())
            .collect();
    names_in_the_table.sort();
    assert_eq!(
        names_in_the_table,
        feature_names_of_the_core_manifest(),
        "构建脚本的特性表与 singlefs-core 清单 [features] 逐个相同"
    );
}
