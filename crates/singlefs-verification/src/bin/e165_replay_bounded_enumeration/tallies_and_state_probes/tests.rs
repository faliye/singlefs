use super::*;

/// 逐态判定行里「这个状态让哪几个量加了一」：照 `Counter::ALL` 的次序，加了不止一的带次数，一个都没加写 none。
#[test]
fn the_counters_a_state_added_are_listed_in_order_with_their_multiplicity() {
    let before = vec![0u64; Counter::ALL.len()];
    let mut after = before.clone();
    assert_eq!(
        labels_of_the_counters_a_state_added(&before, &after),
        "none"
    );
    after[Counter::FarthestVersionViolations.index()] += 1;
    after[Counter::States.index()] += 1;
    after[Counter::Failed.index()] += 2;
    assert_eq!(
        labels_of_the_counters_a_state_added(&before, &after),
        "states,failed*2,J4"
    );
}

/// 单状态重放的参数：三段是重放一个状态，两段是逐态列；段数不对、域不认识、N 小于这个域最小的 N 都拒。
#[test]
fn the_state_probe_argument_names_a_domain_a_size_and_an_ordinal() {
    let arguments = |option: &str, value: &str| vec![option.to_string(), value.to_string()];
    assert_eq!(
        parse_the_state_probe(&arguments("--replay-state", "U_verify_off:2:17")),
        Ok(Some((
            Domain::NamedUnitsNotVerified,
            2,
            StateProbe::ReplayOneState { ordinal: 17 }
        )))
    );
    assert_eq!(
        parse_the_state_probe(&arguments("--list-states", "AC:3")),
        Ok(Some((
            Domain::AdversarialBackChain,
            3,
            StateProbe::ListEveryState
        )))
    );
    assert_eq!(
        parse_the_state_probe(&arguments("--bounds", "segment1")),
        Ok(None)
    );
    assert!(parse_the_state_probe(&arguments("--replay-state", "L:2")).is_err());
    assert!(parse_the_state_probe(&arguments("--list-states", "L:2:0")).is_err());
    assert!(parse_the_state_probe(&arguments("--replay-state", "U:2:0")).is_err());
    assert!(parse_the_state_probe(&arguments("--replay-state", "X:1:0")).is_err());
}
