#!/usr/bin/env python3
"""从 <门禁根>/.claude/gate.d/code-tooling.sh 抽出 heavy-thread-defaults 格的 python 与 CODE_WITHOUT_TEXT_LIBRARY，在 <被判树> 里原样跑。
基准状态默认 none（只缩不涨那一条不比）；HTD_BASE_TABLE=<文件> 时按 table 比。HTD_ANCHOR_DUMP=1 时在判完之后多打每一行排除表锚串命中几条被判行（只加打印，不改判法）。
用法：z2_htd_cell.py <门禁根> <被判树> [弄坏开关]。只读门禁根，cwd 换到被判树；不起 gate.sh，不跑 --check。"""
import os, subprocess, sys
gate_root, tree = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
broken = sys.argv[3] if len(sys.argv) > 3 else ""
lines = open(os.path.join(gate_root, ".claude/gate.d/code-tooling.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith("CODE_WITHOUT_TEXT_LIBRARY=\"$(cat <<'PY_CODE_WITHOUT_TEXT'"))
end = next(i for i in range(start + 1, len(lines)) if lines[i] == "PY_CODE_WITHOUT_TEXT")
library = "\n".join(lines[start + 1:end])
cell = next(i for i, l in enumerate(lines) if l.startswith('python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$heavy_threads_library_file"'))
cell_end = next(i for i in range(cell + 1, len(lines)) if lines[i] == "PY")
code = "\n".join(lines[cell + 1:cell_end])
if os.environ.get("HTD_ANCHOR_DUMP"):
    marker = "grown, growth_note = [], \"\"\n"
    assert code.count(marker) == 1
    dump = ("for _n, _p, _a in exempt_rows:\n"
            "    _h = [num for num, _names, _line in hits.get(_p, []) if _a in _line]\n"
            "    print(f'ANCHOR\\t{_n}\\t{_p}\\t{_a}\\t命中被判行 {len(_h)} 条\\t{_h[:8]}')\n")
    code = code.replace(marker, dump + marker, 1)
heavy = os.path.join(tree, "scripts/hosts/heavy_threads.py")
base_table = os.environ.get("HTD_BASE_TABLE", "")
base_args = ["table", "base-sample", base_table] if base_table else ["none", "", ""]
result = subprocess.run([sys.executable, "-", library, heavy, broken, *base_args], input=code, text=True, cwd=tree,
                        capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
sys.stdout.write(result.stdout); sys.stderr.write(result.stderr)
print(f"退出={result.returncode}")
sys.exit(result.returncode)
