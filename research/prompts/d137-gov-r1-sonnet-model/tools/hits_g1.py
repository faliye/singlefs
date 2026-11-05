#!/usr/bin/env python3
"""G1 打中清单：每条命令给「钩子判什么 / 真 bash 里 cargo 垫片收到什么」；子 agent 与主 agent（不带前缀）各喂一遍。"""
import sys, os, warnings
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed, truth, sandbox_reset as R
HITS = [
 ("H1a", "bash -euo pipefail -c 'cargo test --all'"),
 ("H1b", "bash -eo pipefail -c 'cargo test --all'"),
 ("H1c", "bash -euo pipefail heavy.sh"),
 ("H1d", "bash -ouc pipefail 'cargo test --all'"),
 ("H1e", "env bash -euo pipefail -c 'cargo test --all'"),
 ("H1f", "bash -o pipefail -c 'cargo test --all'"),      # 对照：分开写的 -o 照拒
 ("H2a", "trap 'cargo test --all' EXIT"),
 ("H2b", "trap 'bash heavy.sh' EXIT"),
 ("H2c", "trap 'cargo test --all' DEBUG; true"),
 ("H2d", "eval 'cargo test --all'"),                      # 对照：eval 字面照拒
 ("H3a", "alias cargo='cargo test --all'\nshopt -s expand_aliases\ncargo"),
 ("H3b", "shopt -s expand_aliases\nalias ll='bash heavy.sh'\nll"),
 ("H4a", "source /dev/stdin <<< 'cargo test --all'"),
 ("H4b", "bash /dev/stdin < heavy.sh"),
 ("H4c", "exec < heavy.sh; bash"),
 ("H4d", "{ bash; } < heavy.sh"),
 ("H4e", "bash < heavy.sh"),                              # 对照：直接 < 文件照拒
 ("H5a", "cd missing_d137 2>/dev/null; bash heavy.sh"),
 ("H5b", "cd missing_d137 || true; bash heavy.sh"),
 ("H5c", "cd \"$(echo missing_d137)\" 2>/dev/null; bash heavy.sh"),
 ("H5d", "cd ' '; bash heavy.sh"),
 ("H6a", "bash ~+/heavy.sh"),
 ("H6b", "BASH_ENV=~+/heavy.sh bash -c :"),
 ("H7a", "echo 'cargo test --all' > m1.txt; cat src_ok.sh m1.txt > dd1.sh; bash dd1.sh"),
 ("H7b", "echo 'cargo test --all' > m9.txt; cat m9.txt >> dd9.sh; bash dd9.sh"),
 ("H7c", "echo 'cargo test --all' > m10.txt; cat m10.txt | tee dd10.sh > /dev/null; bash dd10.sh"),
 ("H7d", "echo 'cargo test --all' > m3.txt && cat m3.txt > dd3.sh && bash dd3.sh"),   # 对照：单源 > 照拒
 ("H8a", "cp /bin/bash ./mysh_d137; ./mysh_d137 heavy.sh"),
 ("H8b", "ln -sf /bin/bash ./mysh_d137b; ./mysh_d137b -c 'cargo test --all'"),
 ("H8c", "rbash -c 'cargo test --all'"),
 ("H8d", "busybox sh -c 'cargo test --all'"),
 ("H9a", "tclsh <<< 'exec cargo test --all'"),
 ("H9b", "awk 'BEGIN{system(\"cargo test --all\")}'"),
 ("H9c", "sed -n '1e cargo test --all' heavy.sh"),
 ("H9d", "git -c alias.x='!cargo test --all' x"),
 ("H9e", "perl -e 'system(\"cargo test --all\")'"),
 ("H9f", "python3 -c 'import os; os.system(\"cargo test --all\")'"),
 ("H10", "'{cargo,test,--all}'"),
]
def main():
    sel = sys.argv[1:]
    for hid, cmd in HITS:
        if sel and hid not in sel: continue
        out = []
        R.reset()
        res = {}
        for who, agent in (("子agent", "implementation-writer"), ("主agent", None)):
            R.reset()
            rc, err, notes = feed.feed("heavy-test-guard.sh", cmd, agent=agent)
            res[who] = (rc, len(notes))
        R.reset()
        trc, calls, tout = truth.truth(cmd)
        R.reset()
        ran = "真起" if any(c.startswith("CARGO test") for c in calls) else "没起"
        print("%s\t子agent rc=%s 检出%d\t主agent rc=%s 检出%d\t真bash %s\t%s" % (hid, res["子agent"][0], res["子agent"][1], res["主agent"][0], res["主agent"][1], ran, cmd.replace("\n", "\\n")))
if __name__ == "__main__":
    main()
