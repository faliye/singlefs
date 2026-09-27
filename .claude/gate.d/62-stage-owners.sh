#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 门禁阶段归属表与门禁目录、agent 定义一致
#
# 判据：`.claude/gate.d/stage-owners.tsv` 登记每个项目本地阶段先由哪个 agent 在干完自己的活之后跑
# （提交前的整轮门禁照旧跑全部阶段）。四条，任一条不成立判红：
#   ① `.claude/gate.d/` 顶层每个 `*.sh`（普通文件）在表里恰好一行——共享 gate.sh 当本地阶段跑的就是这一批
#      （`find .claude/gate.d -maxdepth 1 -name '*.sh' -type f`），名字不是两位数开头的（`100-x.sh`、`a-x.sh`）一样要登记；
#   ② 表里每一行的阶段文件都存在；
#   ③ 第二列每个 agent 名都有 `.claude/agents/<名字>.md`；
#   ④ 每一行三列齐全、第二列与第三列不为空；
#   ⑤ 每个阶段都认第一个参数当项目根：`ROOT="${1…`（或先解析选项再 `ROOT="${root_argument…`）、或直接 `cd "${1…`。
#      只看当前目录的阶段，被人从别的目录带着根目录参数调用时安静地判错地方，而 stage-selftest 既 cd 又传参，看不出来。
#
# 为什么：阶段归谁跑，原来在 CLAUDE.md 与 kb-scribe 定义里各手抄一份清单，没有任何东西盯着它们与目录同步——
# CLAUDE.md 那份 52 行清单里 87 号的说明已经与脚本头部对不上（records/2026-09-17-CLAUDE.md去冗余.md）。
# 表只写一份，各 agent 按自己的名字从表里取；新加一个阶段忘了登记、删了一个定义没改表，这一道当场红。
# ① ② 的双向比对与读表用共用库 lib-manifest.py，与 50、63、98 号同一份代码。
#
#   bash .claude/gate.d/62-stage-owners.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - "$(cd "$(dirname "$0")" && pwd)/lib-manifest.py" <<'PY'
import glob, importlib.util, os, re, sys
try:
    spec = importlib.util.spec_from_file_location("lib_manifest", sys.argv[1])
    manifest = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(manifest)
except OSError as error:
    print(f"  ✗ 读不到共用库 {sys.argv[1]}：{error}")
    print("     → 怎么办：目录与清单的双向比对只有那一份，恢复它，别在阶段里再抄一份。")
    sys.exit(1)
table_path = ".claude/gate.d/stage-owners.tsv"
if not os.path.isfile(table_path):
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表，三列用制表符分隔：阶段文件名、先跑它的 agent 名（逗号分隔）、为什么归它；# 开头的行是注释。")
    sys.exit(1)
ROOT_ARGUMENT_PATTERN = re.compile(r'\b(?:ROOT|root|REPOSITORY_ROOT|repository_root|PROJECT_ROOT)="?\$\{(?:1|root_argument)(?::-|\})|\bcd +"?\$\{?1\b')
# 与共享 gate.sh 认的本地阶段同一批：.claude/gate.d/ 顶层每个 *.sh 普通文件（它用 find -maxdepth 1 -name '*.sh' -type f）
stage_files = sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh") if os.path.isfile(path))
rows_by_stage = {}
malformed_rows = []
unknown_owner_rows = []
table = manifest.table_rows(table_path)
for line_number, line, fields in table:
    if len(fields) != 3 or not fields[1].strip() or not fields[2].strip():
        malformed_rows.append(f"第 {line_number} 行：{line}")
        continue
    stage, owners, _reason = fields
    rows_by_stage.setdefault(stage, []).append(line_number)
    for owner in owners.split(","):
        if not os.path.isfile(f".claude/agents/{owner.strip()}.md"):
            unknown_owner_rows.append(f"第 {line_number} 行 {stage}：{owner.strip()}")
missing_from_table, not_on_disk = manifest.two_way(stage_files, list(rows_by_stage))
duplicated = [f"{stage}（第 {', '.join(map(str, numbers))} 行）" for stage, numbers in rows_by_stage.items() if len(numbers) > 1]
stages_ignoring_the_root_argument = [stage for stage in stage_files if not ROOT_ARGUMENT_PATTERN.search(open(os.path.join('.claude/gate.d', stage), encoding='utf-8', errors='replace').read())]
failed = False
if malformed_rows:
    failed = True
    print("  ✗ 这些行不是三列、或 agent 名与理由有空的：")  # gate-lint:summary
    for entry in malformed_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行三列用制表符分隔：阶段文件名、agent 名（逗号分隔）、为什么归它；三列都要有内容。")
if missing_from_table:
    failed = True
    print("  ✗ 门禁目录里有这些阶段，表里没有登记——没有哪个 agent 会在自己的活之后先跑它：")  # gate-lint:summary
    for stage in missing_from_table:
        print(f"     {stage}")  # gate-lint:detail
    print(f"     → 怎么办：在 {table_path} 给它加一行，写明哪个 agent 干完活之后该先跑它、为什么；没有合适的 agent 就写 gate-triage。")
if duplicated:
    failed = True
    print("  ✗ 这些阶段在表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，几个 agent 名写在第二列、用逗号分隔。")
if not_on_disk:
    failed = True
    print("  ✗ 表里登记了这些阶段，门禁目录里没有这个文件——多半改了名或删了：")  # gate-lint:summary
    for stage in not_on_disk:
        print(f"     {stage}")  # gate-lint:detail
    print("     → 怎么办：改成现在的文件名，或删掉这一行。")
if unknown_owner_rows:
    failed = True
    print("  ✗ 这些 agent 名在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for entry in unknown_owner_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：改成已有定义的名字（文件名去掉 .md），或先建那个定义。")
if stages_ignoring_the_root_argument:
    failed = True
    print("  ✗ 这些阶段不认第一个参数当项目根（只看当前目录，从别的目录带着根目录参数调用时判的是错的地方）：")  # gate-lint:summary
    for stage in stages_ignoring_the_root_argument:
        print(f"     {stage}")  # gate-lint:detail
    print('     → 怎么办：在 preflight 那一行之后加 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}" 与 cd "$ROOT"（或者给每个路径加 "$ROOT/" 前缀），写法照 33 号。')
if failed:
    sys.exit(1)
owners = sorted({owner.strip() for _line_number, _line, fields in table for owner in fields[1].split(",")})
print(f"  ✓ 阶段归属表与门禁目录一致（{len(stage_files)} 个阶段，归 {len(owners)} 个 agent；{len(stage_files)} 个阶段都认第一个参数当项目根）")
PY
