#!/usr/bin/env python3
"""第三处副本改动（攻方改法 F3 的原型，零轮）：B18_VARIANT=s3_decorrelate 时二次崩溃的种子再混进第一次崩溃那个子集，
同一段里不同的第一次崩溃态各抽各的。"""
import sys, pathlib
F = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker-tier/src/crash_injection.rs"
t = F.read_text()
old = """    let mut source = SeededRandomSource::from_seed(
        seed.0
            ^ SECOND_CRASH_POINT_SEED_SALT
            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64"),
    );"""
new = """    let subset_mix = if b18_variant("s3_decorrelate") {
        crash_point
            .persisted_within_the_segment
            .iter()
            .fold(0x51_7c_c1_b7_27_22_0a_95u64, |acc, bit| {
                (acc ^ u64::from(*bit)).wrapping_mul(0x0000_0100_0000_01b3).rotate_left(17)
            })
    } else {
        0
    };
    let mut source = SeededRandomSource::from_seed(
        seed.0
            ^ SECOND_CRASH_POINT_SEED_SALT
            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
            ^ subset_mix,
    );"""
assert t.count(old) == 1
F.write_text(t.replace(old, new))
print("patched 3")
