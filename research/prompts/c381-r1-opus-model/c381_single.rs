#[path = "c381_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

#[test]
fn single() {
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let arms: Vec<u8> = std::env::var("C381_ARMS").map(|v| v.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or(vec![0, 1, 2, 3]);
    let k: Option<u64> = std::env::var("C381_K").ok().and_then(|v| v.parse().ok());
    let later: Vec<u64> = std::env::var("C381_LATER").ok().filter(|v| !v.is_empty()).map(|v| v.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or_default();
    let crash: Option<u64> = std::env::var("C381_CRASH").ok().and_then(|v| v.parse().ok());
    let lying = std::env::var("C381_LYING").is_ok();
    let actions: Vec<Action> = std::env::var("C381_ACTIONS").unwrap_or_default().split(',').filter(|s| !s.is_empty()).map(|s| match s {
        "Retry" => Action::Retry, "Empty" => Action::Empty, "Other" => Action::Other, "EmptyFromB" => Action::EmptyFromB, "Remount" => Action::Remount, _ => panic!("{s}"),
    }).collect();
    for arm in arms {
        LYING.with(|flag| flag.set(lying));
        let outcome = run(&parameters, &snapshot, arm, k, &later, &actions, crash);
        println!("[{}] k={k:?} later={later:?} actions={actions:?} crash={crash:?} lying={lying}\n  steps={:?} writes_after_c={} writes={}\n  checker@crash={:?}\n  remount={}\n  checker@after={:?}", arm_name(arm), outcome.steps, outcome.writes_after_c, outcome.writes, outcome.checker_at_crash, outcome.remount, outcome.checker_after_remount);
    }
}
