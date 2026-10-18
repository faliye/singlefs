#!/usr/bin/env bash
# H4 的量：对本仓此刻的 HEAD（只读，临时克隆里量，不动本仓的 ref、暂存区、工作区），
# 用冻结副本的 gate_added_lines 量四种基准下 quoted-result-lines 格会判的「整行抄的产物行」有多少、其中多少在 research/results/*.out 里逐字找不到。
# 用法：FROZEN=… WORK=… REPO=<仓根> bash measure-h4.sh
# 「找不到」是粗估的上界：这里拿全部 research/results/*.out 里 E7RESULT 开头的行当对照；格自己只对照 replay.sh 登记着的产物加归档，会更严。
set -uo pipefail
export LC_ALL=C
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${FROZEN:?}" "${WORK:?}" "${REPO:?}"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_OPTIONAL_LOCKS=0
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GATE_BASE
source "$FROZEN/scripts/gate/changed-paths.sh"
M="$WORK/measure-clone"; rm -rf "$M"
git clone -q --shared --no-checkout "$REPO" "$M" || exit 2
cd "$M" || exit 2
head_commit="${MEASURE_COMMIT:-$(git -C "$REPO" rev-parse HEAD)}"   # 要钉在某个提交上量就设 MEASURE_COMMIT
git fetch -q "$REPO" refs/sop/staged-green:refs/sop/staged-green || exit 2
git checkout -q --detach "$head_commit"
# 只检出格会读的 md（.claude/kb 与 research 下的 md）
git ls-files -z -- .claude/kb ':(glob)research/**/*.md' | xargs -0 git checkout -q "$head_commit" --
echo "HEAD=$(git rev-parse --short HEAD) HEAD~1=$(git rev-parse --short HEAD~1) staged-green=$(git rev-parse --short refs/sop/staged-green)（提交日期 $(git log -1 --format=%cs refs/sop/staged-green)，树 $(git rev-parse --short refs/sop/staged-green^{tree})，HEAD 的树 $(git rev-parse --short HEAD^{tree})）"
echo "staged-green 到 HEAD 的提交数（不是祖先，数的是 HEAD 的全部历史）：$(git rev-list --count refs/sop/staged-green..HEAD)；merge-base：$(git merge-base refs/sop/staged-green HEAD 2>&1 | head -1)"
LC_ALL=C grep -h '^[[:space:]]*E7RESULT ' "$REPO"/research/results/*.out 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | LC_ALL=C sort -u > "$WORK/out-lines.txt"
echo ".out 里 E7RESULT 行（去重）：$(wc -l < "$WORK/out-lines.txt")"
in_cell_scope() {   # 与 doc-experiments.sh 1630-1633 同一个路径筛
  awk -F'\t' '{ p=$1; if (p ~ /-history\.md$/ || p ~ /\/decisions-history\// || p ~ /^research\/prompts\// || p ~ /^research\/results\//) next; print }'
}
count_base() {   # count_base <名字> <基准>
  local name="$1" base="$2" mode="${3:-}" all inscope n_all n_scope n_missing
  local t0 t1; t0=$(date +%s)
  all="$(gate_added_lines "$base" .claude/kb ':(glob)research/**/*.md')" || { echo "$name：gate_added_lines 失败"; return; }
  t1=$(date +%s); printf "  （gate_added_lines %s 用时 %s 秒）\n" "$name" "$((t1-t0))"
  n_all="$(printf '%s\n' "$all" | grep -c . )"
  inscope="$(printf '%s\n' "$all" | in_cell_scope | awk -F'\t' '{ t=$2; sub(/^[ \t]+/,"",t); sub(/[ \t]+$/,"",t); if (t ~ /^E7RESULT /) print t }')"
  n_scope="$(printf '%s\n' "$inscope" | grep -c . )"
  n_missing="$(printf '%s\n' "$inscope" | grep -c . >/dev/null; printf '%s\n' "$inscope" | LC_ALL=C sort -u | LC_ALL=C comm -23 - "$WORK/out-lines.txt" | grep -c .)"
  printf '%s\n' "$all" > "$WORK/added-lines.tmp"
  printf '%-28s 新增行总数=%s  格射程内的 E7RESULT 新增行=%s  其中（去重后）.out 里逐字找不到=%s\n' "$name" "$n_all" "$n_scope" "$n_missing"
  echo "  照格的口径（登记着的产物 + 归档版）粗估："; python3 "$HERE/estimate-red.py" "$WORK/added-lines.tmp" "$REPO" $mode | sed "s/^/    /"
}
count_base "N 今天（HEAD）" HEAD
count_base "S 今天（HEAD~1）" HEAD~1
count_base "丙（staged-green）" refs/sop/staged-green
count_base "戊（空树，全量）" "$(git hash-object -t tree /dev/null)" full
cd / && rm -rf "$M"
