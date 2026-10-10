#!/usr/bin/env python3
"""攻方副本里给 overwrite_in_one_instance.rs 的两条 812 用例加打印：末尾那一版与每一版上不是「成立」的不变量（P4 C551 推翻条件那一格）。
用法：patch_c551_probe.py <副本根>。每处旧串必须恰好命中一次，否则退 3。"""
import pathlib
import sys

p = pathlib.Path(sys.argv[1]) / "crates/singlefs-harness/tests/overwrite_in_one_instance.rs"
t = p.read_text()
old = """    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    let violated_invariants: Vec<&(&str, InvariantVerdict)> = verdicts"""
new = """    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    for (name, verdict) in &verdicts {
        if !matches!(verdict, InvariantVerdict::Holds) {
            println!("B33C551 end_of_812 {name} {}", format!("{verdict:?}").chars().take(160).collect::<String>());
        }
    }
    println!("B33C551 end_of_812 holds={} total={}", verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Holds)).count(), verdicts.len());
    let violated_invariants: Vec<&(&str, InvariantVerdict)> = verdicts"""
old2 = """        let violated_invariants: Vec<(&str, InvariantVerdict)> = check_pool_image(&image)
            .into_iter()"""
new2 = """        let all_verdicts = check_pool_image(&image);
        let not_holding: Vec<String> = all_verdicts.iter().filter(|(_, v)| !matches!(v, InvariantVerdict::Holds)).map(|(n, v)| format!("{n}:{}", match v { InvariantVerdict::NotApplicable(_) => "NA", InvariantVerdict::PartiallyJudged(_) => "PJ", InvariantVerdict::Violated(_) => "V", InvariantVerdict::Holds => "H" })).collect();
        println!("B33C551 per_version txg={} not_holding={not_holding:?}", output.root.checkpoint_txg.0);
        let violated_invariants: Vec<(&str, InvariantVerdict)> = all_verdicts
            .into_iter()"""
for a, b in ((old, new), (old2, new2)):
    if t.count(a) != 1:
        print("旧串没恰好命中一次", file=sys.stderr)
        sys.exit(3)
    t = t.replace(a, b)
p.write_text(t)
print(f"patched {p}")
