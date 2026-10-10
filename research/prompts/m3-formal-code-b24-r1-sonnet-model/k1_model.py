#!/usr/bin/env python3
"""K1 小模型（正推腿，m3-formal-code-b24-r1）：把冻结副本 crash_injection.rs 里的真函数与真测试原文抽进一个无依赖的小 crate，
再照 crates/mutations.tsv 的 F3-d 六行（原文要恰好命中一次）与几条额外的改坏逐个改写，逐个跑 F3-c 原用例与 F3-d 三条用例。
用法：k1_model.py <工作目录> <变异名|all> [--stronger]（--stronger：A 段号放到 0..=127、B 前缀 20 个写（段长 32）、C 历史种子 0..128，量加强后的取样放不放得过额外改坏）。变异名：none、row1..row6（tsv 第 2436..2441 行）、x:<名字>（见 EXTRA）。
只写 <工作目录>；不碰仓内文件。"""
import os, re, subprocess, sys

TREE = "/tmp/claude-1000/m3-formal-code-b24-r1/tree"
SRC = f"{TREE}/crates/singlefs-checker-tier/src/crash_injection.rs"
TSV = f"{TREE}/crates/mutations.tsv"
ROW_LINES = {"row1": 2436, "row2": 2437, "row3": 2438, "row4": 2439, "row5": 2440, "row6": 2441}

# 额外的改坏（正推腿自己构造，原文同样要恰好命中一次）：(名字, 原文, 改后)
FOLD = "    persisted_within_the_segment.iter().fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"
SEGMENT = '            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")\n'
SEED = "        seed.0\n            ^ SECOND_CRASH_POINT_SEED_SALT"
EXTRA = {
    "take28": (FOLD, "    persisted_within_the_segment.iter().take(28).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"),
    "take29": (FOLD, "    persisted_within_the_segment.iter().take(29).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"),
    "take24": (FOLD, "    persisted_within_the_segment.iter().take(24).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"),
    "take13": (FOLD, "    persisted_within_the_segment.iter().take(13).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"),
    # 只折段末 12 / 16 个写（b23 判决 K1 行里正推腿点名的「末 12 个写窗口」）
    "last12-window": (FOLD, "    persisted_within_the_segment.iter().skip(persisted_within_the_segment.len().saturating_sub(12)).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"),
    "last16-window": (FOLD, "    persisted_within_the_segment.iter().skip(persisted_within_the_segment.len().saturating_sub(16)).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,"),
    "segment-and-63": (SEGMENT, '            ^ (u64::try_from(crash_point.segment_index).expect("段号装得进 u64") & 63)\n'),
    "segment-as-u8": (SEGMENT, '            ^ u64::from(u8::try_from(crash_point.segment_index % 256).expect("段号装得进 u8"))\n'),
    "segment-and-255": (SEGMENT, '            ^ (u64::try_from(crash_point.segment_index).expect("段号装得进 u64") & 255)\n'),
    "history-seed-and-1": (SEED, "        (seed.0 & 1)\n            ^ SECOND_CRASH_POINT_SEED_SALT"),
    "history-seed-and-63": (SEED, "        (seed.0 & 63)\n            ^ SECOND_CRASH_POINT_SEED_SALT"),
    "drop-salt": (SEED, "        seed.0"),
    "or-segment-only": (SEGMENT, '            | u64::try_from(crash_point.segment_index).expect("段号装得进 u64")\n'),
}

def rows():
    out = {}
    with open(TSV, encoding="utf8") as f:
        for n, line in enumerate(f, 1):
            for name, ln in ROW_LINES.items():
                if n == ln:
                    cols = line.rstrip("\n").split("\t")
                    assert len(cols) == 6, (n, len(cols))
                    out[name] = (cols[2].replace("\\n", "\n"), cols[3].replace("\\n", "\n"), cols[0])
    return out

STUBS = '''#![allow(dead_code)]
// 桩：只给被抽进来的原文用到的类型（形状照 harness 的 HistorySeed、StepPosition、HistoryOperationKind 与 crash_injection 的 CrashPoint）。
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HistorySeed(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StepPosition { StartingPoint, Operation(usize) }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HistoryOperationKind { PublishOverwrite }
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrashPoint {
    pub segment_index: usize,
    pub persisted_within_the_segment: Vec<bool>,
    pub step: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
}
'''

def between(text, start_marker, end_marker):
    a = text.index(start_marker)
    b = text.index(end_marker, a)
    return text[a:b]

STRONGER = [
    ("0..=63;\n        let seeds: Vec<SecondCrashPointSeed> = SEGMENT_INDEXES", "0..=127;\n        let seeds: Vec<SecondCrashPointSeed> = SEGMENT_INDEXES"),
    ("const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;", "const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 20;"),
    ("const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];", "let history_seeds: Vec<HistorySeed> = (0..128).map(HistorySeed).collect();"),
    ("        let seeds: Vec<SecondCrashPointSeed> = HISTORY_SEEDS\n            .into_iter()", "        let seeds: Vec<SecondCrashPointSeed> = history_seeds\n            .iter().copied()"),
    ("            HISTORY_SEEDS.len() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,", "            128 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,"),
]

def build(workdir, variant, stronger=False):
    text = open(SRC, encoding="utf8").read()
    if stronger:
        for old, new in STRONGER:
            assert text.count(old) == 1, f"stronger: 原文命中 {text.count(old)} 次：{old[:60]!r}"
            text = text.replace(old, new)
    if variant != "none":
        if variant in ROW_LINES:
            orig, mut, _ = rows()[variant]
        else:
            orig, mut = EXTRA[variant[2:]]
        assert text.count(orig) == 1, f"{variant}: 原文命中 {text.count(orig)} 次"
        text = text.replace(orig, mut)
    consts = "\n".join(
        l for l in text.splitlines()
        if l.startswith("const SECOND_CRASH_POINT_SEED_SALT") or l.startswith("const PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS")
        or l.startswith("const PERSISTED_SUBSET_FINGERPRINT_PRIME")
    )
    funcs = between(text, "fn fingerprint_of_the_persisted_subset", "/// 挂载的每一段各摆一个二次崩溃")
    # 测试原文：F3-c 那一条起到「实例表清扫两段」之前（含 F3-d 的常数、共用搭建与三条用例）
    tests = between(text, "    #[test]\n    fn all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently",
                    "    /// 实例表清扫两段")
    lib = STUBS + consts + "\n" + funcs + "\n#[cfg(test)]\nmod tests {\n    use super::*;\n" + tests + "\n}\n"
    d = f"{workdir}/{variant.replace(':','_')}"
    os.makedirs(f"{d}/src", exist_ok=True)
    open(f"{d}/Cargo.toml", "w").write(f'[package]\nname = "k1model"\nversion = "0.1.0"\nedition = "2021"\n[workspace]\n')
    open(f"{d}/src/lib.rs", "w", encoding="utf8").write(lib)
    return d

if __name__ == "__main__":
    workdir, which = sys.argv[1], sys.argv[2]
    stronger = len(sys.argv) > 3 and sys.argv[3] == "--stronger"
    names = (["none"] + list(ROW_LINES) + [f"x:{n}" for n in EXTRA]) if which == "all" else [which]
    for v in names:
        print(build(workdir, v, stronger))
