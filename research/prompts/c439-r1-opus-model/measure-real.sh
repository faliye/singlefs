#!/usr/bin/env bash
# H4 的量：在本仓的一份 --shared 克隆（检出在给定提交、工作区干净）上，拿冻结副本的 gate_added_lines 对几种基准各取一次窗口，
# 照 quoted-result-lines 格的射程过滤（kb 与 research 下的 .md，去掉 *-history.md、decisions-history/、research/prompts/、research/results/，
# 行去掉首尾空白后以「E7RESULT 」开头），数窗口里的行数，再用简化判法数「产物里逐字找不到」的行数。
# 简化判法拿 research/results/*.out 全部产物比（格里新增的行只比 replay.sh 登记的产物），所以这里数出的找不到的行数是格判红行数的下界。
# 用法：bash measure-real.sh <冻结副本> <克隆目录> <名字=基准>…（基准写 empty 取空树）
set -uo pipefail
FROZEN="$1"; CLONE="$2"; shift 2
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
unset GATE_BASE GATE_DIFF_BASE
# shellcheck disable=SC1091
source "$FROZEN/scripts/gate/changed-paths.sh"
cd "$CLONE" || exit 2
echo "name=real_head head=$(git rev-parse HEAD) dirty=$(git status --porcelain | wc -l)"
for spec in "$@"; do
  label="${spec%%=*}"; base="${spec#*=}"
  [[ "$base" == empty ]] && base="$(git hash-object -t tree /dev/null)"
  gate_added_lines "$base" .claude/kb ':(glob)research/**/*.md' > "$CLONE.added.$label" || { echo "name=real_window label=$label error=gate_added_lines_failed"; continue; }
  python3 - "$CLONE.added.$label" "$label" "$base" <<'PY'
import glob, sys
path, label, base = sys.argv[1], sys.argv[2], sys.argv[3]
products = set()
for out in glob.glob('research/results/**/*.out', recursive=True):
    with open(out, encoding='utf-8', errors='replace') as handle:
        products.update(line.rstrip('\n') for line in handle)
lines = []
for record in open(path, encoding='utf-8', errors='replace').read().split('\n'):
    file, sep, text = record.partition('\t')
    if not sep:
        continue
    if not (file.startswith('.claude/kb/') or file.startswith('research/')) or not file.endswith('.md'):
        continue
    if file.endswith('-history.md') or '/decisions-history/' in file or file.startswith('research/prompts/') or file.startswith('research/results/'):
        continue
    if text.strip().startswith('E7RESULT '):
        lines.append((file, text.strip()))
missing = [entry for entry in lines if entry[1] not in products]
files = len({file for file, _ in lines})
print(f'name=real_window label={label} base={base[:12]} added_lines_total={sum(1 for _ in open(path, encoding="utf-8", errors="replace"))} quoted_in_scope={len(lines)} files={files} not_found_in_any_out={len(missing)}')
PY
done
