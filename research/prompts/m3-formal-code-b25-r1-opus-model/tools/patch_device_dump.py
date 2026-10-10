#!/usr/bin/env python3
"""攻方副本里给 E171 装置加两样只读的探针（不改判法）：
① E171_OPUS_PAIRING_K=<k>：「等价类对拍」那一形截在第 3 次调用的前 k 个写之后（缺省照装置的 2）；
② E171_OPUS_DUMP=1：对拍那一形逐态打一行 OPUSSTATE（类键、落地串、恢复落点、两个崩溃子列、O2 红的不变量），
   并打末段每个写一行 OPUSWRITE（盘、偏移、长度、种类、属于哪次调用）。
用法：patch_device_dump.py <副本根>"""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
s = p.read_text()
a = """            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
            ),
        );
        let pairing = equivalence_pairing_of(&pairing_stream);"""
b = """            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                std::env::var("E171_OPUS_PAIRING_K")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT),
            ),
        );
        let pairing = equivalence_pairing_of(&pairing_stream);"""
assert s.count(a) == 1; s = s.replace(a, b)
a2 = """    EquivalencePairing {
        states: u64::try_from(observations.len()).expect("态数"),
        product_of_the_choices,
        class_states: evaluation.classes.values().map(|class| class.states).sum(),"""
b2 = """    if std::env::var("E171_OPUS_DUMP").is_ok() {
        for write_index in &digits.write_indexes {
            let write = &stream.writes[*write_index];
            println!(
                "OPUSWRITE index={write_index} device={} offset={} length={} kind={} call={:?}",
                write.device.0,
                write.offset.0,
                write.length_in_bytes(),
                write.kind.name(),
                context.call_of_write[*write_index]
            );
        }
        for observation in &observations {
            let key = classification.partition.class_of_ordinal(&digits, observation.ordinal);
            let class_columns = evaluation.classes.get(&key).map(|class| &class.columns);
            println!(
                "OPUSSTATE ordinal={} class={}:{} landing={} consulted_root={:?} crash_judge={} reference_filesystem={} o2={:?} matches_class={} recovery_panic={:?}",
                observation.ordinal,
                key.keyed_assignment,
                key.in_flight_writes_landed.word(),
                observation.landing_vector,
                observation.consulted_root,
                observation.crash_judge.word(),
                observation.reference_filesystem.word(),
                observation.pool_checker_red,
                class_columns == ClassColumns::of_the_observation(observation).as_ref(),
                observation.recovery_panic
            );
        }
    }
    EquivalencePairing {
        states: u64::try_from(observations.len()).expect("态数"),
        product_of_the_choices,
        class_states: evaluation.classes.values().map(|class| class.states).sum(),"""
assert s.count(a2) == 1; s = s.replace(a2, b2)
tmp = pathlib.Path(str(p) + ".tmp"); tmp.write_text(s); tmp.replace(p)
print("patched")
