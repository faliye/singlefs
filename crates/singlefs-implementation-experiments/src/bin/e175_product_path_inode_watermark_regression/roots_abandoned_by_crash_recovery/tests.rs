//! `roots_abandoned_by_crash_recovery` 的单测。
use super::super::tests::breakage_of_this_process;
use super::*;

/// V5 的靶：持有旧出生代的根已不在可读根里 ⇒ 已转出；一条可读、不被抛弃、在 F 之上 ⇒ 还在。
#[test]
fn older_birth_candidacy_says_gone_when_no_holder_is_readable() {
    let breakage = breakage_of_this_process();
    let gone = [HolderAtThisStep {
        readable_now: false,
        abandoned_now: false,
        txg: 9,
    }];
    assert_eq!(
        classify_older_birth_candidacy(&gone, 0, breakage),
        OlderBirthStillInTheCandidateSet::Gone
    );
    let below_the_floor = [HolderAtThisStep {
        readable_now: true,
        abandoned_now: false,
        txg: 4,
    }];
    assert_eq!(
        classify_older_birth_candidacy(&below_the_floor, 5, breakage),
        OlderBirthStillInTheCandidateSet::Gone
    );
    let still = [HolderAtThisStep {
        readable_now: true,
        abandoned_now: false,
        txg: 5,
    }];
    assert_eq!(
        classify_older_birth_candidacy(&still, 5, breakage),
        OlderBirthStillInTheCandidateSet::Still
    );
}
