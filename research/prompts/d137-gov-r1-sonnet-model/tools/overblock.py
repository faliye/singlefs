#!/usr/bin/env python3
"""G1：偏严的代价。子 agent 平常会写的、本身不起重活的命令里，哪些被「认不出」拒了。每行：期望 / 实际 / 命令。"""
import os, sys, warnings
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed
CASES = [
 ("$HOME/.cargo/bin/foo --version", 0), ('"$HOME/.cargo/bin/foo" --version', 0), ("$EDITOR CLAUDE.md", 2), ('PY=python3; $PY -c "print(1)"', 0), ('"$PY" x.py', 2),
 ("${CARGO:-cargo} fmt --all -- --check", 2), ('$(which python3) -c "print(1)"', 2), ('exec "$@"', 2), ('bash "$SCRIPT"', 2),
 ("for f in $(ls scripts/hosts/*.sh); do bash -n $f; done", 0), ('for f in scripts/hosts/capped.sh; do bash -n "$f"; done', 0), ('S=scripts/hosts/capped.sh; bash -n "$S"', 0),
 ("tool=git; $tool status", 0), ('x=$(command -v git); $x status', 2), ('python3 -c "import sys" && echo ok', 0),
 ('while read -r f; do bash -n "$f"; done < <(ls scripts/hosts/*.sh)', 0), ('find scripts/hosts -name "*.sh" -exec bash -n {} \;', 0),
 ("ls scripts/hosts/*.sh | xargs -n1 bash -n", 0), ("ls | xargs -I{} echo {}", 0), ('echo hi | xargs -I{} sh -c "echo {}"', 0),
]
ok = 0
for cmd, want in CASES:
    rc, err, notes = feed.feed("heavy-test-guard.sh", cmd, agent="implementation-writer", cwd=feed.TREE)
    ok += (rc == want)
    print("%s\t%s\t%s\t%s" % ("overblock", want, rc, cmd))
print("合计\t%d\t符合\t%d" % (len(CASES), ok))
