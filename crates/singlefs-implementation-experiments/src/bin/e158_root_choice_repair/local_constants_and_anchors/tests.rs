use super::*;

/// M6（变异表）的反面对照：本地常量与 `crates::` 的默认值今天必须全部相等（回归锚，
/// `crates/` 改了这些常量而没同步登记时，这条测试先红）。
#[test]
fn local_constants_match_crates_today() {
    for check in local_constants_checks() {
        assert!(
            check.matches(),
            "{} 本地={} crates={} 不等",
            check.name,
            check.local_value,
            check.crates_value
        );
    }
    let (region_ok, local, crates_value) = region_devices_matches_crates();
    assert!(
        region_ok,
        "三个区域住哪块盘：本地={local:?} crates={crates_value:?} 不等"
    );
}

/// 5.7 常量回比 + 第七节 7.2 算术全过（M6/M7 变异的红测：改坏本地常量或算术字面量，这条测试先红）。
#[test]
fn constants_and_anchors_all_pass() {
    assert!(
        run_constants_and_anchors(),
        "run_constants_and_anchors 应当全 PASS（本地常量与 crates:: 同名常量、第七节 7.2 算术都对得上）"
    );
}

/// PC2：环境变量没设，本地常量按今天那一份取 489（D22（单元原子性怎么合成） 已定项 9 的字段表合计）。
#[test]
fn parse_local_system_configuration_bytes_defaults_to_489_when_unset() {
    let value = parse_local_system_configuration_bytes(Err(env::VarError::NotPresent));
    assert_eq!(value, 489, "没设环境变量应当取今天的 489");
}

/// PC2：环境变量给了十进制数字，原样解出来。取一个与没设时的默认 489 不同的数，
/// 才分得出是解出来的，还是退回了默认值。
#[test]
fn parse_local_system_configuration_bytes_parses_the_given_decimal() {
    let value = parse_local_system_configuration_bytes(Ok("497".to_string()));
    assert_eq!(value, 497, "环境变量给的十进制数要原样解出来");
}

/// PC2：环境变量给了不是十进制数的内容，panic（不许静默退回默认值掩盖配错）。
#[test]
#[should_panic(expected = "不是十进制正整数")]
fn parse_local_system_configuration_bytes_panics_on_non_decimal_value() {
    let _ = parse_local_system_configuration_bytes(Ok("not-a-number".to_string()));
}
