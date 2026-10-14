#!/usr/bin/env python3
"""Z5 喂法：从 <门禁根>/.claude/gate.d/doc-text.sh 抽出 private_value_sources、scan_form、cell_date_once、cell_term_renames、cell_private_values 几个函数
与 PRIVATE_VALUES_LIBRARY（那一版有就带上），在 <样本 git 仓> 里原样跑（不起 doc-text.sh 本身、不跑 --check）。
样本根不是门禁根，private_value_sources 走样本那一支：只认样本根下的 multi-host.env 与 .private-values-hostname。
用法：z5_cells.py <门禁根> <样本 git 仓> <格名>…（clock-times、prime-marks、session-names、date-once、term-renames、private-values）"""
import os, re, subprocess, sys
gate_root, sample = map(os.path.abspath, sys.argv[1:3])
cells = sys.argv[3:]
lines = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")

def block(start_test, end_test):
    start = next(i for i, l in enumerate(lines) if start_test(l))
    end = next(i for i in range(start + 1, len(lines)) if end_test(lines[i]))
    return "\n".join(lines[start:end + 1])

parts = []
if any(l.startswith('PRIVATE_VALUES_LIBRARY="$(cat <<') for l in lines):
    parts.append(block(lambda l: l.startswith('PRIVATE_VALUES_LIBRARY="$(cat <<'), lambda l: l == ')"'))
for name in ("private_value_sources", "scan_form", "cell_date_once", "cell_term_renames", "cell_private_values"):
    if any(l == f"{name}() {{" for l in lines):
        parts.append(block(lambda l, n=name: l == f"{n}() {{", lambda l: l == "}"))
stage = os.path.join(gate_root, ".claude/gate.d")
prelude = f"""set -u
STAGE_DIR={stage!r}
REPO_ROOT={gate_root!r}
LIBRARY="$STAGE_DIR/lib-forbidden-notations.py"
SWEEP_TERM="$REPO_ROOT/scripts/kb/sweep-term.py"
DOC_CONSOLIDATE="$REPO_ROOT/scripts/kb/doc-consolidate.py"
source "$REPO_ROOT/scripts/gate/changed-paths.sh"
text_notations_break="${{GATE_TEXT_NOTATIONS_BREAK:-}}"
ROOT={sample!r}
cd "$ROOT"
"""
harness = prelude + "\n".join(parts) + "\n"
for cell in cells:
    function = "cell_" + cell.replace("-", "_")
    body = f"( {function} )" if f"{function}() {{" in harness else f"( scan_form {cell} )"
    result = subprocess.run(["bash", "-c", harness + body + "\n"], capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    print(f"=== {cell} 退出={result.returncode}")
    sys.stdout.write(result.stdout)
    sys.stdout.write(result.stderr)
