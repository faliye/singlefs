#!/usr/bin/env python3
"""从冻结副本的 code-tooling.sh 里抽出 heavy-thread-defaults 格的 python 判法，在该冻结副本根上直接跑（不经门禁入口，base_state=none）。
用法：run_htd_cell.py <冻结副本根> [BREAK开关]
只读：python 不写字节码；抽出的两段临时文件放在 $TMPDIR（默认草稿目录）。
输出：格自己的原样输出（头部几行）与退出码。"""
import os, re, subprocess, sys, tempfile
root = os.path.abspath(sys.argv[1]); broken = sys.argv[2] if len(sys.argv) > 2 else ""
src = open(os.path.join(root, ".claude/gate.d/code-tooling.sh"), encoding="utf-8").read().split("\n")
def heredoc(start_pat, end_pat, first_after):
    out, on = [], False
    for i, l in enumerate(src):
        if not on and re.search(start_pat, l):
            on = True; continue
        if on:
            if re.fullmatch(end_pat, l): return "\n".join(out)
            out.append(l)
    raise SystemExit("heredoc not found: " + start_pat)
lib = heredoc(r"CODE_WITHOUT_TEXT_LIBRARY=\"\$\(cat <<'PY_CODE_WITHOUT_TEXT'", r"PY_CODE_WITHOUT_TEXT", True)
# 格的 python：紧跟在 cell_heavy_thread_defaults() 里的 python3 - ... <<'PY'
idx = next(i for i, l in enumerate(src) if l.startswith("cell_heavy_thread_defaults()"))
sub = src[idx:]
j = next(i for i, l in enumerate(sub) if l.startswith('python3 - "$CODE_WITHOUT_TEXT_LIBRARY"') and "<<'PY'" in l)
body = []
for l in sub[j + 1:]:
    if l == "PY": break
    body.append(l)
code = "\n".join(body)
tmp = tempfile.mkdtemp(prefix="htd-", dir=os.environ.get("TMPDIR") or None)
open(os.path.join(tmp, "cell.py"), "w").write(code)
base_table = os.path.join(tmp, "base_table"); open(base_table, "w").close()
lib_file = os.path.join(root, "scripts/hosts/heavy_threads.py")
r = subprocess.run([sys.executable, os.path.join(tmp, "cell.py"), lib, lib_file, broken, "none", "", base_table],
                   cwd=root, capture_output=True, text=True)
for l in (r.stdout + r.stderr).splitlines():
    print(l[:400])
print("name=htd_cell rc=%d" % r.returncode)
