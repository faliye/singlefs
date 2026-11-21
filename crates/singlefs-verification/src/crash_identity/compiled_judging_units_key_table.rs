//! 编进二进制的判法单元键表与按组取键的入口：核心层编进来的特性、组成清单行，本包的表与别的包的表（观察者引到的被检测产品代码），
//! 判器引到的被检测产品代码那一组的键。

use super::*;

/// 构建脚本写进 `OUT_DIR` 的那份判法单元键表（`singlefs_unit_graph::entry_groups` 的写法，文件名与 `build.rs` 的
/// `JUDGING_UNITS_KEYS_FILE_NAME` 相同）。
const COMPILED_JUDGING_UNITS_KEYS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/judging_units_keys.tsv"));

/// 编进二进制的那份判法单元键表，读回一次。
///
/// # Panics
/// 表认不回来：它是构建脚本用 `entry_group_table_text` 写的，认不回来是两边写法不一致的 bug。
#[must_use]
pub fn compiled_judging_units_keys() -> &'static ReadBackEntryGroupTable {
    static TABLE: std::sync::OnceLock<ReadBackEntryGroupTable> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        parse_entry_group_table(COMPILED_JUDGING_UNITS_KEYS_TEXT).unwrap_or_else(|reason| {
            panic!("构建脚本写进二进制的判法单元键表认不回来（两边写法不一致）：{reason}")
        })
    })
}

/// 核心层的包名（清单 `[package] name`，编进去的那份表 `build-feature` 行里点名它的写法）。
pub const CORE_LAYER_PACKAGE_NAME: &str = "singlefs-core";

/// 核心层按身份查的计数那个特性（验证包的 dev 依赖打开它）。
pub const LOOKUP_WORK_COUNTERS_FEATURE: &str = "lookup-work-counters";

/// 核心层发布触发阈值的只供测试来源那个特性（验证包的 dev 依赖打开它，判决 `research/prompts/m4-publish-trigger-r1-main-verification.md`
/// 第二节「只供测试的开关」那一格）。
pub const TEST_ONLY_PUBLISH_THRESHOLDS_FEATURE: &str = "test-only-publish-thresholds";

/// 核心层清单 `[features]` 里的特性，[`features_the_core_layer_was_compiled_with`] 认得出开没开的就是这几个（按清单里的名字排好）。
pub const CORE_LAYER_FEATURES_RECOGNIZED_FROM_ITS_CODE: [&str; 2] = [
    LOOKUP_WORK_COUNTERS_FEATURE,
    TEST_ONLY_PUBLISH_THRESHOLDS_FEATURE,
];

/// 这个二进制链进来的核心层实际开着哪些特性，从核心层编出来的代码里认、不经构建脚本：拿来对照编进去的那份表里依赖包的特性
/// （`build-feature` 行，构建脚本读核心层经 `links` 交出的那一份）。只编验证包、不编它的测试时一个都不开；编验证包的测试时，
/// 验证包的 dev 依赖替核心层并上 `lookup-work-counters` 与 `test-only-publish-thresholds`。一个都没开时交空表（与构建脚本取到空集时一样，不留核心层那一项）。
#[must_use]
pub fn features_the_core_layer_was_compiled_with() -> FeaturesEnabledByTheBuild {
    let mut features_of_each_package = BTreeMap::new();
    let mut features_of_the_core_layer = BTreeSet::new();
    if core_layer_feature_probe::lookup_work_counters_are_compiled_into_the_core_layer() {
        features_of_the_core_layer.insert(LOOKUP_WORK_COUNTERS_FEATURE.to_string());
    }
    if core_layer_publish_thresholds_probe::test_only_publish_thresholds_are_compiled_into_the_core_layer()
    {
        features_of_the_core_layer.insert(TEST_ONLY_PUBLISH_THRESHOLDS_FEATURE.to_string());
    }
    if !features_of_the_core_layer.is_empty() {
        features_of_each_package.insert(
            CORE_LAYER_PACKAGE_NAME.to_string(),
            features_of_the_core_layer,
        );
    }
    FeaturesEnabledByTheBuild {
        features_of_each_package,
    }
}

