#!/usr/bin/env python3
"""Q5：豁免表里被钉的脚本挪了位置（内容不变，sha256 不变），表里的路径跟着改，exempt-check 带基准判。
用法：q5_pathmove.py <名>=<冻结副本根> ... --work <新目录>"""
import argparse, os, shutil, subprocess
parser = argparse.ArgumentParser(); parser.add_argument("roots", nargs="+"); parser.add_argument("--work", required=True)
arguments = parser.parse_args()
for entry in arguments.roots:
    label, root = entry.split("=", 1)
    table = open(root + "/.claude/long-run-live-hosts-exempt", encoding="utf-8").read().split("\n")
    rows = [l for l in table if l and not l.startswith("#")]
    old = rows[0].split()[0]; new = old.replace("e171_three_oracle_collusion_after_the_oracle_fixes/", "e171_three_oracle_collusion_after_the_oracle_fixes/moved/")
    d = os.path.join(arguments.work, label); os.makedirs(d + "/.claude", exist_ok=True)
    open(d + "/base-table", "w", encoding="utf-8").write("\n".join(table))
    open(d + "/.claude/long-run-live-hosts-exempt", "w", encoding="utf-8").write("\n".join(l.replace(old, new) if l.startswith(old) else l for l in table))
    for l in rows:
        p = l.split()[0]; q = new if p == old else p
        os.makedirs(os.path.dirname(d + "/" + q), exist_ok=True); shutil.copy(root + "/" + p, d + "/" + q)
    done = subprocess.run(["python3", "-B", root + "/.claude/hooks/lib_long_run.py", "exempt-check", d, "--base-table", d + "/base-table"], capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    detail = [x.strip() for x in done.stderr.split("\n") if "涨" in x]
    print(f"{label}\t表 {len(rows)} 行\t退出={done.returncode}\t{(done.stdout + done.stderr).strip().splitlines()[0][:60]}\t{detail[0][:150] if detail else '-'}")
