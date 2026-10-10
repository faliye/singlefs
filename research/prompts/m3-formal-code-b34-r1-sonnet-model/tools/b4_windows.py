#!/usr/bin/env python3
"""B4 取样点敏感性：把 34-3 那一行变异的窗口 59..=117 换成别的窗口，跑 lib 里那条模型单测，记红 / 绿（绿 = 变异存活）。
用法 b4_windows.py <副本根> <lo> <hi>...（每个窗口一对 lo hi，含两端）"""
import hashlib, os, subprocess, sys
root = sys.argv[1]; wins = [(int(sys.argv[i]), int(sys.argv[i+1])) for i in range(2, len(sys.argv), 2)]
path = f"{root}/crates/singlefs-harness/src/model.rs"
orig = open(path, encoding="utf-8").read(); digest = hashlib.sha256(orig.encode()).hexdigest()
find = ("                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })")
assert orig.count(find) == 1
runner = os.environ["RUNNER"]
for lo, hi in wins:
    mutated = find + f"\n                        .take(if ({lo}..={hi}).contains(&target_root.role_written_at.keys().filter(|role| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))).count()) {{ 3 }} else {{ usize::MAX }})"
    open(path, "w", encoding="utf-8").write(orig.replace(find, mutated))
    try:
        p = subprocess.run(f"{runner} -p singlefs-harness --lib -- a_rollback_takes_exactly_the_inode_leaf_containers", shell=True, cwd=root, capture_output=True, text=True, timeout=230)
        t = p.stdout + p.stderr
    finally:
        open(path, "w", encoding="utf-8").write(orig)
    res = [l for l in t.split("\n") if l.startswith("test result")]
    print(f"窗口 {lo}..={hi}: exit={p.returncode} {'红（抓到）' if p.returncode == 101 else ('绿（存活）' if p.returncode == 0 else '其他')} {res}", flush=True)
assert hashlib.sha256(open(path, "rb").read()).hexdigest() == digest