/// 认核心层开没开 `lookup-work-counters`：开着时核心层才有 `transaction::lookup_work::lookup_work_while`。函数体里 glob 引入那个模块，
/// 名字在那里找得到就用核心层的；找不到，落到本模块里同名的替身上。再按交回的计数是不是替身的类型认是哪一个。
/// 核心层把那个函数改名、或改成不挂特性，这里认错的那一边会让对照它的测试红，不会静默。
mod core_layer_feature_probe {
    /// 替身交回的计数（核心层没编进计数时才构造）。
    #[allow(
        dead_code,
        reason = "核心层开着计数时名字落到核心层那一个上，替身不构造这个值"
    )]
    pub(super) struct CountersNotCompiledIntoTheCoreLayer;

    /// 核心层没编进计数时，`lookup_work_while` 这个名字落到这里。
    #[allow(
        dead_code,
        reason = "核心层开着计数时名字落到核心层那一个上，替身用不到"
    )]
    pub(super) fn lookup_work_while<Output>(
        action: impl FnOnce() -> Output,
    ) -> (Output, CountersNotCompiledIntoTheCoreLayer) {
        (action(), CountersNotCompiledIntoTheCoreLayer)
    }

    pub(super) fn lookup_work_counters_are_compiled_into_the_core_layer() -> bool {
        #[allow(
            unused_imports,
            reason = "核心层没编进计数时这个模块里没有这里用得上的名字"
        )]
        use singlefs_core::transaction::lookup_work::*;
        let ((), work_while_doing_nothing) = lookup_work_while(|| ());
        !is_the_lookup_work_counters_stand_in(&work_while_doing_nothing)
    }

    fn is_the_lookup_work_counters_stand_in<Work: 'static>(_work: &Work) -> bool {
        std::any::TypeId::of::<Work>()
            == std::any::TypeId::of::<CountersNotCompiledIntoTheCoreLayer>()
    }
}

/// 认核心层开没开 `test-only-publish-thresholds`：开着时核心层的 `PublishTriggerThresholds` 才有固有的
/// `of_test_only_values_clamped_by_the_journal_ring`。按类型名调它：固有的那个先于本模块里同名的特征方法（替身），没有固有的才落到替身上；
/// 再按交回的值是不是替身的类型认是哪一个。核心层把那个函数改名、或改成不挂特性，这里认错的那一边会让对照它的测试红，不会静默。
mod core_layer_publish_thresholds_probe {
    use singlefs_core::system_configuration::PublishTriggerThresholds;

    /// 替身交回的值（核心层没编进只供测试来源时才构造）。
    #[allow(
        dead_code,
        reason = "核心层开着那个特性时名字落到核心层固有的那一个上，替身不构造这个值"
    )]
    pub(super) struct ThresholdsNotCompiledIntoTheCoreLayer;

    /// 核心层没编进只供测试来源时，`PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring` 这个名字落到这里。
    #[allow(
        dead_code,
        reason = "核心层开着那个特性时名字落到核心层固有的那一个上，替身用不到"
    )]
    trait StandInForTheTestOnlyPublishThresholds {
        fn of_test_only_values_clamped_by_the_journal_ring(
            time_threshold: std::time::Duration,
            dirty_threshold_before_the_clamp: u64,
            journal_ring_bytes: u64,
        ) -> ThresholdsNotCompiledIntoTheCoreLayer;
    }

    impl StandInForTheTestOnlyPublishThresholds for PublishTriggerThresholds {
        fn of_test_only_values_clamped_by_the_journal_ring(
            _time_threshold: std::time::Duration,
            _dirty_threshold_before_the_clamp: u64,
            _journal_ring_bytes: u64,
        ) -> ThresholdsNotCompiledIntoTheCoreLayer {
            ThresholdsNotCompiledIntoTheCoreLayer
        }
    }

    pub(super) fn test_only_publish_thresholds_are_compiled_into_the_core_layer() -> bool {
        let made = PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring(
            std::time::Duration::ZERO,
            0,
            0,
        );
        !is_the_publish_thresholds_stand_in(&made)
    }

    fn is_the_publish_thresholds_stand_in<Made: 'static>(_made: &Made) -> bool {
        std::any::TypeId::of::<Made>()
            == std::any::TypeId::of::<ThresholdsNotCompiledIntoTheCoreLayer>()
    }
}

/// 表里一组的组成清单换成组成清单行：一个单元一行，单元写 `<种类> <限定名> @ <文件>`（不带行号：行挪了不算单元变了），
/// 同一份文件里种类与限定名都相同的几个单元按出现次序另加 `#<第几个>`。
fn composition_of_the_compiled_group(group: &ReadBackEntryGroup) -> Vec<JudgingUnitFingerprint> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    group
        .composition
        .iter()
        .map(|unit| {
            let file = unit
                .location
                .rsplit_once(':')
                .map_or(unit.location.as_str(), |(file, _lines)| file);
            let base = format!("{} {} @ {file}", unit.kind, unit.qualified_name);
            let occurrence = seen.entry(base.clone()).or_insert(0);
            *occurrence += 1;
            let name = if *occurrence == 1 {
                base
            } else {
                format!("{base} #{occurrence}")
            };
            JudgingUnitFingerprint {
                unit: name,
                fingerprint: unit.fingerprint,
            }
        })
        .collect()
}

