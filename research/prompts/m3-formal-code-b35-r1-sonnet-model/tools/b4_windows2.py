#!/usr/bin/env python3
"""B4 取样点敏感性（同 b4_windows.py，另打第一条 panic 的位置与消息头，区分红在断言还是别处）：
把变异的窗口换成 [lo, hi]（目标那一版最左之外的叶容器片数，含两端），跑 lib 里那条模型单测。
用法 b4_windows2.py <副本根> <lo> <hi>…"""
import hashlib, os, re, subprocess, sys
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
    lines = t.split("\n")
    where = next((l.strip() for l in lines if "panicked at" in l), "")
    head = next((" ".join(x.strip() for x in lines[k + 1:k + 3])[:200] for k, l in enumerate(lines) if "panicked at" in l), "")
    print(f"窗口 {lo}..={hi}: exit={p.returncode} {'红（抓到）' if p.returncode == 101 else ('绿（存活）' if p.returncode == 0 else '其他')} | {where[:110]} | {head}", flush=True)
assert hashlib.sha256(open(path, "rb").read()).hexdigest() == digest
