#!/usr/bin/env python3
"""把 <门禁根>/.claude/gate.d/doc-text.sh 里 scan_form 的 python（prime-marks / clock-times / session-names 三格共用）抽出来，在 <样本 git 仓> 里原样跑。
用法：run_scan_form.py <门禁根> <样本 git 仓> <形态键>   （库取门禁根的 .claude/gate.d/lib-forbidden-notations.py）"""
import os, subprocess, sys
gate_root, sample, form = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2]), sys.argv[3]
lines = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('  python3 - "$LIBRARY" "$1" <<\'PY\''))
stop = next(i for i in range(start + 1, len(lines)) if lines[i] == "PY")
code = "\n".join(lines[start + 1:stop])
library = os.path.join(gate_root, ".claude/gate.d/lib-forbidden-notations.py")
result = subprocess.run([sys.executable, "-", library, form], input=code, text=True, cwd=sample, capture_output=True)
sys.stdout.write(result.stdout); sys.stderr.write(result.stderr)
sys.exit(result.returncode)