/// 进键的判法单元的唯一入口：观察者与判器引到的被检测产品代码都经它取。读编进二进制的那份表（构建脚本按入口登记表逐组算的）里
/// 这一组的键与组成清单，跑时不读盘上的树，也不收仓根（仓根不是键的输入）：`toolchain` 不用（工具链不进键），
/// 签名给 `crash_library`、`crash_state_enumeration` 那几处调用方照一种写法调。
///
/// # Errors
/// 拒算（[`JudgingUnitsKeyRefused`]）：表里没有这一组、这一组在这一趟的特性下不编进来、编译时闭包里有认不出的形态。调用方照判、不入库，计入拒算。
pub fn judging_units_key(
    units: &JudgingUnits,
    _toolchain: &CrashPointReplayToolchainIdentity,
) -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    judging_units_key_in_the_table(
        compiled_judging_units_keys(),
        units,
        where_to_register_in_the_verification_package(units),
    )
}

/// 验证包这一张表里没有这一组时，红句里写它该登在哪：观察者登 `OBSERVER_ENTRIES`，
/// 判器引到的被检测产品代码那一组由构建脚本照 `JUDGED_PRODUCT_CODE_ENTRIES` 写。
fn where_to_register_in_the_verification_package(units: &JudgingUnits) -> &'static str {
    match units {
        JudgingUnits::ObserverCode { .. } => {
            "观察者要在 crate::judging_units_entries 的 OBSERVER_ENTRIES 登记它的入口 fn"
        }
        JudgingUnits::JudgedProductCode => {
            "判器引到的被检测产品代码那一组由验证包的构建脚本照 crate::judging_units_entries 的 JUDGED_PRODUCT_CODE_ENTRIES 写进表"
        }
    }
}

/// 判器引到的被检测产品代码那一组（[`JudgingUnits::JudgedProductCode`]）在编进二进制的那份表里的键：判器判出的三种结论的版本
/// （[`ConclusionVersion::of_the_crash_image_judgement`] 一族）与对拍结论的键拼它。
///
/// # Errors
/// 表里没有这一组、这一组没编进来、编译时它的闭包里有认不出的形态：调用方照判、不入库（结论版本取这个进程的占位）。
pub fn judged_product_code_key() -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    judging_units_key_in_the_table(
        compiled_judging_units_keys(),
        &JudgingUnits::JudgedProductCode,
        where_to_register_in_the_verification_package(&JudgingUnits::JudgedProductCode),
    )
}

/// 一个观察者（[`JudgingUnits::ObserverCode`]）引到的被检测产品代码的键，在 `observer_keys` 点名的那一张表里取：验证包那一张的观察者组
/// 只留产品包的单元（观察者自己的代码不进，[`crate::crash_amplification::ObserverVersions`]）；别的包那一张照那个包的构建脚本算的那一把。
///
/// # Errors
/// 点名的那一张表里没有这一组（没登记）、这一组没编进来、编译时它的闭包里有认不出的形态：调用方照判、不入库。
///
/// # Panics
/// 别的包那一张表认不回来：它是那个包的构建脚本用 `entry_group_table_text_with_the_features_of_the_build` 写的，认不回来是两边写法不一致的 bug。
pub fn observer_product_code_key_in(
    units: &JudgingUnits,
    observer_keys: ObserverJudgingUnitsKeyTable,
) -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    match observer_keys {
        ObserverJudgingUnitsKeyTable::OfTheVerificationPackage => judging_units_key_in_the_table(
            compiled_judging_units_keys(),
            units,
            where_to_register_in_the_verification_package(units),
        ),
        ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage { table_text } => {
            let table = parse_entry_group_table(table_text).unwrap_or_else(|reason| {
                panic!(
                    "别的包的构建脚本写进二进制的判法单元键表认不回来（两边写法不一致）：{reason}"
                )
            });
            judging_units_key_in_the_table(
                &table,
                units,
                "观察者要在点名这张键表的那个包自己的观察者登记表（实验包的 judging_units_entries::OBSERVER_ENTRIES）登记它的入口 fn",
            )
        }
    }
}

/// 观察者登在哪一张键表里（[`observer_product_code_key_in`] 照它取键）：挂观察者的那一方在崩溃枚举的入口点名
/// （`crate::crash_state_enumeration::enumerate_crash_states_with_expansion_keying_the_observers_in`）。两张表的写法、组名（[`JudgingUnits::group_name`]）
/// 都是同一套（`singlefs_unit_graph::entry_groups`），只差观察者的入口登在哪个包、由哪个包的构建脚本写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverJudgingUnitsKeyTable {
    /// 验证包构建脚本编进本包的那一张（[`compiled_judging_units_keys`]）：观察者登在 `crate::judging_units_entries::OBSERVER_ENTRIES`。
    OfTheVerificationPackage,
    /// 别的包（实验包）的构建脚本照同一套写进它自己 `OUT_DIR`、编进它二进制的那一张的原文：观察者登在那个包自己的观察者登记表，
    /// 观察者代码文件在那个包里。
    CompiledIntoAnotherPackage { table_text: &'static str },
}

/// 一组判法单元的键，观察者那几组从 `observer_keys` 点名的那一张键表取，别的组照 [`judging_units_key`] 取编进本包的那一张
/// （实验包的测试照它核自己登的观察者都编进了那一张）。
///
/// # Errors
/// 同 [`judging_units_key`]：点名的那一张表里没有这个观察者那一组（没在那个包的观察者登记表登记）、它在那一趟的特性下不编进来、
/// 编译时它的闭包里有认不出的形态。
///
/// # Panics
/// 别的包那一张表认不回来（同 [`observer_product_code_key_in`]）。
pub fn judging_units_key_with_the_observers_keyed_in(
    units: &JudgingUnits,
    observer_keys: ObserverJudgingUnitsKeyTable,
    toolchain: &CrashPointReplayToolchainIdentity,
) -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    match units {
        JudgingUnits::ObserverCode { .. } => observer_product_code_key_in(units, observer_keys),
        JudgingUnits::JudgedProductCode => judging_units_key(units, toolchain),
    }
}

/// 在一张读回的键表里取这一组的键与组成清单（[`judging_units_key`] 与 [`judging_units_key_with_the_observers_keyed_in`] 共用）；
/// 表里没有这一组时拒算的那一句接上 `where_to_register`（这一张表的观察者登在哪）。
fn judging_units_key_in_the_table(
    table: &ReadBackEntryGroupTable,
    units: &JudgingUnits,
    where_to_register: &str,
) -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    let group_name = units.group_name();
    let Some(group) = table
        .groups
        .iter()
        .find(|group| group.group_name == group_name)
    else {
        return Err(JudgingUnitsKeyRefused::NotComputedAtTheBuild {
            units: units.clone(),
            reason: format!("编译时的判法单元键表里没有组 {group_name}（{where_to_register}）"),
        });
    };
    match &group.verdict {
        ReadBackEntryGroupVerdict::Keyed { key } => Ok(JudgingUnitsKey::of_the_parts(
            *key,
            composition_of_the_compiled_group(group),
        )),
        ReadBackEntryGroupVerdict::Refused { refusal_lines } => {
            Err(JudgingUnitsKeyRefused::RefusedAtTheBuild {
                units: units.clone(),
                refusal_lines: refusal_lines.clone(),
            })
        }
        ReadBackEntryGroupVerdict::NotCompiled { reason } => {
            Err(JudgingUnitsKeyRefused::NotComputedAtTheBuild {
                units: units.clone(),
                reason: reason.clone(),
            })
        }
    }
}

/// 别的包的构建脚本照同一套（`singlefs-unit-graph` 的 `entry_group_table_text_with_the_features_of_the_build`）写下的键表里点名一组的判法单元键：
/// 认表、取指纹、拼组成清单都是 [`judging_units_key`] 那一份代码。产品 GPU 包的测试包与实验包的 E185（GC一批搬迁的挂钟拆分） 装置拿它取
/// CPU 参照那一侧的单元键，拼进对拍参照的录入键（`crate::crash_library::pairing_input_and_reference_through_the_pairing_library`）。
///
/// # Errors
/// 表认不回来；表里没有这一组；这一组拒算或没编进来。交回那一句（调用方当拒算处置：照录、不复用）。
pub fn judging_units_key_of_the_group_in_the_table_text(
    table_text: &str,
    group_name: &str,
) -> Result<JudgingUnitsKey, String> {
    let table = parse_entry_group_table(table_text)
        .map_err(|reason| format!("判法单元键表认不回来：{reason}"))?;
    let group = table
        .groups
        .iter()
        .find(|group_of_the_table| group_of_the_table.group_name == group_name)
        .ok_or_else(|| format!("判法单元键表里没有组 {group_name}"))?;
    match &group.verdict {
        ReadBackEntryGroupVerdict::Keyed { key } => Ok(JudgingUnitsKey::of_the_parts(
            *key,
            composition_of_the_compiled_group(group),
        )),
        ReadBackEntryGroupVerdict::Refused { refusal_lines } => Err(format!(
            "组 {group_name} 的判法单元拒算：{}",
            refusal_lines.join("；")
        )),
        ReadBackEntryGroupVerdict::NotCompiled { reason } => {
            Err(format!("组 {group_name} 这一次没编进来：{reason}"))
        }
    }
}

#[cfg(test)]
mod tests;
